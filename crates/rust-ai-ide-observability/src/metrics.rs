//! Metrics collection and recording for observability
//!
//! Provides comprehensive metrics collection for system resources,
//! application performance, and custom business metrics.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};
use tokio::sync::RwLock;

use crate::errors::Result;
use crate::ObservabilityConfig;

/// Metrics recorder for collecting and managing metrics
pub struct MetricsRecorder {
    config:         ObservabilityConfig,
    system:         Arc<RwLock<System>>,
    custom_metrics: Arc<RwLock<HashMap<String, MetricValue>>>,
}

impl MetricsRecorder {
    /// Create a new metrics recorder
    pub fn new(config: ObservabilityConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            config,
            system: Arc::new(RwLock::new(system)),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start metrics collection
    pub async fn start(&self) -> Result<()> {
        if !self.config.metrics.enabled {
            return Ok(());
        }

        // Initialize Prometheus exporter if enabled
        #[cfg(feature = "metrics")]
        {
            use metrics_exporter_prometheus::PrometheusBuilder;

            if let Ok(recorder) = PrometheusBuilder::new()
                .listen_address(([0, 0, 0, 0], self.config.metrics.prometheus_port))
                .build()
            {
                metrics::set_global_recorder(recorder).map_err(|e| {
                    crate::errors::ObservabilityError::metrics(format!("Failed to set global recorder: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Collect current system metrics
    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = self.system.write().await;
        system.refresh_all();

        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;
        let memory_usage = if total_memory > 0.0 {
            (used_memory / total_memory) * 100.0
        } else {
            0.0
        };

        let disk_usage = system
            .disks()
            .iter()
            .map(|disk| {
                let total = disk.total_space() as f64;
                let available = disk.available_space() as f64;
                if total > 0.0 {
                    ((total - available) / total) * 100.0
                } else {
                    0.0
                }
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let metrics = SystemMetrics {
            timestamp:            Utc::now(),
            cpu_usage_percent:    cpu_usage,
            memory_usage_percent: memory_usage,
            memory_used_mb:       used_memory / 1024.0 / 1024.0,
            memory_total_mb:      total_memory / 1024.0 / 1024.0,
            disk_usage_percent:   disk_usage,
            process_count:        system.processes().len() as u32,
            load_average:         system.load_average(),
            temperature_celsius:  None, // Would need additional crate for temperature
        };

        // Record metrics if enabled
        #[cfg(feature = "metrics")]
        if self.config.metrics.system_metrics_enabled {
            metrics::gauge!("system_cpu_usage_percent", metrics.cpu_usage_percent);
            metrics::gauge!("system_memory_usage_percent", metrics.memory_usage_percent);
            metrics::gauge!("system_memory_used_mb", metrics.memory_used_mb);
            metrics::gauge!("system_disk_usage_percent", metrics.disk_usage_percent);
            metrics::gauge!("system_process_count", metrics.process_count as f64);
        }

        Ok(metrics)
    }

    /// Record LSP performance metrics
    pub async fn record_lsp_metrics(
        &self,
        server_name: &str,
        request_count: u64,
        error_count: u64,
        avg_response_time_ms: f64,
    ) -> Result<()> {
        #[cfg(feature = "metrics")]
        if self.config.metrics.app_metrics_enabled {
            metrics::counter!("lsp_requests_total", request_count, "server" => server_name.to_string());
            metrics::counter!("lsp_errors_total", error_count, "server" => server_name.to_string());
            metrics::histogram!("lsp_response_time_ms", avg_response_time_ms, "server" => server_name.to_string());
        }

        let error_rate = if request_count > 0 {
            (error_count as f64 / request_count as f64) * 100.0
        } else {
            0.0
        };

        // Check for alerting thresholds
        if error_rate > self.config.alerting.lsp_error_rate_threshold {
            tracing::warn!(
                server = %server_name,
                error_rate = %error_rate,
                threshold = %self.config.alerting.lsp_error_rate_threshold,
                "High LSP error rate detected"
            );
        }

        Ok(())
    }

    /// Record AI processing metrics
    pub async fn record_ai_metrics(
        &self,
        operation: &str,
        processing_time_ms: f64,
        tokens_processed: u64,
    ) -> Result<()> {
        #[cfg(feature = "metrics")]
        if self.config.metrics.app_metrics_enabled {
            metrics::histogram!("ai_processing_time_ms", processing_time_ms, "operation" => operation.to_string());
            metrics::counter!("ai_tokens_processed_total", tokens_processed, "operation" => operation.to_string());
        }

        Ok(())
    }

    /// Record custom metric
    pub async fn record_custom_metric(&self, name: String, value: MetricValue) -> Result<()> {
        let mut metrics = self.custom_metrics.write().await;
        metrics.insert(name, value);
        Ok(())
    }

    /// Get current performance metrics summary
    pub async fn get_performance_summary(&self) -> Result<PerformanceMetrics> {
        let system_metrics = self.collect_system_metrics().await?;
        let custom_metrics = self.custom_metrics.read().await.clone();

        Ok(PerformanceMetrics {
            system_metrics,
            custom_metrics,
            timestamp: Utc::now(),
        })
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> Result<String> {
        #[cfg(feature = "metrics")]
        {
            use metrics_exporter_prometheus::Matcher;
            let recorder = metrics::try_recorder()
                .ok_or_else(|| crate::errors::ObservabilityError::metrics("No global recorder set"))?;

            if let Some(prometheus_recorder) =
                recorder.downcast_ref::<metrics_exporter_prometheus::PrometheusRecorder>()
            {
                return Ok(prometheus_recorder.render());
            }
        }

        Ok("# Metrics export not available\n".to_string())
    }
}

/// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp:            DateTime<Utc>,
    pub cpu_usage_percent:    f64,
    pub memory_usage_percent: f64,
    pub memory_used_mb:       f64,
    pub memory_total_mb:      f64,
    pub disk_usage_percent:   f64,
    pub process_count:        u32,
    pub load_average:         sysinfo::LoadAvg,
    pub temperature_celsius:  Option<f64>,
}

/// Performance metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub system_metrics: SystemMetrics,
    pub custom_metrics: HashMap<String, MetricValue>,
    pub timestamp:      DateTime<Utc>,
}

/// Value types for custom metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    String(String),
}

impl MetricValue {
    /// Get the gauge value if applicable
    pub fn as_gauge(&self) -> Option<f64> {
        match self {
            Self::Gauge(v) => Some(*v),
            Self::Counter(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Get the counter value if applicable
    pub fn as_counter(&self) -> Option<u64> {
        match self {
            Self::Counter(v) => Some(*v),
            _ => None,
        }
    }
}

/// Helper functions for common metrics
pub mod helpers {
    use super::*;

    /// Record HTTP request metrics
    pub async fn record_http_request(method: &str, path: &str, status: u16, duration_ms: f64) -> Result<()> {
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("http_requests_total", 1, "method" => method.to_string(), "status" => status.to_string());
            metrics::histogram!("http_request_duration_ms", duration_ms, "method" => method.to_string(), "path" => path.to_string());
        }

        Ok(())
    }

    /// Record database operation metrics
    pub async fn record_db_operation(operation: &str, table: &str, duration_ms: f64) -> Result<()> {
        #[cfg(feature = "metrics")]
        {
            metrics::histogram!("db_operation_duration_ms", duration_ms, "operation" => operation.to_string(), "table" => table.to_string());
        }

        Ok(())
    }

    /// Record cache operation metrics
    pub async fn record_cache_operation(operation: &str, hit: bool, duration_ms: f64) -> Result<()> {
        #[cfg(feature = "metrics")]
        {
            let hit_miss = if hit { "hit" } else { "miss" };
            metrics::counter!("cache_operations_total", 1, "operation" => operation.to_string(), "result" => hit_miss.to_string());
            metrics::histogram!("cache_operation_duration_ms", duration_ms, "operation" => operation.to_string());
        }

        Ok(())
    }

    /// Record security event metrics
    pub async fn record_security_event(event_type: &str, severity: &str) -> Result<()> {
        #[cfg(feature = "metrics")]
        {
            metrics::counter!("security_events_total", 1, "type" => event_type.to_string(), "severity" => severity.to_string());
        }

        Ok(())
    }
}
