//! Cargo build system integration for the Rust AI IDE.
//! 
//! This module provides functionality for managing Cargo builds, including:
//! - Build task execution
//! - Build hooks (pre/post build)
//! - Environment variable management
//! - Cross-compilation support

pub mod build_task;
pub mod build_manager;
pub mod service;

// Re-exports
pub use build_manager::BuildManager;
pub use build_task::{BuildConfig, BuildHooks, BuildTask};
pub use service::{CargoService, CargoMetadata, CargoManifest, CargoPackage, CargoDependency, PerformanceMetrics, CrateMetrics};

use rust_ai_ide_core::Result;
use std::path::Path;
use tokio::sync::mpsc;

/// Initialize the Cargo build system for a project
pub async fn init_cargo(project_path: &Path) -> Result<BuildManager> {
    let mut manager = BuildManager::new();
    manager.load_config(project_path).await?;
    Ok(manager)
}

/// Execute a Cargo build with the given configuration
pub async fn execute_build(
    project_path: &Path,
    profile: &str,
    config: BuildConfig,
    tx: mpsc::Sender<String>,
) -> Result<build_manager::BuildResult> {
    let manager = BuildManager {
        config,
        config_path: None,
    };
    manager.execute_build(project_path, profile, tx).await
}

/// Get the default build configuration
pub fn default_config() -> BuildConfig {
    BuildConfig::default()
}
