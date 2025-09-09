//! Diagnostic parsing functions

use crate::diagnostics::*;
use anyhow::Result;
use serde_json::Value;

/// Parse a compiler diagnostic from JSON message
pub async fn parse_compiler_diagnostic(
    message: &Value,
    workspace_path: &str,
) -> Option<CompilerDiagnostic> {
    let level = message.get("level")?.as_str()?.to_string();
    let msg = message.get("message")?.as_str()?.to_string();

    let code = if let Some(code_obj) = message.get("code") {
        Some(CompilerErrorCode {
            code: code_obj.get("code")?.as_str()?.to_string(),
            explanation: code_obj.get("explanation")
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

    let children = if let Some(children_array) = message.get("children").and_then(|c| c.as_array()) {
        let mut children = Vec::new();
        for child in children_array {
            if let Some(child_diagnostic) = Box::pin(parse_compiler_diagnostic(child, workspace_path)).await {
                children.push(child_diagnostic);
            }
        }
        children
    } else {
        Vec::new()
    };

    let rendered = message.get("rendered")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    // Extract context information
    let context = extract_diagnostic_context(&spans, workspace_path).await;

    // Note: Using only fields available in rust_ai_ide_lsp::CompilerDiagnostic (file_path, line, column)
    // Other fields from full definition are omitted as LSP version is incomplete
    // Providing defaults for main diagnostic information
    Some(CompilerDiagnostic {
        level,
        message: msg,
        file_path: context.file_path.clone(),
        line: spans.first().map_or(0, |s| s.line_start), // Use first span's line if available
        column: spans.first().map_or(0, |s| s.column_start), // Use first span's column if available
    })
}

/// Parse a compiler span from JSON
pub fn parse_compiler_span(span: &Value) -> Option<CompilerSpan> {
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

    let label = span.get("label")
        .and_then(|l| l.as_str())
        .map(|s| s.to_string());
    let suggested_replacement = span.get("suggested_replacement")
        .and_then(|sr| sr.as_str())
        .map(|s| s.to_string());
    let suggestion_applicability = span.get("suggestion_applicability")
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

/// Parse span text information
pub fn parse_span_text(text: &Value) -> Option<SpanText> {
    let text_str = text.get("text")?.as_str()?.to_string();
    let highlight_start = text.get("highlight_start")?.as_u64().unwrap_or(0) as u32;
    let highlight_end = text.get("highlight_end")?.as_u64().unwrap_or(0) as u32;

    Some(SpanText {
        text: text_str,
        highlight_start,
        highlight_end,
    })
}

/// Extract diagnostic context from spans
pub async fn extract_diagnostic_context(
    spans: &[CompilerSpan],
    workspace_path: &str,
) -> DiagnosticContext {
    let mut context = DiagnosticContext {
        file_path: String::new(),
        function_name: None,
        module_path: None,
        surrounding_code: None,
        related_diagnostics: Vec::new(),
    };

    if let Some(main_span) = spans.iter().find(|s| s.is_main_span) {
        context.file_path = main_span.file_name.clone();

        // Try to extract surrounding code context
        if let Ok(file_content) = tokio::fs::read_to_string(&main_span.file_name).await {
            let lines: Vec<&str> = file_content.lines().collect();
            let start_line = main_span.line_start.saturating_sub(3) as usize;
            let end_line = ((main_span.line_end + 3) as usize).min(lines.len());

            if start_line < lines.len() && end_line <= lines.len() {
                let surrounding = lines[start_line..end_line].join("\n");
                context.surrounding_code = Some(surrounding);
            }

            // Try to extract function name
            if let Some(line) = lines.get(main_span.line_start.saturating_sub(1) as usize) {
                if let Some(func_name) = extract_function_name(line) {
                    context.function_name = Some(func_name);
                }
            }

            // Try to extract module path
            context.module_path = extract_module_path(&file_content);
        }
    }

    context
}

/// Extract function name from a line of code
pub fn extract_function_name(line: &str) -> Option<String> {
    // Simple regex to extract function name
    if let Some(start) = line.find("fn ") {
        let after_fn = &line[start + 3..];
        if let Some(end) = after_fn.find('(') {
            let func_name = after_fn[..end].trim().trim_end_matches('&').to_string();
            if !func_name.is_empty() {
                return Some(func_name);
            }
        }
    }
    None
}

/// Extract module path from content
pub fn extract_module_path(content: &str) -> Option<String> {
    // Look for module declarations
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("mod ") && !trimmed.contains('{') {
            if let Some(module_name) = trimmed.strip_prefix("mod ").and_then(|s| s.split(';').next()) {
                return Some(module_name.trim().to_string());
            }
        }
    }
    None
}

/// Generate suggested fixes from diagnostic
pub async fn generate_suggested_fixes(diagnostic: &CompilerDiagnostic) -> Result<Vec<FixSuggestion>> {
    let mut fixes = Vec::new();

    // Note: LSP CompilerDiagnostic doesn't have spans field, so we cannot access suggested_replacement
    // Since we can't modify the LSP crate, we'll create a basic fix suggestion without span-specific info
    // In a complete implementation, spans would be parsed separately if available

    // For now, create a basic suggestion when spans are not available in the incomplete LSP definition
    // Create a basic fix suggestion since spans are not available in incomplete LSP definition
    let fix = FixSuggestion {
        id: uuid::Uuid::new_v4().to_string(),
        title: "Basic compiler suggestion".to_string(),
        description: diagnostic.message.clone(),
        changes: vec![], // Empty changes since spans are not available
        confidence: 0.5, // Medium confidence
        explanation: format!("Basic suggestion for compiler error: {}", diagnostic.message),
        documentation_links: vec![],
        auto_applicable: false, // Cannot auto-apply without specific changes
        impact: FixImpact::Local,
        source_pattern: None,
        warnings: vec![],
    };
    fixes.push(fix);
    // Since spans are unavailable, we don't process suggested_replacement here

    Ok(fixes)
}