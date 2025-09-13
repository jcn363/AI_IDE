//! Error types for work-stealing scheduler

use std::fmt;

/// Main scheduler error type
#[derive(Debug, Clone)]
pub enum SchedulerError {
    /// Worker creation failed
    WorkerCreationFailed(String),
    /// Task submission failed
    TaskSubmissionFailed(String),
    /// Task execution timeout
    TaskExecutionTimeout {
        task_id: String,
        timeout_ms: u64,
    },
    /// Worker communication error
    WorkerCommunicationError(String),
    /// Queue overflow
    QueueOverflow(String),
    /// Invalid configuration
    InvalidConfiguration(String),
    /// Scheduler shutdown error
    SchedulerShutdownError(String),
    /// Metrics collection error
    MetricsCollectionError(String),
    /// CPU monitoring error
    CpuMonitoringError(String),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulerError::WorkerCreationFailed(msg) => {
                write!(f, "Failed to create worker: {}", msg)
            }
            SchedulerError::TaskSubmissionFailed(msg) => {
                write!(f, "Failed to submit task: {}", msg)
            }
            SchedulerError::TaskExecutionTimeout { task_id, timeout_ms } => {
                write!(f, "Task {} execution timed out after {}ms", task_id, timeout_ms)
            }
            SchedulerError::WorkerCommunicationError(msg) => {
                write!(f, "Worker communication error: {}", msg)
            }
            SchedulerError::QueueOverflow(msg) => {
                write!(f, "Queue overflow: {}", msg)
            }
            SchedulerError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            SchedulerError::SchedulerShutdownError(msg) => {
                write!(f, "Scheduler shutdown error: {}", msg)
            }
            SchedulerError::MetricsCollectionError(msg) => {
                write!(f, "Metrics collection error: {}", msg)
            }
            SchedulerError::CpuMonitoringError(msg) => {
                write!(f, "CPU monitoring error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SchedulerError {}

/// Result type alias for scheduler operations
pub type SchedulerResult<T> = Result<T, SchedulerError>;

/// Error conversion implementations
impl From<std::io::Error> for SchedulerError {
    fn from(err: std::io::Error) -> Self {
        SchedulerError::WorkerCommunicationError(err.to_string())
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for SchedulerError {
    fn from(_err: tokio::sync::mpsc::error::SendError<()>) -> Self {
        SchedulerError::WorkerCommunicationError("Channel send failed".to_string())
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for SchedulerError {
    fn from(_err: tokio::sync::oneshot::error::RecvError) -> Self {
        SchedulerError::WorkerCommunicationError("Oneshot receive failed".to_string())
    }
}

/// Error aggregator for batch operations
#[derive(Debug, Clone)]
pub struct ErrorAggregator {
    errors: Vec<SchedulerError>,
}

impl ErrorAggregator {
    /// Create new error aggregator
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Add error to aggregator
    pub fn add_error(&mut self, error: SchedulerError) {
        self.errors.push(error);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get all errors
    pub fn errors(&self) -> &[SchedulerError] {
        &self.errors
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Convert to single error (first error if multiple)
    pub fn into_single_error(self) -> Option<SchedulerError> {
        self.errors.into_iter().next()
    }
}

impl Default for ErrorAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorAggregator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Multiple errors occurred: {}", self.errors.len())?;
        for (i, error) in self.errors.iter().enumerate() {
            write!(f, "\n  {}: {}", i + 1, error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ErrorAggregator {}