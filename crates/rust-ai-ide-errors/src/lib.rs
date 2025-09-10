//! Unified error handling for Rust AI IDE
//!
//! This crate provides comprehensive error definitions and handling utilities
//! that are reused across all crates in the Rust AI IDE ecosystem.
//!
//! # Features
//!
//! - Comprehensive 17-variant error enum with thiserror derive
//! - Automatic From trait implementations for common error conversions
//! - Enhanced error context with spans, traces, and metadata
//! - Error chaining and source tracking capabilities
//! - Backward compatibility during migration
//!
//! # Migration Guide
//!
//! ## From IDEError to RustAIError
//!
//! Replace `use crate::errors::IDEError;` with `use rust_ai_ide_errors::RustAIError;`
//!
//! ### Error Variant Mapping
//!
//! ```text
//! Old: IDEError::Generic(msg)          → New: RustAIError::Generic(msg)
//! Old: IDEError::Network(msg)          → New: RustAIError::Network(msg)
//! Old: IDEError::Validation(msg)       → New: RustAIError::Validation(msg)
//! Old: IDEError::Protocol(msg)         → New: RustAIError::Protocol(msg)
//! Old: IDEError::Timeout(msg)          → New: RustAIError::Timeout(msg)
//! Old: IDEError::InternalError(msg)    → New: RustAIError::InternalError(msg)
//! Old: IDEError::Path(err)             → New: RustAIError::Path(err)
//! Old: IDEError::Authentication(msg)   → New: RustAIError::Authentication(msg)
//! Old: IDEError::Config(err)           → New: RustAIError::Config(err)
//! Old: IDEError::RateLimit(msg)        → New: RustAIError::RateLimit(msg)
//! Old: IDEError::ServiceUnavailable(msg) → New: RustAIError::ServiceUnavailable(msg)
//! Old: IDEError::Io(err)               → New: RustAIError::Io(err)
//! Old: IDEError::FileSystem(msg)       → New: RustAIError::FileSystem(msg)
//!
//! New variants available:
//! - RustAIError::Database(String)
//! - RustAIError::Serialization(String)
//! - RustAIError::Concurrency(String)
//! - RustAIError::Compilation(String)
//! ```
//!
//! ### Enhanced Context Usage
//!
//! ```rust
//! use rust_ai_ide_errors::{RustAIError, EnhancedContext, ResultContextExt};
//!
//! // Old way: Custom error handling
//! // New way: Leverage built-in context
//! let result: Result<String, RustAIError> = some_operation();
//! let result_with_context = result.context(EnhancedContext::new("operation_name"));
//! ```
//!
//! ### Automatic Conversions
//!
//! ```rust
//! use rust_ai_ide_errors::{RustAIError, IDEResult};
//!
//! fn old_function() -> Result<i32, std::io::Error> {
//!     // Implementation
//!     Ok(42)
//! }
//!
//! fn new_function() -> IDEResult<i32> {
//!     // Automatic conversion from std::io::Error to RustAIError
//!     old_function()
//! }
//! ```

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Result type alias for IDE operations
///
/// This is the recommended result type for all operations in the Rust AI IDE.
/// It provides automatic error classification and enhanced error context capabilities.
///
/// # Examples
///
/// ```rust
/// use rust_ai_ide_errors::{IDEResult, RustAIError, EnhancedContext};
///
/// fn safe_file_operation() -> IDEResult<String> {
///     // Some file operation that might fail
///     Ok("content".to_string())
/// }
///
/// fn handle_error_with_context() -> IDEResult<String> {
///     safe_file_operation().context(EnhancedContext::new("file_operation"))
/// }
/// ```
pub type IDEResult<T> = Result<T, RustAIError>;

/// Main unified error type for the Rust AI IDE - 17 comprehensive variants
///
/// This enum provides comprehensive error categorization for all possible error conditions
/// in the Rust AI IDE ecosystem. Each variant is specifically designed to handle common
/// failure scenarios with appropriate context.
///
/// # Error Variants Overview
///
/// - `Generic` - General purpose errors without specific categorization
/// - `Network` - Network-related failures (connections, timeouts, DNS)
/// - `Validation` - Input/data validation errors
/// - `Protocol` - Protocol-level communication errors
/// - `Timeout` - Operation timeout scenarios
/// - `InternalError` - System internal failures
/// - `Path` - Filesystem path related errors
/// - `Authentication` - Authentication/authorization failures
/// - `Config` - Configuration loading/parsing errors
/// - `RateLimit` - API rate limit exceeded
/// - `ServiceUnavailable` - External service availability issues
/// - `Io` - Low-level I/O operation failures
/// - `FileSystem` - Higher-level filesystem operations
/// - `Database` - Database connection/query errors
/// - `Serialization` - Data serialization/deserialization failures
/// - `Concurrency` - Concurrent operation conflicts
/// - `Compilation` - Rust compilation errors and diagnostics
///
/// # Usage Examples
///
/// ```rust
/// use rust_ai_ide_errors::RustAIError;
///
/// // Direct error creation
/// let not_found = RustAIError::Path(rust_ai_ide_errors::PathError::new("File not found"));
///
/// // Error conversion from standard types
/// let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
/// let rustai_error: RustAIError = io_error.into();
///
/// // Error with enhanced context
/// use rust_ai_ide_errors::{EnhancedContext, ResultContextExt};
/// let context = EnhancedContext::new("read_file")
///     .with_resource("/path/to/file")
///     .with_metadata("user_id", "123");
/// ```
#[derive(thiserror::Error, Debug, Clone)]
pub enum RustAIError {
    /// Generic error with custom message
    #[error("Generic error: {0}")]
    Generic(String),

    /// Network-related error
    #[error("Network error: {0}")]
    Network(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Invalid argument error
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Internal system error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Path-related error
    #[error("Path error: {0}")]
    Path(#[from] PathError),

    /// Authentication/authorization error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Rate limit exceeded
    #[error("Rate limit error: {0}")]
    RateLimit(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    /// Filesystem error
    #[error("Filesystem error: {0}")]
    FileSystem(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Concurrency error
    #[error("Concurrency error: {0}")]
    Concurrency(String),

    /// Compilation error
    #[error("Compilation error: {0}")]
    Compilation(String),
}

/// Context information for enhanced error debugging
///
/// EnhancedContext provides rich metadata for error scenarios, enabling better
/// error reporting, debugging, and tracing capabilities throughout the system.
///
/// # Fields
///
/// - `operation` - The operation that failed
/// - `resource` - Resource involved (file path, URL, etc.)
/// - `metadata` - Additional key-value metadata
/// - `timestamp` - When the error occurred
/// - `trace_id` - Distributed tracing identifier
/// - `source_location` - Source code location (file:line)
/// - `operation_chain` - Chain of operations leading to this error
///
/// # Usage Examples
///
/// ```rust
/// use rust_ai_ide_errors::EnhancedContext;
///
/// // Basic context
/// let context = EnhancedContext::new("process_file");
///
/// // Full context with metadata
/// let context = EnhancedContext::new("api_request")
///     .with_resource("https://api.example.com/files")
///     .with_metadata("method", "POST")
///     .with_metadata("content_type", "application/json")
///     .with_trace("trace-123")
///     .push_operation("validate_input")
///     .push_operation("process_data");
/// ```
///
/// # Migration Patterns
///
/// When migrating from the old IDEError system:
/// 1. Replace `IDEError::*` variants with `RustAIError::*` equivalents
/// 2. Use `EnhancedContext` for complex error scenarios requiring metadata
/// 3. Leverage the `ResultContextExt` trait for automatic error enhancement
/// 4. Update imports: `use rust_ai_ide_errors::*;` instead of local error types
#[derive(Debug, Clone, Default)]
pub struct EnhancedContext {
    /// Operation that failed
    pub operation: String,
    /// Resource involved (file path, URL, etc.)
    pub resource: Option<String>,
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
    /// Timestamp of error occurrence
    pub timestamp: DateTime<Utc>,
    /// Stack trace or span information
    pub trace_id: Option<String>,
    /// Source location (file:line)
    pub source_location: Option<String>,
    /// Chain of operations leading to this error
    pub operation_chain: Vec<String>,
}

impl EnhancedContext {
    /// Create new context with basic information
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            timestamp: Utc::now(),
            ..Default::default()
        }
    }

    /// Add resource information
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add trace information
    pub fn with_trace(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Add source location
    pub fn with_source(mut self, location: impl Into<String>) -> Self {
        self.source_location = Some(location.into());
        self
    }

    /// Add operation to chain
    pub fn push_operation(mut self, op: impl Into<String>) -> Self {
        self.operation_chain.push(op.into());
        self
    }
}

/// Extension trait for Result to add context
pub trait ResultContextExt<T> {
    fn with_context<F>(self, f: F) -> Result<T, RustAIError>
    where
        F: FnOnce() -> EnhancedContext;

    fn context(self, ctx: EnhancedContext) -> Result<T, RustAIError>;
}

impl<T, E> ResultContextExt<T> for Result<T, E>
where
    E: Into<RustAIError>,
{
    fn with_context<F>(self, f: F) -> Result<T, RustAIError>
    where
        F: FnOnce() -> EnhancedContext,
    {
        self.map_err(|e| {
            let error: RustAIError = e.into();
            match error {
                RustAIError::InternalError(msg) => {
                    let ctx = f();
                    RustAIError::InternalError(format!(
                        "{} | Context: op={}, resource={:?}, trace={:?}",
                        msg, ctx.operation, ctx.resource, ctx.trace_id
                    ))
                }
                _ => error,
            }
        })
    }

    fn context(self, ctx: EnhancedContext) -> Result<T, RustAIError> {
        self.with_context(|| ctx)
    }
}

/// Automatic conversions for common error types
impl From<std::io::Error> for RustAIError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => RustAIError::Path(PathError::new("File not found")),
            std::io::ErrorKind::PermissionDenied => {
                RustAIError::Path(PathError::new("Permission denied"))
            }
            std::io::ErrorKind::AlreadyExists => {
                RustAIError::InternalError("Resource already exists".into())
            }
            std::io::ErrorKind::TimedOut => RustAIError::Timeout("I/O operation timed out".into()),
            _ => RustAIError::Io(IoError::new(error.to_string())),
        }
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for RustAIError {
    fn from(error: serde_json::Error) -> Self {
        RustAIError::Serialization(error.to_string())
    }
}

// Note: This conversion is conditionally available when candle-core is used
// In the AI quantization crate, candle_core errors are handled via direct implementation

impl From<std::string::FromUtf8Error> for RustAIError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        RustAIError::InternalError(format!("UTF-8 conversion error: {}", error))
    }
}

impl From<std::num::ParseIntError> for RustAIError {
    fn from(error: std::num::ParseIntError) -> Self {
        RustAIError::Validation(format!("Integer parsing error: {}", error))
    }
}

/// Path-related error type
#[derive(thiserror::Error, Debug, Clone)]
#[error("{message}")]
pub struct PathError {
    message: String,
    path: Option<String>,
}

impl PathError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// Configuration error type
#[derive(thiserror::Error, Debug, Clone)]
#[error("Configuration error: {message}")]
pub struct ConfigError {
    message: String,
    field: Option<String>,
}

impl ConfigError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
        }
    }

    pub fn for_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

/// I/O error type
#[derive(thiserror::Error, Debug, Clone)]
#[error("I/O error: {message}")]
pub struct IoError {
    message: String,
}

impl IoError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub mod conversions;
