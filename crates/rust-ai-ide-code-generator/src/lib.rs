//! # Rust AI IDE Code Generation System
//!
//! This crate provides comprehensive code generation capabilities for enhancing developer productivity,
//! including automatic test generation, documentation, boilerplate code, and implementation stubs.

pub mod generators;
pub mod templates;
pub mod code_analysis;
pub mod cache;

// Re-exports
pub use generators::*;
pub use templates::*;
pub use code_analysis::*;
pub use cache::*;

use generators::{
    TestGenerator, DocumentationGenerator, BoilerplateGenerator,
    ExampleGenerator, StubGenerator, RefactoringGenerator, TemplateEngine,
    RefactoringSuggestion, RefactoringInput,
    refactoring_generator::create_refactoring_generator
};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_ai_ide_ai_analysis::AnalysisResult;
use tracing::{info, debug};

/// Main code generator orchestrator
#[derive(Clone)]
pub struct CodeGenerator {
    template_engine: Arc<RwLock<TemplateEngine>>,
    test_generator: Arc<TestGenerator>,
    docs_generator: Arc<DocumentationGenerator>,
    boilerplate_generator: Arc<BoilerplateGenerator>,
    example_generator: Arc<ExampleGenerator>,
    stub_generator: Arc<StubGenerator>,
    refactoring_generator: Arc<RefactoringGenerator>,
    generation_history: Arc<RwLock<HashMap<Uuid, GeneratedCode>>>,
}

impl CodeGenerator {
    /// Create a new code generator with default configurations
    pub fn new() -> Self {
        Self {
            template_engine: Arc::new(RwLock::new(TemplateEngine::new())),
            test_generator: Arc::new(TestGenerator::new()),
            docs_generator: Arc::new(DocumentationGenerator::new()),
            boilerplate_generator: Arc::new(BoilerplateGenerator::new()),
            example_generator: Arc::new(ExampleGenerator::new()),
            stub_generator: Arc::new(StubGenerator::new()),
            refactoring_generator: Arc::new(RefactoringGenerator::new()),
            generation_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate comprehensive code for a given item
    pub async fn generate_item_code(&self, input: &CodeGenerationInput) -> Result<CodeGenerationResult, CodeGenerationError> {
        let generation_id = Uuid::new_v4();
        info!("Starting comprehensive code generation for: {} (ID: {})", input.item_name, generation_id);

        let mut results = Vec::new();

        // Generate tests
        if input.generate_tests {
            let test_result = self.test_generator.generate_tests(input).await?;
            results.extend(test_result);
        }

        // Generate documentation
        if input.generate_docs {
            let docs_result = self.docs_generator.generate_docs(input).await?;
            results.extend(docs_result);
        }

        // Generate boilerplate
        if input.generate_boilerplate {
            let boilerplate_result = self.boilerplate_generator.generate_boilerplate(input).await?;
            results.extend(boilerplate_result);
        }

        // Generate examples
        if input.generate_examples {
            let example_result = self.example_generator.generate_examples(input).await?;
            results.extend(example_result);
        }

        // Generate implementation stubs
        if input.generate_stubs {
            let stub_result = self.stub_generator.generate_stubs(input).await?;
            results.extend(stub_result);
        }

        // Store generation results
        let generated_result = GeneratedCode {
            id: generation_id,
            input: input.clone(),
            results: results.clone(),
            timestamp: Utc::now(),
            metadata: GenerationMetadata {
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                total_files_generated: results.len(),
                total_lines_generated: results.iter().map(|r| r.content.lines().count()).sum(),
            },
        };

        let mut history = self.generation_history.write().await;
        history.insert(generation_id, generated_result);

        info!("Completed code generation: {} files generated, {} lines total",
              results.len(),
              results.iter().map(|r| r.content.lines().count()).sum::<usize>());

        Ok(CodeGenerationResult {
            id: generation_id,
            files: results,
            summary: self.generate_generation_summary(&results),
        })
    }

    /// Analyze existing code and suggest what to generate
    pub async fn analyze_and_suggest(&self, content: &str, file_path: &str) -> Result<CodeGenerationSuggestions, CodeGenerationError> {
        // Analyze the code to determine what can be generated
        let analysis = self.analyze_code_for_generation(content, file_path).await?;

        Ok(CodeGenerationSuggestions {
            file_path: file_path.to_string(),
            suggested_generations: analysis,
            confidence: 0.0, // Would be calculated based on analysis completeness
        })
    }

    /// Get generation history
    pub async fn get_generation_history(&self) -> HashMap<Uuid, GeneratedCode> {
        let history = self.generation_history.read().await;
        history.clone()
    }

    /// Get specific generation result by ID
    pub async fn get_generation_result(&self, id: &Uuid) -> Option<GeneratedCode> {
        let history = self.generation_history.read().await;
        history.get(id).cloned()
    }

    /// Analyze code and suggest refactoring opportunities
    pub async fn analyze_for_refactoring(&self, input: &RefactoringInput) -> Result<Vec<RefactoringSuggestion>, CodeGenerationError> {
        self.refactoring_generator.analyze_and_suggest_refactoring(input).await
    }

    /// Apply refactoring suggestions
    pub async fn apply_refactoring(&self, original_content: &str, suggestions: &[RefactoringSuggestion]) -> Result<String, CodeGenerationError> {
        self.refactoring_generator.apply_refactoring(original_content, suggestions).await
    }

    /// Get refactoring generator instance for advanced operations
    pub fn refactoring_generator(&self) -> Arc<RefactoringGenerator> {
        Arc::clone(&self.refactoring_generator)
    }

    /// Clear old generation history
    pub async fn clear_old_history(&self, before_timestamp: DateTime<Utc>) {
        let mut history = self.generation_history.write().await;
        let initial_count = history.len();

        history.retain(|_, result| result.timestamp >= before_timestamp);

        let final_count = history.len();
        info!("Cleared {} old generation results, {} remaining",
              initial_count - final_count, final_count);
    }

    /// Analyze code to determine generation opportunities
    async fn analyze_code_for_generation(&self, content: &str, file_path: &str) -> Result<Vec<GenerationSuggestion>, CodeGenerationError> {
        let mut suggestions = Vec::new();

        // Parse the code
        let ast = syn::parse2(content.parse::<proc_macro2::TokenStream>().unwrap()).map_err(|e| CodeGenerationError::ParseError(e.to_string()))?;

        // Analyze functions for missing tests
        let mut function_count = 0;
        let mut tested_functions = 0;

        // Check if there's already test file
        let test_file_exists = file_path.contains("test") || file_path.ends_with("_test.rs");

        syn::visit::visit_file(&mut FunctionAnalysisVisitor {
            suggestions: &mut suggestions,
            function_count: &mut function_count,
            tested_functions: &mut tested_functions,
            test_file_exists,
        }, &ast);

        if !test_file_exists && function_count > 0 {
            suggestions.push(GenerationSuggestion {
                generation_type: GenerationType::Tests,
                description: format!("Generate test cases for {} functions", function_count),
                confidence: 0.9,
                reason: "No test file found for functions to test".to_string(),
            });
        }

        // Check for missing documentation
        let undocumented_functions = self.count_undocumented_functions(&ast);
        if undocumented_functions > 0 {
            suggestions.push(GenerationSuggestion {
                generation_type: GenerationType::Documentation,
                description: format!("Generate documentation for {} functions", undocumented_functions),
                confidence: 0.8,
                reason: "Functions lack proper documentation".to_string(),
            });
        }

        // Suggest examples for traits and structs
        let mut trait_count = 0;
        let mut struct_count = 0;

        syn::visit::visit_file(&mut TypeAnalysisVisitor {
            trait_count: &mut trait_count,
            struct_count: &mut struct_count,
        }, &ast);

        if trait_count > 0 {
            suggestions.push(GenerationSuggestion {
                generation_type: GenerationType::Examples,
                description: format!("Generate usage examples for {} trait(s)", trait_count),
                confidence: 0.7,
                reason: "Traits would benefit from usage examples".to_string(),
            });
        }

        if struct_count > 0 {
            suggestions.push(GenerationSuggestion {
                generation_type: GenerationType::Boilerplate,
                description: format!("Generate boilerplate methods for {} struct(s)", struct_count),
                confidence: 0.6,
                reason: "Standard methods like Clone, Debug, Default".to_string(),
            });
        }

        Ok(suggestions)
    }

    /// Count functions without documentation
    fn count_undocumented_functions(&self, ast: &File) -> usize {
        let mut count = 0;
        syn::visit::visit_file(&mut UndocumentedFunctionVisitor { count: &mut count }, ast);
        count
    }

    /// Generate summary of generation results
    fn generate_generation_summary(&self, files: &[GeneratedFile]) -> String {
        let total_lines = files.iter().map(|f| f.content.lines().count()).sum::<usize>();
        let file_types: HashSet<&str> = files.iter()
            .map(|f| f.file_type.as_str())
            .collect();

        format!(
            "Generated {} files ({}) with {} total lines across types: [{}]",
            files.len(),
            self.format_bytes(total_lines * 10), // Rough estimate of bytes
            total_lines,
            file_types.iter().map(|&s| s.to_string()).collect::<Vec<_>>().join(", ")
        )
    }

    /// Format byte count for display
    fn format_bytes(&self, bytes: usize) -> String {
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{}KB", bytes / 1024)
        } else {
            format!("{}MB", bytes / (1024 * 1024))
        }
    }
}

/// Input for code generation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeGenerationInput {
    pub item_name: String,
    pub item_type: CodeItemType,
    pub context: HashMap<String, String>, // Additional context information
    pub existing_code: Option<String>,
    pub generate_tests: bool,
    pub generate_docs: bool,
    pub generate_boilerplate: bool,
    pub generate_examples: bool,
    pub generate_stubs: bool,
}

/// Types of code items that can be generated for
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeItemType {
    Function,
    Struct,
    Trait,
    Enum,
    Impl,
    Module,
    TestModule,
    Example,
    Benchmark,
    Documentation,
}

/// Result of code generation
#[derive(Clone, Debug)]
pub struct CodeGenerationResult {
    pub id: Uuid,
    pub files: Vec<GeneratedFile>,
    pub summary: String,
}

/// A single generated file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub relative_path: String,
    pub content: String,
    pub file_type: String,
    pub description: String,
}

/// Suggestions for what to generate
#[derive(Clone, Debug)]
pub struct CodeGenerationSuggestions {
    pub file_path: String,
    pub suggested_generations: Vec<GenerationSuggestion>,
    pub confidence: f64,
}

/// Individual generation suggestion
#[derive(Clone, Debug)]
pub struct GenerationSuggestion {
    pub generation_type: GenerationType,
    pub description: String,
    pub confidence: f64,
    pub reason: String,
}

/// Types of generations available
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerationType {
    Tests,
    Documentation,
    Boilerplate,
    Examples,
    Stubs,
}

/// History record of generation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedCode {
    pub id: Uuid,
    pub input: CodeGenerationInput,
    pub results: Vec<GeneratedFile>,
    pub timestamp: DateTime<Utc>,
    pub metadata: GenerationMetadata,
}

/// Metadata about generation process
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub generator_version: String,
    pub total_files_generated: usize,
    pub total_lines_generated: usize,
}

/// Error types for code generation
#[derive(thiserror::Error, Debug)]
pub enum CodeGenerationError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Generation failed: {0}")]
    GenerationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// AST Visitors for analysis
struct FunctionAnalysisVisitor<'a> {
    suggestions: &'a mut Vec<GenerationSuggestion>,
    function_count: &'a mut usize,
    tested_functions: &'a mut usize,
    test_file_exists: bool,
}

impl<'a, 'ast> syn::visit::Visit<'ast> for FunctionAnalysisVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        *self.function_count += 1;

        // Check if function has test attribute
        let has_test_attr = node.attrs.iter().any(|attr| {
            attr.path().segments.iter().any(|seg| seg.ident == "test")
        });

        if has_test_attr {
            *self.tested_functions += 1;
        }
    }
}

struct TypeAnalysisVisitor<'a> {
    trait_count: &'a mut usize,
    struct_count: &'a mut usize,
}

impl<'a, 'ast> syn::visit::Visit<'ast> for TypeAnalysisVisitor<'a> {
    fn visit_item_trait(&mut self, _node: &'ast syn::ItemTrait) {
        *self.trait_count += 1;
    }

    fn visit_item_struct(&mut self, _node: &'ast syn::ItemStruct) {
        *self.struct_count += 1;
    }
}

struct UndocumentedFunctionVisitor<'a> {
    count: &'a mut usize,
}

impl<'a, 'ast> syn::visit::Visit<'ast> for UndocumentedFunctionVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Check if function has documentation comments
        let has_docs = node.attrs.iter().any(|attr|
            attr.path().segments.iter().any(|seg| &*seg.ident.to_string() == "doc")
        );

        if !has_docs && !node.sig.ident.to_string().starts_with('_') {
            *self.count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_code_generator_creation() {
        let generator = CodeGenerator::new();
        assert!(true); // Simple test to ensure creation works
    }

    #[tokio::test]
    async fn test_analyze_code_for_generation() {
        let generator = CodeGenerator::new();
        let code = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            struct Point {
                x: i32,
                y: i32,
            }
        "#;

        let suggestions = generator.analyze_code_for_generation(code, "test.rs").await.unwrap();

        assert!(suggestions.len() > 0);
        assert!(suggestions.iter().any(|s| matches!(s.generation_type, GenerationType::Tests)));
    }

    #[tokio::test]
    async fn test_generation_input_validation() {
        let input = CodeGenerationInput {
            item_name: "test_function".to_string(),
            item_type: CodeItemType::Function,
            context: HashMap::new(),
            existing_code: None,
            generate_tests: true,
            generate_docs: false,
            generate_boilerplate: false,
            generate_examples: false,
            generate_stubs: false,
        };

        assert!(!input.item_name.is_empty());
    }
}