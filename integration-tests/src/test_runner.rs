//! Comprehensive test runner for integration tests

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use rust_ai_ide_errors::RustAIError;
use shared_test_utils::IntegrationContext;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use crate::common::EnhancedIntegrationTestRunner;
use crate::test_config::*;
use crate::{GlobalTestConfig, IntegrationTestResult};

/// Test suite runner enum to avoid dyn compatibility issues
#[derive(Clone)]
pub enum TestSuiteRunnerImpl {
    LSP(crate::lsp_integration::LSPIntegrationTestRunner),
    AIML(crate::ai_ml_integration::AIMLIntegrationTestRunner),
}

/// Test execution statistics
#[derive(Debug, Clone)]
pub struct TestExecutionStats {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub total_duration_ms: u64,
    pub avg_duration_ms: u64,
    pub max_duration_ms: u64,
    pub min_duration_ms: u64,
}

impl TestExecutionStats {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            skipped_tests: 0,
            total_duration_ms: 0,
            avg_duration_ms: 0,
            max_duration_ms: u64::MIN,
            min_duration_ms: u64::MAX,
        }
    }

    pub fn add_result(&mut self, result: &IntegrationTestResult) {
        self.total_tests += 1;
        self.total_duration_ms += result.duration_ms;

        if result.success {
            self.passed_tests += 1;
        } else {
            self.failed_tests += 1;
        }

        self.max_duration_ms = self.max_duration_ms.max(result.duration_ms);
        self.min_duration_ms = self.min_duration_ms.min(result.duration_ms);

        if self.total_tests > 0 {
            self.avg_duration_ms = self.total_duration_ms / self.total_tests as u64;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        }
    }
}

/// Comprehensive integration test runner
pub struct ComprehensiveTestRunner {
    config: MasterTestConfig,
    global_config: GlobalTestConfig,
    test_runners: Vec<TestSuiteRunnerImpl>,
    semaphore: Arc<Semaphore>,
    stats: TestExecutionStats,
}

impl ComprehensiveTestRunner {
    pub fn new() -> Result<Self, RustAIError> {
        let config = MasterTestConfig::default();
        let global_config = GlobalTestConfig::default();

        Ok(Self {
            config,
            global_config,
            test_runners: Vec::new(),
            semaphore: Arc::new(Semaphore::new(if global_config.parallel_execution {
                4
            } else {
                1
            })),
            stats: TestExecutionStats::new(),
        })
    }

    /// Load configuration from file
    pub fn with_config_path<P: AsRef<std::path::Path>>(
        mut self,
        path: P,
    ) -> Result<Self, RustAIError> {
        let loaded_config = TestConfigLoader::load_config(path)?;
        self.config = loaded_config;
        Ok(self)
    }

    /// Register a test suite runner
    pub fn register_test_suite(&mut self, runner: TestSuiteRunnerImpl) {
        self.test_runners.push(runner);
    }

    /// Run all enabled test suites
    pub async fn run_all_suites(&mut self) -> Result<Vec<IntegrationTestResult>, RustAIError> {
        let mut all_results = Vec::new();

        for runner in &self.test_runners {
            let suite_name = runner.suite_name();
            if self.is_suite_enabled(&suite_name) {
                let suite_results = self.run_test_suite(runner.as_ref()).await?;
                all_results.extend(suite_results);
            }
        }

        Ok(all_results)
    }

    /// Run a specific test suite
    pub async fn run_test_suite(
        &self,
        runner: &TestSuiteRunnerImpl,
    ) -> Result<Vec<IntegrationTestResult>, RustAIError> {
        let permit = self.semaphore.acquire().await?;
        let start_time = std::time::Instant::now();

        let results = runner.run_test_suite().await;

        drop(permit);

        let duration = start_time.elapsed();
        tracing::info!(
            "Test suite '{}' completed in {:.2}s",
            runner.suite_name(),
            duration.as_secs_f64()
        );

        results
    }

    /// Check if a test suite is enabled
    fn is_suite_enabled(&self, suite_name: &str) -> bool {
        match suite_name {
            "lsp" => self.config.global_settings.enable_all_tests && self.config.lsp_tests.enabled,
            "ai_ml" => {
                self.config.global_settings.enable_all_tests && self.config.ai_ml_tests.enabled
            }
            "cargo" => {
                self.config.global_settings.enable_all_tests && self.config.cargo_tests.enabled
            }
            "cross_crate" => {
                self.config.global_settings.enable_all_tests
                    && self.config.cross_crate_tests.enabled
            }
            "performance" => {
                self.config.global_settings.enable_all_tests
                    && self.config.performance_tests.enabled
            }
            _ => false,
        }
    }

    /// Get current execution statistics
    pub fn stats(&self) -> &TestExecutionStats {
        &self.stats
    }

    /// Generate configuration for enhanced test runner
    pub fn create_integration_config(&self) -> shared_test_utils::IntegrationContext {
        let mut config = IntegrationConfig {
            cleanup_on_exit: self.config.global_settings.cleanup_on_failure,
            isolated_tests: true,
            enable_logging: self.config.global_settings.log_level == "debug"
                || self.config.global_settings.log_level == "trace",
            timeout_seconds: self
                .config
                .global_settings
                .report_directory
                .parse()
                .unwrap_or(300),
        };

        if self.config.global_settings.parallel_execution {
            config.timeout_seconds = config.timeout_seconds * 2; // Allow more time for parallel
                                                                 // execution
        }

        config
    }
}

/// Trait for test suite runners
#[async_trait]
pub trait TestSuiteRunner: Send + Sync {
    fn suite_name(&self) -> &'static str;
    async fn run_test_suite(&self) -> Result<Vec<IntegrationTestResult>, RustAIError>;

    /// Get list of test names in this suite
    fn test_names(&self) -> Vec<String>;

    /// Check if a specific test is enabled
    fn is_test_enabled(&self, test_name: &str) -> bool;

    /// Get prerequisites for this suite
    fn prerequisites(&self) -> Vec<String>;
}

/// Implement TestSuiteRunner for the enum to avoid dyn compatibility issues
#[async_trait]
impl TestSuiteRunner for TestSuiteRunnerImpl {
    fn suite_name(&self) -> &'static str {
        match self {
            TestSuiteRunnerImpl::LSP(runner) => runner.suite_name(),
            TestSuiteRunnerImpl::AIML(runner) => runner.suite_name(),
        }
    }

    async fn run_test_suite(&self) -> Result<Vec<IntegrationTestResult>, RustAIError> {
        match self {
            TestSuiteRunnerImpl::LSP(runner) => runner.run_test_suite().await,
            TestSuiteRunnerImpl::AIML(runner) => runner.run_test_suite().await,
        }
    }

    fn test_names(&self) -> Vec<String> {
        match self {
            TestSuiteRunnerImpl::LSP(runner) => runner.test_names(),
            TestSuiteRunnerImpl::AIML(runner) => runner.test_names(),
        }
    }

    fn is_test_enabled(&self, test_name: &str) -> bool {
        match self {
            TestSuiteRunnerImpl::LSP(runner) => runner.is_test_enabled(test_name),
            TestSuiteRunnerImpl::AIML(runner) => runner.is_test_enabled(test_name),
        }
    }

    fn prerequisites(&self) -> Vec<String> {
        match self {
            TestSuiteRunnerImpl::LSP(runner) => runner.prerequisites(),
            TestSuiteRunnerImpl::AIML(runner) => runner.prerequisites(),
        }
    }
}

/// Helper to execute tests with retry logic
pub async fn execute_with_retry<F, Fut, T>(
    test_fn: F,
    test_name: &str,
    max_retries: u32,
) -> IntegrationTestResult
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, RustAIError>>,
{
    let mut result = IntegrationTestResult::new(test_name);
    let start_time = std::time::Instant::now();

    for attempt in 1..=max_retries + 1 {
        let attempt_start = std::time::Instant::now();
        let test_future = test_fn();

        match tokio::time::timeout(
            std::time::Duration::from_secs(60), // 60 second timeout per attempt
            test_future,
        )
        .await
        {
            Ok(Ok(_)) => {
                result.success = true;
                result.duration_ms = attempt_start.elapsed().as_millis() as u64;
                result.add_metric("attempts", attempt.to_string());
                break;
            }
            Ok(Err(e)) => {
                if attempt <= max_retries {
                    result
                        .errors
                        .push(format!("Attempt {} failed: {}", attempt, e));
                    tokio::time::sleep(std::time::Duration::from_millis(500 * attempt as u64))
                        .await;
                } else {
                    result.errors.push(format!(
                        "Test failed after {} attempts: {}",
                        max_retries + 1,
                        e
                    ));
                }
            }
            Err(_) => {
                if attempt <= max_retries {
                    result.errors.push(format!("Attempt {} timed out", attempt));
                    tokio::time::sleep(std::time::Duration::from_millis(1000 * attempt as u64))
                        .await;
                } else {
                    result
                        .errors
                        .push("Test timed out after all retry attempts".to_string());
                }
            }
        }
    }

    if !result.success && result.errors.is_empty() {
        result
            .errors
            .push("Test failed with unknown error".to_string());
    }

    result
}

/// Helper to run tests in parallel with limited concurrency
pub async fn run_parallel_tests<F, Fut>(
    tests: Vec<F>,
    max_concurrency: usize,
    test_names: Vec<String>,
) -> Vec<IntegrationTestResult>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = IntegrationTestResult> + Send + 'static,
{
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut handles: Vec<JoinHandle<IntegrationTestResult>> = Vec::new();

    for (test_fn, test_name) in tests.into_iter().zip(test_names) {
        let sem_clone = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            test_fn().await
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    results
}

/// Test result reporter
pub struct TestResultReporter {
    results: Vec<IntegrationTestResult>,
    start_time: std::time::Instant,
    output_format: OutputFormat,
}

#[derive(Clone)]
pub enum OutputFormat {
    Standard,
    Json,
    Junit,
    Html,
}

impl TestResultReporter {
    pub fn new(output_format: OutputFormat) -> Self {
        Self {
            results: Vec::new(),
            start_time: std::time::Instant::now(),
            output_format,
        }
    }

    pub fn add_result(&mut self, result: IntegrationTestResult) {
        self.results.push(result);
    }

    pub fn add_results(&mut self, mut results: Vec<IntegrationTestResult>) {
        self.results.append(&mut results);
    }

    /// Generate report in specified format
    pub fn generate_report(&self) -> String {
        match self.output_format {
            OutputFormat::Standard => self.generate_standard_report(),
            OutputFormat::Json => self.generate_json_report(),
            OutputFormat::Junit => self.generate_junit_report(),
            OutputFormat::Html => self.generate_html_report(),
        }
    }

    fn generate_standard_report(&self) -> String {
        let total_tests = self.results.len();
        let passed = self.results.iter().filter(|r| r.success).count();
        let failed = self.results.iter().filter(|r| !r.success).count();
        let total_duration = self.start_time.elapsed();

        let mut report = format!(
            "=== Integration Test Results ===\nTotal Tests: {}\nPassed: {} ({:.1}%)\nFailed: {}\nTotal Duration: \
             {:.2}s\n\n",
            total_tests,
            passed,
            if total_tests > 0 {
                (passed as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            },
            failed,
            total_duration.as_secs_f64()
        );

        if !self.results.iter().all(|r| r.success) {
            report.push_str("=== Failed Tests ===\n");
            for result in &self.results {
                if !result.success {
                    report.push_str(&format!(
                        "❌ {} ({}ms)\n",
                        result.test_name, result.duration_ms
                    ));
                    for error in &result.errors {
                        report.push_str(&format!("   {}\n", error));
                    }
                    report.push_str("\n");
                }
            }
        }

        report.push_str("=== Passed Tests ===\n");
        for result in &self.results {
            if result.success {
                report.push_str(&format!(
                    "✅ {} ({}ms)\n",
                    result.test_name, result.duration_ms
                ));
            }
        }

        report
    }

    fn generate_json_report(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "summary": {
                "total_tests": self.results.len(),
                "passed": self.results.iter().filter(|r| r.success).count(),
                "failed": self.results.iter().filter(|r| !r.success).count(),
                "total_duration_ms": self.start_time.elapsed().as_millis(),
                "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
            },
            "results": self.results
        }))
        .unwrap_or_default()
    }

    fn generate_junit_report(&self) -> String {
        let total_duration = self.start_time.elapsed().as_millis() as f64 / 1000.0;

        let mut xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites time=\"{:.3}\" tests=\"{}\" failures=\"{}\">\n",
            total_duration,
            self.results.len(),
            self.results.iter().filter(|r| !r.success).count()
        );

        for result in &self.results {
            xml.push_str(&format!(
                "  <testcase name=\"{}\" time=\"{:.3}\" classname=\"integration-tests\">\n",
                result.test_name,
                result.duration_ms as f64 / 1000.0
            ));

            if !result.success {
                xml.push_str("    <failure message=\"Test failed\">\n");
                xml.push_str("      <![CDATA[\n");
                for error in &result.errors {
                    xml.push_str(&format!("        {}\n", error));
                }
                xml.push_str("      ]]>\n");
                xml.push_str("    </failure>\n");
            }

            xml.push_str("  </testcase>\n");
        }

        xml.push_str("</testsuites>\n");
        xml
    }

    fn generate_html_report(&self) -> String {
        let passed = self.results.iter().filter(|r| r.success).count();
        let failed = self.results.iter().filter(|r| !r.success).count();

        format!(
            "<!DOCTYPE html>\n<html>\n<head>\n<title>Integration Test Results</title>\n<style>\nbody {{ font-family: \
             Arial, sans-serif; margin: 40px; }}\n.summary {{ background: #f0f0f0; padding: 20px; border-radius: 5px; \
             }}\n.passed {{ color: green; }}\n.failed {{ color: red; }}\n.test {{ margin: 10px 0; padding: 10px; \
             border-bottom: 1px solid #ccc; }}\n</style>\n</head>\n<body>\n<h1>Integration Test Results</h1>\n<div \
             class=\"summary\">\n<h2>Summary</h2>\n<p>Total Tests: {}</p>\n<p class=\"passed\">Passed: {}</p>\n<p \
             class=\"failed\">Failed: {}</p>\n</div>\n<h2>Detailed Results</h2>\n<div class=\"results\">\n",
            self.results.len(),
            passed,
            failed
        ) + &self
            .results
            .iter()
            .map(|result| {
                format!(
                    "<div class=\"test {}\">\n<h3>{}</h3>\n<p>Duration: {}ms</p>\n{}</div>",
                    if result.success { "passed" } else { "failed" },
                    result.test_name,
                    result.duration_ms,
                    if result.success {
                        "<p>✅ Passed</p>".to_string()
                    } else {
                        format!("<p>❌ Failed: {}</p>", result.errors.join(", "))
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n</div>\n</body>\n</html>"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_calculation() {
        let mut stats = TestExecutionStats::new();
        let mut result1 = IntegrationTestResult::new("test1");
        result1.success = true;
        result1.duration_ms = 100;
        stats.add_result(&result1);

        let mut result2 = IntegrationTestResult::new("test2");
        result2.success = false;
        result2.duration_ms = 200;
        stats.add_result(&result2);

        assert_eq!(stats.total_tests, 2);
        assert_eq!(stats.passed_tests, 1);
        assert_eq!(stats.failed_tests, 1);
        assert_eq!(stats.max_duration_ms, 200);
        assert_eq!(stats.min_duration_ms, 100);
        assert_eq!(stats.success_rate(), 50.0);
    }

    #[test]
    fn test_comprehensive_runner_creation() {
        let runner = ComprehensiveTestRunner::new().unwrap();
        assert!(runner.config.global_settings.enable_all_tests);
        assert!(!runner.config.global_settings.parallel_execution);
    }

    #[test]
    fn test_standard_report_generation() {
        let mut reporter = TestResultReporter::new(OutputFormat::Standard);

        for i in 0..5 {
            let mut result = IntegrationTestResult::new(&format!("test{}", i));
            result.success = i % 2 == 0; // Alternate pass/fail
            result.duration_ms = 100 * (i + 1);
            reporter.add_result(result);
        }

        let report = reporter.generate_report();
        assert!(report.contains("Total Tests: 5"));
        assert!(report.contains("Passed: 3"));
        assert!(report.contains("Failed: 2"));
    }

    #[test]
    fn test_json_report_generation() {
        let mut reporter = TestResultReporter::new(OutputFormat::Json);

        let mut result = IntegrationTestResult::new("test1");
        result.success = true;
        result.duration_ms = 150;
        reporter.add_result(result);

        let report = reporter.generate_report();
        assert!(report.contains("\"total_tests\":1"));
        assert!(report.contains("\"passed\":1"));
        assert!(report.contains("\"failed\":0"));
    }
}
