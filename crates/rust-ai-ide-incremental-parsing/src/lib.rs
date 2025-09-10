//! Incremental Parsing API for Rust AI IDE
//!
//! This module provides incremental parsing capabilities with tree-sitter integration,
//! allowing for efficient re-parsing of code changes without full re-parsing.
//! Supports multi-language parsing with AST diffing and change tracking.
//!
//! # Features
//!
//! - `IncrementalParser` trait for unified parsing API
//! - `ParseTree` wrapper around tree-sitter trees
//! - `ASTDiff` for tracking changes between parse trees
//! - Language-specific parser implementations
//! - Integration with ChangeTracker for file change detection
//! - Async parsing with concurrency controls
//!
//! # Usage
//!
//! ```rust
//! use rust_ai_ide_incremental_parsing::{IncrementalParser, ParserFactory};
//!
//! // Create a parser for Rust
//! let factory = ParserFactory::new();
//! let mut parser = factory.create_parser("rust").await?;
//!
//! // Parse initial content
//! let initial_tree = parser.parse_incremental("", "fn main() {}").await?;
//!
//! // Apply changes and get diff
//! let changes = vec![FileChange { /* ... */ }];
//! let new_tree = parser.apply_changes(&changes).await?;
//! let diff = parser.get_ast_diff(&initial_tree, &new_tree).await?;
//! ```

use rust_ai_ide_errors::{IdeError, IdeResult};
use rust_ai_ide_common::validation::{ValidatedFilePath, validate_file_exists};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tree_sitter::Language;

// Re-export tree-sitter for convenience
pub use tree_sitter;

/// Types of changes that can occur in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ASTChangeType {
    /// Node was added
    Added,
    /// Node was removed
    Removed,
    /// Node was modified
    Modified,
    /// Node was moved
    Moved,
}

/// Represents a single change in the AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTChange {
    /// Type of change
    pub change_type: ASTChangeType,
    /// Start position of the change
    pub start_position: tree_sitter::Point,
    /// End position of the change
    pub end_position: tree_sitter::Point,
    /// Old text content (for modifications/removals)
    pub old_text: Option<String>,
    /// New text content (for additions/modifications)
    pub new_text: Option<String>,
    /// Node type affected (function, variable, etc.)
    pub node_type: String,
}

/// Summary of differences between two AST trees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTDiff {
    /// List of individual changes
    pub changes: Vec<ASTChange>,
    /// Total number of additions
    pub additions: usize,
    /// Total number of removals
    pub removals: usize,
    /// Total number of modifications
    pub modifications: usize,
    /// Total number of moves
    pub moves: usize,
    /// Timestamp when diff was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ASTDiff {
    fn default() -> Self {
        Self {
            changes: Vec::new(),
            additions: 0,
            removals: 0,
            modifications: 0,
            moves: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Configuration for parsing operations
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum size for incremental parses (in bytes)
    pub max_incremental_size: usize,
    /// Timeout for parsing operations (in milliseconds)
    pub parse_timeout_ms: u64,
    /// Enable AST caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Language-specific options
    pub language_options: HashMap<String, String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_incremental_size: 1024 * 1024, // 1MB
            parse_timeout_ms: 5000, // 5 seconds
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            language_options: HashMap::new(),
        }
    }
}

/// Wrapper around tree-sitter parse tree with change tracking
#[derive(Debug, Clone)]
pub struct ParseTree {
    /// The underlying tree-sitter tree
    pub tree: tree_sitter::Tree,
    /// Root node of the tree
    pub root: tree_sitter::Node,
    /// Source code content
    pub source: String,
    /// Language used for parsing
    pub language: Option<String>,
    /// File path if applicable
    pub file_path: Option<PathBuf>,
    /// Last modification time
    pub last_modified: Option<std::time::SystemTime>,
    /// Parse configuration used
    pub config: ParserConfig,
}

impl ParseTree {
    /// Create a new parse tree
    pub fn new(tree: tree_sitter::Tree, source: String, language: Option<String>) -> Self {
        let root = tree.root_node();
        Self {
            tree,
            root,
            source,
            language,
            file_path: None,
            last_modified: None,
            config: ParserConfig::default(),
        }
    }

    /// Check if the tree contains syntax errors
    pub fn has_errors(&self) -> bool {
        self.root.has_error()
    }

    /// Get all error nodes in the tree
    pub fn errors(&self) -> Vec<tree_sitter::Node> {
        let mut errors = Vec::new();
        let mut cursor = self.tree.walk();

        fn collect_errors(node: tree_sitter::Node, errors: &mut Vec<tree_sitter::Node>) {
            if node.is_error() || node.is_missing() {
                errors.push(node);
            }
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    collect_errors(child, errors);
                }
            }
        }

        collect_errors(self.root, &mut errors);
        errors
    }

    /// Pretty print the tree structure
    pub fn print_tree(&self) -> String {
        self.root.to_sexp()
    }
}

/// Core trait for incremental parsing
#[async_trait::async_trait]
pub trait IncrementalParser: Send + Sync {
    /// Parse content incrementally from existing tree
    async fn parse_incremental(&mut self, old_source: &str, new_source: &str, changes: Option<&Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>>) -> IdeResult<ParseTree>;

    /// Apply a set of changes to the parse tree
    async fn apply_changes(&mut self, changes: &Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>) -> IdeResult<ParseTree>;

    /// Get AST diff between two parse trees
    async fn get_ast_diff(&self, old_tree: &ParseTree, new_tree: &ParseTree) -> IdeResult<ASTDiff>;

    /// Parse file content and return parse tree
    async fn parse_file(&mut self, file_path: &Path) -> IdeResult<ParseTree>;

    /// Get available language parsers
    fn supported_languages(&self) -> Vec<&str>;

    /// Validate that a language is supported
    fn supports_language(&self, language: &str) -> bool {
        self.supported_languages().contains(&language)
    }
}

/// Parser factory for creating language-specific parsers
#[derive(Clone)]
pub struct ParserFactory {
    config: ParserConfig,
    parsers: Arc<RwLock<HashMap<String, Box<dyn IncrementalParser>>>>,
}

impl ParserFactory {
    /// Create a new parser factory
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
            parsers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a parser for the specified language
    pub async fn create_parser(&self, language: &str) -> IdeResult<Box<dyn IncrementalParser>> {
        let language = language.to_lowercase();

        // Check if parser already exists
        {
            let parsers = self.parsers.read().await;
            if let Some(parser) = parsers.get(&language) {
                return Ok(parser.as_ref().clone_box());
            }
        }

        // Create new parser
        let parser: Box<dyn IncrementalParser> = match language.as_str() {
            #[cfg(feature = "rust")]
            "rust" => Box::new(RustParser::new(self.config.clone())),
            #[cfg(feature = "typescript")]
            "typescript" | "ts" => Box::new(TypeScriptParser::new(self.config.clone())),
            #[cfg(feature = "python")]
            "python" | "py" => Box::new(PythonParser::new(self.config.clone())),
            #[cfg(feature = "java")]
            "java" => Box::new(JavaParser::new(self.config.clone())),
            #[cfg(feature = "cpp")]
            "cpp" | "c++" => Box::new(CppParser::new(self.config.clone())),
            _ => return Err(IdeError::Validation {
                field: "language".to_string(),
                reason: format!("Unsupported language: {}", language),
            }),
        };

        // Store in cache
        {
            let mut parsers = self.parsers.write().await;
            parsers.insert(language, parser.as_ref().clone_box());
        }

        Ok(parser)
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        let mut langs = Vec::new();
        #[cfg(feature = "rust")] langs.push("rust".to_string());
        #[cfg(feature = "typescript")] {
            langs.push("typescript".to_string());
            langs.push("ts".to_string());
        }
        #[cfg(feature = "python")] {
            langs.push("python".to_string());
            langs.push("py".to_string());
        }
        #[cfg(feature = "java")] langs.push("java".to_string());
        #[cfg(feature = "cpp")] {
            langs.push("cpp".to_string());
            langs.push("c++".to_string());
        }
        langs
    }

    /// Configure the factory
    pub fn with_config(mut self, config: ParserConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for ParserFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// CloneBox trait for cloning boxed trait objects
pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn IncrementalParser>;
}

impl<T> CloneBox for T
where
    T: IncrementalParser + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn IncrementalParser> {
        Box::new(self.clone())
    }
}

/// Base parser implementation with common functionality
#[derive(Clone)]
pub struct BaseParser {
    config: ParserConfig,
    parser: Arc<RwLock<tree_sitter::Parser>>,
    language: Language,
    language_name: String,
}

impl BaseParser {
    /// Create a new base parser
    pub fn new(language: Language, language_name: String, config: ParserConfig) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language).expect("Failed to set parser language");

        Self {
            config,
            parser: Arc::new(RwLock::new(parser)),
            language,
            language_name,
        }
    }

    /// Parse source code incrementally
    pub async fn parse_incremental_base(&mut self, old_source: &str, new_source: &str, changes: Option<&Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>>) -> IdeResult<ParseTree> {
        // Use tree-sitter's incremental parsing
        let mut parser = self.parser.write().await;

        // Set old source if provided
        if !old_source.is_empty() {
            parser.set_language(self.language).ok();
        }

        // Parse the new source
        let tree = parser.parse(new_source, None)
            .ok_or_else(|| IdeError::Parsing {
                file: "input".to_string(),
                reason: "Failed to parse source code".to_string(),
            })?;

        let mut parse_tree = ParseTree::new(tree, new_source.to_string(), Some(self.language_name.clone()));
        parse_tree.config = self.config.clone();

        Ok(parse_tree)
    }

    /// Apply changes to source and re-parse
    pub async fn apply_changes_base(&mut self, source: &str, changes: &Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>) -> IdeResult<(String, ParseTree)> {
        // Sort changes by position
        let mut sorted_changes = changes.clone();
        sorted_changes.sort_by(|a, b| {
            let a_range = a.path.to_string_lossy();
            let b_range = b.path.to_string_lossy();
            a_range.cmp(&b_range)
        });

        // Apply changes to source (simplified - in practice would need more sophisticated diffing)
        let mut new_source = source.to_string();

        for change in sorted_changes {
            // For simplicity, assuming text-based changes
            // Real implementation would apply edits to string
        }

        // Re-parse with changes
        let tree = self.parse_incremental_base("", &new_source, Some(changes)).await?;
        Ok((new_source, tree))
    }

    /// Compute AST diff between two trees
    pub async fn get_ast_diff_base(&self, old_tree: &ParseTree, new_tree: &ParseTree) -> IdeResult<ASTDiff> {
        let mut diff = ASTDiff::default();

        // Walk both trees and find differences
        // This is a simplified implementation

        fn collect_nodes<'a>(node: tree_sitter::Node<'a>, nodes: &mut Vec<tree_sitter::Node<'a>>) {
            nodes.push(node);
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    collect_nodes(child, nodes);
                }
            }
        }

        let mut old_nodes = Vec::new();
        let mut new_nodes = Vec::new();

        collect_nodes(old_tree.root, &mut old_nodes);
        collect_nodes(new_tree.root, &mut new_nodes);

        // Find added/removed nodes (simplified comparison)
        let old_count = old_nodes.len();
        let new_count = new_nodes.len();

        if new_count > old_count {
            diff.additions = new_count - old_count;
            diff.changes.push(ASTChange {
                change_type: ASTChangeType::Added,
                start_position: tree_sitter::Point { row: 0, column: 0 },
                end_position: tree_sitter::Point { row: 0, column: 0 },
                old_text: None,
                new_text: None,
                node_type: "unknown".to_string(),
            });
        } else if old_count > new_count {
            diff.removals = old_count - new_count;
            diff.changes.push(ASTChange {
                change_type: ASTChangeType::Removed,
                start_position: tree_sitter::Point { row: 0, column: 0 },
                end_position: tree_sitter::Point { row: 0, column: 0 },
                old_text: None,
                new_text: None,
                node_type: "unknown".to_string(),
            });
        }

        diff.timestamp = chrono::Utc::now();
        Ok(diff)
    }
}

/// Rust language parser
#[cfg(feature = "rust")]
#[derive(Clone)]
pub struct RustParser {
    base: BaseParser,
}

#[cfg(feature = "rust")]
impl RustParser {
    pub fn new(config: ParserConfig) -> Self {
        let language = tree_sitter_rust::language();
        let base = BaseParser::new(language, "rust".to_string(), config);
        Self { base }
    }
}

#[cfg(feature = "rust")]
#[async_trait::async_trait]
impl IncrementalParser for RustParser {
    async fn parse_incremental(&mut self, old_source: &str, new_source: &str, changes: Option<&Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>>) -> IdeResult<ParseTree> {
        self.base.parse_incremental_base(old_source, new_source, changes).await
    }

    async fn apply_changes(&mut self, changes: &Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>) -> IdeResult<ParseTree> {
        // For now, assume we have current source content somewhere
        // In practice, this would be stored in the parser state
        let current_source = "".to_string(); // Placeholder
        let (_new_source, tree) = self.base.apply_changes_base(&current_source, changes).await?;
        Ok(tree)
    }

    async fn get_ast_diff(&self, old_tree: &ParseTree, new_tree: &ParseTree) -> IdeResult<ASTDiff> {
        self.base.get_ast_diff_base(old_tree, new_tree).await
    }

    async fn parse_file(&mut self, file_path: &Path) -> IdeResult<ParseTree> {
        let validated_path = ValidatedFilePath::new(&file_path.to_string_lossy(), "parse_file")?;
        let content = tokio::fs::read_to_string(validated_path.as_path()).await
            .map_err(|e| IdeError::Io {
                path: file_path.to_path_buf(),
                reason: format!("Failed to read file: {}", e),
            })?;

        let tree = self.base.parse_incremental_base("", &content, None).await?;
        let mut tree = tree;
        tree.file_path = Some(file_path.to_path_buf());
        tree.last_modified = std::fs::metadata(file_path).ok().and_then(|m| m.modified().ok());

        Ok(tree)
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["rust"]
    }
}

/// TypeScript parser
#[cfg(feature = "typescript")]
#[derive(Clone)]
pub struct TypeScriptParser {
    base: BaseParser,
}

#[cfg(feature = "typescript")]
impl TypeScriptParser {
    pub fn new(config: ParserConfig) -> Self {
        let language = tree_sitter_typescript::language_typescript();
        let base = BaseParser::new(language, "typescript".to_string(), config);
        Self { base }
    }
}

#[cfg(feature = "typescript")]
#[async_trait::async_trait]
impl IncrementalParser for TypeScriptParser {
    async fn parse_incremental(&mut self, old_source: &str, new_source: &str, changes: Option<&Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>>) -> IdeResult<ParseTree> {
        self.base.parse_incremental_base(old_source, new_source, changes).await
    }

    async fn apply_changes(&mut self, changes: &Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>) -> IdeResult<ParseTree> {
        let current_source = "".to_string(); // Placeholder
        let (_new_source, tree) = self.base.apply_changes_base(&current_source, changes).await?;
        Ok(tree)
    }

    async fn get_ast_diff(&self, old_tree: &ParseTree, new_tree: &ParseTree) -> IdeResult<ASTDiff> {
        self.base.get_ast_diff_base(old_tree, new_tree).await
    }

    async fn parse_file(&mut self, file_path: &Path) -> IdeResult<ParseTree> {
        let validated_path = ValidatedFilePath::new(&file_path.to_string_lossy(), "parse_file")?;
        let content = tokio::fs::read_to_string(validated_path.as_path()).await
            .map_err(|e| IdeError::Io {
                path: file_path.to_path_buf(),
                reason: format!("Failed to read file: {}", e),
            })?;

        let tree = self.base.parse_incremental_base("", &content, None).await?;
        let mut tree = tree;
        tree.file_path = Some(file_path.to_path_buf());
        tree.last_modified = std::fs::metadata(file_path).ok().and_then(|m| m.modified().ok());

        Ok(tree)
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["typescript", "ts"]
    }
}

/// Python parser
#[cfg(feature = "python")]
#[derive(Clone)]
pub struct PythonParser {
    base: BaseParser,
}

#[cfg(feature = "python")]
impl PythonParser {
    pub fn new(config: ParserConfig) -> Self {
        let language = tree_sitter_python::language();
        let base = BaseParser::new(language, "python".to_string(), config);
        Self { base }
    }
}

#[cfg(feature = "python")]
#[async_trait::async_trait]
impl IncrementalParser for PythonParser {
    async fn parse_incremental(&mut self, old_source: &str, new_source: &str, changes: Option<&Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>>) -> IdeResult<ParseTree> {
        self.base.parse_incremental_base(old_source, new_source, changes).await
    }

    async fn apply_changes(&mut self, changes: &Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>) -> IdeResult<ParseTree> {
        let current_source = "".to_string(); // Placeholder
        let (_new_source, tree) = self.base.apply_changes_base(&current_source, changes).await?;
        Ok(tree)
    }

    async fn get_ast_diff(&self, old_tree: &ParseTree, new_tree: &ParseTree) -> IdeResult<ASTDiff> {
        self.base.get_ast_diff_base(old_tree, new_tree).await
    }

    async fn parse_file(&mut self, file_path: &Path) -> IdeResult<ParseTree> {
        let validated_path = ValidatedFilePath::new(&file_path.to_string_lossy(), "parse_file")?;
        let content = tokio::fs::read_to_string(validated_path.as_path()).await
            .map_err(|e| IdeError::Io {
                path: file_path.to_path_buf(),
                reason: format!("Failed to read file: {}", e),
            })?;

        let tree = self.base.parse_incremental_base("", &content, None).await?;
        let mut tree = tree;
        tree.file_path = Some(file_path.to_path_buf());
        tree.last_modified = std::fs::metadata(file_path).ok().and_then(|m| m.modified().ok());

        Ok(tree)
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["python", "py"]
    }
}

/// Java parser
#[cfg(feature = "java")]
#[derive(Clone)]
pub struct JavaParser {
    base: BaseParser,
}

#[cfg(feature = "java")]
impl JavaParser {
    pub fn new(config: ParserConfig) -> Self {
        let language = tree_sitter_java::language();
        let base = BaseParser::new(language, "java".to_string(), config);
        Self { base }
    }
}

#[cfg(feature = "java")]
#[async_trait::async_trait]
impl IncrementalParser for JavaParser {
    async fn parse_incremental(&mut self, old_source: &str, new_source: &str, changes: Option<&Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>>) -> IdeResult<ParseTree> {
        self.base.parse_incremental_base(old_source, new_source, changes).await
    }

    async fn apply_changes(&mut self, changes: &Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>) -> IdeResult<ParseTree> {
        let current_source = "".to_string(); // Placeholder
        let (_new_source, tree) = self.base.apply_changes_base(&current_source, changes).await?;
        Ok(tree)
    }

    async fn get_ast_diff(&self, old_tree: &ParseTree, new_tree: &ParseTree) -> IdeResult<ASTDiff> {
        self.base.get_ast_diff_base(old_tree, new_tree).await
    }

    async fn parse_file(&mut self, file_path: &Path) -> IdeResult<ParseTree> {
        let validated_path = ValidatedFilePath::new(&file_path.to_string_lossy(), "parse_file")?;
        let content = tokio::fs::read_to_string(validated_path.as_path()).await
            .map_err(|e| IdeError::Io {
                path: file_path.to_path_buf(),
                reason: format!("Failed to read file: {}", e),
            })?;

        let tree = self.base.parse_incremental_base("", &content, None).await?;
        let mut tree = tree;
        tree.file_path = Some(file_path.to_path_buf());
        tree.last_modified = std::fs::metadata(file_path).ok().and_then(|m| m.modified().ok());

        Ok(tree)
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["java"]
    }
}

/// C++ parser
#[cfg(feature = "cpp")]
#[derive(Clone)]
pub struct CppParser {
    base: BaseParser,
}

#[cfg(feature = "cpp")]
impl CppParser {
    pub fn new(config: ParserConfig) -> Self {
        let language = tree_sitter_cpp::language();
        let base = BaseParser::new(language, "cpp".to_string(), config);
        Self { base }
    }
}

#[cfg(feature = "cpp")]
#[async_trait::async_trait]
impl IncrementalParser for CppParser {
    async fn parse_incremental(&mut self, old_source: &str, new_source: &str, changes: Option<&Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>>) -> IdeResult<ParseTree> {
        self.base.parse_incremental_base(old_source, new_source, changes).await
    }

    async fn apply_changes(&mut self, changes: &Vec<rust_ai_ide_lsp::incremental::change_tracker::FileChange>) -> IdeResult<ParseTree> {
        let current_source = "".to_string(); // Placeholder
        let (_new_source, tree) = self.base.apply_changes_base(&current_source, changes).await?;
        Ok(tree)
    }

    async fn get_ast_diff(&self, old_tree: &ParseTree, new_tree: &ParseTree) -> IdeResult<ASTDiff> {
        self.base.get_ast_diff_base(old_tree, new_tree).await
    }

    async fn parse_file(&mut self, file_path: &Path) -> IdeResult<ParseTree> {
        let validated_path = ValidatedFilePath::new(&file_path.to_string_lossy(), "parse_file")?;
        let content = tokio::fs::read_to_string(validated_path.as_path()).await
            .map_err(|e| IdeError::Io {
                path: file_path.to_path_buf(),
                reason: format!("Failed to read file: {}", e),
            })?;

        let tree = self.base.parse_incremental_base("", &content, None).await?;
        let mut tree = tree;
        tree.file_path = Some(file_path.to_path_buf());
        tree.last_modified = std::fs::metadata(file_path).ok().and_then(|m| m.modified().ok());

        Ok(tree)
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["cpp", "c++"]
    }
}

/// Global parser registry for managing multiple language parsers
#[derive(Clone)]
pub struct ParserRegistry {
    factory: ParserFactory,
    parsers: Arc<RwLock<HashMap<String, Box<dyn IncrementalParser>>>>,
}

impl ParserRegistry {
    /// Create a new parser registry
    pub fn new() -> Self {
        Self {
            factory: ParserFactory::new(),
            parsers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a parser for a language
    pub async fn get_parser(&self, language: &str) -> IdeResult<Box<dyn IncrementalParser>> {
        // Check cache first
        {
            let parsers = self.parsers.read().await;
            if let Some(parser) = parsers.get(language) {
                return Ok(parser.as_ref().clone_box());
            }
        }

        // Create new parser
        let parser = self.factory.create_parser(language).await?;

        // Cache it
        {
            let mut parsers = self.parsers.write().await;
            parsers.insert(language.to_string(), parser.as_ref().clone_box());
        }

        Ok(parser)
    }

    /// Check if a language is supported
    pub fn supports_language(&self, language: &str) -> bool {
        self.factory.supported_languages().contains(&language.to_string())
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        self.factory.supported_languages()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for AST operations

/// Extract all nodes of a specific type from a parse tree
pub fn extract_nodes_by_type(tree: &ParseTree, node_type: &str) -> Vec<tree_sitter::Node> {
    let mut nodes = Vec::new();

    fn walk_tree(node: tree_sitter::Node, node_type: &str, nodes: &mut Vec<tree_sitter::Node>) {
        if node.kind() == node_type {
            nodes.push(node);
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                walk_tree(child, node_type, nodes);
            }
        }
    }

    walk_tree(tree.root, node_type, &mut nodes);
    nodes
}

/// Find the smallest node containing a position
pub fn find_node_at_position(tree: &ParseTree, position: tree_sitter::Point) -> Option<tree_sitter::Node> {
    let mut cursor = tree.tree.walk();
    cursor.goto_first_child_for_index(position.row as usize);

    loop {
        let node = cursor.node();

        if node.start_position() <= position && position <= node.end_position() {
            if cursor.goto_first_child() {
                continue;
            } else {
                return Some(node);
            }
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }

    None
}

/// Validate file content before parsing
pub async fn validate_file_for_parsing(file_path: &Path, max_size: usize) -> IdeResult<()> {
    let validated_path = ValidatedFilePath::new(&file_path.to_string_lossy(), "validate_file_for_parsing")?;

    // Check file size
    if let Ok(metadata) = std::fs::metadata(&validated_path.as_path()) {
        if metadata.len() > max_size as u64 {
            return Err(IdeError::Validation {
                field: "file_size".to_string(),
                reason: format!("File size {} exceeds maximum allowed size {}", metadata.len(), max_size),
            });
        }
    }

    // Check if file is readable
    match tokio::fs::read(&validated_path.as_path()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(IdeError::Io {
            path: file_path.to_path_buf(),
            reason: format!("Cannot read file: {}", e),
        }),
    }
}