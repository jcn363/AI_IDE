//! # Performance Gate Checking System
//!
//! Automated performance gate checking system that integrates with CI/CD pipelines
//! to prevent performance regressions and ensure quality standards.

use crate::{IntegrationTestResult, GlobalTestConfig};
use chrono::{DateTime, Utc};
use rust_ai_ide_errors::RustAIError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::sync::Mutex;

/// Main performance gate checker
#[derive(Debug)]
pub struct PerformanceGateChecker {
    baseline_metrics: HashMap<String, PerformanceBaseline>,
    current_thresholds: PerformanceThresholds,
    gate_config: GateConfig,
    enabled_gates: HashSet<PerformanceGate>,
}

#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub branch: String,
    pub commit_hash: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_regression_percent: f64,
    pub max_variation_percent: f64,
    pub min_samples_required: u32,
    pub confidence_level: f64,
    pub statistical_test: StatisticalTest,
}

#[derive(Debug, Clone)]
pub enum StatisticalTest {
    TTest,
    MannWhitney,
    KolmogorovSmirnov,
    VarianceRatio,
}

#[derive(Debug, Clone)]
pub struct GateConfig {
    pub fail_on_regression: bool,
    pub warn_on_degradation: bool,
    pub strict_mode: bool,
    pub allow_waiver: bool,
    pub collect_trends: bool,
    pub notification_channel: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum PerformanceGate {
    /// Memory usage gate
    MemoryUsage,
    /// CPU usage gate
    CpuUsage,
    /// Build time gate
    BuildTime,
    /// Runtime performance gate
    RuntimePerformance,
    /// Memory leakage gate
    MemoryLeakage,
    /// Compilation speed gate
    CompileSpeed,
    /// Startup time gate
    StartupTime,
}

/// Performance measurement result
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub baseline_name: String,
    pub confidence_interval: Option<(f64, f64)>,
    pub metadata: HashMap<String, String>,
}

/// Gate check result
#[derive(Debug, Clone)]
pub struct GateCheckResult {
    pub gate: PerformanceGate,
    pub status: GateCheckStatus,
    pub measurements: Vec<PerformanceMeasurement>,
    pub violations: Vec<PerformanceGateViolation>,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PerformanceGateViolation {
    pub metric_name: String,
    pub measured_value: f64,
    pub baseline_value: f64,
    pub threshold_percentage: f64,
    pub violation_type: ViolationType,
    pub severity: ViolationSeverity,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum GateCheckStatus {
    Passed,
    Warning,
    Failed,
    Inconclusive,
}

#[derive(Debug, Clone)]
pub enum ViolationType {
    Regression,
    Degradation,
    Variation,
    Anomaly,
    Timeout,
}

#[derive(Debug, Clone)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_regression_percent: 5.0,
            max_variation_percent: 10.0,
            min_samples_required: 5,
            confidence_level: 0.95,
            statistical_test: StatisticalTest::TTest,
        }
    }
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            fail_on_regression: true,
            warn_on_degradation: true,
            strict_mode: false,
            allow_waiver: true,
            collect_trends: true,
            notification_channel: Some("performance-alerts".to_string()),
        }
    }
}

impl PerformanceGateChecker {
    pub fn new() -> Self {
        Self {
            baseline_metrics: HashMap::new(),
            current_thresholds: PerformanceThresholds::default(),
            gate_config: GateConfig::default(),
            enabled_gates: HashSet::from([
                PerformanceGate::BuildTime,
                PerformanceGate::RuntimePerformance,
                PerformanceGate::MemoryUsage,
                PerformanceGate::CompileSpeed,
            ]),
        }
    }

    /// Enable a specific performance gate
    pub fn enable_gate(&mut self, gate: PerformanceGate) {
        self.enabled_gates.insert(gate);
    }

    /// Disable a specific performance gate
    pub fn disable_gate(&mut self, gate: PerformanceGate) {
        self.enabled_gates.remove(&gate);
    }

    /// Set performance threshold for a specific metric
    pub fn set_threshold(&mut self, metric_name: &str, threshold: PerformanceBaseline) {
        self.baseline_metrics.insert(metric_name.to_string(), threshold);
    }

    /// Execute all enabled performance gates
    pub async fn execute_gates(&self) -> Result<Vec<GateCheckResult>, RustAIError> {
        let mut results = Vec::new();

        for gate in &self.enabled_gates {
            let result = self.execute_gate(gate.clone()).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute a specific performance gate
    async fn execute_gate(&self, gate: PerformanceGate) -> Result<GateCheckResult, RustAIError> {
        tracing::info!("Executing performance gate: {:?}", gate);

        let measurements = self.collect_measurements(&gate).await?;
        let violations = self.check_violations(&measurements, &gate)?;

        let status = self.determine_gate_status(&violations);
        let recommendations = self.generate_recommendations(&violations);

        Ok(GateCheckResult {
            gate: gate.clone(),
            status,
            measurements,
            violations,
            recommendations,
            timestamp: Utc::now(),
        })
    }

    /// Collect performance measurements for a gate
    async fn collect_measurements(&self, gate: &PerformanceGate) -> Result<Vec<PerformanceMeasurement>, RustAIError> {
        // Simulate collecting real performance metrics
        // In practice, this would integrate with the actual performance measurement tools

        match gate {
            PerformanceGate::BuildTime => {
                // Simulate build time measurement
                let build_time = self.measure_build_time().await?;
                Ok(vec![PerformanceMeasurement {
                    metric_name: "build_time".to_string(),
                    value: build_time,
                    unit: "ms".to_string(),
                    timestamp: Utc::now(),
                    baseline_name: "main_brach_build_time".to_string(),
                    confidence_interval: Some((build_time * 0.95, build_time * 1.05)),
                    metadata: HashMap::new(),
                }])
            }
            PerformanceGate::RuntimePerformance => {
                let runtime_perf = self.measure_runtime_performance().await?;
                Ok(vec![runtime_perf])
            }
            PerformanceGate::MemoryUsage => {
                let memory_usage = self.measure_memory_usage().await?;
                Ok(vec![memory_usage])
            }
            PerformanceGate::CompileSpeed => {
                let compile_speed = self.measure_compile_speed().await?;
                Ok(vec![compile_speed])
            }
            _ => Ok(vec![]),
        }
    }

    /// Check for performance violations
    fn check_violations(
        &self,
        measurements: &[PerformanceMeasurement],
        gate: &PerformanceGate,
    ) -> Result<Vec<PerformanceGateViolation>, RustAIError> {
        let mut violations = Vec::new();

        for measurement in measurements {
            if let Some(baseline) = self.baseline_metrics.get(&measurement.metric_name) {
                let regression_percent = ((measurement.value - baseline.value) / baseline.value) * 100.0;

                // Check if regression exceeds threshold
                if regression_percent.abs() > self.current_thresholds.max_regression_percent {
                    let violation = PerformanceGateViolation {
                        metric_name: measurement.metric_name.clone(),
                        measured_value: measurement.value,
                        baseline_value: baseline.value,
                        threshold_percentage: self.current_thresholds.max_regression_percent,
                        violation_type: if regression_percent > 0.0 {
                            ViolationType::Regression
                        } else {
                            ViolationType::Degradation
                        },
                        severity: if regression_percent.abs() > 10.0 {
                            ViolationSeverity::Critical
                        } else if regression_percent.abs() > 7.0 {
                            ViolationSeverity::High
                        } else {
                            ViolationSeverity::Medium
                        },
                        description: format!(
                            "Performance {} of {:.1}% detected in {} ({:.2} {} vs baseline {:.2})",
                            if regression_percent > 0.0 { "regression" } else { "improvement" },
                            regression_percent.abs(),
                            measurement.metric_name,
                            measurement.value,
                            measurement.unit,
                            baseline.value
                        ),
                    };

                    violations.push(violation);
                }
            }
        }

        Ok(violations)
    }

    /// Determine gate check status
    fn determine_gate_status(&self, violations: &[PerformanceGateViolation]) -> GateCheckStatus {
        let has_critical = violations.iter().any(|v| matches!(v.severity, ViolationSeverity::Critical));
        let has_high = violations.iter().any(|v| matches!(v.severity, ViolationSeverity::High));

        if has_critical && self.gate_config.fail_on_regression {
            GateCheckStatus::Failed
        } else if has_high && self.gate_config.warn_on_degradation {
            GateCheckStatus::Warning
        } else {
            GateCheckStatus::Passed
        }
    }

    /// Generate recommendations for violations
    fn generate_recommendations(&self, violations: &[PerformanceGateViolation]) -> Vec<String> {
        let mut recommendations = Vec::new();

        for violation in violations {
            match violation.violation_type {
                ViolationType::Regression => {
                    recommendations.push(format!(
                        "Address {}.{} performance regression: optimize {}, potentially reduce to below threshold of {:.1}%",
                        violation.metric_name, violation.violation_type, violation.metric_name, violation.threshold_percentage
                    ));

                    if violation.metric_name.contains("build") {
                        recommendations.push("Consider incremental compilation or parallel builds".to_string());
                    } else if violation.metric_name.contains("memory") {
                        recommendations.push("Review memory allocation patterns and consider memory pooling".to_string());
                    }
                }
                ViolationType::Degradation => {
                    if self.gate_config.strict_mode {
                        recommendations.push(format!("Monitor {}.{} improvement trend", violation.metric_name, violation.violation_type));
                    }
                }
                _ => {
                    recommendations.push(format!("Investigate {}.{} anomaly", violation.metric_name, violation.violation_type));
                }
            }
        }

        recommendations
    }

    /// Get gate summary for CI/CD integration
    pub fn get_gate_summary(&self, results: &[GateCheckResult]) -> GateCheckSummary {
        let total_gates = results.len();
        let passed_gates = results.iter().filter(|r| matches!(r.status, GateCheckStatus::Passed)).count();
        let warning_gates = results.iter().filter(|r| matches!(r.status, GateCheckStatus::Warning)).count();
        let failed_gates = results.iter().filter(|r| matches!(r.status, GateCheckStatus::Failed)).count();

        let overall_status = if failed_gates > 0 {
            GateCheckStatus::Failed
        } else if warning_gates > 0 {
            GateCheckStatus::Warning
        } else {
            GateCheckStatus::Passed
        };

        let all_violations: Vec<_> = results.iter().flat_map(|r| r.violations.iter()).collect();
        let all_recommendations: Vec<_> = results.iter().flat_map(|r| r.recommendations.iter()).collect();

        GateCheckSummary {
            total_gates,
            passed_gates,
            warning_gates,
            failed_gates,
            overall_status,
            violations_count: all_violations.len(),
            recommendations: all_recommendations.into_iter().cloned().collect(),
            timestamp: Utc::now(),
        }
    }
}

/// Summary of all gate check results
#[derive(Debug, Clone)]
pub struct GateCheckSummary {
    pub total_gates: usize,
    pub passed_gates: usize,
    pub warning_gates: usize,
    pub failed_gates: usize,
    pub overall_status: GateCheckStatus,
    pub violations_count: usize,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

// Implementation of measurement methods (simulated for now)

impl PerformanceGateChecker {
    async fn measure_build_time(&self) -> Result<f64, RustAIError> {
        // In practice, this would run `cargo build --release` and measure time
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(4520.5) // Simulated build time in milliseconds
    }

    async fn measure_runtime_performance(&self) -> Result<PerformanceMeasurement, RustAIError> {
        // In practice, this would run benchmarks and measure execution time
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(PerformanceMeasurement {
            metric_name: "runtime_performance".to_string(),
            value: 120.5, // Simulated response time in milliseconds
            unit: "ms".to_string(),
            timestamp: Utc::now(),
            baseline_name: "runtime_baseline".to_string(),
            confidence_interval: Some((118.0, 123.0)),
            metadata: HashMap::new(),
        })
    }

    async fn measure_memory_usage(&self) -> Result<PerformanceMeasurement, RustAIError> {
        // In practice, this would monitor memory usage during execution
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok(PerformanceMeasurement {
            metric_name: "memory_usage".to_string(),
            value: 128.5, // Simulated memory usage in MB
            unit: "MB".to_string(),
            timestamp: Utc::now(),
            baseline_name: "memory_baseline".to_string(),
            confidence_interval: Some((126.0, 131.0)),
            metadata: HashMap::new(),
        })
    }

    async fn measure_compile_speed(&self) -> Result<PerformanceMeasurement, RustAIError> {
        // In practice, this would measure compilation speed metrics
        tokio::time::sleep(Duration::from_millis(20)).await;
        Ok(PerformanceMeasurement {
            metric_name: "compile_speed".to_string(),
            value: 45.2, // Simulated compilation speed
            unit: "files_per_second".to_string(),
            timestamp: Utc::now(),
            baseline_name: "compile_baseline".to_string(),
            confidence_interval: Some((43.0, 47.0)),
            metadata: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_gate_checker_creation() {
        let checker = PerformanceGateChecker::new();
        assert!(!checker.enabled_gates.is_empty());
        assert!(checker.baseline_metrics.is_empty());
    }

    #[tokio::test]
    async fn test_gate_enable_disable() {
        let mut checker = PerformanceGateChecker::new();

        // Initially should have default gates enabled
        assert!(checker.enabled_gates.contains(&PerformanceGate::BuildTime));

        // Disable a gate
        checker.disable_gate(PerformanceGate::BuildTime);
        assert!(!checker.enabled_gates.contains(&PerformanceGate::BuildTime));

        // Re-enable the gate
        checker.enable_gate(PerformanceGate::BuildTime);
        assert!(checker.enabled_gates.contains(&PerformanceGate::BuildTime));
    }

    #[tokio::test]
    async fn test_baseline_setting() {
        let mut checker = PerformanceGateChecker::new();

        let baseline = PerformanceBaseline {
            timestamp: Utc::now(),
            value: 4000.0,
            branch: "main".to_string(),
            commit_hash: "abc123".to_string(),
            metadata: HashMap::new(),
        };

        checker.set_threshold("test_metric", baseline);
        assert!(checker.baseline_metrics.contains_key("test_metric"));
    }

    #[test]
    fn test_gate_check_status_determination() {
        let checker = PerformanceGateChecker::new();

        // Test with no violations
        let status = checker.determine_gate_status(&[]);
        assert!(matches!(status, GateCheckStatus::Passed));

        // Test with critical violation
        let critical_violation = PerformanceGateViolation {
            metric_name: "test_metric".to_string(),
            measured_value: 5000.0,
            baseline_value: 4000.0,
            threshold_percentage: 5.0,
            violation_type: ViolationType::Regression,
            severity: ViolationSeverity::Critical,
            description: "Test violation".to_string(),
        };

        let status = checker.determine_gate_status(&[critical_violation]);
        assert!(matches!(status, GateCheckStatus::Failed));
    }

    #[tokio::test]
    async fn test_measurement_collection() {
        let checker = PerformanceGateChecker::new();

        // Test build time measurement
        let result = checker.measure_build_time().await;
        assert!(result.is_ok());

        // Test runtime performance measurement
        let result = checker.measure_runtime_performance().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().metric_name, "runtime_performance");
    }
}

impl std::fmt::Display for PerformanceGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceGate::MemoryUsage => write!(f, "Memory Usage"),
            PerformanceGate::CpuUsage => write!(f, "CPU Usage"),
            PerformanceGate::BuildTime => write!(f, "Build Time"),
            PerformanceGate::RuntimePerformance => write!(f, "Runtime Performance"),
            PerformanceGate::MemoryLeakage => write!(f, "Memory Leakage"),
            PerformanceGate::CompileSpeed => write!(f, "Compile Speed"),
            PerformanceGate::StartupTime => write!(f, "Startup Time"),
        }
    }
}

impl std::fmt::Display for GateCheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateCheckStatus::Passed => write!(f, "PASSED"),
            GateCheckStatus::Warning => write!(f, "WARNING"),
            GateCheckStatus::Failed => write!(f, "FAILED"),
            GateCheckStatus::Inconclusive => write!(f, "INCONCLUSIVE"),
        }
    }
}

impl std::fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationSeverity::Low => write!(f, "LOW"),
            ViolationSeverity::Medium => write!(f, "MEDIUM"),
            ViolationSeverity::High => write!(f, "HIGH"),
            ViolationSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}