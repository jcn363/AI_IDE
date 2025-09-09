//! Error types for the monitoring framework

use std::fmt;
use thiserror::Error;

/// Result type alias for monitoring operations
pub type Result<T> = std::result::Result<T, MonitoringError>;

/// Monitoring framework errors
#[derive(Error, Debug)]
pub enum MonitoringError {
    /// Configuration-related errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Command execution errors
    #[error("Command execution failed: {command} - {source}")]
    CommandExecution { command: String, source: std::io::Error },

    /// Process execution errors with exit code
    #[error("Command '{command}' failed with exit code {exit_code}: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    /// JSON parsing errors
    #[error("JSON parsing error: {source}")]
    JsonParse {
        #[from]
        source: serde_json::Error,
    },

    /// File system errors
    #[error("File system error: {source}")]
    Fs {
        #[from]
        source: std::io::Error,
    },

    /// Path-related errors
    #[error("Path error: {path} - {message}")]
    Path { path: String, message: String },

    /// Analysis timeout errors
    #[error("Analysis timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// Cargo check output parsing errors
    #[error("Cargo check parsing error: {message}")]
    CargoParse { message: String },

    /// HTTP/network errors for notifications
    #[cfg(feature = "reqwest")]
    #[error("HTTP error: {source}")]
    Http {
        #[from]
        source: reqwest::Error,
    },

    /// Database errors for metrics storage
    #[cfg(feature = "rusqlite")]
    #[error("Database error: {source}")]
    Database {
        #[from]
        source: rusqlite::Error,
    },

    /// Workspace analysis errors
    #[error("Workspace analysis error: {message}")]
    Analysis { message: String },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Timeout-related errors
    #[cfg(feature = "tokio")]
    #[error("Async timeout error: {source}")]
    AsyncTimeout {
        #[from]
        source: tokio::time::error::Elapsed,
    },

    /// Generic errors
    #[error("Monitoring error: {message}")]
    Other { message: String },
}

impl MonitoringError {
    /// Create a new configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new command execution error
    pub fn command_execution(command: impl Into<String>, source: std::io::Error) -> Self {
        Self::CommandExecution {
            command: command.into(),
            source,
        }
    }

    /// Create a new command failure error
    pub fn command_failed(
        command: impl Into<String>,
        exit_code: i32,
        stderr: impl Into<String>,
    ) -> Self {
        Self::CommandFailed {
            command: command.into(),
            exit_code,
            stderr: stderr.into(),
        }
    }

    /// Create a new path error
    pub fn path(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Path {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a new cargo parsing error
    pub fn cargo_parse(message: impl Into<String>) -> Self {
        Self::CargoParse {
            message: message.into(),
        }
    }

    /// Create a new analysis error
    pub fn analysis(message: impl Into<String>) -> Self {
        Self::Analysis {
            message: message.into(),
        }
    }

    /// Create a new validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a new generic error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }

    /// Check if this is a critical error that should stop monitoring
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::Config { .. }
                | Self::CommandExecution { .. }
                | Self::Timeout { .. }
                | Self::Validation { .. }
        )
    }

    /// Check if this error should trigger notifications
    pub fn should_notify(&self) -> bool {
        matches!(
            self,
            Self::CommandFailed { .. }
                | Self::CargoParse { .. }
                | Self::Analysis { .. }
                | Self::Timeout { .. }
        )
    }
}