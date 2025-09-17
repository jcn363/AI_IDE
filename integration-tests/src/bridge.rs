//! Bridge functions for integration testing
//!
//! This module contains bridge functions that connect different parts of the
//! integration testing framework, such as test execution, result processing,
//! and coordination between test modules.

use crate::types::*;

/// Create a test result with success status
pub fn create_success_result(test_name: &str, duration_ms: u64) -> IntegrationTestResult {
    let mut result = IntegrationTestResult::new(test_name);
    result.success = true;
    result.duration_ms = duration_ms;
    result
}

/// Create a test result with failure status
pub fn create_failure_result(test_name: &str, duration_ms: u64, error: &str) -> IntegrationTestResult {
    let mut result = IntegrationTestResult::new(test_name);
    result.success = false;
    result.duration_ms = duration_ms;
    result.errors.push(error.to_string());
    result
}

/// Create a test result with multiple errors
pub fn create_multi_error_result(test_name: &str, duration_ms: u64, errors: Vec<String>) -> IntegrationTestResult {
    let mut result = IntegrationTestResult::new(test_name);
    result.success = false;
    result.duration_ms = duration_ms;
    result.errors = errors;
    result
}

/// Bridge function to aggregate test results
pub fn aggregate_results(results: Vec<IntegrationTestResult>) -> (usize, usize, u64) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let total_duration = results.iter().map(|r| r.duration_ms).sum();
    (passed, total, total_duration)
}

/// Bridge function to filter results by success status
pub fn filter_results_by_success(results: &[IntegrationTestResult], success: bool) -> Vec<IntegrationTestResult> {
    results.iter()
        .filter(|r| r.success == success)
        .cloned()
        .collect()
}

/// Bridge function to create a summary of test execution
pub fn create_test_summary(results: &[IntegrationTestResult]) -> std::collections::HashMap<String, String> {
    let mut summary = std::collections::HashMap::new();
    let (passed, total, total_duration) = aggregate_results(results.to_vec());

    summary.insert("total_tests".to_string(), total.to_string());
    summary.insert("passed_tests".to_string(), passed.to_string());
    summary.insert("failed_tests".to_string(), (total - passed).to_string());
    summary.insert("total_duration_ms".to_string(), total_duration.to_string());
    summary.insert("success_rate".to_string(), format!("{:.2}%", (passed as f64 / total as f64) * 100.0));

    summary
}