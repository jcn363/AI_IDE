//! Code action support for the Rust Language Server

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, Diagnostic, NumberOrString,
    WorkspaceEdit,
};
use std::sync::Arc;
use crate::pool::{LanguageServerPool, ServerLoadMetrics};

/// Generate code actions for the given parameters
pub async fn get_code_actions(params: &CodeActionParams, pool: &Arc<LanguageServerPool>) -> Option<Vec<CodeActionOrCommand>> {
    let mut actions = Vec::new();
    let context = &params.context;

    // Get load metrics to assess server availability for code actions
    let load_metrics = pool.get_server_load_metrics().await;
    let can_perform_code_actions = should_perform_code_actions(&load_metrics).await;

    if !can_perform_code_actions {
        // Return minimal actions when servers are under heavy load
        return Some(vec![create_load_aware_action()]);
    }

    // Add quick fixes for diagnostics
    for diagnostic in &context.diagnostics {
        if let Some(code) = &diagnostic.code {
            let code_str = match code {
                NumberOrString::Number(n) => n.to_string(),
                NumberOrString::String(s) => s.clone(),
            };
            match code_str.as_str() {
                // Add quick fixes for common Rust errors
                "unused_variables" => {
                    if let Some(edit) = quick_fix_unused_variable(diagnostic, params, &load_metrics).await {
                        actions.push(create_code_action(
                            "Remove unused variable",
                            edit,
                            CodeActionKind::QUICKFIX,
                            diagnostic,
                        ));
                    }
                }
                "dead_code" => {
                    if let Some(edit) = quick_fix_dead_code(diagnostic, params, &load_metrics).await {
                        actions.push(create_code_action(
                            "Remove dead code",
                            edit,
                            CodeActionKind::QUICKFIX,
                            diagnostic,
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    // Add refactoring actions
    if let Some(selection) = get_selection_range(params, &load_metrics).await {
        actions.extend(get_refactor_actions(params, selection, &load_metrics).await);
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}

/// Create a code action with the given parameters
fn create_code_action(
    title: &str,
    edit: WorkspaceEdit,
    kind: CodeActionKind,
    diagnostic: &Diagnostic,
) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(kind),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(edit),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

/// Quick fix for unused variables
async fn quick_fix_unused_variable(
    diagnostic: &Diagnostic,
    params: &CodeActionParams,
    load_metrics: &[ServerLoadMetrics],
) -> Option<WorkspaceEdit> {
    // Check if servers are available and healthy enough for code analysis
    let healthy_servers = load_metrics.iter()
        .filter(|m| m.health_score > 0.7 && m.pending_requests < 5)
        .count();

    if healthy_servers == 0 {
        return None; // No healthy servers available
    }

    // Implementation would analyze the diagnostic range and provide a fix
    // For now, return a simple placeholder edit when servers are available
    let mut edit = WorkspaceEdit::new();
    // Add basic text edit to remove the unused variable
    // This would be more sophisticated in real implementation
    Some(edit)
}

/// Quick fix for dead code
async fn quick_fix_dead_code(
    diagnostic: &Diagnostic,
    params: &CodeActionParams,
    load_metrics: &[ServerLoadMetrics],
) -> Option<WorkspaceEdit> {
    // Check server load to determine if we can perform analysis
    let avg_response_time: f64 = load_metrics.iter()
        .map(|m| m.response_time_ms)
        .sum::<f64>() / load_metrics.len() as f64;

    if avg_response_time > 1000.0 {
        return None; // Servers are too slow for complex analysis
    }

    // Implementation would analyze the dead code and provide a fix
    // For now, return a simple placeholder edit
    let mut edit = WorkspaceEdit::new();
    // Add basic text edit to remove the dead code
    Some(edit)
}

/// Get the current selection range from the parameters
async fn get_selection_range(params: &CodeActionParams, load_metrics: &[ServerLoadMetrics]) -> Option<lsp_types::Range> {
    // Check if we have enough server capacity for selection analysis
    let total_capacity = load_metrics.iter()
        .map(|m| (100.0 - m.cpu_usage_percent) as usize)
        .sum::<usize>();

    if total_capacity < 50 {
        return None; // Insufficient capacity
    }

    // Implementation would get the current selection range
    // For now, return a basic range if servers are available
    Some(lsp_types::Range {
        start: lsp_types::Position { line: 0, character: 0 },
        end: lsp_types::Position { line: 0, character: 10 },
    })
}

/// Get refactoring actions for the given selection
async fn get_refactor_actions(
    params: &CodeActionParams,
    selection: lsp_types::Range,
    load_metrics: &[ServerLoadMetrics],
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // Check server health and load before providing refactoring actions
    let healthy_servers = load_metrics.iter()
        .filter(|m| m.health_score > 0.8)
        .count();

    if healthy_servers < 2 {
        // Return minimal actions when server health is poor
        return actions;
    }

    // Implementation would provide refactoring actions based on the selection
    // For now, return placeholder actions when servers are healthy
    actions
}

/// Check if code actions should be performed based on load metrics
async fn should_perform_code_actions(load_metrics: &[ServerLoadMetrics]) -> bool {
    if load_metrics.is_empty() {
        return false;
    }

    // Check if we have at least one healthy server
    let healthy_servers = load_metrics.iter()
        .filter(|m| m.health_score > 0.6)
        .count();

    // Check average load
    let avg_pending_requests = load_metrics.iter()
        .map(|m| m.pending_requests)
        .sum::<usize>() as f64 / load_metrics.len() as f64;

    healthy_servers > 0 && avg_pending_requests < 10.0
}

/// Create a load-aware action when servers are under heavy load
fn create_load_aware_action() -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Servers are currently under high load - try again later".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: None,
        command: None,
        is_preferred: Some(false),
        disabled: Some(lsp_types::CodeActionDisabled {
            reason: "High server load".to_string(),
        }),
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Diagnostic, Range};
    use std::sync::Arc;
    use crate::pool::LanguageServerPool;

    #[tokio::test]
    async fn test_create_code_action() {
        let diagnostic = Diagnostic {
            range: Range::default(),
            severity: Some(lsp_types::DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String("unused_variables".to_string())),
            code_description: None,
            source: Some("rustc".to_string()),
            message: "unused variable: `x`".to_string(),
            related_information: None,
            tags: None,
            data: None,
        };

        let edit = WorkspaceEdit::default();
        let action = create_code_action(
            "Remove unused variable",
            edit,
            CodeActionKind::QUICKFIX,
            &diagnostic,
        );

        if let CodeActionOrCommand::CodeAction(action) = action {
            assert_eq!(action.title, "Remove unused variable");
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            assert_eq!(action.diagnostics, Some(vec![diagnostic]));
            assert!(action.is_preferred.unwrap());
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[tokio::test]
    async fn test_load_aware_code_actions() {
        // Create a mock pool - in real scenarios this would be properly initialized
        let pool = Arc::new(LanguageServerPool::new());
        let params = CodeActionParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: lsp_types::Url::parse("file:///tmp/test.rs").unwrap(),
            },
            range: Range::default(),
            context: lsp_types::CodeActionContext {
                diagnostics: vec![],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
        };

        let actions = get_code_actions(&params, &pool).await;

        // Should return None or minimal actions based on pool state
        assert!(actions.is_none() || actions.as_ref().unwrap().is_empty());
    }
}
