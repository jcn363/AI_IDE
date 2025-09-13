//! # Quality Gates for CI/CD Pipelines
//!
//! Comprehensive quality gate system that integrates performance monitoring,
//! coverage analysis, UI testing, and E2E workflows into CI/CD pipelines.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rust_ai_ide_errors::RustAIError;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::coverage_analysis::*;
use crate::e2e_user_workflows::*;
use crate::performance_gates::*;
use crate::ui_testing::*;
use crate::{GlobalTestConfig, IntegrationTestResult};

/// Main quality gate orchestrator
#[derive(Debug)]
pub struct QualityGateOrchestrator {
    ui_framework:        UITestFramework,
    e2e_runner:          E2EWorkflowRunner,
    performance_checker: PerformanceGateChecker,
    coverage_analyzer:   CoverageAnalyzer,
    gate_config:         QualityGateConfig,
    execution_history:   Vec<GateExecution>,
    current_execution:   Option<GateExecution>,
}

#[derive(Debug, Clone)]
pub struct QualityGateConfig {
    pub enable_all_gates:         bool,
    pub enable_performance_gates: bool,
    pub enable_coverage_gates:    bool,
    pub enable_ui_gates:          bool,
    pub enable_e2e_gates:         bool,
    pub strict_mode:              bool,
    pub fail_fast:                bool,
    pub max_execution_time:       Duration,
    pub gate_thresholds:          GateThresholds,
    pub notification_settings:    NotificationSettings,
}

#[derive(Debug, Clone)]
pub struct GateThresholds {
    pub min_coverage_percentage:            f64,
    pub max_performance_regression_percent: f64,
    pub max_ui_test_failures:               u32,
    pub max_e2e_test_failures:              u32,
    pub max_execution_time_percent:         f64,
}

#[derive(Debug, Clone)]
pub struct NotificationSettings {
    pub slack_webhook:       Option<String>,
    pub email_recipients:    Vec<String>,
    pub notification_levels: Vec<NotificationLevel>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct GateExecution {
    pub id:         String,
    pub start_time: DateTime<Utc>,
    pub end_time:   Option<DateTime<Utc>>,
    pub status:     ExecutionStatus,
    pub results:    HashMap<String, QualityGateResult>,
    pub summary:    Option<GateExecutionSummary>,
    pub metadata:   HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Aborted,
}

#[derive(Debug, Clone)]
pub struct QualityGateResult {
    pub gate_name:      String,
    pub gate_type:      GateType,
    pub status:         GateStatus,
    pub score:          f64, // 0.0 to 1.0
    pub key_metrics:    Vec<GateMetric>,
    pub violations:     Vec<GateViolation>,
    pub execution_time: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GateType {
    Performance,
    Coverage,
    UI,
    E2E,
    Security,
    Dependencies,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GateStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct GateMetric {
    pub name:      String,
    pub value:     f64,
    pub unit:      String,
    pub threshold: Option<f64>,
    pub status:    MetricStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricStatus {
    Good,
    Warning,
    Critical,
    Failed,
}

#[derive(Debug, Clone)]
pub struct GateViolation {
    pub rule_name:       String,
    pub description:     String,
    pub severity:        ViolationSeverity,
    pub impact:          f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct GateExecutionSummary {
    pub total_gates:          usize,
    pub passed_gates:         usize,
    pub warning_gates:        usize,
    pub failed_gates:         usize,
    pub skipped_gates:        usize,
    pub overall_score:        f64,
    pub total_execution_time: Duration,
    pub breaches:             Vec<String>,
    pub recommendations:      Vec<String>,
}

/// Integration with CI/CD systems
#[derive(Debug, Clone)]
pub struct CICDIntegration {
    pub git_ref:            String,
    pub branch:             String,
    pub commit_hash:        String,
    pub build_number:       Option<String>,
    pub pull_request:       Option<PullRequestInfo>,
    pub test_environment:   TestEnvironment,
    pub should_block_merge: bool,
}

#[derive(Debug, Clone)]
pub struct PullRequestInfo {
    pub number:        u32,
    pub author:        String,
    pub reviewers:     Vec<String>,
    pub target_branch: String,
}

#[derive(Debug, Clone)]
pub enum TestEnvironment {
    Local,
    CI,
    Nightly,
    Release,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            enable_all_gates:         true,
            enable_performance_gates: true,
            enable_coverage_gates:    true,
            enable_ui_gates:          true,
            enable_e2e_gates:         true,
            strict_mode:              false,
            fail_fast:                false,
            max_execution_time:       Duration::from_secs(1800), // 30 minutes
            gate_thresholds:          GateThresholds::default(),
            notification_settings:    NotificationSettings::default(),
        }
    }
}

impl Default for GateThresholds {
    fn default() -> Self {
        Self {
            min_coverage_percentage:            80.0,
            max_performance_regression_percent: 5.0,
            max_ui_test_failures:               3,
            max_e2e_test_failures:              1,
            max_execution_time_percent:         150.0, // 150% of baseline
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            slack_webhook:       None,
            email_recipients:    vec![],
            notification_levels: vec![NotificationLevel::Error, NotificationLevel::Critical],
        }
    }
}

impl QualityGateOrchestrator {
    pub fn new() -> Self {
        Self {
            ui_framework:        UITestFramework::new(),
            e2e_runner:          E2EWorkflowRunner::new(),
            performance_checker: PerformanceGateChecker::new(),
            coverage_analyzer:   CoverageAnalyzer::new(),
            gate_config:         QualityGateConfig::default(),
            execution_history:   vec![],
            current_execution:   None,
        }
    }

    /// Configure the quality gate orchestrator
    pub fn with_config(&mut self, config: QualityGateConfig) -> &mut Self {
        self.gate_config = config;
        self
    }

    /// Execute all enabled quality gates
    pub async fn execute_all_gates(
        &mut self,
        cicd_info: &CICDIntegration,
    ) -> Result<GateExecutionSummary, RustAIError> {
        let execution_id = format!(
            "{}_{}_{}",
            cicd_info.commit_hash.chars().take(8).collect::<String>(),
            chron::Utc::now().timestamp(),
            rand::random::<u32>()
        );

        let start_time = Utc::now();
        let mut execution = GateExecution {
            id: execution_id.clone(),
            start_time,
            end_time: None,
            status: ExecutionStatus::Running,
            results: HashMap::new(),
            summary: None,
            metadata: HashMap::new(),
        };

        self.current_execution = Some(execution);
        let mut results = HashMap::new();

        // Execute gates based on configuration
        if self.gate_config.enable_performance_gates && self.gate_config.enable_all_gates {
            let perf_results = self.execute_performance_gates().await?;
            results.extend(perf_results);
        }

        if self.gate_config.enable_coverage_gates && self.gate_config.enable_all_gates {
            let coverage_results = self.execute_coverage_gates().await?;
            results.extend(coverage_results);
        }

        if self.gate_config.enable_ui_gates && self.gate_config.enable_all_gates {
            let ui_results = self.execute_ui_gates().await?;
            results.extend(ui_results);
        }

        if self.gate_config.enable_e2e_gates && self.gate_config.enable_all_gates {
            let e2e_results = self.execute_e2e_gates().await?;
            results.extend(e2e_results);
        }

        let summary = self.create_execution_summary(&results);
        let end_time = Utc::now();

        // Update execution
        let mut execution = self.current_execution.take().unwrap();
        execution.end_time = Some(end_time);
        execution.results = results;
        execution.summary = Some(summary.clone());
        execution.status = ExecutionStatus::Completed;
        execution.metadata.insert(
            "total_execution_time_ms".to_string(),
            (end_time - start_time).num_milliseconds().to_string(),
        );
        execution
            .metadata
            .insert("cicd_branch".to_string(), cicd_info.branch.clone());
        execution
            .metadata
            .insert("commit_hash".to_string(), cicd_info.commit_hash.clone());

        self.execution_history.push(execution);

        // Send notifications if configured
        if !summary.breaches.is_empty() {
            self.send_notifications(&summary, cicd_info).await?;
        }

        Ok(summary)
    }

    /// Execute performance gates
    async fn execute_performance_gates(&mut self) -> Result<HashMap<String, QualityGateResult>, RustAIError> {
        let mut results = HashMap::new();
        let gate_results = self.performance_checker.execute_gates().await?;

        for gate_result in gate_results {
            let status = match gate_result.status {
                GateCheckStatus::Passed => GateStatus::Passed,
                GateCheckStatus::Warning => GateStatus::Warning,
                GateCheckStatus::Failed => GateStatus::Failed,
                GateCheckStatus::Inconclusive => GateStatus::Warning,
            };

            let score = match status {
                GateStatus::Passed => 1.0,
                GateStatus::Warning => 0.7,
                GateStatus::Failed => 0.3,
                GateStatus::Skipped => 0.0,
            };

            let key_metrics = vec![GateMetric {
                name:      "violation_count".to_string(),
                value:     gate_result.violations.len() as f64,
                unit:      "count".to_string(),
                threshold: Some(0.0),
                status:    if gate_result.violations.is_empty() {
                    MetricStatus::Good
                } else {
                    MetricStatus::Critical
                },
            }];

            let violations: Vec<GateViolation> = gate_result
                .violations
                .into_iter()
                .map(|v| GateViolation {
                    rule_name:       format!("{}", v.metric_name),
                    description:     v.description,
                    severity:        Self::convert_performance_severity(&v.severity),
                    impact:          Self::calculate_violation_impact(&v.severity),
                    recommendations: vec![format!(
                        "Address {}.{} performance issue",
                        v.metric_name, v.violation_type
                    )],
                })
                .collect();

            let gate_name = format!("performance_{}", gate_result.gate);
            results.insert(gate_name.clone(), QualityGateResult {
                gate_name,
                gate_type: GateType::Performance,
                status,
                score,
                key_metrics,
                violations,
                execution_time: Duration::from_millis(100), // Placeholder
            });
        }

        Ok(results)
    }

    /// Execute coverage gates
    async fn execute_coverage_gates(&mut self) -> Result<HashMap<String, QualityGateResult>, RustAIError> {
        let mut results = HashMap::new();

        let coverage_report = self
            .coverage_analyzer
            .generate_coverage_report()
            .map_err(|_| RustAIError::ConfigurationError("Coverage analysis failed".to_string()))?;

        let status = match coverage_report.gate_status {
            CoverageGateStatus::Passed => GateStatus::Passed,
            CoverageGateStatus::Warning => GateStatus::Warning,
            CoverageGateStatus::Failed(_) => GateStatus::Failed,
        };

        let score = match status {
            GateStatus::Passed => 1.0,
            GateStatus::Warning => 0.8,
            GateStatus::Failed => 0.4,
            GateStatus::Skipped => 0.0,
        };

        let key_metrics = vec![GateMetric {
            name:      "overall_coverage".to_string(),
            value:     coverage_report.snapshot.coverage_data.overall_percentage,
            unit:      "%".to_string(),
            threshold: Some(self.gate_config.gate_thresholds.min_coverage_percentage),
            status:    if coverage_report.snapshot.coverage_data.overall_percentage >= 80.0 {
                MetricStatus::Good
            } else if coverage_report.snapshot.coverage_data.overall_percentage >= 60.0 {
                MetricStatus::Warning
            } else {
                MetricStatus::Critical
            },
        }];

        let violations = coverage_report
            .recommendations
            .iter()
            .filter(|r| matches!(r.priority, Priority::High))
            .map(|r| GateViolation {
                rule_name:       "coverage_gap".to_string(),
                description:     r.description.clone(),
                severity:        Self::convert_priority_severity(&r.priority),
                impact:          r.estimated_impact,
                recommendations: vec![r.description.clone()],
            })
            .collect();

        results.insert("coverage_gate".to_string(), QualityGateResult {
            gate_name: "coverage_gate".to_string(),
            gate_type: GateType::Coverage,
            status,
            score,
            key_metrics,
            violations,
            execution_time: Duration::from_millis(100),
        });

        Ok(results)
    }

    /// Execute UI gates
    async fn execute_ui_gates(&mut self) -> Result<HashMap<String, QualityGateResult>, RustAIError> {
        let mut results = HashMap::new();

        // Add some predefined scenarios
        self.ui_framework
            .add_scenario(crate::ui_test_scenarios::UITestScenarios::app_loading_scenario());
        self.ui_framework
            .add_scenario(crate::ui_test_scenarios::UITestScenarios::file_operations_scenario());

        let ui_reports = self.ui_framework.execute_all_scenarios().await?;
        let failed_tests = ui_reports.iter().filter(|r| !r.success).count();

        let status = if failed_tests == 0 {
            GateStatus::Passed
        } else if failed_tests <= self.gate_config.gate_thresholds.max_ui_test_failures as usize {
            GateStatus::Warning
        } else {
            GateStatus::Failed
        };

        let score = 1.0 - (failed_tests as f64 / ui_reports.len() as f64);

        let key_metrics = vec![GateMetric {
            name:      "ui_test_pass_rate".to_string(),
            value:     score * 100.0,
            unit:      "%".to_string(),
            threshold: Some(90.0),
            status:    if score >= 0.95 {
                MetricStatus::Good
            } else if score >= 0.85 {
                MetricStatus::Warning
            } else {
                MetricStatus::Critical
            },
        }];

        let violations = ui_reports
            .iter()
            .filter(|r| !r.success)
            .map(|r| GateViolation {
                rule_name:       format!("ui_test_{}", r.scenario_name),
                description:     format!("UI test '{}' failed", r.scenario_name),
                severity:        ViolationSeverity::Medium,
                impact:          0.8,
                recommendations: vec![
                    "Review UI test failures".to_string(),
                    "Check browser automation setup".to_string(),
                ],
            })
            .collect();

        results.insert("ui_gate".to_string(), QualityGateResult {
            gate_name: "ui_gate".to_string(),
            gate_type: GateType::UI,
            status,
            score,
            key_metrics,
            violations,
            execution_time: Duration::from_millis(100),
        });

        Ok(results)
    }

    /// Execute E2E gates
    async fn execute_e2e_gates(&mut self) -> Result<HashMap<String, QualityGateResult>, RustAIError> {
        let mut results = HashMap::new();

        // Test basic personas
        let workflow_results = self
            .e2e_runner
            .execute_user_workflow(UserWorkflowType::NewUserOnboarding, UserPersona::BEGINNER)
            .await?;

        let failed_workflows = if workflow_results.success { 0 } else { 1 };

        let status = if failed_workflows == 0 {
            GateStatus::Passed
        } else if failed_workflows <= self.gate_config.gate_thresholds.max_e2e_test_failures as usize {
            GateStatus::Warning
        } else {
            GateStatus::Failed
        };

        let score = if workflow_results.success { 1.0 } else { 0.4 };

        let key_metrics = vec![GateMetric {
            name:      "e2e_success_rate".to_string(),
            value:     score * 100.0,
            unit:      "%".to_string(),
            threshold: Some(95.0),
            status:    if score >= 0.95 {
                MetricStatus::Good
            } else if score >= 0.80 {
                MetricStatus::Warning
            } else {
                MetricStatus::Critical
            },
        }];

        let violations = if !workflow_results.success {
            vec![GateViolation {
                rule_name:       "e2e_workflow_failure".to_string(),
                description:     "E2E workflow execution failed".to_string(),
                severity:        ViolationSeverity::High,
                impact:          0.9,
                recommendations: workflow_results.errors,
            }]
        } else {
            vec![]
        };

        results.insert("e2e_gate".to_string(), QualityGateResult {
            gate_name: "e2e_gate".to_string(),
            gate_type: GateType::E2E,
            status,
            score,
            key_metrics,
            violations,
            execution_time: workflow_results.duration,
        });

        Ok(results)
    }

    /// Create execution summary
    fn create_execution_summary(&self, results: &HashMap<String, QualityGateResult>) -> GateExecutionSummary {
        let total_gates = results.len();
        let passed_gates = results
            .values()
            .filter(|r| matches!(r.status, GateStatus::Passed))
            .count();
        let warning_gates = results
            .values()
            .filter(|r| matches!(r.status, GateStatus::Warning))
            .count();
        let failed_gates = results
            .values()
            .filter(|r| matches!(r.status, GateStatus::Failed))
            .count();
        let skipped_gates = results
            .values()
            .filter(|r| matches!(r.status, GateStatus::Skipped))
            .count();

        let overall_score = results.values().map(|r| r.score).sum::<f64>() / total_gates as f64;
        let total_execution_time = results.values().map(|r| r.execution_time).sum();

        let breaches: Vec<String> = results
            .values()
            .filter(|r| matches!(r.status, GateStatus::Failed | GateStatus::Warning))
            .map(|r| format!("{} {}", r.gate_name, r.status))
            .collect();

        let all_recommendations: Vec<String> = results
            .values()
            .flat_map(|r| r.violations.iter().flat_map(|v| v.recommendations.clone()))
            .collect();

        GateExecutionSummary {
            total_gates,
            passed_gates,
            warning_gates,
            failed_gates,
            skipped_gates,
            overall_score,
            total_execution_time,
            breaches,
            recommendations: all_recommendations,
        }
    }

    /// Send notifications for gate failures
    async fn send_notifications(
        &self,
        summary: &GateExecutionSummary,
        cicd_info: &CICDIntegration,
    ) -> Result<(), RustAIError> {
        // Placeholder for notification logic
        // In practice, this would send Slack messages, emails, etc.

        if !summary.breaches.is_empty() {
            tracing::warn!(
                "Quality gates failed for commit {}: {:?}",
                cicd_info.commit_hash,
                summary.breaches
            );
        }

        Ok(())
    }

    /// Utility method to convert performance severity
    fn convert_performance_severity(severity: &ViolationSeverity) -> ViolationSeverity {
        match severity {
            ViolationSeverity::Low => ViolationSeverity::Low,
            ViolationSeverity::Medium => ViolationSeverity::Medium,
            ViolationSeverity::High => ViolationSeverity::High,
            ViolationSeverity::Critical => ViolationSeverity::Critical,
        }
    }

    /// Utility method to calculate violation impact
    fn calculate_violation_impact(severity: &ViolationSeverity) -> f64 {
        match severity {
            ViolationSeverity::Low => 0.2,
            ViolationSeverity::Medium => 0.5,
            ViolationSeverity::High => 0.8,
            ViolationSeverity::Critical => 1.0,
        }
    }

    /// Utility method to convert priority to severity
    fn convert_priority_severity(priority: &Priority) -> ViolationSeverity {
        match priority {
            Priority::Low => ViolationSeverity::Low,
            Priority::Medium => ViolationSeverity::Medium,
            Priority::High => ViolationSeverity::High,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quality_gate_orchestrator_creation() {
        let orchestrator = QualityGateOrchestrator::new();
        assert!(orchestrator.execution_history.is_empty());
        assert!(orchestrator.current_execution.is_none());
    }

    #[tokio::test]
    async fn test_execution_summary_creation() {
        let orchestrator = QualityGateOrchestrator::new();

        let mut results = HashMap::new();

        results.insert("test_gate_1".to_string(), QualityGateResult {
            gate_name:      "test_gate_1".to_string(),
            gate_type:      GateType::Performance,
            status:         GateStatus::Passed,
            score:          1.0,
            key_metrics:    vec![],
            violations:     vec![],
            execution_time: Duration::from_secs(1),
        });

        results.insert("test_gate_2".to_string(), QualityGateResult {
            gate_name:      "test_gate_2".to_string(),
            gate_type:      GateType::Coverage,
            status:         GateStatus::Failed,
            score:          0.3,
            key_metrics:    vec![],
            violations:     vec![GateViolation {
                rule_name:       "test_violation".to_string(),
                description:     "Test violation".to_string(),
                severity:        ViolationSeverity::High,
                impact:          0.8,
                recommendations: vec!["Fix test".to_string()],
            }],
            execution_time: Duration::from_secs(2),
        });

        let summary = orchestrator.create_execution_summary(&results);

        assert_eq!(summary.total_gates, 2);
        assert_eq!(summary.passed_gates, 1);
        assert_eq!(summary.failed_gates, 1);
        assert_eq!(summary.breaches.len(), 1);
        assert_eq!(summary.recommendations.len(), 1);
    }

    #[test]
    fn test_cicd_integration_creation() {
        let cicd = CICDIntegration {
            git_ref:            "refs/heads/main".to_string(),
            branch:             "main".to_string(),
            commit_hash:        "abc123".to_string(),
            build_number:       Some("123".to_string()),
            pull_request:       None,
            test_environment:   TestEnvironment::CI,
            should_block_merge: true,
        };

        assert_eq!(cicd.branch, "main");
        assert_eq!(cicd.commit_hash, "abc123");
    }

    #[test]
    fn test_violation_severity_conversion() {
        let orchestrator = QualityGateOrchestrator::new();

        assert_eq!(
            orchestrator.convert_performance_severity(&ViolationSeverity::Low),
            ViolationSeverity::Low
        );
        assert_eq!(
            orchestrator.convert_performance_severity(&ViolationSeverity::Critical),
            ViolationSeverity::Critical
        );
    }

    #[test]
    fn test_violation_impact_calculation() {
        let orchestrator = QualityGateOrchestrator::new();

        assert_eq!(
            orchestrator.calculate_violation_impact(&ViolationSeverity::Low),
            0.2
        );
        assert_eq!(
            orchestrator.calculate_violation_impact(&ViolationSeverity::High),
            0.8
        );
    }
}

/// CI/CD Pipeline Integration Helpers
pub mod cicd_integration {
    use super::*;

    /// GitHub Actions integration
    pub struct GitHubActionsIntegration;

    impl GitHubActionsIntegration {
        /// Export gate results as GitHub Actions outputs
        pub fn export_results_as_outputs(results: &HashMap<String, QualityGateResult>) {
            for (key, result) in results {
                println!("::set-output name=gate_status_{}::{}", key, result.status);
                println!("::set-output name=gate_score_{}::{:.2}", key, result.score);
            }
        }

        /// Check if workflow should fail based on results
        pub fn should_fail_workflow(results: &HashMap<String, QualityGateResult>) -> bool {
            results
                .values()
                .any(|r| matches!(r.status, GateStatus::Failed))
        }

        /// Generate GitHub Actions summary
        pub fn generate_workflow_summary(summary: &GateExecutionSummary) -> String {
            format!(
                "# Quality Gate Results\n\n## Summary\n- **Total Gates**: {}\n- **Passed**: {} ✅\n- **Warnings**: {} \
                 ⚠️\n- **Failed**: {} ❌\n- **Overall Score**: {:.1}%\n- **Execution Time**: {:.1}s\n\n## \
                 Breaches\n{}\n## Recommendations\n{}",
                summary.total_gates,
                summary.passed_gates,
                summary.warning_gates,
                summary.failed_gates,
                summary.overall_score * 100.0,
                summary.total_execution_time.as_secs_f64(),
                if summary.breaches.is_empty() {
                    "- No breaches detected ✅".to_string()
                } else {
                    summary
                        .breaches
                        .iter()
                        .map(|b| format!("- {}", b))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
                if summary.recommendations.is_empty() {
                    "- No recommendations".to_string()
                } else {
                    summary
                        .recommendations
                        .iter()
                        .take(5)
                        .map(|r| format!("- {}", r))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            )
        }
    }

    /// Jenkins CI integration
    pub struct JenkinsIntegration;

    impl JenkinsIntegration {
        /// Set Jenkins build description
        pub fn set_build_description(description: &str) {
            println!("Build Description: {}", description);
        }

        /// Set Jenkins build status
        pub fn set_build_result(result: &str) {
            println!("Build Result: {}", result);
        }

        /// Publish test results in JUnit format
        pub fn publish_junit_results(results: &HashMap<String, QualityGateResult>, timestamp: DateTime<Utc>) -> String {
            let mut xml = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites timestamp=\"{}\">\n",
                timestamp.to_rfc3339()
            );

            for (name, result) in results {
                xml.push_str(&format!(
                    "  <testsuite name=\"{}\" tests=\"1\" failures=\"{}\" time=\"{:.3}\">\n",
                    name,
                    if matches!(result.status, GateStatus::Failed) {
                        "1"
                    } else {
                        "0"
                    },
                    result.execution_time.as_secs_f64()
                ));

                xml.push_str(&format!(
                    "    <testcase name=\"gate_check\" time=\"{:.3}\">\n",
                    result.execution_time.as_secs_f64()
                ));

                if matches!(result.status, GateStatus::Failed) {
                    xml.push_str("      <failure message=\"Quality gate failed\">\n");
                    for violation in &result.violations {
                        xml.push_str(&format!(
                            "        <![CDATA[{}: {}]]>\n",
                            violation.rule_name, violation.description
                        ));
                    }
                    xml.push_str("      </failure>\n");
                }

                xml.push_str("    </testcase>\n  </testsuite>\n");
            }

            xml.push_str("</testsuites>\n");
            xml
        }
    }

    /// Azure DevOps integration
    pub struct AzureDevOpsIntegration;

    impl AzureDevOpsIntegration {
        /// Log test results to Azure DevOps
        pub fn log_test_results(results: &HashMap<String, QualityGateResult>) {
            println!("##vso[task.logissue type=info]Quality Gate Results:");
            for (name, result) in results {
                let status = match result.status {
                    GateStatus::Passed => "✅ PASS",
                    GateStatus::Warning => "⚠️ WARNING",
                    GateStatus::Failed => "❌ FAIL",
                    GateStatus::Skipped => "⏭️ SKIP",
                };
                println!("##vso[task.logissue type=info]  {} - {}", name, status);
            }
        }

        /// Set Azure DevOps build status
        pub fn set_build_status(status: &str, description: &str) {
            println!("##vso[build.updatebuildnumber]{}, {}", status, description);
        }
    }
}
