use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use syn::File;

use crate::analysis::{AnalysisFinding, AnalysisPreferences};

/// Preferences that control analyzer behavior
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisPreferences {
    /// Whether to include security-related findings
    pub security:       bool,
    /// Whether to include performance-related findings
    pub performance:    bool,
    /// Whether to include architectural findings
    pub architectural:  bool,
    /// Whether to include code metrics
    pub metrics:        bool,
    /// Minimum confidence level for findings (0.0 to 1.0)
    pub min_confidence: f32,
    /// Maximum number of findings to return (0 for unlimited)
    pub max_findings:   usize,
}

/// Trait for all analysis components to ensure consistent interface
pub trait Analyzer: Send + Sync + 'static {
    /// The type of findings this analyzer produces
    type Finding: Send + Sync + 'static;

    /// Analyze the given code and return findings
    fn analyze(&self, ast: &File, code: &str, file_path: &str) -> Result<Vec<Self::Finding>>;

    /// Get the name/identifier of this analyzer
    fn name(&self) -> &'static str;

    /// Get a brief description of what this analyzer does
    fn description(&self) -> &'static str;

    /// Check if this analyzer is enabled based on preferences
    fn is_enabled(&self, _preferences: &AnalysisPreferences) -> bool {
        true
    }

    /// Get the default configuration for this analyzer
    fn default_config() -> serde_json::Value
    where
        Self: Sized,
    {
        serde_json::json!({})
    }
}

/// Extension trait for analyzers to provide additional functionality
pub trait AnalyzerExt: Analyzer {
    /// Run the analyzer only if it's enabled according to preferences
    fn run_if_enabled(
        &self,
        ast: &File,
        code: &str,
        file_path: &str,
        preferences: &AnalysisPreferences,
    ) -> Result<Vec<Self::Finding>> {
        if self.is_enabled(preferences) {
            self.analyze(ast, code, file_path)
        } else {
            Ok(Vec::new())
        }
    }

    /// Filter findings based on confidence level and other criteria
    fn filter_findings(&self, findings: Vec<Self::Finding>, preferences: &AnalysisPreferences) -> Vec<Self::Finding>
    where
        Self::Finding: HasConfidence,
    {
        let mut filtered = findings
            .into_iter()
            .filter(|f| f.confidence() >= preferences.min_confidence)
            .collect::<Vec<_>>();

        if preferences.max_findings > 0 && filtered.len() > preferences.max_findings {
            filtered.truncate(preferences.max_findings);
        }

        filtered
    }
}

impl<A: Analyzer> AnalyzerExt for A {}

/// Blanket implementation for any type that implements Analyzer
impl<A> Analyzer for Box<dyn Analyzer<Finding = A>>
where
    A: Send + Sync + 'static,
{
    type Finding = A;

    fn analyze(&self, ast: &File, code: &str, file_path: &str) -> Result<Vec<Self::Finding>> {
        self.as_ref().analyze(ast, code, file_path)
    }

    fn name(&self) -> &'static str {
        self.as_ref().name()
    }

    fn description(&self) -> &'static str {
        self.as_ref().description()
    }

    fn is_enabled(&self, preferences: &AnalysisPreferences) -> bool {
        self.as_ref().is_enabled(preferences)
    }
}

/// A finding that can be reported by an analyzer
pub trait Finding: Send + Sync + 'static {
    /// Get the location of this finding in the source code
    fn location(&self) -> CodeLocation;

    /// Get a short title describing this finding
    fn title(&self) -> &str;

    /// Get a detailed description of this finding
    fn description(&self) -> &str;

    /// Get the severity level of this finding
    fn severity(&self) -> Severity;
}

/// A finding that includes a confidence level
pub trait HasConfidence: Finding {
    /// Get the confidence level of this finding (0.0 to 1.0)
    fn confidence(&self) -> f32;
}

/// Severity level for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Information only
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical issue
    Critical,
}

/// Location in the source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    /// File path
    pub file:       String,
    /// Line number (1-based)
    pub line:       usize,
    /// Column number (1-based)
    pub column:     usize,
    /// End line number (1-based, inclusive)
    pub end_line:   Option<usize>,
    /// End column number (1-based, exclusive)
    pub end_column: Option<usize>,
}

impl CodeLocation {
    /// Create a new code location
    pub fn new(file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            end_line: None,
            end_column: None,
        }
    }

    /// Set the end position of this location
    pub fn with_end(mut self, line: usize, column: usize) -> Self {
        self.end_line = Some(line);
        self.end_column = Some(column);
        self
    }

    /// Check if this location contains the given position
    pub fn contains(&self, line: usize, column: usize) -> bool {
        if line < self.line || (line == self.line && column < self.column) {
            return false;
        }

        if let Some(end_line) = self.end_line {
            if line > end_line || (line == end_line && column >= self.end_column.unwrap_or(usize::MAX)) {
                return false;
            }
        }

        true
    }
}

/// Helper function to convert any analyzer's output to AnalysisFindings
pub fn to_analysis_findings<F>(findings: Vec<F>) -> Vec<AnalysisFinding>
where
    F: Into<AnalysisFinding>,
{
    findings.into_iter().map(Into::into).collect()
}
