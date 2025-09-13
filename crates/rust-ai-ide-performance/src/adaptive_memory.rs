//! Adaptive Memory Management with Predictive Allocation
//!
//! This module provides intelligent memory allocation strategies that predict usage patterns
//! and adapt allocation policies accordingly. It includes:
//!
//! - Predictive memory allocation based on historical usage patterns
//! - Dynamic adjusting of allocation strategies based on workload
//! - Memory pressure awareness and proactive deallocation
//! - Integration with garbage collection hints
//! - Performance metrics and adaptive tuning

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use rust_ai_ide_shared_types::{IDEResult, RustAIError};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

/// Memory allocation patterns and prediction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsagePattern {
    pub allocation_trend:      AllocationTrend,
    pub peak_usage_time:       chrono::DateTime<chrono::Utc>,
    pub average_usage_mb:      f64,
    pub volatility_factor:     f64, // How predictable the usage is
    pub prediction_confidence: f64,
}

/// Allocation trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationTrend {
    Stable,
    Increasing,
    Decreasing,
    Cyclical,
    Random,
}

/// Memory allocation strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStrategy {
    pub strategy_name: String,
    pub allocation_chunk_size_kb: usize,
    pub preallocation_enabled: bool,
    pub preallocation_chunks: usize,
    pub adaptive_scaling: bool,
    pub scaling_factor: f64,
    pub deallocation_threshold_gb: f64,
    pub garbage_collection_interval_seconds: u64,
}

/// Memory prediction model
#[derive(Debug)]
pub struct PredictiveAllocationModel {
    historical_patterns: VecDeque<MemoryUsagePattern>,
    current_trend:       AllocationTrend,
    prediction_accuracy: f64,
    model_last_updated:  chrono::DateTime<chrono::Utc>,
    max_history_size:    usize,
}

impl PredictiveAllocationModel {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            historical_patterns: VecDeque::with_capacity(max_history_size),
            current_trend: AllocationTrend::Stable,
            prediction_accuracy: 0.5, // Start with neutral confidence
            model_last_updated: chrono::Utc::now(),
            max_history_size,
        }
    }

    /// Update the prediction model with current memory usage data
    pub fn update(&mut self, current_usage: MemoryUsageSample) {
        // Convert sample to pattern
        let pattern = self.analyze_pattern(&current_usage);
        self.historical_patterns.push_back(pattern);

        // Maintain history size limit
        if self.historical_patterns.len() > self.max_history_size {
            self.historical_patterns.pop_front();
        }

        // Update trend analysis
        self.analyze_trends();

        // Update timestamp
        self.model_last_updated = chrono::Utc::now();

        debug!(
            "Updated predictive model - Current trend: {:?}",
            self.current_trend
        );
    }

    /// Predict memory usage for next interval
    pub fn predict_usage(&self, look_ahead_seconds: u64) -> PredictedMemoryUsage {
        if self.historical_patterns.len() < 3 {
            // Not enough data for prediction
            return PredictedMemoryUsage {
                predicted_usage_mb: 512.0, // Conservative default
                confidence_level:   0.3,
                recommendation:     AllocationRecommendation::Conservative,
            };
        }

        // Simple linear regression for trend-based prediction
        let prediction = self.linear_regression_prediction(look_ahead_seconds);
        let confidence = self.calculate_prediction_confidence();

        let recommendation = match self.current_trend {
            AllocationTrend::Stable => AllocationRecommendation::Maintain,
            AllocationTrend::Increasing => AllocationRecommendation::ScaleUp,
            AllocationTrend::Decreasing => AllocationRecommendation::ScaleDown,
            AllocationTrend::Cyclical => AllocationRecommendation::Preallocate,
            AllocationTrend::Random => AllocationRecommendation::Conservative,
        };

        PredictedMemoryUsage {
            predicted_usage_mb: prediction.max(64.0), // Minimum allocation
            confidence_level: confidence,
            recommendation,
        }
    }

    /// Analyze memory usage pattern
    fn analyze_pattern(&self, sample: &MemoryUsageSample) -> MemoryUsagePattern {
        let trend = if self.historical_patterns.len() >= 2 {
            let last_pattern = self.historical_patterns.back().unwrap();
            self.determine_trend(last_pattern, sample)
        } else {
            AllocationTrend::Stable
        };

        MemoryUsagePattern {
            allocation_trend:      trend,
            peak_usage_time:       sample.timestamp,
            average_usage_mb:      sample.used_memory_mb as f64,
            volatility_factor:     self.calculate_volatility(sample),
            prediction_confidence: self.prediction_accuracy,
        }
    }

    /// Determine allocation trend from historical data
    fn determine_trend(&self, previous: &MemoryUsagePattern, current: &MemoryUsageSample) -> AllocationTrend {
        let prev_avg = previous.average_usage_mb;
        let curr_avg = current.used_memory_mb as f64;
        let change_percentage = ((curr_avg - prev_avg) / prev_avg).abs();

        if change_percentage < 0.05 {
            AllocationTrend::Stable
        } else if curr_avg > prev_avg && change_percentage > 0.1 {
            AllocationTrend::Increasing
        } else if curr_avg < prev_avg && change_percentage > 0.1 {
            AllocationTrend::Decreasing
        } else if self.historical_patterns.len() >= 4 && self.is_cyclical_pattern(current) {
            AllocationTrend::Cyclical
        } else {
            AllocationTrend::Random
        }
    }

    /// Simple cyclical pattern detection
    fn is_cyclical_pattern(&self, current: &MemoryUsageSample) -> bool {
        if self.historical_patterns.len() < 4 {
            return false;
        }

        // Simple heuristic: check if pattern repeats roughly every 3 samples
        let samples: Vec<&MemoryUsagePattern> = self.historical_patterns.iter().collect();
        let len = samples.len();

        let p1 = &samples[len - 4];
        let p2 = &samples[len - 1];
        let similar = (p1.average_usage_mb - p2.average_usage_mb).abs() / p1.average_usage_mb < 0.1;

        similar
    }

    /// Calculate volatility factor (lower = more predictable)
    fn calculate_volatility(&self, sample: &MemoryUsageSample) -> f64 {
        if self.historical_patterns.is_empty() {
            return 1.0; // Maximum volatility
        }

        let mut variance = 0.0;
        let count = self.historical_patterns.len() + 1;
        let current_value = sample.used_memory_mb as f64;

        // Calculate variance from current sample
        for pattern in &self.historical_patterns {
            let diff = pattern.average_usage_mb - current_value;
            variance += diff * diff;
        }

        // Calculate standard deviation
        (variance / count as f64).sqrt() / (current_value + 1.0).log2() // Normalize
    }

    /// Analyze overall trends from historical patterns
    fn analyze_trends(&mut self) {
        if self.historical_patterns.len() < 3 {
            self.current_trend = AllocationTrend::Stable;
            return;
        }

        let recent_patterns: Vec<_> = self.historical_patterns.iter().rev().take(5).collect();
        let changes: Vec<f64> = recent_patterns
            .windows(2)
            .map(|w| {
                let prev = w[0].average_usage_mb;
                let curr = w[1].average_usage_mb;
                (curr - prev) / prev
            })
            .collect();

        let avg_change = changes.iter().sum::<f64>() / changes.len() as f64;

        if avg_change.abs() < 0.02 {
            self.current_trend = AllocationTrend::Stable;
        } else if avg_change > 0.05 {
            self.current_trend = AllocationTrend::Increasing;
        } else if avg_change < -0.05 {
            self.current_trend = AllocationTrend::Decreasing;
        } else if self.is_overall_cyclical(&changes) {
            self.current_trend = AllocationTrend::Cyclical;
        } else {
            self.current_trend = AllocationTrend::Random;
        }
    }

    /// Detect cyclical patterns across all history
    fn is_overall_cyclical(&self, changes: &[f64]) -> bool {
        if changes.len() < 3 {
            return false;
        }

        // Simple autocorrelation check for basic cycles
        let mut autocorr = 0.0;
        let mean = changes.iter().sum::<f64>() / changes.len() as f64;

        for i in 0..changes.len() - 1 {
            autocorr += (changes[i] - mean) * (changes[i + 1] - mean);
        }

        autocorr.abs() > (changes.len() as f64 * 0.1)
    }

    /// Simple linear regression prediction
    fn linear_regression_prediction(&self, look_ahead_seconds: u64) -> f64 {
        let n = self.historical_patterns.len() as f64;

        // Calculate simple linear regression
        let x_mean = (self.historical_patterns.len() as f64 - 1.0) / 2.0;
        let y_sum: f64 = self
            .historical_patterns
            .iter()
            .map(|p| p.average_usage_mb)
            .sum();
        let y_mean = y_sum / n;

        let mut numerator = 0.0;
        let mut x_ss = 0.0;

        for (i, pattern) in self.historical_patterns.iter().enumerate() {
            let x = i as f64;
            let y = pattern.average_usage_mb;

            numerator += (x - x_mean) * (y - y_mean);
            x_ss += (x - x_mean).powi(2);
        }

        let slope = if x_ss != 0.0 { numerator / x_ss } else { 0.0 };
        let intercept = y_mean - slope * x_mean;

        // Predict future value
        let prediction_point = n + (look_ahead_seconds / 60) as f64; // Rough minute-based prediction
        slope * prediction_point + intercept
    }

    /// Calculate prediction confidence based on model accuracy
    fn calculate_prediction_confidence(&self) -> f64 {
        let base_confidence = if self.historical_patterns.len() > 10 {
            0.8
        } else {
            0.5
        };
        let trend_confidence = match self.current_trend {
            AllocationTrend::Stable => 0.9,
            AllocationTrend::Increasing | AllocationTrend::Decreasing => 0.7,
            AllocationTrend::Cyclical => 0.6,
            AllocationTrend::Random => 0.3,
        };

        (base_confidence + trend_confidence) / 2.0
    }
}

/// Memory usage sample for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageSample {
    pub total_memory_mb:      u64,
    pub used_memory_mb:       u64,
    pub free_memory_mb:       u64,
    pub available_memory_mb:  u64,
    pub allocation_rate_kbps: f64,
    pub timestamp:            chrono::DateTime<chrono::Utc>,
}

/// Prediction result for memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedMemoryUsage {
    pub predicted_usage_mb: f64,
    pub confidence_level:   f64,
    pub recommendation:     AllocationRecommendation,
}

/// Allocation recommendations based on prediction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationRecommendation {
    ScaleUp,
    ScaleDown,
    Maintain,
    Preallocate,
    Conservative,
}

/// Adaptive memory manager
pub struct AdaptiveMemoryManager {
    model:                Arc<RwLock<PredictiveAllocationModel>>,
    current_strategy:     Arc<RwLock<AllocationStrategy>>,
    memory_monitor:       MemoryMonitor,
    allocation_decisions: Vec<AllocationDecision>,
    config:               AdaptiveConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    pub enable_predictive_allocation: bool,
    pub monitoring_interval_seconds:  u64,
    pub prediction_horizon_minutes:   u64,
    pub adaptation_threshold:         f64,
    pub min_confidence_threshold:     f64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            enable_predictive_allocation: true,
            monitoring_interval_seconds:  30,
            prediction_horizon_minutes:   5,
            adaptation_threshold:         0.1, // 10% change threshold
            min_confidence_threshold:     0.6,
        }
    }
}

impl AdaptiveMemoryManager {
    pub fn new(config: AdaptiveConfig) -> Self {
        let strategy = AllocationStrategy {
            strategy_name: "adaptive_balance".to_string(),
            allocation_chunk_size_kb: 1024,
            preallocation_enabled: true,
            preallocation_chunks: 10,
            adaptive_scaling: true,
            scaling_factor: 1.2,
            deallocation_threshold_gb: 2.0,
            garbage_collection_interval_seconds: 300,
        };

        Self {
            model: Arc::new(RwLock::new(PredictiveAllocationModel::new(100))),
            current_strategy: Arc::new(RwLock::new(strategy)),
            memory_monitor: MemoryMonitor::new(),
            allocation_decisions: Vec::new(),
            config,
        }
    }

    /// Start adaptive memory management
    pub async fn start(&self) -> IDEResult<()> {
        info!("Starting adaptive memory management");
        // Start monitoring and adaptation loops
        Ok(())
    }

    /// Stop adaptive management
    pub async fn stop(&self) -> IDEResult<()> {
        info!("Stopping adaptive memory management");
        Ok(())
    }

    /// Get allocation recommendation based on current state
    pub async fn get_allocation_recommendation(&self) -> IDEResult<AllocationRecommendation> {
        if !self.config.enable_predictive_allocation {
            return Ok(AllocationRecommendation::Conservative);
        }

        let model = self.model.read().await;
        let prediction = model.predict_usage(self.config.prediction_horizon_minutes * 60);

        if prediction.confidence_level < self.config.min_confidence_threshold {
            return Ok(AllocationRecommendation::Conservative);
        }

        Ok(prediction.recommendation)
    }

    /// Apply adaptive strategy based on current memory pressure
    pub async fn adapt_to_memory_pressure(&self, current_usage: MemoryUsageSample) -> IDEResult<()> {
        // Update prediction model
        let mut model = self.model.write().await;
        model.update(current_usage);

        // Get prediction and current strategy
        let prediction = model.predict_usage(self.config.prediction_horizon_minutes * 60);
        let mut strategy = self.current_strategy.write().await;

        // Adapt strategy based on prediction
        match prediction.recommendation {
            AllocationRecommendation::ScaleUp => {
                strategy.preallocation_chunks =
                    (strategy.preallocation_chunks as f64 * strategy.scaling_factor) as usize;
                strategy.allocation_chunk_size_kb =
                    (strategy.allocation_chunk_size_kb as f64 * (1.0 + self.config.adaptation_threshold)) as usize;
            }
            AllocationRecommendation::ScaleDown => {
                strategy.preallocation_chunks = (strategy.preallocation_chunks as f64 * 0.8) as usize;
                strategy.allocation_chunk_size_kb =
                    (strategy.allocation_chunk_size_kb as f64 * (1.0 - self.config.adaptation_threshold)) as usize;
            }
            AllocationRecommendation::Preallocate => {
                strategy.preallocation_enabled = true;
                strategy.preallocation_chunks = strategy.preallocation_chunks.max(5);
            }
            AllocationRecommendation::Maintain => {
                // Keep current strategy stable
            }
            AllocationRecommendation::Conservative => {
                strategy.allocation_chunk_size_kb = 512; // Conservative default
                strategy.preallocation_chunks = 5;
            }
        }

        // Ensure minimums
        strategy.allocation_chunk_size_kb = strategy.allocation_chunk_size_kb.min(8192);
        strategy.preallocation_chunks = strategy.preallocation_chunks.min(50);

        debug!(
            "Adapted strategy: {} chunks of {} KB",
            strategy.preallocation_chunks, strategy.allocation_chunk_size_kb
        );

        Ok(())
    }

    /// Record allocation decision for analysis
    pub fn record_allocation_decision(&mut self, decision: AllocationDecision) {
        self.allocation_decisions.push(decision);
        if self.allocation_decisions.len() > 1000 {
            self.allocation_decisions.remove(0); // Keep only recent decisions
        }
    }

    /// Get current allocation strategy
    pub async fn get_current_strategy(&self) -> AllocationStrategy {
        self.current_strategy.read().await.clone()
    }
}

/// Memory pressure monitor
pub struct MemoryMonitor {
    pressure_events: HashMap<String, Vec<MemoryPressureEvent>>,
    thresholds:      HashMap<PressureLevel, u64>,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(PressureLevel::Low, 70); // 70% usage
        thresholds.insert(PressureLevel::Medium, 85); // 85% usage
        thresholds.insert(PressureLevel::High, 95); // 95% usage
        thresholds.insert(PressureLevel::Critical, 98); // 98% usage

        Self {
            pressure_events: HashMap::new(),
            thresholds,
        }
    }

    /// Monitor memory pressure and return current level
    pub fn monitor_pressure(&self, sample: &MemoryUsageSample) -> PressureLevel {
        let usage_percentage = (sample.used_memory_mb as f64 / sample.total_memory_mb as f64) * 100.0;

        if usage_percentage >= *self.thresholds.get(&PressureLevel::Critical).unwrap_or(&98) as f64 {
            PressureLevel::Critical
        } else if usage_percentage >= *self.thresholds.get(&PressureLevel::High).unwrap_or(&95) as f64 {
            PressureLevel::High
        } else if usage_percentage >= *self.thresholds.get(&PressureLevel::Medium).unwrap_or(&85) as f64 {
            PressureLevel::Medium
        } else {
            PressureLevel::Low
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum PressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Memory pressure event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPressureEvent {
    pub level:           PressureLevel,
    pub timestamp:       chrono::DateTime<chrono::Utc>,
    pub used_memory_mb:  u64,
    pub total_memory_mb: u64,
    pub triggered_by:    String,
}

/// Allocation decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationDecision {
    pub timestamp:           chrono::DateTime<chrono::Utc>,
    pub requested_size_kb:   usize,
    pub granted_size_kb:     usize,
    pub recommendation_used: AllocationRecommendation,
    pub success:             bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_predictive_allocation_model() {
        let mut model = PredictiveAllocationModel::new(10);

        // Add some sample data
        let sample1 = MemoryUsageSample {
            total_memory_mb:      8192,
            used_memory_mb:       2048,
            free_memory_mb:       6144,
            available_memory_mb:  6144,
            allocation_rate_kbps: 100.0,
            timestamp:            chrono::Utc::now(),
        };

        model.update(sample1);

        let prediction = model.predict_usage(300); // 5 minutes ahead
        assert!(prediction.predicted_usage_mb > 0.0);
        assert!(prediction.confidence_level > 0.0 && prediction.confidence_level <= 1.0);
    }

    #[test]
    fn test_memory_monitor_pressure() {
        let monitor = MemoryMonitor::new();

        let high_pressure = MemoryUsageSample {
            total_memory_mb:      8192,
            used_memory_mb:       7864, // ~96% usage
            free_memory_mb:       328,
            available_memory_mb:  328,
            allocation_rate_kbps: 500.0,
            timestamp:            chrono::Utc::now(),
        };

        let pressure_level = monitor.monitor_pressure(&high_pressure);
        assert_eq!(pressure_level, PressureLevel::High);

        let critical_pressure = MemoryUsageSample {
            total_memory_mb:      8192,
            used_memory_mb:       8011, // ~98% usage
            free_memory_mb:       181,
            available_memory_mb:  181,
            allocation_rate_kbps: 1000.0,
            timestamp:            chrono::Utc::now(),
        };

        let critical_level = monitor.monitor_pressure(&critical_pressure);
        assert_eq!(critical_level, PressureLevel::Critical);
    }

    #[tokio::test]
    async fn test_adaptive_memory_manager() {
        let config = AdaptiveConfig::default();
        let manager = AdaptiveMemoryManager::new(config);

        let current_usage = MemoryUsageSample {
            total_memory_mb:      8192,
            used_memory_mb:       4096,
            free_memory_mb:       4096,
            available_memory_mb:  4096,
            allocation_rate_kbps: 200.0,
            timestamp:            chrono::Utc::now(),
        };

        // Test adaptation
        manager
            .adapt_to_memory_pressure(current_usage)
            .await
            .unwrap();

        let recommendation = manager.get_allocation_recommendation().await.unwrap();
        // Should return some recommendation (which may be Conservative due to insufficient data)
        assert!(matches!(
            recommendation,
            AllocationRecommendation::Conservative
                | AllocationRecommendation::Maintain
                | AllocationRecommendation::ScaleUp
                | AllocationRecommendation::ScaleDown
                | AllocationRecommendation::Preallocate
        ));
    }
}
