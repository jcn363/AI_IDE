//! # Rust AI IDE Integration Tests
//!
//! Comprehensive integration testing framework for validating end-to-end functionality
//! of the Rust AI IDE, including LSP integration, AI/ML workflows, dependency management,
//! cross-crate interactions, and performance regression validation.
//!
//! ## Test Categories
//!
//! ### LSP Integration Tests
//! - Server initialization and lifecycle management
//! - Client-server message protocol validation
//! - Multi-language support and routing
//! - AI-enhanced LSP features (completion, diagnostics, hover)
//!
//! ### AI/ML Workflow Tests
//! - End-to-end code analysis pipelines
//! - ML model inference and learning cycles
//! - Code generation and refactoring workflows
//! - Error resolution and pattern matching
//!
//! ### Dependency and Cargo Integration Tests
//! - Cargo metadata parsing and dependency resolution
//! - Cross-crate symbol resolution
//! - Build dependency graph validation
//! - Unused variable detection and regression tracking
//!
//! ### Cross-Crate Integration Tests
//! - Module interaction validation
//! - Data flow between components
//! - API contract compliance
//! - Resource sharing and coordination
//!
//! ### Performance Regression Tests
//! - AI analysis performance benchmarking
//! - LSP request/response latency measurement
//! - Memory usage tracking and optimization
//! - Concurrent operation throughput testing

pub mod ai_ml_integration;
pub mod cargo_dependency_integration;
pub mod common;
pub mod coverage_analysis;
pub mod cross_crate_integration;
pub mod e2e_user_workflows;
pub mod lsp_integration;
pub mod performance_gates;
pub mod performance_regression;
pub mod quality_gates;
pub mod ui_test_scenarios;
pub mod ui_testing;

/// Test execution configuration and helpers
pub mod test_config;
pub mod test_runner;

/// Re-export commonly used utilities
pub use common::*;
pub use test_config::*;
pub use test_runner::*;

/// Integration test result types
#[derive(Debug, Clone, serde::Serialize)]
pub struct IntegrationTestResult {
    pub test_name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub errors: Vec<String>,
    pub metrics: std::collections::HashMap<String, String>,
}

impl IntegrationTestResult {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            success: false,
            duration_ms: 0,
            errors: Vec::new(),
            metrics: std::collections::HashMap::new(),
        }
    }

    pub fn add_metric(&mut self, key: &str, value: String) {
        self.metrics.insert(key.to_string(), value);
    }
}

/// Global test configuration for integration suite
#[derive(Debug, Clone)]
pub struct GlobalTestConfig {
    pub enable_full_integration: bool,
    pub enable_performance_benchmarks: bool,
    pub enable_cross_crate_tests: bool,
    pub timeout_seconds: u64,
    pub parallel_execution: bool,
    pub report_detailed_metrics: bool,
}

impl Default for GlobalTestConfig {
    fn default() -> Self {
        Self {
            enable_full_integration: true,
            enable_performance_benchmarks: true,
            enable_cross_crate_tests: true,
            timeout_seconds: 300,
            parallel_execution: false,
            report_detailed_metrics: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_framework_setup() {
        // Basic smoke test for integration framework
        let config = GlobalTestConfig::default();

        assert_eq!(config.timeout_seconds, 300);
        assert!(config.enable_full_integration);
        assert!(config.report_detailed_metrics);
    }

    #[tokio::test]
    async fn test_test_result_creation() {
        let result = IntegrationTestResult::new("smoke_test");

        assert_eq!(result.test_name, "smoke_test");
        assert!(!result.success);
        assert_eq!(result.errors.len(), 0);
    }
}
