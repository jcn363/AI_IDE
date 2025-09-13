//! Common traits for test generation and code generation interfaces

use std::collections::HashMap;

use async_trait::async_trait;
use rust_ai_ide_common::types::*;

/// Unified trait for test generation across different modules
#[async_trait]
pub trait TestGenerator {
    /// Generate basic unit tests for given code
    async fn generate_unit_tests(
        &self,
        code: &str,
        context: &TestGenerationContext,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate integration tests for component interactions
    async fn generate_integration_tests(
        &self,
        context: &TestGenerationContext,
        dependencies: Vec<String>,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate property-based tests
    async fn generate_property_tests(
        &self,
        code: &str,
        language: &ProgrammingLanguage,
        properties: Vec<String>,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate tests for refactoring operations
    async fn generate_refactoring_tests(
        &self,
        refactoring_type: &RefactoringType,
        context: &RefactoringContext,
        result: &RefactoringResult,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate comprehensive test suite for given code
    async fn generate_test_suite(
        &self,
        code: &str,
        context: &TestGenerationContext,
    ) -> Result<GeneratedTests, Box<dyn std::error::Error + Send + Sync>>;

    /// Estimate test coverage for generated tests
    async fn estimate_coverage(
        &self,
        tests: &[GeneratedTest],
        language: &ProgrammingLanguage,
    ) -> Result<Vec<TestCoverage>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate tests for rename refactoring
    async fn generate_rename_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
        language: &ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate tests for extract function refactoring
    async fn generate_extract_function_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
        language: &ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate tests for extract variable refactoring
    async fn generate_extract_variable_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
        language: &ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate tests for extract interface refactoring
    async fn generate_extract_interface_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
        language: &ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate tests for async conversion refactoring
    async fn generate_async_conversion_tests(
        &self,
        context: &RefactoringContext,
        result: &RefactoringResult,
        language: &ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Generate generic tests for any refactoring type
    async fn generate_generic_tests(
        &self,
        refactoring_type: &RefactoringType,
        context: &RefactoringContext,
        result: &RefactoringResult,
        language: &ProgrammingLanguage,
        framework: &str,
    ) -> Result<Vec<GeneratedTest>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get supported programming languages
    fn supported_languages(&self) -> Vec<ProgrammingLanguage>;

    /// Get available test frameworks for a language
    fn get_test_frameworks(&self, language: &ProgrammingLanguage) -> Vec<String>;

    /// Validate generated tests for correctness
    fn validate_tests(&self, tests: &[GeneratedTest]) -> Vec<ValidationError>;
}

/// Language detection and framework identification service
#[async_trait]
pub trait LanguageDetector {
    /// Detect programming language from file path
    fn detect_language(&self, file_path: &str) -> (ProgrammingLanguage, String);

    /// Detect programming language from code content
    fn detect_language_from_content(&self, content: &str) -> Option<ProgrammingLanguage>;

    /// Get available test frameworks for a language
    fn get_test_frameworks(&self, language: &ProgrammingLanguage) -> Vec<String>;

    /// Get file extensions for a language
    fn get_file_extensions(&self, language: &ProgrammingLanguage) -> Vec<String>;

    /// Get preferred test framework for a language
    fn get_preferred_framework(&self, language: &ProgrammingLanguage) -> String;
}

/// Code analysis and transformation interface
#[async_trait]
pub trait CodeAnalyzer {
    /// Analyze code and extract relevant information for testing
    async fn analyze_code(
        &self,
        code: &str,
        language: &ProgrammingLanguage,
    ) -> Result<CodeAnalysisResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Extract function and method signatures
    fn extract_signatures(&self, code: &str, language: &ProgrammingLanguage) -> Vec<FunctionSignature>;

    /// Find potential test cases in code
    fn identify_test_targets(&self, code: &str, language: &ProgrammingLanguage) -> Vec<TestTarget>;

    /// Analyze dependencies and imports
    fn analyze_dependencies(&self, code: &str, language: &ProgrammingLanguage) -> Vec<String>;
}

/// Configuration management for test generation
#[async_trait]
pub trait ConfigurationProvider {
    /// Load test generation configuration
    async fn load_config(&self) -> Result<TestGenerationConfig, Box<dyn std::error::Error + Send + Sync>>;

    /// Save test generation configuration
    async fn save_config(&self, config: &TestGenerationConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get language-specific configuration
    fn get_language_config(&self, language: &ProgrammingLanguage) -> Option<LanguageConfig>;

    /// Update language-specific configuration
    fn update_language_config(&self, language: &ProgrammingLanguage, config: LanguageConfig);
}

/// Error types for test generation validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Test name conflicts with existing tests
    NameConflict(String),
    /// Missing required dependencies
    MissingDependencies(Vec<String>),
    /// Invalid test structure or syntax
    InvalidStructure(String),
    /// Unsupported language feature used
    UnsupportedFeature(String),
    /// Test would not actually test anything meaningful
    TestIneffective(String),
}

/// Result of code analysis
#[derive(Debug, Clone)]
pub struct CodeAnalysisResult {
    /// Extracted function signatures
    pub functions:    Vec<FunctionSignature>,
    /// Identified classes/structs
    pub classes:      Vec<ClassDefinition>,
    /// Identified imports/dependencies
    pub imports:      Vec<String>,
    /// Potential test targets
    pub test_targets: Vec<TestTarget>,
    /// Language-specific analysis data
    pub metadata:     HashMap<String, String>,
}

/// Function/method signature information
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Function name
    pub name:        String,
    /// Parameters with types
    pub parameters:  Vec<Parameter>,
    /// Return type (if any)
    pub return_type: Option<String>,
    /// Access modifier (public, private, etc.)
    pub visibility:  Option<String>,
    /// Whether the function is async
    pub is_async:    bool,
    /// Line where function starts
    pub line_start:  usize,
    /// Line where function ends
    pub line_end:    usize,
}

/// Class/struct definition information
#[derive(Debug, Clone)]
pub struct ClassDefinition {
    /// Class/struct name
    pub name:       String,
    /// Fields/properties
    pub fields:     Vec<Field>,
    /// Methods
    pub methods:    Vec<FunctionSignature>,
    /// Inheritance information
    pub inherits:   Vec<String>,
    /// Line where class starts
    pub line_start: usize,
    /// Line where class ends
    pub line_end:   usize,
}

/// Parameter information
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name:        String,
    /// Parameter type
    pub param_type:  String,
    /// Whether parameter has a default value
    pub has_default: bool,
}

/// Field/property information
#[derive(Debug, Clone)]
pub struct Field {
    /// Field name
    pub name:       String,
    /// Field type
    pub field_type: String,
    /// Access modifier
    pub visibility: Option<String>,
    /// Whether field is mutable
    pub is_mutable: bool,
}

/// Potential test target identification
#[derive(Debug, Clone)]
pub struct TestTarget {
    /// Type of test target (function, class, module, etc.)
    pub target_type: TestTargetType,
    /// Name of the target
    pub name:        String,
    /// Line where target starts
    pub line_start:  usize,
    /// Line where target ends
    pub line_end:    usize,
    /// Suggested test coverage priority (0.0-1.0)
    pub priority:    f32,
}

/// Types of test targets
#[derive(Debug, Clone, PartialEq)]
pub enum TestTargetType {
    /// Individual function or method
    Function,
    /// Class or struct
    Class,
    /// Module or file
    Module,
    /// Public API method
    PublicAPI,
    /// Error handling code
    ErrorPath,
}
