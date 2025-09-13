//! Unified Parallel Processing Library for Rust AI IDE
//!
//! This library provides comprehensive parallel processing capabilities with:
//! - Work-stealing scheduler with dynamic load balancing
//! - Intelligent task partitioning
//! - Resource pool management for CPU/GPU coordination
//! - Distributed task queue with work-stealing algorithms
//! - Adaptive concurrency control for dynamic thread scaling
//!
//! Integrated with crossbeam for lock-free operations, Tokio for async management,
//! and unified error handling via rust_ai_ide_errors.

// Re-exports for convenience
pub use futures::Future;
pub use rust_ai_ide_errors::{IDEResult, RustAIError};

// Re-export zero-copy modules
pub mod zero_copy;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use crossbeam::deque::{Injector, Stealer, Worker};
use parking_lot::{Mutex, RwLock};
use rayon::ThreadPool;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot, Semaphore};
use tokio::task::{spawn_blocking, JoinHandle};
use tracing::{debug, info, warn};
use uuid::Uuid;
pub use zero_copy::*;

/// Unique task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority levels for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low      = 0,
    Normal   = 1,
    High     = 2,
    Critical = 3,
}

/// Task execution state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

/// Generic task representation
#[derive(Debug)]
pub struct Task<T: Send + 'static> {
    pub id:           TaskId,
    pub priority:     Priority,
    pub data:         T,
    pub state:        Arc<Mutex<TaskState>>,
    pub created_at:   std::time::Instant,
    pub dependencies: Vec<TaskId>,
}

/// Result of task execution
pub type TaskResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Generic task future
pub type TaskFuture<T> = std::pin::Pin<Box<dyn Future<Output = TaskResult<T>> + Send + 'static>>;

/// Worker thread representation
#[derive(Debug)]
pub struct WorkerHandle {
    pub id:           usize,
    pub active_tasks: Arc<Mutex<usize>>,
    pub throughput:   Arc<Mutex<f64>>,
}

/// Configuration for the scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub max_workers:              usize,
    pub worker_queue_size:        usize,
    pub load_balance_interval:    std::time::Duration,
    pub adaptive_scaling_enabled: bool,
    pub min_workers:              usize,
    pub max_throughput_samples:   usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_workers:              num_cpus::get(),
            worker_queue_size:        1024,
            load_balance_interval:    std::time::Duration::from_millis(100),
            adaptive_scaling_enabled: true,
            min_workers:              1,
            max_throughput_samples:   10,
        }
    }
}

/// Work-stealing scheduler with dynamic load balancing
pub struct WorkStealingScheduler {
    config:                 SchedulerConfig,
    thread_pool:            Arc<ThreadPool>,
    injector:               Arc<Injector<TaskId>>,
    stealers:               Vec<Stealer<TaskId>>,
    task_registry:          Arc<RwLock<HashMap<TaskId, Box<dyn TaskHandle>>>>,
    resource_manager:       Arc<ResourcePoolManager>,
    concurrency_control:    Arc<AdaptiveConcurrencyControl>,
    running:                Arc<Mutex<bool>>,
    // Zero-copy enhancements
    mmap_manager:           Option<Arc<MmapManager>>,
    zero_copy_pool:         Option<Arc<ZeroCopyResourcePool>>,
    mmap_cleanup_scheduler: Arc<Mutex<BTreeMap<std::time::Instant, VecDeque<String>>>>,
    zero_copy_task_cache:   Arc<Mutex<HashMap<TaskId, Vec<String>>>>,
}

impl WorkStealingScheduler {
    pub fn new(config: SchedulerConfig, resource_manager: Arc<ResourcePoolManager>) -> Self {
        Self::new_with_zero_copy(config, resource_manager, None, None)
    }

    pub fn new_with_zero_copy(
        config: SchedulerConfig,
        resource_manager: Arc<ResourcePoolManager>,
        mmap_manager: Option<Arc<MmapManager>>,
        zero_copy_pool: Option<Arc<ZeroCopyResourcePool>>,
    ) -> Self {
        let mut stealers = Vec::with_capacity(config.max_workers);
        let mut workers: Vec<Worker<TaskId>> = (0..config.max_workers)
            .map(|_| Worker::new_fifo())
            .collect();

        let injector = Arc::new(Injector::new());

        for worker in &mut workers {
            stealers.push(worker.stealer());
        }

        let thread_pool = Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.max_workers)
                .build()
                .expect("Failed to create scheduler thread pool"),
        );

        let concurrency_control = Arc::new(AdaptiveConcurrencyControl::new(config.clone()));

        Self {
            config,
            thread_pool,
            injector,
            stealers,
            task_registry: Arc::new(RwLock::new(HashMap::new())),
            resource_manager,
            concurrency_control,
            running: Arc::new(Mutex::new(false)),
            mmap_manager,
            zero_copy_pool,
            mmap_cleanup_scheduler: Arc::new(Mutex::new(BTreeMap::new())),
            zero_copy_task_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(&self) -> IDEResult<()> {
        let mut running = self.running.lock();
        if *running {
            return Err(RustAIError::Concurrency("Scheduler already running".into()));
        }
        *running = true;
        drop(running);

        info!(
            "Starting work-stealing scheduler with {} workers",
            self.config.max_workers
        );

        // Start worker threads
        for i in 0..self.config.max_workers {
            let stealer = self.stealers[i].clone();
            let injector = self.injector.clone();
            let task_registry = self.task_registry.clone();
            let resource_manager = self.resource_manager.clone();
            let concurrency_control = self.concurrency_control.clone();

            self.thread_pool.spawn(move || {
                let mmap_mgr = mmap_manager.clone();
                let zc_pool = zero_copy_pool.clone();
                Self::worker_loop(
                    i,
                    stealer,
                    injector,
                    task_registry,
                    resource_manager,
                    concurrency_control,
                    mmap_mgr,
                    zc_pool,
                );
            });
        }

        Ok(())
    }

    pub fn stop(&self) -> IDEResult<()> {
        let mut running = self.running.lock();
        if !*running {
            return Ok(());
        }
        *running = false;

        info!("Stopping work-stealing scheduler");

        // Clear any remaining tasks
        while let Some(_) = self.injector.steal_batch_and_pop(&self.stealers[0]) {}

        Ok(())
    }

    fn worker_loop(
        worker_id: usize,
        stealer: Stealer<TaskId>,
        injector: Arc<Injector<TaskId>>,
        task_registry: Arc<RwLock<HashMap<TaskId, Box<dyn TaskHandle>>>>,
        resource_manager: Arc<ResourcePoolManager>,
        concurrency_control: Arc<AdaptiveConcurrencyControl>,
        mmap_manager: Option<Arc<MmapManager>>,
        zero_copy_pool: Option<Arc<ZeroCopyResourcePool>>,
    ) {
        debug!("Worker {} starting", worker_id);

        loop {
            // Try to steal work from global queue
            if let crossbeam::deque::Steal::Success(task_id) = injector.steal() {
                if let Some(task_handle) = task_registry.read().get(&task_id) {
                    if let Err(e) = Self::execute_task(
                        task_handle.as_ref(),
                        resource_manager.clone(),
                        mmap_manager.as_ref(),
                        zero_copy_pool.as_ref(),
                        task_id,
                    ) {
                        warn!(
                            "Worker {} failed to execute task {}: {}",
                            worker_id, task_id.0, e
                        );
                    }
                }
            }

            // Try to pop from local queue (if available)
            if let crossbeam::deque::Steal::Success(task_id) = stealer.steal() {
                if let Some(task_handle) = task_registry.read().get(&task_id) {
                    if let Err(e) = Self::execute_task(
                        task_handle.as_ref(),
                        resource_manager.clone(),
                        mmap_manager.as_ref(),
                        zero_copy_pool.as_ref(),
                        task_id,
                    ) {
                        warn!(
                            "Worker {} failed to execute task {}: {}",
                            worker_id, task_id.0, e
                        );
                    }
                }
            }

            // Small yield to prevent busy waiting
            std::thread::yield_now();
        }
    }

    async fn execute_task(
        task: &dyn TaskHandle,
        resource_manager: Arc<ResourcePoolManager>,
        mmap_manager: Option<&Arc<MmapManager>>,
        zero_copy_pool: Option<&Arc<ZeroCopyResourcePool>>,
        task_id: TaskId,
    ) -> IDEResult<()> {
        // Acquire resources
        let resources = resource_manager
            .acquire_resources(&task.required_resources())
            .await?;

        // Setup zero-copy buffers if available
        let mut mmap_files: Vec<String> = Vec::new();
        if let (Some(mmap_mgr), Some(zc_pool)) = (mmap_manager, zero_copy_pool) {
            if let Some(resources) = resources_as_mmap_resources(&task.required_resources()) {
                for (path, size) in resources {
                    let file_id = zc_pool.allocate_zero_copy_buffer(&path, size).await?;
                    mmap_files.push(file_id);
                }
            }
        }

        // Execute task with zero-copy support
        let result = if let (Some(_mmap_mgr), Some(zc_pool)) = (mmap_manager, zero_copy_pool) {
            task.execute_with_zero_copy(zc_pool).await
        } else {
            task.execute().await
        };

        // Cleanup zero-copy resources
        let mut cleanup_result = Ok(());
        if let Some(zc_pool) = zero_copy_pool {
            for file_id in mmap_files {
                if let Err(e) = zc_pool.release_zero_copy_buffer(&file_id).await {
                    warn!("Failed to cleanup zero-copy buffer {}: {}", file_id, e);
                    cleanup_result = Err(e);
                }
            }
        }

        // Release main resources
        resource_manager.release_resources(resources)?;

        match result {
            Ok(_) => cleanup_result,
            Err(e) => Err(e),
        }
    }

    /// Register memory-mapped files for automatic cleanup
    pub async fn register_mmap_cleanup(
        &self,
        file_ids: Vec<String>,
        cleanup_time: std::time::Instant,
    ) -> IDEResult<()> {
        let mut scheduler = self.mmap_cleanup_scheduler.lock();
        scheduler
            .entry(cleanup_time)
            .or_insert_with(VecDeque::new)
            .extend(file_ids);
        Ok(())
    }

    /// Clean up expired memory-mapped files
    pub async fn cleanup_expired_mmaps(&self, current_time: std::time::Instant) -> IDEResult<usize> {
        let mut scheduler = self.mmap_cleanup_scheduler.lock();
        let expired_keys: Vec<std::time::Instant> = scheduler.range(..=current_time).map(|(k, _)| *k).collect();

        let mut cleaned_count = 0;
        for key in expired_keys {
            if let Some(file_ids) = scheduler.remove(&key) {
                for file_id in file_ids {
                    if let Some(mgr) = &self.mmap_manager {
                        if let Err(e) = mgr.remove_mmap(&file_id).await {
                            warn!("Failed to cleanup expired mmap {}: {}", file_id, e);
                        } else {
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }
        Ok(cleaned_count)
    }

    pub async fn submit_task<T: Send + 'static>(&self, task: Box<dyn TaskHandle>) -> IDEResult<TaskId> {
        let task_id = TaskId::new();

        {
            let mut registry = self.task_registry.write();
            registry.insert(task_id, task);
        }

        self.injector.push(task_id);

        debug!("Submitted task {}", task_id.0);
        Ok(task_id)
    }

    pub fn get_task_state(&self, task_id: TaskId) -> Option<TaskState> {
        self.task_registry
            .read()
            .get(&task_id)
            .map(|task| task.get_state())
    }
}

/// Abstract task execution trait
pub trait TaskHandle: Send + Sync {
    fn execute(&self) -> IDEResult<()>;
    fn execute_with_zero_copy(&self, _pool: &ZeroCopyResourcePool) -> IDEResult<()> {
        // Default implementation falls back to regular execution
        self.execute()
    }
    fn required_resources(&self) -> ResourceRequirements;
    fn get_state(&self) -> TaskState;
    fn priority(&self) -> Priority;
}

/// Intelligent task partitioner for splitting work
pub struct TaskPartitioner {
    config: PartitionConfig,
}

#[derive(Debug, Clone)]
pub struct PartitionConfig {
    pub max_task_size:     usize,
    pub min_task_size:     usize,
    pub balance_load:      bool,
    pub use_data_locality: bool,
}

impl Default for PartitionConfig {
    fn default() -> Self {
        Self {
            max_task_size:     1000,
            min_task_size:     10,
            balance_load:      true,
            use_data_locality: true,
        }
    }
}

impl TaskPartitioner {
    pub fn new(config: PartitionConfig) -> Self {
        Self { config }
    }

    pub fn partition<T: Send + Clone + 'static>(
        &self,
        data: Vec<T>,
        partition_fn: fn(Vec<T>) -> Box<dyn TaskHandle>,
    ) -> Vec<Box<dyn TaskHandle>> {
        let mut partitions = Vec::new();

        if data.is_empty() {
            return partitions;
        }

        let chunk_size = (data.len() + self.config.max_task_size - 1) / self.config.max_task_size;
        let partitions_count = std::cmp::min(chunk_size, data.len() / self.config.min_task_size);

        for chunk in data.chunks(partitions_count.max(1)) {
            let chunk_vec = chunk.to_vec();
            partitions.push(partition_fn(chunk_vec));
        }

        partitions
    }

    pub fn partition_with_locality<T: Send + Clone + 'static>(
        &self,
        data: Vec<T>,
        locality_fn: fn(&T) -> usize,
        partition_fn: fn(Vec<T>) -> Box<dyn TaskHandle>,
    ) -> Vec<Box<dyn TaskHandle>> {
        if !self.config.use_data_locality {
            return self.partition(data, partition_fn);
        }

        let mut groups: HashMap<usize, Vec<T>> = HashMap::new();

        for item in data {
            let locality_key = locality_fn(&item);
            groups
                .entry(locality_key)
                .or_insert_with(Vec::new)
                .push(item);
        }

        let mut partitions = Vec::new();

        for (_, group_data) in groups {
            partitions.extend(self.partition(group_data, partition_fn));
        }

        partitions
    }
}

/// Resource requirements definition
#[derive(Debug, Clone, Default)]
pub struct ResourceRequirements {
    pub cpu_cores:         usize,
    pub memory_mb:         usize,
    pub gpu_memory_mb:     usize,
    pub network_bandwidth: usize,
}

/// Resource pool manager for CPU/GPU coordination
pub struct ResourcePoolManager {
    cpu_pool:     Arc<Semaphore>,
    gpu_pool:     Arc<RwLock<HashMap<String, Semaphore>>>,
    memory_limit: Arc<Mutex<usize>>,
    network_pool: Arc<Semaphore>,
}

impl ResourcePoolManager {
    pub fn new(total_cpu_cores: usize, memory_mb: usize, network_connections: usize) -> Self {
        Self {
            cpu_pool:     Arc::new(Semaphore::new(total_cpu_cores)),
            gpu_pool:     Arc::new(RwLock::new(HashMap::new())),
            memory_limit: Arc::new(Mutex::new(memory_mb)),
            network_pool: Arc::new(Semaphore::new(network_connections)),
        }
    }

    pub async fn acquire_resources(&self, requirements: &ResourceRequirements) -> IDEResult<ResourceHandle> {
        // Acquire CPU cores
        let cpu_permits = self
            .cpu_pool
            .acquire_many(requirements.cpu_cores as u32)
            .await
            .map_err(|_| RustAIError::Concurrency("Failed to acquire CPU resources".into()))?;

        // Acquire memory
        {
            let mut memory = self.memory_limit.lock();
            if *memory < requirements.memory_mb {
                return Err(RustAIError::Concurrency("Insufficient memory".into()));
            }
            *memory -= requirements.memory_mb;
        }

        // Acquire GPU if needed
        let gpu_permits = if requirements.gpu_memory_mb > 0 {
            let gpu_lock = self.gpu_pool.read();
            if let Some(gpu) = gpu_lock.get("default") {
                Some(
                    gpu.acquire()
                        .await
                        .map_err(|_| RustAIError::Concurrency("Failed to acquire GPU resources".into()))?,
                )
            } else {
                None
            }
        } else {
            None
        };

        // Acquire network if needed
        let network_permits = if requirements.network_bandwidth > 0 {
            Some(
                self.network_pool
                    .acquire()
                    .await
                    .map_err(|_| RustAIError::Concurrency("Failed to acquire network resources".into()))?,
            )
        } else {
            None
        };

        Ok(ResourceHandle {
            cpu_permits,
            gpu_permits,
            memory_used: requirements.memory_mb,
            network_permits,
            manager: self.clone(),
        })
    }

    pub fn release_resources(&self, handle: ResourceHandle) -> IDEResult<()> {
        drop(handle.cpu_permits);
        drop(handle.gpu_permits);
        drop(handle.network_permits);

        let mut memory = self.memory_limit.lock();
        *memory += handle.memory_used;

        Ok(())
    }
}

impl Clone for ResourcePoolManager {
    fn clone(&self) -> Self {
        Self {
            cpu_pool:     self.cpu_pool.clone(),
            gpu_pool:     self.gpu_pool.clone(),
            memory_limit: self.memory_limit.clone(),
            network_pool: self.network_pool.clone(),
        }
    }
}

/// Resource handle for automatic cleanup
pub struct ResourceHandle {
    cpu_permits:     tokio::sync::SemaphorePermit,
    gpu_permits:     Option<tokio::sync::SemaphorePermit>,
    memory_used:     usize,
    network_permits: Option<tokio::sync::SemaphorePermit>,
    manager:         ResourcePoolManager,
}

/// Distributed task queue with work-stealing algorithms
pub struct DistributedTaskQueue {
    local_queues:  Arc<Vec<Mutex<VecDeque<TaskId>>>>,
    global_queue:  Arc<Injector<TaskId>>,
    task_registry: Arc<RwLock<HashMap<TaskId, Box<dyn TaskHandle>>>>,
    load_balancer: Arc<LoadBalancer>,
}

impl DistributedTaskQueue {
    pub fn new(num_queues: usize) -> Self {
        let local_queues = Arc::new(
            (0..num_queues)
                .map(|_| Mutex::new(VecDeque::new()))
                .collect::<Vec<_>>(),
        );

        Self {
            local_queues,
            global_queue: Arc::new(Injector::new()),
            task_registry: Arc::new(RwLock::new(HashMap::new())),
            load_balancer: Arc::new(LoadBalancer::new(num_queues)),
        }
    }

    pub async fn push(&self, task: Box<dyn TaskHandle>) -> IDEResult<TaskId> {
        let task_id = TaskId::new();

        {
            let mut registry = self.task_registry.write();
            registry.insert(task_id, task);
        }

        let queue_index = self.load_balancer.select_queue();

        {
            let mut queue = self.local_queues[queue_index].lock();
            queue.push_back(task_id);
        }

        Ok(task_id)
    }

    pub fn steal_work(&self, from_queue: usize, to_queue: usize) -> Option<TaskId> {
        if let Ok(mut from_queue_lock) = self.local_queues[from_queue].try_lock() {
            if let Some(task_id) = from_queue_lock.pop_front() {
                let mut to_queue_lock = self.local_queues[to_queue].lock();
                to_queue_lock.push_back(task_id);
                return Some(task_id);
            }
        }
        None
    }

    pub fn get_task(&self, queue_index: usize) -> Option<TaskId> {
        let mut queue = self.local_queues[queue_index].lock();

        if let Some(task_id) = queue.pop_front() {
            Some(task_id)
        } else {
            // Try to steal from other queues
            for i in 0..self.local_queues.len() {
                if i != queue_index {
                    if let Some(task_id) = self.steal_work(i, queue_index) {
                        return Some(task_id);
                    }
                }
            }
            // Try global queue
            self.global_queue.steal().into_inner()
        }
    }
}

/// Load balancer for queue distribution
struct LoadBalancer {
    queue_loads: Arc<Vec<Mutex<usize>>>,
}

impl LoadBalancer {
    fn new(num_queues: usize) -> Self {
        Self {
            queue_loads: Arc::new((0..num_queues).map(|_| Mutex::new(0)).collect()),
        }
    }

    fn select_queue(&self) -> usize {
        let mut min_load = usize::MAX;
        let mut selected = 0;

        for (i, load_mutex) in self.queue_loads.iter().enumerate() {
            let load = *load_mutex.lock();
            if load < min_load {
                min_load = load;
                selected = i;
            }
        }

        selected
    }
}

/// Adaptive concurrency control for dynamic thread scaling
pub struct AdaptiveConcurrencyControl {
    config:             SchedulerConfig,
    current_throughput: Arc<Mutex<Vec<f64>>>,
    target_throughput:  f64,
    last_adjustment:    Arc<Mutex<std::time::Instant>>,
    semaphore:          Arc<Semaphore>,
}

impl AdaptiveConcurrencyControl {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_workers)),
            current_throughput: Arc::new(Mutex::new(Vec::new())),
            target_throughput: 1000.0, // tasks per second
            last_adjustment: Arc::new(Mutex::new(std::time::Instant::now())),
            config,
        }
    }

    pub async fn acquire_slot(&self) -> IDEResult<ConcurrencySlot> {
        let permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| RustAIError::Concurrency("Failed to acquire concurrency slot".into()))?;

        Ok(ConcurrencySlot { _permit: permit })
    }

    pub fn record_throughput(&self, tasks_completed: usize, duration: std::time::Duration) {
        let throughput = tasks_completed as f64 / duration.as_secs_f64();

        {
            let mut throughputs = self.current_throughput.lock();
            throughputs.push(throughput);

            if throughputs.len() > self.config.max_throughput_samples {
                throughputs.remove(0);
            }
        }

        if self.config.adaptive_scaling_enabled {
            self.adjust_concurrency()
                .unwrap_or_else(|e| warn!("Failed to adjust concurrency: {}", e));
        }
    }

    fn adjust_concurrency(&self) -> IDEResult<()> {
        let mut last_adjustment = self.last_adjustment.lock();

        if last_adjustment.elapsed() < self.config.load_balance_interval {
            return Ok(());
        }

        let throughputs = self.current_throughput.lock();
        let avg_throughput: f64 = throughputs.iter().sum::<f64>() / throughputs.len() as f64;

        let current_permits = self.semaphore.available_permits();
        let new_permits = if avg_throughput < self.target_throughput * 0.8 {
            // Increase concurrency
            std::cmp::min(current_permits + 1, self.config.max_workers)
        } else if avg_throughput > self.target_throughput * 1.2 {
            // Decrease concurrency
            std::cmp::max(current_permits.saturating_sub(1), self.config.min_workers)
        } else {
            current_permits
        };

        drop(throughputs);
        *last_adjustment = std::time::Instant::now();

        if new_permits != current_permits {
            info!(
                "Adjusting concurrency from {} to {}",
                current_permits, new_permits
            );
            // Note: In real implementation, we'd adjust the semaphore capacity
            // For now, we just log the intended change
        }

        Ok(())
    }
}

/// Concurrency slot handle
pub struct ConcurrencySlot {
    _permit: tokio::sync::SemaphorePermit,
}

/// Utility functions for parallel processing
pub mod utils {
    use super::*;

    /// Execute tasks in parallel with work-stealing distribution
    pub async fn parallel_execute<T, F, Fut>(
        scheduler: &WorkStealingScheduler,
        tasks: Vec<T>,
        task_fn: F,
    ) -> IDEResult<Vec<TaskResult<T::Output>>>
    where
        T: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel(tasks.len());

        for (i, task) in tasks.into_iter().enumerate() {
            let tx = tx.clone();
            let task_fn = task_fn.clone();

            tokio::spawn(async move {
                let result = task_fn(task).await;
                let _ = tx.send((i, result)).await;
            });
        }

        drop(tx);

        let mut results = vec![None; tasks.len()];

        while let Some((index, result)) = rx.recv().await {
            results[index] = Some(result);
        }

        Ok(results.into_iter().collect())
    }

    /// Batch processing with controlled concurrency
    pub async fn batch_process<T, F, Fut>(items: Vec<T>, batch_size: usize, processor: F) -> IDEResult<Vec<Fut::Output>>
    where
        T: Send + 'static,
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(batch_size));
        let mut handles = Vec::new();

        for chunk in items.chunks(batch_size) {
            let chunk_vec = chunk.to_vec();
            let semaphore = semaphore.clone();
            let processor = processor.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                processor(chunk_vec).await
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(
                handle
                    .await
                    .map_err(|e| RustAIError::Concurrency(e.to_string()))?,
            );
        }

        Ok(results)
    }
}

// Export commonly used types
pub use futures::Future;
pub use tracing::{debug, error, info, warn};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_basic() {
        let resource_manager = Arc::new(ResourcePoolManager::new(4, 1024, 10));
        let config = SchedulerConfig::default();
        let scheduler = WorkStealingScheduler::new(config, resource_manager);

        assert!(scheduler.start().is_ok());
        assert!(scheduler.stop().is_ok());
    }

    #[tokio::test]
    async fn test_resource_acquisition() {
        let manager = ResourcePoolManager::new(2, 1024, 1);

        let requirements = ResourceRequirements {
            cpu_cores:         1,
            memory_mb:         256,
            gpu_memory_mb:     0,
            network_bandwidth: 0,
        };

        let handle = manager.acquire_resources(&requirements).await.unwrap();
        assert!(manager.release_resources(handle).is_ok());
    }

    #[test]
    fn test_task_partitioner() {
        let partitioner = TaskPartitioner::new(PartitionConfig::default());
        let data: Vec<i32> = (0..100).collect();

        let tasks = partitioner.partition(data, |chunk| {
            Box::new(move || async { Ok(()) } as Box<dyn TaskHandle>)
        });

        assert!(!tasks.is_empty());
    }
}

/// Helper function to convert ResourceRequirements to mmap resource mappings
fn resources_as_mmap_resources(req: &ResourceRequirements) -> Option<Vec<(std::path::PathBuf, usize)>> {
    // For demonstration, assume memory_mb corresponds to mmap file sizes with temp paths
    if req.memory_mb > 0 {
        let temp_path = std::env::temp_dir().join(format!("mmap_{}", req.memory_mb));
        Some(vec![(temp_path, req.memory_mb * 1024 * 1024)]) // Convert MB to bytes
    } else {
        None
    }
}
