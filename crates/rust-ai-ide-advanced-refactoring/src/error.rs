use std::fmt;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum RefactoringError {
    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Execution error: {message}")]
    Execution { message: String },

    #[error("Analysis error: {message}")]
    Analysis { message: String },

    #[error("Safety violation: {message}")]
    SafetyViolation { message: String },

    #[error("Dependency resolution failed: {message}")]
    DependencyResolution { message: String },

    #[error("Circular dependency detected: {message}")]
    CircularDependency { message: String },

    #[error("Functional equivalence check failed: {message}")]
    FunctionalEquivalence { message: String },

    #[error("Rollback failed: {message}")]
    Rollback { message: String },

    #[error("Timeout error: {message}")]
    Timeout { message: String },

    #[error("Resource exhaustion: {message}")]
    ResourceExhaustion { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("AI service error: {message}")]
    AIService { message: String },

    #[error("LSP integration error: {message}")]
    LSPIntegration { message: String },

    #[error("File access error: {path} - {message}")]
    FileAccess { path: String, message: String },

    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Semantic error: {message}")]
    SemanticError { message: String },

    #[error("Performance degradation detected: {metric} exceeded threshold {threshold}")]
    PerformanceDegradation {
        metric: String,
        threshold: f64,
    },

    #[error("Security violation: {violation_type}")]
    SecurityViolation { violation_type: String },

    #[error("Audit trail corruption: {message}")]
    AuditTrailCorruption { message: String },
}

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Syntactic validation failed: {details}")]
    Syntactic { details: String },

    #[error("Type validation failed: {details}")]
    TypeValidation { details: String },

    #[error("Functional equivalence validation failed: {confidence_level}")]
    FunctionalEquivalence { confidence_level: f64 },

    #[error("Behavior preservation validation failed: {risk_level}")]
    BehaviorPreservation { risk_level: String },

    #[error("Dependency validation failed: {dependency_chain}")]
    DependencyValidation { dependency_chain: Vec<String> },

    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: Vec<String> },

    #[error("Performance impact validation failed: {metric} = {value}")]
    PerformanceImpact { metric: String, value: f64 },

    #[error("Memory safety validation failed: {issue}")]
    MemorySafety { issue: String },
}

#[derive(Error, Debug, Clone)]
pub enum ExecutionError {
    #[error("Transformation execution failed: {transformation_id}")]
    TransformationFailed { transformation_id: String },

    #[error("Rolling back transformation: {transformation_id}")]
    RollbackRequired { transformation_id: String },

    #[error("Concurrent modification conflict: {resource}")]
    ConcurrentModification { resource: String },

    #[error("Resource limit exceeded: {resource_type}")]
    ResourceLimitExceeded { resource_type: String },

    #[error("Execution timeout after {duration_seconds} seconds")]
    ExecutionTimeout { duration_seconds: u64 },

    #[error("Dependency execution failed: {dependency}")]
    DependencyExecution { dependency: String },

    #[error("Execution interrupted by user")]
    UserInterrupt,

    #[error("Execution paused due to safety concerns")]
    SafetyPause,
}

#[derive(Error, Debug, Clone)]
pub enum AnalysisError {
    #[error("Pattern recognition failed: {details}")]
    PatternRecognition { details: String },

    #[error("Context analysis failed: {details}")]
    ContextAnalysis { details: String },

    #[error("Impact assessment failed: {details}")]
    ImpactAssessment { details: String },

    #[error("Cost-benefit analysis failed: {details}")]
    CostBenefitAnalysis { details: String },

    #[error("Risk assessment failed: {details}")]
    RiskAssessment { details: String },

    #[error("Model inference failed: {model_name}")]
    ModelInference { model_name: String },

    #[error("Data processing failed: {stage}")]
    DataProcessing { stage: String },

    #[error("Statistical analysis failed: {method}")]
    StatisticalAnalysis { method: String },

    #[error("Graph analysis failed: {algorithm}")]
    GraphAnalysis { algorithm: String },
}

pub type RefactoringResult<T> = Result<T, RefactoringError>;
pub type ValidationResult<T> = Result<T, ValidationError>;
pub type ExecutionResult<T> = Result<T, ExecutionError>;
pub type AnalysisResult<T> = Result<T, AnalysisError>;

/// Utility functions for error handling
pub fn aggregate_errors<T, E>(results: Vec<Result<T, E>>) -> Result<Vec<T>, Vec<E>>
where
    E: std::fmt::Debug,
{
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for result in results {
        match result {
            Ok(value) => successes.push(value),
            Err(error) => failures.push(error),
        }
    }

    if failures.is_empty() {
        Ok(successes)
    } else {
        Err(failures)
    }
}

/// Create a recoverable error that doesn't stop the entire process
pub fn recoverable_error<E: Into<RefactoringError>>(error: E) -> RefactoringResult<()> {
    Err(error.into()).map_err(|e| {
        tracing::warn!("Recoverable error occurred: {}", e);
        e
    })
}

/// Check if an error is recoverable
pub fn is_recoverable_error(error: &RefactoringError) -> bool {
    match error {
        RefactoringError::Validation { .. } => true,
        RefactoringError::DependencyResolution { .. } => true,
        RefactoringError::Timeout { .. } => true,
        RefactoringError::ResourceExhaustion { .. } => true,
        _ => false,
    }
}

/// Attempt to recover from an error with a recovery strategy
pub fn attempt_recovery<T, F>(
    result: RefactoringResult<T>,
    recovery_strategy: F,
) -> RefactoringResult<T>
where
    F: FnOnce(&RefactoringError) -> Option<RefactoringResult<T>>,
{
    result.or_else(|error| {
        if let Some(recovery_result) = recovery_strategy(&error) {
            recovery_result
        } else {
            Err(error)
        }
    })
}

/// Log errors without failing the operation
pub fn log_error_silently<T>(result: &RefactoringResult<T>, context: &str) {
    if let Err(error) = result {
        tracing::warn!("Silent error in {}: {}", context, error);
    }
}

/// Convert error types between domains
pub trait ErrorConverter<T> {
    fn convert_to_refactoring_error(self) -> RefactoringResult<T>;
    fn convert_to_validation_error(self) -> ValidationResult<T>;
    fn convert_to_execution_error(self) -> ExecutionResult<T>;
    fn convert_to_analysis_error(self) -> AnalysisResult<T>;
}

impl<T, E> ErrorConverter<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn convert_to_refactoring_error(self) -> RefactoringResult<T> {
        self.map_err(|e| RefactoringError::Execution {
            message: format!("External error: {}", e),
        })
    }

    fn convert_to_validation_error(self) -> ValidationResult<T> {
        self.map_err(|e| ValidationError::Syntactic {
            details: format!("External validation error: {}", e),
        })
    }

    fn convert_to_execution_error(self) -> ExecutionResult<T> {
        self.map_err(|e| ExecutionError::TransformationFailed {
            transformation_id: "unknown".to_string(),
        })
    }

    fn convert_to_analysis_error(self) -> AnalysisResult<T> {
        self.map_err(|e| AnalysisError::DataProcessing {
            stage: format!("External analysis error: {}", e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_aggregation() {
        let results: Vec<Result<i32, String>> = vec![
            Ok(1),
            Err("error1".to_string()),
            Ok(2),
            Err("error2".to_string()),
        ];

        let aggregated = aggregate_errors(results);
        assert!(aggregated.is_err());

        if let Err(errors) = aggregated {
            assert_eq!(errors.len(), 2);
        }
    }

    #[test]
    fn test_recoverable_error_detection() {
        let recoverable = RefactoringError::Validation {
            message: "test".to_string(),
        };
        let non_recoverable = RefactoringError::SafetyViolation {
            message: "test".to_string(),
        };

        assert!(is_recoverable_error(&recoverable));
        assert!(!is_recoverable_error(&non_recoverable));
    }

    #[test]
    fn test_error_recovery_attempt() {
        let error = RefactoringError::Validation {
            message: "test".to_string(),
        };
        let result: RefactoringResult<i32> = Err(error.clone());

        let recovery_fn = |_: &RefactoringError| Some(Ok(42));

        let recovered = attempt_recovery(result, recovery_fn);
        assert!(recovered.is_ok());
        assert_eq!(recovered.unwrap(), 42);
    }
}