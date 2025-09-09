#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! Performance monitoring and telemetry for Rust AI IDE
//!
//! This crate provides:
//! - Operation metrics and performance tracking
//! - System health monitoring
//! - Custom metrics collection
//! - Performance timers and utilities

// Removed unused imports: IDEResult, IDEError (not used in this crate)

/// Core metrics module
pub mod metrics;

/// Re-export for convenient access
pub use metrics::*;
