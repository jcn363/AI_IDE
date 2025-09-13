//! Quality metrics and scoring system

use serde::{Deserialize, Serialize};

use crate::errors::Result;

/// Overall quality metrics for analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    /// Total number of issues found
    pub total_issues: usize,

    /// Number of critical issues
    pub critical_issues: usize,

    /// Number of high-severity issues
    pub high_issues: usize,

    /// Number of medium-severity issues
    pub medium_issues: usize,

    /// Number of low-severity issues
    pub low_issues: usize,

    /// Number of info-level findings
    pub info_issues: usize,

    /// Average severity score (0-100 scale)
    pub average_severity: f64,

    /// Total analysis time in seconds
    pub analysis_time_seconds: f64,

    /// Code coverage percentage (if available)
    pub code_coverage_percent: Option<f64>,

    /// Compilation time in seconds (if available)
    pub compilation_time_seconds: Option<f64>,

    /// Memory usage in MB (if available)
    pub memory_usage_mb: Option<f64>,

    /// CPU usage percentage (if available)
    pub cpu_usage_percent: Option<f64>,
}

/// Quality score calculation and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    /// Overall score (0-100)
    pub overall: f64,

    /// Static analysis score
    pub static_analysis: f64,

    /// Performance score
    pub performance: f64,

    /// Security score
    pub security: f64,

    /// Code quality score
    pub code_quality: f64,

    /// Dependencies score
    pub dependencies: f64,

    /// Cross-platform score
    pub cross_platform: f64,

    /// Score calculation timestamp
    pub calculated_at: chrono::DateTime<chrono::Utc>,

    /// Trend direction compared to previous score
    pub trend_direction: crate::types::TrendDirection,

    /// Change percentage from previous score
    pub change_percent: Option<f64>,
}

impl Default for QualityScore {
    fn default() -> Self {
        Self {
            overall:         0.0,
            static_analysis: 0.0,
            performance:     0.0,
            security:        0.0,
            code_quality:    0.0,
            dependencies:    0.0,
            cross_platform:  0.0,
            calculated_at:   chrono::Utc::now(),
            trend_direction: crate::types::TrendDirection::Unknown,
            change_percent:  None,
        }
    }
}

impl QualityScore {
    /// Calculate overall quality score from metrics
    pub fn from_metrics(metrics: &QualityMetrics) -> Self {
        // Calculate component scores based on metrics
        let static_analysis = Self::calculate_static_analysis_score(metrics);
        let performance = Self::calculate_performance_score(metrics);
        let security = Self::calculate_security_score(metrics);
        let code_quality = Self::calculate_code_quality_score(metrics);
        let dependencies = Self::calculate_dependencies_score(metrics);
        let cross_platform = Self::calculate_cross_platform_score(metrics);

        // Weighted overall score
        let overall = (static_analysis * 0.25)
            + (performance * 0.20)
            + (security * 0.15)
            + (code_quality * 0.25)
            + (dependencies * 0.10)
            + (cross_platform * 0.05);

        Self {
            overall,
            static_analysis,
            performance,
            security,
            code_quality,
            dependencies,
            cross_platform,
            calculated_at: chrono::Utc::now(),
            trend_direction: crate::types::TrendDirection::Unknown,
            change_percent: None,
        }
    }

    /// Calculate static analysis score (0-100)
    fn calculate_static_analysis_score(metrics: &QualityMetrics) -> f64 {
        // Base score of 100, reduced by critical/high issues
        let base_score = 100.0;

        // Critical issues have highest impact
        let critical_penalty = metrics.critical_issues as f64 * 20.0;

        // High issues have moderate impact
        let high_penalty = metrics.high_issues as f64 * 10.0;

        // Medium issues have lower impact
        let medium_penalty = metrics.medium_issues as f64 * 5.0;

        // Low issues have minimal impact
        let low_penalty = metrics.low_issues as f64 * 1.0;

        let total_penalty = critical_penalty + high_penalty + medium_penalty + low_penalty;

        (base_score - total_penalty).max(0.0).min(100.0)
    }

    /// Calculate performance score (0-100)
    fn calculate_performance_score(metrics: &QualityMetrics) -> f64 {
        // Performance is scored based on compilation time and resource usage
        let mut score = 100.0;

        if let Some(compilation_time) = metrics.compilation_time_seconds {
            // Penalize slow compilation (more than 5 minutes is bad)
            if compilation_time > 300.0 {
                score -= ((compilation_time - 300.0) / 60.0) * 10.0;
            }
        }

        if let Some(memory_mb) = metrics.memory_usage_mb {
            // Penalize high memory usage (more than 2GB is concerning)
            if memory_mb > 2048.0 {
                score -= ((memory_mb - 2048.0) / 512.0) * 5.0;
            }
        }

        score.max(0.0).min(100.0)
    }

    /// Calculate security score (0-100)
    fn calculate_security_score(_metrics: &QualityMetrics) -> f64 {
        // Security score would be calculated based on security analysis results
        // For now, return a placeholder
        85.0 // Placeholder - would be computed from security analyzer results
    }

    /// Calculate code quality score (0-100)
    fn calculate_code_quality_score(metrics: &QualityMetrics) -> f64 {
        // Code quality is primarily based on warnings count
        const IDEAL_WARNINGS: usize = 10;
        const MAX_WARNINGS: usize = 100;

        if metrics.total_issues <= IDEAL_WARNINGS {
            100.0
        } else if metrics.total_issues >= MAX_WARNINGS {
            0.0
        } else {
            let excess_warnings = metrics.total_issues - IDEAL_WARNINGS;
            let penalty_per_warning = 100.0 / (MAX_WARNINGS - IDEAL_WARNINGS) as f64;
            (100.0 - (excess_warnings as f64 * penalty_per_warning)).max(0.0)
        }
    }

    /// Calculate dependencies score (0-100)
    fn calculate_dependencies_score(_metrics: &QualityMetrics) -> f64 {
        // Dependencies score would be calculated from dependency analysis
        90.0 // Placeholder
    }

    /// Calculate cross-platform score (0-100)
    fn calculate_cross_platform_score(_metrics: &QualityMetrics) -> f64 {
        // Cross-platform score would be calculated from compilation across targets
        95.0 // Placeholder
    }

    /// Update trend information compared to previous score
    pub fn update_trend(&mut self, previous_score: Option<f64>) {
        if let Some(prev) = previous_score {
            if (self.overall - prev).abs() < 0.1 {
                self.trend_direction = crate::types::TrendDirection::Stable;
            } else if self.overall > prev {
                self.trend_direction = crate::types::TrendDirection::Increasing;
            } else {
                self.trend_direction = crate::types::TrendDirection::Decreasing;
            }

            self.change_percent = Some(((self.overall - prev) / prev) * 100.0);
        }
    }

    /// Check if the quality score is acceptable
    pub fn is_acceptable(&self, threshold: f64) -> bool {
        self.overall >= threshold
    }

    /// Get quality grade based on score
    pub fn get_grade(&self) -> &'static str {
        match self.overall {
            95.0..=100.0 => "A+",
            90.0..=95.0 => "A",
            85.0..=90.0 => "B+",
            80.0..=85.0 => "B",
            75.0..=80.0 => "C+",
            70.0..=75.0 => "C",
            65.0..=70.0 => "D+",
            60.0..=65.0 => "D",
            _ => "F",
        }
    }
}

/// Metrics aggregator for collecting and combining results from multiple analyzers
pub struct MetricsAggregator {
    metrics: QualityMetrics,
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new() -> Self {
        Self {
            metrics: QualityMetrics::default(),
        }
    }

    /// Add findings from an analyzer result
    pub fn add_findings(&mut self, findings: &[crate::types::Finding]) {
        for finding in findings {
            self.metrics.total_issues += 1;

            match finding.severity {
                crate::types::Severity::Critical => self.metrics.critical_issues += 1,
                crate::types::Severity::High => self.metrics.high_issues += 1,
                crate::types::Severity::Medium => self.metrics.medium_issues += 1,
                crate::types::Severity::Low => self.metrics.low_issues += 1,
                crate::types::Severity::Info => self.metrics.info_issues += 1,
                crate::types::Severity::None => {}
            }
        }
    }

    /// Set analysis timing information
    pub fn set_timing(&mut self, duration_seconds: f64, compilation_time: Option<f64>) {
        self.metrics.analysis_time_seconds = duration_seconds;
        self.metrics.compilation_time_seconds = compilation_time;
    }

    /// Set resource usage information
    pub fn set_resources(&mut self, memory_mb: Option<f64>, cpu_percent: Option<f64>) {
        self.metrics.memory_usage_mb = memory_mb;
        self.metrics.cpu_usage_percent = cpu_percent;
    }

    /// Calculate average severity score
    pub fn calculate_average_severity(&mut self) {
        if self.metrics.total_issues == 0 {
            self.metrics.average_severity = 0.0;
            return;
        }

        // Severity values: Critical=100, High=75, Medium=50, Low=25, Info=10, None=0
        let total_severity_score = (self.metrics.critical_issues * 100)
            + (self.metrics.high_issues * 75)
            + (self.metrics.medium_issues * 50)
            + (self.metrics.low_issues * 25)
            + (self.metrics.info_issues * 10);

        self.metrics.average_severity = total_severity_score as f64 / self.metrics.total_issues as f64;
    }

    /// Get the accumulated metrics
    pub fn get_metrics(&self) -> &QualityMetrics {
        &self.metrics
    }

    /// Take ownership of the accumulated metrics
    pub fn take_metrics(mut self) -> QualityMetrics {
        self.calculate_average_severity();
        self.metrics
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}
