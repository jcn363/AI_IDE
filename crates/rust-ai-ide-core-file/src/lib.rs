#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! File system operations and path management for Rust AI IDE
//!
//! This crate provides comprehensive file system utilities including:
//! - Safe file operations with error handling
//! - Directory management and traversal
//! - Path utilities for IDE operations
//! - File hashing and checksums
//! - Workspace management and analysis

pub use rust_ai_ide_core_fundamentals::constants::*;
pub use rust_ai_ide_core_fundamentals::error::{IDEError, IDEResult};
pub use rust_ai_ide_core_fundamentals::traits::*;

pub mod fs_utils;
pub mod workspace;

// Re-exports for convenience
pub use fs_utils::*;
pub use workspace::*;
