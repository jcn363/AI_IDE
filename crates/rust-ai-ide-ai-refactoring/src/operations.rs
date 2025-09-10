use crate::types::*;
use async_trait::async_trait;
use std::fs;

// AST manipulation imports
use prettyplease;
use syn::{visit_mut::VisitMut, Ident};

/// Check if a file is supported by AST-based operations (Rust files)
fn is_ast_supported(file_path: &str) -> bool {
    use std::path::Path;
    rust_ai_ide_shared_utils::get_extension(Path::new(file_path))
        .map(|ext| ext == "rs")
        .unwrap_or(false)
}

/// AST visitor to rename identifiers in code
struct IdentifierRenamer {
    old_name: String,
    new_name: String,
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
                "fn", "let", "const", "mut", "if", "else", "while", "for", "loop", "match",
                "return", "break", "continue", "struct", "enum", "trait", "impl", "pub", "use",
                "mod", "type", "where", "as", "crate", "super", "self", "Self", "true", "false",
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

/// Core trait for all refactoring operations
#[async_trait]
pub trait RefactoringOperation {
    /// Execute the refactoring operation
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Analyze the refactoring operation before execution
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>>;

    /// Check if this operation is applicable in the given context
    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the type of refactoring this operation implements
    fn refactoring_type(&self) -> RefactoringType;

    /// Get a user-friendly name for this operation
    fn name(&self) -> &str;

    /// Get a description of this operation
    fn description(&self) -> &str;

    /// Check if experimental features are enabled
    fn is_experimental_enabled(&self, options: &RefactoringOptions) -> bool {
        options
            .extra_options
            .as_ref()
            .and_then(|opts| opts.get("experimental"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
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
        let mut syntax_tree: syn::File =
            syn::parse_file(&content).map_err(|e| format!("Failed to parse Rust file: {}", e))?;

        // Perform AST-safe rename
        let mut renamer = IdentifierRenamer::new(old_name.clone(), new_name.to_string());
        renamer.visit_file_mut(&mut syntax_tree);

        // Generate the modified source code with proper formatting
        let modified_content = prettyplease::unparse(&syntax_tree);

        // Calculate the change range (full file for now, could be optimized to find specific ranges)
        let change = CodeChange {
            file_path: context.file_path.clone(),
            range: CodeRange {
                start_line: 1,
                start_character: 0,
                end_line: content.lines().count() as usize,
                end_character: content.lines().last().map_or(0, |line| line.len()),
            },
            old_text: content,
            new_text: modified_content,
            change_type: ChangeType::Replacement,
        };

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![change],
            error_message: None,
            warnings: if renamer.rename_count == 0 {
                vec!["No occurrences of the symbol were found to rename".to_string()]
            } else {
                vec![]
            },
            new_content: Some(prettyplease::unparse(&syntax_tree)),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: true,
            confidence_score: 0.9,
            potential_impact: RefactoringImpact::Low,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![context.symbol_name.clone().unwrap_or_default()],
            breaking_changes: vec![],
            suggestions: vec!["Rename appears safe to execute".to_string()],
            warnings: vec![],
        })
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
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
            return Err("Extract Function operation uses placeholder text editing instead of full AST analysis. Set options.extra_options.experimental = true to use this feature.".into());
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
        let syntax_tree: syn::File =
            syn::parse_file(&content).map_err(|e| format!("Failed to parse Rust file: {}", e))?;

        // Extract the selected code (simplified - in a real implementation, we'd need more sophisticated AST analysis)
        let lines: Vec<&str> = content.lines().collect();

        // Normalize frontend range (1-based) to backend range (0-based)
        let normalized_selection = crate::utils::RangeNormalizer::frontend_to_backend(&selection);

        // Validate and clamp the normalized range
        crate::utils::RangeNormalizer::validate_range(
            &normalized_selection,
            "ExtractFunctionOperation",
        )?;
        let clamped_selection = crate::utils::RangeNormalizer::clamp_to_file_bounds(
            &normalized_selection,
            lines.len(),
            &content,
        );

        if clamped_selection.start_line > lines.len() as usize
            || clamped_selection.end_line > lines.len() as usize
        {
            return Err("Selection is out of bounds".into());
        }

        // Extract code using consistent 0-based indexing
        let extracted_code: Vec<&str> =
            if clamped_selection.end_line >= clamped_selection.start_line {
                lines[clamped_selection.start_line..=clamped_selection.end_line].to_vec()
            } else {
                vec![] // Empty selection
            };
        let extracted_text = extracted_code.join("\n");

        // Generate the extracted function
        let function_definition =
            format!("fn {}() {{\n    {}\n}}\n", function_name, extracted_text);

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
            file_path: context.file_path.clone(),
            range: CodeRange {
                start_line: 1,
                start_character: 0,
                end_line: lines.len(),
                end_character: lines.last().map_or(0, |line| line.len()),
            },
            old_text: content,
            new_text: modified_content.clone(),
            change_type: ChangeType::Replacement,
        };

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![change],
            error_message: None,
            warnings: vec!["Function extraction may need parameter adjustment".to_string()],
            new_content: Some(modified_content),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: context.selection.is_some(),
            confidence_score: if context.selection.is_some() {
                0.8
            } else {
                0.5
            },
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec![],
            suggestions: vec!["Function extraction requires valid selection".to_string()],
            warnings: if context.selection.is_none() {
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

pub struct ExtractVariableOperation;

#[async_trait]
impl RefactoringOperation for ExtractVariableOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        // Check experimental feature flag for non-AST-safe operation
        if !self.is_experimental_enabled(options) {
            return Err("Extract Variable operation is experimental. Set options.extra_options.experimental = true to use this feature.".into());
        }

        println!("Executing extract variable operation (experimental enabled)!");

        let changes = vec![CodeChange {
            file_path: context.file_path.clone(),
            range: CodeRange {
                start_line: context.cursor_line,
                start_character: context.cursor_character,
                end_line: context.cursor_line,
                end_character: context.cursor_character + 5,
            },
            old_text: "expression".to_string(),
            new_text: "const extractedVariable = expression; extractedVariable".to_string(),
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
            is_safe: context.selection.is_some(),
            confidence_score: if context.selection.is_some() {
                0.85
            } else {
                0.6
            },
            potential_impact: RefactoringImpact::Low,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec![],
            suggestions: vec!["Variable extraction is straightforward".to_string()],
            warnings: vec![],
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

/// Inline Variable operation
pub struct InlineVariableOperation;

#[async_trait]
impl RefactoringOperation for InlineVariableOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("Executing inline variable operation");

        let changes = vec![CodeChange {
            file_path: context.file_path.clone(),
            range: CodeRange {
                start_line: context.cursor_line,
                start_character: 0,
                end_line: context.cursor_line + 2,
                end_character: 0,
            },
            old_text: "const varName = expression;\nusage(varName);\n".to_string(),
            new_text: "usage(expression);\n".to_string(),
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
            is_safe: context.symbol_kind == Some(SymbolKind::Variable),
            confidence_score: 0.75,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![context.symbol_name.clone().unwrap_or_default()],
            breaking_changes: vec![
                "Variable references will be replaced with expression".to_string()
            ],
            suggestions: vec!["Ensure expression is side-effect free".to_string()],
            warnings: vec![],
        })
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
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

/// Extract Interface operation - extracts an interface from a class/struct
pub struct ExtractInterfaceOperation;

/// Helper struct for extracted method information
struct ExtractedMethod {
    signature: syn::Signature,
    attrs: Vec<syn::Attribute>,
}

impl ExtractInterfaceOperation {
    /// Parse and extract public methods from a struct's impl blocks
    fn extract_public_methods(
        &self,
        syntax: &syn::File,
        struct_name: &str,
    ) -> Result<Vec<ExtractedMethod>, Box<dyn std::error::Error + Send + Sync>> {
        let mut methods = Vec::new();

        for item in &syntax.items {
            if let syn::Item::Impl(impl_block) = item {
                // Check if this impl block is for our target struct
                if let Some(struct_path) = &impl_block.self_ty {
                    if let syn::Type::Path(type_path) = struct_path {
                        if let Some(last_segment) = type_path.path.segments.last() {
                            if last_segment.ident == struct_name {
                                for impl_item in &impl_block.items {
                                    if let syn::ImplItem::Method(method) = impl_item {
                                        if matches!(method.vis, syn::Visibility::Public(_)) {
                                            methods.push(ExtractedMethod {
                                                signature: method.sig.clone(),
                                                attrs: method.attrs.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(methods)
    }

    /// Generate a proper trait definition
    fn generate_trait_code(
        &self,
        trait_name: &str,
        methods: &[ExtractedMethod],
    ) -> String {
        let mut trait_code = format!("/// Auto-generated trait from struct methods\npub trait {} {{\n", trait_name);

        for method in methods {
            // Add documentation comments
            for attr in &method.attrs {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if nv.path.is_ident("doc") {
                        let attr_str = quote::quote!(#attr).to_string();
                        // Extract the doc comment content
                        if let Some(content) = attr_str.strip_prefix("///").or_else(|| attr_str.strip_prefix("/**")) {
                            let clean_content = content.trim_end_matches("*/").trim();
                            trait_code.push_str(&format!("    {}\n", clean_content));
                        }
                    }
                }
            }

            // Add method signature (remove pub and async from trait methods)
            let sig_str = quote::quote!(#method.signature).to_string();
            let clean_sig = sig_str
                .replace("pub ", "")
                .replace("async ", "");
            trait_code.push_str(&format!("    {};\n", clean_sig));
        }

        trait_code.push_str("}\n");
        trait_code
    }

    /// Generate the implementation for the struct
    fn generate_impl_code(
        &self,
        trait_name: &str,
        struct_name: &str,
        methods: &[ExtractedMethod],
    ) -> String {
        let mut impl_code = format!("/// Auto-generated implementation of {} for {}\nimpl {} for {} {{\n",
            trait_name, struct_name, trait_name, struct_name);

        for method in methods {
            let method_ident = &method.signature.ident;

            // Generate method implementation that delegates to self
            let params: Vec<_> = method.signature.inputs.iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg {
                        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                            Some(pat_ident.ident.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            let params_str = if params.is_empty() {
                String::new()
            } else {
                format!("({})", params.join(", "))
            };

            let async_prefix = if method.signature.asyncness.is_some() {
                "async "
            } else {
                ""
            };

            impl_code.push_str(&format!(
                "    {}fn {} {} {{\n        {}self.{}{ {\n            todo!(\"Implement {} method\")\n        }}\n    }}\n",
                async_prefix,
                method_ident,
                quote::quote!(#(method.signature.inputs),*),
                async_prefix,
                method_ident,
                params_str.strip_prefix("(").unwrap_or(""),
                method_ident
            ));
        }

        impl_code.push_str("}\n");
        impl_code
    }

    /// Validate that the struct has enough methods for interface extraction
    fn validate_extraction(&self, methods: &[ExtractedMethod], struct_name: &str) -> Result<(), String> {
        if methods.is_empty() {
            return Err(format!("Struct '{}' has no public methods to extract into an interface", struct_name));
        }

        if methods.len() < 2 {
            return Err(format!("Interface extraction needs at least 2 public methods. Struct '{}' has only {}",
                struct_name, methods.len()));
        }

        Ok(())
    }
}

#[async_trait]
impl RefactoringOperation for ExtractInterfaceOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("Advanced extract interface operation executing");

        let file_path = &context.file_path;

        // Parse the Rust file
        let content = fs::read_to_string(file_path)?;
        let syntax = syn::parse_file(&content)?;

        // Extract the target struct name
        let struct_name = context.symbol_name.as_deref()
            .ok_or("No symbol name provided for extract interface operation")?;

        // Extract public methods from the struct's impl blocks
        let methods = self.extract_public_methods(&syntax, struct_name)?;

        // Validate extraction prerequisites
        self.validate_extraction(&methods, struct_name)?;

        // Generate trait name (allow custom naming via options)
        let trait_name = options
            .extra_options
            .as_ref()
            .and_then(|opts| opts.get("interfaceName"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| &format!("{}Interface", struct_name));

        // Generate trait definition
        let trait_code = self.generate_trait_code(trait_name, &methods);

        // Generate implementation
        let impl_code = self.generate_impl_code(trait_name, struct_name, &methods);

        // Find the best insertion point for the trait (before the struct)
        let lines: Vec<&str> = content.lines().collect();
        let mut insertion_point = 0;

        for (i, line) in lines.iter().enumerate() {
            if line.contains(&format!("struct {}", struct_name)) ||
               line.contains(&format!("pub struct {}", struct_name)) {
                insertion_point = i;
                break;
            }
        }

        // Build new content by inserting trait before struct
        let mut new_content = String::new();

        // Add lines before insertion point
        for (i, line) in lines.iter().enumerate() {
            if i == insertion_point {
                // Insert trait and a blank line
                new_content.push_str(&format!("{}\n\n", trait_code));
            }
            new_content.push_str(line);
            if i < lines.len() - 1 {
                new_content.push('\n');
            }
        }

        // Add implementation after the original content
        new_content.push_str(&format!("\n\n{}", impl_code));

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![CodeChange {
                file_path: file_path.clone(),
                range: CodeRange {
                    start_line: 1,
                    start_character: 0,
                    end_line: lines.len(),
                    end_character: 0,
                },
                old_text: content,
                new_text: new_content.clone(),
                change_type: ChangeType::Replacement,
            }],
            error_message: None,
            warnings: vec![
                format!("Make sure to update any existing implementations in other files to use the new trait '{}'", trait_name),
                "Review the generated implementation and replace todo!() calls with actual logic".to_string(),
            ],
            new_content: Some(new_content),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: context.symbol_kind == Some(SymbolKind::Struct)
                || context.symbol_kind == Some(SymbolKind::Class),
            confidence_score: 0.7,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![context.symbol_name.clone().unwrap_or_default()],
            breaking_changes: vec!["New interface needs to be implemented".to_string()],
            suggestions: vec!["Implement the interface in the original type".to_string()],
            warnings: vec![],
        })
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(context.symbol_kind == Some(SymbolKind::Struct)
            || context.symbol_kind == Some(SymbolKind::Class))
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::ExtractInterface
    }

    fn name(&self) -> &str {
        "Extract Interface"
    }

    fn description(&self) -> &str {
        "Extracts an interface from a class or struct"
    }
}

/// Convert to Async operation - converts a function to async
pub struct ConvertToAsyncOperation;

/// Analyzed function information for async conversion
struct AsyncConversionInfo {
    function_name: String,
    function_span: proc_macro2::Span,
    is_already_async: bool,
    return_type: Option<syn::Type>,
    calls_awaitable: bool,
    callees: Vec<String>,
}

impl ConvertToAsyncOperation {
    /// Analyze a function to determine if it should be converted to async
    fn analyze_function_for_async(
        &self,
        syntax: &syn::File,
        function_name: &str,
    ) -> Result<AsyncConversionInfo, Box<dyn std::error::Error + Send + Sync>> {
        let mut is_already_async = false;
        let mut return_type = None;
        let mut calls_awaitable = false;
        let mut callees = Vec::new();
        let mut function_span = proc_macro2::Span::call_site();

        for item in &syntax.items {
            if let syn::Item::Fn(item_fn) = item {
                if item_fn.sig.ident == function_name {
                    function_span = item_fn.sig.ident.span();
                    is_already_async = item_fn.sig.asyncness.is_some();
                    return_type = match &item_fn.sig.output {
                        syn::ReturnType::Default => None,
                        syn::ReturnType::Type(_, ty) => Some(*ty.clone()),
                    };

                    // Analyze function body for async operations
                    self.analyze_block_for_async(&item_fn.block, &mut calls_awaitable, &mut callees);

                    break;
                }
            } else if let syn::Item::Impl(impl_block) = item {
                for impl_item in &impl_block.items {
                    if let syn::ImplItem::Method(method) = impl_item {
                        if method.sig.ident == function_name {
                            function_span = method.sig.ident.span();
                            is_already_async = method.sig.asyncness.is_some();
                            return_type = match &method.sig.output {
                                syn::ReturnType::Default => None,
                                syn::ReturnType::Type(_, ty) => Some(*ty.clone()),
                            };

                            self.analyze_block_for_async(&method.block, &mut calls_awaitable, &mut callees);

                            break;
                        }
                    }
                }
            }
        }

        Ok(AsyncConversionInfo {
            function_name: function_name.to_string(),
            function_span,
            is_already_async,
            return_type,
            calls_awaitable,
            callees,
        })
    }

    /// Analyze function body for async operations
    fn analyze_block_for_async(
        &self,
        block: &syn::Block,
        calls_awaitable: &mut bool,
        callees: &mut Vec<String>,
    ) {
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Expr(expr, _) | syn::Stmt::Semi(expr, _) => {
                    self.analyze_expr_for_async(expr, calls_awaitable, callees);
                }
                syn::Stmt::Local(local) => {
                    if let Some(init) = &local.init {
                        self.analyze_expr_for_async(&init.expr, calls_awaitable, callees);
                    }
                }
                _ => {}
            }
        }
    }

    /// Analyze expression for async operations
    fn analyze_expr_for_async(
        &self,
        expr: &syn::Expr,
        calls_awaitable: &mut bool,
        callees: &mut Vec<String>,
    ) {
        match expr {
            syn::Expr::Await(await_expr) => {
                *calls_awaitable = true;
                self.analyze_expr_for_async(&await_expr.base, calls_awaitable, callees);
            }
            syn::Expr::Call(call) => {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(ident) = path.path.get_ident() {
                        callees.push(ident.to_string());
                        // Consider adding heuristics for known async functions
                        if matches!(ident.to_string().as_str(),
                            "tokio" | "async" | "sleep" | "spawn" | "join" | "select") {
                            *calls_awaitable = true;
                        }
                    }
                }
                for arg in &call.args {
                    self.analyze_expr_for_async(arg, calls_awaitable, callees);
                }
            }
            syn::Expr::MethodCall(method_call) => {
                callees.push(method_call.method.to_string());
                self.analyze_expr_for_async(&method_call.receiver, calls_awaitable, callees);
                for arg in &method_call.args {
                    self.analyze_expr_for_async(arg, calls_awaitable, callees);
                }
            }
            syn::Expr::Block(block) => {
                self.analyze_block_for_async(&block.block, calls_awaitable, callees);
            }
            syn::Expr::If(if_expr) => {
                self.analyze_expr_for_async(&if_expr.cond, calls_awaitable, callees);
                self.analyze_block_for_async(&if_expr.then_branch, calls_awaitable, callees);
                for else_branch in &if_expr.else_branch {
                    if let syn::Expr::Block(else_block) = else_branch.1 {
                        self.analyze_block_for_async(&else_block, calls_awaitable, callees);
                    }
                }
            }
            syn::Expr::Loop(loop_expr) => {
                self.analyze_block_for_async(&loop_expr.body, calls_awaitable, callees);
            }
            syn::Expr::Match(match_expr) => {
                self.analyze_expr_for_async(&match_expr.expr, calls_awaitable, callees);
                for arm in &match_expr.arms {
                    self.analyze_expr_for_async(&arm.body, calls_awaitable, callees);
                }
            }
            _ => {}
        }
    }

    /// Generate async function signature
    fn make_async_signature(
        &self,
        existing_sig: &syn::Signature,
        return_type: Option<&syn::Type>,
    ) -> syn::Signature {
        let mut new_sig = existing_sig.clone();
        new_sig.asyncness = Some(syn::token::Async::default());

        // If return type is not already a Future/Result, wrap it
        if let Some(rt) = return_type {
            // Check if already returns a Future
            if !self.is_already_future_type(rt) {
                new_sig.output = syn::ReturnType::Type(
                    syn::token::RArrow::default(),
                    Box::new(syn::parse_quote!(impl std::future::Future<Output = #rt>)),
                );
            }
        } else {
            new_sig.output = syn::ReturnType::Type(
                syn::token::RArrow::default(),
                Box::new(syn::parse_quote!(impl std::future::Future<Output = ()>)),
            );
        }

        new_sig
    }

    /// Check if a type is already a Future
    fn is_already_future_type(&self, ty: &syn::Type) -> bool {
        match ty {
            syn::Type::ImplTrait(impl_trait) => {
                impl_trait.bounds.iter().any(|bound| {
                    if let syn::TypeParamBound::Trait(trait_bound) = bound {
                        trait_bound.path.segments.last().unwrap().ident == "Future"
                    } else {
                        false
                    }
                })
            }
            syn::Type::Path(type_path) => {
                type_path.path.segments.last().unwrap().ident == "Future"
            }
            _ => false,
        }
    }

    /// Validate conversion compatibility
    fn validate_conversion(&self, info: &AsyncConversionInfo) -> Result<(), String> {
        if info.is_already_async {
            return Err(format!("Function '{}' is already async", info.function_name));
        }

        if info.calls_awaitable {
            Ok(())
        } else {
            Err(format!("Function '{}' doesn't appear to use awaitable operations, conversion may not be beneficial", info.function_name))
        }
    }
}

#[async_trait]
impl RefactoringOperation for ConvertToAsyncOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("Advanced convert to async operation executing");

        let file_path = &context.file_path;

        // Read and parse file
        let content = fs::read_to_string(file_path)?;
        let mut syntax: syn::File = syn::parse_file(&content)?;

        // Get target function name
        let function_name = context.symbol_name.as_deref()
            .ok_or_else(|| format!("No function name provided for async conversion"))?;

        // Validate file type
        if !is_ast_supported(file_path) {
            return Err(format!("Async conversion only supports Rust (.rs) files, got: {}", file_path).into());
        }

        // Analyze function for async conversion
        let conversion_info = self.analyze_function_for_async(&syntax, function_name)?;

        // Validate conversion
        self.validate_conversion(&conversion_info)?;

        // Visitor to modify function async-ness
        struct AsyncFunctionVisitor {
            target_name: String,
            should_make_async: bool,
        }

        impl VisitMut for AsyncFunctionVisitor {
            fn visit_item_fn_mut(&mut self, i: &mut syn::ItemFn) {
                if i.sig.ident == self.target_name {
                    println!("Converting function '{}' to async", self.target_name);
                    i.sig.asyncness = Some(syn::token::Async::default());

                    // Update return type if needed
                    if self.should_make_async {
                        match &i.sig.output {
                            syn::ReturnType::Default => {
                                i.sig.output = syn::ReturnType::Type(
                                    syn::token::RArrow::default(),
                                    Box::new(syn::parse_quote!(impl std::future::Future<Output = ()>)),
                                );
                            }
                            syn::ReturnType::Type(_, ty) => {
                                if !matches!(**ty, syn::Type::ImplTrait(_)) {
                                    i.sig.output = syn::ReturnType::Type(
                                        syn::token::RArrow::default(),
                                        Box::new(syn::parse_quote!(impl std::future::Future<Output = #ty>)),
                                    );
                                }
                            }
                        }
                    }
                }
                syn::visit_mut::visit_item_fn_mut(self, i);
            }
        }

        // Apply AST transformation
        let mut visitor = AsyncFunctionVisitor {
            target_name: function_name.to_string(),
            should_make_async: true,
        };
        visitor.visit_file_mut(&mut syntax);

        // Generate modified content
        let modified_content = prettyplease::unparse(&syntax);

        // Find callers that need .await
        let caller_updates = self.find_callers_needing_await(&syntax, function_name);

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![CodeChange {
                file_path: file_path.clone(),
                range: CodeRange {
                    start_line: 1,
                    start_character: 0,
                    end_line: content.lines().count(),
                    end_character: 0,
                },
                old_text: content,
                new_text: modified_content.clone(),
                change_type: ChangeType::Replacement,
            }],
            error_message: None,
            warnings: vec![
                format!("Function '{}' converted to async - update all callers to use .await", function_name),
                "May need to adjust error handling for async context".to_string(),
                "Consider using tokio::spawn for long-running async operations".to_string(),
            ].into_iter()
              .chain(caller_updates.into_iter().map(|caller| format!("Update caller in {} to use .await", caller)))
              .collect(),
            new_content: Some(modified_content),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        // Parse file to analyze function
        let content = fs::read_to_string(&context.file_path)?;
        let syntax: syn::File = syn::parse_file(&content)?;

        let function_name = context.symbol_name.clone().unwrap_or_default();
        let analysis = match self.analyze_function_for_async(&syntax, &function_name) {
            Ok(info) => {
                let mut confidence = 0.8;

                if info.is_already_async {
                    confidence = 0.0; // Can't convert if already async
                } else if info.calls_awaitable {
                    confidence = 0.9; // High confidence if already calling awaitable operations
                }

                RefactoringAnalysis {
                    is_safe: !info.is_already_async,
                    confidence_score: confidence,
                    potential_impact: RefactoringImpact::High,
                    affected_files: vec![context.file_path.clone()],
                    affected_symbols: vec![function_name.clone()],
                    breaking_changes: if info.calls_awaitable {
                        vec![
                        vec![
                            format!("All callers of "{}" must use .await", function_name),
                            "Return type now wrapped in Future<Output = _>".to_string()".to_string(),
                        ]
                    }
                    } else {
                        vec![format!("Callers may need to use .await for "{}"", function_name)]
                    },
                    suggestions: vec![
                        "Ensure all async chains properly handle errors ".to_string(),
                        "Consider using tokio::spawn for CPU-intensive operations ".to_string(),
                        "Review await points for potential deadlocks ".to_string(),
                    ],
                    warnings: if info.calls_awaitable {
                        vec!["High impact - all callers must be updated ".to_string()]
                    } else {
                        vec!["Lower risk - can be gradually migrated ".to_string()]
                    },
                }
            }
            Err(_) => RefactoringAnalysis {
                is_safe: false,
                confidence_score: 0.0,
                potential_impact: RefactoringImpact::High,
                affected_files: vec![context.file_path.clone()],
                affected_symbols: vec![function_name],
                breaking_changes: vec!["Unable to analyze function for async conversion ".to_string()],
                suggestions: vec!["Verify function exists and is convertible ".to_string()],
                warnings: vec!["Analysis failed - conversion may be unsafe ".to_string()],
            }
        };

        Ok(analysis)
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(context.symbol_kind == Some(SymbolKind::Function) && context.symbol_name.is_some())
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::ConvertToAsync
    }

    fn name(&self) -> &str {
        "Convert to Async "
    }

    fn description(&self) -> &str {
        "Converts a synchronous function to async with proper Future return types "
    }
}

impl ConvertToAsyncOperation {
    /// Find functions that call the target function and need .await
    fn find_callers_needing_await(&self, syntax: &syn::File, target_function: &str) -> Vec<String> {
        let mut callers = Vec::new();

        for item in &syntax.items {
            match item {
                syn::Item::Fn(fn_item) => {
                    if self.function_calls_target(&fn_item.block, target_function) {
                        let fn_name = format!("{}", fn_item.sig.ident);
                        callers.push(format!("function \"{}\"", fn_name));
                    }
                }
                syn::Item::Impl(impl_block) => {
                    for impl_item in &impl_block.items {
                        if let syn::ImplItem::Method(method) = impl_item {
                            if self.function_calls_target(&method.block, target_function) {
                                let method_name = format!("{}", method.sig.ident);
                                callers.push(format!("method \"{}\"", method_name));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        callers
    }

    /// Check if a block contains calls to the target function
    fn function_calls_target(&self, block: &syn::Block, target: &str) -> bool {
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Expr(expr, _) | syn::Stmt::Semi(expr, _) => {
                    if self.expr_calls_target(expr, target) {
                        return true;
                    }
                }
                syn::Stmt::Local(local) => {
                    if let Some(init) = &local.init {
                        if self.expr_calls_target(&init.expr, target) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Check if an expression calls the target function
    fn expr_calls_target(&self, expr: &syn::Expr, target: &str) -> bool {
        match expr {
            syn::Expr::Call(call) => {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(ident) = path.path.get_ident() {
                        return ident == target;
                    }
                }
            }
            syn::Expr::MethodCall(_) => {
                // Method calls would be more complex to track
                // For now, we'll skip detailed method call analysis
            }
            syn::Expr::Block(block) => {
                if self.function_calls_target(&block.block, target) {
                    return true;
                }
            }
            syn::Expr::If(if_expr) => {
                if self.expr_calls_target(&if_expr.cond, target) {
                    return true;
                }
                if self.function_calls_target(&if_expr.then_branch, target) {
                    return true;
                }
                for branch in &if_expr.else_branch {
                    if let syn::Expr::Block(else_block) = branch.1 {
                        if self.function_calls_target(&else_block, target) {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }
}

/// Pattern Conversion operation - converts patterns between different styles
pub struct PatternConversionOperation;

/// Detected pattern information
#[derive(Clone)]
struct PatternMatch {
    pattern_type: String,
    start_line: usize,
    end_line: usize,
    confidence: f64,
    category: PatternCategory,
}

/// Categories of detectable patterns
#[derive(Clone, PartialEq)]
enum PatternCategory {
    ControlFlow,
    ErrorHandling,
    CollectionManipulation,
    ResourceManagement,
    FunctionalStyle,
}

/// Conversion options for patterns
#[derive(Clone)]
struct ConversionOption {
    from_pattern: String,
    to_pattern: String,
    benefits: Vec<String>,
    risks: Vec<String>,
    confidence: f64,
}

impl PatternConversionOperation {
    /// Analyze code to identify convertible patterns
    fn detect_patterns(&self, syntax: &syn::File) -> Vec<PatternMatch> {
        let mut patterns = Vec::new();

        for item in &syntax.items {
            match item {
                syn::Item::Fn(fn_item) => {
                    // Analyze function for patterns
                    let func_patterns = self.analyze_function_patterns(&fn_item.block);
                    for mut pattern in func_patterns {
                        pattern.start_line = fn_item.sig.ident.span().start().line;
                        patterns.push(pattern);
                    }
                }
                syn::Item::Impl(impl_block) => {
                    for impl_item in &impl_block.items {
                        if let syn::ImplItem::Method(method) = impl_item {
                            let method_patterns = self.analyze_function_patterns(&method.block);
                            for mut pattern in method_patterns {
                                pattern.start_line = method.sig.ident.span().start().line;
                                patterns.push(pattern);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        patterns
    }

    /// Analyze function body for convertible patterns
    fn analyze_function_patterns(&self, block: &syn::Block) -> Vec<PatternMatch> {
        let mut patterns = Vec::new();

        for (i, stmt) in block.stmts.iter().enumerate() {
            match stmt {
                syn::Stmt::Expr(expr, _) | syn::Stmt::Semi(expr, _) => {
                    if let Some(pattern) = self.detect_pattern_in_expr(expr) {
                        patterns.push(pattern);
                    }
                }
                syn::Stmt::Local(local) => {
                    // Check for initialization patterns
                    if let Some(init) = &local.init {
                        if let Some(pattern) = self.detect_collection_pattern(&init.expr) {
                            patterns.push(pattern);
                        }
                    }
                }
                _ => {}
            }
        }

        patterns
    }

    /// Detect convertible patterns in expressions
    fn detect_pattern_in_expr(&self, expr: &syn::Expr) -> Option<PatternMatch> {
        match expr {
            syn::Expr::If(if_expr) => {
                if self.is_guarded_resource_management(if_expr) {
                    return Some(PatternMatch {
                        pattern_type: r"Guarded Resource Management ".to_string(),
                        start_line: if_expr.if_token.span.start().line,
                        end_line: if_expr.if_token.span.end().line,
                        confidence: 0.7,
                        category: PatternCategory::ResourceManagement,
                    });
                }
            }
            syn::Expr::Call(call) => {
                if let Some(pattern) = self.detect_collection_pattern(&call.func) {
                    return Some(pattern);
                }
            }
            syn::Expr::MethodCall(method_call) => {
                if self.is_map_filter_chain(method_call) {
                    return Some(PatternMatch {
                        pattern_type: r"Functional Chaining ".to_string(),
                        start_line: method_call.dot_token.span.start().line,
                        end_line: method_call.dot_token.span.end().line,
                        confidence: 0.8,
                        category: PatternCategory::FunctionalStyle,
                    });
                }
            }
            syn::Expr::Loop(_) => {
                return Some(PatternMatch {
                    pattern_type: r"Loop Pattern ".to_string(),
                    start_line: 0, // Will be set by caller
                    end_line: 0,
                    confidence: 0.6,
                    category: PatternCategory::CollectionManipulation,
                });
            }
            syn::Expr::Match(match_expr) => {
                if self.is_conditional_dispatch(match_expr) {
                    return Some(PatternMatch {
                        pattern_type: r"Conditional Dispatch ".to_string(),
                        start_line: match_expr.match_token.span.start().line,
                        end_line: match_expr.match_token.span.end().line,
                        confidence: 0.7,
                        category: PatternCategory::ControlFlow,
                    });
                }
            }
            _ => {}
        }
        None
    }

    /// Detect collection manipulation patterns
    fn detect_collection_pattern(&self, expr: &syn::Expr) -> Option<PatternMatch> {
        if let syn::Expr::Call(call) = expr {
            if let syn::Expr::Path(path) = &*call.func {
                if let Some(ident) = path.path.get_ident() {
                    match ident.to_string().as_str() {
                        "vec" | r"Vec::new " | r"HashMap::new " | r"HashSet::new " => {
                            return Some(PatternMatch {
                                pattern_type: r"Collection Construction ".to_string(),
                                start_line: 0,
                                end_line: 0,
                                confidence: 0.8,
                                category: PatternCategory::CollectionManipulation,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Check if expression is a functional map/filter chain
    fn is_map_filter_chain(&self, _method_call: &syn::ExprMethodCall) -> bool {
        // Simplified detection - in production would analyze the chain
        false
    }

    /// Check if expression is guarded resource management
    fn is_guarded_resource_management(&self, _if_expr: &syn::ExprIf) -> bool {
        // Would analyze if the pattern follows resource acquisition is initialization
        false
    }

    /// Check if match is conditional dispatch pattern
    fn is_conditional_dispatch(&self, _match_expr: &syn::ExprMatch) -> bool {
        // Would analyze match patterns for dispatch-like behavior
        false
    }

    /// Get available conversion options for a pattern type
    fn get_conversion_options(&self, pattern_type: &str) -> Vec<ConversionOption> {
        match pattern_type {
            r"Functional Chaining " => vec![
                ConversionOption {
                    from_pattern: r"map().filter().collect() ".to_string(),
                    to_pattern: r"Comprehension-like structure ".to_string(),
                    benefits: vec![
                        r"More readable ".to_string(),
                        r"Potentially more efficient ".to_string(),
                        r"Follows functional paradigm ".to_string(),
                    ],
                    risks: vec![
                        r"May require learning new syntax ".to_string(),
                        r"Different performance characteristics ".to_string(),
                    ],
                    confidence: 0.8,
                },
            ],
            r"Loop Pattern " => vec![
                ConversionOption {
                    from_pattern: r"for loop with collection ".to_string(),
                    to_pattern: r"Iterator methods ".to_string(),
                    benefits: vec![
                        r"More concise ".to_string(),
                        r"Lazy evaluation ".to_string(),
                        r"Better composability ".to_string(),
                    ],
                    risks: vec![
                        r"Different error handling ".to_string(),
                        r"Potentially different allocation patterns ".to_string(),
                    ],
                    confidence: 0.7,
                },
            ],
            r"Conditional Dispatch " => vec![
                ConversionOption {
                    from_pattern: r"match on enum/type ".to_string(),
                    to_pattern: r"Trait objects/dynamic dispatch ".to_string(),
                    benefits: vec![
                        r"Extensibility ".to_string(),
                        r"No need to update match on additions ".to_string(),
                        r"Cleaner separation of concerns ".to_string(),
                    ],
                    risks: vec![
                        r"Runtime overhead ".to_string(),
                        r"Boxing/unboxing ".to_string(),
                        r"May lose exhaustiveness checking ".to_string(),
                    ],
                    confidence: 0.6,
                },
            ],
            _ => vec![
                ConversionOption {
                    from_pattern: pattern_type.to_string(),
                    to_pattern: r"Alternative implementation ".to_string(),
                    benefits: vec![r"May improve readability ".to_string()],
                    risks: vec![r"Unknown conversion impact ".to_string()],
                    confidence: 0.5,
                },
            ],
        }
    }

    /// Apply pattern conversion
    fn apply_conversion(&self, pattern: &PatternMatch, conversion: &ConversionOption, syntax: &syn::File) -> Vec<CodeChange> {
        // For now, return a placeholder change
        // In full implementation, would generate actual AST transformations
        vec![CodeChange {
            file_path: r"/target/file.rs ".to_string(), // Would get from context
            range: CodeRange {
                start_line: pattern.start_line,
                start_character: 0,
                end_line: pattern.end_line,
                end_character: 100, // approximate
            },
            old_text: format!("// Old {} pattern", pattern.pattern_type),
            new_text: format!("// New {} pattern - converted from {}", conversion.to_pattern, conversion.from_pattern),
            change_type: ChangeType::Replacement,
        }]
    }
}

#[async_trait]
impl RefactoringOperation for PatternConversionOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("Advanced pattern conversion operation executing");

        let file_path = &context.file_path;

        // Read and parse file
        let content = fs::read_to_string(file_path)?;
        let syntax = syn::parse_file(&content)?;

        // Detect patterns in the code
        let patterns = self.detect_patterns(&syntax);

        if patterns.is_empty() {
            return Err("No convertible patterns detected in the code".into());
        }

        // Get conversion options for detected patterns
        let mut all_changes = Vec::new();
        let mut warnings = Vec::new();

        for pattern in &patterns {
            let conversions = self.get_conversion_options(&pattern.pattern_type);

            if !conversions.is_empty() {
                // Use the highest confidence conversion
                let best_conversion = conversions.into_iter()
                    .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                    .unwrap();

                if best_conversion.confidence > 0.6 {
                    let changes = self.apply_conversion(&pattern, &best_conversion, &syntax);
                    all_changes.extend(changes);

                    warnings.push(format!(
                        "Converted {} pattern with {:.1}% confidence",
                        pattern.pattern_type, best_conversion.confidence * 100.0
                    ));

                    for benefit in &best_conversion.benefits {
                        warnings.push(format!("Benefit: {}", benefit));
                    }
                }
            }
        }

        if all_changes.is_empty() {
            return Err("No confident conversion options available".into());
        }

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: all_changes,
            error_message: None,
            warnings,
            new_content: Some(content),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(&context.file_path)?;
        let syntax: syn::File = syn::parse_file(&content)?;

        let patterns = self.detect_patterns(&syntax);
        let pattern_count = patterns.len();

        let analysis = RefactoringAnalysis {
            is_safe: pattern_count > 0,
            confidence_score: if pattern_count > 2 { 0.7 } else if pattern_count > 0 { 0.5 } else { 0.0 },
            potential_impact: if pattern_count > 3 {
                RefactoringImpact::High
            } else {
                RefactoringImpact::Medium
            },
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: patterns.iter()
                .filter(|p| p.category == PatternCategory::ControlFlow)
                .map(|p| format!("Control flow pattern '{}' may change semantics", p.pattern_type))
                .collect(),
            suggestions: vec![
                format!("Found {} convertible patterns", pattern_count),
                "Pattern conversions are heuristic-based".to_string(),
                "Review changes for semantic correctness".to_string(),
            ],
            warnings: vec![
                "Pattern conversions may not preserve exact semantics".to_string(),
                "Functional transformations may change performance characteristics".to_string(),
            ],
        };
    };

    Ok(analysis)
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        _options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Pattern conversion is available when selection exists or for general analysis
        Ok(context.selection.is_some() || context.symbol_name.is_some())
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::PatternConversion
    }

    fn name(&self) -> &str {
        "Pattern Conversion"
    }

    fn description(&self) -> &str {
        "Converts code patterns between different programming paradigms and styles"
    }
}

// Placeholder implementations for missing RefactoringType variants

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
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Inline function operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Feature not yet implemented".to_string()],
            suggestions: vec![],
            warnings: vec!["Operation not yet implemented".to_string()],
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
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Inline method operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Feature not yet implemented".to_string()],
            suggestions: vec![],
            warnings: vec!["Operation not yet implemented".to_string()],
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
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Move method operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Feature not yet implemented".to_string()],
            suggestions: vec![],
            warnings: vec!["Operation not yet implemented".to_string()],
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

// Implementation for MoveClassOperation
pub struct MoveClassOperation;
#[async_trait]
impl RefactoringOperation for MoveClassOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Move class operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Class move may break imports".to_string()],
            suggestions: vec![],
            warnings: vec!["Move class operation requires implementation".to_string()],
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
        RefactoringType::MoveClass
    }
    fn name(&self) -> &str {
        "Move Class"
    }
    fn description(&self) -> &str {
        "Moves a class to a different file or location"
    }
}

// Implementation for MoveFileOperation
pub struct MoveFileOperation;
#[async_trait]
impl RefactoringOperation for MoveFileOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Move file operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["File move may break imports".to_string()],
            suggestions: vec![],
            warnings: vec!["Move file operation requires implementation".to_string()],
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
        RefactoringType::MoveFile
    }
    fn name(&self) -> &str {
        "Move File"
    }
    fn description(&self) -> &str {
        "Moves a file to a different location"
    }
}

// Implementation for RemoveParameterOperation
pub struct RemoveParameterOperation;
#[async_trait]
impl RefactoringOperation for RemoveParameterOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Remove parameter operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Removing parameter may break callers".to_string()],
            suggestions: vec![],
            warnings: vec!["Remove parameter operation requires implementation".to_string()],
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
        RefactoringType::RemoveParameter
    }
    fn name(&self) -> &str {
        "Remove Parameter"
    }
    fn description(&self) -> &str {
        "Removes a parameter from a function"
    }
}

// Implementation for IntroduceParameterOperation
pub struct IntroduceParameterOperation;
#[async_trait]
impl RefactoringOperation for IntroduceParameterOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Introduce parameter operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Adding parameter may affect callers".to_string()],
            suggestions: vec![],
            warnings: vec!["Introduce parameter operation requires implementation".to_string()],
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
        RefactoringType::IntroduceParameter
    }
    fn name(&self) -> &str {
        "Introduce Parameter"
    }
    fn description(&self) -> &str {
        "Introduces a new parameter to a function"
    }
}

// Implementation for ReplaceConstructorOperation
pub struct ReplaceConstructorOperation;
#[async_trait]
impl RefactoringOperation for ReplaceConstructorOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Replace constructor operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Constructor replacement may break object creation".to_string()],
            suggestions: vec![],
            warnings: vec!["Replace constructor operation requires implementation".to_string()],
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
        RefactoringType::ReplaceConstructor
    }
    fn name(&self) -> &str {
        "Replace Constructor"
    }
    fn description(&self) -> &str {
        "Replaces a constructor with a different implementation"
    }
}
// Implementation for ReplaceConditionalsOperation
pub struct ReplaceConditionalsOperation;
#[async_trait]
impl RefactoringOperation for ReplaceConditionalsOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Replace conditionals operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Conditional replacement may change logic".to_string()],
            suggestions: vec![],
            warnings: vec!["Replace conditionals operation requires implementation".to_string()],
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
        RefactoringType::ReplaceConditionals
    }
    fn name(&self) -> &str {
        "Replace Conditionals"
    }
    fn description(&self) -> &str {
        "Replaces conditional statements with different constructs"
    }
}

// Implementation for ConvertMethodToFunctionOperation
pub struct ConvertMethodToFunctionOperation;
#[async_trait]
impl RefactoringOperation for ConvertMethodToFunctionOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec![
                "Convert method to function operation requires implementation".to_string(),
            ],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Method conversion may break inheritance".to_string()],
            suggestions: vec![],
            warnings: vec![
                "Convert method to function operation requires implementation".to_string(),
            ],
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

// Implementation for SplitClassOperation
pub struct SplitClassOperation;

/// Analyzed field information for splitting
#[derive(Clone)]
struct FieldInfo {
    name: String,
    ty: syn::Type,
    visibility: syn::Visibility,
    methods_using: Vec<String>,
}

/// Method information for splitting
#[derive(Clone)]
struct MethodInfo {
    name: String,
    signature: syn::Signature,
    fields_used: Vec<String>,
    visibility: syn::Visibility,
}

/// Suggested split configuration
#[derive(Clone)]
struct SplitConfiguration {
    class_name: String,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
    dependencies: Vec<String>,
}

impl SplitClassOperation {
    /// Analyze struct fields and their usage patterns
    fn analyze_struct_patterns(
        &self,
        syntax: &syn::File,
        struct_name: &str,
    ) -> Result<Vec<FieldInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut fields = Vec::new();

        for item in &syntax.items {
            if let syn::Item::Struct(struct_item) = item {
                if struct_item.ident == struct_name {
                    for field in &struct_item.fields {
                        if let syn::Fields::Named(named_fields) = &field.named {
                            for field in named_fields.named {
                                let field_name = field.ident.as_ref().unwrap().to_string();
                                let mut methods_using = Vec::new();

                                // Find methods that use this field
                                for item in &syntax.items {
                                    if let syn::Item::Impl(impl_block) = item {
                                        for impl_item in &impl_block.items {
                                            if let syn::ImplItem::Method(method) = impl_item {
                                                if self.method_uses_field(&method.block, &field_name) {
                                                    methods_using.push(method.sig.ident.to_string());
                                                }
                                            }
                                        }
                                    }
                                }

                                fields.push(FieldInfo {
                                    name: field_name,
                                    ty: field.ty.clone(),
                                    visibility: field.vis.clone(),
                                    methods_using,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(fields)
    }

    /// Analyze methods and their field dependencies
    fn analyze_method_patterns(
        &self,
        syntax: &syn::File,
        struct_name: &str,
    ) -> Result<Vec<MethodInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut methods = Vec::new();

        for item in &syntax.items {
            if let syn::Item::Impl(impl_block) = item {
                for impl_item in &impl_block.items {
                    if let syn::ImplItem::Method(method) = impl_item {
                        let mut fields_used = Vec::new();

                        // Check which fields this method uses
                        for field in &self.analyze_struct_patterns(syntax, struct_name)? {
                            if self.method_uses_field(&method.block, &field.name) {
                                fields_used.push(field.name.clone());
                            }
                        }

                        methods.push(MethodInfo {
                            name: method.sig.ident.to_string(),
                            signature: method.sig.clone(),
                            fields_used,
                            visibility: syn::Visibility::Public(syn::token::Pub::default()),
                        });
                    }
                }
            }
        }

        Ok(methods)
    }

    /// Check if a method uses a specific field
    fn method_uses_field(&self, block: &syn::Block, field_name: &str) -> bool {
        let mut visitor = FieldUsageVisitor {
            field_name: field_name.to_string(),
            found: false,
        };
        syn::visit::visit_block(&mut visitor, block);
        visitor.found
    }

    /// Cluster fields and methods into logical groups
    fn suggest_split_configurations(
        &self,
        fields: &[FieldInfo],
        methods: &[MethodInfo],
    ) -> Vec<SplitConfiguration> {
        // Simple clustering based on field usage in methods
        let mut field_groups = std::collections::HashMap::new();

        for field in fields {
            let methods_count = field.methods_using.len();
            let key = if methods_count == 1 {
                format!("{}_specific", field.methods_using.get(0).unwrap_or(&"private".to_string()))
            } else if methods_count > 3 {
                "core".to_string()
            } else {
                "shared".to_string()
            };

            field_groups.entry(key).or_insert_with(Vec::new).push(field.clone());
        }

        // Create split configurations from groups
        field_groups.into_iter()
            .filter(|(_, group_fields)| group_fields.len() >= 2) // Only split if group has 2+ fields
            .enumerate()
            .map(|(i, (group_name, group_fields))| {
                let class_name = if group_name == "core" {
                    format!("{}Core", group_name)
                } else {
                    format!("{}{}", group_name, i + 1)
                };

                let associated_methods = methods.iter()
                    .filter(|method| {
                        method.fields_used.iter()
                            .any(|field| group_fields.iter().any(|gf| gf.name == *field))
                    })
                    .cloned()
                    .collect();

                SplitConfiguration {
                    class_name,
                    fields: group_fields,
                    methods: associated_methods,
                    dependencies: vec![], // Will be populated later
                }
            })
            .collect()
    }

    /// Check if splitting would be beneficial
    fn validate_split_benefits(&self, fields: &[FieldInfo], _methods: &[MethodInfo]) -> Result<(), String> {
        if fields.len() < 4 {
            return Err(format!("Struct has only {} fields, splitting may not provide benefits", fields.len()));
        }

        // Check for clear separation of concerns
        let mut core_fields = Vec::new();
        let mut specific_fields = Vec::new();

        for field in fields {
            if field.methods_using.len() > 3 {
                core_fields.push(field.clone());
            } else if field.methods_using.len() == 1 {
                specific_fields.push(field.clone());
            }
        }

        if core_fields.len() < 2 && specific_fields.len() < 4 {
            return Err("Insufficient field separation to warrant splitting".to_string());
        }

        Ok(())
    }
}

#[async_trait]
impl RefactoringOperation for SplitClassOperation {
    async fn execute(
        &self,
        context: &RefactoringContext,
        options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("Advanced split class operation executing");

        let file_path = &context.file_path;

        // Read and parse source file
        let content = fs::read_to_string(file_path)?;
        let syntax: syn::File = syn::parse_file(&content)?;

        let struct_name = context.symbol_name.as_deref()
            .ok_or("No struct name provided for splitting operation")?;

        // Analyze current structure
        let fields = self.analyze_struct_patterns(&syntax, struct_name)?;
        let methods = self.analyze_method_patterns(&syntax, struct_name)?;

        // Validate benefits of splitting
        self.validate_split_benefits(&fields, &methods)?;

        // Generate split configurations
        let split_configs = self.suggest_split_configurations(&fields, &methods);

        if split_configs.is_empty() {
            return Err("No suitable split configurations found".into());
        }

        // Generate refactored code
        let mut new_content = content.clone();
        let mut changes = Vec::new();

        // Create new structs
        for (i, config) in split_configs.iter().enumerate() {
            let struct_code = self.generate_split_struct(&config, i)?;

            // Find insertion point (after original struct)
            let lines: Vec<&str> = new_content.lines().collect();
            let mut insert_after = 0;

            for (j, line) in lines.iter().enumerate() {
                if line.contains("}") && insert_after == 0 && j > 0 && lines[j-1].contains("struct") {
                    insert_after = j + 1;
                    break;
                }
            }

            // Insert new struct definition
            let mut result_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
            result_lines.insert(insert_after, format!("{}\n", struct_code));
            new_content = result_lines.join("\n");

            changes.push(CodeChange {
                file_path: file_path.clone(),
                range: CodeRange {
                    start_line: 1,
                    start_character: 0,
                    end_line: content.lines().count(),
                    end_character: 0,
                },
                old_text: content.clone(),
                new_text: new_content.clone(),
                change_type: ChangeType::Replacement,
            });
        }

        // Update original struct to use composition
        let composition_code = self.generate_composition_code(&split_configs, struct_name)?;
        if !composition_code.is_empty() {
            new_content = new_content.replace("}", &composition_code);
        }

        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes,
            error_message: None,
            warnings: vec![
                format!("Split {} into {} separate structs", struct_name, split_configs.len()),
                "Original struct now uses composition pattern - verify all field accesses".to_string(),
                "Consider updating trait implementations to delegate to split structs".to_string(),
                "Check for any circular dependencies between split structs".to_string(),
            ],
            new_content: Some(new_content),
        })
    }

    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(&context.file_path)?;
        let syntax: syn::File = syn::parse_file(&content)?;

        let struct_name = context.symbol_name.clone().unwrap_or_default();

        let analysis = match (self.analyze_struct_patterns(&syntax, &struct_name),
                              self.analyze_method_patterns(&syntax, &struct_name)) {
            (Ok(fields), Ok(methods)) => {
                let confidence = if fields.len() >= 6 && methods.len() >= 4 { 0.8 } else { 0.6 };

                RefactoringAnalysis {
                    is_safe: confidence > 0.7,
                    confidence_score: confidence,
                    potential_impact: RefactoringImpact::High,
                    affected_files: vec![context.file_path.clone()],
                    affected_symbols: vec![struct_name.clone()],
                    breaking_changes: vec![
                        format!("{} will be split into multiple structs with composition", struct_name),
                        "Public interface may change - methods remain but internal structure changes".to_string(),
                        "Field access patterns will require composition traversal".to_string(),
                    ],
                    suggestions: vec![
                        "Consider creating traits for each split structure".to_string(),
                        "Use composition over inheritance for new code".to_string(),
                        "Update tests to work with the new structure".to_string(),
                        "Document the new architectural boundaries".to_string(),
                    ],
                    warnings: if confidence < 0.7 {
                        vec!["Split may not provide significant benefits for this struct".to_string()]
                    } else {
                        vec![]
                    },
                }
            }
            _ => RefactoringAnalysis {
                is_safe: false,
                confidence_score: 0.0,
                potential_impact: RefactoringImpact::High,
                affected_files: vec![context.file_path.clone()],
                affected_symbols: vec![struct_name],
                breaking_changes: vec!["Unable to analyze struct for splitting".to_string()],
                suggestions: vec!["Verify struct exists and has sufficient complexity for splitting".to_string()],
                warnings: vec!["Analysis failed".to_string()],
            }
        };

        Ok(analysis)
    }

    async fn is_applicable(
        &self,
        context: &RefactoringContext,
        _options: Option<&RefactoringOptions>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(context.symbol_kind == Some(SymbolKind::Struct) && context.symbol_name.is_some())
    }

    fn refactoring_type(&self) -> RefactoringType {
        RefactoringType::SplitClass
    }

    fn name(&self) -> &str {
        "Split Class"
    }

    fn description(&self) -> &str {
        "Splits a large class into multiple smaller classes using composition"
    }
}

impl SplitClassOperation {
    /// Generate code for a split struct
    fn generate_split_struct(
        &self,
        config: &SplitConfiguration,
        index: usize,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut struct_code = String::new();

        // Add struct definition
        struct_code.push_str(&format!("/// Split struct {} containing specific functionality\n", config.class_name));
        struct_code.push_str(&format!("pub struct {} {{\n", config.class_name));

        for field in &config.fields {
            struct_code.push_str(&format!("    pub {}: {},\n", field.name, quote::quote!(#field.ty)));
        }

        struct_code.push_str("}\n\n");

        // Add impl block
        struct_code.push_str(&format!("impl {} {{\n", config.class_name));

        for method in &config.methods {
            struct_code.push_str(&format!(
                "    {}\n        todo!(\"Implement {} method\")\n    }}\n",
                quote::quote!(#method.signature),
                method.name
            ));
        }

        struct_code.push_str("}\n");

        Ok(struct_code)
    }

    /// Generate composition code for the original struct
    fn generate_composition_code(
        &self,
        configs: &[SplitConfiguration],
        original_struct_name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut composition_code = String::new();

        composition_code.push_str("    // Split struct composition\n");

        for config in configs {
            composition_code.push_str(&format!("    {}: {},\n",
                config.class_name.to_lowercase(),
                config.class_name
            ));
        }

        composition_code.push_str("}\n");

        // Generate delegation methods
        for config in configs {
            for method in &config.methods {
                composition_code.push_str(&format!(
                    "\nimpl {} {{\n    {} {{\n        self.{}.{}\n    }}\n}}\n",
                    original_struct_name,
                    quote::quote!(#method.signature),
                    config.class_name.to_lowercase(),
                    method.name
                ));
            }
        }

        Ok(composition_code)
    }
}

/// Visitor to detect field usage in expressions
struct FieldUsageVisitor {
    field_name: String,
    found: bool,
}

impl syn::visit::Visit<'_> for FieldUsageVisitor {
    fn visit_expr_field(&mut self, i: &syn::ExprField) {
        if let syn::Expr::Path(path_expr) = &*i.base {
            if let Some(ident) = path_expr.path.get_ident() {
                if ident == "self" && i.member.to_string() == self.field_name {
                    self.found = true;
                }
            }
        }
        syn::visit::visit_expr_field(self, i);
    }
}

// Implementation for MergeClassesOperation
pub struct MergeClassesOperation;
#[async_trait]
impl RefactoringOperation for MergeClassesOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Merge classes operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Class merging changes interfaces".to_string()],
            suggestions: vec![],
            warnings: vec!["Merge classes operation requires implementation".to_string()],
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
        RefactoringType::MergeClasses
    }
    fn name(&self) -> &str {
        "Merge Classes"
    }
    fn description(&self) -> &str {
        "Merges multiple classes into one"
    }
}

// Implementation for ChangeSignatureOperation
pub struct ChangeSignatureOperation;
#[async_trait]
impl RefactoringOperation for ChangeSignatureOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Change signature operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Signature change may break callers".to_string()],
            suggestions: vec![],
            warnings: vec!["Change signature operation requires implementation".to_string()],
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
        RefactoringType::ChangeSignature
    }
    fn name(&self) -> &str {
        "Change Signature"
    }
    fn description(&self) -> &str {
        "Changes the signature of a function or method"
    }
}
// Implementation for AddDelegationOperation
pub struct AddDelegationOperation;
#[async_trait]
impl RefactoringOperation for AddDelegationOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Add delegation operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Delegation may change class behavior".to_string()],
            suggestions: vec![],
            warnings: vec!["Add delegation operation requires implementation".to_string()],
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
        RefactoringType::AddDelegation
    }
    fn name(&self) -> &str {
        "Add Delegation"
    }
    fn description(&self) -> &str {
        "Adds delegation to a class"
    }
}

// Implementation for RemoveDelegationOperation
pub struct RemoveDelegationOperation;
#[async_trait]
impl RefactoringOperation for RemoveDelegationOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Remove delegation operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Removing delegation may break dependencies".to_string()],
            suggestions: vec![],
            warnings: vec!["Remove delegation operation requires implementation".to_string()],
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
        RefactoringType::RemoveDelegation
    }
    fn name(&self) -> &str {
        "Remove Delegation"
    }
    fn description(&self) -> &str {
        "Removes delegation from a class"
    }
}

// Implementation for EncapsulateFieldOperation
pub struct EncapsulateFieldOperation;
#[async_trait]
impl RefactoringOperation for EncapsulateFieldOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Encapsulate field operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Field encapsulation changes access pattern".to_string()],
            suggestions: vec![],
            warnings: vec!["Encapsulate field operation requires implementation".to_string()],
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
        RefactoringType::EncapsulateField
    }
    fn name(&self) -> &str {
        "Encapsulate Field"
    }
    fn description(&self) -> &str {
        "Encapsulates a field with getter/setter methods"
    }
}

// Implementation for LocalizeVariableOperation
pub struct LocalizeVariableOperation;
#[async_trait]
impl RefactoringOperation for LocalizeVariableOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Localize variable operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Low,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Variable localization may change scope".to_string()],
            suggestions: vec![],
            warnings: vec!["Localize variable operation requires implementation".to_string()],
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
// Implementation for AddMissingImportsOperation
pub struct AddMissingImportsOperation;
#[async_trait]
impl RefactoringOperation for AddMissingImportsOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Add missing imports operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: true,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Low,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec![],
            suggestions: vec![],
            warnings: vec!["Add missing imports operation requires implementation".to_string()],
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
        RefactoringType::AddMissingImports
    }
    fn name(&self) -> &str {
        "Add Missing Imports"
    }
    fn description(&self) -> &str {
        "Adds missing import statements"
    }
}

// Implementation for SortImportsOperation
pub struct SortImportsOperation;
#[async_trait]
impl RefactoringOperation for SortImportsOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Sort imports operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: true,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Low,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec![],
            suggestions: vec![],
            warnings: vec!["Sort imports operation requires implementation".to_string()],
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
        RefactoringType::SortImports
    }
    fn name(&self) -> &str {
        "Sort Imports"
    }
    fn description(&self) -> &str {
        "Sorts import statements"
    }
}

// Implementation for GenerateGettersSettersOperation
pub struct GenerateGettersSettersOperation;
#[async_trait]
impl RefactoringOperation for GenerateGettersSettersOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Generate getters/setters operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: true,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Low,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec![],
            suggestions: vec![],
            warnings: vec!["Generate getters/setters operation requires implementation".to_string()],
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
        RefactoringType::GenerateGettersSetters
    }
    fn name(&self) -> &str {
        "Generate Getters/Setters"
    }
    fn description(&self) -> &str {
        "Generates getter and setter methods for fields"
    }
}

// Implementation for BatchInterfaceExtractionOperation
pub struct BatchInterfaceExtractionOperation;
#[async_trait]
impl RefactoringOperation for BatchInterfaceExtractionOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec![
                "Batch interface extraction operation requires implementation".to_string(),
            ],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Batch operations may affect multiple files".to_string()],
            suggestions: vec![],
            warnings: vec![
                "Batch interface extraction operation requires implementation".to_string(),
            ],
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
        RefactoringType::BatchInterfaceExtraction
    }
    fn name(&self) -> &str {
        "Batch Interface Extraction"
    }
    fn description(&self) -> &str {
        "Extracts interfaces from multiple classes"
    }
}

// Implementation for BatchPatternConversionOperation
pub struct BatchPatternConversionOperation;
#[async_trait]
impl RefactoringOperation for BatchPatternConversionOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Batch pattern conversion operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::Medium,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Batch operations may affect multiple files".to_string()],
            suggestions: vec![],
            warnings: vec!["Batch pattern conversion operation requires implementation".to_string()],
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
        RefactoringType::BatchPatternConversion
    }
    fn name(&self) -> &str {
        "Batch Pattern Conversion"
    }
    fn description(&self) -> &str {
        "Converts patterns across multiple files"
    }
}

// Implementation for ExtractClassOperation
pub struct ExtractClassOperation;
#[async_trait]
impl RefactoringOperation for ExtractClassOperation {
    async fn execute(
        &self,
        _context: &RefactoringContext,
        _options: &RefactoringOptions,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringResult {
            id: Some(crate::utils::RefactoringUtils::generate_refactoring_id()),
            success: true,
            changes: vec![],
            error_message: None,
            warnings: vec!["Extract class operation requires implementation".to_string()],
            new_content: None,
        })
    }
    async fn analyze(
        &self,
        context: &RefactoringContext,
    ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        Ok(RefactoringAnalysis {
            is_safe: false,
            confidence_score: 0.0,
            potential_impact: RefactoringImpact::High,
            affected_files: vec![context.file_path.clone()],
            affected_symbols: vec![],
            breaking_changes: vec!["Class extraction changes class structure".to_string()],
            suggestions: vec![],
            warnings: vec!["Extract class operation requires implementation".to_string()],
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
        RefactoringType::ExtractClass
    }
    fn name(&self) -> &str {
        "Extract Class"
    }
    fn description(&self) -> &str {
        "Extracts a class from existing code"
    }
}

/// Factory for creating refactoring operations
pub struct RefactoringOperationFactory;

impl RefactoringOperationFactory {
    /// Create an operation instance for the given refactoring type
    pub fn create_operation(
        refactoring_type: &RefactoringType,
    ) -> Result<Box<dyn RefactoringOperation>, Box<dyn std::error::Error + Send + Sync>> {
        match refactoring_type {
            RefactoringType::Rename => Ok(Box::new(RenameOperation)),
            RefactoringType::ExtractFunction => Ok(Box::new(ExtractFunctionOperation)),
            RefactoringType::ExtractVariable => Ok(Box::new(ExtractVariableOperation)),
            RefactoringType::InlineVariable => Ok(Box::new(InlineVariableOperation)),
            RefactoringType::InlineFunction => Ok(Box::new(InlineFunctionOperation)),
            RefactoringType::InlineMethod => Ok(Box::new(InlineMethodOperation)),
            RefactoringType::ExtractInterface => Ok(Box::new(ExtractInterfaceOperation)),
            RefactoringType::ConvertToAsync => Ok(Box::new(ConvertToAsyncOperation)),
            RefactoringType::PatternConversion => Ok(Box::new(PatternConversionOperation)),
            RefactoringType::MoveMethod => Ok(Box::new(MoveMethodOperation)),
            RefactoringType::MoveClass => Ok(Box::new(MoveClassOperation)),
            RefactoringType::MoveFile => Ok(Box::new(MoveFileOperation)),
            RefactoringType::RemoveParameter => Ok(Box::new(RemoveParameterOperation)),
            RefactoringType::IntroduceParameter => Ok(Box::new(IntroduceParameterOperation)),
            RefactoringType::ReplaceConstructor => Ok(Box::new(ReplaceConstructorOperation)),
            RefactoringType::ReplaceConditionals => Ok(Box::new(ReplaceConditionalsOperation)),
            RefactoringType::ConvertMethodToFunction => {
                Ok(Box::new(ConvertMethodToFunctionOperation))
            }
            RefactoringType::SplitClass => Ok(Box::new(SplitClassOperation)),
            RefactoringType::MergeClasses => Ok(Box::new(MergeClassesOperation)),
            RefactoringType::ChangeSignature => Ok(Box::new(ChangeSignatureOperation)),
            RefactoringType::AddDelegation => Ok(Box::new(AddDelegationOperation)),
            RefactoringType::RemoveDelegation => Ok(Box::new(RemoveDelegationOperation)),
            RefactoringType::EncapsulateField => Ok(Box::new(EncapsulateFieldOperation)),
            RefactoringType::LocalizeVariable => Ok(Box::new(LocalizeVariableOperation)),
            RefactoringType::AddMissingImports => Ok(Box::new(AddMissingImportsOperation)),
            RefactoringType::SortImports => Ok(Box::new(SortImportsOperation)),
            RefactoringType::GenerateGettersSetters => {
                Ok(Box::new(GenerateGettersSettersOperation))
            }
            RefactoringType::ExtractClass => Ok(Box::new(ExtractClassOperation {})),
            RefactoringType::BatchInterfaceExtraction => {
                Ok(Box::new(BatchInterfaceExtractionOperation {}))
            }
            RefactoringType::BatchPatternConversion => {
                Ok(Box::new(BatchPatternConversionOperation {}))
            }
            // InterfaceExtraction removed - use ExtractInterface instead
            // AsyncAwaitPatternConversion removed - use ConvertToAsync instead
        }
    }

    /// Get all available refactoring types
    pub fn available_refactorings() -> Vec<RefactoringType> {
        vec![
            RefactoringType::Rename,
            RefactoringType::ExtractFunction,
            RefactoringType::ExtractVariable,
            RefactoringType::InlineVariable,
            RefactoringType::InlineFunction,
            RefactoringType::InlineMethod,
            RefactoringType::ExtractInterface,
            RefactoringType::ConvertToAsync,
            RefactoringType::PatternConversion,
            RefactoringType::MoveMethod,
            RefactoringType::MoveClass,
            RefactoringType::MoveFile,
            RefactoringType::RemoveParameter,
            RefactoringType::IntroduceParameter,
            RefactoringType::ReplaceConstructor,
            RefactoringType::ReplaceConditionals,
            RefactoringType::ConvertMethodToFunction,
            RefactoringType::SplitClass,
            RefactoringType::MergeClasses,
            RefactoringType::ChangeSignature,
            RefactoringType::AddDelegation,
            RefactoringType::RemoveDelegation,
            RefactoringType::EncapsulateField,
            RefactoringType::LocalizeVariable,
            RefactoringType::AddMissingImports,
            RefactoringType::SortImports,
            RefactoringType::GenerateGettersSetters,
            // Removed InterfaceExtraction - use ExtractInterface instead
            // Removed AsyncAwaitPatternConversion - use ConvertToAsync instead
        ]
    }
}
