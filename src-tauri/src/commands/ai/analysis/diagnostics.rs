//! Compiler diagnostics, error resolution, and code explanations
//!
//! This module provides functionality for handling compiler diagnostics,
//! resolving errors with AI assistance, and explaining error codes.

// Import centralized diagnostic types
use crate::diagnostics::{
    ChangeType, CodeChange, CompilerDiagnostic, CompilerDiagnosticsRequest,
    CompilerDiagnosticsResult, CompilerErrorCode, CompilerSpan, DocumentationLink,
    ErrorCodeExplanation, ErrorExample, EstimatedEffort, FixSuggestion, FixType, SpanText,
};

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use rust_ai_ide_lsp::error_resolution::FixType;
use rust_ai_ide_core::shell_utils::{cargo, rustc};
use serde::{Deserialize, Serialize};
use serde_json;
use tauri::State;
use uuid::Uuid;

use crate::command_templates::*;
use rust_ai_ide_lsp::AIService;

/// Error resolution request
#[derive(Debug, Deserialize)]
pub struct ErrorResolutionRequest {
    pub file_path: String,
    pub content: String,
    pub errors: Vec<String>,
    pub cursor_position: Option<(u32, u32)>,
    pub use_learned_patterns: bool,
}

/// AI service state (re-exported for convenience)
pub type AIServiceState = Arc<tokio::sync::Mutex<Option<AIService>>>;

/// Get compiler diagnostics
#[tauri::command]
pub async fn get_compiler_diagnostics(
    request: CompilerDiagnosticsRequest,
) -> Result<CompilerDiagnosticsResult, String> {
    log::info!(
        "Getting compiler diagnostics for: {}",
        request.workspace_path
    );

    // Run cargo check using unified utility
    let workspace_path_buf = Path::new(&request.workspace_path);
    let result = cargo::check(workspace_path_buf)
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;

    if !result.success {
        return Err(format!("Cargo check failed: {}", result.stderr).into());
    }

    let stdout = result.stdout;
    let mut diagnostics = Vec::new();
    let mut explanations = HashMap::new();
    let mut suggested_fixes = Vec::new();

    // Parse JSON output
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(message) = json.get("message") {
                if let Some(diagnostic) = parse_compiler_diagnostic(message) {
                    // Get explanation for error codes if requested
                    if request.include_explanations {
                        if let Some(code) = &diagnostic.code {
                            if let Ok(explanation) = get_error_code_explanation(&code.code).await {
                                explanations.insert(code.code.clone(), explanation);
                            }
                        }
                    }

                    // Generate suggested fixes if requested
                    if request.include_suggested_fixes {
                        if let Ok(fixes) = generate_suggested_fixes(&diagnostic).await {
                            suggested_fixes.extend(fixes);
                        }
                    }

                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    Ok(CompilerDiagnosticsResult {
        diagnostics,
        explanations,
        suggested_fixes,
    })
}

/// Resolve errors with AI assistance
#[tauri::command]
pub async fn resolve_errors_with_ai(
    request: ErrorResolutionRequest,
    ai_service: State<'_, AIServiceState>,
) -> Result<Vec<FixSuggestion>, String> {
    log::info!("Resolving errors with AI for: {}", request.file_path);

    let ai_service_guard = ai_service.lock().await;
    let service = ai_service_guard
        .as_ref()
        .ok_or("AI service not initialized")?;

pub use rust_ai_ide_lsp::learning::types::AnalysisPreferences;
use rust_ai_ide_lsp::AIContext;

use rust_ai_ide_lsp::learning::types::AnalysisPreferences;

    let context = AIContext {
        current_code: request.content,
        file_name: Some(request.file_path.clone()),
        cursor_position: request.cursor_position,
        selection: None,
        project_context: HashMap::new(),
        dependencies: Vec::new(),
        workspace_structure: HashMap::new(),
        analysis_preferences: AnalysisPreferences::default(),
    };

    let mut fixes = service
        .resolve_errors(context, request.errors)
        .await
        .map_err(|e| format!("Error resolution failed: {}", e))?;

    // Enhance with learned patterns if requested
    if request.use_learned_patterns {
        for error in &request.errors {
            if let Ok(learned_patterns) = service.get_learned_patterns(error).await {
                for pattern in learned_patterns {
                    // Convert learned pattern to fix suggestion
                    use rust_ai_ide_lsp::LearnedPattern;
                    let learned_fix = FixSuggestion {
                        id: format!("learned_{}", pattern.id),
                        title: format!("Learned fix: {}", pattern.successful_fix.title),
                        description: format!(
                            "Based on {} successful applications",
                            pattern.success_count
                        ),
                        fix_type: pattern.successful_fix.fix_type,
                        changes: pattern.successful_fix.changes,
                        confidence: pattern.confidence,
                        estimated_effort: pattern.successful_fix.estimated_effort,
                        benefits: pattern.successful_fix.benefits,
                        risks: pattern.successful_fix.risks,
                    };
                    fixes.push(learned_fix);
                }
            }
        }
    }

    Ok(fixes)
}

/// Explain error code
#[tauri::command]
pub async fn explain_error_code(error_code: String) -> Result<ErrorCodeExplanation, String> {
    log::info!("Explaining error code: {}", error_code);

    get_error_code_explanation(&error_code)
        .await
        .map_err(|e| format!("Failed to get error explanation: {}", e))
}

/// Parse compiler diagnostic from JSON
pub fn parse_compiler_diagnostic(message: &serde_json::Value) -> Option<CompilerDiagnostic> {
    let level = message.get("level")?.as_str()?.to_string();
    let msg = message.get("message")?.as_str()?.to_string();

    let code = if let Some(code_obj) = message.get("code") {
        Some(CompilerErrorCode {
            code: code_obj.get("code")?.as_str()?.to_string(),
            explanation: code_obj
                .get("explanation")
                .and_then(|e| e.as_str())
                .map(|s| s.to_string()),
        })
    } else {
        None
    };

    let spans = if let Some(spans_array) = message.get("spans").and_then(|s| s.as_array()) {
        spans_array.iter().filter_map(parse_compiler_span).collect()
    } else {
        Vec::new()
    };

    let children = if let Some(children_array) = message.get("children").and_then(|c| c.as_array())
    {
        children_array
            .iter()
            .filter_map(parse_compiler_diagnostic)
            .collect()
    } else {
        Vec::new()
    };

    let rendered = message
        .get("rendered")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    // Extract file_path, line, and column from the first span if available
    let (file_path, line, column) = if let Some(first_span) = spans.first() {
        (first_span.file_name.clone(), first_span.line_start, first_span.column_start)
    } else {
        (String::new(), 0, 0)
    };

    Some(CompilerDiagnostic {
        level,
        message: msg,
        file_path,
        line,
        column,
    })
}

/// Parse compiler span from JSON
pub fn parse_compiler_span(span: &serde_json::Value) -> Option<CompilerSpan> {
    let file_name = span.get("file_name")?.as_str()?.to_string();
    let byte_start = span.get("byte_start")?.as_u64()? as u32;
    let byte_end = span.get("byte_end")?.as_u64()? as u32;
    let line_start = span.get("line_start")?.as_u64()? as u32;
    let line_end = span.get("line_end")?.as_u64()? as u32;
    let column_start = span.get("column_start")?.as_u64()? as u32;
    let column_end = span.get("column_end")?.as_u64()? as u32;
    let is_main_span = span.get("is_primary")?.as_bool().unwrap_or(false);

    let text = if let Some(text_array) = span.get("text").and_then(|t| t.as_array()) {
        text_array.iter().filter_map(parse_span_text).collect()
    } else {
        Vec::new()
    };

    let label = span
        .get("label")
        .and_then(|l| l.as_str())
        .map(|s| s.to_string());
    let suggested_replacement = span
        .get("suggested_replacement")
        .and_then(|sr| sr.as_str())
        .map(|s| s.to_string());
    let suggestion_applicability = span
        .get("suggestion_applicability")
        .and_then(|sa| sa.as_str())
        .map(|s| s.to_string());

    Some(CompilerSpan {
        file_name,
        byte_start,
        byte_end,
        line_start,
        line_end,
        column_start,
        column_end,
        is_main_span,
        text,
        label,
        suggested_replacement,
        suggestion_applicability,
    })
}

/// Parse span text from JSON
pub fn parse_span_text(text: &serde_json::Value) -> Option<SpanText> {
    let text_str = text.get("text")?.as_str()?.to_string();
    let highlight_start = text.get("highlight_start")?.as_u64().unwrap_or(0) as u32;
    let highlight_end = text.get("highlight_end")?.as_u64().unwrap_or(0) as u32;

    Some(SpanText {
        text: text_str,
        highlight_start,
        highlight_end,
    })
}

/// Get error code explanation from rustc
pub async fn get_error_code_explanation(error_code: &str) -> Result<ErrorCodeExplanation> {
    // Run rustc --explain using unified utility
    let result = rustc::explain_error(error_code)
        .map_err(|e| anyhow::anyhow!("Failed to run rustc --explain: {}", e))?;

    if !result.success {
        return Err(anyhow::anyhow!("Rustc explain failed: {}", result.stderr));
    }

    let explanation_text = result.stdout;

    // Parse the explanation (simplified - in reality you'd want more sophisticated parsing)
    let lines: Vec<&str> = explanation_text.lines().collect();
    let title = lines.first().unwrap_or(&"").to_string();
    let explanation = lines.join("\n");

    // Generate documentation links
    let documentation_links = vec![
        DocumentationLink {
            title: format!("Rust Error Index - {}", error_code),
            url: format!("https://doc.rust-lang.org/error-index.html#{}", error_code),
            description: "Official Rust documentation for this error".to_string(),
        },
        DocumentationLink {
            title: "Rust Book".to_string(),
            url: "https://doc.rust-lang.org/book/".to_string(),
            description: "The Rust Programming Language book".to_string(),
        },
    ];

    Ok(ErrorCodeExplanation {
        error_code: error_code.to_string(),
        title,
        explanation,
        examples: Vec::new(), // Would be parsed from the explanation text
        documentation_links,
    })
}

/// Generate suggested fixes from compiler diagnostic
pub async fn generate_suggested_fixes(
    diagnostic: &CompilerDiagnostic,
) -> Result<Vec<FixSuggestion>> {
    let mut fixes = Vec::new();

    // Extract suggestions from compiler spans
    for span in &diagnostic.spans {
        if let Some(replacement) = &span.suggested_replacement {
            let fix = FixSuggestion {
                id: uuid::Uuid::new_v4().to_string(),
                title: format!("Apply compiler suggestion"),
                description: span
                    .label
                    .clone()
                    .unwrap_or_else(|| "Compiler suggested fix".to_string()),
                fix_type: rust_ai_ide_lsp::error_resolution::FixType::QuickFix,
                changes: vec![CodeChange {
                    file_path: span.file_name.clone(),
                    range: (
                        span.line_start,
                        span.column_start,
                        span.line_end,
                        span.column_end,
                    ),
                    old_text: String::new(), // Would need to extract from source
                    new_text: replacement.clone(),
                    change_type: crate::diagnostics::ChangeType::Replace,
                }],
                confidence: if span
                    .suggestion_applicability
                    .as_ref()
                    .map_or(false, |a| a == "machine-applicable")
                {
                    0.9
                } else {
                    0.7
                },
                estimated_effort: "Low".to_string(),
                benefits: vec!["Fixes compiler error".to_string()],
                risks: vec![],
            };
            fixes.push(fix);
        }
    }

    Ok(fixes)
}
