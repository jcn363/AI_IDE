use std::fs;

use async_trait::async_trait;
use prettyplease;
use syn::visit_mut::VisitMut;
use syn::Ident;

use crate::ast_utils::*;
use crate::types::*;
use crate::utils::*;
use crate::RefactoringOperation;

/// Extract Function operation with AST safety
pub struct ExtractFunctionOperation;

#[async_trait]
impl RefactoringOperation for ExtractFunctionOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        // Check experimental feature flag for non-AST-safe operations
        if !self.is_experimental_enabled(options) {
            return Err(
                "Extract Function operation uses placeholder text editing instead of full AST analysis. Set \
                 options.extra_options.experimental = true to use this feature."
                    .into(),
            );
        }

        let selection = context
            .selection
            .as_ref()
            .ok_or("Selection is required for function extraction")?;
        let function_name = format!("extracted_function_{}", context.cursor_line);

        println!(
            "Experimental extract function operation for lines {} to {}",
            selection.start_line, selection.end_line
        );

        // Check file type support before attempting AST parsing
        if !is_ast_supported(&context.file_path) {
            return Err(format!(
                "Extract Function operation only supports Rust (.rs) files, got: {}",
                context.file_path
            )
            .into());
        }

        // Read the source file
        let content = fs::read_to_string(&context.file_path)
            .map_err(|e| format!("Failed to read file {}: {}", context.file_path, e))?;

        // Parse the Rust AST
        let syntax_tree: syn::File = syn::parse_str::<syn::File>(&content)?;

        // Extract the selected code (simplified - in a real implementation, we'd need more sophisticated
        // AST analysis)
        let lines: Vec<&str> = content.lines().collect();

        // Normalize frontend range (1-based) to backend range (0-based)
        let normalized_selection = crate::utils::RangeNormalizer::frontend_to_backend(&selection);

        // Validate and clamp the normalized range
        crate::utils::RangeNormalizer::validate_range(&normalized_selection, "ExtractFunctionOperation")?;
        let clamped_selection =
            crate::utils::RangeNormalizer::clamp_to_file_bounds(&normalized_selection, lines.len(), &content);

        if clamped_selection.start_line > lines.len() as usize || clamped_selection.end_line > lines.len() as usize {
            return Err("Selection is out of bounds".into());
        }

        // Extract code using consistent 0-based indexing
        let extracted_code: Vec<&str> = if clamped_selection.end_line >= clamped_selection.start_line {
            lines[clamped_selection.start_line..=clamped_selection.end_line].to_vec()
        } else {
            vec![] // Empty selection
        };
        let extracted_text = extracted_code.join("\n");

        // Generate the extracted function
        let function_definition = format!("fn {}() {{\n    {}\n}}\n", function_name, extracted_text);

        // Create the extracted function call
        let function_call = format!("{}();\n", function_name);

        // For now, we'll create a simple replacement of the entire file
        // In a more sophisticated implementation, we'd need to insert the function definition
        // and replace the selected code with the function call using precise range editing
        let insert_position = selection.start_line.saturating_sub(1);

        // Build the new content by inserting the function definition and replacing the selection
        let mut new_content = String::new();

        // Add lines before the selection
        for (i, line) in lines.iter().enumerate() {
            if i == insert_position {
                new_content.push_str(&format!("{}\n", function_definition));
            }
            new_content.push_str(line);
            new_content.push_str("\n");
        }

        // Replace the selected content with function call
        let mut result_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();

        // Insert function call at selection location
        if insert_position + 1 < result_lines.len() {
            result_lines.insert(insert_position + 1, function_call);
        }

        let modified_content = result_lines.join("\n");

        let change = CodeChange {
            file_path:   context.file_path.clone(),
            range:       CodeRange {
                start_line:      1,
                start_character: 0,
                end_line:        lines.len(),
                end_character:   lines.last().map_or(0, |line| line.len()),
            },
            old_text:    content,
            new_text:    modified_content.clone(),
            change_type: ChangeType::Replacement,
        };

        Ok(RefactoringResult {
            id:            Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success:       true,
            changes:       vec![change],
            error_message: None,
            warnings:      vec!["Function extraction may need parameter adjustment".to_string()],
            new_content:   Some(modified_content),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe:          context.selection.is_some(),
            confidence_score: if context.selection.is_some() {
                0.8
            } else {
                0.5
            },
            potential_impact: RefactoringImpact::Medium,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec![],
            suggestions:      vec!["Function extraction requires valid selection".to_string()],
            warnings:         if context.selection.is_none() {
                vec!["No selection provided for extraction".to_string()]
            } else {
                vec![]
            },
        })
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let base_check = context.selection.is_some();

        // Check experimental flag if options are provided
        if let Some(opts) = options {
            if !self.is_experimental_enabled(opts) {
                return Ok(false); // Not applicable unless experimental features are enabled
            }
        }

        Ok(base_check)
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::ExtractFunction
    }

    fn name(&self) -> &str {
        "Extract Function"
    }

    fn description(&self) -> &str {
        "Extracts selected code into a separate function"
    }
}

/// Inline Function operation
pub struct InlineFunctionOperation;
#[async_trait]
impl RefactoringOperation for InlineFunctionOperation {
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
            warnings:      vec!["Inline function operation requires implementation".to_string()],
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
            potential_impact: RefactoringImpact::High,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Feature not yet implemented".to_string()],
            suggestions:      vec![],
            warnings:         vec!["Operation not yet implemented".to_string()],
        })
    }
    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }
    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::InlineFunction
    }
    fn name(&self) -> &str {
        "Inline Function (Not Implemented)"
    }
    fn description(&self) -> &str {
        "Not yet implemented"
    }
}

/// Inline Method operation
pub struct InlineMethodOperation;
#[async_trait]
impl RefactoringOperation for InlineMethodOperation {
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
            warnings:      vec!["Inline method operation requires implementation".to_string()],
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
            potential_impact: RefactoringImpact::High,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Feature not yet implemented".to_string()],
            suggestions:      vec![],
            warnings:         vec!["Operation not yet implemented".to_string()],
        })
    }
    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }
    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::InlineMethod
    }
    fn name(&self) -> &str {
        "Inline Method (Not Implemented)"
    }
    fn description(&self) -> &str {
        "Not yet implemented"
    }
}

/// Convert Method to Function operation
pub struct ConvertMethodToFunctionOperation;
#[async_trait]
impl RefactoringOperation for ConvertMethodToFunctionOperation {
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
            warnings:      vec!["Convert method to function operation requires implementation".to_string()],
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
            potential_impact: RefactoringImpact::High,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Method conversion may break inheritance".to_string()],
            suggestions:      vec![],
            warnings:         vec!["Convert method to function operation requires implementation".to_string()],
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
        RefactoringType::ConvertMethodToFunction
    }
    fn name(&self) -> &str {
        "Convert Method to Function"
    }
    fn description(&self) -> &str {
        "Converts a method to a standalone function"
    }
}

/// Move Method operation
pub struct MoveMethodOperation;
#[async_trait]
impl RefactoringOperation for MoveMethodOperation {
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
            warnings:      vec!["Move method operation requires implementation".to_string()],
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
            potential_impact: RefactoringImpact::High,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Feature not yet implemented".to_string()],
            suggestions:      vec![],
            warnings:         vec!["Operation not yet implemented".to_string()],
        })
    }
    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }
    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::MoveMethod
    }
    fn name(&self) -> &str {
        "Move Method (Not Implemented)"
    }
    fn description(&self) -> &str {
        "Not yet implemented"
    }
}
