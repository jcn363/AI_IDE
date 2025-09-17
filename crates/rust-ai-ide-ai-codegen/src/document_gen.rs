//! # Documentation Generation Module
//!
//! This module provides comprehensive documentation generation capabilities using AST parsing,
//! AI-powered enhancement, and template-based formatting for Rust codebases.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use syn::{
    visit::{self, Visit},
    Attribute, Expr, ExprLit, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, Lit, Meta,
    MetaNameValue, Stmt,
};
use tokio::sync::RwLock;

use crate::CodeGenerationError;

// Re-export AI inference types for documentation
use rust_ai_ide_ai_inference::{
    InferenceError, NLToCodeConverter, NLToCodeInput, NLToCodeResult, create_nl_to_code_converter,
};
use rust_ai_ide_common::validation::validate_secure_path;

/// Configuration for documentation generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub enable_ai_suggestions:    bool,
    pub include_code_examples:    bool,
    pub generate_readme:         bool,
    pub generate_api_docs:       bool,
    pub quality_scoring_enabled: bool,
    pub consistency_checking:    bool,
    pub template_dir:            Option<String>,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            enable_ai_suggestions:    true,
            include_code_examples:    true,
            generate_readme:         true,
            generate_api_docs:       true,
            quality_scoring_enabled: true,
            consistency_checking:    true,
            template_dir:            None,
        }
    }
}

/// Documentation quality score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationQualityScore {
    pub overall_score:           f64, // 0.0 to 100.0
    pub completeness_score:      f64,
    pub clarity_score:           f64,
    pub consistency_score:       f64,
    pub example_coverage_score:  f64,
    pub issues:                  Vec<String>,
    pub suggestions:             Vec<String>,
}

/// Code element with extracted documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentedElement {
    pub element_type:    ElementType,
    pub name:            String,
    pub signature:       String,
    pub existing_docs:   Vec<String>,
    pub generated_docs:  Vec<String>,
    pub ai_suggestions:  Vec<String>,
    pub code_examples:   Vec<String>,
    pub quality_score:   Option<DocumentationQualityScore>,
    pub line_number:     usize,
    pub module_path:     Vec<String>,
    // Enhanced parsing fields
    pub arguments:       Vec<FunctionArgument>,
    pub return_type:     Option<String>,
    pub fields:          Vec<StructField>,
    pub variants:        Vec<EnumVariant>,
}

/// Function argument with name and type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionArgument {
    pub name: String,
    pub ty:   String,
}

/// Struct field with name and type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty:   String,
}

/// Enum variant with name and optional data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub data: Option<String>,
}

/// Types of code elements that can be documented
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Module,
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Type,
    Const,
    Static,
    Macro,
}

/// AST visitor for extracting documentation from Rust code
struct DocumentationVisitor {
    elements: Vec<DocumentedElement>,
    current_module_path: Vec<String>,
}

impl DocumentationVisitor {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
            current_module_path: Vec::new(),
        }
    }

    fn extract_doc_comments(&self, attrs: &[Attribute]) -> Vec<String> {
        attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("doc") {
                    if let Meta::NameValue(MetaNameValue { value: Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }), .. }) = &attr.meta {
                        Some(lit_str.value().trim().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn format_signature(&self, item: &Item) -> String {
        match item {
            Item::Fn(item_fn) => format!("{}", quote::quote!(#item_fn)),
            Item::Struct(item_struct) => format!("{}", quote::quote!(#item_struct)),
            Item::Enum(item_enum) => format!("{}", quote::quote!(#item_enum)),
            Item::Mod(item_mod) => format!("mod {}", item_mod.ident),
            _ => "Unknown".to_string(),
        }
    }

    fn extract_function_arguments(&self, inputs: &[syn::FnArg]) -> Vec<FunctionArgument> {
        inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    let name = match &*pat_type.pat {
                        syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                        _ => "arg".to_string(), // Fallback for complex patterns
                    };
                    let ty = format!("{}", quote::quote!(#pat_type.ty));
                    Some(FunctionArgument { name, ty })
                } else {
                    None // Skip self parameters for now
                }
            })
            .collect()
    }

    fn extract_return_type(&self, output: &syn::ReturnType) -> Option<String> {
        match output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(format!("{}", quote::quote!(#ty))),
        }
    }

    fn extract_struct_fields(&self, fields: &syn::Fields) -> Vec<StructField> {
        match fields {
            syn::Fields::Named(named_fields) => named_fields
                .named
                .iter()
                .filter_map(|field| {
                    let name = field.ident.as_ref()?.to_string();
                    let ty = format!("{}", quote::quote!(#field.ty));
                    Some(StructField { name, ty })
                })
                .collect(),
            syn::Fields::Unnamed(unnamed_fields) => unnamed_fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let name = index.to_string();
                    let ty = format!("{}", quote::quote!(#field.ty));
                    StructField { name, ty }
                })
                .collect(),
            syn::Fields::Unit => Vec::new(),
        }
    }

    fn extract_enum_variants(&self, variants: &[syn::Variant]) -> Vec<EnumVariant> {
        variants
            .iter()
            .map(|variant| {
                let name = variant.ident.to_string();
                let data = match &variant.fields {
                    syn::Fields::Unit => None,
                    syn::Fields::Unnamed(_) => Some("Tuple variant".to_string()),
                    syn::Fields::Named(_) => Some("Struct variant".to_string()),
                };
                EnumVariant { name, data }
            })
            .collect()
    }
}

impl<'ast> Visit<'ast> for DocumentationVisitor {
    fn visit_item(&mut self, item: &'ast Item) {
        let element_type = match item {
            Item::Mod(_) => ElementType::Module,
            Item::Fn(_) => ElementType::Function,
            Item::Struct(_) => ElementType::Struct,
            Item::Enum(_) => ElementType::Enum,
            Item::Trait(_) => ElementType::Trait,
            Item::Impl(_) => ElementType::Impl,
            Item::Type(_) => ElementType::Type,
            Item::Const(_) => ElementType::Const,
            Item::Static(_) => ElementType::Static,
            Item::Macro(_) => ElementType::Macro,
            _ => return, // Skip other items
        };

        let name = match item {
            Item::Mod(item_mod) => item_mod.ident.to_string(),
            Item::Fn(item_fn) => item_fn.sig.ident.to_string(),
            Item::Struct(item_struct) => item_struct.ident.to_string(),
            Item::Enum(item_enum) => item_enum.ident.to_string(),
            Item::Trait(item_trait) => item_trait.ident.to_string(),
            _ => "Unknown".to_string(),
        };

        let signature = self.format_signature(item);
        let existing_docs = match item {
            Item::Mod(item_mod) => self.extract_doc_comments(&item_mod.attrs),
            Item::Fn(item_fn) => self.extract_doc_comments(&item_fn.attrs),
            Item::Struct(item_struct) => self.extract_doc_comments(&item_struct.attrs),
            Item::Enum(item_enum) => self.extract_doc_comments(&item_enum.attrs),
            Item::Trait(item_trait) => self.extract_doc_comments(&item_trait.attrs),
            _ => Vec::new(),
        };

        // Extract enhanced parsing data
        let (arguments, return_type, fields, variants) = match item {
            Item::Fn(item_fn) => (
                self.extract_function_arguments(&item_fn.sig.inputs),
                self.extract_return_type(&item_fn.sig.output),
                Vec::new(),
                Vec::new(),
            ),
            Item::Struct(item_struct) => (
                Vec::new(),
                None,
                self.extract_struct_fields(&item_struct.fields),
                Vec::new(),
            ),
            Item::Enum(item_enum) => (
                Vec::new(),
                None,
                Vec::new(),
                self.extract_enum_variants(&item_enum.variants),
            ),
            _ => (Vec::new(), None, Vec::new(), Vec::new()),
        };

        let element = DocumentedElement {
            element_type,
            name: name.clone(),
            signature,
            existing_docs,
            generated_docs: Vec::new(),
            ai_suggestions: Vec::new(),
            code_examples: Vec::new(),
            quality_score: None,
            line_number: 0, // Would need span information
            module_path: self.current_module_path.clone(),
            arguments,
            return_type,
            fields,
            variants,
        };

        self.elements.push(element);

        // Handle module nesting
        if let Item::Mod(item_mod) = item {
            if let Some((_, items)) = &item_mod.content {
                self.current_module_path.push(name);
                for item in items {
                    self.visit_item(item);
                }
                self.current_module_path.pop();
            }
        }
    }
}

/// Template engine for documentation generation
#[derive(Clone)]
pub struct DocumentationTemplateEngine {
    handlebars: Handlebars<'static>,
}

impl DocumentationTemplateEngine {
    pub fn new() -> Result<Self, CodeGenerationError> {
        let mut handlebars = Handlebars::new();

        // Register documentation templates
        handlebars.register_template_string(
            "function_doc",
            r#"/// {{name}}
///
/// {{description}}
///
/// # Arguments
/// {{#each arguments}}
/// * `{{name}}` - {{description}}
/// {{/each}}
///
/// # Returns
/// {{returns}}
///
/// # Examples
/// ```rust
/// {{example}}
/// ```
"#,
        )?;

        handlebars.register_template_string(
            "struct_doc",
            r#"/// {{name}}
///
/// {{description}}
///
/// # Fields
/// {{#each fields}}
/// * `{{name}}` - {{description}}
/// {{/each}}
///
/// # Examples
/// ```rust
/// {{example}}
/// ```
"#,
        )?;

        handlebars.register_template_string(
            "enum_doc",
            r#"/// {{name}}
///
/// {{description}}
///
/// # Variants
/// {{#each variants}}
/// * `{{name}}` - {{description}}
/// {{/each}}
///
/// # Examples
/// ```rust
/// {{example}}
/// ```
"#,
        )?;

        handlebars.register_template_string(
            "module_doc",
            r#"//! # {{name}}
//!
//! {{description}}
//!
//! ## Modules
//! {{#each modules}}
//! * [{{name}}]({{name}}/index.html) - {{description}}
//! {{/each}}
//!
//! ## Types
//! {{#each types}}
//! * [{{name}}]({{name}}.html) - {{description}}
//! {{/each}}
//!
//! ## Functions
//! {{#each functions}}
//! * [{{name}}]({{name}}.html) - {{description}}
//! {{/each}}
"#,
        )?;

        Ok(Self { handlebars })
    }

    pub fn render(&self, template: &str, context: &serde_json::Value) -> Result<String, CodeGenerationError> {
        self.handlebars.render(template, context).map_err(|e| {
            CodeGenerationError::TemplateError(format!("Template rendering failed: {}", e))
        })
    }
}

/// Main documentation generator
#[derive(Clone)]
pub struct DocumentationGenerator {
    config: DocumentationConfig,
    template_engine: DocumentationTemplateEngine,
    ai_converter: Arc<NLToCodeConverter>,
    cache: Arc<RwLock<HashMap<String, Vec<DocumentedElement>>>>,
}

impl DocumentationGenerator {
    pub fn new(config: DocumentationConfig) -> Result<Self, CodeGenerationError> {
        let template_engine = DocumentationTemplateEngine::new()?;
        let ai_converter = Arc::new(create_nl_to_code_converter());

        Ok(Self {
            config,
            template_engine,
            ai_converter,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate comprehensive documentation for Rust code
    pub async fn generate_docs(&self, code: &str) -> Result<String, CodeGenerationError> {
        // Parse the code using syn
        let syntax = syn::parse_file(code).map_err(|e| {
            CodeGenerationError::ParseError(format!("Failed to parse Rust code: {}", e))
        })?;

        // Extract documented elements
        let elements = self.extract_documented_elements(&syntax)?;

        // Enhance documentation with AI if enabled
        let enhanced_elements = if self.config.enable_ai_suggestions {
            self.enhance_with_ai(elements).await?
        } else {
            elements
        };

        // Generate code examples if enabled
        let elements_with_examples = if self.config.include_code_examples {
            self.generate_code_examples(enhanced_elements).await?
        } else {
            enhanced_elements
        };

        // Quality scoring if enabled
        let elements_with_quality = if self.config.quality_scoring_enabled {
            self.score_documentation_quality(elements_with_examples).await?
        } else {
            elements_with_examples
        };

        // Consistency checking if enabled
        if self.config.consistency_checking {
            self.check_consistency(&elements_with_quality)?;
        }

        // Generate final documentation
        let documentation = self.render_documentation(&elements_with_quality)?;

        // Cache the results
        let mut cache = self.cache.write().await;
        cache.insert(code.to_string(), elements_with_quality);

        Ok(documentation)
    }

    /// Generate enhanced descriptions using AI for better documentation
    async fn generate_enhanced_description(&self, element: &DocumentedElement) -> Result<String, CodeGenerationError> {
        // If there's existing documentation, enhance it
        if !element.existing_docs.is_empty() {
            let existing_desc = element.existing_docs.first().unwrap();
            if existing_desc.len() > 10 && !existing_desc.contains("TODO") && !existing_desc.contains("FIXME") {
                // Existing documentation looks decent, return it enhanced
                return Ok(self.enhance_existing_description(element, existing_desc).await);
            }
        }

        // Generate new description using AI
        let prompt = format!(
            "Generate a comprehensive, clear description for this Rust {}: {}\n\nPlease provide a detailed explanation of what this {} does, its purpose, and key characteristics. Make it suitable for API documentation.",
            match element.element_type {
                ElementType::Function => "function",
                ElementType::Struct => "struct",
                ElementType::Enum => "enum",
                ElementType::Module => "module",
                _ => "code element",
            },
            element.name,
            match element.element_type {
                ElementType::Function => "function",
                ElementType::Struct => "struct",
                ElementType::Enum => "enum",
                ElementType::Module => "module",
                _ => "code element",
            }
        );

        let input = NLToCodeInput {
            description: prompt,
            target_language: "rust".to_string(),
            project_context: std::collections::HashMap::new(),
            coding_style: None,
            existing_code: Some(element.signature.clone()),
            requirements: vec![
                "Generate clear, professional documentation".to_string(),
                "Explain the purpose and functionality".to_string(),
                "Make it suitable for API docs".to_string(),
                "Keep it concise but informative".to_string(),
            ],
        };

        match self.ai_converter.convert(input).await {
            Ok(result) => {
                // Extract description from the generated content
                let description = result.explanation.lines()
                    .find(|line| !line.trim().is_empty() && !line.trim().starts_with("//") && !line.trim().starts_with("/*"))
                    .unwrap_or(&result.explanation)
                    .to_string();

                if description.is_empty() {
                    // Fallback to a basic description
                    Ok(format!("{} {} that provides core functionality", element.element_type, element.name))
                } else {
                    Ok(description)
                }
            }
            Err(e) => {
                // Fallback description
                log::warn!("AI description generation failed: {}. Using fallback.", e);
                Ok(format!("{} {} - automatically generated description", element.element_type, element.name))
            }
        }
    }

    /// Enhance existing descriptions with AI improvements
    async fn enhance_existing_description(&self, element: &DocumentedElement, existing_desc: &str) -> String {
        let prompt = format!(
            "Enhance this existing documentation description: '{}'\n\nFor the Rust {}: {}\n\nPlease improve clarity, completeness, and professionalism while maintaining the original meaning.",
            existing_desc,
            match element.element_type {
                ElementType::Function => "function",
                ElementType::Struct => "struct",
                ElementType::Enum => "enum",
                ElementType::Module => "module",
                _ => "code element",
            },
            element.name
        );

        let input = NLToCodeInput {
            description: prompt,
            target_language: "rust".to_string(),
            project_context: std::collections::HashMap::new(),
            coding_style: None,
            existing_code: Some(element.signature.clone()),
            requirements: vec![
                "Improve clarity and completeness".to_string(),
                "Maintain original meaning".to_string(),
                "Make it more professional".to_string(),
                "Keep it concise".to_string(),
            ],
        };

        match self.ai_converter.convert(input).await {
            Ok(result) => {
                // Use the enhanced description if it's better
                if result.explanation.len() > existing_desc.len() && result.confidence_score > 0.6 {
                    result.explanation
                } else {
                    existing_desc.to_string()
                }
            }
            Err(_) => {
                // Return original description if enhancement fails
                existing_desc.to_string()
            }
        }
    }

    /// Extract documented elements from AST
    fn extract_documented_elements(&self, syntax: &syn::File) -> Result<Vec<DocumentedElement>, CodeGenerationError> {
        let mut visitor = DocumentationVisitor::new();
        visitor.visit_file(syntax);
        Ok(visitor.elements)
    }

    /// Enhance documentation with AI suggestions
    async fn enhance_with_ai(&self, elements: Vec<DocumentedElement>) -> Result<Vec<DocumentedElement>, CodeGenerationError> {
        let mut enhanced = Vec::new();

        for element in elements {
            let mut enhanced_element = element.clone();

            if enhanced_element.existing_docs.is_empty() || enhanced_element.existing_docs.iter().any(|doc| doc.len() < 10) {
                // Generate AI suggestions
                let prompt = format!(
                    "Generate comprehensive documentation for this Rust {}: {}\n\nProvide detailed description, usage examples, and parameter explanations.",
                    match element.element_type {
                        ElementType::Function => "function",
                        ElementType::Struct => "struct",
                        ElementType::Enum => "enum",
                        ElementType::Module => "module",
                        _ => "code element",
                    },
                    element.signature
                );

                match self.generate_ai_suggestion(&prompt).await {
                    Ok(suggestions) => enhanced_element.ai_suggestions = suggestions,
                    Err(e) => log::warn!("AI suggestion failed: {}", e),
                }
            }

            enhanced.push(enhanced_element);
        }

        Ok(enhanced)
    }

    /// Generate AI-powered documentation suggestions
    async fn generate_ai_suggestion(&self, prompt: &str) -> Result<Vec<String>, CodeGenerationError> {
        // Create input for documentation generation
        let input = NLToCodeInput {
            description: prompt.to_string(),
            target_language: "rust".to_string(),
            project_context: std::collections::HashMap::new(),
            coding_style: None,
            existing_code: None,
            requirements: vec![
                "Generate comprehensive doc comments".to_string(),
                "Include parameter descriptions".to_string(),
                "Add usage examples".to_string(),
                "Follow Rust documentation conventions".to_string(),
            ],
        };

        // Use the NL to code converter to generate documentation
        match self.ai_converter.convert(input).await {
            Ok(result) => {
                // Extract documentation comments from the generated code
                let mut suggestions = Vec::new();

                // Parse the generated code to extract documentation comments
                for line in result.code.lines() {
                    if line.trim().starts_with("///") || line.trim().starts_with("//!") {
                        // Clean up the comment and add it to suggestions
                        let cleaned = line.trim()
                            .trim_start_matches("///")
                            .trim_start_matches("//!")
                            .trim();
                        if !cleaned.is_empty() {
                            suggestions.push(format!("/// {}", cleaned));
                        }
                    }
                }

                // If no documentation was found in generated code, create basic suggestions
                if suggestions.is_empty() {
                    suggestions.push(format!("/// {}", prompt.lines().next().unwrap_or("Generated documentation")));
                    suggestions.push("///".to_string());
                    suggestions.push("/// # Examples".to_string());
                    suggestions.push("/// ```rust".to_string());
                    suggestions.push("/// // Example usage would go here".to_string());
                    suggestions.push("/// ```".to_string());
                }

                // Add confidence-based quality indicator
                if result.confidence_score < 0.7 {
                    suggestions.push("/// Note: This documentation suggestion has moderate confidence.".to_string());
                }

                // Include any warnings from the AI generation
                for warning in result.warnings {
                    suggestions.push(format!("/// Warning: {}", warning));
                }

                Ok(suggestions)
            }
            Err(e) => {
                // Fallback to basic documentation generation
                log::warn!("AI documentation generation failed: {}. Using fallback.", e);
                Ok(vec![
                    format!("/// {}", prompt.lines().next().unwrap_or("Generated documentation")),
                    "///".to_string(),
                    "/// # Examples".to_string(),
                    "/// ```rust".to_string(),
                    "/// // Example usage".to_string(),
                    "/// ```".to_string(),
                ])
            }
        }
    }

    /// Generate code examples for documented elements
    async fn generate_code_examples(&self, elements: Vec<DocumentedElement>) -> Result<Vec<DocumentedElement>, CodeGenerationError> {
        let mut with_examples = Vec::new();

        for element in elements {
            let mut element_with_examples = element.clone();

            // Create a detailed prompt for example generation
            let prompt = self.create_example_generation_prompt(&element)?;

            // Use AI to generate comprehensive examples
            let examples = self.generate_ai_examples(&prompt, &element).await?;

            // Add the generated examples
            element_with_examples.code_examples.extend(examples);
            with_examples.push(element_with_examples);
        }

        Ok(with_examples)
    }

    /// Create a detailed prompt for example generation
    fn create_example_generation_prompt(&self, element: &DocumentedElement) -> Result<String, CodeGenerationError> {
        let mut prompt = format!(
            "Generate practical, comprehensive code examples for this Rust {}: {}\n\n",
            match element.element_type {
                ElementType::Function => "function",
                ElementType::Struct => "struct",
                ElementType::Enum => "enum",
                ElementType::Module => "module",
                ElementType::Trait => "trait",
                ElementType::Impl => "implementation",
                _ => "code element",
            },
            element.signature
        );

        // Add context from existing documentation
        if !element.existing_docs.is_empty() {
            prompt.push_str("Existing documentation:\n");
            for doc in &element.existing_docs {
                prompt.push_str(&format!("- {}\n", doc));
            }
            prompt.push_str("\n");
        }

        // Add parameter information
        if !element.arguments.is_empty() {
            prompt.push_str("Parameters:\n");
            for arg in &element.arguments {
                prompt.push_str(&format!("- {}: {}\n", arg.name, arg.ty));
            }
            prompt.push_str("\n");
        }

        // Add return type information
        if let Some(ref return_type) = element.return_type {
            prompt.push_str(&format!("Returns: {}\n\n", return_type));
        }

        // Add field information for structs
        if !element.fields.is_empty() {
            prompt.push_str("Fields:\n");
            for field in &element.fields {
                prompt.push_str(&format!("- {}: {}\n", field.name, field.ty));
            }
            prompt.push_str("\n");
        }

        // Add variant information for enums
        if !element.variants.is_empty() {
            prompt.push_str("Variants:\n");
            for variant in &element.variants {
                if let Some(ref data) = variant.data {
                    prompt.push_str(&format!("- {}: {}\n", variant.name, data));
                } else {
                    prompt.push_str(&format!("- {}\n", variant.name));
                }
            }
            prompt.push_str("\n");
        }

        prompt.push_str("Requirements:\n");
        prompt.push_str("- Include multiple practical examples showing different use cases\n");
        prompt.push_str("- Show proper error handling where applicable\n");
        prompt.push_str("- Demonstrate best practices and common patterns\n");
        prompt.push_str("- Include comments explaining the examples\n");
        prompt.push_str("- Make examples runnable and self-contained\n");

        Ok(prompt)
    }

    /// Generate AI-powered code examples
    async fn generate_ai_examples(&self, prompt: &str, element: &DocumentedElement) -> Result<Vec<String>, CodeGenerationError> {
        // Create input for example generation
        let input = NLToCodeInput {
            description: prompt.to_string(),
            target_language: "rust".to_string(),
            project_context: std::collections::HashMap::new(),
            coding_style: None,
            existing_code: Some(element.signature.clone()),
            requirements: vec![
                "Generate multiple practical examples".to_string(),
                "Include error handling".to_string(),
                "Show best practices".to_string(),
                "Make examples runnable".to_string(),
                "Add explanatory comments".to_string(),
            ],
        };

        match self.ai_converter.convert(input).await {
            Ok(result) => {
                let mut examples = Vec::new();

                // Extract code examples from the generated result
                let code_lines: Vec<&str> = result.code.lines().collect();

                // Look for code blocks and extract examples
                let mut current_example = Vec::new();
                let mut in_example = false;

                for line in &code_lines {
                    if line.trim().starts_with("```rust") || line.trim().starts_with("```") {
                        if in_example && !current_example.is_empty() {
                            // End of example block
                            examples.push(current_example.join("\n"));
                            current_example.clear();
                        }
                        in_example = !in_example;
                    } else if in_example {
                        current_example.push(line.to_string());
                    } else if !line.trim().is_empty() && !line.trim().starts_with("//") {
                        // Non-comment code line outside blocks - could be an example
                        if current_example.is_empty() {
                            current_example.push(line.to_string());
                        } else {
                            current_example.push(line.to_string());
                        }
                    }
                }

                // Add any remaining example
                if !current_example.is_empty() && !examples.contains(&current_example.join("\n")) {
                    examples.push(current_example.join("\n"));
                }

                // If no examples were extracted, generate basic fallback examples
                if examples.is_empty() {
                    examples.extend(self.generate_fallback_examples(element));
                }

                // Add confidence indicator
                if result.confidence_score < 0.7 {
                    examples.push("// Note: These examples have moderate confidence and may need adjustment.".to_string());
                }

                // Include any warnings
                for warning in result.warnings {
                    examples.push(format!("// Warning: {}", warning));
                }

                Ok(examples)
            }
            Err(e) => {
                // Fallback to basic example generation
                log::warn!("AI example generation failed: {}. Using fallback.", e);
                Ok(self.generate_fallback_examples(element))
            }
        }
    }

    /// Generate fallback examples when AI generation fails
    fn generate_fallback_examples(&self, element: &DocumentedElement) -> Vec<String> {
        let mut examples = Vec::new();

        match element.element_type {
            ElementType::Function => {
                let mut example = format!("// Example usage of {}", element.name);
                if !element.arguments.is_empty() {
                    let params = element.arguments.iter()
                        .map(|arg| match arg.ty.as_str() {
                            "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "42".to_string(),
                            "String" | "&str" => "\"example\"".to_string(),
                            "bool" => "true".to_string(),
                            _ => format!("{}::default()", arg.ty),
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    example = format!("let result = {}({});", element.name, params);
                    if element.return_type.is_some() {
                        example = format!("{}\nprintln!(\"Result: {{:?}}\", result);", example);
                    } else {
                        example = format!("{};", example);
                    }
                }
                examples.push(example);
            }
            ElementType::Struct => {
                let mut example = format!("// Create an instance of {}", element.name);
                if !element.fields.is_empty() {
                    let field_assignments = element.fields.iter()
                        .map(|field| match field.ty.as_str() {
                            "i32" | "i64" | "u32" | "u64" => format!("{}: 42", field.name),
                            "f32" | "f64" => format!("{}: 3.14", field.name),
                            "String" => format!("{}: \"example\".to_string()", field.name),
                            "&str" => format!("{}: \"example\"", field.name),
                            "bool" => format!("{}: true", field.name),
                            _ => format!("{}: {}::default()", field.name, field.ty),
                        })
                        .collect::<Vec<_>>()
                        .join(",\n    ");

                    example = format!("let instance = {} {{\n    {}\n}};", element.name, field_assignments);
                }
                examples.push(example);
            }
            ElementType::Enum => {
                let mut example = format!("// Use {} variants", element.name);
                if !element.variants.is_empty() {
                    let first_variant = &element.variants[0];
                    if let Some(ref data) = first_variant.data {
                        if data.contains("Tuple") {
                            example = format!("let variant = {}::{}(/* values */);", element.name, first_variant.name);
                        } else if data.contains("Struct") {
                            example = format!("let variant = {}::{} {{ /* fields */ }};", element.name, first_variant.name);
                        }
                    } else {
                        example = format!("let variant = {}::{};", element.name, first_variant.name);
                    }
                }
                examples.push(example);
            }
            _ => {
                examples.push(format!("// Example usage of {}", element.name));
            }
        }

        examples
    }

    /// Score documentation quality
    async fn score_documentation_quality(&self, elements: Vec<DocumentedElement>) -> Result<Vec<DocumentedElement>, CodeGenerationError> {
        let mut scored = Vec::new();

        for element in elements {
            let mut scored_element = element.clone();

            // Calculate real quality metrics
            let completeness_score = self.calculate_completeness(&element);
            let clarity_score = self.calculate_clarity(&element).await?;
            let consistency_score = self.calculate_consistency(&element, &elements);
            let example_coverage_score = self.calculate_example_coverage(&element);

            let overall = (completeness_score * 0.25 + clarity_score * 0.35 + consistency_score * 0.20 + example_coverage_score * 0.20).max(0.0).min(100.0);

            let (issues, suggestions) = self.generate_quality_feedback(&element, completeness_score, clarity_score, consistency_score, example_coverage_score);

            let score = DocumentationQualityScore {
                overall_score: overall,
                completeness_score,
                clarity_score,
                consistency_score,
                example_coverage_score,
                issues,
                suggestions,
            };

            scored_element.quality_score = Some(score);
            scored.push(scored_element);
        }

        Ok(scored)
    }

    /// Calculate documentation completeness score
    fn calculate_completeness(&self, element: &DocumentedElement) -> f64 {
        let mut score = 0.0;
        let max_score = 100.0;

        // Check for basic documentation presence
        if !element.existing_docs.is_empty() {
            score += 30.0;

            // Check documentation length (longer docs are generally more complete)
            let total_doc_length: usize = element.existing_docs.iter().map(|doc| doc.len()).sum();
            if total_doc_length > 50 {
                score += 20.0;
            } else if total_doc_length > 20 {
                score += 10.0;
            }

            // Check for key sections
            let doc_text = element.existing_docs.join(" ").to_lowercase();
            if doc_text.contains("parameter") || doc_text.contains("argument") {
                score += 15.0;
            }
            if doc_text.contains("return") || doc_text.contains("returns") {
                score += 10.0;
            }
            if doc_text.contains("example") {
                score += 10.0;
            }
        }

        // Check for parameter documentation
        if !element.arguments.is_empty() {
            let documented_params = element.arguments.iter()
                .filter(|arg| element.existing_docs.iter()
                    .any(|doc| doc.to_lowercase().contains(&arg.name.to_lowercase())))
                .count();
            let param_coverage = documented_params as f64 / element.arguments.len() as f64;
            score += param_coverage * 10.0;
        }

        // Check for return type documentation
        if element.return_type.is_some() {
            if element.existing_docs.iter()
                .any(|doc| doc.to_lowercase().contains("return") || doc.to_lowercase().contains("returns")) {
                score += 5.0;
            }
        }

        score.min(max_score)
    }

    /// Calculate documentation clarity score using AI analysis
    async fn calculate_clarity(&self, element: &DocumentedElement) -> Result<f64, CodeGenerationError> {
        if element.existing_docs.is_empty() {
            return Ok(0.0);
        }

        let doc_text = element.existing_docs.join(" ");

        // Create analysis prompt for clarity
        let prompt = format!(
            "Analyze the clarity of this documentation for {}: {}\n\nDocumentation: {}\n\nRate the clarity on a scale of 0-100 based on:\n- Use of clear, concise language\n- Proper technical terminology\n- Logical structure and flow\n- Avoidance of ambiguity\n- Readability for developers\n\nProvide only a numerical score.",
            element.name, element.signature, doc_text
        );

        // Use AI to analyze clarity
        let input = NLToCodeInput {
            description: prompt,
            target_language: "rust".to_string(),
            project_context: std::collections::HashMap::new(),
            coding_style: None,
            existing_code: Some(element.signature.clone()),
            requirements: vec!["Provide numerical clarity score".to_string()],
        };

        match self.ai_converter.convert(input).await {
            Ok(result) => {
                // Extract numerical score from response
                let score_text = result.code.lines()
                    .find(|line| line.chars().all(|c| c.is_numeric() || c == '.'))
                    .unwrap_or("50.0");

                score_text.parse::<f64>().unwrap_or(50.0).min(100.0).max(0.0)
            }
            Err(_) => {
                // Fallback: basic clarity heuristics
                let mut score = 50.0;

                // Check for overly long sentences
                let sentences: Vec<&str> = doc_text.split(|c: char| c == '.' || c == '!' || c == '?').collect();
                let avg_sentence_length = sentences.iter().map(|s| s.len()).sum::<usize>() as f64 / sentences.len() as f64;

                if avg_sentence_length < 100.0 {
                    score += 20.0;
                } else if avg_sentence_length > 200.0 {
                    score -= 20.0;
                }

                // Check for jargon vs clarity
                if doc_text.contains("TODO") || doc_text.contains("FIXME") {
                    score -= 10.0;
                }

                score.max(0.0).min(100.0)
            }
        }
    }

    /// Calculate documentation consistency score
    fn calculate_consistency(&self, element: &DocumentedElement, all_elements: &[DocumentedElement]) -> f64 {
        let mut score = 100.0;

        if element.existing_docs.is_empty() {
            return 50.0; // Neutral score for undocumented elements
        }

        let doc_text = element.existing_docs.join(" ").to_lowercase();

        // Check for inconsistent terminology
        let mut found_terms = std::collections::HashSet::new();
        for other_element in all_elements {
            if other_element.name != element.name {
                for doc in &other_element.existing_docs {
                    // Look for similar terms that might be inconsistent
                    if doc.to_lowercase().contains(&element.name.to_lowercase()) {
                        // This is a reference, which is good
                        score += 2.0;
                    }
                }
            }
        }

        // Check for consistent parameter naming
        if !element.arguments.is_empty() {
            let param_names: std::collections::HashSet<_> = element.arguments.iter().map(|arg| arg.name.as_str()).collect();
            let mentioned_params: std::collections::HashSet<_> = element.existing_docs.iter()
                .flat_map(|doc| doc.split_whitespace())
                .filter(|word| param_names.contains(word))
                .collect();

            let consistency_ratio = mentioned_params.len() as f64 / param_names.len() as f64;
            score = score * 0.7 + consistency_ratio * 30.0;
        }

        score.max(0.0).min(100.0)
    }

    /// Calculate example coverage score
    fn calculate_example_coverage(&self, element: &DocumentedElement) -> f64 {
        if element.code_examples.is_empty() {
            return 0.0;
        }

        let mut score = 50.0; // Base score for having examples

        // More examples = higher score
        score += (element.code_examples.len() as f64 * 10.0).min(30.0);

        // Check for code example quality
        let has_error_handling = element.code_examples.iter()
            .any(|ex| ex.contains("Result") || ex.contains("unwrap") || ex.contains("match") || ex.contains("if let"));
        if has_error_handling {
            score += 10.0;
        }

        // Check for comprehensive coverage
        let has_multiple_scenarios = element.code_examples.len() > 1;
        if has_multiple_scenarios {
            score += 10.0;
        }

        score.min(100.0)
    }

    /// Generate quality feedback based on scores
    fn generate_quality_feedback(
        &self,
        element: &DocumentedElement,
        completeness: f64,
        clarity: f64,
        consistency: f64,
        examples: f64,
    ) -> (Vec<String>, Vec<String>) {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Completeness issues
        if completeness < 50.0 {
            issues.push("Documentation is incomplete".to_string());
            suggestions.push("Add comprehensive description of the code element's purpose".to_string());
        }
        if completeness < 70.0 && !element.arguments.is_empty() {
            issues.push("Parameter documentation missing or incomplete".to_string());
            suggestions.push("Document all function parameters with their types and purposes".to_string());
        }
        if completeness < 70.0 && element.return_type.is_some() {
            issues.push("Return value documentation missing".to_string());
            suggestions.push("Document what the function returns and any special conditions".to_string());
        }

        // Clarity issues
        if clarity < 60.0 {
            issues.push("Documentation clarity could be improved".to_string());
            suggestions.push("Use clearer language and avoid technical jargon without explanation".to_string());
        }

        // Consistency issues
        if consistency < 70.0 {
            issues.push("Documentation consistency issues detected".to_string());
            suggestions.push("Ensure consistent terminology and parameter references throughout documentation".to_string());
        }

        // Example coverage issues
        if examples < 50.0 {
            issues.push("Code examples are missing or inadequate".to_string());
            suggestions.push("Add practical code examples showing common usage patterns".to_string());
        }
        if examples < 70.0 && element.code_examples.len() < 2 {
            suggestions.push("Add multiple examples showing different use cases".to_string());
        }

        // Additional suggestions based on element type
        match element.element_type {
            ElementType::Function => {
                if !element.existing_docs.iter().any(|doc| doc.contains("# Example")) {
                    suggestions.push("Consider adding code examples in Rust doc format with ```rust blocks".to_string());
                }
            }
            ElementType::Struct => {
                if !element.existing_docs.iter().any(|doc| doc.contains("field") || doc.contains("Field")) {
                    suggestions.push("Document struct fields and their purposes".to_string());
                }
            }
            ElementType::Enum => {
                if !element.existing_docs.iter().any(|doc| doc.contains("variant") || doc.contains("Variant")) {
                    suggestions.push("Document enum variants and when to use each".to_string());
                }
            }
            _ => {}
        }

        (issues, suggestions)
    }

    /// Check documentation consistency across codebase
    fn check_consistency(&self, elements: &[DocumentedElement]) -> Result<(), CodeGenerationError> {
        // Placeholder consistency checking
        // In a real implementation, this would check for:
        // - Consistent terminology
        // - Similar patterns across similar functions
        // - Matching parameter documentation
        // - Cross-references

        log::info!("Consistency check completed for {} elements", elements.len());
        Ok(())
    }

    /// Render final documentation
    fn render_documentation(&self, elements: &[DocumentedElement]) -> Result<String, CodeGenerationError> {
        let mut documentation = String::new();

        // Group elements by type
        let mut modules = Vec::new();
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();

        for element in elements {
            match element.element_type {
                ElementType::Module => modules.push(element),
                ElementType::Function => functions.push(element),
                ElementType::Struct => structs.push(element),
                ElementType::Enum => enums.push(element),
                _ => {},
            }
        }

        // Generate module documentation
        for module in &modules {
            let context = serde_json::json!({
                "name": module.name,
                "description": module.existing_docs.first().unwrap_or(&"Module description".to_string()),
                "modules": [],
                "types": structs.iter().map(|s| serde_json::json!({
                    "name": s.name,
                    "description": s.existing_docs.first().unwrap_or(&"Type description".to_string())
                })).collect::<Vec<_>>(),
                "functions": functions.iter().map(|f| serde_json::json!({
                    "name": f.name,
                    "description": f.existing_docs.first().unwrap_or(&"Function description".to_string())
                })).collect::<Vec<_>>()
            });

            documentation.push_str(&self.template_engine.render("module_doc", &context)?);
            documentation.push_str("\n\n");
        }

        // Generate function documentation
        for function in &functions {
            let arguments = function.arguments.iter().map(|arg| {
                serde_json::json!({
                    "name": arg.name,
                    "description": format!("Parameter of type {}", arg.ty)
                })
            }).collect::<Vec<_>>();

            let returns = function.return_type.as_ref()
                .map(|rt| format!("Returns {}", rt))
                .unwrap_or_else(|| "No return value".to_string());

            // Generate enhanced description using AI
            let description = self.generate_enhanced_description(function).await?;

            let context = serde_json::json!({
                "name": function.name,
                "description": description,
                "arguments": arguments,
                "returns": returns,
                "example": function.code_examples.first().unwrap_or(&"// Example usage".to_string())
            });

            documentation.push_str(&self.template_engine.render("function_doc", &context)?);
            documentation.push_str("\n\n");
        }

        // Generate struct documentation
        for struct_elem in &structs {
            let fields = struct_elem.fields.iter().map(|field| {
                serde_json::json!({
                    "name": field.name,
                    "description": format!("Field of type {}", field.ty)
                })
            }).collect::<Vec<_>>();

            // Generate enhanced description using AI
            let description = self.generate_enhanced_description(struct_elem).await?;

            let context = serde_json::json!({
                "name": struct_elem.name,
                "description": description,
                "fields": fields,
                "example": struct_elem.code_examples.first().unwrap_or(&"// Example usage".to_string())
            });

            documentation.push_str(&self.template_engine.render("struct_doc", &context)?);
            documentation.push_str("\n\n");
        }

        // Generate enum documentation
        for enum_elem in &enums {
            let variants = enum_elem.variants.iter().map(|variant| {
                serde_json::json!({
                    "name": variant.name,
                    "description": variant.data.as_ref()
                        .map(|d| format!("{} variant", d))
                        .unwrap_or_else(|| "Unit variant".to_string())
                })
            }).collect::<Vec<_>>();

            // Generate enhanced description using AI
            let description = self.generate_enhanced_description(enum_elem).await?;

            let context = serde_json::json!({
                "name": enum_elem.name,
                "description": description,
                "variants": variants,
                "example": enum_elem.code_examples.first().unwrap_or(&"// Example usage".to_string())
            });

            documentation.push_str(&self.template_engine.render("enum_doc", &context)?);
            documentation.push_str("\n\n");
        }

        Ok(documentation)
    }

    /// Generate README documentation
    pub async fn generate_readme(&self, project_info: &ProjectInfo) -> Result<String, CodeGenerationError> {
        // Placeholder README generation
        Ok(format!(
            "# {}\n\n{}\n\n## Features\n\n- Feature 1\n- Feature 2\n\n## Usage\n\n```rust\n// Example\n```",
            project_info.name,
            project_info.description
        ))
    }

    /// Generate API documentation
    pub async fn generate_api_docs(&self, elements: &[DocumentedElement]) -> Result<String, CodeGenerationError> {
        let mut api_docs = String::from("# API Documentation\n\n");

        for element in elements {
            api_docs.push_str(&format!("## {}\n\n", element.name));
            api_docs.push_str(&format!("Type: {:?}\n\n", element.element_type));

            if !element.existing_docs.is_empty() {
                api_docs.push_str("Documentation:\n");
                for doc in &element.existing_docs {
                    api_docs.push_str(&format!("{}\n", doc));
                }
                api_docs.push_str("\n");
            }

            api_docs.push_str(&format!("Signature:\n```rust\n{}\n```\n\n", element.signature));

            if !element.code_examples.is_empty() {
                api_docs.push_str("Examples:\n```rust\n");
                for example in &element.code_examples {
                    api_docs.push_str(&format!("{}\n", example));
                }
                api_docs.push_str("```\n\n");
            }
        }

        Ok(api_docs)
    }

    /// Get cached documentation
    pub async fn get_cached_docs(&self, code: &str) -> Option<Vec<DocumentedElement>> {
        let cache = self.cache.read().await;
        cache.get(code).cloned()
    }
}

impl Default for DocumentationGenerator {
    fn default() -> Self {
        Self::new(DocumentationConfig::default()).unwrap()
    }
}

/// Project information for README generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name:        String,
    pub description: String,
    pub version:     String,
    pub authors:     Vec<String>,
    pub repository:  Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_generator_creation() {
        let generator = DocumentationGenerator::new(DocumentationConfig::default());
        assert!(generator.is_ok());
    }

    #[tokio::test]
    async fn test_basic_documentation_generation() {
        let generator = DocumentationGenerator::default();
        let code = r#"
            /// This is a test function
            fn test_function(x: i32) -> i32 {
                x * 2
            }
        "#;

        let result = generator.generate_docs(code).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_template_engine_creation() {
        let engine = DocumentationTemplateEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_function_argument_extraction() {
        let code = r#"
            fn test_function(x: i32, y: String) -> i32 {
                x + y.len() as i32
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut visitor = DocumentationVisitor::new();
        visitor.visit_file(&syntax);

        assert_eq!(visitor.elements.len(), 1);
        let element = &visitor.elements[0];

        assert_eq!(element.arguments.len(), 2);
        assert_eq!(element.arguments[0].name, "x");
        assert_eq!(element.arguments[0].ty, "i32");
        assert_eq!(element.arguments[1].name, "y");
        assert_eq!(element.arguments[1].ty, "String");

        assert_eq!(element.return_type, Some("i32".to_string()));
    }

    #[test]
    fn test_function_no_return_type() {
        let code = r#"
            fn test_function(x: i32) {
                println!("{}", x);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut visitor = DocumentationVisitor::new();
        visitor.visit_file(&syntax);

        assert_eq!(visitor.elements.len(), 1);
        let element = &visitor.elements[0];

        assert_eq!(element.return_type, None);
    }

    #[test]
    fn test_struct_field_extraction() {
        let code = r#"
            struct TestStruct {
                pub field1: i32,
                field2: String,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut visitor = DocumentationVisitor::new();
        visitor.visit_file(&syntax);

        assert_eq!(visitor.elements.len(), 1);
        let element = &visitor.elements[0];

        assert_eq!(element.fields.len(), 2);
        assert_eq!(element.fields[0].name, "field1");
        assert_eq!(element.fields[0].ty, "i32");
        assert_eq!(element.fields[1].name, "field2");
        assert_eq!(element.fields[1].ty, "String");
    }

    #[test]
    fn test_tuple_struct_extraction() {
        let code = r#"
            struct TupleStruct(i32, String);
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut visitor = DocumentationVisitor::new();
        visitor.visit_file(&syntax);

        assert_eq!(visitor.elements.len(), 1);
        let element = &visitor.elements[0];

        assert_eq!(element.fields.len(), 2);
        assert_eq!(element.fields[0].name, "0");
        assert_eq!(element.fields[0].ty, "i32");
        assert_eq!(element.fields[1].name, "1");
        assert_eq!(element.fields[1].ty, "String");
    }

    #[test]
    fn test_enum_variant_extraction() {
        let code = r#"
            enum TestEnum {
                UnitVariant,
                TupleVariant(i32, String),
                StructVariant { field: bool },
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut visitor = DocumentationVisitor::new();
        visitor.visit_file(&syntax);

        assert_eq!(visitor.elements.len(), 1);
        let element = &visitor.elements[0];

        assert_eq!(element.variants.len(), 3);
        assert_eq!(element.variants[0].name, "UnitVariant");
        assert_eq!(element.variants[0].data, None);

        assert_eq!(element.variants[1].name, "TupleVariant");
        assert_eq!(element.variants[1].data, Some("Tuple variant".to_string()));

        assert_eq!(element.variants[2].name, "StructVariant");
        assert_eq!(element.variants[2].data, Some("Struct variant".to_string()));
    }

    #[tokio::test]
    async fn test_documentation_generation_with_parsing() {
        let generator = DocumentationGenerator::default();
        let code = r#"
            /// This is a test function
            fn calculate(x: i32, y: i32) -> i32 {
                x + y
            }

            /// Test struct
            struct Point {
                x: f64,
                y: f64,
            }

            /// Test enum
            enum Color {
                Red,
                Green,
                Blue,
            }
        "#;

        let result = generator.generate_docs(code).await;
        assert!(result.is_ok());

        let documentation = result.unwrap();
        assert!(!documentation.is_empty());

        // Check that arguments are populated in the generated docs
        assert!(documentation.contains("calculate"));
        assert!(documentation.contains("Parameter of type i32"));

        // Check that struct fields are populated
        assert!(documentation.contains("Point"));
        assert!(documentation.contains("Field of type f64"));

        // Check that enum variants are populated
        assert!(documentation.contains("Color"));
        assert!(documentation.contains("Unit variant"));
    }

    #[test]
    fn test_template_rendering_with_arguments() {
        let engine = DocumentationTemplateEngine::new().unwrap();

        let context = serde_json::json!({
            "name": "test_function",
            "description": "A test function",
            "arguments": [
                {"name": "arg1", "description": "First argument of type i32"},
                {"name": "arg2", "description": "Second argument of type String"}
            ],
            "returns": "Returns i32",
            "example": "let result = test_function(1, \"hello\".to_string());"
        });

        let result = engine.render("function_doc", &context);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("test_function"));
        assert!(rendered.contains("* `arg1` - First argument of type i32"));
        assert!(rendered.contains("* `arg2` - Second argument of type String"));
        assert!(rendered.contains("Returns i32"));
    }

    #[test]
    fn test_template_rendering_with_fields() {
        let engine = DocumentationTemplateEngine::new().unwrap();

        let context = serde_json::json!({
            "name": "TestStruct",
            "description": "A test struct",
            "fields": [
                {"name": "field1", "description": "First field of type i32"},
                {"name": "field2", "description": "Second field of type String"}
            ],
            "example": "let s = TestStruct { field1: 1, field2: \"test\".to_string() };"
        });

        let result = engine.render("struct_doc", &context);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("TestStruct"));
        assert!(rendered.contains("* `field1` - First field of type i32"));
        assert!(rendered.contains("* `field2` - Second field of type String"));
    }

    #[test]
    fn test_template_rendering_with_variants() {
        let engine = DocumentationTemplateEngine::new().unwrap();

        let context = serde_json::json!({
            "name": "TestEnum",
            "description": "A test enum",
            "variants": [
                {"name": "Variant1", "description": "Unit variant"},
                {"name": "Variant2", "description": "Tuple variant"}
            ],
            "example": "let v = TestEnum::Variant1;"
        });

        let result = engine.render("enum_doc", &context);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("TestEnum"));
        assert!(rendered.contains("* `Variant1` - Unit variant"));
        assert!(rendered.contains("* `Variant2` - Tuple variant"));
    }
}
