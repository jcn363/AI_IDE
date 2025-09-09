//! Shared services for LSP operations and unified workspace handling
//!
//! This crate provides common services for language server protocol operations,
//! workspace management, and other shared service functionality across the
//! Rust AI IDE project.

// Re-export workspace functionality
pub use workspace::{WorkspaceConfig, WorkspaceManager, WorkspaceManagerTrait};

pub mod completion;
pub mod diagnostics;
pub mod lsp;
pub mod workspace;
