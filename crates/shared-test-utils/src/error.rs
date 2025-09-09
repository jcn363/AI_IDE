use thiserror::Error;

/// Comprehensive error types for testing utilities
#[derive(Debug, Error, Clone)]
pub enum TestError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Filesystem error: {0}")]
    Filesystem(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("JSON serialization error: {0}")]
    Json(String),

    #[error("YAML serialization error: {0}")]
    Yaml(String),

    #[error("TOML serialization error: {0}")]
    Toml(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Async error: {0}")]
    Async(String),

    #[error("Concurrency error: {0}")]
    Concurrency(String),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("TAURI error: {0}")]
    Tauri(String),

    #[error("Cleanup error: {0}")]
    Cleanup(String),

    #[error("UTF8 string conversion error: {0}")]
    Utf8String(String),

    #[error("UTF8 slice conversion error: {0}")]
    Utf8Slice(String),
}

// Manual From implementations for errors that don't implement Clone
impl From<std::io::Error> for TestError {
    fn from(err: std::io::Error) -> Self {
        TestError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for TestError {
    fn from(err: serde_json::Error) -> Self {
        TestError::Json(err.to_string())
    }
}

impl From<serde_yaml::Error> for TestError {
    fn from(err: serde_yaml::Error) -> Self {
        TestError::Yaml(err.to_string())
    }
}

impl From<toml::de::Error> for TestError {
    fn from(err: toml::de::Error) -> Self {
        TestError::Toml(err.to_string())
    }
}

#[cfg(feature = "database")]
impl From<rusqlite::Error> for TestError {
    fn from(err: rusqlite::Error) -> Self {
        TestError::Async(format!("SQLite error: {}", err.to_string()))
    }
}

/// Specific validation errors for tests
#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    #[error("Path validation failed: {message}")]
    PathValidation { message: String },

    #[error("Security validation failed: {message}")]
    SecurityValidation { message: String },

    #[error("Content validation failed: {message}")]
    ContentValidation { message: String },

    #[error("Invalid test setup: {message}")]
    InvalidTestSetup { message: String },
}

/// Trait for easy test result conversion
pub trait TestResult<T> {
    fn expect_test(self, msg: &str) -> T;
}

impl<T> TestResult<T> for Result<T, TestError> {
    fn expect_test(self, msg: &str) -> T {
        self.unwrap_or_else(|e| panic!("Test error in {}: {}", msg, e))
    }
}

impl From<std::string::FromUtf8Error> for TestError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        TestError::Utf8String(err.to_string())
    }
}

impl From<std::str::Utf8Error> for TestError {
    fn from(err: std::str::Utf8Error) -> Self {
        TestError::Utf8Slice(err.to_string())
    }
}

impl ValidationError {
    pub fn path_validation(msg: impl Into<String>) -> Self {
        ValidationError::PathValidation { message: msg.into() }
    }

    pub fn security_validation(msg: impl Into<String>) -> Self {
        ValidationError::SecurityValidation { message: msg.into() }
    }

    pub fn content_validation(msg: impl Into<String>) -> Self {
        ValidationError::ContentValidation { message: msg.into() }
    }

    pub fn invalid_setup(msg: impl Into<String>) -> Self {
        ValidationError::InvalidTestSetup { message: msg.into() }
    }
}