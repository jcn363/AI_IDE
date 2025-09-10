//! AI-powered code refactoring for Rust AI IDE
//!
//! This crate provides intelligent code refactoring capabilities using machine learning
//! and static analysis to improve code quality and maintainability.

// Module declarations
pub mod analysis;
pub mod batch;
pub mod confidence;
pub mod enhanced_backup;
pub mod logging;
pub mod operations;
pub mod progress;
pub mod safety;
pub mod service;
pub mod suggestions;
pub mod test_generation;
pub mod types;
pub mod utils;

// Re-exports for external use
pub use confidence::ConfidenceScorer;
pub use progress::ProgressTracker;
pub use safety::SafetyAnalyzer;
pub use logging::{RefactoringLogger, SessionType, SessionStatus};
pub use suggestions::SuggestionEngine;
pub use enhanced_backup::EnhancedBackupManager;

// Core AI Refactoring Service - Main Integration Point
pub use service::RefactoringService;

// Re-export commonly used types
pub use types::*;
pub use operations::*;