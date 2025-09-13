use async_trait::async_trait;

use crate::types::*;
use crate::RefactoringOperation;

/// Extract Variable operation - extracts a selected expression into a variable
pub struct ExtractVariableOperation;

/// Inline Variable operation - replaces variable usages with the variable's expression
pub struct InlineVariableOperation;

/// Localize Variable operation - moves a variable to a more local scope
pub struct LocalizeVariableOperation;

#[async_trait]
impl RefactoringOperation for ExtractVariableOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        // Check experimental feature flag for non-AST-safe operation
        if !self.is_experimental_enabled(options) {
            return Err(
                "Extract Variable operation is experimental. Set options.extra_options.experimental = true to use \
                 this feature."
                    .into(),
            );
        }

        println!("Executing extract variable operation (experimental enabled)!");

        let changes = vec![CodeChange {
            file_path:   context.file_path.clone(),
            range:       CodeRange {
                start_line:      context.cursor_line,
                start_character: context.cursor_character,
                end_line:        context.cursor_line,
                end_character:   context.cursor_character + 5,
            },
            old_text:    "expression".to_string(),
            new_text:    "const extractedVariable = expression; extractedVariable".to_string(),
            change_type: ChangeType::Replacement,
        }];

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes,
            error_message: None,
            warnings: vec![],
            new_content: None,
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe:          context.selection.is_some(),
            confidence_score: if context.selection.is_some() {
                0.85
            } else {
                0.6
            },
            potential_impact: RefactoringImpact::Low,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec![],
            suggestions:      vec!["Variable extraction is straightforward".to_string()],
            warnings:         vec![],
        })
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(context.selection.is_some() && context.symbol_kind == Some(SymbolKind::Variable))
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::ExtractVariable
    }

    fn name(&self) -> &str {
        "Extract Variable"
    }

    fn description(&self) -> &str {
        "Extracts a selected expression into a variable"
    }
}

#[async_trait]
impl RefactoringOperation for InlineVariableOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("Executing inline variable operation");

        let changes = vec![CodeChange {
            file_path:   context.file_path.clone(),
            range:       CodeRange {
                start_line:      context.cursor_line,
                start_character: 0,
                end_line:        context.cursor_line + 2,
                end_character:   0,
            },
            old_text:    "const varName = expression;\nusage(varName);\n".to_string(),
            new_text:    "usage(expression);\n".to_string(),
            change_type: ChangeType::Replacement,
        }];

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes,
            error_message: None,
            warnings: vec![],
            new_content: None,
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe:          context.symbol_kind == Some(SymbolKind::Variable),
            confidence_score: 0.75,
            potential_impact: RefactoringImpact::Medium,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![context.symbol_name.clone().unwrap_or_default()],
            breaking_changes: vec!["Variable references will be replaced with expression".to_string()],
            suggestions:      vec!["Ensure expression is side-effect free".to_string()],
            warnings:         vec![],
        })
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        _options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(context.symbol_kind == Some(SymbolKind::Variable))
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::InlineVariable
    }

    fn name(&self) -> &str {
        "Inline Variable"
    }

    fn description(&self) -> &str {
        "Replaces variable usages with the variable's expression"
    }
}

#[async_trait]
impl RefactoringOperation for LocalizeVariableOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id:            Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success:       true,
            changes:       vec![],
            error_message: None,
            warnings:      vec!["Localize variable operation requires implementation".to_string()],
            new_content:   None,
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe:          false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Low,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Variable localization may change scope".to_string()],
            suggestions:      vec![],
            warnings:         vec!["Localize variable operation requires implementation".to_string()],
        })
    }

    async fn is_applicable(
        &self,
        _context: &RefactoringContext,
        _options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::LocalizeVariable
    }

    fn name(&self) -> &str {
        "Localize Variable"
    }

    fn description(&self) -> &str {
        "Moves a variable to a more local scope"
    }
}
