//! Event system for background defragmentation

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use dashmap::DashMap;
use crate::algorithms::DefragmentationResult;
use crate::InfraResult;

/// Event bus for background defragmentation events
#[derive(Debug)]
pub struct EventBus {
    /// Event subscribers
    subscribers: Arc<RwLock<DashMap<String, Vec<Box<dyn EventSubscriber>>>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(DashMap::new())),
        }
    }

    /// Subscribe to events of a specific type
    pub async fn subscribe<T: EventSubscriber + 'static>(&self, event_type: &str, subscriber: T) {
        let mut subscribers = self.subscribers.write().await;
        subscribers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(subscriber));
    }

    /// Publish an event to all subscribers
    pub async fn publish(&self, event: DefragmentationEvent) -> InfraResult<()> {
        let event_type = event.event_type();
        let subscribers = self.subscribers.read().await;

        if let Some(subscribers_for_type) = subscribers.get(&event_type) {
            for subscriber in subscribers_for_type.iter() {
                if let Err(e) = subscriber.handle_event(&event).await {
                    tracing::error!("Event subscriber failed: {:?}", e);
                }
            }
        }

        Ok(())
    }

    /// Get subscriber count for a specific event type
    pub async fn subscriber_count(&self, event_type: &str) -> usize {
        let subscribers = self.subscribers.read().await;
        subscribers
            .get(event_type)
            .map(|subs| subs.len())
            .unwrap_or(0)
    }

    /// Clear all subscribers
    pub async fn clear_subscribers(&self) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.clear();
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for event subscribers
#[async_trait::async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handle an incoming event
    async fn handle_event(&self, event: &DefragmentationEvent) -> InfraResult<()>;
}

/// Events related to background defragmentation
#[derive(Debug, Clone)]
pub enum DefragmentationEvent {
    /// Coordinator started
    CoordinatorStarted {
        timestamp: Instant,
    },

    /// Coordinator stopped
    CoordinatorStopped {
        timestamp: Instant,
    },

    /// Defragmentation cycle started
    CycleStarted {
        timestamp: Instant,
        fragmentation_before: f64,
        memory_pressure: f64,
    },

    /// Defragmentation cycle completed
    CycleCompleted {
        timestamp: Instant,
        result: DefragmentationResult,
    },

    /// Defragmentation cycle failed
    CycleFailed {
        timestamp: Instant,
        error: String,
        fragmentation_before: f64,
    },

    /// Memory block allocated
    BlockAllocated {
        timestamp: Instant,
        block_id: uuid::Uuid,
        pool_id: String,
        size: usize,
    },

    /// Memory block freed
    BlockFreed {
        timestamp: Instant,
        block_id: uuid::Uuid,
        pool_id: String,
        size: usize,
    },

    /// Fragmentation threshold exceeded
    FragmentationThresholdExceeded {
        timestamp: Instant,
        current_fragmentation: f64,
        threshold: f64,
    },

    /// Performance guard triggered
    PerformanceGuardTriggered {
        timestamp: Instant,
        reason: String,
        cpu_usage: f64,
        memory_pressure: f64,
    },

    /// Algorithm selected for defragmentation
    AlgorithmSelected {
        timestamp: Instant,
        algorithm: String,
        block_count: usize,
        fragmentation_level: f64,
    },
}

impl DefragmentationEvent {
    /// Get the event type string
    pub fn event_type(&self) -> String {
        match self {
            DefragmentationEvent::CoordinatorStarted { .. } => "coordinator_started",
            DefragmentationEvent::CoordinatorStopped { .. } => "coordinator_stopped",
            DefragmentationEvent::CycleStarted { .. } => "cycle_started",
            DefragmentationEvent::CycleCompleted { .. } => "cycle_completed",
            DefragmentationEvent::CycleFailed { .. } => "cycle_failed",
            DefragmentationEvent::BlockAllocated { .. } => "block_allocated",
            DefragmentationEvent::BlockFreed { .. } => "block_freed",
            DefragmentationEvent::FragmentationThresholdExceeded { .. } => "fragmentation_threshold_exceeded",
            DefragmentationEvent::PerformanceGuardTriggered { .. } => "performance_guard_triggered",
            DefragmentationEvent::AlgorithmSelected { .. } => "algorithm_selected",
        }.to_string()
    }

    /// Convert event to JSON for serialization
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            DefragmentationEvent::CoordinatorStarted { timestamp } => {
                serde_json::json!({
                    "event_type": "coordinator_started",
                    "timestamp": timestamp.elapsed().as_millis(),
                })
            }
            DefragmentationEvent::CoordinatorStopped { timestamp } => {
                serde_json::json!({
                    "event_type": "coordinator_stopped",
                    "timestamp": timestamp.elapsed().as_millis(),
                })
            }
            DefragmentationEvent::CycleStarted { timestamp, fragmentation_before, memory_pressure } => {
                serde_json::json!({
                    "event_type": "cycle_started",
                    "timestamp": timestamp.elapsed().as_millis(),
                    "fragmentation_before": fragmentation_before,
                    "memory_pressure": memory_pressure,
                })
            }
            DefragmentationEvent::CycleCompleted { timestamp, result } => {
                serde_json::json!({
                    "event_type": "cycle_completed",
                    "timestamp": timestamp.elapsed().as_millis(),
                    "blocks_relocated": result.blocks_relocated,
                    "memory_freed": result.memory_freed,
                    "fragmentation_before": result.fragmentation_before,
                    "fragmentation_after": result.fragmentation_after,
                    "duration_ms": result.duration.as_millis(),
                    "algorithm": result.algorithm,
                })
            }
            DefragmentationEvent::CycleFailed { timestamp, error, fragmentation_before } => {
                serde_json::json!({
                    "event_type": "cycle_failed",
                    "timestamp": timestamp.elapsed().as_millis(),
                    "error": error,
                    "fragmentation_before": fragmentation_before,
                })
            }
            DefragmentationEvent::BlockAllocated { timestamp, block_id, pool_id, size } => {
                serde_json::json!({
                    "event_type": "block_allocated",
                    "timestamp": timestamp.elapsed().as_millis(),
                    "block_id": block_id.to_string(),
                    "pool_id": pool_id,
                    "size": size,
                })
            }
            DefragmentationEvent::BlockFreed { timestamp, block_id, pool_id, size } => {
                serde_json::json!({
                    "event_type": "block_freed",
                    "timestamp": timestamp.elapsed().as_millis(),
                    "block_id": block_id.to_string(),
                    "pool_id": pool_id,
                    "size": size,
                })
            }
            DefragmentationEvent::FragmentationThresholdExceeded { timestamp, current_fragmentation, threshold } => {
                serde_json::json!({
                    "event_type": "fragmentation_threshold_exceeded",
                    "timestamp": timestamp.elapsed().as_millis(),
                    "current_fragmentation": current_fragmentation,
                    "threshold": threshold,
                })
            }
            DefragmentationEvent::PerformanceGuardTriggered { timestamp, reason, cpu_usage, memory_pressure } => {
                serde_json::json!({
                    "event_type": "performance_guard_triggered",
                    "timestamp": timestamp.elapsed().as_millis(),
                    "reason": reason,
                    "cpu_usage": cpu_usage,
                    "memory_pressure": memory_pressure,
                })
            }
            DefragmentationEvent::AlgorithmSelected { timestamp, algorithm, block_count, fragmentation_level } => {
                serde_json::json!({
                    "event_type": "algorithm_selected",
                    "timestamp": timestamp.elapsed().as_millis(),
                    "algorithm": algorithm,
                    "block_count": block_count,
                    "fragmentation_level": fragmentation_level,
                })
            }
        }
    }
}

/// Logging event subscriber
pub struct LoggingEventSubscriber;

#[async_trait::async_trait]
impl EventSubscriber for LoggingEventSubscriber {
    async fn handle_event(&self, event: &DefragmentationEvent) -> InfraResult<()> {
        match event {
            DefragmentationEvent::CoordinatorStarted { .. } => {
                tracing::info!("Background defragmentation coordinator started");
            }
            DefragmentationEvent::CoordinatorStopped { .. } => {
                tracing::info!("Background defragmentation coordinator stopped");
            }
            DefragmentationEvent::CycleCompleted { result, .. } => {
                tracing::info!(
                    "Defragmentation cycle completed: {} blocks relocated, {} bytes freed, {:.2}% fragmentation reduction",
                    result.blocks_relocated,
                    result.memory_freed,
                    (result.fragmentation_before - result.fragmentation_after) * 100.0
                );
            }
            DefragmentationEvent::CycleFailed { error, .. } => {
                tracing::warn!("Defragmentation cycle failed: {}", error);
            }
            DefragmentationEvent::FragmentationThresholdExceeded { current_fragmentation, threshold, .. } => {
                tracing::warn!(
                    "Fragmentation threshold exceeded: {:.2}% > {:.2}%",
                    current_fragmentation * 100.0,
                    threshold * 100.0
                );
            }
            DefragmentationEvent::PerformanceGuardTriggered { reason, .. } => {
                tracing::warn!("Performance guard triggered: {}", reason);
            }
            _ => {
                // Log other events at debug level
                tracing::debug!("Defragmentation event: {}", event.event_type());
            }
        }

        Ok(())
    }
}

/// Metrics collection event subscriber
pub struct MetricsEventSubscriber {
    metrics_sender: Option<tokio::sync::mpsc::UnboundedSender<DefragmentationEvent>>,
}

impl MetricsEventSubscriber {
    pub fn new() -> Self {
        Self {
            metrics_sender: None,
        }
    }

    pub fn with_sender(mut self, sender: tokio::sync::mpsc::UnboundedSender<DefragmentationEvent>) -> Self {
        self.metrics_sender = Some(sender);
        self
    }
}

#[async_trait::async_trait]
impl EventSubscriber for MetricsEventSubscriber {
    async fn handle_event(&self, event: &DefragmentationEvent) -> InfraResult<()> {
        if let Some(sender) = &self.metrics_sender {
            let _ = sender.send(event.clone());
        }

        Ok(())
    }
}

/// Custom event subscriber for user-defined handling
pub struct CustomEventSubscriber<F> {
    handler: F,
}

impl<F> CustomEventSubscriber<F>
where
    F: Fn(&DefragmentationEvent) -> InfraResult<()> + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait::async_trait]
impl<F> EventSubscriber for CustomEventSubscriber<F>
where
    F: Fn(&DefragmentationEvent) -> InfraResult<()> + Send + Sync,
{
    async fn handle_event(&self, event: &DefragmentationEvent) -> InfraResult<()> {
        (self.handler)(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let event_bus = EventBus::new();

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let subscriber = CustomEventSubscriber::new(move |event| {
            if let DefragmentationEvent::CoordinatorStarted { .. } = event {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
            }
            Ok(())
        });

        event_bus.subscribe("coordinator_started", subscriber).await;

        let event = DefragmentationEvent::CoordinatorStarted {
            timestamp: Instant::now(),
        };

        event_bus.publish(event).await.unwrap();

        // Give some time for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_event_json_serialization() {
        let event = DefragmentationEvent::CoordinatorStarted {
            timestamp: Instant::now(),
        };

        let json = event.to_json();
        assert_eq!(json["event_type"], "coordinator_started");
        assert!(json["timestamp"].is_number());
    }

    #[tokio::test]
    async fn test_logging_subscriber() {
        let subscriber = LoggingEventSubscriber;

        let event = DefragmentationEvent::CoordinatorStarted {
            timestamp: Instant::now(),
        };

        // This should not panic
        subscriber.handle_event(&event).await.unwrap();
    }
}