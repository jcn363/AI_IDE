//! Advanced architectural analysis patterns for detecting design issues
//!
//! This module provides sophisticated architectural analysis including:
//! - Circular dependency detection using graph algorithms
//! - Layer violation detection for architectural boundaries
//! - Interface segregation analysis for trait design
//! - Dependency inversion analysis for abstraction usage

pub mod analyzer;
pub mod graph;
pub mod principles;
pub mod types;
pub mod visitors;

// Re-export the main analyzer type for convenience
pub use analyzer::ArchitecturalAnalyzer;
pub use graph::DependencyGraph;
pub use principles::{
    DependencyInversionViolation,
    InterfaceSegregationViolation,
    LayerViolation,
};
