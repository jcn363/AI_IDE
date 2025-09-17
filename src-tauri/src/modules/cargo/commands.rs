//! Cargo integration commands module
//!
//! This module provides all Cargo-related Tauri commands for the Rust AI IDE.

use std::path::{Path, PathBuf};

use anyhow::Result;
use rust_ai_ide_core::shell_utils::cargo;
use tokio::sync::mpsc;

// Testing Commands

/// Get list of tests in a Cargo project
#[tauri::command]
pub async fn test_list(project_path: String) -> Result<Vec<String>, String> {
    let project_path_buf = Path::new(&project_path);

    // Use unified cargo test_list utility
    match cargo::test_list(project_path_buf) {
        Ok(result) => {
            if result.success {
                let stdout = result.stdout;
                let mut tests = Vec::new();
                for line in stdout.lines() {
                    if line.starts_with("test ") {
                        if let Some(name) = line
                            .strip_prefix("test ")
                            .and_then(|s| s.split_whitespace().next())
                        {
                            tests.push(name.to_string());
                        }
                    }
                }
                Ok(tests)
            } else {
                Err(result.stderr)
            }
        }
        Err(e) => Err(format!("Failed to execute cargo test --list: {}", e)),
    }
}

/// Run tests in streaming mode
#[tauri::command]
pub async fn test_run_stream(
    project_path: String,
    test_filter: Option<String>,
    app_handle: tauri::AppHandle,
    command_id: Option<String>,
) -> Result<(), String> {
    let mut args: Vec<String> = vec!["test".into()];
    if let Some(f) = test_filter {
        args.push("--".into());
        args.push(f);
    }

    // Generate a command ID or use provided one
    let command_id = match command_id {
        Some(id) => id,
        None => {
            let generated_id = uuid::Uuid::new_v4().to_string();
            log::debug!("No command ID provided, generated: {}", generated_id);
            generated_id
        }
    };

    super::super::cargo::CargoService::execute_command_stream(
        app_handle,
        "cargo",
        args,
        Path::new(&project_path),
        &command_id,
    )
    .await
    .map_err(|e| e.to_string())
}

// Dependency Management Commands

/// Check for dependency updates in a Cargo project
#[tauri::command]
pub async fn check_dependency_updates(
    project_path: String,
) -> Result<Vec<super::super::dependency::DependencyInfo>, String> {
    let checker =
        super::super::dependency::update_checker::DependencyUpdateChecker::new(project_path.into());
    checker.check_updates()
}

/// Update dependencies in a Cargo project
#[tauri::command]
pub async fn update_dependencies(
    manifest_path: String,
    dry_run: bool,
) -> Result<Vec<super::super::dependency::updater::DependencyUpdate>, String> {
    let updater = super::super::dependency::DependencyUpdater::new(manifest_path);
    updater
        .update_dependencies(dry_run)
        .await
        .map_err(|e| e.to_string())
}

/// Batch update dependencies in specified ranges
#[tauri::command]
pub async fn batch_update_dependencies(
    manifest_path: String,
    updates: Vec<(String, String)>,
    dry_run: bool,
) -> Result<super::super::dependency::BatchUpdateResult, String> {
    let updates_ref: Vec<(&str, &str)> = updates
        .iter()
        .map(|(name, version)| (name.as_str(), version.as_str()))
        .collect();

    let updater = super::super::dependency::BatchUpdater::new(manifest_path, dry_run);
    updater
        .update_dependencies(&updates_ref)
        .map_err(|e| e.to_string())
}

// Build Management Commands

/// Initialize build manager for Cargo project
#[tauri::command]
pub async fn init_build_manager(project_path: String) -> Result<(), String> {
    let mut build_manager = super::super::cargo::init_cargo(Path::new(&project_path))
        .await
        .map_err(|e| e.to_string())?;

    // Store the build manager in the app state
    let state = crate::state::AppState::new()?;
    let mut build_managers = state.build_managers.lock().map_err(|e| e.to_string())?;
    build_managers.insert(project_path, build_manager);

    Ok(())
}

/// Get build configuration for a Cargo project
#[tauri::command]
pub async fn get_build_config(
    project_path: String,
) -> Result<super::super::cargo::BuildConfig, String> {
    let state = crate::state::AppState::new()?;
    let build_managers = state.build_managers.lock().map_err(|e| e.to_string())?;
    let manager = build_managers
        .get(&project_path)
        .ok_or_else(|| "Build manager not initialized for this project".to_string())?;

    Ok(manager.get_config().clone())
}

/// Update build configuration for a Cargo project
#[tauri::command]
pub async fn update_build_config(
    project_path: String,
    config: super::super::cargo::BuildConfig,
) -> Result<(), String> {
    let state = crate::state::AppState::new()?;
    let mut build_managers = state.build_managers.lock().map_err(|e| e.to_string())?;
    let manager = build_managers
        .get_mut(&project_path)
        .ok_or_else(|| "Build manager not initialized for this project".to_string())?;

    *manager.get_config_mut() = config;
    manager.save_config().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// Execute a build task for a Cargo project
#[tauri::command]
pub async fn execute_build_task(project_path: String, profile: String) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    // Clone the sender to pass to the build task
    let tx_clone = tx.clone();

    // Spawn the build task
    tokio::spawn(async move {
        let state = match crate::state::AppState::new() {
            Ok(s) => s,
            Err(e) => {
                let _ = tx_clone
                    .send(format!("Error initializing app state: {}", e))
                    .await;
                return;
            }
        };

        let build_managers = match state.build_managers.lock() {
            Ok(m) => m,
            Err(e) => {
                let _ = tx_clone.send(format!("Error acquiring lock: {}", e)).await;
                return;
            }
        };

        let manager = match build_managers.get(&project_path) {
            Some(m) => m,
            None => {
                let _ = tx_clone
                    .send("Build manager not initialized for this project".to_string())
                    .await;
                return;
            }
        };

        let config = manager.get_config().clone();

        // Execute the build
        let result = manager
            .execute_build(Path::new(&project_path), &profile, tx_clone)
            .await;

        // Handle build result
        match result {
            Ok(result) => {
                let status = if result.success {
                    "succeeded"
                } else {
                    "failed"
                };
                let _ = tx
                    .send(format!("Build {} in {:.2?}\n", status, result.duration))
                    .await;
            }
            Err(e) => {
                let _ = tx.send(format!("Build error: {}\n", e)).await;
            }
        }
    });

    // Stream build output back to the frontend
    while let Some(output) = rx.recv().await {
        // This will be handled by the frontend to show build output
        println!("BUILD_OUTPUT:{}", output);
    }

    Ok(())
}
