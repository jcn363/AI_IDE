//! Main work-stealing scheduler implementation

use std::sync::Arc;

use crossbeam_deque::Injector;
use tokio::sync::{mpsc, RwLock};
use tokio::task;

use crate::error::{SchedulerError, SchedulerResult};
use crate::metrics::{MetricsCollector, SchedulerMetrics};
use crate::task::{BoxedTask, TaskResult};
use crate::worker::{Worker, WorkerPool};
use crate::SchedulerConfig;

/// Main work-stealing scheduler
pub struct WorkStealingScheduler {
    /// Worker pool
    worker_pool:     WorkerPool,
    /// Global task injector
    global_injector: Arc<Injector<BoxedTask>>,
    /// Metrics collector
    metrics:         Arc<MetricsCollector>,
    /// Result receiver
    result_rx:       mpsc::UnboundedReceiver<TaskResult<serde_json::Value>>,
    /// Configuration
    config:          SchedulerConfig,
    /// Running flag
    is_running:      std::sync::atomic::AtomicBool,
    /// Start time for uptime tracking
    start_time:      std::time::Instant,
}

impl WorkStealingScheduler {
    /// Create new work-stealing scheduler
    pub async fn new(config: SchedulerConfig) -> SchedulerResult<Self> {
        // Validate configuration
        Self::validate_config(&config)?;

        // Determine number of workers
        let num_workers = config.num_workers.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4)
        });

        // Create worker pool
        let worker_pool = WorkerPool::new(num_workers, config);

        // Start worker pool
        worker_pool.start().await?;

        let (result_tx, result_rx) = mpsc::unbounded_channel();

        // Create metrics collector
        let metrics = Arc::new(MetricsCollector::new());

        // Start metrics collection task if enabled
        if config.enable_cpu_monitoring {
            Self::start_metrics_collection(Arc::clone(&metrics), config.metrics_interval_ms);
        }

        // Start result processor task
        Self::start_result_processor(result_rx.resubscribe(), Arc::clone(&metrics));

        Ok(Self {
            worker_pool,
            global_injector: Arc::new(Injector::new()),
            metrics,
            result_rx,
            config,
            is_running: std::sync::atomic::AtomicBool::new(true),
            start_time: std::time::Instant::now(),
        })
    }

    /// Validate scheduler configuration
    fn validate_config(config: &SchedulerConfig) -> SchedulerResult<()> {
        if config.max_queue_size == 0 {
            return Err(SchedulerError::InvalidConfiguration(
                "max_queue_size must be greater than 0".to_string(),
            ));
        }

        if config.max_steal_attempts == 0 {
            return Err(SchedulerError::InvalidConfiguration(
                "max_steal_attempts must be greater than 0".to_string(),
            ));
        }

        if let Some(num_workers) = config.num_workers {
            if num_workers == 0 {
                return Err(SchedulerError::InvalidConfiguration(
                    "num_workers must be greater than 0 if specified".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Submit task for execution
    pub async fn submit_task(&self, task: BoxedTask) -> SchedulerResult<String> {
        if !self.is_running.load(std::sync::atomic::Ordering::Acquire) {
            return Err(SchedulerError::SchedulerShutdownError(
                "Scheduler is shutting down".to_string(),
            ));
        }

        let task_id = task.task_id();

        // Try to submit to global injector first
        self.global_injector.push(task);

        Ok(task_id)
    }

    /// Submit multiple tasks for batch execution
    pub async fn submit_batch(&self, tasks: Vec<BoxedTask>) -> SchedulerResult<Vec<String>> {
        let mut task_ids = Vec::with_capacity(tasks.len());

        for task in tasks {
            task_ids.push(self.submit_task(task).await?);
        }

        Ok(task_ids)
    }

    /// Wait for task completion and get result
    pub async fn wait_for_task(&mut self, task_id: &str) -> SchedulerResult<TaskResult<serde_json::Value>> {
        while let Some(result) = self.result_rx.recv().await {
            if result.task_id == task_id {
                return Ok(result);
            }
            // Continue waiting for the specific task
        }

        Err(SchedulerError::TaskExecutionTimeout {
            task_id:    task_id.to_string(),
            timeout_ms: 0, // Would need timeout implementation
        })
    }

    /// Wait for multiple tasks to complete
    pub async fn wait_for_batch(&mut self, task_ids: &[String]) -> SchedulerResult<Vec<TaskResult<serde_json::Value>>> {
        let mut results = Vec::new();
        let mut remaining_ids: std::collections::HashSet<_> = task_ids.iter().cloned().collect();

        while !remaining_ids.is_empty() {
            if let Some(result) = self.result_rx.recv().await {
                if remaining_ids.remove(&result.task_id) {
                    results.push(result);
                }
            } else {
                break;
            }
        }

        Ok(results)
    }

    /// Get scheduler metrics
    pub async fn metrics(&self) -> SchedulerResult<SchedulerMetrics> {
        self.metrics.collect_metrics().await
    }

    /// Get scheduler status
    pub async fn status(&self) -> SchedulerStatus {
        let metrics = self.metrics().await.unwrap_or_default();

        SchedulerStatus {
            is_running:           self.is_running.load(std::sync::atomic::Ordering::Acquire),
            num_workers:          self.worker_pool.num_workers(),
            uptime_seconds:       self.start_time.elapsed().as_secs() as u64,
            total_tasks_executed: metrics.task_metrics.total_executed,
            active_tasks:         self
                .worker_pool
                .status()
                .await
                .iter()
                .map(|w| w.queue_depth)
                .sum::<usize>(),
            worker_status:        self.worker_pool.status().await,
        }
    }

    /// Shutdown scheduler gracefully
    pub async fn shutdown(self) -> SchedulerResult<()> {
        self.is_running
            .store(false, std::sync::atomic::Ordering::Release);

        // Stop worker pool
        self.worker_pool.stop().await?;

        // Close channels
        // Note: Channels will be closed when Self is dropped

        Ok(())
    }

    /// Start metrics collection background task
    fn start_metrics_collection(metrics: Arc<MetricsCollector>, interval_ms: u64) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));

            loop {
                interval.tick().await;

                // Collect CPU metrics (simplified - would integrate with system monitoring)
                // In production, this would collect actual CPU usage

                // Update metrics
                let _ = metrics.collect_metrics().await;
            }
        });
    }

    /// Start result processor background task
    fn start_result_processor(
        mut result_rx: mpsc::UnboundedReceiver<TaskResult<serde_json::Value>>,
        metrics: Arc<MetricsCollector>,
    ) {
        tokio::spawn(async move {
            while let Some(result) = result_rx.recv().await {
                // Record execution time
                let _ = metrics
                    .record_execution_time(result.execution_time_ms)
                    .await;

                // Record task completion
                let success = result.result.is_ok();
                let _ = metrics.record_task_completion("generic", success).await;

                // Could add more sophisticated metrics recording here
            }
        });
    }

    /// Get number of workers
    pub fn num_workers(&self) -> usize {
        self.worker_pool.num_workers()
    }

    /// Check if scheduler is running
    pub fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::Acquire)
    }
}

/// Scheduler status information
#[derive(Debug, Clone)]
pub struct SchedulerStatus {
    /// Whether scheduler is running
    pub is_running:           bool,
    /// Number of worker threads
    pub num_workers:          usize,
    /// Scheduler uptime in seconds
    pub uptime_seconds:       u64,
    /// Total tasks executed
    pub total_tasks_executed: u64,
    /// Currently active tasks (in queues)
    pub active_tasks:         usize,
    /// Individual worker status
    pub worker_status:        Vec<crate::worker::WorkerStatus>,
}

impl Default for SchedulerStatus {
    fn default() -> Self {
        Self {
            is_running:           false,
            num_workers:          0,
            uptime_seconds:       0,
            total_tasks_executed: 0,
            active_tasks:         0,
            worker_status:        Vec::new(),
        }
    }
}

/// Builder pattern for scheduler configuration
pub struct SchedulerBuilder {
    config: SchedulerConfig,
}

impl SchedulerBuilder {
    /// Create new scheduler builder
    pub fn new() -> Self {
        Self {
            config: SchedulerConfig::default(),
        }
    }

    /// Set number of workers
    pub fn num_workers(mut self, num_workers: usize) -> Self {
        self.config.num_workers = Some(num_workers);
        self
    }

    /// Set maximum queue size per worker
    pub fn max_queue_size(mut self, size: usize) -> Self {
        self.config.max_queue_size = size;
        self
    }

    /// Set maximum steal attempts
    pub fn max_steal_attempts(mut self, attempts: usize) -> Self {
        self.config.max_steal_attempts = attempts;
        self
    }

    /// Enable/disable CPU monitoring
    pub fn enable_cpu_monitoring(mut self, enable: bool) -> Self {
        self.config.enable_cpu_monitoring = enable;
        self
    }

    /// Set metrics collection interval
    pub fn metrics_interval_ms(mut self, interval: u64) -> Self {
        self.config.metrics_interval_ms = interval;
        self
    }

    /// Build the scheduler
    pub async fn build(self) -> SchedulerResult<WorkStealingScheduler> {
        WorkStealingScheduler::new(self.config).await
    }
}

impl Default for SchedulerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::helpers;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = SchedulerBuilder::new()
            .num_workers(2)
            .max_queue_size(10)
            .build()
            .await;

        assert!(scheduler.is_ok());
        let scheduler = scheduler.unwrap();
        assert_eq!(scheduler.num_workers(), 2);
    }

    #[tokio::test]
    async fn test_task_submission() {
        let scheduler = SchedulerBuilder::new()
            .num_workers(2)
            .build()
            .await
            .unwrap();

        let task = helpers::cpu_task("test-task", 42i32, |data| async move {
            // Simulate CPU work
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            Ok(serde_json::json!({ "result": data, "processed": true }))
        });

        let task_id = scheduler.submit_task(Box::new(task)).await;
        assert!(task_id.is_ok());
    }
}
