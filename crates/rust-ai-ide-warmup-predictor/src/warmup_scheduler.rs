//! Intelligent warmup scheduler for orchestrating model warmup operations
//!
//! This module provides sophisticated scheduling algorithms that consider:
//! - Resource availability and constraints
//! - Task priorities and dependencies
//! - Time-based scheduling and deadlines
//! - Performance impact assessment
//! - Background processing capabilities

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::error::{Result, WarmupError};
use crate::types::{
    ModelId, ModelPrediction, RequestPriority, ResourceAvailability,
    ResourceRequirements, WarmupConfig, WarmupSchedule, WarmupTask,
};

/// Intelligent warmup scheduler with advanced scheduling algorithms
#[derive(Debug)]
pub struct WarmupScheduler {
    /// Configuration settings
    config: Arc<RwLock<WarmupConfig>>,
    /// Scheduled tasks queue
    task_queue: Arc<RwLock<BinaryHeap<ScheduledTask>>>,
    /// Running tasks tracker
    running_tasks: Arc<RwLock<HashMap<ModelId, RunningTask>>>,
    /// Task dependencies graph
    dependencies: Arc<RwLock<HashMap<ModelId, HashSet<ModelId>>>>,
    /// Resource allocation tracker
    resource_allocation: Arc<RwLock<ResourceAllocation>>,
    /// Scheduling metrics
    metrics: Arc<RwLock<SchedulerMetrics>>,
    /// Background scheduler task
    background_scheduler: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

/// Scheduled task with priority and timing information
#[derive(Debug, Clone, Eq)]
struct ScheduledTask {
    /// The warmup task to execute
    task: WarmupTask,
    /// Scheduled execution time
    scheduled_time: Instant,
    /// Priority score for ordering
    priority_score: u64,
    /// Task sequence number for tie-breaking
    sequence_number: u64,
}

impl ScheduledTask {
    /// Create new scheduled task
    fn new(task: WarmupTask, priority_score: u64, sequence_number: u64) -> Self {
        Self {
            task,
            scheduled_time: Instant::now(),
            priority_score,
            sequence_number,
        }
    }

    /// Calculate delay until execution
    fn delay(&self) -> Duration {
        self.scheduled_time.saturating_duration_since(Instant::now())
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority_score == other.priority_score && self.sequence_number == other.sequence_number
    }
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier sequence number
        other.priority_score.cmp(&self.priority_score)
            .then_with(|| self.sequence_number.cmp(&other.sequence_number))
    }
}

/// Currently running warmup task
#[derive(Debug, Clone)]
struct RunningTask {
    /// Task being executed
    task: WarmupTask,
    /// Start time
    start_time: Instant,
    /// Allocated resources
    allocated_resources: ResourceRequirements,
}

/// Resource allocation tracker
#[derive(Debug, Clone)]
struct ResourceAllocation {
    /// Currently allocated memory (MB)
    allocated_memory_mb: u64,
    /// Currently allocated CPU percentage
    allocated_cpu_percent: f64,
    /// Currently allocated network bandwidth (Mbps)
    allocated_network_mbps: f64,
    /// Currently allocated storage (MB)
    allocated_storage_mb: u64,
    /// Maximum concurrent tasks
    max_concurrent_tasks: usize,
    /// Currently running task count
    running_task_count: usize,
}

impl ResourceAllocation {
    /// Create new resource allocation tracker
    fn new(config: &WarmupConfig) -> Self {
        Self {
            allocated_memory_mb: 0,
            allocated_cpu_percent: 0.0,
            allocated_network_mbps: 0.0,
            allocated_storage_mb: 0,
            max_concurrent_tasks: 5, // Configurable
            running_task_count: 0,
        }
    }

    /// Check if resources are available for a task
    fn can_allocate(&self, requirements: &ResourceRequirements, config: &WarmupConfig) -> bool {
        // Check memory
        if self.allocated_memory_mb + requirements.memory_mb > config.max_memory_mb {
            return false;
        }

        // Check CPU
        if self.allocated_cpu_percent + requirements.cpu_percent > config.max_cpu_percent {
            return false;
        }

        // Check concurrent task limit
        if self.running_task_count >= self.max_concurrent_tasks {
            return false;
        }

        // Check storage
        if self.allocated_storage_mb + requirements.storage_mb > 1024 { // 1GB limit
            return false;
        }

        true
    }

    /// Allocate resources for a task
    fn allocate(&mut self, requirements: &ResourceRequirements) {
        self.allocated_memory_mb += requirements.memory_mb;
        self.allocated_cpu_percent += requirements.cpu_percent;
        self.allocated_network_mbps += requirements.network_bandwidth_mbps.unwrap_or(0.0);
        self.allocated_storage_mb += requirements.storage_mb;
        self.running_task_count += 1;
    }

    /// Deallocate resources from a completed task
    fn deallocate(&mut self, requirements: &ResourceRequirements) {
        self.allocated_memory_mb = self.allocated_memory_mb.saturating_sub(requirements.memory_mb);
        self.allocated_cpu_percent = self.allocated_cpu_percent - requirements.cpu_percent;
        self.allocated_network_mbps = self.allocated_network_mbps - requirements.network_bandwidth_mbps.unwrap_or(0.0);
        self.allocated_storage_mb = self.allocated_storage_mb.saturating_sub(requirements.storage_mb);
        self.running_task_count = self.running_task_count.saturating_sub(1);
    }
}

/// Scheduler performance metrics
#[derive(Debug, Clone)]
struct SchedulerMetrics {
    /// Total tasks scheduled
    total_scheduled: u64,
    /// Total tasks completed
    total_completed: u64,
    /// Total tasks failed
    total_failed: u64,
    /// Average scheduling delay
    avg_scheduling_delay_ms: f64,
    /// Resource utilization percentage
    resource_utilization_percent: f64,
    /// Task completion rate
    completion_rate: f64,
    /// Last updated timestamp
    last_updated: Instant,
}

impl SchedulerMetrics {
    /// Create new metrics tracker
    fn new() -> Self {
        Self {
            total_scheduled: 0,
            total_completed: 0,
            total_failed: 0,
            avg_scheduling_delay_ms: 0.0,
            resource_utilization_percent: 0.0,
            completion_rate: 0.0,
            last_updated: Instant::now(),
        }
    }

    /// Record task completion
    fn record_completion(&mut self, scheduling_delay_ms: f64) {
        self.total_completed += 1;
        self.update_metrics(scheduling_delay_ms);
    }

    /// Record task failure
    fn record_failure(&mut self) {
        self.total_failed += 1;
        self.update_metrics(0.0);
    }

    /// Record task scheduling
    fn record_scheduled(&mut self) {
        self.total_scheduled += 1;
    }

    /// Update derived metrics
    fn update_metrics(&mut self, delay_ms: f64) {
        // Update average delay using exponential moving average
        let alpha = 0.1;
        self.avg_scheduling_delay_ms = alpha * delay_ms + (1.0 - alpha) * self.avg_scheduling_delay_ms;

        // Update completion rate
        let total_processed = self.total_completed + self.total_failed;
        if total_processed > 0 {
            self.completion_rate = self.total_completed as f64 / total_processed as f64;
        }

        self.last_updated = Instant::now();
    }
}

impl WarmupScheduler {
    /// Create new warmup scheduler
    pub async fn new(config: WarmupConfig) -> Result<Self> {
        let resource_allocation = ResourceAllocation::new(&config);

        let scheduler = Self {
            config: Arc::new(RwLock::new(config)),
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            resource_allocation: Arc::new(RwLock::new(resource_allocation)),
            metrics: Arc::new(RwLock::new(SchedulerMetrics::new())),
            background_scheduler: Arc::new(RwLock::new(None)),
        };

        // Start background scheduler
        scheduler.start_background_scheduler().await?;

        Ok(scheduler)
    }

    /// Schedule warmup operations based on predictions
    pub async fn schedule_warmup(
        &self,
        predictions: &[ModelPrediction],
        available_resources: &ResourceAvailability,
    ) -> Result<WarmupSchedule> {
        let config = self.config.read().await;

        // Create warmup tasks from predictions
        let mut tasks = Vec::new();
        let mut sequence_counter = 0u64;

        for prediction in predictions {
            if prediction.confidence_score >= config.prediction_threshold {
                let task = self.create_warmup_task(prediction, &config).await?;
                tasks.push(task);
                sequence_counter += 1;
            }
        }

        // Resolve dependencies
        self.resolve_dependencies(&mut tasks).await?;

        // Sort tasks by priority and dependencies
        self.prioritize_tasks(&mut tasks).await?;

        // Schedule tasks considering resources and timing
        let scheduled_tasks = self.schedule_tasks_with_resources(tasks, available_resources).await?;

        // Create schedule
        let schedule = WarmupSchedule {
            tasks: scheduled_tasks,
            total_estimated_time: self.calculate_total_time(&tasks).await,
            resource_requirements: self.aggregate_resource_requirements(&tasks).await,
            priority: RequestPriority::Medium, // Can be made configurable
        };

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_scheduled();
        }

        Ok(schedule)
    }

    /// Get next task ready for execution
    pub async fn get_next_task(&self) -> Result<Option<WarmupTask>> {
        let mut queue = self.task_queue.write().await;
        let mut allocation = self.resource_allocation.write().await;
        let config = self.config.read().await;

        // Find next task that can be executed
        while let Some(scheduled_task) = queue.peek() {
            // Check if task can be allocated resources
            if allocation.can_allocate(&scheduled_task.task.resource_requirements, &config) {
                // Check if dependencies are satisfied
                if self.dependencies_satisfied(&scheduled_task.task).await? {
                    // Remove from queue and allocate resources
                    let task = queue.pop().unwrap().task;
                    allocation.allocate(&task.resource_requirements);

                    // Track as running
                    let running_task = RunningTask {
                        task: task.clone(),
                        start_time: Instant::now(),
                        allocated_resources: task.resource_requirements.clone(),
                    };

                    let mut running = self.running_tasks.write().await;
                    running.insert(task.model_id, running_task);

                    return Ok(Some(task));
                }
            }

            // If we can't execute this task, check if it's ready
            if scheduled_task.delay() > Duration::from_secs(1) {
                // Task not ready yet
                return Ok(None);
            }

            // Remove task that can't be executed (expired or resource constrained)
            queue.pop();
        }

        Ok(None)
    }

    /// Mark task as completed
    pub async fn complete_task(&self, model_id: &ModelId, success: bool) -> Result<()> {
        let mut running = self.running_tasks.write().await;
        let mut allocation = self.resource_allocation.write().await;

        if let Some(running_task) = running.remove(model_id) {
            // Deallocate resources
            allocation.deallocate(&running_task.allocated_resources);

            // Update metrics
            let scheduling_delay = running_task.start_time.elapsed().as_millis() as f64;
            let mut metrics = self.metrics.write().await;

            if success {
                metrics.record_completion(scheduling_delay);
            } else {
                metrics.record_failure();
            }
        }

        Ok(())
    }

    /// Get scheduler metrics
    pub async fn get_metrics(&self) -> SchedulerMetrics {
        self.metrics.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: WarmupConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    /// Create warmup task from prediction
    async fn create_warmup_task(&self, prediction: &ModelPrediction, config: &WarmupConfig) -> Result<WarmupTask> {
        let resource_requirements = self.estimate_resource_requirements(prediction, config).await;

        let task = WarmupTask {
            model_id: prediction.model_id,
            priority: self.map_prediction_to_priority(prediction),
            estimated_time: self.estimate_warmup_time(prediction),
            resource_requirements,
            dependencies: Vec::new(), // Will be filled by resolve_dependencies
            deadline: Some(Instant::now() + prediction.time_until_needed),
        };

        Ok(task)
    }

    /// Map prediction confidence to task priority
    fn map_prediction_to_priority(&self, prediction: &ModelPrediction) -> RequestPriority {
        if prediction.confidence_score > 0.9 && prediction.usage_probability > 0.8 {
            RequestPriority::Critical
        } else if prediction.confidence_score > 0.7 && prediction.usage_probability > 0.6 {
            RequestPriority::High
        } else if prediction.confidence_score > 0.5 && prediction.usage_probability > 0.4 {
            RequestPriority::Medium
        } else {
            RequestPriority::Low
        }
    }

    /// Estimate warmup time based on prediction
    fn estimate_warmup_time(&self, prediction: &ModelPrediction) -> Duration {
        // Base warmup time (simplified)
        let base_time_ms = 1000.0; // 1 second base

        // Adjust based on complexity (simplified estimation)
        let complexity_factor = match prediction.confidence_score {
            x if x > 0.8 => 1.5,  // High confidence = potentially complex model
            x if x > 0.6 => 1.0,  // Medium confidence = standard complexity
            _ => 0.7,             // Low confidence = simple model
        };

        Duration::from_millis((base_time_ms * complexity_factor) as u64)
    }

    /// Estimate resource requirements for a prediction
    async fn estimate_resource_requirements(&self, prediction: &ModelPrediction, config: &WarmupConfig) -> ResourceRequirements {
        // Simplified resource estimation based on prediction confidence
        let memory_mb = (prediction.confidence_score * 200.0) as u64; // 0-200MB based on confidence
        let cpu_percent = prediction.confidence_score * 10.0; // 0-10% CPU
        let network_mbps = if prediction.confidence_score > 0.7 { Some(5.0) } else { None };
        let storage_mb = (prediction.usage_probability * 50.0) as u64; // 0-50MB based on usage probability

        // Cap at configuration limits
        ResourceRequirements {
            memory_mb: memory_mb.min(config.max_memory_mb / 4), // Max 25% of total memory
            cpu_percent: cpu_percent.min(config.max_cpu_percent / 4.0), // Max 25% of CPU
            network_bandwidth_mbps: network_mbps,
            storage_mb,
        }
    }

    /// Resolve task dependencies
    async fn resolve_dependencies(&self, tasks: &mut [WarmupTask]) -> Result<()> {
        // For now, implement simple dependency resolution
        // In practice, this would analyze model dependencies from metadata

        let mut dependencies = self.dependencies.write().await;

        // Example: Some models depend on base models
        for task in tasks.iter_mut() {
            let mut task_deps = HashSet::new();

            // Add dependencies based on model type (simplified)
            if task.model_id.to_string().contains("large") {
                // Large models might depend on base models
                for other_task in tasks.iter() {
                    if other_task.model_id.to_string().contains("base") && other_task.model_id != task.model_id {
                        task_deps.insert(other_task.model_id);
                    }
                }
            }

            task.dependencies = task_deps.into_iter().collect();
            dependencies.insert(task.model_id, task_deps);
        }

        Ok(())
    }

    /// Prioritize tasks based on priority, dependencies, and timing
    async fn prioritize_tasks(&self, tasks: &mut [WarmupTask]) -> Result<()> {
        // Sort by priority, then by dependencies, then by deadline
        tasks.sort_by(|a, b| {
            // First by priority (higher first)
            let priority_cmp = b.priority.cmp(&a.priority);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }

            // Then by dependency count (fewer dependencies first)
            let dep_cmp = a.dependencies.len().cmp(&b.dependencies.len());
            if dep_cmp != std::cmp::Ordering::Equal {
                return dep_cmp;
            }

            // Finally by deadline (earlier first)
            match (a.deadline, b.deadline) {
                (Some(da), Some(db)) => da.cmp(&db),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        Ok(())
    }

    /// Schedule tasks considering resource constraints and timing
    async fn schedule_tasks_with_resources(
        &self,
        tasks: Vec<WarmupTask>,
        available_resources: &ResourceAvailability,
    ) -> Result<Vec<WarmupTask>> {
        let mut scheduled_tasks = Vec::new();
        let mut scheduled_time = Instant::now();

        for task in tasks {
            // Check if we can schedule this task
            if self.can_schedule_task(&task, &scheduled_tasks, available_resources).await? {
                // Update scheduled time if needed
                if let Some(deadline) = task.deadline {
                    if scheduled_time + task.estimated_time > deadline {
                        // Adjust scheduling to meet deadline
                        scheduled_time = deadline - task.estimated_time;
                    }
                }

                scheduled_tasks.push(task);
            }
        }

        Ok(scheduled_tasks)
    }

    /// Check if a task can be scheduled given current constraints
    async fn can_schedule_task(
        &self,
        task: &WarmupTask,
        scheduled_tasks: &[WarmupTask],
        available_resources: &ResourceAvailability,
    ) -> Result<bool> {
        let config = self.config.read().await;

        // Check resource availability
        if !self.resource_allocation.read().await.can_allocate(&task.resource_requirements, &config) {
            return Ok(false);
        }

        // Check if dependencies are already scheduled
        for dep in &task.dependencies {
            if !scheduled_tasks.iter().any(|t| t.model_id == *dep) {
                return Ok(false);
            }
        }

        // Check deadline
        if let Some(deadline) = task.deadline {
            if Instant::now() + task.estimated_time > deadline {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if task dependencies are satisfied
    async fn dependencies_satisfied(&self, task: &WarmupTask) -> Result<bool> {
        let running = self.running_tasks.read().await;

        for dep in &task.dependencies {
            if !running.contains_key(dep) {
                // Dependency not currently running/completed
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Calculate total estimated time for all tasks
    async fn calculate_total_time(&self, tasks: &[WarmupTask]) -> Duration {
        let mut total_time = Duration::from_millis(0);
        let mut max_parallel_time = Duration::from_millis(0);

        // Simplified: assume some tasks can run in parallel
        let max_concurrent = 3; // Configurable
        let mut concurrent_tasks = 0;
        let mut current_parallel_time = Duration::from_millis(0);

        for task in tasks {
            if concurrent_tasks < max_concurrent {
                current_parallel_time = current_parallel_time.max(task.estimated_time);
                concurrent_tasks += 1;
            } else {
                total_time += current_parallel_time;
                current_parallel_time = task.estimated_time;
                concurrent_tasks = 1;
            }
        }

        total_time += current_parallel_time;
        total_time
    }

    /// Aggregate resource requirements from all tasks
    async fn aggregate_resource_requirements(&self, tasks: &[WarmupTask]) -> ResourceRequirements {
        let mut total_memory = 0u64;
        let mut total_cpu = 0.0f64;
        let mut max_network = None;
        let mut total_storage = 0u64;

        for task in tasks {
            total_memory += task.resource_requirements.memory_mb;
            total_cpu += task.resource_requirements.cpu_percent;
            total_storage += task.resource_requirements.storage_mb;

            if let Some(network) = task.resource_requirements.network_bandwidth_mbps {
                max_network = Some(max_network.unwrap_or(0.0).max(network));
            }
        }

        ResourceRequirements {
            memory_mb: total_memory,
            cpu_percent: total_cpu,
            network_bandwidth_mbps: max_network,
            storage_mb: total_storage,
        }
    }

    /// Start background scheduler task
    async fn start_background_scheduler(&self) -> Result<()> {
        let task_queue = Arc::clone(&self.task_queue);
        let running_tasks = Arc::clone(&self.running_tasks);
        let resource_allocation = Arc::clone(&self.resource_allocation);
        let config = Arc::clone(&self.config);

        let handle = tokio::spawn(async move {
            loop {
                // Check for tasks ready to execute
                let task_result = {
                    let config_read = config.read().await;
                    timeout(Duration::from_millis(100), async {
                        // In a real implementation, this would execute ready tasks
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }).await
                };

                match task_result {
                    Ok(_) => {
                        // Check if we need to clean up completed tasks
                        // This would be done by monitoring task completion
                    }
                    Err(_) => {
                        // Timeout - continue loop
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        let mut scheduler = self.background_scheduler.write().await;
        *scheduler = Some(handle);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModelPrediction, ResourceAvailability};

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = WarmupConfig::default();
        let scheduler = WarmupScheduler::new(config).await.unwrap();

        let metrics = scheduler.get_metrics().await;
        assert_eq!(metrics.total_scheduled, 0);
    }

    #[tokio::test]
    async fn test_task_creation() {
        let config = WarmupConfig::default();
        let scheduler = WarmupScheduler::new(config).await.unwrap();

        let prediction = ModelPrediction {
            model_id: ModelId::new(),
            confidence_score: 0.8,
            usage_probability: 0.7,
            time_until_needed: Duration::from_secs(60),
            reasoning: vec!["Test prediction".to_string()],
        };

        let task = scheduler.create_warmup_task(&prediction, &config).await.unwrap();
        assert_eq!(task.model_id, prediction.model_id);
        assert_eq!(task.priority, RequestPriority::High);
    }

    #[tokio::test]
    async fn test_resource_allocation() {
        let config = WarmupConfig::default();
        let allocation = ResourceAllocation::new(&config);

        let requirements = ResourceRequirements {
            memory_mb: 100,
            cpu_percent: 5.0,
            network_bandwidth_mbps: Some(2.0),
            storage_mb: 10,
        };

        assert!(allocation.can_allocate(&requirements, &config));

        let mut allocation_mut = allocation.clone();
        allocation_mut.allocate(&requirements);

        assert_eq!(allocation_mut.allocated_memory_mb, 100);
        assert_eq!(allocation_mut.allocated_cpu_percent, 5.0);
        assert_eq!(allocation_mut.running_task_count, 1);

        allocation_mut.deallocate(&requirements);
        assert_eq!(allocation_mut.running_task_count, 0);
    }

    #[tokio::test]
    async fn test_task_prioritization() {
        let config = WarmupConfig::default();
        let scheduler = WarmupScheduler::new(config).await.unwrap();

        let mut tasks = vec![
            WarmupTask {
                model_id: ModelId::new(),
                priority: RequestPriority::Low,
                estimated_time: Duration::from_secs(10),
                resource_requirements: ResourceRequirements::default(),
                dependencies: vec![],
                deadline: None,
            },
            WarmupTask {
                model_id: ModelId::new(),
                priority: RequestPriority::High,
                estimated_time: Duration::from_secs(5),
                resource_requirements: ResourceRequirements::default(),
                dependencies: vec![],
                deadline: Some(Instant::now() + Duration::from_secs(30)),
            },
        ];

        scheduler.prioritize_tasks(&mut tasks).await.unwrap();

        // High priority task should come first
        assert_eq!(tasks[0].priority, RequestPriority::High);
        assert_eq!(tasks[1].priority, RequestPriority::Low);
    }
}