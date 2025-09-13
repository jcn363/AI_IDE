//! Core types and data structures for the monitoring framework

use crate::{errors::Result, metrics::QualityMetrics};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main analysis report containing all monitoring results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Unique identifier for this analysis run
    pub id: String,

    /// Timestamp when analysis was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Overall quality score (0-100)
    pub quality_score: f64,

    /// Quality metrics breakdown
    pub metrics: QualityMetrics,

    /// Individual analysis results
    pub results: Vec<AnalysisResult>,

    /// System information
    pub system_info: SystemInfo,

    /// Analysis configuration used
    pub config_summary: ConfigSummary,

    /// Analysis duration in seconds
    pub duration_seconds: f64,
}

/// Individual analysis result from a specific analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Name of the analyzer that produced this result
    pub analyzer: String,

    /// Whether analysis completed successfully
    pub success: bool,

    /// Severity level of issues found
    pub severity: Severity,

    /// Category of analysis
    pub category: Category,

    /// Number of issues found
    pub issue_count: usize,

    /// Detailed findings
    pub findings: Vec<Finding>,

    /// Performance metrics for this analysis
    pub performance: Option<AnalysisPerformance>,

    /// Error message if analysis failed
    pub error: Option<String>,
}

/// Individual finding or issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// File path where issue was found
    pub file: PathBuf,

    /// Line number (1-based)
    pub line: Option<usize>,

    /// Column number (1-based)
    pub column: Option<usize>,

    /// Type of issue found
    pub issue_type: String,

    /// Severity level
    pub severity: Severity,

    /// Descriptive message
    pub message: String,

    /// Code snippet for context
    pub code: Option<String>,

    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Analysis performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPerformance {
    /// Duration in seconds
    pub duration_seconds: f64,

    /// Memory usage in MB
    pub memory_mb: Option<f64>,

    /// CPU usage percentage
    pub cpu_percent: Option<f64>,

    /// Peak memory usage
    pub peak_memory_mb: Option<f64>,
}

/// System information collected during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,

    /// Architecture
    pub arch: String,

    /// Rust version
    pub rust_version: String,

    /// Cargo version
    pub cargo_version: String,

    /// CPU core count
    pub cpu_count: usize,

    /// Total memory in MB
    pub total_memory_mb: usize,

    /// Available memory in MB
    pub available_memory_mb: usize,
}

/// Configuration summary used for this analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    /// Workspace root directory
    pub workspace_root: PathBuf,

    /// Enabled analyzers
    pub enabled_analyzers: Vec<String>,

    /// Quality score thresholds
    pub thresholds: HashMap<String, f64>,

    /// Target platforms for cross-compilation
    pub target_platforms: Vec<String>,
}

/// Severity levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// No issues
    None,
    /// Informational findings
    Info,
    /// Low priority issues
    Low,
    /// Medium priority issues
    Medium,
    /// High priority issues
    High,
    /// Critical issues that prevent successful analysis
    Critical,
}

/// Category of analysis or finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// Static analysis (compiler warnings, clippy)
    StaticAnalysis,
    /// Performance analysis (compilation time, memory usage)
    Performance,
    /// Security analysis (vulnerabilities, unsafe code)
    Security,
    /// Code quality (unused variables, code style)
    CodeQuality,
    /// Dependency analysis (crates, versions, conflicts)
    Dependencies,
    /// Cross-platform compatibility
    CrossPlatform,
    /// General system health
    System,
    /// Custom category
    Custom,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Info => write!(f, "info"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaticAnalysis => write!(f, "static_analysis"),
            Self::Performance => write!(f, "performance"),
            Self::Security => write!(f, "security"),
            Self::CodeQuality => write!(f, "code_quality"),
            Self::Dependencies => write!(f, "dependencies"),
            Self::CrossPlatform => write!(f, "cross_platform"),
            Self::System => write!(f, "system"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl Default for Severity {
    fn default() -> Self {
        Self::None
    }
}

impl Default for Category {
    fn default() -> Self {
        Self::Custom
    }
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Current value
    pub current: f64,

    /// Previous value
    pub previous: Option<f64>,

    /// Change from previous (percentage)
    pub change_percent: Option<f64>,

    /// Trend direction
    pub direction: TrendDirection,

    /// Is this an improvement?
    pub improvement: bool,

    /// Statistical significance of change
    pub significant: bool,
}

/// Direction of trend change
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendDirection {
    /// Value is increasing
    Increasing,
    /// Value is decreasing
    Decreasing,
    /// Value hasn't changed significantly
    Stable,
    /// Not enough data to determine trend
    Unknown,
}

impl Default for TrendDirection {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Workspace analysis target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTarget {
    /// Name of the target (usually crate name)
    pub name: String,

    /// Path to target directory
    pub path: PathBuf,

    /// Target kind (lib, bin, etc.)
    pub kind: String,

    /// Whether this target should be analyzed
    pub enabled: bool,
}
