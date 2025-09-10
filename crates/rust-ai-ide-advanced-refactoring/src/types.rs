use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_ai_ide_types::*;

/// Core refactoring suggestion data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    pub id: Uuid,
    pub suggestion_type: RefactoringType,
    pub target_file: String,
    pub target_line: usize,
    pub target_column: usize,
    pub description: String,
    pub confidence_score: f64,
    pub risk_level: RiskLevel,
    pub estimated_complexity: Complexity,
    pub behavioural_preservation_guarantees: Vec<String>,
    pub code_before: String,
    pub code_after: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringType {
    ExtractMethod,
    InlineMethod,
    RenameSymbol,
    ExtractVariable,
    InlineVariable,
    RenameLocal,
    ExtractConstant,
    InlineConstant,
    AddParameter,
    RemoveParameter,
    ReorderParameters,
    MoveMethod,
    MoveField,
    ExtractClass,
    ExtractInterface,
    PullUpMethod,
    PullUpField,
    PushDownMethod,
    PushDownField,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Complexity {
    Trivial,
    Simple,
    Moderate,
    Complex,
    High,
    VeryHigh,
}

/// Transformation operation with its context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringTransformation {
    pub id: Uuid,
    pub suggestion_id: Uuid,
    pub operation_type: TransformationOperation,
    pub file_path: String,
    pub line_number: usize,
    pub column_number: usize,
    pub original_text: String,
    pub transformed_text: String,
    pub dependencies: Vec<TransformationDependency>,
    pub rollback_steps: Vec<RollbackStep>,
    pub validation_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationOperation {
    ReplaceText,
    InsertText,
    DeleteText,
    MoveText,
    CopyText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationDependency {
    pub dependency_type: DependencyType,
    pub dependent_file: Option<String>,
    pub dependent_symbol: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    SymbolReference,
    TypeReference,
    SemanticDependency,
    FileDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    pub operation: RollbackOperation,
    pub target_file: String,
    pub line_number: usize,
    pub column_number: usize,
    pub original_text: String,
    pub transformed_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackOperation {
    RestoreOriginal,
    UndoOperation,
    CustomCommand(String),
}

/// Impact assessment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub assessment_id: Uuid,
    pub suggestion_id: Uuid,
    pub overall_impact_score: f64,
    pub cost_benefit_analysis: CostBenefitAnalysis,
    pub dependency_chain: Vec<FileDependency>,
    pub performance_impact: PerformanceImpact,
    pub risk_assessment: RiskAssessment,
    pub timeline_estimate: TimelineEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBenefitAnalysis {
    pub development_cost: f64,
    pub maintenance_cost: f64,
    pub performance_benefit: f64,
    pub maintainability_improvement: f64,
    pub breaking_change_risk: f64,
    pub net_benefit_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDependency {
    pub file_path: String,
    pub dependency_type: DependencyType,
    pub impact_severity: ImpactSeverity,
    pub lines_affected: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    pub memory_overhead_change: f64,
    pub cpu_usage_change: f64,
    pub disk_io_change: f64,
    pub network_io_change: f64,
    pub estimated_runtime_impact_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub likelihood_of_failure: f64,
    pub impact_if_failed: f64,
    pub mitigation_strategies: Vec<RiskMitigationStrategy>,
    pub confidence_in_assessment: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMitigationStrategy {
    pub strategy_type: MitigationStrategyType,
    pub description: String,
    pub implementation_cost: f64,
    pub effectiveness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationStrategyType {
    BackupPlan,
    FallbackMechanism,
    GradualRollout,
    FeatureFlag,
    Monitoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEstimate {
    pub estimated_duration_hours: f64,
    pub confidence_level: f64,
    pub critical_path_tasks: Vec<String>,
    pub resource_requirements: Vec<ResourceRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirement {
    pub resource_type: String,
    pub amount: f64,
    pub availability_score: f64,
}

/// Safety validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyValidation {
    pub validation_id: Uuid,
    pub transformation_id: Uuid,
    pub is_safe: bool,
    pub validation_checks: Vec<ValidationCheck>,
    pub functional_equivalence_verified: bool,
    pub behavior_preservation_confirmed: bool,
    pub circular_dependencies_detected: Vec<CircularDependency>,
    pub overall_safety_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub check_type: ValidationCheckType,
    pub passed: bool,
    pub severity: ValidationSeverity,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationCheckType {
    SyntacticCorrectness,
    TypeChecking,
    FunctionalEquivalence,
    BehaviorPreservation,
    DependencyResolution,
    SecurityValidation,
    PerformanceImpact,
    MemorySafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub dependency_chain: Vec<String>,
    pub circular_symptoms: Vec<String>,
    pub resolution_suggestions: Vec<String>,
}

/// Execution context and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringExecutionContext {
    pub execution_id: Uuid,
    pub session_id: Uuid,
    pub transformations: Vec<RefactoringTransformation>,
    pub execution_order: Vec<Uuid>,
    pub status: ExecutionStatus,
    pub progress: ExecutionProgress,
    pub start_time: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProgress {
    pub current_transformation: usize,
    pub total_transformations: usize,
    pub completed_transformations: Vec<Uuid>,
    pub failed_transformations: Vec<Uuid>,
    pub percentage_complete: f64,
    pub estimated_time_remaining_seconds: Option<f64>,
}

/// Configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringConfig {
    pub max_concurrent_transformations: usize,
    pub safety_threshold: f64,
    pub auto_rollback_enabled: bool,
    pub backup_interval_seconds: u64,
    pub monitoring_enabled: bool,
    pub audit_log_enabled: bool,
    pub performance_limits: PerformanceLimits,
    pub risk_limits: RiskLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceLimits {
    pub max_memory_mb: usize,
    pub max_execution_time_seconds: u64,
    pub max_cpu_usage_percent: u8,
    pub max_disk_io_operations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_circular_dependency_depth: usize,
    pub min_functional_equivalence_confidence: f64,
    pub max_behavior_change_probability: f64,
    pub min_rollback_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSummary {
    pub summary_id: Uuid,
    pub session_id: Uuid,
    pub total_suggestions_generated: usize,
    pub suggestions_accepted: usize,
    pub suggestions_rejected: usize,
    pub transformations_executed: usize,
    pub transformations_rolled_back: usize,
    pub overall_success_rate: f64,
    pub time_saved_estimate: Option<f64>,
    pub quality_improvements: Vec<String>,
}

pub type ValidationResult<T> = Result<T, ValidationError>;
pub type ExecutionResult<T> = Result<T, ExecutionError>;
pub type AnalysisResult<T> = Result<T, AnalysisError>;