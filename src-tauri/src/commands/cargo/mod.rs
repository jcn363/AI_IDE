//! Cargo command handler module.
//!
//! This module consolidates all Cargo-related command handlers and their dependencies.
//! It provides functionality for Cargo management, build analysis, dependency features,
//! and build configuration management.

use rust_ai_ide_core::read_file_to_string;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Table;
use uuid;

// Import required types from parent modules
use crate::cargo::{CargoMetadata, CargoService, PerformanceMetrics};

// Re-export types needed by handlers
pub use crate::cargo::{CargoMetadata, CargoService, PerformanceMetrics};

use crate::utils;

/// Dependency information for lock file parsing
#[derive(Debug, Serialize)]
pub struct LockDependency {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub is_direct: bool,
}

/// Cancel a running Cargo command by its command_id
#[tauri::command]
pub async fn cargo_cancel_command(command_id: String) -> Result<bool, String> {
    CargoService::cancel_command(&command_id)
        .await
        .map_err(|e| e.to_string())
}

/// Check if Cargo is available
#[tauri::command]
pub async fn cargo_check_available() -> Result<bool, String> {
    Ok(CargoService::is_available())
}

/// Get Cargo version
#[tauri::command]
pub async fn cargo_get_version() -> Result<String, String> {
    CargoService::get_version().map_err(|e| e.to_string())
}

/// Execute a Cargo command
#[tauri::command]
pub async fn cargo_execute_command(
    command: String,
    args: Vec<String>,
    directory: String,
) -> Result<(String, String, i32), String> {
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    CargoService::execute_command(&command, &args_ref, Path::new(&directory))
        .map_err(|e| e.to_string())
}

/// Execute a Cargo command with real-time streaming via events
#[tauri::command]
pub async fn cargo_execute_stream(
    command: String,
    args: Vec<String>,
    directory: String,
    command_id: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Generate a command ID or use provided one
    let cid = match command_id {
        Some(id) => id,
        None => {
            let generated_id = uuid::Uuid::new_v4().to_string();
            log::debug!(
                "No command ID provided for cargo execution, generated: {}",
                generated_id
            );
            generated_id
        }
    };

    CargoService::execute_command_stream(app_handle, &command, args, Path::new(&directory), &cid)
        .await
        .map_err(|e| e.to_string())
}

/// Get Cargo metadata for a project
#[tauri::command]
pub async fn cargo_get_metadata(project_path: String) -> Result<CargoMetadata, String> {
    CargoService::get_metadata(Path::new(&project_path)).map_err(|e| e.to_string())
}

/// Analyze build performance for a Cargo project
#[tauri::command]
pub async fn cargo_analyze_performance(
    project_path: String,
    release: bool,
    incremental: bool,
) -> Result<PerformanceMetrics, String> {
    CargoService::analyze_performance(Path::new(&project_path), release, incremental)
        .await
        .map_err(|e| e.to_string())
}

// New Cargo commands for enhanced dependency management
#[tauri::command]
pub async fn cargo_generate_dependency_graph(
    project_path: String,
) -> Result<serde_json::Value, String> {
    CargoService::generate_dependency_graph(Path::new(&project_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cargo_get_dependency_info(
    dependency_name: String,
) -> Result<serde_json::Value, String> {
    CargoService::get_dependency_info(&dependency_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cargo_update_all_dependencies(
    project_path: String,
) -> Result<(String, String, i32), String> {
    CargoService::update_all_dependencies(Path::new(&project_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cargo_get_build_profiles(project_path: String) -> Result<Vec<String>, String> {
    CargoService::get_build_profiles(Path::new(&project_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cargo_build_with_profile(
    project_path: String,
    profile: String,
    args: Vec<String>,
) -> Result<(String, String, i32), String> {
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    CargoService::build_with_profile(Path::new(&project_path), &profile, &args_ref)
        .map_err(|e| e.to_string())
}

// Dependency management commands
#[tauri::command]
pub async fn cargo_get_full_metadata_json(
    project_path: String,
) -> Result<serde_json::Value, String> {
    CargoService::get_full_metadata_json(Path::new(&project_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cargo_read_lockfile(project_path: String) -> Result<serde_json::Value, String> {
    CargoService::read_lockfile(Path::new(&project_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cargo_list_features(
    manifest_path: String,
) -> Result<Option<std::collections::HashMap<String, Vec<String>>>, String> {
    CargoService::list_features(Path::new(&manifest_path)).map_err(|e| e.to_string())
}

/// Update dependency features in Cargo.toml (single source of truth).
#[tauri::command]
pub async fn update_dependency_features(
    manifest_path: String,
    dependency_name: String,
    features: Vec<String>,
    default_features: Option<bool>,
) -> Result<(), String> {
    log::info!(
        "Updating features for dependency: {} in {}",
        dependency_name,
        manifest_path
    );

    CargoService::set_dependency_features(
        Path::new(&manifest_path),
        &dependency_name,
        &features,
        default_features,
    )
    .map_err(|e| e.to_string())
}

/// Parse Cargo.lock file and extract dependency information
#[tauri::command]
pub async fn parse_cargo_lock(project_path: PathBuf) -> Result<Vec<LockDependency>, String> {
    let lock_path = project_path.join("Cargo.lock");
    if !lock_path.exists() {
        return Err("Cargo.lock not found".to_string());
    }

    let lock_content = read_file_to_string(&lock_path)
        .await
        .map_err(|e| format!("Failed to read Cargo.lock: {}", e))?;

    let lock_data: Table =
        toml::from_str(&lock_content).map_err(|e| format!("Failed to parse Cargo.lock: {}", e))?;

    let mut dependencies = Vec::new();

    // Get direct dependencies from Cargo.toml for reference
    let direct_deps = get_direct_dependencies(&project_path)
        .await
        .unwrap_or_default();

    if let Some(packages) = lock_data.get("package").and_then(|v| v.as_array()) {
        for pkg in packages {
            if let (Some(name), Some(version)) = (pkg.get("name"), pkg.get("version")) {
                let name_str = name.as_str().unwrap_or("").to_string();
                let version_str = version.as_str().unwrap_or("").to_string();

                let deps = pkg
                    .get("dependencies")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|d| d.as_str())
                            .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();

                dependencies.push(LockDependency {
                    name: name_str.clone(),
                    version: version_str,
                    dependencies: deps,
                    is_direct: direct_deps.contains(&name_str),
                });
            }
        }
    }

    Ok(dependencies)
}

/// Helper function to get direct dependencies from Cargo.toml
async fn get_direct_dependencies(project_path: &PathBuf) -> Result<Vec<String>, String> {
    let toml_path = project_path.join("Cargo.toml");
    if !toml_path.exists() {
        return Ok(Vec::new());
    }

    let toml_content = read_file_to_string(&toml_path)
        .await
        .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

    let cargo_toml: Table =
        toml::from_str(&toml_content).map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;

    let mut deps = Vec::new();

    // Check [dependencies] section
    if let Some(dependencies) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
        deps.extend(dependencies.keys().cloned());
    }

    // Check [dev-dependencies] section
    if let Some(dev_deps) = cargo_toml
        .get("dev-dependencies")
        .and_then(|d| d.as_table())
    {
        deps.extend(dev_deps.keys().cloned());
    }

    // Check [build-dependencies] section
    if let Some(build_deps) = cargo_toml
        .get("build-dependencies")
        .and_then(|d| d.as_table())
    {
        deps.extend(build_deps.keys().cloned());
    }

    // Check workspace dependencies
    if let Some(workspace) = cargo_toml.get("workspace").and_then(|w| w.as_table()) {
        if let Some(workspace_deps) = workspace.get("dependencies").and_then(|d| d.as_table()) {
            deps.extend(workspace_deps.keys().cloned());
        }
    }

    Ok(deps)
}
