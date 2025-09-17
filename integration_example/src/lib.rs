//! Integration Example: Shared Types Crate
//!
//! This example shows how to integrate the shared-types crate into your existing Rust project.
//! The types defined here will be automatically generated for multiple platforms.
//!
//! ## Module Structure
//!
//! This crate is organized into modules for better maintainability:
//!
//! - `types`: Core data structures and types
//! - `error`: Error handling types and utilities
//! - `bridge`: API bridge functions and utilities

pub mod types;
pub mod error;
pub mod bridge;

// Re-export all types and functions for backward compatibility
pub use types::*;
pub use error::*;
pub use bridge::*;
