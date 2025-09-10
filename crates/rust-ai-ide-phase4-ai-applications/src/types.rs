//! # Phase 4.1 Advanced AI Applications Types
//!
//! This module defines all core types used throughout the Phase 4.1 Advanced AI Applications system.
//! These types provide the foundation for sophisticated AI-powered development workflows.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// Core AI workflow request and response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIWorkflowRequest {
    /// Unique workflow identifier
    pub id: String,

    /// Workflow type
    pub workflow_type: WorkflowType,

    /// Development context information
    pub context: DevelopmentContext,

    /// Specific workflow parameters
    pub parameters: serde_json::Value,

    /// Quality requirements
    pub requirements: Option<QualityRequirements>,
}

/// AI workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIWorkflowResult {
    /// Workflow identifier
    pub workflow_id: String,

    /// Execution status
    pub status: WorkflowStatus,

    /// Generated outputs
    pub outputs: Vec<WorkflowOutput>,

    /// Quality metrics
    pub metrics: WorkflowMetrics,

    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Workflow types enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowType {
    /// Code generation and completion
    CodeGeneration,

    /// Code analysis and review
    CodeAnalysis,

    /// Project planning and management
    ProjectPlanning,

    /// Testing and quality assurance
    Testing,

    /// Refactoring and optimization
    Refactoring,

    /// Documentation generation
    Documentation,

    /// Custom user-defined workflow
    Custom(String),
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Successfully completed
    Completed,

    /// Partially completed with warnings
    CompletedWithWarnings(Vec<String>),

    /// Execution failed
    Failed(String),

    /// Currently executing
    InProgress{f64}, // Progress percentage

    /// Execution was cancelled
    Cancelled(String),
}

/// Development context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentContext {
    /// Project information
    pub project: ProjectContext,

    /// Current file context
    pub file_context: Option<FileContext>,

    /// User information and preferences
    pub user_context: Option<UserContext>,

    /// System context and capabilities
    pub system_context: Option<SystemContext>,
}

/// Project context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Project name
    pub name: String,

    /// Programming languages used
    pub languages: Vec<String>,

    /// Project size in lines of code
    pub size_loc: Option<u64>,

    /// Project dependencies
    pub dependencies: Vec<String>,

    /// Current project status
    pub status: ProjectStatus,

    /// Last modification time
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

/// Project status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    /// New project setup
    New,

    /// Actively being developed
    Active,

    /// Maintenance mode
    Stable,

    /// Deprecated or archived
    Archived,
}

/// File context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    /// File path
    pub path: String,

    /// File content
    pub content: Option<String>,

    /// Cursor position
    pub cursor_position: Option<CursorPosition>,

    /// Selected text
    pub selection: Option<String>,

    /// File language
    pub language: String,
}

/// User context and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User identifier
    pub user_id: String,

    /// User experience level
    pub experience_level: ExperienceLevel,

    /// Preferred programming languages
    pub preferred_languages: Vec<String>,

    /// Work style preferences
    pub work_style: WorkStyle,

    /// Development environment preferences
    pub preferences: UserPreferences,
}

/// User experience levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceLevel {
    /// Beginner developer
    Beginner,

    /// Intermediate developer
    Intermediate,

    /// Advanced/expert developer
    Advanced,

    /// Senior/experienced developer
    Senior,
}

/// Work style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStyle {
    /// Prefers test-driven development
    pub test_driven: bool,

    /// Prefers pair programming
    pub pair_programming: bool,

    /// Prefers comprehensive documentation
    pub comprehensive_doc: bool,

    /// Prefers minimal/viable solutions
    pub mvp_focus: bool,

    /// Innovation preference level
    pub innovation_level: i32, // -10 to +10 scale
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Code style preferences
    pub code_style: CodeStyle,

    /// Documentation preferences
    pub documentation_style: DocStyle,

    /// Testing preferences
    pub testing_level: TestingLevel,
}

/// Code style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeStyle {
    /// Functional programming style
    Functional,

    /// Object-oriented programming style
    ObjectOriented,

    /// Procedural programming style
    Procedural,

    /// Mixed/comprehensive approach
    Mixed,
}

/// Documentation style preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocStyle {
    /// Extensive inline documentation
    Extensive,

    /// Standard documentation coverage
    Standard,

    /// Minimal documentation
    Minimal,

    /// Generated/comprehensive documentation
    Generated,
}

/// Testing level preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestingLevel {
    /// Comprehensive test suite
    Full,

    /// Standard test coverage
    Standard,

    /// Minimal testing
    Minimal,

    /// Testing on demand
    OnDemand,
}

/// System context and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    /// Available AI models
    pub available_models: Vec<String>,

    /// Active AI services
    pub active_services: Vec<String>,

    /// System performance metrics
    pub performance: SystemPerformance,

    /// Current system load
    pub current_load: SystemLoad,
}

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformance {
    /// Available CPU cores
    pub cpu_cores: usize,

    /// Available memory in MB
    pub available_memory: u64,

    /// Current memory usage in MB
    pub current_memory: u64,

    /// System uptime in hours
    pub uptime_hours: f64,
}

/// System load information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLoad {
    /// CPU usage percentage
    pub cpu_usage: f32,

    /// Memory usage percentage
    pub memory_usage: f32,

    /// Active tasks/processes
    pub active_tasks: u32,

    /// Network latency in ms
    pub network_latency: f32,
}

/// Cursor position in file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Line number (0-based)
    pub line: usize,

    /// Column number (0-based)
    pub column: usize,

    /// Character offset from start of file
    pub offset: usize,
}

/// Quality requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    /// Minimum code quality score
    pub min_quality_score: f32,

    /// Required testing coverage
    pub test_coverage: f32,

    /// Maximum cyclomatic complexity
    pub max_complexity: u32,

    /// Performance constraints
    pub performance_requirements: Option<PerformanceReq>,

    /// Security requirements
    pub security_requirements: Option<SecurityReq>,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReq {
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,

    /// Maximum memory usage in MB
    pub max_memory_mb: u64,

    /// Minimum throughput operations per second
    pub min_throughput_ops: Option<u64>,
}

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReq {
    /// Required security scanning level
    pub scan_level: SecurityLevel,

    /// Required compliance standards
    pub compliance: Vec<String>,

    /// Sensitive data handling requirements
    pub sensitive_data_handling: SensitiveDataHandling,
}

/// Security scanning levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Basic security checks
    Basic,

    /// Comprehensive security analysis
    Comprehensive,

    /// Enterprise security requirements
    Enterprise,
}

/// Sensitive data handling requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveDataHandling {
    /// Require data encryption
    pub encryption_required: bool,

    /// Audit logging requirements
    pub audit_logging: bool,

    /// Privacy compliance requirements
    pub privacy_compliance: Vec<String>,
}

/// Workflow output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowOutput {
    /// Generated code
    Code(GeneratedCode),

    /// Analysis results
    Analysis(analysis::AnalysisResult),

    /// Test cases
    TestCase(TestCase),

    /// Documentation
    Documentation(String),

    /// Refactoring suggestions
    Refactoring(Vec<refactoring::RefactorSuggestion>),

    /// Custom output
    Custom {
        /// Output type identifier
        output_type: String,

        /// Output data
        data: serde_json::Value,

        /// Metadata
        metadata: HashMap<String, String>,
    },
}

/// Generated code output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    /// Code content
    pub content: String,

    /// Programming language
    pub language: String,

    /// File path suggestion
    pub suggested_path: Option<String>,

    /// Dependencies required
    pub dependencies: Vec<String>,

    /// Usage examples
    pub examples: Vec<String>,
}

/// Test case generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test case name
    pub name: String,

    /// Test description
    pub description: String,

    /// Test code
    pub code: String,

    /// Expected behavior
    pub expected_behavior: String,

    /// Test coverage
    pub coverage: f32,

    /// Test priority
    pub priority: TestPriority,
}

/// Test priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestPriority {
    /// Critical test case
    Critical,

    /// Highly important
    High,

    /// Standard importance
    Medium,

    /// Low priority
    Low,
}

/// Workflow execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    /// Total execution time
    pub execution_time: Duration,

    /// CPU time used
    pub cpu_time: Duration,

    /// Memory usage in MB
    pub memory_usage: f32,

    /// AI services utilized
    pub ai_services_used: Vec<String>,

    /// Number of operations performed
    pub operations_count: u32,

    /// Success rate of operations
    pub success_rate: f32,
}

/// Development insights comprehensive analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentInsights {
    /// Project health metrics
    pub project_health: ProjectHealth,

    /// Code quality insights
    pub code_quality: CodeQualityInsights,

    /// Development efficiency metrics
    pub development_efficiency: DevelopmentEfficiency,

    /// Recommendations for improvement
    pub recommendations: Vec<Recommendation>,

    /// Risk assessments
    pub risks: Vec<RiskAssessment>,
}

/// Project health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    /// Overall health score (0-100)
    pub overall_score: f32,

    /// Component health scores
    pub component_scores: HashMap<String, f32>,

    /// Health trend
    pub trend: HealthTrend,

    /// Critical issues
    pub critical_issues: u32,

    /// Major issues
    pub major_issues: u32,
}

/// Health trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthTrend {
    /// Health is improving
    Improving,

    /// Health is stable
    Stable,

    /// Health is deteriorating
    Deteriorating,

    /// Insufficient data for trend analysis
    Unknown,
}

/// Code quality insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityInsights {
    /// Overall quality score
    pub quality_score: f32,

    /// Technical debt estimation
    pub technical_debt_hours: f32,

    /// Maintainability index
    pub maintainability_index: f32,

    /// Cyclomatic complexity average
    pub complexity_average: f32,

    /// Test coverage percentage
    pub test_coverage: f32,
}

/// Development efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentEfficiency {
    /// Lines of code per hour
    pub loc_per_hour: f32,

    /// Bug fix rate (issues resolved per day)
    pub bug_fix_rate: f32,

    /// Feature completion rate
    pub feature_completion_rate: f32,

    /// Code review turnaround time
    pub review_turnaround_hours: f32,

    /// Build success rate
    pub build_success_rate: f32,
}

/// Improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation type
    pub r#type: RecommendationType,

    /// Title
    pub title: String,

    /// Description
    pub description: String,

    /// Priority level
    pub priority: Priority,

    /// Estimated effort in hours
    pub effort_hours: f32,

    /// Potential impact score
    pub impact_score: f32,

    /// Implementation steps
    pub implementation_steps: Vec<String>,
}

/// Recommendation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Code quality improvement
    CodeQuality,

    /// Architecture improvement
    Architecture,

    /// Performance optimization
    Performance,

    /// Security enhancement
    Security,

    /// Testing improvement
    Testing,

    /// Documentation improvement
    Documentation,

    /// Process improvement
    Process,

    /// Tooling improvement
    Tooling,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    /// Critical priority
    Critical,

    /// High priority
    High,

    /// Medium priority
    Medium,

    /// Low priority
    Low,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Risk type
    pub r#type: RiskType,

    /// Risk description
    pub description: String,

    /// Probability score (0-1)
    pub probability: f32,

    /// Impact score (0-1)
    pub impact: f32,

    /// Risk score (probability * impact)
    pub risk_score: f32,

    /// Mitigation strategies
    pub mitigation_strategies: Vec<String>,
}

/// Risk types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    /// Technical debt risk
    TechnicalDebt,

    /// Security vulnerability risk
    Security,

    /// Performance bottleneck risk
    Performance,

    /// Dependency risk
    Dependency,

    /// Code quality risk
    CodeQuality,

    /// Team productivity risk
    TeamProductivity,

    /// Schedule risk
    Schedule,
}

/// Code analysis and understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeUnderstanding {
    /// Semantic understanding score
    pub semantic_score: f32,

    /// Code patterns identified
    pub identified_patterns: Vec<CodePattern>,

    /// Potential issues detected
    pub detected_issues: Vec<CodeIssue>,

    /// Optimization opportunities
    pub optimization_opportunities: Vec<OptimizationOpportunity>,

    /// Semantic relationships
    pub relationships: Vec<SemanticRelationship>,
}

/// Code pattern identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    /// Pattern name
    pub name: String,

    /// Pattern type
    pub pattern_type: PatternType,

    /// Confidence score
    pub confidence: f32,

    /// Location in code
    pub location: CodeLocation,

    /// Pattern metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Pattern types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Design pattern
    Design,

    /// Anti-pattern
    AntiPattern,

    /// Code smell
    CodeSmell,

    /// Best practice
    BestPractice,

    /// Performance effective pattern
    Performance,

    /// Security pattern
    Security,
}

/// Code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    /// File path
    pub file: String,

    /// Start line number
    pub start_line: usize,

    /// End line number
    pub end_line: usize,

    /// Context lines around the location
    pub context: Option<String>,
}

/// Code issue detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    /// Issue type
    pub issue_type: IssueType,

    /// Severity level
    pub severity: Severity,

    /// Issue description
    pub description: String,

    /// Location in code
    pub location: CodeLocation,

    /// Suggested fixes
    pub suggested_fixes: Vec<String>,
}

/// Issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    /// Security vulnerability
    Security,

    /// Performance issue
    Performance,

    /// Code quality issue
    CodeQuality,

    /// Architecture issue
    Architecture,

    /// Maintainability issue
    Maintainability,

    /// Reliability issue
    Reliability,

    /// Documentation issue
    Documentation,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    /// Low impact issue
    Low,

    /// Medium impact issue
    Medium,

    /// High impact issue
    High,

    /// Critical impact issue
    Critical,
}

/// Optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    /// Opportunity type
    pub opportunity_type: OptimizationType,

    /// Description
    pub description: String,

    /// Expected improvement
    pub expected_improvement: f32,

    /// Difficulty level
    pub difficulty: Difficulty,

    /// Implementation complexity
    pub complexity: ImplementationComplexity,

    /// Location in code
    pub location: CodeLocation,
}

/// Optimization types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Memory optimization
    Memory,

    /// CPU optimization
    CPU,

    /// I/O optimization
    IO,

    /// Network optimization
    Network,

    /// Concurrency optimization
    Concurrency,

    /// Algorithm optimization
    Algorithm,
}

/// Difficulty levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    /// Easy to implement
    Easy,

    /// Medium difficulty
    Medium,

    /// Hard to implement
    Hard,

    /// Very challenging
    VeryHard,
}

/// Implementation complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationComplexity {
    /// Low complexity
    Low,

    /// Medium complexity
    Medium,

    /// High complexity
    High,

    /// Very high complexity
    VeryHigh,
}

/// Semantic relationship between code elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelationship {
    /// Relationship type
    pub relationship_type: RelationshipType,

    /// Source element location
    pub source_location: CodeLocation,

    /// Target element location
    pub target_location: CodeLocation,

    /// Strength of relationship (0-1)
    pub strength: f32,

    /// Relationship metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Relationship types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Function calls relationship
    Calls,

    /// Type usage relationship
    Uses,

    /// Implementation relationship
    Implements,

    /// Inheritance relationship
    InheritsFrom,

    /// Dependency relationship
    DependsOn,

    /// Association relationship
    AssociatesWith,

    /// Composition relationship
    ComposedOf,
}

/// Assistant request and response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantRequest {
    /// Request type
    pub request_type: AssistantRequestType,

    /// Request content
    pub content: String,

    /// Context information
    pub context: Option<DevelopmentContext>,

    /// User preferences
    pub preferences: Option<UserPreferences>,
}

/// Assistant request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssistantRequestType {
    /// Code explanation request
    ExplainCode,

    /// Code generation request
    GenerateCode,

    /// Code review request
    ReviewCode,

    /// Debugging assistance
    DebugHelp,

    /// Best practices guidance
    BestPractices,

    /// Architecture advice
    ArchitectureAdvice,

    /// Documentation help
    DocumentationHelp,

    /// Testing assistance
    TestingAdvice,

    /// General development question
    GeneralQuestion,

    /// Custom request type
    Custom(String),
}

/// Assistant response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantResponse {
    /// Response type
    pub response_type: AssistantResponseType,

    /// Response content
    pub content: String,

    /// Suggested actions
    pub suggested_actions: Vec<String>,

    /// Confidence level
    pub confidence: f32,

    /// Related resources
    pub resources: Vec<Resource>,

    /// Follow-up questions
    pub follow_up_questions: Vec<String>,
}

/// Assistant response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssistantResponseType {
    /// Explanatory response
    Explanation,

    /// Generated code response
    CodeGeneration,

    /// Review feedback response
    ReviewFeedback,

    /// Debugging guidance
    DebugGuidance,

    /// Best practices advice
    BestPractices,

    /// Architecture recommendation
    ArchitectureRecommendation,

    /// Documentation generation
    Documentation,

    /// Testing guidance
    TestingGuidance,

    /// Direct answer
    Answer,

    /// Clarification request
    ClarificationRequired,
}

/// Resource suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource type
    pub r#type: ResourceType,

    /// Resource title
    pub title: String,

    /// Resource URL or location
    pub location: String,

    /// Relevance score
    pub relevance: f32,
}

/// Resource types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// Documentation resource
    Documentation,

    /// Tutorial resource
    Tutorial,

    /// Example code
    Example,

    /// Reference implementation
    Reference,

    /// Tool or library
    Tool,

    /// Video resource
    Video,

    /// Article or blog post
    Article,

    /// Other type of resource
    Other(String),
}

/// Phase 4 event system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase4Event {
    /// Workflow started
    WorkflowStarted {
        workflow_id: String,
        workflow_type: WorkflowType,
    },

    /// Workflow completed
    WorkflowCompleted {
        workflow_id: String,
        status: WorkflowStatus,
        metrics: WorkflowMetrics,
    },

    /// New insight generated
    InsightGenerated {
        project_id: String,
        insight_type: String,
        importance: f32,
    },

    /// Assistant interaction
    AssistantInteraction {
        user_id: String,
        request_type: AssistantRequestType,
        response_type: AssistantResponseType,
    },

    /// Code analysis completed
    CodeAnalysisCompleted {
        file_path: String,
        analysis_score: f32,
        issues_found: usize,
    },

    /// System performance update
    PerformanceUpdate {
        component: String,
        metric_name: String,
        value: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Placeholder modules for cross-references
pub mod analysis {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AnalysisResult {
        pub score: f32,
        pub findings: Vec<String>,
        pub recommendations: Vec<String>,
    }
}

pub mod refactoring {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RefactorSuggestion {
        pub description: String,
        pub impact: f32,
        pub complexity: String,
        pub location: String,
    }
}

// Configuration traits
pub trait ConfigValidatable {
    fn validate_config(&self) -> std::result::Result<(), String>;
}

impl ConfigValidatable for AIWorkflowRequest {
    fn validate_config(&self) -> std::result::Result<(), String> {
        if self.id.is_empty() {
            return Err("Workflow ID cannot be empty".to_string());
        }
        if self.context.project.name.is_empty() {
            return Err("Project name cannot be empty".to_string());
        }
        Ok(())
    }
}

impl ConfigValidatable for DevelopmentInsights {
    fn validate_config(&self) -> std::result::Result<(), String> {
        if self.project_health.overall_score < 0.0 || self.project_health.overall_score > 100.0 {
            return Err("Project health score must be between 0 and 100".to_string());
        }
        Ok(())
    }
}