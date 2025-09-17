//! # Rust AI IDE AI Code Generation
//!
//! An AI-powered multi-language code generation system with intelligent
//! context-aware capabilities for the Rust AI IDE.
//!
//! ## Features
//!
//! - **Context-aware Code Generation**: Generate code from natural language specifications
//! - **Intelligent Code Completion**: AI-powered suggestions for partial code
//! - **Automated Test Generation**: Create comprehensive tests from code analysis
//! - **Safe Refactoring**: Intelligent refactoring with impact analysis
//! - **Documentation Generation**: Automated documentation from code analysis
//! - **Security Validation**: Built-in security checking and validation
//! - **Performance Optimization**: Caching and efficient resource usage
//!
//! ## Architecture
//!
//! This crate provides:
//! - `CodeGenerator`: Context-aware code generation from natural language
//! - `CompletionEngine`: Intelligent code completion and suggestions
//! - `TestGenerator`: Automated test generation based on analysis
//! - `RefactoringEngine`: Safe refactoring suggestions and transformations
//! - `DocumentationGenerator`: Automated documentation generation
//!
//! ## Usage
//!
//! ```ignore
//! use rust_ai_ide_ai_codegen::*;
//!
//! let generator = CodeGenerator::new().await?;
//! let code = generator.generate_from_spec("Create a user authentication function").await?;
//! ```

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Configuration for AI code generation
#[derive(Debug, Clone)]
pub struct CodegenConfig {
    /// Maximum generation latency in milliseconds
    pub max_latency_ms: u64,
    /// Quality threshold for generated code
    pub quality_threshold: f64,
    /// Enable caching for performance
    pub enable_caching: bool,
    /// Security validation level
    pub security_level: SecurityLevel,
    /// Performance requirements
    pub performance_reqs: PerformanceReqs,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self {
            max_latency_ms: 2000,
            quality_threshold: 0.8,
            enable_caching: true,
            security_level: SecurityLevel::High,
            performance_reqs: PerformanceReqs::default(),
        }
    }
}

/// Security level for validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Strict,
}

/// Performance requirements
#[derive(Debug, Clone, Default)]
pub struct PerformanceReqs {
    pub memory_limit_mb: Option<usize>,
    pub max_concurrent_requests: Option<usize>,
}

// Re-export shared types for convenience
pub use rust_ai_ide_shared_codegen::generator::{
    CodeGenerationContext, CodeGenerationContextBuilder, CodeGenerationError, CodePattern, CodingStandards, Dependency,
    GenerationQuality, GenerationScope, PerformanceReqs as SharedPerformanceReqs, ProjectContext, QualityIssue, QualityRequirements,
    TargetLanguage, UserPreferences,
};
pub use rust_ai_ide_shared_codegen::traits::{CodeAnalyzer, CodeAnalysisResult, FunctionSignature, ClassDefinition};

// Re-export from submodules
pub use error::{CodegenError, Result};

// Re-export types
pub use types::*;

/// Convert shared-codegen TargetLanguage to common ProgrammingLanguage
fn target_to_programming_language(target: &TargetLanguage) -> rust_ai_ide_common::types::ProgrammingLanguage {
    match target {
        TargetLanguage::Rust => rust_ai_ide_common::types::ProgrammingLanguage::Rust,
        TargetLanguage::Python => rust_ai_ide_common::types::ProgrammingLanguage::Python,
        TargetLanguage::JavaScript => rust_ai_ide_common::types::ProgrammingLanguage::JavaScript,
        TargetLanguage::TypeScript => rust_ai_ide_common::types::ProgrammingLanguage::TypeScript,
        TargetLanguage::Java => rust_ai_ide_common::types::ProgrammingLanguage::Java,
        TargetLanguage::CSharp => rust_ai_ide_common::types::ProgrammingLanguage::CSharp,
        TargetLanguage::Go => rust_ai_ide_common::types::ProgrammingLanguage::Go,
        TargetLanguage::Cpp => rust_ai_ide_common::types::ProgrammingLanguage::Cpp,
        _ => rust_ai_ide_common::types::ProgrammingLanguage::Unknown,
    }
}

/// Real AI Inference Service implementation using ai-inference crate
pub struct RealInferenceService {
    nl_converter: Arc<rust_ai_ide_ai_inference::natural_language_to_code::NLToCodeConverter>,
    inference_engine: Arc<rust_ai_ide_ai_inference::InferenceEngine>,
    completion_engine: Arc<rust_ai_ide_ai_inference::predictive_completion::PredictiveCompletionEngine>,
}

impl RealInferenceService {
    pub async fn new() -> Result<Self> {
        // Initialize the AI inference system
        rust_ai_ide_ai_inference::init_inference_system().await
            .map_err(|e| CodegenError::AiInferenceError(format!("Failed to initialize AI system: {}", e)))?;

        // Create NL to code converter
        let nl_converter = Arc::new(rust_ai_ide_ai_inference::natural_language_to_code::create_nl_to_code_converter());

        // Get inference engine (it should be initialized globally)
        let inference_engine = rust_ai_ide_ai_inference::INFERENCE_ENGINE.clone();

        // Create completion engine with default config
        let completion_config = rust_ai_ide_ai_inference::predictive_completion::CompletionConfig::default();
        let completion_engine = Arc::new(
            rust_ai_ide_ai_inference::predictive_completion::PredictiveCompletionEngine::new().await
                .map_err(|e| CodegenError::AiInferenceError(format!("Failed to create completion engine: {}", e)))?
        );

        Ok(Self {
            nl_converter,
            inference_engine,
            completion_engine,
        })
    }

    /// Load a code generation model for local inference
    pub async fn load_code_generation_model(&self, model_path: &str) -> Result<String> {
        let config = rust_ai_ide_ai_inference::ModelLoadConfig {
            quantization: rust_ai_ide_ai_inference::QuantizationLevel::None,
            backend: rust_ai_ide_ai_inference::HardwareBackend::Cpu, // Start with CPU for safety
            max_memory_mb: 1024,
            enable_profiling: false,
        };

        // Use ONNX loader as it's the most compatible
        let loader = rust_ai_ide_ai_inference::ONNXLoader;

        self.inference_engine.load_model(&loader, model_path, &config).await
            .map_err(|e| CodegenError::AiInferenceError(format!("Failed to load model: {}", e)))
    }
}

#[async_trait::async_trait]
pub trait InferenceService: Send + Sync {
    async fn analyze_code_spec(&self, spec: &str) -> crate::Result<CodeGenerationContext>;
    async fn generate_code(&self, context: CodeGenerationContext) -> crate::Result<GeneratedCode>;
    async fn generate_completions(&self, context: CompletionContext) -> crate::Result<Vec<CompletionSuggestion>>;
    async fn generate_code_from_spec(&self, spec: &str) -> crate::Result<GeneratedCode>;
}

#[async_trait::async_trait]
impl InferenceService for RealInferenceService {
    async fn analyze_code_spec(&self, spec: &str) -> crate::Result<CodeGenerationContext> {
        // Use NL converter to analyze the specification
        let input = rust_ai_ide_ai_inference::natural_language_to_code::NLToCodeInput {
            description: spec.to_string(),
            target_language: "rust".to_string(), // Default to Rust, will be updated based on analysis
            project_context: std::collections::HashMap::new(),
            coding_style: None,
            existing_code: None,
            requirements: vec![],
        };

        let result = self.nl_converter.convert(input).await
            .map_err(|e| CodegenError::AiInferenceError(format!("NL analysis failed: {}", e)))?;

        // Convert the result to our CodeGenerationContext
        Ok(CodeGenerationContext {
            language: match result.language.as_str() {
                "rust" => TargetLanguage::Rust,
                "python" => TargetLanguage::Python,
                "javascript" => TargetLanguage::JavaScript,
                "typescript" => TargetLanguage::TypeScript,
                "java" => TargetLanguage::Java,
                "csharp" => TargetLanguage::CSharp,
                "go" => TargetLanguage::Go,
                "cpp" => TargetLanguage::Cpp,
                _ => TargetLanguage::Rust, // Default fallback
            },
            target_scope: GenerationScope::Function, // Could be enhanced to detect from NL
            quality_requirements: QualityRequirements::default(),
            project_context: ProjectContext::default(),
            user_preferences: UserPreferences::default(),
        })
    }

    async fn generate_code(&self, context: CodeGenerationContext) -> crate::Result<GeneratedCode> {
        // Create NL description from context
        let language_str = match context.language {
            TargetLanguage::Rust => "rust",
            TargetLanguage::Python => "python",
            TargetLanguage::JavaScript => "javascript",
            TargetLanguage::TypeScript => "typescript",
            TargetLanguage::Java => "java",
            TargetLanguage::CSharp => "csharp",
            TargetLanguage::Go => "go",
            TargetLanguage::Cpp => "cpp",
            _ => "rust",
        };

        let description = format!("Generate {} code for a {} implementation",
                                language_str,
                                match context.target_scope {
                                    GenerationScope::Function => "function",
                                    _ => "component",
                                });

        let input = rust_ai_ide_ai_inference::natural_language_to_code::NLToCodeInput {
            description,
            target_language: language_str.to_string(),
            project_context: std::collections::HashMap::new(),
            coding_style: None,
            existing_code: None,
            requirements: vec![],
        };

        let result = self.nl_converter.convert(input).await
            .map_err(|e| CodegenError::AiInferenceError(format!("Code generation failed: {}", e)))?;

        Ok(GeneratedCode {
            content: result.code,
            language: context.language,
            quality_score: result.confidence_score,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("explanation".to_string(), serde_json::Value::String(result.explanation));
                meta.insert("warnings".to_string(), serde_json::Value::String(result.warnings.join("; ")));
                meta.insert("alternatives_count".to_string(), serde_json::Value::Number(result.alternatives.len().into()));
                meta
            },
        })
    }

    async fn generate_completions(&self, context: CompletionContext) -> crate::Result<Vec<CompletionSuggestion>> {
        // Convert our CompletionContext to the ai-inference CompletionContext
        let ai_completion_context = rust_ai_ide_ai_inference::predictive_completion::CompletionContext {
            prefix: context.prefix,
            suffix: context.suffix,
            position: rust_ai_ide_ai_inference::predictive_completion::Position {
                line: context.position.line as u32,
                character: context.position.column as u32,
            },
            file_info: rust_ai_ide_ai_inference::predictive_completion::FileInfo {
                path: context.file_path.unwrap_or_default(),
                language: context.language.to_string(),
                encoding: "utf-8".to_string(),
                size_bytes: 0,
                last_modified: chrono::Utc::now(),
                dependencies: vec![],
            },
            recent_changes: vec![],
            scope_context: rust_ai_ide_ai_inference::predictive_completion::ScopeContext {
                function_name: None,
                function_signature: None,
                class_name: None,
                namespace: None,
                accessible_symbols: vec![],
                variable_types: std::collections::HashMap::new(),
                import_statements: vec![],
            },
            symbol_context: rust_ai_ide_ai_inference::predictive_completion::SymbolContext {
                local_symbols: std::collections::HashSet::new(),
                global_symbols: std::collections::HashSet::new(),
                imported_symbols: std::collections::HashSet::new(),
                type_definitions: std::collections::HashMap::new(),
                module_functions: std::collections::HashMap::new(),
            },
            user_profile: rust_ai_ide_ai_inference::predictive_completion::UserProfile {
                coding_style: rust_ai_ide_ai_inference::predictive_completion::CodingStyle::Functional,
                preferred_libraries: vec![],
                naming_conventions: rust_ai_ide_ai_inference::predictive_completion::NamingConvention::SnakeCase,
                indentation_style: rust_ai_ide_ai_inference::predictive_completion::IndentationStyle::Spaces { width: 4 },
                language_proficiency: std::collections::HashMap::new(),
            },
            security_context: rust_ai_ide_ai_inference::predictive_completion::SecurityContext {
                restricted_keywords: std::collections::HashSet::new(),
                allowed_patterns: vec![],
                confidence_threshold: 0.5,
                privacy_level: rust_ai_ide_ai_inference::predictive_completion::PrivacyLevel::Public,
            },
        };

        let ai_completions = self.completion_engine.generate_completions(ai_completion_context).await
            .map_err(|e| CodegenError::AiInferenceError(format!("Completion generation failed: {}", e)))?;

        // Convert back to our CompletionSuggestion format
        let suggestions = ai_completions.suggestions.into_iter()
            .map(|completion| CompletionSuggestion {
                text: completion.completion_text.clone(),
                kind: match completion.completion_type {
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Function => CompletionKind::Function,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Method => CompletionKind::Method,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Class => CompletionKind::Class,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Module => CompletionKind::Module,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Property => CompletionKind::Property,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Class => CompletionKind::Class,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Module => CompletionKind::Module,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Function => CompletionKind::Function,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    rust_ai_ide_ai_inference::predictive_completion::CompletionType::Variable => CompletionKind::Variable,
                    _ => CompletionKind::Text,
                },
                detail: completion.display_text,
                documentation: completion.documentation,
                sort_text: Some(completion.sort_priority.to_string()),
                filter_text: Some(completion.completion_text),
            })
            .collect();

        Ok(suggestions)
    }

    async fn generate_code_from_spec(&self, spec: &str) -> crate::Result<GeneratedCode> {
        let input = rust_ai_ide_ai_inference::natural_language_to_code::NLToCodeInput {
            description: spec.to_string(),
            target_language: "rust".to_string(), // Default, could be made configurable
            project_context: std::collections::HashMap::new(),
            coding_style: None,
            existing_code: None,
            requirements: vec![],
        };

        let result = self.nl_converter.convert(input).await
            .map_err(|e| CodegenError::AiInferenceError(format!("Code generation from spec failed: {}", e)))?;

        Ok(GeneratedCode {
            content: result.code,
            language: TargetLanguage::Rust, // Could be enhanced to detect from result
            quality_score: result.confidence_score,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("explanation".to_string(), serde_json::Value::String(result.explanation));
                meta.insert("warnings".to_string(), serde_json::Value::String(result.warnings.join("; ")));
                meta.insert("alternatives_count".to_string(), serde_json::Value::Number(result.alternatives.len().into()));
                meta.insert("generation_id".to_string(), serde_json::Value::String(result.metadata.generation_id));
                meta
            },
        })
    }
}

/// Create a real inference service using the ai-inference infrastructure
pub async fn create_inference_service() -> Result<Arc<dyn InferenceService>> {
    let service = RealInferenceService::new().await
        .map_err(|e| CodegenError::AiInferenceError(format!("Failed to create real inference service: {}", e)))?;
    Ok(Arc::new(service))
}

// Module declarations
pub mod error;
pub mod types;
pub mod cache;
pub mod security;
pub mod performance;

#[cfg(feature = "templates")]
pub mod templates;

// Import the types module for CompletionContext and other types
use crate::types::*;

// Re-export cache functionality
pub use cache::CodegenCache;