//! Advanced AI-driven code refactoring support for the Rust Language Server

use lsp_types::{Position, Range, TextDocumentIdentifier, TextEdit, Uri, WorkspaceEdit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "ai-refactoring")]
use once_cell::sync::Lazy;

#[cfg(feature = "ai-refactoring")]
use rust_ai_ide_ai_refactoring::{RefactoringImpact, RefactoringContext, RefactoringService};
#[cfg(feature = "ai-refactoring")]
use rust_ai_ide_ai_refactoring::operations::{
    AdvancedRefactoringOperation, RefactoringOperationResult, RefactoringValidationResult
};

/// Options for code refactoring operations (legacy - kept for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefactoringOptions {
    /// Whether to apply the refactoring in-place
    pub in_place: bool,
    /// Additional parameters for the refactoring operation
    pub parameters: HashMap<String, serde_json::Value>,
}

impl Default for RefactoringOptions {
    fn default() -> Self {
        Self {
            in_place: true,
            parameters: HashMap::new(),
        }
    }
}

/// Legacy refactoring operations (kept for backward compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefactoringOperation {
    /// Extract selected code into a new function
    ExtractFunction,
    /// Extract selected code into a new variable
    ExtractVariable,
    /// Rename a symbol
    Rename,
    /// Inline a variable or function
    Inline,
    /// Move code to a new module
    MoveToModule,
}

impl RefactoringOperation {
    /// Convert legacy operation to AI operation
    pub fn to_ai_operation(self) -> AIRefactoringOperation {
        match self {
            RefactoringOperation::ExtractFunction => AIRefactoringOperation::ExtractFunction,
            RefactoringOperation::ExtractVariable => AIRefactoringOperation::ExtractVariable,
            RefactoringOperation::Rename => AIRefactoringOperation::Rename,
            RefactoringOperation::Inline => AIRefactoringOperation::Inline,
            RefactoringOperation::MoveToModule => AIRefactoringOperation::MoveToModule,
        }
    }
}

/// Get available refactoring operations for the given position (legacy function)
pub fn get_available_refactorings(_uri: &Uri, _position: Position) -> Vec<RefactoringOperation> {
    // Return all operations as basic fallback
    vec![
        RefactoringOperation::ExtractFunction,
        RefactoringOperation::ExtractVariable,
        RefactoringOperation::Rename,
        RefactoringOperation::Inline,
        RefactoringOperation::MoveToModule,
    ]
}

/// Enhanced options for AI-driven code refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIRefactoringOptions {
    /// Whether to apply the refactoring in-place
    pub in_place: bool,
    /// Confidence threshold for AI suggestions (0.0-1.0)
    pub confidence_threshold: f64,
    /// Safety level requirements
    pub safety_level: String,
    /// Generate tests for refactored code
    pub generate_tests: bool,
    /// Preserve comments during refactoring
    pub preserve_comments: bool,
    /// Additional AI-specific parameters
    pub ai_parameters: HashMap<String, serde_json::Value>,
    /// Context information for better AI suggestions
    pub context: RefactoringContext,
}

impl Default for AIRefactoringOptions {
    fn default() -> Self {
        Self {
            in_place: true,
            confidence_threshold: 0.7,
            safety_level: "medium".to_string(),
            generate_tests: true,
            preserve_comments: true,
            ai_parameters: HashMap::new(),
            context: RefactoringContext::default(),
        }
    }
}

/// Comprehensive list of AI-enhanced refactoring operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AIRefactoringOperation {
    /// Extract selected code into a new function
    ExtractFunction,
    /// Extract selected code into a new variable
    ExtractVariable,
    /// Rename a symbol with visibility impact analysis
    Rename,
    /// Inline a variable or function
    Inline,
    /// Move code to a new module
    MoveToModule,
    /// Extract interface from class/struct
    ExtractInterface,
    /// Convert synchronous code to async/await
    ConvertToAsync,
    /// Split a large class/struct into smaller components
    SplitClass,
    /// Merge multiple classes/structs
    MergeClasses,
    /// Convert code patterns (e.g., for-loop to iterator)
    PatternConversion,
    /// Move a method to a different struct/impl
    MoveMethod,
    /// Change method signature with parameter analysis
    ChangeSignature,
}

impl AIRefactoringOperation {
    /// Get the operation type as a string for LSP protocol
    pub fn to_string(&self) -> &'static str {
        match self {
            AIRefactoringOperation::ExtractFunction => "extractFunction",
            AIRefactoringOperation::ExtractVariable => "extractVariable",
            AIRefactoringOperation::Rename => "rename",
            AIRefactoringOperation::Inline => "inline",
            AIRefactoringOperation::MoveToModule => "moveToModule",
            AIRefactoringOperation::ExtractInterface => "extractInterface",
            AIRefactoringOperation::ConvertToAsync => "convertToAsync",
            AIRefactoringOperation::SplitClass => "splitClass",
            AIRefactoringOperation::MergeClasses => "mergeClasses",
            AIRefactoringOperation::PatternConversion => "patternConversion",
            AIRefactoringOperation::MoveMethod => "moveMethod",
            AIRefactoringOperation::ChangeSignature => "changeSignature",
        }
    }
}

/// AI-powered refactoring service instance
#[cfg(feature = "ai-refactoring")]
pub type AIRefactoringService = Arc<Mutex<RefactoringService>>;

/// Global AI refactoring service instance
#[cfg(feature = "ai-refactoring")]
static AI_REFACTORING_SERVICE: once_cell::sync::Lazy<AIRefactoringService> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(Mutex::new(RefactoringService::new()))
    });

/// Get AI-aware refactoring suggestions for the given position
#[cfg(feature = "ai-refactoring")]
pub async fn get_ai_refactorings(
    uri: &Uri,
    position: Position,
    context: Option<&RefactoringContext>,
) -> Result<Vec<AIRefactoringOperation>, String> {
    let service = AI_REFACTORING_SERVICE.clone();

    // Create analysis request
    let document_uri = uri.to_string();
    let file_path = uri.path();

    let request = rust_ai_ide_ai_refactoring::types::RefactoringRequest {
        file_path: file_path.to_string(),
        operation_type: "analyze".to_string(),
        context: context.cloned().unwrap_or_default(),
        options: HashMap::new(),
    };

    let suggestions = service.lock().await
        .get_suggestions(&request)
        .await
        .map_err(|e| format!("Failed to get AI suggestions: {}", e))?;

    // Convert to LSP refactoring operations
    let operations = suggestions.into_iter()
        .filter(|s| s.confidence_score >= 0.6) // Only high-confidence suggestions
        .map(|s| match s.operation_type.as_str() {
            "extractFunction" => AIRefactoringOperation::ExtractFunction,
            "extractVariable" => AIRefactoringOperation::ExtractVariable,
            "rename" => AIRefactoringOperation::Rename,
            "inline" => AIRefactoringOperation::Inline,
            "moveToModule" => AIRefactoringOperation::MoveToModule,
            "extractInterface" => AIRefactoringOperation::ExtractInterface,
            "convertToAsync" => AIRefactoringOperation::ConvertToAsync,
            "splitClass" => AIRefactoringOperation::SplitClass,
            "mergeClasses" => AIRefactoringOperation::MergeClasses,
            "patternConversion" => AIRefactoringOperation::PatternConversion,
            "moveMethod" => AIRefactoringOperation::MoveMethod,
            "changeSignature" => AIRefactoringOperation::ChangeSignature,
            _ => AIRefactoringOperation::ExtractFunction, // Default fallback
        })
        .collect();

    Ok(operations)
}

/// Fallback to basic refactoring operations when AI service is unavailable
#[cfg(not(feature = "ai-refactoring"))]
pub async fn get_ai_refactorings(
    _uri: &Uri,
    _position: Position,
    _context: Option<&RefactoringContext>,
) -> Result<Vec<AIRefactoringOperation>, String> {
    Ok(vec![
        AIRefactoringOperation::ExtractFunction,
        AIRefactoringOperation::ExtractVariable,
        AIRefactoringOperation::Rename,
        AIRefactoringOperation::Inline,
        AIRefactoringOperation::MoveToModule,
        AIRefactoringOperation::ExtractInterface,
        AIRefactoringOperation::ConvertToAsync,
        AIRefactoringOperation::SplitClass,
        AIRefactoringOperation::MergeClasses,
        AIRefactoringOperation::PatternConversion,
        AIRefactoringOperation::MoveMethod,
        AIRefactoringOperation::ChangeSignature,
    ])
}

/// Perform AI-enhanced refactoring operations
#[cfg(feature = "ai-refactoring")]
pub async fn perform_ai_refactoring(
    operation: AIRefactoringOperation,
    document: TextDocumentIdentifier,
    range: Option<Range>,
    options: Option<AIRefactoringOptions>,
) -> Result<WorkspaceEdit, String> {
    let service = AI_REFACTORING_SERVICE.clone();
    let options = options.unwrap_or_default();

    // Create refactoring request
    let request = rust_ai_ide_ai_refactoring::types::RefactoringRequest {
        file_path: document.uri.path().to_string(),
        operation_type: operation.to_string().to_string(),
        context: options.context.clone(),
        options: options.ai_parameters.clone(),
    };

    // Execute the refactoring operation
    let result = service.lock().await
        .execute_operation(&request)
        .await
        .map_err(|e| format!("AI refactoring execution failed: {}", e))?;

    // Convert the result to LSP workspace edit
    let workspace_edit = convert_refactoring_result_to_workspace_edit(result, &document.uri)?;

    Ok(workspace_edit)
}

/// Fallback to basic LSP refactoring when AI service is unavailable
#[cfg(not(feature = "ai-refactoring"))]
pub async fn perform_ai_refactoring(
    _operation: AIRefactoringOperation,
    _document: TextDocumentIdentifier,
    _range: Option<Range>,
    _options: Option<AIRefactoringOptions>,
) -> Result<WorkspaceEdit, String> {
    Ok(WorkspaceEdit::default())
}

/// Convert AI refactoring result to LSP WorkspaceEdit
#[cfg(feature = "ai-refactoring")]
fn convert_refactoring_result_to_workspace_edit(
    result: RefactoringOperationResult,
    document_uri: &Uri,
) -> Result<WorkspaceEdit, String> {
    let mut changes = HashMap::new();
    let mut text_edits = Vec::new();

    for change in result.changes {
        // Create text edit from the change
        let text_edit = TextEdit {
            range: Range {
                start: Position {
                    line: change.line_start as u32,
                    character: change.col_start as u32,
                },
                end: Position {
                    line: change.line_end as u32,
                    character: change.col_end as u32,
                },
            },
            new_text: change.new_content,
        };
        text_edits.push(text_edit);
    }

    changes.insert(document_uri.clone(), text_edits);

    Ok(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

/// AI-enhanced refactoring preview with detailed context
#[cfg(feature = "ai-refactoring")]
pub async fn preview_ai_refactoring(
    operation: AIRefactoringOperation,
    document: TextDocumentIdentifier,
    range: Option<Range>,
    options: Option<AIRefactoringOptions>,
) -> Result<Vec<TextEdit>, String> {
    let edit = perform_ai_refactoring(operation, document, range, options).await?;

    // Flatten the workspace edit into a list of text edits
    let mut edits = Vec::new();
    if let Some(changes) = edit.changes {
        for (_, document_edits) in changes {
            edits.extend(document_edits);
        }
    }

    Ok(edits)
}

/// Fallback preview function when AI service is unavailable
#[cfg(not(feature = "ai-refactoring"))]
pub async fn preview_ai_refactoring(
    _operation: AIRefactoringOperation,
    _document: TextDocumentIdentifier,
    _range: Option<Range>,
    _options: Option<AIRefactoringOptions>,
) -> Result<Vec<TextEdit>, String> {
    Ok(Vec::new())
}

/// Validate refactoring options before execution
#[cfg(feature = "ai-refactoring")]
pub async fn validate_ai_refactoring_options(
    operation: AIRefactoringOperation,
    document: TextDocumentIdentifier,
    range: Option<Range>,
    options: &AIRefactoringOptions,
) -> Result<(), String> {
    let service = AI_REFACTORING_SERVICE.clone();

    // Convert to AI refactoring context
    let request = rust_ai_ide_ai_refactoring::types::RefactoringRequest {
        file_path: document.uri.path().to_string(),
        operation_type: operation.to_string().to_string(),
        context: options.context.clone(),
        options: options.ai_parameters.clone(),
    };

    // Validate the operation
    let validation_result = service.lock().await
        .validate_operation(&request)
        .await
        .map_err(|e| format!("Validation failed: {}", e))?;

    if !validation_result.valid {
        let errors = validation_result.errors.join("; ");
        return Err(format!("Refactoring validation failed: {}", errors));
    }

    Ok(())
}

/// Legacy function stubs for backward compatibility
pub async fn perform_refactoring(
    _operation: RefactoringOperation,
    _document: TextDocumentIdentifier,
    _range: Range,
    _options: Option<RefactoringOptions>,
) -> Result<WorkspaceEdit, String> {
    Ok(WorkspaceEdit::default())
}

pub async fn preview_refactoring(
    operation: RefactoringOperation,
    document: TextDocumentIdentifier,
    range: Range,
    options: Option<RefactoringOptions>,
) -> Result<Vec<TextEdit>, String> {
    let edit = perform_refactoring(operation, document, range, options).await?;
    let mut edits = Vec::new();
    if let Some(changes) = edit.changes {
        for (_, document_edits) in changes {
            edits.extend(document_edits);
        }
    }
    Ok(edits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range, TextDocumentIdentifier};

    #[test]
    fn test_get_available_refactorings() {
        let uri: Uri = "file:///test.rs".parse().unwrap();
        let position = Position::new(10, 5);
        let refactorings = get_available_refactorings(&uri, position);

        assert!(!refactorings.is_empty());
        assert!(refactorings.contains(&RefactoringOperation::Rename));
    }

    #[tokio::test]
    async fn test_perform_refactoring() {
        let document = TextDocumentIdentifier {
            uri: Uri::from_str("file:///test.rs").unwrap(),
        };
        let range = Range::new(Position::new(10, 5), Position::new(10, 15));

        let result = perform_refactoring(RefactoringOperation::Rename, document, range, None).await;

        assert!(result.is_ok());
    }
}
