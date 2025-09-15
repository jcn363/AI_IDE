//! CI/CD integration utilities for automated testing workflows
//!
//! Provides test coverage analysis, benchmarking utilities, CI pipeline helpers,
//! and reporting tools for automated testing environments.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::error::TestError;

/// CI/CD environment detector
pub struct CIEnvironment {
    pub provider: CIProvider,
    pub branch: String,
    pub commit: String,
    pub build_number: Option<String>,
    pub is_pull_request: bool,
}

#[derive(Debug, Clone)]
pub enum CIProvider {
    GitHubActions,
    GitLabCI,
    Jenkins,
    TravisCI,
    CircleCI,
    AzureDevOps,
    Unknown,
}

impl CIEnvironment {
    /// Detect the CI environment from environment variables
    pub fn detect() -> Self {
        let env_vars = std::env::vars().collect::<HashMap<_, _>>();

        let provider = if env_vars.contains_key("GITHUB_ACTIONS") {
            CIProvider::GitHubActions
        } else if env_vars.contains_key("GITLAB_CI") {
            CIProvider::GitLabCI
        } else if env_vars.contains_key("JENKINS_HOME") {
            CIProvider::Jenkins
        } else if env_vars.contains_key("TRAVIS") {
            CIProvider::TravisCI
        } else if env_vars.contains_key("CIRCLECI") {
            CIProvider::CircleCI
        } else if env_vars.contains_key("TF_BUILD") {
            CIProvider::AzureDevOps
        } else {
            CIProvider::Unknown
        };

        let branch = env_vars
            .get("BRANCH_NAME")
            .or_else(|| env_vars.get("GITHUB_REF_NAME"))
            .or_else(|| env_vars.get("CI_COMMIT_REF_NAME"))
            .or_else(|| env_vars.get("BRANCH"))
            .map(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();

        let commit = env_vars
            .get("COMMIT_SHA")
            .or_else(|| env_vars.get("GITHUB_SHA"))
            .or_else(|| env_vars.get("CI_COMMIT_SHA"))
            .or_else(|| env_vars.get("COMMIT"))
            .map(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();

        let build_number = env_vars
            .get("BUILD_NUMBER")
            .or_else(|| env_vars.get("GITHUB_RUN_NUMBER"))
            .or_else(|| env_vars.get("CI_JOB_ID"))
            .or_else(|| env_vars.get("BUILD_ID"))
            .cloned();

        let is_pull_request = match provider {
            CIProvider::GitHubActions => env_vars.contains_key("GITHUB_HEAD_REF"),
            CIProvider::GitLabCI => env_vars
                .get("CI_MERGE_REQUEST_TARGET_BRANCH_NAME")
                .is_some(),
            _ => {
                env_vars.get("CI_PULL_REQUEST").is_some() || env_vars.get("PULL_REQUEST").is_some()
            }
        };

        Self {
            provider,
            branch,
            commit,
            build_number,
            is_pull_request,
        }
    }

    /// Check if running in CI environment
    pub fn is_ci() -> bool {
        std::env::var("CI").is_ok()
            || std::env::var("CONTINUOUS_INTEGRATION").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("GITLAB_CI").is_ok()
    }
}

/// Test coverage analyzer
pub struct CoverageAnalyzer {
    coverage_data: HashMap<String, FileCoverage>,
    threshold: f64,
}

#[derive(Debug, Clone)]
pub struct FileCoverage {
    pub file_path: String,
    pub line_coverage: HashMap<usize, bool>,
    pub branch_coverage: HashMap<String, bool>,
    pub total_lines: usize,
    pub covered_lines: usize,
}

impl CoverageAnalyzer {
    pub fn new() -> Self {
        Self {
            coverage_data: HashMap::new(),
            threshold: 80.0,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Add coverage data from a file
    pub fn add_file_coverage(
        &mut self,
        file_path: &str,
        lines: &[(usize, bool)],
    ) -> Result<(), TestError> {
        let mut covered_lines = 0;
        let mut line_coverage = HashMap::new();

        for (line, covered) in lines {
            line_coverage.insert(*line, *covered);
            if *covered {
                covered_lines += 1;
            }
        }

        let total_lines = lines.len();
        let coverage = FileCoverage {
            file_path: file_path.to_string(),
            line_coverage,
            branch_coverage: HashMap::new(),
            total_lines,
            covered_lines,
        };

        self.coverage_data.insert(file_path.to_string(), coverage);
        Ok(())
    }

    /// Calculate overall coverage percentage
    pub fn overall_coverage(&self) -> f64 {
        let total_lines: usize = self.coverage_data.values().map(|f| f.total_lines).sum();
        let covered_lines: usize = self.coverage_data.values().map(|f| f.covered_lines).sum();

        if total_lines == 0 {
            0.0
        } else {
            (covered_lines as f64 / total_lines as f64) * 100.0
        }
    }

    /// Check if coverage meets threshold
    pub fn meets_threshold(&self) -> bool {
        self.overall_coverage() >= self.threshold
    }

    /// Generate coverage report
    pub fn generate_report(&self) -> CoverageReport {
        let overall_coverage = self.overall_coverage();

        let files = self
            .coverage_data
            .values()
            .map(|file| FileCoverageReport {
                file_path: file.file_path.clone(),
                coverage_percentage: (file.covered_lines as f64 / file.total_lines as f64) * 100.0,
                covered_lines: file.covered_lines,
                total_lines: file.total_lines,
            })
            .collect();

        CoverageReport {
            overall_coverage,
            file_count: self.coverage_data.len(),
            files,
            threshold: self.threshold,
            passes_threshold: self.meets_threshold(),
        }
    }
}

impl Default for CoverageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct CoverageReport {
    pub overall_coverage: f64,
    pub file_count: usize,
    pub files: Vec<FileCoverageReport>,
    pub threshold: f64,
    pub passes_threshold: bool,
}

#[derive(Debug)]
pub struct FileCoverageReport {
    pub file_path: String,
    pub coverage_percentage: f64,
    pub covered_lines: usize,
    pub total_lines: usize,
}

/// Benchmarking utilities for performance testing
pub struct BenchmarkRunner {
    iterations: usize,
    warmup_iterations: usize,
    results: Vec<BenchmarkResult>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub iterations: usize,
    pub throughput: f64, // operations per second
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 100,
            results: Vec::new(),
        }
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup_iterations = warmup;
        self
    }

    /// Run benchmark for a synchronous function
    pub fn benchmark_sync<F>(
        &mut self,
        name: &str,
        mut function: F,
    ) -> Result<BenchmarkResult, TestError>
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            function();
        }

        // Benchmark
        let mut times = Vec::new();
        for _ in 0..self.iterations {
            let start = Instant::now();
            function();
            times.push(start.elapsed());
        }

        let total_time: Duration = times.iter().sum();
        let average_time = total_time / self.iterations as u32;
        let min_time = times
            .iter()
            .min()
            .copied()
            .unwrap_or(Duration::from_secs(0));
        let max_time = times
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::from_secs(0));
        let throughput = self.iterations as f64 / total_time.as_secs_f64();

        let result = BenchmarkResult {
            name: name.to_string(),
            average_time,
            min_time,
            max_time,
            iterations: self.iterations,
            throughput,
        };

        self.results.push(result.clone());
        Ok(result)
    }

    /// Run benchmark for an async function
    pub async fn benchmark_async<F, Fut>(
        &mut self,
        name: &str,
        function: F,
    ) -> Result<BenchmarkResult, TestError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            function().await;
        }

        // Benchmark
        let mut times = Vec::new();
        for _ in 0..self.iterations {
            let start = Instant::now();
            function().await;
            times.push(start.elapsed());
        }

        let total_time: Duration = times.iter().sum();
        let average_time = total_time / self.iterations as u32;
        let min_time = times
            .iter()
            .min()
            .copied()
            .unwrap_or(Duration::from_secs(0));
        let max_time = times
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::from_secs(0));
        let throughput = self.iterations as f64 / total_time.as_secs_f64();

        let result = BenchmarkResult {
            name: name.to_string(),
            average_time,
            min_time,
            max_time,
            iterations: self.iterations,
            throughput,
        };

        self.results.push(result.clone());
        Ok(result)
    }

    /// Get all benchmark results
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Generate benchmark report
    pub fn generate_report(&self) -> BenchmarkReport {
        BenchmarkReport {
            total_benchmarks: self.results.len(),
            results: self.results.clone(),
        }
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct BenchmarkReport {
    pub total_benchmarks: usize,
    pub results: Vec<BenchmarkResult>,
}

/// Test reporting utilities
pub struct TestReporter {
    reports: Vec<TestSuiteReport>,
}

#[derive(Debug, Clone)]
pub struct TestSuiteReport {
    pub name: String,
    pub tests: Vec<TestCaseReport>,
    pub duration: Duration,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone)]
pub struct TestCaseReport {
    pub name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

impl TestReporter {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    /// Add a test suite report
    pub fn add_suite(&mut self, name: &str, tests: Vec<TestCaseReport>) -> TestSuiteReport {
        let passed = tests
            .iter()
            .filter(|t| matches!(t.status, TestStatus::Passed))
            .count();
        let failed = tests
            .iter()
            .filter(|t| matches!(t.status, TestStatus::Failed))
            .count();
        let skipped = tests
            .iter()
            .filter(|t| matches!(t.status, TestStatus::Skipped))
            .count();
        let duration = tests.iter().map(|t| t.duration).sum();

        let report = TestSuiteReport {
            name: name.to_string(),
            tests,
            duration,
            passed,
            failed,
            skipped,
        };

        self.reports.push(report.clone());
        report
    }

    /// Generate comprehensive test report
    pub fn generate_report(&self) -> TestReport {
        let total_suites = self.reports.len();
        let total_tests: usize = self.reports.iter().map(|r| r.tests.len()).sum();
        let total_passed: usize = self.reports.iter().map(|r| r.passed).sum();
        let total_failed: usize = self.reports.iter().map(|r| r.failed).sum();
        let total_skipped: usize = self.reports.iter().map(|r| r.skipped).sum();
        let total_duration: Duration = self.reports.iter().map(|r| r.duration).sum();

        TestReport {
            total_suites,
            total_tests,
            total_passed,
            total_failed,
            total_skipped,
            total_duration,
            coverage_percentage: None,
            suites: self.reports.clone(),
        }
    }

    /// Export report to various formats
    pub fn export_report(&self, format: ReportFormat, path: &PathBuf) -> Result<(), TestError> {
        let report = self.generate_report();
        let content = match format {
            ReportFormat::Json => serde_json::to_string_pretty(&report)?,
            ReportFormat::Xml => self.generate_xml_report(),
            ReportFormat::Markdown => self.generate_markdown_report(),
            ReportFormat::JUnit => self.generate_junit_report(),
        };

        std::fs::write(path, content)?;
        Ok(())
    }

    fn generate_xml_report(&self) -> String {
        // Simple XML report generation
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<testsuites>\n");

        for suite in &self.reports {
            xml.push_str(&format!(
                "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" time=\"{}\">\n",
                suite.name,
                suite.tests.len(),
                suite.failed,
                suite.duration.as_secs_f64()
            ));

            for test in &suite.tests {
                let status = match test.status {
                    TestStatus::Passed => "passed",
                    TestStatus::Failed => "failed",
                    TestStatus::Skipped => "skipped",
                    TestStatus::Error => "error",
                };

                xml.push_str(&format!(
                    "    <testcase name=\"{}\" status=\"{}\" time=\"{}\">\n",
                    test.name,
                    status,
                    test.duration.as_secs_f64()
                ));

                if let Some(error) = &test.error_message {
                    xml.push_str(&format!("      <failure message=\"{}\"/>\n", error));
                }

                xml.push_str("    </testcase>\n");
            }

            xml.push_str("  </testsuite>\n");
        }

        xml.push_str("</testsuites>\n");
        xml
    }

    fn generate_markdown_report(&self) -> String {
        let report = self.generate_report();
        let mut md = String::new();

        md.push_str("# Test Report\n\n");
        md.push_str(&format!("**Total Tests:** {}\n", report.total_tests));
        md.push_str(&format!("**Passed:** {}\n", report.total_passed));
        md.push_str(&format!("**Failed:** {}\n", report.total_failed));
        md.push_str(&format!("**Skipped:** {}\n\n", report.total_skipped));

        if let Some(coverage) = report.coverage_percentage {
            md.push_str(&format!("**Coverage:** {:.2}%\n\n", coverage));
        }

        for suite in &report.suites {
            md.push_str(&format!("## {}\n\n", suite.name));
            for test in &suite.tests {
                let status_emoji = match test.status {
                    TestStatus::Passed => "✅",
                    TestStatus::Failed => "❌",
                    TestStatus::Skipped => "⏭️",
                    TestStatus::Error => "⚠️",
                };
                md.push_str(&format!(
                    "- {} {} ({}ms)\n",
                    status_emoji,
                    test.name,
                    test.duration.as_millis()
                ));
            }
            md.push_str("\n");
        }

        md
    }

    fn generate_junit_report(&self) -> String {
        self.generate_xml_report() // JUnit is XML-based
    }
}

impl Default for TestReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TestReport {
    pub total_suites: usize,
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_skipped: usize,
    pub total_duration: Duration,
    pub coverage_percentage: Option<f64>,
    pub suites: Vec<TestSuiteReport>,
}

#[derive(Debug)]
pub enum ReportFormat {
    Json,
    Xml,
    Markdown,
    JUnit,
}

/// CI pipeline utilities
pub struct CIPipeline {
    env: CIEnvironment,
}

impl CIPipeline {
    pub fn new() -> Self {
        Self {
            env: CIEnvironment::detect(),
        }
    }

    /// Run coverage analysis and fail if below threshold
    pub fn require_coverage(&self, analyzer: &CoverageAnalyzer) -> Result<(), TestError> {
        if self.env.is_pull_request || self.env.branch != "main" {
            println!("Skipping coverage check for branch: {}", self.env.branch);
            return Ok(());
        }

        if !analyzer.meets_threshold() {
            return Err(TestError::Validation(
                crate::ValidationError::invalid_setup(format!(
                    "Coverage {:.2}% is below required threshold {:.2}%",
                    analyzer.overall_coverage(),
                    analyzer.threshold
                )),
            ));
        }

        println!("Coverage check passed: {:.2}%", analyzer.overall_coverage());
        Ok(())
    }

    /// Run performance regression tests
    pub fn check_performance_regression(
        &self,
        report: &BenchmarkReport,
        baseline_path: &PathBuf,
    ) -> Result<(), TestError> {
        if !baseline_path.exists() {
            // Create baseline for future comparisons
            let json = serde_json::to_string_pretty(report)?;
            std::fs::write(baseline_path, json)?;
            println!("Created performance baseline at: {:?}", baseline_path);
            return Ok(());
        }

        let baseline_content = std::fs::read_to_string(baseline_path)?;
        let baseline: BenchmarkReport = serde_json::from_str(&baseline_content)?;

        let regression_threshold = 1.1; // 10% regression threshold

        for result in &report.results {
            if let Some(baseline_result) = baseline.results.iter().find(|r| r.name == result.name) {
                let ratio =
                    result.average_time.as_secs_f64() / baseline_result.average_time.as_secs_f64();

                if ratio > regression_threshold {
                    return Err(TestError::Validation(
                        crate::ValidationError::invalid_setup(format!(
                            "Performance regression in '{}': {}x slower (threshold: {}x)",
                            result.name, ratio, regression_threshold
                        )),
                    ));
                }

                println!("✅ {}: {:.2}x (improvement)", result.name, ratio);
            }
        }

        Ok(())
    }
}

impl Default for CIPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Cross-platform command execution for CI
pub struct CICommandRunner {
    working_dir: PathBuf,
    env_vars: HashMap<String, String>,
}

impl CICommandRunner {
    pub fn new() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env_vars: HashMap::new(),
        }
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = dir;
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn run(&self, command: &str, args: &[&str]) -> Result<String, TestError> {
        let mut cmd = Command::new(command);
        cmd.args(args).current_dir(&self.working_dir);

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(TestError::Async(format!(
                "Command failed: {}\nstdout: {}\nstderr: {}",
                command, stdout, stderr
            )));
        }

        Ok(stdout)
    }

    /// Run cargo test with coverage
    pub fn run_cargo_test_with_coverage(&self, output_path: &PathBuf) -> Result<(), TestError> {
        if cfg!(feature = "ci") {
            // Assume cargo-tarpaulin is available
            self.run(
                "cargo",
                &[
                    "tarpaulin",
                    "--out",
                    "Xml",
                    "--output-dir",
                    &output_path.to_string_lossy(),
                ],
            )?;
        } else {
            // Fallback to regular cargo test
            self.run("cargo", &["test"])?;
        }
        Ok(())
    }

    /// Run cargo bench
    pub fn run_cargo_bench(&self) -> Result<String, TestError> {
        self.run("cargo", &["bench"])
    }
}

impl Default for CICommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_analyzer() {
        let mut analyzer = CoverageAnalyzer::new().with_threshold(80.0);

        analyzer
            .add_file_coverage("test.rs", &[(1, true), (2, false), (3, true)])
            .unwrap();

        assert_eq!(analyzer.overall_coverage(), 66.67);
        assert!(!analyzer.meets_threshold());
    }

    #[test]
    fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new().with_iterations(10).with_warmup(2);

        let result = runner
            .benchmark_sync("test_bench", || {
                std::thread::sleep(Duration::from_millis(1));
            })
            .unwrap();

        assert!(result.average_time > Duration::from_millis(0));
        assert_eq!(result.name, "test_bench");
    }

    #[test]
    fn test_test_reporter() {
        let mut reporter = TestReporter::new();

        let test_cases = vec![
            TestCaseReport {
                name: "test_pass".to_string(),
                status: TestStatus::Passed,
                duration: Duration::from_millis(10),
                error_message: None,
            },
            TestCaseReport {
                name: "test_fail".to_string(),
                status: TestStatus::Failed,
                duration: Duration::from_millis(5),
                error_message: Some("assertion failed".to_string()),
            },
        ];

        let suite = reporter.add_suite("test_suite", test_cases);
        assert_eq!(suite.passed, 1);
        assert_eq!(suite.failed, 1);
    }

    #[test]
    fn test_ci_environment_detection() {
        let env = CIEnvironment::detect();
        // In test environment, this should detect as Unknown or CI
        assert!(matches!(
            env.provider,
            CIProvider::GitHubActions | CIProvider::GitLabCI | CIProvider::Unknown
        ));
    }
}
