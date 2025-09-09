#![feature(unix_send_signal)]

//! Cargo integration for Rust AI IDE
//!
//! This crate provides integration with Cargo, Rust's build tool and package manager.
//! It handles project creation, building, testing, and dependency management.
//!
//! # Modules
//! - `build`: Build system implementation with real-time monitoring
//! - `commands`: Execution of Cargo commands
//! - `models`: Data structures for Cargo project metadata
//! - `performance`: Performance analysis tools
//! - `refactor`: Code refactoring operations
//! - `task`: Task management
//! - `utils`: Utility functions
//! - `workspace`: Workspace management and project structure
//! - `dependency`: Dependency management and Cargo.toml editing
//! - `version_alignment`: Workspace-wide version alignment for dependencies

pub mod build;
pub mod commands;
pub mod dependency;
pub mod models;
pub mod performance;
pub mod refactor;
pub mod task;
pub mod utils;
pub mod workspace;

// Re-exports for easier access to commonly used items
pub use dependency::{DependencyInfo, DependencyKind, DependencyManager, VersionAlignment};
pub use models::{BuildResult, CargoDepVersion, CargoProject, TestResult};
pub use performance::{
    generate_flamegraph, BuildMetrics, BuildMetricsCollector, CachedBuildResult, CargoBuildCache,
    CrateMetrics, OptimizationSuggestion, PerformanceAnalyzer, PerformanceMetrics,
};
pub use refactor::{find_references, get_dependency_graph, rename_symbol, workspace_replace};
pub use task::{
    CargoTask, CommandExecutor, CommandHistory, CommandStatus, ExecutionStrategy, TaskChain,
    TaskMonitor, TaskStatus,
};
pub use workspace::CargoManager;

// Re-export common types for convenience
pub use anyhow::Result;
pub use std::collections::HashMap;
pub use std::fs;
pub use std::path::{Path, PathBuf};
pub use std::process::Command;
