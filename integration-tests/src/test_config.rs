//! Configuration management for integration tests

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Configuration for integration test suites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteConfig {
    pub suite_name: String,
    pub description: String,
    pub timeout_seconds: u64,
    pub enable_parallel: bool,
    pub required_features: Vec<String>,
    pub mock_data_directory: Option<String>,
    pub test_scenarios: Vec<TestScenarioConfig>,
    pub reporting: TestReportingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenarioConfig {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub prerequisites: Vec<String>,
    pub setup_commands: Vec<String>,
    pub cleanup_commands: Vec<String>,
    pub environment_variables: HashMap<String, String>,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_time_seconds: u64,
    pub max_concurrent_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReportingConfig {
    pub enable_detailed_logs: bool,
    pub save_screenshots: bool,
    pub generate_performance_report: bool,
    pub export_test_results: bool,
    pub report_formats: Vec<String>,
}

/// Master configuration for all integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterTestConfig {
    pub global_settings: GlobalSettings,
    pub lsp_tests: LspTestConfig,
    pub ai_ml_tests: AIMLTestConfig,
    pub cargo_tests: CargoTestConfig,
    pub cross_crate_tests: CrossCrateTestConfig,
    pub performance_tests: PerformanceTestConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub enable_all_tests: bool,
    pub parallel_execution: bool,
    pub log_level: String,
    pub cleanup_on_failure: bool,
    pub retry_failed_tests: bool,
    pub max_retries: u32,
    pub report_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspTestConfig {
    pub enabled: bool,
    pub test_server_startup: bool,
    pub test_completion_requests: bool,
    pub test_diagnostic_requests: bool,
    pub test_hover_requests: bool,
    pub test_symbol_search: bool,
    pub test_multi_language_support: bool,
    pub performance_targets: PerformanceTargets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMLTestConfig {
    pub enabled: bool,
    pub test_code_analysis: bool,
    pub test_error_resolution: bool,
    pub test_code_generation: bool,
    pub test_refactoring_suggestions: bool,
    pub test_learning_cycles: bool,
    pub test_model_inference: bool,
    pub quality_thresholds: QualityThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoTestConfig {
    pub enabled: bool,
    pub test_metadata_parsing: bool,
    pub test_dependency_resolution: bool,
    pub test_workspace_analysis: bool,
    pub test_build_processes: bool,
    pub test_unused_variables: bool,
    pub test_cross_target_support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCrateTestConfig {
    pub enabled: bool,
    pub test_interactions: bool,
    pub test_data_flow: bool,
    pub test_api_contracts: bool,
    pub test_resource_sharing: bool,
    pub test_error_propagation: bool,
    pub test_configuration_sharing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestConfig {
    pub enabled: bool,
    pub benchmark_iterations: u32,
    pub enable_memory_profiling: bool,
    pub enable_concurrency_testing: bool,
    pub cpu_usage_threshold: f64,
    pub memory_usage_threshold: u64,
    pub response_time_thresholds: ResponseTimeThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub completion_latency_ms: u64,
    pub diagnostic_time_ms: u64,
    pub hover_response_ms: u32,
    pub symbol_search_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub analysis_accuracy: f64,
    pub suggestion_confidence: f64,
    pub error_detection_rate: f64,
    pub code_generation_effectiveness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeThresholds {
    pub p95_response_time_ms: u64,
    pub p99_response_time_ms: u64,
    pub avg_response_time_ms: u64,
}

impl Default for MasterTestConfig {
    fn default() -> Self {
        Self {
            global_settings: GlobalSettings {
                enable_all_tests: true,
                parallel_execution: false,
                log_level: "info".to_string(),
                cleanup_on_failure: true,
                retry_failed_tests: true,
                max_retries: 3,
                report_directory: "test-reports".to_string(),
            },
            lsp_tests: LspTestConfig {
                enabled: true,
                test_server_startup: true,
                test_completion_requests: true,
                test_diagnostic_requests: true,
                test_hover_requests: true,
                test_symbol_search: true,
                test_multi_language_support: true,
                performance_targets: PerformanceTargets {
                    completion_latency_ms: 100,
                    diagnostic_time_ms: 200,
                    hover_response_ms: 150,
                    symbol_search_ms: 300,
                },
            },
            ai_ml_tests: AIMLTestConfig {
                enabled: true,
                test_code_analysis: true,
                test_error_resolution: true,
                test_code_generation: true,
                test_refactoring_suggestions: true,
                test_learning_cycles: true,
                test_model_inference: true,
                quality_thresholds: QualityThresholds {
                    analysis_accuracy: 0.85,
                    suggestion_confidence: 0.75,
                    error_detection_rate: 0.90,
                    code_generation_effectiveness: 0.80,
                },
            },
            cargo_tests: CargoTestConfig {
                enabled: true,
                test_metadata_parsing: true,
                test_dependency_resolution: true,
                test_workspace_analysis: true,
                test_build_processes: true,
                test_unused_variables: true,
                test_cross_target_support: true,
            },
            cross_crate_tests: CrossCrateTestConfig {
                enabled: true,
                test_interactions: true,
                test_data_flow: true,
                test_api_contracts: true,
                test_resource_sharing: true,
                test_error_propagation: true,
                test_configuration_sharing: true,
            },
            performance_tests: PerformanceTestConfig {
                enabled: true,
                benchmark_iterations: 100,
                enable_memory_profiling: true,
                enable_concurrency_testing: true,
                cpu_usage_threshold: 80.0,
                memory_usage_threshold: 256,
                response_time_thresholds: ResponseTimeThresholds {
                    p95_response_time_ms: 200,
                    p99_response_time_ms: 500,
                    avg_response_time_ms: 100,
                },
            },
        }
    }
}

/// Configuration loader with environment variable overrides
pub struct TestConfigLoader;

impl TestConfigLoader {
    /// Load configuration from file with environment overrides
    pub fn load_config<P: AsRef<Path>>(path: P) -> std::io::Result<MasterTestConfig> {
        let mut config = if path.as_ref().exists() {
            let content = std::fs::read_to_string(path)?;
            serde_json::from_str(&content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        } else {
            MasterTestConfig::default()
        };

        // Apply environment variable overrides
        Self::apply_environment_overrides(&mut config);

        Ok(config)
    }

    /// Save configuration to file
    pub fn save_config<P: AsRef<Path>>(path: P, config: &MasterTestConfig) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Apply environment variable overrides to configuration
    fn apply_environment_overrides(config: &mut MasterTestConfig) {
        if let Ok(value) = std::env::var("RUST_AI_IDE_TEST_PARALLEL") {
            config.global_settings.parallel_execution = value.parse().unwrap_or(false);
        }

        if let Ok(value) = std::env::var("RUST_AI_IDE_TEST_ENABLE_ALL") {
            config.global_settings.enable_all_tests = value.parse().unwrap_or(true);
        }

        if let Ok(value) = std::env::var("RUST_AI_IDE_TEST_LOG_LEVEL") {
            config.global_settings.log_level = value;
        }

        // LSP test overrides
        if let Ok(value) = std::env::var("RUST_AI_IDE_TEST_LSP_COMPLETION_LATENCY") {
            if let Ok(latency) = value.parse() {
                config.lsp_tests.performance_targets.completion_latency_ms = latency;
            }
        }
    }

    /// Create default test directory structure
    pub fn setup_test_directories<P: AsRef<Path>>(base_path: P) -> std::io::Result<()> {
        let base = base_path.as_ref();
        std::fs::create_dir_all(base.join("test-reports"))?;
        std::fs::create_dir_all(base.join("test-data"))?;
        std::fs::create_dir_all(base.join("test-logs"))?;
        std::fs::create_dir_all(base.join("performance-reports"))?;
        Ok(())
    }
}

/// Helper functions for test configuration validation
pub mod validators {
    use super::*;

    pub fn validate_performance_targets(target: &PerformanceTargets) -> Vec<String> {
        let mut errors = Vec::new();

        if target.completion_latency_ms > 1000 {
            errors.push("Completion latency target too high (>1000ms)".to_string());
        }

        if target.diagnostic_time_ms > 2000 {
            errors.push("Diagnostic time target too high (>2000ms)".to_string());
        }

        if target.hover_response_ms > 500 {
            errors.push("Hover response target too high (>500ms)".to_string());
        }

        if target.symbol_search_ms > 1000 {
            errors.push("Symbol search target too high (>1000ms)".to_string());
        }

        errors
    }

    pub fn validate_quality_thresholds(thresholds: &QualityThresholds) -> Vec<String> {
        let mut errors = Vec::new();

        if !(0.0..=1.0).contains(&thresholds.analysis_accuracy) {
            errors.push("Analysis accuracy must be between 0.0 and 1.0".to_string());
        }

        if !(0.0..=1.0).contains(&thresholds.suggestion_confidence) {
            errors.push("Suggestion confidence must be between 0.0 and 1.0".to_string());
        }

        if !(0.0..=1.0).contains(&thresholds.error_detection_rate) {
            errors.push("Error detection rate must be between 0.0 and 1.0".to_string());
        }

        if !(0.0..=1.0).contains(&thresholds.code_generation_effectiveness) {
            errors.push("Code generation effectiveness must be between 0.0 and 1.0".to_string());
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = MasterTestConfig::default();
        assert!(config.global_settings.enable_all_tests);
        assert!(config.lsp_tests.enabled);
        assert!(config.ai_ml_tests.enabled);
        assert_eq!(
            config.lsp_tests.performance_targets.completion_latency_ms,
            100
        );
    }

    #[test]
    fn test_performance_targets_validation() {
        let valid_targets = PerformanceTargets {
            completion_latency_ms: 100,
            diagnostic_time_ms: 200,
            hover_response_ms: 150,
            symbol_search_ms: 300,
        };
        assert!(validators::validate_performance_targets(&valid_targets).is_empty());

        let invalid_targets = PerformanceTargets {
            completion_latency_ms: 2000,
            diagnostic_time_ms: 200,
            hover_response_ms: 150,
            symbol_search_ms: 300,
        };
        assert!(!validators::validate_performance_targets(&invalid_targets).is_empty());
    }

    #[test]
    fn test_quality_thresholds_validation() {
        let valid_thresholds = QualityThresholds {
            analysis_accuracy: 0.85,
            suggestion_confidence: 0.75,
            error_detection_rate: 0.90,
            code_generation_effectiveness: 0.80,
        };
        assert!(validators::validate_quality_thresholds(&valid_thresholds).is_empty());

        let invalid_thresholds = QualityThresholds {
            analysis_accuracy: 1.5,
            suggestion_confidence: 0.75,
            error_detection_rate: 0.90,
            code_generation_effectiveness: 0.80,
        };
        assert!(!validators::validate_quality_thresholds(&invalid_thresholds).is_empty());
    }
}
