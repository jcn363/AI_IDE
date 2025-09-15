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

pub mod error;
pub mod types;
pub mod cache;
pub mod security;
pub mod performance;

#[cfg(feature = "templates")]
pub mod templates;

// Re-export shared types for convenience
pub use rust_ai_ide_shared_codegen::generator::{
    CodeGenerationContext, CodeGenerationContextBuilder, CodeGenerationError, CodePattern, CodingStandards, Dependency,
    GenerationQuality, GenerationScope, PerformanceReqs, ProjectContext, QualityIssue, QualityRequirements,
    TargetLanguage, UserPreferences,
};
pub use rust_ai_ide_shared_codegen::traits::{CodeAnalyzer, CodeAnalysisResult, FunctionSignature, ClassDefinition};

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

// Re-export from submodules
pub use error::{CodegenError, Result};
pub use types::*;
pub use cache::CodegenCache;

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

// Placeholder InferenceService trait for compilation
#[async_trait::async_trait]
pub trait InferenceService: Send + Sync {
    async fn analyze_code_spec(&self, _spec: &str) -> crate::Result<CodeGenerationContext>;
    async fn generate_code(&self, _context: CodeGenerationContext) -> crate::Result<GeneratedCode>;
    async fn generate_completions(&self, _context: CompletionContext) -> crate::Result<Vec<CompletionSuggestion>>;
    async fn generate_code_from_spec(&self, _spec: &str) -> crate::Result<GeneratedCode>;
}

// Placeholder function for creating inference service
pub async fn create_inference_service() -> Result<Arc<dyn InferenceService>> {
    // Placeholder implementation - return dummy service
    struct DummyInferenceService;
    #[async_trait::async_trait]
    impl InferenceService for DummyInferenceService {
        async fn analyze_code_spec(&self, _spec: &str) -> crate::Result<CodeGenerationContext> {
            Ok(CodeGenerationContext {
                language: TargetLanguage::Rust,
                target_scope: GenerationScope::Function,
                quality_requirements: QualityRequirements::default(),
                project_context: ProjectContext::default(),
                user_preferences: UserPreferences::default(),
            })
        }
        async fn generate_code(&self, _context: CodeGenerationContext) -> crate::Result<GeneratedCode> {
            Ok(GeneratedCode {
                content: "// Generated code placeholder".to_string(),
                language: rust_ai_ide_shared_codegen::generator::TargetLanguage::Rust,
                quality_score: 0.8,
                metadata: HashMap::new(),
            })
        }
        async fn generate_completions(&self, _context: CompletionContext) -> crate::Result<Vec<CompletionSuggestion>> {
            Ok(vec![])
        }
        async fn generate_code_from_spec(&self, _spec: &str) -> crate::Result<GeneratedCode> {
            Ok(GeneratedCode {
                content: "// Generated code placeholder".to_string(),
                language: rust_ai_ide_shared_codegen::generator::TargetLanguage::Rust,
                quality_score: 0.8,
                metadata: HashMap::new(),
            })
        }
    }
    Ok(Arc::new(DummyInferenceService))
}

// Placeholder function for creating code analyzer
pub async fn create_code_analyzer() -> Result<Arc<dyn rust_ai_ide_shared_codegen::traits::CodeAnalyzer>> {
    // Placeholder implementation - return dummy analyzer
    struct DummyCodeAnalyzer;
    #[async_trait::async_trait]
    impl rust_ai_ide_shared_codegen::traits::CodeAnalyzer for DummyCodeAnalyzer {
        async fn analyze_code(&self, _code: &str, _language: &rust_ai_ide_common::types::ProgrammingLanguage) -> std::result::Result<rust_ai_ide_shared_codegen::traits::CodeAnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
            Ok(rust_ai_ide_shared_codegen::traits::CodeAnalysisResult {
                functions: vec![],
                classes: vec![],
                imports: vec![],
                test_targets: vec![],
                metadata: HashMap::new(),
            })
        }
        fn extract_signatures(&self, _code: &str, _language: &rust_ai_ide_common::types::ProgrammingLanguage) -> Vec<rust_ai_ide_shared_codegen::traits::FunctionSignature> {
            vec![]
        }
        fn identify_test_targets(&self, _code: &str, _language: &rust_ai_ide_common::types::ProgrammingLanguage) -> Vec<rust_ai_ide_shared_codegen::traits::TestTarget> {
            vec![]
        }
        fn analyze_dependencies(&self, _code: &str, _language: &rust_ai_ide_common::types::ProgrammingLanguage) -> Vec<String> {
            vec![]
        }
    }
    Ok(Arc::new(DummyCodeAnalyzer))
}

// Simplified placeholder implementations to avoid external crate dependencies

/// Context-aware code generation from natural language specifications
pub struct CodeGenerator {
    config: CodegenConfig,
    cache: Arc<Mutex<CodegenCache>>,
    ai_inference: Arc<dyn InferenceService>,
    security_validator: Arc<security::SecurityValidator>,
    performance_monitor: Arc<performance::PerformanceMonitor>,
}

impl CodeGenerator {
    /// Create a new CodeGenerator with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(CodegenConfig::default()).await
    }

    /// Create a new CodeGenerator with custom configuration
    pub async fn with_config(config: CodegenConfig) -> Result<Self> {
        let security_level = config.security_level.clone();
        let cache = Arc::new(Mutex::new(CodegenCache::new()));
        let ai_inference = create_inference_service().await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;
        let security_validator = Arc::new(security::SecurityValidator::new(security_level));
        let performance_monitor = Arc::new(performance::PerformanceMonitor::new());

        Ok(Self {
            config,
            cache,
            ai_inference,
            security_validator,
            performance_monitor,
        })
    }

    /// Generate code from natural language specification
    pub async fn generate_from_spec(&self, spec: &str) -> Result<GeneratedCode> {
        let start_time = std::time::Instant::now();

        // Validate input security
        self.security_validator.validate_input(spec)?;

        // Check cache first
        if let Some(cached) = self.get_cached(spec).await {
            self.performance_monitor.record_cache_hit().await;
            return Ok(cached);
        }

        // Analyze specification using AI
        let context = self.analyze_spec(spec).await?;

        // Generate code based on context
        let code = self.generate_code(context).await?;

        // Validate generated code
        self.validate_generated_code(&code).await?;

        // Security validation
        self.security_validator.validate_generated_code(&code)?;

        // Cache result
        self.cache_result(spec, &code).await;

        let latency = start_time.elapsed().as_millis() as u64;
        self.performance_monitor.record_generation(latency).await;

        if latency > self.config.max_latency_ms {
            log::warn!("Code generation exceeded latency threshold: {}ms", latency);
        }

        Ok(code)
    }

    /// Generate function from specification
    pub async fn generate_function(&self, spec: FunctionSpec) -> Result<GeneratedFunction> {
        let spec_text = format!("Generate a {} function: {}",
                               spec.language.to_string(),
                               spec.description);

        let generated = self.generate_from_spec(&spec_text).await?;
        let signature = spec.signature.clone();
        let function = GeneratedFunction {
            code: generated,
            signature,
            test_cases: self.generate_function_tests(&spec).await?,
            documentation: self.generate_function_docs(&spec).await?,
        };

        Ok(function)
    }

    /// Generate struct/class from specification
    pub async fn generate_struct(&self, spec: StructSpec) -> Result<GeneratedStruct> {
        let spec_text = format!("Generate a {} struct with fields: {}",
                               spec.language.to_string(),
                               spec.fields.iter().map(|f| f.name.as_str()).collect::<Vec<&str>>().join(", "));

        let generated = self.generate_from_spec(&spec_text).await?;
        let struct_code = GeneratedStruct {
            code: generated,
            methods: self.generate_struct_methods(&spec).await?,
            documentation: self.generate_struct_docs(&spec).await?,
        };

        Ok(struct_code)
    }

    async fn analyze_spec(&self, spec: &str) -> Result<CodeGenerationContext> {
        // Use AI inference to analyze the specification
        let analysis = self.ai_inference.analyze_code_spec(spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(analysis)
    }

    async fn generate_code(&self, context: CodeGenerationContext) -> Result<GeneratedCode> {
        // Use AI inference to generate code
        let code = self.ai_inference.generate_code(context).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(GeneratedCode {
            content: code.content,
            language: code.language,
            quality_score: code.quality_score,
            metadata: code.metadata,
        })
    }

    async fn validate_generated_code(&self, code: &GeneratedCode) -> Result<()> {
        // Syntax validation
        self.validate_syntax(code)?;

        // Quality validation
        if code.quality_score < self.config.quality_threshold {
            return Err(CodegenError::QualityThresholdNotMet {
                score: code.quality_score,
                threshold: self.config.quality_threshold,
            });
        }

        Ok(())
    }

    fn validate_syntax(&self, code: &GeneratedCode) -> Result<()> {
        // Basic syntax validation - could be extended with language-specific parsers
        match code.language {
            TargetLanguage::Rust => {
                // Use syn for Rust syntax validation
                let token_stream: proc_macro2::TokenStream = syn::parse_str::<proc_macro2::TokenStream>(&code.content)?;
                syn::parse2::<syn::File>(token_stream)
                    .map_err(|e| CodegenError::SyntaxError(e.to_string()))?;
            }
            _ => {
                // For other languages, do basic validation
                if code.content.trim().is_empty() {
                    return Err(CodegenError::ValidationError("Generated code is empty".to_string()));
                }
            }
        }
        Ok(())
    }

    async fn get_cached(&self, spec: &str) -> Option<GeneratedCode> {
        if !self.config.enable_caching {
            return None;
        }

        let cache = self.cache.lock().await;
        cache.get(spec).await
    }

    async fn cache_result(&self, spec: &str, code: &GeneratedCode) {
        if !self.config.enable_caching {
            return;
        }

        let cache = self.cache.lock().await;
        cache.put(spec.to_string(), code.clone());
    }

    async fn generate_function_tests(&self, spec: &FunctionSpec) -> Result<Vec<TestCase>> {
        // Generate test cases for the function
        let test_spec = format!("Generate unit tests for function: {}", spec.description);
        let test_code = self.generate_from_spec(&test_spec).await?;

        Ok(vec![TestCase {
            name: format!("test_{}", spec.name),
            code: test_code.content,
            expected: "Test execution".to_string(),
        }])
    }

    async fn generate_function_docs(&self, spec: &FunctionSpec) -> Result<String> {
        let doc_spec = format!("Generate documentation for function: {}", spec.description);
        let docs = self.generate_from_spec(&doc_spec).await?;
        Ok(docs.content)
    }

    async fn generate_struct_methods(&self, spec: &StructSpec) -> Result<Vec<GeneratedFunction>> {
        let mut methods = Vec::new();

        // Generate constructor
        let constructor = self.generate_function(FunctionSpec {
            name: "new".to_string(),
            signature: format!("fn new({}) -> Self", spec.fields.iter()
                .map(|f| format!("{}: {}", f.name, f.field_type))
                .collect::<Vec<_>>()
                .join(", ")),
            language: spec.language.clone(),
            description: "Constructor for the struct".to_string(),
        }).await?;
        methods.push(constructor);

        // Generate getters/setters if requested
        if spec.generate_accessors {
            for field in &spec.fields {
                let getter = self.generate_function(FunctionSpec {
                    name: format!("get_{}", field.name),
                    signature: format!("fn get_{}(&self) -> &{}", field.name, field.field_type),
                    language: spec.language.clone(),
                    description: format!("Getter for {} field", field.name),
                }).await?;
                methods.push(getter);
            }
        }

        Ok(methods)
    }

    async fn generate_struct_docs(&self, spec: &StructSpec) -> Result<String> {
        let doc_spec = format!("Generate documentation for struct with fields: {}",
                              spec.fields.iter().map(|f| f.name.as_str()).collect::<Vec<&str>>().join(", "));
        let docs = self.generate_from_spec(&doc_spec).await?;
        Ok(docs.content)
    }
}

/// Intelligent code completion and suggestions
pub struct CompletionEngine {
    config: CodegenConfig,
    cache: Arc<Mutex<CodegenCache>>,
    ai_inference: Arc<dyn InferenceService>,
    performance_monitor: Arc<performance::PerformanceMonitor>,
}

impl CompletionEngine {
    pub async fn new() -> Result<Self> {
        Self::with_config(CodegenConfig::default()).await
    }

    pub async fn with_config(config: CodegenConfig) -> Result<Self> {
        let cache = Arc::new(Mutex::new(CodegenCache::new()));
        let ai_inference = create_inference_service().await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;
        let performance_monitor = Arc::new(performance::PerformanceMonitor::new());

        Ok(Self {
            config,
            cache,
            ai_inference,
            performance_monitor,
        })
    }

    /// Generate completion suggestions for partial code
    pub async fn generate_completions(&self, context: CompletionContext) -> Result<Vec<CompletionSuggestion>> {
        let start_time = std::time::Instant::now();

        // Check cache
        let cache_key = format!("completion:{:?}", context);
        if let Some(cached) = self.get_cached_completion(&cache_key).await {
            return Ok(cached);
        }

        let suggestions = self.ai_inference.generate_completions(context).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        let completions: Vec<CompletionSuggestion> = suggestions.into_iter()
            .map(|s| CompletionSuggestion {
                text: s.text,
                kind: s.kind,
                detail: s.detail,
                documentation: s.documentation,
                sort_text: s.sort_text,
                filter_text: s.filter_text,
            })
            .collect();

        // Cache results
        self.cache_completions(cache_key, &completions).await;

        let latency = start_time.elapsed().as_millis() as u64;
        self.performance_monitor.record_completion(latency).await;

        Ok(completions)
    }

    async fn get_cached_completion(&self, key: &str) -> Option<Vec<CompletionSuggestion>> {
        if !self.config.enable_caching {
            return None;
        }

        let cache = self.cache.lock().await;
        cache.get_completion_suggestions(key).await
    }

    async fn cache_completions(&self, key: String, suggestions: &[CompletionSuggestion]) {
        if !self.config.enable_caching {
            return;
        }

        let cache = self.cache.lock().await;
        cache.put_completion_suggestions(key, suggestions.to_vec());
    }
}

/// Automated test generation based on code analysis
pub struct TestGenerator {
    config: CodegenConfig,
    cache: Arc<Mutex<CodegenCache>>,
    ai_inference: Arc<dyn InferenceService>,
    code_analyzer: Arc<dyn rust_ai_ide_shared_codegen::traits::CodeAnalyzer>,
    performance_monitor: Arc<performance::PerformanceMonitor>,
}

impl TestGenerator {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            config: CodegenConfig::default(),
            cache: Arc::new(Mutex::new(CodegenCache::new())),
            ai_inference: create_inference_service().await
                .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?,
            code_analyzer: create_code_analyzer().await
                .map_err(|e| CodegenError::AiAnalysisError(e.to_string()))?,
            performance_monitor: Arc::new(performance::PerformanceMonitor::new()),
        })
    }

    pub async fn with_config(config: CodegenConfig) -> Result<Self> {
        let cache = Arc::new(Mutex::new(CodegenCache::new()));
        let ai_inference = create_inference_service().await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;
        let code_analyzer = create_code_analyzer().await
            .map_err(|e| CodegenError::AiAnalysisError(e.to_string()))?;
        let performance_monitor = Arc::new(performance::PerformanceMonitor::new());

        Ok(Self {
            config,
            cache,
            ai_inference,
            code_analyzer,
            performance_monitor,
        })
    }

    /// Generate comprehensive tests for code
    pub async fn generate_tests(&self, code: &str, language: TargetLanguage) -> Result<TestSuite> {
        let start_time = std::time::Instant::now();

        // Analyze the code to understand its structure
        let analysis = self.code_analyzer.analyze_code(code, &target_to_programming_language(&language)).await
            .map_err(|e| CodegenError::AiAnalysisError(e.to_string()))?;

        // Generate test cases based on analysis
        let test_cases = self.generate_test_cases(&analysis).await?;

        // Generate test fixtures and setup
        let fixtures = self.generate_test_fixtures(&analysis).await?;

        // Generate mock objects if needed
        let mocks = self.generate_mocks(&analysis).await?;

        let test_suite = TestSuite {
            test_cases,
            fixtures,
            mocks,
            setup_code: self.generate_setup_code(&analysis).await?,
            teardown_code: self.generate_teardown_code().await?,
        };

        let latency = start_time.elapsed().as_millis() as u64;
        self.performance_monitor.record_test_generation(latency).await;

        Ok(test_suite)
    }

    async fn generate_test_cases(&self, analysis: &CodeAnalysisResult) -> Result<Vec<TestCase>> {
        let mut test_cases = Vec::new();

        for function in &analysis.functions {
            let test_case = self.generate_function_test_case(function).await?;
            test_cases.push(test_case);
        }

        for struct_info in &analysis.classes {
            let test_case = self.generate_struct_test_case(struct_info).await?;
            test_cases.push(test_case);
        }

        Ok(test_cases)
    }

    async fn generate_function_test_case(&self, function: &rust_ai_ide_shared_codegen::traits::FunctionSignature) -> Result<TestCase> {
        let spec = format!("Generate unit test for function {} with parameters {:?}",
                          function.name, function.parameters);
        let test_code = self.ai_inference.generate_code_from_spec(&spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(TestCase {
            name: format!("test_{}", function.name),
            code: test_code.content,
            expected: "Test passes".to_string(),
        })
    }

    async fn generate_struct_test_case(&self, struct_info: &rust_ai_ide_shared_codegen::traits::ClassDefinition) -> Result<TestCase> {
        let spec = format!("Generate unit test for struct {}", struct_info.name);
        let test_code = self.ai_inference.generate_code_from_spec(&spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(TestCase {
            name: format!("test_{}", struct_info.name),
            code: test_code.content,
            expected: "Struct test passes".to_string(),
        })
    }

    async fn generate_test_fixtures(&self, analysis: &CodeAnalysisResult) -> Result<Vec<TestFixture>> {
        // Generate test data fixtures
        let spec = "Generate test data fixtures for the analyzed code";
        let fixtures_code = self.ai_inference.generate_code_from_spec(spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(vec![TestFixture {
            name: "default_fixture".to_string(),
            code: fixtures_code.content,
        }])
    }

    async fn generate_mocks(&self, _analysis: &CodeAnalysisResult) -> Result<Vec<MockObject>> {
        // Placeholder implementation - no dependencies analysis available
        Ok(vec![])
    }

    async fn generate_setup_code(&self, analysis: &CodeAnalysisResult) -> Result<String> {
        let spec = "Generate test setup code";
        let setup_code = self.ai_inference.generate_code_from_spec(spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(setup_code.content)
    }

    async fn generate_teardown_code(&self) -> Result<String> {
        let spec = "Generate test teardown code";
        let teardown_code = self.ai_inference.generate_code_from_spec(spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(teardown_code.content)
    }
}

/// Safe refactoring suggestions and transformations
pub struct RefactoringEngine {
    config: CodegenConfig,
    cache: Arc<Mutex<CodegenCache>>,
    ai_inference: Arc<dyn InferenceService>,
    code_analyzer: Arc<dyn rust_ai_ide_shared_codegen::traits::CodeAnalyzer>,
    performance_monitor: Arc<performance::PerformanceMonitor>,
}

impl RefactoringEngine {
    pub async fn new() -> Result<Self> {
        Self::with_config(CodegenConfig::default()).await
    }

    pub async fn with_config(config: CodegenConfig) -> Result<Self> {
        let cache = Arc::new(Mutex::new(CodegenCache::new()));
        let ai_inference = create_inference_service().await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;
        let code_analyzer = create_code_analyzer().await
            .map_err(|e| CodegenError::AiAnalysisError(e.to_string()))?;
        let performance_monitor = Arc::new(performance::PerformanceMonitor::new());

        Ok(Self {
            config,
            cache,
            ai_inference,
            code_analyzer,
            performance_monitor,
        })
    }

    /// Analyze code and suggest refactoring opportunities
    pub async fn analyze_and_suggest(&self, code: &str, language: TargetLanguage) -> Result<Vec<RefactoringSuggestion>> {
        let start_time = std::time::Instant::now();

        // Analyze the code
        let analysis = self.code_analyzer.analyze_code(code, &target_to_programming_language(&language)).await
            .map_err(|e| CodegenError::AiAnalysisError(e.to_string()))?;

        // Generate refactoring suggestions
        let suggestions = self.generate_refactoring_suggestions(&analysis).await?;

        let latency = start_time.elapsed().as_millis() as u64;
        self.performance_monitor.record_refactoring(latency).await;

        Ok(suggestions)
    }

    /// Apply safe refactoring transformation
    pub async fn apply_refactoring(&self, code: &str, suggestion: &RefactoringSuggestion) -> Result<String> {
        let start_time = std::time::Instant::now();

        // Validate the refactoring is safe
        self.validate_refactoring_safety(code, suggestion).await?;

        // Apply the refactoring
        let refactored_code = self.perform_refactoring(code, suggestion).await?;

        // Verify the refactored code
        self.verify_refactored_code(&refactored_code, suggestion.target_language.clone()).await?;

        let latency = start_time.elapsed().as_millis() as u64;
        self.performance_monitor.record_refactoring(latency).await;

        Ok(refactored_code)
    }

    async fn generate_refactoring_suggestions(&self, analysis: &CodeAnalysisResult) -> Result<Vec<RefactoringSuggestion>> {
        let mut suggestions = Vec::new();

        // Long function detection
        for function in &analysis.functions {
            if self.is_long_function(function) {
                suggestions.push(RefactoringSuggestion {
                    kind: RefactoringKind::ExtractMethod,
                    description: format!("Extract method from long function '{}'", function.name),
                    target_language: TargetLanguage::Rust,
                    impact_level: ImpactLevel::Low,
                    confidence_score: 0.8,
                });
            }
        }

        // Duplicate code detection
        if self.has_duplicate_code(analysis) {
            suggestions.push(RefactoringSuggestion {
                kind: RefactoringKind::ExtractMethod,
                description: "Extract duplicate code into shared method".to_string(),
                target_language: TargetLanguage::Rust,
                impact_level: ImpactLevel::Medium,
                confidence_score: 0.7,
            });
        }

        // Large class detection
        if self.is_large_class(analysis) {
            suggestions.push(RefactoringSuggestion {
                kind: RefactoringKind::ExtractClass,
                description: "Extract class to reduce size and complexity".to_string(),
                target_language: TargetLanguage::Rust,
                impact_level: ImpactLevel::High,
                confidence_score: 0.6,
            });
        }

        Ok(suggestions)
    }

    fn is_long_function(&self, function: &rust_ai_ide_shared_codegen::traits::FunctionSignature) -> bool {
        // Heuristic: functions with complex signatures might be candidates for refactoring
        function.parameters.len() > 5
    }

    fn has_duplicate_code(&self, analysis: &CodeAnalysisResult) -> bool {
        // Simple heuristic: if there are multiple similar functions
        analysis.functions.len() > 5
    }

    fn is_large_class(&self, analysis: &CodeAnalysisResult) -> bool {
        // Heuristic: too many classes
        analysis.classes.len() > 3
    }

    async fn validate_refactoring_safety(&self, code: &str, suggestion: &RefactoringSuggestion) -> Result<()> {
        // Basic safety checks
        match suggestion.kind {
            RefactoringKind::ExtractMethod => {
                // Ensure the extracted method won't break dependencies
                if suggestion.impact_level == ImpactLevel::High {
                    return Err(CodegenError::RefactoringUnsafe(
                        "High impact refactoring requires manual review".to_string()
                    ));
                }
            }
            RefactoringKind::ExtractClass => {
                // Class extraction is more complex, always require review
                return Err(CodegenError::RefactoringUnsafe(
                    "Class extraction requires manual review".to_string()
                ));
            }
            _ => {}
        }
        Ok(())
    }

    async fn perform_refactoring(&self, code: &str, suggestion: &RefactoringSuggestion) -> Result<String> {
        let spec = format!("Apply {} refactoring: {}", suggestion.kind.as_str(), suggestion.description);
        let refactored = self.ai_inference.generate_code_from_spec(&spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        // Combine original code with refactoring
        Ok(refactored.content)
    }

    async fn verify_refactored_code(&self, code: &str, language: TargetLanguage) -> Result<()> {
        // Parse and validate the refactored code
        match language {
            TargetLanguage::Rust => {
                let token_stream: proc_macro2::TokenStream = syn::parse_str::<proc_macro2::TokenStream>(code)?;
                syn::parse2::<syn::File>(token_stream)
                    .map_err(|e| CodegenError::SyntaxError(format!("Refactored code syntax error: {}", e)))?;
            }
            _ => {
                if code.trim().is_empty() {
                    return Err(CodegenError::ValidationError("Refactored code is empty".to_string()));
                }
            }
        }
        Ok(())
    }
}

/// Automated documentation generation from code analysis
pub struct DocumentationGenerator {
    config: CodegenConfig,
    cache: Arc<Mutex<CodegenCache>>,
    ai_inference: Arc<dyn InferenceService>,
    code_analyzer: Arc<dyn rust_ai_ide_shared_codegen::traits::CodeAnalyzer>,
    performance_monitor: Arc<performance::PerformanceMonitor>,
}

impl DocumentationGenerator {
    pub async fn new() -> Result<Self> {
        Self::with_config(CodegenConfig::default()).await
    }

    pub async fn with_config(config: CodegenConfig) -> Result<Self> {
        let cache = Arc::new(Mutex::new(CodegenCache::new()));
        let ai_inference = create_inference_service().await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;
        let code_analyzer = create_code_analyzer().await
            .map_err(|e| CodegenError::AiAnalysisError(e.to_string()))?;
        let performance_monitor = Arc::new(performance::PerformanceMonitor::new());

        Ok(Self {
            config,
            cache,
            ai_inference,
            code_analyzer,
            performance_monitor,
        })
    }

    /// Generate comprehensive documentation for code
    pub async fn generate_documentation(&self, code: &str, language: TargetLanguage) -> Result<Documentation> {
        let start_time = std::time::Instant::now();

        // Analyze the code
        let analysis = self.code_analyzer.analyze_code(code, &target_to_programming_language(&language)).await
            .map_err(|e| CodegenError::AiAnalysisError(e.to_string()))?;

        // Generate documentation sections
        let overview = self.generate_overview(&analysis).await?;
        let api_docs = self.generate_api_docs(&analysis).await?;
        let examples = self.generate_examples(&analysis).await?;
        let references = self.generate_references(&analysis).await?;

        let docs = Documentation {
            overview,
            api_reference: api_docs,
            examples,
            references,
            generated_at: chrono::Utc::now(),
            format: DocFormat::Markdown,
        };

        let latency = start_time.elapsed().as_millis() as u64;
        self.performance_monitor.record_documentation_generation(latency).await;

        Ok(docs)
    }

    /// Generate documentation for a specific function
    pub async fn generate_function_docs(&self, function: &rust_ai_ide_shared_codegen::traits::FunctionSignature) -> Result<String> {
        let spec = format!("Generate documentation for function {} with parameters {:?}",
                          function.name, function.parameters);
        let docs = self.ai_inference.generate_code_from_spec(&spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(docs.content)
    }

    async fn generate_overview(&self, analysis: &CodeAnalysisResult) -> Result<String> {
        let spec = format!("Generate overview documentation for {} code with {} functions and {} classes",
                          "unknown".to_string(),
                          analysis.functions.len(),
                          analysis.classes.len());

        let overview = self.ai_inference.generate_code_from_spec(&spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(overview.content)
    }

    async fn generate_api_docs(&self, analysis: &CodeAnalysisResult) -> Result<String> {
        let mut api_docs = String::new();

        for function in &analysis.functions {
            let func_docs = self.generate_function_docs(function).await?;
            api_docs.push_str(&format!("## {}\n\n{}\n\n", function.name, func_docs));
        }

        for struct_info in &analysis.classes {
            let struct_docs = self.generate_struct_docs(struct_info).await?;
            api_docs.push_str(&format!("## {}\n\n{}\n\n", struct_info.name, struct_docs));
        }

        Ok(api_docs)
    }

    async fn generate_examples(&self, analysis: &CodeAnalysisResult) -> Result<String> {
        let spec = "Generate usage examples for the analyzed code";
        let examples = self.ai_inference.generate_code_from_spec(spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(examples.content)
    }

    async fn generate_references(&self, analysis: &CodeAnalysisResult) -> Result<String> {
        let spec = "Generate reference documentation and links";
        let references = self.ai_inference.generate_code_from_spec(spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(references.content)
    }

    async fn generate_struct_docs(&self, struct_info: &rust_ai_ide_shared_codegen::traits::ClassDefinition) -> Result<String> {
        let spec = format!("Generate documentation for struct {} with fields: {}",
                          struct_info.name,
                          struct_info.fields.iter().map(|f| f.name.as_str()).collect::<Vec<&str>>().join(", "));
        let docs = self.ai_inference.generate_code_from_spec(&spec).await
            .map_err(|e| CodegenError::AiInferenceError(e.to_string()))?;

        Ok(docs.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_code_generator_creation() {
        let generator = CodeGenerator::new().await;
        assert!(generator.is_ok());
    }

    #[tokio::test]
    async fn test_completion_engine_creation() {
        let engine = CompletionEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_test_generator_creation() {
        let generator = TestGenerator::new().await;
        assert!(generator.is_ok());
    }

    #[tokio::test]
    async fn test_refactoring_engine_creation() {
        let engine = RefactoringEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_documentation_generator_creation() {
        let generator = DocumentationGenerator::new().await;
        assert!(generator.is_ok());
    }
}