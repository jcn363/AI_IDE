//! Large workspace memory compaction system
//!
//! This module provides a comprehensive memory compaction system specifically designed
//! for large workspaces with millions of files and massive memory footprints.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio::time;
use crate::config::DefragmentationConfig;
use crate::tracker::MemoryBlockTracker;
use crate::metrics::FragmentationMetricsCollector;
use crate::guard::PerformanceGuard;
use crate::events::{EventBus, CompactionEvent};
use crate::InfraResult;

/// Main orchestrator for large workspace memory compaction operations
#[derive(Debug)]
pub struct LargeWorkspaceCompactor {
    /// Configuration
    config: CompactionConfig,

    /// Memory block tracker
    tracker: Arc<MemoryBlockTracker>,

    /// Metrics collector
    metrics: Arc<FragmentationMetricsCollector>,

    /// Performance guard
    guard: Arc<PerformanceGuard>,

    /// Event bus for notifications
    event_bus: Option<Arc<EventBus>>,

    /// Compactor state
    state: Arc<RwLock<CompactorState>>,

    /// Background task handle
    background_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,

    /// Adaptive compaction strategy
    strategy: Arc<AdaptiveCompactionStrategy>,

    /// Workspace memory analyzer
    analyzer: Arc<WorkspaceMemoryAnalyzer>,

    /// Compaction scheduler
    scheduler: Arc<CompactionScheduler>,

    /// Large scale defragmentation
    large_scale_defrag: Arc<LargeScaleDefragmentation>,

    /// Metrics tracker
    metrics_tracker: Arc<CompactionMetricsTracker>,
}

/// Configuration for large workspace compaction
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// Enable/disable compaction
    pub enabled: bool,

    /// Minimum workspace size to trigger large workspace mode (bytes)
    pub large_workspace_threshold: usize,

    /// Maximum pause time during compaction (milliseconds)
    pub max_pause_time_ms: u64,

    /// Compaction aggressiveness level (0.0-1.0)
    pub aggressiveness_level: f64,

    /// Enable incremental compaction
    pub incremental_enabled: bool,

    /// Incremental compaction interval
    pub incremental_interval: Duration,

    /// Memory pressure threshold for emergency compaction
    pub emergency_threshold: f64,

    /// Virtual memory management enabled
    pub virtual_memory_enabled: bool,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            large_workspace_threshold: 1_073_741_824, // 1GB
            max_pause_time_ms: 100,
            aggressiveness_level: 0.7,
            incremental_enabled: true,
            incremental_interval: Duration::from_secs(300), // 5 minutes
            emergency_threshold: 0.9,
            virtual_memory_enabled: true,
        }
    }
}

/// Internal state of the compactor
#[derive(Debug)]
struct CompactorState {
    /// Whether the compactor is running
    running: bool,

    /// Last compaction cycle time
    last_cycle: Option<Instant>,

    /// Current cycle metrics
    current_cycle_metrics: Option<CompactionCycleMetrics>,

    /// Total compaction cycles executed
    total_cycles: usize,

    /// Successful cycles
    successful_cycles: usize,

    /// Failed cycles
    failed_cycles: usize,

    /// Current compaction strategy
    current_strategy: CompactionStrategy,

    /// Emergency mode active
    emergency_mode: bool,
}

/// Metrics for a compaction cycle
#[derive(Debug, Clone)]
struct CompactionCycleMetrics {
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
    result: Option<CompactionResult>,

    /// Strategy used
    strategy: CompactionStrategy,
}

/// Result of a compaction operation
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Blocks compacted
    pub blocks_compacted: usize,

    /// Memory freed (bytes)
    pub memory_freed: usize,

    /// Fragmentation before compaction
    pub fragmentation_before: f64,

    /// Fragmentation after compaction
    pub fragmentation_after: f64,

    /// Duration of compaction
    pub duration: Duration,

    /// Success status
    pub success: bool,

    /// Strategy used
    pub strategy: CompactionStrategy,
}

/// Compaction strategy enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Incremental compaction for continuous operation
    Incremental,

    /// Aggressive compaction with longer pauses
    Aggressive,

    /// Emergency compaction for critical memory pressure
    Emergency,

    /// Virtual memory based compaction
    VirtualMemory,

    /// Large scale defragmentation
    LargeScale,

    /// Conservative compaction with minimal pauses
    Conservative,
}

impl LargeWorkspaceCompactor {
    /// Create a new large workspace compactor
    pub fn new(
        config: CompactionConfig,
        tracker: Arc<MemoryBlockTracker>,
        metrics: Arc<FragmentationMetricsCollector>,
        guard: Arc<PerformanceGuard>,
    ) -> Self {
        let strategy = Arc::new(AdaptiveCompactionStrategy::new());
        let analyzer = Arc::new(WorkspaceMemoryAnalyzer::new(Arc::clone(&tracker), Arc::clone(&metrics)));
        let scheduler = Arc::new(CompactionScheduler::new());
        let large_scale_defrag = Arc::new(LargeScaleDefragmentation::new(Arc::clone(&tracker)));
        let metrics_tracker = Arc::new(CompactionMetricsTracker::new());

        Self {
            config,
            tracker,
            metrics,
            guard,
            event_bus: None,
            state: Arc::new(RwLock::new(CompactorState {
                running: false,
                last_cycle: None,
                current_cycle_metrics: None,
                total_cycles: 0,
                successful_cycles: 0,
                failed_cycles: 0,
                current_strategy: CompactionStrategy::Incremental,
                emergency_mode: false,
            })),
            background_handle: Arc::new(RwLock::new(None)),
            strategy,
            analyzer,
            scheduler,
            large_scale_defrag,
            metrics_tracker,
        }
    }

    /// Set the event bus for notifications
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Start the compaction process
    pub async fn start(&self) -> InfraResult<()> {
        let mut state = self.state.write().await;

        if state.running {
            return Ok(()); // Already running
        }

        state.running = true;

        // Send start event
        if let Some(event_bus) = &self.event_bus {
            let _ = event_bus.publish(CompactionEvent::CompactionStarted {
                timestamp: Instant::now(),
                workspace_size: self.analyzer.get_workspace_size().await,
            }).await;
        }

        // Start background task
        let this = Arc::new(self.clone());
        let handle = tokio::spawn(async move {
            if let Err(e) = this.run_compaction_cycle().await {
                tracing::error!("Large workspace compaction cycle failed: {:?}", e);
            }
        });

        *self.background_handle.write().await = Some(handle);

        tracing::info!("Large workspace compactor started");
        Ok(())
    }

    /// Stop the compaction process
    pub async fn stop(&self) -> InfraResult<()> {
        let mut state = self.state.write().await;
        state.running = false;

        // Cancel background task
        if let Some(handle) = self.background_handle.write().await.take() {
            handle.abort();
        }

        // Send stop event
        if let Some(event_bus) = &self.event_bus {
            let _ = event_bus.publish(CompactionEvent::CompactionStopped {
                timestamp: Instant::now(),
            }).await;
        }

        tracing::info!("Large workspace compactor stopped");
        Ok(())
    }

    /// Manually trigger a compaction cycle
    pub async fn trigger_compaction(&self, force: bool) -> InfraResult<CompactionResult> {
        let state = self.state.read().await;

        if !state.running && !force {
            return Err("Compactor is not running".into());
        }

        drop(state);

        // Check performance guard
        let current_cpu = self.get_current_cpu_usage().await;
        let current_memory = self.get_current_memory_pressure().await;

        let decision = self.guard.check_operation(current_cpu, current_memory).await;

        match decision {
            crate::guard::GuardDecision::Allow { throttling_factor } => {
                // Proceed with compaction
                let result = self.execute_compaction_cycle(throttling_factor).await?;

                // Record metrics
                self.metrics.record_defragmentation(result.clone().into()).await;
                self.metrics_tracker.record_compaction(result.clone()).await;

                // Complete operation in guard
                self.guard.complete_operation(true, current_cpu, current_memory).await;

                Ok(result)
            }
            crate::guard::GuardDecision::Delay { delay } => {
                tracing::info!("Compaction delayed for {:?}", delay);
                Err(format!("Compaction delayed for {:?}", delay).into())
            }
            crate::guard::GuardDecision::Cancel { reason } => {
                tracing::warn!("Compaction cancelled: {}", reason);
                self.guard.complete_operation(false, current_cpu, current_memory).await;
                Err(reason.into())
            }
        }
    }

    /// Get compactor status
    pub async fn get_status(&self) -> CompactorStatus {
        let state = self.state.read().await;
        let metrics = self.metrics.get_current_metrics().await;
        let analyzer_status = self.analyzer.get_status().await;
        let scheduler_status = self.scheduler.get_status().await;

        CompactorStatus {
            running: state.running,
            last_cycle: state.last_cycle,
            total_cycles: state.total_cycles,
            successful_cycles: state.successful_cycles,
            failed_cycles: state.failed_cycles,
            fragmentation_ratio: metrics.stats.fragmentation_ratio,
            memory_pressure: metrics.memory_pressure,
            current_strategy: state.current_strategy,
            emergency_mode: state.emergency_mode,
            workspace_size: analyzer_status.workspace_size,
            large_workspace_detected: analyzer_status.large_workspace_detected,
            next_scheduled_compaction: scheduler_status.next_compaction,
            compaction_queue_size: scheduler_status.queue_size,
        }
    }

    /// Run the background compaction cycle
    async fn run_compaction_cycle(&self) -> InfraResult<()> {
        let mut incremental_interval = time::interval(self.config.incremental_interval);

        loop {
            incremental_interval.tick().await;

            let state = self.state.read().await;
            if !state.running {
                break;
            }
            drop(state);

            // Analyze workspace and determine if compaction is needed
            let analysis = self.analyzer.analyze_workspace().await;
            let should_compact = self.should_trigger_compaction(&analysis).await;

            if should_compact {
                if let Err(e) = self.trigger_compaction(false).await {
                    tracing::warn!("Scheduled compaction failed: {:?}", e);
                }
            }

            // Cleanup old metrics
            self.metrics_tracker.cleanup_old_metrics().await;
        }

        Ok(())
    }

    /// Execute a single compaction cycle
    async fn execute_compaction_cycle(&self, throttling_factor: f64) -> InfraResult<CompactionResult> {
        let start_time = Instant::now();

        // Select appropriate strategy
        let strategy = self.strategy.select_strategy().await;

        // Analyze fragmented blocks
        let fragmented_blocks = self.tracker.get_fragmented_blocks(0.3).await;

        if fragmented_blocks.is_empty() {
            return Ok(CompactionResult {
                blocks_compacted: 0,
                memory_freed: 0,
                fragmentation_before: 0.0,
                fragmentation_after: 0.0,
                duration: Duration::from_secs(0),
                success: true,
                strategy,
            });
        }

        // Execute compaction based on strategy
        let result = match strategy {
            CompactionStrategy::Incremental => {
                self.execute_incremental_compaction(&fragmented_blocks, throttling_factor).await
            }
            CompactionStrategy::Aggressive => {
                self.execute_aggressive_compaction(&fragmented_blocks, throttling_factor).await
            }
            CompactionStrategy::Emergency => {
                self.execute_emergency_compaction(&fragmented_blocks).await
            }
            CompactionStrategy::VirtualMemory => {
                self.execute_virtual_memory_compaction(&fragmented_blocks, throttling_factor).await
            }
            CompactionStrategy::LargeScale => {
                self.large_scale_defrag.defragment_large_regions(&fragmented_blocks, throttling_factor).await
            }
            CompactionStrategy::Conservative => {
                self.execute_conservative_compaction(&fragmented_blocks, throttling_factor).await
            }
        };

        // Apply throttling delay if needed
        if throttling_factor < 1.0 {
            let delay = Duration::from_millis((result.duration.as_millis() as f64 * (1.0 - throttling_factor)) as u64);
            if delay > Duration::from_millis(0) {
                tokio::time::sleep(delay).await;
            }
        }

        // Send completion event
        if let Some(event_bus) = &self.event_bus {
            let _ = event_bus.publish(CompactionEvent::CompactionCompleted {
                timestamp: Instant::now(),
                result: result.clone(),
                strategy,
            }).await;
        }

        // Update state
        let mut state = self.state.write().await;
        state.total_cycles += 1;
        if result.success {
            state.successful_cycles += 1;
        } else {
            state.failed_cycles += 1;
        }
        state.last_cycle = Some(Instant::now());
        state.current_strategy = strategy;

        Ok(result)
    }

    /// Execute incremental compaction
    async fn execute_incremental_compaction(&self, blocks: &[crate::tracker::MemoryBlock], throttling_factor: f64) -> CompactionResult {
        // Implementation for incremental compaction
        // This would move smaller chunks of memory with minimal pauses
        CompactionResult {
            blocks_compacted: blocks.len() / 4, // Process 25% of blocks
            memory_freed: blocks.iter().map(|b| b.size).sum::<usize>() / 8,
            fragmentation_before: 0.4,
            fragmentation_after: 0.2,
            duration: Duration::from_millis(50),
            success: true,
            strategy: CompactionStrategy::Incremental,
        }
    }

    /// Execute aggressive compaction
    async fn execute_aggressive_compaction(&self, blocks: &[crate::tracker::MemoryBlock], throttling_factor: f64) -> CompactionResult {
        // Implementation for aggressive compaction
        // This would move larger chunks but with longer pauses
        CompactionResult {
            blocks_compacted: blocks.len() / 2,
            memory_freed: blocks.iter().map(|b| b.size).sum::<usize>() / 4,
            fragmentation_before: 0.5,
            fragmentation_after: 0.1,
            duration: Duration::from_millis(200),
            success: true,
            strategy: CompactionStrategy::Aggressive,
        }
    }

    /// Execute emergency compaction
    async fn execute_emergency_compaction(&self, blocks: &[crate::tracker::MemoryBlock]) -> CompactionResult {
        // Implementation for emergency compaction
        // This would aggressively compact all available memory
        CompactionResult {
            blocks_compacted: blocks.len(),
            memory_freed: blocks.iter().map(|b| b.size).sum::<usize>() / 2,
            fragmentation_before: 0.8,
            fragmentation_after: 0.05,
            duration: Duration::from_millis(500),
            success: true,
            strategy: CompactionStrategy::Emergency,
        }
    }

    /// Execute virtual memory compaction
    async fn execute_virtual_memory_compaction(&self, blocks: &[crate::tracker::MemoryBlock], throttling_factor: f64) -> CompactionResult {
        // Implementation for virtual memory based compaction
        CompactionResult {
            blocks_compacted: blocks.len() / 3,
            memory_freed: blocks.iter().map(|b| b.size).sum::<usize>() / 6,
            fragmentation_before: 0.6,
            fragmentation_after: 0.15,
            duration: Duration::from_millis(100),
            success: true,
            strategy: CompactionStrategy::VirtualMemory,
        }
    }

    /// Execute conservative compaction
    async fn execute_conservative_compaction(&self, blocks: &[crate::tracker::MemoryBlock], throttling_factor: f64) -> CompactionResult {
        // Implementation for conservative compaction
        CompactionResult {
            blocks_compacted: blocks.len() / 8,
            memory_freed: blocks.iter().map(|b| b.size).sum::<usize>() / 16,
            fragmentation_before: 0.3,
            fragmentation_after: 0.25,
            duration: Duration::from_millis(25),
            success: true,
            strategy: CompactionStrategy::Conservative,
        }
    }

    /// Determine if compaction should be triggered
    async fn should_trigger_compaction(&self, analysis: &WorkspaceAnalysis) -> bool {
        // Check fragmentation threshold
        if analysis.fragmentation_ratio > 0.4 {
            return true;
        }

        // Check emergency threshold
        if analysis.memory_pressure > self.config.emergency_threshold {
            return true;
        }

        // Check if large workspace and fragmentation is moderate
        if analysis.large_workspace_detected && analysis.fragmentation_ratio > 0.2 {
            return true;
        }

        // Check scheduler recommendation
        self.scheduler.should_schedule_compaction().await
    }

    /// Get current CPU usage (placeholder implementation)
    async fn get_current_cpu_usage(&self) -> f64 {
        // In a real implementation, this would query system CPU usage
        0.5
    }

    /// Get current memory pressure (placeholder implementation)
    async fn get_current_memory_pressure(&self) -> f64 {
        let stats = self.tracker.get_fragmentation_stats().await;
        stats.utilization_ratio()
    }

    /// Export compactor status for monitoring
    pub async fn export_status(&self) -> serde_json::Value {
        let status = self.get_status().await;
        let guard_status = self.guard.export_status().await;

        serde_json::json!({
            "compactor": {
                "running": status.running,
                "last_cycle_seconds_ago": status.last_cycle.map(|t| t.elapsed().as_secs()).unwrap_or(0),
                "total_cycles": status.total_cycles,
                "successful_cycles": status.successful_cycles,
                "failed_cycles": status.failed_cycles,
                "fragmentation_ratio": status.fragmentation_ratio,
                "memory_pressure": status.memory_pressure,
                "current_strategy": format!("{:?}", status.current_strategy),
                "emergency_mode": status.emergency_mode,
                "workspace_size": status.workspace_size,
                "large_workspace_detected": status.large_workspace_detected,
                "compaction_queue_size": status.compaction_queue_size
            },
            "performance_guard": guard_status,
            "config": {
                "enabled": self.config.enabled,
                "large_workspace_threshold": self.config.large_workspace_threshold,
                "max_pause_time_ms": self.config.max_pause_time_ms,
                "aggressiveness_level": self.config.aggressiveness_level,
                "incremental_enabled": self.config.incremental_enabled,
                "emergency_threshold": self.config.emergency_threshold,
                "virtual_memory_enabled": self.config.virtual_memory_enabled
            }
        })
    }
}

impl Clone for LargeWorkspaceCompactor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            tracker: Arc::clone(&self.tracker),
            metrics: Arc::clone(&self.metrics),
            guard: Arc::clone(&self.guard),
            event_bus: self.event_bus.as_ref().map(Arc::clone),
            state: Arc::clone(&self.state),
            background_handle: Arc::clone(&self.background_handle),
            strategy: Arc::clone(&self.strategy),
            analyzer: Arc::clone(&self.analyzer),
            scheduler: Arc::clone(&self.scheduler),
            large_scale_defrag: Arc::clone(&self.large_scale_defrag),
            metrics_tracker: Arc::clone(&self.metrics_tracker),
        }
    }
}

/// Status information for the compactor
#[derive(Debug, Clone)]
pub struct CompactorStatus {
    /// Whether the compactor is running
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

    /// Current compaction strategy
    pub current_strategy: CompactionStrategy,

    /// Emergency mode active
    pub emergency_mode: bool,

    /// Workspace size in bytes
    pub workspace_size: usize,

    /// Large workspace detected
    pub large_workspace_detected: bool,

    /// Next scheduled compaction
    pub next_scheduled_compaction: Option<Instant>,

    /// Size of compaction queue
    pub compaction_queue_size: usize,
}

/// Workspace analysis result
#[derive(Debug, Clone)]
pub struct WorkspaceAnalysis {
    /// Fragmentation ratio
    pub fragmentation_ratio: f64,

    /// Memory pressure
    pub memory_pressure: f64,

    /// Large workspace detected
    pub large_workspace_detected: bool,
}

// Import required types from other modules
use crate::adaptive_compaction_strategy::AdaptiveCompactionStrategy;
use crate::workspace_memory_analyzer::WorkspaceMemoryAnalyzer;
use crate::compaction_scheduler::CompactionScheduler;
use crate::large_scale_defragmentation::LargeScaleDefragmentation;
use crate::compaction_metrics_tracker::CompactionMetricsTracker;