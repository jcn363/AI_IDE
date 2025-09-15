//! Error conversion utilities for seamless error handling
//!
//! This module provides traits and utilities for converting various error types
//! to the unified RustAIError hierarchy, ensuring consistent error handling across
//! the entire codebase.
//!
//! # Core Features
//!
//! - Generic conversion from any `std::error::Error` to RustAIError
//! - Pattern-based conversion for string error messages
//! - Utility functions for common error handling patterns
//! - Bridge functions for gradual migration

use super::{ConfigError, EnhancedContext, IoError, PathError, RustAIError};

/// Core trait for converting errors to RustAIError
pub trait ToRustAIError {
    fn to_rustai_error(self) -> RustAIError;
}

/// Generic conversion from any std::error::Error to RustAIError
impl<E: std::error::Error> ToRustAIError for E {
    fn to_rustai_error(self) -> RustAIError {
        RustAIError::InternalError(format!("GENERIC_ERROR: {}", self))
    }
}

/// Convert common error string patterns to RustAIError (bridge for legacy conversions)
pub fn convert_common_error_pattern(error_message: &str) -> RustAIError {
    // This function provides a bridge for string-based error conversions
    // from common error patterns to structured RustAIError variants

    let msg = error_message.to_lowercase();

    if msg.contains("permission denied") || msg.contains("access denied") {
        RustAIError::Path(PathError::new("Permission denied: path=unknown"))
    } else if msg.contains("file not found") || msg.contains("not found") {
        RustAIError::Path(PathError::new("Not found: path=unknown"))
    } else if msg.contains("timeout") || msg.contains("timed out") {
        RustAIError::Timeout("Operation timed out".to_string())
    } else if msg.contains("network") || msg.contains("connection") {
        RustAIError::Network("Network error occurred".to_string())
    } else if msg.contains("parse") || msg.contains("syntax") {
        RustAIError::InternalError(format!("PARSE_ERROR: {}", error_message))
    } else if msg.contains("validation") || msg.contains("invalid") {
        RustAIError::Validation(error_message.to_string())
    } else if msg.contains("http") || msg.contains("status") {
        RustAIError::InternalError(format!("HTTP_ERROR: {}", error_message))
    } else if msg.contains("authentication") || msg.contains("auth") {
        RustAIError::Authentication(error_message.to_string())
    } else if msg.contains("config") || msg.contains("configuration") {
        RustAIError::Config(ConfigError::new(format!(
            "InvalidValue field=unknown: {}",
            error_message
        )))
    } else {
        RustAIError::InternalError(format!("CONVERTED_ERROR: {}", error_message))
    }
}

/// Convert Result<T, E> to Result<T, RustAIError>
pub trait ResultExt<T, E> {
    fn into_rustai_result(self) -> Result<T, RustAIError>
    where
        E: ToRustAIError;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn into_rustai_result(self) -> Result<T, RustAIError>
    where
        E: ToRustAIError,
    {
        self.map_err(|e| e.to_rustai_error())
    }
}

/// Create enhanced error with context
pub fn with_error_context(error: RustAIError, context: EnhancedContext) -> RustAIError {
    // For now, wrap in InternalError with context information
    // This could be extended to support structured context in the future
    let context_msg = format!(
        "Operation: {}, Resource: {}, Metadata: {:?}",
        context.operation,
        context.resource.as_deref().unwrap_or("none"),
        context.metadata
    );

    RustAIError::InternalError(format!(
        "CONTEXT_WRAPPED_ERROR: {} | Context: {}",
        error, context_msg
    ))
}

/// Convert Result with automatic error enhancement
pub async fn with_result_context<F, Fut, T, E>(operation: String, future: F) -> Result<T, RustAIError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: ToRustAIError,
{
    let context = EnhancedContext::new(operation);

    match future().await {
        Ok(value) => Ok(value),
        Err(error) => Err(with_error_context(error.to_rustai_error(), context)),
    }
}

/// HTTP status code to RustAIError mapping
pub fn http_status_to_error(status_code: u16, message: String) -> RustAIError {
    match status_code {
        400 => RustAIError::Validation(format!("Bad request: {}", message)),
        401 | 403 => RustAIError::Authentication(format!("Authentication failed: {}", message)),
        404 => RustAIError::Path(PathError::new("Not found: resource")),
        408 => RustAIError::Timeout(format!("Request timeout: {}", message)),
        429 => RustAIError::RateLimit(format!(
            "Rate limit exceeded: {} (retry after: 60s)",
            message
        )),
        500..=599 => RustAIError::ServiceUnavailable(format!("Server error ({}) {}", status_code, message)),
        _ => RustAIError::InternalError(format!(
            "HTTP_{}: HTTP error ({}) {}",
            status_code, status_code, message
        )),
    }
}

/// Convert std::io::Error with path context
pub fn io_error_with_path(error: std::io::Error, path: &str) -> RustAIError {
    match error.kind() {
        std::io::ErrorKind::NotFound => RustAIError::Path(PathError::new(format!("Not found: {}", path))),
        std::io::ErrorKind::PermissionDenied =>
            RustAIError::Path(PathError::new(format!("Permission denied: {}", path))),
        std::io::ErrorKind::AlreadyExists =>
            RustAIError::InternalError(format!("ALREADY_EXISTS: Path already exists: {}", path)),
        std::io::ErrorKind::TimedOut => RustAIError::Timeout(format!("I/O timeout on: {}", path)),
        _ => RustAIError::Io(IoError::new(format!("Read error for {}: {}", path, error))),
    }
}

/// Conversion utilities for operation results
pub mod operations {
    use super::*;

    /// Convert filesystem operation results
    pub fn fs_result<T>(result: Result<T, std::io::Error>, operation: &str, path: &str) -> Result<T, RustAIError> {
        result.map_err(|e| {
            let context = EnhancedContext {
                operation: operation.to_string(),
                resource: Some(path.to_string()),
                ..Default::default()
            };
            with_error_context(io_error_with_path(e, path), context)
        })
    }

    /// Convert network operation results
    #[cfg(feature = "web")]
    pub fn network_result<T>(result: Result<T, reqwest::Error>, operation: &str) -> Result<T, RustAIError> {
        result.map_err(|e| {
            let context = EnhancedContext {
                operation: operation.to_string(),
                metadata: [("error_type".to_string(), "network".to_string())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            with_error_context(RustAIError::Network(e.to_string()), context)
        })
    }

    /// Convert serialization operation results
    #[cfg(feature = "serde")]
    pub fn serde_result<T>(result: Result<T, serde_json::Error>, operation: &str) -> Result<T, RustAIError> {
        result.map_err(|e| {
            let context = EnhancedContext {
                operation: operation.to_string(),
                metadata: [("error_type".to_string(), "serialization".to_string())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
            with_error_context(
                RustAIError::Serialization(format!("SERIALIZATION_ERROR: {}", e)),
                context,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_generic_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let rustai_error = io_error.to_rustai_error();

        assert!(matches!(rustai_error, RustAIError::InternalError(_)));
    }

    #[test]
    fn test_pattern_based_conversion() {
        let error1 = convert_common_error_pattern("File not found error");
        assert!(matches!(error1, RustAIError::Path(_)));

        let error2 = convert_common_error_pattern("Permission denied access");
        assert!(matches!(error2, RustAIError::Path(_)));

        let error3 = convert_common_error_pattern("Network connection failed");
        assert!(matches!(error3, RustAIError::Network(_)));
    }

    #[test]
    fn test_http_status_conversion() {
        let error404 = http_status_to_error(404, "Not found".to_string());
        assert!(matches!(error404, RustAIError::Path(_)));

        let error401 = http_status_to_error(401, "Unauthorized".to_string());
        assert!(matches!(error401, RustAIError::Authentication(_)));

        let error429 = http_status_to_error(429, "Rate limited".to_string());
        assert!(matches!(error429, RustAIError::RateLimit(_)));
    }

    #[test]
    fn test_result_extension() {
        let result: Result<i32, std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        let rustai_result = result.into_rustai_result();

        assert!(matches!(rustai_result, Err(RustAIError::InternalError(_))));
    }

    #[test]
    fn test_enhanced_context() {
        let base_error = RustAIError::Validation("input invalid".to_string());
        let context = EnhancedContext {
            operation: "validate_input".to_string(),
            resource: Some("/user/input".to_string()),
            metadata: [("field".to_string(), "username".to_string())]
                .into_iter()
                .collect(),
            timestamp: chrono::Utc::now(),
            ..Default::default()
        };

        let enhanced = with_error_context(base_error, context);
        assert!(matches!(enhanced, RustAIError::InternalError(_)));

        if let RustAIError::InternalError(message) = enhanced {
            assert!(message.contains("validate_input"));
            assert!(message.contains("/user/input"));
        }
    }

    #[test]
    fn test_io_error_with_path() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let rustai_error = io_error_with_path(io_error, "/test/file.txt");

        assert!(matches!(rustai_error, RustAIError::Path(_)));
    }
}
