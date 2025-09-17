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
    /// Performance statistics for advanced analysis
    stats:            PerformanceStatistics,
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
    /// Performance statistics for advanced analysis
    stats: PerformanceStatistics,

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
            stats: PerformanceStatistics::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: RegressionConfig) -> Self {
        Self {
            config,
            baseline_metrics: VecDeque::new(),
            baselines: HashMap::new(),
            stats: PerformanceStatistics::default(),
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
        current: &PerformanceMetrics,
        history: &[PerformanceMetrics],
    ) -> Vec<LocalPerformanceAlert> {
        let mut alerts = Vec::new();

        // Analyze each metric type for regressions
        if let Some(alert) = self.analyze_metric_trend("cpu_usage", history, current.rates.cpu_usage_percent) {
            alerts.push(alert);
        }

        if let Some(alert) = self.analyze_metric_trend("response_time", history, current.timing.response_time_ns.map(|v| v as f64)) {
            alerts.push(alert);
        }

        if let Some(alert) = self.analyze_metric_trend("memory_usage", history, current.resources.memory_bytes.map(|v| v as f64)) {
            alerts.push(alert);
        }

        alerts
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

    /// Analyze metric trends using sophisticated statistical methods
    fn analyze_metric_trend(&self, metric_name: &str, history: &[PerformanceMetrics], current_value: Option<f64>) -> Option<LocalPerformanceAlert> {
        if history.len() < 5 || current_value.is_none() {
            return None; // Need minimum data for meaningful analysis
        }

        let values = self.extract_metric_values(metric_name, history);
        let current = current_value.unwrap();

        if values.len() < 5 {
            return None;
        }

        // Perform linear regression analysis
        let regression = self.perform_linear_regression(&values);

        // Calculate statistical significance
        let significance = self.calculate_statistical_significance(&values, &regression);

        // Check for regression trend
        if self.is_regression_trend(&regression, significance) {
            // Calculate forecasted value
            let forecast = self.forecast_performance(&regression, values.len() as f64 + 1.0);

            // Generate root cause suggestions
            let suggestions = self.generate_root_cause_suggestions(metric_name, &regression, significance);
    
            // Generate comprehensive report
            let mut report = self.stats.generate_regression_report(
                metric_name,
                &values,
                &regression,
                significance,
            );
            report.suggestions = suggestions.clone();
            self.stats.regression_reports.push(report);
    
            return Some(LocalPerformanceAlert::RegressionDetected {
                metric_name: metric_name.to_string(),
                baseline_value: regression.intercept + regression.slope * (values.len() as f64 - 1.0),
                current_value: current,
                degradation_percent: (current - regression.predicted_mean) / regression.predicted_mean,
                timestamp: Utc.timestamp_millis_opt(history.last().unwrap().timestamp as i64).unwrap(),
            });
        }

        None
    }

    /// Extract metric values from history for analysis
    fn extract_metric_values(&self, metric_name: &str, history: &[PerformanceMetrics]) -> Vec<f64> {
        match metric_name {
            "cpu_usage" => history.iter()
                .filter_map(|m| m.rates.cpu_usage_percent)
                .collect(),
            "response_time" => history.iter()
                .filter_map(|m| m.timing.response_time_ns.map(|v| v as f64))
                .collect(),
            "memory_usage" => history.iter()
                .filter_map(|m| m.resources.memory_bytes.map(|v| v as f64))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Perform linear regression on time series data
    fn perform_linear_regression(&self, values: &[f64]) -> RegressionResult {
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
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared for goodness of fit
        let y_mean = sum_y / n;
        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;

        for (i, &value) in values.iter().enumerate() {
            let x = i as f64;
            let predicted = slope * x + intercept;
            ss_res += (value - predicted).powi(2);
            ss_tot += (value - y_mean).powi(2);
        }

        let r_squared = if ss_tot != 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };

        RegressionResult {
            slope,
            intercept,
            r_squared,
            predicted_mean: y_mean,
        }
    }

    /// Calculate statistical significance of regression
    fn calculate_statistical_significance(&self, values: &[f64], regression: &RegressionResult) -> f64 {
        let n = values.len() as f64;
        if n < 3.0 {
            return 0.0;
        }

        // Calculate standard error of slope
        let mut sum_squared_errors = 0.0;
        for (i, &value) in values.iter().enumerate() {
            let x = i as f64;
            let predicted = regression.slope * x + regression.intercept;
            sum_squared_errors += (value - predicted).powi(2);
        }

        let se_slope = (sum_squared_errors / (n - 2.0)).sqrt() /
                      ((n * (n - 1.0) * (n - 1.0)).sqrt());

        if se_slope == 0.0 {
            return 1.0;
        }

        // T-statistic for slope
        let t_stat = regression.slope.abs() / se_slope;

        // Approximate p-value using t-distribution approximation
        self.approximate_p_value(t_stat, n - 2.0)
    }

    /// Approximate p-value from t-statistic
    fn approximate_p_value(&self, t_stat: f64, df: f64) -> f64 {
        // Simplified approximation for p-value
        let abs_t = t_stat.abs();
        if abs_t > 3.0 {
            return 0.001; // Very significant
        } else if abs_t > 2.0 {
            return 0.01; // Significant
        } else if abs_t > 1.5 {
            return 0.05; // Moderately significant
        } else {
            return 0.1; // Not significant
        }
    }

    /// Check if regression indicates performance degradation
    fn is_regression_trend(&self, regression: &RegressionResult, significance: f64) -> bool {
        // For response time and memory: positive slope indicates degradation
        // For CPU usage: depends on context, but generally upward trend is concerning
        regression.slope > 0.0 && significance < 0.05 && regression.r_squared > 0.5
    }

    /// Forecast future performance based on regression model
    fn forecast_performance(&self, regression: &RegressionResult, future_point: f64) -> f64 {
        regression.slope * future_point + regression.intercept
    }

    /// Generate root cause analysis suggestions
    fn generate_root_cause_suggestions(&self, metric_name: &str, regression: &RegressionResult, significance: f64) -> Vec<String> {
        let mut suggestions = Vec::new();

        if regression.slope > 0.0 {
            match metric_name {
                "cpu_usage" => {
                    suggestions.push("Check for CPU-intensive operations or memory leaks".to_string());
                    suggestions.push("Review recent code changes for computational complexity increases".to_string());
                    suggestions.push("Analyze thread contention and synchronization bottlenecks".to_string());
                    if significance < 0.01 {
                        suggestions.push("Consider optimizing algorithms or enabling parallel processing".to_string());
                        suggestions.push("Implement CPU affinity and NUMA-aware memory allocation".to_string());
                    }
                }
                "response_time" => {
                    suggestions.push("Investigate database query performance or network latency".to_string());
                    suggestions.push("Check for blocking operations or thread contention".to_string());
                    suggestions.push("Review I/O operations and system call efficiency".to_string());
                    if significance < 0.01 {
                        suggestions.push("Consider implementing caching or async processing".to_string());
                        suggestions.push("Optimize database indexes and connection pooling".to_string());
                    }
                }
                "memory_usage" => {
                    suggestions.push("Monitor for memory leaks or excessive object allocation".to_string());
                    suggestions.push("Review data structure usage and consider streaming for large datasets".to_string());
                    suggestions.push("Check for memory fragmentation and allocation patterns".to_string());
                    if significance < 0.01 {
                        suggestions.push("Implement memory profiling and garbage collection optimization".to_string());
                        suggestions.push("Consider memory-mapped files or off-heap storage".to_string());
                    }
                }
                _ => {}
            }

            // Add general suggestions based on trend strength
            if regression.r_squared > 0.9 {
                suggestions.push("Strong correlation detected - focus on recent changes".to_string());
            } else if regression.r_squared > 0.7 {
                suggestions.push("Moderate correlation - investigate gradual performance degradation".to_string());
            }
        }

        suggestions
    }

    /// Generate comprehensive performance regression report
    pub fn generate_performance_report(&self) -> String {
        self.stats.generate_performance_report(&self.stats.regression_reports)
    }

    /// Export regression analysis data as JSON
    pub fn export_regression_data(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.stats.export_reports()
    }

    /// Get current regression reports
    pub fn get_regression_reports(&self) -> &[RegressionReport] {
        &self.stats.regression_reports
    }

    /// Clear old regression reports
    pub fn clear_old_reports(&mut self, max_age_days: i64) {
        let cutoff = Utc::now() - chrono::Duration::days(max_age_days);
        self.stats.regression_reports.retain(|report| report.timestamp > cutoff);
    }
}

/// Result of linear regression analysis
#[derive(Debug, Clone)]
struct RegressionResult {
    slope: f64,
    intercept: f64,
    r_squared: f64,
    predicted_mean: f64,
}

/// Statistical analysis utilities for performance metrics
pub struct PerformanceStatistics {
    /// Advanced regression analysis results
    regression_reports: Vec<RegressionReport>,
}

/// Report for regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    /// Metric name
    pub metric_name: String,
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
    /// Regression result
    pub regression: RegressionResult,
    /// Statistical significance level
    pub significance_level: f64,
    /// Confidence interval for slope
    pub confidence_interval: (f64, f64),
    /// Forecasted values for next 5 points
    pub forecast: Vec<f64>,
    /// Root cause suggestions
    pub suggestions: Vec<String>,
    /// Risk assessment
    pub risk_level: RiskLevel,
}

/// Risk levels for performance regressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl PerformanceStatistics {

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

    /// Generate comprehensive regression report
    pub fn generate_regression_report(
        &mut self,
        metric_name: &str,
        values: &[f64],
        regression: &RegressionResult,
        significance: f64,
    ) -> RegressionReport {
        let confidence_interval = self.calculate_confidence_interval(values, regression, significance);
        let forecast = self.generate_forecast(regression, values.len() as f64, 5);
        let risk_level = self.assess_risk_level(regression, significance, &forecast);

        RegressionReport {
            metric_name: metric_name.to_string(),
            timestamp: Utc::now(),
            regression: regression.clone(),
            significance_level: significance,
            confidence_interval,
            forecast,
            suggestions: Vec::new(), // Will be filled by RegressionDetector
            risk_level,
        }
    }

    /// Calculate confidence interval for regression slope
    fn calculate_confidence_interval(&self, values: &[f64], regression: &RegressionResult, significance: f64) -> (f64, f64) {
        let n = values.len() as f64;
        let margin_of_error = self.calculate_margin_of_error(values, regression, significance);
        (regression.slope - margin_of_error, regression.slope + margin_of_error)
    }

    /// Calculate margin of error for confidence interval
    fn calculate_margin_of_error(&self, values: &[f64], regression: &RegressionResult, significance: f64) -> f64 {
        // Critical value based on significance level (simplified)
        let critical_value = match significance {
            s if s < 0.01 => 2.576, // 99% confidence
            s if s < 0.05 => 1.96,  // 95% confidence
            s if s < 0.1 => 1.645,  // 90% confidence
            _ => 1.282,             // 80% confidence
        };

        let n = values.len() as f64;
        let mut sum_squared_errors = 0.0;

        for (i, &value) in values.iter().enumerate() {
            let x = i as f64;
            let predicted = regression.slope * x + regression.intercept;
            sum_squared_errors += (value - predicted).powi(2);
        }

        let se_slope = (sum_squared_errors / (n - 2.0)).sqrt() /
                      ((n * (n - 1.0) * (n - 1.0)).sqrt());

        critical_value * se_slope
    }

    /// Generate forecast values
    fn generate_forecast(&self, regression: &RegressionResult, current_point: f64, steps: usize) -> Vec<f64> {
        (1..=steps)
            .map(|i| regression.slope * (current_point + i as f64) + regression.intercept)
            .collect()
    }

    /// Assess risk level based on regression analysis
    fn assess_risk_level(&self, regression: &RegressionResult, significance: f64, forecast: &[f64]) -> RiskLevel {
        let trend_strength = regression.slope.abs() * regression.r_squared;

        match (significance, trend_strength, forecast.last()) {
            (s, t, Some(f)) if s < 0.01 && t > 0.8 && regression.slope > 0.0 => RiskLevel::Critical,
            (s, t, _) if s < 0.05 && t > 0.6 && regression.slope > 0.0 => RiskLevel::High,
            (s, t, _) if s < 0.1 && t > 0.4 && regression.slope > 0.0 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    /// Export regression reports to JSON
    pub fn export_reports(&self) -> Result<String, Box<dyn std::error::Error>> {
        serde_json::to_string_pretty(&self.regression_reports)
            .map_err(|e| e.into())
    }

    /// Generate automated performance regression report
    pub fn generate_performance_report(&self, reports: &[RegressionReport]) -> String {
        let mut report = format!("Performance Regression Analysis Report\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        for report_item in reports {
            report.push_str(&format!("Metric: {}\n", report_item.metric_name));
            report.push_str(&format!("Risk Level: {:?}\n", report_item.risk_level));
            report.push_str(&format!("Regression Slope: {:.4}\n", report_item.regression.slope));
            report.push_str(&format!("R²: {:.4}\n", report_item.regression.r_squared));
            report.push_str(&format!("Significance: {:.4}\n", report_item.significance_level));
            report.push_str(&format!("Confidence Interval: ({:.4}, {:.4})\n",
                report_item.confidence_interval.0, report_item.confidence_interval.1));
            report.push_str(&format!("Forecast (next 5 points): {:?}\n", report_item.forecast));

            if !report_item.suggestions.is_empty() {
                report.push_str("Root Cause Suggestions:\n");
                for suggestion in &report_item.suggestions {
                    report.push_str(&format!("  - {}\n", suggestion));
                }
            }
            report.push_str("\n");
        }

        report
    }
}

impl Default for PerformanceStatistics {
    fn default() -> Self {
        Self {
            regression_reports: Vec::new(),
        }
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

    #[test]
    fn test_linear_regression_analysis() {
        let detector = RegressionDetector::new(5, 0.1);

        // Test data: y = 2x + 1
        let values = vec![3.0, 5.0, 7.0, 9.0, 11.0]; // x = 1,2,3,4,5 -> y = 3,5,7,9,11
        let regression = detector.perform_linear_regression(&values);

        assert!((regression.slope - 2.0).abs() < 0.001, "Slope should be approximately 2.0");
        assert!((regression.intercept - 1.0).abs() < 0.001, "Intercept should be approximately 1.0");
        assert!(regression.r_squared > 0.99, "R-squared should be very high for perfect fit");
    }

    #[test]
    fn test_statistical_significance() {
        let detector = RegressionDetector::new(5, 0.1);

        // Strong trend data
        let strong_trend = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let regression = detector.perform_linear_regression(&strong_trend);
        let significance = detector.calculate_statistical_significance(&strong_trend, &regression);

        assert!(significance < 0.05, "Strong trend should be statistically significant");

        // Weak trend data (random)
        let weak_trend = vec![1.0, 3.0, 2.0, 4.0, 1.5, 3.5, 2.5];
        let regression_weak = detector.perform_linear_regression(&weak_trend);
        let significance_weak = detector.calculate_statistical_significance(&weak_trend, &regression_weak);

        assert!(significance_weak > significance, "Weak trend should be less significant than strong trend");
    }

    #[test]
    fn test_forecasting() {
        let detector = RegressionDetector::new(5, 0.1);

        let values = vec![10.0, 12.0, 14.0, 16.0, 18.0]; // y = 2x + 8
        let regression = detector.perform_linear_regression(&values);
        let forecast = detector.generate_forecast(&regression, 5.0, 3);

        assert_eq!(forecast.len(), 3);
        assert!((forecast[0] - 20.0).abs() < 0.001, "Forecast at x=6 should be 20.0");
        assert!((forecast[1] - 22.0).abs() < 0.001, "Forecast at x=7 should be 22.0");
        assert!((forecast[2] - 24.0).abs() < 0.001, "Forecast at x=8 should be 24.0");
    }

    #[test]
    fn test_confidence_intervals() {
        let mut stats = PerformanceStatistics::default();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let regression = RegressionResult {
            slope: 1.0,
            intercept: 0.0,
            r_squared: 1.0,
            predicted_mean: 5.5,
        };

        let confidence_interval = stats.calculate_confidence_interval(&values, &regression, 0.05);
        assert!(confidence_interval.0 < 1.0, "Lower bound should be less than slope");
        assert!(confidence_interval.1 > 1.0, "Upper bound should be greater than slope");
    }

    #[test]
    fn test_root_cause_suggestions() {
        let detector = RegressionDetector::new(5, 0.1);
        let regression = RegressionResult {
            slope: 1.5,
            intercept: 0.0,
            r_squared: 0.85,
            predicted_mean: 5.0,
        };

        let suggestions_cpu = detector.generate_root_cause_suggestions("cpu_usage", &regression, 0.01);
        assert!(suggestions_cpu.len() > 0, "Should generate CPU usage suggestions");
        assert!(suggestions_cpu.iter().any(|s| s.contains("CPU")), "Should mention CPU-related issues");

        let suggestions_memory = detector.generate_root_cause_suggestions("memory_usage", &regression, 0.01);
        assert!(suggestions_memory.len() > 0, "Should generate memory usage suggestions");
        assert!(suggestions_memory.iter().any(|s| s.contains("memory")), "Should mention memory-related issues");
    }

    #[test]
    fn test_risk_assessment() {
        let mut stats = PerformanceStatistics::default();

        // High risk scenario
        let high_risk_regression = RegressionResult {
            slope: 2.0,
            intercept: 0.0,
            r_squared: 0.9,
            predicted_mean: 5.0,
        };
        let forecast_high = vec![20.0, 25.0, 30.0];
        let risk_high = stats.assess_risk_level(&high_risk_regression, 0.01, &forecast_high);
        assert!(matches!(risk_high, RiskLevel::Critical | RiskLevel::High), "High risk scenario should be critical or high");

        // Low risk scenario
        let low_risk_regression = RegressionResult {
            slope: 0.1,
            intercept: 0.0,
            r_squared: 0.3,
            predicted_mean: 5.0,
        };
        let forecast_low = vec![5.5, 5.6, 5.7];
        let risk_low = stats.assess_risk_level(&low_risk_regression, 0.5, &forecast_low);
        assert_eq!(risk_low, RiskLevel::Low, "Low risk scenario should be low");
    }

    #[test]
    fn test_regression_report_generation() {
        let mut stats = PerformanceStatistics::default();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let regression = RegressionResult {
            slope: 1.0,
            intercept: 0.0,
            r_squared: 1.0,
            predicted_mean: 3.0,
        };

        let report = stats.generate_regression_report("test_metric", &values, &regression, 0.01);

        assert_eq!(report.metric_name, "test_metric");
        assert!(report.risk_level != RiskLevel::Low); // Should be at least medium risk
        assert_eq!(report.significance_level, 0.01);
        assert!(!report.forecast.is_empty());
    }

    #[test]
    fn test_performance_report_formatting() {
        let mut stats = PerformanceStatistics::default();
        let report = RegressionReport {
            metric_name: "cpu_usage".to_string(),
            timestamp: Utc::now(),
            regression: RegressionResult {
                slope: 1.5,
                intercept: 0.0,
                r_squared: 0.85,
                predicted_mean: 50.0,
            },
            significance_level: 0.01,
            confidence_interval: (1.2, 1.8),
            forecast: vec![60.0, 70.0, 80.0],
            suggestions: vec!["Check CPU usage".to_string(), "Optimize algorithms".to_string()],
            risk_level: RiskLevel::High,
        };

        stats.regression_reports.push(report);
        let performance_report = stats.generate_performance_report(&stats.regression_reports);

        assert!(performance_report.contains("cpu_usage"));
        assert!(performance_report.contains("High"));
        assert!(performance_report.contains("Check CPU usage"));
        assert!(performance_report.contains("Optimize algorithms"));
    }
}
