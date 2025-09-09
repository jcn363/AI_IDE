//! # Core Data Structures and Type Definitions for the Architectural Advisor
//!
//! This module defines the core data structures that enable the AI-powered architectural
//! analysis system. The architectures support both traditional software engineering
//! metrics and machine learning-enhanced pattern detection algorithms.
//!
//! ## Key Capabilities
//!
//! - **Multi-dimensional Assessment**: Combines cyclomatic complexity, coupling/cohesion analysis,
//!   and maintainability metrics into comprehensive architectural insights
//! - **ML-Enhanced Pattern Detection**: Uses confidence scoring (0.0-1.0) to identify architectural
//!   patterns with probabilistic accuracy rather than binary classification
//! - **Risk-Aware Decision Making**: Integrates risk assessment into all recommendations with
//!   quantified probability and impact metrics
//! - **Evolutionary Planning**: Supports short/medium/long-term architectural roadmaps with
//!   dependency tracking and success criteria
//!
//! ## Architecture Assessment Pipeline
//!
//! The system follows a structured analytical pipeline:
//!
//! 1. **Static Analysis**: Basic code metrics (LOC, complexity, dependencies)
//! 2. **Pattern Recognition**: ML-enhanced detection of architectural patterns
//! 3. **Quality Assessment**: Multi-factor maintainability and quality scoring
//! 4. **Risk Evaluation**: Probabilistic risk assessment for recommendations
//! 5. **Decision Synthesis**: Weighted decision making with trade-off analysis
//!
//! ```rust
//! use rust_ai_ide_ai::architectural_advisor::*;
//!
//! // Advanced architectural assessment
//! let advisor = create_architectural_advisor();
//! let high_detail_config = ArchitecturalAdvisorConfig {
//!     confidence_threshold: 0.6,
//!     quality_threshold: 75.0,
//!     detailed_analysis: true,
//!     enable_anti_pattern_detection: true,
//!     enable_pattern_detection: true,
//! };
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project types supported by the architectural advisor system
///
/// Each project type influences the pattern recognition algorithms and
/// determines appropriate architectural templates and best practices.
/// The ML model is trained on patterns specific to each category to provide
/// contextually relevant recommendations.
///
/// # Project-Specific Analysis
///
/// - **Library**: Focuses on API design, dependency management, and reusability patterns
/// - **Application**: Emphasizes user experience, scalability, and deployment architectures
/// - **WebService**: Prioritizes API design, concurrency patterns, and fault tolerance
/// - **CLI**: Concentrates on interface design and error handling patterns
/// - **Embedded**: Focuses on resource optimization and real-time constraints
/// - **Game**: Emphasizes performance, modularity, and asset management
/// - **System**: Addresses distributed systems patterns and infrastructure concerns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Library,
    Application,
    WebService,
    CLI,
    Embedded,
    Game,
    System,
}

/// Current architecture description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentArchitecture {
    pub layers: Vec<ArchitectureLayer>,
    pub patterns_used: Vec<String>,
    pub technologies: Vec<String>,
    pub deployment_model: String,
}

/// Architecture layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureLayer {
    pub name: String,
    pub responsibilities: Vec<String>,
    pub technologies: Vec<String>,
    pub interfaces: Vec<String>,
}

/// Architectural context for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalContext {
    pub codebase_path: String,
    pub project_type: ProjectType,
    pub current_architecture: Option<CurrentArchitecture>,
    pub constraints: Vec<String>,
    pub goals: Vec<String>,
    pub team_size: Option<usize>,
    pub expected_lifecycle: Option<String>,
}

/// Comprehensive pattern analysis result synthesizing multiple analysis dimensions
///
/// This structure represents the result of the multi-phase architectural analysis
/// pipeline that combines machine learning pattern recognition with traditional
/// software metrics analysis.
///
/// # Analysis Phases
///
/// The analysis follows a structured approach:
/// 1. **Pattern Detection**: ML-enhanced identification of architectural patterns
/// 2. **Quality Assessment**: Traditional metrics combined with ML insights
/// 3. **Complexity Evaluation**: Cyclomatic and cognitive complexity analysis
/// 4. **Interconnectivity Analysis**: Coupling and cohesion measurements
/// 5. **Synthesis**: AI-powered correlation and insight generation
///
/// # Confidence Scoring
///
/// All pattern detections include probabilistic confidence scores (0.0-1.0) rather
/// than binary classifications to reflect the uncertainty inherent in architectural
/// analysis. This approach enables more nuanced decision-making and risk assessment.
///
/// ```rust
/// use rust_ai_ide_ai::architectural_advisor::*;
///
/// # async fn example() {
/// let analysis = quick_analyze("/path/to/project").await?;
///
/// // High confidence patterns (> 0.8) are most reliable
/// let high_confidence_patterns: Vec<_> = analysis.detected_patterns
///     .iter()
///     .filter(|p| p.confidence > 0.8)
///     .collect();
///
/// // Anti-patterns indicate areas needing immediate attention
/// if analysis.anti_patterns.len() > 0 {
///     println!("Found {} architectural concerns", analysis.anti_patterns.len());
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    /// Successfully identified architectural patterns with confidence scores
    pub detected_patterns: Vec<DetectedPattern>,

    /// Architectural concerns and anti-patterns detected in the codebase
    pub anti_patterns: Vec<AntiPattern>,

    /// Multi-dimensional quality assessment combining traditional and ML metrics
    pub quality_metrics: QualityMetrics,

    /// Complexity evaluation including cognitive and cyclomatic complexity
    pub complexity_assessment: ComplexityAssessment,

    /// Module coupling analysis measuring interdependencies
    pub coupling_analysis: CouplingAnalysis,

    /// Cohesion analysis measuring module internal consistency
    pub cohesion_analysis: CohesionAnalysis,
}

/// Detected architectural pattern with ML-powered confidence scoring
///
/// Represents a successfully identified architectural pattern using machine learning
/// approaches rather than rule-based detection. The confidence score reflects the
/// probability that the pattern detection is accurate, enabling risk-informed
/// decision making in architectural recommendations.
///
/// # Pattern Detection Algorithm
///
/// The pattern detection employs:
/// - **Feature Extraction**: Code structure, call patterns, data flow analysis
/// - **Similarity Matching**: Vector space model comparison against known patterns
/// - **Context Analysis**: Project type, technology stack, and domain considerations
/// - **Confidence Calibration**: Probabilistic scoring based on multiple heuristics
///
/// ## Confidence Score Interpretation
///
/// - **0.9-1.0**: Very high confidence, pattern is extremely likely present
/// - **0.7-0.9**: High confidence, pattern detection is reliable
/// - **0.5-0.7**: Moderate confidence, further investigation recommended
/// - **0.3-0.5**: Low confidence, may be coincidental matches
/// - **0.0-0.3**: Very low confidence, likely false positives
///
/// ```rust
/// use rust_ai_ide_ai::architectural_advisor::*;
///
/// fn analyze_pattern_confidence(pattern: &DetectedPattern) {
///     match pattern.confidence {
///         c if c >= 0.9 => println!("✓ Strong evidence of {} pattern", pattern.pattern_type),
///         c if c >= 0.7 => println!("⚠ Moderate confidence in {} pattern detection", pattern.pattern_type),
///         c if c >= 0.5 => println!("? Further investigation needed for {}", pattern.pattern_type),
///         _ => println!("✗ Low confidence in pattern detection"),
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// The type/identifier of the architectural pattern (e.g., "MVVM", "Observer", "Factory")
    pub pattern_type: String,

    /// Probabilistic confidence score (0.0-1.0) indicating detection reliability
    /// Higher values indicate stronger evidence for the pattern's presence
    pub confidence: f32,

    /// Precise location information where the pattern was detected
    pub location: PatternLocation,

    /// Human-readable description of the pattern and its context
    pub description: String,

    /// Expected benefits of using or maintaining this architectural pattern
    pub benefits: Vec<String>,

    /// Specific areas where this pattern could be beneficially applied
    pub applications: Vec<String>,
}

/// Pattern location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternLocation {
    pub files: Vec<String>,
    pub modules: Vec<String>,
    pub lines: Option<(usize, usize)>,
}

/// Anti-pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub anti_pattern_type: String,
    pub severity: f32,
    pub location: PatternLocation,
    pub description: String,
    pub consequences: Vec<String>,
    pub refactoring_suggestions: Vec<String>,
}

/// Multi-dimensional quality metrics combining traditional and AI-enhanced analysis
///
/// This structure represents a comprehensive quality assessment that integrates
/// classical software metrics with AI-powered insights. The approach provides
/// both quantitative measurements and qualitative risk assessments.
///
/// # Metric Categories
///
/// ## Traditional Software Metrics
/// - **Maintainability Index**: Composite measure combining cyclomatic complexity,
///   lines of code, and Halstead complexity into a single maintainability score
/// - **Cyclomatic Complexity**: Measures the number of linearly independent paths
///   through a program's source code, indicating testing difficulty and understandability
/// - **Halstead Complexity**: Based on operator/operand counts to measure complexity
///   in terms of program vocabulary and implementation
///
/// ## AI-Enhanced Metrics
/// - **Technical Debt Ratio**: Predicted maintenance burden based on code patterns,
///   complexity trends, and architectural inconsistencies
/// - **Test Coverage Integration**: When available, factors into overall quality scoring
///
/// ## Quality Assessment Formula
///
/// The system uses a weighted formula to combine metrics:
/// ```
/// overall_quality = (maintainability_index * 0.4) +
///                   (technical_debt_ratio * 0.35) +
///                   (test_coverage_weight * 0.25)
/// ```
///
/// # Interpretation Guidelines
///
/// ## Maintainability Index Ranges
/// - **85-100**: Highly maintainable code
/// - **65-84**: Moderately maintainable code
/// - **0-64**: Poorly maintainable code requiring significant refactoring
///
/// ## Technical Debt Ratio
/// - **< 0.1**: Low technical debt, code is in good health
/// - **0.1-0.3**: Moderate technical debt, consider targeted refactoring
/// - **> 0.3**: High technical debt, structural improvements needed
///
/// ```rust
/// use rust_ai_ide_ai::architectural_advisor::*;
///
/// fn interpret_quality_metrics(metrics: &QualityMetrics) {
///     // Comprehensive quality assessment
///     match metrics.maintainability_index {
///         m if m > 85.0 => println!("✓ Code is highly maintainable"),
///         m if m > 65.0 => println!("⚠ Consider gradual improvements"),
///         _ => println!("⚠ Refactoring recommended"),
///     }
///
///     if let Some(coverage) = metrics.test_coverage {
///         match coverage {
///             c if c > 0.8 => println!("✓ Good test coverage ({:.1}%)", c * 100.0),
///             c if c > 0.6 => println!("⚠ Moderate test coverage ({:.1}%)", c * 100.0),
///             _ => println!("⚠ Insufficient test coverage"),
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Maintainability Index (0-100) - composite measure of code maintainability
    /// Higher values indicate more maintainable code
    /// Calculated using cyclomatic complexity, LOC, and Halstead metrics
    pub maintainability_index: f32,

    /// Cyclomatic complexity score - measures number of independent paths
    /// Lower values indicate simpler, more testable code
    pub cyclomatic_complexity: f32,

    /// Halstead complexity measure - based on operator/operand analysis
    /// Measures program complexity in terms of vocabulary and implementation
    pub halstead_complexity: f32,

    /// Total lines of code across the analyzed codebase
    /// Used as a baseline for complexity ratios and productivity metrics
    pub lines_of_code: usize,

    /// Technical debt ratio (0-1) - predicted maintenance burden relative to value
    /// Lower values indicate healthier codebase with less accumulated debt
    pub technical_debt_ratio: f32,

    /// Test coverage percentage (0-1) - when available from test frameworks
    /// Factors into overall quality assessment as reliability indicator
    pub test_coverage: Option<f32>,
}

/// Comprehensive complexity assessment integrating multiple analysis techniques
///
/// This structure represents the result of advanced complexity analysis that combines
/// traditional software metrics with AI-powered algorithmic complexity detection.
/// The assessment identifies specific complexity hotspots and trends over time.
///
/// # Complexity Analysis Methodology
///
/// The system uses a multi-factor complexity assessment:
///
/// 1. **Cyclomatic Complexity**: McCabe's complexity measurement for control flow
/// 2. **Cognitive Complexity**: Cognitive load assessment based on code readability
/// 3. **Data Flow Complexity**: Complexity arising from data transformations
/// 4. **Temporal Complexity**: Complexity introduced by async/parallel code
/// 5. **Dependency Complexity**: Complex coupling and import relationships
///
/// # Hotspot Detection Algorithm
///
/// Complexity hotspots are identified using a weighted combination of metrics:
/// ```
/// hotspot_score = (cyclomatic_weight * 0.3) +
///                 (cognitive_weight * 0.3) +
///                 (dependency_weight * 0.2) +
///                 (change_frequency * 0.2)
/// ```
///
/// Where change frequency is derived from commit history and refactoring patterns.
///
/// # Trend Analysis
///
/// The system analyzes complexity trends by comparing against:
/// - Historical baselines
/// - Industry benchmarks for similar projects
/// - Project-specific complexity targets
/// - Architectural complexity budgets
///
/// ```rust
/// use rust_ai_ide_ai::architectural_advisor::*;
///
/// fn analyze_complexity_hotspots(assessment: &ComplexityAssessment) {
///     for hotspot in &assessment.hotspot_complexity {
///         println!("⚠ High complexity in {}: score {:.2}",
///                 hotspot.file, hotspot.complexity_score);
///
///         // Priority based on complexity level
///         match hotspot.complexity_score {
///             s if s > 50.0 => println!("🔴 Critical - immediate refactoring needed"),
///             s if s > 30.0 => println!("🟠 High - plan for refactoring"),
///             s if s > 20.0 => println!("🟡 Medium - monitor and consider refactoring"),
///             _ => println!("🟢 Low - acceptable complexity"),
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAssessment {
    /// Overall complexity classification derived from all hotspot aggregations
    pub overall_complexity: ComplexityLevel,

    /// Specific files or modules with elevated complexity scores
    pub hotspot_complexity: Vec<ComplexityHotspot>,

    /// Historical complexity trends showing evolution over time
    pub complexity_trends: Vec<ComplexityTrend>,
}

/// Complexity classification levels with recommended action thresholds
///
/// These levels provide standardized interpretation guidelines for complexity
/// assessment results, helping teams make consistent decisions about refactoring
/// priorities and technical debt management.
///
/// # Action Guidelines by Level
///
/// - **Low**: Standard development practices sufficient
/// - **Moderate**: Consider targeted complexity reduction
/// - **High**: Implement structured complexity management
/// - **VeryHigh**: Immediate architectural intervention required
///
/// # Complexity Threshold Recommendations
///
/// - **Individual Function**: Cyclomatic complexity should not exceed 10-15
/// - **Module Level**: Cognitive complexity should not exceed 25-30
/// - **System Level**: Overall complexity score should not exceed 40
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// Code is within acceptable complexity bounds
    Low,

    /// Moderate complexity requiring occasional attention
    Moderate,

    /// High complexity necessitating systematic management
    High,

    /// Excessive complexity requiring immediate restructuring
    VeryHigh,
}

/// Complexity hotspot in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHotspot {
    pub file: String,
    pub complexity_score: f32,
    pub description: String,
}

/// Complexity trend over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityTrend {
    pub period: String,
    pub complexity_change: f32,
    pub description: String,
}

/// Coupling analysis between modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingAnalysis {
    pub afferent_coupling: HashMap<String, usize>, // Number of incoming dependencies
    pub efferent_coupling: HashMap<String, usize>, // Number of outgoing dependencies
    pub instability: HashMap<String, f32>,
    pub abstractness: HashMap<String, f32>,
    pub distance_from_main: HashMap<String, f32>,
}

/// Cohesion analysis for modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohesionAnalysis {
    pub lack_of_cohesion: HashMap<String, f32>,
    pub functional_cohesion: HashMap<String, f32>,
}

/// Comprehensive architectural guidance incorporating AI-powered decision analysis
///
/// This structure represents the culmination of the architectural analysis pipeline,
/// providing actionable guidance synthesized from pattern recognition, quality metrics,
/// and risk assessment. The guidance is structured hierarchically to reflect different
/// priority levels and implementation timeframes.
///
/// # Decision Synthesis Process
///
/// The guidance generation follows a multi-step AI-enhanced process:
///
/// 1. **Evidence Aggregation**: Combines all analysis results (patterns, metrics, risks)
/// 2. **Correlation Analysis**: Identifies relationships between detected issues
/// 3. **Impact Assessment**: Predicts outcome probabilities for each recommendation
/// 4. **Priority Ranking**: ML-weighted ranking based on impact and implementation effort
/// 5. **Risk Mitigation**: Incorporates probability-weighted risk reduction strategies
/// 6. **Roadmap Generation**: Creates implementation timelines with dependency tracking
///
/// # Recommendation Classification
///
/// Recommendations are classified by scope and impact:
/// - **Primary Recommendations**: High-confidence, high-impact architectural changes
/// - **Secondary Suggestions**: Lower-risk improvements or alternative approaches
/// - **Priority Actions**: Immediate actions with specific deadlines and responsibilities
/// - **Roadmap Items**: Phased implementations with success criteria and dependencies
///
/// ## Decision Confidence Factors
///
/// Confidence scores consider:
/// - Pattern detection accuracy (ML model reliability)
/// - Historical outcome data for similar recommendations
/// - Risk assessment validity and probability calibration
/// - Implementation complexity estimates from similar projects
/// - Stakeholder alignment and organizational constraints
///
/// ```rust
/// use rust_ai_ide_ai::architectural_advisor::*;
///
/// fn prioritize_recommendations(guidance: &ArchitecturalGuidance) {
///     // Immediate high-priority actions
///     for action in &guidance.priority_actions {
///         println!("🔥 PRIORITY: {}", action.action);
///         if let Some(deadline) = &action.deadline {
///             println!("   Due: {}", deadline);
///         }
///     }
///
///     // Primary architectural recommendations
///     for rec in &guidance.primary_recommendations {
///         match rec.implementation_effort {
///             ImplementationEffort::Low => println!("✅ Easy win: {}", rec.title),
///             ImplementationEffort::Medium => println!("⚠ Strategic: {}", rec.title),
///             _ => println!("🏗️ Major: {}", rec.title),
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalGuidance {
    /// High-confidence, high-impact architectural recommendations requiring careful planning
    pub primary_recommendations: Vec<ArchitecturalRecommendation>,

    /// Additional suggestions for incremental improvements or alternative approaches
    pub secondary_suggestions: Vec<ArchitecturalSuggestion>,

    /// Comprehensive risk assessment with mitigation strategies
    pub risk_assessment: RiskAssessment,

    /// Immediate priority actions with deadlines, dependencies, and responsibilities
    pub priority_actions: Vec<PriorityAction>,

    /// Strategic roadmap with short-term, medium-term, and long-term planning
    pub roadmap: ArchitecturalRoadmap,
}

/// Architectural recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalRecommendation {
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub expected_benefits: Vec<String>,
    pub implementation_effort: ImplementationEffort,
    pub risk_level: RiskLevel,
    pub prerequisites: Vec<String>,
    pub alternatives: Vec<String>,
}

/// Architectural suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalSuggestion {
    pub title: String,
    pub description: String,
    pub category: SuggestionCategory,
    pub priority: PriorityLevel,
    pub impact: ImpactLevel,
}

/// Suggestion categories for architectural improvements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Refactoring,
    DesignPattern,
    TechnologyChoice,
    ArchitectureEvolution,
    Performance,
    Scalability,
    Security,
    Maintainability,
}

/// Priority levels for suggestions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum PriorityLevel {
    High,
    Medium,
    Low,
}

/// Impact levels for architectural changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ImpactLevel {
    Critical,
    Major,
    Moderate,
    Minor,
}

/// Implementation effort required
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Risk level for implementation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk assessment for architectural decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: f32,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_strategies: Vec<String>,
}

/// Risk factor in architectural assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor: String,
    pub probability: f32,
    pub impact: f32,
    pub description: String,
}

/// Priority action for implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityAction {
    pub action: String,
    pub deadline: Option<String>,
    pub responsible: Option<String>,
    pub dependencies: Vec<String>,
}

/// Architectural roadmap for changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalRoadmap {
    pub short_term: Vec<RoadmapItem>,  // Next 3-6 months
    pub medium_term: Vec<RoadmapItem>, // Next 6-12 months
    pub long_term: Vec<RoadmapItem>,   // Beyond 12 months
}

/// Roadmap item for architectural planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapItem {
    pub title: String,
    pub description: String,
    pub timeline: String,
    pub dependencies: Vec<String>,
    pub success_criteria: Vec<String>,
}

/// Decision option for architectural decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub title: String,
    pub description: String,
    pub alternatives: Vec<String>,
    pub criteria: Vec<DecisionCriterion>,
    pub context: String,
}

/// Decision criterion for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionCriterion {
    pub name: String,
    pub weight: f32,
    pub description: String,
}

/// Decision analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionAnalysis {
    pub decision: DecisionOption,
    pub recommendation: DecisionRecommendation,
    pub analysis: HashMap<String, f32>, // Criterion -> Score mapping
    pub trade_offs: Vec<TradeOff>,
    pub risks: Vec<String>,
    pub assumptions: Vec<String>,
}

/// Decision recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecommendation {
    pub recommended_option: String,
    pub confidence: f32,
    pub rationale: Vec<String>,
    pub alternatives_considered: Vec<String>,
}

/// Trade-off analysis between options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOff {
    pub option1: String,
    pub option2: String,
    pub advantages_option1: Vec<String>,
    pub advantages_option2: Vec<String>,
    pub disadvantages_option1: Vec<String>,
    pub disadvantages_option2: Vec<String>,
}

/// Architectural document containing system documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalDocument {
    pub overview: ArchitecturalOverview,
    pub components: Vec<ComponentDocument>,
    pub patterns: Vec<PatternDocument>,
    pub interfaces: Vec<InterfaceDocument>,
    pub decisions: Vec<DecisionRecord>,
    pub quality_attributes: QualityAttributesDocument,
    pub deployment: DeploymentDocument,
}

/// Architectural pattern enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchitecturalPattern {
    /// Model-View-Controller pattern
    Mvc,
    /// Model-View-ViewModel pattern
    Mvvm,
    /// Service-Oriented Architecture
    Soa,
    /// Microservices pattern
    Microservices,
    /// Layered architecture
    Layered,
    /// Event-driven architecture
    EventDriven,
    /// Hexagonal architecture
    Hexagonal,
    /// Clean architecture
    Clean,
}

/// Architectural overview document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalOverview {
    pub description: String,
    pub purpose: String,
    pub scope: String,
    pub assumptions: Vec<String>,
    pub constraints: Vec<String>,
    pub goals: Vec<String>,
}

/// Component documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDocument {
    pub name: String,
    pub description: String,
    pub responsibilities: Vec<String>,
    pub interfaces: Vec<String>,
    pub dependencies: Vec<String>,
    pub technologies: Vec<String>,
}

/// Pattern documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDocument {
    pub pattern_name: String,
    pub description: String,
    pub context: String,
    pub problem: String,
    pub solution: String,
    pub consequences: Vec<String>,
    pub examples: Vec<String>,
}

/// Interface documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDocument {
    pub name: String,
    pub description: String,
    pub methods: Vec<InterfaceMethod>,
    pub protocols: Vec<String>,
    pub data_formats: Vec<String>,
}

/// Interface method documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMethod {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub parameters: Vec<MethodParameter>,
    pub return_type: String,
}

/// Method parameter documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodParameter {
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub optional: bool,
}

/// Decision record (ADR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub title: String,
    pub date: String,
    pub status: DecisionStatus,
    pub context: String,
    pub decision: String,
    pub consequences: Vec<String>,
    pub alternatives: Vec<String>,
}

/// Decision status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superseded,
}

/// Quality attributes documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAttributesDocument {
    pub attributes: Vec<QualityAttribute>,
    pub scenarios: Vec<QualityScenario>,
    pub metrics: Vec<QualityMetric>,
}

/// Quality attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAttribute {
    pub name: String,
    pub description: String,
    pub importance: f32,
    pub measures: Vec<String>,
    pub stakeholders: Vec<String>,
}

/// Quality scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScenario {
    pub stimulus: String,
    pub environment: String,
    pub response: String,
    pub measure: String,
}

/// Quality metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetric {
    pub name: String,
    pub description: String,
    pub formula: String,
    pub target_value: f32,
    pub current_value: Option<f32>,
}

/// Deployment documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentDocument {
    pub environments: Vec<DeploymentEnvironment>,
    pub topologies: Vec<String>,
    pub requirements: DeploymentRequirements,
    pub procedures: DeploymentProcedures,
}

/// Deployment environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEnvironment {
    pub name: String,
    pub purpose: String,
    pub configuration: HashMap<String, String>,
    pub scaling_properties: Vec<String>,
}

/// Deployment requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequirements {
    pub hardware: Vec<String>,
    pub software: Vec<String>,
    pub network: Vec<String>,
    pub security: Vec<String>,
}

/// Deployment procedures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentProcedures {
    pub preparation: Vec<String>,
    pub deployment: Vec<String>,
    pub rollback: Vec<String>,
    pub monitoring: Vec<String>,
}

/// Result type for architectural advisor operations
pub type AdvisorResult<T> = Result<T, AdvisorError>;

/// Custom error type for architectural advisor
#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum AdvisorError {
    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("Invalid context: {0}")]
    InvalidContextError(String),

    #[error("Pattern detection error: {0}")]
    PatternDetectionError(String),

    #[error("Recommendation generation error: {0}")]
    RecommendationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

// Manual implementation to convert std::io::Error to String
impl From<std::io::Error> for AdvisorError {
    fn from(err: std::io::Error) -> Self {
        AdvisorError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_serialization() {
        let project = ProjectType::Library;
        let serialized = serde_json::to_string(&project).unwrap();
        let deserialized: ProjectType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(project, deserialized);
    }

    #[test]
    fn test_architectural_context_creation() {
        let context = ArchitecturalContext {
            codebase_path: "src/".to_string(),
            project_type: ProjectType::Application,
            current_architecture: None,
            constraints: vec!["Performance".to_string()],
            goals: vec!["Scalability".to_string()],
            team_size: Some(5),
            expected_lifecycle: Some("3 years".to_string()),
        };

        assert_eq!(context.project_type, ProjectType::Application);
        assert_eq!(context.constraints.len(), 1);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::High < RiskLevel::Critical);
        assert!(PriorityLevel::Low < PriorityLevel::High);
    }

    #[test]
    fn test_advisor_error_formatting() {
        let error = AdvisorError::AnalysisError("Test analysis failed".to_string());
        let error_string = error.to_string();
        assert!(error_string.contains("Analysis error"));
        assert!(error_string.contains("Test analysis failed"));
    }
}
