//! # Rust AI IDE Infrastructure Crate
//!
//! This crate provides core infrastructure components for the Rust AI IDE,
//! including background defragmentation, memory management, large workspace compaction,
//! and performance monitoring.

pub mod config;
pub mod coordinator;
pub mod algorithms;
pub mod tracker;
pub mod metrics;
pub mod guard;
pub mod events;
pub mod integration;

// Large workspace compaction modules
pub mod large_workspace_compactor;
pub mod adaptive_compaction_strategy;
pub mod workspace_memory_analyzer;
pub mod compaction_scheduler;
pub mod large_scale_defragmentation;
pub mod compaction_metrics_tracker;

// Test modules
#[cfg(test)]
mod large_workspace_compaction_tests;

// Re-exports for convenience
pub use config::DefragmentationConfig;
pub use coordinator::BackgroundDefragmentationCoordinator;
pub use algorithms::{DefragmentationAlgorithm, CopyingDefragmentation, MarkCompactDefragmentation, GenerationalDefragmentation, PoolSpecificDefragmentation};
pub use tracker::MemoryBlockTracker;
pub use metrics::FragmentationMetricsCollector;
pub use guard::PerformanceGuard;
pub use events::{EventBus, DefragmentationEvent, LoggingEventSubscriber, MetricsEventSubscriber};
pub use integration::DefragmentationIntegrationManager;

// Large workspace compaction re-exports
pub use large_workspace_compactor::{LargeWorkspaceCompactor, CompactionConfig, CompactionResult, CompactionStrategy, CompactorStatus};
pub use adaptive_compaction_strategy::AdaptiveCompactionStrategy;
pub use workspace_memory_analyzer::{WorkspaceMemoryAnalyzer, AnalyzerStatus, WorkspaceAnalysis};
pub use compaction_scheduler::{CompactionScheduler, SchedulerStatus, SchedulingMode};
pub use large_scale_defragmentation::{LargeScaleDefragmentation, LargeScaleResult};
pub use compaction_metrics_tracker::{CompactionMetricsTracker, MetricsSummary};

/// Result type for infrastructure operations
pub type InfraResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_infrastructure_components_integration() {
        // Test configuration
        let config = DefragmentationConfig::default();
        assert!(config.validate().is_ok());

        // Test memory block tracker
        let tracker = Arc::new(MemoryBlockTracker::new());
        let block_id = tracker.register_block("test_pool".to_string(), 1024, Some(0x1000)).await;
        let stats = tracker.get_fragmentation_stats().await;
        assert_eq!(stats.allocated_blocks, 1);

        // Test metrics collector
        let metrics = Arc::new(FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        metrics.collect_metrics().await.unwrap();
        let current_metrics = metrics.get_current_metrics().await;
        assert!(current_metrics.timestamp.elapsed() < Duration::from_secs(1));

        // Test performance guard
        let guard = Arc::new(PerformanceGuard::new(crate::guard::GuardConfig::default()));
        let decision = guard.check_operation(0.5, 0.5).await;
        match decision {
            crate::guard::GuardDecision::Allow { .. } => {},
            _ => panic!("Expected operation to be allowed"),
        }

        // Test event bus
        let event_bus = Arc::new(EventBus::new());
        let subscriber = LoggingEventSubscriber;
        event_bus.subscribe("test", subscriber).await;

        // Test coordinator creation
        let coordinator = BackgroundDefragmentationCoordinator::new(
            config,
            Arc::clone(&tracker),
            Arc::clone(&metrics),
            Arc::clone(&guard),
        );

        let status = coordinator.get_status().await;
        assert!(!status.running);
        assert_eq!(status.total_cycles, 0);

        // Test integration manager
        let integration_manager = DefragmentationIntegrationManager::new(
            Arc::clone(&tracker),
            Arc::clone(&metrics),
            Arc::clone(&guard),
            Arc::new(coordinator),
        );

        let integration_status = integration_manager.get_integration_status();
        assert!(!integration_status.active);
        assert_eq!(integration_status.total_integrations, 0);
    }
}