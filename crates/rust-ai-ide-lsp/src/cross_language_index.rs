use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use rust_ai_ide_common::{IDEError, IDEErrorKind};
use tree_sitter::{Parser, Tree, Node};
use CrossLanguageSymbol::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported programming languages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    Cpp,
    C,
}

/// Symbol type across languages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossLanguageSymbol {
    Function {
        name: String,
        return_type: Option<String>,
        parameters: Vec<Parameter>,
        doc: Option<String>,
    },
    Struct {
        name: String,
        fields: Vec<Field>,
        doc: Option<String>,
    },
    Class {
        name: String,
        methods: Vec<CrossLanguageSymbol>,
        fields: Vec<Field>,
        doc: Option<String>,
    },
    Variable {
        name: String,
        type_info: String,
        doc: Option<String>,
    },
    Module {
        name: String,
        symbols: Vec<CrossLanguageSymbol>,
        doc: Option<String>,
    },
    Trait {
        name: String,
        methods: Vec<CrossLanguageSymbol>,
        doc: Option<String>,
    },
    Interface {
        name: String,
        methods: Vec<CrossLanguageSymbol>,
        doc: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_info: String,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub type_info: String,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}

/// Global symbol table entry
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub symbol: CrossLanguageSymbol,
    pub location: SymbolLocation,
    pub references: HashSet<SymbolLocation>,
    pub language: SupportedLanguage,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SymbolLocation {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
}

/// Unified cross-language indexer
pub struct CrossLanguageIndexer {
    pub(crate) symbols: Arc<RwLock<HashMap<String, SymbolEntry>>>,
    pub(crate) symbol_graph: Arc<Mutex<SymbolGraph>>,
    pub(crate) parsers: Arc<RwLock<HashMap<SupportedLanguage, Parser>>>,
    pub(crate) language_router: Arc<LanguageRouter>,
}

impl CrossLanguageIndexer {
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            symbol_graph: Arc::new(Mutex::new(SymbolGraph::new())),
            parsers: Arc::new(RwLock::new(HashMap::new())),
            language_router: Arc::new(LanguageRouter::new()),
        }
    }

    pub async fn index_file(&self, file_path: &str, content: &[u8]) -> Result<String, IDEError> {
        let language = self.detect_language(file_path)?;
        let mut parsers = self.parsers.write().await;

        let parser = if let Some(parser) = parsers.get_mut(&language) {
            parser
        } else {
            let mut new_parser = Parser::new();
            self.configure_parser(&mut new_parser, &language)?;
            parsers.insert(language.clone(), new_parser);
            parsers.get_mut(&language).unwrap()
        };

        let tree = parser.parse(content, None).ok_or_else(|| {
            IDEError::new(
                IDEErrorKind::ParseError,
                format!("Failed to parse file: {}", file_path),
            )
        })?;

        // Extract symbols based on language
        let file_symbols = self.extract_symbols(content, &tree, &language)?;

        // Index symbols
        self.insert_symbols(&file_symbols, file_path, language).await?;

        Ok(file_path.to_string())
    }

    fn detect_language(&self, file_path: &str) -> Result<SupportedLanguage, IDEError> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "rs" => Ok(SupportedLanguage::Rust),
            "ts" => Ok(SupportedLanguage::TypeScript),
            "tsx" => Ok(SupportedLanguage::TypeScript),
            "js" => Ok(SupportedLanguage::JavaScript),
            "jsx" => Ok(SupportedLanguage::JavaScript),
            "py" => Ok(SupportedLanguage::Python),
            "go" => Ok(SupportedLanguage::Go),
            "java" => Ok(SupportedLanguage::Java),
            "cpp" | "cc" | "cxx" => Ok(SupportedLanguage::Cpp),
            "c" => Ok(SupportedLanguage::C),
            _ => Err(IDEError::new(
                IDEErrorKind::Unsupported,
                format!("Unsupported file extension: {}", extension),
            )),
        }
    }

    fn configure_parser(&self, parser: &mut Parser, language: &SupportedLanguage) -> Result<(), IDEError> {
        match language {
            SupportedLanguage::Rust => {
                let language = tree_sitter_rust::language();
                parser.set_language(language).map_err(|_| {
                    IDEError::new(IDEErrorKind::OperationUnsupported, "Failed to set Rust language")
                })?;
            },
            SupportedLanguage::TypeScript => {
                let language = tree_sitter_typescript::language_typescript();
                parser.set_language(language).map_err(|_| {
                    IDEError::new(IDEErrorKind::OperationUnsupported, "Failed to set TypeScript language")
                })?;
            },
            SupportedLanguage::JavaScript => {
                let language = tree_sitter_javascript::language();
                parser.set_language(language).map_err(|_| {
                    IDEError::new(IDEErrorKind::OperationUnsupported, "Failed to set JavaScript language")
                })?;
            },
            SupportedLanguage::Python => {
                let language = tree_sitter_python::language();
                parser.set_language(language).map_err(|_| {
                    IDEError::new(IDEErrorKind::OperationUnsupported, "Failed to set Python language")
                })?;
            },
            SupportedLanguage::Go => {
                let language = tree_sitter_go::language();
                parser.set_language(language).map_err(|_| {
                    IDEError::new(IDEErrorKind::OperationUnsupported, "Failed to set Go language")
                })?;
            },
            SupportedLanguage::Java => {
                let language = tree_sitter_java::language();
                parser.set_language(language).map_err(|_| {
                    IDEError::new(IDEErrorKind::OperationUnsupported, "Failed to set Java language")
                })?;
            },
            SupportedLanguage::Cpp => {
                let language = tree_sitter_cpp::language();
                parser.set_language(language).map_err(|_| {
                    IDEError::new(IDEErrorKind::OperationUnsupported, "Failed to set C++ language")
                })?;
            },
            SupportedLanguage::C => {
                let language = tree_sitter_c::language();
                parser.set_language(language).map_err(|_| {
                    IDEError::new(IDEErrorKind::OperationUnsupported, "Failed to set C language")
                })?;
            },
        }
        Ok(())
    }

    fn extract_symbols(&self, content: &[u8], tree: &Tree, language: &SupportedLanguage) -> Result<Vec<CrossLanguageSymbol>, IDEError> {
        let mut symbols = Vec::new();
        let root_node = tree.root_node();

        match language {
            SupportedLanguage::Rust => {
                self.extract_rust_symbols(content, root_node, &mut symbols)?;
            },
            SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                self.extract_typescript_symbols(content, root_node, &mut symbols)?;
            },
            SupportedLanguage::Python => {
                self.extract_python_symbols(content, root_node, &mut symbols)?;
            },
            SupportedLanguage::Go => {
                self.extract_go_symbols(content, root_node, &mut symbols)?;
            },
            SupportedLanguage::Java => {
                self.extract_java_symbols(content, root_node, &mut symbols)?;
            },
            SupportedLanguage::Cpp | SupportedLanguage::C => {
                self.extract_cpp_symbols(content, root_node, &mut symbols)?;
            },
        }

        Ok(symbols)
    }

    fn extract_rust_symbols(&self, content: &[u8], node: Node, symbols: &mut Vec<CrossLanguageSymbol>) -> Result<(), IDEError> {
        match node.kind() {
            "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    let function = CrossLanguageSymbol::Function {
                        name,
                        return_type: None,
                        parameters: vec![],
                        doc: None,
                    };
                    symbols.push(function);
                }
            },
            "struct_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    let struct_symbol = CrossLanguageSymbol::Struct {
                        name,
                        fields: vec![],
                        doc: None,
                    };
                    symbols.push(struct_symbol);
                }
            },
            "impl_item" => {
                // Handle implementations
            },
            _ => {},
        }

        // Recurse on children
        for child in node.children(&mut node.walk()) {
            self.extract_rust_symbols(content, child, symbols)?;
        }

        Ok(())
    }

    fn extract_typescript_symbols(&self, content: &[u8], node: Node, symbols: &mut Vec<CrossLanguageSymbol>) -> Result<(), IDEError> {
        match node.kind() {
            "function_declaration" | "method_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Function {
                        name,
                        return_type: None,
                        parameters: vec![],
                        doc: None,
                    });
                }
            },
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Class {
                        name,
                        methods: vec![],
                        fields: vec![],
                        doc: None,
                    });
                }
            },
            "interface_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Interface {
                        name,
                        methods: vec![],
                        doc: None,
                    });
                }
            },
            "variable_declaration" => {
                for child in node.children(&mut node.walk()) {
                    if child.kind() == "variable_declarator" {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = self.node_content(content, name_node)?;
                            symbols.push(CrossLanguageSymbol::Variable {
                                name,
                                type_info: "any".to_string(),
                                doc: None,
                            });
                        }
                    }
                }
            },
            _ => {},
        }

        // Recurse on children
        for child in node.children(&mut node.walk()) {
            self.extract_typescript_symbols(content, child, symbols)?;
        }

        Ok(())
    }

    fn extract_python_symbols(&self, content: &[u8], node: Node, symbols: &mut Vec<CrossLanguageSymbol>) -> Result<(), IDEError> {
        match node.kind() {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Function {
                        name,
                        return_type: None,
                        parameters: vec![],
                        doc: None,
                    });
                }
            },
            "class_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Class {
                        name,
                        methods: vec![],
                        fields: vec![],
                        doc: None,
                    });
                }
            },
            _ => {},
        }

        // Recurse on children
        for child in node.children(&mut node.walk()) {
            self.extract_python_symbols(content, child, symbols)?;
        }

        Ok(())
    }

    fn extract_go_symbols(&self, content: &[u8], node: Node, symbols: &mut Vec<CrossLanguageSymbol>) -> Result<(), IDEError> {
        match node.kind() {
            "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Function {
                        name,
                        return_type: None,
                        parameters: vec![],
                        doc: None,
                    });
                }
            },
            "type_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Struct {
                        name,
                        fields: vec![],
                        doc: None,
                    });
                }
            },
            _ => {},
        }

        // Recurse on children
        for child in node.children(&mut node.walk()) {
            self.extract_go_symbols(content, child, symbols)?;
        }

        Ok(())
    }

    fn extract_java_symbols(&self, content: &[u8], node: Node, symbols: &mut Vec<CrossLanguageSymbol>) -> Result<(), IDEError> {
        match node.kind() {
            "method_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Function {
                        name,
                        return_type: None,
                        parameters: vec![],
                        doc: None,
                    });
                }
            },
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Class {
                        name,
                        methods: vec![],
                        fields: vec![],
                        doc: None,
                    });
                }
            },
            "interface_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Interface {
                        name,
                        methods: vec![],
                        doc: None,
                    });
                }
            },
            _ => {},
        }

        // Recurse on children
        for child in node.children(&mut node.walk()) {
            self.extract_java_symbols(content, child, symbols)?;
        }

        Ok(())
    }

    fn extract_cpp_symbols(&self, content: &[u8], node: Node, symbols: &mut Vec<CrossLanguageSymbol>) -> Result<(), IDEError> {
        match node.kind() {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("declarator") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Function {
                        name,
                        return_type: None,
                        parameters: vec![],
                        doc: None,
                    });
                }
            },
            "class_specifier" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Class {
                        name,
                        methods: vec![],
                        fields: vec![],
                        doc: None,
                    });
                }
            },
            "struct_specifier" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_content(content, name_node)?;
                    symbols.push(CrossLanguageSymbol::Struct {
                        name,
                        fields: vec![],
                        doc: None,
                    });
                }
            },
            _ => {},
        }

        // Recurse on children
        for child in node.children(&mut node.walk()) {
            self.extract_cpp_symbols(content, child, symbols)?;
        }

        Ok(())
    }

    fn node_content(&self, content: &[u8], node: Node) -> Result<String, IDEError> {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();

        if start_byte > content.len() || end_byte > content.len() {
            return Err(IDEError::new(
                IDEErrorKind::ParseError,
                "Node range exceeds content length",
            ));
        }

        String::from_utf8(content[start_byte..end_byte].to_vec()).map_err(|e| {
            IDEError::new(IDEErrorKind::ParseError, "Failed to convert bytes to string")
                .with_source(e)
        })
    }

    async fn insert_symbols(&self, symbols: &[CrossLanguageSymbol], file_path: &str, language: SupportedLanguage) -> Result<(), IDEError> {
        let mut symbol_map = self.symbols.write().await;
        let mut graph = self.symbol_graph.lock().await;

        for symbol in symbols {
            let key = format!("{}::{}", language_symbol_prefix(&language), symbol_name(symbol));

            let entry = SymbolEntry {
                symbol: symbol.clone(),
                location: SymbolLocation {
                    file_path: file_path.to_string(),
                    line: 1, // Placeholder - would extract from AST
                    column: 1, // Placeholder
                },
                references: HashSet::new(),
                language: language.clone(),
            };

            symbol_map.insert(key.clone(), entry);

            // Add to symbol graph
            graph.add_symbol(&key, symbol.clone());
        }

        Ok(())
    }

    pub async fn find_symbol(&self, name: &str) -> Option<SymbolEntry> {
        let symbols = self.symbols.read().await;
        symbols.get(name).cloned()
    }

    pub async fn find_references(&self, symbol_key: &str) -> Vec<SymbolLocation> {
        let symbols = self.symbols.read().await;
        symbols.get(symbol_key)
            .map(|entry| entry.references.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn find_definitions(&self, name: &str) -> Vec<SymbolEntry> {
        let symbols = self.symbols.read().await;
        symbols.values()
            .filter(|entry| match &entry.symbol {
                Function { name: fn_name, .. } => fn_name.contains(name),
                Struct { name: struct_name, .. } => struct_name.contains(name),
                Class { name: class_name, .. } => class_name.contains(name),
                Variable { name: var_name, .. } => var_name.contains(name),
                Module { name: mod_name, .. } => mod_name.contains(name),
                _ => false,
            })
            .cloned()
            .collect()
    }

    pub async fn cross_language_navigation(&self, from_symbol: &str, target_symbol: &str) -> Result<Option<SymbolEntry>, IDEError> {
        let graph = self.symbol_graph.lock().await;
        graph.navigate(from_symbol, target_symbol)
    }
}

fn language_symbol_prefix(language: &SupportedLanguage) -> &'static str {
    match language {
        SupportedLanguage::Rust => "rust",
        SupportedLanguage::TypeScript => "ts",
        SupportedLanguage::JavaScript => "js",
        SupportedLanguage::Python => "py",
        SupportedLanguage::Go => "go",
        SupportedLanguage::Java => "java",
        SupportedLanguage::Cpp => "cpp",
        SupportedLanguage::C => "c",
    }
}

fn symbol_name(symbol: &CrossLanguageSymbol) -> String {
    match symbol {
        Function { name, .. } => name.clone(),
        Struct { name, .. } => name.clone(),
        Class { name, .. } => name.clone(),
        Variable { name, .. } => name.clone(),
        Module { name, .. } => name.clone(),
        Trait { name, .. } => name.clone(),
        Interface { name, .. } => name.clone(),
    }
}

/// Symbol dependency graph for cross-language navigation
pub struct SymbolGraph {
    pub(crate) nodes: HashMap<String, CrossLanguageSymbol>,
    pub(crate) dependencies: HashMap<String, HashSet<String>>,
}

impl SymbolGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, key: &str, symbol: CrossLanguageSymbol) {
        self.nodes.insert(key.to_string(), symbol);
        self.dependencies.insert(key.to_string(), HashSet::new());
    }

    pub fn add_dependency(&mut self, from: &str, to: &str) {
        if let Some(deps) = self.dependencies.get_mut(from) {
            deps.insert(to.to_string());
        }
    }

    pub fn navigate(&self, from_symbol: &str, target_symbol: &str) -> Option<SymbolEntry> {
        // Placeholder implementation - would implement actual navigation logic
        // This could include BFS/DFS for finding symbol relationships
        if self.nodes.contains_key(from_symbol) && self.nodes.contains_key(target_symbol) {
            Some(SymbolEntry {
                symbol: CrossLanguageSymbol::Module {
                    name: target_symbol.to_string(),
                    symbols: vec![],
                    doc: None,
                },
                location: SymbolLocation {
                    file_path: "navigated".to_string(),
                    line: 1,
                    column: 1,
                },
                references: HashSet::new(),
                language: SupportedLanguage::Rust, // Placeholder
            })
        } else {
            None
        }
    }
}

/// Language request router
pub struct LanguageRouter {
    pub(crate) routes: Mutex<HashMap<String, String>>,
}

impl LanguageRouter {
    pub fn new() -> Self {
        Self {
            routes: Mutex::new(HashMap::new()),
        }
    }

    pub async fn register_language_server(&self, language: &str, endpoint: &str) {
        let mut routes = self.routes.lock().await;
        routes.insert(language.to_string(), endpoint.to_string());
    }

    pub async fn route_request(&self, file_path: &str, request: &str) -> Result<String, IDEError> {
        // Parse file extension to determine language
        let language = if file_path.ends_with(".rs") {
            "rust"
        } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
            "typescript"
        } else {
            "generic"
        };

        let routes = self.routes.lock().await;
        routes.get(language)
            .cloned()
            .ok_or_else(|| {
                IDEError::new(
                    IDEErrorKind::Unsupported,
                    format!("No language server route found for: {}", language),
                )
            })
    }
}

impl fmt::Debug for CrossLanguageIndexer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrossLanguageIndexer")
            .field("symbol_count", &self.symbols.blocking_read().len())
            .field("language_router", &"LanguageRouter")
            .finish()
    }
}

impl fmt::Debug for SymbolGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolGraph")
            .field("node_count", &self.nodes.len())
            .field("dependency_edges", &self.dependencies.len())
            .finish()
    }
}