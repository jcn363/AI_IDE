//! Error types for workspace optimization operations
//!
//! This module defines comprehensive error types that follow the project's
//! error handling patterns and can be extended to the IDEError enum if needed.

use std::fmt;

use thiserror::Error;

/// Main error type for workspace optimizer operations
#[derive(Debug, Error)]
pub enum OptimizerError {
    /// Initialization failed
    #[error("Failed to initialize workspace optimizer: {message}")]
    InitializationFailed { message: String },

    /// Dependency analysis failed
    #[error("Dependency analysis failed: {message}")]
    DependencyAnalysisFailed { message: String },

    /// Build optimization failed
    #[error("Build optimization failed: {message}")]
    BuildOptimizationFailed { message: String },

    /// Health monitoring failed
    #[error("Health monitoring failed: {message}")]
    HealthMonitoringFailed { message: String },

    /// Consolidation failed
    #[error("Consolidation failed: {message}")]
    ConsolidationFailed { message: String },

    /// File I/O error
    #[error("File I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    /// Cargo metadata error
    #[error("Cargo metadata error: {message}")]
    CargoMetadataError { message: String },

    /// Circular dependency detected
    #[error("Circular dependency detected: {crates:?}")]
    CircularDependencyDetected { crates: Vec<String> },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Memory limit exceeded
    #[error("Memory limit exceeded: used {used_mb:.2}MB, limit {limit_mb:.2}MB")]
    MemoryLimitExceeded { used_mb: f64, limit_mb: f64 },

    /// Build timeout
    #[error("Build timeout: exceeded {timeout:?}")]
    BuildTimeout { timeout: std::time::Duration },

    /// Security violation
    #[error("Security violation: {violation}")]
    SecurityViolation { violation: String },

    /// Network error
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Concurrent access error
    #[error("Concurrent access error: {message}")]
    ConcurrentAccessError { message: String },

    /// Invalid state
    #[error("Invalid state: {message}")]
    InvalidState { message: String },

    /// Resource exhausted
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    /// Timeout error
    #[error("Operation timeout: {operation}")]
    Timeout { operation: String },

    /// Validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Generic error with context
    #[error("Optimizer error: {context}")]
    Generic { context: String },
}

impl OptimizerError {
    /// Create a new initialization failed error
    pub fn initialization_failed<S: Into<String>>(message: S) -> Self {
        Self::InitializationFailed {
            message: message.into(),
        }
    }

    /// Create a new dependency analysis failed error
    pub fn dependency_analysis_failed<S: Into<String>>(message: S) -> Self {
        Self::DependencyAnalysisFailed {
            message: message.into(),
        }
    }

    /// Create a new build optimization failed error
    pub fn build_optimization_failed<S: Into<String>>(message: S) -> Self {
        Self::BuildOptimizationFailed {
            message: message.into(),
        }
    }

    /// Create a new health monitoring failed error
    pub fn health_monitoring_failed<S: Into<String>>(message: S) -> Self {
        Self::HealthMonitoringFailed {
            message: message.into(),
        }
    }

    /// Create a new consolidation failed error
    pub fn consolidation_failed<S: Into<String>>(message: S) -> Self {
        Self::ConsolidationFailed {
            message: message.into(),
        }
    }

    /// Create a new cargo metadata error
    pub fn cargo_metadata_error<S: Into<String>>(message: S) -> Self {
        Self::CargoMetadataError {
            message: message.into(),
        }
    }

    /// Create a new circular dependency error
    pub fn circular_dependency_detected(crates: Vec<String>) -> Self {
        Self::CircularDependencyDetected { crates }
    }

    /// Create a new configuration error
    pub fn configuration_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create a new memory limit exceeded error
    pub fn memory_limit_exceeded(used_mb: f64, limit_mb: f64) -> Self {
        Self::MemoryLimitExceeded { used_mb, limit_mb }
    }

    /// Create a new build timeout error
    pub fn build_timeout(timeout: std::time::Duration) -> Self {
        Self::BuildTimeout { timeout }
    }

    /// Create a new security violation error
    pub fn security_violation<S: Into<String>>(violation: S) -> Self {
        Self::SecurityViolation {
            violation: violation.into(),
        }
    }

    /// Create a new network error
    pub fn network_error<S: Into<String>>(message: S) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    /// Create a new concurrent access error
    pub fn concurrent_access_error<S: Into<String>>(message: S) -> Self {
        Self::ConcurrentAccessError {
            message: message.into(),
        }
    }

    /// Create a new invalid state error
    pub fn invalid_state<S: Into<String>>(message: S) -> Self {
        Self::InvalidState {
            message: message.into(),
        }
    }

    /// Create a new resource exhausted error
    pub fn resource_exhausted<S: Into<String>>(resource: S) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a new validation error
    pub fn validation_error<F: Into<String>, M: Into<String>>(field: F, message: M) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a new generic error
    pub fn generic<S: Into<String>>(context: S) -> Self {
        Self::Generic {
            context: context.into(),
        }
    }

    /// Get the error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::InitializationFailed { .. } => ErrorSeverity::Critical,
            Self::MemoryLimitExceeded { .. } => ErrorSeverity::Critical,
            Self::SecurityViolation { .. } => ErrorSeverity::Critical,
            Self::BuildTimeout { .. } => ErrorSeverity::High,
            Self::CircularDependencyDetected { .. } => ErrorSeverity::High,
            Self::ResourceExhausted { .. } => ErrorSeverity::High,
            Self::Timeout { .. } => ErrorSeverity::Medium,
            Self::ConcurrentAccessError { .. } => ErrorSeverity::Medium,
            Self::NetworkError { .. } => ErrorSeverity::Medium,
            Self::InvalidState { .. } => ErrorSeverity::Medium,
            Self::ValidationError { .. } => ErrorSeverity::Low,
            Self::ConfigurationError { .. } => ErrorSeverity::Low,
            Self::IoError { .. } => ErrorSeverity::Medium,
            Self::SerializationError { .. } => ErrorSeverity::Medium,
            _ => ErrorSeverity::Medium,
        }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::IoError { .. } => true,
            Self::NetworkError { .. } => true,
            Self::Timeout { .. } => true,
            Self::ConcurrentAccessError { .. } => true,
            Self::MemoryLimitExceeded { .. } => false,
            Self::SecurityViolation { .. } => false,
            Self::InitializationFailed { .. } => false,
            _ => true,
        }
    }

    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::IoError { .. } => ErrorCategory::System,
            Self::NetworkError { .. } => ErrorCategory::Network,
            Self::Timeout { .. } => ErrorCategory::Timeout,
            Self::MemoryLimitExceeded { .. } => ErrorCategory::Resource,
            Self::SecurityViolation { .. } => ErrorCategory::Security,
            Self::ConcurrentAccessError { .. } => ErrorCategory::Concurrency,
            Self::ValidationError { .. } => ErrorCategory::Validation,
            Self::ConfigurationError { .. } => ErrorCategory::Configuration,
            Self::CargoMetadataError { .. } => ErrorCategory::Cargo,
            Self::CircularDependencyDetected { .. } => ErrorCategory::Dependency,
            _ => ErrorCategory::General,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Low severity - minor issue
    Low,
    /// Medium severity - notable issue
    Medium,
    /// High severity - significant issue
    High,
    /// Critical severity - requires immediate attention
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// System-level errors (I/O, filesystem)
    System,
    /// Network-related errors
    Network,
    /// Timeout errors
    Timeout,
    /// Resource exhaustion errors
    Resource,
    /// Security-related errors
    Security,
    /// Concurrency-related errors
    Concurrency,
    /// Validation errors
    Validation,
    /// Configuration errors
    Configuration,
    /// Cargo-related errors
    Cargo,
    /// Dependency-related errors
    Dependency,
    /// General errors
    General,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => write!(f, "System"),
            Self::Network => write!(f, "Network"),
            Self::Timeout => write!(f, "Timeout"),
            Self::Resource => write!(f, "Resource"),
            Self::Security => write!(f, "Security"),
            Self::Concurrency => write!(f, "Concurrency"),
            Self::Validation => write!(f, "Validation"),
            Self::Configuration => write!(f, "Configuration"),
            Self::Cargo => write!(f, "Cargo"),
            Self::Dependency => write!(f, "Dependency"),
            Self::General => write!(f, "General"),
        }
    }
}

/// Convert standard library errors to OptimizerError
impl From<std::io::Error> for OptimizerError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError { source: err }
    }
}

/// Convert JSON serialization errors to OptimizerError
impl From<serde_json::Error> for OptimizerError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError { source: err }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = OptimizerError::initialization_failed("test");
        assert!(matches!(err, OptimizerError::InitializationFailed { .. }));
    }

    #[test]
    fn test_error_severity() {
        let critical_err = OptimizerError::initialization_failed("test");
        assert_eq!(critical_err.severity(), ErrorSeverity::Critical);

        let low_err = OptimizerError::validation_error("field", "message");
        assert_eq!(low_err.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_category() {
        let io_err = OptimizerError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        };
        assert_eq!(io_err.category(), ErrorCategory::System);

        let validation_err = OptimizerError::validation_error("field", "message");
        assert_eq!(validation_err.category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_error_display() {
        let err = OptimizerError::initialization_failed("test message");
        let display = format!("{}", err);
        assert!(display.contains("Failed to initialize workspace optimizer"));
        assert!(display.contains("test message"));
    }
}
