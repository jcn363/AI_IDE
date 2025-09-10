use rust_ai_ide_errors::IDEError;
use serde::{Deserialize, Serialize};

/// Orchestration-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationError {
    /// Service discovery or registration failed
    ServiceRegistrationError(String),
    /// Service communication failed
    CommunicationError(String),
    /// Health check failed
    HealthCheckFailed(String),
    /// Lifecycle operation failed
    LifecycleError(String),
    /// Message routing failed
    MessageRoutingError(String),
    /// Configuration error
    ConfigurationError(String),
    /// Timeout error
    TimeoutError(String),
    /// Resource exhaustion
    ResourceExhaustion(String),
    /// Unknown service
    UnknownService(String),
    /// Version compatibility issue
    VersionIncompatible(String),
    /// Validation failed
    ValidationError(String),
}

impl std::fmt::Display for OrchestrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestrationError::ServiceRegistrationError(msg) =>
                write!(f, "Service registration error: {}", msg),
            OrchestrationError::CommunicationError(msg) =>
                write!(f, "Communication error: {}", msg),
            OrchestrationError::HealthCheckFailed(msg) =>
                write!(f, "Health check failed: {}", msg),
            OrchestrationError::LifecycleError(msg) =>
                write!(f, "Lifecycle error: {}", msg),
            OrchestrationError::MessageRoutingError(msg) =>
                write!(f, "Message routing error: {}", msg),
            OrchestrationError::ConfigurationError(msg) =>
                write!(f, "Configuration error: {}", msg),
            OrchestrationError::TimeoutError(msg) =>
                write!(f, "Timeout error: {}", msg),
            OrchestrationError::ResourceExhaustion(msg) =>
                write!(f, "Resource exhaustion: {}", msg),
            OrchestrationError::UnknownService(msg) =>
                write!(f, "Unknown service: {}", msg),
            OrchestrationError::VersionIncompatible(msg) =>
                write!(f, "Version incompatible: {}", msg),
            OrchestrationError::ValidationError(msg) =>
                write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for OrchestrationError {}

impl From<OrchestrationError> for IDEError {
    fn from(error: OrchestrationError) -> Self {
        IDEError::OrchestrationError(error)
    }
}

pub type OrchestrationResult<T> = Result<T, OrchestrationError>;

/// Helper for aggregating errors at function boundaries
pub fn aggregate_errors(results: Vec<OrchestrationResult<()>>) -> OrchestrationResult<()> {
    let errors: Vec<_> = results.into_iter()
        .filter_map(|r| r.err())
        .map(|e| e.to_string())
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(OrchestrationError::ValidationError(
            format!("Multiple errors occurred: {}", errors.join("; "))
        ))
    }
}