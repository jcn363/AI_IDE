//! JSON output parsing for cargo commands

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::{MonitoringError, Result};
use crate::types::{Finding, Severity};

/// Raw message from cargo check --message-format=json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCargoMessage {
    /// Reason field that indicates the message type
    pub reason: String,

    #[serde(flatten)]
    pub message: Option<CargoMessageData>,
}

/// Parsed cargo message data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "reason")]
pub enum CargoMessageData {
    /// Compiler message (warning, error, etc.)
    #[serde(rename = "compiler-message")]
    CompilerMessage(CompilerMessage),

    /// Build script message
    #[serde(rename = "build-script-executed")]
    BuildScriptExecuted(BuildScriptMessage),

    /// Build finished message
    #[serde(rename = "build-finished")]
    BuildFinished(BuildFinishedMessage),

    /// Other message types
    #[serde(other)]
    Other,
}

/// Compiler message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerMessage {
    /// The actual message content
    pub message: CargoDiagnostic,

    /// Target information
    pub target: Option<CargoTarget>,
}

/// Diagnostic message from rustc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoDiagnostic {
    /// Rendered diagnostic text
    pub rendered: Option<String>,

    /// Diagnostic level (error, warning, note, help)
    pub level: String,

    /// Error code (e.g., "unused_variables", "dead_code")
    pub code: Option<ErrorCode>,

    /// Message content
    pub message: String,

    /// Spans (source locations)
    pub spans: Vec<DiagnosticSpan>,

    /// Child messages
    pub children: Vec<CargoDiagnostic>,

    /// Additional rendered sections
    pub rendered_parts: Vec<String>,
}

/// Error code with optional explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCode {
    /// The error code (e.g., "E0308", "unused_variables")
    pub code: String,

    /// Explanation URL
    pub explanation: Option<String>,
}

/// Source span in a diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSpan {
    /// File path
    pub file_name: String,

    /// Byte offset in file
    pub byte_start: usize,

    /// Byte offset in file (end)
    pub byte_end: usize,

    /// Line number (1-based)
    pub line_start: usize,

    /// Line number (1-based, end)
    pub line_end: usize,

    /// Column number (1-based)
    pub column_start: usize,

    /// Column number (1-based, end)
    pub column_end: usize,

    /// Whether this span is the primary one
    pub is_primary: bool,

    /// Source text for this span
    pub text: Vec<DiagnosticSpanText>,

    /// Label for this span
    pub label: Option<String>,

    /// Suggested replacement
    pub suggested_replacement: Option<String>,

    /// Expansion information for macros
    pub expansion: Option<DiagnosticSpanExpansion>,
}

/// Text content of a diagnostic span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSpanText {
    /// Text content
    pub text: String,

    /// Whether this is a highlight
    pub highlight_start: usize,

    /// Highlight end
    pub highlight_end: usize,
}

/// Span expansion information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSpanExpansion {
    /// Expansion kind (e.g., "Macro", "Desugaring")
    pub span: Option<Box<DiagnosticSpan>>,
}

/// Build script execution message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildScriptMessage {
    /// Package the build script belongs to
    pub package_id: String,
}

/// Build finished message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildFinishedMessage {
    /// Whether the build succeeded
    pub success: bool,
}

/// Target information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoTarget {
    /// Target kind (lib, bin, etc.)
    pub kind: Vec<String>,

    /// Crate type
    pub crate_types: Vec<String>,

    /// Target name
    pub name: String,

    /// Source path
    pub src_path: String,
}

/// Parsed cargo check results
#[derive(Debug, Clone)]
pub struct CargoCheckResults {
    /// All diagnostic messages
    pub diagnostics: Vec<CargoDiagnostic>,

    /// Build success status
    pub success: bool,

    /// Compilation time if available
    pub compilation_time_ms: Option<u64>,

    /// Summary counts
    pub summary: ResultsSummary,
}

/// Summary of results
#[derive(Debug, Clone, Default)]
pub struct ResultsSummary {
    /// Number of errors
    pub errors: usize,

    /// Number of warnings
    pub warnings: usize,

    /// Number of notes
    pub notes: usize,

    /// Number of help messages
    pub helps: usize,

    /// Number of build script executions
    pub build_scripts: usize,
}

/// Classification of diagnostic messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticClass {
    /// Compilation error
    Error,
    /// Warning that could be cleaned up
    Warning,
    /// Informational note
    Note,
    /// Help suggestion
    Help,
    /// Unknown or other
    Other,
}

/// Parser for cargo JSON output
pub struct CargoJsonParser {
    /// Raw JSON lines from cargo output
    raw_lines: Vec<String>,
}

impl CargoJsonParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            raw_lines: Vec::new(),
        }
    }

    /// Create a parser from raw cargo output
    pub fn from_output(output: &str) -> Self {
        let raw_lines = output
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        Self { raw_lines }
    }

    /// Parse all messages from the output
    pub fn parse_output(&self) -> Result<CargoCheckResults> {
        let mut diagnostics = Vec::new();
        let mut success = true;
        let mut build_scripts = 0;

        for line in &self.raw_lines {
            match self.parse_single_message(line) {
                Ok(Some(message)) => {
                    match message {
                        CargoMessageData::CompilerMessage(msg) => {
                            diagnostics.push(msg.message);
                        }
                        CargoMessageData::BuildScriptExecuted(_) => {
                            build_scripts += 1;
                        }
                        CargoMessageData::BuildFinished(finished) => {
                            success = finished.success;
                        }
                        CargoMessageData::Other => {
                            // Skip other message types
                        }
                    }
                }
                Ok(None) => {
                    // Skip empty or invalid lines
                }
                Err(e) => {
                    // Log warning but continue parsing
                    tracing::warn!("Failed to parse cargo message: {} (line: {})", e, line);
                }
            }
        }

        let summary = self.build_summary(&diagnostics, build_scripts);

        Ok(CargoCheckResults {
            diagnostics,
            success,
            compilation_time_ms: None, // Would need timing info from command execution
            summary,
        })
    }

    /// Parse a single JSON message
    fn parse_single_message(&self, line: &str) -> Result<Option<CargoMessageData>> {
        if line.trim().is_empty() {
            return Ok(None);
        }

        let raw_message: RawCargoMessage = serde_json::from_str(line)
            .map_err(|e| MonitoringError::cargo_parse(format!("JSON parse error: {} (line: {})", e, line)))?;

        match raw_message.message {
            Some(message) => Ok(Some(message)),
            None => Ok(None),
        }
    }

    /// Build results summary
    fn build_summary(&self, diagnostics: &[CargoDiagnostic], build_scripts: usize) -> ResultsSummary {
        let mut summary = ResultsSummary {
            build_scripts,
            ..Default::default()
        };

        for diagnostic in diagnostics {
            match diagnostic.level.as_str() {
                "error" => summary.errors += 1,
                "warning" => summary.warnings += 1,
                "note" => summary.notes += 1,
                "help" => summary.helps += 1,
                _ => {}
            }
        }

        summary
    }

    /// Convert cargo diagnostics to monitoring findings
    pub fn diagnostics_to_findings(diagnostics: &[CargoDiagnostic]) -> Vec<Finding> {
        diagnostics
            .iter()
            .filter_map(|diagnostic| Self::diagnostic_to_finding(diagnostic))
            .collect()
    }

    /// Convert a single diagnostic to a finding
    fn diagnostic_to_finding(diagnostic: &CargoDiagnostic) -> Option<Finding> {
        let severity = Self::level_to_severity(&diagnostic.level)?;

        // Get the primary span for file/line information
        let primary_span = diagnostic.spans.iter().find(|span| span.is_primary);

        let (file, line, column, code_snippet) = if let Some(span) = primary_span {
            let snippet = span.text.first().map(|t| t.text.clone());
            (
                Path::new(&span.file_name).to_path_buf(),
                Some(span.line_start),
                Some(span.column_start),
                snippet,
            )
        } else {
            (Path::new("unknown").to_path_buf(), None, None, None)
        };

        let message = if let Some(ref rendered) = diagnostic.rendered {
            if !rendered.is_empty() {
                rendered.clone()
            } else {
                diagnostic.message.clone()
            }
        } else {
            diagnostic.message.clone()
        };

        let issue_type = diagnostic
            .code
            .as_ref()
            .map(|code| code.code.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let suggestion = primary_span.and_then(|span| span.suggested_replacement.clone());

        Some(Finding {
            file,
            line,
            column,
            issue_type,
            severity,
            message,
            code: code_snippet,
            suggestion,
        })
    }

    /// Convert diagnostic level to severity
    fn level_to_severity(level: &str) -> Option<Severity> {
        match level {
            "error" => Some(Severity::High),
            "warning" => Some(Severity::Medium),
            "note" => Some(Severity::Low),
            "help" => Some(Severity::Info),
            _ => Some(Severity::Info),
        }
    }
}

impl Default for CargoJsonParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute cargo check and parse results
pub async fn cargo_check_workspace(workspace_root: &Path) -> Result<CargoCheckResults> {
    use tokio::process::Command;

    let start_time = std::time::Instant::now();

    let output = Command::new("cargo")
        .args(&["check", "--workspace", "--message-format=json"])
        .current_dir(workspace_root)
        .output()
        .await
        .map_err(|e| MonitoringError::command_execution("cargo check".to_string(), e))?;

    let duration = start_time.elapsed();
    let success = output.status.success();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !success {
        return Err(MonitoringError::command_failed(
            "cargo check".to_string(),
            output.status.code().unwrap_or(-1),
            stderr.to_string(),
        ));
    }

    let mut parser = CargoJsonParser::from_output(&stdout);
    let mut results = parser.parse_output()?;
    results.compilation_time_ms = Some(duration.as_millis() as u64);
    results.success = success;

    Ok(results)
}

/// Parse cargo check output from string
pub fn parse_cargo_output(output: &str) -> Result<CargoCheckResults> {
    CargoJsonParser::from_output(output).parse_output()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_output() {
        let results = parse_cargo_output("").unwrap();
        assert!(results.diagnostics.is_empty());
        assert_eq!(results.summary.errors, 0);
        assert_eq!(results.summary.warnings, 0);
    }

    #[test]
    fn test_parse_warning_message() {
        let warning_json = r#"{"reason":"compiler-message","message":{"rendered":"warning: unused variable `x`\n --> src/main.rs:2:9\n...\n","level":"warning","spans":[{"file_name":"src/main.rs","byte_start":10,"byte_end":11,"line_start":2,"line_end":2,"column_start":9,"column_end":10,"is_primary":true,"text":[{"text":"    let x = 1;","highlight_start":8,"highlight_end":9}],"label":"unused variable","suggested_replacement":"_x"}],"children":[],"rendered_parts":["warning: unused variable `x`\n --> src/main.rs:2:9\n  |\n2 |     let x = 1;\n  |         ^ unused variable\n  |\n  = note: `#[warn(unused_variables)]` on by default\n  = note: to avoid this warning, consider using `_x` instead"]}}"#;

        let results = parse_cargo_output(warning_json).unwrap();
        assert_eq!(results.summary.warnings, 1);
        assert_eq!(results.diagnostics.len(), 1);

        let diagnostic = &results.diagnostics[0];
        assert_eq!(diagnostic.level, "warning");
        assert!(diagnostic.message.contains("unused variable"));

        let findings = CargoJsonParser::diagnostics_to_findings(&results.diagnostics);
        assert_eq!(findings.len(), 1);

        let finding = &findings[0];
        assert_eq!(finding.severity, Severity::Medium);
        assert_eq!(finding.line, Some(2));
        assert_eq!(finding.issue_type, "unknown"); // No error code in this example
    }
}
