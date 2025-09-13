// Re-export from the unified error crate for backward compatibility
pub use rust_ai_ide_errors::{IDEResult, RustAIError};

// Alias for backward compatibility
pub type IDEError = RustAIError;

// Add std::fmt import for Display trait
use std::fmt;
// TODO: Consider removing this deprecated code in a future version

// Note: Cannot implement methods for external IDEError type due to orphan rules

/// AI-specific error types
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    /// Error occurred while loading an AI model
    #[error("Model loading error: {model} - {message}")]
    ModelLoadingError {
        /// Name of the model that failed to load
        model:   String,
        /// Detailed error message explaining the failure
        message: String,
    },

    /// Error occurred during model inference execution
    #[error("Model inference error: {operation} failed")]
    InferenceError {
        /// Description of the inference operation that failed
        operation: String,
    },

    /// Error in AI provider configuration
    #[error("Provider configuration error: {provider} - {message}")]
    ProviderConfigError {
        /// Name of the AI provider with configuration issues
        provider: String,
        /// Detailed error message about the configuration problem
        message:  String,
    },

    /// Error during context preparation for AI operations
    #[error("Context preparation error: {message}")]
    ContextError {
        /// Description of the context preparation error
        message: String,
    },

    /// Insufficient memory for AI operation
    #[error("Memory allocation error: requested {requested_mb}MB")]
    MemoryError {
        /// Amount of memory requested in MB
        requested_mb: u64,
    },

    /// AI provider rate limit has been exceeded
    #[error("Rate limit exceeded: {limit} per {time_window}")]
    RateLimitError {
        /// Maximum number of requests allowed
        limit:       u32,
        /// Time window for the rate limit
        time_window: String,
    },

    /// Specified AI model was not found
    #[error("Model not found: {model}")]
    ModelNotFound {
        /// Name of the model that was not found
        model: String,
    },

    /// AI inference operation timed out
    #[error("Inference timeout: {operation} took longer than {timeout_seconds}s")]
    InferenceTimeout {
        /// Description of the operation that timed out
        operation:       String,
        /// Timeout duration in seconds
        timeout_seconds: u32,
    },

    /// AI provider API returned an error response
    #[error("Provider API error: {provider} returned {status_code}")]
    ProviderAPIError {
        /// Name of the AI provider
        provider:    String,
        /// HTTP status code returned by the API
        status_code: u16,
    },

    /// Input exceeded the maximum token limit
    #[error("Token limit exceeded: {max_tokens} tokens")]
    TokenLimitError {
        /// Maximum number of tokens allowed
        max_tokens: u32,
    },
}

/// Analysis-specific error types
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Target file not found: {path}")]
    FileNotFound { path: String },

    #[error("Target directory not found: {path}")]
    DirectoryNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("File too large: {path} is {size_mb}MB (max: {max_mb}MB)")]
    FileTooLarge {
        path:    String,
        size_mb: f64,
        max_mb:  f64,
    },

    #[error("Unsupported language: {language}")]
    UnsupportedLanguage { language: String },

    #[error("Parser error: {parser} failed at line {line}")]
    ParserError {
        parser:  String,
        line:    u32,
        message: String,
    },

    #[error("Analysis timeout: {analyzer} took longer than {timeout_seconds}s")]
    AnalysisTimeout {
        analyzer:        String,
        timeout_seconds: u32,
    },

    #[error("Analysis configuration error: {message}")]
    ConfigError { message: String },

    #[error("Cyclic dependency detected: {cycle}")]
    CyclicDependency { cycle: String },

    #[error("Analysis cancelled")]
    Cancelled,

    #[error("Memory limit exceeded: {analyzer} used {used_mb}MB")]
    MemoryLimitExceeded { analyzer: String, used_mb: u64 },
}

/// Unified result type for AI operations
pub type AIResult<T> = Result<T, AIError>;

/// Unified result type for analysis operations
pub type AnalysisResult<T> = Result<T, AnalysisError>;

/// Trace information for debugging
#[derive(Debug, Clone)]
pub struct ErrorTrace {
    /// Name of the operation that triggered the error
    pub operation: String,
    /// Timestamp when the error occurred
    pub timestamp: std::time::SystemTime,
    /// Component or module where the error occurred
    pub component: String,
    /// Additional metadata associated with the error
    pub metadata:  std::collections::HashMap<String, String>,
}

impl Default for ErrorTrace {
    fn default() -> Self {
        Self {
            operation: "unknown".to_string(),
            timestamp: std::time::SystemTime::now(),
            component: "unknown".to_string(),
            metadata:  std::collections::HashMap::new(),
        }
    }
}

impl ErrorTrace {
    /// Creates a new ErrorTrace with the specified operation and component
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            component: component.into(),
            ..Default::default()
        }
    }

    /// Adds a metadata key-value pair to the error trace
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Enhanced error with trace information
#[derive(Debug, thiserror::Error)]
pub struct TracedError<E> {
    #[source]
    pub inner:                E,
    pub trace:                ErrorTrace,
    pub recovery_suggestions: Vec<String>,
}

impl<E> TracedError<E> {
    pub fn new(inner: E, trace: ErrorTrace) -> Self {
        Self {
            inner,
            trace,
            recovery_suggestions: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.recovery_suggestions.push(suggestion.into());
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.recovery_suggestions = suggestions;
        self
    }
}

impl<E: fmt::Display> fmt::Display for TracedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (at {}:{})",
            self.inner, self.trace.component, self.trace.operation
        )?;
        if !self.recovery_suggestions.is_empty() {
            write!(f, "\nSuggestions:")?;
            for suggestion in &self.recovery_suggestions {
                write!(f, "\n  - {}", suggestion)?;
            }
        }
        Ok(())
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,      // Minor issues, warnings
    Medium,   // Functional issues but can continue
    High,     // Serious issues that affect core functionality
    Critical, // System-level issues requiring immediate attention
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "low"),
            ErrorSeverity::Medium => write!(f, "medium"),
            ErrorSeverity::High => write!(f, "high"),
            ErrorSeverity::Critical => write!(f, "critical"),
        }
    }
}

// Note: From implementations for external types to external IDEError cannot be added here due to
// orphan rules Consider using conversion functions instead if needed

/// Error handling utilities
pub mod utils {
    use super::*;

    /// Create a traced error with recovery suggestions
    pub fn trace_error<E>(error: E, operation: &str, component: &str) -> TracedError<E> {
        let trace = ErrorTrace {
            operation: operation.to_string(),
            component: component.to_string(),
            timestamp: std::time::SystemTime::now(),
            metadata:  std::collections::HashMap::new(),
        };

        TracedError {
            inner: error,
            trace,
            recovery_suggestions: Vec::new(),
        }
    }

    /// Convert error to IDEError with context
    pub fn contextualize_error<E: std::error::Error>(
        error: E,
        operation: impl Into<String>,
        component: impl Into<String>,
    ) -> IDEError {
        let op = operation.into();
        let comp = component.into();
        let error_msg = error.to_string();

        // Try to categorize the error based on error message content
        if error_msg.contains("file") || error_msg.contains("path") {
            IDEError::Generic(format!("{} failed in {}: {}", op, comp, error))
        } else if error_msg.contains("parse") || error_msg.contains("[") {
            IDEError::Generic(format!("{} failed in {}: {}", op, comp, error))
        } else {
            IDEError::Generic(format!("{} failed in {}: {}", op, comp, error))
        }
    }

    /// Retry a fallible operation with exponential backoff
    pub async fn retry_with_backoff<T, F, Fut, E>(
        mut operation: F,
        max_attempts: u32,
        base_delay_ms: u64,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let mut attempt = 0;

        loop {
            attempt += 1;
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt >= max_attempts {
                        return Err(error);
                    }

                    let delay_ms = base_delay_ms * (2u64.pow(attempt - 1));
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }
}
