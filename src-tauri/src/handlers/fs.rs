//! File system command handlers
//!
//! This module contains handlers for file system related Tauri commands.

// Import validation from unified rust-ai-ide-common
use std::path::Path;

use rust_ai_ide_common::validation::{
    validate_file_extension, validate_file_size, validate_secure_path, validate_string_input,
};
use rust_ai_ide_common::{file_exists, is_directory, read_dir, read_file_to_bytes};
use rust_ai_ide_common::{ContextualError, IDEError, IDEResult}; // Error types re-exported from common
use sha2::{Digest, Sha256};

// Protocol-specific types not available in current workspace - using local FileInfo
use crate::infra::{EventBus, RateLimiter};
use crate::FileInfo; // Using local FileInfo definition

/// List files in a directory with comprehensive validation
#[tauri::command]
pub async fn list_files(path: String) -> Result<Vec<FileInfo>, String> {
    // Input validation
    if let Err(e) = validate_secure_path(&path, false) {
        return Err(e);
    }

    let path_buf = Path::new(&path);

    // Check if path exists and is a directory using consolidated functions
    if !file_exists(&path_buf) {
        return Err(format!("Directory does not exist: {}", path));
    }

    if !is_directory(&path_buf).await.unwrap_or(false) {
        return Err("Path exists but is not a directory".to_string());
    }

    // Use consolidated read_dir function
    let entries = read_dir(&path).await.map_err(|e| match e {
        rust_ai_ide_common::IdeError::Io { message } if message.contains("NotFound") => {
            format!("Directory does not exist: {}", path)
        }
        rust_ai_ide_common::IdeError::Permission { message } => format!(
            "Permission denied accessing '{}'. Check your permissions.",
            path
        ),
        _ => format!("Failed to read directory '{}': {}", path, e),
    })?;

    let mut files = Vec::new();
    for entry_path in entries {
        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<invalid>")
            .to_string();

        // Check if each entry is a directory using consolidated function
        let is_dir = is_directory(&entry_path).await.unwrap_or(false);

        files.push(FileInfo {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_directory: is_dir,
        });
    }

    // Sort by type (directories first) then by name
    files.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(files)
}

/// Watch file for changes (simplified for now)
#[tauri::command]
pub async fn watch_file(path: String) -> Result<(), String> {
    if let Err(e) = validate_secure_path(&path, true) {
        return Err(e);
    }

    // Use consolidated file_exists function
    if !file_exists(&path).await {
        return Err("File does not exist".to_string());
    }

    // TODO: Implement proper file watching with event bus
    // For now, just return success
    log::debug!("File watching not yet implemented for: {}", path);
    Ok(())
}

/// Unwatch file
#[tauri::command]
pub async fn unwatch_file(path: String) -> Result<(), String> {
    if let Err(e) = validate_secure_path(&path, true) {
        return Err(e);
    }

    // TODO: Implement proper file unwatching
    log::debug!("File unwatching not yet implemented for: {}", path);
    Ok(())
}

/// Get file checksum for integrity checking
#[tauri::command]
pub async fn get_file_checksum(path: String) -> Result<String, String> {
    if let Err(e) = validate_secure_path(&path, true) {
        return Err(e);
    }

    // Use consolidated read_file_to_bytes function
    let content = read_file_to_bytes(&path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Calculate SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
