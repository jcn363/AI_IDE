// Temporarily disable missing_docs for development
// #![deny(missing_docs)]
#![allow(dead_code)]
#![warn(clippy::all, clippy::pedantic)]

//! Shared utilities for Rust AI IDE
//!
//! This crate provides canonical implementations of utility functions
//! used across multiple crates in the Rust AI IDE codebase.

// Re-export modules for backward compatibility
pub mod file;
pub mod fs;
pub mod path;

// Re-export commonly used functions at crate root for backward compatibility
pub use file::{get_extension, is_code_file};
pub use fs::{get_file_size, get_file_size_cached, is_readable, is_writable};
pub use path::{normalize_path, relative_path};
