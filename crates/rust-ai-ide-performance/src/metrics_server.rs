//! HTTP Metrics Server for Prometheus-Compatible Endpoints
//!
//! This module provides an HTTP server that exposes performance metrics
//! in Prometheus-compatible format for scraping.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use serde::{Deserialize, Serialize};

use crate::metrics::{PrometheusExporter, MetricsRegistry};
use crate::UnifiedPerformanceCollector;

/// Configuration for the metrics server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsServerConfig {
    /// Server address to bind to
    pub address: String,
    /// Server port
    pub port: u16,
    /// Path for metrics endpoint (default: /metrics)
    pub metrics_path: String,
    /// Enable health check endpoint
    pub enable_health_check: bool,
    /// Health check path (default: /health)
    pub health_check_path: String,
}

impl Default for MetricsServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 9090,
            metrics_path: "/metrics".to_string(),
            enable_health_check: true,
            health_check_path: "/health".to_string(),
        }
    }
}

/// Metrics HTTP server
pub struct MetricsServer {
    config: MetricsServerConfig,
    exporter: Arc<PrometheusExporter>,
}

impl MetricsServer {
    pub fn new(config: MetricsServerConfig, exporter: Arc<PrometheusExporter>) -> Self {
        Self { config, exporter }
    }

    pub async fn start(&self) -> Result<(), MetricsServerError> {
        let addr: SocketAddr = format!("{}:{}", self.config.address, self.config.port)
            .parse()
            .map_err(|e| MetricsServerError::InvalidAddress(e.to_string()))?;

        log::info!("Starting metrics server on {}", addr);

        // Clone exporter for use in routes
        let metrics_exporter = Arc::clone(&self.exporter);
        let health_exporter = Arc::clone(&self.exporter);

        // Metrics endpoint
        let metrics_route = warp::path(self.config.metrics_path.trim_start_matches('/'))
            .and(warp::get())
            .and(warp::any().map(move || Arc::clone(&metrics_exporter)))
            .and_then(Self::handle_metrics);

        // Health check endpoint
        let health_route = if self.config.enable_health_check {
            warp::path(self.config.health_check_path.trim_start_matches('/'))
                .and(warp::get())
                .and(warp::any().map(move || Arc::clone(&health_exporter)))
                .and_then(Self::handle_health)
                .boxed()
        } else {
            warp::any().map(|| warp::reply::with_status("Not Found", warp::http::StatusCode::NOT_FOUND)).boxed()
        };

        // Combine routes
        let routes = metrics_route.or(health_route);

        // Add CORS and logging
        let routes = routes
            .with(warp::cors().allow_any_origin())
            .with(warp::log::custom(|info| {
                log::info!("{} {} {}", info.method(), info.path(), info.status());
            }));

        warp::serve(routes)
            .run(addr)
            .await;

        Ok(())
    }

    async fn handle_metrics(exporter: Arc<PrometheusExporter>) -> Result<impl warp::Reply, warp::Rejection> {
        match exporter.collect_and_export().await {
            Ok(metrics) => {
                Ok(warp::reply::with_header(
                    metrics,
                    "Content-Type",
                    "text/plain; version=0.0.4; charset=utf-8"
                ))
            }
            Err(e) => {
                log::error!("Failed to collect metrics: {}", e);
                Ok(warp::reply::with_status(
                    format!("Error collecting metrics: {}", e),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR
                ))
            }
        }
    }

    async fn handle_health(exporter: Arc<PrometheusExporter>) -> Result<impl warp::Reply, warp::Rejection> {
        // Simple health check - just verify we can access the exporter
        let registry = exporter.get_registry();
        let metrics_count = registry.export_all().await.lines().count();

        let health_response = serde_json::json!({
            "status": "healthy",
            "metrics_count": metrics_count,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(warp::reply::json(&health_response))
    }
}

/// Metrics server errors
#[derive(Debug, Clone)]
pub enum MetricsServerError {
    InvalidAddress(String),
    ServerError(String),
}

impl std::fmt::Display for MetricsServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricsServerError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            MetricsServerError::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

impl std::error::Error for MetricsServerError {}

/// Builder for creating and configuring a metrics server
pub struct MetricsServerBuilder {
    config: MetricsServerConfig,
}

impl MetricsServerBuilder {
    pub fn new() -> Self {
        Self {
            config: MetricsServerConfig::default(),
        }
    }

    pub fn with_address(mut self, address: String) -> Self {
        self.config.address = address;
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    pub fn with_metrics_path(mut self, path: String) -> Self {
        self.config.metrics_path = path;
        self
    }

    pub fn enable_health_check(mut self, enable: bool) -> Self {
        self.config.enable_health_check = enable;
        self
    }

    pub fn with_health_check_path(mut self, path: String) -> Self {
        self.config.health_check_path = path;
        self
    }

    pub fn with_config(mut self, config: MetricsServerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self, exporter: Arc<PrometheusExporter>) -> MetricsServer {
        MetricsServer::new(self.config, exporter)
    }
}

impl Default for MetricsServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollectorBuilder, SystemMetricsProvider};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_metrics_server_builder() {
        let collector = Arc::new(CollectorBuilder::new().build());
        let exporter = Arc::new(PrometheusExporter::new(Arc::clone(&collector)));

        let server = MetricsServerBuilder::new()
            .with_address("127.0.0.1".to_string())
            .with_port(9091)
            .with_metrics_path("/metrics".to_string())
            .build(exporter);

        assert_eq!(server.config.address, "127.0.0.1");
        assert_eq!(server.config.port, 9091);
        assert_eq!(server.config.metrics_path, "/metrics");
    }

    #[tokio::test]
    async fn test_prometheus_exporter_initialization() {
        let collector = Arc::new(CollectorBuilder::new().build());
        let exporter = PrometheusExporter::new(Arc::clone(&collector));

        // Initialize default metrics
        exporter.initialize_default_metrics().await.unwrap();

        // Check that metrics were registered
        let registry = exporter.get_registry();
        let metrics_output = registry.export_all().await;

        assert!(metrics_output.contains("rust_ai_ide_cpu_usage_percent"));
        assert!(metrics_output.contains("rust_ai_ide_memory_usage_bytes"));
        assert!(metrics_output.contains("# HELP"));
        assert!(metrics_output.contains("# TYPE"));
    }
}