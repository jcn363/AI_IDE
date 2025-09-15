// Diagnostic log for syn version compatibility
#[cfg(test)]
mod diagnostics {
    #[test]
    fn check_syn_version() {
        println!("Syn version: {}", env!("CARGO_PKG_VERSION_syn"));
        println!("Syn crate features: {:?}", std::env::var("CARGO_FEATURES").unwrap_or_default());
    }
}
//! AI-powered code refactoring for Rust AI IDE
//!
//! This crate provides intelligent code refactoring capabilities using machine learning
//! and static analysis to improve code quality and maintainability.

// Module declarations
pub mod analysis;
pub mod ast_utils;
pub mod async_operations;
pub mod audit;
pub mod batch;
pub mod batch_operations;
pub mod class_struct_operations;
pub mod code_organization;
pub mod confidence;
pub mod core_traits;
pub mod delegation_operations;
pub mod enhanced_backup;
pub mod engine;
pub mod file_operations;
pub mod function_method_operations;
pub mod logging;
pub mod operation_factory;
pub mod operations;
pub mod pattern_recognition;
pub mod progress;
pub mod rename_operations;
pub mod safety;
pub mod service;
pub mod signature_operations;
pub mod suggestions;
pub mod test_generation;
pub mod types;
pub mod utils;
pub mod variable_operations;

// Re-exports for external use
pub use audit::{AuditLogger, ErrorHandler, AuditReport, ContextualError, RefactoringError, error_utils};
pub use confidence::ConfidenceScorer;
// Re-export core traits and types
pub use core_traits::RefactoringOperation;
pub use enhanced_backup::{EnhancedBackupManager, BackupMetadata, BackupStats};
pub use engine::{RefactoringEngine, ExecutionContext, RefactoringExecutionResult, EngineMetrics};
pub use logging::{RefactoringLogger, SessionStatus, SessionType};
pub use operations::*;
pub use progress::ProgressTracker;
pub use safety::{SafetyAnalyzer, SafetyAnalysisResult, RefactoringRisk};
// Core AI Refactoring Service - Main Integration Point
pub use service::RefactoringService;
pub use suggestions::{SuggestionEngine, SuggestionEngineImpl};
pub use types::*;
