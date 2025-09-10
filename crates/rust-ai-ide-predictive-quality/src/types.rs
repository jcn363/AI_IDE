//! Core types for predictive quality intelligence

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Request for vulnerability prediction analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisRequest {
    pub files: Vec<CodeFile>,
    pub dependencies: Vec<String>,
    pub security_context: SecurityContext,
    pub code_complexity: CodeComplexityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    pub path: String,
    pub content: String,
    pub language: String,
    pub hash: String,
    pub ast_tree: Option<String>, // Serialized AST representation
}

/// Security context for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub project_type: String,
    pub dependencies: Vec<Dependency>,
    pub known_vulnerabilities: Vec<String>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub vulnerability_score: u8, // 0-100
}

/// Code complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeComplexityMetrics {
    pub cyclomatic_complexity: f64,
    pub lines_of_code: u32,
    pub functions_count: u32,
    pub classes_count: u32,
    pub code_duplication_ratio: f64,
}

/// Vulnerability prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityPredictionResult {
    pub predictions: Vec<VulnerabilityPrediction>,
    pub overall_risk_score: f64, // 0.0 - 1.0
    pub confidence_level: f64,   // 0.0 - 1.0
    pub processed_at: DateTime<Utc>,
    pub model_version: String,
}

/// Individual vulnerability prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityPrediction {
    pub vulnerability_type: String,
    pub description: String,
    pub severity: SeverityLevel,
    pub confidence_score: f64, // 0.0 - 1.0
    pub affected_files: Vec<String>,
    pub recommended_fixes: Vec<String>,
    pub cwe_ids: Vec<String>,
    pub owasp_top_10_ids: Vec<String>,
}

/// Severity levels with numerical values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityLevel {
    pub fn as_score(&self) -> f64 {
        match self {
            SeverityLevel::Low => 0.25,
            SeverityLevel::Medium => 0.5,
            SeverityLevel::High => 0.75,
            SeverityLevel::Critical => 1.0,
        }
    }
}

/// Maintenance schedule request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceScheduleRequest {
    pub project_files: Vec<String>,
    pub tech_debt_threshold: f64,
    pub time_horizon_days: i32,
    pub priority_factors: HashMap<String, f64>, // Custom priority weights
}

/// Maintenance forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceForecastResult {
    pub total_estimated_cost: f64,
    pub tasks_by_priority: Vec<MaintenanceTask>,
    pub forecast_by_period: Vec<PeriodForecast>,
    pub risk_assessment: RiskAssessment,
    pub recommendations: Vec<String>,
}

/// Individual maintenance task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceTask {
    pub id: String,
    pub description: String,
    pub estimated_cost: f64,
    pub estimated_effort_hours: f64,
    pub priority_score: f64,
    pub risk_level: SeverityLevel,
    pub affected_files: Vec<String>,
    pub remediation_steps: Vec<String>,
}

/// Forecast for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodForecast {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub estimated_tasks: Vec<MaintenanceTask>,
    pub total_cost: f64,
    pub confidence_intervals: ConfidenceIntervals,
}

/// Statistical confidence intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceIntervals {
    pub confidence_95_low: f64,
    pub confidence_95_high: f64,
    pub confidence_99_low: f64,
    pub confidence_99_high: f64,
}

/// Overall risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: SeverityLevel,
    pub risk_factors: HashMap<String, f64>,
    pub mitigations: Vec<String>,
}

/// Health score request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScoreRequest {
    pub project_path: Option<String>,
    pub files: Vec<String>,
    pub include_trends: bool,
    pub benchmark_against: Option<String>, // Project or industry benchmarks
}

/// Health score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScoreResult {
    pub overall_health: f64, // 0.0 - 1.0
    pub metric_scores: HashMap<String, f64>,
    pub trend_analysis: Option<TrendAnalysis>,
    pub recommendations: Vec<HealthRecommendation>,
    pub benchmark_comparison: Option<BenchmarkComparison>,
    pub calculated_at: DateTime<Utc>,
}

/// Trend analysis for health scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub period_days: i32,
    pub health_trend: TrendDirection,
    pub significant_changes: Vec<MetricChange>,
    pub forecast_next_30_days: f64,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Volatile,
}

/// Significant metric change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricChange {
    pub metric_name: String,
    pub absolute_change: f64,
    pub relative_change_percentage: f64,
    pub trend_direction: TrendDirection,
    pub significance_level: SignificanceLevel,
}

/// Statistical significance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignificanceLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Health improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecommendation {
    pub priority: SeverityLevel,
    pub category: String,
    pub description: String,
    pub estimated_impact: f64,
    pub estimated_effort: EffortLevel,
    pub related_metrics: Vec<String>,
}

/// Effort level estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Benchmark comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub benchmark_name: String,
    pub benchmark_score: f64,
    pub percentile_rank: f64, // 0.0 - 1.0
    pub areas_above_benchmark: Vec<String>,
    pub areas_below_benchmark: Vec<String>,
}

/// Cross-file dependency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFileDependencyAnalysis {
    pub dependency_graph: DependencyGraph,
    pub circular_dependencies: Vec<CircularDependency>,
    pub impact_assessment: HashMap<String, ImpactAssessment>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

/// Dependency graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<String>, // File paths
    pub edges: Vec<DependencyEdge>,
    pub metrics: DependencyMetrics,
}

/// Dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
    pub strength: f64, // 0.0 - 1.0
}

/// Types of dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    Inherits,
    Composes,
    Calls,
    References,
}

/// Dependency metrics for the entire graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetrics {
    pub average_path_length: f64,
    pub clustering_coefficient: f64,
    pub centrality_scores: HashMap<String, f64>,
    pub strongly_connected_components: Vec<Vec<String>>,
}

/// Circular dependency detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub cycle: Vec<String>,
    pub impact_severity: SeverityLevel,
    pub refactoring_suggestions: Vec<String>,
}

/// Impact assessment for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub centrality_score: f64,
    pub change_risk: f64,
    pub dependency_depth: u32,
    pub affected_components: Vec<String>,
}

/// Optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub description: String,
    pub potential_impact: f64,
    pub affected_files: Vec<String>,
    pub implementation_effort: EffortLevel,
}

impl CodeAnalysisRequest {
    /// Generate a cache hash for this request
    pub fn cache_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.files.iter().for_each(|f| f.hash.hash(&mut hasher));
        self.dependencies.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}