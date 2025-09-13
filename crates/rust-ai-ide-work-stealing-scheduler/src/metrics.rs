//! Performance metrics and CPU utilization monitoring for work-stealing scheduler

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// CPU utilization metrics
#[derive(Debug, Clone)]
pub struct CpuMetrics {
    /// Overall CPU usage percentage (0.0-100.0)
    pub overall_usage: f64,
    /// Per-core usage percentages
    pub per_core_usage: Vec<f64>,
    /// System load average (1, 5, 15 minute averages)
    pub load_average: (f64, f64, f64),
    /// Timestamp when metrics were collected
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for CpuMetrics {
    fn default() -> Self {
        Self {
            overall_usage: 0.0,
            per_core_usage: Vec::new(),
            load_average: (0.0, 0.0, 0.0),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Task execution metrics
#[derive(Debug, Clone)]
pub struct TaskExecutionMetrics {
    /// Total tasks executed
    pub total_executed: u64,
    /// Tasks executed successfully
    pub successful: u64,
    /// Tasks that failed
    pub failed: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Median execution time in milliseconds
    pub median_execution_time_ms: f64,
    /// 95th percentile execution time
    pub p95_execution_time_ms: f64,
    /// 99th percentile execution time
    pub p99_execution_time_ms: f64,
}

impl Default for TaskExecutionMetrics {
    fn default() -> Self {
        Self {
            total_executed: 0,
            successful: 0,
            failed: 0,
            avg_execution_time_ms: 0.0,
            median_execution_time_ms: 0.0,
            p95_execution_time_ms: 0.0,
            p99_execution_time_ms: 0.0,
        }
    }
}

/// Work-stealing metrics
#[derive(Debug, Clone)]
pub struct WorkStealingMetrics {
    /// Total steal attempts
    pub steal_attempts: u64,
    /// Successful steals
    pub successful_steals: u64,
    /// Failed steal attempts
    pub failed_steals: u64,
    /// Steal success rate (0.0-1.0)
    pub steal_success_rate: f64,
    /// Average tasks per worker
    pub avg_tasks_per_worker: f64,
    /// Worker load distribution variance
    pub load_variance: f64,
}

impl Default for WorkStealingMetrics {
    fn default() -> Self {
        Self {
            steal_attempts: 0,
            successful_steals: 0,
            failed_steals: 0,
            steal_success_rate: 0.0,
            avg_tasks_per_worker: 0.0,
            load_variance: 0.0,
        }
    }
}

/// Queue performance metrics
#[derive(Debug, Clone)]
pub struct QueueMetrics {
    /// Average queue depth across all workers
    pub avg_queue_depth: f64,
    /// Maximum queue depth observed
    pub max_queue_depth: usize,
    /// Queue overflow events
    pub overflow_events: u64,
    /// Average time tasks spend in queue (milliseconds)
    pub avg_queue_time_ms: f64,
}

impl Default for QueueMetrics {
    fn default() -> Self {
        Self {
            avg_queue_depth: 0.0,
            max_queue_depth: 0,
            overflow_events: 0,
            avg_queue_time_ms: 0.0,
        }
    }
}

/// Comprehensive scheduler metrics
#[derive(Debug, Clone)]
pub struct SchedulerMetrics {
    /// CPU utilization metrics
    pub cpu_metrics: CpuMetrics,
    /// Task execution metrics
    pub task_metrics: TaskExecutionMetrics,
    /// Work-stealing metrics
    pub work_stealing: WorkStealingMetrics,
    /// Queue performance metrics
    pub queue_metrics: QueueMetrics,
    /// Per-worker statistics
    pub worker_stats: HashMap<usize, WorkerStats>,
    /// Per-task-type statistics
    pub task_type_stats: HashMap<String, TaskTypeStats>,
    /// Scheduler uptime in seconds
    pub uptime_seconds: u64,
    /// Timestamp when metrics were collected
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

impl Default for SchedulerMetrics {
    fn default() -> Self {
        Self {
            cpu_metrics: CpuMetrics::default(),
            task_metrics: TaskExecutionMetrics::default(),
            work_stealing: WorkStealingMetrics::default(),
            queue_metrics: QueueMetrics::default(),
            worker_stats: HashMap::new(),
            task_type_stats: HashMap::new(),
            uptime_seconds: 0,
            collected_at: chrono::Utc::now(),
        }
    }
}

/// Individual worker statistics
#[derive(Debug, Clone)]
pub struct WorkerStats {
    /// Worker ID
    pub worker_id: usize,
    /// Tasks executed by this worker
    pub tasks_executed: u64,
    /// Current queue depth
    pub queue_depth: usize,
    /// Successful steals performed
    pub steals_performed: u64,
    /// Successful steals received
    pub steals_received: u64,
    /// Worker uptime in seconds
    pub uptime_seconds: u64,
    /// CPU time used by worker (seconds)
    pub cpu_time_seconds: f64,
}

impl Default for WorkerStats {
    fn default() -> Self {
        Self {
            worker_id: 0,
            tasks_executed: 0,
            queue_depth: 0,
            steals_performed: 0,
            steals_received: 0,
            uptime_seconds: 0,
            cpu_time_seconds: 0.0,
        }
    }
}

/// Task type statistics
#[derive(Debug, Clone)]
pub struct TaskTypeStats {
    /// Task type name
    pub task_type: String,
    /// Total tasks of this type
    pub total_count: u64,
    /// Successful executions
    pub successful_count: u64,
    /// Failed executions
    pub failed_count: u64,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: f64,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
}

impl Default for TaskTypeStats {
    fn default() -> Self {
        Self {
            task_type: String::new(),
            total_count: 0,
            successful_count: 0,
            failed_count: 0,
            avg_execution_time_ms: 0.0,
            success_rate: 0.0,
        }
    }
}

/// Metrics collector for gathering system and scheduler statistics
pub struct MetricsCollector {
    start_time: std::time::Instant,
    execution_times: Arc<RwLock<Vec<u64>>>,
    task_counts: Arc<RwLock<HashMap<String, u64>>>,
    worker_stats: Arc<RwLock<HashMap<usize, WorkerStats>>>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            execution_times: Arc::new(RwLock::new(Vec::new())),
            task_counts: Arc::new(RwLock::new(HashMap::new())),
            worker_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record task execution time
    pub async fn record_execution_time(&self, duration_ms: u64) {
        let mut times = self.execution_times.write().await;
        times.push(duration_ms);

        // Keep only last 1000 measurements to prevent unbounded growth
        if times.len() > 1000 {
            times.remove(0);
        }
    }

    /// Record task completion by type
    pub async fn record_task_completion(&self, task_type: &str, success: bool) {
        let mut counts = self.task_counts.write().await;
        let key = format!("{}_{}", task_type, if success { "success" } else { "failure" });
        *counts.entry(key).or_insert(0) += 1;
    }

    /// Update worker statistics
    pub async fn update_worker_stats(&self, worker_id: usize, stats: WorkerStats) {
        let mut worker_stats = self.worker_stats.write().await;
        worker_stats.insert(worker_id, stats);
    }

    /// Collect comprehensive metrics
    pub async fn collect_metrics(&self) -> SchedulerResult<SchedulerMetrics> {
        let uptime = self.start_time.elapsed().as_secs();

        let execution_times = self.execution_times.read().await;
        let task_counts = self.task_counts.read().await;
        let worker_stats = self.worker_stats.read().await;

        // Calculate percentiles from execution times
        let mut sorted_times = execution_times.clone();
        sorted_times.sort_unstable();

        let (median, p95, p99) = if sorted_times.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let len = sorted_times.len();
            (
                sorted_times[len / 2] as f64,
                sorted_times[(len * 95) / 100] as f64,
                sorted_times[(len * 99) / 100] as f64,
            )
        };

        let avg_execution_time = if sorted_times.is_empty() {
            0.0
        } else {
            sorted_times.iter().sum::<u64>() as f64 / sorted_times.len() as f64
        };

        // Aggregate task statistics
        let mut task_type_stats = HashMap::new();
        for (key, count) in &*task_counts {
            let parts: Vec<&str> = key.split('_').collect();
            if parts.len() >= 2 {
                let task_type = parts[..parts.len() - 1].join("_");
                let is_success = parts.last() == Some(&"success");

                let stats = task_type_stats.entry(task_type.clone()).or_insert(TaskTypeStats {
                    task_type: task_type.clone(),
                    total_count: 0,
                    successful_count: 0,
                    failed_count: 0,
                    avg_execution_time_ms: avg_execution_time,
                    success_rate: 0.0,
                });

                stats.total_count += *count;
                if is_success {
                    stats.successful_count += *count;
                } else {
                    stats.failed_count += *count;
                }
                stats.success_rate = stats.successful_count as f64 / stats.total_count as f64;
            }
        }

        Ok(SchedulerMetrics {
            cpu_metrics: CpuMetrics::default(), // Would be collected from system
            task_metrics: TaskExecutionMetrics {
                total_executed: sorted_times.len() as u64,
                successful: task_counts.get("success").copied().unwrap_or(0),
                failed: task_counts.get("failure").copied().unwrap_or(0),
                avg_execution_time_ms: avg_execution_time,
                median_execution_time_ms: median,
                p95_execution_time_ms: p95,
                p99_execution_time_ms: p99,
            },
            work_stealing: WorkStealingMetrics::default(), // Would be collected from scheduler
            queue_metrics: QueueMetrics::default(), // Would be collected from queues
            worker_stats: worker_stats.clone(),
            task_type_stats,
            uptime_seconds: uptime,
            collected_at: chrono::Utc::now(),
        })
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

use crate::error::{SchedulerError, SchedulerResult};