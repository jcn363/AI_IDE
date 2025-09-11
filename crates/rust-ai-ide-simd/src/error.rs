/// SIMD-specific error types and handling
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SIMDError {
    #[error("SIMD instructions are not supported on this platform")]
    SIMDUnavailable,

    #[error("Memory allocation failed for SIMD vector: {reason}")]
    MemoryAllocationError { reason: String },

    #[error("Memory alignment requirement not satisfied: required {required}, got {actual}")]
    AlignmentError { required: usize, actual: usize },

    #[error("Vector operation size mismatch: expected {expected}, got {actual}")]
    VectorSizeMismatch { expected: usize, actual: usize },

    #[error("CPU feature detection failed: {reason}")]
    CapabilityDetectionError { reason: String },

    #[error("Fallback operation failed after SIMD acceleration failed: {reason}")]
    FallbackError { reason: String },

    #[error("Matrix multiplication dimensions are incompatible: A={a_dims}, B={b_dims}")]
    MatrixDimensionsError {
        a_dims: (usize, usize),
        b_dims: (usize, usize),
    },

    #[error("Performance monitoring operation failed: {reason}")]
    PerformanceMonitoringError { reason: String },

    #[error("Concurrent SIMD operation error: {reason}")]
    ConcurrencyError { reason: String },

    #[error("Vector operation type mismatch: expected {expected_type}")]
    TypeMismatchError { expected_type: String },

    #[error("SIMD cache operation failed: {reason}")]
    CacheError { reason: String },

    #[error("Unknown SIMD operation error: {details}")]
    UnknownError { details: String },
}

pub type SIMDResult<T> = Result<T, SIMDError>;

/// SIMD error recovery strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryStrategy {
    /// Continue with scalar fallback operations
    ScalarFallback,
    /// Fail fast with error
    FastFail,
    /// Retry with different SIMD implementation
    RetryDifferentImplementation,
    /// Degrade gracefully to reduced functionality
    DegradedMode,
    /// Log error and continue without SIMD
    SilentDegradation,
}

/// SIMD error handler for recovery strategies
pub struct SIMDRecoveryHandler {
    pub strategy: RecoveryStrategy,
    pub error_threshold: usize,
    pub current_error_count: usize,
}

impl SIMDRecoveryHandler {
    pub fn new(strategy: RecoveryStrategy) -> Self {
        Self {
            strategy,
            error_threshold: 5,
            current_error_count: 0,
        }
    }

    pub fn handle_error<T>(&mut self, error: SIMDError) -> SIMDResult<T> {
        self.current_error_count += 1;

        match self.strategy {
            RecoveryStrategy::FastFail => Err(error),
            RecoveryStrategy::ScalarFallback => {
                tracing::warn!("SIMD operation failed, falling back to scalar: {:?}", error);
                Err(SIMDError::FallbackError {
                    reason: format!("SIMD failed: {:?}", error),
                })
            }
            RecoveryStrategy::RetryDifferentImplementation => {
                if self.current_error_count < 3 {
                    tracing::warn!(
                        "SIMD operation failed, retrying with alternate implementation..."
                    );
                    Err(SIMDError::FallbackError {
                        reason: format!("Retry failed: {:?}", error),
                    })
                } else {
                    Err(error)
                }
            }
            RecoveryStrategy::DegradedMode => {
                tracing::error!("SIMD operation failed, entering degraded mode: {:?}", error);
                Err(SIMDError::FallbackError {
                    reason: format!("Degraded mode: {:?}", error),
                })
            }
            RecoveryStrategy::SilentDegradation => {
                tracing::debug!("SIMD operation failed, silently degrading: {:?}", error);
                Err(SIMDError::FallbackError {
                    reason: format!("Silent degradation: {:?}", error),
                })
            }
        }
    }

    pub fn reset_error_count(&mut self) {
        self.current_error_count = 0;
    }

    pub fn should_disable_simd(&self) -> bool {
        self.current_error_count >= self.error_threshold
    }
}

/// Utility functions for error handling
pub mod error_utils {
    use super::*;

    pub fn log_simd_error(error: &SIMDError, context: &str) {
        match error {
            SIMDError::SIMDUnavailable => {
                tracing::info!("{}: SIMD unsupported on this platform", context);
            }
            SIMDError::MemoryAllocationError { reason } => {
                tracing::error!("{}: SIMD memory allocation failed: {}", context, reason);
            }
            SIMDError::AlignmentError { required, actual } => {
                tracing::error!(
                    "{}: SIMD memory misaligned: required {}, actual {}",
                    context,
                    required,
                    actual
                );
            }
            _ => {
                tracing::error!("{}: SIMD operation error: {:?}", context, error);
            }
        }
    }

    pub fn is_recoverable_error(error: &SIMDError) -> bool {
        match error {
            SIMDError::SIMDUnavailable => false,
            SIMDError::MemoryAllocationError { .. } => false,
            SIMDError::CapabilityDetectionError { .. } => false,
            _ => true,
        }
    }

    pub fn get_error_severity(error: &SIMDError) -> ErrorSeverity {
        match error {
            SIMDError::SIMDUnavailable => ErrorSeverity::Critical,
            SIMDError::MemoryAllocationError { .. } => ErrorSeverity::Critical,
            SIMDError::CapabilityDetectionError { .. } => ErrorSeverity::High,
            SIMDError::FallbackError { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }

    /// Try to convert SIMDError to a user-friendly message
    pub fn user_friendly_error(error: &SIMDError) -> String {
        match error {
            SIMDError::SIMDUnavailable => {
                "SIMD acceleration is not available on this system".to_string()
            }
            SIMDError::MemoryAllocationError { reason } => {
                format!("SIMD operation failed due to memory issues: {}", reason)
            }
            SIMDError::AlignmentError { required, actual } => {
                format!(
                    "SIMD operation failed due to memory alignment: required {}, but got {}",
                    required, actual
                )
            }
            SIMDError::VectorSizeMismatch { expected, actual } => {
                format!(
                    "SIMD operation failed due to size mismatch: expected {}, but got {}",
                    expected, actual
                )
            }
            _ => "SIMD operation encountered an error".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}
