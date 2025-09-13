//! # Code Generation Module
//!
//! A sophisticated AI-powered code generation system for the Rust AI IDE.
//! This module provides intelligent code generation capabilities including:
//!
//! ## Core Features
//!
//! ### 🤖 AI-Powered Code Generation
//! - **Context-aware completion**: Generate code based on current file context
//! - **Function generation**: Auto-complete function signatures and bodies
//! - **Class/struct generation**: Generate data models with appropriate fields
//! - **Test generation**: Automated unit and integration test generation
//! - **Documentation generation**: Auto-generate comprehensive documentation
//!
//! ### 🏗️ Architectural Code Generation
//! - **CRUD operations**: Generate complete CRUD implementations
//! - **API endpoints**: RESTful API endpoint generation
//! - **Database models**: Generate database integrations and migrations
//! - **Configuration**: Auto-generate configuration structures
//! - **Integration code**: Bridge code between different systems
//!
//! ### 🔧 Language-Specific Generation
//! - **Rust traits**: Generate trait implementations
//! - **Python classes**: Class generation with type hints
//! - **TypeScript interfaces**: Generate typed interfaces
//! - **SQL queries**: Generate database queries and schemas
//! - **HTML templates**: Generate web templates
//!
//! ### 📈 Advanced Features
//! - **Pattern recognition**: Learn from existing codebase patterns
//! - **Quality assurance**: Generated code quality validation
//! - **Multi-file generation**: Generate complete project structures
//! - **Error handling**: Intelligent error and exception generation
//! - **Performance optimization**: Optimize generated code for performance

pub mod architectural;
pub mod completion;
pub mod document_gen;
pub mod function_generation;
pub mod language_specific;
pub mod patterns;
pub mod quality_assurance;
pub mod test_generation;
pub mod validation;

// Re-export commonly used types for convenience
pub use architectural::ArchitecturalGenerator;
pub use completion::{CodeCompleter, CompletionContext};
pub use document_gen::DocumentationGenerator;
pub use function_generation::{FunctionGenerator, GeneratedFunction};
pub use quality_assurance::CodeQualityValidator;
pub use test_generation::{GeneratedTests, TestGenerator};

// Core context for code generation
#[derive(Debug, Clone)]
pub struct CodeGenerationContext {
    pub project_context:      ProjectContext,
    pub language:             TargetLanguage,
    pub target_scope:         GenerationScope,
    pub quality_requirements: QualityRequirements,
    pub user_preferences:     UserPreferences,
}

// Project context information
#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub project_structure: std::collections::HashMap<String, String>,
    pub dependencies:      Vec<Dependency>,
    pub existing_patterns: Vec<CodePattern>,
    pub coding_standards:  CodingStandards,
}

// Target language
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TargetLanguage {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
    CSharp,
    SQL,
    HTML,
    CSS,
    Shell,
    Other(String),
}

// Generation scope
#[derive(Debug, Clone)]
pub enum GenerationScope {
    /// Generate a single function
    Function,
    /// Generate a class/struct
    Type,
    /// Generate module/package
    Module,
    /// Generate complete API
    Api,
    /// Generate database schema and operations
    Database,
    /// Generate configuration
    Configuration,
    /// Generate tests
    Tests,
    /// Generate documentation
    Documentation,
    /// Generate complete file
    File,
}

// Quality requirements
#[derive(Debug, Clone)]
pub struct QualityRequirements {
    pub min_complexity_score:     f32,
    pub require_documentation:    bool,
    pub require_error_handling:   bool,
    pub require_tests:            bool,
    pub code_style:               String,
    pub performance_requirements: Option<PerformanceReqs>,
}

// User preferences
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub naming_convention: String,
    pub max_line_length:   usize,
    pub indentation:       String,
    pub comment_style:     String,
}

// Generated code quality assessment
#[derive(Debug, Clone)]
pub struct GenerationQuality {
    pub readability_score:     f32,
    pub maintainability_score: f32,
    pub performance_score:     f32,
    pub security_score:        f32,
    pub compliance_score:      f32,
    pub overall_score:         f32,
    pub issues:                Vec<QualityIssue>,
}

// Quality issues found during validation
#[derive(Debug, Clone)]
pub struct QualityIssue {
    pub category:   String,
    pub severity:   String,
    pub message:    String,
    pub suggestion: Option<String>,
}

// Generator metadata
#[derive(Debug, Clone)]
pub struct GeneratorMetadata {
    pub name:             String,
    pub version:          String,
    pub language_support: Vec<TargetLanguage>,
    pub description:      String,
    pub author:           String,
}

// Supporting types
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name:     String,
    pub version:  String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodePattern {
    pub pattern_type:  String,
    pub example:       String,
    pub usage_context: String,
}

#[derive(Debug, Clone)]
pub struct CodingStandards {
    pub formatting_rules:    Vec<String>,
    pub naming_rules:        Vec<String>,
    pub documentation_rules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceReqs {
    pub max_execution_time: std::time::Duration,
    pub memory_limit:       u64,
    pub thread_safety:      bool,
}

// Error types for code generation
#[derive(thiserror::Error, Debug)]
pub enum CodeGenerationError {
    #[error("Invalid context: {0}")]
    InvalidContext(String),

    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    #[error("Quality validation failed: {0}")]
    QualityValidationFailed(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Dependency conflict: {0}")]
    DependencyConflict(String),

    #[error("Code generation timeout")]
    Timeout,

    #[error("Internal generation error: {0}")]
    InternalError(String),
}

/// Main code generation service
#[derive(Debug)]
pub struct CodeGenerationService {
    function_generators: std::collections::HashMap<TargetLanguage, Vec<function_generation::FunctionGenerator>>,
}

impl CodeGenerationService {
    /// Create a new code generation service
    pub fn new() -> Self {
        Self {
            function_generators: std::collections::HashMap::new(),
        }
    }

    /// Register a function generator for a specific language
    pub fn register_function_generator(
        &mut self,
        language: TargetLanguage,
        generator: function_generation::FunctionGenerator,
    ) -> Result<(), CodeGenerationError> {
        let generators = self
            .function_generators
            .entry(language)
            .or_insert_with(Vec::new);
        generators.push(generator);
        Ok(())
    }

    /// Generate code using the appropriate generator
    pub async fn generate_code(&self, context: CodeGenerationContext) -> Result<String, CodeGenerationError> {
        let generators = self
            .function_generators
            .get(&context.language)
            .ok_or_else(|| CodeGenerationError::UnsupportedLanguage(format!("{:?}", context.language)))?;

        // For now, use the first available generator
        // In a more sophisticated implementation, we might rank/select generators
        if let Some(generator) = generators.first() {
            // Create function generation context from general context
            let function_context = function_generation::FunctionGenerationContext::default();

            let result = generator
                .generate_function(function_context)
                .await
                .map_err(|_| CodeGenerationError::InternalError("Generator failed".to_string()))?;

            // Validate generated code quality
            let quality = generator
                .validate(&result.signature)
                .map_err(|e| CodeGenerationError::QualityValidationFailed(format!("{:?}", e)))?;

            // Log quality assessment
            log::info!(
                "Generated code quality: {}%",
                (quality.overall_score * 100.0) as i32
            );

            Ok(result.signature)
        } else {
            Err(CodeGenerationError::InternalError(
                "No generator available for language".to_string(),
            ))
        }
    }

    /// Get available generators for a language
    pub fn get_function_generators(
        &self,
        language: &TargetLanguage,
    ) -> Option<&Vec<function_generation::FunctionGenerator>> {
        self.function_generators.get(language)
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<TargetLanguage> {
        self.function_generators.keys().cloned().collect()
    }
}

impl Default for CodeGenerationService {
    fn default() -> Self {
        Self::new()
    }
}

// Global code generation service instance
static SERVICE: once_cell::sync::Lazy<CodeGenerationService> = once_cell::sync::Lazy::new(|| {
    let service = CodeGenerationService::new();

    // Register default generators
    // These would be implemented in the respective modules
    // For now, we'll leave this as a placeholder

    service
});

/// Get the global code generation service instance
pub fn get_global_service() -> &'static CodeGenerationService {
    &SERVICE
}
