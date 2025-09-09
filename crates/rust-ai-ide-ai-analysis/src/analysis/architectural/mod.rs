//! Architectural analysis module for AI-powered code analysis
//!
//! This module provides comprehensive architectural pattern detection,
//! anti-pattern analysis, and ML-enhanced confidence scoring for intelligent
//! code analysis in the Rust AI IDE.

pub mod patterns;
pub mod anti_patterns;
pub mod ml_scorer;
pub mod detectors;
pub mod suggester;
pub mod dummy_cache;

pub use patterns::*;
pub use anti_patterns::*;
pub use ml_scorer::*;
pub use detectors::*;
pub use suggester::*;