//! Centralized diagnostic module providing unified types, parsing, and error handling patterns
//!
//! This module consolidates all diagnostic-related functionality from compiler_integration.rs,
//! ai_analysis_commands.rs, and io.rs to eliminate ~47% code duplication.

// Re-export types from the centralized diagnostics module
pub use crate::diagnostics::*;

// Cache for diagnostic results
pub type DiagnosticCacheState = crate::diagnostics::DiagnosticCacheState;

// Cache for error code explanations
pub type ExplanationCacheState = crate::diagnostics::ExplanationCacheState;

// Real-time diagnostic streaming state
pub type DiagnosticStreamState = crate::diagnostics::DiagnosticStreamState;

pub struct DiagnosticStream;

// Submodules (if needed, will be moved later)
// pub mod parsing;
// pub mod caching;
// pub mod error_handling;

// Re-export commonly used types from submodules for convenience (to be updated)
// pub use parsing::*;
// pub use caching::*;
// pub use error_handling::*;
