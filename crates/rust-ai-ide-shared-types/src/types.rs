/// Shared Types Module for Rust AI IDE
///
/// This module contains the core shared types that need to be converted to TypeScript
/// for cross-platform type safety between Rust backend and TypeScript frontend.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Application configuration for the Rust AI IDE
/// This type will be converted to TypeScript for frontend configuration management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Core IDE configuration
    pub core: CoreConfig,
    /// AI systems configuration
    pub ai: AIConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

/// Core IDE configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Application name
    pub app_name: String,
    /// Application version
    pub app_version: String,
    /// Editor theme
    pub theme: String,
    /// Font preferences
    pub fonts: FontConfig,
    /// Editor settings
    pub editor: EditorConfig,
}

/// Font configuration for the IDE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Editor font family
    pub editor_font_family: String,
    /// Editor font size
    pub editor_font_size: f32,
    /// UI font family
    pub ui_font_family: String,
    /// UI font size
    pub ui_font_size: f32,
}

/// Editor configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Tab size
    pub tab_size: u32,
    /// Insert spaces instead of tabs
    pub insert_spaces: bool,
    /// Word wrap enabled
    pub word_wrap: bool,
    /// Minimap enabled
    pub minimap: bool,
    /// Line numbers enabled
    pub line_numbers: bool,
    /// Automatic save delay in seconds
    pub auto_save_delay: Option<u32>,
    /// Bracket matching enabled
    pub bracket_matching: bool,
    /// Highlight current line
    pub highlight_current_line: bool,
}

/// AI systems configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Default AI provider
    pub default_provider: AIProvider,
    /// API endpoints
    pub endpoints: HashMap<String, String>,
}

/// AI provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    /// Mock provider for testing
    Mock,
    /// OpenAI API
    OpenAI,
    /// Anthropic API
    Anthropic,
    /// Local provider
    Local,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum analysis threads
    pub max_analysis_threads: usize,
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// AI concurrency limit
    pub ai_concurrency_limit: usize,
    /// IO thread pool size
    pub io_thread_pool_size: usize,
}

/// User information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID
    pub id: u32,
    /// User's full name
    pub name: String,
    /// User's email address
    pub email: Option<String>,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System preference
    System,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// UI theme
    pub theme: Theme,
    /// Application settings
    pub settings: UserSettings,
}

/// User settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    /// Enable notifications
    pub notifications: bool,
    /// Enable auto-save
    pub auto_save: bool,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response status
    pub status: Status,
    /// Response data
    pub data: Option<T>,
    /// Response message
    pub message: Option<String>,
    /// Timestamp
    pub timestamp: String,
}

/// Status enum for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    /// Success status
    Ok,
    /// Error status
    Error,
    /// Warning status
    Warning,
    /// Info status
    Info,
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project ID
    pub id: String,
    /// Project name
    pub name: String,
    /// Project path
    pub path: String,
    /// Project settings
    pub settings: ProjectSettings,
}

/// Project settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Default file encoding
    pub encoding: String,
    /// Default line endings
    pub line_endings: String,
    /// Project language
    pub language: Option<String>,
}

/// Generic error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error details
    pub details: Option<HashMap<String, serde_json::Value>>,
}

// AST parsing types for type analysis

/// Parsed type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedType {
    /// Type name
    pub name: String,
    /// Type kind
    pub kind: TypeKind,
    /// Additional metadata
    pub metadata: Option<TypeMetadata>,
    /// Documentation
    pub documentation: Option<String>,
    /// Visibility
    pub visibility: Visibility,
    /// Generics
    pub generics: Vec<String>,
    /// Struct fields
    pub fields: Vec<Field>,
    /// Enum variants
    pub variants: Vec<Variant>,
    /// Associated items
    pub associated_items: Vec<String>,
    /// Attributes
    pub attributes: Vec<String>,
    /// Source location
    pub source_location: SourceLocation,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Type kind classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TypeKind {
    /// Struct type
    Struct,
    /// Enum type
    Enum,
    /// Union type
    Union,
    /// Type alias
    TypeAlias,
}

/// Type metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypeMetadata {
    /// Documentation
    pub docs: Option<String>,
    /// Visibility
    pub visibility: Option<Visibility>,
    /// Attributes
    pub attributes: Vec<String>,
}

/// Visibility levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    /// Public
    Public,
    /// Crate level
    Crate,
    /// Private
    Private,
    /// Module level
    Module,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path (alias for compatibility)
    pub file: String,
    /// File path
    pub file_path: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Module path
    pub module_path: Vec<String>,
}

/// Enum variant representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    /// Variant name
    pub name: String,
    /// Variant fields
    pub fields: Vec<VariantField>,
    /// Documentation
    pub documentation: Option<String>,
    /// Discriminant
    pub discriminant: Option<String>,
    /// Attributes
    pub attributes: Vec<String>,
}

/// Variant field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantField {
    /// Field name
    pub name: Option<String>,
    /// Field type
    pub ty: String,
    /// Field type (alias)
    pub field_type: String,
}

/// Struct field representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Field name
    pub name: String,
    /// Field type
    pub ty: String,
    /// Field type (alias)
    pub field_type: String,
    /// Visibility
    pub visibility: Visibility,
    /// Documentation
    pub documentation: Option<String>,
    /// Is mutable
    pub is_mutable: bool,
    /// Attributes
    pub attributes: Vec<String>,
}

// Code generation types

/// Code generation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    /// Types processed
    pub types_processed: usize,
    /// Types generated
    pub types_generated: usize,
    /// Bytes generated
    pub bytes_generated: usize,
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    /// Number of warnings
    pub warnings_count: u32,
    /// Number of errors
    pub errors_count: u32,
}

/// Code generation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationStatus {
    /// Generation succeeded
    Success,
    /// Generation failed
    Failed,
    /// Generation is in progress
    InProgress,
    /// Generation was cancelled
    Cancelled,
}

/// Code generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    /// Time the generation was performed
    pub generated_at: String,
    /// Version of the generator
    pub generator_version: String,
    /// Configuration snapshot
    pub config_snapshot: serde_json::Value,
    /// Generation statistics
    pub stats: GenerationStats,
    /// Generation status
    pub status: GenerationStatus,
}

/// Transformation context for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationContext {
    /// Source platform
    pub source_platform: String,
    /// Target platform
    pub target_platform: String,
    /// Configuration options
    pub config: HashMap<String, serde_json::Value>,
}

impl Default for TransformationContext {
    fn default() -> Self {
        Self {
            source_platform: "rust".to_string(),
            target_platform: "typescript".to_string(),
            config: HashMap::new(),
        }
    }
}

/// Generated code result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    /// Generated content
    pub content: String,
    /// Target platform
    pub target_platform: String,
    /// Source types used
    pub source_types: Vec<ParsedType>,
    /// Generation metadata
    pub metadata: GenerationMetadata,
    /// Dependencies
    pub dependencies: Vec<String>,
}
