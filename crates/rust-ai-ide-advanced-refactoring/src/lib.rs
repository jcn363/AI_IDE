pub mod ai_suggester;
pub mod engine;

pub mod confidence_scorer;
pub mod context_analyzer;
pub mod pattern_recognizer;
pub mod safety_filter;
pub mod suggestion_generator;

pub mod error;
pub mod types;

pub use engine::AdvancedRefactoringEngine;
pub use error::*;
pub use types::*;
