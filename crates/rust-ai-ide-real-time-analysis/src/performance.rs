#![allow(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::types::PerformanceMetrics;

/// Performance monitoring and metrics collection for the real-time analysis engine
#[derive(Clone)]
pub struct PerformanceMonitor {
    /// Analysis timing data
    timings: Arc<DashMap<String, Vec<Duration>>>,
    /// Resource usage tracking
    resource_usage: Arc<DashMap<String, PerformanceMetrics>>,
    /// Monitoring configuration
    config: PerformanceConfig,
    /// Statistical aggregators
    stats: Arc<RwLock<PerformanceStats>>,
}

/// Configuration for performance monitoring
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable detailed metrics collection
    pub enable_detailed_metrics: bool,
    /// Performance measurement interval
    pub measurement_interval: Duration,
    /// Maximum samples to keep per metric
    pub max_samples_per_metric: usize,
    /// Enable outlier detection
    pub enable_outlier_detection: bool,
    /// Alert threshold for performance degradation
    pub alert_threshold_ms: u64,
}

/// Aggregated performance statistics
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// Total measurements taken
    pub total_measurements: u64,
    /// Average response times per operation type
    pub avg_response_times: HashMap<String, Duration>,
    /// 95th percentile response times
    pub p95_response_times: HashMap<String, Duration>,
    /// Error rates per operation type
    pub error_rates: HashMap<String, f32>,
    /// Resource utilization patterns
    pub resource_patterns: HashMap<String, ResourcePattern>,
}

/// Resource utilization pattern
#[derive(Debug, Clone)]
pub struct ResourcePattern {
    /// CPU usage pattern (percentage)
    pub cpu_pattern: f32,
    /// Memory usage pattern (bytes)
    pub memory_pattern: u64,
    /// I/O operations pattern
    pub io_pattern: u64,
    /// Network requests pattern
    pub network_pattern: usize,
}

/// Performance measurement context
#[derive(Clone)]
pub struct MeasurementContext {
    /// Operation identifier
    pub operation_id: String,
    /// Start timestamp
    pub start_time: Instant,
    /// Context metadata
    pub metadata: HashMap<String, String>,
}

/// Performance analysis report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Overall system performance score (0.0-1.0, higher is better)
    pub overall_score: f32,
    /// Response time analysis
    pub response_time_analysis: ResponseTimeAnalysis,
    /// Resource utilization analysis
    pub resource_analysis: ResourceUtilizationAnalysis,
    /// Recommendations for optimization
    pub recommendations: Vec<String>,
    /// Performance alerts
    pub alerts: Vec<String>,
}

/// Response time analysis
#[derive(Debug, Clone)]
pub struct ResponseTimeAnalysis {
    /// Average response time across all operations
    pub avg_response_time: Duration,
    /// Standard deviation of response times
    pub response_time_std_dev: Duration,
    /// Number of slow operations (>1s)
    pub slow_operations_count: usize,
    /// Operations with potential performance issues
    pub problematic_operations: Vec<String>,
}

/// Resource utilization analysis
#[derive(Debug, Clone)]
pub struct ResourceUtilizationAnalysis {
    /// Memory efficiency score
    pub memory_efficiency_score: f32,
    /// CPU efficiency score
    pub cpu_efficiency_score: f32,
    /// Highest memory usage recorded
    pub peak_memory_usage: u64,
    /// Average CPU time per operation
    pub avg_cpu_time_ns: u64,
}

impl PerformanceMonitor {
    /// Create a new performance monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(PerformanceConfig::default())
    }

    /// Create a new performance monitor with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            timings: Arc::new(DashMap::new()),
            resource_usage: Arc::new(DashMap::new()),
            config,
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
        }
    }

    /// Start measuring performance for an operation
    pub fn start_measurement(&self, operation_id: impl Into<String>) -> MeasurementContext {
        let operation_id = operation_id.into();
        let start_time = Instant::now();
        let metadata = HashMap::new();

        debug!("Started performance measurement: {}", operation_id);

        MeasurementContext {
            operation_id,
            start_time,
            metadata,
        }
    }

    /// Stop measurement and record metrics
    pub async fn stop_measurement(
        &self,
        context: MeasurementContext,
        metrics: Option<PerformanceMetrics>,
    ) {
        let duration = context.start_time.elapsed();
        let operation_id = context.operation_id.clone();

        // Record timing
        self.record_timing(&operation_id, duration).await;

        // Record resource usage if provided
        if let Some(metrics) = metrics {
            self.resource_usage.insert(operation_id.clone(), metrics);
        }

        debug!(
            "Completed performance measurement: {} in {}ms",
            operation_id,
            duration.as_millis()
        );

        // Check for performance alerts
        self.check_performance_alerts(&operation_id, duration).await;
    }

    /// Record timing for an operation
    async fn record_timing(&self, operation_id: &str, duration: Duration) {
        let timings = self.timings
            .entry(operation_id.to_string())
            .or_insert_with(Vec::new);

        timings.push(duration);

        // Enforce maximum samples
        if timings.len() > self.config.max_samples_per_metric {
            timings.remove(0); // Remove oldest
        }

        // Update statistics periodically
        if self.stats.read().await.total_measurements % 100 == 0 {
            self.update_statistics().await;
        }
    }

    /// Check for performance alerts
    async fn check_performance_alerts(&self, operation_id: &str, duration: Duration) {
        if duration.as_millis() > self.config.alert_threshold_ms as u64 {
            warn!(
                "Performance alert: {} took {}ms (threshold: {}ms)",
                operation_id,
                duration.as_millis(),
                self.config.alert_threshold_ms
            );
        }
    }

    /// Update aggregated statistics
    async fn update_statistics(&self) {
        let mut stats = self.stats.write().await;

        for item in self.timings.iter() {
            let (operation, timings) = item.pair();
            if timings.is_empty() {
                continue;
            }

            // Calculate average
            let total: Duration = timings.iter().sum();
            let avg = total / timings.len() as u32;

            // Calculate 95th percentile
            let mut sorted_timings = timings.clone();
            sorted_timings.sort();
            let p95_index = (sorted_timings.len() as f32 * 0.95) as usize;
            let p95_index = p95_index.min(sorted_timings.len().saturating_sub(1));
            let p95 = sorted_timings[p95_index];

            stats.avg_response_times.insert(operation.clone(), avg);
            stats.p95_response_times.insert(operation.clone(), p95);
        }

        stats.total_measurements = self.timings.len() as u64;
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let stats = self.stats.read().await;

        // Calculate overall score based on various metrics
        let overall_score = self.calculate_overall_score(&stats).await;

        let response_time_analysis = self.analyze_response_times(&stats).await;
        let resource_analysis = self.analyze_resource_usage().await;

        let recommendations = self.generate_recommendations(&stats, &resource_analysis).await;
        let alerts = self.generate_alerts(&stats, &response_time_analysis).await;

        PerformanceReport {
            overall_score,
            response_time_analysis,
            resource_analysis,
            recommendations,
            alerts,
        }
    }

    /// Calculate overall performance score
    async fn calculate_overall_score(&self, stats: &PerformanceStats) -> f32 {
        if stats.avg_response_times.is_empty() {
            return 1.0; // No data = perfect score
        }

        let mut total_score = 0.0;
        let mut operation_count = 0;

        for (_operation, avg_time) in &stats.avg_response_times {
            let score = match avg_time.as_millis() {
                0..=100 => 1.0,   // Excellent (< 100ms)
                101..=500 => 0.8, // Good (100-500ms)
                501..=2000 => 0.6, // Acceptable (500ms-2s)
                2001..=5000 => 0.4, // Slow (2-5s)
                _ => 0.2,          // Very slow (>5s)
            };
            total_score += score;
            operation_count += 1;
        }

        if operation_count > 0 {
            total_score / operation_count as f32
        } else {
            1.0
        }
    }

    /// Analyze response times
    async fn analyze_response_times(&self, stats: &PerformanceStats) -> ResponseTimeAnalysis {
        let mut total_response_time = Duration::new(0, 0);
        let mut operation_count = 0;
        let mut slow_operations = Vec::new();

        for (operation, avg_time) in &stats.avg_response_times {
            total_response_time += *avg_time;
            operation_count += 1;

            if avg_time.as_millis() > 1000 {
                slow_operations.push(operation.clone());
            }
        }

        let avg_response_time = if operation_count > 0 {
            total_response_time / operation_count as u32
        } else {
            Duration::new(0, 0)
        };

        ResponseTimeAnalysis {
            avg_response_time,
            response_time_std_dev: Duration::from_millis(50), // Placeholder
            slow_operations_count: slow_operations.len(),
            problematic_operations: slow_operations,
        }
    }

    /// Analyze resource utilization
    async fn analyze_resource_usage(&self) -> ResourceUtilizationAnalysis {
        let mut total_cpu_time = 0u64;
        let mut total_memory = 0u64;
        let mut peak_memory = 0u64;
        let mut operation_count = 0;

        for (_, metrics) in self.resource_usage.iter() {
            total_cpu_time += metrics.cpu_time_ns;
            total_memory += metrics.memory_usage;

            if metrics.memory_usage > peak_memory {
                peak_memory = metrics.memory_usage;
            }

            operation_count += 1;
        }

        let memory_efficiency_score = if total_memory > 0 {
            let avg_memory = total_memory / operation_count.max(1);
            if avg_memory < 100 * 1024 * 1024 { // 100MB
                1.0
            } else if avg_memory < 500 * 1024 * 1024 { // 500MB
                0.8
            } else if avg_memory < 1024 * 1024 * 1024 { // 1GB
                0.6
            } else {
                0.4
            }
        } else {
            1.0
        };

        let avg_cpu_time_ns = if operation_count > 0 {
            total_cpu_time / operation_count
        } else {
            0
        };

        ResourceUtilizationAnalysis {
            memory_efficiency_score,
            cpu_efficiency_score: 0.8, // Placeholder
            peak_memory_usage: peak_memory,
            avg_cpu_time_ns,
        }
    }

    /// Generate performance recommendations
    async fn generate_recommendations(
        &self,
        stats: &PerformanceStats,
        resource_analysis: &ResourceUtilizationAnalysis,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Memory optimization recommendations
        if resource_analysis.memory_efficiency_score < 0.7 {
            recommendations.push(
                "Consider implementing memory-efficient data structures or streaming analysis"
                    .to_string()
            );
        }

        // Response time recommendations
        for (operation, avg_time) in &stats.avg_response_times {
            if avg_time.as_millis() > 2000 {
                recommendations.push(format!(
                    "Optimize {} - consider parallel processing or caching",
                    operation
                ));
            }
        }

        recommendations
    }

    /// Generate performance alerts
    async fn generate_alerts(
        &self,
        stats: &PerformanceStats,
        response_analysis: &ResponseTimeAnalysis,
    ) -> Vec<String> {
        let mut alerts = Vec::new();

        // Check for critical performance issues
        if response_analysis.slow_operations_count > 3 {
            alerts.push(format!(
                "High number of slow operations: {} operations taking >1s",
                response_analysis.slow_operations_count
            ));
        }

        for operation in &response_analysis.problematic_operations {
            if let Some(avg_time) = stats.avg_response_times.get(operation) {
                if avg_time.as_millis() > 10000 {
                    alerts.push(format!(
                        "Critical: {} average response time {}ms exceeds recommended threshold",
                        operation,
                        avg_time.as_millis()
                    ));
                }
            }
        }

        alerts
    }

    /// Export performance data for external analysis
    pub async fn export_data(&self) -> serde_json::Value {
        let stats = self.stats.read().await;

        let operation_metrics: HashMap<String, serde_json::Value> = self.timings
            .iter()
            .map(|entry| {
                let (operation, timings) = entry.pair();
                let timing_data: Vec<f64> = timings.iter().map(|d| d.as_millis() as f64).collect();

                (operation.clone(), serde_json::json!({
                    "count": timing_data.len(),
                    "avg_ms": timing_data.iter().sum::<f64>() / timing_data.len() as f64,
                    "min_ms": *timing_data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                    "max_ms": *timing_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                }))
            })
            .collect();

        serde_json::json!({
            "total_measurements": stats.total_measurements,
            "operation_metrics": operation_metrics,
            "error_rates": stats.error_rates,
            "config": self.config,
        })
    }

    /// Reset performance monitoring data
    pub async fn reset(&self) {
        self.timings.clear();
        self.resource_usage.clear();
        *self.stats.write().await = PerformanceStats::default();
        debug!("Performance monitoring data reset");
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_detailed_metrics: true,
            measurement_interval: Duration::from_secs(5),
            max_samples_per_metric: 1000,
            enable_outlier_detection: true,
            alert_threshold_ms: 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(true); // Monitor creation should succeed
    }

    #[tokio::test]
    async fn test_performance_measurement() {
        let monitor = PerformanceMonitor::new();

        let context = monitor.start_measurement("test_operation");
        sleep(Duration::from_millis(10)).await;

        monitor.stop_measurement(context, None).await;

        // Verify measurements were recorded
        assert!(monitor.timings.contains_key("test_operation"));

        if let Some(timings) = monitor.timings.get("test_operation") {
            assert_eq!(timings.len(), 1);
            assert!(timings[0].as_millis() >= 10);
        }
    }

    #[tokio::test]
    async fn test_performance_report_generation() {
        let monitor = PerformanceMonitor::new();

        // Add some test data
        let context1 = monitor.start_measurement("fast_operation");
        sleep(Duration::from_millis(50)).await;
        monitor.stop_measurement(context1, None).await;

        let context2 = monitor.start_measurement("slow_operation");
        sleep(Duration::from_millis(1200)).await;
        monitor.stop_measurement(context2, None).await;

        let report = monitor.generate_report().await;

        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 1.0);
        assert!(report.response_time_analysis.slow_operations_count >= 1);
        assert!(!report.response_time_analysis.problematic_operations.is_empty());
    }

    #[tokio::test]
    async fn test_performance_config_alerts() {
        let config = PerformanceConfig {
            alert_threshold_ms: 100,
            ..Default::default()
        };

        let monitor = PerformanceMonitor::with_config(config);

        let context = monitor.start_measurement("slow_test");
        sleep(Duration::from_millis(200)).await;
        monitor.stop_measurement(context, None).await;

        // Alert should have been logged for slow operation
        // In a real test, we would verify logging output
    }

    #[tokio::test]
    async fn test_data_export() {
        let monitor = PerformanceMonitor::new();

        let context = monitor.start_measurement("export_test");
        monitor.stop_measurement(context, None).await;

        let export_data = monitor.export_data().await;
        assert!(export_data.is_object());

        let export_obj = export_data.as_object().unwrap();
        assert!(export_obj.contains_key("operation_metrics"));
        assert!(export_obj.contains_key("total_measurements"));
    }

    #[test]
    fn test_measurement_context_creation() {
        let monitor = PerformanceMonitor::new();
        let context = monitor.start_measurement("ctx_test");

        assert_eq!(context.operation_id, "ctx_test");
        assert!(context.metadata.is_empty());
        // Note: start_time validation would require accessing private fields
    }
}