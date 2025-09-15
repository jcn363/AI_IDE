//! Error types for AI code generation

use std::fmt;

/// Result type alias for codegen operations
pub type Result<T> = std::result::Result<T, CodegenError>;

/// Comprehensive error enum for code generation operations
#[derive(Debug, Clone, PartialEq)]
pub enum CodegenError {
    /// AI inference service errors
    AiInferenceError(String),
    /// AI analysis service errors
    AiAnalysisError(String),
    /// Syntax validation errors
    SyntaxError(String),
    /// Security validation errors
    SecurityError(String),
    /// Quality threshold not met
    QualityThresholdNotMet { score: f64, threshold: f64 },
    /// Validation errors
    ValidationError(String),
    /// Refactoring is unsafe
    RefactoringUnsafe(String),
    /// Input/output errors
    IoError(String),
    /// Configuration errors
    ConfigError(String),
    /// Cache operation errors
    CacheError(String),
    /// Performance timeout errors
    TimeoutError(String),
    /// Resource exhaustion errors
    ResourceError(String),
    /// Template rendering errors
    TemplateError(String),
    /// Language support errors
    LanguageError(String),
    /// Context analysis errors
    ContextError(String),
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::AiInferenceError(msg) => write!(f, "AI inference error: {}", msg),
            CodegenError::AiAnalysisError(msg) => write!(f, "AI analysis error: {}", msg),
            CodegenError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            CodegenError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            CodegenError::QualityThresholdNotMet { score, threshold } => write!(
                f,
                "Quality threshold not met: score {:.2}, required {:.2}",
                score, threshold
            ),
            CodegenError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CodegenError::RefactoringUnsafe(msg) => write!(f, "Unsafe refactoring: {}", msg),
            CodegenError::IoError(msg) => write!(f, "I/O error: {}", msg),
            CodegenError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            CodegenError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            CodegenError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            CodegenError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            CodegenError::TemplateError(msg) => write!(f, "Template error: {}", msg),
            CodegenError::LanguageError(msg) => write!(f, "Language error: {}", msg),
            CodegenError::ContextError(msg) => write!(f, "Context error: {}", msg),
        }
    }
}

impl std::error::Error for CodegenError {}

/// Convert standard I/O errors to CodegenError
impl From<std::io::Error> for CodegenError {
    fn from(err: std::io::Error) -> Self {
        CodegenError::IoError(err.to_string())
    }
}

/// Convert serde JSON errors to CodegenError
impl From<serde_json::Error> for CodegenError {
    fn from(err: serde_json::Error) -> Self {
        CodegenError::ValidationError(format!("JSON serialization error: {}", err))
    }
}

/// Convert regex errors to CodegenError
impl From<regex::Error> for CodegenError {
    fn from(err: regex::Error) -> Self {
        CodegenError::ValidationError(format!("Regex error: {}", err))
    }
}

/// Convert handlebars template errors to CodegenError
impl From<handlebars::RenderError> for CodegenError {
    fn from(err: handlebars::RenderError) -> Self {
        CodegenError::TemplateError(format!("Template render error: {}", err))
    }
}

/// Convert handlebars template errors to CodegenError
impl From<handlebars::TemplateError> for CodegenError {
    fn from(err: handlebars::TemplateError) -> Self {
        CodegenError::TemplateError(format!("Template error: {}", err))
    }
}

/// Convert syn parse errors to CodegenError
impl From<syn::Error> for CodegenError {
    fn from(err: syn::Error) -> Self {
        CodegenError::SyntaxError(err.to_string())
    }
}

/// Convert chrono parse errors to CodegenError
impl From<chrono::ParseError> for CodegenError {
    fn from(err: chrono::ParseError) -> Self {
        CodegenError::ValidationError(format!("Date parsing error: {}", err))
    }
}

/// Utility trait for error aggregation
pub trait ErrorAggregator<T> {
    fn aggregate_errors<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>;
}

impl<T> ErrorAggregator<T> for Result<T> {
    fn aggregate_errors<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        self.or_else(|original_err| {
            f().map_err(|new_err| {
                CodegenError::ValidationError(format!("Multiple errors: {}; {}", original_err, new_err))
            })
        })
    }
}

/// Error reporting context
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation that failed
    pub operation: String,
    /// Input that caused the error
    pub input:     Option<String>,
    /// Timestamp of the error
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional context information
    pub context:   std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            input:     None,
            timestamp: chrono::Utc::now(),
            context:   std::collections::HashMap::new(),
        }
    }

    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.input = Some(input.into());
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operation: {}", self.operation)?;
        if let Some(input) = &self.input {
            write!(f, ", Input: {}", input)?;
        }
        write!(f, ", Time: {}", self.timestamp)?;
        if !self.context.is_empty() {
            write!(f, ", Context: {:?}", self.context)?;
        }
        Ok(())
    }
}

/// Enhanced error with context
#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error:   CodegenError,
    pub context: ErrorContext,
}

impl ContextualError {
    pub fn new(error: CodegenError, context: ErrorContext) -> Self {
        Self { error, context }
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (Context: {})", self.error, self.context)
    }
}

impl std::error::Error for ContextualError {}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum ErrorRecoveryStrategy {
    /// Retry the operation
    Retry {
        max_attempts: usize,
        backoff_ms:   u64,
    },
    /// Use a fallback implementation
    Fallback,
    /// Skip the operation
    Skip,
    /// Log and continue
    LogAndContinue,
}

/// Error handler with recovery strategies
pub struct ErrorHandler {
    strategies: std::collections::HashMap<String, ErrorRecoveryStrategy>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        let mut strategies = std::collections::HashMap::new();

        // Default strategies
        strategies.insert(
            "AiInferenceError".to_string(),
            ErrorRecoveryStrategy::Retry {
                max_attempts: 3,
                backoff_ms:   1000,
            },
        );
        strategies.insert("SyntaxError".to_string(), ErrorRecoveryStrategy::Fallback);
        strategies.insert(
            "ValidationError".to_string(),
            ErrorRecoveryStrategy::LogAndContinue,
        );

        Self { strategies }
    }

    pub fn handle_error(&self, error: &CodegenError, context: &ErrorContext) -> ErrorRecoveryStrategy {
        let error_type = match error {
            CodegenError::AiInferenceError(_) => "AiInferenceError",
            CodegenError::AiAnalysisError(_) => "AiAnalysisError",
            CodegenError::SyntaxError(_) => "SyntaxError",
            CodegenError::SecurityError(_) => "SecurityError",
            CodegenError::QualityThresholdNotMet { .. } => "QualityThresholdNotMet",
            CodegenError::ValidationError(_) => "ValidationError",
            CodegenError::RefactoringUnsafe(_) => "RefactoringUnsafe",
            CodegenError::IoError(_) => "IoError",
            CodegenError::ConfigError(_) => "ConfigError",
            CodegenError::CacheError(_) => "CacheError",
            CodegenError::TimeoutError(_) => "TimeoutError",
            CodegenError::ResourceError(_) => "ResourceError",
            CodegenError::TemplateError(_) => "TemplateError",
            CodegenError::LanguageError(_) => "LanguageError",
            CodegenError::ContextError(_) => "ContextError",
        };

        self.strategies
            .get(error_type)
            .cloned()
            .unwrap_or(ErrorRecoveryStrategy::LogAndContinue)
    }

    pub fn add_strategy(&mut self, error_type: impl Into<String>, strategy: ErrorRecoveryStrategy) {
        self.strategies.insert(error_type.into(), strategy);
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}
