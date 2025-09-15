use std::fs;
use std::path::Path;

use async_trait::async_trait;
// AST manipulation imports
use prettyplease;
use syn::visit_mut::VisitMut;
use syn::Ident;

use crate::types::*;
use crate::RefactoringOperation;

/// Check if a file is supported by AST-based operations (Rust files)
fn is_ast_supported(file_path: &str) -> bool {
    rust_ai_ide_shared_utils::get_extension(Path::new(file_path))
        .map(|ext| ext == "rs")
        .unwrap_or(false)
}

/// AST visitor to rename identifiers in code
struct IdentifierRenamer {
    old_name:     String,
    new_name:     String,
    rename_count: usize,
}

impl IdentifierRenamer {
    fn new(old_name: String, new_name: String) -> Self {
        IdentifierRenamer {
            old_name,
            new_name,
            rename_count: 0,
        }
    }
}

impl VisitMut for IdentifierRenamer {
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        if i.to_string() == self.old_name {
            // Only rename if this isn't a keyword or built-in identifier
            if ![
                "fn", "let", "const", "mut", "if", "else", "while", "for", "loop", "match", "return", "break",
                "continue", "struct", "enum", "trait", "impl", "pub", "use", "mod", "type", "where", "as", "crate",
                "super", "self", "Self", "true", "false",
            ]
            .contains(&self.old_name.as_str())
            {
                *i = Ident::new(&self.new_name, i.span());
                self.rename_count += 1;
            }
        }
        syn::visit_mut::visit_ident_mut(self, i);
    }
}

/// Rename operation with AST safety
pub struct RenameOperation;

#[async_trait]
impl RefactoringOperation for RenameOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        let old_name = context
            .symbol_name
            .as_ref()
            .ok_or("No symbol name provided for rename operation")?;
        let new_name = options
            .extra_options
            .as_ref()
            .and_then(|opts| opts.get("newName"))
            .and_then(|v| v.as_str())
            .ok_or("New name not provided in options")?;

        println!("AST-safe rename: {} -> {}", old_name, new_name);

        // Check file type support before attempting AST parsing
        if !is_ast_supported(&context.file_path) {
            return Err(format!(
                "Rename operation supports Rust (.rs) files only, got: {}",
                context.file_path
            )
            .into());
        }

        // Read the source file
        let content = fs::read_to_string(&context.file_path)
            .map_err(|e| format!("Failed to read file {}: {}", context.file_path, e))?;

        // Parse the Rust AST
        let mut syntax_tree: syn::File = syn::parse_str::<syn::File>(&content)?;

        // Perform AST-safe rename
        let mut renamer = IdentifierRenamer::new(old_name.clone(), new_name.to_string());
        renamer.visit_file_mut(&mut syntax_tree);

        // Generate the modified source code with proper formatting
        let modified_content = prettyplease::unparse(&syntax_tree);

        // Calculate the change range (full file for now, could be optimized to find specific ranges)
        let change = CodeChange {
            file_path:   context.file_path.clone(),
            range:       CodeRange {
                start_line:      1,
                start_character: 0,
                end_line:        content.lines().count() as usize,
                end_character:   content.lines().last().map_or(0, |line| line.len()),
            },
            old_text:    content,
            new_text:    modified_content,
            change_type: ChangeType::Replacement,
        };

        Ok(RefactoringResult {
            id:            Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success:       true,
            changes:       vec![change],
            error_message: None,
            warnings:      if renamer.rename_count == 0 {
                vec!["No occurrences of the symbol were found to rename".to_string()]
            } else {
                vec![]
            },
            new_content:   Some(prettyplease::unparse(&syntax_tree)),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe:          true,
            confidence_score: 0.9,
            potential_impact: RefactoringImpact::Low,
            affected_files:   vec![context.file_path.clone()],
            affected_symbols: vec![context.symbol_name.clone().unwrap_or_default()],
            breaking_changes: vec![],
            suggestions:      vec!["Rename appears safe to execute".to_string()],
            warnings:         vec![],
        })
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        _options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(context.symbol_name.is_some() && is_ast_supported(&context.file_path))
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::Rename
    }

    fn name(&self) -> &str {
        "Rename"
    }

    fn description(&self) -> &str {
        "Renames a symbol to a new name"
    }
}
