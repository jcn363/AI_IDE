//! Style types and structures

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;
pub use super::*;

/// Configuration for style checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleCheckConfig {
    pub max_line_length: usize,
    pub use_spaces: bool,
    pub indent_size: usize,
    pub require_comments: bool,
    pub strict_naming: bool,
}

impl Default for StyleCheckConfig {
    fn default() -> Self {
        Self {
            max_line_length: 100,
            use_spaces: true,
            indent_size: 4,
            require_comments: true,
            strict_naming: false,
        }
    }
}

/// Result of style checking
#[derive(Debug, Clone)]
pub struct StyleCheckResult {
    pub issues: Vec<StyleIssue>,
    pub consistency_score: f64,
    pub metrics: StyleMetrics,
    pub suggestions: Vec<rust_ai_ide_ai_analysis::Suggestion>,
}

/// A style-related issue
#[derive(Debug, Clone)]
pub struct StyleIssue {
    pub id: Uuid,
    pub rule: StyleRule,
    pub message: String,
    pub location: rust_ai_ide_ai_analysis::Location,
    pub severity: rust_ai_ide_ai_analysis::Severity,
    pub suggestion: Option<String>,
}

/// Style metrics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub comment_ratio: f64,
}

/// Error during style checking operations
#[derive(thiserror::Error, Debug)]
pub enum StyleCheckError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// Re-export for convenience
pub use StyleRule::*;