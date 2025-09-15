//! Default logger implementation for the Rust AI IDE
//!
//! This module provides a default implementation of the `Loggable` trait
//! that can be used throughout the application.

use std::sync::Arc;

use crate::logging::LoggingManager;
use crate::traits::{Loggable, LoggableWrapper};
use crate::unified_logger::UnifiedLogger;

/// Default logger implementation that wraps a `UnifiedLogger`
#[derive(Debug, Clone)]
pub struct DefaultLogger {
    logger: Arc<UnifiedLogger>,
}

impl Default for DefaultLogger {
    fn default() -> Self {
        let _manager = LoggingManager::new();
        let logger = UnifiedLogger::new("default").with_context(crate::logging::LogContext::new("init", "default"));

        Self {
            logger: Arc::new(logger),
        }
    }
}

impl Loggable for DefaultLogger {
    fn logger(&self) -> &UnifiedLogger {
        &self.logger
    }
}

impl From<UnifiedLogger> for DefaultLogger {
    fn from(logger: UnifiedLogger) -> Self {
        Self {
            logger: Arc::new(logger),
        }
    }
}

/// Initialize the default logger with the specified configuration
pub fn init_default_logger() -> Box<dyn Loggable> {
    let logger = DefaultLogger::default();
    Box::new(LoggableWrapper {
        logger: logger.logger,
    })
}

/// Get a reference to the global logger
pub fn global_logger() -> Box<dyn Loggable> {
    // In a real implementation, this would use a global static or lazy_static
    init_default_logger()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::LogLevel;

    #[tokio::test]
    async fn test_default_logger() {
        let logger = DefaultLogger::default();
        logger.log_info("This is an info message").await;
        logger.log_warn("This is a warning").await;
        logger.log_error("This is an error", None).await;

        // Test with error context
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        logger
            .log_error("IO operation failed", Some(&io_error))
            .await;
    }

    #[tokio::test]
    async fn test_global_logger() {
        let logger = global_logger();
        logger.log_info("Global logger test").await;
    }
}
