//! Cargo module for the Rust AI IDE
//!
//! This module provides all Cargo-related functionality including
//! build management, dependency updates, testing, and workspace management.

pub mod commands;

// Re-export commonly used commands
pub use commands::*;

// Application interface
pub use super::cargo::{CargoMetadata, CargoService, PerformanceMetrics};
pub use super::lifecycle::LifecycleManager;
pub use super::state;
