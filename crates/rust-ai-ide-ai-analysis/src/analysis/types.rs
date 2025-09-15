use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Severity levels for analysis issues
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Location in source code
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    pub file:   String,
    pub line:   usize,
    pub column: usize,
    pub offset: usize,
}

/// Code range in source
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Range {
    pub start_line: u32,
    pub start_col:  u32,
    pub end_line:   u32,
    pub end_col:    u32,
}

/// Categories of analysis
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisCategory {
    Security,
    Performance,
    CodeQuality,
    Architecture,
    Documentation,
    Other,
}

impl fmt::Display for AnalysisCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Security => write!(f, "Security"),
            Self::Performance => write!(f, "Performance"),
            Self::CodeQuality => write!(f, "CodeQuality"),
            Self::Architecture => write!(f, "Architecture"),
            Self::Documentation => write!(f, "Documentation"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Suggestion for improvement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Suggestion {
    pub id:          Uuid,
    pub title:       String,
    pub description: String,
    pub location:    Option<Location>,
    pub actions:     Vec<SuggestionAction>,
    pub priority:    Priority,
}

/// Action that can be taken to resolve an issue
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestionAction {
    pub description:  String,
    pub code_changes: Vec<CodeChange>,
}

/// Priority level for suggestions
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Code change specification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeChange {
    pub file:        String,
    pub start_line:  usize,
    pub end_line:    usize,
    pub replacement: String,
    pub description: String,
}

/// Security vulnerability
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub id:          Uuid,
    pub cwe_id:      Option<String>,
    pub title:       String,
    pub description: String,
    pub severity:    Severity,
    pub location:    Location,
    pub evidence:    String,
    pub mitigation:  String,
    pub category:    SecurityCategory,
}

/// Security vulnerability categories
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityCategory {
    Injection,
    Authentication,
    Authorization,
    Cryptography,
    Configuration,
    InputValidation,
    ErrorHandling,
    Logging,
    DenialOfService,
    RaceConditions,
    Memory,
    Other,
}

/// Performance issue
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceHint {
    pub id:          Uuid,
    pub title:       String,
    pub description: String,
    pub impact:      PerformanceImpact,
    pub location:    Location,
    pub suggestion:  String,
}

/// Performance impact levels
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PerformanceImpact {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Code smell detection result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeSmell {
    pub id:                  Uuid,
    pub smell_type:          CodeSmellType,
    pub title:               String,
    pub description:         String,
    pub location:            Location,
    pub severity:            Severity,
    pub refactoring_pattern: Option<String>,
}

/// Types of code smells
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeSmellType {
    LongMethod,
    LargeClass,
    DuplicateCode,
    TooManyParameters,
    ComplexCondition,
    UnusedVariables,
    DeadCode,
    TightCoupling,
    MagicNumbers,
    InconsistentNaming,
    MissingDocumentation,
    PoorErrorHandling,
    ResourceLeak,
    ThreadingIssue,
    Other,
}

/// Code quality metrics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code:          usize,
    pub complexity:             f64,
    pub maintainability_index:  f64,
    pub cyclomatic_complexity:  usize,
    pub coupling:               f64,
    pub cohesion:               f64,
    pub test_coverage:          Option<f64>,
    pub documentation_coverage: f64,
}

/// Architecture pattern suggestion
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchitectureSuggestion {
    pub pattern:              String,
    pub confidence:           f64,
    pub location:             Location,
    pub description:          String,
    pub benefits:             Vec<String>,
    pub implementation_steps: Vec<String>,
}

/// Complete analysis result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: Uuid,
    pub file_path: String,
    pub timestamp: DateTime<Utc>,
    pub security_issues: Vec<SecurityIssue>,
    pub performance_hints: Vec<PerformanceHint>,
    pub code_smells: Vec<CodeSmell>,
    pub architecture_suggestions: Vec<ArchitectureSuggestion>,
    pub metrics: CodeMetrics,
}

impl AnalysisResult {
    /// Get total number of issues
    pub fn total_issues(&self) -> usize {
        self.security_issues.len()
            + self.performance_hints.len()
            + self.code_smells.len()
            + self.architecture_suggestions.len()
    }

    /// Get issues grouped by severity
    pub fn issues_by_severity(&self) -> std::collections::HashMap<Severity, usize> {
        let mut counts = std::collections::HashMap::new();

        for issue in &self.security_issues {
            *counts.entry(issue.severity).or_insert(0) += 1;
        }

        for smell in &self.code_smells {
            *counts.entry(smell.severity).or_insert(0) += 1;
        }

        for hint in &self.performance_hints {
            *counts.entry(hint.impact.into()).or_insert(0) += 1;
        }

        counts
    }
}

impl From<PerformanceImpact> for Severity {
    fn from(impact: PerformanceImpact) -> Self {
        match impact {
            PerformanceImpact::None => Severity::Info,
            PerformanceImpact::Low => Severity::Info,
            PerformanceImpact::Medium => Severity::Warning,
            PerformanceImpact::High => Severity::Error,
            PerformanceImpact::Critical => Severity::Critical,
        }
    }
}
