//! Metrics collection and monitoring for distributed AI processing
//!
//! This module provides comprehensive monitoring of distributed AI operations:
//! - Request latency and throughput metrics
//! - Worker utilization and health metrics
//! - Model performance statistics
//! - Distributed processing efficiency metrics

use std::collections::HashMap;
use std::sync::Arc;

use lazy_static::lazy_static;
use prometheus::{register_counter, register_gauge, register_histogram, Encoder, TextEncoder};
use tokio::sync::RwLock;
use tracing::info;

/// Distributed AI metrics collection
pub struct DistributedMetrics {
    total_requests:           prometheus::Counter,
    successful_requests:      prometheus::Counter,
    failed_requests:          prometheus::Counter,
    request_latency:          prometheus::Histogram,
    worker_load_distribution: prometheus::GaugeVec,
    gpu_utilization:          prometheus::GaugeVec,
    model_inference_time:     prometheus::HistogramVec,
    cache_hit_ratio:          prometheus::Gauge,
    memory_usage:             prometheus::GaugeVec,
    current_active_workers:   prometheus::Gauge,
}

impl DistributedMetrics {
    pub fn new() -> Self {
        let total_requests = register_counter!(
            "distributed_ai_requests_total",
            "Total number of distributed AI requests"
        )
        .unwrap();

        let successful_requests = register_counter!(
            "distributed_ai_requests_successful",
            "Number of successful distributed AI requests"
        )
        .unwrap();

        let failed_requests = register_counter!(
            "distributed_ai_requests_failed",
            "Number of failed distributed AI requests"
        )
        .unwrap();

        let request_latency = register_histogram!(
            "distributed_ai_request_latency_ms",
            "Request latency in milliseconds",
            vec![100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0]
        )
        .unwrap();

        let worker_load_distribution = register_gauge_vec!(
            "distributed_ai_worker_load",
            "Worker load distribution (0.0-1.0)",
            &["worker_id"]
        )
        .unwrap();

        let gpu_utilization = register_gauge_vec!(
            "distributed_ai_gpu_utilization",
            "GPU utilization per worker (0.0-1.0)",
            &["worker_id", "gpu_type"]
        )
        .unwrap();

        let model_inference_time = register_histogram_vec!(
            "distributed_ai_model_inference_time_ms",
            "Model inference time in milliseconds",
            &["model_type", "operation"],
            vec![50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0]
        )
        .unwrap();

        let cache_hit_ratio = register_gauge!(
            "distributed_ai_cache_hit_ratio",
            "Cache hit ratio (0.0-1.0)"
        )
        .unwrap();

        let memory_usage = register_gauge_vec!("distributed_ai_memory_usage_gb", "Memory usage in GB", &[
            "worker_id",
            "memory_type"
        ])
        .unwrap();

        let current_active_workers = register_gauge!(
            "distributed_ai_active_workers",
            "Current number of active workers"
        )
        .unwrap();

        Self {
            total_requests,
            successful_requests,
            failed_requests,
            request_latency,
            worker_load_distribution,
            gpu_utilization,
            model_inference_time,
            cache_hit_ratio,
            memory_usage,
            current_active_workers,
        }
    }

    pub async fn record_request(&self, latency_ms: u64, success_count: usize) {
        self.total_requests.inc();
        self.request_latency.observe(latency_ms as f64);

        if success_count > 0 {
            self.successful_requests.inc();
        } else {
            self.failed_requests.inc();
        }
    }

    pub fn record_worker_load(&self, worker_id: &str, load: f64) {
        self.worker_load_distribution
            .with_label_values(&[worker_id])
            .set(load);
    }

    pub fn record_gpu_utilization(&self, worker_id: &str, gpu_type: &str, utilization: f64) {
        self.gpu_utilization
            .with_label_values(&[worker_id, gpu_type])
            .set(utilization);
    }

    pub fn record_model_inference_time(&self, model_type: &str, operation: &str, latency_ms: u64) {
        self.model_inference_time
            .with_label_values(&[model_type, operation])
            .observe(latency_ms as f64);
    }

    pub fn record_cache_hit_ratio(&self, ratio: f64) {
        self.cache_hit_ratio.set(ratio);
    }

    pub fn record_memory_usage(&self, worker_id: &str, memory_type: &str, usage_gb: f64) {
        self.memory_usage
            .with_label_values(&[worker_id, memory_type])
            .set(usage_gb);
    }

    pub fn record_active_workers(&self, count: usize) {
        self.current_active_workers.set(count as f64);
    }

    pub fn export_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();

        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

lazy_static! {
    static ref METRICS: RwLock<Option<Arc<DistributedMetrics>>> = RwLock::new(None);
}

/// Initialize global metrics instance
pub async fn init_metrics() -> Arc<DistributedMetrics> {
    let metrics = Arc::new(DistributedMetrics::new());
    *METRICS.write().await = Some(metrics.clone());
    metrics
}

/// Get global metrics instance
pub async fn get_metrics() -> Option<Arc<DistributedMetrics>> {
    METRICS.read().await.as_ref().cloned()
}

/// Export current metrics as Prometheus format
pub async fn export_prometheus_metrics() -> Result<String, Box<dyn std::error::Error>> {
    if let Some(metrics) = get_metrics().await {
        metrics.export_metrics()
    } else {
        Ok("# Metrics not initialized\n".to_string())
    }
}

/// Metrics collector for periodic updates
pub struct MetricsCollector {
    metrics:       Arc<DistributedMetrics>,
    worker_states: Arc<RwLock<HashMap<String, WorkerState>>>,
}

#[derive(Debug, Clone)]
pub struct WorkerState {
    pub load_factor:     f64,
    pub gpu_utilization: Option<f64>,
    pub gpu_type:        Option<String>,
    pub memory_usage_gb: f64,
    pub active_requests: u32,
    pub last_update:     std::time::Instant,
}

impl MetricsCollector {
    pub fn new(metrics: Arc<DistributedMetrics>) -> Self {
        Self {
            metrics,
            worker_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update_worker_state(&self, worker_id: &str, state: WorkerState) {
        let mut states = self.worker_states.write().await;
        states.insert(worker_id.to_string(), state.clone());

        // Update Prometheus metrics
        self.metrics
            .record_worker_load(worker_id, state.load_factor);

        if let (Some(gpu_util), Some(gpu_type)) = (state.gpu_utilization, state.gpu_type.as_ref()) {
            self.metrics
                .record_gpu_utilization(worker_id, gpu_type, gpu_util);
        }

        self.metrics
            .record_memory_usage(worker_id, "total", state.memory_usage_gb);

        info!(
            "Updated metrics for worker {}: load={:.2}, gpu={:.2}, mem={:.1}GB",
            worker_id,
            state.load_factor,
            state.gpu_utilization.unwrap_or(0.0),
            state.memory_usage_gb
        );
    }

    pub async fn collect_aggregated_metrics(&self) {
        let states = self.worker_states.read().await;

        let active_workers = states
            .values()
            .filter(|s| s.last_update.elapsed() < std::time::Duration::from_secs(60))
            .count();

        let avg_load = if active_workers > 0 {
            states.values().map(|s| s.load_factor).sum::<f64>() / active_workers as f64
        } else {
            0.0
        };

        let total_memory = states.values().map(|s| s.memory_usage_gb).sum::<f64>();

        // Update global metrics
        self.metrics.record_active_workers(active_workers);

        info!(
            "Aggregated metrics: {} active workers, avg load {:.2}, total mem {:.1}GB",
            active_workers, avg_load, total_memory
        );
    }

    pub async fn get_worker_health_summary(&self) -> HashMap<String, WorkerHealth> {
        let states = self.worker_states.read().await;
        let mut summary = HashMap::new();

        for (worker_id, state) in states.iter() {
            let health = if state.last_update.elapsed() > std::time::Duration::from_secs(120) {
                WorkerHealth::Offline
            } else if state.load_factor > 0.95 {
                WorkerHealth::Overloaded
            } else if state.load_factor < 0.8 {
                WorkerHealth::Healthy
            } else {
                WorkerHealth::Busy
            };

            summary.insert(worker_id.clone(), health);
        }

        summary
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerHealth {
    Healthy,
    Busy,
    Overloaded,
    Offline,
}

impl std::fmt::Display for WorkerHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerHealth::Healthy => write!(f, "Healthy"),
            WorkerHealth::Busy => write!(f, "Busy"),
            WorkerHealth::Overloaded => write!(f, "Overloaded"),
            WorkerHealth::Offline => write!(f, "Offline"),
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MetricsConfig {
    pub enabled:                  bool,
    pub prometheus_port:          u16,
    pub metrics_interval_seconds: u64,
    pub export_interval_seconds:  u64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled:                  true,
            prometheus_port:          9091,
            metrics_interval_seconds: 30,
            export_interval_seconds:  60,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_metrics_initialization() {
        let metrics = DistributedMetrics::new();

        // Test basic counters
        metrics.total_requests.inc();
        metrics.successful_requests.inc();

        let exported = metrics.export_metrics().unwrap();
        assert!(exported.contains("distributed_ai_requests_total"));
        assert!(exported.contains("distributed_ai_requests_successful"));
    }

    #[tokio::test]
    async fn test_worker_state_updates() {
        let metrics = Arc::new(DistributedMetrics::new());
        let collector = MetricsCollector::new(metrics);

        let worker_state = WorkerState {
            load_factor:     0.6,
            gpu_utilization: Some(0.8),
            gpu_type:        Some("A100".to_string()),
            memory_usage_gb: 24.5,
            active_requests: 5,
            last_update:     std::time::Instant::now(),
        };

        collector
            .update_worker_state("worker-1", worker_state)
            .await;

        let states = collector.worker_states.read().await;
        assert_eq!(states.len(), 1);
        assert!(states.contains_key("worker-1"));
    }

    #[tokio::test]
    async fn test_worker_health_assessment() {
        let metrics = Arc::new(DistributedMetrics::new());
        let collector = MetricsCollector::new(metrics);

        // Add healthy worker
        let healthy_state = WorkerState {
            load_factor:     0.3,
            gpu_utilization: Some(0.5),
            gpu_type:        Some("A100".to_string()),
            memory_usage_gb: 16.0,
            active_requests: 2,
            last_update:     std::time::Instant::now(),
        };

        collector
            .update_worker_state("healthy-worker", healthy_state)
            .await;

        // Add overloaded worker
        let overloaded_state = WorkerState {
            load_factor:     0.97,
            gpu_utilization: Some(0.95),
            gpu_type:        Some("A100".to_string()),
            memory_usage_gb: 30.0,
            active_requests: 8,
            last_update:     std::time::Instant::now(),
        };

        collector
            .update_worker_state("overloaded-worker", overloaded_state)
            .await;

        let health_summary = collector.get_worker_health_summary().await;
        assert_eq!(
            health_summary[&"healthy-worker".to_string()],
            WorkerHealth::Healthy
        );
        assert_eq!(
            health_summary[&"overloaded-worker".to_string()],
            WorkerHealth::Overloaded
        );
    }

    #[tokio::test]
    async fn test_metrics_export_format() {
        let metrics = Arc::new(DistributedMetrics::new());

        // Record some metrics
        metrics.total_requests.inc();
        metrics.successful_requests.inc();
        metrics.request_latency.observe(150.0);
        metrics.record_worker_load("test-worker", 0.5);

        let exported = metrics.export_metrics().unwrap();

        // Validate Prometheus format
        assert!(exported.starts_with("# HELP"));
        assert!(exported.contains("# TYPE"));
        assert!(exported.contains("distributed_ai_requests_total"));
        assert!(exported.contains("distributed_ai_worker_load{worker_id=\"test-worker\"}"));
    }
}
