//! Background task monitoring for Cargo commands

use super::{CargoTask, CommandExecutor, CommandResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Represents the status of a background task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Information about a running or completed task
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub task: CargoTask,
    pub status: TaskStatus,
    pub progress: Option<f32>,
    pub result: Option<CommandResult>,
}

/// Manages background tasks and provides status updates
// Prepared for future task monitoring functionality integration
pub struct TaskMonitor {
    tasks: Arc<RwLock<HashMap<String, TaskInfo>>>,
    task_counter: AtomicU64,
}

impl Default for TaskMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskMonitor {
    /// Create a new task monitor
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_counter: AtomicU64::new(0),
        }
    }

    /// Start a new background task
    pub async fn start_task(&self, task: CargoTask) -> String {
        let task_id = Uuid::new_v4().to_string();
        let task_info = TaskInfo {
            id: task_id.clone(),
            task: task.clone(),
            status: TaskStatus::Pending,
            progress: Some(0.0),
            result: None,
        };

        // Add to task list
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task_info);
        }

        // Clone the task ID and task list for the background task
        let task_id_clone = task_id.clone();
        let tasks_clone = self.tasks.clone();

        // Spawn the task
        tokio::spawn(async move {
            let mut executor = CommandExecutor::new(task);

            // Update status to running
            {
                let mut tasks = tasks_clone.write().await;
                if let Some(task_info) = tasks.get_mut(&task_id_clone) {
                    task_info.status = TaskStatus::Running;
                    task_info.progress = Some(0.2); // 20% - task started
                }
            }

            // Execute the command
            let result = executor.execute().await;

            // Update task status based on result
            let status = match &result {
                Ok(_) => TaskStatus::Completed,
                Err(_) => TaskStatus::Failed,
            };

            // Update task info with result
            {
                let mut tasks = tasks_clone.write().await;
                if let Some(task_info) = tasks.get_mut(&task_id_clone) {
                    task_info.status = status;
                    task_info.progress = Some(1.0);
                    task_info.result = Some(result);
                }
            }
        });

        task_id
    }

    /// Get the status of a task
    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskInfo> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// Get all tasks
    pub async fn get_all_tasks(&self) -> Vec<TaskInfo> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// Get recent tasks
    pub async fn get_recent_tasks(&self, limit: usize) -> Vec<TaskInfo> {
        let tasks = self.tasks.read().await;
        let mut task_list: Vec<_> = tasks.values().cloned().collect();
        task_list.sort_by(|a, b| a.id.cmp(&b.id)); // Simple sort by ID for now
        task_list.into_iter().take(limit).collect()
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task_info) = tasks.get_mut(task_id) {
            if task_info.status == TaskStatus::Running {
                // TODO: Implement actual task cancellation
                task_info.status = TaskStatus::Cancelled;
                task_info.progress = None;
                return true;
            }
        }
        false
    }

    /// Clean up completed tasks older than the specified duration
    pub async fn cleanup_old_tasks(&self, older_than: std::time::Duration) {
        let _now = std::time::SystemTime::now();
        let mut tasks = self.tasks.write().await;

        tasks.retain(|_, task_info| {
            match &task_info.result {
                Some(Ok(result)) => {
                    if let Some(end_time) = result.end_time {
                        match end_time.elapsed() {
                            Ok(elapsed) => return elapsed < older_than,
                            Err(_) => return false, // If time went backwards, remove the task
                        }
                    }
                    true // Keep tasks without end_time
                }
                Some(Err(_)) => false, // Remove failed tasks
                None => true,          // Keep pending tasks
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task() -> CargoTask {
        CargoTask {
            command: "version".to_string(),
            args: vec![],
            working_dir: std::env::current_dir().unwrap(),
            release: false,
            env: vec![],
        }
    }

    #[tokio::test]
    async fn test_task_monitor() {
        let monitor = TaskMonitor::new();
        let task = create_test_task();

        // Start a task
        let task_id = monitor.start_task(task).await;

        // Check task status
        let status = monitor.get_task_status(&task_id).await;
        assert!(status.is_some());

        // Give the task some time to complete
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Check final status
        let status = monitor.get_task_status(&task_id).await.unwrap();
        assert_eq!(status.status, TaskStatus::Completed);
    }
}
