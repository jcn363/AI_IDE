use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser as TreeSitterParser};

use crate::error_handling::{AnalysisError, AnalysisResult};

/// Multi-language AST parsing and abstraction layer
///
/// This module provides unified AST parsing for multiple programming languages
/// using appropriate parsers (tree-sitter for dynamic languages, syn for Rust).

/// Programming language types for AST parsing
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    Cpp,
    CSharp,
    HTML,
    CSS,
    Shell,
    SQL,
    Other(String),
}

/// Unified AST representation for cross-language analysis
#[derive(Debug, Clone)]
pub struct UnifiedAST {
    pub language:  Language,
    pub root_node: ASTNode,
    pub metadata:  ASTMetadata,
}

/// Metadata about the parsed AST
#[derive(Debug, Clone, Default)]
pub struct ASTMetadata {
    pub parse_errors:     Vec<ParseError>,
    pub language_version: Option<String>,
    pub compiler_flags:   Vec<String>,
    pub includes:         Vec<String>,
}

/// Parse error information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line:    usize,
    pub column:  usize,
    pub source:  String,
}

/// Unified AST node representation
#[derive(Debug, Clone)]
pub enum ASTNode {
    Document {
        children:     Vec<ASTNode>,
        source_range: SourceRange,
    },
    Function {
        name:         String,
        parameters:   Vec<Parameter>,
        return_type:  Option<String>,
        body:         Vec<ASTNode>,
        visibility:   Visibility,
        source_range: SourceRange,
    },
    Class {
        name:         String,
        extends:      Vec<String>,
        implements:   Vec<String>,
        methods:      Vec<ASTNode>,
        fields:       Vec<Field>,
        visibility:   Visibility,
        source_range: SourceRange,
    },
    Variable {
        name:         String,
        var_type:     Option<String>,
        value:        Option<String>,
        visibility:   Visibility,
        source_range: SourceRange,
    },
    Import {
        module:       String,
        symbols:      Vec<String>,
        alias:        Option<String>,
        source_range: SourceRange,
    },
    Statement {
        kind:         StatementKind,
        content:      Vec<ASTNode>,
        source_range: SourceRange,
    },
    Expression {
        kind:         ExpressionKind,
        operands:     Vec<ASTNode>,
        source_range: SourceRange,
    },
    Comment {
        content:      String,
        source_range: SourceRange,
    },
    Other {
        node_type:    String,
        properties:   HashMap<String, String>,
        source_range: SourceRange,
    },
}

/// Source range information
#[derive(Debug, Clone)]
pub struct SourceRange {
    pub start_line:   usize,
    pub start_column: usize,
    pub end_line:     usize,
    pub end_column:   usize,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name:          String,
    pub param_type:    Option<String>,
    pub default_value: Option<String>,
}

/// Class field
#[derive(Debug, Clone)]
pub struct Field {
    pub name:          String,
    pub field_type:    Option<String>,
    pub visibility:    Visibility,
    pub default_value: Option<String>,
}

/// Visibility modifier
#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Package,
    Unknown,
}

/// Statement types
#[derive(Debug, Clone)]
pub enum StatementKind {
    If,
    For,
    While,
    Return,
    Break,
    Continue,
    Try,
    Catch,
    Assignment,
    Declaration,
    Expression,
    Other(String),
}

/// Expression types
#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Call,
    BinaryOp,
    UnaryOp,
    Literal,
    Identifier,
    MemberAccess,
    ArrayAccess,
    Other(String),
}

/// Multi-language AST parser
pub struct MultiASTParser {
    parsers: HashMap<Language, TreeSitterParser>,
}

impl MultiASTParser {
    /// Create a new multi-language AST parser
    pub fn new() -> Self {
        let mut parsers = HashMap::new();

        // Initialize TypeScript parser
        let mut ts_parser = TreeSitterParser::new();
        let ts_language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        ts_parser.set_language(&ts_language).unwrap();
        parsers.insert(Language::TypeScript, ts_parser);

        // Initialize JavaScript parser (using TypeScript parser for JS as well)
        let mut js_parser = TreeSitterParser::new();
        js_parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .unwrap();
        parsers.insert(Language::JavaScript, js_parser);

        // Initialize Python parser
        let mut py_parser = TreeSitterParser::new();
        let py_language = tree_sitter_python::LANGUAGE.into();
        py_parser.set_language(&py_language).unwrap();
        parsers.insert(Language::Python, py_parser);

        // Initialize Go parser
        let mut go_parser = TreeSitterParser::new();
        let go_language = tree_sitter_go::LANGUAGE.into();
        go_parser.set_language(&go_language).unwrap();
        parsers.insert(Language::Go, go_parser);

        // Initialize Java parser
        let mut java_parser = TreeSitterParser::new();
        let java_language = tree_sitter_java::LANGUAGE.into();
        java_parser.set_language(&java_language).unwrap();
        parsers.insert(Language::Java, java_parser);

        // Initialize C++ parser
        let mut cpp_parser = TreeSitterParser::new();
        let cpp_language = tree_sitter_cpp::LANGUAGE.into();
        cpp_parser.set_language(&cpp_language).unwrap();
        parsers.insert(Language::Cpp, cpp_parser);

        Self { parsers }
    }

    /// Parse source code for a specific language
    pub fn parse(&mut self, language: &Language, content: &str) -> AnalysisResult<UnifiedAST> {
        match language {
            Language::Rust => self.parse_rust(content),
            lang if self.parsers.contains_key(lang) => self.parse_treesitter(lang, content),
            _ => Err(AnalysisError::UnsupportedLanguage(format!(
                "{:?}",
                language
            ))),
        }
    }

    /// Parse Rust code using syn
    fn parse_rust(&self, content: &str) -> AnalysisResult<UnifiedAST> {
        let ast = syn::parse_file(content).map_err(|e| AnalysisError::ParseError(e.to_string()))?;

        let root_node = self.convert_syn_to_unified(&ast, content);
        Ok(UnifiedAST {
            language: Language::Rust,
            root_node,
            metadata: ASTMetadata::default(),
        })
    }

    /// Parse code using tree-sitter
    fn parse_treesitter(&mut self, language: &Language, content: &str) -> AnalysisResult<UnifiedAST> {
        let parser = self
            .parsers
            .get_mut(language)
            .ok_or_else(|| AnalysisError::UnsupportedLanguage(format!("No parser for {:?}", language)))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| AnalysisError::ParseError("Tree-sitter parsing failed".to_string()))?;

        // Convert tree-sitter tree to unified AST
        let root_node = self.convert_treesitter_to_unified(tree.root_node(), content);

        Ok(UnifiedAST {
            language: language.clone(),
            root_node,
            metadata: ASTMetadata::default(),
        })
    }

    /// Convert syn AST to unified AST
    fn convert_syn_to_unified(&self, ast: &syn::File, content: &str) -> ASTNode {
        let children = ast
            .items
            .iter()
            .map(|item| {
                match item {
                    syn::Item::Fn(func) => ASTNode::Function {
                        name:         func.sig.ident.to_string(),
                        parameters:   func
                            .sig
                            .inputs
                            .iter()
                            .map(|param| match param {
                                syn::FnArg::Receiver(_) => Parameter {
                                    name:          "self".to_string(),
                                    param_type:    None,
                                    default_value: None,
                                },
                                syn::FnArg::Typed(pat_type) => {
                                    let name = match &*pat_type.pat {
                                        syn::Pat::Ident(ident) => ident.ident.to_string(),
                                        _ => "_".to_string(),
                                    };
                                    let param_type = Some(quote::quote!(#pat_type.ty).to_string());
                                    Parameter {
                                        name,
                                        param_type,
                                        default_value: None,
                                    }
                                }
                            })
                            .collect(),
                        return_type:  match &func.sig.output {
                            syn::ReturnType::Default => None,
                            syn::ReturnType::Type(_, ty) => Some(quote::quote!(#ty).to_string()),
                        },
                        body:         vec![],             // Would need deeper conversion
                        visibility:   Visibility::Public, // Assume public for now
                        source_range: SourceRange {
                            start_line:   0,
                            start_column: 0,
                            end_line:     0,
                            end_column:   0,
                        },
                    },
                    _ => ASTNode::Other {
                        node_type:    "item".to_string(),
                        properties:   HashMap::new(),
                        source_range: SourceRange {
                            start_line:   0,
                            start_column: 0,
                            end_line:     0,
                            end_column:   0,
                        },
                    },
                }
            })
            .collect();

        ASTNode::Document {
            children,
            source_range: SourceRange {
                start_line:   1,
                start_column: 0,
                end_line:     content.lines().count(),
                end_column:   0,
            },
        }
    }

    /// Convert tree-sitter node to unified AST
    fn convert_treesitter_to_unified(&self, node: Node, source: &str) -> ASTNode {
        let source_range = SourceRange {
            start_line:   node.start_position().row + 1,
            start_column: node.start_position().column,
            end_line:     node.end_position().row + 1,
            end_column:   node.end_position().column,
        };

        match node.kind() {
            "function_definition" | "function_declaration" => {
                let name = self
                    .find_child_text(&node, &["identifier", "name"], source)
                    .unwrap_or_default();
                let parameters = self.extract_parameters(&node, source);
                let return_type = self.find_child_text(&node, &["return_type", "type"], source);

                ASTNode::Function {
                    name,
                    parameters,
                    return_type,
                    body: node
                        .named_children(&mut node.walk())
                        .map(|child| self.convert_treesitter_to_unified(child, source))
                        .collect(),
                    visibility: Visibility::Public, // Default
                    source_range,
                }
            }
            "class_definition" | "class_declaration" => {
                let name = self
                    .find_child_text(&node, &["identifier", "name"], source)
                    .unwrap_or_default();

                ASTNode::Class {
                    name,
                    extends: vec![],    // Would need to extract inheritance
                    implements: vec![], // Would need to extract interfaces
                    methods: node
                        .named_children(&mut node.walk())
                        .filter_map(|child| {
                            if matches!(child.kind(), "function_definition" | "method_definition") {
                                Some(self.convert_treesitter_to_unified(child, source))
                            } else {
                                None
                            }
                        })
                        .collect(),
                    fields: node
                        .named_children(&mut node.walk())
                        .filter_map(|child| {
                            if matches!(child.kind(), "field_declaration" | "variable_declaration") {
                                Some(Field {
                                    name:          self
                                        .find_child_text(&child, &["identifier"], source)
                                        .unwrap_or_default(),
                                    field_type:    self.find_child_text(&child, &["type"], source),
                                    visibility:    Visibility::Public,
                                    default_value: None,
                                })
                            } else {
                                None
                            }
                        })
                        .collect(),
                    visibility: Visibility::Public,
                    source_range,
                }
            }
            "import_statement" | "import_declaration" => {
                let module = self.extract_import_module(&node, source);
                let symbols = self.extract_import_symbols(&node, source);

                ASTNode::Import {
                    module,
                    symbols,
                    alias: None,
                    source_range,
                }
            }
            "comment" => {
                let content = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                ASTNode::Comment {
                    content,
                    source_range,
                }
            }
            _ => {
                let mut properties = HashMap::new();
                properties.insert(
                    "text".to_string(),
                    node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                );

                ASTNode::Other {
                    node_type: node.kind().to_string(),
                    properties,
                    source_range,
                }
            }
        }
    }

    /// Helper method to find child node text
    fn find_child_text(&self, node: &Node, kinds: &[&str], source: &str) -> Option<String> {
        for child in node.named_children(&mut node.walk()) {
            if kinds.contains(&child.kind()) {
                return child
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
            // Recursively search in children
            if let Some(text) = self.find_child_text(&child, kinds, source) {
                return Some(text);
            }
        }
        None
    }

    /// Extract parameters from a function node
    fn extract_parameters(&self, node: &Node, source: &str) -> Vec<Parameter> {
        let mut parameters = vec![];

        if let Some(params_node) = node.named_child(1) {
            // Usually parameters are the second child
            for child in params_node.named_children(&mut params_node.walk()) {
                if matches!(child.kind(), "identifier" | "parameter") {
                    let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    parameters.push(Parameter {
                        name,
                        param_type: None, // Would need more complex logic
                        default_value: None,
                    });
                }
            }
        }

        parameters
    }

    /// Extract import module name
    fn extract_import_module(&self, node: &Node, source: &str) -> String {
        self.find_child_text(node, &["string_literal", "module_name"], source)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string()
    }

    /// Extract imported symbols
    fn extract_import_symbols(&self, node: &Node, source: &str) -> Vec<String> {
        let mut symbols = vec![];

        for child in node.named_children(&mut node.walk()) {
            if matches!(child.kind(), "identifier" | "import_specifier") {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    symbols.push(text.to_string());
                }
            }
        }

        symbols
    }
}

impl Default for MultiASTParser {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MultiASTParser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MultiASTParser {{ parsers: {} language parsers }}",
            self.parsers.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = MultiASTParser::new();
        assert!(parser.parsers.contains_key(&Language::TypeScript));
        assert!(parser.parsers.contains_key(&Language::Python));
        assert!(parser.parsers.contains_key(&Language::Go));
    }

    #[test]
    fn test_parse_typescript_function() {
        let mut parser = MultiASTParser::new();
        let code = "function hello(name: string): void { console.log('Hello ' + name); }";

        let result = parser.parse(&Language::TypeScript, code);
        assert!(result.is_ok());

        if let Ok(ast) = result {
            if let ASTNode::Document { children, .. } = &ast.root_node {
                assert!(!children.is_empty());
            }
        }
    }
}
