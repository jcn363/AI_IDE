//! Central observability manager
//!
//! Orchestrates all observability components including metrics,
//! tracing, health checks, and alerting.

use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::config::ObservabilityConfig;
use crate::errors::Result;
use crate::health::{HealthChecker, HealthStatus};
use crate::metrics::{MetricsRecorder, PerformanceMetrics};
use crate::tracing::Tracer;

/// Main observability manager
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    tracer: Option<Tracer>,
    metrics_recorder: Option<MetricsRecorder>,
    health_checker: Option<HealthChecker>,
    is_running: Arc<RwLock<bool>>,
}

impl ObservabilityManager {
    /// Create a new observability manager
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            tracer: Some(Tracer::new(config.clone())),
            metrics_recorder: Some(MetricsRecorder::new(config.clone())),
            health_checker: Some(HealthChecker::new(config.clone())),
            config,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start all observability systems
    pub async fn start(&self) -> Result<()> {
        let mut running = self.is_running.write().await;
        if *running {
            return Ok(());
        }

        tracing::info!("Starting observability manager");

        // Initialize tracing
        if let Some(tracer) = &self.tracer {
            tracer.init()?;
            tracing::info!("Tracing system initialized");
        }

        // Start metrics collection
        if let Some(metrics) = &self.metrics_recorder {
            metrics.start().await?;
            tracing::info!("Metrics collection started");
        }

        // Start background health checks
        if self.config.health.enabled {
            self.start_health_check_loop();
            tracing::info!("Health check system started");
        }

        // Start metrics collection loop
        if self.config.metrics.enabled && self.config.metrics.collection_interval_secs > 0 {
            self.start_metrics_collection_loop();
            tracing::info!("Metrics collection loop started");
        }

        *running = true;
        tracing::info!("Observability manager started successfully");

        Ok(())
    }

    /// Stop all observability systems
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.is_running.write().await;
        if !*running {
            return Ok(());
        }

        tracing::info!("Stopping observability manager");

        // Flush tracing data
        if let Some(tracer) = &self.tracer {
            tracer.flush().await?;
        }

        *running = false;
        tracing::info!("Observability manager stopped");

        Ok(())
    }

    /// Get current health status
    pub async fn health_check(&self) -> Result<HealthStatus> {
        if let Some(checker) = &self.health_checker {
            checker.get_health_status().await
        } else {
            Ok(crate::health::HealthStatus {
                overall_status: crate::health::HealthCheckStatus::Healthy,
                timestamp: chrono::Utc::now(),
                checks: std::collections::HashMap::new(),
                message: "Health checks disabled".to_string(),
            })
        }
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> Result<PerformanceMetrics> {
        if let Some(metrics) = &self.metrics_recorder {
            metrics.get_performance_summary().await
        } else {
            Err(crate::errors::ObservabilityError::metrics(
                "Metrics collection not enabled",
            ))
        }
    }

    /// Record a custom metric
    pub async fn record_metric(
        &self,
        name: String,
        value: crate::metrics::MetricValue,
    ) -> Result<()> {
        if let Some(metrics) = &self.metrics_recorder {
            metrics.record_custom_metric(name, value).await
        } else {
            Ok(()) // Silently ignore if metrics disabled
        }
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus_metrics(&self) -> Result<String> {
        if let Some(metrics) = &self.metrics_recorder {
            metrics.export_prometheus().await
        } else {
            Ok("# Metrics collection disabled\n".to_string())
        }
    }

    /// Check if the manager is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get current configuration
    pub fn config(&self) -> &ObservabilityConfig {
        &self.config
    }

    /// Start the health check background loop
    fn start_health_check_loop(&self) {
        let checker = self.health_checker.as_ref().unwrap().clone();
        let interval_secs = self.config.health.check_interval_secs;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                match checker.perform_all_checks().await {
                    Ok(status) => {
                        if !status.overall_status.is_healthy() {
                            tracing::warn!(
                                status = ?status.overall_status,
                                checks = status.checks.len(),
                                "Health check detected issues"
                            );
                        } else {
                            tracing::debug!("Health check passed");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Health check failed: {}", e);
                    }
                }
            }
        });
    }

    /// Start the metrics collection background loop
    fn start_metrics_collection_loop(&self) {
        let recorder = self.metrics_recorder.as_ref().unwrap().clone();
        let interval_secs = self.config.metrics.collection_interval_secs;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                match recorder.collect_system_metrics().await {
                    Ok(metrics) => {
                        // Check for alerting thresholds
                        if metrics.cpu_usage_percent > 90.0 {
                            tracing::warn!(
                                cpu_usage = %metrics.cpu_usage_percent,
                                "High CPU usage detected"
                            );
                        }

                        if metrics.memory_usage_percent > 95.0 {
                            tracing::warn!(
                                memory_usage = %metrics.memory_usage_percent,
                                "High memory usage detected"
                            );
                        }

                        tracing::debug!(
                            cpu = %metrics.cpu_usage_percent,
                            memory = %metrics.memory_usage_percent,
                            "System metrics collected"
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to collect system metrics: {}", e);
                    }
                }
            }
        });
    }

    /// Record LSP metrics
    pub async fn record_lsp_metrics(
        &self,
        server_name: &str,
        request_count: u64,
        error_count: u64,
        avg_response_time_ms: f64,
    ) -> Result<()> {
        if let Some(metrics) = &self.metrics_recorder {
            metrics
                .record_lsp_metrics(
                    server_name,
                    request_count,
                    error_count,
                    avg_response_time_ms,
                )
                .await
        } else {
            Ok(())
        }
    }

    /// Record AI processing metrics
    pub async fn record_ai_metrics(
        &self,
        operation: &str,
        processing_time_ms: f64,
        tokens_processed: u64,
    ) -> Result<()> {
        if let Some(metrics) = &self.metrics_recorder {
            metrics
                .record_ai_metrics(operation, processing_time_ms, tokens_processed)
                .await
        } else {
            Ok(())
        }
    }

    /// Create a tracing span
    pub fn create_span(&self, name: &str) -> tracing::Span {
        if let Some(tracer) = &self.tracer {
            tracer.create_span(name)
        } else {
            tracing::info_span!("disabled", name = name)
        }
    }

    /// Record a tracing event
    pub fn record_event(&self, event: &str, fields: &[(&str, &str)]) {
        if let Some(tracer) = &self.tracer {
            tracer.record_event(event, fields);
        }
    }
}

impl Drop for ObservabilityManager {
    fn drop(&mut self) {
        // Note: We can't do async cleanup in Drop, but the tokio tasks will be cancelled
        // when the runtime shuts down
    }
}

/// Helper for creating a global observability manager instance
pub struct GlobalObservability;

static mut GLOBAL_MANAGER: Option<ObservabilityManager> = None;

impl GlobalObservability {
    /// Initialize global observability
    pub fn init(config: ObservabilityConfig) -> Result<()> {
        unsafe {
            if GLOBAL_MANAGER.is_some() {
                return Err(crate::errors::ObservabilityError::general(
                    "Global observability already initialized",
                ));
            }

            let manager = ObservabilityManager::new(config);
            // Note: We can't await in unsafe context, so we need to handle this differently
            // In practice, this would be called from an async context
            GLOBAL_MANAGER = Some(manager);
        }

        Ok(())
    }

    /// Get reference to global manager
    pub fn get() -> Option<&'static ObservabilityManager> {
        unsafe { GLOBAL_MANAGER.as_ref() }
    }

    /// Check if global observability is initialized
    pub fn is_initialized() -> bool {
        unsafe { GLOBAL_MANAGER.is_some() }
    }
}
