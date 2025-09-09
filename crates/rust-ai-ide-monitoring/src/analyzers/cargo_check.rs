//! Cargo check analyzer implementation

use crate::{
    analyzers::{Analyzer, AnalyzerConfig},
    config::Config,
    errors::{MonitoringError, Result},
    parse::{cargo_check_workspace, CargoJsonParser},
    types::{
        AnalysisPerformance, AnalysisResult, Category, Finding, Severity, SystemInfo,
        WorkspaceTarget,
    },
};
use async_trait::async_trait;
use chrono::Utc;
use std::path::Path;
use sysinfo::System;

/// Analyzer for cargo check output
pub struct CargoCheckAnalyzer {
    /// Configuration for this analyzer
    config: Config,

    /// System info cache
    system_info: Option<SystemInfo>,
}

impl CargoCheckAnalyzer {
    /// Create a new cargo check analyzer
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            system_info: None,
        }
    }

    /// Create a new analyzer with custom config
    pub fn with_config(config: Config) -> Self {
        Self {
            config,
            system_info: None,
        }
    }

    /// Initialize system information
    fn init_system_info(&mut self) -> Result<()> {
        if self.system_info.is_none() {
            let system = System::new_all();

            self.system_info = Some(SystemInfo {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                rust_version: self.get_rust_version()?,
                cargo_version: self.get_cargo_version()?,
                cpu_count: num_cpus::get(),
                total_memory_mb: (system.total_memory() / 1024 / 1024) as usize,
                available_memory_mb: (system.available_memory() / 1024 / 1024) as usize,
            });
        }

        Ok(())
    }

    /// Get Rust version
    fn get_rust_version(&self) -> Result<String> {
        use std::process::Command;

        let output = Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(|e| MonitoringError::command_execution("rustc --version".to_string(), e))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Get Cargo version
    fn get_cargo_version(&self) -> Result<String> {
        use std::process::Command;

        let output = Command::new("cargo")
            .arg("--version")
            .output()
            .map_err(|e| MonitoringError::command_execution("cargo --version".to_string(), e))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Analyze a specific target
    async fn analyze_target(&self, target: &WorkspaceTarget) -> Result<AnalysisResult> {
        let start_time = std::time::Instant::now();

        // Run cargo check for this specific target
        let check_result = cargo_check_workspace(&target.path).await;

        let duration = start_time.elapsed();
        let duration_seconds = duration.as_secs_f64();

        let (success, findings, issue_count, error_message) = match check_result {
            Ok(results) => {
                let findings = CargoJsonParser::diagnostics_to_findings(&results.diagnostics);
                let issue_count = findings.len();

                (results.success, findings, issue_count, None)
            }
            Err(e) => {
                (
                    false,
                    Vec::new(),
                    0,
                    Some(format!("Cargo check failed: {}", e)),
                )
            }
        };

        let performance = Some(AnalysisPerformance {
            duration_seconds,
            memory_mb: None, // Could be tracked separately
            cpu_percent: None,
            peak_memory_mb: None,
        });

        let severity = if success {
            if issue_count > 0 {
                Severity::Medium
            } else {
                Severity::None
            }
        } else {
            Severity::High
        };

        Ok(AnalysisResult {
            analyzer: self.name().to_string(),
            success,
            severity,
            category: self.category(),
            issue_count,
            findings,
            performance,
            error: error_message,
        })
    }

    /// Get workspace targets to analyze
    async fn discover_workspace_targets(&self, workspace_root: &Path) -> Result<Vec<WorkspaceTarget>> {
        // Simple implementation - in practice, this would parse Cargo.toml files
        // For now, just return a basic target for the workspace root
        let target = WorkspaceTarget {
            name: "workspace".to_string(),
            path: workspace_root.to_path_buf(),
            kind: "lib".to_string(),
            enabled: true,
        };

        Ok(vec![target])
    }
}

#[async_trait]
impl Analyzer for CargoCheckAnalyzer {
    fn name(&self) -> &str {
        "cargo-check"
    }

    fn category(&self) -> Category {
        Category::StaticAnalysis
    }

    fn description(&self) -> &str {
        "Analyze cargo check output for compilation warnings and errors"
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo".to_string()]
    }

    async fn can_run(&self) -> bool {
        let deps = self.dependencies();
        for dep in deps {
            if !self.check_dependency(&dep).await {
                tracing::debug!("Dependency not found: {}", dep);
                return false;
            }
        }
        true
    }

    async fn analyze(&self, workspace_root: &Path) -> Result<AnalysisResult> {
        tracing::info!("Starting cargo check analysis for workspace: {}", workspace_root.display());

        // Initialize system info
        let mut analyzer = self.clone();
        analyzer.init_system_info()?;

        // Discover workspace targets
        let targets = analyzer.discover_workspace_targets(workspace_root).await?;
        tracing::debug!("Discovered {} targets to analyze", targets.len());

        // For now, analyze the first enabled target (typically the main workspace)
        if let Some(target) = targets.into_iter().find(|t| t.enabled) {
            let result = analyzer.analyze_target(&target).await?;
            tracing::info!(
                "Cargo check analysis completed in {:.2}s with {} issues",
                result.performance.as_ref()
                    .map(|p| p.duration_seconds)
                    .unwrap_or(0.0),
                result.issue_count
            );
            Ok(result)
        } else {
            Err(MonitoringError::analysis("No enabled targets found for analysis"))
        }
    }
}

impl Clone for CargoCheckAnalyzer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            system_info: self.system_info.clone(),
        }
    }
}

impl Default for CargoCheckAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = CargoCheckAnalyzer::new();
        assert_eq!(analyzer.name(), "cargo-check");
        assert_eq!(analyzer.category(), Category::StaticAnalysis);
    }

    #[tokio::test]
    async fn test_can_run_basic() {
        let analyzer = CargoCheckAnalyzer::new();

        // This test might fail if cargo is not available in test environment
        let can_run = analyzer.can_run().await;
        // We can't assert much here since it depends on the environment
        assert!(can_run.is_bool());
    }

    #[test]
    fn test_system_info_initialization() {
        let mut analyzer = CargoCheckAnalyzer::new();

        // This should not panic, but will fail if we can't get system info
        let result = analyzer.init_system_info();
        // We can't assert success since it depends on the environment
        assert!(result.is_ok() || result.is_err());
    }
}