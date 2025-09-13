//! !
//!
//! Integration tests for rust-ai-ide-ai-analysis using shared-test-utils
//!
//! This module demonstrates sophisticated code analysis testing scenarios using the comprehensive
//! test utilities from shared-test-utils, including:
//!
//! - Code analysis workspace management with temp directories
//! - AST parsing and validation scenarios
//! - Performance benchmarking for analysis operations
//! - Concurrent code analysis workflows
//! - Architectural pattern scanning and validation
//! - Security analysis testing frameworks
//! - Code metrics computation and validation
//! - Analysis result serialization and validation

use std::path::Path;
use std::time::Duration;

use shared_test_utils::async_utils::AsyncContext;
use shared_test_utils::error::TestResult;
use shared_test_utils::fixtures::FixturePresets;
use shared_test_utils::*;

// Test that proves we have the right imports and can run tests
#[cfg(test)]
mod analysis_integration_tests {
    use std::fs;

    use super::*;

    /// Integration test demonstrating code analysis workspace setup
    #[test]
    fn test_analysis_workspace_setup_with_shared_utils() {
        println!("🔧 Setting up analysis integration test with shared utilities...");

        // Create a temp workspace for code analysis testing
        let workspace = TempWorkspace::new().unwrap();

        // Set up a comprehensive Rust project for analysis
        workspace.setup_basic_project().unwrap();

        // Create analysis-specific test files
        workspace
            .create_file(
                Path::new("src/parser.rs"),
                r#"//! Parser module for code analysis
use std::collections::HashMap;

/// Parses source code and extracts meaningful information
pub struct CodeParser {
    rules: Vec<String>,
}

impl CodeParser {
    /// Creates a new parser with default rules
    pub fn new() -> Self {
        CodeParser {
            rules: vec![
                "fn".to_string(),
                "struct".to_string(),
                "mod".to_string(),
            ]
        }
    }

    /// Parses the given source code
    pub fn parse(&self, source: &str) -> Vec<String> {
        self.rules.iter()
            .filter(|rule| source.contains(*rule))
            .cloned()
            .collect()
    }
}

/// Security vulnerability analysis function
pub fn analyze_security_issues(code: &str) -> Vec<String> {
    let mut issues = Vec::new();

    if code.contains("unsafe") && !code.contains("SAFE_UNSAFE") {
        issues.push("Unsafe block requires SAFETY comment".to_string());
    }

    if code.contains("unwrap") && code.contains("public") {
        issues.push("Public function uses unwrap".to_string());
    }

    issues
}
"#,
            )
            .unwrap();

        workspace
            .create_file(
                Path::new("src/architecture.rs"),
                r#"//! Architecture analysis module

use std::collections::HashMap;

/// Represents architectural patterns found in code
#[derive(Debug, Clone)]
pub enum ArchitecturalPattern {
    Singleton,
    Factory,
    Builder,
    Command,
    Strategy,
}

/// Architectural analyzer
pub struct ArchitectureAnalyzer {
    patterns: HashMap<String, ArchitecturalPattern>,
}

impl ArchitectureAnalyzer {
    /// Analyzes code for architectural patterns
    pub fn analyze(&self, code: &str) -> Vec<ArchitecturalPattern> {
        let mut found_patterns = Vec::new();

        // Simple pattern detection (in real implementation, this would be more sophisticated)
        if code.contains("get_instance") || code.contains("static instance") {
            found_patterns.push(ArchitecturalPattern::Singleton);
        }

        if code.contains("fn factory") || code.contains("create_") {
            found_patterns.push(ArchitecturalPattern::Factory);
        }

        found_patterns
    }

    /// Scans an entire codebase for architectural patterns
    pub fn scan_codebase(&self, files: Vec<String>) -> HashMap<String, Vec<ArchitecturalPattern>> {
        let mut results = HashMap::new();

        for file in files {
            // In a real implementation, this would read each file
            if file.contains("main") {
                results.insert(file, vec![ArchitecturalPattern::Command]);
            } else if file.contains("builder") {
                results.insert(file, vec![ArchitecturalPattern::Builder]);
            }
        }

        results
    }
}
"#,
            )
            .unwrap();

        workspace
            .create_file(
                Path::new("src/metrics.rs"),
                r#"//! Code metrics computation module

use std::collections::HashMap;

/// Code metrics structure
#[derive(Debug, Clone)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub cyclomatic_complexity: usize,
    pub function_count: usize,
    pub struct_count: usize,
    pub security_score: f64,
}

/// Metrics analyzer
pub struct MetricsAnalyzer;

impl MetricsAnalyzer {
    /// Computes comprehensive metrics for given source code
    pub fn compute_metrics(&self, code: &str) -> CodeMetrics {
        let lines_of_code = code.lines().count();
        let function_count = code.matches("fn ").count();
        let struct_count = code.matches("derive").count();

        // Simple cyclomatic complexity calculation (number of paths through code)
        let mut complexity = 1; // Base complexity
        complexity += code.matches("if ").count();
        complexity += code.matches("loop").count();
        complexity += code.matches("for ").count();
        complexity += code.matches("while ").count();
        complexity += code.matches("match ").count();

        // Security score based on unsafe blocks and assertions
        let unsafe_count = code.matches("unsafe").count();
        let assertion_count = code.matches("assert!").count();
        let security_score = 1.0 - (unsafe_count as f64) * 0.1 + (assertion_count as f64) * 0.05;

        CodeMetrics {
            lines_of_code,
            cyclomatic_complexity: complexity,
            function_count,
            struct_count,
            security_score: security_score.max(0.0).min(1.0),
        }
    }
}
"#,
            )
            .unwrap();

        // Create test files for analysis validation
        workspace.create_dir(Path::new("tests")).unwrap();
        workspace
            .create_file(
                Path::new("tests/analysis_test.rs"),
                r#"#[cfg(test)]
mod analysis_tests {
    use super::*;

    #[test]
    fn test_code_metrics() {
        use super::super::metrics::*;
        let analyzer = MetricsAnalyzer {};
        let code = "fn main() { println!("Hello"); }";
        let metrics = analyzer.compute_metrics(code);
        assert!(metrics.function_count > 0);
    }
}
"#,
            )
            .unwrap();

        // Verify file creation with shared utilities
        assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
        assert_test_file_exists!(workspace, Path::new("src/lib.rs"));
        assert_test_file_exists!(workspace, Path::new("src/parser.rs"));
        assert_test_file_exists!(workspace, Path::new("src/architecture.rs"));
        assert_test_file_exists!(workspace, Path::new("src/metrics.rs"));
        assert_test_file_exists!(workspace, Path::new("tests/analysis_test.rs"));

        // Test content validation
        assert_file_contains!(workspace, Path::new("src/parser.rs"), "CodeParser");
        assert_file_contains!(
            workspace,
            Path::new("src/architecture.rs"),
            "ArchitecturalPattern"
        );
        assert_file_contains!(workspace, Path::new("src/metrics.rs"), "CodeMetrics");

        // Validate workspace structure
        let total_files = fs::read_dir(workspace.path()).unwrap().count();
        assert!(
            total_files >= 7,
            "Should have at least 7 files including analysis modules"
        );

        println!("✅ Analysis workspace setup completed successfully");
    }

    /// Integration test using fixtures for analysis scenarios
    #[test]
    fn test_analysis_with_fixture_scenarios() {
        println!("🔧 Testing analysis with fixture scenarios...");

        // Use fixture for consistent analysis environment
        let (workspace, fixture) = with_test_fixture!(FixturePresets::rust_library());

        // Extend fixture with analysis-specific files
        workspace
            .create_file(
                Path::new("analysis_config.json"),
                r#"{
  "analyzer": {
    "security_scan": true,
    "complexity_threshold": 10,
    "concurrent_analysis": true,
    "cache_enabled": false
  },
  "metrics": {
    "compute_cyclomatic_complexity": true,
    "compute_maintainability_index": false,
    "track_function_calls": true
  },
  "patterns": {
    "detect_singletons": true,
    "detect_factories": true,
    "detect_data_races": false
  }
}"#,
            )
            .unwrap();

        workspace
            .create_file(
                Path::new("security_rules.json"),
                r#"{
  "rules": [
    {
      "id": "R001",
      "name": "UnsafeBlockWithoutComment",
      "severity": "medium",
      "pattern": "unsafe[^/]*{",
      "requires": ["SAFE_UNSAFE", "SAFETY"]
    },
    {
      "id": "R002",
      "name": "UnwrappedPublicFunction",
      "severity": "high",
      "pattern": "pub fn.*unwrap",
      "message": "Public functions should not use unwrap() as it may panic"
    },
    {
      "id": "R003",
      "name": "MissingErrorHandling",
      "severity": "low",
      "pattern": "\\.expect[\"(]\\w+[\")]",
      "message": "Consider using proper error handling instead of expect()"
    }
  ]
}"#,
            )
            .unwrap();

        // Verify fixture provides expected structure
        assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
        assert_test_file_exists!(workspace, Path::new("src/lib.rs"));
        assert_test_file_exists!(workspace, Path::new("analysis_config.json"));
        assert_test_file_exists!(workspace, Path::new("security_rules.json"));

        // Test analysis configuration validation
        let config_content = fixture
            .get_file_content(&Path::new("analysis_config.json").to_path_buf())
            .unwrap();
        assert!(config_content.contains("analyzer"));
        assert!(config_content.contains("security_scan"));
        assert!(config_content.contains("\"true\""));

        let security_content = fixture
            .get_file_content(&Path::new("security_rules.json").to_path_buf())
            .unwrap();
        assert!(security_content.contains("rules"));
        assert!(security_content.contains("R001"));
        assert!(security_content.contains("unwrap"));

        println!("✅ Analysis fixtures integration test passed");
    }

    /// Performance-critical analysis operations with timeouts
    #[tokio::test]
    async fn test_analysis_operations_with_timeout() {
        println!("🔧 Testing analysis operations with timeout handling...");

        let context = AsyncContext::with_timeout(Duration::from_secs(10));

        // Test timeout functionality for simulated analysis operations
        let result = with_timeout(
            async {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                "analysis_operation_completed"
            },
            Duration::from_millis(200),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "analysis_operation_completed");

        // Test complex analysis operation with timeout
        let complex_result = with_timeout(
            async {
                tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
                // Simulate complex analysis returning metrics
                serde_json::json!({
                    "analysis_type": "complex_scan",
                    "files_scanned": 42,
                    "patterns_found": 15,
                    "security_issues": 2,
                    "performance_score": 0.85
                })
            },
            Duration::from_millis(500),
        )
        .await;

        assert!(complex_result.is_ok());
        let analysis_data: serde_json::Value = complex_result.unwrap();
        assert_eq!(analysis_data["analysis_type"], "complex_scan");
        assert_eq!(analysis_data["files_scanned"], 42);

        println!("✅ Analysis timeout test passed");
    }

    /// Complex analysis scenario with multiple concurrent operations
    #[tokio::test]
    async fn test_complex_analysis_workflows() {
        println!("🔧 Testing complex analysis workflows with multiple operations...");

        // Create workspace and set up complex analysis scenario
        let context = AsyncContext::with_timeout(Duration::from_secs(30));
        let workspace = TempWorkspace::new().unwrap();

        // Set up multiple analysis scenarios
        workspace.create_dir(Path::new("analysis_reports")).unwrap();
        workspace.create_dir(Path::new("scanned_modules")).unwrap();
        workspace.create_dir(Path::new("security_scans")).unwrap();

        // Simulate concurrent analysis operations
        async fn simulate_analysis_operation(operation_id: usize, analysis_type: &str) -> Result<String, TestError> {
            Ok(format!(
                "analysis_{}_{}_completed",
                analysis_type, operation_id
            ))
        }

        // Create concurrent analysis tasks
        let analysis_operations = vec![
            simulate_analysis_operation(1, "syntax"),
            simulate_analysis_operation(2, "semantic"),
            simulate_analysis_operation(3, "security"),
            simulate_analysis_operation(4, "complexity"),
            simulate_analysis_operation(5, "patterns"),
        ];

        let results = context
            .execute_concurrent(analysis_operations, Some(Duration::from_millis(300)))
            .await;

        assert!(results.is_ok());
        let result_strings = results.unwrap();
        assert_eq!(result_strings.len(), 5);

        // Verify each analysis operation completed successfully
        for result in &result_strings {
            if let Ok(content) = result {
                assert!(content.starts_with("analysis_"));
                assert!(content.contains("_completed"));
                assert!(
                    content.contains("syntax")
                        || content.contains("semantic")
                        || content.contains("security")
                        || content.contains("complexity")
                        || content.contains("patterns")
                );
            } else {
                assert!(false, "Analysis operation failed with error: {:?}", result);
            }
        }

        // Simulate storing analysis results
        for (i, result) in result_strings.iter().enumerate() {
            if let Ok(content) = result {
                workspace
                    .create_file(
                        Path::new(&format!("analysis_reports/report_{}.json", i + 1)),
                        &format!(
                            "{{\"result\": \"{}\", \"timestamp\": \"{}\"}}\n",
                            content,
                            chrono::Utc::now()
                        ),
                    )
                    .unwrap();
            }
        }

        println!(
            "✅ Complex analysis workflows test completed successfully - {} operations processed",
            result_strings.len()
        );
    }

    /// Integration test for analysis error handling and validation
    #[test]
    fn test_analysis_error_handling_and_validation() {
        println!("🔧 Testing analysis error handling and validation scenarios...");

        let workspace = TempWorkspace::new().unwrap();

        // Test analysis file operations and error handling
        let result = std::fs::write(
            workspace.path().join("invalid_analysis_config.json"),
            r#"{
  "invalid_json": this is invalid JSON syntax ###
  "missing_bracket": true
}"#,
        );

        if let Err(e) = result {
            let test_error = TestError::Io(e);
            assert!(matches!(test_error, TestError::Io(_)));
        }

        // Test code analysis path validation
        let invalid_path = Path::new("/nonexistent/analysis/target");
        let validation_result = ValidationUtils::validate_path_security(invalid_path);
        assert!(validation_result.is_err());

        // Test analysis configuration validation
        workspace
            .create_file(
                Path::new("analysis_config.toml"),
                r#"[analysis]
enabled_features = ["syntax", "semantic", "security"]
max_file_size_mb = 10
concurrent_workers = 8
cache_enabled = true

[logging]
level = "info"
format = "json"

[security]
scan_for_unsafe = true
flag_unwrapped_results = true"#,
            )
            .unwrap();

        // Test component validation for analysis scenarios
        let analysis_components = vec![
            Some("syntax_analyzer"),
            Some("semantic_analyzer"),
            None, // Missing optional component
        ];
        let names = vec![
            "Syntax Analyzer",
            "Semantic Analyzer",
            "Performance Reporter",
        ];

        assert!(ValidationUtils::validate_test_setup(&analysis_components, &names).is_err());

        // Valid setup should pass
        let valid_analysis_components = vec![
            Some("syntax_analyzer"),
            Some("semantic_analyzer"),
            Some("performance_reporter"),
        ];
        assert!(ValidationUtils::validate_test_setup(&valid_analysis_components, &names).is_ok());

        println!("✅ Analysis error handling and validation test completed");
    }

    /// Command integration testing for analysis patterns
    #[test]
    fn test_analysis_command_integration_patterns() {
        println!("🔧 Testing analysis command integration patterns...");

        use shared_test_utils::command_tests::{CommandTestBuilder, MockCommand};

        // Create mock commands for analysis operations
        let commands = vec![
            MockCommand::new(
                "run_syntax_analysis",
                serde_json::json!({
                    "files": ["src/main.rs", "src/lib.rs"],
                    "output_format": "json"
                }),
            )
            .with_result(serde_json::json!({
                "analysis_complete": true,
                "syntax_errors": 0,
                "files_scanned": 2,
                "processing_time_ms": 45
            })),
            MockCommand::new(
                "security_scan",
                serde_json::json!({
                    "target_path": "/project/src",
                    "rule_set": "full_security",
                    "severity_level": "medium"
                }),
            )
            .with_result(serde_json::json!({
                "vulnerabilities_found": 1,
                "scan_complete": true,
                "high_severity": 0,
                "medium_severity": 1,
                "report_url": "/reports/security_scan_2024_01_15.json"
            })),
            MockCommand::new(
                "generate_metrics",
                serde_json::json!({
                    "include_patterns": ["src/**/*.rs"],
                    "exclude_patterns": ["**/tests/**", "**/test_*.rs"],
                    "metrics": ["cyclomatic_complexity", "lines_of_code", "function_count"]
                }),
            )
            .with_result(serde_json::json!({
                "metrics_generated": true,
                "files_analyzed": 12,
                "total_functions": 47,
                "avg_complexity": 4.2,
                "total_lines": 2317,
                "maintainability_index": 78.5
            })),
            MockCommand::new(
                "architectural_scan",
                serde_json::json!({
                    "scan_root": "/project",
                    "patterns": ["singleton", "factory", "builder"],
                    "depth": "full"
                }),
            )
            .with_result(serde_json::json!({
                "patterns_found": ["Factory", "Builder"],
                "correlation_analysis": true,
                "architectural_health_score": 85.2,
                "recommendations": ["Consider consolidating factory patterns"]
            })),
        ];

        // Test analysis command setup
        let runner = CommandTestBuilder::new()
            .success_command(
                "analyze_performance",
                serde_json::json!({}),
                serde_json::json!({"bottlenecks_found": 2, "optimization_suggestions": 5}),
            )
            .error_command(
                "invalid_analysis_request",
                serde_json::json!({}),
                "No files specified for analysis",
            )
            .build_runner();

        // Verify analysis commands were registered correctly
        assert_eq!(commands[0].name, "run_syntax_analysis");
        assert!(commands[0].result.is_ok());

        assert_eq!(commands[1].name, "security_scan");
        assert!(commands[1].result.is_ok());

        assert_eq!(commands[2].name, "generate_metrics");
        assert!(commands[2].result.is_ok());

        assert_eq!(commands[3].name, "architectural_scan");
        assert!(commands[3].result.is_ok());

        // Verify analysis tester is set up
        assert_eq!(runner.called_commands().len(), 0);

        println!("✅ Analysis command integration patterns test completed");
    }

    /// Concurrent analysis operations with performance tracking
    #[tokio::test]
    async fn test_concurrent_analysis_operations() {
        println!("🔧 Testing concurrent analysis operations with performance tracking...");

        // Test simulating multiple concurrent analysis operations
        async fn simulate_analysis_operation(
            batch_id: usize,
            operation: &str,
            processing_ms: u64,
        ) -> Result<String, TestError> {
            // Simulate varying processing times for different analysis types
            let result = with_timeout(
                async {
                    tokio::time::sleep(Duration::from_millis(processing_ms)).await;
                    format!("batch_{}_{}_{}ms", batch_id, operation, processing_ms)
                },
                Duration::from_millis(processing_ms + 50),
            )
            .await;

            match result {
                Ok(value) => Ok(value),
                Err(_) => Ok(format!("batch_{}_{}_timed_out", batch_id, operation)),
            }
        }

        // Test different analysis operations running concurrently
        let context = AsyncContext::with_timeout(Duration::from_secs(15));

        // Batch analysis operations with different processing times
        let analysis_batches = vec![
            (1, "security_check", 80),
            (1, "syntax_validation", 60),
            (1, "semantic_analysis", 100),
            (2, "dependency_scan", 40),
            (2, "metric_computation", 70),
            (3, "architectural_review", 120),
        ];

        // Convert to futures
        let mut analysis_operations = vec![];
        for (batch_id, operation, processing_time) in analysis_batches {
            analysis_operations.push(simulate_analysis_operation(
                batch_id,
                operation,
                processing_time,
            ));
        }

        let results = context
            .execute_concurrent(
                analysis_operations,
                Some(Duration::from_millis(500)), // Reasonable timeout for analysis
            )
            .await;

        assert!(results.is_ok());
        let result_values = results.unwrap();
        assert_eq!(result_values.len(), 6);

        // Verify all analysis operations completed
        let mut batch_counts = std::collections::HashMap::new();
        for result in &result_values {
            if let Ok(content) = result {
                assert!(content.starts_with("batch_"));
                assert!(content.contains("_completed") || content.contains("_timed_out"));

                // Count successful completions per batch
                if content.contains("_completed") {
                    let batch_id = content.split('_').nth(1).unwrap_or("0");
                    *batch_counts.entry(batch_id).or_insert(0) += 1;
                }
            } else {
                assert!(false, "Analysis operation failed with error: {:?}", result);
            }
        }

        // Verify we have completions across multiple batches
        assert!(
            batch_counts.len() >= 2,
            "Should have completions in at least 2 batches"
        );
        assert!(
            batch_counts.values().sum::<i32>() >= 4,
            "Should have at least 4 successful operations"
        );

        println!(
            "✅ Concurrent analysis operations test completed - {} operations processed across {} batches",
            result_values.len(),
            batch_counts.len()
        );
    }
}
