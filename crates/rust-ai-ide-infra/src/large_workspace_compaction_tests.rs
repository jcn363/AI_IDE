//! Integration tests for large workspace compaction system

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::time;

    #[tokio::test]
    async fn test_large_workspace_compaction_integration() {
        // Create all components
        let tracker = Arc::new(crate::tracker::MemoryBlockTracker::new());
        let metrics = Arc::new(crate::metrics::FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        let guard = Arc::new(crate::guard::PerformanceGuard::new(crate::guard::GuardConfig::default()));

        // Create large workspace compactor
        let compactor = Arc::new(crate::large_workspace_compactor::LargeWorkspaceCompactor::new(
            crate::large_workspace_compactor::CompactionConfig::default(),
            Arc::clone(&tracker),
            Arc::clone(&metrics),
            Arc::clone(&guard),
        ));

        // Test compactor initialization
        let status = compactor.get_status().await;
        assert!(!status.running);
        assert_eq!(status.total_cycles, 0);

        // Test analyzer
        let analyzer = compactor.analyzer.clone();
        let analysis = analyzer.analyze_workspace().await;
        assert!(!analysis.large_workspace_detected); // Default state

        // Test scheduler
        let scheduler = compactor.scheduler.clone();
        let should_schedule = scheduler.should_schedule_compaction().await;
        assert!(!should_schedule); // No urgent need

        // Test metrics tracker
        let metrics_tracker = compactor.metrics_tracker.clone();
        let summary = metrics_tracker.get_metrics_summary().await;
        assert_eq!(summary.total_operations, 0);

        tracing::info!("Large workspace compaction integration test passed");
    }

    #[tokio::test]
    async fn test_adaptive_strategy_selection() {
        let strategy = crate::adaptive_compaction_strategy::AdaptiveCompactionStrategy::new();

        // Test strategy selection
        let selected = strategy.select_strategy().await;

        // Should select incremental by default
        assert_eq!(selected, crate::large_workspace_compactor::CompactionStrategy::Incremental);

        // Test performance recording
        let performance = crate::adaptive_compaction_strategy::StrategyPerformance {
            timestamp: Instant::now(),
            strategy: selected,
            duration: Duration::from_millis(100),
            memory_freed: 1024 * 1024,
            fragmentation_before: 0.5,
            fragmentation_after: 0.2,
            success: true,
            cpu_usage: 0.4,
            memory_pressure: 0.6,
        };

        strategy.record_performance(performance).await;

        // Test metrics export
        let metrics = strategy.get_performance_metrics().await;
        assert!(!metrics.avg_compaction_time.is_empty());

        tracing::info!("Adaptive strategy selection test passed");
    }

    #[tokio::test]
    async fn test_workspace_memory_analysis() {
        let tracker = Arc::new(crate::tracker::MemoryBlockTracker::new());
        let metrics = Arc::new(crate::metrics::FragmentationMetricsCollector::new(Arc::clone(&tracker)));

        let analyzer = crate::workspace_memory_analyzer::WorkspaceMemoryAnalyzer::new(
            tracker,
            metrics,
        );

        // Test analysis
        let analysis = analyzer.analyze_workspace().await;

        // Should return valid analysis
        assert!(analysis.fragmentation_ratio >= 0.0 && analysis.fragmentation_ratio <= 1.0);
        assert!(analysis.memory_pressure >= 0.0 && analysis.memory_pressure <= 1.0);

        // Test status
        let status = analyzer.get_status().await;
        assert_eq!(status.analysis_cycles, 1);

        tracing::info!("Workspace memory analysis test passed");
    }

    #[tokio::test]
    async fn test_compaction_scheduling() {
        let scheduler = crate::compaction_scheduler::CompactionScheduler::new();

        // Test scheduling
        let system_state = crate::compaction_scheduler::SystemStateSnapshot {
            cpu_usage: 0.5,
            memory_pressure: 0.6,
            fragmentation_ratio: 0.4,
            active_processes: 10,
            io_activity: 0.3,
        };

        let scheduled_time = scheduler.schedule_compaction(
            0.7,
            crate::large_workspace_compactor::CompactionStrategy::Incremental,
            crate::compaction_scheduler::SchedulingReason::RegularInterval,
            system_state,
        ).await.unwrap();

        assert!(scheduled_time > Instant::now());

        // Test status
        let status = scheduler.get_status().await;
        assert_eq!(status.queue_size, 1);

        tracing::info!("Compaction scheduling test passed");
    }

    #[tokio::test]
    async fn test_large_scale_defragmentation() {
        let tracker = Arc::new(crate::tracker::MemoryBlockTracker::new());
        let defrag = crate::large_scale_defragmentation::LargeScaleDefragmentation::new(tracker);

        // Test with empty blocks
        let blocks = vec![];
        let result = defrag.defragment_large_regions(&blocks, 0.8).await;

        // Should handle empty input gracefully
        assert!(result.is_ok());

        tracing::info!("Large scale defragmentation test passed");
    }

    #[tokio::test]
    async fn test_compaction_metrics_tracking() {
        let tracker = crate::compaction_metrics_tracker::CompactionMetricsTracker::new();

        // Test metrics recording
        let result = crate::large_workspace_compactor::CompactionResult {
            operation_id: "test_op".to_string(),
            bytes_processed: 1024,
            bytes_freed: 512,
            fragmentation_before: 0.5,
            fragmentation_after: 0.25,
            duration: Duration::from_millis(100),
            success: true,
            strategy: crate::large_workspace_compactor::CompactionStrategy::Incremental,
        };

        tracker.record_compaction(result).await;

        // Test metrics retrieval
        let summary = tracker.get_metrics_summary().await;
        assert_eq!(summary.total_operations, 1);
        assert_eq!(summary.success_rate, 1.0);
        assert!(summary.avg_efficiency > 0.0);

        // Test metrics export
        let exported = tracker.export_metrics().await;
        assert!(exported.is_object());

        tracing::info!("Compaction metrics tracking test passed");
    }

    #[tokio::test]
    async fn test_compaction_lifecycle() {
        // Create all components
        let tracker = Arc::new(crate::tracker::MemoryBlockTracker::new());
        let metrics = Arc::new(crate::metrics::FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        let guard = Arc::new(crate::guard::PerformanceGuard::new(crate::guard::GuardConfig::default()));

        let compactor = Arc::new(crate::large_workspace_compactor::LargeWorkspaceCompactor::new(
            crate::large_workspace_compactor::CompactionConfig {
                enabled: true,
                large_workspace_threshold: 1024 * 1024, // 1MB for testing
                max_pause_time_ms: 100,
                aggressiveness_level: 0.7,
                incremental_enabled: true,
                incremental_interval: Duration::from_millis(100), // Fast for testing
                emergency_threshold: 0.9,
                virtual_memory_enabled: false,
            },
            Arc::clone(&tracker),
            Arc::clone(&metrics),
            Arc::clone(&guard),
        ));

        // Test startup
        compactor.start().await.unwrap();

        let status = compactor.get_status().await;
        assert!(status.running);

        // Wait a bit for background operations
        time::sleep(Duration::from_millis(50)).await;

        // Test manual compaction trigger
        let result = compactor.trigger_compaction(false).await.unwrap();
        assert!(result.success);

        // Test shutdown
        compactor.stop().await.unwrap();

        let final_status = compactor.get_status().await;
        assert!(!final_status.running);

        tracing::info!("Compaction lifecycle test passed");
    }

    #[tokio::test]
    async fn test_performance_under_load() {
        let tracker = Arc::new(crate::tracker::MemoryBlockTracker::new());
        let metrics = Arc::new(crate::metrics::FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        let guard = Arc::new(crate::guard::PerformanceGuard::new(crate::guard::GuardConfig::default()));

        let compactor = Arc::new(crate::large_workspace_compactor::LargeWorkspaceCompactor::new(
            crate::large_workspace_compactor::CompactionConfig::default(),
            Arc::clone(&tracker),
            Arc::clone(&metrics),
            Arc::clone(&guard),
        ));

        // Simulate multiple compaction operations
        let mut tasks = vec![];

        for i in 0..5 {
            let compactor_clone = Arc::clone(&compactor);
            let task = tokio::spawn(async move {
                let result = compactor_clone.trigger_compaction(false).await;
                result.map(|r| r.duration).unwrap_or_default()
            });
            tasks.push(task);
        }

        // Wait for all operations to complete
        let results = futures::future::join_all(tasks).await;

        // Verify all operations completed
        for result in results {
            let duration = result.unwrap();
            assert!(duration > Duration::from_millis(0));
        }

        tracing::info!("Performance under load test passed");
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let tracker = Arc::new(crate::tracker::MemoryBlockTracker::new());
        let metrics = Arc::new(crate::metrics::FragmentationMetricsCollector::new(Arc::clone(&tracker)));
        let guard = Arc::new(crate::guard::PerformanceGuard::new(crate::guard::GuardConfig::default()));

        let compactor = Arc::new(crate::large_workspace_compactor::LargeWorkspaceCompactor::new(
            crate::large_workspace_compactor::CompactionConfig::default(),
            Arc::clone(&tracker),
            Arc::clone(&metrics),
            Arc::clone(&guard),
        ));

        // Test triggering compaction when compactor is not running
        let result = compactor.trigger_compaction(false).await;
        assert!(result.is_ok()); // Should handle gracefully

        // Test starting already running compactor
        compactor.start().await.unwrap();
        let start_result = compactor.start().await;
        assert!(start_result.is_ok()); // Should handle gracefully

        // Test stopping
        compactor.stop().await.unwrap();

        tracing::info!("Error handling and recovery test passed");
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // Test default configuration
        let default_config = crate::large_workspace_compactor::CompactionConfig::default();
        assert!(default_config.enabled);
        assert!(default_config.large_workspace_threshold > 0);
        assert!(default_config.aggressiveness_level >= 0.0 && default_config.aggressiveness_level <= 1.0);

        // Test scheduler configuration
        let scheduler_config = crate::compaction_scheduler::SchedulerConfig::default();
        assert!(scheduler_config.base_interval > Duration::from_secs(0));
        assert!(scheduler_config.max_queue_size > 0);

        tracing::info!("Configuration validation test passed");
    }

    #[tokio::test]
    async fn test_metrics_data_integrity() {
        let tracker = crate::compaction_metrics_tracker::CompactionMetricsTracker::new();

        // Record multiple operations
        for i in 0..3 {
            let result = crate::large_workspace_compactor::CompactionResult {
                operation_id: format!("test_op_{}", i),
                bytes_processed: 1024 * (i + 1),
                bytes_freed: 512 * (i + 1),
                fragmentation_before: 0.5 - (i as f64 * 0.1),
                fragmentation_after: 0.25 - (i as f64 * 0.05),
                duration: Duration::from_millis(100 + (i as u64 * 20)),
                success: i < 2, // First two succeed, third fails
                strategy: crate::large_workspace_compactor::CompactionStrategy::Incremental,
            };

            tracker.record_compaction(result).await;
        }

        // Verify aggregated stats
        let summary = tracker.get_metrics_summary().await;
        assert_eq!(summary.total_operations, 3);
        assert_eq!(summary.success_rate, 2.0 / 3.0);
        assert!(summary.avg_efficiency > 0.0);

        // Verify detailed metrics
        let start_time = Instant::now() - Duration::from_secs(3600);
        let end_time = Instant::now() + Duration::from_secs(1);
        let detailed = tracker.get_detailed_metrics(start_time, end_time).await;

        assert_eq!(detailed.len(), 3);

        // Verify data integrity
        for record in &detailed {
            assert!(record.bytes_processed > 0);
            assert!(record.duration > Duration::from_millis(0));
            assert!(record.performance_score >= 0.0 && record.performance_score <= 1.0);
        }

        tracing::info!("Metrics data integrity test passed");
    }
}