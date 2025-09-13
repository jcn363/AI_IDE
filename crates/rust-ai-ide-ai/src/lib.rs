//! # Rust AI IDE Core AI Module
//!
//! This crate serves as the central hub for all AI-powered functionality in the Rust AI IDE.
//! It provides a unified interface to various AI capabilities including code analysis,
//! refactoring, architectural guidance, error resolution, and intelligent code generation.
//!
//! ## Architecture Overview
//!
//! The AI system is organized into multiple specialized crates for modularity and performance:
//!
//! - **rust-ai-ide-ai** (this crate): Main coordination and public API
//! - **rust-ai-ide-ai-analysis**: Code analysis and understanding
//! - **rust-ai-ide-ai-codegen**: Code generation and completion
//! - **rust-ai-ide-ai-inference**: Model inference and execution
//! - **rust-ai-ide-ai-learning**: Model training and adaptation
//! - **rust-ai-ide-ai-refactoring**: Intelligent code refactoring
//!
//! ## Key Features
//!
//! ### Code Intelligence
//! - **Advanced Error Analysis**: Deep semantic error detection and resolution
//! - **Code Review**: Automated code quality assessment and suggestions
//! - **Architectural Guidance**: Design pattern recognition and architectural recommendations
//!
//! ### Development Assistance
//! - **Intelligent Refactoring**: Context-aware code transformations
//! - **Specification Generation**: Automated documentation and spec creation
//! - **Rate Limiting**: Prevents AI service overload and ensures fair usage
//!
//! ## Usage Examples
//!
//! ### Basic Code Analysis
//! ```rust,ignore
//! use rust_ai_ide_ai::analysis::{CodeAnalysisRequest, analyze_code};
//!
//! let request = CodeAnalysisRequest {
//!     code: "fn main() { println!(\"Hello\"); }".to_string(),
//!     language: "rust".to_string(),
//!     analysis_type: AnalysisType::Semantic,
//! };
//!
//! let result = analyze_code(request).await?;
//! println!("Analysis: {:?}", result);
//! ```
//!
//! ### Intelligent Refactoring
//! ```rust,ignore
//! use rust_ai_ide_ai::refactoring::{RefactoringRequest, refactor_code};
//!
//! let request = RefactoringRequest {
//!     code: source_code,
//!     refactoring_type: RefactoringType::ExtractFunction,
//!     target_range: Some(range),
//! };
//!
//! let refactored = refactor_code(request).await?;
//! ```
//!
//! ### Architectural Analysis
//! ```rust,ignore
//! use rust_ai_ide_ai::architectural_advisor::{ArchitecturalAnalysis, analyze_architecture};
//!
//! let analysis = ArchitecturalAnalysis {
//!     codebase_path: "/project/src".into(),
//!     analysis_scope: Scope::Full,
//! };
//!
//! let recommendations = analyze_architecture(analysis).await?;
//! for rec in recommendations {
//!     println!("Architectural suggestion: {}", rec.description);
//! }
//! ```
//!
//! ## AI Model Integration
//!
//! Following AGENTS.md guidelines, all AI model interactions are mediated through the LSP service:
//!
//! - Models are loaded/unloaded through the LSP service exclusively
//! - Direct model access is forbidden for security and resource management
//! - Hyperparameter tuning is restricted to approved pipelines
//! - All processing happens locally (no cloud dependencies)
//!
//! ## Security and Performance
//!
//! - **Rate Limiting**: Prevents resource exhaustion through configurable limits
//! - **Input Validation**: All inputs validated before AI processing
//! - **Memory Management**: Efficient handling of large codebases
//! - **Audit Logging**: Sensitive AI operations are logged for compliance
//!
//! ## Error Handling
//!
//! The AI system uses structured error handling with specific error types:
//!
//! - `AIError`: General AI processing errors
//! - `InferenceError`: Model inference failures
//! - `AnalysisError`: Code analysis failures
//! - `RefactoringError`: Refactoring operation failures
//!
//! ## Configuration
//!
//! AI behavior is controlled through configuration structures:
//!
//! ```rust,ignore
//! use rust_ai_ide_ai_inference::AIAnalysisConfig;
//!
//! let config = AIAnalysisConfig {
//!     max_tokens: 4096,
//!     temperature: 0.7,
//!     model_preference: ModelType::CodeLlama,
//!     enable_caching: true,
//!     timeout_seconds: 30,
//! };
//! ```
//!
//! ## Thread Safety
//!
//! All AI operations are designed to be thread-safe:
//!
//! - Services use `Arc<Mutex<T>>` for shared state
//! - Concurrent requests are handled safely
//! - Resource pooling prevents conflicts
//! - Async operations use Tokio runtime
//!
//! ## Performance Considerations
//!
//! - **Caching**: Intelligent caching of analysis results
//! - **Batching**: Request batching for efficiency
//! - **Streaming**: Incremental results for large operations
//! - **Resource Limits**: Configurable limits prevent runaway resource usage

// Core modules
pub mod advanced_error_analysis;
pub mod architectural_advisor;
pub mod code_review;
pub mod error_resolution;
pub mod rate_limiter;

// Re-export refactoring functionality from separate crate
pub mod refactoring {
    pub use rust_ai_ide_ai_refactoring::*;
}
pub mod spec_generation;

// Re-export analysis functionality from separate crate
pub mod analysis {
    pub use rust_ai_ide_ai_analysis::*;
}

// The architectural_advisor module is defined in this crate
pub use architectural_advisor::*;

// Re-export inference modules from separate crate
pub mod inference {
    pub use rust_ai_ide_ai_inference::inference::*;
    pub use rust_ai_ide_ai_inference::loaders;
    pub use rust_ai_ide_ai_inference::model_loader;
}

// Re-export learning functionality from separate crate
pub mod learning {
    pub use rust_ai_ide_ai_learning::*;
}
pub mod model_loader {
    pub use rust_ai_ide_ai_inference::loaders;
    pub use rust_ai_ide_ai_inference::model_loader::*;
}

// Explicit import of all types items to prevent namespace conflicts with glob
pub use architectural_advisor::types::{
    AntiPattern,
    ArchitecturalContext,
    ArchitecturalDocument,
    ArchitecturalGuidance,
    ArchitecturalOverview as AdvisorArchitecturalOverview, // avoid naming conflict
    ArchitecturalOverview,
    ArchitecturalRecommendation,
    ArchitecturalSuggestion,
    CohesionAnalysis,
    ComplexityAssessment,
    ComponentDocument,
    CouplingAnalysis,
    DecisionAnalysis,
    DecisionOption,
    DecisionRecommendation,
    DecisionRecord,
    DecisionStatus,
    DeploymentDocument,
    DeploymentEnvironment,
    DeploymentProcedures,
    DeploymentRequirements,
    DetectedPattern,
    InterfaceDocument,
    InterfaceMethod,
    MethodParameter,
    PatternAnalysis,
    PatternDocument,
    ProjectType,
    QualityAttribute,
    QualityAttributesDocument,
    QualityMetric,
    QualityMetrics,
    QualityScenario,
    RiskAssessment,
};

// Re-export inference types from the inference crate
pub use rust_ai_ide_ai_inference::{
    AIAnalysisConfig, AIProvider, AIService, AnalysisIssue, CodeAnalysisResult, ModelDevice,
    ModelHandle, ModelInfo, ModelLoadConfig, ModelSize, Quantization,
};

// Re-export code generation functionality from the codegen crate
pub mod code_generation {
    pub use rust_ai_ide_ai_codegen::*;
}
