//! Core types and traits for the DSL code generation system

use crate::ast::*;
use crate::error::DslResult;
use async_trait::async_trait;
use rust_ai_ide_cache::Cache;
use rust_ai_ide_common::{GeneratedTest, ProgrammingLanguage};
use rust_ai_ide_lsp::diagnostics::AIAnalysisResult;
use std::collections::HashMap;

/// Core trait for DSL templates - executable code generation units
#[async_trait]
pub trait DslTemplate: Send + Sync + 'static {
    /// Get the template name
    fn name(&self) -> &str;

    /// Get the template description
    fn description(&self) -> Option<&str>;

    /// Get supported programming languages for this template
    fn supported_languages(&self) -> Vec<ProgrammingLanguage>;

    /// Get the template parameters
    fn parameters(&self) -> &[Parameter];

    /// Validate template parameters before execution
    async fn validate_parameters(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> DslResult<()>;

    /// Execute the template with given parameters and context
    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
        context: &GenerationContext,
    ) -> DslResult<GeneratedCode>;
}

/// Execution context for template generation
#[derive(Clone)]
pub struct GenerationContext {
    /// Target programming language
    pub language: ProgrammingLanguage,
    /// Generation configuration
    pub config: GenerationConfig,
    /// Current workspace information
    pub workspace_root: std::path::PathBuf,
    /// AI analysis results (for context-aware generation)
    pub ai_analysis: Option<AIAnalysisResult>,
    /// Cache for template results
    pub cache: Option<std::sync::Arc<dyn Cache<String, serde_json::Value>>>,
}

/// Configuration for code generation
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// Enable security validation
    pub security_validation: bool,
    /// Enable AI-enhanced suggestions
    pub ai_suggestions: bool,
    /// Generate tests alongside code
    pub generate_tests: bool,
    /// Validation level (strict, relaxed, none)
    pub validation_level: ValidationLevel,
    /// Custom generation settings
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            security_validation: true,
            ai_suggestions: true,
            generate_tests: false,
            validation_level: ValidationLevel::Strict,
            custom_settings: HashMap::new(),
        }
    }
}

/// Validation strictness levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationLevel {
    /// Strict validation - all rules must pass
    Strict,
    /// Relaxed validation - warnings allowed
    Relaxed,
    /// No validation - skip all checks
    None,
}

/// Generated code result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeneratedCode {
    /// The generated code content
    pub code: String,
    /// Language-specific information
    pub language: ProgrammingLanguage,
    /// Generated imports/dependencies
    pub imports: Vec<String>,
    /// Generated tests (if requested)
    pub tests: Vec<GeneratedTest>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Generation metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// DSL plugin interface for extensibility
#[async_trait]
pub trait DslPlugin: Send + Sync + std::fmt::Debug + 'static {
    /// Plugin identifier
    fn id(&self) -> &str;

    /// Plugin name
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// Get supported template types
    fn supported_templates(&self) -> Vec<String>;

    /// Create a template instance
    async fn create_template(&self, name: &str, ast: &Template) -> DslResult<Box<dyn DslTemplate>>;

    /// Validate plugin-specific parameters
    async fn validate_parameters(
        &self,
        template_name: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> DslResult<()>;
}

/// DSL engine that orchestrates template execution and plugins
#[async_trait]
pub trait DslEngine: Send + Sync + 'static {
    /// Register a DSL plugin
    async fn register_plugin(&mut self, plugin: Box<dyn DslPlugin>) -> DslResult<()>;

    /// Load a template from DSL source
    async fn load_template(&self, dsl_source: &str) -> DslResult<Box<dyn DslTemplate>>;

    /// Execute a template by name with parameters
    async fn execute_template(
        &self,
        template_name: &str,
        params: HashMap<String, serde_json::Value>,
        context: &GenerationContext,
    ) -> DslResult<GeneratedCode>;

    /// Get available templates
    fn available_templates(&self) -> Vec<String>;

    /// Get template information
    async fn template_info(&self, name: &str) -> DslResult<TemplateInfo>;
}

/// Template information for discovery and introspection
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// Template name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Supported languages
    pub supported_languages: Vec<ProgrammingLanguage>,
    /// Required parameters
    pub parameters: Vec<Parameter>,
    /// Associated patterns
    pub patterns: Vec<String>,
    /// Version information
    pub version: Option<String>,
}

/// Builder pattern for creating DSL templates
pub struct DslTemplateBuilder {
    name: String,
    description: Option<String>,
    parameters: Vec<Parameter>,
    guards: Vec<Guard>,
    generate_block: GenerateBlock,
    patterns: Vec<String>,
}

impl DslTemplateBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters: Vec::new(),
            guards: Vec::new(),
            generate_block: GenerateBlock::default(),
            patterns: Vec::new(),
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn parameter(mut self, param: Parameter) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn guard(mut self, guard: Guard) -> Self {
        self.guards.push(guard);
        self
    }

    pub fn generate(mut self, block: GenerateBlock) -> Self {
        self.generate_block = block;
        self
    }

    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    pub fn build(self) -> Template {
        Template {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
            guards: self.guards,
            generate: self.generate_block,
            patterns: self.patterns,
            metadata: Vec::new(),
            location: None,
        }
    }
}

impl Default for GenerationContext {
    fn default() -> Self {
        Self {
            language: ProgrammingLanguage::Rust,
            config: GenerationConfig::default(),
            workspace_root: std::env::current_dir().unwrap_or_default(),
            ai_analysis: None,
            cache: None,
        }
    }
}

/// Helper trait for converting DSL types to common types
pub trait ToCommonType {
    type Target;
    fn to_common(&self) -> Self::Target;
}

impl ToCommonType for ParameterType {
    type Target = String;

    fn to_common(&self) -> String {
        match self {
            ParameterType::String => "String".to_string(),
            ParameterType::Integer => "i64".to_string(),
            ParameterType::Boolean => "bool".to_string(),
            ParameterType::Float => "f64".to_string(),
            ParameterType::Array(inner) => format!("Vec<{}>", inner.to_common()),
            ParameterType::Custom(name) => name.clone(),
            ParameterType::ProgrammingLanguage => "ProgrammingLanguage".to_string(),
            ParameterType::Identifier(name) => name.clone(),
        }
    }
}

impl ToCommonType for ProgrammingLanguage {
    type Target = rust_ai_ide_lsp::LanguageServerKind;

    fn to_common(&self) -> rust_ai_ide_lsp::LanguageServerKind {
        use rust_ai_ide_lsp::LanguageServerKind;
        match self {
            ProgrammingLanguage::Rust => LanguageServerKind::Rust,
            ProgrammingLanguage::TypeScript => LanguageServerKind::TypeScript,
            ProgrammingLanguage::JavaScript => LanguageServerKind::JavaScript,
            ProgrammingLanguage::Python => LanguageServerKind::Python,
            // Stub mappings for missing variants - can be updated when LSP crate adds them
            ProgrammingLanguage::Java => LanguageServerKind::JavaScript, // placeholder
            ProgrammingLanguage::CSharp => LanguageServerKind::JavaScript, // placeholder
            ProgrammingLanguage::Go => LanguageServerKind::JavaScript,   // placeholder
            ProgrammingLanguage::Cpp => LanguageServerKind::JavaScript,  // placeholder
            ProgrammingLanguage::C => LanguageServerKind::JavaScript,    // placeholder
            ProgrammingLanguage::Unknown => LanguageServerKind::Python,  // placeholder
            _ => LanguageServerKind::Python, // handle any future variants
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_builder() {
        let template = DslTemplateBuilder::new("TestTemplate")
            .description("A test template")
            .pattern("test")
            .build();

        assert_eq!(template.name, "TestTemplate");
        assert_eq!(template.description, Some("A test template".to_string()));
        assert_eq!(template.patterns, vec!["test"]);
    }

    #[test]
    fn test_parameter_type_conversion() {
        assert_eq!(ParameterType::String.to_common(), "String");
        assert_eq!(ParameterType::Integer.to_common(), "i64");
        assert_eq!(ParameterType::Boolean.to_common(), "bool");

        let array_type = ParameterType::Array(Box::new(ParameterType::String));
        assert_eq!(array_type.to_common(), "Vec<String>");

        let custom_type = ParameterType::Custom("MyType".to_string());
        assert_eq!(custom_type.to_common(), "MyType");
    }
}
