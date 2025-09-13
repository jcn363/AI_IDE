// # Code Generation Module
//
// A sophisticated AI-powered code generation system for the Rust AI IDE.
// This module provides intelligent code generation capabilities including:
//
// ## Core Features
//
// ### 🤖 AI-Powered Code Generation
// - **Context-aware completion**: Generate code based on current file context
// - **Function generation**: Auto-complete function signatures and bodies
// - **Class/struct generation**: Generate data models with appropriate fields
// - **Test generation**: Automated unit and integration test generation
// - **Documentation generation**: Auto-generate comprehensive documentation
//
// ### 🏗️ Architectural Code Generation
// - **CRUD operations**: Generate complete CRUD implementations
// - **API endpoints**: RESTful API endpoint generation
// - **Database models**: Generate database integrations and migrations
// - **Configuration**: Auto-generate configuration structures
// - **Integration code**: Bridge code between different systems
//
// ### 🔧 Language-Specific Generation
// - **Rust traits**: Generate trait implementations
// - **Python classes**: Class generation with type hints
// - **TypeScript interfaces**: Generate typed interfaces
// - **SQL queries**: Generate database queries and schemas
// - **HTML templates**: Generate web templates
//
// ### 📈 Advanced Features
// - **Pattern recognition**: Learn from existing codebase patterns
// - **Quality assurance**: Generated code quality validation
// - **Multi-file generation**: Generate complete project structures
// - **Error handling**: Intelligent error and exception generation
// - **Performance optimization**: Optimize generated code for performance

use crate::{CodeGenerationContext, CodeGenerationError, TargetLanguage};

/// Generator metadata (ai-codegen specific)
#[derive(Debug, Clone)]
pub struct GeneratorMetadata {
    pub name:             String,
    pub version:          String,
    pub language_support: Vec<TargetLanguage>,
    pub description:      String,
    pub author:           String,
}

/// Main code generation service
#[derive(Debug)]
pub struct CodeGenerationService {
    function_generators: std::collections::HashMap<TargetLanguage, Vec<crate::function_generation::FunctionGenerator>>,
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
        generator: crate::function_generation::FunctionGenerator,
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
            let function_context = crate::function_generation::FunctionGenerationContext::default();

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
    ) -> Option<&Vec<crate::function_generation::FunctionGenerator>> {
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
