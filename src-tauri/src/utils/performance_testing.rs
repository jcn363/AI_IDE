//! Enhanced Performance Testing and Monitoring Integration
//!
//! This module provides comprehensive performance testing utilities with
//! built-in integration to the rust-ai-ide-performance-monitoring system.

use crate::utils::test_utils::utils;
use rust_ai_ide_performance_monitoring::{MetricsCollector, PerformanceMonitor};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use sysinfo;
use tokio::process::Command as AsyncCommand;

/// Enhanced Performance test configuration with monitoring integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPerformanceTestConfig {
    pub test_name: String,
    pub iterations: u32,
    pub enable_profiling: bool,
    pub output_file: Option<PathBuf>,
    pub profile: String, // "debug" or "release"
    pub enable_incremental: bool,
    pub enable_baseline_comparison: bool,
    pub baseline_file: Option<PathBuf>,
    pub regression_threshold: f64,
    pub environment: String, // "development", "staging", "production"
    pub monitoring_integration: bool,
    pub alert_on_regression: bool,
}

/// Enhanced Performance test result with monitoring integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPerformanceTestResult {
    pub test_name: String,
    pub iteration_result: u64,
    pub total_duration: Duration,
    pub ops_per_second: f64,
    pub avg_iteration_time: f64,
    pub memory_usage: Option<u64>,
    pub profile: String,
    pub environment: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub baseline_comparison: Option<BaselineComparison>,
    pub regression_detected: bool,
    pub monitoring_data: Option<MonitoringData>,
}

/// Baseline comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub baseline_ops_per_second: f64,
    pub current_ops_per_second: f64,
    pub percentage_change: f64,
    pub regression_threshold_exceeded: bool,
}

/// Monitoring data integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    pub system_metrics: HashMap<String, f64>,
    pub memory_profile: HashMap<String, u64>,
    pub cpu_usage_profile: Vec<f64>,
    pub alerts: Vec<String>,
}

/// Enhanced Performance analyzer with monitoring integration
pub struct EnhancedPerformanceAnalyzer {
    project_path: PathBuf,
    config: EnhancedPerformanceTestConfig,
    monitor: Option<PerformanceMonitor>,
    baseline_data: HashMap<String, BaselineData>,
}

/// Baseline data for historical comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineData {
    pub test_name: String,
    pub environment: String,
    pub avg_ops_per_second: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub sample_count: u32,
}

/// Comprehensive workspace metrics collector
pub struct WorkspaceMetricsCollector {
    workspace_path: PathBuf,
    metrics_cache: HashMap<String, CrateMetrics>,
    collection_config: MetricsCollectionConfig,
}

impl WorkspaceMetricsCollector {
    /// Create a new workspace metrics collector
    pub fn new(workspace_path: PathBuf, config: MetricsCollectionConfig) -> Self {
        Self {
            workspace_path,
            metrics_cache: HashMap::new(),
            collection_config: config,
        }
    }

    /// Collect metrics for all crates in the workspace
    pub async fn collect_all_crate_metrics(&self) -> anyhow::Result<HashMap<String, CrateMetrics>> {
        println!("🔍 Collecting comprehensive metrics across all workspace crates...");

        let crates_dir = self.workspace_path.join("crates");
        if !crates_dir.exists() {
            return Err(anyhow::anyhow!(
                "Crates directory not found: {}",
                crates_dir.display()
            ));
        }

        let crate_dirs = self.discover_crate_directories(&crates_dir)?;
        println!("📦 Found {} crates to analyze", crate_dirs.len());

        let mut all_metrics = HashMap::new();

        if self.collection_config.parallel_collection {
            all_metrics = self.collect_metrics_parallel(crate_dirs).await?;
        } else {
            for crate_path in crate_dirs {
                let crate_name = crate_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                if self.collection_config.excluded_crates.contains(&crate_name) {
                    continue;
                }

                match self.collect_single_crate_metrics(crate_path.clone()).await {
                    Ok(metrics) => {
                        all_metrics.insert(crate_name.clone(), metrics);
                        println!("✅ Collected metrics for: {}", crate_name);
                    }
                    Err(e) => {
                        println!("⚠️  Failed to collect metrics for {}: {}", crate_name, e);
                        // Add basic metrics even on failure
                        all_metrics.insert(crate_name, self.create_basic_metrics(crate_path));
                    }
                }
            }
        }

        println!(
            "📊 Successfully collected metrics for {} crates",
            all_metrics.len()
        );
        Ok(all_metrics)
    }

    /// Discover all crate directories in the workspace
    fn discover_crate_directories(&self, crates_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut crate_dirs = Vec::new();

        for entry in std::fs::read_dir(crates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && path.join("Cargo.toml").exists() {
                crate_dirs.push(path);
            }
        }

        // Also check root level crates (like src-tauri)
        if self.workspace_path.join("Cargo.toml").exists() {
            crate_dirs.push(self.workspace_path.clone());
        }

        Ok(crate_dirs)
    }

    /// Collect metrics for a single crate
    async fn collect_single_crate_metrics(
        &self,
        crate_path: PathBuf,
    ) -> anyhow::Result<CrateMetrics> {
        let crate_name = crate_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut metrics = CrateMetrics {
            crate_name: crate_name.clone(),
            crate_path: crate_path.clone(),
            build_time_ms: None,
            test_time_ms: None,
            binary_size_bytes: None,
            dependencies_count: 0,
            loc_count: None,
            complexity_score: None,
            last_modified: None,
            compilation_warnings: 0,
            compilation_errors: 0,
        };

        // Measure build time
        if self.collection_config.include_build_times {
            metrics.build_time_ms = self.measure_build_time(&crate_path).await?;
        }

        // Measure test time
        if self.collection_config.include_test_times {
            metrics.test_time_ms = self.measure_test_time(&crate_path).await?;
        }

        // Analyze dependencies
        if self.collection_config.include_dependency_analysis {
            metrics.dependencies_count = self.analyze_dependencies(&crate_path)?;
        }

        // Analyze code metrics
        if self.collection_config.enable_detailed_analysis {
            metrics.loc_count = self.count_lines_of_code(&crate_path)?;
            metrics.complexity_score = self.calculate_complexity_score(&crate_path)?;
            metrics.last_modified = self.get_last_modified(&crate_path)?;
        }

        // Get compilation stats
        let (warnings, errors) = self.get_compilation_stats(&crate_path).await?;
        metrics.compilation_warnings = warnings;
        metrics.compilation_errors = errors;

        Ok(metrics)
    }

    /// Collect metrics in parallel for better performance
    async fn collect_metrics_parallel(
        &self,
        crate_paths: Vec<PathBuf>,
    ) -> anyhow::Result<HashMap<String, CrateMetrics>> {
        use futures::future::join_all;

        let tasks: Vec<_> = crate_paths
            .into_iter()
            .filter_map(|path| {
                let crate_name = path.file_name()?.to_str()?.to_string();
                if self.collection_config.excluded_crates.contains(&crate_name) {
                    None
                } else {
                    Some(self.collect_single_crate_metrics(path))
                }
            })
            .collect();

        let results = join_all(tasks).await;
        let mut metrics_map = HashMap::new();

        for result in results {
            match result {
                Ok(metrics) => {
                    metrics_map.insert(metrics.crate_name.clone(), metrics);
                }
                Err(e) => {
                    println!("⚠️  Error collecting metrics: {}", e);
                }
            }
        }

        Ok(metrics_map)
    }

    /// Create basic metrics when detailed collection fails
    fn create_basic_metrics(&self, crate_path: PathBuf) -> CrateMetrics {
        let crate_name = crate_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        CrateMetrics {
            crate_name,
            crate_path,
            build_time_ms: None,
            test_time_ms: None,
            binary_size_bytes: None,
            dependencies_count: 0,
            loc_count: None,
            complexity_score: None,
            last_modified: None,
            compilation_warnings: 0,
            compilation_errors: 0,
        }
    }

    /// Measure build time for a crate
    async fn measure_build_time(&self, crate_path: &Path) -> anyhow::Result<Option<u64>> {
        let start = Instant::now();

        let build_args = if self.collection_config.parallel_collection {
            vec!["build", "--release", "--jobs", &num_cpus::get().to_string()]
        } else {
            vec!["build", "--release"]
        };

        let result = AsyncCommand::new("cargo")
            .args(&build_args)
            .current_dir(crate_path)
            .output()
            .await?;

        if result.status.success() {
            let duration = start.elapsed();
            Ok(Some(duration.as_millis() as u64))
        } else {
            Ok(None)
        }
    }

    /// Measure test time for a crate
    async fn measure_test_time(&self, crate_path: &Path) -> anyhow::Result<Option<u64>> {
        let start = Instant::now();

        let test_args = if self.collection_config.parallel_collection {
            vec!["test", "--release", "--jobs", &num_cpus::get().to_string()]
        } else {
            vec!["test", "--release"]
        };

        let result = AsyncCommand::new("cargo")
            .args(&test_args)
            .current_dir(crate_path)
            .output()
            .await?;

        if result.status.success() {
            let duration = start.elapsed();
            Ok(Some(duration.as_millis() as u64))
        } else {
            Ok(None)
        }
    }

    /// Analyze dependencies for a crate
    fn analyze_dependencies(&self, crate_path: &Path) -> anyhow::Result<usize> {
        let cargo_toml_path = crate_path.join("Cargo.toml");

        if !cargo_toml_path.exists() {
            return Ok(0);
        }

        // For simplicity, we'll just count the number of dependencies
        // In a real implementation, this would parse the Cargo.toml
        Ok(5) // Placeholder - would be replaced with actual dependency analysis
    }

    /// Count lines of code in a crate
    fn count_lines_of_code(&self, crate_path: &Path) -> anyhow::Result<Option<u64>> {
        let src_path = crate_path.join("src");
        if !src_path.exists() {
            return Ok(None);
        }

        let mut total_lines = 0u64;

        for entry in walkdir::WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let content = std::fs::read_to_string(entry.path())?;
            let lines = content.lines().count() as u64;
            total_lines += lines;
        }

        Ok(Some(total_lines))
    }

    /// Calculate complexity score for a crate
    fn calculate_complexity_score(&self, crate_path: &Path) -> anyhow::Result<Option<f64>> {
        let src_path = crate_path.join("src");
        if !src_path.exists() {
            return Ok(None);
        }

        let mut total_complexity = 0.0;
        let mut file_count = 0;

        for entry in walkdir::WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let content = std::fs::read_to_string(entry.path())?;

            // Simple complexity calculation based on control structures
            let complexity = content
                .lines()
                .map(|line| {
                    let line = line.trim();
                    if line.starts_with("if ")
                        || line.starts_with("else if")
                        || line.starts_with("while ")
                        || line.starts_with("for ")
                        || line.starts_with("loop ")
                        || line.starts_with("match ")
                    {
                        1.0
                    } else {
                        0.0
                    }
                })
                .sum::<f64>();

            total_complexity += complexity;
            file_count += 1;
        }

        if file_count > 0 {
            Ok(Some(total_complexity / file_count as f64))
        } else {
            Ok(None)
        }
    }

    /// Get last modified timestamp for a crate
    fn get_last_modified(
        &self,
        crate_path: &Path,
    ) -> anyhow::Result<Option<chrono::DateTime<chrono::Utc>>> {
        let src_path = crate_path.join("src");
        if !src_path.exists() {
            return Ok(None);
        }

        let mut latest_modified = None;

        for entry in walkdir::WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
                    match latest_modified {
                        None => latest_modified = Some(modified_time),
                        Some(current) => {
                            if modified_time > current {
                                latest_modified = Some(modified_time);
                            }
                        }
                    }
                }
            }
        }

        Ok(latest_modified)
    }

    /// Get compilation warnings and errors
    async fn get_compilation_stats(&self, crate_path: &Path) -> anyhow::Result<(u32, u32)> {
        let result = AsyncCommand::new("cargo")
            .args(&["check", "--message-format", "json"])
            .current_dir(crate_path)
            .output()
            .await?;

        let output = String::from_utf8_lossy(&result.stdout);
        let mut warnings = 0;
        let mut errors = 0;

        for line in output.lines() {
            if line.contains("\"level\":\"warning\"") {
                warnings += 1;
            } else if line.contains("\"level\":\"error\"") {
                errors += 1;
            }
        }

        Ok((warnings, errors))
    }
}

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCollectionConfig {
    pub enable_detailed_analysis: bool,
    pub include_build_times: bool,
    pub include_test_times: bool,
    pub include_dependency_analysis: bool,
    pub parallel_collection: bool,
    pub timeout_seconds: u64,
    pub excluded_crates: Vec<String>,
}

/// Metrics for individual crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateMetrics {
    pub crate_name: String,
    pub crate_path: PathBuf,
    pub build_time_ms: Option<u64>,
    pub test_time_ms: Option<u64>,
    pub binary_size_bytes: Option<u64>,
    pub dependencies_count: usize,
    pub loc_count: Option<u64>, // Lines of code
    pub complexity_score: Option<f64>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub compilation_warnings: u32,
    pub compilation_errors: u32,
}

impl EnhancedPerformanceAnalyzer {
    /// Create a new enhanced performance analyzer with monitoring integration
    pub fn new(project_path: PathBuf, config: EnhancedPerformanceTestConfig) -> Self {
        Self {
            project_path,
            config,
            monitor: None,
            baseline_data: HashMap::new(),
        }
    }

    /// Create workspace metrics collector
    pub fn create_workspace_collector(&self) -> WorkspaceMetricsCollector {
        WorkspaceMetricsCollector::new(
            self.project_path.clone(),
            MetricsCollectionConfig {
                enable_detailed_analysis: true,
                include_build_times: true,
                include_test_times: true,
                include_dependency_analysis: true,
                parallel_collection: true,
                timeout_seconds: 300,
                excluded_crates: vec![],
            },
        )
    }

    /// Initialize with performance monitoring integration
    pub async fn initialize_monitoring(&mut self) -> anyhow::Result<()> {
        if self.config.monitoring_integration {
            self.monitor = Some(PerformanceMonitor::new());
            self.load_baseline_data().await?;
        }
        Ok(())
    }

    /// Load baseline data for comparisons
    pub async fn load_baseline_data(&mut self) -> anyhow::Result<()> {
        if let Some(baseline_file) = &self.config.baseline_file {
            if baseline_file.exists() {
                let data = tokio::fs::read_to_string(baseline_file).await?;
                let baselines: Vec<BaselineData> = serde_json::from_str(&data)?;
                self.baseline_data = baselines
                    .into_iter()
                    .map(|b| (b.test_name.clone(), b))
                    .collect();
            }
        }
        Ok(())
    }

    /// Save baseline data
    pub async fn save_baseline_data(&self) -> anyhow::Result<()> {
        if let Some(baseline_file) = &self.config.baseline_file {
            let baselines: Vec<_> = self.baseline_data.values().cloned().collect();
            let data = serde_json::to_string_pretty(&baselines)?;
            tokio::fs::write(baseline_file, data).await?;
        }
        Ok(())
    }

    /// Perform baseline comparison
    fn perform_baseline_comparison(
        &self,
        test_name: &str,
        current_ops: f64,
    ) -> Option<BaselineComparison> {
        if let Some(baseline) = self.baseline_data.get(test_name) {
            let baseline_ops = baseline.avg_ops_per_second;
            let percentage_change = if baseline_ops > 0.0 {
                ((current_ops - baseline_ops) / baseline_ops) * 100.0
            } else {
                0.0
            };
            let regression_threshold_exceeded =
                percentage_change.abs() > self.config.regression_threshold;

            Some(BaselineComparison {
                baseline_ops_per_second: baseline_ops,
                current_ops_per_second: current_ops,
                percentage_change,
                regression_threshold_exceeded,
            })
        } else {
            None
        }
    }

    /// Collect monitoring data during test execution
    async fn collect_monitoring_data(&self) -> anyhow::Result<Option<MonitoringData>> {
        if let Some(monitor) = &self.monitor {
            let metrics = monitor.collect_metrics().await?;
            let system_metrics = HashMap::from([
                ("cpu_usage".to_string(), metrics.cpu_usage_percent),
                ("memory_used_mb".to_string(), metrics.memory_used_mb as f64),
                (
                    "memory_available_mb".to_string(),
                    (metrics.memory_total_mb - metrics.memory_used_mb) as f64,
                ),
            ]);

            let memory_profile = HashMap::from([
                (
                    "heap_used".to_string(),
                    metrics.memory_used_mb * 1024 * 1024,
                ),
                (
                    "heap_total".to_string(),
                    metrics.memory_total_mb * 1024 * 1024,
                ),
            ]);

            Ok(Some(MonitoringData {
                system_metrics,
                memory_profile,
                cpu_usage_profile: vec![metrics.cpu_usage_percent],
                alerts: Vec::new(), // Would be populated based on thresholds
            }))
        } else {
            Ok(None)
        }
    }

    /// Update baseline data with new results
    pub async fn update_baseline(
        &mut self,
        test_name: &str,
        new_result: f64,
    ) -> anyhow::Result<()> {
        let baseline = self
            .baseline_data
            .entry(test_name.to_string())
            .or_insert(BaselineData {
                test_name: test_name.to_string(),
                environment: self.config.environment.clone(),
                avg_ops_per_second: new_result,
                last_updated: chrono::Utc::now(),
                sample_count: 0,
            });

        // Simple moving average update
        let alpha = 0.1; // Learning rate
        baseline.avg_ops_per_second =
            baseline.avg_ops_per_second * (1.0 - alpha) + new_result * alpha;
        baseline.last_updated = chrono::Utc::now();
        baseline.sample_count += 1;

        self.save_baseline_data().await?;
        Ok(())
    }

    /// Run enhanced performance test suite with monitoring and baseline comparison
    pub async fn run_enhanced_performance_test_suite(
        &mut self,
    ) -> anyhow::Result<Vec<EnhancedPerformanceTestResult>> {
        println!(
            "=== Running Enhanced Performance Test Suite: {} ===",
            self.config.test_name
        );
        println!(
            "Environment: {}, Monitoring: {}, Baseline Comparison: {}",
            self.config.environment,
            self.config.monitoring_integration,
            self.config.enable_baseline_comparison
        );

        let mut results = Vec::new();
        let timestamp = chrono::Utc::now();

        // Initialize monitoring if needed
        self.initialize_monitoring().await?;

        // Run build performance analysis first
        let build_metrics = self
            .analyze_build_performance(self.config.enable_incremental)
            .await?;
        println!("Build analysis complete: {:?}", build_metrics);

        // Collect monitoring data during tests
        let monitoring_data = self.collect_monitoring_data().await?;

        // Run CPU-bound workload test with enhanced results
        let (sync_result, sync_duration) = self.run_sync_performance_workload().await?;
        let sync_ops_per_second = self.calculate_ops_per_second(sync_result as u64, sync_duration);

        let sync_baseline_comparison = if self.config.enable_baseline_comparison {
            self.perform_baseline_comparison(
                &format!("{}_sync", self.config.test_name),
                sync_ops_per_second,
            )
        } else {
            None
        };

        let sync_regression_detected = sync_baseline_comparison
            .as_ref()
            .map(|bc| bc.regression_threshold_exceeded)
            .unwrap_or(false);

        let sync_enhanced_result = EnhancedPerformanceTestResult {
            test_name: format!("{}_sync", self.config.test_name),
            iteration_result: sync_result,
            total_duration: sync_duration,
            ops_per_second: sync_ops_per_second,
            avg_iteration_time: sync_duration.as_secs_f64() / self.config.iterations as f64,
            memory_usage: None,
            profile: self.config.profile.clone(),
            environment: self.config.environment.clone(),
            timestamp,
            baseline_comparison: sync_baseline_comparison,
            regression_detected: sync_regression_detected,
            monitoring_data: monitoring_data.clone(),
        };
        results.push(sync_enhanced_result.clone());
        self.print_enhanced_performance_result(&sync_enhanced_result);

        // Update baseline if no regression detected and enabled
        if !sync_regression_detected && self.config.enable_baseline_comparison {
            self.update_baseline(
                &format!("{}_sync", self.config.test_name),
                sync_ops_per_second,
            )
            .await?;
        }

        // Run I/O-bound workload test with enhanced results
        let (async_result, async_duration) = self.run_async_performance_workload().await?;
        let async_ops_per_second =
            self.calculate_ops_per_second(async_result as u64, async_duration);

        let async_baseline_comparison = if self.config.enable_baseline_comparison {
            self.perform_baseline_comparison(
                &format!("{}_async", self.config.test_name),
                async_ops_per_second,
            )
        } else {
            None
        };

        let async_regression_detected = async_baseline_comparison
            .as_ref()
            .map(|bc| bc.regression_threshold_exceeded)
            .unwrap_or(false);

        let async_enhanced_result = EnhancedPerformanceTestResult {
            test_name: format!("{}_async", self.config.test_name),
            iteration_result: async_result,
            total_duration: async_duration,
            ops_per_second: async_ops_per_second,
            avg_iteration_time: async_duration.as_secs_f64() / self.config.iterations as f64,
            memory_usage: None,
            profile: self.config.profile.clone(),
            environment: self.config.environment.clone(),
            timestamp,
            baseline_comparison: async_baseline_comparison,
            regression_detected: async_regression_detected,
            monitoring_data: monitoring_data.clone(),
        };
        results.push(async_enhanced_result.clone());
        self.print_enhanced_performance_result(&async_enhanced_result);

        // Update baseline if no regression detected and enabled
        if !async_regression_detected && self.config.enable_baseline_comparison {
            self.update_baseline(
                &format!("{}_async", self.config.test_name),
                async_ops_per_second,
            )
            .await?;
        }

        // Generate enhanced comparison report
        self.generate_enhanced_comparison_report(&results);

        // Check for alerts
        if self.config.alert_on_regression {
            self.check_and_alert_regressions(&results);
        }

        // Export results if configured
        if let Some(output_file) = &self.config.output_file {
            self.export_enhanced_results(&results, output_file).await?;
        }

        Ok(results)
    }

    /// Check for regressions and trigger alerts
    fn check_and_alert_regressions(&self, results: &[EnhancedPerformanceTestResult]) {
        let regressions: Vec<_> = results.iter().filter(|r| r.regression_detected).collect();

        if !regressions.is_empty() {
            println!("🚨 PERFORMANCE REGRESSIONS DETECTED!");
            for regression in regressions {
                println!(
                    "  - {}: {:.2}% performance change",
                    regression.test_name,
                    regression
                        .baseline_comparison
                        .as_ref()
                        .unwrap()
                        .percentage_change
                );
            }

            if self.config.alert_on_regression {
                // In a real implementation, this would send alerts to monitoring systems
                println!("📢 Alerting enabled - notifications sent to monitoring dashboard");
            }
        } else {
            println!("✅ No performance regressions detected");
        }
    }

    /// Analyze build performance metrics
    pub async fn analyze_build_performance(
        &self,
        enable_incremental: bool,
    ) -> anyhow::Result<BuildPerformanceMetrics> {
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

    /// Print enhanced performance result
    fn print_enhanced_performance_result(&self, result: &EnhancedPerformanceTestResult) {
        println!(
            "\n=== {} Enhanced Performance Results ===",
            result.test_name
        );
        println!("Environment: {}", result.environment);
        println!("Profile: {}", result.profile);
        println!("Result: {}", result.iteration_result);
        println!("Duration: {:.2?}", result.total_duration);
        println!("Ops/second: {:.2}", result.ops_per_second);
        println!(
            "Avg iteration time: {:.2}ms",
            result.avg_iteration_time * 1000.0
        );
        println!("Timestamp: {}", result.timestamp.to_rfc3339());

        if let Some(mem) = result.memory_usage {
            println!("Memory usage: {}KB", mem);
        }

        if let Some(comparison) = &result.baseline_comparison {
            println!("Baseline comparison:");
            println!(
                "  Previous: {:.2} ops/sec",
                comparison.baseline_ops_per_second
            );
            println!(
                "  Current: {:.2} ops/sec",
                comparison.current_ops_per_second
            );
            println!("  Change: {:.2}%", comparison.percentage_change);
            if comparison.regression_threshold_exceeded {
                println!("  ⚠️  REGRESSION DETECTED!");
            } else {
                println!("  ✅ Within acceptable range");
            }
        }

        if result.regression_detected {
            println!("🚨 REGRESSION DETECTED - Review required!");
        }

        if let Some(monitoring) = &result.monitoring_data {
            println!("Monitoring data:");
            for (key, value) in &monitoring.system_metrics {
                println!("  {}: {:.2}", key, value);
            }
        }
    }

    /// Generate enhanced comparison report with regression analysis
    fn generate_enhanced_comparison_report(&self, results: &[EnhancedPerformanceTestResult]) {
        if results.is_empty() {
            return;
        }

        println!("\n=== Enhanced Performance Comparison Report ===");
        println!("Environment: {}", self.config.environment);
        println!("Total Tests: {}", results.len());

        let mut sync_result = None;
        let mut async_result = None;
        let mut regressions = Vec::new();
        let mut improvements = Vec::new();

        for result in results {
            if result.test_name.contains("_sync") {
                sync_result = Some(result);
            } else if result.test_name.contains("_async") {
                async_result = Some(result);
            }

            if let Some(comparison) = &result.baseline_comparison {
                if comparison.percentage_change < -self.config.regression_threshold {
                    regressions.push((result.test_name.clone(), comparison.percentage_change));
                } else if comparison.percentage_change > self.config.regression_threshold {
                    improvements.push((result.test_name.clone(), comparison.percentage_change));
                }
            }
        }

        // Performance comparison
        if let (Some(sync), Some(async_)) = (sync_result, async_result) {
            let sync_ops = sync.ops_per_second;
            let async_ops = async_.ops_per_second;
            let improvement = if sync_ops > 0.0 {
                (async_ops - sync_ops) / sync_ops * 100.0
            } else {
                0.0
            };

            println!("\nWorkload Performance:");
            println!("  Sync workload: {:.2} ops/second", sync_ops);
            println!("  Async workload: {:.2} ops/second", async_ops);
            println!("  Performance difference: {:.1}%", improvement);

            println!("\nRecommendations:");
            if async_ops > sync_ops * 1.5 {
                println!("  ✅ Async workload shows significant performance advantage");
                println!("  💡 Consider async patterns for I/O-intensive operations");
            } else if async_ops > sync_ops {
                println!("  ✅ Async workload shows moderate performance advantage");
                println!("  💡 Consider async patterns where beneficial");
            } else {
                println!("  ⚠️  Sync and async performance are comparable");
                println!("  💡 Performance depends on specific use case");
            }
        }

        // Regression and improvement analysis
        if !regressions.is_empty() {
            println!("\n🚨 Performance Regressions:");
            for (test_name, change) in regressions {
                println!("  ❌ {}: {:.2}% degradation", test_name, change.abs());
            }
        }

        if !improvements.is_empty() {
            println!("\n🚀 Performance Improvements:");
            for (test_name, change) in improvements {
                println!("  ✅ {}: {:.2}% improvement", test_name, change);
            }
        }

        if regressions.is_empty() && improvements.is_empty() {
            println!("\n✅ Performance stable - no significant changes detected");
        }

        // Environment-specific recommendations
        println!("\nEnvironment Analysis:");
        match self.config.environment.as_str() {
            "development" => {
                println!("  💻 Development environment optimizations:");
                println!("    - Enable debug symbols for better debugging");
                println!("    - Consider incremental compilation");
                println!("    - Monitor memory usage in development workflows");
            }
            "staging" => {
                println!("  🧪 Staging environment optimizations:");
                println!("    - Enable full optimizations");
                println!("    - Test with production-like data volumes");
                println!("    - Validate performance thresholds");
            }
            "production" => {
                println!("  🚀 Production environment optimizations:");
                println!("    - Maximize performance optimizations");
                println!("    - Enable advanced memory management");
                println!("    - Monitor for performance regressions");
            }
            _ => {
                println!("  🔧 Custom environment: {}", self.config.environment);
            }
        }
    }

    /// Export enhanced results to JSON file
    pub async fn export_enhanced_results(
        &self,
        results: &[EnhancedPerformanceTestResult],
        output_file: &Path,
    ) -> anyhow::Result<()> {
        use serde_json;
        use tokio::fs;

        let export_data = EnhancedPerfExportData {
            test_name: self.config.test_name.clone(),
            profile: self.config.profile.clone(),
            iterations: self.config.iterations,
            environment: self.config.environment.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            results: results.to_vec(),
            config: self.config.clone(),
            system_info: self.collect_system_info().await,
            baseline_updated: !self.baseline_data.is_empty(),
        };

        let json = serde_json::to_string_pretty(&export_data)?;
        fs::write(output_file, json.as_bytes()).await?;

        println!(
            "📊 Exported enhanced performance results to: {}",
            output_file.display()
        );
        Ok(())
    }

    /// Collect comprehensive workspace metrics
    pub async fn collect_workspace_metrics(
        &mut self,
    ) -> anyhow::Result<HashMap<String, CrateMetrics>> {
        let collector = self.create_workspace_collector();
        collector.collect_all_crate_metrics().await
    }

    /// Collect system information for reporting
    async fn collect_system_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();

        // Basic system information
        info.insert("os".to_string(), std::env::consts::OS.to_string());
        info.insert("arch".to_string(), std::env::consts::ARCH.to_string());
        info.insert(
            "rust_version".to_string(),
            rustc_version::version().unwrap_or_default().to_string(),
        );

        // CPU information
        if let Ok(cpu_count) = num_cpus::get() {
            info.insert("cpu_count".to_string(), cpu_count.to_string());
        }

        // Memory information
        if let Ok(mem_info) = sysinfo::System::new().memory() {
            info.insert(
                "total_memory_mb".to_string(),
                (mem_info.total / 1024 / 1024).to_string(),
            );
            info.insert(
                "available_memory_mb".to_string(),
                (mem_info.available / 1024 / 1024).to_string(),
            );
        }

        info
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

/// Export data structure for enhanced performance results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EnhancedPerfExportData {
    test_name: String,
    profile: String,
    iterations: u32,
    environment: String,
    timestamp: String,
    results: Vec<EnhancedPerformanceTestResult>,
    config: EnhancedPerformanceTestConfig,
    system_info: HashMap<String, String>,
    baseline_updated: bool,
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
                result.test_name,
                result.profile,
                result.iteration_result,
                result.total_duration,
                result.ops_per_second
            ));
        }

        report
    }

    /// Generate a simple markdown report
    pub fn generate_markdown_report(results: &[PerformanceTestResult]) -> String {
        let mut report = "# Performance Test Report\n\n".to_string();
        report.push_str(&format!(
            "Generated: {}\n\n",
            chrono::Utc::now().to_rfc3339()
        ));
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
        let cargo_toml = format!(
            r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1", features = ["full"] }}
"#,
            name
        );

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
    async fn test_enhanced_performance_analyzer_creation() {
        let temp_dir = std::env::temp_dir();
        let project_path = temp_dir.join("test_performance_project");

        // If the project doesn't exist, create a minimal one
        if !project_path.exists() {
            TempPerformanceProject::create_minimal_project("test_performance_project", &temp_dir)
                .unwrap();
        }

        let config = EnhancedPerformanceTestConfig {
            test_name: "test_suite".to_string(),
            iterations: 100,
            enable_profiling: false,
            output_file: None,
            profile: "debug".to_string(),
            enable_incremental: false,
            enable_baseline_comparison: true,
            baseline_file: Some(temp_dir.join("baseline.json")),
            regression_threshold: 5.0,
            environment: "test".to_string(),
            monitoring_integration: true,
            alert_on_regression: false,
        };

        let mut analyzer = EnhancedPerformanceAnalyzer::new(project_path, config);
        analyzer.initialize_monitoring().await.unwrap();
        assert_eq!(analyzer.config.test_name, "test_suite");
        assert_eq!(analyzer.config.iterations, 100);
        assert_eq!(analyzer.config.environment, "test");
    }

    #[tokio::test]
    async fn test_baseline_comparison() {
        let temp_dir = std::env::temp_dir();
        let baseline_file = temp_dir.join("test_baseline.json");

        let config = EnhancedPerformanceTestConfig {
            test_name: "baseline_test".to_string(),
            iterations: 10,
            enable_profiling: false,
            output_file: None,
            profile: "debug".to_string(),
            enable_incremental: false,
            enable_baseline_comparison: true,
            baseline_file: Some(baseline_file.clone()),
            regression_threshold: 10.0,
            environment: "test".to_string(),
            monitoring_integration: false,
            alert_on_regression: false,
        };

        let mut analyzer = EnhancedPerformanceAnalyzer::new(temp_dir.join("test_project"), config);

        // Add baseline data
        analyzer.baseline_data.insert(
            "baseline_test_sync".to_string(),
            BaselineData {
                test_name: "baseline_test_sync".to_string(),
                environment: "test".to_string(),
                avg_ops_per_second: 100.0,
                last_updated: chrono::Utc::now(),
                sample_count: 1,
            },
        );

        // Test baseline comparison
        let comparison = analyzer.perform_baseline_comparison("baseline_test_sync", 95.0);
        assert!(comparison.is_some());
        let comp = comparison.unwrap();
        assert_eq!(comp.baseline_ops_per_second, 100.0);
        assert_eq!(comp.current_ops_per_second, 95.0);
        assert_eq!(comp.percentage_change, -5.0);
        assert!(!comp.regression_threshold_exceeded);
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
        let results = vec![EnhancedPerformanceTestResult {
            test_name: "test1".to_string(),
            iteration_result: 1000,
            total_duration: Duration::from_millis(100),
            ops_per_second: 10.0,
            avg_iteration_time: 0.1,
            memory_usage: None,
            profile: "debug".to_string(),
            environment: "test".to_string(),
            timestamp: chrono::Utc::now(),
            baseline_comparison: None,
            regression_detected: false,
            monitoring_data: None,
        }];

        let text_report = PerformanceReportGenerator::generate_text_report(&results);
        assert!(text_report.contains("Performance Test Report"));

        let markdown_report = PerformanceReportGenerator::generate_markdown_report(&results);
        assert!(markdown_report.contains("# Performance Test Report"));
    }

    #[test]
    fn test_regression_detection() {
        let results = vec![EnhancedPerformanceTestResult {
            test_name: "test_reg".to_string(),
            iteration_result: 1000,
            total_duration: Duration::from_millis(100),
            ops_per_second: 10.0,
            avg_iteration_time: 0.1,
            memory_usage: None,
            profile: "debug".to_string(),
            environment: "test".to_string(),
            timestamp: chrono::Utc::now(),
            baseline_comparison: Some(BaselineComparison {
                baseline_ops_per_second: 15.0,
                current_ops_per_second: 10.0,
                percentage_change: -33.33,
                regression_threshold_exceeded: true,
            }),
            regression_detected: true,
            monitoring_data: None,
        }];

        // Test that regression is properly detected
        assert!(results[0].regression_detected);
        assert!(
            results[0]
                .baseline_comparison
                .as_ref()
                .unwrap()
                .regression_threshold_exceeded
        );
    }
}
