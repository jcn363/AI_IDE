#![allow(missing_docs)]

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque, hash_map::Entry};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use dashmap::DashMap;
use futures::future::join_all;
use petgraph::prelude::*;
use petgraph::Graph;
use rayon::prelude::*;
use tokio::sync::{Mutex, RwLock, Semaphore, mpsc, oneshot};
use tokio::task;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn, instrument};

use crate::types::{
    AnalysisMetadata, AnalysisResult, AnalysisType, PipelineConfig, PriorityConfig, TaskPriority,
    PerformanceMetrics, AnalysisTrigger,
};

/// Errors that can occur in the analysis pipeline
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Analysis task failed: {0}")]
    TaskFailed(String),

    #[error("Dependency resolution failed: {0}")]
    DependencyFailed(String),

    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),

    #[error("Task timeout exceeded")]
    Timeout,

    #[error("Pipeline cancelled")]
    Cancelled,

    #[error("Invalid task: {0}")]
    InvalidTask(String),
}

/// Result type for pipeline operations
type PipelineResult<T> = Result<T, PipelineError>;

/// Analysis task as managed by the pipeline
#[derive(Debug, Clone)]
pub struct AnalysisTask {
    /// Unique task identifier
    pub id: String,
    /// File path to analyze
    pub file_path: PathBuf,
    /// Analysis type
    pub analysis_type: AnalysisType,
    /// Task priority
    pub priority: TaskPriority,
    /// Dependencies (task IDs this task depends on)
    pub dependencies: Vec<String>,
    /// Creation timestamp
    pub created_at: Instant,
    /// Timeout duration
    pub timeout: Duration,
    /// Task metadata
    pub metadata: HashMap<String, String>,
}

impl AnalysisTask {
    /// Create a new analysis task
    pub fn new(
        id: String,
        file_path: PathBuf,
        analysis_type: AnalysisType,
        priority: TaskPriority,
    ) -> Self {
        Self {
            id,
            file_path,
            analysis_type,
            priority,
            dependencies: Vec::new(),
            created_at: Instant::now(),
            timeout: Duration::from_secs(300), // 5 minutes default
            metadata: HashMap::new(),
        }
    }

    /// Add a dependency to this task
    pub fn with_dependency(mut self, dependency_id: impl Into<String>) -> Self {
        self.dependencies.push(dependency_id.into());
        self
    }

    /// Add metadata to the task
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if task has timed out
    pub fn is_timed_out(&self) -> bool {
        self.created_at.elapsed() > self.timeout
    }
}

/// Worker node for processing analysis tasks
#[derive(Clone)]
struct PipelineWorker {
    /// Worker identifier
    id: usize,
    /// Semaphore for coordinating work distribution
    work_semaphore: Arc<Semaphore>,
    /// Cancellation token
    cancellation: CancellationToken,
}

/// Multi-threaded analysis pipeline with dependency resolution
#[derive(Clone)]
pub struct MultiThreadedAnalysisPipeline {
    /// Internal pipeline state
    inner: Arc<PipelineInner>,
}

struct PipelineInner {
    /// Configuration
    config: PipelineConfig,

    /// Task queue with priority ordering
    task_queue: Mutex<BinaryHeap<AnalysisTask>>,

    /// Running tasks tracker
    running_tasks: DashMap<String, RunningTask>,

    /// Completed tasks cache
    completed_tasks: DashMap<String, AnalysisResult>,

    /// Dependency resolver
    dependency_resolver: DependencyResolver,

    /// Resource monitor
    resource_monitor: ResourceMonitor,

    /// Progress tracker
    progress_tracker: Arc<ProgressTracker>,

    /// Pipeline workers
    workers: Vec<PipelineWorker>,

    /// Work distribution channels
    work_tx: RwLock<Option<mpsc::UnboundedSender<AnalysisTask>>>,

    /// Fallback handler
    fallback_handler: FallbackHandler,

    /// Cancellation token for graceful shutdown
    cancellation: CancellationToken,
}

/// Currently running analysis task
#[derive(Debug, Clone)]
struct RunningTask {
    /// Task details
    task: AnalysisTask,
    /// Start timestamp
    started_at: Instant,
    /// Worker ID processing this task
    worker_id: usize,
}

/// Dependency resolver for task scheduling
#[derive(Clone)]
struct DependencyResolver {
    /// Dependency graph
    dependency_graph: Arc<RwLock<Graph<String, ()>>>,

    /// Reverse dependency mapping (task -> tasks that depend on it)
    reverse_deps: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

/// Resource monitor for dynamic scaling
#[derive(Clone)]
struct ResourceMonitor {
    /// Current system load
    system_load: AtomicUsize,

    /// Available memory in bytes
    available_memory: AtomicUsize,

    /// Active worker count
    active_workers: AtomicUsize,

    /// Resource check interval
    check_interval: Duration,
}

/// Progress tracking for user feedback
#[derive(Clone)]
pub struct ProgressTracker {
    /// Total tasks processed
    total_processed: AtomicUsize,

    /// Total queued tasks
    total_queued: AtomicUsize,

    /// Failed tasks count
    failed_tasks: AtomicUsize,

    /// Average processing time
    avg_processing_time: Mutex<Duration>,
}

/// Analysis executor trait
#[async_trait]
pub trait AnalysisExecutor {
    /// Execute an analysis task
    async fn execute_analysis(&self, task: &AnalysisTask) -> PipelineResult<AnalysisResult>;
}

/// Fallback handler for failed analysis tasks
#[derive(Clone)]
struct FallbackHandler {
    /// Maximum retry attempts
    max_retries: usize,

    /// Retry delay
    retry_delay: Duration,

    /// Failure callback
    on_failure: Option<Arc<dyn Fn(&AnalysisTask, &PipelineError) + Send + Sync>>,
}

impl MultiThreadedAnalysisPipeline {
    /// Create a new multi-threaded analysis pipeline
    pub async fn new(config: PipelineConfig) -> PipelineResult<Self> {
        let (work_tx, work_rx) = mpsc::unbounded_channel();

        let inner = PipelineInner {
            config: config.clone(),
            task_queue: Mutex::new(BinaryHeap::new()),
            running_tasks: DashMap::new(),
            completed_tasks: DashMap::new(),
            dependency_resolver: DependencyResolver::new(),
            resource_monitor: ResourceMonitor::new(config.analysis_thread_pool_size),
            progress_tracker: Arc::new(ProgressTracker::new()),
            workers: Vec::new(),
            work_tx: RwLock::new(Some(work_tx)),
            fallback_handler: FallbackHandler::new(),
            cancellation: CancellationToken::new(),
        };

        let pipeline = Self {
            inner: Arc::new(inner),
        };

        // Start worker threads
        pipeline.initialize_workers().await?;

        // Start work distribution loop
        pipeline.start_work_distribution(work_rx);

        Ok(pipeline)
    }

    /// Submit an analysis task to the pipeline
    #[instrument(skip(self), err)]
    pub async fn submit_task(&self, task: AnalysisTask) -> PipelineResult<String> {
        debug!("Submitting task: {} ({:?})", task.id, task.analysis_type);

        // Validate task
        self.validate_task(&task).await?;

        // Add to dependency graph
        self.inner.dependency_resolver.add_task(&task).await;

        // Add to queue
        {
            let mut queue = self.inner.task_queue.lock().await;
            queue.push(task.clone());

            // Update progress tracking
            self.inner.progress_tracker.inc_queued();
        }

        // Notify workers of new work
        if let Some(tx) = self.inner.work_tx.read().await.as_ref() {
            if let Err(_) = tx.send(task) {
                warn!("Failed to send task to worker - channel may be closed");
            }
        }

        Ok(task.id)
    }

    /// Submit multiple analysis tasks to the pipeline
    #[instrument(skip(self), err)]
    pub async fn submit_tasks(&self, tasks: Vec<AnalysisTask>) -> PipelineResult<Vec<String>> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for task in tasks {
            match self.submit_task(task).await {
                Ok(task_id) => results.push(task_id),
                Err(e) => errors.push(e),
            }
        }

        // If all tasks failed, return the first error
        if results.is_empty() && !errors.is_empty() {
            return Err(errors.into_iter().next().unwrap());
        }

        // Log partial failures
        if !errors.is_empty() {
            warn!("Some tasks failed to submit: {} succeeded, {} failed", results.len(), errors.len());
        }

        Ok(results)
    }

    /// Get the status of a task
    pub async fn get_task_status(&self, task_id: &str) -> TaskStatus {
        // Check if running
        if self.inner.running_tasks.contains_key(task_id) {
            return TaskStatus::Running;
        }

        // Check if completed
        if self.inner.completed_tasks.contains_key(task_id) {
            return TaskStatus::Completed;
        }

        // Check if queued
        let queue = self.inner.task_queue.lock().await;
        for task in queue.iter() {
            if task.id == task_id {
                return TaskStatus::Queued;
            }
        }

        TaskStatus::NotFound
    }

    /// Get the results of completed tasks
    pub async fn get_task_results(&self, task_ids: &[String]) -> Vec<AnalysisResult> {
        let mut results = Vec::new();

        for task_id in task_ids {
            if let Some(result) = self.inner.completed_tasks.get(task_id) {
                results.push((*result).clone());
            }
        }

        results
    }

    /// Get pipeline statistics
    pub async fn get_statistics(&self) -> PipelineStatistics {
        let progress = &self.inner.progress_tracker;
        let avg_time = *progress.avg_processing_time.lock().await;

        PipelineStatistics {
            total_processed: progress.total_processed.load(Ordering::Relaxed),
            total_queued: progress.total_queued.load(Ordering::Relaxed),
            failed_tasks: progress.failed_tasks.load(Ordering::Relaxed),
            running_tasks: self.inner.running_tasks.len(),
            avg_processing_time: avg_time,
            active_workers: self.get_active_workers(),
            queue_length: self.inner.task_queue.lock().await.len(),
        }
    }

    /// Shutdown the pipeline gracefully
    pub async fn shutdown(&self) {
        info!("Shutting down analysis pipeline");

        // Cancel all workers
        self.inner.cancellation.cancel();

        // Wait for running tasks to complete
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    /// Initialize worker threads
    async fn initialize_workers(&self) -> PipelineResult<()> {
        let worker_count = self.inner.config.analysis_thread_pool_size;

        for i in 0..worker_count {
            let worker = PipelineWorker {
                id: i,
                work_semaphore: Arc::new(Semaphore::new(1)),
                cancellation: self.inner.cancellation.clone(),
            };

            self.inner.workers.push(worker.clone());

            // Spawn worker task
            let inner_clone = Arc::clone(&self.inner);
            tokio::spawn(async move {
                worker.run(inner_clone).await;
            });
        }

        info!("Initialized {} pipeline workers", worker_count);
        Ok(())
    }

    /// Start the work distribution loop
    fn start_work_distribution(&self, mut work_rx: mpsc::UnboundedReceiver<AnalysisTask>) {
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            while let Some(task) = work_rx.recv().await {
                // Find optimal worker for this task
                if let Some(worker) = inner.select_worker(&task).await {
                    let semaphore = Arc::clone(&worker.work_semaphore);

                    // Send task to worker via semaphore coordination
                    let _permit = semaphore.acquire().await.unwrap();
                    // Worker will pick up task through the running loop
                } else {
                    warn!("No available worker for task: {}", task.id);
                }
            }
        });
    }

    /// Validate a task before submission
    async fn validate_task(&self, task: &AnalysisTask) -> PipelineResult<()> {
        if task.id.is_empty() {
            return Err(PipelineError::InvalidTask("Task ID cannot be empty".to_string()));
        }

        if task.file_path.as_os_str().is_empty() {
            return Err(PipelineError::InvalidTask("File path cannot be empty".to_string()));
        }

        if !task.file_path.exists() {
            return Err(PipelineError::InvalidTask(format!("File does not exist: {:?}", task.file_path)));
        }

        Ok(())
    }

    /// Get the number of active workers
    fn get_active_workers(&self) -> usize {
        self.inner.workers.iter().filter(|w| w.is_active()).count()
    }
}

impl PipelineWorker {
    /// Run the worker processing loop
    async fn run(self, inner: Arc<PipelineInner>) {
        info!("Pipeline worker {} started", self.id);

        loop {
            tokio::select! {
                // Wait for work or cancellation
                _ = self.cancellation.cancelled() => {
                    info!("Worker {} shutting down", self.id);
                    break;
                }

                // Try to acquire work permit
                permit = self.work_semaphore.acquire() => {
                    if let Ok(permit) = permit {
                        if let Some(task) = inner.get_next_task().await {
                            if let Err(e) = self.process_task(&inner, task).await {
                                error!("Worker {} failed to process task: {}", self.id, e);
                                inner.progress_tracker.inc_failed();
                            }
                        }

                        // Release permit for next iteration
                        drop(permit);
                    }
                }
            }
        }

        info!("Pipeline worker {} stopped", self.id);
    }

    /// Process a single analysis task
    async fn process_task(&self, inner: &Arc<PipelineInner>, task: AnalysisTask) -> PipelineResult<()> {
        // Mark task as running
        let running_task = RunningTask {
            task: task.clone(),
            started_at: Instant::now(),
            worker_id: self.id,
        };

        inner.running_tasks.insert(task.id.clone(), running_task);

        debug!("Worker {} processing task: {}", self.id, task.id);

        // Wait for dependencies
        if !task.dependencies.is_empty() {
            self.wait_for_dependencies(inner, &task).await?;
        }

        // Process the task
        let result = self.execute_task(&task).await;

        // Clean up running task
        inner.running_tasks.remove(&task.id);

        match result {
            Ok(analysis_result) => {
                inner.completed_tasks.insert(task.id.clone(), analysis_result);
                inner.progress_tracker.inc_processed();
                debug!("Task {} completed successfully", task.id);
            }

            Err(e) => {
                // Handle fallback if configured
                if let Err(fallback_err) = inner.fallback_handler.handle_failure(&task, &e).await {
                    error!("Fallback failed for task {}: {}", task.id, fallback_err);
                }

                inner.progress_tracker.inc_failed();
                return Err(e);
            }
        }

        Ok(())
    }

    /// Wait for task dependencies to complete
    async fn wait_for_dependencies(&self, inner: &Arc<PipelineInner>, task: &AnalysisTask) -> PipelineResult<()> {
        for dep_id in &task.dependencies {
            loop {
                // Check if dependency is completed
                if inner.completed_tasks.contains_key(dep_id) {
                    break;
                }

                // Check if dependency still exists (not failed/cancelled)
                if !inner.running_tasks.contains_key(dep_id) {
                    let queue = inner.task_queue.lock().await;
                    let still_queued = queue.iter().any(|t| t.id == *dep_id);

                    if !still_queued {
                        return Err(PipelineError::DependencyFailed(
                            format!("Dependency {} not found", dep_id)
                        ));
                    }
                }

                // Wait before checking again
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }

        Ok(())
    }

    /// Execute the actual analysis task
    async fn execute_task(&self, task: &AnalysisTask) -> PipelineResult<AnalysisResult> {
        let start_time = Instant::now();

        // Create analysis metadata
        let metadata = AnalysisMetadata::new(
            task.id.clone(),
            task.file_path.clone(),
            task.analysis_type.clone(),
            task.priority.clone(),
        );

        // Simulate analysis work based on task type
        // In a real implementation, this would dispatch to appropriate analyzers
        let duration = self.simulate_analysis_work(task).await;

        // Create performance metrics
        let metrics = PerformanceMetrics {
            cpu_time_ns: duration.as_nanos() as u64,
            memory_usage: 1024 * 1024, // 1MB placeholder
            io_operations: 10,
            network_requests: 0,
        };

        // Create analysis results based on task type
        let result = self.generate_analysis_result(&metadata, &metrics, duration).await;

        Ok(result)
    }

    /// Simulate analysis work duration based on file type and size
    async fn simulate_analysis_work(&self, task: &AnalysisTask) -> Duration {
        let base_duration = match task.analysis_type {
            AnalysisType::Syntax => Duration::from_millis(100),
            AnalysisType::Security => Duration::from_millis(200),
            AnalysisType::Performance => Duration::from_millis(150),
            AnalysisType::Quality => Duration::from_millis(120),
            AnalysisType::Dependencies => Duration::from_millis(80),
            AnalysisType::AiAssisted => Duration::from_millis(300),
        };

        // Simulate variable processing time
        let variability = (task.id.len() % 100) as u64;
        tokio::time::sleep(base_duration + Duration::from_millis(variability)).await;

        base_duration + Duration::from_millis(variability)
    }

    /// Generate mock analysis results
    async fn generate_analysis_result(
        &self,
        metadata: &AnalysisMetadata,
        metrics: &PerformanceMetrics,
        duration: Duration,
    ) -> AnalysisResult {
        AnalysisResult::success(metadata.clone(), Vec::new(), *metrics)
    }

    /// Check if worker is active
    fn is_active(&self) -> bool {
        !self.cancellation.is_cancelled()
    }
}

impl PipelineInner {
    /// Get the next task to process from the queue
    async fn get_next_task(&self) -> Option<AnalysisTask> {
        let mut queue = self.task_queue.lock().await;

        // Find a task whose dependencies are satisfied
        while let Some(task) = queue.pop() {
            if self.can_execute_task(&task).await {
                return Some(task);
            }

            // If dependencies not satisfied, put back in queue with lower priority
            queue.push(task);
            break; // Avoid infinite loop
        }

        None
    }

    /// Check if a task can be executed (dependencies satisfied)
    async fn can_execute_task(&self, task: &AnalysisTask) -> bool {
        for dep_id in &task.dependencies {
            if !self.completed_tasks.contains_key(dep_id) {
                return false;
            }
        }
        true
    }

    /// Select the optimal worker for a task
    async fn select_worker(&self, task: &AnalysisTask) -> Option<&PipelineWorker> {
        // Simple round-robin selection (could be enhanced with load balancing)
        if self.workers.is_empty() {
            return None;
        }

        // Find worker with lowest load
        self.workers.iter().min_by_key(|w| w.work_semaphore.available_permits())
    }
}

impl DependencyResolver {
    /// Create a new dependency resolver
    fn new() -> Self {
        Self {
            dependency_graph: Arc::new(RwLock::new(Graph::new())),
            reverse_deps: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a task to the dependency graph
    async fn add_task(&self, task: &AnalysisTask) {
        let graph = self.dependency_graph.write().await;
        // In a real implementation, this would build a proper dependency graph
        // For now, we skip complex graph construction
    }
}

impl ResourceMonitor {
    /// Create a new resource monitor
    fn new(max_workers: usize) -> Self {
        Self {
            system_load: AtomicUsize::new(0),
            available_memory: AtomicUsize::new(1024 * 1024 * 1024), // 1GB placeholder
            active_workers: AtomicUsize::new(max_workers),
            check_interval: Duration::from_secs(5),
        }
    }
}

impl ProgressTracker {
    /// Create a new progress tracker
    fn new() -> Self {
        Self {
            total_processed: AtomicUsize::new(0),
            total_queued: AtomicUsize::new(0),
            failed_tasks: AtomicUsize::new(0),
            avg_processing_time: Mutex::new(Duration::from_millis(100)),
        }
    }

    /// Increment processed counter
    fn inc_processed(&self) {
        self.total_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment queued counter
    fn inc_queued(&self) {
        self.total_queued.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment failed counter
    fn inc_failed(&self) {
        self.failed_tasks.fetch_add(1, Ordering::Relaxed);
    }
}

impl FallbackHandler {
    /// Create a new fallback handler
    fn new() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            on_failure: None,
        }
    }

    /// Handle a task failure with fallback mechanism
    async fn handle_failure(&self, task: &AnalysisTask, error: &PipelineError) -> PipelineResult<()> {
        warn!("Task {} failed: {}", task.id, error);

        // Simple exponential backoff retry logic (in a real implementation)
        for attempt in 1..=self.max_retries {
            if attempt > 1 {
                tokio::time::sleep(self.retry_delay * attempt as u32).await;
            }

            debug!("Retry attempt {} for task {}", attempt, task.id);

            // In a real implementation, this would attempt the analysis again
            // with different parameters or alternative analyzers
        }

        // Call failure callback if configured
        if let Some(callback) = &self.on_failure {
            callback(task, error);
        }

        Err(error.clone())
    }
}

/// Task status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is queued
    Queued,
    /// Task is currently running
    Running,
    /// Task has completed
    Completed,
    /// Task not found
    NotFound,
}

/// Pipeline statistics
#[derive(Debug, Clone)]
pub struct PipelineStatistics {
    /// Total tasks processed
    pub total_processed: usize,
    /// Total tasks queued
    pub total_queued: usize,
    /// Failed tasks count
    pub failed_tasks: usize,
    /// Currently running tasks
    pub running_tasks: usize,
    /// Average processing time
    pub avg_processing_time: Duration,
    /// Active workers
    pub active_workers: usize,
    /// Current queue length
    pub queue_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::fs;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let config = PipelineConfig::default();
        let pipeline = MultiThreadedAnalysisPipeline::new(config).await;
        assert!(pipeline.is_ok());
    }

    #[tokio::test]
    async fn test_task_submission() {
        let config = PipelineConfig::default();
        let pipeline = MultiThreadedAnalysisPipeline::new(config).await.unwrap();

        let task = AnalysisTask::new(
            "test-task".to_string(),
            PathBuf::from("Cargo.toml"),
            AnalysisType::Syntax,
            TaskPriority::High,
        );

        let result = pipeline.submit_task(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_validation() {
        let config = PipelineConfig::default();
        let pipeline = MultiThreadedAnalysisPipeline::new(config).await.unwrap();

        // Test invalid task (empty ID)
        let invalid_task = AnalysisTask::new(
            "".to_string(),
            PathBuf::from("test.rs"),
            AnalysisType::Syntax,
            TaskPriority::Normal,
        );

        assert!(pipeline.submit_task(invalid_task).await.is_err());
    }

    #[tokio::test]
    async fn test_statistics() {
        let config = PipelineConfig::default();
        let pipeline = MultiThreadedAnalysisPipeline::new(config).await.unwrap();

        let stats = pipeline.get_statistics().await;
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.failed_tasks, 0);
    }

    #[test]
    fn test_task_priority() {
        let high_task = AnalysisTask::new(
            "high".to_string(),
            PathBuf::from("test.rs"),
            AnalysisType::Syntax,
            TaskPriority::High,
        );

        let low_task = AnalysisTask::new(
            "low".to_string(),
            PathBuf::from("test.rs"),
            AnalysisType::Syntax,
            TaskPriority::Low,
        );

        assert_eq!(high_task.priority, TaskPriority::High);
        assert_eq!(low_task.priority, TaskPriority::Low);

        // Test that tasks with dependencies can be created
        let dependent_task = high_task.with_dependency("low");
        assert_eq!(dependent_task.dependencies.len(), 1);
        assert_eq!(dependent_task.dependencies[0], "low");
    }
}