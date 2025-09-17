//! Module for executing Cargo commands

pub mod build;
pub mod dependency;
pub mod version_alignment;

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;

use anyhow::Result;
use serde::Serialize;
use tokio::process::Command;

use crate::models::{BuildResult, TestResult};
use crate::performance::PerformanceAnalyzer;

/// Execute Cargo build command
pub async fn build(project_path: &Path, release: bool) -> Result<BuildResult> {
    let mut args = vec!["build".to_string()];
    if release {
        args.push("--release".to_string());
    }

    let output = Command::new("cargo")
        .args(&args)
        .current_dir(project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(BuildResult {
        success: output.status.success(),
        output: stdout,
        error: stderr,
    })
}

/// Execute Cargo test command
pub async fn test(project_path: &Path) -> Result<TestResult> {
    let output = Command::new("cargo")
        .args(["test", "--no-fail-fast", "--", "--test-threads=1"])
        .current_dir(project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Parse test results (simplified)
    let passed_tests = stdout.matches("test result: ok").count() as u32;
    let failed_tests = stdout.matches("test result: FAILED").count() as u32;
    let ignored_tests = stdout.matches("test result: ignored").count() as u32;

    Ok(TestResult {
        success: output.status.success(),
        total_tests: passed_tests + failed_tests + ignored_tests,
        passed_tests,
        failed_tests,
        ignored_tests,
        stdout,
        stderr,
    })
}

/// Add a dependency to the project
pub async fn add_dependency(project_path: &Path, name: &str, version: Option<&str>) -> Result<()> {
    let mut args = vec!["add".to_string(), name.to_string()];
    if let Some(v) = version {
        args.push("--version".to_string());
        args.push(v.to_string());
    }

    let status = Command::new("cargo")
        .args(&args)
        .current_dir(project_path)
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to add dependency {}", name);
    }

    Ok(())
}

/// Initialize all command handlers
pub fn init_commands<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize dependency management
    let deps_dir = std::env::current_dir()?.join("target").join("deps_cache");
    std::fs::create_dir_all(&deps_dir)?;

    let manager = std::sync::Arc::new(crate::dependency::DependencyManager::new(
        deps_dir.to_string_lossy().to_string(),
    ));

    // Initialize dependency management with the shared manager
    dependency::init_dependency_management(app, manager.clone())?;

    // Initialize version alignment with the shared manager
    version_alignment::init_version_alignment(app, manager)?;

    // Initialize build commands
    build::init_build_commands(app)?;

    Ok(())
}

/// Format the project code
pub async fn format(project_path: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .args(["fmt"])
        .current_dir(project_path)
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to format code");
    }

    Ok(())
}

/// Run clippy lints
/// Analyze build performance
#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    #[serde(rename = "total_time")]
    pub total_time_ms: f64,
    pub crates: HashMap<String, CrateMetrics>,
    pub dependencies: HashMap<String, f64>,
}

#[derive(Debug, Serialize)]
pub struct CrateMetrics {
    #[serde(rename = "build_time")]
    pub build_time_ms: f64,
    #[serde(rename = "codegen_time")]
    pub codegen_time_ms: f64,
    pub codegen_units: usize,
    pub incremental: bool,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
}

// Helper function to convert Duration to milliseconds as f64
fn duration_to_ms(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1000.0 + f64::from(duration.subsec_micros()) / 1000.0
}

/// Run performance analysis on the project
pub async fn analyze_performance(
    project_path: &Path,
    release: bool,
    incremental: bool,
) -> Result<PerformanceMetrics> {
    // Pass the path reference directly to avoid unnecessary cloning
    let analyzer = PerformanceAnalyzer::new(project_path, release, incremental);
    let metrics = analyzer.analyze_build().await?;

    // Convert to the serializable format with durations as milliseconds
    let mut crates = HashMap::new();
    for (name, crate_metrics) in metrics.crates {
        crates.insert(
            name,
            CrateMetrics {
                build_time_ms: duration_to_ms(crate_metrics.build_time),
                codegen_time_ms: duration_to_ms(crate_metrics.codegen_time),
                codegen_units: crate_metrics.codegen_units,
                incremental: crate_metrics.incremental,
                dependencies: crate_metrics.dependencies,
                features: crate_metrics.features,
            },
        );
    }

    // Convert dependencies to use milliseconds as well
    let dependencies = metrics
        .dependencies
        .into_iter()
        .map(|(k, v)| (k, duration_to_ms(v)))
        .collect();

    Ok(PerformanceMetrics {
        total_time_ms: duration_to_ms(metrics.total_time),
        crates,
        dependencies,
    })
}

pub async fn clippy(project_path: &Path) -> Result<BuildResult> {
    let output = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .current_dir(project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(BuildResult {
        success: output.status.success(),
        output: stdout,
        error: stderr,
    })
}
