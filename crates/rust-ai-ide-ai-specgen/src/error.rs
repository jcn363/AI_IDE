//! Error types for the specification generation crate

use std::fmt;

/// Main error type for specification generation operations
#[derive(Debug, thiserror::Error)]
pub enum SpecGenError {
    /// Errors during parsing of specifications
    #[error("Parse error: {message}")]
    ParseError { message: String },

    /// Errors during code generation
    #[error("Generation error: {message}")]
    GenerateError { message: String },

    /// Errors during validation
    #[error("Validation error: {message}")]
    ValidationError { message: String },

    /// Errors during template processing
    #[error("Template error: {message}")]
    TemplateError { message: String },

    /// Errors during documentation generation
    #[error("Documentation error: {message}")]
    DocumentationError { message: String },

    /// I/O related errors
    #[error("I/O error: {message}")]
    IoError { message: String },

    /// JSON serialization/deserialization errors
    #[error("JSON error: {message}")]
    JsonError { message: String },

    /// Handlebars template rendering errors
    #[error("Template rendering error: {message}")]
    TemplateRenderError { message: String },

    /// Configuration validation errors
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// External dependency errors (e.g., filesystem issues)
    #[error("External error: {message}")]
    ExternalError { message: String },

    /// Validation issues collected during processing
    #[error("Multiple validation issues found: {issues:?}")]
    ValidationIssues { issues: Vec<ValidationIssue> },
}

impl Clone for SpecGenError {
    fn clone(&self) -> Self {
        match self {
            SpecGenError::ParseError { message } => SpecGenError::ParseError { message: message.clone() },
            SpecGenError::GenerateError { message } => SpecGenError::GenerateError { message: message.clone() },
            SpecGenError::ValidationError { message } => SpecGenError::ValidationError { message: message.clone() },
            SpecGenError::TemplateError { message } => SpecGenError::TemplateError { message: message.clone() },
            SpecGenError::DocumentationError { message } => SpecGenError::DocumentationError { message: message.clone() },
            SpecGenError::IoError { message } => SpecGenError::IoError { message: message.clone() },
            SpecGenError::JsonError { message } => SpecGenError::JsonError { message: message.clone() },
            SpecGenError::TemplateRenderError { message } => SpecGenError::TemplateRenderError { message: message.clone() },
            SpecGenError::ConfigError { message } => SpecGenError::ConfigError { message: message.clone() },
            SpecGenError::ExternalError { message } => SpecGenError::ExternalError { message: message.clone() },
            SpecGenError::ValidationIssues { issues } => SpecGenError::ValidationIssues { issues: issues.clone() },
        }
    }
}

impl From<std::io::Error> for SpecGenError {
    fn from(err: std::io::Error) -> Self {
        SpecGenError::IoError { message: err.to_string() }
    }
}

impl From<serde_json::Error> for SpecGenError {
    fn from(err: serde_json::Error) -> Self {
        SpecGenError::JsonError { message: err.to_string() }
    }
}

/// Convenient result type alias
pub type Result<T> = std::result::Result<T, SpecGenError>;

/// Represents a single validation issue
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationIssue {
    /// The severity of the issue
    pub severity: ValidationSeverity,
    /// Human-readable message describing the issue
    pub message: String,
    /// Optional location information (file:line:column:)
    pub location: Option<String>,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
    /// Category of the validation issue
    pub category: ValidationCategory,
}

/// Severity levels for validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
}

// Alias for backward compatibility
pub type ValidationSeverity = Severity;

/// Categories of validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ValidationCategory {
    /// Issues with specification parsing
    Parse,
    /// Issues with code generation
    Generation,
    /// Issues with template processing
    Template,
    /// Issues with documentation
    Documentation,
    /// Security-related issues
    Security,
    /// Performance-related issues
    Performance,
    /// Type safety issues
    TypeSafety,
    /// Best practices violations
    BestPractice,
    /// General/miscellaneous validation issues
    General,
}

impl ValidationIssue {
    pub fn new(severity: ValidationSeverity, message: String) -> Self {
        Self {
            severity,
            message,
            location: None,
            suggestion: None,
            category: ValidationCategory::General,
        }
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn with_category(mut self, category: ValidationCategory) -> Self {
        self.category = category;
        self
    }
}

impl ValidationCategory {
    /// Get default severity for a validation category
    pub fn default_severity(self) -> ValidationSeverity {
        match self {
            ValidationCategory::Security => ValidationSeverity::Error,
            ValidationCategory::TypeSafety => ValidationSeverity::Error,
            ValidationCategory::Parse => ValidationSeverity::Error,
            ValidationCategory::Performance => ValidationSeverity::Warning,
            ValidationCategory::BestPractice => ValidationSeverity::Warning,
            ValidationCategory::Generation => ValidationSeverity::Error,
            ValidationCategory::Template => ValidationSeverity::Error,
            ValidationCategory::Documentation => ValidationSeverity::Info,
            ValidationCategory::General => ValidationSeverity::Info,
        }
    }
}

impl fmt::Display for ValidationCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationCategory::Parse => write!(f, "Parse"),
            ValidationCategory::Generation => write!(f, "Generation"),
            ValidationCategory::Template => write!(f, "Template"),
            ValidationCategory::Documentation => write!(f, "Documentation"),
            ValidationCategory::Security => write!(f, "Security"),
            ValidationCategory::Performance => write!(f, "Performance"),
            ValidationCategory::TypeSafety => write!(f, "TypeSafety"),
            ValidationCategory::BestPractice => write!(f, "BestPractice"),
            ValidationCategory::General => write!(f, "General"),
        }
    }
}