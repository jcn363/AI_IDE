//! # Unified Test Generation Utilities
//!
//! Language-aware test generation system supporting multiple programming languages
//! and refactoring scenarios with consolidated LanguageTestGenerator trait.

// Main modules
#[path = "language_detector.rs"]
pub mod language_detector;
#[path = "language_generator.rs"]
pub mod language_generator;
#[path = "test_config.rs"]
pub mod test_config;
#[path = "unified_generator.rs"]
pub mod unified_generator;
#[path = "utils.rs"]
pub mod utils;

// Re-exports for backward compatibility and consolidation
pub use language_detector::*;
pub use language_generator::*;
// Additional exports for unified harness integration
pub use test_config::TestGenerationContext;
pub use test_config::*;
pub use unified_generator::{RefactoringContext, RefactoringResult, *};
pub use utils::*;

// Legacy compatibility aliases for codegen crate - removed unused glob imports
