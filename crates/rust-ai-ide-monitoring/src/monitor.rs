//! Main monitoring orchestrator

use crate::{
    analyzers::AnalyzerRegistry,
    config::Config,
    errors::{MonitoringError, Result},
    metrics::{MetricsAggregator, QualityScore},
    types::AnalysisReport,
};
use chrono::Utc;
use std::path::Path;

/// Main monitoring orchestrator
pub struct Monitor {
    /// Configuration for monitoring
    config: Config,

    /// Analyzer registry
    analyzers: AnalyzerRegistry,

    /// Metrics aggregator
    metrics: MetricsAggregator,
}

impl Monitor {
    /// Create a new monitor instance
    pub fn new(config: Config) -> Self {
        Self {
            analyzers: AnalyzerRegistry::default(),
            config,
            metrics: MetricsAggregator::new(),
        }
    }

    /// Create a new monitor with default configuration
    pub async fn default() -> Result<Self> {
        let config = Config::default();
        Ok(Self::new(config))
    }

    /// Run full monitoring analysis
    pub async fn run_analysis(&mut self) -> Result<AnalysisReport> {
        tracing::info!("Starting comprehensive monitoring analysis");

        let start_time = std::time::Instant::now();
        let system_info = self.collect_system_info().await?;

        // Run all enabled analyzers
        let mut analysis_results = Vec::new();

        for analyzer_name in self.analyzers.list() {
            if self.config.is_analyzer_enabled(&analyzer_name) {
                match self.run_analyzer(&analyzer_name).await {
                    Ok(result) => {
                        // Add findings to metrics aggregator
                        self.metrics.add_findings(&result.findings);
                        analysis_results.push(result);
                    }
                    Err(e) => {
                        tracing::error!("Analyzer '{}' failed: {}", analyzer_name, e);
                        // Continue with other analyzers
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        self.metrics.set_timing(duration.as_secs_f64(), None);

        let metrics = std::mem::take(&mut self.metrics).take_metrics();
        self.metrics = MetricsAggregator::new();
        let quality_score = QualityScore::from_metrics(&metrics);

        Ok(AnalysisReport {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            quality_score: quality_score.overall,
            metrics,
            results: analysis_results,
            system_info,
            config_summary: self.config.get_config_summary(),
            duration_seconds: duration.as_secs_f64(),
        })
    }

    /// Run a specific analyzer
    async fn run_analyzer(&self, name: &str) -> Result<crate::types::AnalysisResult> {
        if let Some(analyzer) = self.analyzers.get(name) {
            analyzer.analyze(&self.config.workspace_root).await
        } else {
            Err(MonitoringError::analysis(format!(
                "Analyzer '{}' not found",
                name
            )))
        }
    }

    /// Collect system information
    async fn collect_system_info(&self) -> Result<crate::types::SystemInfo> {
        use sysinfo::System;

        let system = System::new_all();

        Ok(crate::types::SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: self.get_rust_version()?,
            cargo_version: self.get_cargo_version()?,
            cpu_count: num_cpus::get(),
            total_memory_mb: (system.total_memory() / 1024 / 1024) as usize,
            available_memory_mb: (system.available_memory() / 1024 / 1024) as usize,
        })
    }

    /// Get Rust version
    fn get_rust_version(&self) -> Result<String> {
        std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(|e| MonitoringError::command_execution("rustc --version".to_string(), e))
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).trim().to_string();
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    Ok("unknown".to_string())
                }
            })
    }

    /// Get Cargo version
    fn get_cargo_version(&self) -> Result<String> {
        std::process::Command::new("cargo")
            .arg("--version")
            .output()
            .map_err(|e| MonitoringError::command_execution("cargo --version".to_string(), e))
            .and_then(|output| {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    Ok("unknown".to_string())
                }
            })
    }

    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get analyzer registry
    pub fn analyzers(&self) -> &AnalyzerRegistry {
        &self.analyzers
    }
}
