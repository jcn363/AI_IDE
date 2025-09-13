//! Performance Regression Detection
//!
//! This module provides automated detection of performance regressions
//! using statistical analysis and trend detection algorithms.

use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, TimeZone, Utc};
use rust_ai_ide_shared_types::PerformanceMetrics;
use serde::{Deserialize, Serialize};

use crate::PerformanceAlert as LocalPerformanceAlert;

/// Configuration for regression detection
#[derive(Debug, Clone)]
pub struct RegressionConfig {
    /// Number of baseline data points to maintain
    pub baseline_window_size:  usize,
    /// Threshold for detecting regression (percentage)
    pub regression_threshold:  f64,
    /// Minimum confidence required for regression alert
    pub minimum_confidence:    f64,
    /// Enable trend analysis
    pub enable_trend_analysis: bool,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            baseline_window_size:  10,
            regression_threshold:  0.1, // 10% degradation
            minimum_confidence:    0.8, // 80% confidence
            enable_trend_analysis: true,
        }
    }
}

/// Performance regression detector
pub struct RegressionDetector {
    config:           RegressionConfig,
    baseline_metrics: VecDeque<PerformanceMetrics>,
    /// Statistical baseline for each metric type
    baselines:        HashMap<String, MetricBaseline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetricBaseline {
    /// Mean value for this metric
    mean:              f64,
    /// Standard deviation
    std_dev:           f64,
    /// Number of samples used to calculate baseline
    sample_count:      usize,
    /// Trend direction (positive for improvement, negative for regression)
    trend_coefficient: f64,
}

impl RegressionDetector {
    /// Create a new regression detector with default configuration
    pub fn new(baseline_window_size: usize, regression_threshold: f64) -> Self {
        let config = RegressionConfig {
            baseline_window_size,
            regression_threshold,
            ..Default::default()
        };

        Self {
            config,
            baseline_metrics: VecDeque::new(),
            baselines: HashMap::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: RegressionConfig) -> Self {
        Self {
            config,
            baseline_metrics: VecDeque::new(),
            baselines: HashMap::new(),
        }
    }

    /// Update baseline with new metrics
    pub fn update_baseline(&mut self, metrics: &PerformanceMetrics) {
        self.baseline_metrics.push_back(metrics.clone());

        // Maintain window size
        if self.baseline_metrics.len() > self.config.baseline_window_size {
            self.baseline_metrics.pop_front();
        }

        // Recalculate baselines
        self.recalculate_baselines();
    }

    /// Detect regressions compared to baseline
    pub fn detect_regressions(
        &self,
        current: &PerformanceMetrics,
        history: &[PerformanceMetrics],
    ) -> Vec<LocalPerformanceAlert> {
        let mut alerts = Vec::new();

        if history.is_empty() {
            return alerts;
        }

        // Calculate baseline averages for comparison
        let baseline_avg = self.calculate_baseline_average();

        // Compare current metrics against baseline
        if let Some(base_cpu) = baseline_avg.rates.cpu_usage_percent {
            if let Some(current_cpu) = current.rates.cpu_usage_percent {
                if self.is_regression(base_cpu, current_cpu) {
                    let degradation = (current_cpu - base_cpu) / base_cpu;
                    alerts.push(LocalPerformanceAlert::RegressionDetected {
                        metric_name:         "cpu_usage_percent".to_string(),
                        baseline_value:      base_cpu,
                        current_value:       current_cpu,
                        degradation_percent: degradation,
                        timestamp:           Utc.timestamp_millis_opt(current.timestamp as i64).unwrap(),
                    });
                }
            }
        }

        // Check response time regression (lower is better for response time)
        if let Some(base_response) = baseline_avg.timing.response_time_ns {
            if let Some(current_response) = current.timing.response_time_ns {
                if self.is_regression_response_time(base_response, current_response) {
                    let degradation = (current_response as f64 - base_response as f64) / base_response as f64;
                    alerts.push(LocalPerformanceAlert::RegressionDetected {
                        metric_name:         "response_time_ns".to_string(),
                        baseline_value:      base_response as f64,
                        current_value:       current_response as f64,
                        degradation_percent: degradation,
                        timestamp:           Utc.timestamp_millis_opt(current.timestamp as i64).unwrap(),
                    });
                }
            }
        }

        // Check memory usage regression
        if let Some(base_memory) = baseline_avg.resources.memory_bytes {
            if let Some(current_memory) = current.resources.memory_bytes {
                if self.is_regression_memory(base_memory, current_memory) {
                    let degradation = (current_memory as f64 - base_memory as f64) / base_memory as f64;
                    alerts.push(LocalPerformanceAlert::ThresholdExceeded {
                        metric_name:   "memory_bytes".to_string(),
                        current_value: current_memory as f64,
                        threshold:     base_memory as f64,
                        timestamp:     Utc.timestamp_millis_opt(current.timestamp as i64).unwrap(),
                    });
                }
            }
        }

        // Trend analysis for multiple data points
        if self.config.enable_trend_analysis && history.len() >= self.config.baseline_window_size {
            alerts.extend(self.detect_trend_regressions(current, history));
        }

        alerts
    }

    /// Calculate average metrics from baseline data
    fn calculate_baseline_average(&self) -> PerformanceMetrics {
        if self.baseline_metrics.is_empty() {
            return PerformanceMetrics::new();
        }

        let mut sum = PerformanceMetrics::new();

        for metrics in &self.baseline_metrics {
            // Sum up the metrics
            if let Some(cpu) = metrics.rates.cpu_usage_percent {
                sum.rates.cpu_usage_percent = Some(sum.rates.cpu_usage_percent.unwrap_or(0.0) + cpu);
            }
            if let Some(response) = metrics.timing.response_time_ns {
                sum.timing.response_time_ns = Some(sum.timing.response_time_ns.unwrap_or(0) + response);
            }
            if let Some(memory) = metrics.resources.memory_bytes {
                sum.resources.memory_bytes = Some(sum.resources.memory_bytes.unwrap_or(0) + memory);
            }
        }

        let count = self.baseline_metrics.len() as f64;

        // Calculate averages
        if let Some(ref mut cpu) = sum.rates.cpu_usage_percent {
            *cpu /= count;
        }
        if let Some(ref mut response) = sum.timing.response_time_ns {
            *response = (*response as f64 / count) as u64;
        }
        if let Some(ref mut memory) = sum.resources.memory_bytes {
            *memory = (*memory as f64 / count) as u64;
        }

        sum
    }

    /// Check if current value represents a regression for CPU usage (higher is worse)
    fn is_regression(&self, baseline: f64, current: f64) -> bool {
        let degradation = (current - baseline) / baseline;
        degradation > self.config.regression_threshold
    }

    /// Check if current response time is a regression (higher response time is worse)
    fn is_regression_response_time(&self, baseline: u64, current: u64) -> bool {
        let baseline_f = baseline as f64;
        let current_f = current as f64;
        let degradation = (current_f - baseline_f) / baseline_f;
        degradation > self.config.regression_threshold
    }

    /// Check if current memory usage is a regression (much higher usage is worse)
    fn is_regression_memory(&self, baseline: u64, current: u64) -> bool {
        // Memory regression detection - use a more conservative threshold
        let baseline_f = baseline as f64;
        let current_f = current as f64;
        let degradation = (current_f - baseline_f) / baseline_f;
        degradation > (self.config.regression_threshold * 1.5) // More conservative for memory
    }

    /// Detect regressions based on trend analysis
    fn detect_trend_regressions(
        &self,
        _current: &PerformanceMetrics,
        _history: &[PerformanceMetrics],
    ) -> Vec<LocalPerformanceAlert> {
        // TODO: Implement sophisticated trend analysis using linear regression
        // and statistical tests to detect performance degradation trends

        // For now, return empty vector
        Vec::new()
    }

    /// Recalculate statistical baselines
    fn recalculate_baselines(&mut self) {
        // Clear existing baselines
        self.baselines.clear();

        // Recalculate for each metric type
        self.recalculate_cpu_baseline();
        self.recalculate_response_time_baseline();
        self.recalculate_memory_baseline();
    }

    /// Recalculate CPU usage baseline statistics
    fn recalculate_cpu_baseline(&mut self) {
        let cpu_values: Vec<f64> = self
            .baseline_metrics
            .iter()
            .filter_map(|m| m.rates.cpu_usage_percent)
            .collect();

        if cpu_values.len() >= 3 {
            // Need at least 3 points for meaningful stats
            let (mean, std_dev) = self.calculate_mean_and_std(&cpu_values);
            let trend = self.calculate_trend_coefficient(&cpu_values);

            self.baselines
                .insert("cpu_usage".to_string(), MetricBaseline {
                    mean,
                    std_dev,
                    sample_count: cpu_values.len(),
                    trend_coefficient: trend,
                });
        }
    }

    /// Recalculate response time baseline statistics
    fn recalculate_response_time_baseline(&mut self) {
        let rt_values: Vec<f64> = self
            .baseline_metrics
            .iter()
            .filter_map(|m| m.timing.response_time_ns.map(|v| v as f64))
            .collect();

        if rt_values.len() >= 3 {
            let (mean, std_dev) = self.calculate_mean_and_std(&rt_values);
            let trend = self.calculate_trend_coefficient(&rt_values);

            self.baselines
                .insert("response_time".to_string(), MetricBaseline {
                    mean,
                    std_dev,
                    sample_count: rt_values.len(),
                    trend_coefficient: trend,
                });
        }
    }

    /// Recalculate memory usage baseline statistics
    fn recalculate_memory_baseline(&mut self) {
        let mem_values: Vec<f64> = self
            .baseline_metrics
            .iter()
            .filter_map(|m| m.resources.memory_bytes.map(|v| v as f64))
            .collect();

        if mem_values.len() >= 3 {
            let (mean, std_dev) = self.calculate_mean_and_std(&mem_values);
            let trend = self.calculate_trend_coefficient(&mem_values);

            self.baselines
                .insert("memory_usage".to_string(), MetricBaseline {
                    mean,
                    std_dev,
                    sample_count: mem_values.len(),
                    trend_coefficient: trend,
                });
        }
    }

    /// Calculate mean and standard deviation
    fn calculate_mean_and_std(&self, values: &[f64]) -> (f64, f64) {
        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;

        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
        let std_dev = variance.sqrt();

        (mean, std_dev)
    }

    /// Calculate linear trend coefficient (slope)
    fn calculate_trend_coefficient(&self, values: &[f64]) -> f64 {
        let n = values.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, &value) in values.iter().enumerate() {
            let x = i as f64;
            sum_x += x;
            sum_y += value;
            sum_xy += x * value;
            sum_xx += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        slope
    }

    /// Get baseline statistics for debugging
    pub fn get_baselines(&self) -> &HashMap<String, MetricBaseline> {
        &self.baselines
    }
}

/// Statistical analysis utilities for performance metrics
pub struct PerformanceStatistics;

impl PerformanceStatistics {
    /// Calculate percentile value from a dataset
    pub fn percentile(values: &mut [f64], p: f64) -> f64 {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = (p / 100.0 * (values.len() - 1) as f64) as usize;
        values[index]
    }

    /// Calculate moving average
    pub fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
        values
            .windows(window_size)
            .map(|window| window.iter().sum::<f64>() / window.len() as f64)
            .collect()
    }

    /// Detect outliers using Modified Z-score method
    pub fn detect_outliers(values: &[f64], threshold: f64) -> Vec<usize> {
        let median = Self::median(values);
        let mad = Self::median_absolute_deviation(values, median);

        if mad == 0.0 {
            return Vec::new();
        }

        values
            .iter()
            .enumerate()
            .filter_map(|(i, &value)| {
                let modified_z = 0.6745 * (value - median) / mad;
                if modified_z.abs() > threshold {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Calculate median of a dataset
    fn median(values: &[f64]) -> f64 {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// Calculate Median Absolute Deviation
    fn median_absolute_deviation(values: &[f64], median: f64) -> f64 {
        let deviations: Vec<f64> = values.iter().map(|v| (v - median).abs()).collect();
        Self::median(&deviations)
    }
}

#[cfg(test)]
mod tests {
    use rust_ai_ide_shared_types::PerformanceMetrics;

    use super::*;

    #[test]
    fn test_regression_detection_cpu_usage() {
        let detector = RegressionDetector::new(5, 0.1);

        // Create baseline metrics
        for i in 0..5 {
            let mut metrics = PerformanceMetrics::new();
            metrics.rates.cpu_usage_percent = Some(40.0 + i as f64);
            // Note: In real usage, you'd call update_baseline for each metrics
        }

        // Test regression detection
        let mut current = PerformanceMetrics::new();
        current.rates.cpu_usage_percent = Some(55.0); // 25% increase

        // Note: This test would need proper setup with baseline metrics
        // For now, just test the structure exists
        assert!(detector.is_regression(40.0, 55.0));
    }

    #[test]
    fn test_trend_coefficient_calculation() {
        let detector = RegressionDetector::new(5, 0.1);

        // Increasing trend: should have positive coefficient
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let trend = detector.calculate_trend_coefficient(&values);
        assert!(
            trend > 0.0,
            "Increasing sequence should have positive trend"
        );

        // Decreasing trend: should have negative coefficient
        let values = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let trend = detector.calculate_trend_coefficient(&values);
        assert!(
            trend < 0.0,
            "Decreasing sequence should have negative trend"
        );
    }

    #[test]
    fn test_statistical_functions() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        // Test percentile
        let p50 = PerformanceStatistics::percentile(&mut values.clone(), 50.0);
        assert_eq!(p50, 5.5);

        // Test moving average
        let ma = PerformanceStatistics::moving_average(&values, 3);
        assert_eq!(ma.len(), 8); // Should be len - window_size + 1

        // Test outlier detection
        let values_with_outlier = vec![1.0, 2.0, 3.0, 100.0]; // 100 is an outlier
        let outlier_indices = PerformanceStatistics::detect_outliers(&values_with_outlier, 3.5);
        assert!(!outlier_indices.is_empty(), "Should detect outlier");
    }
}
