//! Test generation configuration and related types
//!
//! This module contains configuration structs and enums for test generation.

use std::collections::HashMap;

/// Programming language detection and configuration
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ProgrammingLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Java,
    CSharp,
    Go,
    Cpp,
    C,
    Unknown,
}

/// Test type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum TestType {
    Unit,
    Integration,
    Property,
    Benchmark,
    Fuzz,
    E2e,
}

/// Refactoring type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum RefactoringType {
    Rename,
    ExtractFunction,
    ExtractVariable,
    ExtractInterface,
    ConvertToAsync,
    Move,
    Inline,
    ChangeSignature,
    ReplaceWithMethodCall,
    Other(String),
}

/// Coverage granularity types
#[derive(Debug, Clone)]
pub enum CoverageType {
    Function,
    Line,
    Branch,
    Edge,
    Statement,
}

/// Language-specific configuration
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub naming_conventions: Vec<String>,
    pub test_patterns: Vec<String>,
    pub assertion_styles: Vec<String>,
    pub mock_frameworks: Vec<String>,
}

/// Test generation configuration
#[derive(Debug, Clone)]
pub struct TestGenerationConfig {
    pub include_edge_cases: bool,
    pub generate_integration_tests: bool,
    pub max_tests_per_generation: usize,
    pub target_coverage_percentage: f32,
    pub language_specific: HashMap<ProgrammingLanguage, LanguageConfig>,
    pub timeout_seconds: u32,
}

impl Default for TestGenerationConfig {
    fn default() -> Self {
        Self {
            include_edge_cases: true,
            generate_integration_tests: false,
            max_tests_per_generation: 10,
            target_coverage_percentage: 80.0,
            language_specific: HashMap::new(),
            timeout_seconds: 30,
        }
    }
}

/// Test generation context for codegen compatibility
#[derive(Debug, Clone)]
pub struct TestGenerationContext {
    pub file_path: String,
    pub is_performance_critical: bool,
    pub required_coverage: Option<f32>,
    pub target_languages: Vec<ProgrammingLanguage>,
}

// Compatibility aliases for codegen crate
pub use ProgrammingLanguage::*;
pub use TestType::*;

/// Test framework information is defined in language_detector.rs
pub use super::language_detector::TestFrameworkInfo;
