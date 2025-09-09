//! Common types used across the Rust AI IDE project

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::Debug;

// Security types re-export
pub mod security;

// ===== CUSTOM SERIALIZATION HELPERS =====================================================================

// Wrapper for semver::Version to enable serde serialization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemverVersion(pub semver::Version);

impl Serialize for SemverVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for SemverVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        semver::Version::parse(&s)
            .map(SemverVersion)
            .map_err(serde::de::Error::custom)
    }
}

// ===== COMMON UTILITY TYPES =====================================================================

// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: Status,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Paginated response for collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub per_page: u32,
    pub has_more: bool,
}

// Event streaming types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent<T> {
    pub id: String,
    pub r#type: String,
    pub data: T,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Progress tracking for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandProgress {
    pub id: String,
    pub stage: String,
    pub progress: f32,
    pub message: String,
}

// Common status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
    Warning,
    Info,
    Pending,
    Running,
    Completed,
    Cancelled,
}

// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

// ===== CARGO TYPES =====================================================================

// Complete Cargo manifest representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoManifest {
    pub package: Option<CargoPackage>,
    pub dependencies: Option<HashMap<String, CargoDependency>>,
    pub dev_dependencies: Option<HashMap<String, CargoDependency>>,
    pub build_dependencies: Option<HashMap<String, CargoDependency>>,
    pub workspace: Option<CargoWorkspace>,
    #[serde(rename = "lib")]
    pub lib_config: Option<serde_json::Value>,
    #[serde(rename = "bin")]
    pub bin_config: Option<Vec<serde_json::Value>>,
    #[serde(rename = "test")]
    pub test_config: Option<Vec<serde_json::Value>>,
    #[serde(rename = "bench")]
    pub bench_config: Option<Vec<serde_json::Value>>,
    pub features: Option<HashMap<String, Vec<String>>>,
    pub target: Option<HashMap<String, serde_json::Value>>,
    pub patch: Option<HashMap<String, HashMap<String, CargoDependency>>>,
    pub replace: Option<HashMap<String, CargoDependency>>,
    pub profile: Option<HashMap<String, serde_json::Value>>,
    pub badges: Option<HashMap<String, serde_json::Value>>,
    pub metadata: Option<serde_json::Value>,
}

// Core dependency specification (shared with TypeScript frontend)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct CargoDependency {
    pub version: Option<String>,
    pub path: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub default_features: Option<bool>,
    pub package: Option<String>,
    pub registry: Option<String>,
    pub workspace: Option<bool>,
}

// Feature usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureUsage {
    pub name: String,
    pub enabled_by_default: bool,
    pub is_used: bool,
    pub used_by: Vec<String>,
    pub is_default: Option<bool>,
}

// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CargoPackage {
    pub name: Option<String>,
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub edition: Option<String>,
    pub rust_version: Option<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub readme: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub publish: Option<PublishConfig>,
    pub default_features: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

// Publishing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PublishConfig {
    Boolean(bool),
    RegistryList(Vec<String>),
}

// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CargoWorkspace {
    pub members: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub default_members: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub dependencies: Option<HashMap<String, CargoDependency>>,
    pub dev_dependencies: Option<HashMap<String, CargoDependency>>,
    pub build_dependencies: Option<HashMap<String, CargoDependency>>,
    pub package: Option<WorkspacePackageConfig>,
    pub resolver: Option<String>,
}

// Workspace package config defaults
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePackageConfig {
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

// ===== PERFORMANCE TYPES =====================================================================

// Performance metrics shared across frontend and backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrateMetrics {
    pub build_time: Option<u64>,
    pub dependencies: Option<Vec<String>>,
    pub features_used: Option<Vec<String>>,
}

// Core performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct PerformanceMetrics {
    pub total_time: u64,
    pub compilation_time: Option<u64>,
    pub analysis_time: Option<u64>,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f32>,
}

// ===== ERROR TYPES =====================================================================

// Command error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandError {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

// API error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

// ===== CONFIGURATION TYPES =====================================================================

// User preferences (shared configuration)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub theme: Theme,
    pub keybindings: HashMap<String, String>,
    #[serde(rename = "editor")]
    pub editor_prefs: EditorPreferences,
    #[serde(rename = "cargo")]
    pub cargo_prefs: CargoPreferences,
}

// UI themes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

// Editor preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPreferences {
    pub font_size: u32,
    pub tab_size: u32,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub line_numbers: bool,
}

// Cargo-specific preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoPreferences {
    pub auto_update: bool,
    pub default_profile: String,
    pub show_timings: bool,
    pub offline_mode: Option<bool>,
}

// Project configuration combining user and project settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    pub path: std::path::PathBuf,
    pub cargo_config: Option<CargoManifest>,
    pub user_preferences: UserPreferences,
}

// ===== UTILITY TYPES =====================================================================

// Module and command identifiers
pub type ModuleId = String;
pub type CommandId = String;
pub type StreamId = String;
pub type SessionId = String;

// Path types
// ===== ALERT TYPES =====================================================================

// Alert levels for logging and notifications
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl AsRef<str> for AlertLevel {
    fn as_ref(&self) -> &str {
        match self {
            AlertLevel::Info => "info",
            AlertLevel::Warning => "warning",
            AlertLevel::Error => "error",
            AlertLevel::Critical => "critical",
        }
    }
}

// ===== CONFIGURATION TYPES =====================================================================

// Source priority for configuration handling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum SourcePriority {
    Low,
    Medium,
    High,
    Critical,
}

pub type ProjectPath = std::path::PathBuf;
pub type FilePath = std::path::PathBuf;

// ===== RESPONSE HELPERS =====================================================================

// Convenience constructors for API responses
impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            status: Status::Ok,
            data: Some(data),
            message: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: Status::Error,
            data: None,
            message: Some(message.into()),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn warning(data: Option<T>, message: impl Into<String>) -> Self {
        Self {
            status: Status::Warning,
            data,
            message: Some(message.into()),
            timestamp: chrono::Utc::now(),
        }
    }
}

// Type guards and utilities
impl<T> ApiResponse<T> {
    pub fn is_ok(&self) -> bool {
        matches!(self.status, Status::Ok)
    }

    pub fn is_error(&self) -> bool {
        matches!(self.status, Status::Error | Status::Warning)
    }
}

/// Represents supported programming languages for code generation and analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProgrammingLanguage {
    /// Rust programming language
    Rust,
    /// TypeScript programming language
    TypeScript,
    /// JavaScript programming language
    JavaScript,
    /// Python programming language
    Python,
    /// Java programming language
    Java,
    /// C# programming language
    CSharp,
    /// Go programming language
    Go,
    /// C++ programming language
    Cpp,
    /// C programming language
    C,
    /// Unknown programming language
    Unknown,
}

impl std::fmt::Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let language_str = match self {
            ProgrammingLanguage::Rust => "Rust",
            ProgrammingLanguage::TypeScript => "TypeScript",
            ProgrammingLanguage::JavaScript => "JavaScript",
            ProgrammingLanguage::Python => "Python",
            ProgrammingLanguage::Java => "Java",
            ProgrammingLanguage::CSharp => "C#",
            ProgrammingLanguage::Go => "Go",
            ProgrammingLanguage::Cpp => "C++",
            ProgrammingLanguage::C => "C",
            ProgrammingLanguage::Unknown => "Unknown",
        };
        write!(f, "{}", language_str)
    }
}

/// Types of tests that can be generated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TestType {
    /// Unit tests testing individual functions/components
    Unit,
    /// Integration tests testing component interactions
    Integration,
    /// Property-based tests using generated inputs
    Property,
    /// Benchmark tests measuring performance
    Benchmark,
    /// Fuzzing tests using generated inputs to find defects
    Fuzz,
    /// End-to-end tests testing complete user flows
    E2e,
}

/// Coverage granularity types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CoverageType {
    /// Function-level coverage
    Function,
    /// Line-level coverage
    Line,
    /// Branch-level coverage
    Branch,
    /// Statement-level coverage (subset of Line)
    Statement,
    /// Edge coverage in control flow graphs
    Edge,
}

/// Comprehensive test generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTest {
    /// Test name/identifier
    pub name: String,
    /// Generated test code
    pub code: String,
    /// Type of test generated
    pub test_type: TestType,
    /// Human-readable description
    pub description: String,
    /// Test framework used
    pub framework: String,
    /// Programming language target
    pub language: ProgrammingLanguage,
    /// Expected coverage targets
    pub expected_coverage: Vec<String>,
    /// External dependencies required
    pub dependencies: Vec<String>,
    /// Test categorization tags
    pub tags: Vec<String>,
    /// Confidence score (0.0-1.0) in test effectiveness
    pub confidence_score: f32,
}

/// Test coverage estimation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverage {
    /// Target function/class being covered
    pub target: String,
    /// Type of coverage being measured
    pub coverage_type: CoverageType,
    /// Number of lines covered
    pub lines_covered: u32,
    /// Number of branches covered
    pub branches_covered: u32,
    /// Estimated coverage percentage
    pub estimated_coverage_percent: f32,
}

/// Collection of generated tests with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTests {
    /// Unit tests
    pub unit_tests: Vec<GeneratedTest>,
    /// Integration tests
    pub integration_tests: Vec<GeneratedTest>,
    /// Property tests
    pub property_tests: Vec<GeneratedTest>,
    /// Benchmark tests
    pub benchmark_tests: Vec<GeneratedTest>,
    /// Coverage estimates
    pub coverage_estimates: Vec<TestCoverage>,
}

/// Refactoring operation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RefactoringType {
    /// Rename symbol (function, variable, class)
    Rename,
    /// Extract function from code block
    ExtractFunction,
    /// Extract variable from expression
    ExtractVariable,
    /// Extract interface/trait from implementation
    ExtractInterface,
    /// Convert synchronous function to async
    ConvertToAsync,
    /// Move symbol to different location
    Move,
    /// Inline function call or variable reference
    Inline,
    /// Change function/method signature
    ChangeSignature,
    /// Replace method call with inline implementation
    ReplaceWithMethodCall,
    /// Other refactoring types
    Other(String),
}

/// Context information for refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringContext {
    /// File path containing the refactoring
    pub file_path: String,
    /// Name of the symbol being refactored
    pub symbol_name: Option<String>,
    /// Start line of the symbol
    pub symbol_line_start: usize,
    /// End line of the symbol
    pub symbol_line_end: usize,
    /// Type of the symbol (function, class, variable, etc.)
    pub symbol_type: Option<String>,
    /// Programming language of the file
    pub language: ProgrammingLanguage,
}

/// Result of a refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringResult {
    /// Whether the refactoring succeeded
    pub success: bool,
    /// Changes made during refactoring
    pub changes_made: Vec<CodeChange>,
    /// New name of the symbol (for rename operations)
    pub new_symbol_name: Option<String>,
    /// Name of extracted function/interface (for extraction operations)
    pub extracted_function_name: Option<String>,
    /// Name of extracted interface/trait (for interface extraction)
    pub extracted_interface_name: Option<String>,
}

/// Record of a code change made during refactoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    /// File path where change was made
    pub file_path: String,
    /// Start line of the change
    pub line_start: usize,
    /// End line of the change
    pub line_end: usize,
    /// Original code before change
    pub original_code: String,
    /// New code after change
    pub new_code: String,
}

/// Context information for test generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGenerationContext {
    /// Path to the file being tested
    pub file_path: String,
    /// Whether the target code is performance-critical
    pub is_performance_critical: bool,
    /// Required test coverage percentage
    pub required_coverage: Option<f32>,
    /// Supported programming languages
    pub target_languages: Vec<ProgrammingLanguage>,
}

/// Test framework information for a language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFrameworkInfo {
    /// Programming language
    pub language: ProgrammingLanguage,
    /// Available test frameworks for this language
    pub test_frameworks: Vec<String>,
    /// Common file extensions for this language
    pub file_extensions: Vec<String>,
    /// Preferred/recommended test framework
    pub preferred_framework: String,
}

/// Configuration for test generation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGenerationConfig {
    /// Whether to include edge cases in generated tests
    pub include_edge_cases: bool,
    /// Whether to generate integration tests
    pub generate_integration_tests: bool,
    /// Maximum number of tests to generate per target
    pub max_tests_per_generation: usize,
    /// Target coverage percentage (0.0-100.0)
    pub target_coverage_percentage: f32,
    /// Language-specific configuration overrides
    pub language_specific: std::collections::HashMap<ProgrammingLanguage, LanguageConfig>,
    /// Timeout seconds for test generation
    pub timeout_seconds: u32,
}

/// Language-specific test generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Naming conventions for test functions
    pub naming_conventions: Vec<String>,
    /// Test code patterns specific to the language
    pub test_patterns: Vec<String>,
    /// Assertion styles available
    pub assertion_styles: Vec<String>,
    /// Supported mocking frameworks
    pub mock_frameworks: Vec<String>,
}

/// Context information for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Root directory of the project
    pub root_path: std::path::PathBuf,
    /// Primary programming language of the project
    pub language: ProgrammingLanguage,
    /// Target frameworks or platforms
    pub frameworks: Vec<String>,
    /// Project-specific configuration
    pub config: serde_json::Value,
}

/// Position in a text document (zero-based line and character)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Range in a text document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Location in a text document
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// The text document's URI
    pub uri: String,
    /// The position inside the text document
    pub range: Range,
}

/// Internal position representation (using usize for compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InternalPosition {
    pub line: usize,
    pub character: usize,
}

/// Internal range representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InternalRange {
    pub start: InternalPosition,
    pub end: InternalPosition,
}

/// Type conversion utilities between LSP types and internal types

impl From<InternalPosition> for Position {
    fn from(pos: InternalPosition) -> Self {
        Position {
            line: pos.line as u32,
            character: pos.character as u32,
        }
    }
}

impl From<Position> for InternalPosition {
    fn from(pos: Position) -> Self {
        InternalPosition {
            line: pos.line as usize,
            character: pos.character as usize,
        }
    }
}

impl From<InternalRange> for Range {
    fn from(range: InternalRange) -> Self {
        Range {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl From<Range> for InternalRange {
    fn from(range: Range) -> Self {
        InternalRange {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl InternalRange {
    pub fn to_lsp(self) -> Range {
        self.into()
    }

    pub fn from_lsp(range: Range) -> Self {
        range.into()
    }
}

impl Position {
    /// Convert to internal representation
    pub fn to_internal(self) -> InternalPosition {
        self.into()
    }

    /// Create from internal representation
    pub fn from_internal(pos: InternalPosition) -> Self {
        pos.into()
    }
}

impl Range {
    /// Convert to internal representation
    pub fn to_internal(self) -> InternalRange {
        self.into()
    }

    /// Create from internal representation
    pub fn from_internal(range: InternalRange) -> Self {
        range.into()
    }
}

/// Safe conversion that checks for overflow
pub fn safe_position_conversion(position: InternalPosition) -> Option<Position> {
    if position.line <= u32::MAX as usize && position.character <= u32::MAX as usize {
        Some(Position {
            line: position.line as u32,
            character: position.character as u32,
        })
    } else {
        None
    }
}

/// Convert project language to LSP language identifier string
pub fn language_to_lsp_identifier(language: &ProgrammingLanguage) -> String {
    match language {
        ProgrammingLanguage::Rust => "rust".to_string(),
        ProgrammingLanguage::TypeScript => "typescript".to_string(),
        ProgrammingLanguage::JavaScript => "javascript".to_string(),
        ProgrammingLanguage::Python => "python".to_string(),
        ProgrammingLanguage::Java => "java".to_string(),
        ProgrammingLanguage::CSharp => "csharp".to_string(),
        ProgrammingLanguage::Go => "go".to_string(),
        ProgrammingLanguage::Cpp => "cpp".to_string(),
        ProgrammingLanguage::C => "c".to_string(),
        ProgrammingLanguage::Unknown => "plaintext".to_string(),
    }
}

/// Convert LSP language identifier to project language
pub fn lsp_identifier_to_language(identifier: &str) -> ProgrammingLanguage {
    match identifier.to_lowercase().as_str() {
        "rust" => ProgrammingLanguage::Rust,
        "typescript" => ProgrammingLanguage::TypeScript,
        "javascript" => ProgrammingLanguage::JavaScript,
        "python" => ProgrammingLanguage::Python,
        "java" => ProgrammingLanguage::Java,
        "csharp" => ProgrammingLanguage::CSharp,
        "go" => ProgrammingLanguage::Go,
        "cpp" | "c++" => ProgrammingLanguage::Cpp,
        "c" => ProgrammingLanguage::C,
        _ => ProgrammingLanguage::Unknown,
    }
}

/// Plugin metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: String,
    /// Human-readable plugin name
    pub name: String,
    /// Plugin description
    pub description: String,
    /// Plugin version
    pub version: SemverVersion,
    /// Plugin author
    pub author: String,
    /// Plugin website/documentation URL
    pub homepage: Option<String>,
    /// Minimum IDE version required
    pub min_ide_version: Option<SemverVersion>,
    /// Maximum IDE version supported
    pub max_ide_version: Option<SemverVersion>,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Plugin icon/path if any
    pub icon: Option<String>,
}

/// Plugin capability types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PluginCapability {
    /// LSP language server support
    LspLanguageServer(String),
    /// Code generation
    CodeGeneration,
    /// Code analysis
    CodeAnalysis,
    /// Code refactoring
    CodeRefactoring,
    /// Debugging capabilities
    Debugging,
    /// File watching
    FileWatching,
    /// Version control integration
    VersionControl,
    /// Terminal integration
    Terminal,
    /// UI components
    UiComponents,
    /// Configuration management
    Configuration,
    /// Communication with external services
    ExternalCommunication(String),
    /// Custom capability
    Custom(String),
}

/// Plugin event types for inter-plugin communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PluginEvent {
    /// Plugin is starting
    Started { plugin_id: String },
    /// Plugin is shutting down
    ShuttingDown { plugin_id: String },
    /// Plugin initialization complete
    Initialized { plugin_id: String },
    /// Plugin error occurred
    Error { plugin_id: String, message: String },
    /// Custom plugin event
    Custom {
        plugin_id: String,
        event_type: String,
        data: serde_json::Value,
    },
}

/// Plugin message types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginMessageType {
    Request,
    Response,
    Notification,
    Error,
}

/// Plugin message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    pub id: String,
    pub message_type: PluginMessageType,
    pub from_plugin: String,
    pub to_plugin: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Event bus for plugin communication
#[derive(Debug)]
pub struct PluginEventBus {
    pub sender: tokio::sync::broadcast::Sender<PluginEvent>,
    pub receiver: tokio::sync::broadcast::Receiver<PluginEvent>,
}

/// Plugin performance monitor
#[derive(Debug, Clone)]
pub struct PluginPerformanceMonitor {
    pub metrics: std::sync::Arc<dashmap::DashMap<String, PluginMetrics>>,
}

/// Plugin metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetrics {
    pub plugin_id: String,
    pub init_time_ms: Option<u64>,
    pub shutdown_time_ms: Option<u64>,
    pub total_executions: u64,
    pub avg_execution_time_ms: f64,
    pub error_count: u64,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

/// Plugin context for dependency injection and shared access
#[derive(Debug)]
pub struct PluginContext {
    /// IDE configuration
    pub config: serde_json::Value,
    /// Plugin cache
    pub cache: std::sync::Arc<dyn crate::caching::Cache<String, serde_json::Value>>,
    /// Event bus for plugin communication
    pub event_bus: std::sync::Arc<PluginEventBus>,
    /// Logger
    pub logger: Box<dyn crate::traits::Loggable + Send + Sync>,
    /// Performance monitor
    pub performance_monitor: std::sync::Arc<PluginPerformanceMonitor>,
    /// Working directory
    pub working_directory: std::path::PathBuf,
}

impl PluginContext {
    /// Create a new plugin context with defaults
    pub fn new(
        config: serde_json::Value,
        cache: std::sync::Arc<dyn crate::caching::Cache<String, serde_json::Value>>,
        logger: Box<dyn crate::traits::Loggable>,
        working_directory: std::path::PathBuf,
    ) -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(100);
        let event_bus = PluginEventBus { sender, receiver };

        Self {
            config,
            cache,
            event_bus: std::sync::Arc::new(event_bus),
            logger,
            performance_monitor: std::sync::Arc::new(PluginPerformanceMonitor::default()),
            working_directory,
        }
    }

    /// Send an event to the event bus
    pub async fn send_event(&self, event: PluginEvent) -> crate::errors::IdeResult<()> {
        self.event_bus
            .sender
            .send(event)
            .map_err(|_| crate::errors::IdeError::Internal {
                message: "Failed to send event".to_string(),
            })?;
        Ok(())
    }

    /// Get performance metrics for a plugin
    pub fn get_plugin_metrics(&self, plugin_id: &str) -> Option<PluginMetrics> {
        self.performance_monitor
            .metrics
            .get(plugin_id)
            .as_deref()
            .cloned()
    }
}

impl Default for PluginEventBus {
    fn default() -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(100);
        Self { sender, receiver }
    }
}

impl PluginEventBus {
    /// Send an event to all subscribers
    pub async fn send_event(&self, event: PluginEvent) -> crate::errors::IdeResult<()> {
        self.sender
            .send(event)
            .map_err(|_| crate::errors::IdeError::Internal {
                message: "Failed to send event".to_string(),
            })?;
        Ok(())
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<PluginEvent> {
        self.sender.subscribe()
    }
}

impl Default for PluginPerformanceMonitor {
    fn default() -> Self {
        Self {
            metrics: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }
}

impl PluginPerformanceMonitor {
    /// Record execution time for a plugin
    pub fn record_execution(&self, plugin_id: &str, duration_ms: u64) {
        let mut metrics = self
            .metrics
            .entry(plugin_id.to_string())
            .or_insert_with(|| PluginMetrics {
                plugin_id: plugin_id.to_string(),
                init_time_ms: None,
                shutdown_time_ms: None,
                total_executions: 0,
                avg_execution_time_ms: 0.0,
                error_count: 0,
                last_execution: None,
            });

        metrics.total_executions += 1;

        // Update running average
        let alpha = 0.1; // decay factor for exponential moving average
        metrics.avg_execution_time_ms =
            (1.0 - alpha) * metrics.avg_execution_time_ms + alpha * duration_ms as f64;

        metrics.last_execution = Some(chrono::Utc::now());
    }

    /// Record initialization time
    pub fn record_initialization(&self, plugin_id: &str, duration_ms: u64) {
        let mut metrics = self
            .metrics
            .entry(plugin_id.to_string())
            .or_insert_with(|| PluginMetrics {
                plugin_id: plugin_id.to_string(),
                init_time_ms: Some(0),
                shutdown_time_ms: None,
                total_executions: 0,
                avg_execution_time_ms: 0.0,
                error_count: 0,
                last_execution: None,
            });

        *metrics.init_time_ms.as_mut().unwrap() = duration_ms;
    }

    /// Record error for a plugin
    pub fn record_error(&self, plugin_id: &str) {
        let mut metrics = self
            .metrics
            .entry(plugin_id.to_string())
            .or_insert_with(|| PluginMetrics {
                plugin_id: plugin_id.to_string(),
                init_time_ms: None,
                shutdown_time_ms: None,
                total_executions: 0,
                avg_execution_time_ms: 0.0,
                error_count: 0,
                last_execution: None,
            });

        metrics.error_count += 1;
    }
}
