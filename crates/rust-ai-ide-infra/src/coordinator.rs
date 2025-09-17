//! Background defragmentation coordinator

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio::time;
use crate::config::DefragmentationConfig;
use crate::tracker::MemoryBlockTracker;
use crate::metrics::FragmentationMetricsCollector;
use crate::guard::PerformanceGuard;
use crate::algorithms::{DefragmentationAlgorithm, DefragmentationResult};
use crate::events::{DefragmentationEvent, EventBus};
use crate::InfraResult;

/// Main coordinator for background defragmentation operations
#[derive(Debug)]
pub struct BackgroundDefragmentationCoordinator {
    /// Configuration
    config: DefragmentationConfig,

    /// Memory block tracker
    tracker: Arc<MemoryBlockTracker>,

    /// Metrics collector
    metrics: Arc<FragmentationMetricsCollector>,

    /// Performance guard
    guard: Arc<PerformanceGuard>,

    /// Event bus for notifications
    event_bus: Option<Arc<EventBus>>,

    /// Coordinator state
    state: Arc<RwLock<CoordinatorState>>,

    /// Background task handle
    background_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug)]
struct CoordinatorState {
    /// Whether the coordinator is running
    running: bool,

    /// Last defragmentation cycle time
    last_cycle: Option<Instant>,

    /// Current cycle metrics
    current_cycle_metrics: Option<CycleMetrics>,

    /// Total cycles executed
    total_cycles: usize,

    /// Successful cycles
    successful_cycles: usize,

    /// Failed cycles
    failed_cycles: usize,

    /// Active defragmentation algorithms
    algorithms: Vec<Box<dyn DefragmentationAlgorithm>>,
}

#[derive(Debug, Clone)]
struct CycleMetrics {
    /// Cycle start time
    start_time: Instant,

    /// Fragmentation before cycle
    fragmentation_before: f64,

    /// Memory usage before cycle
    memory_before: usize,

    /// CPU usage during cycle
    cpu_usage: f64,

    /// Memory pressure during cycle
    memory_pressure: f64,

    /// Cycle result
    result: Option<DefragmentationResult>,
}

impl BackgroundDefragmentationCoordinator {
    /// Create a new background defragmentation coordinator
    pub fn new(
        config: DefragmentationConfig,
        tracker: Arc<MemoryBlockTracker>,
        metrics: Arc<FragmentationMetricsCollector>,
        guard: Arc<PerformanceGuard>,
    ) -> Self {
        Self {
            config,
            tracker,
            metrics,
            guard,
            event_bus: None,
            state: Arc::new(RwLock::new(CoordinatorState {
                running: false,
                last_cycle: None,
                current_cycle_metrics: None,
                total_cycles: 0,
                successful_cycles: 0,
                failed_cycles: 0,
                algorithms: Vec::new(),
            })),
            background_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the event bus for notifications
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Add a defragmentation algorithm
    pub async fn add_algorithm(&self, algorithm: Box<dyn DefragmentationAlgorithm>) {
        let mut state = self.state.write().await;
        state.algorithms.push(algorithm);
    }

    /// Start the background defragmentation process
    pub async fn start(&self) -> InfraResult<()> {
        let mut state = self.state.write().await;

        if state.running {
            return Ok(()); // Already running
        }

        state.running = true;

        // Send start event
        if let Some(event_bus) = &self.event_bus {
            let _ = event_bus.publish(DefragmentationEvent::CoordinatorStarted {
                timestamp: Instant::now(),
            }).await;
        }

        // Start background task
        let this = Arc::new(self.clone());
        let handle = tokio::spawn(async move {
            if let Err(e) = this.run_background_cycle().await {
                tracing::error!("Background defragmentation cycle failed: {:?}", e);
            }
        });

        *self.background_handle.write().await = Some(handle);

        tracing::info!("Background defragmentation coordinator started");
        Ok(())
    }

    /// Stop the background defragmentation process
    pub async fn stop(&self) -> InfraResult<()> {
        let mut state = self.state.write().await;
        state.running = false;

        // Cancel background task
        if let Some(handle) = self.background_handle.write().await.take() {
            handle.abort();
        }

        // Send stop event
        if let Some(event_bus) = &self.event_bus {
            let _ = event_bus.publish(DefragmentationEvent::CoordinatorStopped {
                timestamp: Instant::now(),
            }).await;
        }

        tracing::info!("Background defragmentation coordinator stopped");
        Ok(())
    }

    /// Manually trigger a defragmentation cycle
    pub async fn trigger_cycle(&self) -> InfraResult<DefragmentationResult> {
        let state = self.state.read().await;

        if !state.running {
            return Err("Coordinator is not running".into());
        }

        if state.algorithms.is_empty() {
            return Err("No defragmentation algorithms configured".into());
        }

        // Check performance guard
        let current_cpu = self.get_current_cpu_usage().await;
        let current_memory = self.get_current_memory_pressure().await;

        let decision = self.guard.check_operation(current_cpu, current_memory).await;

        match decision {
            crate::guard::GuardDecision::Allow { throttling_factor } => {
                // Proceed with defragmentation
                let result = self.execute_defragmentation_cycle(throttling_factor).await?;

                // Record metrics
                self.metrics.record_defragmentation(result.clone()).await;

                // Complete operation in guard
                self.guard.complete_operation(true, current_cpu, current_memory).await;

                Ok(result)
            }
            crate::guard::GuardDecision::Delay { delay } => {
                tracing::info!("Defragmentation delayed for {:?}", delay);
                Err(format!("Defragmentation delayed for {:?}", delay).into())
            }
            crate::guard::GuardDecision::Cancel { reason } => {
                tracing::warn!("Defragmentation cancelled: {}", reason);
                self.guard.complete_operation(false, current_cpu, current_memory).await;
                Err(reason.into())
            }
        }
    }

    /// Get coordinator status
    pub async fn get_status(&self) -> CoordinatorStatus {
        let state = self.state.read().await;
        let metrics = self.metrics.get_current_metrics().await;

        CoordinatorStatus {
            running: state.running,
            last_cycle: state.last_cycle,
            total_cycles: state.total_cycles,
            successful_cycles: state.successful_cycles,
            failed_cycles: state.failed_cycles,
            fragmentation_ratio: metrics.stats.fragmentation_ratio,
            memory_pressure: metrics.memory_pressure,
            algorithms_count: state.algorithms.len(),
        }
    }

    /// Run the background defragmentation cycle
    async fn run_background_cycle(&self) -> InfraResult<()> {
        let mut interval = time::interval(self.config.cycle_interval);

        loop {
            interval.tick().await;

            let state = self.state.read().await;
            if !state.running {
                break;
            }
            drop(state);

            // Check if defragmentation is needed
            let should_defragment = self.metrics.should_defragment(self.config.fragmentation_threshold).await;

            if should_defragment {
                match self.trigger_cycle().await {
                    Ok(result) => {
                        tracing::info!(
                            "Defragmentation cycle completed: {} blocks relocated, {} bytes freed",
                            result.blocks_relocated,
                            result.memory_freed
                        );

                        let mut state = self.state.write().await;
                        state.total_cycles += 1;
                        state.successful_cycles += 1;
                        state.last_cycle = Some(Instant::now());
                    }
                    Err(e) => {
                        tracing::warn!("Defragmentation cycle failed: {:?}", e);

                        let mut state = self.state.write().await;
                        state.total_cycles += 1;
                        state.failed_cycles += 1;
                    }
                }
            }

            // Cleanup old metrics
            self.metrics.collect_metrics().await?;
        }

        Ok(())
    }

    /// Execute a single defragmentation cycle
    async fn execute_defragmentation_cycle(&self, throttling_factor: f64) -> InfraResult<DefragmentationResult> {
        let start_time = Instant::now();

        // Get fragmented blocks
        let fragmented_blocks = self.tracker.get_fragmented_blocks(self.config.fragmentation_threshold).await;

        if fragmented_blocks.is_empty() {
            return Ok(DefragmentationResult {
                blocks_relocated: 0,
                memory_freed: 0,
                fragmentation_before: 0.0,
                fragmentation_after: 0.0,
                duration: Duration::from_secs(0),
                success: true,
                algorithm: "None".to_string(),
            });
        }

        // Select appropriate algorithm
        let algorithm = self.select_algorithm(&fragmented_blocks).await?;

        // Execute defragmentation
        let result = algorithm.defragment(&self.tracker, fragmented_blocks).await?;

        // Apply throttling delay if needed
        if throttling_factor < 1.0 {
            let delay = Duration::from_millis((result.duration.as_millis() as f64 * (1.0 - throttling_factor)) as u64);
            if delay > Duration::from_millis(0) {
                tokio::time::sleep(delay).await;
            }
        }

        // Send completion event
        if let Some(event_bus) = &self.event_bus {
            let _ = event_bus.publish(DefragmentationEvent::CycleCompleted {
                timestamp: Instant::now(),
                result: result.clone(),
            }).await;
        }

        Ok(result)
    }

    /// Select the most appropriate defragmentation algorithm
    async fn select_algorithm(&self, blocks: &[crate::tracker::MemoryBlock]) -> InfraResult<&Box<dyn DefragmentationAlgorithm>> {
        let state = self.state.read().await;

        if state.algorithms.is_empty() {
            return Err("No defragmentation algorithms available".into());
        }

        // For now, use the first algorithm that can handle the blocks
        // In a more sophisticated implementation, this could use ML to select the best algorithm
        for algorithm in &state.algorithms {
            if algorithm.can_handle(blocks) {
                return Ok(algorithm);
            }
        }

        // Fallback to first algorithm if none can handle the specific case
        Ok(&state.algorithms[0])
    }

    /// Get current CPU usage (placeholder implementation)
    async fn get_current_cpu_usage(&self) -> f64 {
        // In a real implementation, this would query system CPU usage
        // For now, return a conservative estimate
        0.5
    }

    /// Get current memory pressure (placeholder implementation)
    async fn get_current_memory_pressure(&self) -> f64 {
        let stats = self.tracker.get_fragmentation_stats().await;
        stats.utilization_ratio()
    }

    /// Export coordinator status for monitoring
    pub async fn export_status(&self) -> serde_json::Value {
        let status = self.get_status().await;
        let guard_status = self.guard.export_status().await;

        serde_json::json!({
            "coordinator": {
                "running": status.running,
                "last_cycle_seconds_ago": status.last_cycle.map(|t| t.elapsed().as_secs()).unwrap_or(0),
                "total_cycles": status.total_cycles,
                "successful_cycles": status.successful_cycles,
                "failed_cycles": status.failed_cycles,
                "algorithms_count": status.algorithms_count,
                "fragmentation_ratio": status.fragmentation_ratio,
                "memory_pressure": status.memory_pressure
            },
            "performance_guard": guard_status,
            "config": {
                "cycle_interval_seconds": self.config.cycle_interval.as_secs(),
                "fragmentation_threshold": self.config.fragmentation_threshold,
                "enabled": self.config.enabled
            }
        })
    }
}

impl Clone for BackgroundDefragmentationCoordinator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            tracker: Arc::clone(&self.tracker),
            metrics: Arc::clone(&self.metrics),
            guard: Arc::clone(&self.guard),
            event_bus: self.event_bus.as_ref().map(Arc::clone),
            state: Arc::clone(&self.state),
            background_handle: Arc::clone(&self.background_handle),
        }
    }
}

/// Status information for the coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorStatus {
    /// Whether the coordinator is running
    pub running: bool,

    /// Last cycle timestamp
    pub last_cycle: Option<Instant>,

    /// Total cycles executed
    pub total_cycles: usize,

    /// Successful cycles
    pub successful_cycles: usize,

    /// Failed cycles
    pub failed_cycles: usize,

    /// Current fragmentation ratio
    pub fragmentation_ratio: f64,

    /// Current memory pressure
    pub memory_pressure: f64,

    /// Number of configured algorithms
    pub algorithms_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DefragmentationConfig;
    use crate::tracker::MemoryBlockTracker;
    use crate::metrics::FragmentationMetricsCollector;
    use crate::guard::PerformanceGuard;
    use crate::algorithms::CopyingDefragmentation;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let config = DefragmentationConfig::default();
        let tracker = Arc::new(MemoryBlockTracker::new());
        let metrics = Arc::new(FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        let guard = Arc::new(PerformanceGuard::new(crate::guard::GuardConfig::default()));

        let coordinator = BackgroundDefragmentationCoordinator::new(
            config,
            tracker,
            metrics,
            guard,
        );

        let status = coordinator.get_status().await;
        assert!(!status.running);
        assert_eq!(status.total_cycles, 0);
    }

    #[tokio::test]
    async fn test_algorithm_selection() {
        let config = DefragmentationConfig::default();
        let tracker = Arc::new(MemoryBlockTracker::new());
        let metrics = Arc::new(FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        let guard = Arc::new(PerformanceGuard::new(crate::guard::GuardConfig::default()));

        let coordinator = BackgroundDefragmentationCoordinator::new(
            config,
            tracker,
            metrics,
            guard,
        );

        // Add an algorithm
        let algorithm = Box::new(CopyingDefragmentation::new(1024 * 1024, 0.7));
        coordinator.add_algorithm(algorithm).await;

        // Create some test blocks
        let blocks = vec![]; // Empty for this test

        // Test algorithm selection (should not panic)
        let result = coordinator.select_algorithm(&blocks).await;
        assert!(result.is_ok());
    }
}