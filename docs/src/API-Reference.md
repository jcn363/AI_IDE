# Rust AI IDE - API Reference

## 📖 Overview

This document provides a comprehensive reference for the public APIs of the three shared crates that form the foundation of the Rust AI IDE:

- `rust-ai-ide-common` - Core utilities, types, and patterns
- `rust-ai-ide-shared-codegen` - Code generation and AST operations
- `rust-ai-ide-shared-services` - LSP integration and workspace management

> **Usage Tip**: Import the three shared crates at the top of every Rust module to access all unified functionality.

---

## rust-ai-ide-common

### Core Types

#### Programming Linguistics

```rust
pub enum ProgrammingLanguage {
    Rust, TypeScript, JavaScript, Python, Java, Cpp, Go, Swift, Unknown(String)
}

impl ProgrammingLanguage {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Rust => ".rs",
            Self::TypeScript => ".ts",
            // ... and so on
            _ => "",
        }
    }

    pub fn lsp_identifier(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            // ... and so on
            _ => "unknown",
        }
    }
}
```

#### Position and Range System

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,      // 0-based line number
    pub character: u32, // 0-based character offset
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

pub struct PositionNormalizer;

impl PositionNormalizer {
    pub fn to_backend(position: Position) -> Position {
        // Converts from 1-based to 0-based indexing
    }

    pub fn to_frontend(position: Position) -> Position {
        // Converts from 0-based to 1-based indexing
    }
}
```

### Error Handling

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IdeError {
    Io(String),
    Config(String),
    Compilation(String),
    Analysis(String),
    Refactoring(String),
    InvalidInput(String),
    FileNotFound(String),
    PermissionDenied(String),
    Network(String),
    Generic {
        category: String,
        message: String,
        context: Option<String>,
    },
}

pub type IdeResult<T> = Result<T, IdeError>;

// Error conversion utilities
pub trait IntoIdeError<T> {
    fn into_ide_error(self, category: impl Into<String>) -> IdeResult<T>;
}

pub fn convert_error<T, E, F>(result: Result<T, E>, mapper: F) -> IdeResult<T>
where F: FnOnce(E) -> IdeError;

pub fn option_to_result<T, F>(option: Option<T>, err_fn: F) -> IdeResult<T>
where F: FnOnce() -> IdeError;
```

### Caching System

```rust
#[async_trait]
pub trait Cache<K, V> {
    async fn get(&self, key: &K) -> IdeResult<V>;
    async fn set(&self, key: K, value: V) -> IdeResult<()>;
    async fn contains(&self, key: &K) -> bool;
    async fn remove(&self, key: &K) -> IdeResult<Option<V>>;
    async fn clear(&self) -> IdeResult<()>;
}

pub struct MemoryCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static;

impl<K, V> MemoryCache<K, V> {
    pub fn new(max_entries: usize, ttl: Duration) -> Self;
    pub fn with_eviction_policy(policy: EvictionPolicy) -> Self;

    pub async fn get_with_metrics(&self, key: &K) -> IdeResult<(V, CacheMetrics)>;
    pub async fn stats(&self) -> CacheStats;

    // Entry operations
    pub async fn get_entry(&self, key: &K) -> Option<CacheEntry<V>>;
    pub async fn touch(&self, key: &K) -> bool; // Updates last accessed time
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub sets: u64,
    pub hit_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub data: T,
    pub inserted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub size_bytes: usize,
}
```

### Performance Utilities

```rust
pub fn time_operation<T, F>(name: &str, operation: F) -> IdeResult<(T, Duration)>
where F: FnOnce() -> IdeResult<T>;

pub fn time_async_operation<Fut>(operation: Fut) -> impl Future<Output = (Fut::Output, Duration)>
where Fut::Output: Sized;

// Scoped timing with automatic reporting
pub struct ScopedTimer {
    name: String,
    start_time: Instant,
    markers: Vec<PerformanceMarker>,
}

impl ScopedTimer {
    pub fn new(name: impl Into<String>) -> Self;
    pub fn add_marker(&mut self, name: impl Into<String>);
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        // Automatically logs completion time
    }
}
```

### File System Utilities

```rust
pub async fn read_file_to_string<P: AsRef<Path>>(path: P) -> IdeResult<String>;
pub async fn read_file_to_bytes<P: AsRef<Path>>(path: P) -> IdeResult<Vec<u8>>;
pub async fn write_string_to_file<P: AsRef<Path>>(path: P, content: &str) -> IdeResult<()>;
pub async fn file_exists<P: AsRef<Path>>(path: P) -> bool;
pub async fn dir_exists<P: AsRef<Path>>(path: P) -> bool;
pub async fn get_metadata<P: AsRef<Path>>(path: P) -> IdeResult<Metadata>;
pub async fn ensure_parent_dirs<P: AsRef<Path>>(path: P) -> IdeResult<()>;
pub async fn remove_file<P: AsRef<Path>>(path: P) -> IdeResult<()>;
pub async fn remove_dir_all<P: AsRef<Path>>(path: P) -> IdeResult<()>;
pub async fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> IdeResult<u64>;
pub async fn list_files_recursive<P: AsRef<Path>>(path: P, pattern: Option<&str>) -> IdeResult<Vec<PathBuf>>;
pub async fn watch_file_changes<P: AsRef<Path>>(path: P) -> IdeResult<(Sender<FileChange>, Receiver<FileChange>)>;

#[derive(Debug, Clone)]
pub enum FileChange {
    Created(PathBuf),
    Modified(String, PathBuf), // content + path
    Deleted(PathBuf),
    Renamed(PathBuf, PathBuf), // from -> to
}

pub async fn update_file_atomically<P: AsRef<Path>>(path: P, content: &str) -> IdeResult<()>;
pub fn validate_path<P: AsRef<Path>>(path: P) -> IdeResult<PathBuf>;
```

### Duplication Detection

```rust
pub struct DuplicationStats {
    pub total_files: usize,
    pub duplicated_functions: usize,
    pub similar_structs: usize,
    pub repeated_patterns: usize,
    pub trait_duplicates: usize,
    pub total_duplicates: usize,
}

#[derive(Debug, Clone)]
pub struct DuplicationResult {
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub kind: DuplicationKind,
    pub confidence: f64,
    pub similar_to: String,
    pub code_snippet: String,
}

#[derive(Debug, Clone)]
pub enum DuplicationKind {
    Function,
    Struct,
    TraitImpl,
    CodePattern(String),
    TypeDefinition,
}

#[derive(Debug, Clone)]
pub struct SimilarityMatch {
    pub similarity_score: f64,
    pub matched_content: String,
    pub source_location: String,
}

// Core functions
pub fn detect_duplications(files: &HashMap<String, String>) -> Result<DuplicationStats, String>;
pub fn check_potential_duplication(new_code: &str, existing_files: &HashMap<String, String>) -> Vec<SimilarityMatch>;
pub fn calculate_similarity(code1: &str, code2: &str) -> f64;
pub fn create_duplication_prevention_template(module_name: &str) -> String;
pub fn create_safe_function_template(function_name: &str, parameters: &[&str]) -> String;
pub fn verify_duplication_free(file_content: &str) -> Result<(), Vec<String>>;

// Analysis functions
pub fn extract_functions(code: &str) -> Result<Vec<(String, String)>, String>;
pub fn extract_structs(code: &str) -> Result<Vec<(String, String)>, String>;
pub fn extract_trait_implementations(code: &str) -> Result<Vec<(String, String)>, String>;
```

### Path Utilities

```rust
pub fn normalize_path<P: AsRef<Path>>(path: P) -> IdeResult<PathBuf>;
pub fn relative_path_from<P: AsRef<Path>, Q: AsRef<Path>>(path: P, base: Q) -> IdeResult<PathBuf>;
pub fn safe_path_join<P: AsRef<Path>, Q: AsRef<Path>>(base: P, segment: Q) -> IdeResult<PathBuf>;
pub fn validate_path<P: AsRef<Path>>(path: P) -> IdeResult<PathBuf>;
pub fn ensure_directory<P: AsRef<Path>>(path: P) -> IdeResult<()>;
pub fn temporary_path_in<P: AsRef<Path>>(dir: P, prefix: &str) -> IdeResult<PathBuf>;
pub fn safe_canonicalize<P: AsRef<Path>>(path: P) -> IdeResult<PathBuf>;
```

### Rate Limiting

```rust
pub struct RateLimiter {
    capacity: usize,
    refill_rate: usize,
    tokens: Mutex<usize>,
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(capacity: usize, refill_rate: usize) -> Self;
    pub async fn acquire(&self) -> bool;
    pub async fn acquire_many(&self, tokens: usize) -> bool;
    pub async fn available_tokens(&self) -> usize;
    pub async fn time_until_next_token(&self) -> Duration;
    pub async fn try_acquire(&self) -> Option<Permit>;
}
```

---

## rust-ai-ide-shared-codegen

### Code Generation

```rust
#[derive(Debug, Clone)]
pub enum EntityType {
    Struct,
    Enum,
    Trait,
    Impl,
    Function,
    Module,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_hint: String,
    pub is_optional: bool,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub language: ProgrammingLanguage,
    pub entity_type: EntityType,
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub methods: Vec<MethodDef>,
    pub imports: Vec<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub include_docs: bool,
    pub include_tests: bool,
    pub style_preference: Option<String>, // "builder_pattern", "fluent_api", etc.
    pub indentation_style: IndentationStyle,
    pub naming_convention: NamingConvention,
}

pub trait CodeGenerator {
    async fn generate(&self, request: GenerateRequest, config: &GenerationConfig) -> IdeResult<String>;
}

impl CodeGenerator for RustCodeGenerator {
    async fn generate(&self, request: GenerateRequest, config: &GenerationConfig) -> IdeResult<String> {
        // Implementation details...
    }
}

// Factory functions
impl CodeGenerator {
    pub fn for_language(lang: ProgrammingLanguage) -> Box<dyn CodeGenerator>;
    pub fn rust() -> RustCodeGenerator;
    pub fn typescript() -> TypeScriptCodeGenerator;
}
```

### AST Operations

```rust
#[derive(Debug)]
pub enum AstNode {
    Module(Vec<AstNode>),
    Struct(StructDef),
    Enum(EnumDef),
    Function(FunctionDef),
    Trait(TraitDef),
    Impl(ImplDef),
    Expression(Expression),
    Statement(Statement),
}

#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub methods: Vec<MethodDef>,
    pub derives: Vec<String>,
    pub documentation: Option<String>,
}

pub struct AstParser;

impl AstParser {
    pub fn new(lang: ProgrammingLanguage) -> Self;
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> IdeResult<AstNode>;
    pub fn parse_code(&self, code: &str) -> IdeResult<AstNode>;
    pub fn parse_expression(&self, expr: &str) -> IdeResult<Expression>;
}

pub trait AstTransformer {
    async fn apply(&self, node: AstNode) -> IdeResult<AstNode>;
    fn supports_language(&self, lang: ProgrammingLanguage) -> bool;
}

// Built-in transformers
pub struct RefactorTransformer { rules: Vec<TransformRule> }
pub struct OptimizationTransformer { strategies: Vec<OptimizationStrategy> }
pub struct SafetyTransformer { checks: Vec<SafetyCheck> }
```

### Pattern Recognition

```rust
#[derive(Debug, Clone)]
pub enum PatternType {
    AntiPattern,
    CodeSmell,
    Refactoring,
    Optimization,
    Safety,
    Performance,
}

#[derive(Debug)]
pub struct CodePattern {
    pub pattern_type: PatternType,
    pub name: String,
    pub description: String,
    pub confidence_threshold: f64,
    pub regex_patterns: Vec<String>,
    pub semantic_rules: Vec<SemanticRule>,
}

#[derive(Debug)]
pub struct PatternMatch {
    pub pattern: CodePattern,
    pub range: Range,
    pub confidence: f64,
    pub context: HashMap<String, String>,
    pub suggestion: Option<String>,
}

pub struct PatternMatcher {
    patterns: Vec<CodePattern>,
}

impl PatternMatcher {
    pub fn new(patterns: Vec<CodePattern>) -> Self;
    pub fn match_patterns(&self, code: &str, language: ProgrammingLanguage) -> Vec<PatternMatch>;
    pub fn find_duplicates(&self, code: &str) -> Vec<PatternMatch>;
    pub fn suggest_refactors(&self, code: &str) -> Vec<RefactorSuggestion>;
}

// Common pattern libraries
pub mod patterns {
    pub fn extract_async_function_call() -> CodePattern;
    pub fn extract_error_handling_block() -> CodePattern;
    pub fn consolidate_duplicate_imports() -> CodePattern;
    pub fn optimize_collection_creation() -> CodePattern;
    pub fn normalize_naming_conventions() -> CodePattern;
    pub fn eliminate_dead_code() -> CodePattern;
    pub fn split_large_functions() -> CodePattern;
}
```

### Template System

```rust
#[derive(Debug)]
pub struct TemplateContext {
    pub values: HashMap<String, serde_json::Value>,
    pub functions: Vec<TemplateFunction>,
    pub partials: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TemplateFunction {
    pub name: String,
    pub callback: Box<dyn Fn(&[String]) -> String + Send + Sync>,
}

pub struct TemplateEngine {
    templates: HashMap<String, Template>,
    functions: HashMap<String, TemplateFunction>,
}

impl TemplateEngine {
    pub fn new() -> Self;
    pub fn register_template(&mut self, name: &str, content: &str) -> IdeResult<()>;
    pub fn register_function(&mut self, name: &str, function: TemplateFunction) -> IdeResult<()>;
    pub fn render(&self, template_name: &str, context: &TemplateContext) -> IdeResult<String>;
    pub fn render_string(&self, template: &str, context: &TemplateContext) -> IdeResult<String>;
    pub fn validate_template(&self, template: &str) -> Result<(), TemplateErrors>;
}

// Template helpers
pub fn camel_case(input: &str) -> String;
pub fn pascal_case(input: &str) -> String;
pub fn snake_case(input: &str) -> String;
pub fn kebab_case(input: &str) -> String;
pub fn pluralize(word: &str, count: usize) -> String;
pub fn indent(text: &str, levels: usize, style: IndentationStyle) -> String;
pub fn wrap_quotes(text: &str, quote_type: QuoteType) -> String;
```

### Validation Framework

```rust
#[derive(Debug, Clone)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub range: Option<Range>,
    pub suggestion: Option<String>,
    pub context: HashMap<String, String>,
}

pub enum ValidationCheck {
    Syntax(SyntaxCheck),
    Semantics(SemanticCheck),
    Performance(PerformanceCheck),
    Security(SecurityCheck),
    Style(StyleCheck),
}

pub struct CodeValidator {
    checks: Vec<ValidationCheck>,
}

impl CodeValidator {
    pub fn new(checks: Vec<ValidationCheck>) -> Self;
    pub fn add_check(&mut self, check: ValidationCheck);
    pub fn validate(&self, code: &str, language: ProgrammingLanguage) -> Vec<ValidationIssue>;
    pub fn validate_with_config(&self, code: &str, config: &ValidationConfig) -> Vec<ValidationIssue>;
    pub fn is_safe(&self, code: &str, language: ProgrammingLanguage) -> bool;
}
```

---

## rust-ai-ide-shared-services

### LSP Client

```rust
pub struct LspClient {
    language: ProgrammingLanguage,
    server_process: Child,
    stdin: WriteHalf,
    stdout: ReadHalf,
    pending_requests: HashMap<RequestId, PendingRequest>,
}

#[derive(Debug, Clone)]
pub struct LspRequest {
    pub id: RequestId,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug)]
pub struct LspResponse {
    pub id: RequestId,
    pub result: Option<serde_json::Value>,
    pub error: Option<LspError>,
}

impl LspClient {
    pub fn new(language: ProgrammingLanguage, server_command: &str) -> IdeResult<Self>;
    pub async fn initialize(&mut self, project_root: &Path) -> IdeResult<InitializeResult>;
    pub async fn request(&self, request: LspRequest) -> IdeResult<LspResponse>;
    pub async fn notify(&self, method: String, params: serde_json::Value) -> IdeResult<()>;
    pub async fn shutdown(&mut self) -> IdeResult<()>;

    // Completion requests
    pub async fn get_completions(&self, position: Position, file: &Path) -> IdeResult<Vec<CompletionItem>>;
    pub async fn resolve_completion(&self, item: &CompletionItem) -> IdeResult<CompletionItem>;

    // Diagnostics
    pub async fn get_diagnostics(&self, file: &Path) -> IdeResult<Vec<Diagnostic>>;
    pub async fn get_workspace_diagnostics(&self) -> IdeResult<Vec<Diagnostic>>;

    // Navigation
    pub async fn get_definitions(&self, position: Position, file: &Path) -> IdeResult<Vec<Location>>;
    pub async fn get_references(&self, position: Position, file: &Path) -> IdeResult<Vec<Location>>;
    pub async fn get_implementations(&self, position: Position, file: &Path) -> IdeResult<Vec<Location>>;

    // Document operations
    pub async fn format_document(&self, file: &Path) -> IdeResult<String>;
    pub async fn format_range(&self, file: &Path, range: Range) -> IdeResult<String>;
    pub async fn apply_edit(&self, edit: WorkspaceEdit) -> IdeResult<()>;

    // Refactoring
    pub async fn rename_symbol(&self, position: Position, new_name: String, file: &Path) -> IdeResult<WorkspaceEdit>;
    pub async fn prepare_call_hierarchy(&self, position: Position, file: &Path) -> IdeResult<Vec<CallHierarchyItem>>;

    // Document symbols
    pub async fn get_document_symbols(&self, file: &Path) -> IdeResult<Vec<DocumentSymbol>>;
    pub async fn get_workspace_symbols(&self, query: String) -> IdeResult<Vec<SymbolInformation>>;

    // Hover information
    pub async fn get_hover(&self, position: Position, file: &Path) -> IdeResult<Option<Hover>>;
}

#[derive(Debug)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub sort_text: Option<String>,
    pub filter_text: Option<String>,
    pub insert_text: Option<String>,
    pub insert_text_format: Option<InsertTextFormat>,
    pub text_edit: Option<TextEdit>,
    pub additional_text_edits: Option<Vec<TextEdit>>,
    pub command: Option<Command>,
    pub commit_characters: Option<Vec<String>>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}
```

### Workspace Management

```rust
#[derive(Debug)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub root_path: PathBuf,
    pub source_dirs: Vec<PathBuf>,
    pub target_dir: PathBuf,
    pub dependencies: HashMap<String, String>,
    pub features: HashMap<String, Feature>,
    pub workspace_members: Vec<String>,
}

#[derive(Debug)]
pub struct WorkspaceManager {
    config: ProjectConfig,
    file_watcher: Option<FileWatcher>,
    lsp_clients: HashMap<ProgrammingLanguage, LspClient>,
}

impl WorkspaceManager {
    pub fn new(project_root: &Path) -> IdeResult<Self>;
    pub async fn load_config(&mut self) -> IdeResult<()>;
    pub async fn save_config(&self) -> IdeResult<()>;
    pub fn get_config(&self) -> &ProjectConfig;
    pub fn get_config_mut(&mut self) -> &mut ProjectConfig;

    // File operations
    pub async fn get_files(&self, pattern: Option<&str>) -> IdeResult<Vec<PathBuf>>;
    pub async fn read_file(&self, path: &Path) -> IdeResult<String>;
    pub async fn write_file(&self, path: &Path, content: &str) -> IdeResult<()>;
    pub async fn delete_file(&self, path: &Path) -> IdeResult<()>;
    pub async fn create_directory(&self, path: &Path) -> IdeResult<()>;
    pub async fn file_exists(&self, path: &Path) -> bool;
    pub async fn is_directory(&self, path: &Path) -> bool;

    // LSP operations
    pub async fn start_lsp_clients(&mut self) -> IdeResult<()>;
    pub async fn stop_lsp_clients(&mut self) -> IdeResult<()>;
    pub async fn get_lsp_client(&self, language: ProgrammingLanguage) -> Option<&LspClient>;
    pub async fn get_diagnostics(&self) -> IdeResult<Vec<WorkspaceDiagnostic>>;
    pub async fn apply_workspace_edit(&self, edit: WorkspaceEdit) -> IdeResult<()>;

    // Project analysis
    pub async fn analyze_dependencies(&self) -> IdeResult<DependencyGraph>;
    pub async fn find_references(&self, symbol: &str, file: &Path, position: Position) -> IdeResult<Vec<Location>>;
    pub async fn get_definitions(&self, symbol: &str, file: &Path, position: Position) -> IdeResult<Vec<Location>>;
    pub async fn rename_symbol(&self, symbol: &str, new_name: &str, position: Position, file: &Path) -> IdeResult<WorkspaceEdit>;

    // Build operations
    pub async fn build_project(&self) -> IdeResult<BuildResult>;
    pub async fn run_tests(&self) -> IdeResult<TestResult>;
    pub async fn clean_build_artifacts(&self) -> IdeResult<()>;

    // File watching
    pub async fn watch_files(&mut self, patterns: Vec<String>) -> IdeResult<()>;
    pub async fn unwatch_files(&mut self) -> IdeResult<()>;
    pub async fn on_file_change<F>(&mut self, callback: F) -> IdeResult<()>
    where F: Fn(&FileChangeEvent) + Send + Sync + 'static;
}
```

### Service Orchestration

```rust
#[derive(Debug, Clone)]
pub enum ServiceType {
    LSP(ProgrammingLanguage),
    Codegen,
    Formatting,
    Testing,
    Debugging,
    Analysis,
}

#[derive(Debug)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed(String),
    Degraded(String),
}

pub trait ServiceManager {
    async fn start_service(&self, service_type: ServiceType) -> IdeResult<()>;
    async fn stop_service(&self, service_type: ServiceType) -> IdeResult<()>;
    async fn restart_service(&self, service_type: ServiceType) -> IdeResult<()>;
    async fn get_service_status(&self, service_type: ServiceType) -> ServiceState;
    async fn list_services(&self) -> Vec<(ServiceType, ServiceState)>;
}

#[async_trait]
pub trait ServiceRunner {
    async fn start(&self) -> IdeResult<()>;
    async fn stop(&self) -> IdeResult<()>;
    async fn status(&self) -> IdeResult<ServiceStatus>;
    async fn health_check(&self) -> IdeResult<HealthStatus>;
}

#[derive(Debug)]
pub struct OrchestratorResult<T> {
    pub result: T,
    pub service_utilization: HashMap<ServiceType, ServiceMetrics>,
    pub warnings: Vec<String>,
    pub executed_in: Duration,
}

pub struct ServiceOrchestrator<Registry> {
    services: Arc<RwLock<Registry>>,
    executor: Arc<dyn Executor>,
}

impl<Registry> ServiceOrchestrator<Registry> {
    pub fn orchestrate_operation<F, Fut, T>(&self, operation: F) -> IdeResult<OrchestratorResult<T>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = IdeResult<T>> + Send + 'static,
        T: Send + 'static;
}
```

### Diagnostic Aggregation

```rust
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub code_description: Option<CodeDescription>,
    pub source: Option<String>,
    pub message: String,
    pub tags: Option<Vec<DiagnosticTag>>,
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

#[derive(Debug)]
pub struct WorkspaceDiagnostic {
    pub uri: Uri,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct DiagnosticAggregator {
    sources: HashMap<String, Box<dyn DiagnosticSource>>,
}

impl DiagnosticAggregator {
    pub fn new() -> Self;
    pub async fn add_source(&mut self, name: String, source: Box<dyn DiagnosticSource>);
    pub async fn remove_source(&self, name: &str);
    pub async fn get_all_diagnostics(&self) -> IdeResult<Vec<WorkspaceDiagnostic>>;
    pub async fn get_diagnostics_for_file(&self, uri: &Uri) -> IdeResult<Vec<Diagnostic>>;
}

#[async_trait]
pub trait DiagnosticSource {
    async fn collect_diagnostics(&self, workspace: &WorkspaceManager) -> IdeResult<Vec<WorkspaceDiagnostic>>;
    fn name(&self) -> &str;
    fn supported_languages(&self) -> Vec<ProgrammingLanguage>;
}
```

---

## Import Patterns

### Primary Pattern (Recommended)

```rust
// Always include this block at the top of every Rust module
use rust_ai_ide_common::{
    // Core ecosystem types
    ProgrammingLanguage, Position, Range, Location,
    IdeError, IdeResult,

    // Essential utilities
    PerformanceMetrics, time_operation,
    Cache, MemoryCache,

    // File operations
    fs_utils::*,
};

// Language-specific imports
use rust_ai_ide_shared_codegen::{CodeGenerator, AstParser};
use rust_ai_ide_shared_services::{WorkspaceManager, LspClient};
```

### Minimal Pattern (Quick Prototypes)

```rust
// For minimal imports in prototypes/experiments
use rust_ai_ide_common::*; // Pulls all public APIs
use rust_ai_ide_shared_codegen::*;
use rust_ai_ide_shared_services::*;
```

### Feature-Specific Patterns

```rust
// For cache-heavy operations
use rust_ai_ide_common::{
    Cache, MemoryCache, CacheStats,
    CacheEntry, EvictionPolicy
};

// For LSP operations
use rust_ai_ide_shared_services::{
    LspClient, LspRequest, LspResponse,
    CompletionItem, CompletionItemKind
};

// For code generation
use rust_ai_ide_shared_codegen::{
    GenerateRequest, GenerationConfig,
    EntityType, FieldDef
};
```

---

## Version Information

| Component | Version | Status |
|-----------|---------|--------|
| rust-ai-ide-common | 0.1.0 | Stable |
| rust-ai-ide-shared-codegen | 0.1.0 | Stable |
| rust-ai-ide-shared-services | 0.1.0 | Stable |
| Architecture Pattern | Unified v2.4.0 | Active |

**API Stability**: All documented APIs are stable and follow semantic versioning. Breaking changes are announced with deprecation warnings 3 versions in advance.

**Support**: For API questions or suggestions, use the GitHub discussions or create an issue with the "api" label.

> **Navigation Tip**: Jump between related types using the search function in your IDE for faster exploration.
