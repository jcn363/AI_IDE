//! Task trait and implementations for work-stealing scheduler

use std::fmt;

use async_trait::async_trait;

/// Task priority levels for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low      = 0,
    Normal   = 1,
    High     = 2,
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Generic task trait that can be executed by the work-stealing scheduler
#[async_trait]
pub trait Task: Send + Sync + fmt::Debug {
    /// Task type identifier for metrics and debugging
    fn task_type(&self) -> &'static str;

    /// Task priority for scheduling decisions
    fn priority(&self) -> TaskPriority {
        TaskPriority::Normal
    }

    /// Execute the task asynchronously
    async fn execute(&self) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;

    /// Estimated execution time in milliseconds (for scheduling hints)
    fn estimated_duration_ms(&self) -> Option<u64> {
        None
    }

    /// Task identifier for tracking
    fn task_id(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}", self).hash(&mut hasher);
        format!("task-{}", hasher.finish())
    }

    /// Memory requirements hint (in MB)
    fn memory_hint_mb(&self) -> Option<usize> {
        None
    }
}

/// Boxed task for dynamic dispatch
pub type BoxedTask = Box<dyn Task>;

/// CPU-bound task wrapper
pub struct CpuBoundTask<T, F, Fut>
where
    T: Send + Sync + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    pub task_type:          &'static str,
    pub priority:           TaskPriority,
    pub data:               T,
    pub processor:          F,
    pub estimated_duration: Option<u64>,
    pub memory_hint:        Option<usize>,
}

#[async_trait]
impl<T, F, Fut> Task for CpuBoundTask<T, F, Fut>
where
    T: Send + Sync + fmt::Debug + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    fn task_type(&self) -> &'static str {
        self.task_type
    }

    fn priority(&self) -> TaskPriority {
        self.priority
    }

    async fn execute(&self) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        (self.processor)(self.data.clone()).await
    }

    fn estimated_duration_ms(&self) -> Option<u64> {
        self.estimated_duration
    }

    fn memory_hint_mb(&self) -> Option<usize> {
        self.memory_hint
    }
}

impl<T, F, Fut> fmt::Debug for CpuBoundTask<T, F, Fut>
where
    T: Send + Sync + fmt::Debug + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CpuBoundTask")
            .field("task_type", &self.task_type)
            .field("priority", &self.priority)
            .field("data", &self.data)
            .field("estimated_duration", &self.estimated_duration)
            .field("memory_hint", &self.memory_hint)
            .finish()
    }
}

/// IO-bound task wrapper for operations that may block
pub struct IoBoundTask<T, F, Fut>
where
    T: Send + Sync + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    pub task_type:  &'static str,
    pub priority:   TaskPriority,
    pub data:       T,
    pub processor:  F,
    pub timeout_ms: Option<u64>,
}

#[async_trait]
impl<T, F, Fut> Task for IoBoundTask<T, F, Fut>
where
    T: Send + Sync + fmt::Debug + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    fn task_type(&self) -> &'static str {
        self.task_type
    }

    fn priority(&self) -> TaskPriority {
        self.priority
    }

    async fn execute(&self) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let result = if let Some(timeout) = self.timeout_ms {
            tokio::time::timeout(
                std::time::Duration::from_millis(timeout),
                (self.processor)(self.data.clone()),
            )
            .await
            .map_err(|_| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Task execution timed out",
                ))
            })?
        } else {
            (self.processor)(self.data.clone()).await
        };

        result
    }
}

impl<T, F, Fut> fmt::Debug for IoBoundTask<T, F, Fut>
where
    T: Send + Sync + fmt::Debug + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IoBoundTask")
            .field("task_type", &self.task_type)
            .field("priority", &self.priority)
            .field("data", &self.data)
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

/// Task creation helpers
pub mod helpers {
    use super::*;

    /// Create a CPU-bound task
    pub fn cpu_task<T, F, Fut>(task_type: &'static str, data: T, processor: F) -> CpuBoundTask<T, F, Fut>
    where
        T: Send + Sync + fmt::Debug + 'static,
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static,
    {
        CpuBoundTask {
            task_type,
            priority: TaskPriority::Normal,
            data,
            processor,
            estimated_duration: None,
            memory_hint: None,
        }
    }

    /// Create an IO-bound task
    pub fn io_task<T, F, Fut>(task_type: &'static str, data: T, processor: F) -> IoBoundTask<T, F, Fut>
    where
        T: Send + Sync + fmt::Debug + 'static,
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static,
    {
        IoBoundTask {
            task_type,
            priority: TaskPriority::Normal,
            data,
            processor,
            timeout_ms: None,
        }
    }
}
