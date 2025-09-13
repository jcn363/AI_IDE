//! Prometheus-compatible Metrics Collection Module
//!
//! This module provides Prometheus-compatible metrics collection and exposure
//! for the Rust AI IDE performance monitoring system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::{PerformanceMetrics, UnifiedPerformanceCollector};

/// Prometheus metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Prometheus metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<Bucket>),
    Summary(SummaryData),
}

/// Histogram bucket data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Summary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryData {
    pub sum: f64,
    pub count: u64,
    pub quantiles: HashMap<String, f64>,
}

/// Prometheus-compatible metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusMetric {
    pub name: String,
    pub help: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: Option<u64>,
}

impl PrometheusMetric {
    pub fn new(name: String, help: String, metric_type: MetricType) -> Self {
        Self {
            name,
            help,
            metric_type,
            value: match metric_type {
                MetricType::Counter => MetricValue::Counter(0),
                MetricType::Gauge => MetricValue::Gauge(0.0),
                MetricType::Histogram => MetricValue::Histogram(Vec::new()),
                MetricType::Summary => MetricValue::Summary(SummaryData {
                    sum: 0.0,
                    count: 0,
                    quantiles: HashMap::new(),
                }),
            },
            labels: HashMap::new(),
            timestamp: None,
        }
    }

    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }

    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Convert to Prometheus text format
    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();

        // Help comment
        output.push_str(&format!("# HELP {} {}\n", self.name, self.help));

        // Type comment
        let type_str = match self.metric_type {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::Summary => "summary",
        };
        output.push_str(&format!("# TYPE {} {}\n", self.name, type_str));

        // Metric value
        let labels_str = if self.labels.is_empty() {
            String::new()
        } else {
            let label_parts: Vec<String> = self
                .labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            format!("{{{}}}", label_parts.join(","))
        };

        match &self.value {
            MetricValue::Counter(count) => {
                output.push_str(&format!("{}{} {}\n", self.name, labels_str, count));
            }
            MetricValue::Gauge(value) => {
                output.push_str(&format!("{}{} {}\n", self.name, labels_str, value));
            }
            MetricValue::Histogram(buckets) => {
                for bucket in buckets {
                    let mut bucket_labels = self.labels.clone();
                    bucket_labels.insert("le".to_string(), bucket.upper_bound.to_string());
                    let bucket_label_parts: Vec<String> = bucket_labels
                        .iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    let bucket_labels_str = format!("{{{}}}", bucket_label_parts.join(","));
                    output.push_str(&format!(
                        "{}_bucket{} {}\n",
                        self.name, bucket_labels_str, bucket.count
                    ));
                }
            }
            MetricValue::Summary(summary) => {
                output.push_str(&format!(
                    "{}_sum{} {}\n",
                    self.name, labels_str, summary.sum
                ));
                output.push_str(&format!(
                    "{}_count{} {}\n",
                    self.name, labels_str, summary.count
                ));
                for (quantile, value) in &summary.quantiles {
                    let mut quantile_labels = self.labels.clone();
                    quantile_labels.insert("quantile".to_string(), quantile.clone());
                    let quantile_label_parts: Vec<String> = quantile_labels
                        .iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    let quantile_labels_str = format!("{{{}}}", quantile_label_parts.join(","));
                    output.push_str(&format!("{}{} {}\n", self.name, quantile_labels_str, value));
                }
            }
        }

        output
    }
}

/// Registry for Prometheus metrics
#[derive(Debug, Clone)]
pub struct MetricsRegistry {
    metrics: Arc<RwLock<HashMap<String, PrometheusMetric>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_metric(&self, metric: PrometheusMetric) -> Result<(), MetricsError> {
        let mut metrics = self.metrics.write().await;
        if metrics.contains_key(&metric.name) {
            return Err(MetricsError::MetricAlreadyExists(metric.name));
        }
        metrics.insert(metric.name.clone(), metric);
        Ok(())
    }

    pub async fn get_metric(&self, name: &str) -> Option<PrometheusMetric> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned()
    }

    pub async fn update_counter(&self, name: &str, increment: u64) -> Result<(), MetricsError> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Counter(ref mut count) = metric.value {
                *count += increment;
                metric.timestamp = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                );
                Ok(())
            } else {
                Err(MetricsError::WrongMetricType)
            }
        } else {
            Err(MetricsError::MetricNotFound(name.to_string()))
        }
    }

    pub async fn set_gauge(&self, name: &str, value: f64) -> Result<(), MetricsError> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Gauge(ref mut gauge_value) = metric.value {
                *gauge_value = value;
                metric.timestamp = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                );
                Ok(())
            } else {
                Err(MetricsError::WrongMetricType)
            }
        } else {
            Err(MetricsError::MetricNotFound(name.to_string()))
        }
    }

    pub async fn export_all(&self) -> String {
        let metrics = self.metrics.read().await;
        let mut output = String::new();

        for metric in metrics.values() {
            output.push_str(&metric.to_prometheus_format());
            output.push('\n');
        }

        output
    }
}

/// Metrics collection errors
#[derive(Debug, Clone)]
pub enum MetricsError {
    MetricAlreadyExists(String),
    MetricNotFound(String),
    WrongMetricType,
    CollectionFailed(String),
}

impl std::fmt::Display for MetricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricsError::MetricAlreadyExists(name) => {
                write!(f, "Metric '{}' already exists", name)
            }
            MetricsError::MetricNotFound(name) => write!(f, "Metric '{}' not found", name),
            MetricsError::WrongMetricType => write!(f, "Wrong metric type"),
            MetricsError::CollectionFailed(msg) => write!(f, "Collection failed: {}", msg),
        }
    }
}

impl std::error::Error for MetricsError {}

/// Performance metrics exporter for Prometheus
pub struct PrometheusExporter {
    registry: MetricsRegistry,
    collector: Arc<UnifiedPerformanceCollector>,
}

impl PrometheusExporter {
    pub fn new(collector: Arc<UnifiedPerformanceCollector>) -> Self {
        let registry = MetricsRegistry::new();
        Self {
            registry,
            collector,
        }
    }

    pub async fn initialize_default_metrics(&self) -> Result<(), MetricsError> {
        // CPU usage metrics
        self.registry
            .register_metric(
                PrometheusMetric::new(
                    "rust_ai_ide_cpu_usage_percent".to_string(),
                    "Current CPU usage percentage".to_string(),
                    MetricType::Gauge,
                )
                .with_label("service".to_string(), "rust-ai-ide".to_string()),
            )
            .await?;

        // Memory usage metrics
        self.registry
            .register_metric(
                PrometheusMetric::new(
                    "rust_ai_ide_memory_usage_bytes".to_string(),
                    "Current memory usage in bytes".to_string(),
                    MetricType::Gauge,
                )
                .with_label("service".to_string(), "rust-ai-ide".to_string()),
            )
            .await?;

        // Response time metrics
        self.registry
            .register_metric(
                PrometheusMetric::new(
                    "rust_ai_ide_response_time_seconds".to_string(),
                    "Response time in seconds".to_string(),
                    MetricType::Histogram,
                )
                .with_label("service".to_string(), "rust-ai-ide".to_string()),
            )
            .await?;

        // Request rate metrics
        self.registry
            .register_metric(
                PrometheusMetric::new(
                    "rust_ai_ide_requests_total".to_string(),
                    "Total number of requests".to_string(),
                    MetricType::Counter,
                )
                .with_label("service".to_string(), "rust-ai-ide".to_string()),
            )
            .await?;

        Ok(())
    }

    pub async fn collect_and_export(&self) -> Result<String, MetricsError> {
        // Get latest metrics from collector
        if let Some(metrics) = self.collector.get_latest_metrics() {
            // Update Prometheus metrics based on PerformanceMetrics
            if let Some(cpu_usage) = metrics.cpu_usage_percent {
                self.registry
                    .set_gauge("rust_ai_ide_cpu_usage_percent", cpu_usage)
                    .await?;
            }

            if let Some(memory_mb) = metrics.memory_usage_mb {
                let memory_bytes = (memory_mb * 1024.0 * 1024.0) as f64;
                self.registry
                    .set_gauge("rust_ai_ide_memory_usage_bytes", memory_bytes)
                    .await?;
            }

            if let Some(response_ms) = metrics.response_time_ms {
                // For histogram, we'd need to implement bucket logic
                // This is simplified for now
            }

            // Increment request counter (simplified)
            self.registry
                .update_counter("rust_ai_ide_requests_total", 1)
                .await?;
        }

        // Export all metrics in Prometheus format
        Ok(self.registry.export_all().await)
    }

    pub fn get_registry(&self) -> &MetricsRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prometheus_metric_creation() {
        let metric = PrometheusMetric::new(
            "test_metric".to_string(),
            "A test metric".to_string(),
            MetricType::Counter,
        )
        .with_label("service".to_string(), "test".to_string());

        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.help, "A test metric");
        assert!(matches!(metric.metric_type, MetricType::Counter));
        assert!(metric.labels.contains_key("service"));
    }

    #[tokio::test]
    async fn test_metrics_registry() {
        let registry = MetricsRegistry::new();

        let metric = PrometheusMetric::new(
            "test_counter".to_string(),
            "Test counter".to_string(),
            MetricType::Counter,
        );

        registry.register_metric(metric).await.unwrap();

        let retrieved = registry.get_metric("test_counter").await.unwrap();
        assert_eq!(retrieved.name, "test_counter");

        registry.update_counter("test_counter", 5).await.unwrap();

        let updated = registry.get_metric("test_counter").await.unwrap();
        match updated.value {
            MetricValue::Counter(count) => assert_eq!(count, 5),
            _ => panic!("Wrong metric type"),
        }
    }

    #[test]
    fn test_prometheus_format_output() {
        let metric = PrometheusMetric::new(
            "test_gauge".to_string(),
            "A test gauge".to_string(),
            MetricType::Gauge,
        )
        .with_label("service".to_string(), "test".to_string());

        let output = metric.to_prometheus_format();
        assert!(output.contains("# HELP test_gauge A test gauge"));
        assert!(output.contains("# TYPE test_gauge gauge"));
        assert!(output.contains("test_gauge{service=\"test\"}"));
    }
}
