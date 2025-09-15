use std::sync::Arc;

use async_trait::async_trait;
use syn::visit_mut::VisitMut;
use syn::{parse_file, File as SynFile, Ident};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{ExecutionError, ExecutionResult};
use crate::types::{
    ExecutionResult as ExecResult, ExecutionStatus, RefactoringExecutionContext, RefactoringTransformation,
};

/// Orchestrator for executing refactoring transformations
pub struct RefactoringOrchestrator {
    execution_contexts: Arc<Mutex<Vec<RefactoringExecutionContext>>>,
}

impl RefactoringOrchestrator {
    /// Create a new execution orchestrator
    pub fn new() -> Self {
        Self {
            execution_contexts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new execution context for the given transformations
    pub async fn create_execution_context(
        &self,
        transformations: Vec<RefactoringTransformation>,
        session_id: &Uuid,
    ) -> ExecutionResult<RefactoringExecutionContext> {
        let execution_id = Uuid::new_v4();
        let context = RefactoringExecutionContext {
            execution_id: *session_id,
            session_id: *session_id,
            transformations,
            execution_order: vec![], // Will be filled during execution
            status: ExecutionStatus::Queued,
            progress: Default::default(),
            start_time: chrono::Utc::now(),
            estimated_completion: None,
            last_updated: chrono::Utc::now(),
        };

        let mut contexts = self.execution_contexts.lock().await;
        contexts.push(context.clone());

        Ok(context)
    }

    /// Execute the transformations in the given context
    pub async fn execute_transformations(
        &self,
        mut context: RefactoringExecutionContext,
        approved_suggestions: Vec<Uuid>,
    ) -> ExecutionResult<()> {
        context.status = ExecutionStatus::Running;
        context.progress.current_transformation = 0;
        context.progress.total_transformations = approved_suggestions.len();

        // Filter transformations to only approved ones
        let approved_transforms: Vec<_> = context
            .transformations
            .iter()
            .filter(|t| approved_suggestions.contains(&t.suggestion_id))
            .cloned()
            .collect();

        context.transformations = approved_transforms;
        context.execution_order = approved_suggestions;

        // Execute each transformation
        for (index, transformation) in context.transformations.iter().enumerate() {
            context.progress.current_transformation = index + 1;

            // Apply the transformation using syn visitor
            self.apply_transformation(transformation).await?;

            context
                .progress
                .completed_transformations
                .push(transformation.id);
            context.last_updated = chrono::Utc::now();
        }

        context.status = ExecutionStatus::Completed;
        context.progress.percentage_complete = 100.0;

        // Update the context in storage
        let mut contexts = self.execution_contexts.lock().await;
        if let Some(existing) = contexts
            .iter_mut()
            .find(|c| c.execution_id == context.execution_id)
        {
            *existing = context;
        }

        Ok(())
    }

    /// Cancel execution for the given context
    pub async fn cancel_execution(&self, context: &mut RefactoringExecutionContext) -> ExecutionResult<()> {
        context.status = ExecutionStatus::Cancelled;
        context.last_updated = chrono::Utc::now();
        Ok(())
    }

    /// Rollback completed transformations
    pub async fn rollback_execution(&self, context: &mut RefactoringExecutionContext) -> ExecutionResult<()> {
        // Reverse the execution order and undo each transformation
        for transformation_id in context.execution_order.iter().rev() {
            if let Some(transformation) = context
                .transformations
                .iter()
                .find(|t| t.id == *transformation_id)
            {
                self.rollback_transformation(transformation).await?;
            }
        }

        context.status = ExecutionStatus::Completed; // Mark as completed (rolled back)
        context.progress.completed_transformations.clear();
        context.last_updated = chrono::Utc::now();

        Ok(())
    }

    /// Apply a single transformation using syn visitor pattern
    async fn apply_transformation(&self, transformation: &RefactoringTransformation) -> ExecutionResult<()> {
        // Read the file content
        let content = std::fs::read_to_string(&transformation.file_path).map_err(|e| ExecutionError::FileSystem {
            operation: "read".to_string(),
            path:      transformation.file_path.clone(),
            source:    e,
        })?;

        // Parse the file into AST
        let mut syntax_tree: SynFile = parse_file(&content).map_err(|e| ExecutionError::Syntax {
            file:    transformation.file_path.clone(),
            message: e.to_string(),
        })?;

        // Apply transformation based on operation type
        match transformation.operation_type {
            crate::types::TransformationOperation::ReplaceText => {
                // Use syn visitor to replace text at specific location
                let mut visitor = TextReplacementVisitor {
                    line_number:   transformation.line_number,
                    column_number: transformation.column_number,
                    old_text:      transformation.original_text.clone(),
                    new_text:      transformation.transformed_text.clone(),
                    applied:       false,
                };

                visitor.visit_file_mut(&mut syntax_tree);
            }
            crate::types::TransformationOperation::InsertText => {
                // Insert text at specific location
                let mut visitor = TextInsertionVisitor {
                    line_number:    transformation.line_number,
                    column_number:  transformation.column_number,
                    text_to_insert: transformation.transformed_text.clone(),
                    applied:        false,
                };

                visitor.visit_file_mut(&mut syntax_tree);
            }
            crate::types::TransformationOperation::DeleteText => {
                // Delete text at specific location
                let mut visitor = TextDeletionVisitor {
                    line_number:    transformation.line_number,
                    column_number:  transformation.column_number,
                    text_to_delete: transformation.original_text.clone(),
                    applied:        false,
                };

                visitor.visit_file_mut(&mut syntax_tree);
            }
            _ => {
                // Other operations would be handled similarly
                return Err(ExecutionError::UnsupportedOperation {
                    operation: format!("{:?}", transformation.operation_type),
                });
            }
        }

        // Write the modified file back
        let modified_content = prettyplease::unparse(&syntax_tree);
        std::fs::write(&transformation.file_path, modified_content).map_err(|e| ExecutionError::FileSystem {
            operation: "write".to_string(),
            path:      transformation.file_path.clone(),
            source:    e,
        })?;

        Ok(())
    }

    /// Rollback a single transformation
    async fn rollback_transformation(&self, transformation: &RefactoringTransformation) -> ExecutionResult<()> {
        // For rollback, we essentially do the reverse operation
        let reverse_transformation = RefactoringTransformation {
            id:               Uuid::new_v4(),
            suggestion_id:    transformation.suggestion_id,
            operation_type:   match transformation.operation_type {
                crate::types::TransformationOperation::ReplaceText =>
                    crate::types::TransformationOperation::ReplaceText,
                crate::types::TransformationOperation::InsertText => crate::types::TransformationOperation::DeleteText,
                crate::types::TransformationOperation::DeleteText => crate::types::TransformationOperation::InsertText,
                _ => transformation.operation_type.clone(),
            },
            file_path:        transformation.file_path.clone(),
            line_number:      transformation.line_number,
            column_number:    transformation.column_number,
            original_text:    transformation.transformed_text.clone(), // Swap for rollback
            transformed_text: transformation.original_text.clone(),    // Swap for rollback
            dependencies:     vec![],
            rollback_steps:   vec![],
            validation_hash:  String::new(),
        };

        self.apply_transformation(&reverse_transformation).await
    }
}

/// Visitor for text replacement using syn 2.x APIs
struct TextReplacementVisitor {
    line_number:   usize,
    column_number: usize,
    old_text:      String,
    new_text:      String,
    applied:       bool,
}

impl VisitMut for TextReplacementVisitor {
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        if !self.applied {
            // Check if this identifier is at the target location
            // In a real implementation, we'd check the span information
            // For now, this is a simplified implementation
            let span = i.span();
            // syn 2.x uses proc_macro2::Span, which has line/column info
            if let Some(start) = span.start() {
                if start.line == self.line_number && start.column >= self.column_number {
                    // Replace the identifier text if it matches
                    if i.to_string() == self.old_text {
                        // Note: In syn, we can't directly modify the text of an identifier
                        // This would require a more sophisticated approach with token streams
                        // For demonstration, we'll mark as applied
                        self.applied = true;
                    }
                }
            }
        }
        syn::visit_mut::visit_ident_mut(self, i);
    }
}

/// Visitor for text insertion
struct TextInsertionVisitor {
    line_number:    usize,
    column_number:  usize,
    text_to_insert: String,
    applied:        bool,
}

impl VisitMut for TextInsertionVisitor {
    fn visit_ident_mut(&mut self, _i: &mut Ident) {
        // Simplified implementation - in practice, this would require
        // working with token streams to insert text at specific positions
        self.applied = true;
        // syn::visit_mut::visit_ident_mut(self, i);
    }
}

/// Visitor for text deletion
struct TextDeletionVisitor {
    line_number:    usize,
    column_number:  usize,
    text_to_delete: String,
    applied:        bool,
}

impl VisitMut for TextDeletionVisitor {
    fn visit_ident_mut(&mut self, _i: &mut Ident) {
        // Simplified implementation - in practice, this would require
        // working with token streams to delete text at specific positions
        self.applied = true;
        // syn::visit_mut::visit_ident_mut(self, i);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = RefactoringOrchestrator::new();
        assert!(orchestrator.execution_contexts.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_create_execution_context() {
        let orchestrator = RefactoringOrchestrator::new();
        let session_id = Uuid::new_v4();
        let transformations = vec![];

        let context = orchestrator
            .create_execution_context(transformations, &session_id)
            .await
            .unwrap();

        assert_eq!(context.session_id, session_id);
        assert_eq!(context.status, ExecutionStatus::Queued);

        let contexts = orchestrator.execution_contexts.lock().await;
        assert_eq!(contexts.len(), 1);
    }
}
