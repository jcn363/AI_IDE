use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IDEError {
    #[error("Path validation error: {0}")]
    PathValidation(String),

    #[error("File operation error: {0}")]
    FileOperation(String),

    #[error("AI service error: {0}")]
    AIService(String),

    #[error("Command execution error: {0}")]
    CommandExecution(String),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Cargo error: {0}")]
    Cargo(String),

    #[error("Debugging error: {0}")]
    Debugging(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Learning system error: {0}")]
    LearningSystem(String),

    #[error("Dependency management error: {0}")]
    Dependency(String),

    #[error("License error: {0}")]
    License(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type IDEResult<T> = Result<T, IDEError>;

// Validation functions are now in the shared validation module