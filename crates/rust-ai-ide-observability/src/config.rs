//! Configuration for the observability framework
//!
//! Provides secure configuration management for all observability features,
//! supporting environment variable injection and secrets management.

use serde::{Deserialize, Serialize};
use std::env;

/// Main configuration structure for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable/disable observability features
    pub enabled: bool,

    /// Tracing configuration
    pub tracing: TracingConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Health check configuration
    pub health: HealthConfig,

    /// Alerting configuration
    pub alerting: AlertingConfig,

    /// Security monitoring configuration
    pub security: SecurityConfig,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tracing: TracingConfig::default(),
            metrics: MetricsConfig::default(),
            health: HealthConfig::default(),
            alerting: AlertingConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl ObservabilityConfig {
    /// Load configuration from environment variables with secure defaults
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("RUST_AI_IDE_OBSERVABILITY_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            tracing: TracingConfig::from_env(),
            metrics: MetricsConfig::from_env(),
            health: HealthConfig::from_env(),
            alerting: AlertingConfig::from_env(),
            security: SecurityConfig::from_env(),
        }
    }

    /// Validate configuration for security and correctness
    pub fn validate(&self) -> Result<(), String> {
        if self.metrics.prometheus_port > 65535 {
            return Err("Invalid Prometheus port".to_string());
        }

        if self.health.check_interval_secs == 0 {
            return Err("Health check interval must be greater than 0".to_string());
        }

        if self.alerting.max_alerts_per_hour > 1000 {
            return Err("Max alerts per hour too high".to_string());
        }

        Ok(())
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,

    /// Tracing level (ERROR, WARN, INFO, DEBUG, TRACE)
    pub level: String,

    /// Enable OpenTelemetry export
    pub otel_enabled: bool,

    /// OpenTelemetry endpoint (if enabled)
    pub otel_endpoint: Option<String>,

    /// Service name for tracing
    pub service_name: String,

    /// Service version
    pub service_version: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "INFO".to_string(),
            otel_enabled: false,
            otel_endpoint: None,
            service_name: "rust-ai-ide".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl TracingConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("RUST_AI_IDE_TRACING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            level: env::var("RUST_AI_IDE_TRACING_LEVEL").unwrap_or_else(|_| "INFO".to_string()),

            otel_enabled: env::var("RUST_AI_IDE_OTEL_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            otel_endpoint: env::var("RUST_AI_IDE_OTEL_ENDPOINT").ok(),

            service_name: env::var("RUST_AI_IDE_SERVICE_NAME")
                .unwrap_or_else(|_| "rust-ai-ide".to_string()),

            service_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics prefix for all metrics
    pub prefix: String,

    /// Prometheus exporter port
    pub prometheus_port: u16,

    /// Metrics retention period in hours
    pub retention_hours: u64,

    /// Enable system metrics collection
    pub system_metrics_enabled: bool,

    /// Enable application metrics collection
    pub app_metrics_enabled: bool,

    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: "rust_ai_ide".to_string(),
            prometheus_port: 9090,
            retention_hours: 24,
            system_metrics_enabled: true,
            app_metrics_enabled: true,
            collection_interval_secs: 10,
        }
    }
}

impl MetricsConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("RUST_AI_IDE_METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            prefix: env::var("RUST_AI_IDE_METRICS_PREFIX")
                .unwrap_or_else(|_| "rust_ai_ide".to_string()),

            prometheus_port: env::var("RUST_AI_IDE_PROMETHEUS_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()
                .unwrap_or(9090),

            retention_hours: env::var("RUST_AI_IDE_METRICS_RETENTION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),

            system_metrics_enabled: env::var("RUST_AI_IDE_SYSTEM_METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            app_metrics_enabled: env::var("RUST_AI_IDE_APP_METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            collection_interval_secs: env::var("RUST_AI_IDE_METRICS_COLLECTION_INTERVAL")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Health check interval in seconds
    pub check_interval_secs: u64,

    /// Maximum response time for health checks in seconds
    pub max_response_time_secs: u64,

    /// Enable database health checks
    pub database_checks_enabled: bool,

    /// Enable LSP server health checks
    pub lsp_checks_enabled: bool,

    /// Enable AI service health checks
    pub ai_checks_enabled: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 30,
            max_response_time_secs: 5,
            database_checks_enabled: true,
            lsp_checks_enabled: true,
            ai_checks_enabled: true,
        }
    }
}

impl HealthConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("RUST_AI_IDE_HEALTH_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            check_interval_secs: env::var("RUST_AI_IDE_HEALTH_CHECK_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),

            max_response_time_secs: env::var("RUST_AI_IDE_HEALTH_MAX_RESPONSE_TIME")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),

            database_checks_enabled: env::var("RUST_AI_IDE_HEALTH_DATABASE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            lsp_checks_enabled: env::var("RUST_AI_IDE_HEALTH_LSP_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            ai_checks_enabled: env::var("RUST_AI_IDE_HEALTH_AI_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Enable alerting
    pub enabled: bool,

    /// Maximum alerts per hour to prevent spam
    pub max_alerts_per_hour: u32,

    /// CPU usage warning threshold
    pub cpu_warning_threshold: f64,

    /// Memory usage warning threshold
    pub memory_warning_threshold: f64,

    /// Disk usage warning threshold
    pub disk_warning_threshold: f64,

    /// LSP error rate warning threshold
    pub lsp_error_rate_threshold: f64,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_alerts_per_hour: 100,
            cpu_warning_threshold: 85.0,
            memory_warning_threshold: 90.0,
            disk_warning_threshold: 95.0,
            lsp_error_rate_threshold: 5.0,
        }
    }
}

impl AlertingConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("RUST_AI_IDE_ALERTING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            max_alerts_per_hour: env::var("RUST_AI_IDE_MAX_ALERTS_PER_HOUR")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),

            cpu_warning_threshold: env::var("RUST_AI_IDE_CPU_WARNING_THRESHOLD")
                .unwrap_or_else(|_| "85.0".to_string())
                .parse()
                .unwrap_or(85.0),

            memory_warning_threshold: env::var("RUST_AI_IDE_MEMORY_WARNING_THRESHOLD")
                .unwrap_or_else(|_| "90.0".to_string())
                .parse()
                .unwrap_or(90.0),

            disk_warning_threshold: env::var("RUST_AI_IDE_DISK_WARNING_THRESHOLD")
                .unwrap_or_else(|_| "95.0".to_string())
                .parse()
                .unwrap_or(95.0),

            lsp_error_rate_threshold: env::var("RUST_AI_IDE_LSP_ERROR_RATE_THRESHOLD")
                .unwrap_or_else(|_| "5.0".to_string())
                .parse()
                .unwrap_or(5.0),
        }
    }
}

/// Security monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security monitoring
    pub enabled: bool,

    /// Log security events
    pub log_security_events: bool,

    /// Monitor authentication attempts
    pub monitor_auth_attempts: bool,

    /// Monitor file access patterns
    pub monitor_file_access: bool,

    /// Monitor network connections
    pub monitor_network: bool,

    /// Alert on suspicious activities
    pub alert_on_suspicious_activity: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_security_events: true,
            monitor_auth_attempts: true,
            monitor_file_access: true,
            monitor_network: true,
            alert_on_suspicious_activity: true,
        }
    }
}

impl SecurityConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("RUST_AI_IDE_SECURITY_MONITORING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            log_security_events: env::var("RUST_AI_IDE_LOG_SECURITY_EVENTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            monitor_auth_attempts: env::var("RUST_AI_IDE_MONITOR_AUTH_ATTEMPTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            monitor_file_access: env::var("RUST_AI_IDE_MONITOR_FILE_ACCESS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            monitor_network: env::var("RUST_AI_IDE_MONITOR_NETWORK")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            alert_on_suspicious_activity: env::var("RUST_AI_IDE_ALERT_SUSPICIOUS_ACTIVITY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}
