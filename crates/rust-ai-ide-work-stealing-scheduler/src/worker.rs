//! Worker implementation for work-stealing scheduler

use crossbeam_deque::{Injector, Stealer, Worker as DequeWorker};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::error::{SchedulerError, SchedulerResult};
use crate::task::{BoxedTask, TaskResult};
use crate::SchedulerConfig;

/// Task queue using crossbeam's work-stealing deque
pub struct TaskQueue {
    /// Local worker deque for LIFO operations
    local_worker: DequeWorker<BoxedTask>,
    /// Stealer for other workers to steal tasks
    stealer: Stealer<BoxedTask>,
    /// Global injector for task submission
    global_injector: Arc<Injector<BoxedTask>>,
    /// Current queue depth
    depth: AtomicUsize,
    /// Maximum queue size
    max_size: usize,
}

impl TaskQueue {
    /// Create new task queue
    pub fn new(max_size: usize, global_injector: Arc<Injector<BoxedTask>>) -> Self {
        let local_worker = DequeWorker::new_fifo();
        let stealer = local_worker.stealer();

        Self {
            local_worker,
            stealer,
            global_injector,
            depth: AtomicUsize::new(0),
            max_size,
        }
    }

    /// Push task to local end (LIFO)
    pub fn push(&self, task: BoxedTask) -> SchedulerResult<()> {
        let current_depth = self.depth.load(Ordering::Acquire);
        if current_depth >= self.max_size {
            return Err(SchedulerError::QueueOverflow(
                format!("Queue depth {} exceeds maximum {}", current_depth, self.max_size)
            ));
        }

        self.local_worker.push(task);
        self.depth.fetch_add(1, Ordering::Release);
        Ok(())
    }

    /// Pop task from local end (LIFO)
    pub fn pop(&self) -> Option<BoxedTask> {
        let task = self.local_worker.pop()?;
        self.depth.fetch_sub(1, Ordering::Release);
        Some(task)
    }

    /// Steal task from opposite end (FIFO)
    pub fn steal(&self) -> Option<BoxedTask> {
        let task = self.stealer.steal()?;
        self.depth.fetch_sub(1, Ordering::Release);
        Some(task)
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.depth.load(Ordering::Acquire) == 0
    }

    /// Get current queue depth
    pub fn depth(&self) -> usize {
        self.depth.load(Ordering::Acquire)
    }

    /// Try to steal from global injector
    pub fn steal_global(&self) -> Option<BoxedTask> {
        self.global_injector.steal()
    }
}

/// Individual worker thread in the work-stealing scheduler
pub struct Worker {
    /// Worker ID
    id: usize,
    /// Task queue
    queue: Arc<TaskQueue>,
    /// Worker statistics
    stats: Arc<RwLock<crate::metrics::WorkerStats>>,
    /// Running flag
    is_running: Arc<AtomicBool>,
    /// Result sender
    result_tx: mpsc::UnboundedSender<TaskResult<serde_json::Value>>,
    /// Other workers for stealing (populated after construction)
    other_workers: parking_lot::RwLock<Vec<Arc<TaskQueue>>>,
    /// Configuration
    config: SchedulerConfig,
}

impl Worker {
    /// Create new worker
    pub fn new(
        id: usize,
        global_injector: Arc<Injector<BoxedTask>>,
        result_tx: mpsc::UnboundedSender<TaskResult<serde_json::Value>>,
        config: SchedulerConfig,
    ) -> Self {
        let queue = Arc::new(TaskQueue::new(config.max_queue_size, global_injector));

        let stats = Arc::new(RwLock::new(crate::metrics::WorkerStats {
            worker_id: id,
            tasks_executed: 0,
            queue_depth: 0,
            steals_performed: 0,
            steals_received: 0,
            uptime_seconds: 0,
            cpu_time_seconds: 0.0,
        }));

        Self {
            id,
            queue,
            stats,
            is_running: Arc::new(AtomicBool::new(true)),
            result_tx,
            other_workers: parking_lot::RwLock::new(Vec::new()),
            config,
        }
    }

    /// Set other workers for stealing (called after all workers are created)
    pub fn set_other_workers(&self, other_workers: Vec<Arc<TaskQueue>>) {
        let mut workers = self.other_workers.write();
        *workers = other_workers;
    }

    /// Get worker ID
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get worker queue
    pub fn queue(&self) -> &Arc<TaskQueue> {
        &self.queue
    }

    /// Get worker statistics
    pub fn stats(&self) -> &Arc<RwLock<crate::metrics::WorkerStats>> {
        &self.stats
    }

    /// Stop worker
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Release);
    }

    /// Check if worker is running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }

    /// Submit task to this worker
    pub fn submit_task(&self, task: BoxedTask) -> SchedulerResult<()> {
        self.queue.push(task)
    }

    /// Run worker loop
    pub async fn run(&self) {
        let start_time = std::time::Instant::now();

        while self.is_running() {
            // Try to get task from local queue first (LIFO)
            if let Some(task) = self.queue.pop() {
                self.execute_task(task).await;
                continue;
            }

            // Try to steal from global injector
            if let Some(task) = self.queue.steal_global() {
                self.execute_task(task).await;
                continue;
            }

            // Try to steal from other workers
            let mut stole_task = false;
            let other_workers = self.other_workers.read().clone();
            for other_queue in &other_workers {
                if let Some(task) = other_queue.steal() {
                    // Update steal statistics
                    {
                        let mut stats = self.stats.write().await;
                        stats.steals_performed += 1;
                    }

                    // Update victim's steal received count
                    // Note: This is a simplification. In practice, you'd need proper cross-worker stats
                    self.execute_task(task).await;
                    stole_task = true;
                    break;
                }
            }

            if stole_task {
                continue;
            }

            // No tasks available, sleep briefly before retrying
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        }

        // Update final statistics
        let uptime = start_time.elapsed().as_secs();
        {
            let mut stats = self.stats.write().await;
            stats.uptime_seconds = uptime as u64;
        }
    }

    /// Execute a single task
    async fn execute_task(&self, task: BoxedTask) {
        let task_id = task.task_id();
        let start_time = std::time::Instant::now();

        let result = task.execute().await;
        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.tasks_executed += 1;
            stats.queue_depth = self.queue.depth();
            stats.cpu_time_seconds += execution_time as f64 / 1000.0;
        }

        // Send result
        let task_result = TaskResult {
            task_id: task_id.clone(),
            result: result.map_err(|e| SchedulerError::TaskExecutionTimeout {
                task_id,
                timeout_ms: execution_time,
            }),
            execution_time_ms: execution_time,
            worker_id: self.id,
            completed_at: chrono::Utc::now(),
        };

        if let Err(e) = self.result_tx.send(task_result) {
            tracing::error!("Failed to send task result: {}", e);
        }
    }

    /// Get worker status summary
    pub async fn status(&self) -> WorkerStatus {
        let stats = self.stats.read().await;
        WorkerStatus {
            worker_id: self.id,
            is_running: self.is_running(),
            queue_depth: self.queue.depth(),
            tasks_executed: stats.tasks_executed,
            steals_performed: stats.steals_performed,
            uptime_seconds: stats.uptime_seconds,
        }
    }
}

/// Worker status summary
#[derive(Debug, Clone)]
pub struct WorkerStatus {
    /// Worker ID
    pub worker_id: usize,
    /// Whether worker is running
    pub is_running: bool,
    /// Current queue depth
    pub queue_depth: usize,
    /// Total tasks executed
    pub tasks_executed: u64,
    /// Successful steals performed
    pub steals_performed: u64,
    /// Worker uptime in seconds
    pub uptime_seconds: u64,
}

/// Worker pool for managing multiple workers
pub struct WorkerPool {
    workers: Vec<Arc<Worker>>,
    global_injector: Arc<Injector<BoxedTask>>,
    result_rx: mpsc::UnboundedReceiver<TaskResult<serde_json::Value>>,
    result_tx: mpsc::UnboundedSender<TaskResult<serde_json::Value>>,
}

impl WorkerPool {
    /// Create new worker pool
    pub fn new(num_workers: usize, config: SchedulerConfig) -> Self {
        let global_injector = Arc::new(Injector::new());
        let (result_tx, result_rx) = mpsc::unbounded_channel();

        let mut workers = Vec::with_capacity(num_workers);
        let mut queues = Vec::with_capacity(num_workers);

        // Create workers first to build the queues list
        for id in 0..num_workers {
            let worker = Arc::new(Worker::new(
                id,
                Arc::clone(&global_injector),
                result_tx.clone(),
                config.clone(),
            ));
            let queue = Arc::clone(worker.queue());
            workers.push(worker);
            queues.push(queue);
        }

        // Set up cross-worker stealing relationships
        for i in 0..num_workers {
            let mut other_queues = queues.clone();
            other_queues.remove(i); // Remove this worker's own queue
            workers[i].set_other_workers(other_queues);
        }

        Self {
            workers,
            global_injector,
            result_rx,
            result_tx,
        }
    }

    /// Start all workers
    pub async fn start(&self) -> SchedulerResult<()> {
        for worker in &self.workers {
            let worker_clone = Arc::clone(worker);
            tokio::spawn(async move {
                worker_clone.run().await;
            });
        }
        Ok(())
    }

    /// Stop all workers
    pub async fn stop(&self) -> SchedulerResult<()> {
        for worker in &self.workers {
            worker.stop();
        }
        Ok(())
    }

    /// Submit task to worker pool
    pub fn submit_task(&self, task: BoxedTask) -> SchedulerResult<()> {
        self.global_injector.push(task);
        Ok(())
    }

    /// Receive task result
    pub async fn receive_result(&mut self) -> Option<TaskResult<serde_json::Value>> {
        self.result_rx.recv().await
    }

    /// Get worker pool status
    pub async fn status(&self) -> Vec<WorkerStatus> {
        let mut status = Vec::with_capacity(self.workers.len());
        for worker in &self.workers {
            status.push(worker.status().await);
        }
        status
    }

    /// Get number of workers
    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }
}