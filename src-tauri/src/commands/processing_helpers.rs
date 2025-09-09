//! Processing and helper functions for diagnostic operations
//!
//! This module contains helper functions for parsing diagnostic information,
//! extracting error explanations, generating fixes, and related utility functions.

use crate::modules::shared::diagnostics::*;
use crate::commands::utils::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use uuid;

/// Run cargo check on a workspace
async fn run_cargo_check(workspace_path: &str) -> Result<String> {
    // Use centralized run_cargo_check function from error_handling module
    crate::diagnostics::error_handling::run_cargo_check(workspace_path).await
}

/// Parse a compiler diagnostic from JSON
async fn parse_compiler_diagnostic(
    message: &serde_json::Value,
    workspace_path: &str,
) -> Option<CompilerDiagnostic> {
    // Use centralized parsing function
    parse_compiler_diagnostic(message, workspace_path).await
}

/// Parse a compiler span from JSON
fn parse_compiler_span(span: &serde_json::Value) -> Option<CompilerSpan> {
    // Use centralized parsing function
    crate::diagnostics::parsing::parse_compiler_span(span)
}

/// Parse span text information
fn parse_span_text(text: &serde_json::Value) -> Option<SpanText> {
    // Use centralized parsing function
    crate::diagnostics::parsing::parse_span_text(text)
}

/// Get cached error explanation with TTL
async fn get_cached_error_explanation(
    error_code: &str,
    explanation_cache: tauri::State<'_, ExplanationCacheState>,
    ttl_seconds: u64,
) -> Result<ErrorCodeExplanation> {
    // Use centralized function
    crate::diagnostics::error_handling::get_cached_error_explanation(
        error_code,
        explanation_cache,
        ttl_seconds
    ).await
}

/// Get error code explanation from external source
async fn get_error_code_explanation(error_code: &str) -> Result<ErrorCodeExplanation> {
    // Use centralized function
    crate::diagnostics::error_handling::get_error_code_explanation(error_code).await
}

/// Parse Rust compiler error explanation text
pub fn parse_rustc_explanation(text: &str) -> (String, String, Vec<ErrorExample>) {
    let lines: Vec<&str> = text.lines().collect();

    let title = lines.first()
        .unwrap_or(&"")
        .trim()
        .to_string();

    let explanation = text.to_string();

    // Extract examples (simplified - would need more sophisticated parsing)
    let mut examples = Vec::new();
    let mut in_example = false;
    let mut current_example = String::new();
    let mut example_description = String::new();

    for line in lines {
        if line.trim().starts_with("```") {
            if in_example {
                // End of example
                examples.push(ErrorExample {
                    description: example_description.clone(),
                    code: current_example.clone(),
                    explanation: "Example code".to_string(),
                    fix: None,
                });
                current_example.clear();
                example_description.clear();
                in_example = false;
            } else {
                // Start of example
                in_example = true;
            }
        } else if in_example {
            current_example.push_str(line);
            current_example.push('\n');
        } else if !line.trim().is_empty() && !in_example {
            example_description = line.trim().to_string();
        }
    }

    (title, explanation, examples)
}

/// Extract related error codes from explanation text
pub fn extract_related_errors(text: &str) -> Vec<String> {
    let mut related = Vec::new();

    // Look for error code patterns like E0001, E0002, etc.
    for line in text.lines() {
        if let Some(captures) = regex::Regex::new(r"E\d{4}")
            .ok()
            .and_then(|re| re.find(line)) {
            let error_code = captures.as_str().to_string();
            if !related.contains(&error_code) {
                related.push(error_code);
            }
        }
    }

    related
}

/// Extract common causes from explanation text
pub fn extract_common_causes(text: &str) -> Vec<String> {
    let mut causes = Vec::new();

    // Look for common patterns that indicate causes
    let cause_patterns = [
        "This error occurs when",
        "This happens when",
        "The cause of this error",
        "This is caused by",
    ];

    for line in text.lines() {
        for pattern in &cause_patterns {
            if line.contains(pattern) {
                causes.push(line.trim().to_string());
                break;
            }
        }
    }

    causes
}

/// Extract suggested solutions from explanation text
pub fn extract_suggested_solutions(text: &str) -> Vec<String> {
    let mut solutions = Vec::new();

    // Look for solution patterns
    let solution_patterns = [
        "To fix this",
        "You can fix this by",
        "The solution is",
        "Try",
        "Consider",
    ];

    for line in text.lines() {
        for pattern in &solution_patterns {
            if line.contains(pattern) {
                solutions.push(line.trim().to_string());
                break;
            }
        }
    }

    solutions
}

/// Generate suggested fixes from compiler diagnostic spans
pub async fn generate_suggested_fixes(diagnostic: &CompilerDiagnostic) -> Result<Vec<FixSuggestion>> {
    let mut fixes = Vec::new();

    // Extract suggestions from compiler spans
    for span in &diagnostic.spans {
        if let Some(replacement) = &span.suggested_replacement {
            let fix = FixSuggestion {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Apply compiler suggestion".to_string(),
                description: span.label.clone()
                    .unwrap_or_else(|| "Compiler suggested fix".to_string()),
                fix_type: FixType::QuickFix,
                changes: vec![CodeChange {
                    file_path: span.file_name.clone(),
                    range: (span.line_start, span.column_start, span.line_end, span.column_end),
                    old_text: String::new(), // Would need to extract from source
                    new_text: replacement.clone(),
                    change_type: CompilerChangeType::Replace,
                }],
                confidence: if span.suggestion_applicability
                    .as_ref()
                    .map_or(false, |a| a == "machine-applicable") {
                    0.9
                } else {
                    0.7
                },
                estimated_effort: EstimatedEffort::Trivial,
                benefits: vec!["Fixes compiler error".to_string()],
                risks: vec![],
            };
            fixes.push(fix);
        }
    }

    Ok(fixes)
}