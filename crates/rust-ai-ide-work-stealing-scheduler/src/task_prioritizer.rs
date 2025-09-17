/*!
 * Task Prioritizer for intelligent task scheduling
 *
 * This module provides priority-based task scheduling with deadline awareness,
 * dependency tracking, and resource-aware prioritization for optimal throughput.
 */

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Task priority levels with execution characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 0,    // Must execute immediately (system stability)
    High = 1,        // User-facing operations
    Normal = 2,      // Background analysis
    Low = 3,         // Maintenance tasks
    Idle = 4,        // Only when system is idle
}

/// Task metadata for prioritization
#[derive(Debug, Clone)]
pub struct TaskMetadata {
    pub task_id: String,
    pub priority: TaskPriority,
    pub deadline: Option<std::time::Instant>,
    pub dependencies: HashSet<String>,
    pub estimated_duration: std::time::Duration,
    pub resource_requirements: ResourceRequirements,
    pub submission_time: std::time::Instant,
    pub complexity_score: f64,
}

/// Resource requirements for task execution
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu_cores: usize,
    pub memory_mb: usize,
    pub io_intensity: f64, // 0.0 = CPU bound, 1.0 = I/O bound
}

/// Prioritized task with ordering
#[derive(Debug, Clone)]
pub struct PrioritizedTask {
    pub metadata: TaskMetadata,
    pub priority_score: f64,
}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority_score == other.priority_score
    }
}

impl Eq for PrioritizedTask {}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.priority_score.partial_cmp(&self.priority_score) // Reverse for max-heap
    }
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority_score.partial_cmp(&self.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.metadata.submission_time.cmp(&other.metadata.submission_time)) // FCFS for same priority
    }
}

/// Task prioritizer with advanced scheduling algorithms
pub struct TaskPrioritizer {
    /// Priority queue for tasks
    task_queue: Arc<RwLock<BinaryHeap<PrioritizedTask>>>,
    /// Task metadata storage
    task_metadata: Arc<RwLock<HashMap<String, TaskMetadata>>>,
    /// Dependency graph
    dependency_graph: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Completed tasks for dependency tracking
    completed_tasks: Arc<RwLock<HashSet<String>>>,
    /// Current system load factors
    system_load: Arc<RwLock<SystemLoad>>,
    /// Priority weights for different factors
    priority_weights: PriorityWeights,
}

#[derive(Debug, Clone)]
pub struct SystemLoad {
    pub cpu_utilization: f64,
    pub memory_pressure: f64,
    pub io_wait: f64,
    pub active_tasks: usize,
}

#[derive(Debug, Clone)]
pub struct PriorityWeights {
    pub base_priority_weight: f64,
    pub deadline_weight: f64,
    pub dependency_weight: f64,
    pub resource_weight: f64,
    pub age_weight: f64,
}

impl Default for PriorityWeights {
    fn default() -> Self {
        Self {
            base_priority_weight: 100.0,
            deadline_weight: 50.0,
            dependency_weight: 30.0,
            resource_weight: 20.0,
            age_weight: 10.0,
        }
    }
}

impl TaskPrioritizer {
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            task_metadata: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashSet::new())),
            system_load: Arc::new(RwLock::new(SystemLoad {
                cpu_utilization: 0.0,
                memory_pressure: 0.0,
                io_wait: 0.0,
                active_tasks: 0,
            })),
            priority_weights: PriorityWeights::default(),
        }
    }

    /// Submit a new task for prioritization
    pub async fn submit_task(&self, metadata: TaskMetadata) -> Result<(), String> {
        // Validate dependencies
        self.validate_dependencies(&metadata).await?;

        // Store metadata
        {
            let mut task_metadata = self.task_metadata.write().await;
            task_metadata.insert(metadata.task_id.clone(), metadata.clone());
        }

        // Update dependency graph
        {
            let mut dependency_graph = self.dependency_graph.write().await;
            for dep in &metadata.dependencies {
                dependency_graph.entry(dep.clone())
                    .or_insert_with(HashSet::new)
                    .insert(metadata.task_id.clone());
            }
        }

        // Create prioritized task
        let prioritized_task = self.create_prioritized_task(metadata).await;

        // Add to priority queue
        {
            let mut task_queue = self.task_queue.write().await;
            task_queue.push(prioritized_task);
        }

        Ok(())
    }

    /// Get the next highest priority task that can be executed
    pub async fn next_task(&self) -> Option<TaskMetadata> {
        let mut task_queue = self.task_queue.write().await;

        while let Some(prioritized_task) = task_queue.pop() {
            let task_id = prioritized_task.metadata.task_id.clone();

            // Check if dependencies are satisfied
            if self.dependencies_satisfied(&task_id).await {
                return Some(prioritized_task.metadata);
            } else {
                // Re-queue if dependencies not satisfied
                task_queue.push(prioritized_task);
                break; // Don't keep looping indefinitely
            }
        }

        None
    }

    /// Mark a task as completed
    pub async fn complete_task(&self, task_id: &str) {
        let mut completed_tasks = self.completed_tasks.write().await;
        completed_tasks.insert(task_id.to_string());

        // Update priority scores of dependent tasks
        self.update_dependent_priorities(task_id).await;
    }

    /// Update system load factors
    pub async fn update_system_load(&self, load: SystemLoad) {
        let mut system_load = self.system_load.write().await;
        *system_load = load;

        // Recalculate all priority scores with new load factors
        self.recalculate_all_priorities().await;
    }

    /// Create prioritized task with calculated score
    async fn create_prioritized_task(&self, metadata: TaskMetadata) -> PrioritizedTask {
        let priority_score = self.calculate_priority_score(&metadata).await;

        PrioritizedTask {
            metadata,
            priority_score,
        }
    }

    /// Calculate comprehensive priority score
    async fn calculate_priority_score(&self, metadata: &TaskMetadata) -> f64 {
        let system_load = self.system_load.read().await;
        let weights = &self.priority_weights;

        // Base priority score (higher number = higher priority)
        let base_score = weights.base_priority_weight * (10 - metadata.priority as i32) as f64;

        // Deadline factor
        let deadline_score = if let Some(deadline) = metadata.deadline {
            let time_remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let time_ratio = time_remaining.as_secs_f64() / metadata.estimated_duration.as_secs_f64();
            weights.deadline_weight * (1.0 / (1.0 + time_ratio)) // Urgency increases as deadline approaches
        } else {
            0.0
        };

        // Dependency factor
        let unsatisfied_deps = self.count_unsatisfied_dependencies(&metadata.task_id).await;
        let dependency_score = weights.dependency_weight * (metadata.dependencies.len() - unsatisfied_deps) as f64;

        // Resource availability factor
        let resource_score = self.calculate_resource_score(&metadata.resource_requirements, &system_load);

        // Age factor (favor older tasks slightly)
        let age_seconds = metadata.submission_time.elapsed().as_secs_f64();
        let age_score = weights.age_weight * (1.0 - (-age_seconds / 3600.0).exp()); // Diminishing returns

        // Complexity factor (prefer simpler tasks when system is loaded)
        let complexity_penalty = if system_load.cpu_utilization > 0.8 {
            metadata.complexity_score * 0.5
        } else {
            0.0
        };

        base_score + deadline_score + dependency_score + resource_score + age_score - complexity_penalty
    }

    /// Calculate resource availability score
    fn calculate_resource_score(&self, requirements: &ResourceRequirements, load: &SystemLoad) -> f64 {
        let cpu_score = if requirements.cpu_cores > 0 {
            (1.0 - load.cpu_utilization) * requirements.cpu_cores as f64
        } else {
            1.0
        };

        let memory_score = if requirements.memory_mb > 0 {
            (1.0 - load.memory_pressure) * (requirements.memory_mb as f64 / 1024.0).min(1.0)
        } else {
            1.0
        };

        let io_score = if requirements.io_intensity > 0.5 {
            (1.0 - load.io_wait) * requirements.io_intensity
        } else {
            1.0
        };

        (cpu_score + memory_score + io_score) / 3.0 * self.priority_weights.resource_weight
    }

    /// Validate task dependencies exist
    async fn validate_dependencies(&self, metadata: &TaskMetadata) -> Result<(), String> {
        let task_metadata = self.task_metadata.read().await;

        for dep in &metadata.dependencies {
            if !task_metadata.contains_key(dep) && !self.completed_tasks.read().await.contains(dep) {
                return Err(format!("Dependency {} does not exist", dep));
            }
        }

        Ok(())
    }

    /// Check if all dependencies of a task are satisfied
    async fn dependencies_satisfied(&self, task_id: &str) -> bool {
        let task_metadata = self.task_metadata.read().await;
        let completed_tasks = self.completed_tasks.read().await;

        if let Some(metadata) = task_metadata.get(task_id) {
            metadata.dependencies.iter().all(|dep| completed_tasks.contains(dep))
        } else {
            false
        }
    }

    /// Count unsatisfied dependencies
    async fn count_unsatisfied_dependencies(&self, task_id: &str) -> usize {
        let task_metadata = self.task_metadata.read().await;
        let completed_tasks = self.completed_tasks.read().await;

        if let Some(metadata) = task_metadata.get(task_id) {
            metadata.dependencies.iter()
                .filter(|dep| !completed_tasks.contains(*dep))
                .count()
        } else {
            0
        }
    }

    /// Update priority scores of tasks that depend on completed task
    async fn update_dependent_priorities(&self, completed_task_id: &str) {
        let dependency_graph = self.dependency_graph.read().await;
        let dependent_tasks = dependency_graph.get(completed_task_id);

        if let Some(dependents) = dependent_tasks {
            let mut task_queue = self.task_queue.write().await;
            let mut task_metadata = self.task_metadata.write().await;

            // Collect tasks that need updating
            let mut tasks_to_update = Vec::new();

            for dependent in dependents {
                if let Some(metadata) = task_metadata.get(dependent) {
                    tasks_to_update.push(metadata.clone());
                }
            }

            // Recalculate and re-queue updated tasks
            for metadata in tasks_to_update {
                let new_priority_score = self.calculate_priority_score(&metadata).await;
                let prioritized_task = PrioritizedTask {
                    metadata,
                    priority_score: new_priority_score,
                };
                task_queue.push(prioritized_task);
            }
        }
    }

    /// Recalculate all priority scores when system load changes
    async fn recalculate_all_priorities(&self) {
        let mut task_queue = self.task_queue.write().await;
        let task_metadata = self.task_metadata.read().await;

        // Clear and rebuild the priority queue with updated scores
        let mut updated_queue = BinaryHeap::new();

        // Process all current tasks
        while let Some(mut prioritized_task) = task_queue.pop() {
            prioritized_task.priority_score = self.calculate_priority_score(&prioritized_task.metadata).await;
            updated_queue.push(prioritized_task);
        }

        // Swap the queues
        *task_queue = updated_queue;
    }

    /// Get current queue statistics
    pub async fn queue_stats(&self) -> QueueStats {
        let task_queue = self.task_queue.read().await;
        let task_metadata = self.task_metadata.read().await;
        let completed_tasks = self.completed_tasks.read().await;

        let total_tasks = task_metadata.len();
        let queued_tasks = task_queue.len();
        let completed_count = completed_tasks.len();
        let blocked_tasks = total_tasks - queued_tasks - completed_count;

        QueueStats {
            total_tasks,
            queued_tasks,
            completed_tasks: completed_count,
            blocked_tasks,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total_tasks: usize,
    pub queued_tasks: usize,
    pub completed_tasks: usize,
    pub blocked_tasks: usize,
}

impl Default for TaskPrioritizer {
    fn default() -> Self {
        Self::new()
    }
}