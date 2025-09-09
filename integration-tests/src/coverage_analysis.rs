//! # Coverage Analysis and Trend Monitoring System
//!
//! Comprehensive coverage analysis system that tracks test coverage over time,
//! provides trend analysis, and generates quality gate checks for CI/CD pipelines.

use crate::{IntegrationTestResult, GlobalTestConfig};
use chrono::{DateTime, Utc};
use rust_ai_ide_errors::RustAIError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::time::Duration;

/// Main coverage analysis system
#[derive(Debug)]
pub struct CoverageAnalyzer {
    trend_history: BTreeMap<DateTime<Utc>, CoverageSnapshot>,
    current_snapshot: Option<CoverageSnapshot>,
    trend_thresholds: TrendThresholds,
    analysis_config: CoverageAnalysisConfig,
}

#[derive(Debug, Clone)]
pub struct CoverageSnapshot {
    pub timestamp: DateTime<Utc>,
    pub branch: String,
    pub commit_hash: String,
    pub coverage_data: CoverageData,
    pub test_execution_data: TestExecutionData,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CoverageData {
    pub overall_percentage: f64,
    pub lines_covered: u64,
    pub total_lines: u64,
    pub branches_covered: u64,
    pub total_branches: u64,
    pub functions_covered: u64,
    pub total_functions: u64,
    pub files_with_coverage: Vec<FileCoverage>,
    pub uncovered_lines: Vec<UncoveredLine>,
    pub coverage_trends: CoverageTrends,
}

#[derive(Debug, Clone)]
pub struct FileCoverage {
    pub file_path: String,
    pub lines_covered: u64,
    pub total_lines: u64,
    pub branches_covered: u64,
    pub total_branches: u64,
    pub functions_covered: u64,
    pub total_functions: u64,
    pub categories: Vec<CoverageCategory>,
}

#[derive(Debug, Clone)]
pub enum CoverageCategory {
    Good,      // > 80%
    Acceptable, // 70-80%
    Warning,   // 50-70%
    Critical,  // < 50%
    Uncovered, // 0%
}

#[derive(Debug, Clone)]
pub struct UncoveredLine {
    pub file_path: String,
    pub line_number: u32,
    pub line_content: Option<String>,
    pub reason: UncoveredReason,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub enum UncoveredReason {
    NoTest,
    Conditional,
    PlatformSpecific,
    IntegrationOnly,
    DeadCode,
    ErrorHandling,
}

#[derive(Debug, Clone)]
pub struct CoverageTrends {
    pub coverage_trend: TrendDirection,
    pub velocity: f64, // percentage points per day
    pub plateau_duration: u32, // days without significant change
    pub coverage_gaps: Vec<CoverageGap>,
}

#[derive(Debug, Clone)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Volatile,
}

#[derive(Debug, Clone)]
pub struct CoverageGap {
    pub file_path: String,
    pub gap_type: GapType,
    pub impact: f64,
    pub priority: Priority,
}

#[derive(Debug, Clone)]
pub enum GapType {
    UncoveredFunction,
    LowCoverageBranch,
    ErrorPathUncovered,
    DeadCodeNotPruned,
}

#[derive(Debug, Clone)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct TestExecutionData {
    pub test_results: Vec<TestResult>,
    pub test_duration: Duration,
    pub test_count: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub skipped_tests: u32,
    pub flaky_tests: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub duration: Duration,
    pub success: bool,
    pub coverage_contribution: u64,
}

#[derive(Debug, Clone)]
pub struct TrendThresholds {
    pub min_coverage_percentage: f64,
    pub max_decline_rate: f64, // percentage points per day
    pub max_plateau_days: u32,
    pub min_velocity: f64,
}

#[derive(Debug, Clone)]
pub struct CoverageAnalysisConfig {
    pub track_individual_files: bool,
    pub track_uncovered_reasons: bool,
    pub generate_trend_reports: bool,
    pub enable_consistency_checks: bool,
    pub coverage_tool: CoverageTool,
}

#[derive(Debug, Clone)]
pub enum CoverageTool {
    Tarpaulin,
    LLVM,
    GRcov,
    CargoCov,
}

/// Coverage analysis report
#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub snapshot: CoverageSnapshot,
    pub trends: TrendAnalysis,
    pub recommendations: Vec<CoverageRecommendation>,
    pub gate_status: CoverageGateStatus,
    pub risk_assessment: RiskAssessment,
}

#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub direction: TrendDirection,
    pub confidence_level: f64,
    pub prediction: Option<CoveragePrediction>,
    pub velocities: HashMap<String, f64>,
    pub bottlenecks: Vec<BottleneckAnalysis>,
}

#[derive(Debug, Clone)]
pub struct CoveragePrediction {
    pub predicted_coverage: f64,
    pub confidence_interval: (f64, f64),
    pub time_horizon_days: u32,
    pub factors_influencing: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    pub file_path: String,
    pub bottleneck_type: BottleneckType,
    pub impact: f64,
    pub ease_of_fix: f64,
}

#[derive(Debug, Clone)]
pub enum BottleneckType {
    ComplexBranching,
    ErrorConditions,
    EdgeCases,
    PlatformIntegration,
    ExternalDependencies,
}

#[derive(Debug, Clone)]
pub struct CoverageRecommendation {
    pub priority: Priority,
    pub category: RecommendationCategory,
    pub description: String,
    pub estimated_impact: f64,
    pub implementation_effort: EffortLevel,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RecommendationCategory {
    AddUnitTests,
    ImproveBranchCoverage,
    AddIntegrationTests,
    RefactorCode,
    AddMocking,
    PerformanceOptimization,
}

#[derive(Debug, Clone)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub enum CoverageGateStatus {
    Passed,
    Warning,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_strategies: Vec<String>,
    pub deadline_at_risk: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub factor_type: RiskType,
    pub severity: f64,
    pub description: String,
    pub impact: f64,
}

#[derive(Debug, Clone)]
pub enum RiskType {
    LowCoverageFile,
    DecliningTrends,
    MissingTests,
    DeadCode,
    ComplexLogicWithLowCoverage,
}

impl Default for TrendThresholds {
    fn default() -> Self {
        Self {
            min_coverage_percentage: 80.0,
            max_decline_rate: 2.0, // 2 percentage points per day max decline
            max_plateau_days: 14, // 2 weeks
            min_velocity: 0.1, // 0.1 percentage points per day improvement
        }
    }
}

impl Default for CoverageAnalysisConfig {
    fn default() -> Self {
        Self {
            track_individual_files: true,
            track_uncovered_reasons: true,
            generate_trend_reports: true,
            enable_consistency_checks: true,
            coverage_tool: CoverageTool::Tarpaulin,
        }
    }
}

impl CoverageAnalyzer {
    pub fn new() -> Self {
        Self {
            trend_history: BTreeMap::new(),
            current_snapshot: None,
            trend_thresholds: TrendThresholds::default(),
            analysis_config: CoverageAnalysisConfig::default(),
        }
    }

    /// Add a coverage snapshot to the trend history
    pub fn add_snapshot(&mut self, snapshot: CoverageSnapshot) -> Result<(), RustAIError> {
        self.current_snapshot = Some(snapshot.clone());
        self.trend_history.insert(snapshot.timestamp, snapshot);
        Ok(())
    }

    /// Generate comprehensive coverage report
    pub fn generate_coverage_report(&self) -> Result<CoverageReport, RustAIError> {
        let snapshot = self.current_snapshot.clone()
            .ok_or_else(|| RustAIError::ConfigurationError("No coverage snapshot available".to_string()))?;

        let trends = self.analyze_trends()?;
        let recommendations = self.generate_recommendations(&snapshot, &trends)?;
        let gate_status = self.check_coverage_gates(&snapshot)?;
        let risk_assessment = self.assess_risk(&snapshot, &trends)?;

        Ok(CoverageReport {
            snapshot,
            trends,
            recommendations,
            gate_status,
            risk_assessment,
        })
    }

    /// Analyze coverage trends over time
    fn analyze_trends(&self) -> Result<TrendAnalysis, RustAIError> {
        if self.trend_history.len() < 2 {
            // Not enough data for trend analysis
            return Ok(TrendAnalysis {
                direction: TrendDirection::Stable,
                confidence_level: 0.5,
                prediction: None,
                velocities: HashMap::new(),
                bottlenecks: vec![],
            });
        }

        let recent_snapshots: Vec<_> = self.trend_history.values().rev().take(10).collect();

        // Calculate overall trend direction
        let coverage_values: Vec<f64> = recent_snapshots.iter()
            .map(|s| s.coverage_data.overall_percentage)
            .collect();

        let direction = self.calculate_trend_direction(&coverage_values);

        // Calculate velocity (changes per day)
        let mut velocities = HashMap::new();
        velocities.insert("overall_coverage".to_string(), self.calculate_velocity(&coverage_values));

        // Find bottlenecks
        let bottlenecks = self.identify_bottlenecks(&recent_snapshots);

        // Generate prediction if we have enough data
        let prediction = if recent_snapshots.len() >= 7 {
            Some(self.generate_coverage_prediction(&coverage_values))
        } else {
            None
        };

        Ok(TrendAnalysis {
            direction,
            confidence_level: self.calculate_confidence_level(&coverage_values),
            prediction,
            velocities,
            bottlenecks,
        })
    }

    /// Calculate trend direction from coverage values
    fn calculate_trend_direction(&self, coverage_values: &[f64]) -> TrendDirection {
        if coverage_values.len() < 2 {
            return TrendDirection::Stable;
        }

        // Use linear regression to determine trend
        let n = coverage_values.len() as f64;
        let x_sum: f64 = (0..coverage_values.len()).map(|i| i as f64).sum();
        let y_sum: f64 = coverage_values.iter().sum();
        let xy_sum: f64 = coverage_values.iter().enumerate()
            .map(|(i, &y)| i as f64 * y).sum();
        let xx_sum: f64 = (0..coverage_values.len()).map(|i| (i as f64 * i as f64)).sum();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * xx_sum - x_sum * x_sum);

        if slope.abs() < 0.01 {
            TrendDirection::Stable
        } else if slope > 0.05 {
            TrendDirection::Improving
        } else if slope < -0.05 {
            TrendDirection::Declining
        } else {
            TrendDirection::Volatile
        }
    }

    /// Calculate velocity (change per day)
    fn calculate_velocity(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        // Assuming daily measurements
        let first = values[0];
        let last = values[values.len() - 1];
        let days = values.len() as f64 - 1.0;

        if days > 0.0 {
            (last - first) / days
        } else {
            0.0
        }
    }

    /// Identify coverage bottlenecks
    fn identify_bottlenecks(&self, snapshots: &[&CoverageSnapshot]) -> Vec<BottleneckAnalysis> {
        let mut bottlenecks = Vec::new();

        if let Some(latest) = snapshots.last() {
            for file_coverage in &latest.coverage_data.files_with_coverage {
                let coverage_percentage = if file_coverage.total_lines > 0 {
                    file_coverage.lines_covered as f64 / file_coverage.total_lines as f64 * 100.0
                } else {
                    100.0
                };

                if coverage_percentage < 70.0 {
                    // Identify the type of bottleneck
                    let bottleneck_type = self.identify_bottleneck_type(file_coverage);
                    let complexity_score = self.estimate_bottleneck_complexity(file_coverage);
                    let coverage_score = (100.0 - coverage_percentage) / 100.0;

                    let impact = coverage_score * complexity_score;

                    // Ease of fix is inversely related to complexity and coverage gap
                    let ease_of_fix = (1.0 - complexity_score) * coverage_score;

                    bottlenecks.push(BottleneckAnalysis {
                        file_path: file_coverage.file_path.clone(),
                        bottleneck_type,
                        impact,
                        ease_of_fix,
                    });
                }
            }
        }

        // Sort by impact
        bottlenecks.sort_by(|a, b| b.impact.partial_cmp(&a.impact).unwrap());
        bottlenecks.truncate(10); // Top 10 bottlenecks

        bottlenecks
    }

    /// Identify the type of coverage bottleneck
    fn identify_bottleneck_type(&self, file_coverage: &FileCoverage) -> BottleneckType {
        // Analyze file path and coverage patterns to determine bottleneck type
        if file_coverage.file_path.contains("error") || file_coverage.file_path.contains("panic") {
            BottleneckType::ErrorConditions
        } else if file_coverage.branches_covered < file_coverage.total_branches / 2 {
            BottleneckType::ComplexBranching
        } else if file_coverage.file_path.contains("platform") || file_coverage.file_path.contains("ffi") {
            BottleneckType::PlatformIntegration
        } else if file_coverage.functions_covered < file_coverage.total_functions / 2 {
            BottleneckType::ExternalDependencies
        } else {
            BottleneckType::EdgeCases
        }
    }

    /// Estimate complexity of a bottleneck based on file characteristics
    fn estimate_bottleneck_complexity(&self, file_coverage: &FileCoverage) -> f64 {
        let coverage_ratio = if file_coverage.total_lines > 0 {
            file_coverage.lines_covered as f64 / file_coverage.total_lines as f64
        } else {
            1.0
        };

        let branch_complexity = if file_coverage.total_branches > 0 {
            1.0 - (file_coverage.branches_covered as f64 / file_coverage.total_branches as f64)
        } else {
            0.0
        };

        // Complexity is a function of line count and branch complexity
        let size_factor = (file_coverage.total_lines as f64 / 1000.0).min(1.0);
        let branch_factor = branch_complexity.min(1.0);

        (size_factor * 0.6 + branch_factor * 0.4).min(1.0)
    }

    /// Calculate confidence level for trend analysis
    fn calculate_confidence_level(&self, coverage_values: &[f64]) -> f64 {
        if coverage_values.len() < 3 {
            return 0.5;
        }

        // Calculate coefficient of variation as a proxy for confidence
        let mean = coverage_values.iter().sum::<f64>() / coverage_values.len() as f64;
        let variance = coverage_values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / coverage_values.len() as f64;
        let std_dev = variance.sqrt();

        if mean > 0.0 {
            let coeff_of_variation = std_dev / mean;
            // Lower coefficient of variation = higher confidence (capped at 0.95)
            1.0 - coeff_of_variation.min(0.95)
        } else {
            0.5
        }
    }

    /// Generate coverage prediction based on historical data
    fn generate_coverage_prediction(&self, coverage_values: &[f64]) -> CoveragePrediction {
        // Simple linear prediction for demonstration
        let current = coverage_values.last().copied().unwrap_or(0.0);
        let velocity = self.calculate_velocity(coverage_values);

        let predicted_coverage = (current + velocity * 30.0).max(0.0).min(100.0); // 30-day prediction
        let confidence_interval = (
            (current + velocity * 30.0 * 0.8).max(0.0).min(100.0),
            (current + velocity * 30.0 * 1.2).max(0.0).min(100.0),
        );

        CoveragePrediction {
            predicted_coverage,
            confidence_interval,
            time_horizon_days: 30,
            factors_influencing: vec![
                format!("Current velocity: {:.2} percentage points per day", velocity),
                "Historical trend stability".to_string(),
                "Codebase growth rate".to_string(),
                "Test addition rate".to_string(),
            ],
        }
    }
}

/// Implementation methods for recommendations, gate checking, and risk assessment continue...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_analyzer_creation() {
        let analyzer = CoverageAnalyzer::new();
        assert!(analyzer.trend_history.is_empty());
        assert!(analyzer.current_snapshot.is_none());
    }

    #[tokio::test]
    async fn test_trend_direction_calculation() {
        let analyzer = CoverageAnalyzer::new();

        // Test stable trend
        let values = vec![80.0, 80.1, 79.9, 80.2];
        let direction = analyzer.calculate_trend_direction(&values);
        assert!(matches!(direction, TrendDirection::Stable));

        // Test improving trend
        let values = vec![75.0, 76.0, 77.0, 78.0];
        let direction = analyzer.calculate_trend_direction(&values);
        assert!(matches!(direction, TrendDirection::Improving));

        // Test declining trend
        let values = vec![85.0, 84.0, 83.0, 82.0];
        let direction = analyzer.calculate_trend_direction(&values);
        assert!(matches!(direction, TrendDirection::Declining));
    }

    #[tokio::test]
    async fn test_velocity_calculation() {
        let analyzer = CoverageAnalyzer::new();

        // Test with increasing values
        let values = vec![80.0, 82.0, 84.0, 86.0];
        let velocity = analyzer.calculate_velocity(&values);
        assert_eq!(velocity, 2.0);

        // Test with decreasing values
        let values = vec![90.0, 88.0, 86.0, 84.0];
        let velocity = analyzer.calculate_velocity(&values);
        assert_eq!(velocity, -2.0);

        // Test with single value
        let values = vec![85.0];
        let velocity = analyzer.calculate_velocity(&values);
        assert_eq!(velocity, 0.0);
    }

    #[test]
    fn test_confidence_level_calculation() {
        let analyzer = CoverageAnalyzer::new();

        // Test with stable values (high confidence)
        let values = vec![80.0, 80.1, 79.9, 80.0, 80.2];
        let confidence = analyzer.calculate_confidence_level(&values);
        assert!(confidence > 0.8);

        // Test with volatile values (lower confidence)
        let values = vec![80.0, 85.0, 75.0, 90.0, 70.0];
        let confidence = analyzer.calculate_confidence_level(&values);
        assert!(confidence < 0.5);
    }

    #[test]
    fn test_bottleneck_type_identification() {
        let analyzer = CoverageAnalyzer::new();

        let file_coverage = FileCoverage {
            file_path: "src/error_handling.rs".to_string(),
            lines_covered: 50,
            total_lines: 100,
            branches_covered: 5,
            total_branches: 20,
            functions_covered: 8,
            total_functions: 10,
            categories: vec![],
        };

        let bottleneck_type = analyzer.identify_bottleneck_type(&file_coverage);
        assert!(matches!(bottleneck_type, BottleneckType::ErrorConditions));
    }
}

impl FileCoverage {
    pub fn coverage_percentage(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            self.lines_covered as f64 / self.total_lines as f64 * 100.0
        }
    }
}

impl From<&FileCoverage> for CoverageCategory {
    fn from(fc: &FileCoverage) -> Self {
        let percentage = fc.coverage_percentage();
        match percentage {
            p if p > 80.0 => CoverageCategory::Good,
            p if p > 70.0 => CoverageCategory::Acceptable,
            p if p > 50.0 => CoverageCategory::Warning,
            p if p > 0.0 => CoverageCategory::Critical,
            _ => CoverageCategory::Uncovered,
        }
    }
}