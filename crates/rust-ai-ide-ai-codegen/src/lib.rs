//! # Rust AI IDE AI Code Generation
//!
//! An AI-powered multi-language code generation system with template engines
//! for the Rust AI IDE.
//!
//! ## Features
//!
//! - **Multi-language Code Generation**: Generate code for Rust, Python, TypeScript, and more
//! - **Template-based Generation**: Uses Handlebars templates for flexible code generation
//! - **Context-aware**: Generates code based on project structure and patterns
//! - **Quality Assurance**: Built-in validation and quality checking
//! - **Architectural Generation**: Support for MVC, API, database, and configuration code
//!
//! ## Architecture
//!
//! This crate provides:
//! - `CodeGenerationService`: Main service for code generation orchestration
//! - `CodeGenerator` traits: Individual generators for different code types
//! - Template system: Reusable templates for common patterns
//! - Quality validation: Ensures generated code meets standards
//!
//! ## Usage
//!
//! ```ignore
//! use rust_ai_ide_ai_codegen::*;
//!
//! let service = CodeGenerationService::new();
//! let context = rust_ai_ide_shared_codegen::generator::CodeGenerationContext::builder()
//!     .language(rust_ai_ide_shared_codegen::generator::TargetLanguage::Rust)
//!     .target_scope(rust_ai_ide_shared_codegen::generator::GenerationScope::Function)
//!     .build();
//! let code = service.generate_code(context).await?;
//! ```

// Re-export shared types for convenience
pub use rust_ai_ide_shared_codegen::generator::{
    CodeGenerationContext, CodeGenerationContextBuilder, CodeGenerationError, CodePattern,
    CodingStandards, Dependency, GenerationQuality, GenerationScope, PerformanceReqs,
    ProjectContext, QualityIssue, QualityRequirements, TargetLanguage, UserPreferences,
};

// Core submodules
pub mod architectural;
pub mod code_generation;
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
pub use language_specific::LanguageSpecificGenerator;
pub use quality_assurance::CodeQualityValidator;
pub use test_generation::TestGenerator;
