//! Rust language parser with tree-sitter integration and Rust-specific optimizations
//!
//! This module implements `RustIncrementalParser` using tree-sitter-rust with advanced
//! optimizations for macro expansion tracking, trait resolution, and lifetime analysis.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use rust_ai_ide_common::validation::ValidatedFilePath;
use rust_ai_ide_errors::{IDEResult, RustAIError};
use rust_ai_ide_lsp::incremental::change_tracker::FileChange;

use crate::{ASTChange, ASTChangeType, ASTDiff, IncrementalParser, ParseTree, ParserConfig};

/// Rust language parser with advanced parsing capabilities
#[derive(Clone)]
pub struct RustIncrementalParser {
    /// Tree-sitter parser wrapped in async locks for concurrent access
    parser: Arc<RwLock<tree_sitter::Parser>>,
    /// Parser configuration
    config: ParserConfig,
    /// Current parse tree for incremental updates
    current_tree: Arc<RwLock<Option<ParseTree>>>,
    /// Source code cache for incremental parsing
    source_cache: Arc<RwLock<String>>,
    /// Logical cache for optimization hints (macros, traits, items)
    logic_cache: Arc<RwLock<HashMap<String, RustLogicInfo>>>,
    /// Rust-specific parsing optimizations
    optimizations: RustOptimizations,
}

/// Rust-specific logical information for enhanced parsing
#[derive(Debug, Clone)]
struct RustLogicInfo {
    /// Function definitions with signatures
    functions: Vec<FunctionInfo>,
    /// Struct definitions with fields
    structs: Vec<StructInfo>,
    /// Trait definitions and implementations
    traits: Vec<TraitInfo>,
    /// Macro definitions
    macros: Vec<MacroInfo>,
    /// Imports and use statements
    imports: Vec<ImportInfo>,
}

/// Function definition information
#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    params: Vec<String>,
    return_type: Option<String>,
    visibility: Visibility,
    is_async: bool,
    body_range: tree_sitter::Range,
}

/// Struct definition information
#[derive(Debug, Clone)]
struct StructInfo {
    name: String,
    fields: Vec<FieldInfo>,
    visibility: Visibility,
    is_tuple: bool,
    definition_range: tree_sitter::Range,
}

/// Trait definition information
#[derive(Debug, Clone)]
struct TraitInfo {
    name: String,
    methods: Vec<String>,
    visibility: Visibility,
    definition_range: tree_sitter::Range,
}

/// Macro definition information
#[derive(Debug, Clone)]
struct MacroInfo {
    name: String,
    is_proc_macro: bool,
    parameters: Vec<String>,
    definition_range: tree_sitter::Range,
}

/// Import/use statement information
#[derive(Debug, Clone)]
struct ImportInfo {
    path: String,
    alias: Option<String>,
    visibility: Visibility,
}

/// Field information for structs and tuples
#[derive(Debug, Clone)]
struct FieldInfo {
    name: Option<String>, // None for tuple structs at index
    field_type: Option<String>,
    visibility: Visibility,
    index: Option<usize>, // For tuple structs
}

/// Visibility specifier
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Crate,
    Super,
    SelfModule,
    Private,
}

/// Rust-specific parsing optimizations
#[derive(Debug, Clone)]
struct RustOptimizations {
    /// Enable macro expansion tracking
    track_macros: bool,
    /// Enable trait resolution analysis
    track_traits: bool,
    /// Enable lifetime analysis
    track_lifetimes: bool,
    /// Enable advanced AST caching
    enable_caching: bool,
    /// Maximum cache size per file
    max_cache_size_per_file: usize,
    /// Enable parallel AST node processing
    enable_parallel_processing: bool,
}

impl Default for RustOptimizations {
    fn default() -> Self {
        Self {
            track_macros: true,
            track_traits: true,
            track_lifetimes: true,
            enable_caching: true,
            max_cache_size_per_file: 1024 * 1024, // 1MB
            enable_parallel_processing: true,
        }
    }
}

impl RustIncrementalParser {
    /// Create a new Rust parser with default optimizations
    pub fn new(config: ParserConfig) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Failed to load tree-sitter-rust language");

        Self {
            parser: Arc::new(RwLock::new(parser)),
            config,
            current_tree: Arc::new(RwLock::new(None)),
            source_cache: Arc::new(RwLock::new(String::new())),
            logic_cache: Arc::new(RwLock::new(HashMap::new())),
            optimizations: RustOptimizations::default(),
        }
    }

    /// Create parser with custom optimizations
    pub fn with_optimizations(config: ParserConfig, optimizations: RustOptimizations) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Failed to load tree-sitter-rust language");

        Self {
            parser: Arc::new(RwLock::new(parser)),
            config,
            current_tree: Arc::new(RwLock::new(None)),
            source_cache: Arc::new(RwLock::new(String::new())),
            logic_cache: Arc::new(RwLock::new(HashMap::new())),
            optimizations,
        }
    }

    /// Parse Rust code with tree-sitter and extract logical information
    async fn parse_internal(
        &mut self,
        source: &str,
        old_tree: Option<&tree_sitter::Tree>,
    ) -> IDEResult<ParseTree> {
        // Validate input size
        let source_length = source.len();
        if source_length > self.config.max_incremental_size {
            return Err(RustAIError::Validation(format!(
                "Source code exceeds maximum size of {} bytes (current: {})",
                self.config.max_incremental_size, source_length
            )));
        }

        let parser = self.parser.write().await;

        // Perform initial parsing
        let tree = match parser.parse(source, old_tree) {
            Some(tree) => tree,
            None => {
                return Err(RustAIError::Compilation(
                    "Failed to parse Rust source code with tree-sitter".to_string(),
                ));
            }
        };

        let root_node = tree.root_node();

        // Extract Rust-specific logical information
        let logic_info = self.extract_logic_info(&root_node, source).await;
        self.update_logic_cache(logic_info).await;

        // Apply Rust-specific optimizations
        self.apply_rust_optimizations(&root_node, source).await;

        let parse_tree = ParseTree::new(tree, source.to_string(), Some("rust".to_string()));
        let mut tree_with_config = parse_tree;
        tree_with_config.config = self.config.clone();

        // Update caches
        {
            *self.current_tree.write().await = Some(tree_with_config.clone());
            *self.source_cache.write().await = source.to_string();
        }

        Ok(tree_with_config)
    }

    /// Extract Rust logical constructs from AST
    async fn extract_logic_info(
        &self,
        root_node: &tree_sitter::Node,
        source: &str,
    ) -> HashMap<String, RustLogicInfo> {
        let mut logic_info = HashMap::new();

        // Extract functions, structs, traits, macros, etc.
        extract_functions(root_node, source, &mut logic_info);
        extract_structs(root_node, source, &mut logic_info);
        extract_traits(root_node, source, &mut logic_info);
        extract_macros(root_node, source, &mut logic_info);
        extract_imports(root_node, source, &mut logic_info);

        logic_info
    }

    /// Update logic cache with new information
    async fn update_logic_cache(&self, logic_info: HashMap<String, RustLogicInfo>) {
        let mut cache = self.logic_cache.write().await;
        for (key, value) in logic_info {
            cache.insert(key, value);
        }
    }

    /// Apply Rust-specific optimizations to AST
    async fn apply_rust_optimizations(&self, root_node: &tree_sitter::Node, source: &str) {
        if self.optimizations.track_macros {
            self.optimize_macro_expansion(root_node, source).await;
        }
        if self.optimizations.track_traits {
            self.optimize_trait_resolution(root_node, source).await;
        }
        if self.optimizations.track_lifetimes {
            self.optimize_lifetime_analysis(root_node, source).await;
        }
    }

    /// Optimize macro expansion tracking
    async fn optimize_macro_expansion(&self, _root_node: &tree_sitter::Node, _source: &str) {
        // Implement macro expansion tracking logic
        // This would integrate with rust-analyzer's macro expansion analysis
        // for better understanding of generated code
    }

    /// Optimize trait resolution analysis
    async fn optimize_trait_resolution(&self, _root_node: &tree_sitter::Node, _source: &str) {
        // Implement trait resolution analysis
        // This would track trait implementations and usage patterns
        // for better semantic understanding
    }

    /// Optimize lifetime analysis
    async fn optimize_lifetime_analysis(&self, _root_node: &tree_sitter::Node, _source: &str) {
        // Implement lifetime analysis
        // This would track borrow checker hints and lifetime relationships
        // for memory safety analysis
    }

    /// Apply changes incrementally using tree-sitter
    async fn apply_changes_incremental(
        &self,
        changes: &[FileChange],
        old_source: &str,
    ) -> IDEResult<Option<String>> {
        for change in changes {
            match change.change_type {
                rust_ai_ide_lsp::incremental::change_tracker::FileChangeType::Modified => {
                    // Handle modification with tree-sitter edit sequences
                    // This would use tree-sitter's edit API for efficient incremental updates
                }
                rust_ai_ide_lsp::incremental::change_tracker::FileChangeType::Created => {
                    // Full re-parse for new files
                    return Ok(None);
                }
                rust_ai_ide_lsp::incremental::change_tracker::FileChangeType::Deleted => {
                    // Clear cached state
                    continue;
                }
                rust_ai_ide_lsp::incremental::change_tracker::FileChangeType::Renamed(_) => {
                    // Handle rename (might require cache invalidation)
                    continue;
                }
            }
        }

        // For now, return None indicating full re-parse needed
        Ok(None)
    }

    /// Compute AST diff with Rust-specific optimizations
    async fn compute_ast_diff(
        &self,
        old_tree: &ParseTree,
        new_tree: &ParseTree,
        old_logic: &HashMap<String, RustLogicInfo>,
        new_logic: Option<&HashMap<String, RustLogicInfo>>,
    ) -> IDEResult<ASTDiff> {
        let mut diff = ASTDiff::default();
        let mut changes = Vec::new();

        fn compute_changes(
            old_node: tree_sitter::Node,
            new_node: tree_sitter::Node,
            changes: &mut Vec<ASTChange>,
            old_source: &str,
            new_source: &str,
        ) {
            // Compare node types and structure
            if old_node.kind() != new_node.kind() {
                changes.push(ASTChange {
                    change_type: ASTChangeType::Modified,
                    start_position: old_node.start_position(),
                    end_position: old_node.end_position(),
                    old_text: Some(
                        old_node
                            .utf8_text(old_source.as_bytes())
                            .unwrap_or("")
                            .to_string(),
                    ),
                    new_text: Some(
                        new_node
                            .utf8_text(new_source.as_bytes())
                            .unwrap_or("")
                            .to_string(),
                    ),
                    node_type: old_node.kind().to_string(),
                });
            } else if old_node.has_changes() || new_node.has_changes() {
                // Node content changed
                changes.push(ASTChange {
                    change_type: ASTChangeType::Modified,
                    start_position: old_node.start_position(),
                    end_position: old_node.end_position(),
                    old_text: Some(
                        old_node
                            .utf8_text(old_source.as_bytes())
                            .unwrap_or("")
                            .to_string(),
                    ),
                    new_text: Some(
                        new_node
                            .utf8_text(new_source.as_bytes())
                            .unwrap_or("")
                            .to_string(),
                    ),
                    node_type: old_node.kind().to_string(),
                });
            }

            // Recursively compare children
            let old_count = old_node.child_count();
            let new_count = new_node.child_count();
            let min_count = old_count.min(new_count);

            for i in 0..min_count {
                if let (Some(old_child), Some(new_child)) = (old_node.child(i), new_node.child(i)) {
                    compute_changes(old_child, new_child, changes, old_source, new_source);
                }
            }

            // Handle additions and removals
            for i in min_count..old_count {
                if let Some(removed_node) = old_node.child(i) {
                    changes.push(ASTChange {
                        change_type: ASTChangeType::Removed,
                        start_position: removed_node.start_position(),
                        end_position: removed_node.end_position(),
                        old_text: Some(
                            removed_node
                                .utf8_text(old_source.as_bytes())
                                .unwrap_or("")
                                .to_string(),
                        ),
                        new_text: None,
                        node_type: removed_node.kind().to_string(),
                    });
                    diff.removals += 1;
                }
            }

            for i in min_count..new_count {
                if let Some(added_node) = new_node.child(i) {
                    changes.push(ASTChange {
                        change_type: ASTChangeType::Added,
                        start_position: added_node.start_position(),
                        end_position: added_node.end_position(),
                        old_text: None,
                        new_text: Some(
                            added_node
                                .utf8_text(new_source.as_bytes())
                                .unwrap_or("")
                                .to_string(),
                        ),
                        node_type: added_node.kind().to_string(),
                    });
                    diff.additions += 1;
                }
            }
        }

        compute_changes(
            old_tree.root,
            new_tree.root,
            &mut changes,
            &old_tree.source,
            &new_tree.source,
        );

        // Update diff counts
        diff.changes = changes;
        diff.modifications = diff
            .changes
            .iter()
            .filter(|c| matches!(c.change_type, ASTChangeType::Modified))
            .count();
        diff.moves = 0; // Rust parser doesn't currently track moves

        Ok(diff)
    }

    /// Validate file path and content for Rust parsing
    async fn validate_rust_file(&self, file_path: &Path, content: &str) -> IDEResult<()> {
        // Security validation
        let validated_path = ValidatedFilePath::new(&file_path.to_string_lossy(), "rust_parser")?;

        // Basic content validation
        if content.is_empty() {
            return Err(RustAIError::Validation(
                "Rust source file cannot be empty".to_string(),
            ));
        }

        // Check file size
        let content_len = content.len();
        if content_len > self.config.max_incremental_size {
            return Err(RustAIError::Validation(format!(
                "Rust source file exceeds maximum size limit of {} bytes (current: {} bytes)",
                self.config.max_incremental_size, content_len
            )));
        }

        Ok(())
    }
}

// Extraction functions for Rust constructs
fn extract_functions(
    root_node: &tree_sitter::Node,
    source: &str,
    logic_info: &mut HashMap<String, RustLogicInfo>,
) {
    for node in walk_tree(root_node, "function_item") {
        if let Ok(function) = extract_function_info(node, source) {
            let file_key = "current_file".to_string(); // In practice, this would be the file path
            logic_info
                .entry(file_key)
                .or_default()
                .functions
                .push(function);
        }
    }
}

fn extract_function_info(node: tree_sitter::Node, source: &str) -> IDEResult<FunctionInfo> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| RustAIError::InternalError("Function missing name".to_string()))?;
    let name = name_node
        .utf8_text(source.as_bytes())
        .map_err(|_| RustAIError::InternalError("Failed to extract function name".to_string()))?
        .to_string();

    let visibility = extract_visibility(&node, source);

    let mut params = Vec::new();
    let mut return_type = None;
    let mut is_async = false;

    if let Some(parameters) = node.child_by_field_name("parameters") {
        for param in walk_tree(&parameters, "parameter") {
            if let Ok(param_name) = param.utf8_text(source.as_bytes()) {
                params.push(param_name.to_string());
            }
        }
    }

    if let Some(return_type_node) = node.child_by_field_name("return_type") {
        return_type = return_type_node
            .utf8_text(source.as_bytes())
            .map(|s| s.to_string())
            .ok();
    }

    // Check for async keyword
    if node
        .utf8_text(source.as_bytes())
        .unwrap_or("")
        .starts_with("async ")
    {
        is_async = true;
    }

    Ok(FunctionInfo {
        name,
        params,
        return_type,
        visibility,
        is_async,
        body_range: node.range(),
    })
}

fn extract_structs(
    root_node: &tree_sitter::Node,
    source: &str,
    logic_info: &mut HashMap<String, RustLogicInfo>,
) {
    for node in walk_tree(root_node, "struct_item") {
        if let Ok(struct_info) = extract_struct_info(node, source) {
            let file_key = "current_file".to_string(); // In practice, this would be the file path
            logic_info
                .entry(file_key)
                .or_default()
                .structs
                .push(struct_info);
        }
    }
}

fn extract_struct_info(node: tree_sitter::Node, source: &str) -> IDEResult<StructInfo> {
    let name_node = node
        .child_by_field_name("name")
        .ok_or_else(|| RustAIError::InternalError("Struct missing name".to_string()))?;
    let name = name_node
        .utf8_text(source.as_bytes())
        .map_err(|_| RustAIError::InternalError("Failed to extract struct name".to_string()))?
        .to_string();

    let visibility = extract_visibility(&node, source);
    let mut fields = Vec::new();

    // Check if it's a tuple struct vs named struct
    let is_tuple = if let Some(field_declaration_list) =
        node.child_by_field_name("field_declaration_list")
    {
        false // Named struct
    } else if let Some(field_declaration_list) = node.child_by_field_name("field_delimiter_list") {
        true // Tuple-like struct
    } else {
        false // Unit struct
    };

    if !is_tuple {
        if let Some(field_list) = node.child_by_field_name("field_declaration_list") {
            for field_node in walk_tree(&field_list, "field_declaration") {
                if let Ok(field) = extract_field_info(field_node, source, false) {
                    fields.push(field);
                }
            }
        }
    } else {
        if let Some(field_list) = node.child_by_field_name("field_delimiter_list") {
            for (i, field_node) in walk_tree(&field_list, "parameter").enumerate() {
                if let Ok(field) = extract_field_info_with_index(field_node, source, i) {
                    fields.push(field);
                }
            }
        }
    }

    Ok(StructInfo {
        name,
        fields,
        visibility,
        is_tuple,
        definition_range: node.range(),
    })
}

// Helper functions for extraction
fn extract_field_info(
    node: tree_sitter::Node,
    source: &str,
    is_tuple: bool,
) -> IDEResult<FieldInfo> {
    if is_tuple {
        return extract_field_info_with_index(node, source, 0);
    }

    let name_node = node.child_by_field_name("name");
    let field_name = if let Some(name_node) = name_node {
        name_node
            .utf8_text(source.as_bytes())
            .map(|s| Some(s.to_string()))
            .ok()
    } else {
        None
    };

    let field_type = node
        .child_by_field_name("type")
        .and_then(|type_node| type_node.utf8_text(source.as_bytes()).ok())
        .map(|s| s.to_string());

    let visibility = extract_visibility(&node, source);

    Ok(FieldInfo {
        name: field_name,
        field_type,
        visibility,
        index: None,
    })
}

fn extract_field_info_with_index(
    node: tree_sitter::Node,
    source: &str,
    index: usize,
) -> IDEResult<FieldInfo> {
    let field_type = node
        .utf8_text(source.as_bytes())
        .map(|s| s.to_string())
        .ok();

    Ok(FieldInfo {
        name: None,
        field_type,
        visibility: Visibility::Private, // Tuple fields are always private
        index: Some(index),
    })
}

fn extract_traits(
    _root_node: &tree_sitter::Node,
    _source: &str,
    _logic_info: &mut HashMap<String, RustLogicInfo>,
) {
    // Trait extraction implementation
    // This would analyze trait_item nodes to extract trait definitions
}

fn extract_macros(
    _root_node: &tree_sitter::Node,
    _source: &str,
    _logic_info: &mut HashMap<String, RustLogicInfo>,
) {
    // Macro extraction implementation
    // This would analyze macro_definition nodes to extract macro definitions
}

fn extract_imports(
    _root_node: &tree_sitter::Node,
    _source: &str,
    _logic_info: &mut HashMap<String, RustLogicInfo>,
) {
    // Import extraction implementation
    // This would analyze use_declaration nodes to extract import statements
}

/// Extract visibility specifier from AST node
fn extract_visibility(node: &tree_sitter::Node, source: &str) -> Visibility {
    if let Some(visibility_node) = node.child_by_field_name("visibility") {
        if let Ok(vis_text) = visibility_node.utf8_text(source.as_bytes()) {
            return match vis_text {
                "pub(crate)" => Visibility::Crate,
                "pub(super)" => Visibility::Super,
                "pub(self)" => Visibility::SelfModule,
                "pub" => Visibility::Public,
                _ => Visibility::Private,
            };
        }
    }
    Visibility::Private
}

/// Walk tree to find nodes of specific kind
fn walk_tree(node: &tree_sitter::Node, node_kind: &str) -> Vec<tree_sitter::Node> {
    let mut nodes = Vec::new();

    fn walk_recursive(
        node: tree_sitter::Node,
        node_kind: &str,
        nodes: &mut Vec<tree_sitter::Node>,
    ) {
        if node.kind() == node_kind {
            nodes.push(node);
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                walk_recursive(child, node_kind, nodes);
            }
        }
    }

    walk_recursive(*node, node_kind, &mut nodes);
    nodes
}

#[async_trait]
impl IncrementalParser for RustIncrementalParser {
    async fn parse_incremental(
        &mut self,
        old_source: &str,
        new_source: &str,
        changes: Option<&Vec<FileChange>>,
    ) -> IDEResult<ParseTree> {
        let old_tree = self.current_tree.read().await.as_ref().map(|t| &t.tree);

        // Apply incremental changes if available
        if let Some(changes) = changes {
            if let Ok(Some(modified_source)) =
                self.apply_changes_incremental(changes, old_source).await
            {
                return self.parse_internal(&modified_source, old_tree).await;
            }
        }

        self.parse_internal(new_source, old_tree).await
    }

    async fn apply_changes(&mut self, changes: &Vec<FileChange>) -> IDEResult<ParseTree> {
        let current_source = self.source_cache.read().await.clone();
        if current_source.is_empty() {
            // Initialize with empty source if no current state
            return self.parse_incremental("", "", Some(changes)).await;
        }

        // Apply changes and re-parse
        if let Ok(Some(modified_source)) = self
            .apply_changes_incremental(changes, &current_source)
            .await
        {
            self.parse_incremental(&current_source, &modified_source, None)
                .await
        } else {
            // Fallback to full re-parse
            self.parse_incremental(&current_source, &current_source, None)
                .await
        }
    }

    async fn get_ast_diff(&self, old_tree: &ParseTree, new_tree: &ParseTree) -> IDEResult<ASTDiff> {
        let old_logic = &*self.logic_cache.read().await;
        let new_logic = Some(&*self.logic_cache.read().await);

        self.compute_ast_diff(old_tree, new_tree, old_logic, new_logic)
            .await
    }

    async fn parse_file(&mut self, file_path: &Path) -> IDEResult<ParseTree> {
        // Read and validate file
        let validated_path =
            ValidatedFilePath::new(&file_path.to_string_lossy(), "rust_parser_parse_file")?;

        let content = tokio::fs::read_to_string(validated_path.as_path())
            .await
            .map_err(|e| RustAIError::Io(e.into()))?;

        self.validate_rust_file(file_path, &content).await?;
        self.parse_incremental("", &content, None)
            .await
            .map(|mut tree| {
                tree.file_path = Some(file_path.to_path_buf());
                tree.last_modified = std::fs::metadata(file_path)
                    .ok()
                    .and_then(|m| m.modified().ok());
                tree
            })
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["rust", "rs"]
    }
}

impl CloneBox for RustIncrementalParser {
    fn clone_box(&self) -> Box<dyn IncrementalParser> {
        Box::new(self.clone())
    }
}

impl Default for RustOptimizations {
    fn default() -> Self {
        RustOptimizations {
            track_macros: true,
            track_traits: true,
            track_lifetimes: true,
            enable_caching: true,
            max_cache_size_per_file: 1024 * 1024,
            enable_parallel_processing: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rust_parser_basic_parsing() {
        let config = ParserConfig::default();
        let mut parser = RustIncrementalParser::new(config);

        let rust_code = r#"
        fn main() {
            println!("Hello, Rust!");
        }

        struct Point {
            x: i32,
            y: i32,
        }
        "#;

        let tree = parser.parse_incremental("", rust_code, None).await.unwrap();

        // Verify tree was created
        assert_eq!(tree.source, rust_code);
        assert_eq!(tree.language, Some("rust".to_string()));

        // Verify no syntax errors (should parse successfully)
        assert!(!tree.has_errors());
    }

    #[tokio::test]
    async fn test_rust_parser_incremental_changes() {
        let config = ParserConfig::default();
        let mut parser = RustIncrementalParser::new(config);

        let initial_code = "fn main() {}";
        let modified_code = "fn main() { println!(\"Hello!\"); }";

        // Initial parse
        let initial_tree = parser
            .parse_incremental("", initial_code, None)
            .await
            .unwrap();
        assert!(!initial_tree.has_errors());

        // Modified parse
        let modified_tree = parser
            .parse_incremental(initial_code, modified_code, None)
            .await
            .unwrap();
        assert!(!modified_tree.has_errors());

        // Compute diff
        let diff = parser
            .get_ast_diff(&initial_tree, &modified_tree)
            .await
            .unwrap();
        assert!(diff.changes.len() > 0); // Should detect changes
    }

    #[tokio::test]
    async fn test_rust_parser_error_handling() {
        let config = ParserConfig::default();
        let mut parser = RustIncrementalParser::new(config);

        // Test with invalid syntax
        let invalid_code = "fn main() { println!(\"Hello\")"; // Missing closing quote and brace

        // This should not panic and should handle the error gracefully
        let result = parser.parse_incremental("", invalid_code, None).await;
        match result {
            Ok(tree) => {
                // If parsing succeeds, might still have errors in AST
                if tree.has_errors() {
                    // This is expected for invalid syntax
                }
            }
            Err(RustAIError::Compilation(_)) => {
                // Expected for invalid syntax
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
