use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Main trait for specification-driven code generation
#[async_trait]
pub trait SpecificationGenerator: Send + Sync {
    /// Generate code from a specification request
    async fn generate_from_spec(&self, request: &SpecificationRequest) -> Result<GeneratedCode>;

    /// Parse a natural language specification into a structured format
    async fn parse_specification(&self, text: &str) -> Result<ParsedSpecification>;

    /// Generate a code template for a specific architectural pattern
    async fn generate_pattern(&self, pattern: &ArchitecturalPattern) -> Result<GeneratedCode>;

    /// Validate generated code against the specification
    async fn validate_generation(&self, code: &str, spec: &ParsedSpecification) -> Result<ValidationResult>;

    /// Refine generated code based on feedback
    async fn refine_generation(&self, code: &str, spec: &ParsedSpecification, feedback: &str) -> Result<RefinedCode>;
}

/// Request for generating code from a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationRequest {
    /// Natural language description of the desired code
    pub description: String,
    /// Target programming language (e.g., "rust")
    pub language:    String,
    /// Additional context or constraints
    pub context:     Option<HashMap<String, String>>,
}

/// Parsed specification containing structured requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSpecification {
    /// List of requirements extracted from the specification
    pub requirements: Vec<Requirement>,
    /// Detected architectural patterns
    pub patterns:     Vec<ArchitecturalPattern>,
    /// Identified entities (structs, enums, etc.)
    pub entities:     Vec<Entity>,
    /// Identified functions and their specifications
    pub functions:    Vec<FunctionSpec>,
}

/// A single requirement from the specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Unique identifier for the requirement
    pub id:          String,
    /// The requirement text
    pub description: String,
    /// Priority level (1-5, with 1 being highest)
    pub priority:    u8,
    /// Related entities or functions
    pub related_to:  Vec<String>,
}

/// An entity (struct, enum, etc.) in the specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Name of the entity
    pub name:         String,
    /// Type of entity (struct, enum, trait, etc.)
    pub entity_type:  EntityType,
    /// Fields or variants of the entity
    pub fields:       Vec<Field>,
    /// Documentation comments
    pub docs:         Vec<String>,
    /// Associated requirements
    pub requirements: Vec<String>,
}

/// Type of entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    Struct,
    Enum,
    Trait,
    Module,
    TypeAlias,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Struct => write!(f, "struct"),
            EntityType::Enum => write!(f, "enum"),
            EntityType::Trait => write!(f, "trait"),
            EntityType::Module => write!(f, "module"),
            EntityType::TypeAlias => write!(f, "type alias"),
        }
    }
}

/// A field in a struct or variant in an enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Name of the field
    pub name:        String,
    /// Type of the field
    pub field_type:  String,
    /// Whether the field is optional
    pub is_optional: bool,
    /// Documentation comments
    pub docs:        Vec<String>,
}

/// Function specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    /// Function name
    pub name:         String,
    /// Return type (empty for unit)
    pub return_type:  String,
    /// List of parameters
    pub parameters:   Vec<Parameter>,
    /// Documentation comments
    pub docs:         Vec<String>,
    /// Associated requirements
    pub requirements: Vec<String>,
    /// Error types that might be returned
    pub error_types:  Vec<String>,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name:       String,
    /// Parameter type
    pub param_type: String,
    /// Whether the parameter is mutable
    pub is_mut:     bool,
    /// Whether the parameter is a reference
    pub is_ref:     bool,
}

/// Architectural pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalPattern {
    /// Name of the pattern (e.g., "Repository", "CQRS")
    pub name:        String,
    /// Confidence level (0.0 to 1.0)
    pub confidence:  f32,
    /// Description of the pattern
    pub description: String,
    /// Related components
    pub components:  Vec<PatternComponent>,
}

/// Component of an architectural pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternComponent {
    /// Role of the component in the pattern
    pub role:           String,
    /// Name of the component
    pub name:           String,
    /// Type of the component
    pub component_type: String,
}

/// Generated code result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    /// Main code files
    pub files:              Vec<CodeFile>,
    /// Additional resources (configs, assets, etc.)
    pub resources:          Vec<ResourceFile>,
    /// Build instructions
    pub build_instructions: String,
    /// Next steps or TODOs
    pub next_steps:         Vec<String>,
}

/// A single generated code file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    /// File path relative to project root
    pub path:    String,
    /// File content
    pub content: String,
    /// Whether the file is a test file
    pub is_test: bool,
}

/// A non-code resource file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFile {
    /// File path relative to project root
    pub path:    String,
    /// File content
    pub content: String,
}

/// Result of code validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// List of issues found
    pub issues:   Vec<ValidationIssue>,
    /// Overall score (0.0 to 1.0)
    pub score:    f32,
}

/// A validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity level
    pub severity:   Severity,
    /// Description of the issue
    pub message:    String,
    /// Location of the issue (file:line:column)
    pub location:   String,
    /// Suggestion for fixing the issue
    pub suggestion: Option<String>,
}

/// Severity level for validation issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
}

/// Result of code refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedCode {
    /// The refined code
    pub code:        String,
    /// List of changes made
    pub changes:     Vec<CodeChange>,
    /// Explanation of the changes
    pub explanation: String,
}

/// A single change made during refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Location of the change (file:line:column)
    pub location:    String,
    /// Old content (if applicable)
    pub old_content: Option<String>,
    /// New content (if applicable)
    pub new_content: Option<String>,
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
}
