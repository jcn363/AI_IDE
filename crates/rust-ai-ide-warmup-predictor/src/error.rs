//! Error types for the warmup prediction system

use std::fmt;
use thiserror::Error;

/// Result type alias for warmup operations
pub type Result<T> = std::result::Result<T, WarmupError>;

/// Comprehensive error types for the warmup prediction system
#[derive(Debug, Error)]
pub enum WarmupError {
    /// Configuration-related errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Prediction engine errors
    #[error("Prediction engine error: {message}")]
    PredictionEngine { message: String },

    /// Resource management errors
    #[error("Resource management error: {message}")]
    ResourceManager { message: String },

    /// Scheduling errors
    #[error("Scheduling error: {message}")]
    Scheduler { message: String },

    /// Queue management errors
    #[error("Queue error: {message}")]
    Queue { message: String },

    /// Performance prediction errors
    #[error("Performance prediction error: {message}")]
    PerformancePredictor { message: String },

    /// Metrics collection errors
    #[error("Metrics error: {message}")]
    Metrics { message: String },

    /// Usage pattern analysis errors
    #[error("Usage pattern analysis error: {message}")]
    UsagePattern { message: String },

    /// Model loading/unloading errors
    #[error("Model operation error: {message}")]
    ModelOperation { message: String },

    /// Security validation errors
    #[error("Security validation error: {message}")]
    SecurityValidation { message: String },

    /// Resource exhaustion errors
    #[error("Resource exhausted: {resource_type}")]
    ResourceExhausted { resource_type: String },

    /// Timeout errors
    #[error("Operation timeout: {operation}")]
    Timeout { operation: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// External service communication errors
    #[error("External service error: {service}")]
    ExternalService { service: String },

    /// Validation errors
    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    /// Concurrency errors
    #[error("Concurrency error: {message}")]
    Concurrency { message: String },

    /// Memory management errors
    #[error("Memory error: {message}")]
    Memory { message: String },

    /// IO errors
    #[error("IO error: {message}")]
    Io { message: String },

    /// Unknown errors
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Low severity - operation can continue
    Low,
    /// Medium severity - operation may be degraded
    Medium,
    /// High severity - operation significantly impacted
    High,
    /// Critical severity - system may be unstable
    Critical,
}

impl WarmupError {
    /// Get the severity level of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            WarmupError::Configuration { .. } => ErrorSeverity::High,
            WarmupError::PredictionEngine { .. } => ErrorSeverity::Medium,
            WarmupError::ResourceManager { .. } => ErrorSeverity::High,
            WarmupError::Scheduler { .. } => ErrorSeverity::Medium,
            WarmupError::Queue { .. } => ErrorSeverity::Low,
            WarmupError::PerformancePredictor { .. } => ErrorSeverity::Low,
            WarmupError::Metrics { .. } => ErrorSeverity::Low,
            WarmupError::UsagePattern { .. } => ErrorSeverity::Medium,
            WarmupError::ModelOperation { .. } => ErrorSeverity::High,
            WarmupError::SecurityValidation { .. } => ErrorSeverity::Critical,
            WarmupError::ResourceExhausted { .. } => ErrorSeverity::High,
            WarmupError::Timeout { .. } => ErrorSeverity::Medium,
            WarmupError::Serialization { .. } => ErrorSeverity::Medium,
            WarmupError::ExternalService { .. } => ErrorSeverity::High,
            WarmupError::Validation { .. } => ErrorSeverity::Medium,
            WarmupError::Concurrency { .. } => ErrorSeverity::High,
            WarmupError::Memory { .. } => ErrorSeverity::Critical,
            WarmupError::Io { .. } => ErrorSeverity::High,
            WarmupError::Unknown { .. } => ErrorSeverity::Medium,
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self.severity() {
            ErrorSeverity::Low | ErrorSeverity::Medium => true,
            ErrorSeverity::High | ErrorSeverity::Critical => false,
        }
    }

    /// Get error category for grouping and monitoring
    pub fn category(&self) -> &'static str {
        match self {
            WarmupError::Configuration { .. } => "configuration",
            WarmupError::PredictionEngine { .. } => "prediction",
            WarmupError::ResourceManager { .. } => "resource_management",
            WarmupError::Scheduler { .. } => "scheduling",
            WarmupError::Queue { .. } => "queue_management",
            WarmupError::PerformancePredictor { .. } => "performance",
            WarmupError::Metrics { .. } => "metrics",
            WarmupError::UsagePattern { .. } => "usage_analysis",
            WarmupError::ModelOperation { .. } => "model_operations",
            WarmupError::SecurityValidation { .. } => "security",
            WarmupError::ResourceExhausted { .. } => "resource_limits",
            WarmupError::Timeout { .. } => "timeouts",
            WarmupError::Serialization { .. } => "serialization",
            WarmupError::ExternalService { .. } => "external_services",
            WarmupError::Validation { .. } => "validation",
            WarmupError::Concurrency { .. } => "concurrency",
            WarmupError::Memory { .. } => "memory",
            WarmupError::Io { .. } => "io",
            WarmupError::Unknown { .. } => "unknown",
        }
    }
}

impl From<std::io::Error> for WarmupError {
    fn from(err: std::io::Error) -> Self {
        WarmupError::Io {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for WarmupError {
    fn from(err: serde_json::Error) -> Self {
        WarmupError::Serialization {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        let config_error = WarmupError::Configuration {
            message: "test".to_string(),
        };
        assert_eq!(config_error.severity(), ErrorSeverity::High);

        let validation_error = WarmupError::Validation {
            field: "test".to_string(),
            message: "test".to_string(),
        };
        assert_eq!(validation_error.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_categories() {
        let config_error = WarmupError::Configuration {
            message: "test".to_string(),
        };
        assert_eq!(config_error.category(), "configuration");

        let security_error = WarmupError::SecurityValidation {
            message: "test".to_string(),
        };
        assert_eq!(security_error.category(), "security");
    }

    #[test]
    fn test_recoverability() {
        let low_severity = WarmupError::Queue {
            message: "test".to_string(),
        };
        assert!(low_severity.is_recoverable());

        let critical_severity = WarmupError::Memory {
            message: "test".to_string(),
        };
        assert!(!critical_severity.is_recoverable());
    }
}