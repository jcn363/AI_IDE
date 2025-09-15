//! Common error types used across the Rust AI IDE project

use std::time::Duration;

use thiserror::Error;

/// Common error type for all Rust AI IDE operations
#[derive(Error, Debug, Clone)]
pub enum IdeError {
    #[error("IO error: {message}")]
    Io { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Parsing error: {message}")]
    Parse { message: String },

    #[error("Code generation error: {message}")]
    Codegen { message: String },

    #[error("Language server error: {message}")]
    LanguageServer { message: String },

    #[error("Analysis error: {message} (file: {file_path})")]
    Analysis { message: String, file_path: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Timeout error: {message} (operation: {operation})")]
    Timeout {
        message: String,
        operation: String,
        duration: Duration,
    },

    #[error("Networking error: {message}")]
    Network { message: String },

    #[error("Model/AI error: {message} (provider: {provider})")]
    Model { message: String, provider: String },

    #[error("Cancellation error: {message} (operation: {operation})")]
    Cancellation { message: String, operation: String },

    #[error("Concurrent access error: {message}")]
    Concurrency { message: String },

    #[error("Cache error: {message}")]
    Cache { message: String },

    #[error("File operation error: {message}")]
    FileOperation { message: String },

    #[error("Permission denied: {message}")]
    Permission { message: String },

    #[error("Async operation error: {message} (operation: {operation})")]
    AsyncOperation {
        message: String,
        operation: String,
        source: Box<IdeError>,
    },

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Validation error: {field} - {reason}")]
    Validation { field: String, reason: String },

    #[error("Dependency resolution error: {message}")]
    Dependency { message: String },

    #[error("Build/Cargo error: {message}")]
    Build { message: String },

    #[error("Missing resource error: {message}")]
    NotFound { message: String },

    #[error("Invalid input error: {message}")]
    InvalidInput { message: String },

    #[error("Compilation error: {message}")]
    Compilation { message: String },

    #[error("Generic error: {message}")]
    Generic { message: String },
}

/// Result type alias for IDE operations
pub type IdeResult<T> = Result<T, IdeError>;

/// Trait for types that can be converted to IDE errors
pub trait IntoIdeError<T> {
    fn into_ide_error(self) -> IdeResult<T>;
}

impl<T, E: std::error::Error> IntoIdeError<T> for Result<T, E> {
    fn into_ide_error(self) -> IdeResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(IdeError::Generic {
                message: err.to_string(),
            }),
        }
    }
}

/// Common trait for all services that can be initialized
#[async_trait::async_trait]
pub trait Service: Send + Sync {
    /// Initialize the service
    async fn initialize(&mut self) -> IdeResult<()>;

    /// Shutdown the service
    async fn shutdown(&mut self) -> IdeResult<()> {
        Ok(())
    }
}

/// Error conversion utilities for consistent error handling

impl From<std::io::Error> for IdeError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::PermissionDenied {
            IdeError::Permission {
                message: format!("Permission denied: {}", err),
            }
        } else if err.kind() == std::io::ErrorKind::NotFound {
            IdeError::NotFound {
                message: format!("File not found: {}", err),
            }
        } else {
            IdeError::Io {
                message: err.to_string(),
            }
        }
    }
}

impl From<serde_json::Error> for IdeError {
    fn from(err: serde_json::Error) -> Self {
        IdeError::Serialization {
            message: err.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for IdeError {
    fn from(err: std::num::ParseIntError) -> Self {
        IdeError::Parse {
            message: err.to_string(),
        }
    }
}

impl From<regex::Error> for IdeError {
    fn from(err: regex::Error) -> Self {
        IdeError::Parse {
            message: format!("Regex compilation error: {}", err),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for IdeError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        IdeError::Concurrency {
            message: format!("Lock poisoned: {}", err),
        }
    }
}

impl From<std::time::SystemTimeError> for IdeError {
    fn from(err: std::time::SystemTimeError) -> Self {
        IdeError::Generic {
            message: format!("System time error: {}", err),
        }
    }
}

impl From<tokio::time::error::Elapsed> for IdeError {
    fn from(_err: tokio::time::error::Elapsed) -> Self {
        IdeError::Timeout {
            message: "Operation timed out".to_string(),
            operation: "unknown".to_string(),
            duration: std::time::Duration::new(30, 0),
        }
    }
}

/// Generic conversion from any error to IdeError
pub fn convert_error<E: std::error::Error>(err: E) -> IdeError {
    IdeError::Generic {
        message: err.to_string(),
    }
}

/// Convert Option to IdeResult with custom message
pub fn option_to_result<T>(option: Option<T>, message: &str) -> IdeResult<T> {
    option.ok_or_else(|| IdeError::Generic {
        message: message.to_string(),
    })
}

/// Wrap operations with consistent error handling
pub fn wrap_result<T, E: std::error::Error, F: FnOnce(E) -> IdeError>(
    result: Result<T, E>,
    error_converter: F,
) -> IdeResult<T> {
    result.map_err(error_converter)
}

/// Safe unwrap with custom error message
pub fn safe_unwrap<T>(option: Option<T>, error_message: &str) -> IdeResult<T> {
    if let Some(value) = option {
        Ok(value)
    } else {
        Err(IdeError::Generic {
            message: error_message.to_string(),
        })
    }
}

/// Error recovery utilities

/// Attempts an operation with fallback behavior
pub async fn with_fallback<T, E, F, Fut>(
    primary: F,
    fallback: Option<impl FnOnce() -> Fut>,
) -> Result<T, E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::error::Error,
{
    match primary().await {
        Ok(result) => Ok(result),
        Err(err) => {
            if let Some(fallback_fn) = fallback {
                fallback_fn().await
            } else {
                Err(err)
            }
        }
    }
}

/// Retry an operation with exponential backoff
pub async fn retry_with_backoff<T, F, Fut>(
    operation: F,
    max_attempts: usize,
    base_delay_ms: u64,
) -> IdeResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = IdeResult<T>>,
{
    let mut attempts = 0;
    let mut delay_ms = base_delay_ms;

    loop {
        attempts += 1;
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if attempts >= max_attempts {
                    return Err(err);
                }

                log::warn!(
                    "Operation failed (attempt {}/{}): {}",
                    attempts,
                    max_attempts,
                    err
                );
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

                // Exponential backoff
                delay_ms = (delay_ms as f64 * 2.0) as u64;
            }
        }
    }
}

/// Wrap critical operations with error context
pub fn with_context<T, E>(result: Result<T, E>, _context: &str) -> Result<T, E>
where
    E: std::error::Error + std::fmt::Display,
{
    result
}

/// Convert an IdeError to include additional context
pub fn with_error_context(err: IdeError, context: &str) -> IdeError {
    match err {
        IdeError::Io { message } => IdeError::Io {
            message: format!("{}: {}", context, message),
        },
        IdeError::Config { message } => IdeError::Config {
            message: format!("{}: {}", context, message),
        },
        IdeError::Parse { message } => IdeError::Parse {
            message: format!("{}: {}", context, message),
        },
        IdeError::Codegen { message } => IdeError::Codegen {
            message: format!("{}: {}", context, message),
        },
        IdeError::LanguageServer { message } => IdeError::LanguageServer {
            message: format!("{}: {}", context, message),
        },
        IdeError::Analysis { message, file_path } => IdeError::Analysis {
            message: format!("{}: {}", context, message),
            file_path,
        },
        IdeError::Serialization { message } => IdeError::Serialization {
            message: format!("{}: {}", context, message),
        },
        IdeError::Timeout {
            message,
            operation,
            duration,
        } => IdeError::Timeout {
            message: format!("{}: {}", context, message),
            operation,
            duration,
        },
        IdeError::Network { message } => IdeError::Network {
            message: format!("{}: {}", context, message),
        },
        IdeError::Model { message, provider } => IdeError::Model {
            message: format!("{}: {}", context, message),
            provider,
        },
        IdeError::Cancellation { message, operation } => IdeError::Cancellation {
            message: format!("{}: {}", context, message),
            operation,
        },

        IdeError::Concurrency { message } => IdeError::Concurrency {
            message: format!("{}: {}", context, message),
        },
        IdeError::Cache { message } => IdeError::Cache {
            message: format!("{}: {}", context, message),
        },
        IdeError::Permission { message } => IdeError::Permission {
            message: format!("{}: {}", context, message),
        },
        IdeError::Validation { field, reason } => IdeError::Validation {
            field: format!("{}: {}", context, field),
            reason,
        },
        IdeError::Dependency { message } => IdeError::Dependency {
            message: format!("{}: {}", context, message),
        },
        IdeError::Build { message } => IdeError::Build {
            message: format!("{}: {}", context, message),
        },
        IdeError::NotFound { message } => IdeError::NotFound {
            message: format!("{}: {}", context, message),
        },
        IdeError::InvalidInput { message } => IdeError::InvalidInput {
            message: format!("{}: {}", context, message),
        },
        IdeError::Compilation { message } => IdeError::Compilation {
            message: format!("{}: {}", context, message),
        },
        IdeError::FileOperation { message } => IdeError::FileOperation {
            message: format!("{}: {}", context, message),
        },
        IdeError::AsyncOperation {
            message,
            operation,
            source,
        } => IdeError::AsyncOperation {
            message: format!("{}: {}", context, message),
            operation,
            source,
        },
        IdeError::Internal { message } => IdeError::Internal {
            message: format!("{}: {}", context, message),
        },
        IdeError::Generic { message } => IdeError::Generic {
            message: format!("{}: {}", context, message),
        },
    }
}
