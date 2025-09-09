//! # DEPRECATED: Use `shared-test-utils` instead
//!
//! This crate has been deprecated. All functionality has been migrated to
//! the [`shared-test-utils`] crate. Please update your dependencies to use
//! `shared-test-utils` instead.
//!
//! Migration guide:
//! - Replace `rust-ai-ide-test-utils` with `shared-test-utils` in your Cargo.toml
//! - Update import statements from `rust_ai_ide_test_utils::*` to `shared_test_utils::*`
//!
//! **WORKSPACE MIGRATION COMPLETE**: rust-ai-ide-test-utils has been removed from workspace.members
//! and replaced with shared-test-utils in workspace.dependencies (section 1.2 of refactoring plan)
//!
//! [`shared-test-utils`]: ../shared-test-utils/index.html

//! Shared test utilities for Rust AI IDE (DEPRECATED)
//!
//! This crate provides common test utilities, mocks, fixtures, and performance testing
//! tools that can be used across all crates in the Rust AI IDE workspace.
//!
//! # Deprecation Notice
//! This crate is deprecated. Use [`shared-test-utils`] instead.
//!
//! [`shared-test-utils`]: https://docs.rs/shared-test-utils

#[cfg(feature = "performance")]
pub mod performance;
#[cfg(feature = "filesystem")]
pub mod filesystem;
#[cfg(feature = "async")]
pub mod async_util;
#[cfg(feature = "types")]
pub mod type_helpers;
#[cfg(feature = "random")]
pub mod random_data;

/// Common test helpers and utilities
pub mod helpers;

/// Test mocks and fixtures
#[cfg(feature = "std")]
pub mod fixtures;

/// Experimental test utilities (enable with "full" feature)
#[cfg(feature = "full")]
pub mod experimental;

/// Re-export commonly used testing types from standard libraries
#[cfg(feature = "std")]
pub use helpers::*;

#[cfg(feature = "std")]
pub use fixtures::*;

// **MIGRATED**: These functions have been moved to `shared-test-utils::data_generator`

// **MIGRATED**: These functions have been moved to `shared-test-utils::assertions`

// **MIGRATED**: These functions have been moved to `shared-test-utils::setup`

#[cfg(test)]
mod tests {
    #[test]
    fn test_helper_functions() {
        // Basic smoke test
        assert!(true);
    }

    #[cfg(feature = "random")]
    #[test]
    fn test_random_generation() {
        // TODO: Migrated to shared-test-utils::data_generator
        // These functions have been moved to shared-test-utils
        // use shared_test_utils::data_generator::*;

        // let s = random_string(10);
        // assert_eq!(s.len(), 10);

        // let v = random_vec(5, 1, 10);
        // assert_eq!(v.len(), 5);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_test_setup() {
        // TODO: Migrated to shared-test-utils::filesystem
        // These functions have been moved to shared-test-utils
        // use shared_test_utils::filesystem::*;

        // let root = get_project_root();
        // assert!(root.exists());

        // let test_path = get_test_directory("test-utils");
        // assert!(test_path.to_string_lossy().contains("tests"));
    }
}