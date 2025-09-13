//! # Rust AI IDE Observability Framework
//!
//! Comprehensive observability framework providing structured logging,
//! metrics collection, health checks, and performance monitoring for the
//! Rust AI IDE. Integrates with OpenTelemetry, Prometheus, and custom
//! monitoring systems.
//!
//! ## Features
//!
//! - **Structured Logging**: OpenTelemetry-compatible tracing with spans and events
//! - **Metrics Collection**: Prometheus-compatible metrics for system and application monitoring
//! - **Health Checks**: Automated health verification for critical services
//! - **Performance Monitoring**: Real-time performance metrics and alerting
//! - **Event-Driven Monitoring**: Integration with EventBus for reactive monitoring
//! - **Security Monitoring**: Enhanced security operation tracking
//!
//! ## Architecture
//!
//! The observability framework consists of several key components:
//!
//! - `ObservabilityManager`: Central orchestrator for all monitoring activities
//! - `MetricsRecorder`: Handles metrics collection and export
//! - `HealthChecker`: Manages health check endpoints and status
//! - `Tracer`: Provides distributed tracing capabilities
//! - `AlertManager`: Handles threshold-based alerting
//!
//! ## Usage
//!
//! ```rust,no_run
//! use rust_ai_ide_observability::{ObservabilityManager, ObservabilityConfig};
//!
//! // Initialize observability with default configuration
//! let config = ObservabilityConfig::default();
//! let manager = ObservabilityManager::new(config).await?;
//!
//! // Start monitoring
//! manager.start().await?;
//!
//! // Record custom metrics
//! metrics::counter!("lsp_requests_total", 1);
//! metrics::histogram!("lsp_response_time", 15.2);
//!
//! // Health check
//! let health = manager.health_check().await?;
//! println!("System health: {:?}", health.status);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod config;
pub mod errors;
pub mod health;
pub mod manager;
pub mod metrics;
pub mod tracing;

pub use config::ObservabilityConfig;
pub use errors::{ObservabilityError, Result};
pub use manager::ObservabilityManager;
pub use metrics::{MetricsRecorder, PerformanceMetrics, SystemMetrics};
pub use tracing::Tracer;

// Re-export commonly used types
pub use serde::{Deserialize, Serialize};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize global observability systems
///
/// This function sets up tracing, metrics collection, and health monitoring
/// with the provided configuration. Should be called early in application startup.
pub async fn init_observability(config: ObservabilityConfig) -> Result<ObservabilityManager> {
    let manager = ObservabilityManager::new(config).await?;
    manager.start().await?;
    Ok(manager)
}

/// Convenience macro for recording metrics with context
#[macro_export]
macro_rules! record_metric {
    ($name:expr, $value:expr) => {
        metrics::gauge!($name, $value);
    };
    ($name:expr, $value:expr, $($label:expr => $label_value:expr),*) => {
        metrics::gauge!($name, $value, $($label => $label_value),*);
    };
}

/// Convenience macro for tracing operations
#[macro_export]
macro_rules! trace_operation {
    ($operation:expr, $body:block) => {
        let span = tracing::info_span!("operation", name = $operation);
        let _enter = span.enter();
        tracing::info!("Starting operation: {}", $operation);
        let result = $body;
        tracing::info!("Completed operation: {}", $operation);
        result
    };
}

/// Default configuration values
pub mod defaults {
    pub const DEFAULT_METRICS_PORT: u16 = 9090;
    pub const DEFAULT_HEALTH_CHECK_INTERVAL_SECS: u64 = 30;
    pub const DEFAULT_METRICS_RETENTION_HOURS: u64 = 24;
    pub const DEFAULT_TRACING_LEVEL: &str = "INFO";
    pub const DEFAULT_METRICS_PREFIX: &str = "rust_ai_ide";
}
