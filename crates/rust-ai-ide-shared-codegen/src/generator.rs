//! Code generation interfaces and unified context
//! This module provides shared abstractions for AI-powered code generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unified context for code generation across all ai-codegen modules
#[derive(Debug, Clone)]
pub struct CodeGenerationContext {
    pub project_context: ProjectContext,
    pub language: TargetLanguage,
    pub target_scope: GenerationScope,
    pub quality_requirements: QualityRequirements,
    pub user_preferences: UserPreferences,
}

/// Builder pattern for CodeGenerationContext
#[derive(Debug, Clone, Default)]
pub struct CodeGenerationContextBuilder {
    project_context: Option<ProjectContext>,
    language: Option<TargetLanguage>,
    target_scope: Option<GenerationScope>,
    quality_requirements: Option<QualityRequirements>,
    user_preferences: Option<UserPreferences>,
}

impl CodeGenerationContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn project_context(mut self, context: ProjectContext) -> Self {
        self.project_context = Some(context);
        self
    }

    pub fn language(mut self, language: TargetLanguage) -> Self {
        self.language = Some(language);
        self
    }

    pub fn target_scope(mut self, scope: GenerationScope) -> Self {
        self.target_scope = Some(scope);
        self
    }

    pub fn quality_requirements(mut self, reqs: QualityRequirements) -> Self {
        self.quality_requirements = Some(reqs);
        self
    }

    pub fn user_preferences(mut self, prefs: UserPreferences) -> Self {
        self.user_preferences = Some(prefs);
        self
    }

    pub fn build(self) -> CodeGenerationContext {
        CodeGenerationContext {
            project_context: self.project_context.unwrap_or_default(),
            language: self.language.unwrap_or(TargetLanguage::Rust),
            target_scope: self.target_scope.unwrap_or(GenerationScope::Function),
            quality_requirements: self.quality_requirements.unwrap_or_default(),
            user_preferences: self.user_preferences.unwrap_or_default(),
        }
    }
}

impl CodeGenerationContext {
    /// Create a new context using the builder pattern
    pub fn builder() -> CodeGenerationContextBuilder {
        CodeGenerationContextBuilder::new()
    }

    /// Create a default context for Rust function generation
    pub fn default_rust_function() -> Self {
        Self::builder()
            .language(TargetLanguage::Rust)
            .target_scope(GenerationScope::Function)
            .build()
    }

    /// Create a context for demo scenarios
    pub fn demo_context(language: TargetLanguage, scope: GenerationScope) -> Self {
        Self::builder()
            .project_context(ProjectContext::with_default_structure())
            .language(language)
            .target_scope(scope)
            .build()
    }

    /// Create a context for testing scenarios
    pub fn test_context(language: TargetLanguage) -> Self {
        Self::builder()
            .project_context(ProjectContext::minimal())
            .language(language)
            .target_scope(GenerationScope::Function)
            .user_preferences(UserPreferences::minimal())
            .build()
    }
}

/// Project context information
#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    pub project_structure: HashMap<String, String>,
    pub dependencies: Vec<Dependency>,
    pub existing_patterns: Vec<CodePattern>,
    pub coding_standards: CodingStandards,
}

impl ProjectContext {
    /// Create a minimal project context for testing
    pub fn minimal() -> Self {
        Self {
            project_structure: HashMap::new(),
            dependencies: vec![],
            existing_patterns: vec![],
            coding_standards: CodingStandards::default(),
        }
    }

    /// Create project context with default structure for demos
    pub fn with_default_structure() -> Self {
        Self {
            project_structure: Self::default_project_structure(),
            dependencies: vec![],
            existing_patterns: vec![],
            coding_standards: CodingStandards::default(),
        }
    }

    /// Create project context for specific language
    pub fn for_language(language: TargetLanguage) -> Self {
        let mut structure = HashMap::new();
        match language {
            TargetLanguage::Rust => {
                structure.insert("lib.rs".to_string(), "mod main;".to_string());
                structure.insert("Cargo.toml".to_string(), "[package]".to_string());
            }
            TargetLanguage::Python => {
                structure.insert(
                    "__init__.py".to_string(),
                    "# Package initialization".to_string(),
                );
                structure.insert("requirements.txt".to_string(), "# Dependencies".to_string());
            }
            _ => {
                // Default empty structure
            }
        }

        Self {
            project_structure: structure,
            ..Default::default()
        }
    }

    fn default_project_structure() -> HashMap<String, String> {
        let mut structure = HashMap::new();
        structure.insert("src/main.rs".to_string(), "fn main() {}".to_string());
        structure.insert("Cargo.toml".to_string(), "[package]".to_string());
        structure
    }
}

/// Target language for code generation
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TargetLanguage {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
    Cpp,
    CSharp,
    SQL,
    HTML,
    CSS,
    Shell,
    Other(String),
}

impl Default for TargetLanguage {
    fn default() -> Self {
        TargetLanguage::Rust
    }
}

/// Generation scope - what type of code to generate
#[derive(Debug, Clone, Default)]
pub enum GenerationScope {
    /// Generate a single function
    #[default]
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

/// Quality requirements for generated code
#[derive(Debug, Clone)]
pub struct QualityRequirements {
    pub min_complexity_score: f32,
    pub require_documentation: bool,
    pub require_error_handling: bool,
    pub require_tests: bool,
    pub code_style: String,
    pub performance_requirements: Option<PerformanceReqs>,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_complexity_score: 0.7,
            require_documentation: true,
            require_error_handling: true,
            require_tests: false,
            code_style: "standard".to_string(),
            performance_requirements: None,
        }
    }
}

impl QualityRequirements {
    /// Quality requirements for production code
    pub fn production() -> Self {
        Self {
            min_complexity_score: 0.85,
            require_documentation: true,
            require_error_handling: true,
            require_tests: true,
            code_style: "strict".to_string(),
            performance_requirements: Some(PerformanceReqs::default()),
        }
    }

    /// Relaxed requirements for demo/prototyping
    pub fn demo() -> Self {
        Self {
            min_complexity_score: 0.5,
            require_documentation: false,
            require_error_handling: false,
            require_tests: false,
            code_style: "relaxed".to_string(),
            performance_requirements: None,
        }
    }
}

/// User preferences for code generation
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub naming_convention: String,
    pub max_line_length: usize,
    pub indentation: String,
    pub comment_style: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            naming_convention: "snake_case".to_string(),
            max_line_length: 80,
            indentation: "    ".to_string(),
            comment_style: "//".to_string(),
        }
    }
}

impl UserPreferences {
    /// Minimal preferences for testing
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Preferences for Python development
    pub fn python() -> Self {
        Self {
            naming_convention: "snake_case".to_string(),
            max_line_length: 88,
            indentation: "    ".to_string(),
            comment_style: "#".to_string(),
        }
    }

    /// Preferences for TypeScript development
    pub fn typescript() -> Self {
        Self {
            naming_convention: "camelCase".to_string(),
            max_line_length: 100,
            indentation: "  ".to_string(),
            comment_style: "//".to_string(),
        }
    }
}

/// Generated code quality assessment
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationQuality {
    pub readability_score: f32,
    pub maintainability_score: f32,
    pub performance_score: f32,
    pub security_score: f32,
    pub compliance_score: f32,
    pub overall_score: f32,
    pub issues: Vec<QualityIssue>,
}

impl GenerationQuality {
    /// Calculate overall score from individual metrics
    pub fn calculated_overall_score(&self) -> f32 {
        let weights = [
            (self.readability_score, 0.25),
            (self.maintainability_score, 0.25),
            (self.performance_score, 0.2),
            (self.security_score, 0.15),
            (self.compliance_score, 0.15),
        ];

        weights.iter().map(|(score, weight)| score * weight).sum()
    }

    /// Create a sample quality assessment for demos
    pub fn sample_success() -> Self {
        Self {
            readability_score: 0.85,
            maintainability_score: 0.82,
            performance_score: 0.78,
            security_score: 0.92,
            compliance_score: 0.88,
            overall_score: 0.85,
            issues: vec![],
        }
    }

    /// Normalize a score between 0 and 1
    pub fn normalize_score(score: f32) -> f32 {
        score.clamp(0.0, 1.0)
    }

    /// Check if quality meets minimum requirements
    pub fn meets_requirements(&self, requirements: &QualityRequirements) -> bool {
        self.overall_score >= requirements.min_complexity_score
    }
}

/// Quality issues found during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub category: String,
    pub severity: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Supporting types for context structures

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodePattern {
    pub pattern_type: String,
    pub example: String,
    pub usage_context: String,
}

#[derive(Debug, Clone, Default)]
pub struct CodingStandards {
    pub formatting_rules: Vec<String>,
    pub naming_rules: Vec<String>,
    pub documentation_rules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceReqs {
    pub max_execution_time: std::time::Duration,
    pub memory_limit: u64,
    pub thread_safety: bool,
}

impl Default for PerformanceReqs {
    fn default() -> Self {
        Self {
            max_execution_time: std::time::Duration::from_millis(100),
            memory_limit: 1024 * 1024 * 1024, // 1GB
            thread_safety: true,
        }
    }
}

/// Error types for code generation
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
