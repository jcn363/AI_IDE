//! Core types for the analysis system

use serde::{Deserialize, Serialize};
use std::fmt;

/// Location information for code elements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeLocation {
    /// File path
    pub file_path: String,
    /// Range in the file
    pub range: Range,
    /// Optional module path
    pub module_path: Option<String>,
}

/// Range information for code locations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    /// Starting line (1-based)
    pub start_line: u32,
    /// Starting column (1-based)
    pub start_col: u32,
    /// Ending line (1-based)
    pub end_line: u32,
    /// Ending column (1-based)
    pub end_col: u32,
}

/// Severity levels for analysis findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Critical issue that will likely cause failures
    Error,
    /// Potential issue that should be addressed
    Warning,
    /// Informational finding
    Info,
    /// Suggestion for improvement
    Hint,
}

/// Categories of analysis findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AnalysisCategory {
    /// Code style and formatting issues
    Style,
    /// Potential bugs or error-prone code
    Correctness,
    /// Security vulnerabilities
    Security,
    /// Performance issues
    Performance,
    /// Code complexity and maintainability
    Complexity,
    /// Architectural issues
    Architecture,
    /// Code smell detection
    CodeSmell,
    /// Documentation issues
    Documentation,
    /// Other categories of issues
    Other,
}

impl fmt::Display for AnalysisCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisCategory::Style => write!(f, "style"),
            AnalysisCategory::Correctness => write!(f, "correctness"),
            AnalysisCategory::Security => write!(f, "security"),
            AnalysisCategory::Performance => write!(f, "performance"),
            AnalysisCategory::Complexity => write!(f, "complexity"),
            AnalysisCategory::Architecture => write!(f, "architecture"),
            AnalysisCategory::Documentation => write!(f, "documentation"),
            AnalysisCategory::Other => write!(f, "other"),
        }
    }
}

/// Main analysis result type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFinding {
    /// The type of the finding
    pub kind: String,
    /// Description of the finding
    pub message: String,
    /// The actual finding data as a JSON string
    pub data: String,
    /// Severity level
    pub severity: Severity,
    /// Category of the finding
    pub category: AnalysisCategory,
    /// Location in the source code
    pub location: String,
    /// Code range where the finding was detected
    pub range: Range,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Unique identifier for the rule that generated this finding
    pub rule_id: String,
}

/// Alternative analysis finding type (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFindingOld {
    /// Description of the finding
    pub message: String,
    /// Severity level
    pub severity: Severity,
    /// Category of the finding
    pub category: AnalysisCategory,
    /// Location in the code
    pub range: Range,
    /// Suggested fix (if any)
    pub suggestion: Option<String>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Unique identifier for the rule that triggered this finding
    pub rule_id: String,
}

/// Analysis preferences that control analyzer behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPreferences {
    /// Enable architectural analysis
    pub enable_architecture: bool,
    /// Enable security analysis
    pub enable_security: bool,
    /// Enable performance analysis
    pub enable_performance: bool,
    /// Enable metrics calculation
    pub enable_code_style: bool,
    /// Enable code smell analysis
    pub enable_code_smells: bool,
    /// Whether to use incremental analysis
    pub incremental_analysis: bool,
    /// Minimum confidence threshold for findings (0.0 to 1.0)
    pub min_confidence: f32,
    /// Whether to include suggestions in findings
    pub include_suggestions: bool,
}

impl Default for AnalysisPreferences {
    fn default() -> Self {
        Self {
            enable_architecture: true,
            enable_security: true,
            enable_performance: true,
            enable_code_style: true,
            enable_code_smells: true,
            incremental_analysis: true,
            min_confidence: 0.7,
            include_suggestions: true,
        }
    }
}

/// Configuration for analysis modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Whether caching is enabled
    #[serde(default = "default_true")]
    pub cache_enabled: bool,
    /// Time-to-live for cached results in seconds (None = no expiration)
    #[serde(default)]
    pub cache_ttl_seconds: Option<u64>,
    /// Whether to enable incremental analysis
    #[serde(default = "default_true")]
    pub incremental_analysis: bool,
    /// Maximum number of files to analyze in parallel
    #[serde(default = "default_max_parallel_files")]
    pub max_parallel_files: usize,
    /// File patterns to include in analysis
    #[serde(default = "default_include_patterns")]
    pub include_patterns: Vec<String>,
    /// Version of the analysis configuration (bump this to invalidate all caches)
    #[serde(default = "default_version")]
    pub version: u32,
    /// Minimum confidence threshold for findings (0.0 - 1.0)
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
    /// Custom analysis rules
    #[serde(default)]
    pub custom_rules: std::collections::HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}
fn default_max_parallel_files() -> usize {
    num_cpus::get().max(1)
}
fn default_include_patterns() -> Vec<String> {
    vec![r"\.rs$".to_string()]
}
fn default_version() -> u32 {
    1
}
fn default_min_confidence() -> f32 {
    0.7
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_ttl_seconds: Some(3600 * 24 * 7), // 1 week
            version: 1,
            min_confidence: 0.7,
            incremental_analysis: true,
            max_parallel_files: num_cpus::get(),
            include_patterns: vec![r"\.rs$".to_string()],
            custom_rules: std::collections::HashMap::new(),
        }
    }
}

/// Trait for all analysis findings
pub trait Finding {
    /// Get the unique identifier for the finding
    fn id(&self) -> &str;

    /// Get the human-readable message
    fn message(&self) -> &str;

    /// Get the severity level
    fn severity(&self) -> Severity;

    /// Get the location in the source code
    fn location(&self) -> &CodeLocation;

    /// Get an optional suggestion for fixing the issue
    fn suggestion(&self) -> Option<&str>;

    /// Get the confidence level (0.0 to 1.0)
    fn confidence(&self) -> f32;

    /// Get the rule ID that triggered this finding
    fn rule_id(&self) -> &str;
}

impl Finding for AnalysisFinding {
    fn id(&self) -> &str {
        &self.rule_id
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn location(&self) -> &CodeLocation {
        // For backward compatibility, create a CodeLocation from the AnalysisFinding fields
        // In a real implementation, this would be stored properly
        unimplemented!("AnalysisFinding does not have a proper CodeLocation field")
    }

    fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    fn confidence(&self) -> f32 {
        self.confidence
    }

    fn rule_id(&self) -> &str {
        &self.rule_id
    }
}
