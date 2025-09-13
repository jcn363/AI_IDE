//! Common traits used across the Rust AI IDE project

use std::fmt;

use chrono;

// Plugin types needed for trait definitions
use crate::types::{PluginCapability, PluginContext, PluginMessage, PluginMessageType, PluginMetadata};

/// Trait for types that can provide configuration

/// Trait for types that can provide configuration
#[async_trait::async_trait]
pub trait Configurable: Send + Sync {
    type Config;

    /// Configure the instance with the provided configuration
    async fn configure(&mut self, config: Self::Config) -> crate::errors::IdeResult<()>;
}

/// Trait for types that can be validated
pub trait Validatable {
    /// Validate the current state of the instance
    fn validate(&self) -> crate::errors::IdeResult<()>;
}

/// Trait for types that can be refreshed or updated
#[async_trait::async_trait]
pub trait Refreshable: Send + Sync {
    /// Refresh the instance state from external sources
    async fn refresh(&mut self) -> crate::errors::IdeResult<()>;
}

/// Trait for types that can serialize themselves to a string format
pub trait Serializable {
    /// Serialize to JSON string
    fn to_json(&self) -> crate::errors::IdeResult<String>
    where
        Self: serde::Serialize,
    {
        serde_json::to_string(self).map_err(|e| crate::errors::IdeError::Serialization {
            message: e.to_string(),
        })
    }

    /// Deserialize from JSON string
    fn from_json(json: &str) -> crate::errors::IdeResult<Self>
    where
        Self: serde::de::DeserializeOwned,
    {
        serde_json::from_str(json).map_err(|e| crate::errors::IdeError::Serialization {
            message: e.to_string(),
        })
    }
}

/// Trait for code generation capabilities
#[async_trait::async_trait]
pub trait CodeGenerator: Send + Sync {
    /// Generate code based on context and configuration
    async fn generate_code(
        &self,
        context: &crate::types::ProjectContext,
        config: serde_json::Value,
    ) -> crate::errors::IdeResult<String>;

    /// Validate generated code
    async fn validate_code(&self, code: &str) -> crate::errors::IdeResult<()>;
}

use std::collections::HashMap;
use std::sync::Arc;

use crate::logging::LogContext;
use crate::unified_logger::UnifiedLogger;

/// Trait for logging and telemetry
#[async_trait::async_trait]
pub trait Loggable: std::fmt::Debug + Send + Sync {
    /// Get the underlying UnifiedLogger instance
    fn logger(&self) -> &UnifiedLogger;

    /// Log a message at the specified level
    async fn log(&self, level: crate::logging::LogLevel, message: &str) -> Result<(), crate::logging::LoggingError> {
        self.logger().log(level, message).await
    }

    /// Log an informational message
    async fn log_info(&self, message: &str) {
        let _ = self.log(crate::logging::LogLevel::Info, message).await;
    }

    /// Log a warning message
    async fn log_warn(&self, message: &str) {
        let _ = self.log(crate::logging::LogLevel::Warn, message).await;
    }

    /// Log an error message
    async fn log_error(&self, message: &str, error: Option<&(dyn std::fmt::Debug + Send + Sync)>) {
        if let Err(e) = self.logger().error(message, error).await {
            eprintln!("Failed to log error: {:?}", e);
        }
    }

    /// Log a debug message
    async fn log_debug(&self, message: &str) {
        let _ = self.log(crate::logging::LogLevel::Debug, message).await;
    }

    /// Log a trace message
    async fn log_trace(&self, message: &str) {
        let _ = self.log(crate::logging::LogLevel::Trace, message).await;
    }

    /// Create a child logger with additional context
    fn with_context(&self, context: HashMap<String, String>) -> Box<dyn Loggable> {
        let log_context = LogContext {
            operation: "child".to_string(),
            metadata: context.into_iter().map(|(k, v)| (k, v.into())).collect(),
            ..Default::default()
        };

        Box::new(LoggableWrapper {
            logger: Arc::new(self.logger().child(log_context)),
        })
    }
}

/// Simple wrapper to implement Loggable for UnifiedLogger
#[derive(Clone)]
pub struct LoggableWrapper {
    pub logger: Arc<UnifiedLogger>,
}

impl fmt::Debug for LoggableWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LoggableWrapper")
    }
}

#[async_trait::async_trait]
impl Loggable for LoggableWrapper {
    fn logger(&self) -> &UnifiedLogger {
        &self.logger
    }
}

impl From<UnifiedLogger> for Box<dyn Loggable> {
    fn from(logger: UnifiedLogger) -> Self {
        Box::new(LoggableWrapper {
            logger: Arc::new(logger),
        })
    }
}

/// Extension trait for easier logging of Results
#[async_trait::async_trait]
pub trait LoggableResultExt<T, E>: Send + Sync
where
    T: Send + Sync,
    E: Send + Sync,
{
    /// Log the error if the result is an Err
    async fn log_if_err(self, logger: &dyn Loggable, context: &str) -> Result<T, E>;
}

#[async_trait::async_trait]
impl<T, E: std::fmt::Debug + Send + Sync + 'static> LoggableResultExt<T, E> for Result<T, E>
where
    T: Send + Sync,
{
    async fn log_if_err(self, logger: &dyn Loggable, context: &str) -> Result<T, E> {
        match self {
            Ok(val) => Ok(val),
            Err(e) => {
                logger
                    .log_error(&format!("{}: {:?}", context, &e), Some(&e))
                    .await;
                Err(e)
            }
        }
    }
}

/// Trait for plugins in the Rust AI IDE system
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin with the provided context
    async fn initialize(&mut self, context: &mut PluginContext) -> crate::errors::IdeResult<()>;

    /// Shutdown the plugin
    async fn shutdown(&mut self) -> crate::errors::IdeResult<()>;

    /// Check if the plugin is compatible with the current IDE version
    fn is_compatible(&self, ide_version: &semver::Version) -> bool;

    /// Get plugin capabilities
    fn capabilities(&self) -> Vec<PluginCapability>;

    /// Handle incoming messages from other plugins or the IDE
    async fn handle_message(
        &mut self,
        message: &PluginMessage,
        _context: &mut PluginContext,
    ) -> crate::errors::IdeResult<PluginMessage> {
        // Default implementation does nothing and returns empty response
        Ok(PluginMessage {
            id:           message.id.clone(),
            message_type: PluginMessageType::Response,
            from_plugin:  self.metadata().id.clone(),
            to_plugin:    Some(message.from_plugin.clone()),
            payload:      serde_json::Value::Null,
            timestamp:    chrono::Utc::now(),
        })
    }
}

/// Trait for plugin components that can be started/stopped
#[async_trait::async_trait]
pub trait Pluggable: Send + Sync {
    /// Start the component
    async fn start(&mut self, context: &PluginContext) -> crate::errors::IdeResult<()>;

    /// Stop the component
    async fn stop(&mut self) -> crate::errors::IdeResult<()>;

    /// Check if the component is running
    fn is_running(&self) -> bool;
}
