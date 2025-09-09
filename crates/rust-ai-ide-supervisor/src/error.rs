//! Supervisor-specific error types and utilities

use std::fmt;
use thiserror::Error;
use rusqlite::Error as RusqliteError;
use tokio::process::Command as TokioCommand;

/// Supervisor error types extending the core IDEError
#[derive(Error, Debug, Clone)]
pub enum SupervisorError {
    #[error("Process monitoring error: {message}")]
    ProcessError { message: String },

    #[error("Service restart failed: {service_name} - {attempts} attempts")]
    RestartFailed { service_name: String, attempts: u32 },

    #[error("State persistence error: {message}")]
    PersistenceError { message: String },

    #[error("Database migration error: {message}")]
    MigrationError { message: String },

    #[error("IPC recovery failure: {channel_id} - {reason}")]
    IpcRecoveryError { channel_id: String, reason: String },

    #[error("Health check timeout: {service_name} after {timeout}s")]
    HealthCheckTimeout { service_name: String, timeout: u64 },

    #[error("Configuration validation failed: {field} - {reason}")]
    ValidationError { field: String, reason: String },

    #[error("Security validation failed: {operation} - {reason}")]
    SecurityError { operation: String, reason: String },

    #[error("Resource limit exceeded: {resource} - current: {current}, limit: {limit}")]
    ResourceLimitExceeded { resource: String, current: u64, limit: u64 },

    #[error("Initialization failure: {component} - {reason}")]
    InitError { component: String, reason: String },

    #[error("Checkpoint save failed: {id} - {reason}")]
    CheckpointError { id: String, reason: String },
}

impl SupervisorError {
    /// Create a process monitoring error
    pub fn process_error<S: Into<String>>(message: S) -> Self {
        Self::ProcessError {
            message: message.into(),
        }
    }

    /// Create a restart failure error
    pub fn restart_failed(service_name: &str, attempts: u32) -> Self {
        Self::RestartFailed {
            service_name: service_name.to_string(),
            attempts,
        }
    }

    /// Create a persistence error
    pub fn persistence_error<S: Into<String>>(message: S) -> Self {
        Self::PersistenceError {
            message: message.into(),
        }
    }

    /// Create a migration error
    pub fn migration_error<S: Into<String>>(message: S) -> Self {
        Self::MigrationError {
            message: message.into(),
        }
    }

    /// Create an IPC recovery error
    pub fn ipc_recovery_error(channel_id: &str, reason: &str) -> Self {
        Self::IpcRecoveryError {
            channel_id: channel_id.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a health check timeout error
    pub fn health_check_timeout(service_name: &str, timeout: u64) -> Self {
        Self::HealthCheckTimeout {
            service_name: service_name.to_string(),
            timeout,
        }
    }

    /// Create a validation error
    pub fn validation_error<S: Into<String>>(field: &str, reason: S) -> Self {
        Self::ValidationError {
            field: field.to_string(),
            reason: reason.into(),
        }
    }

    /// Create a security error
    pub fn security_error<S: Into<String>>(operation: &str, reason: S) -> Self {
        Self::SecurityError {
            operation: operation.to_string(),
            reason: reason.into(),
        }
    }

    /// Create a resource limit error
    pub fn resource_limit_exceeded(resource: &str, current: u64, limit: u64) -> Self {
        Self::ResourceLimitExceeded {
            resource: resource.to_string(),
            current,
            limit,
        }
    }

    /// Create an initialization error
    pub fn init_error<S: Into<String>>(component: &str, reason: S) -> Self {
        Self::InitError {
            component: component.to_string(),
            reason: reason.into(),
        }
    }

    /// Create a checkpoint error
    pub fn checkpoint_error<S: Into<String>>(id: &str, reason: S) -> Self {
        Self::CheckpointError {
            id: id.to_string(),
            reason: reason.into(),
        }
    }
}

/// Convert Rusqlite errors to SupervisorError
impl From<RusqliteError> for SupervisorError {
    fn from(err: RusqliteError) -> Self {
        Self::PersistenceError {
            message: format!("SQLite error: {:?}", err),
        }
    }
}

/// Specialized Result type for supervisor operations
pub type SupervisorResult<T> = Result<T, SupervisorError>;

/// Aggregate errors from multiple operations
pub struct ErrorAggregator {
    errors: Vec<SupervisorError>,
}

impl ErrorAggregator {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: SupervisorError) {
        self.errors.push(error);
    }

    pub fn add_result<T>(&mut self, result: SupervisorResult<T>) {
        if let Err(e) = result {
            self.errors.push(e);
        }
    }

    pub fn into_result<T>(self, success_result: T) -> SupervisorResult<T> {
        if self.errors.is_empty() {
            Ok(success_result)
        } else {
            // Return the first error, but log others
            let first_error = self.errors.remove(0);
            for error in &self.errors {
                log::warn!("Additional error: {:?}", error);
            }
            Err(first_error)
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

/// Helper functions for error formatting and aggregation
pub struct SupervisorErrorUtils;

impl SupervisorErrorUtils {
    /// Format multiple errors into a single string
    pub fn format_multiple_errors(errors: &[SupervisorError], operation: &str) -> String {
        if errors.is_empty() {
            format!("{} succeeded without errors", operation)
        } else {
            format!(
                "{} failed with {} errors:\n{}",
                operation,
                errors.len(),
                errors
                    .iter()
                    .enumerate()
                    .map(|(i, err)| format!("{}. {}", i + 1, err))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }

    /// Check if an error represents a critical failure that requires system shutdown
    pub fn is_critical_error(error: &SupervisorError) -> bool {
        matches!(
            error,
            SupervisorError::InitError { .. } | SupervisorError::SecurityError { .. }
        )
    }

    /// Check if an error is retriable (transient vs permanent)
    pub fn is_retriable_error(error: &SupervisorError) -> bool {
        match error {
            SupervisorError::HealthCheckTimeout { .. } => true,
            SupervisorError::ProcessError { message } => {
                // Check if it's a temporary process error
                message.contains("timeout") || message.contains("connection") || message.contains("network")
            }
            SupervisorError::IpcRecoveryError { .. } => true,
            _ => false,
        }
    }

    /// Create a detailed error context
    pub fn with_context(error: SupervisorError, context: &str) -> SupervisorError {
        match error {
            SupervisorError::ProcessError { message } => {
                SupervisorError::ProcessError {
                    message: format!("{}: {}", context, message),
                }
            }
            SupervisorError::PersistenceError { message } => {
                SupervisorError::PersistenceError {
                    message: format!("{}: {}", context, message),
                }
            }
            SupervisorError::IpcRecoveryError { channel_id, reason } => {
                SupervisorError::IpcRecoveryError {
                    channel_id,
                    reason: format!("{}: {}", context, reason),
                }
            }
            _ => error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_error_creation() {
        let error = SupervisorError::process_error("Test error");
        assert!(matches!(error, SupervisorError::ProcessError { .. }));

        let restart_error = SupervisorError::restart_failed("test_service", 3);
        assert!(matches!(restart_error, SupervisorError::RestartFailed { .. }));
    }

    #[test]
    fn test_error_display() {
        let error = SupervisorError::health_check_timeout("ai_lsp", 30);
        let error_str = format!("{}", error);
        assert!(error_str.contains("ai_lsp"));
        assert!(error_str.contains("30"));
    }

    #[test]
    fn test_error_aggregator() {
        let mut aggregator = ErrorAggregator::new();
        assert!(!aggregator.has_errors());

        aggregator.add_error(SupervisorError::process_error("test1"));
        aggregator.add_error(SupervisorError::process_error("test2"));

        assert_eq!(aggregator.error_count(), 2);
        assert!(aggregator.has_errors());

        let result: SupervisorResult<i32> = aggregator.into_result(42);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_utils() {
        let critical_error = SupervisorError::init_error("test", "failed");
        assert!(SupervisorErrorUtils::is_critical_error(&critical_error));

        let non_critical = SupervisorError::process_error("test");
        assert!(!SupervisorErrorUtils::is_critical_error(&non_critical));

        let retriable = SupervisorError::health_check_timeout("test", 30);
        assert!(SupervisorErrorUtils::is_retriable_error(&retriable));

        let non_retriable = SupervisorError::validation_error("field", "invalid");
        assert!(!SupervisorErrorUtils::is_retriable_error(&non_retriable));
    }
}