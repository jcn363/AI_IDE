//! Core type definitions for predictive maintenance forecasting system

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Configuration for the predictive maintenance system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceConfig {
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,

    /// Maximum number of historical debt points to analyze
    pub max_historical_points: usize,

    /// Confidence threshold for recommendations
    pub confidence_threshold: f64,

    /// Time horizons for forecasting (in weeks)
    pub forecast_horizons: Vec<u32>,

    /// Minimum file size threshold for analysis
    pub min_file_size_threshold: u64,

    /// Maximum dependency depth for impact analysis
    pub max_dependency_depth: usize,

    /// Cost estimation multiplier base
    pub cost_multiplier_base: f64,

    /// Database path for persistent storage
    pub database_path: Option<String>,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 3600, // 1 hour
            max_historical_points: 1000,
            confidence_threshold: 0.7,
            forecast_horizons: vec![4, 8, 12, 24], // 1, 2, 3, 6 months
            min_file_size_threshold: 1024, // 1KB
            max_dependency_depth: 10,
            cost_multiplier_base: 1.0,
            database_path: None,
        }
    }
}

/// Workspace and project context for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub total_lines_of_code: u64,
    pub file_count: usize,
    pub last_modified: DateTime<Utc>,
}

/// Analysis context containing current state and historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// Current technical debt items
    pub current_debt_items: Vec<DebtItem>,

    /// Historical debt measurements
    pub historical_measurements: Vec<DebtMeasurement>,

    /// Recent refactoring activities
    pub recent_refactorings: Vec<RefactoringActivity>,

    /// Dependence graph for cross-file analysis
    pub dependency_graph: HashMap<String, Vec<String>>,
}

/// Core technical debt item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtItem {
    /// Unique identifier for the debt item
    pub id: String,

    /// File path where debt was identified
    pub file_path: String,

    /// Line number where debt starts
    pub start_line: u32,

    /// Line number where debt ends (for multi-line items)
    pub end_line: u32,

    /// Type of technical debt
    pub debt_type: DebtType,

    /// Severity level (0.0 to 1.0, where 1.0 is most severe)
    pub severity: f64,

    /// Description of the technical debt
    pub description: String,

    /// Estimated effort to fix (in developer hours)
    pub estimated_effort_hours: f64,

    /// Impact on system maintainability (0.0 to 1.0)
    pub maintainability_impact: f64,

    /// Age of the debt (in days)
    pub age_days: u32,

    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Type of technical debt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DebtType {
    /// Code complexity issues
    Complexity,
    /// Duplication of code
    Duplication,
    /// Outdated dependencies
    OutdatedDependency,
    /// Dead code
    DeadCode,
    /// Lack of documentation
    Documentation,
    /// Security vulnerabilities
    Security,
    /// Performance issues
    Performance,
    /// Architectural violations
    Architecture,
    /// Test debt
    Test,
    /// Custom debt type
    Custom(String),
}

/// Historical debt measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtMeasurement {
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,

    /// Total debt items count
    pub total_count: usize,

    /// Total severity score
    pub total_severity: f64,

    /// Total estimated effort
    pub total_effort_hours: f64,

    /// Average maintainability impact
    pub avg_maintainability_impact: f64,
}

/// Recent refactoring activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringActivity {
    /// Timestamp of refactoring
    pub timestamp: DateTime<Utc>,

    /// Files affected
    pub affected_files: Vec<String>,

    /// Type of refactoring
    pub refactoring_type: String,

    /// Lines of code changed
    pub lines_changed: u32,

    /// Effort spent (in hours)
    pub effort_hours: f64,

    /// Success rating (0.0 to 1.0)
    pub success_rating: f64,
}

/// Technical debt forecasting result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtForecast {
    /// Projected debt items over time
    pub projected_debt: Vec<ProjectedDebtItem>,

    /// Forecast confidence score
    pub confidence_score: f64,

    /// Forecast window (in weeks)
    pub forecast_window_weeks: u32,

    /// Critical thresholds identified
    pub critical_thresholds: Vec<Threshold>,
}

/// Projected debt item with timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedDebtItem {
    /// Reference to original debt item
    pub original_debt_id: String,

    /// Projected evolution timeline
    pub timeline: Vec<TimelinePoint>,

    /// Critical threshold crossings
    pub threshold_crossings: Vec<DateTime<Utc>>,

    /// Risk factors
    pub risk_factors: Vec<RiskFactor>,
}

/// Single point in debt evolution timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    /// Timestamp of prediction
    pub timestamp: DateTime<Utc>,

    /// Projected severity at this point
    pub severity: f64,

    /// Projected maintainability impact
    pub maintainability_impact: f64,

    /// Confidence interval for severity (low, high)
    pub confidence_interval: (f64, f64),
}

/// Risk factor affecting debt evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Type of risk
    pub factor_type: RiskType,

    /// Risk score (0.0 to 1.0)
    pub score: f64,

    /// Description of the risk
    pub description: String,

    /// Time when risk becomes active
    pub activation_time: DateTime<Utc>,
}

/// Type of risk factor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskType {
    /// File changes frequency
    ChangeFrequency,
    /// Team member churn
    TeamChurn,
    /// Technology obsolescence
    TechnologyObsolescence,
    /// Growing complexity
    ComplexityGrowth,
    /// External dependencies
    ExternalDependency,
}

/// Critical threshold in debt evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    /// Type of threshold
    pub threshold_type: ThresholdType,

    /// Value that triggers threshold
    pub value: f64,

    /// Time when threshold is expected to be crossed
    pub expected_crossing: DateTime<Utc>,

    /// Description of consequences
    pub consequences: String,
}

/// Type of threshold
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThresholdType {
    /// Severity threshold
    Severity,
    /// Impact threshold
    Impact,
    /// Budget threshold
    Budget,
    /// Timeline threshold
    Timeline,
}

/// Cost estimation for maintenance activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimation {
    /// Total estimated cost
    pub total_estimated_cost: f64,

    /// Breakdown by debt item
    pub breakdown: Vec<CostBreakdown>,

    /// Currency unit used
    pub currency: String,
}

/// Cost breakdown for a single debt item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Base estimation from analysis
    pub estimated_effort_hours: f64,

    /// Risk factor multiplier
    pub risk_factor: f64,

    /// Complexity multiplier
    pub complexity_multiplier: f64,

    /// Urgency score for prioritization
    pub urgency_score: f64,

    /// Detailed cost components
    pub components: Vec<CostComponent>,
}

/// Component of maintenance cost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostComponent {
    /// Name of the cost component
    pub component_name: String,

    /// Base effort hours
    pub base_effort_hours: f64,

    /// Adjusted effort with multipliers
    pub adjusted_effort_hours: f64,

    /// Justification for the estimate
    pub justification: String,
}

/// Dependency impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Impact assessment for each debt item
    pub impacts: Vec<DependencyImpact>,

    /// Overall risk score for changes
    pub overall_risk_score: f64,

    /// Safe refactoring sequences
    pub safe_sequences: Vec<SafeSequence>,
}

/// Impact of a debt item on dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyImpact {
    /// Debt item ID
    pub debt_id: String,

    /// Files that would be affected by fixing this debt
    pub affected_files: Vec<String>,

    /// Risk level of changing this item
    pub risk_level: RiskLevel,

    /// Change propagation likelihood
    pub propagation_likelihood: f64,

    /// Required testing effort multiplier
    pub testing_multiplier: f64,
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Safe sequence for refactoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeSequence {
    /// Debt items in sequence
    pub sequence: Vec<String>,

    /// Risk score for this sequence
    pub risk_score: f64,

    /// Time buffer needed between steps
    pub time_buffer_days: u32,

    /// Rationale for this sequence
    pub rationale: String,
}

/// Prioritized task list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedTaskList {
    /// Tasks sorted by priority
    pub tasks: Vec<PrioritizedTask>,

    /// Strategy used for prioritization
    pub prioritization_strategy: String,
}

/// Single prioritized maintenance task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedTask {
    /// Original index in the forecast
    pub original_index: usize,

    /// Computed priority score
    pub priority_score: f64,

    /// Recommended timeline for completion
    pub recommended_timeline: TimeFrame,

    /// Rationale for prioritization
    pub rationale: Vec<String>,
}

/// Time frame for task completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeFrame {
    Immediate,
    ThisWeek,
    ThisMonth,
    NextQuarter,
    NextYear,
    Future,
}

/// Final maintenance recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRecommendations {
    /// Actionable recommendations
    pub recommendations: Vec<Recommendation>,

    /// Automated implementations available
    pub automated_implementations: Vec<AutomatedImplementation>,

    /// Documentation updates needed
    pub documentation_updates: Vec<DocumentationUpdate>,
}

/// Single maintenance recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation ID (links back to debt item)
    pub id: String,

    /// Type of recommendation
    pub recommendation_type: String,

    /// Priority level
    pub priority: PriorityLevel,

    /// Recommended action
    pub action: String,

    /// Expected outcome
    pub expected_outcome: String,

    /// Rationale
    pub rationale: String,
}

/// Priority level for recommendations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PriorityLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// Automated implementation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedImplementation {
    /// Associated recommendation ID
    pub recommendation_id: String,

    /// Automated action available
    pub available_action: String,

    /// Confidence in automation safety
    pub safety_confidence: f64,

    /// Code that can be generated
    pub generated_code: Option<String>,
}

/// Documentation update requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationUpdate {
    /// Associated recommendation ID
    pub recommendation_id: String,

    /// Documentation type needed
    pub doc_type: String,

    /// Required updates
    pub updates: Vec<String>,

    /// Effort estimate
    pub effort_hours: f64,
}

/// Complete maintenance forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceForecast {
    /// Technical debt forecast
    pub debt_forecast: DebtForecast,

    /// Cost estimation
    pub cost_estimation: CostEstimation,

    /// Impact analysis
    pub impact_analysis: ImpactAnalysis,

    /// Prioritized tasks
    pub prioritized_tasks: PrioritizedTaskList,

    /// Final recommendations
    pub recommendations: MaintenanceRecommendations,

    /// Timestamp when forecast was generated
    pub generated_at: DateTime<Utc>,

    /// Overall confidence score
    pub confidence_score: f64,
}