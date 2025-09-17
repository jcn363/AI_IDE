//! Adaptive compaction strategy system
//!
//! This module provides dynamic strategy selection for memory compaction
//! based on workspace size, fragmentation patterns, and performance metrics.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::tracker::MemoryBlockTracker;
use crate::metrics::FragmentationMetricsCollector;
use crate::InfraResult;

/// Adaptive compaction strategy selector
#[derive(Debug)]
pub struct AdaptiveCompactionStrategy {
    /// Configuration
    config: StrategyConfig,

    /// Memory block tracker
    tracker: Arc<MemoryBlockTracker>,

    /// Metrics collector
    metrics: Arc<FragmentationMetricsCollector>,

    /// Strategy state
    state: Arc<RwLock<StrategyState>>,

    /// Performance history
    performance_history: Arc<RwLock<Vec<StrategyPerformance>>>,
}

/// Configuration for adaptive strategy selection
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    /// Enable adaptive strategy selection
    pub adaptive_enabled: bool,

    /// History window for performance analysis (seconds)
    pub history_window_seconds: u64,

    /// Minimum samples required for analysis
    pub min_samples_for_analysis: usize,

    /// CPU threshold for conservative mode
    pub cpu_threshold_conservative: f64,

    /// Memory threshold for aggressive mode
    pub memory_threshold_aggressive: f64,

    /// Fragmentation threshold for emergency mode
    pub fragmentation_threshold_emergency: f64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            adaptive_enabled: true,
            history_window_seconds: 3600, // 1 hour
            min_samples_for_analysis: 5,
            cpu_threshold_conservative: 0.8,
            memory_threshold_aggressive: 0.7,
            fragmentation_threshold_emergency: 0.8,
        }
    }
}

/// Internal state of the adaptive strategy
#[derive(Debug)]
struct StrategyState {
    /// Current selected strategy
    current_strategy: super::large_workspace_compactor::CompactionStrategy,

    /// Last strategy change time
    last_change: Option<Instant>,

    /// Strategy performance metrics
    performance_metrics: StrategyPerformanceMetrics,

    /// Adaptation cycle count
    adaptation_cycles: usize,
}

/// Performance metrics for strategies
#[derive(Debug, Clone)]
struct StrategyPerformanceMetrics {
    /// Average compaction time by strategy
    avg_compaction_time: std::collections::HashMap<String, Duration>,

    /// Average memory freed by strategy
    avg_memory_freed: std::collections::HashMap<String, usize>,

    /// Average fragmentation reduction by strategy
    avg_fragmentation_reduction: std::collections::HashMap<String, f64>,

    /// Success rate by strategy
    success_rate: std::collections::HashMap<String, f64>,
}

/// Performance record for a strategy execution
#[derive(Debug, Clone)]
pub struct StrategyPerformance {
    /// Timestamp of execution
    pub timestamp: Instant,

    /// Strategy used
    pub strategy: super::large_workspace_compactor::CompactionStrategy,

    /// Execution duration
    pub duration: Duration,

    /// Memory freed
    pub memory_freed: usize,

    /// Fragmentation before compaction
    pub fragmentation_before: f64,

    /// Fragmentation after compaction
    pub fragmentation_after: f64,

    /// Success status
    pub success: bool,

    /// CPU usage during execution
    pub cpu_usage: f64,

    /// Memory pressure during execution
    pub memory_pressure: f64,
}

impl AdaptiveCompactionStrategy {
    /// Create a new adaptive compaction strategy
    pub fn new() -> Self {
        Self {
            config: StrategyConfig::default(),
            tracker: Arc::new(MemoryBlockTracker::new()),
            metrics: Arc::new(FragmentationMetricsCollector::new(Arc::new(MemoryBlockTracker::new()))),
            state: Arc::new(RwLock::new(StrategyState {
                current_strategy: super::large_workspace_compactor::CompactionStrategy::Incremental,
                last_change: None,
                performance_metrics: StrategyPerformanceMetrics {
                    avg_compaction_time: std::collections::HashMap::new(),
                    avg_memory_freed: std::collections::HashMap::new(),
                    avg_fragmentation_reduction: std::collections::HashMap::new(),
                    success_rate: std::collections::HashMap::new(),
                },
                adaptation_cycles: 0,
            })),
            performance_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Select the most appropriate compaction strategy
    pub async fn select_strategy(&self) -> super::large_workspace_compactor::CompactionStrategy {
        if !self.config.adaptive_enabled {
            return super::large_workspace_compactor::CompactionStrategy::Incremental;
        }

        // Analyze current system state
        let system_state = self.analyze_system_state().await;

        // Check for emergency conditions
        if self.should_use_emergency_strategy(&system_state).await {
            return super::large_workspace_compactor::CompactionStrategy::Emergency;
        }

        // Analyze performance history
        let history_analysis = self.analyze_performance_history().await;

        // Select strategy based on analysis
        self.select_optimal_strategy(&system_state, &history_analysis).await
    }

    /// Record strategy performance for learning
    pub async fn record_performance(&self, performance: StrategyPerformance) {
        let mut history = self.performance_history.write().await;
        history.push(performance.clone());

        // Keep only recent history
        let cutoff = Instant::now() - Duration::from_secs(self.config.history_window_seconds);
        history.retain(|p| p.timestamp > cutoff);

        // Update performance metrics
        self.update_performance_metrics(&performance).await;

        // Trigger adaptation if needed
        if history.len() >= self.config.min_samples_for_analysis {
            self.adapt_strategy().await;
        }
    }

    /// Analyze current system state
    async fn analyze_system_state(&self) -> SystemStateAnalysis {
        let metrics = self.metrics.get_current_metrics().await;
        let stats = self.tracker.get_fragmentation_stats().await;

        SystemStateAnalysis {
            fragmentation_ratio: metrics.stats.fragmentation_ratio,
            memory_pressure: metrics.memory_pressure,
            memory_utilization: stats.utilization_ratio(),
            large_workspace: stats.allocated_blocks > 10000, // Arbitrary threshold
            high_cpu_usage: self.get_current_cpu_usage().await > self.config.cpu_threshold_conservative,
        }
    }

    /// Check if emergency strategy should be used
    async fn should_use_emergency_strategy(&self, state: &SystemStateAnalysis) -> bool {
        state.fragmentation_ratio > self.config.fragmentation_threshold_emergency
            || (state.memory_pressure > 0.9 && state.large_workspace)
    }

    /// Analyze performance history to find optimal strategies
    async fn analyze_performance_history(&self) -> HistoryAnalysis {
        let history = self.performance_history.read().await;

        if history.is_empty() {
            return HistoryAnalysis {
                best_strategy: super::large_workspace_compactor::CompactionStrategy::Incremental,
                avg_efficiency: 0.0,
                success_rate: 0.0,
            };
        }

        // Analyze each strategy's performance
        let mut strategy_scores = std::collections::HashMap::new();

        for performance in history.iter() {
            let strategy_key = format!("{:?}", performance.strategy);
            let score = self.calculate_strategy_score(performance);

            strategy_scores.entry(strategy_key)
                .or_insert_with(Vec::new)
                .push(score);
        }

        // Find strategy with best average score
        let mut best_strategy = super::large_workspace_compactor::CompactionStrategy::Incremental;
        let mut best_score = 0.0;

        for (strategy_str, scores) in strategy_scores {
            let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
            if avg_score > best_score {
                best_score = avg_score;
                best_strategy = self.parse_strategy_from_string(&strategy_str);
            }
        }

        HistoryAnalysis {
            best_strategy,
            avg_efficiency: best_score,
            success_rate: self.calculate_overall_success_rate(&history),
        }
    }

    /// Select optimal strategy based on system state and history
    async fn select_optimal_strategy(
        &self,
        system_state: &SystemStateAnalysis,
        history_analysis: &HistoryAnalysis,
    ) -> super::large_workspace_compactor::CompactionStrategy {
        // Emergency conditions override adaptive selection
        if system_state.fragmentation_ratio > self.config.fragmentation_threshold_emergency {
            return super::large_workspace_compactor::CompactionStrategy::Emergency;
        }

        // High memory pressure favors aggressive strategies
        if system_state.memory_pressure > self.config.memory_threshold_aggressive {
            return super::large_workspace_compactor::CompactionStrategy::Aggressive;
        }

        // High CPU usage favors conservative strategies
        if system_state.high_cpu_usage {
            return super::large_workspace_compactor::CompactionStrategy::Conservative;
        }

        // Large workspaces benefit from specialized strategies
        if system_state.large_workspace {
            return super::large_workspace_compactor::CompactionStrategy::LargeScale;
        }

        // Use history-based recommendation if available and successful
        if history_analysis.success_rate > 0.7 {
            return history_analysis.best_strategy;
        }

        // Default to incremental for stable operation
        super::large_workspace_compactor::CompactionStrategy::Incremental
    }

    /// Calculate performance score for a strategy execution
    fn calculate_strategy_score(&self, performance: &StrategyPerformance) -> f64 {
        if !performance.success {
            return 0.0;
        }

        // Score based on efficiency: memory freed per time unit
        let efficiency = performance.memory_freed as f64 / performance.duration.as_millis() as f64;

        // Score based on fragmentation reduction
        let fragmentation_reduction = performance.fragmentation_before - performance.fragmentation_after;

        // Combined score with weights
        efficiency * 0.6 + fragmentation_reduction * 0.4
    }

    /// Calculate overall success rate from history
    fn calculate_overall_success_rate(&self, history: &[StrategyPerformance]) -> f64 {
        if history.is_empty() {
            return 0.0;
        }

        let successful = history.iter().filter(|p| p.success).count();
        successful as f64 / history.len() as f64
    }

    /// Parse strategy from string representation
    fn parse_strategy_from_string(&self, strategy_str: &str) -> super::large_workspace_compactor::CompactionStrategy {
        match strategy_str {
            "Incremental" => super::large_workspace_compactor::CompactionStrategy::Incremental,
            "Aggressive" => super::large_workspace_compactor::CompactionStrategy::Aggressive,
            "Emergency" => super::large_workspace_compactor::CompactionStrategy::Emergency,
            "VirtualMemory" => super::large_workspace_compactor::CompactionStrategy::VirtualMemory,
            "LargeScale" => super::large_workspace_compactor::CompactionStrategy::LargeScale,
            "Conservative" => super::large_workspace_compactor::CompactionStrategy::Conservative,
            _ => super::large_workspace_compactor::CompactionStrategy::Incremental,
        }
    }

    /// Update performance metrics with new data
    async fn update_performance_metrics(&self, performance: &StrategyPerformance) {
        let mut state = self.state.write().await;
        let strategy_key = format!("{:?}", performance.strategy);

        // Update average compaction time
        let current_avg = state.performance_metrics.avg_compaction_time
            .get(&strategy_key)
            .copied()
            .unwrap_or(Duration::from_millis(100));
        let new_avg = (current_avg + performance.duration) / 2;
        state.performance_metrics.avg_compaction_time.insert(strategy_key.clone(), new_avg);

        // Update average memory freed
        let current_avg_memory = state.performance_metrics.avg_memory_freed
            .get(&strategy_key)
            .copied()
            .unwrap_or(1024 * 1024);
        let new_avg_memory = (current_avg_memory + performance.memory_freed) / 2;
        state.performance_metrics.avg_memory_freed.insert(strategy_key.clone(), new_avg_memory);

        // Update fragmentation reduction
        let frag_reduction = performance.fragmentation_before - performance.fragmentation_after;
        let current_avg_frag = state.performance_metrics.avg_fragmentation_reduction
            .get(&strategy_key)
            .copied()
            .unwrap_or(0.1);
        let new_avg_frag = (current_avg_frag + frag_reduction) / 2.0;
        state.performance_metrics.avg_fragmentation_reduction.insert(strategy_key.clone(), new_avg_frag);

        // Update success rate
        let current_success_rate = state.performance_metrics.success_rate
            .get(&strategy_key)
            .copied()
            .unwrap_or(0.5);
        let success_increment = if performance.success { 0.1 } else { -0.1 };
        let new_success_rate = (current_success_rate + success_increment).max(0.0).min(1.0);
        state.performance_metrics.success_rate.insert(strategy_key, new_success_rate);
    }

    /// Adapt strategy based on performance history
    async fn adapt_strategy(&self) {
        let history = self.performance_history.read().await;

        if history.len() < self.config.min_samples_for_analysis {
            return;
        }

        let mut state = self.state.write().await;

        // Simple adaptation: switch to better performing strategy
        let current_strategy_key = format!("{:?}", state.current_strategy);
        let current_success_rate = state.performance_metrics.success_rate
            .get(&current_strategy_key)
            .copied()
            .unwrap_or(0.5);

        // Find best performing strategy
        let mut best_strategy = state.current_strategy;
        let mut best_success_rate = current_success_rate;

        for (strategy_str, &success_rate) in &state.performance_metrics.success_rate {
            if success_rate > best_success_rate {
                best_success_rate = success_rate;
                best_strategy = self.parse_strategy_from_string(strategy_str);
            }
        }

        // Only change if improvement is significant
        if best_success_rate > current_success_rate + 0.1 {
            state.current_strategy = best_strategy;
            state.last_change = Some(Instant::now());
            state.adaptation_cycles += 1;

            tracing::info!(
                "Adaptive strategy changed to {:?} (success rate: {:.2})",
                best_strategy,
                best_success_rate
            );
        }
    }

    /// Get current CPU usage (placeholder implementation)
    async fn get_current_cpu_usage(&self) -> f64 {
        // In a real implementation, this would query system CPU usage
        0.5
    }

    /// Get strategy performance metrics
    pub async fn get_performance_metrics(&self) -> StrategyPerformanceMetrics {
        let state = self.state.read().await;
        state.performance_metrics.clone()
    }

    /// Export strategy status for monitoring
    pub async fn export_status(&self) -> serde_json::Value {
        let state = self.state.read().await;
        let history = self.performance_history.read().await;

        serde_json::json!({
            "current_strategy": format!("{:?}", state.current_strategy),
            "last_change_seconds_ago": state.last_change.map(|t| t.elapsed().as_secs()).unwrap_or(0),
            "adaptation_cycles": state.adaptation_cycles,
            "history_size": history.len(),
            "performance_metrics": {
                "strategies_count": state.performance_metrics.avg_compaction_time.len(),
                "avg_fragmentation_reduction": state.performance_metrics.avg_fragmentation_reduction,
                "success_rates": state.performance_metrics.success_rate
            },
            "config": {
                "adaptive_enabled": self.config.adaptive_enabled,
                "history_window_seconds": self.config.history_window_seconds,
                "min_samples_for_analysis": self.config.min_samples_for_analysis
            }
        })
    }
}

/// System state analysis result
#[derive(Debug)]
struct SystemStateAnalysis {
    /// Current fragmentation ratio
    fragmentation_ratio: f64,

    /// Current memory pressure
    memory_pressure: f64,

    /// Memory utilization ratio
    memory_utilization: f64,

    /// Large workspace detected
    large_workspace: bool,

    /// High CPU usage detected
    high_cpu_usage: bool,
}

/// Performance history analysis result
#[derive(Debug)]
struct HistoryAnalysis {
    /// Best performing strategy
    best_strategy: super::large_workspace_compactor::CompactionStrategy,

    /// Average efficiency score
    avg_efficiency: f64,

    /// Overall success rate
    success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_strategy_creation() {
        let strategy = AdaptiveCompactionStrategy::new();
        let selected = strategy.select_strategy().await;

        // Should default to incremental when no history
        assert_eq!(selected, super::large_workspace_compactor::CompactionStrategy::Incremental);
    }

    #[tokio::test]
    async fn test_performance_recording() {
        let strategy = AdaptiveCompactionStrategy::new();

        let performance = StrategyPerformance {
            timestamp: Instant::now(),
            strategy: super::large_workspace_compactor::CompactionStrategy::Incremental,
            duration: Duration::from_millis(100),
            memory_freed: 1024 * 1024,
            fragmentation_before: 0.5,
            fragmentation_after: 0.2,
            success: true,
            cpu_usage: 0.4,
            memory_pressure: 0.6,
        };

        strategy.record_performance(performance).await;

        let metrics = strategy.get_performance_metrics().await;
        assert!(!metrics.avg_compaction_time.is_empty());
    }
}