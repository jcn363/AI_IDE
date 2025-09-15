//! Core types for AI code generation

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Generated code with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    /// The generated code content
    pub content:       String,
    /// Programming language of the generated code
    pub language:      TargetLanguage,
    /// Quality score (0.0 to 1.0)
    pub quality_score: f64,
    /// Additional metadata about the generation
    pub metadata:      HashMap<String, serde_json::Value>,
}

/// Specification for function generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    /// Function name
    pub name:        String,
    /// Function signature
    pub signature:   String,
    /// Programming language
    pub language:    TargetLanguage,
    /// Natural language description
    pub description: String,
}

/// Generated function with tests and documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFunction {
    /// The generated function code
    pub code:          GeneratedCode,
    /// Function signature
    pub signature:     String,
    /// Generated test cases
    pub test_cases:    Vec<TestCase>,
    /// Generated documentation
    pub documentation: String,
}

/// Specification for struct/class generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructSpec {
    /// Struct name
    pub name:               String,
    /// Struct fields
    pub fields:             Vec<StructField>,
    /// Programming language
    pub language:           TargetLanguage,
    /// Whether to generate accessor methods
    pub generate_accessors: bool,
}

/// Struct field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    /// Field name
    pub name:          String,
    /// Field type
    pub field_type:    String,
    /// Field documentation
    pub documentation: Option<String>,
}

/// Generated struct with methods and documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedStruct {
    /// The generated struct code
    pub code:          GeneratedCode,
    /// Generated methods
    pub methods:       Vec<GeneratedFunction>,
    /// Generated documentation
    pub documentation: String,
}

/// Context for code completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionContext {
    /// Current code at cursor position
    pub prefix:       String,
    /// Code after cursor position
    pub suffix:       String,
    /// Programming language
    pub language:     TargetLanguage,
    /// Current file path
    pub file_path:    Option<String>,
    /// Cursor position
    pub position:     Position,
    /// Additional context information
    pub context_info: HashMap<String, serde_json::Value>,
}

/// Position in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-based)
    pub line:   usize,
    /// Column number (0-based)
    pub column: usize,
}

/// Completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSuggestion {
    /// The completion text
    pub text:          String,
    /// Kind of completion (function, variable, etc.)
    pub kind:          CompletionKind,
    /// Short description
    pub detail:        Option<String>,
    /// Full documentation
    pub documentation: Option<String>,
    /// Text to use for sorting
    pub sort_text:     Option<String>,
    /// Text to use for filtering
    pub filter_text:   Option<String>,
}

/// Kind of completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

/// Test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test name
    pub name:     String,
    /// Test code
    pub code:     String,
    /// Expected result
    pub expected: String,
}

/// Test fixture for test setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFixture {
    /// Fixture name
    pub name: String,
    /// Fixture code
    pub code: String,
}

/// Mock object for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockObject {
    /// Mock name
    pub name: String,
    /// Mock implementation code
    pub code: String,
}

/// Complete test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Individual test cases
    pub test_cases:    Vec<TestCase>,
    /// Test fixtures
    pub fixtures:      Vec<TestFixture>,
    /// Mock objects
    pub mocks:         Vec<MockObject>,
    /// Setup code
    pub setup_code:    String,
    /// Teardown code
    pub teardown_code: String,
}

/// Refactoring suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    /// Type of refactoring
    pub kind:             RefactoringKind,
    /// Description of the refactoring
    pub description:      String,
    /// Target programming language
    pub target_language:  TargetLanguage,
    /// Impact level of the refactoring
    pub impact_level:     ImpactLevel,
    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f64,
}

/// Type of refactoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefactoringKind {
    ExtractMethod,
    ExtractVariable,
    ExtractClass,
    MoveMethod,
    Rename,
    Inline,
    IntroduceParameter,
    RemoveParameter,
    ChangeSignature,
    ExtractInterface,
    PullUp,
    PushDown,
    ReplaceMethod,
    ReplaceConditional,
    ReplaceLoop,
    IntroduceFactory,
}

/// Impact level of refactoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
}

/// Generated documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    /// Overview section
    pub overview:      String,
    /// API reference section
    pub api_reference: String,
    /// Usage examples
    pub examples:      String,
    /// Additional references
    pub references:    String,
    /// Generation timestamp
    pub generated_at:  chrono::DateTime<chrono::Utc>,
    /// Documentation format
    pub format:        DocFormat,
}

/// Documentation format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocFormat {
    Markdown,
    Html,
    Pdf,
    Json,
}

/// Security level for validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Strict,
}

// Re-export TargetLanguage from shared-codegen
pub use rust_ai_ide_shared_codegen::generator::TargetLanguage;

// Note: Display implementation for TargetLanguage is in the shared-codegen crate

impl RefactoringKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RefactoringKind::ExtractMethod => "extract method",
            RefactoringKind::ExtractVariable => "extract variable",
            RefactoringKind::ExtractClass => "extract class",
            RefactoringKind::MoveMethod => "move method",
            RefactoringKind::Rename => "rename",
            RefactoringKind::Inline => "inline",
            RefactoringKind::IntroduceParameter => "introduce parameter",
            RefactoringKind::RemoveParameter => "remove parameter",
            RefactoringKind::ChangeSignature => "change signature",
            RefactoringKind::ExtractInterface => "extract interface",
            RefactoringKind::PullUp => "pull up",
            RefactoringKind::PushDown => "push down",
            RefactoringKind::ReplaceMethod => "replace method",
            RefactoringKind::ReplaceConditional => "replace conditional",
            RefactoringKind::ReplaceLoop => "replace loop",
            RefactoringKind::IntroduceFactory => "introduce factory",
        }
    }
}
