//! Project management handlers
//!
//! This module contains handlers for project build, run, and test operations.

// Import validation from unified rust-ai-ide-core and rust-ai-ide-common
use std::fs;
use std::path::Path;

use rust_ai_ide_common::validation::validate_file_extension;
use rust_ai_ide_core::shell_utils::{execute_command, CommandResult};
use rust_ai_ide_core::validation::{validate_file_size, validate_rust_project, validate_secure_path};
use rust_ai_ide_core::{ContextualError, IDEError};

#[tauri::command]
pub async fn build_project(project_path: String) -> Result<String, String> {
    use rust_ai_ide_core::validation::validate_project_path;

    // Validate project path comprehensively using unified rust-ai-ide-core validation
    let project_path_buf = Path::new(&project_path);
    let validation_result = rust_ai_ide_core::validation::validate_rust_project(project_path_buf)
        .map_err(|e| format!("Project validation failed: {}", e))?;

    if !validation_result.is_valid {
        return Err(format!(
            "Project validation failed: {:?}",
            validation_result.errors
        ));
    }

    log::info!("Building project at: {}", project_path);

    // Execute cargo build using unified command execution
    let result = execute_command("cargo", &["build", "--message-format=json"]).map_err(|e| {
        ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute cargo build: {}", e)),
            format!("Project build failed: {}", project_path),
        )
    })?;

    if result.success {
        log::info!("Build completed successfully");
        Ok("Build completed successfully".to_string())
    } else {
        let error_msg = result
            .stderr
            .unwrap_or_else(|| "Build failed without error output".to_string());
        log::error!("Build failed: {}", error_msg);
        Err(ContextualError::new(
            IDEError::BuildError(error_msg.clone()),
            format!("Cargo build failed for project: {}", project_path),
        )
        .into())
    }
}

#[tauri::command]
pub async fn run_project(project_path: String) -> Result<String, String> {
    use std::process::Stdio;

    use tokio::process::Command as TokioCommand;

    log::info!("Running project at: {}", project_path);

    // Validate project path using unified rust-ai-ide-core validation
    let project_path_buf = Path::new(&project_path);
    let validation_result = rust_ai_ide_core::validation::validate_rust_project(project_path_buf).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(format!("Project validation failed: {}", e)),
            format!("Invalid project path for run: {}", project_path),
        )
    })?;

    if !validation_result.is_valid {
        return Err(ContextualError::new(
            IDEError::ValidationError(format!(
                "Project validation failed: {:?}",
                validation_result.errors
            )),
            format!("Invalid Rust project for run: {}", project_path),
        )
        .into());
    }

    // Check if project path exists
    if !project_path_buf.exists() {
        return Err(ContextualError::new(
            IDEError::IoError(format!("Project path does not exist: {}", project_path)),
            "Run project failed: path not found".to_string(),
        )
        .into());
    }

    // Build the project first to ensure it's up to date
    build_project(project_path.clone()).await.map_err(|e| {
        ContextualError::new(
            e,
            format!("Failed to build project before running: {}", project_path),
        )
    })?;

    // Get the target directory
    let target_dir = project_path_buf.join("target/debug");

    // Find the binary (simplified - assumes single binary project)
    let bin_name = std::fs::read_dir(&target_dir)
        .map_err(|e| {
            ContextualError::new(
                IDEError::IoError(format!("Failed to read target directory: {}", e)),
                "Run project failed: target directory error".to_string(),
            )
        })?
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();
            path.is_file()
                && path.extension().is_none()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| !name.contains('.') && !name.starts_with('.'))
                    .unwrap_or(false)
        })
        .next()
        .ok_or_else(|| {
            ContextualError::new(
                IDEError::IoError("No binary found in target/debug directory".to_string()),
                format!("Run project failed for: {}", project_path),
            )
        })?;

    // Execute the binary
    let child = TokioCommand::new(bin_name.path())
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            ContextualError::new(
                IDEError::CommandExecutionError(format!("Failed to start project: {}", e)),
                format!("Process spawn failed for project: {}", project_path),
            )
        })?;

    // Store the process ID for potential later use (e.g., stopping the process)
    let pid = child.id();
    log::info!("Started process with PID: {:?}", pid);

    Ok(format!("Project started with PID: {:?}", pid))
}

#[tauri::command]
pub async fn test_project(project_path: String) -> Result<String, String> {
    use std::process::Stdio;

    use tokio::process::Command;

    log::info!("Running tests for project at: {}", project_path);

    // Validate project path using unified rust-ai-ide-core validation
    let project_path_buf = Path::new(&project_path);
    let validation_result = rust_ai_ide_core::validation::validate_rust_project(project_path_buf).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(format!("Project validation failed: {}", e)),
            format!("Invalid project path for test: {}", project_path),
        )
    })?;

    if !validation_result.is_valid {
        return Err(ContextualError::new(
            IDEError::ValidationError(format!(
                "Project validation failed: {:?}",
                validation_result.errors
            )),
            format!("Invalid Rust project for test: {}", project_path),
        )
        .into());
    }

    // Check if project path exists
    if !project_path_buf.exists() {
        return Err(ContextualError::new(
            IDEError::IoError(format!("Project path does not exist: {}", project_path)),
            "Test project failed: path not found".to_string(),
        )
        .into());
    }

    // Execute cargo test command using unified command execution
    let result = execute_command("cargo", &["test", "--", "--nocapture"]).map_err(|e| {
        ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute cargo test: {}", e)),
            format!("Test execution failed for project: {}", project_path),
        )
    })?;

    // Capture and log the output
    let stdout = result.stdout.unwrap_or_else(|| String::new());
    let stderr = result.stderr.unwrap_or_else(|| String::new());

    log::debug!("Test stdout: {}", stdout);
    log::debug!("Test stderr: {}", stderr);

    // Check test status
    if result.success {
        log::info!("All tests passed successfully");
        Ok(stdout)
    } else {
        log::error!("Tests failed: {}", stderr);
        Err(ContextualError::new(
            IDEError::TestError(stderr.clone()),
            format!("Cargo tests failed for project: {}", project_path),
        )
        .into())
    }
}

/// Generate documentation for a Rust project
#[tauri::command]
pub async fn doc_generate(project_path: String) -> Result<String, String> {
    // Validate project path using unified rust-ai-ide-core validation
    let project_path_buf = Path::new(&project_path);
    let validation_result = rust_ai_ide_core::validation::validate_rust_project(project_path_buf).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(format!("Project validation failed: {}", e)),
            format!("Invalid project path for doc generation: {}", project_path),
        )
    })?;

    if !validation_result.is_valid {
        return Err(ContextualError::new(
            IDEError::ValidationError(format!(
                "Project validation failed: {:?}",
                validation_result.errors
            )),
            format!("Invalid Rust project for doc generation: {}", project_path),
        )
        .into());
    }

    // Execute cargo doc using unified command execution
    let result = execute_command("cargo", &["doc", "--no-deps"]).map_err(|e| {
        ContextualError::new(
            IDEError::CommandExecutionError(format!("Failed to execute cargo doc: {}", e)),
            format!(
                "Documentation generation failed for project: {}",
                project_path
            ),
        )
    })?;

    if result.success {
        let index = project_path_buf.join("target/doc/index.html");
        Ok(index.to_string_lossy().to_string())
    } else {
        let error_msg = result
            .stderr
            .unwrap_or_else(|| "Documentation generation failed without error output".to_string());
        log::error!("Documentation generation failed: {}", error_msg);
        Err(ContextualError::new(
            IDEError::CommandExecutionError(error_msg.clone()),
            format!("Cargo doc failed for project: {}", project_path),
        )
        .into())
    }
}

/// Read documentation file
#[tauri::command]
pub async fn doc_read_file(path: String) -> Result<String, String> {
    // Unified path validation using rust-ai-ide-core
    validate_secure_path(&path, false).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(format!("Path validation failed: {}", e)),
            format!("Invalid documentation file path: {}", path),
        )
    })?;

    // Validate file extension for HTML files
    let path_obj = Path::new(&path);
    validate_file_extension(path_obj, &["html", "htm"]).map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(format!("File extension validation failed: {}", e)),
            format!("Invalid file extension for documentation: {}", path),
        )
    })?;

    // Limit file size to prevent resource exhaustion
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit
    validate_file_size(
        fs::metadata(&path)
            .map_err(|e| {
                ContextualError::new(
                    IDEError::IoError(format!("Failed to get file metadata: {}", e)),
                    format!("Cannot read file metadata: {}", path),
                )
            })?
            .len(),
        Some(MAX_FILE_SIZE),
    )
    .map_err(|e| {
        ContextualError::new(
            IDEError::ValidationError(format!("File size validation failed: {}", e)),
            format!("File too large: {}", path),
        )
    })?;

    // Read the file
    fs::read_to_string(&path).map_err(|e| {
        ContextualError::new(
            IDEError::IoError(format!("Failed to read documentation file: {}", e)),
            format!("Cannot read file: {}", path),
        )
        .into()
    })
}
