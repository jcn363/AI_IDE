//! Unified error handling for shared types functionality
//!
//! This module provides comprehensive error types for type analysis,
//! TypeScript generation, plugin operations, and cross-platform validation.

use std::fmt;

/// Main error type for type generation operations
#[derive(Debug, thiserror::Error)]
pub enum TypeGenerationError {
    /// Errors during AST parsing and type analysis
    #[error("Type analysis failed: {0}")]
    AnalysisError(String),

    /// Errors during TypeScript code generation
    #[error("TypeScript generation failed: {0}")]
    TypeScriptGenerationError(String),

    /// File I/O errors
    #[error("File operation failed: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Template rendering errors
    #[cfg(feature = "templates")]
    #[error("Template rendering failed: {0}")]
    TemplateError(String),

    /// Plugin-related errors
    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Type bridging and validation errors
    #[error("Type bridge error: {0}")]
    BridgeError(#[from] TypeBridgeError),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Caching errors
    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Error type for cross-platform type bridging operations
#[derive(Debug, thiserror::Error)]
pub enum TypeBridgeError {
    /// Platform compatibility issues
    #[error("Platform compatibility error: {0}")]
    CompatibilityError(String),

    /// Type mapping errors between platforms
    #[error("Type mapping error: {0}")]
    MappingError(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Version compatibility issues
    #[error("Version compatibility error: {0}")]
    VersionError(String),

    /// Translation errors
    #[error("Translation error: {0}")]
    TranslationError(String),
}

/// Error type for plugin system operations
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Plugin loading errors
    #[error("Plugin loading failed: {0}")]
    LoadError(String),

    /// Plugin execution errors
    #[error("Plugin execution failed: {0}")]
    ExecutionError(String),

    /// Plugin configuration errors
    #[error("Plugin configuration error: {0}")]
    ConfigError(String),

    /// Plugin compatibility errors
    #[error("Plugin compatibility error: {0}")]
    CompatibilityError(String),

    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Dynamic library errors
    #[cfg(feature = "plugins")]
    #[error("Dynamic library error: {0}")]
    LibError(#[from] libloading::Error),
}

/// Contextual error information for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The operation that failed
    pub operation: String,

    /// Source file or module where error occurred
    pub source: Option<String>,

    /// Line number in source file
    pub line: Option<usize>,

    /// Column number in source file
    pub column: Option<usize>,

    /// Additional context information
    pub context: serde_json::Value,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            operation: "unknown".to_string(),
            source:    None,
            line:      None,
            column:    None,
            context:   serde_json::Value::Null,
        }
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operation: {}", self.operation)?;
        if let Some(ref source) = self.source {
            write!(f, ", Source: {}", source)?;
        }
        if let Some(line) = self.line {
            write!(f, ", Line: {}", line)?;
            if let Some(column) = self.column {
                write!(f, ", Column: {}", column)?;
            }
        }
        Ok(())
    }
}

/// Enhanced result type for type generation operations
pub type TypeResult<T> = Result<T, TypeGenerationError>;

/// Enhanced result type for bridging operations
pub type BridgeResult<T> = Result<T, TypeBridgeError>;

/// Enhanced result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn test_error_creation() {
        let analysis_error = TypeGenerationError::AnalysisError("Test error".to_string());
        assert_eq!(
            analysis_error.to_string(),
            "Type analysis failed: Test error"
        );

        let io_error = TypeGenerationError::from(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        assert!(io_error.to_string().contains("File operation failed"));
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext {
            operation: "type_analysis".to_string(),
            source:    Some("types.rs".to_string()),
            line:      Some(42),
            column:    Some(10),
            context:   serde_json::json!({"additional": "info"}),
        };

        assert_eq!(
            context.to_string(),
            "Operation: type_analysis, Source: types.rs, Line: 42, Column: 10"
        );
    }
}
