//! Priority-based queue for warmup tasks

use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{Result, WarmupError};
use crate::types::{ModelId, RequestPriority, WarmupConfig, WarmupTask};

#[derive(Debug)]
pub struct WarmupQueue {
    queue: Arc<RwLock<BinaryHeap<PrioritizedTask>>>,
    config: Arc<RwLock<WarmupConfig>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PrioritizedTask {
    task: WarmupTask,
    priority_score: u64,
}

impl WarmupQueue {
    pub async fn new(config: WarmupConfig) -> Result<Self> {
        Ok(Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            config: Arc::new(RwLock::new(config)),
        })
    }

    pub async fn enqueue_task(&self, task: WarmupTask) -> Result<()> {
        let config = self.config.read().await;
        let queue_size = self.queue.read().await.len();

        if queue_size >= config.max_queue_size {
            return Err(WarmupError::Queue {
                message: "Queue capacity exceeded".to_string(),
            });
        }

        let priority_score = self.calculate_priority(&task);
        let prioritized_task = PrioritizedTask { task, priority_score };

        let mut queue = self.queue.write().await;
        queue.push(prioritized_task);
        Ok(())
    }

    pub async fn dequeue_task(&self) -> Result<Option<WarmupTask>> {
        let mut queue = self.queue.write().await;
        Ok(queue.pop().map(|pt| pt.task))
    }

    pub async fn queue_size(&self) -> usize {
        self.queue.read().await.len()
    }

    fn calculate_priority(&self, task: &WarmupTask) -> u64 {
        match task.priority {
            RequestPriority::Critical => 100,
            RequestPriority::High => 75,
            RequestPriority::Medium => 50,
            RequestPriority::Low => 25,
        }
    }

    pub async fn update_config(&self, config: WarmupConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }
}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority_score.cmp(&self.priority_score)
    }
}