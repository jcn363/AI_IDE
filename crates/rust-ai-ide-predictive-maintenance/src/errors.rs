//! Error types for the predictive maintenance system

use std::fmt;

use rust_ai_ide_errors::IDEError;

/// Result type for maintenance operations
pub type MaintenanceResult<T> = Result<T, MaintenanceError>;

/// Errors that can occur during maintenance operations
#[derive(Debug)]
pub enum MaintenanceError {
    /// Errors from underlying IDE components
    IDEError(IDEError),

    /// Forecasting calculation errors
    ForecastingError(String),

    /// Cost estimation errors
    CostEstimationError(String),

    /// Impact analysis errors
    ImpactAnalysisError(String),

    /// Priority calculation errors
    PriorityCalculationError(String),

    /// Recommendation generation errors
    RecommendationError(String),

    /// Database access errors
    DatabaseError(String),

    /// Configuration errors
    ConfigurationError(String),

    /// Validation errors
    ValidationError(String),
}

impl fmt::Display for MaintenanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaintenanceError::IDEError(err) => write!(f, "IDE error: {}", err),
            MaintenanceError::ForecastingError(msg) => write!(f, "Forecasting error: {}", msg),
            MaintenanceError::CostEstimationError(msg) => {
                write!(f, "Cost estimation error: {}", msg)
            }
            MaintenanceError::ImpactAnalysisError(msg) => {
                write!(f, "Impact analysis error: {}", msg)
            }
            MaintenanceError::PriorityCalculationError(msg) => {
                write!(f, "Priority calculation error: {}", msg)
            }
            MaintenanceError::RecommendationError(msg) => {
                write!(f, "Recommendation error: {}", msg)
            }
            MaintenanceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            MaintenanceError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            MaintenanceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for MaintenanceError {}

impl From<IDEError> for MaintenanceError {
    fn from(err: IDEError) -> Self {
        MaintenanceError::IDEError(err)
    }
}

impl From<rusqlite::Error> for MaintenanceError {
    fn from(err: rusqlite::Error) -> Self {
        MaintenanceError::DatabaseError(format!("SQLite error: {}", err))
    }
}

impl From<serde_json::Error> for MaintenanceError {
    fn from(err: serde_json::Error) -> Self {
        MaintenanceError::ValidationError(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for MaintenanceError {
    fn from(err: std::io::Error) -> Self {
        MaintenanceError::ValidationError(format!("IO error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let forecast_error = MaintenanceError::ForecastingError("Test error".to_string());
        assert_eq!(
            format!("{}", forecast_error),
            "Forecasting error: Test error"
        );
    }

    #[test]
    fn test_error_from_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let maintenance_error: MaintenanceError = io_error.into();
        match maintenance_error {
            MaintenanceError::ValidationError(msg) => {
                assert!(msg.contains("IO error"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
