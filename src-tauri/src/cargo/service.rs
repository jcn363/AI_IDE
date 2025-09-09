//! Cargo service integration for the Rust AI IDE.
//!
//! This module provides Cargo-related operations including:
//! - Version checking and availability
//! - Command execution and streaming
//! - Metadata management
//! - Dependency management

use std::process::Command;
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use once_cell::sync::Lazy;

static RUNNING: Lazy<Mutex<HashMap<String, Arc<Mutex<std::process::Child>>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct CargoService;

impl CargoService {
    /// Check if Cargo is available
    pub fn is_available() -> bool {
        Command::new("cargo")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get Cargo version
    pub fn get_version() -> Result<String, String> {
        let output = Command::new("cargo")
            .arg("--version")
            .output()
            .map_err(|e| format!("Failed to execute cargo --version: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Execute a Cargo command
    pub fn execute_command(
        command: &str,
        args: &[&str],
        directory: &Path,
    ) -> Result<(String, String, i32), String> {
        let output = Command::new("cargo")
            .arg(command)
            .args(args)
            .current_dir(directory)
            .output()
            .map_err(|e| format!("Failed to execute cargo command: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        Ok((stdout, stderr, exit_code))
    }

    /// Get Cargo metadata for a project
    pub fn get_metadata(project_path: &Path) -> Result<CargoMetadata, String> {
        let output = Command::new("cargo")
            .args(&["metadata", "--format-version=1", "--no-deps"])
            .current_dir(project_path)
            .output()
            .map_err(|e| format!("Failed to execute cargo metadata: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to get cargo metadata: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let metadata: CargoMetadata = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Failed to parse cargo metadata: {}", e))?;

        Ok(metadata)
    }

    /// Get full Cargo metadata (raw JSON) for advanced use cases
    pub fn get_full_metadata_json(project_path: &Path) -> Result<serde_json::Value, String> {
        let output = Command::new("cargo")
            .args(["metadata", "--format-version=1"])
            .current_dir(project_path)
            .output()
            .map_err(|e| format!("Failed to execute cargo metadata: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to get cargo metadata: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let value: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Failed to parse cargo metadata JSON: {}", e))?;
        Ok(value)
    }

    /// Execute a Cargo command and stream stdout/stderr as Tauri events
    pub async fn execute_command_stream(
        app_handle: tauri::AppHandle,
        command: &str,
        args: Vec<String>,
        directory: &Path,
        command_id: &str,
    ) -> Result<(), String> {
        // Emit start event
        let start = serde_json::json!({
            "command_id": command_id,
            "command": command,
            "args": args.clone(),
            "cwd": directory.display().to_string(),
            "ts": chrono::Utc::now().timestamp_millis(),
        });
        app_handle.emit("cargo:command-start", start).map_err(|e| e.to_string())?;

        // Determine if JSON diagnostics are requested
        let json_mode = args.iter().any(|a| a == "--message-format=json");

        // Spawn cargo using tokio for async IO
        let child = TokioCommand::new("cargo")
            .arg(command)
            .args(&args)
            .current_dir(directory)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn cargo process: {}", e))?;

        // Wrap child for shared access and track running process
        let child_arc = Arc::new(Mutex::new(child));
        {
            let mut map = RUNNING.lock().await;
            map.insert(command_id.to_string(), Arc::clone(&child_arc));
        }

        // Take stdout/stderr under lock
        let (stdout, stderr) = {
            let mut locked = child_arc.lock().await;
            let stdout = locked.stdout.take().ok_or("Failed to capture stdout")?;
            let stderr = locked.stderr.take().ok_or("Failed to capture stderr")?;
            (stdout, stderr)
        };

        let app_handle_stdout = app_handle.clone();
        let app_handle_stderr = app_handle.clone();
        let cid_stdout = command_id.to_string();
        let cid_stderr = command_id.to_string();

        // Stream stdout
        let stdout_task = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_handle_stdout.emit(
                    "cargo:command-output",
                    serde_json::json!({
                        "commandId": cid_stdout,
                        "stream": "stdout",
                        "line": line,
                        "ts": chrono::Utc::now().timestamp_millis(),
                    }),
                );
                // Additionally, if JSON mode, attempt to parse and emit structured diagnostics
                if json_mode {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                        if val.get("message").is_some() && val.get("spans").is_some() {
                            let _ = app_handle_stdout.emit(
                                "cargo:command-diagnostic",
                                serde_json::json!({
                                    "commandId": cid_stdout,
                                    "payload": val,
                                }),
                            );
                        }
                    }
                }
            }
        });

        // Stream stderr
        let stderr_task = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_handle_stderr.emit(
                    "cargo:command-output",
                    serde_json::json!({
                        "commandId": cid_stderr,
                        "stream": "stderr",
                        "line": line,
                        "ts": chrono::Utc::now().timestamp_millis(),
                    }),
                );
                // Additionally, if JSON mode, attempt to parse and emit structured diagnostics
                if json_mode {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                        if val.get("message").is_some() && val.get("spans").is_some() {
                            let _ = app_handle_stderr.emit(
                                "cargo:command-diagnostic",
                                serde_json::json!({
                                    "commandId": cid_stderr,
                                    "payload": val,
                                }),
                            );
                        }
                    }
                }
            }
        });

        // Wait for completion
        let status = {
            let mut locked = child_arc.lock().await;
            locked.wait().await.map_err(|e| format!("Failed to wait on cargo process: {}", e))?
        };
        // Remove from registry when finished
        {
            let mut map = RUNNING.lock().await;
            map.remove(command_id);
        }
        let _ = tokio::join!(stdout_task, stderr_task);

        // Emit finish event
        app_handle.emit(
            "cargo:command-finish",
            serde_json::json!({
                "commandId": command_id,
                "code": status.code().unwrap_or(-1),
                "ts": chrono::Utc::now().timestamp_millis(),
            }),
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Cancel a running Cargo command by its command_id
    pub async fn cancel_command(command_id: &str) -> tokio::io::Result<bool> {
        let opt = {
            let mut map = RUNNING.lock().await;
            map.remove(command_id)
        };
        if let Some(child_arc) = opt {
            let mut child = child_arc.lock().await;
            let _ = child.kill().await; // ignore errors if already exited
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Analyze build performance for a Cargo project
    pub async fn analyze_performance(
        project_path: &Path,
        release: bool,
        incremental: bool,
    ) -> Result<PerformanceMetrics, String> {
        // For now, return a simple metrics structure
        // This could be extended to actually analyze build performance
        let metrics = PerformanceMetrics {
            total_time: 100,
            crates: HashMap::new(),
            dependencies: HashMap::new(),
            features: Vec::new(),
        };

        Ok(metrics)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub manifest_path: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CargoMetadata {
    pub packages: Vec<CargoPackage>,
    pub workspace_root: String,
    pub target_directory: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CargoDependency {
    pub name: String,
    pub req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: String,
    pub source: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CargoManifest {
    pub package: CargoPackage,
    pub dependencies: HashMap<String, serde_json::Value>,
    pub features: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PerformanceMetrics {
    pub total_time: u64,
    pub crates: HashMap<String, CrateMetrics>,
    pub dependencies: HashMap<String, u64>,
    pub features: Vec<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CrateMetrics {
    pub build_time: u64,
    pub codegen_time: Option<u64>,
    pub codegen_units: Option<u32>,
    pub incremental: Option<bool>,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
}