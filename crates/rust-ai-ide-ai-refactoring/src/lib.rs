//! AI-powered refactoring module
//!
//! This module provides comprehensive refactoring capabilities
//! for various programming languages, including analysis,
//! execution, and test generation.

// Module declarations
pub mod analysis;
pub mod batch;
pub mod operations;
pub mod types;
pub mod test_generation;
pub mod utils;

// New enhanced modules
pub mod confidence;
pub mod safety;
pub mod suggestions;
pub mod enhanced_backup;
pub mod logging;
pub mod progress;

// Re-exports for external use
pub use confidence::ConfidenceScorer;
pub use progress::ProgressTracker;
pub use safety::SafetyAnalyzer;
pub use logging::{RefactoringLogger, SessionType, SessionStatus};
pub use suggestions::SuggestionEngine;
pub use enhanced_backup::EnhancedBackupManager;

// Dependencies
