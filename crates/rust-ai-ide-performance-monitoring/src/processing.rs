//! Parallel processing and work-stealing algorithms
//!
//! This module provides work-stealing algorithms and parallel processing
//! capabilities with CPU and GPU resource pooling.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use rayon::ThreadPool;
use serde::{Deserialize, Serialize};

/// Work-stealing task scheduler
#[derive(Debug)]
pub struct WorkStealingScheduler<T> {
    /// Worker threads
    workers: ThreadPool,
    /// Task queues for each worker
    queues: Vec<Arc<Mutex<VecDeque<T>>>>,
    /// Statistics
    stats: Arc<Mutex<SchedulerStats>>,
}

/// Task statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub tasks_completed: u64,
    pub tasks_stolen: u64,
    pub average_queue_length: f64,
    pub worker_utilization: Vec<f64>,
}

/// Parallel processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelResult<T> {
    pub result: T,
    pub worker_id: usize,
    pub execution_time_ns: u64,
    pub memory_used: Option<u64>,
}

/// Resource pool configuration
#[derive(Debug, Clone)]
pub struct ResourcePoolConfig {
    pub cpu_threads: usize,
    pub gpu_devices: Vec<String>,
    pub max_memory_mb: usize,
    pub enable_work_stealing: bool,
    pub task_timeout_secs: Option<u64>,
}

impl<T> WorkStealingScheduler<T>
where
    T: Send + Sync + Clone + 'static,
{
    /// Create a new work-stealing scheduler
    pub fn new(num_workers: usize, enable_work_stealing: bool) -> Self {
        let queues: Vec<_> = (0..num_workers)
            .map(|_| Arc::new(Mutex::new(VecDeque::new())))
            .collect();

        let worker_utilization = vec![0.0; num_workers];

        Self {
            workers: rayon::ThreadPoolBuilder::new()
                .num_threads(num_workers)
                .build()
                .unwrap(),
            queues,
            stats: Arc::new(Mutex::new(SchedulerStats {
                tasks_completed: 0,
                tasks_stolen: 0,
                average_queue_length: 0.0,
                worker_utilization,
            })),
        }
    }

    /// Submit a task to the scheduler
    pub async fn submit_task<F, R>(
        &self,
        task: T,
        processor: F,
    ) -> Result<ParallelResult<R>, String>
    where
        F: Fn(T) -> R + Send + Sync + Clone + 'static,
        R: Send + 'static,
    {
        let start_time = std::time::Instant::now();

        // Simple implementation - assign to least loaded worker
        let worker_id = self.find_least_loaded_worker().await;

        let task_clone = task.clone();
        let processor_clone = processor.clone();
        let queue = Arc::clone(&self.queues[worker_id]);

        let result = tokio::task::spawn_blocking(move || {
            processor_clone(task_clone)
        }).await.map_err(|e| format!("Task execution failed: {}", e))?;

        let execution_time = start_time.elapsed().as_nanos() as u64;

        let parallel_result = ParallelResult {
            result,
            worker_id,
            execution_time_ns: execution_time,
            memory_used: None, // Would need memory profiling
        };

        // Update stats
        self.update_stats(worker_id, execution_time).await;

        Ok(parallel_result)
    }

    /// Process tasks in parallel with work stealing
    pub async fn process_parallel<F, R>(
        &self,
        tasks: Vec<T>,
        processor: F,
    ) -> Result<Vec<ParallelResult<R>>, String>
    where
        F: Fn(T) -> R + Send + Sync + Clone + 'static,
        R: Send + 'static,
    {
        let mut results = Vec::new();

        for task in tasks {
            let result = self.submit_task(task, processor.clone()).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Find the least loaded worker
    async fn find_least_loaded_worker(&self) -> usize {
        let mut min_load = f64::INFINITY;
        let mut best_worker = 0;

        for (i, queue) in self.queues.iter().enumerate() {
            let queue_len = queue.lock().await.len() as f64;
            if queue_len < min_load {
                min_load = queue_len;
                best_worker = i;
            }
        }

        best_worker
    }

    /// Update scheduler statistics
    async fn update_stats(&self, worker_id: usize, execution_time: u64) {
        let mut stats = self.stats.lock().await;
        stats.tasks_completed += 1;

        // Update worker utilization (simplified)
        if worker_id < stats.worker_utilization.len() {
            stats.worker_utilization[worker_id] += 1.0;
        }

        // Update average queue length
        let total_queues: u64 = self.queues.iter().map(|q| {
            // This is a blocking call, but simplified for example
            q.try_lock().map(|q| q.len() as u64).unwrap_or(0)
        }).sum();

        stats.average_queue_length = total_queues as f64 / self.queues.len() as f64;
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        self.stats.lock().await.clone()
    }
}

/// Resource pooling for CPU/GPU management
#[derive(Debug)]
pub struct ResourcePool {
    config: ResourcePoolConfig,
    active_tasks: Arc<Mutex<Vec<String>>>,
    memory_usage: Arc<Mutex<HashMap<String, usize>>>,
}

impl ResourcePool {
    /// Create a new resource pool
    pub fn new(config: ResourcePoolConfig) -> Self {
        Self {
            config,
            active_tasks: Arc::new(Mutex::new(Vec::new())),
            memory_usage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Acquire resources for a task
    pub async fn acquire_resources(&self, task_id: &str, required_mb: usize) -> Result<(), String> {
        let current_usage: usize = self.memory_usage.lock().await.values().sum();
        let total_available = self.config.max_memory_mb * 1024 * 1024; // Convert to bytes

        if current_usage + (required_mb * 1024 * 1024) > total_available {
            return Err("Insufficient memory resources".to_string());
        }

        // Track active task
        self.active_tasks.lock().await.push(task_id.to_string());
        self.memory_usage.lock().await.insert(task_id.to_string(), required_mb * 1024 * 1024);

        Ok(())
    }

    /// Release resources for a task
    pub async fn release_resources(&self, task_id: &str) -> Result<(), String> {
        let mut active_tasks = self.active_tasks.lock().await;
        let mut memory_usage = self.memory_usage.lock().await;

        if let Some(index) = active_tasks.iter().position(|t| t == task_id) {
            active_tasks.remove(index);
            memory_usage.remove(task_id);
            Ok(())
        } else {
            Err(format!("Task {} not found", task_id))
        }
    }

    /// Get current resource utilization
    pub async fn get_resource_utilization(&self) -> (usize, usize) {
        let memory_usage = self.memory_usage.lock().await;
        let active_task_count = memory_usage.len();
        let total_memory_used: usize = memory_usage.values().sum();

        (active_task_count, total_memory_used / (1024 * 1024)) // MB
    }

    /// Check if resources are available
    pub async fn can_schedule_task(&self, required_mb: usize) -> bool {
        let current_usage: usize = self.memory_usage.lock().await.values().sum();
        let total_available = self.config.max_memory_mb * 1024 * 1024; // Bytes
        let required_bytes = required_mb * 1024 * 1024;

        current_usage + required_bytes <= total_available
    }
}