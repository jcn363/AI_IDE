//! Common types used across task modules

use serde::{Deserialize, Serialize};

/// Represents the status of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is queued but not yet started
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed to complete
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Represents the execution strategy for a task chain
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    /// Execute tasks one after another (sequential)
    Sequential,
    /// Execute all tasks in parallel
    Parallel,
    /// Stop on first failure (like `&&` in shell)
    StopOnFailure,
}
