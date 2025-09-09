//! Testing command handlers
//!
//! This module contains handlers for test execution and coverage.

use crate::handlers::validation::validate_secure_path;
use std::process::Command;
use tokio::process::Command as TokioCommand;

/// Run tests for a Rust project
#[tauri::command]
pub async fn test_list(project_path: String) -> Result<Vec<String>, String> {
    if let Err(e) = validate_secure_path(&project_path, true) {
        return Err(e);
    }

    let output = Command::new("cargo")
        .args(["test", "--", "--list"])
        .current_dir(&project_path)
        .output()
        .map_err(|e| format!("Failed to list tests: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut tests = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("test ") {
            if let Some(name) = line.strip_prefix("test ").and_then(|s| s.split_whitespace().next()) {
                tests.push(name.to_string());
            }
        }
    }

    Ok(tests)
}

/// Run tests with streaming output
#[tauri::command]
pub async fn test_run_stream(
    project_path: String,
    _test_filter: Option<String>,
    app_handle: tauri::AppHandle,
    command_id: Option<String>,
) -> Result<(), String> {
    if let Err(e) = validate_secure_path(&project_path, true) {
        return Err(e);
    }

    let mut args: Vec<String> = vec!["test".into()];
    // if let Some(f) = test_filter { args.push("--".into()); args.push(f); }

    let command_id = command_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    super::cargo::cargo_execute_stream(
        "cargo".to_string(),
        args,
        project_path,
        Some(command_id),
        app_handle,
    ).await
}

/// Check test coverage availability
#[tauri::command]
pub async fn coverage_is_available() -> Result<bool, String> {
    let has_llvm_cov = std::process::Command::new("cargo").args(["llvm-cov", "--version"]).output().is_ok();
    let has_tarpaulin = std::process::Command::new("cargo").args(["tarpaulin", "--version"]).output().is_ok();
    Ok(has_llvm_cov || has_tarpaulin)
}

/// Run test coverage
#[tauri::command]
pub async fn coverage_run(project_path: String) -> Result<String, String> {
    use std::process::Command;

    if let Err(e) = validate_secure_path(&project_path, true) {
        return Err(e);
    }

    let try_llvm = Command::new("cargo").args(["llvm-cov", "--version"]).output().is_ok();
    let output = if try_llvm {
        Command::new("cargo").args(["llvm-cov", "--json"]).current_dir(&project_path).output()
    } else {
        Command::new("cargo").args(["tarpaulin", "--out", "Stdout"]).current_dir(&project_path).output()
    }.map_err(|e| format!("Failed to run coverage: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(stderr);
    }

    Ok(stdout)
}