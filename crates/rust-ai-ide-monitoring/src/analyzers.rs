//! Analyzer framework and trait definitions

use crate::{
    config::AnalyzerConfig,
    errors::{MonitoringError, Result},
    types::{AnalysisResult, Category, Finding, Severity},
};
use async_trait::async_trait;
use regex::Regex;
use rust_ai_ide_performance::{PerformanceAnalyzer as PerfAnalyzer, PerformanceMetrics};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::{fs, process::Command};

/// Core trait for all analyzers
#[async_trait]
pub trait Analyzer: Send + Sync {
    /// Get the name of this analyzer
    fn name(&self) -> &str;

    /// Get the category this analyzer belongs to
    fn category(&self) -> Category;

    /// Check if this analyzer is enabled for the current configuration
    fn is_enabled(&self, config: &AnalyzerConfig) -> bool {
        config.enabled
    }

    /// Run the analysis
    async fn analyze(&self, workspace_root: &std::path::Path) -> Result<AnalysisResult>;

    /// Get the description of what this analyzer does
    fn description(&self) -> &str;

    /// Get the severity level for findings from this analyzer
    fn default_severity(&self) -> Severity {
        Severity::Medium
    }

    /// Get dependencies this analyzer requires
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }

    /// Check if this analyzer can run on the current system
    async fn can_run(&self) -> bool {
        // Default implementation checks for required dependencies
        let deps = self.dependencies();
        if deps.is_empty() {
            return true;
        }

        for dep in deps {
            if !self.check_dependency(&dep).await {
                return false;
            }
        }

        true
    }

    /// Check if a dependency is available (default implementation)
    async fn check_dependency(&self, dep: &str) -> bool {
        use tokio::process::Command;

        match dep {
            "cargo" => {
                Command::new("cargo")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "rustc" => {
                Command::new("rustc")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "clippy" => {
                Command::new("cargo")
                    .arg("clippy")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "cargo-geiger" => {
                Command::new("cargo")
                    .arg("geiger")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "cargo-audit" => {
                Command::new("cargo")
                    .arg("audit")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            "cargo-deny" => {
                Command::new("cargo")
                    .arg("deny")
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            _ => {
                // Generic command check
                Command::new(dep)
                    .arg("--version")
                    .output()
                    .await
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
        }
    }
}

/// Analyzer factory function type
pub type AnalyzerFactory = Box<dyn Fn() -> Box<dyn Analyzer> + Send + Sync>;

/// Analyzer registry for managing available analyzers
pub struct AnalyzerRegistry {
    analyzers: std::collections::HashMap<String, AnalyzerFactory>,
}

impl AnalyzerRegistry {
    /// Create a new analyzer registry
    pub fn new() -> Self {
        Self {
            analyzers: std::collections::HashMap::new(),
        }
    }

    /// Register an analyzer factory
    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> Box<dyn Analyzer> + Send + Sync + 'static,
    {
        self.analyzers.insert(name.to_string(), Box::new(factory));
    }

    /// Get an analyzer instance by name
    pub fn get(&self, name: &str) -> Option<Box<dyn Analyzer>> {
        self.analyzers.get(name).map(|factory| factory())
    }

    /// List all registered analyzer names
    pub fn list(&self) -> Vec<String> {
        self.analyzers.keys().cloned().collect()
    }

    /// Check if analyzer is registered
    pub fn has(&self, name: &str) -> bool {
        self.analyzers.contains_key(name)
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register built-in analyzers
        registry.register("cargo-check", || Box::new(CargoCheckAnalyzer::new()));
        registry.register("unused-variables", || Box::new(UnusedVariableAnalyzer::new()));
        registry.register("performance", || Box::new(PerformanceAnalyzer::new()));
        registry.register("security", || Box::new(SecurityAnalyzer::new()));
        registry.register("cross-platform", || Box::new(CrossPlatformAnalyzer::new()));
        registry.register("dependencies", || Box::new(DependencyAnalyzer::new()));

        registry
    }
}

/// Placeholder analyzer implementations (to be replaced with actual implementations)
pub mod cargo_check;

// Re-export actual analyzer implementations
pub use cargo_check::CargoCheckAnalyzer;
#[cfg(feature = "unused-variables")]
pub mod unused_variables;
#[cfg(feature = "performance")]
pub mod performance;
#[cfg(feature = "security")]
pub mod security;
#[cfg(feature = "cross-platform")]
pub mod cross_platform;
#[cfg(feature = "dependencies")]
pub mod dependencies;


pub struct UnusedVariableAnalyzer;
impl UnusedVariableAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for UnusedVariableAnalyzer {
    fn name(&self) -> &str {
        "unused-variables"
    }

    fn category(&self) -> Category {
        Category::CodeQuality
    }

    fn description(&self) -> &str {
        "Detect and categorize unused variables"
    }

    async fn analyze(&self, workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        if !self.can_run().await {
            return Ok(AnalysisResult {
                analyzer: self.name().to_string(),
                success: false,
                severity: Severity::None,
                category: self.category(),
                issue_count: 0,
                findings: vec![Finding {
                    file: workspace_root.join("Cargo.toml"),
                    line: None,
                    column: None,
                    issue_type: "tool_not_available".to_string(),
                    severity: Severity::None,
                    message: "tool not available".to_string(),
                    code: None,
                    suggestion: Some("Install required tools".to_string()),
                }],
                performance: None,
                error: None,
            });
        }
        let mut findings = Vec::new();
        let mut issue_count = 0;

        // Run cargo check with warnings enabled for unused variables
        let output = Command::new("cargo")
            .args(&["check", "--message-format=json", "--", "-W", "unused-variables"])
            .current_dir(workspace_root)
            .output()
            .await
            .map_err(|e| MonitoringError::other(format!("Failed to run cargo check: {}", e)))?;

        // Always parse stdout JSON stream for diagnostics
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        for line in lines {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(reason) = json_value.get("reason") {
                    if reason == "compiler-message" {
                        if let Some(message) = json_value.get("message") {
                            let code = message.get("code").and_then(|c| c.get("code")).and_then(|cc| cc.as_str());
                            let rendered = message.get("rendered").and_then(|r| r.as_str()).unwrap_or("");

                            if code == Some("unused_variables") || rendered.contains("unused") {
                                if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                                    if let Some(span) = spans.get(0) {
                                        let file_name = span.get("file_name").and_then(|f| f.as_str()).unwrap_or("");
                                        let line_num = span.get("line_start").and_then(|l| l.as_u64()).unwrap_or(0) as usize;
                                        let col_num = span.get("column_start").and_then(|c| c.as_u64()).unwrap_or(0) as usize;

                                        let var_name = if let Some(captures) = Regex::new(r#"unused variable: `(\w+)`"#)
                                            .ok()
                                            .and_then(|re| re.captures(rendered))
                                        {
                                            captures.get(1).unwrap().as_str()
                                        } else {
                                            "unknown"
                                        };

                                        let scope = self.categorize_variable_scope(var_name, rendered);
                                        let usage_pattern = self.analyze_usage_pattern(var_name, rendered);

                                        findings.push(Finding {
                                            file: workspace_root.join(file_name),
                                            line: Some(line_num),
                                            column: Some(col_num),
                                            issue_type: format!("unused_variable_{}", scope),
                                            severity: Severity::Low,
                                            message: format!("Unused variable '{}' in {} scope", var_name, scope),
                                            code: Some(format!("let {} = ...;", var_name)),
                                            suggestion: Some(format!("Remove the unused variable '{}' or prefix with '_' if intentional", var_name)),
                                        });
                                        issue_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let severity = if issue_count > 10 {
            Severity::Medium
        } else if issue_count > 0 {
            Severity::Low
        } else {
            Severity::None
        };

        Ok(AnalysisResult {
            analyzer: self.name().to_string(),
            success: true,
            severity,
            category: self.category(),
            issue_count,
            findings,
            performance: None,
            error: None,
        })
    }

    fn categorize_variable_scope(&self, var_name: &str, message: &str) -> &'static str {
        if message.contains("function") {
            "function"
        } else if message.contains("method") {
            "method"
        } else if message.contains("closure") {
            "closure"
        } else if message.contains("module") {
            "module"
        } else if var_name.starts_with(|c: char| c.is_uppercase()) {
            "constant"
        } else {
            "local"
        }
    }

    fn analyze_usage_pattern(&self, var_name: &str, message: &str) -> &'static str {
        if message.contains("never used") {
            "never_used"
        } else if message.contains("assigned but never used") {
            "assigned_never_used"
        } else if message.contains("used in pattern") {
            "pattern_binding"
        } else {
            "other"
        }
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo".to_string()]
    }
}

pub struct PerformanceAnalyzer;
impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for PerformanceAnalyzer {
    fn name(&self) -> &str {
        "performance"
    }

    fn category(&self) -> Category {
        Category::Performance
    }

    fn description(&self) -> &str {
        "Monitor compilation and runtime performance"
    }

    async fn can_run(&self) -> bool {
        // Check if cargo is available and either nightly is available or at least cargo build works
        if !self.check_dependency("cargo").await {
            return false;
        }

        // Check if nightly rust is available
        if self.is_nightly_available().await {
            return true;
        }

        // Check if cargo build --release works as fallback
        let test_build = Command::new("cargo")
            .args(&["build", "--release", "--quiet"])
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);

        test_build
    }

    async fn is_nightly_available(&self) -> bool {
        let output = Command::new("rustc")
            .arg("--version")
            .output()
            .await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    version_str.contains("nightly")
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    async fn analyze(&self, workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        // Gate collection if analyzer cannot run
        if !self.can_run().await {
            return Ok(AnalysisResult {
                analyzer: self.name().to_string(),
                success: false,
                severity: Severity::None,
                category: self.category(),
                issue_count: 0,
                findings: vec![Finding {
                    file: workspace_root.join("Cargo.toml"),
                    line: None,
                    column: None,
                    issue_type: "tool_not_available".to_string(),
                    severity: Severity::None,
                    message: "tool not available".to_string(),
                    code: None,
                    suggestion: Some("Install required tools".to_string()),
                }],
                performance: None,
                error: None,
            });
        }

        let mut findings = Vec::new();
        let mut issue_count = 0;
        let mut using_fallback = false;

        // Check if nightly is available for detailed timings
        let nightly_available = self.is_nightly_available().await;

        // Measure compilation time
        let start_time = Instant::now();
        let output = if nightly_available {
            // Use nightly timings for detailed crate-level metrics
            Command::new("cargo")
                .args(&["build", "--release", "-Z", "timings"])
                .current_dir(workspace_root)
                .output()
                .await
        } else {
            // Fallback to standard build and parse completion time
            using_fallback = true;
            Command::new("cargo")
                .args(&["build", "--release"])
                .current_dir(workspace_root)
                .output()
                .await
        };

        let output = output.map_err(|e| {
            MonitoringError::other(format!("Failed to run cargo build: {}", e))
        })?;

        let compilation_time = start_time.elapsed();

        let mut total_build_time = Duration::from_secs(0);
        let mut crate_times = HashMap::new();

        // Parse timing output based on available features
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();

            if nightly_available {
                // Parse detailed nightly timings
                for line in lines {
                    if line.contains("time:") && line.contains("finished in") {
                        if let Some(captures) = Regex::new(r#"(\w+).*time:\s*([\d.]+)\s*s"#)
                            .ok()
                            .and_then(|re| re.captures(line))
                        {
                            let crate_name = captures.get(1).unwrap().as_str();
                            if let Ok(time) = captures.get(2).unwrap().as_str().parse::<f64>() {
                                let duration = Duration::from_secs_f64(time);
                                crate_times.insert(crate_name.to_string(), duration);
                                total_build_time += duration;
                            }
                        }
                    }
                }
            } else {
                // Fallback: parse total build time from "finished in" message
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                let all_output = format!("{}\n{}", output_str, stderr_str);

                if let Some(captures) = Regex::new(r#"finished in ([\d.]+)s"#)
                    .ok()
                    .and_then(|re| re.captures(&all_output))
                {
                    if let Ok(time) = captures.get(1).unwrap().as_str().parse::<f64>() {
                        total_build_time = Duration::from_secs_f64(time);
                    }
                }
                // Note: Individual crate times not available in fallback mode
            }
        }

        // Integrate with performance crate
        let perf_analyzer = PerfAnalyzer::new();
        let mut perf_metrics = PerformanceMetrics::new();
        perf_metrics.cpu_usage_percent = Some(0.0); // Placeholder - would need system monitoring
        perf_metrics.memory_usage_mb = Some(0.0); // Placeholder

        // Use total_build_time if available from nightly timings, otherwise use measured time
        let recorded_time_ms = if nightly_available && total_build_time.as_secs_f64() > 0.0 {
            total_build_time.as_millis() as f64
        } else {
            compilation_time.as_millis() as f64
        };
        perf_metrics.response_time_ms = Some(recorded_time_ms);

        perf_analyzer.record_metric(perf_metrics);

        // Analyze for regressions (compare against thresholds)
        if let Some(avg_metrics) = perf_analyzer.get_averages() {
            if let Some(avg_response_time) = avg_metrics.response_time_ms {
                if recorded_time_ms > avg_response_time * 1.5 {
                    let mut severity = Severity::High;
                    if using_fallback {
                        severity = Severity::Medium; // Lower severity for fallback measurements
                    }

                    findings.push(Finding {
                        file: workspace_root.join("Cargo.toml"),
                        line: None,
                        column: None,
                        issue_type: "compilation_regression".to_string(),
                        severity,
                        message: format!("Compilation time regression: {:.2}s (avg: {:.2}s){}",
                            recorded_time_ms / 1000.0,
                            avg_response_time / 1000.0,
                            if using_fallback { " [fallback measurement]" } else { "" }
                        ),
                        code: None,
                        suggestion: Some("Consider optimizing build dependencies or using incremental compilation".to_string()),
                    });
                    issue_count += 1;
                }
            }
        }

        // Check for slow crates (only available with nightly timings)
        if nightly_available {
            for (crate_name, duration) in crate_times.iter() {
                if duration.as_secs_f64() > 30.0 { // Threshold for slow compilation
                    findings.push(Finding {
                        file: workspace_root.join(format!("crates/{}/Cargo.toml", crate_name)),
                        line: None,
                        column: None,
                        issue_type: "slow_compilation".to_string(),
                        severity: Severity::Medium,
                        message: format!("Slow compilation: {} took {:.2}s", crate_name, duration.as_secs_f64()),
                        code: None,
                        suggestion: Some(format!("Consider optimizing dependencies or code generation in {}", crate_name)),
                    });
                    issue_count += 1;
                }
            }
        }

        // Overall performance assessment
        let mut severity = if issue_count > 5 {
            Severity::High
        } else if issue_count > 2 {
            Severity::Medium
        } else if issue_count > 0 {
            Severity::Low
        } else {
            Severity::None
        };

        // Lower severity when using fallback measurements (less accurate)
        if using_fallback && severity != Severity::None {
            severity = match severity {
                Severity::High => Severity::Medium,
                Severity::Medium => Severity::Low,
                _ => severity,
            };
        }

        Ok(AnalysisResult {
            analyzer: self.name().to_string(),
            success: true,
            severity,
            category: self.category(),
            issue_count,
            findings,
            performance: None,
            error: None,
        })
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo".to_string(), "rustc".to_string()]
    }
}

pub struct SecurityAnalyzer;
impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for SecurityAnalyzer {
    fn name(&self) -> &str {
        "security"
    }

    fn category(&self) -> Category {
        Category::Security
    }

    fn description(&self) -> &str {
        "Analyze security vulnerabilities and unsafe code"
    }

    async fn analyze(&self, workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        if !self.can_run().await {
            return Ok(AnalysisResult {
                analyzer: self.name().to_string(),
                success: false,
                severity: Severity::None,
                category: self.category(),
                issue_count: 0,
                findings: vec![Finding {
                    file: workspace_root.join("Cargo.toml"),
                    line: None,
                    column: None,
                    issue_type: "tool_not_available".to_string(),
                    severity: Severity::None,
                    message: "tool not available".to_string(),
                    code: None,
                    suggestion: Some("Install required tools".to_string()),
                }],
                performance: None,
                error: None,
            });
        }
        let mut findings = Vec::new();
        let mut issue_count = 0;

        // Run cargo audit for vulnerability scanning
        let audit_output = Command::new("cargo")
            .args(&["audit", "--format", "json"])
            .current_dir(workspace_root)
            .output()
            .await;

        if let Ok(output) = audit_output {
            if !output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&output_str) {
                    if let Some(vulnerabilities) = json_value.get("vulnerabilities") {
                        if let Some(list) = vulnerabilities.get("list") {
                            if let Some(vulns_array) = list.as_array() {
                                for vuln_item in vulns_array {
                                    let package = vuln_item.get("package").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("unknown");
                                    let severity_str = vuln_item.get("advisory").and_then(|a| a.get("severity")).and_then(|s| s.as_str()).unwrap_or("unknown");
                                    let title = vuln_item.get("advisory").and_then(|a| a.get("title")).and_then(|t| t.as_str()).unwrap_or("Unknown vulnerability");

                                    let severity = match severity_str {
                                        "critical" => Severity::Critical,
                                        "high" => Severity::High,
                                        "medium" => Severity::Medium,
                                        "low" => Severity::Low,
                                        _ => Severity::Medium,
                                    };

                                    findings.push(Finding {
                                        file: workspace_root.join("Cargo.lock"),
                                        line: None,
                                        column: None,
                                        issue_type: "vulnerability".to_string(),
                                        severity,
                                        message: format!("Security vulnerability in {}: {}", package, title),
                                        code: None,
                                        suggestion: Some(format!("Update {} to a patched version", package)),
                                    });
                                    issue_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            findings.push(Finding {
                file: workspace_root.join("Cargo.toml"),
                line: None,
                column: None,
                issue_type: "tool_not_available".to_string(),
                severity: Severity::None,
                message: "cargo audit not available".to_string(),
                code: None,
                suggestion: Some("Install with: cargo install cargo-audit".to_string()),
            });
        }

        // Run cargo geiger for unsafe code analysis
        let geiger_output = Command::new("cargo")
            .args(&["geiger", "--output-format", "json"])
            .current_dir(workspace_root)
            .output()
            .await;

        if let Ok(output) = geiger_output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&output_str) {
                    if let Some(packages) = json_value.get("packages").and_then(|p| p.as_array()) {
                        for package in packages {
                            let name = package.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                            let unsafe_functions = package.get("unsafety").and_then(|u| u.get("functions")).and_then(|f| f.get("unsafe")).and_then(|us| us.as_u64()).unwrap_or(0);

                            if unsafe_functions > 10 { // Threshold for concerning unsafe usage
                                findings.push(Finding {
                                    file: workspace_root.join(format!("crates/{}/src/lib.rs", name)),
                                    line: None,
                                    column: None,
                                    issue_type: "unsafe_code".to_string(),
                                    severity: Severity::Medium,
                                    message: format!("High unsafe code usage in {}: {} unsafe functions", name, unsafe_functions),
                                    code: None,
                                    suggestion: Some("Review unsafe code usage and consider safer alternatives".to_string()),
                                });
                                issue_count += 1;
                            }
                        }
                    }
                }
            }
        } else {
            findings.push(Finding {
                file: workspace_root.join("Cargo.toml"),
                line: None,
                column: None,
                issue_type: "tool_not_available".to_string(),
                severity: Severity::None,
                message: "cargo-geiger not available".to_string(),
                code: None,
                suggestion: Some("Install with: cargo install cargo-geiger".to_string()),
            });
        }

        // Check for common security anti-patterns
        self.check_security_antipatterns(workspace_root, &mut findings, &mut issue_count).await;

        let severity = if issue_count > 0 {
            findings.iter().map(|f| f.severity).max().unwrap_or(Severity::None)
        } else {
            Severity::None
        };

        Ok(AnalysisResult {
            analyzer: self.name().to_string(),
            success: true,
            severity,
            category: self.category(),
            issue_count,
            findings,
            performance: None,
            error: None,
        })
    }

    async fn check_security_antipatterns(&self, workspace_root: &std::path::Path, findings: &mut Vec<Finding>, issue_count: &mut usize) {
        // Check for plain text secrets (basic pattern matching)
        let secret_patterns = vec![
            Regex::new(r#"password\s*=\s*["'][^"']*["']"#).unwrap(),
            Regex::new(r#"secret\s*=\s*["'][^"']*["']"#).unwrap(),
            Regex::new(r#"token\s*=\s*["'][^"']*["']"#).unwrap(),
        ];

        // This is a simplified check - in practice you'd scan all source files
        let cargo_toml_path = workspace_root.join("Cargo.toml");
        if cargo_toml_path.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml_path).await {
                for pattern in &secret_patterns {
                    if pattern.is_match(&content) {
                        findings.push(Finding {
                            file: cargo_toml_path.clone(),
                            line: None,
                            column: None,
                            issue_type: "plain_text_secret".to_string(),
                            severity: Severity::High,
                            message: "Potential plain text secret found in configuration".to_string(),
                            code: None,
                            suggestion: Some("Move secrets to secure environment variables or encrypted storage".to_string()),
                        });
                        *issue_count += 1;
                    }
                }
            }
        }
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo-audit".to_string(), "cargo-geiger".to_string()]
    }
}

pub struct CrossPlatformAnalyzer;
impl CrossPlatformAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for CrossPlatformAnalyzer {
    fn name(&self) -> &str {
        "cross-platform"
    }

    fn category(&self) -> Category {
        Category::CrossPlatform
    }

    fn description(&self) -> &str {
        "Validate cross-platform compilation compatibility"
    }

    async fn analyze(&self, workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        if !self.can_run().await {
            return Ok(AnalysisResult {
                analyzer: self.name().to_string(),
                success: false,
                severity: Severity::None,
                category: self.category(),
                issue_count: 0,
                findings: vec![Finding {
                    file: workspace_root.join("Cargo.toml"),
                    line: None,
                    column: None,
                    issue_type: "tool_not_available".to_string(),
                    severity: Severity::None,
                    message: "tool not available".to_string(),
                    code: None,
                    suggestion: Some("Install required tools".to_string()),
                }],
                performance: None,
                error: None,
            });
        }
        let mut findings = Vec::new();
        let mut issue_count = 0;
        let mut successful_targets = Vec::new();

        // Common targets to test
        let targets = vec![
            "x86_64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "x86_64-pc-windows-gnu",
            "aarch64-unknown-linux-gnu",
            "aarch64-apple-darwin",
        ];

        for target in &targets {
            let output = Command::new("cargo")
                .args(&["check", "--target", target])
                .current_dir(workspace_root)
                .output()
                .await;

            match output {
                Ok(output) => {
                    if output.status.success() {
                        successful_targets.push(target.to_string());
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        // Parse compilation errors for platform-specific issues
                        let lines: Vec<&str> = stderr.lines().collect();
                        for line in lines {
                            if line.contains("error") || line.contains("warning") {
                                if let Some(captures) = Regex::new(r#"(\w+)\.rs:(\d+):(\d+): (error|warning): (.+)"#)
                                    .ok()
                                    .and_then(|re| re.captures(line))
                                {
                                    let file_path = captures.get(1).unwrap().as_str();
                                    let line_num = captures.get(2).unwrap().as_str().parse::<usize>().unwrap_or(0);
                                    let col_num = captures.get(3).unwrap().as_str().parse::<usize>().unwrap_or(0);
                                    let error_type = captures.get(4).unwrap().as_str();
                                    let message = captures.get(5).unwrap().as_str();

                                    let severity = if error_type == "error" {
                                        Severity::High
                                    } else {
                                        Severity::Medium
                                    };

                                    findings.push(Finding {
                                        file: workspace_root.join(format!("{}.rs", file_path)),
                                        line: Some(line_num),
                                        column: Some(col_num),
                                        issue_type: format!("cross_platform_{}", error_type),
                                        severity,
                                        message: format!("{} on target {}: {}", error_type, target, message),
                                        code: None,
                                        suggestion: Some(format!("Fix platform-specific code for target {}", target)),
                                    });
                                    issue_count += 1;
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    findings.push(Finding {
                        file: workspace_root.join("Cargo.toml"),
                        line: None,
                        column: None,
                        issue_type: "target_unavailable".to_string(),
                        severity: Severity::None,
                        message: format!("Target {} is not available on this system", target),
                        code: None,
                        suggestion: Some(format!("Install target {} with: rustup target add {}", target, target)),
                    });
                }
            }
        }

        // Check for platform-dependent dependencies
        self.check_platform_dependencies(workspace_root, &mut findings, &mut issue_count).await;

        // Check for platform-specific code patterns
        self.check_platform_code_patterns(workspace_root, &mut findings, &mut issue_count).await;

        let severity = if issue_count > targets.len() / 2 {
            Severity::High
        } else if issue_count > 0 {
            Severity::Medium
        } else {
            Severity::None
        };

        Ok(AnalysisResult {
            analyzer: self.name().to_string(),
            success: true,
            severity,
            category: self.category(),
            issue_count,
            findings,
            performance: None,
            error: None,
        })
    }

    async fn check_platform_dependencies(&self, workspace_root: &std::path::Path, findings: &mut Vec<Finding>, issue_count: &mut usize) {
        let cargo_toml_path = workspace_root.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&cargo_toml_path).await {
            // Check for platform-specific dependencies
            if content.contains("target.") && content.contains("dependencies") {
                findings.push(Finding {
                    file: cargo_toml_path,
                    line: None,
                    column: None,
                    issue_type: "platform_dependencies".to_string(),
                    severity: Severity::Info,
                    message: "Platform-specific dependencies detected".to_string(),
                    code: None,
                    suggestion: None,
                });
                *issue_count += 1;
            }
        }
    }

    async fn check_platform_code_patterns(&self, workspace_root: &std::path::Path, findings: &mut Vec<Finding>, issue_count: &mut usize) {
        // This would scan source files for platform-specific patterns like cfg! macros
        // Simplified version - in practice you'd scan all .rs files
        let lib_rs_path = workspace_root.join("src/lib.rs");
        if let Ok(content) = fs::read_to_string(&lib_rs_path).await {
            if content.contains("cfg!(target_os") || content.contains("cfg!(target_arch") {
                findings.push(Finding {
                    file: lib_rs_path,
                    line: None,
                    column: None,
                    issue_type: "platform_cfg".to_string(),
                    severity: Severity::Info,
                    message: "Platform-specific conditional compilation detected".to_string(),
                    code: None,
                    suggestion: Some("Ensure all platform branches are tested".to_string()),
                });
                *issue_count += 1;
            }
        }
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo".to_string()]
    }
}

pub struct DependencyAnalyzer;
impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Analyzer for DependencyAnalyzer {
    fn name(&self) -> &str {
        "dependencies"
    }

    fn category(&self) -> Category {
        Category::Dependencies
    }

    fn description(&self) -> &str {
        "Analyze dependency health and compatibility"
    }

    async fn analyze(&self, workspace_root: &std::path::Path) -> Result<AnalysisResult> {
        if !self.can_run().await {
            return Ok(AnalysisResult {
                analyzer: self.name().to_string(),
                success: false,
                severity: Severity::None,
                category: self.category(),
                issue_count: 0,
                findings: vec![Finding {
                    file: workspace_root.join("Cargo.toml"),
                    line: None,
                    column: None,
                    issue_type: "tool_not_available".to_string(),
                    severity: Severity::None,
                    message: "tool not available".to_string(),
                    code: None,
                    suggestion: Some("Install required tools".to_string()),
                }],
                performance: None,
                error: None,
            });
        }
        let mut findings = Vec::new();
        let mut issue_count = 0;

        // Run cargo deny for license and policy checking
        let deny_output = Command::new("cargo")
            .args(&["deny", "check"])
            .current_dir(workspace_root)
            .output()
            .await;

        if let Ok(output) = deny_output {
            if !output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();

                for line in lines {
                    if line.contains("error") || line.contains("warning") || line.contains("banned") || line.contains("license") {
                        if let Some(captures) = Regex::new(r#"(.+?):\s*(.+)"#)
                            .ok()
                            .and_then(|re| re.captures(line))
                        {
                            let issue = captures.get(1).unwrap().as_str().trim();
                            let details = captures.get(2).unwrap().as_str().trim();

                            let severity = if line.contains("error") || line.contains("banned") {
                                Severity::High
                            } else if line.contains("warning") {
                                Severity::Medium
                            } else {
                                Severity::Low
                            };

                            findings.push(Finding {
                                file: workspace_root.join("Cargo.toml"),
                                line: None,
                                column: None,
                                issue_type: "dependency_policy".to_string(),
                                severity,
                                message: format!("Dependency policy issue: {} - {}", issue, details),
                                code: None,
                                suggestion: Some("Review and update dependency or policy configuration".to_string()),
                            });
                            issue_count += 1;
                        }
                    }
                }
            }
        } else {
            findings.push(Finding {
                file: workspace_root.join("Cargo.toml"),
                line: None,
                column: None,
                issue_type: "cargo_deny_unavailable".to_string(),
                severity: Severity::None,
                message: "cargo-deny not available for dependency policy checking".to_string(),
                code: None,
                suggestion: Some("Install cargo-deny: cargo install cargo-deny".to_string()),
            });
        }

        // Check for outdated dependencies
        let outdated_output = Command::new("cargo")
            .args(&["outdated"])
            .current_dir(workspace_root)
            .output()
            .await;

        if let Ok(output) = outdated_output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();

                for line in lines {
                    if line.contains("->") && line.contains("available") {
                        // Parse outdated dependency info
                        if let Some(captures) = Regex::new(r#"(\w+)\s+(.+?)\s+->\s+(.+?)\s+available"#)
                            .ok()
                            .and_then(|re| re.captures(line))
                        {
                            let crate_name = captures.get(1).unwrap().as_str();
                            let current_version = captures.get(2).unwrap().as_str();
                            let available_version = captures.get(3).unwrap().as_str();

                            findings.push(Finding {
                                file: workspace_root.join("Cargo.toml"),
                                line: None,
                                column: None,
                                issue_type: "outdated_dependency".to_string(),
                                severity: Severity::Info,
                                message: format!("{} is outdated: {} -> {}", crate_name, current_version, available_version),
                                code: None,
                                suggestion: Some(format!("Consider updating {} to version {}", crate_name, available_version)),
                            });
                            issue_count += 1;
                        }
                    }
                }
            }
        } else {
            findings.push(Finding {
                file: workspace_root.join("Cargo.toml"),
                line: None,
                column: None,
                issue_type: "cargo_outdated_unavailable".to_string(),
                severity: Severity::None,
                message: "cargo-outdated not available for dependency checking".to_string(),
                code: None,
                suggestion: Some("Install cargo-outdated: cargo install cargo-outdated".to_string()),
            });
        }

        // Check for dependency conflicts using cargo tree
        let tree_output = Command::new("cargo")
            .args(&["tree", "--duplicates"])
            .current_dir(workspace_root)
            .output()
            .await;

        if let Ok(output) = tree_output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("duplicates") {
                    findings.push(Finding {
                        file: workspace_root.join("Cargo.lock"),
                        line: None,
                        column: None,
                        issue_type: "dependency_conflicts".to_string(),
                        severity: Severity::Medium,
                        message: "Dependency conflicts detected in workspace".to_string(),
                        code: None,
                        suggestion: Some("Run 'cargo tree --duplicates' to identify conflicting dependencies".to_string()),
                    });
                    issue_count += 1;
                }
            }
        } else {
            findings.push(Finding {
                file: workspace_root.join("Cargo.toml"),
                line: None,
                column: None,
                issue_type: "tool_not_available".to_string(),
                severity: Severity::None,
                message: "cargo not available".to_string(),
                code: None,
                suggestion: Some("Install cargo".to_string()),
            });
        }

        // Analyze Cargo.toml for dependency health
        self.analyze_cargo_toml(workspace_root, &mut findings, &mut issue_count).await;

        let severity = if findings.iter().any(|f| f.severity == Severity::High) {
            Severity::High
        } else if findings.iter().any(|f| f.severity == Severity::Medium) {
            Severity::Medium
        } else if issue_count > 0 {
            Severity::Low
        } else {
            Severity::None
        };

        Ok(AnalysisResult {
            analyzer: self.name().to_string(),
            success: true,
            severity,
            category: self.category(),
            issue_count,
            findings,
            performance: None,
            error: None,
        })
    }

    async fn analyze_cargo_toml(&self, workspace_root: &std::path::Path, findings: &mut Vec<Finding>, issue_count: &mut usize) {
        let cargo_toml_path = workspace_root.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&cargo_toml_path).await {
            // Check for wildcard version specifiers
            if content.contains("\"*\"") || content.contains("'*'") {
                findings.push(Finding {
                    file: cargo_toml_path.clone(),
                    line: None,
                    column: None,
                    issue_type: "wildcard_versions".to_string(),
                    severity: Severity::Medium,
                    message: "Wildcard version specifiers found - may cause unstable builds".to_string(),
                    code: None,
                    suggestion: Some("Use specific version ranges instead of wildcards".to_string()),
                });
                *issue_count += 1;
            }

            // Check for pre-release versions
            if Regex::new(r#""\d+\.\d+\.\d+-[a-zA-Z]|\d+\.\d+\.\d+-rc|\d+\.\d+\.\d+-beta"#)
                .unwrap()
                .is_match(&content)
            {
                findings.push(Finding {
                    file: cargo_toml_path,
                    line: None,
                    column: None,
                    issue_type: "prerelease_versions".to_string(),
                    severity: Severity::Low,
                    message: "Pre-release versions detected in dependencies".to_string(),
                    code: None,
                    suggestion: Some("Consider using stable versions for production".to_string()),
                });
                *issue_count += 1;
            }
        }
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["cargo-deny".to_string(), "cargo-outdated".to_string()]
    }
}