use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main trait for specification-driven code generation
#[async_trait::async_trait]
pub trait SpecificationGenerator: Send + Sync {
    /// Generate code from a specification request
    async fn generate_from_spec(
        &self,
        request: &SpecificationRequest,
    ) -> crate::error::Result<GeneratedCode>;

    /// Parse a natural language specification into a structured format
    async fn parse_specification(&self, text: &str) -> crate::error::Result<ParsedSpecification>;

    /// Generate a code template for a specific architectural pattern
    async fn generate_pattern(
        &self,
        pattern: &ArchitecturalPattern,
    ) -> crate::error::Result<GeneratedCode>;

    /// Validate generated code against the specification
    async fn validate_generation(
        &self,
        code: &str,
        spec: &ParsedSpecification,
    ) -> crate::error::Result<ValidationResult>;

    /// Refine generated code based on feedback
    async fn refine_generation(
        &self,
        code: &str,
        spec: &ParsedSpecification,
        feedback: &str,
    ) -> crate::error::Result<RefinedCode>;
}

/// Enhanced request for generating code from a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationRequest {
    /// Natural language description of the desired code
    pub description: String,
    /// Target programming language (e.g., "rust", "python", "javascript")
    pub language: String,
    /// Additional context or constraints
    pub context: Option<HashMap<String, String>>,
    /// Preferred architectural pattern
    pub preferred_pattern: Option<String>,
    /// Quality requirements (0.0-1.0)
    pub quality_threshold: Option<f32>,
    /// Performance requirements
    pub performance_requirements: Option<Vec<String>>,
    /// Security requirements
    pub security_requirements: Option<Vec<String>>,
}

/// Enhanced parsed specification containing structured requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSpecification {
    /// List of requirements extracted from the specification
    pub requirements: Vec<Requirement>,
    /// Detected architectural patterns
    pub patterns: Vec<ArchitecturalPattern>,
    /// Identified entities (structs, enums, etc.)
    pub entities: Vec<Entity>,
    /// Identified functions and their specifications
    pub functions: Vec<FunctionSpec>,
    /// Quality metrics derived from parsing
    pub quality_score: f32,
    /// Complexity assessment
    pub complexity: ComplexityAssessment,
    /// Security considerations
    pub security_considerations: Vec<String>,
}

/// Complexity assessment of the specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAssessment {
    /// Overall complexity score (0.0-10.0)
    pub score: f32,
    /// Number of entities
    pub entity_count: usize,
    /// Number of functions
    pub function_count: usize,
    /// Number of requirements
    pub requirement_count: usize,
    /// Estimated development effort in hours
    pub estimated_effort_hours: f32,
}

/// Enhanced requirement with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Unique identifier for the requirement
    pub id: String,
    /// The requirement text
    pub description: String,
    /// Priority level (1-5, with 1 being highest)
    pub priority: u8,
    /// Related entities or functions
    pub related_to: Vec<String>,
    /// Functional category
    pub category: RequirementCategory,
    /// Testable acceptance criteria
    pub acceptance_criteria: Vec<String>,
}

/// Category of requirement for better classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementCategory {
    Functional,
    NonFunctional,
    Security,
    Performance,
    Usability,
    Maintainability,
}

/// Enhanced entity with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Name of the entity
    pub name: String,
    /// Type of entity (struct, enum, trait, etc.)
    pub entity_type: EntityType,
    /// Fields or variants of the entity
    pub fields: Vec<Field>,
    /// Documentation comments
    pub docs: Vec<String>,
    /// Associated requirements
    pub requirements: Vec<String>,
    /// Visibility level
    pub visibility: Visibility,
    /// Relationships to other entities
    pub relationships: Vec<EntityRelationship>,
}

/// Visibility level for entities and their members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Crate,
    Module,
    Private,
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelationship {
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// The related entity name
    pub target_entity: String,
    /// Cardinality (e.g., "1", "0..*", "1..n")
    pub cardinality: String,
    /// Direction of relationship
    pub direction: RelationshipDirection,
}

/// Types of entity relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Association,
    Aggregation,
    Composition,
    Generalization,
    Dependency,
    Realization,
}

/// Direction of relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipDirection {
    Unidirectional,
    Bidirectional,
}

/// Type of entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    Struct,
    Enum,
    Trait,
    Module,
    TypeAlias,
    Union,
    Const,
    Static,
}

/// Enhanced field with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Name of the field
    pub name: String,
    /// Type of the field
    pub field_type: String,
    /// Whether the field is optional
    pub is_optional: bool,
    /// Documentation comments
    pub docs: Vec<String>,
    /// Visibility of the field
    pub visibility: Visibility,
    /// Default value if any
    pub default_value: Option<String>,
    /// Whether the field should be skipped in serialization
    pub skip_serialize: bool,
    /// Validation constraints
    pub validation: Vec<String>,
}

/// Enhanced function specification with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    /// Function name
    pub name: String,
    /// Return type (empty for unit)
    pub return_type: String,
    /// List of parameters
    pub parameters: Vec<Parameter>,
    /// Documentation comments
    pub docs: Vec<String>,
    /// Associated requirements
    pub requirements: Vec<String>,
    /// Error types that might be returned
    pub error_types: Vec<String>,
    /// Function visibility
    pub visibility: Visibility,
    /// Whether the function is async
    pub is_async: bool,
    /// Whether the function is const
    pub is_const: bool,
    /// Whether the function is unsafe
    pub is_unsafe: bool,
    /// Complexity metrics
    pub complexity: FunctionComplexity,
}

/// Complexity metrics for functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexity {
    /// Cyclomatic complexity score
    pub cyclomatic_complexity: u32,
    /// Number of parameters
    pub parameter_count: usize,
    /// Number of local variables
    pub local_variable_count: usize,
    /// Estimated execution time complexity
    pub time_complexity: String,
    /// Estimated space complexity
    pub space_complexity: String,
}

impl Default for FunctionComplexity {
    fn default() -> Self {
        Self {
            cyclomatic_complexity: 1,
            parameter_count: 0,
            local_variable_count: 0,
            time_complexity: "O(1)".to_string(),
            space_complexity: "O(1)".to_string(),
        }
    }
}

/// Enhanced parameter with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Whether the parameter is mutable
    pub is_mut: bool,
    /// Whether the parameter is a reference
    pub is_ref: bool,
    /// Lifecycle of reference (if applicable)
    pub lifetime: Option<String>,
    /// Documentation for the parameter
    pub docs: Vec<String>,
    /// Validation constraints
    pub validation: Vec<String>,
}

/// Enhanced architectural pattern with more metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalPattern {
    /// Name of the pattern (e.g., "Repository", "CQRS")
    pub name: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Description of the pattern
    pub description: String,
    /// Related components
    pub components: Vec<PatternComponent>,
    /// Benefits of using this pattern
    pub benefits: Vec<String>,
    /// Trade-offs and considerations
    pub tradeoffs: Vec<String>,
    /// Implementation complexity (1-5)
    pub complexity_level: u8,
    /// Use cases where this pattern is appropriate
    pub use_cases: Vec<String>,
}

/// Enhanced pattern component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternComponent {
    /// Role of the component in the pattern
    pub role: String,
    /// Name of the component
    pub name: String,
    /// Type of the component
    pub component_type: String,
    /// Responsibilities of this component
    pub responsibilities: Vec<String>,
    /// Dependencies of this component
    pub dependencies: Vec<String>,
}

/// Enhanced generated code result with more comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    /// Main code files
    pub files: Vec<CodeFile>,
    /// Additional resources (configs, assets, etc.)
    pub resources: Vec<ResourceFile>,
    /// Build instructions
    pub build_instructions: Vec<String>,
    /// Next steps or TODOs
    pub next_steps: Vec<String>,
    /// Documentation files
    pub documentation: Vec<DocumentationFile>,
    /// Test files
    pub tests: Vec<CodeFile>,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// Security analysis results
    pub security_analysis: Option<SecurityAnalysis>,
}

/// Quality metrics for generated code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall quality score (0.0-1.0)
    pub overall_score: f32,
    /// Maintainability index
    pub maintainability_index: f32,
    /// Cyclomatic complexity
    pub complexity_score: f32,
    /// Documentation coverage (0.0-1.0)
    pub documentation_coverage: f32,
    /// Test coverage (0.0-1.0)
    pub test_coverage: f32,
}

/// Security analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Security score (0.0-1.0, higher is better)
    pub security_score: f32,
    /// Identified vulnerabilities
    pub vulnerabilities: Vec<String>,
    /// Security recommendations
    pub recommendations: Vec<String>,
    /// Compliance status
    pub compliance_status: Vec<String>,
}

/// Documentation file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationFile {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
    /// Documentation format (markdown, html, etc.)
    pub format: DocumentationFormat,
    /// Type of documentation (api, user, architecture, etc.)
    pub doc_type: DocumentationType,
}

/// Documentation format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationFormat {
    Markdown,
    HTML,
    PDF,
    JSON,
    YAML,
}

/// Type of documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationType {
    API,
    UserGuide,
    Architecture,
    TechnicalSpec,
    Testing,
    Deployment,
}

/// Enhanced code file with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    /// File path relative to project root
    pub path: String,
    /// File content
    pub content: String,
    /// Whether the file is a test file
    pub is_test: bool,
    /// File type/language
    pub file_type: FileType,
    /// Lines of code
    pub lines_of_code: usize,
    /// Complexity score for this file
    pub complexity_score: f32,
}

/// File type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    SourceCode,
    Test,
    Configuration,
    Documentation,
    BuildScript,
}

/// Enhanced resource file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFile {
    /// File path relative to project root
    pub path: String,
    /// File content
    pub content: String,
    /// Resource type
    pub resource_type: ResourceType,
}

/// Resource type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Configuration,
    Template,
    Data,
    Asset,
}

/// Enhanced validation result with detailed issue tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// List of issues found
    pub issues: Vec<crate::error::ValidationIssue>,
    /// Overall score (0.0 to 1.0)
    pub score: f32,
    /// Issues grouped by category
    pub issues_by_category: HashMap<ValidationCategory, Vec<crate::error::ValidationIssue>>,
    /// Blocking issues that prevent generation
    pub blocking_issues: Vec<crate::error::ValidationIssue>,
}

/// Validation category (re-export for consistency)
pub use crate::error::ValidationCategory;

/// Severity level for validation issues (re-export)
pub use crate::error::ValidationSeverity;

/// Enhanced refined code with better change tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedCode {
    /// The refined code
    pub code: String,
    /// List of changes made
    pub changes: Vec<CodeChange>,
    /// Explanation of the changes
    pub explanation: String,
    /// Quality improvement metrics
    pub improvement_metrics: ImprovementMetrics,
    /// Code review comments
    pub review_comments: Vec<String>,
}

/// Improvement metrics for code refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementMetrics {
    /// Quality score before/after
    pub quality_before: f32,
    pub quality_after: f32,
    /// Complexity before/after
    pub complexity_before: f32,
    pub complexity_after: f32,
    /// Issues resolved
    pub issues_resolved: usize,
    /// New issues introduced
    pub new_issues: usize,
}

/// Enhanced code change with more detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Location of the change (file:line:column)
    pub location: String,
    /// Old content (if applicable)
    pub old_content: Option<String>,
    /// New content (if applicable)
    pub new_content: Option<String>,
    /// Justification for the change
    pub justification: String,
    /// Impact assessment
    pub impact: String,
}

/// Type of code change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// Added new code
    Addition,
    /// Modified existing code
    Modification,
    /// Deleted code
    Deletion,
    /// Moved code
    Move,
    /// Refactored code without changing behavior
    Refactoring,
    /// Fixed a bug or issue
    BugFix,
}
