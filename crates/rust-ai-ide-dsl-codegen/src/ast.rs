//! Abstract Syntax Tree (AST) definitions for the DSL

/// Location information for AST nodes
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

/// Complete DSL document containing templates and metadata
#[derive(Debug, Clone, PartialEq)]
pub struct DslDocument {
    pub templates: Vec<Template>,
    pub metadata: Vec<Metadata>,
    pub location: Option<Location>,
}

/// Template definition with all its components
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Vec<Parameter>,
    pub guards: Vec<Guard>,
    pub generate: GenerateBlock,
    pub patterns: Vec<String>,
    pub metadata: Vec<Metadata>,
    pub location: Option<Location>,
}

/// Parameter definition with type and constraints
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<Literal>,
    pub description: Option<String>,
}

/// Parameter types supported by the DSL
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterType {
    String,
    Integer,
    Boolean,
    Float,
    Array(Box<ParameterType>),
    Custom(String),
    ProgrammingLanguage,
    Identifier(String),
}

/// Guard conditions for template execution
#[derive(Debug, Clone, PartialEq)]
pub struct Guard {
    pub condition: Expression,
    pub location: Option<Location>,
}

/// Expressions for guards and template logic
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Binary(BinaryOp),
    Unary(UnaryOp),
    Literal(Literal),
    Variable(String),
    Call(FunctionCall),
    Bracketed(Box<Expression>),
}

/// Binary operations
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOp {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

/// Available binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    And,
    Or,
    Contains,
    Matches,
}

/// Unary operations
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOp {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
}

/// Available unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Literal>),
    Null,
}

/// Function calls in expressions
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}

/// Template generation block
#[derive(Debug, Clone, PartialEq)]
pub struct GenerateBlock {
    pub kind: GenerationKind,
    pub content: TemplateContent,
    pub validations: Vec<ValidationRule>,
    pub metadata: Vec<Metadata>,
}

/// Types of code generation
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationKind {
    Function,
    Class,
    Struct,
    Interface,
    Module,
    Test,
    Custom(String),
}

/// Template content with interpolation support
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateContent {
    pub parts: Vec<ContentPart>,
}

/// Parts of template content (literal text or placeholders)
#[derive(Debug, Clone, PartialEq)]
pub enum ContentPart {
    Literal(String),
    Placeholder(Placeholder),
    Conditional(Conditional),
    Loop(Loop),
}

/// Placeholder for template interpolation
#[derive(Debug, Clone, PartialEq)]
pub struct Placeholder {
    pub name: String,
    pub filters: Vec<Filter>,
    pub location: Option<Location>,
}

/// Template filters (transformations)
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Upper,
    Lower,
    Camel,
    Pascal,
    Snake,
    Kebab,
    Trim,
    Join(String),
    Length,
    Custom(String, Vec<String>),
}

/// Conditional template blocks
#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    pub condition: Expression,
    pub then_part: Vec<ContentPart>,
    pub else_part: Option<Vec<ContentPart>>,
}

/// Loop constructs in templates
#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    pub variable: String,
    pub iterable: Expression,
    pub body: Vec<ContentPart>,
}

/// Validation rules for generated code
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationRule {
    pub name: String,
    pub rule: String,
    pub severity: ValidationSeverity,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    Warning,
    Error,
    Info,
}

/// Metadata attachments
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub key: String,
    pub value: MetadataValue,
}

/// Metadata value types
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<MetadataValue>),
    Object(std::collections::HashMap<String, MetadataValue>),
}

impl Default for DslDocument {
    fn default() -> Self {
        Self {
            templates: Vec::new(),
            metadata: Vec::new(),
            location: None,
        }
    }
}

impl Template {
    /// Create a new template with required fields
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters: Vec::new(),
            guards: Vec::new(),
            generate: GenerateBlock::default(),
            patterns: Vec::new(),
            metadata: Vec::new(),
            location: None,
        }
    }
}

impl Default for GenerateBlock {
    fn default() -> Self {
        Self {
            kind: GenerationKind::Function,
            content: TemplateContent::new(),
            validations: Vec::new(),
            metadata: Vec::new(),
        }
    }
}

impl TemplateContent {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }
}
