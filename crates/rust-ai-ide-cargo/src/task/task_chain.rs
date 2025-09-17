//! Task chaining functionality for executing multiple Cargo commands

use std::sync::Arc;

use anyhow::{anyhow, Result};
use futures::future::join_all;
use log::error;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{CargoTask, CommandError, TaskResult};
// Re-export ExecutionStrategy from the types module
#[doc(inline)]
pub use crate::task::ExecutionStrategy;
use crate::task::TaskStatus;

/// Represents a chain of tasks to be executed
pub struct TaskChain {
    id: String,
    tasks: Vec<CargoTask>,
    strategy: ExecutionStrategy,
    results: Arc<RwLock<Vec<TaskResult>>>,
    status: Arc<RwLock<TaskStatus>>,
    current_task_index: Arc<RwLock<usize>>,
}

impl TaskChain {
    /// Create a new task chain with the given execution strategy
    pub fn new(strategy: ExecutionStrategy) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            tasks: Vec::new(),
            strategy,
            results: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(TaskStatus::Pending)),
            current_task_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the ID of the task chain
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the current status of the task chain
    pub async fn status(&self) -> TaskStatus {
        self.status.read().await.clone()
    }

    /// Get the results of completed tasks
    pub async fn results(&self) -> Vec<TaskResult> {
        self.results.read().await.clone()
    }

    /// Add a task to the chain
    pub fn add_task(&mut self, task: CargoTask) -> &mut Self {
        self.tasks.push(task);
        self
    }

    /// Add multiple tasks to the chain
    pub fn add_tasks<I>(&mut self, tasks: I) -> &mut Self
    where
        I: IntoIterator<Item = CargoTask>,
    {
        self.tasks.extend(tasks);
        self
    }

    /// Execute the task chain
    pub async fn execute(&self) -> Result<()> {
        *self.status.write().await = TaskStatus::Running;
        *self.results.write().await = Vec::with_capacity(self.tasks.len());
        *self.current_task_index.write().await = 0;

        let result = match self.strategy {
            ExecutionStrategy::Sequential => self.execute_sequential().await,
            ExecutionStrategy::Parallel => self.execute_parallel().await,
            ExecutionStrategy::StopOnFailure => self.execute_stop_on_failure().await,
        };

        match &result {
            Ok(_) => *self.status.write().await = TaskStatus::Completed,
            Err(e) => {
                error!("Task chain failed: {}", e);
                *self.status.write().await = TaskStatus::Failed;
            }
        }

        result
    }

    /// Execute tasks sequentially
    async fn execute_sequential(&self) -> Result<()> {
        let mut results = Vec::with_capacity(self.tasks.len());

        for (index, task) in self.tasks.iter().enumerate() {
            *self.current_task_index.write().await = index;

            let task = task.clone();
            let result = self.execute_and_convert_result(task).await;

            if !result.success {
                error!("Task {} failed, stopping chain", index);
                return Err(anyhow!("Task {} failed", index));
            }

            results.push(result);
            *self.results.write().await = results.clone();
        }

        *self.current_task_index.write().await = self.tasks.len();
        Ok(())
    }

    /// Execute tasks in parallel
    async fn execute_parallel(&self) -> Result<()> {
        let tasks = self.tasks.clone();
        let task_count = tasks.len();

        // Create a future for each task
        let task_futures = tasks.into_iter().map(|task| {
            let task_clone = task.clone();
            async move { self.execute_and_convert_result(task_clone).await }
        });

        // Execute all tasks in parallel
        let results = join_all(task_futures).await;

        // Process results
        let mut success = true;
        let mut successful_results = Vec::with_capacity(task_count);

        for result in results {
            if result.success {
                successful_results.push(result);
            } else {
                success = false;
            }
        }

        // Update results with successful tasks
        *self.results.write().await = successful_results;
        *self.current_task_index.write().await = task_count;

        if !success {
            return Err(anyhow!("One or more parallel tasks failed"));
        }

        Ok(())
    }

    /// Execute tasks, stopping on first failure
    async fn execute_stop_on_failure(&self) -> Result<()> {
        let mut results = Vec::with_capacity(self.tasks.len());

        for (index, task) in self.tasks.iter().enumerate() {
            *self.current_task_index.write().await = index;

            let task = task.clone();
            let result = self.execute_and_convert_result(task).await;

            if result.success {
                results.push(result);
                *self.results.write().await = results.clone();
            } else {
                error!("Task {} failed, stopping chain", index);
                return Err(anyhow!("Task {} failed", index));
            }
        }

        *self.current_task_index.write().await = self.tasks.len();
        Ok(())
    }

    /// Execute a single task and return its result
    async fn execute_single_task(&self, task: CargoTask) -> Result<TaskResult, CommandError> {
        let mut executor = super::CommandExecutor::new(task);
        executor.execute().await
    }

    /// Execute a single task and convert the result to a TaskResult
    async fn execute_and_convert_result(&self, task: CargoTask) -> TaskResult {
        TaskResult::from_command_result(self.execute_single_task(task).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(command: &str) -> CargoTask {
        CargoTask {
            command: command.to_string(),
            args: vec![],
            working_dir: std::env::current_dir().unwrap(),
            release: false,
            env: vec![],
        }
    }

    #[tokio::test]
    async fn test_sequential_execution() {
        let mut chain = TaskChain::new(ExecutionStrategy::Sequential);
        chain
            .add_task(create_test_task("version"))
            .add_task(create_test_task("--version"));

        assert!(chain.execute().await.is_ok());
        assert_eq!(chain.status().await, TaskStatus::Completed);
        assert_eq!(chain.results().await.len(), 2);
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let mut chain = TaskChain::new(ExecutionStrategy::Parallel);
        chain
            .add_task(create_test_task("version"))
            .add_task(create_test_task("--version"));

        assert!(chain.execute().await.is_ok());
        assert_eq!(chain.status().await, TaskStatus::Completed);
        assert_eq!(chain.results().await.len(), 2);
    }

    #[tokio::test]
    async fn test_stop_on_failure() {
        let mut chain = TaskChain::new(ExecutionStrategy::StopOnFailure);
        chain
            .add_task(create_test_task("version"))
            .add_task(create_test_task("invalid-command"));

        // Should fail on the second command
        assert!(chain.execute().await.is_err());
        assert_eq!(chain.status().await, TaskStatus::Failed);
        assert_eq!(chain.results().await.len(), 1);
    }
}
