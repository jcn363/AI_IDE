//! Shared validation utilities
//!
//! This module consolidates common validation functions from various modules,
//! including path security, string input, dependency format, file size, etc.

use std::path::Path;

/// Path validation functions
pub fn validate_path_security(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    const MAX_PATH_LENGTH: usize = 4096;
    if path.len() > MAX_PATH_LENGTH {
        return Err("Path length exceeds maximum allowed length".to_string());
    }

    // Check for path traversal attacks
    if path.contains("..") || path.contains("\\") {
        return Err("Path traversal detected. Use forward slashes only.".to_string());
    }

    // Prevent absolute path attacks by normalizing
    let clean_path = path.trim_start_matches('/');
    if clean_path != path {
        return Err("Absolute paths are not allowed".to_string());
    }

    let path_obj = Path::new(&clean_path);

    // Validate path components don't contain dangerous characters
    for component in path_obj.components() {
        if let Some(component_str) = component.as_os_str().to_str() {
            if component_str.is_empty() || component_str.contains('\0') {
                return Err("Invalid path component detected".to_string());
            }
        }
    }

    Ok(())
}

/// File size validation
pub fn validate_file_size<P: AsRef<Path>>(path: P, max_size: u64) -> Result<(), String> {
    let metadata =
        std::fs::metadata(path).map_err(|e| format!("Failed to check file metadata: {}", e))?;

    if metadata.len() > max_size {
        return Err("File is too large".to_string());
    }

    Ok(())
}

/// String input validation
pub fn validate_string_input(input: &str, max_length: usize) -> Result<(), String> {
    if input.is_empty() {
        return Err("Input cannot be empty".to_string());
    }

    if input.len() > max_length {
        return Err(format!(
            "Input exceeds maximum length of {} characters",
            max_length
        ));
    }

    Ok(())
}

/// Extended string input validation (for compatibility)
pub fn validate_string_input_ext(
    input: &str,
    max_length: usize,
    allow_special_chars: bool,
) -> Result<(), String> {
    validate_string_input(input, max_length)?;

    if !allow_special_chars {
        if input.contains(";") || input.contains("(") || input.contains(")") {
            return Err("Special characters are forbidden in input".to_string());
        }
    }

    Ok(())
}

/// File size validation for content
pub fn validate_file_size_content(
    content: &[u8],
    max_size_kb: usize,
    operation: &str,
) -> Result<(), String> {
    let size_kb = content.len() / 1024;
    if size_kb > max_size_kb {
        return Err(format!(
            "{}: File size {}KB exceeds maximum allowed size {}KB",
            operation, size_kb, max_size_kb
        ));
    }
    Ok(())
}

/// Validate file size with operation name
pub fn validate_file_size_with_op(
    path: &str,
    max_size_kb: usize,
    operation: &str,
) -> Result<(), String> {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_file() {
                validate_file_size_content(
                    &std::fs::read(path).map_err(|_| format!("Cannot read file: {}", path))?,
                    max_size_kb,
                    operation,
                )
            } else {
                Err(format!("{}: Path is not a file", operation))
            }
        }
        Err(_) => Err(format!("{}: Cannot access file {}", operation, path)),
    }
}

/// Validate path not excluded
pub fn validate_path_not_excluded<P: AsRef<Path>>(
    path: P,
    excluded_paths: &[String],
    operation: &str,
) -> Result<(), String> {
    let path_str = path.as_ref().to_string_lossy();
    for excluded in excluded_paths {
        if path_str.contains(excluded.trim_end_matches('/').trim_start_matches('/')) {
            return Err(format!(
                "{}: Path {} is in excluded paths list",
                operation, path_str
            ));
        }
    }
    Ok(())
}

/// Dependency validation
pub fn validate_dependency_format(dep: &str) -> Result<(), String> {
    if dep.is_empty() {
        return Err("Dependency name cannot be empty".to_string());
    }

    if dep.len() > 256 {
        return Err("Dependency name is too long".to_string());
    }

    // Check for valid package name format (simplified)
    if !dep
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Invalid dependency name format".to_string());
    }

    Ok(())
}

/// AI-related validation
pub fn validate_ai_endpoint(endpoint: &str) -> Result<(), String> {
    if endpoint.is_empty() {
        return Err("AI endpoint cannot be empty".to_string());
    }

    if endpoint.len() > 2048 {
        return Err("AI endpoint URL is too long".to_string());
    }

    Ok(())
}

pub fn validate_ai_model_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Model path cannot be empty".to_string());
    }

    validate_path_security(path).map_err(|_| "Invalid model path".to_string())?;
    Ok(())
}

/// Cargo-specific validation
pub fn validate_cargo_manifest<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();

    if !path.exists() {
        return Err("Cargo.toml does not exist".to_string());
    }

    if !path.is_file() {
        return Err("Cargo.toml is not a file".to_string());
    }

    // Optional: Check if it's actually a valid Cargo.toml by attempting to read it
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if !content.contains("[package]") {
                return Err("Not a valid Cargo.toml - missing [package] section".to_string());
            }
        }
        Err(e) => return Err(format!("Failed to read Cargo.toml: {}", e)),
    }

    Ok(())
}

/// Git repository validation
pub fn validate_git_repo<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();

    if !path.exists() {
        return Err("Directory does not exist".to_string());
    }

    if !path.join(".git").exists() {
        return Err("Not a git repository".to_string());
    }

    Ok(())
}

/// Validate secure path (compatibility wrapper)
pub fn validate_secure_path(path: &str, allow_absolute: bool) -> Result<(), String> {
    validate_path_security(path)?;

    if path.starts_with('/') && !allow_absolute {
        return Err("Absolute paths are not allowed in this context".to_string());
    }

    Ok(())
}
