//! Consolidated performance testing utilities
//!
//! This module consolidates performance testing patterns and utilities
//! from test-performance-analyzer and test-performance-project directories.

use crate::utils::test_utils::utils;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::process::Command as AsyncCommand;

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerformanceTestConfig {
    pub test_name: String,
    pub iterations: u32,
    pub enable_profiling: bool,
    pub output_file: Option<PathBuf>,
    pub profile: String, // "debug" or "release"
    pub enable_incremental: bool,
}

/// Performance test result
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub iteration_result: u64,
    pub total_duration: Duration,
    pub ops_per_second: f64,
    pub avg_iteration_time: f64,
    pub memory_usage: Option<u64>,
    pub profile: String,
}

/// Performance analyzer that consolidates testing patterns
pub struct PerformanceAnalyzer {
    project_path: PathBuf,
    config: PerformanceTestConfig,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new(project_path: PathBuf, config: PerformanceTestConfig) -> Self {
        Self { project_path, config }
    }

    /// Run a complete performance test suite
    pub async fn run_performance_test_suite(&self) -> anyhow::Result<Vec<PerformanceTestResult>> {
        println!("=== Running Performance Test Suite: {} ===", self.config.test_name);

        let mut results = Vec::new();

        // Run build performance analysis first
        let build_metrics = self.analyze_build_performance(self.config.enable_incremental).await?;
        println!("Build analysis complete: {:?}", build_metrics);

        // Run CPU-bound workload test
        let (sync_result, sync_duration) = self.run_sync_performance_workload().await?;
        let sync_result = PerformanceTestResult {
            test_name: format!("{}_sync", self.config.test_name),
            iteration_result: sync_result,
            total_duration: sync_duration,
            ops_per_second: self.calculate_ops_per_second(sync_result as u64, sync_duration),
            avg_iteration_time: sync_duration.as_secs_f64() / self.config.iterations as f64,
            memory_usage: None,
            profile: self.config.profile.clone(),
        };
        results.push(sync_result.clone());
        self.print_performance_result(&sync_result);

        // Run I/O-bound workload test
        let (async_result, async_duration) = self.run_async_performance_workload().await?;
        let async_result = PerformanceTestResult {
            test_name: format!("{}_async", self.config.test_name),
            iteration_result: async_result,
            total_duration: async_duration,
            ops_per_second: self.calculate_ops_per_second(async_result as u64, async_duration),
            avg_iteration_time: async_duration.as_secs_f64() / self.config.iterations as f64,
            memory_usage: None,
            profile: self.config.profile.clone(),
        };
        results.push(async_result.clone());
        self.print_performance_result(&async_result);

        // Generate comparison report
        self.generate_comparison_report(&results);

        // Export results if configured
        if let Some(output_file) = &self.config.output_file {
            self.export_results(&results, output_file).await?;
        }

        Ok(results)
    }

    /// Analyze build performance metrics
    pub async fn analyze_build_performance(&self, enable_incremental: bool) -> anyhow::Result<BuildPerformanceMetrics> {
        // Clean the project first
        utils::clean_and_prepare_project(&self.project_path)?;

        let start = Instant::now();

        // Run build with appropriate profile
        let build_args = match self.config.profile.as_str() {
            "release" => vec!["build", "--release"],
            _ => vec!["build"],
        };

        let result = utils::run_command_async("cargo", &build_args, &self.project_path).await?;

        if !result.success {
            anyhow::bail!("Build failed: {}", result.stderr);
        }

        let build_time = start.elapsed();
        let build_time_ms = build_time.as_millis() as f64;

        // If running with incremental, run a second build to see incremental performance
        let incremental_build_time = if enable_incremental {
            let start = Instant::now();
            let result = utils::run_command_async("cargo", &build_args, &self.project_path).await?;
            if !result.success {
                anyhow::bail!("Incremental build failed: {}", result.stderr);
            }
            Some(start.elapsed())
        } else {
            None
        };

        Ok(BuildPerformanceMetrics {
            total_build_time_ms: build_time_ms,
            incremental_build_time,
            profile: self.config.profile.clone(),
        })
    }

    /// Run synchronous performance workload
    pub async fn run_sync_performance_workload(&self) -> anyhow::Result<(u64, Duration)> {
        let start = Instant::now();
        let result = run_sync_workload(self.config.iterations);
        let duration = start.elapsed();
        Ok((result, duration))
    }

    /// Run asynchronous performance workload
    pub async fn run_async_performance_workload(&self) -> anyhow::Result<(u64, Duration)> {
        let start = Instant::now();
        let result = run_async_workload(self.config.iterations).await;
        let duration = start.elapsed();
        Ok((result, duration))
    }

    /// Calculate operations per second
    fn calculate_ops_per_second(&self, operations: u64, duration: Duration) -> f64 {
        operations as f64 / duration.as_secs_f64()
    }

    /// Print performance result
    fn print_performance_result(&self, result: &PerformanceTestResult) {
        println!("\n=== {} Performance Results ===", result.test_name);
        println!("Result: {}", result.iteration_result);
        println!("Duration: {:.2?}", result.total_duration);
        println!("Ops/second: {:.2}", result.ops_per_second);
        println!("Avg iteration time: {:.2}ms", result.avg_iteration_time * 1000.0);
        if let Some(mem) = result.memory_usage {
            println!("Memory usage: {}KB", mem);
        }
    }

    /// Generate comparison report between sync and async performance
    fn generate_comparison_report(&self, results: &[PerformanceTestResult]) {
        if results.len() < 2 {
            return;
        }

        println!("\n=== Performance Comparison Report ===");
        let mut sync_result = None;
        let mut async_result = None;

        for result in results {
            if result.test_name.contains("_sync") {
                sync_result = Some(result);
            } else if result.test_name.contains("_async") {
                async_result = Some(result);
            }
        }

        if let (Some(sync), Some(async_)) = (sync_result, async_result) {
            let sync_ops = sync.ops_per_second;
            let async_ops = async_.ops_per_second;
            let improvement = if sync_ops > 0.0 {
                (async_ops - sync_ops) / sync_ops * 100.0
            } else {
                0.0
            };

            println!("Sync workload: {:.2} ops/second", sync_ops);
            println!("Async workload: {:.2} ops/second", async_ops);
            println!("Performance difference: {:.1}%", improvement);

            println!("\nRecommendations:");
            if async_ops > sync_ops * 1.5 {
                println!("- Async workload shows significant performance advantage");
                println!("- Consider async patterns for I/O-intensive operations");
            } else {
                println!("- Sync and async performance are comparable");
                println!("- Performance depends on specific use case");
            }
        }
    }

    /// Export results to JSON file
    pub async fn export_results(&self, results: &[PerformanceTestResult], output_file: &Path) -> anyhow::Result<()> {
        use std::fs;
        use serde_json;

        let export_data = PerfExportData {
            test_name: self.config.test_name.clone(),
            profile: self.config.profile.clone(),
            iterations: self.config.iterations,
            timestamp: chrono::Utc::now().to_rfc3339(),
            results: results.to_vec(),
            config: self.config.clone(),
        };

        let json = serde_json::to_string_pretty(&export_data)?;
        fs::write(output_file, json)?;

        println!("Exported performance results to: {}", output_file.display());
        Ok(())
    }
}

/// Synchronous performance workload (CPU-bound)
fn run_sync_workload(iterations: u32) -> u64 {
    let mut result = 0u64;

    // Simulate some CPU-bound work (same pattern across all test projects)
    for i in 0..iterations {
        // Simple hash-like operation
        let x = i as u64 * 2654435761 % (1 << 31);
        result = result.wrapping_add(x);

        // Simulate memory allocation
        let mut vec = Vec::with_capacity(1000);
        for j in 0..1000 {
            vec.push(j as u64);
        }

        // Use the vector to prevent optimization
        result = result.wrapping_add(vec[vec.len() - 1]);
    }

    result
}

/// Asynchronous performance workload (I/O-bound)
async fn run_async_workload(iterations: u32) -> u64 {
    let mut result = 0u64;

    for i in 0..iterations {
        // Simulate async I/O
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Some computation
        let x = i as u64 * 11400714819323198549u64;
        result = result.wrapping_add(x);
    }

    result
}

/// Build performance metrics
#[derive(Debug, Clone)]
pub struct BuildPerformanceMetrics {
    pub total_build_time_ms: f64,
    pub incremental_build_time: Option<Duration>,
    pub profile: String,
}

/// Export data structure for performance results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PerfExportData {
    test_name: String,
    profile: String,
    iterations: u32,
    timestamp: String,
    results: Vec<PerformanceTestResult>,
    config: PerformanceTestConfig,
}

/// Performance report generator
pub struct PerformanceReportGenerator;

impl PerformanceReportGenerator {
    /// Generate a text-based performance report
    pub fn generate_text_report(results: &[PerformanceTestResult]) -> String {
        let mut report = format!(
            "=== Performance Test Report ===\nGenerated: {}\n\n",
            chrono::Utc::now().to_rfc3339()
        );

        for result in results {
            report.push_str(&format!(
                "Test: {}\nProfile: {}\nResult: {}\nDuration: {:.2?}\nOps/second: {:.2}\n\n",
                result.test_name, result.profile, result.iteration_result,
                result.total_duration, result.ops_per_second
            ));
        }

        report
    }

    /// Generate a simple markdown report
    pub fn generate_markdown_report(results: &[PerformanceTestResult]) -> String {
        let mut report = "# Performance Test Report\n\n".to_string();
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().to_rfc3339()));
        report.push_str("| Test | Profile | Duration | Ops/second |\n");
        report.push_str("|------|---------|----------|------------|\n");

        for result in results {
            report.push_str(&format!(
                "| {} | {} | {:.2?} | {:.2} |\n",
                result.test_name, result.profile, result.total_duration, result.ops_per_second
            ));
        }

        report
    }
}

/// Utility for creating temporary performance test projects
pub struct TempPerformanceProject;

impl TempPerformanceProject {
    /// Create a minimal project for performance testing
    pub fn create_minimal_project(name: &str, temp_dir: &Path) -> anyhow::Result<PathBuf> {
        use std::fs;

        let project_path = temp_dir.join(name);
        fs::create_dir_all(project_path.join("src"))?;

        // Create Cargo.toml
        let cargo_toml = format!(r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1", features = ["full"] }}
"#, name);

        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        // Create src/lib.rs with performance testing utilities
        let lib_rs = r#"
//! Performance testing library

/// Simulate CPU-bound work
pub fn run_sync_workload(iterations: u32) -> u64 {
    let mut result = 0u64;

    for i in 0..iterations {
        let x = i as u64 * 2654435761 % (1 << 31);
        result = result.wrapping_add(x);

        let mut vec = Vec::with_capacity(1000);
        for j in 0..1000 {
            vec.push(j as u64);
        }

        result = result.wrapping_add(vec[vec.len() - 1]);
    }

    result
}

/// Simulate I/O-bound work
pub async fn run_async_workload(iterations: u32) -> u64 {
    let mut result = 0u64;

    for i in 0..iterations {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let x = i as u64 * 11400714819323198549u64;
        result = result.wrapping_add(x);
    }

    result
}
"#;

        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        Ok(project_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::utils;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_performance_analyzer_creation() {
        let temp_dir = std::env::temp_dir();
        let project_path = temp_dir.join("test_performance_project");

        // If the project doesn't exist, create a minimal one
        if !project_path.exists() {
            TempPerformanceProject::create_minimal_project("test_performance_project", &temp_dir).unwrap();
        }

        let config = PerformanceTestConfig {
            test_name: "test_suite".to_string(),
            iterations: 100,
            enable_profiling: false,
            output_file: None,
            profile: "debug".to_string(),
            enable_incremental: false,
        };

        let analyzer = PerformanceAnalyzer::new(project_path, config);
        assert_eq!(analyzer.config.test_name, "test_suite");
        assert_eq!(analyzer.config.iterations, 100);
    }

    #[tokio::test]
    async fn test_sync_workload() {
        let result = run_sync_workload(100);
        assert!(result > 0);
    }

    #[tokio::test]
    async fn test_async_workload() {
        let result = run_async_workload(10).await;
        assert!(result > 0);
    }

    #[test]
    fn test_performance_report_generation() {
        let results = vec![
            PerformanceTestResult {
                test_name: "test1".to_string(),
                iteration_result: 1000,
                total_duration: Duration::from_millis(100),
                ops_per_second: 10.0,
                avg_iteration_time: 0.1,
                memory_usage: None,
                profile: "debug".to_string(),
            }
        ];

        let text_report = PerformanceReportGenerator::generate_text_report(&results);
        assert!(text_report.contains("Performance Test Report"));

        let markdown_report = PerformanceReportGenerator::generate_markdown_report(&results);
        assert!(markdown_report.contains("# Performance Test Report"));
    }
}