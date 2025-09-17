//! Common types for integration testing
//!
//! This module contains shared types used across the integration test framework.
//! These types define the structure for test results and configuration.

/// Integration test result types
#[derive(Debug, Clone, serde::Serialize)]
pub struct IntegrationTestResult {
    pub test_name:   String,
    pub success:     bool,
    pub duration_ms: u64,
    pub errors:      Vec<String>,
    pub metrics:     std::collections::HashMap<String, String>,
}

impl IntegrationTestResult {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name:   test_name.to_string(),
            success:     false,
            duration_ms: 0,
            errors:      Vec::new(),
            metrics:     std::collections::HashMap::new(),
        }
    }

    pub fn add_metric(&mut self, key: &str, value: String) {
        self.metrics.insert(key.to_string(), value);
    }
}

/// Global test configuration for integration suite
#[derive(Debug, Clone)]
pub struct GlobalTestConfig {
    pub enable_full_integration:       bool,
    pub enable_performance_benchmarks: bool,
    pub enable_cross_crate_tests:      bool,
    pub timeout_seconds:               u64,
    pub parallel_execution:            bool,
    pub report_detailed_metrics:       bool,
}

impl Default for GlobalTestConfig {
    fn default() -> Self {
        Self {
            enable_full_integration:       true,
            enable_performance_benchmarks: true,
            enable_cross_crate_tests:      true,
            timeout_seconds:               300,
            parallel_execution:            false,
            report_detailed_metrics:       true,
        }
    }
}