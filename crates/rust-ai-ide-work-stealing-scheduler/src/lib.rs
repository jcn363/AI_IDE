//! # Work-Stealing Scheduler for Rust AI IDE
//!
//! This crate provides a high-performance work-stealing scheduler optimized for
//! concurrent operations in AI analysis and dependency initialization tasks.
//!
//! ## Architecture
//!
//! The scheduler consists of:
//! - **WorkStealingScheduler**: Main coordinator managing worker threads
//! - **Worker**: Individual worker threads with local task queues
//! - **TaskQueue**: Double-ended queue for efficient task management
//! - **Task**: Generic task trait for different operation types
//! - **Metrics**: CPU utilization and performance monitoring
//!
//! ## Features
//!
//! - Work-stealing algorithm for load balancing
//! - Tokio runtime integration
//! - CPU utilization monitoring
//! - Adaptive worker scaling
//! - Performance metrics collection

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod metrics;
pub mod scheduler;
pub mod task;
pub mod worker;

pub use scheduler::WorkStealingScheduler;
pub use task::{Task, TaskPriority};
pub use metrics::{SchedulerMetrics, CpuMetrics};
pub use worker::Worker;
pub use error::SchedulerError;

/// Configuration for the work-stealing scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Number of worker threads (0 = auto-detect CPU cores)
    pub num_workers: Option<usize>,
    /// Maximum tasks per worker queue
    pub max_queue_size: usize,
    /// Steal attempts before worker sleeps
    pub max_steal_attempts: usize,
    /// Enable CPU utilization monitoring
    pub enable_cpu_monitoring: bool,
    /// Metrics collection interval in milliseconds
    pub metrics_interval_ms: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            num_workers: None, // Auto-detect
            max_queue_size: 1024,
            max_steal_attempts: 3,
            enable_cpu_monitoring: true,
            metrics_interval_ms: 1000,
        }
    }
}

/// Task execution result
#[derive(Debug, Clone)]
pub struct TaskResult<T> {
    /// Task identifier
    pub task_id: String,
    /// Execution result
    pub result: Result<T, SchedulerError>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Worker ID that executed the task
    pub worker_id: usize,
    /// Timestamp when task completed
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = WorkStealingScheduler::new(config).await;
        assert!(scheduler.is_ok());
    }
}