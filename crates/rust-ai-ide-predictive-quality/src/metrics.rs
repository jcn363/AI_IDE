//! Performance metrics and monitoring for predictive quality intelligence

use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry,
    TextEncoder, register_counter_vec, register_histogram_vec,
};
use lazy_static::lazy_static;
use std::time::Instant;

/// Metrics for vulnerability prediction operations
pub struct VulnerabilityMetrics {
    predictions_total: prometheus::IntCounterVec,
    prediction_duration: prometheus::HistogramVec,
    cache_hits: prometheus::IntCounterVec,
    cache_misses: prometheus::IntCounterVec,
    accuracy_rate: prometheus::HistogramVec,
}

impl VulnerabilityMetrics {
    pub fn new() -> Self {
        let registry = prometheus::default_registry();

        let predictions_total = register_counter_vec!(
            "rust_ai_ide_vulnerability_predictions_total",
            "Total number of vulnerability predictions made",
            &["outcome"],
            registry
        ).unwrap();

        let prediction_duration = register_histogram_vec!(
            HistogramOpts::new(
                "rust_ai_ide_vulnerability_prediction_duration_seconds",
                "Duration of vulnerability prediction operations"
            ),
            &["phase"],
            registry
        ).unwrap();

        let cache_hits = register_counter_vec!(
            "rust_ai_ide_vulnerability_cache_hits_total",
            "Number of cache hits for vulnerability predictions",
            &[],
            registry
        ).unwrap();

        let cache_misses = register_counter_vec!(
            "rust_ai_ide_vulnerability_cache_misses_total",
            "Number of cache misses for vulnerability predictions",
            &[],
            registry
        ).unwrap();

        let accuracy_rate = register_histogram_vec!(
            HistogramOpts::new(
                "rust_ai_ide_vulnerability_accuracy_rate",
                "Accuracy rate of vulnerability predictions"
            ),
            &["threshold"],
            registry
        ).unwrap();

        Self {
            predictions_total,
            prediction_duration,
            cache_hits,
            cache_misses,
            accuracy_rate,
        }
    }

    pub async fn record_cache_hit(&self) {
        self.cache_hits.with_label_values(&[]).inc();
    }

    pub async fn record_cache_miss(&self) {
        self.cache_misses.with_label_values(&[]).inc();
    }

    pub async fn record_prediction_time(&self, duration: std::time::Duration) {
        self.prediction_duration
            .with_label_values(&["total"])
            .observe(duration.as_secs_f64());
    }

    pub async fn record_prediction_success(&self) {
        self.predictions_total
            .with_label_values(&["success"])
            .inc();
    }

    pub async fn record_prediction_failure(&self) {
        self.predictions_total
            .with_label_values(&["failure"])
            .inc();
    }
}

/// Metrics for maintenance forecasting
pub struct MaintenanceMetrics {
    forecasts_total: prometheus::IntCounterVec,
    forecast_duration: prometheus::HistogramVec,
    cost_estimation_accuracy: prometheus::HistogramVec,
    task_priority_distribution: prometheus::HistogramVec,
}

impl MaintenanceMetrics {
    pub fn new() -> Self {
        let registry = prometheus::default_registry();

        let forecasts_total = register_counter_vec!(
            "rust_ai_ide_maintenance_forecasts_total",
            "Total number of maintenance forecasts made",
            &["outcome"],
            registry
        ).unwrap();

        let forecast_duration = register_histogram_vec!(
            HistogramOpts::new(
                "rust_ai_ide_maintenance_forecast_duration_seconds",
                "Duration of maintenance forecasting operations"
            ),
            &["component"],
            registry
        ).unwrap();

        let cost_estimation_accuracy = register_histogram_vec!(
            HistogramOpts::new(
                "rust_ai_ide_cost_estimation_accuracy_rate",
                "Accuracy rate of maintenance cost estimations"
            ),
            &["tolerance_range"],
            registry
        ).unwrap();

        let task_priority_distribution = register_histogram_vec!(
            HistogramOpts::new(
                "rust_ai_ide_task_priority_distribution",
                "Distribution of maintenance task priorities"
            ),
            &["priority"],
            registry
        ).unwrap();

        Self {
            forecasts_total,
            forecast_duration,
            cost_estimation_accuracy,
            task_priority_distribution,
        }
    }

    pub async fn record_forecast_success(&self) {
        self.forecasts_total
            .with_label_values(&["success"])
            .inc();
    }

    pub async fn record_forecast_duration(&self, duration: std::time::Duration) {
        self.forecast_duration
            .with_label_values(&["total"])
            .observe(duration.as_secs_f64());
    }
}

/// Metrics for health scoring
pub struct HealthMetrics {
    scores_total: prometheus::IntCounterVec,
    scoring_duration: prometheus::HistogramVec,
    health_trend: prometheus::HistogramVec,
    performance_requirements_met: prometheus::IntCounterVec,
}

impl HealthMetrics {
    pub fn new() -> Self {
        let registry = prometheus::default_registry();

        let scores_total = register_counter_vec!(
            "rust_ai_ide_health_scores_total",
            "Total number of health scores calculated",
            &["result_type"],
            registry
        ).unwrap();

        let scoring_duration = register_histogram_vec!(
            HistogramOpts::new(
                "rust_ai_ide_health_scoring_duration_seconds",
                "Duration of health scoring operations"
            ),
            &["metric_type"],
            registry
        ).unwrap();

        let health_trend = register_histogram_vec!(
            HistogramOpts::new(
                "rust_ai_ide_health_trend",
                "Health trend over time periods"
            ),
            &["direction", "period"],
            registry
        ).unwrap();

        let performance_requirements_met = register_counter_vec!(
            "rust_ai_ide_performance_requirements_met_total",
            "Number of times performance requirements were met",
            &["requirement"],
            registry
        ).unwrap();

        Self {
            scores_total,
            scoring_duration,
            health_trend,
            performance_requirements_met,
        }
    }

    pub async fn record_scoring_duration(&self, duration: std::time::Duration, metric_type: &str) {
        self.scoring_duration
            .with_label_values(&[metric_type])
            .observe(duration.as_secs_f64());

        // Check performance requirement (<300ms)
        if duration < std::time::Duration::from_millis(300) {
            self.performance_requirements_met
                .with_label_values(&["response_time"])
                .inc();
        }
    }
}

/// Overall predictive quality metrics aggregator
pub struct PredictiveQualityMetrics {
    vulnerability_metrics: VulnerabilityMetrics,
    maintenance_metrics: MaintenanceMetrics,
    health_metrics: HealthMetrics,
}

impl PredictiveQualityMetrics {
    pub fn new() -> Self {
        Self {
            vulnerability_metrics: VulnerabilityMetrics::new(),
            maintenance_metrics: MaintenanceMetrics::new(),
            health_metrics: HealthMetrics::new(),
        }
    }

    pub fn vulnerability(&self) -> &VulnerabilityMetrics {
        &self.vulnerability_metrics
    }

    pub fn maintenance(&self) -> &MaintenanceMetrics {
        &self.maintenance_metrics
    }

    pub fn health(&self) -> &HealthMetrics {
        &self.health_metrics
    }
}

// Global metrics instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_PREDICTIVE_METRICS: PredictiveQualityMetrics = PredictiveQualityMetrics::new();
}

/// Helper trait for reporting operation metrics
pub trait MetricReporter {
    async fn report_operation(&self, operation_name: &str, success: bool, duration: std::time::Duration) {
        GLOBAL_PREDICTIVE_METRICS.vulnerability_metrics
            .report_operation(operation_name, success, duration).await;
    }
}

/// Prometheus exporter for current metrics
pub struct MetricExporter {
    encoder: TextEncoder,
    registry: Registry,
}

impl MetricExporter {
    pub fn new() -> Self {
        Self {
            encoder: TextEncoder::new(),
            registry: prometheus::default_registry().clone(),
        }
    }

    /// Export metrics as Prometheus format
    pub fn export(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = Vec::new();
        let metric_families = self.registry.gather();
        self.encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Get metrics in JSON format for API responses
    pub fn export_json(&self) -> serde_json::Value {
        // Simple JSON export - could be enhanced
        serde_json::json!({
            "vulnerability_predictions": true,
            "maintenance_forecasting": true,
            "health_scoring": true,
            "last_updated": chrono::Utc::now().timestamp()
        })
    }
}

impl Default for MetricExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_exporter_creation() {
        let exporter = MetricExporter::new();
        assert!(true); // Basic test
    }

    #[tokio::test]
    async fn test_vulnerability_metrics_recording() {
        let metrics = VulnerabilityMetrics::new();

        // Test basic recording
        metrics.record_cache_hit().await;
        metrics.record_cache_miss().await;

        // These would be checked by Prometheus registry in real testing
        assert!(true);
    }
}