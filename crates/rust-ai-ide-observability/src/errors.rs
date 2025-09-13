//! Error types for the observability framework

use thiserror::Error;

/// Result type alias for observability operations
pub type Result<T> = std::result::Result<T, ObservabilityError>;

/// Comprehensive error types for observability operations
#[derive(Error, Debug)]
pub enum ObservabilityError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Metrics collection error: {message}")]
    Metrics { message: String },

    #[error("Tracing initialization error: {message}")]
    Tracing { message: String },

    #[error("Health check failed: {service} - {message}")]
    HealthCheck { service: String, message: String },

    #[error("Alerting system error: {message}")]
    Alerting { message: String },

    #[error("Security monitoring error: {message}")]
    Security { message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("System info error: {0}")]
    SystemInfo(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("General observability error: {0}")]
    General(String),
}

impl ObservabilityError {
    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a metrics error
    pub fn metrics(message: impl Into<String>) -> Self {
        Self::Metrics {
            message: message.into(),
        }
    }

    /// Create a tracing error
    pub fn tracing(message: impl Into<String>) -> Self {
        Self::Tracing {
            message: message.into(),
        }
    }

    /// Create a health check error
    pub fn health_check(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::HealthCheck {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create an alerting error
    pub fn alerting(message: impl Into<String>) -> Self {
        Self::Alerting {
            message: message.into(),
        }
    }

    /// Create a security error
    pub fn security(message: impl Into<String>) -> Self {
        Self::Security {
            message: message.into(),
        }
    }

    /// Create a system info error
    pub fn system_info(message: impl Into<String>) -> Self {
        Self::SystemInfo(message.into())
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database(message.into())
    }

    /// Create a general error
    pub fn general(message: impl Into<String>) -> Self {
        Self::General(message.into())
    }

    /// Check if this is a critical error that should trigger alerts
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::HealthCheck { .. } | Self::Security { .. })
    }

    /// Get the error category for reporting
    pub fn category(&self) -> &'static str {
        match self {
            Self::Config { .. } => "configuration",
            Self::Metrics { .. } => "metrics",
            Self::Tracing { .. } => "tracing",
            Self::HealthCheck { .. } => "health_check",
            Self::Alerting { .. } => "alerting",
            Self::Security { .. } => "security",
            Self::Io(_) => "io",
            Self::Serde(_) => "serialization",
            Self::Env(_) => "environment",
            Self::Parse(_) => "parsing",
            Self::SystemInfo(_) => "system_info",
            Self::Network(_) => "network",
            Self::Database(_) => "database",
            Self::General(_) => "general",
        }
    }
}
