#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! Core types, utilities, error handling, and fundamental functionality for Rust AI IDE
//!
//! This crate provides the bedrock layer with common types, error definitions,
//! utility functions, and fundamental traits used across the Rust AI IDE ecosystem.

pub mod error;
pub mod formatters;
pub mod utils;

/// General-purpose utility traits
pub mod traits {
    use std::path::Path;

    /// Trait for objects that can be validated
    pub trait Validatable {
        type Error;

        fn validate(&self) -> Result<(), Self::Error>;
        fn is_valid(&self) -> bool {
            self.validate().is_ok()
        }
    }

    /// Trait for objects that can be displayed in debug format
    pub trait DebugDisplay: std::fmt::Debug + std::fmt::Display {
        fn debug_str(&self) -> String {
            format!("{:?}", self)
        }
    }

    /// Extension trait for Path operations common in IDE operations
    pub trait PathExt {
        fn readable_name(&self) -> String;
        fn is_workspace_root(&self) -> bool;
        fn find_ancestor_with(&self, file_name: &str) -> Option<std::path::PathBuf>;
        fn parent_count(&self) -> usize;
    }

    impl PathExt for Path {
        fn readable_name(&self) -> String {
            self.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        }

        fn is_workspace_root(&self) -> bool {
            self.join("Cargo.toml").exists() && self.join("src").exists() && self.join("src/lib.rs").exists()
        }

        fn find_ancestor_with(&self, file_name: &str) -> Option<std::path::PathBuf> {
            let mut current = Some(self.to_path_buf());
            while let Some(path) = current {
                if path.join(file_name).exists() {
                    return Some(path);
                }
                current = path.parent().map(|p| p.to_path_buf());
            }
            None
        }

        fn parent_count(&self) -> usize {
            let mut count = 0;
            let mut current = self.parent();

            while let Some(parent) = current {
                count += 1;
                current = parent.parent();
            }

            count
        }
    }

    /// Trait for result types that need special handling
    pub trait ResultExt<T, E> {
        fn with_context<F>(self, context_fn: F) -> Result<T, E>
        where
            F: FnOnce(E) -> E;

        fn warn_on_error(self, message: &str) -> Self;
    }

    impl<T, E> ResultExt<T, E> for Result<T, E> {
        fn with_context<F>(self, context_fn: F) -> Result<T, E>
        where
            F: FnOnce(E) -> E,
        {
            self.map_err(context_fn)
        }

        fn warn_on_error(self, message: &str) -> Self {
            if let Err(_) = &self {
                log::warn!("{}", message);
            }
            self
        }
    }
}

/// Common constants used across the IDE
pub mod constants {

    /// Default timeout for network operations
    pub const NETWORK_TIMEOUT_SECS: u64 = 30;

    /// Default timeout for file operations
    pub const FILE_TIMEOUT_SECS: u64 = 10;

    /// Default cache TTL for diagnostics
    pub const DIAGNOSTICS_CACHE_TTL_SECS: u64 = 300;

    /// Default cache TTL for AI suggestions
    pub const AI_CACHE_TTL_SECS: u64 = 600;

    /// Maximum file size for in-memory processing (100MB)
    pub const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;

    /// Maximum number of open files
    pub const MAX_OPEN_FILES: usize = 1000;

    /// Default buffer size for file operations
    pub const DEFAULT_BUFFER_SIZE: usize = 8192;

    /// Workspace detection files
    pub const WORKSPACE_FILES: &[&str] = &["Cargo.toml", "package.json", "setup.py"];

    /// File extensions for source code
    pub const SOURCE_EXTENSIONS: &[&str] = &[
        "rs", "js", "ts", "py", "java", "cpp", "c", "hpp", "h", "cs", "php", "rb", "go", "swift", "kt", "scala", "clj",
        "hs", "ml", "fs", "ex",
    ];

    /// Temporary directory prefix
    pub const TEMP_PREFIX: &str = "rust-ai-ide";

    /// Configuration file names to look for
    pub const CONFIG_FILES: &[&str] = &["rust-ai-ide.toml", ".rust-ai-ide.toml"];
}

// Re-export commonly used items for convenience
// Re-export constants
pub use constants::*;
pub use error::{AIResult, AnalysisResult, IDEError, IDEResult};
// Re-export traits
pub use traits::*;
pub use utils::{
    async_utils, config_merge, conversion, id_utils, log_utils, macros, path_utils, perf_utils, sanitization,
    system_info, validation,
};
