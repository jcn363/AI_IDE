//! # Unified Test Generation Utilities
//!
//! Language-aware test generation system supporting multiple programming languages
//! and refactoring scenarios with consolidated LanguageTestGenerator trait.

// Main modules
#[path = "test_config.rs"]
pub mod test_config;
#[path = "language_detector.rs"]
pub mod language_detector;
#[path = "unified_generator.rs"]
pub mod unified_generator;
#[path = "utils.rs"]
pub mod utils;
#[path = "language_generator.rs"]
pub mod language_generator;

// Re-exports for backward compatibility and consolidation
pub use test_config::*;
pub use language_detector::*;
pub use unified_generator::*;
pub use utils::*;
pub use language_generator::*;

// Additional exports for unified harness integration
pub use unified_generator::{RefactoringContext, RefactoringResult};
pub use test_config::TestGenerationContext;

// Legacy compatibility aliases for codegen crate - removed unused glob imports