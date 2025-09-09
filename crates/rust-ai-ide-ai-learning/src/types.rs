//! Type definitions and aliases for the learning system
//!
//! This module provides type aliases and error types specific to the learning system,
//! making it self-contained and independent of the main crate's type definitions.

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::PathBuf;

/// Result type for learning system operations
pub type LearningResult<T> = Result<T, LearningError>;

/// Custom error type for the learning system
#[derive(Debug, thiserror::Error)]
pub enum LearningError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid preferences: {0}")]
    InvalidPreferencesError(String),

    #[error("Pattern not found: {0}")]
    PatternNotFoundError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Privacy mode violation: {0}")]
    PrivacyModeError(String),
}

/// Privacy modes for the learning system
#[derive(
    Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub enum PrivacyMode {
    /// Opt-out privacy mode - data collection enabled by default
    OptOut,
    /// Anonymous privacy mode - data is anonymized
    Anonymous,
    /// Opt-in privacy mode - explicit consent required
    OptIn,
}

impl Display for PrivacyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivacyMode::OptOut => write!(f, "Opt-out"),
            PrivacyMode::Anonymous => write!(f, "Anonymous"),
            PrivacyMode::OptIn => write!(f, "Opt-in"),
        }
    }
}

/// Analysis preferences placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPreferences {
    pub privacy_mode: PrivacyMode,
}

impl Default for AnalysisPreferences {
    fn default() -> Self {
        Self {
            privacy_mode: PrivacyMode::OptIn,
        }
    }
}

/// Service error placeholder for integration
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Generic service error: {0}")]
    Generic(String),

    #[error("Database error: {0}")]
    Database(String),
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Privacy violation: {0}")]
    Privacy(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Alias for AIServiceError compatibility
pub type AIServiceError = ServiceError;

/// Alias for AIResult compatibility
pub type AIResult<T> = Result<T, AIServiceError>;

impl From<LearningError> for ServiceError {
    fn from(err: LearningError) -> Self {
        match err {
            LearningError::DatabaseError(e) => ServiceError::Database(e),
            LearningError::SerializationError(e) => ServiceError::SerializationError(e),
            _ => ServiceError::Generic(err.to_string()),
        }
    }
}

impl From<rust_ai_ide_errors::RustAIError> for ServiceError {
    fn from(err: rust_ai_ide_errors::RustAIError) -> Self {
        match err {
            rust_ai_ide_errors::RustAIError::Io(_) => ServiceError::Generic(err.to_string()),
            rust_ai_ide_errors::RustAIError::FileSystem(e) => {
                ServiceError::Generic(format!("Filesystem error: {}", e))
            }
            rust_ai_ide_errors::RustAIError::InternalError(e) => {
                ServiceError::Generic(format!("Internal error: {}", e))
            }
            _ => ServiceError::Generic(err.to_string()),
        }
    }
}

/// AI provider placeholder
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AIProvider {
    Mock,
    OpenAI,
    Anthropic,
}

/// Context placeholder for error resolution integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContext {
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub surrounding_code: Option<String>,
}

impl Default for AIContext {
    fn default() -> Self {
        Self {
            file_path: None,
            line_number: None,
            column_number: None,
            surrounding_code: None,
        }
    }
}

/// Custom Hash trait for helping with hash generation
pub trait HasherHelper {
    fn hash_to_string(&self, input: &str) -> String;
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Implementation of hash helper
pub struct StdHasher;

impl HasherHelper for StdHasher {
    fn hash_to_string(&self, input: &str) -> String {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Extension methods for string hashing
pub trait StringHashExt {
    fn hash_to_string(&self) -> String;
}

impl StringHashExt for str {
    fn hash_to_string(&self) -> String {
        StdHasher.hash_to_string(self)
    }
}

/// UUID generation helper
pub struct UuidHelper;

impl UuidHelper {
    pub fn generate() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// File system helpers
pub struct FsHelper;

impl FsHelper {
    pub fn get_data_dir() -> std::io::Result<PathBuf> {
        // Fallback implementation since dirs is available
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("rust-ai-ide");
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }
}

/// Database helper types
pub mod db_types {
    use super::LearningResult;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    /// Error pattern type for learning
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ErrorPattern {
        pub message_pattern: String,
        pub error_code: Option<String>,
        pub context_patterns: Vec<String>,
    }

    impl Default for ErrorPattern {
        fn default() -> Self {
            Self {
                message_pattern: String::new(),
                error_code: None,
                context_patterns: vec![],
            }
        }
    }

    /// Change types for text modifications
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub enum ChangeType {
        Insert,
        Delete,
        Replace,
        Move,
    }

    /// Text change representation
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TextChange {
        pub original_text: String,
        pub new_text: String,
        pub change_type: ChangeType,
    }

    /// Fix suggestion structure
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FixSuggestion {
        pub title: String,
        pub description: String,
        pub changes: Vec<TextChange>,
        pub confidence: f32,
        pub warnings: Vec<String>,
    }

    impl Default for FixSuggestion {
        fn default() -> Self {
            Self {
                title: String::new(),
                description: String::new(),
                changes: vec![],
                confidence: 0.0,
                warnings: vec![],
            }
        }
    }

    /// Generic row trait for database row operations
    pub trait DBRow {
        fn get_string(&self, column: &str) -> String;
        fn get_i32(&self, column: &str) -> i32;
        fn get_i64(&self, column: &str) -> i64;
        fn get_f32(&self, column: &str) -> f32;
        fn get_bool(&self, column: &str) -> bool;
    }

    /// Extension methods for extracting DateTime values
    pub trait DateTimeExt {
        fn to_rfc3339(&self) -> String;
    }

    impl DateTimeExt for DateTime<Utc> {
        fn to_rfc3339(&self) -> String {
            self.to_rfc3339()
        }
    }

    /// Parse RFC3339 datetime string to DateTime<Utc>
    pub fn parse_datetime_rfc3339(s: &str) -> LearningResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                super::LearningError::DatabaseError(format!("DateTime parsing error: {}", e))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_to_string() {
        let hash1 = "test".hash_to_string();
        let hash2 = "test".hash_to_string();
        assert_eq!(hash1, hash2); // Same input should produce same hash
    }

    #[test]
    fn test_learning_error_conversions() {
        let db_err = LearningError::DatabaseError("test".to_string());
        let service_err: ServiceError = db_err.into();
        assert!(matches!(service_err, ServiceError::Database(_)));
    }
}
