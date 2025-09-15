//! Git command handlers
//!
//! This module contains handlers for Git-related Tauri commands.

use std::path::Path;

use rust_ai_ide_core::shell_utils::git;
use rust_ai_ide_core::validation::validate_secure_path;
use rust_ai_ide_core::{ContextualError, IDEError};

/// Check if Git is available on the system
#[tauri::command]
pub async fn git_is_available() -> Result<bool, String> {
    match git::version() {
        Ok(version) => {
            log::info!("Git version: {}", version);
            Ok(true)
        }
        Err(e) => Err(ContextualError::new(
            IDEError::IoError(format!("Git is not available: {}", e)),
            "Git availability check failed".to_string(),
        )
        .into()),
    }
}

/// Initialize a new Git repository
#[tauri::command]
pub async fn git_init_repo(directory: String) -> Result<String, String> {
    // Validate path security
    validate_secure_path(&directory, true).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(e.to_string()),
            format!("Invalid repository directory: {}", directory),
        )
    })?;

    let path_buf = Path::new(&directory);
    if !path_buf.exists() {
        return Err(ContextualError::new(
            IDEError::IoError(format!("Directory does not exist: {}", directory)),
            "Git init failed: directory not found".to_string(),
        )
        .into());
    }
    if !path_buf.is_dir() {
        return Err(ContextualError::new(
            IDEError::IoError(format!("Path is not a directory: {}", directory)),
            "Git init failed: not a directory".to_string(),
        )
        .into());
    }

    // Use unified git init utility
    match git::init(path_buf) {
        Ok(result) =>
            if result.success {
                Ok(result.stdout.trim().to_string())
            } else {
                Err(ContextualError::new(
                    IDEError::GitError(format!("Git init failed: {}", result.stderr)),
                    "Repository initialization failed".to_string(),
                )
                .into())
            },
        Err(e) => Err(ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute git init: {}", e)),
            format!("Git init command failed in: {}", directory),
        )
        .into()),
    }
}

/// Get Git status information
#[tauri::command]
pub async fn git_status(directory: String) -> Result<String, String> {
    // Validate path security
    validate_secure_path(&directory, true).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(e.to_string()),
            format!("Invalid directory path for git status: {}", directory),
        )
    })?;

    let path_buf = Path::new(&directory);

    // Use unified git status utility
    match git::status(path_buf, true) {
        Ok(result) =>
            if result.success {
                Ok(result.stdout.trim().to_string())
            } else {
                Err(ContextualError::new(
                    IDEError::GitError(format!("Git status failed: {}", result.stderr)),
                    "Git status check failed".to_string(),
                ))
            },
        Err(e) => Err(ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute git status: {}", e)),
            format!("Git status command failed in: {}", directory),
        )),
    }
}

/// Add files to Git staging area
#[tauri::command]
pub async fn git_add(directory: String, paths: Vec<String>) -> Result<(), String> {
    // Validate path security for directory
    validate_secure_path(&directory, true).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(e.to_string()),
            format!("Invalid directory path for git add: {}", directory),
        )
    })?;

    // Validate each additional path
    for path in &paths {
        validate_secure_path(path, true).map_err(|e| {
            ContextualError::new(
                IDEError::ValidationError(e.to_string()),
                format!("Invalid path for git add: {}", path),
            )
        })?;
    }

    let path_buf = Path::new(&directory);

    // Use unified git add utility
    match if paths.is_empty() {
        // Add all files if no specific paths provided
        git::add_all(path_buf)
    } else {
        // Add specific files
        let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        git::add(path_buf, &path_refs)
    } {
        Ok(result) =>
            if result.success {
                Ok(())
            } else {
                Err(ContextualError::new(
                    IDEError::GitError(format!("Git add failed: {}", result.stderr)),
                    "Git add operation failed".to_string(),
                ))
            },
        Err(e) => Err(ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute git add: {}", e)),
            format!("Git add command failed in: {}", directory),
        )),
    }
}

/// Create a Git commit
#[tauri::command]
pub async fn git_commit(
    directory: String,
    message: String,
    author_name: Option<String>,
    author_email: Option<String>,
) -> Result<String, String> {
    // Validate path security
    validate_secure_path(&directory, true).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(e.to_string()),
            format!("Invalid directory path for git commit: {}", directory),
        )
    })?;

    // Validate commit message
    if message.trim().is_empty() {
        return Err(ContextualError::new(
            IDEError::ValidationError("Commit message cannot be empty".to_string()),
            "Git commit failed: empty message".to_string(),
        ));
    }

    let path_buf = Path::new(&directory);

    // Note: Current unified git::commit utility doesn't support author config
    // For now, use a basic commit approach - could enhance the git module later
    match git::commit(path_buf, &message) {
        Ok(result) =>
            if result.success {
                Ok(result.stdout.trim().to_string())
            } else {
                Err(ContextualError::new(
                    IDEError::GitError(format!("Git commit failed: {}", result.stderr)),
                    "Git commit operation failed".to_string(),
                ))
            },
        Err(e) => Err(ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute git commit: {}", e)),
            format!("Git commit command failed in: {}", directory),
        )),
    }
}

/// Get Git commit history
#[tauri::command]
pub async fn git_log(directory: String, limit: Option<u32>) -> Result<String, String> {
    // Validate path security
    validate_secure_path(&directory, true).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(e.to_string()),
            format!("Invalid directory path for git log: {}", directory),
        )
    })?;

    let path_buf = Path::new(&directory);
    let limit_val = limit.unwrap_or(20) as usize;

    // Use unified git log utility
    match git::log(path_buf, Some(limit_val)) {
        Ok(result) =>
            if result.success {
                Ok(result.stdout.trim().to_string())
            } else {
                Err(ContextualError::new(
                    IDEError::GitError(format!("Git log failed: {}", result.stderr)),
                    "Git log operation failed".to_string(),
                ))
            },
        Err(e) => Err(ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute git log: {}", e)),
            format!("Git log command failed in: {}", directory),
        )),
    }
}

/// Show Git diff
#[tauri::command]
pub async fn git_diff(directory: String, path: Option<String>, revspec: Option<String>) -> Result<String, String> {
    // Validate path security
    validate_secure_path(&directory, true).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(e.to_string()),
            format!("Invalid directory path for git diff: {}", directory),
        )
    })?;

    // Validate optional path
    if let Some(ref p) = path {
        validate_secure_path(p, true).map_err(|e| {
            ContextualError::new(
                IDEError::ValidationError(e.to_string()),
                format!("Invalid path for git diff: {}", p),
            )
        })?;
    }

    let path_buf = Path::new(&directory);
    let path_option = path.as_ref().map(|s| s.as_str());

    // Note: Current unified git::diff utility doesn't support revspec
    // Using staged=false and path option for basic diff
    match git::diff(path_buf, false, path_option) {
        Ok(result) =>
            if result.success {
                let output = result.stdout.trim().to_string();
                if output.is_empty() {
                    Ok("No changes to show".to_string())
                } else {
                    Ok(output)
                }
            } else {
                Err(ContextualError::new(
                    IDEError::GitError(format!("Git diff failed: {}", result.stderr)),
                    "Git diff operation failed".to_string(),
                ))
            },
        Err(e) => Err(ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute git diff: {}", e)),
            format!("Git diff command failed in: {}", directory),
        )),
    }
}

/// Show Git blame information
#[tauri::command]
pub async fn git_blame(directory: String, path: String) -> Result<String, String> {
    // Validate path security
    validate_secure_path(&directory, true).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(e.to_string()),
            format!("Invalid directory path for git blame: {}", directory),
        )
    })?;

    // Validate file path
    validate_secure_path(&path, true).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(e.to_string()),
            format!("Invalid file path for git blame: {}", path),
        )
    })?;

    let path_buf = Path::new(&directory);

    // Use unified git blame utility (with line numbers enabled)
    match git::blame(path_buf, &path, true) {
        Ok(result) =>
            if result.success {
                Ok(result.stdout.trim().to_string())
            } else {
                Err(ContextualError::new(
                    IDEError::GitError(format!("Git blame failed: {}", result.stderr)),
                    "Git blame operation failed".to_string(),
                ))
            },
        Err(e) => Err(ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute git blame: {}", e)),
            format!(
                "Git blame command failed in: {} for file: {}",
                directory, path
            ),
        )),
    }
}
