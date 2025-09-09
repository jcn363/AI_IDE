//! Code action support for the Rust Language Server

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, Diagnostic, NumberOrString,
    WorkspaceEdit,
};

/// Generate code actions for the given parameters
pub fn get_code_actions(params: &CodeActionParams) -> Option<Vec<CodeActionOrCommand>> {
    let mut actions = Vec::new();
    let context = &params.context;

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
                    if let Some(edit) = quick_fix_unused_variable(diagnostic, params) {
                        actions.push(create_code_action(
                            "Remove unused variable",
                            edit,
                            CodeActionKind::QUICKFIX,
                            diagnostic,
                        ));
                    }
                }
                "dead_code" => {
                    if let Some(edit) = quick_fix_dead_code(diagnostic, params) {
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
    if let Some(selection) = get_selection_range(params) {
        actions.extend(get_refactor_actions(params, selection));
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
fn quick_fix_unused_variable(
    _diagnostic: &Diagnostic,
    _params: &CodeActionParams,
) -> Option<WorkspaceEdit> {
    // Implementation would analyze the diagnostic range and provide a fix
    // For now, return None as this is a placeholder
    None
}

/// Quick fix for dead code
fn quick_fix_dead_code(
    _diagnostic: &Diagnostic,
    _params: &CodeActionParams,
) -> Option<WorkspaceEdit> {
    // Implementation would analyze the dead code and provide a fix
    // For now, return None as this is a placeholder
    None
}

/// Get the current selection range from the parameters
fn get_selection_range(_params: &CodeActionParams) -> Option<lsp_types::Range> {
    // Implementation would get the current selection range
    // For now, return None as this is a placeholder
    None
}

/// Get refactoring actions for the given selection
fn get_refactor_actions(
    _params: &CodeActionParams,
    _selection: lsp_types::Range,
) -> Vec<CodeActionOrCommand> {
    // Implementation would provide refactoring actions based on the selection
    // For now, return an empty vector as this is a placeholder
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Diagnostic, Range};

    #[test]
    fn test_create_code_action() {
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
}
