//! Error handling for the debugger

use std::{fmt, io};

use thiserror::Error;

/// Main error type for the debugger
#[derive(Debug, Error)]
pub enum DebuggerError {
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Debugger process errors
    #[error("Debugger process error: {0}")]
    Process(String),

    /// Operation not implemented
    #[error("Operation not implemented")]
    NotImplemented,

    /// Debugger command errors
    #[error("Command error: {0}")]
    Command(String),

    /// Invalid state errors
    #[error("Invalid state: {0}")]
    State(String),

    /// Breakpoint related errors
    #[error("Breakpoint error: {0}")]
    Breakpoint(String),

    /// Variable evaluation errors
    #[error("Evaluation error: {0}")]
    Evaluation(String),
}

/// Result type for debugger operations
pub type Result<T> = std::result::Result<T, DebuggerError>;

impl From<String> for DebuggerError {
    fn from(s: String) -> Self {
        DebuggerError::Process(s)
    }
}

impl DebuggerError {
    /// Create a new process error
    pub fn process_error(msg: impl Into<String>) -> Self {
        DebuggerError::Process(msg.into())
    }

    /// Create a new command error
    pub fn command_error(msg: impl Into<String>) -> Self {
        DebuggerError::Command(msg.into())
    }

    /// Create a new state error
    pub fn state_error(msg: impl Into<String>) -> Self {
        DebuggerError::State(msg.into())
    }

    /// Create a new breakpoint error
    pub fn breakpoint_error(msg: impl Into<String>) -> Self {
        DebuggerError::Breakpoint(msg.into())
    }

    /// Create a new evaluation error
    pub fn eval_error(msg: impl Into<String>) -> Self {
        DebuggerError::Evaluation(msg.into())
    }
}

/// Extension trait for converting options to results with custom errors
pub trait OptionExt<T> {
    /// Convert an option to a result with a custom error message
    fn ok_or_err<F, E>(self, f: F) -> std::result::Result<T, E>
    where
        F: FnOnce() -> E;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_err<F, E>(self, f: F) -> std::result::Result<T, E>
    where
        F: FnOnce() -> E,
    {
        self.ok_or_else(f)
    }
}

/// Extension trait for converting results with string errors to DebuggerError
pub trait ResultExt<T> {
    /// Convert a result with a string error to a DebuggerResult
    fn map_debugger_err(self) -> Result<T>;
}

impl<T, E: fmt::Display> ResultExt<T> for std::result::Result<T, E> {
    fn map_debugger_err(self) -> Result<T> {
        self.map_err(|e| DebuggerError::Process(e.to_string()))
    }
}
