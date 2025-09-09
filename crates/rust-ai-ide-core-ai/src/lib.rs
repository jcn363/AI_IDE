#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! AI abstractions, analysis interfaces, and AI provider integrations for Rust AI IDE
//!
//! This crate provides:
//! - Unified AI provider abstractions
//! - Analysis interfaces and data structures
//! - Common AI result types and configurations
//! - Async analysis capabilities

// Re-exports for easier access to fundamentals error types
use rust_ai_ide_core_fundamentals::error::*;

/// AI provider abstractions module
pub mod ai_types;

/// Analysis interfaces module
pub mod analysis;

// Re-exports for convenient access
pub use ai_types::*;
pub use analysis::*;
