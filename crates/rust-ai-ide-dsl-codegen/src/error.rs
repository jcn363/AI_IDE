//! DSL-specific error types and result aliases

use rust_ai_ide_common::IdeError;
use std::fmt;

/// Result type alias for DSL operations
pub type DslResult<T> = Result<T, DslError>;

/// Comprehensive error types for the DSL system
#[derive(Debug, Clone, PartialEq)]
pub enum DslError {
    /// Parsing errors (syntax, grammar violations)
    Parse {
        message: String,
        line: usize,
        column: usize,
        context: String,
    },

    /// Template validation failures
    Validation {
        template: String,
        message: String,
        field: Option<String>,
    },

    /// Template execution/runtime errors
    Execution {
        template: String,
        message: String,
        context: Option<String>,
    },

    /// Template not found or loading errors
    Template { name: String, message: String },

    /// Plugin-related errors
    Plugin { name: String, message: String },

    /// AI integration errors
    Ai {
        message: String,
        context: Option<String>,
    },

    /// Generation target/configuration errors
    Generation {
        message: String,
        context: Option<String>,
    },

    /// Generic DSL system errors
    Internal { message: String },
}

impl fmt::Display for DslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DslError::Parse {
                message,
                line,
                column,
                context,
            } => {
                write!(
                    f,
                    "Parse error at line {}, column {}: {}\nContext: {}",
                    line, column, message, context
                )
            }
            DslError::Validation {
                template,
                message,
                field,
            } => {
                let field_info = field
                    .as_ref()
                    .map(|f| format!(" (field: {})", f))
                    .unwrap_or_default();
                write!(
                    f,
                    "Validation error in template '{template}'{}: {message}",
                    field_info
                )
            }
            DslError::Execution {
                template,
                message,
                context,
            } => {
                let context_info = context
                    .as_ref()
                    .map(|c| format!("\nContext: {}", c))
                    .unwrap_or_default();
                write!(
                    f,
                    "Execution error in template '{template}': {message}{}",
                    context_info
                )
            }
            DslError::Template { name, message } => {
                write!(f, "Template error '{name}': {message}")
            }
            DslError::Plugin { name, message } => {
                write!(f, "Plugin error '{name}': {message}")
            }
            DslError::Ai { message, context } => {
                let context_info = context
                    .as_ref()
                    .map(|c| format!("\nContext: {}", c))
                    .unwrap_or_default();
                write!(f, "AI integration error: {message}{}", context_info)
            }
            DslError::Generation { message, context } => {
                let context_info = context
                    .as_ref()
                    .map(|c| format!("\nContext: {}", c))
                    .unwrap_or_default();
                write!(f, "Generation error: {message}{}", context_info)
            }
            DslError::Internal { message } => {
                write!(f, "Internal DSL error: {message}")
            }
        }
    }
}

impl std::error::Error for DslError {}

impl From<DslError> for IdeError {
    fn from(error: DslError) -> Self {
        match error {
            DslError::Parse { message, .. } => IdeError::InvalidInput { message },
            DslError::Validation { message, .. } => IdeError::InvalidInput { message },
            DslError::Execution { message, .. } => IdeError::InvalidInput { message },
            DslError::Template { message, .. } => IdeError::NotFound { message },
            DslError::Plugin { message, .. } => IdeError::Generic {
                message: format!("Plugin: {}", message),
            },
            DslError::Ai { message, .. } => IdeError::Generic {
                message: format!("AI: {}", message),
            },
            DslError::Generation { message, .. } => IdeError::Compilation { message },
            DslError::Internal { message } => IdeError::Generic {
                message: format!("DSL: {}", message),
            },
        }
    }
}

impl DslError {
    /// Create a new parse error
    pub fn parse(
        message: impl Into<String>,
        line: usize,
        column: usize,
        context: impl Into<String>,
    ) -> Self {
        DslError::Parse {
            message: message.into(),
            line,
            column,
            context: context.into(),
        }
    }

    /// Create a new validation error
    pub fn validation(template: impl Into<String>, message: impl Into<String>) -> Self {
        DslError::Validation {
            template: template.into(),
            message: message.into(),
            field: None,
        }
    }

    /// Create a new validation error with field information
    pub fn validation_with_field(
        template: impl Into<String>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        DslError::Validation {
            template: template.into(),
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Create a new execution error
    pub fn execution(template: impl Into<String>, message: impl Into<String>) -> Self {
        DslError::Execution {
            template: template.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Create a new template error
    pub fn template(name: impl Into<String>, message: impl Into<String>) -> Self {
        DslError::Template {
            name: name.into(),
            message: message.into(),
        }
    }

    /// Create a new plugin error
    pub fn plugin(name: impl Into<String>, message: impl Into<String>) -> Self {
        DslError::Plugin {
            name: name.into(),
            message: message.into(),
        }
    }
}
