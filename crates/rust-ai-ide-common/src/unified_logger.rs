//! Unified Logger Implementation
//!
//! This module provides a unified logging interface that implements the `UnifiedLogger` trait
//! and integrates with the rest of the application's logging infrastructure.

use std::sync::Arc;

use async_trait::async_trait;

use crate::logging::{
    LogContext, LogLevel, LoggingError, LoggingManager, UnifiedLogger as UnifiedLoggerTrait,
};

/// Main implementation of the UnifiedLogger trait
#[derive(Clone, Debug)]
pub struct UnifiedLogger {
    inner: Arc<LoggingManager>,
    context: LogContext,
}

impl UnifiedLogger {
    /// Create a new instance of the unified logger
    pub fn new(component: &str) -> Self {
        Self {
            inner: Arc::new(LoggingManager::new()),
            context: LogContext {
                component: component.to_string(),
                ..Default::default()
            },
        }
    }

    /// Create a child logger with additional context
    pub fn with_context(mut self, context: LogContext) -> Self {
        self.context = context;
        self
    }

    /// Log a message with the specified level and context
    pub async fn log(&self, level: LogLevel, message: &str) -> Result<(), LoggingError> {
        self.inner
            .log(level, message, Some(&self.context.clone()))
            .await
    }

    /// Log an error with context
    pub async fn error(
        &self,
        message: &str,
        error: Option<&(dyn std::fmt::Debug + Send + Sync)>,
    ) -> Result<(), LoggingError> {
        let mut context = self.context.clone();
        if let Some(err) = error {
            context
                .metadata
                .insert("error".to_string(), format!("{:?}", err).into());
        }
        self.inner
            .log(LogLevel::Error, message, Some(&context))
            .await
    }

    /// Log a warning with context
    pub async fn warn(&self, message: &str) -> Result<(), LoggingError> {
        self.inner
            .log(LogLevel::Warn, message, Some(&self.context.clone()))
            .await
    }

    /// Log an info message with context
    pub async fn info(&self, message: &str) -> Result<(), LoggingError> {
        self.inner
            .log(LogLevel::Info, message, Some(&self.context.clone()))
            .await
    }

    /// Log a debug message with context
    pub async fn debug(&self, message: &str) -> Result<(), LoggingError> {
        self.inner
            .log(LogLevel::Debug, message, Some(&self.context.clone()))
            .await
    }

    /// Log a trace message with context
    pub async fn trace(&self, message: &str) -> Result<(), LoggingError> {
        self.inner
            .log(LogLevel::Trace, message, Some(&self.context.clone()))
            .await
    }
    /// Create a child logger with additional context
    pub fn child(&self, additional_context: LogContext) -> UnifiedLogger {
        let mut context = self.context.clone();
        context.metadata.extend(additional_context.metadata);

        // Merge context fields, with additional_context taking precedence
        if !additional_context.operation.is_empty() {
            context.operation = additional_context.operation;
        }
        if additional_context.user_id.is_some() {
            context.user_id = additional_context.user_id;
        }
        if additional_context.session_id.is_some() {
            context.session_id = additional_context.session_id;
        }
        if additional_context.request_id.is_some() {
            context.request_id = additional_context.request_id;
        }
        if !additional_context.component.is_empty() {
            context.component = additional_context.component;
        }

        UnifiedLogger {
            inner: self.inner.clone(),
            context,
        }
    }
}

#[async_trait]
impl UnifiedLoggerTrait for UnifiedLogger {
    async fn log<'a>(
        &self,
        level: LogLevel,
        message: &str,
        context: Option<&'a LogContext>,
    ) -> Result<(), LoggingError> {
        match context {
            Some(ctx) => self.inner.log(level, message, Some(&ctx.clone())).await,
            None => {
                self.inner
                    .log(level, message, Some(&self.context.clone()))
                    .await
            }
        }
    }

    fn child(&self, additional_context: LogContext) -> crate::logging::LoggingGuard {
        let mut context = self.context.clone();
        context.metadata.extend(additional_context.metadata);

        // Merge context fields, with additional_context taking precedence
        if !additional_context.operation.is_empty() {
            context.operation = additional_context.operation;
        }
        if additional_context.user_id.is_some() {
            context.user_id = additional_context.user_id;
        }
        if additional_context.session_id.is_some() {
            context.session_id = additional_context.session_id;
        }
        if additional_context.request_id.is_some() {
            context.request_id = additional_context.request_id;
        }
        if !additional_context.component.is_empty() {
            context.component = additional_context.component;
        }

        self.inner.child(context)
    }
}

/// Extension trait for easy conversion of errors to logging context
pub trait LoggableError: std::error::Error {
    fn to_log_context(&self) -> LogContext {
        LogContext {
            operation: "error".to_string(),
            user_id: None,
            session_id: None,
            request_id: None,
            component: "error".to_string(),
            metadata: [
                ("error_type".to_string(), self.to_string().into()),
                ("source".to_string(), format!("{:?}", self.source()).into()),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }
}

// Implement for all types that implement std::error::Error
impl<E: std::error::Error> LoggableError for E {}

/// Extension trait for Result to easily log errors
pub trait LoggableResult<T, E> {
    /// Log the error if the result is an Err
    async fn log_error(self, logger: &UnifiedLogger, context: &str) -> Result<T, E>;
}

impl<T, E: std::fmt::Debug + Send + Sync> LoggableResult<T, E> for Result<T, E> {
    async fn log_error(self, logger: &UnifiedLogger, context: &str) -> Result<T, E> {
        match self {
            Ok(val) => Ok(val),
            Err(e) => {
                logger
                    .error(&format!("{}: {:?}", context, &e), Some(&e))
                    .await
                    .ok();
                Err(e)
            }
        }
    }
}

/// Macro for easy logging with context
#[macro_export]
macro_rules! log_with_ctx {
    ($logger:expr, $level:expr, $msg:expr) => {
        $logger.log($level, $msg, None).await
    };
    ($logger:expr, $level:expr, $msg:expr, $ctx:expr) => {
        $logger.log($level, $msg, Some($ctx)).await
    };
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn test_unified_logger() {
        let logger = UnifiedLogger::new("test_component");

        // Test basic logging
        logger.info("This is an info message").await.unwrap();
        logger.warn("This is a warning").await.unwrap();
        logger.error("This is an error", None).await.unwrap();

        // Test with error context
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        logger
            .error("IO operation failed", Some(&io_error))
            .await
            .unwrap();

        // Test child logger
        let child_ctx = LogContext {
            operation: "test_operation".to_string(),
            ..Default::default()
        };
        let child_logger = logger.child(child_ctx);
        child_logger.info("Child logger message").await.unwrap();
    }

    #[tokio::test]
    async fn test_loggable_result() {
        let logger = UnifiedLogger::new("test_loggable_result");

        // Test successful result
        let success: Result<u32, String> = Ok(42);
        let result = success.log_error(&logger, "This shouldn't log").await;
        assert_eq!(result, Ok(42));

        // Test error result
        let error: Result<u32, String> = Err("test error".to_string());
        let result = error.log_error(&logger, "This should log").await;
        assert!(result.is_err());
    }
}
