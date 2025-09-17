//! Semantic Analyzer Module
//! Provides deep semantic analysis of code for understanding intent, relationships, and potential
//! improvements.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use syn::visit::Visit;

/// Definition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDefinition {
    pub name: String,
    pub kind: SymbolKind,
    pub location: CodeLocation,
    pub visibility: Visibility,
    pub documentation: Option<String>,
}

/// Usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolUsage {
    pub definition: SymbolIndex,
    pub location: CodeLocation,
    pub usage_type: UsageType,
}

/// Semantic relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelationship {
    pub from_symbol: SymbolIndex,
    pub to_symbol: SymbolIndex,
    pub relationship_type: RelationshipType,
    pub strength: f32,
}

/// Symbol kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Type,
    Variable,
    Const,
    Macro,
    Field,
    Method,
}

/// Usage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageType {
    Call,
    Reference,
    Assignment,
    Declaration,
    Import,
}

/// Relationship types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Calls,
    Implements,
    Inherits,
    Uses,
    Contains,
    DependsOn,
}

/// Code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
}

/// Symbol index for fast lookups
pub type SymbolIndex = u32;

/// Visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Super,
}

/// Semantic context from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    pub definitions: Vec<SymbolDefinition>,
    pub usages: Vec<SymbolUsage>,
    pub relationships: Vec<SemanticRelationship>,
    pub code_smells: Vec<CodeSmell>,
    pub complexity_metrics: ComplexityMetrics,
    pub analyzed_files: HashSet<String>,
}

/// Code smell detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmell {
    pub smell_type: String,
    pub location: CodeLocation,
    pub severity: f32,
    pub description: String,
    pub suggestions: Vec<String>,
}

/// Complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: HashMap<String, u32>,
    pub lines_of_code: HashMap<String, u32>,
    pub function_count: u32,
    pub type_count: u32,
    pub average_function_complexity: f32,
}

/// Main semantic analyzer
#[derive(Debug)]
pub struct SemanticAnalyzer {
    config: SemanticConfig,
    context_cache: HashMap<String, SemanticContext>,
}

#[derive(Debug, Clone)]
pub struct SemanticConfig {
    pub max_function_complexity: u32,
    pub enable_code_smell_detection: bool,
    pub enable_relationship_analysis: bool,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new(config: &SemanticConfig) -> Self {
        Self {
            config: config.clone(),
            context_cache: HashMap::new(),
        }
    }

    /// Analyze source code for semantic understanding
    pub async fn analyze(
        &mut self,
        source_code: &str,
        language: &str,
    ) -> Result<SemanticContext, String> {
        if language != "rust" {
            return Err(format!("Unsupported language: {}", language));
        }

        // Parse the source code into AST
        let ast = syn::parse_file(source_code).map_err(|e| format!("Parse error: {}", e))?;

        // Perform semantic analysis
        let mut context = SemanticContext {
            definitions: vec![],
            usages: vec![],
            relationships: vec![],
            code_smells: vec![],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: HashMap::new(),
                lines_of_code: HashMap::new(),
                function_count: 0,
                type_count: 0,
                average_function_complexity: 0.0,
            },
            analyzed_files: HashSet::new(),
        };

        // Walk the AST and extract semantic information
        let mut visitor = SemanticVisitor::new(&mut context);
        visitor.visit_file(&ast);

        // Calculate complexity metrics
        self.calculate_complexity_metrics(&mut context, source_code);

        // Detect code smells
        if self.config.enable_code_smell_detection {
            self.detect_code_smells(&mut context);
        }

        // Analyze relationships
        if self.config.enable_relationship_analysis {
            self.analyze_relationships(&mut context);
        }

        Ok(context)
    }

    /// Find definitions by name
    pub fn find_definition<'a>(
        &self,
        name: &str,
        context: &'a SemanticContext,
    ) -> Option<&'a SymbolDefinition> {
        context.definitions.iter().find(|def| def.name == name)
    }

    /// Find all usages of a symbol
    pub fn find_usages<'a>(
        &self,
        definition: &SymbolDefinition,
        context: &'a SemanticContext,
    ) -> Vec<&'a SymbolUsage> {
        context
            .usages
            .iter()
            .filter(|usage| usage.definition == definition.location.start_line as SymbolIndex) // simplified mapping
            .collect()
    }

    fn calculate_complexity_metrics(&self, context: &mut SemanticContext, source_code: &str) {
        let lines = source_code.lines().count();
        context
            .complexity_metrics
            .lines_of_code
            .insert("total".to_string(), lines as u32);

        // Calculate cyclomatic complexity for functions (simplified)
        for def in &context.definitions {
            if matches!(def.kind, SymbolKind::Function | SymbolKind::Method) {
                let complexity = self.calculate_cyclomatic_complexity(source_code, &def.location);
                context
                    .complexity_metrics
                    .cyclomatic_complexity
                    .insert(def.name.clone(), complexity);
            }
        }

        // Calculate averages
        let total_complexity: u32 = context
            .complexity_metrics
            .cyclomatic_complexity
            .values()
            .sum();
        let function_count = context.complexity_metrics.cyclomatic_complexity.len() as f32;
        context.complexity_metrics.average_function_complexity = if function_count > 0.0 {
            (total_complexity as f32) / function_count
        } else {
            0.0
        };

        context.complexity_metrics.function_count = context
            .definitions
            .iter()
            .filter(|def| matches!(def.kind, SymbolKind::Function | SymbolKind::Method))
            .count() as u32;

        context.complexity_metrics.type_count = context
            .definitions
            .iter()
            .filter(|def| {
                matches!(
                    def.kind,
                    SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Trait
                )
            })
            .count() as u32;
    }

    fn calculate_cyclomatic_complexity(&self, source_code: &str, location: &CodeLocation) -> u32 {
        // Simplified cyclomatic complexity calculation
        let lines: Vec<&str> = source_code.lines().collect();
        let mut complexity = 1; // base complexity

        // Look for control flow keywords
        for line_idx in (location.start_line as usize)..=(location.end_line as usize) {
            if let Some(line) = lines.get(line_idx.saturating_sub(1)) {
                let line_str = line.to_lowercase();
                if line_str.contains("if ")
                    || line_str.contains("else")
                    || line_str.contains("while ")
                    || line_str.contains("for ")
                    || line_str.contains("match ")
                    || line_str.contains("|| ")
                    || line_str.contains("&& ")
                {
                    complexity += 1;
                }
            }
        }

        complexity
    }

    fn detect_code_smells(&self, context: &mut SemanticContext) {
        // Detect long functions
        for def in &context.definitions {
            if matches!(def.kind, SymbolKind::Function | SymbolKind::Method) {
                let length = def.location.end_line - def.location.start_line;
                if length > 30 {
                    context.code_smells.push(CodeSmell {
                        smell_type: "Long Function".to_string(),
                        location: def.location.clone(),
                        severity: 0.7,
                        description: format!(
                            "Function '{}' is too long ({} lines)",
                            def.name, length
                        ),
                        suggestions: vec![
                            "Consider breaking down into smaller functions".to_string(),
                            "Extract duplicate code into separate functions".to_string(),
                        ],
                    });
                }
            }
        }

        // Detect high complexity
        for (name, complexity) in &context.complexity_metrics.cyclomatic_complexity {
            if *complexity > self.config.max_function_complexity {
                if let Some(def) = context.definitions.iter().find(|d| d.name == *name) {
                    context.code_smells.push(CodeSmell {
                        smell_type: "High Complexity".to_string(),
                        location: def.location.clone(),
                        severity: 0.8,
                        description: format!(
                            "Function '{}' has high cyclomatic complexity ({})",
                            name, complexity
                        ),
                        suggestions: vec![
                            "Simplify conditional logic".to_string(),
                            "Extract complex conditions into separate functions".to_string(),
                        ],
                    });
                }
            }
        }
    }

    fn analyze_relationships(&self, context: &mut SemanticContext) {
        // Analyze relationships between symbols (simplified)
        // This would build a call graph, dependency graph, etc.

        for i in 0..context.definitions.len() {
            for j in 0..context.definitions.len() {
                if i != j {
                    let rel_type = self.determine_relationship_type(
                        &context.definitions[i],
                        &context.definitions[j],
                    );

                    if rel_type.is_some() {
                        context.relationships.push(SemanticRelationship {
                            from_symbol: i as SymbolIndex,
                            to_symbol: j as SymbolIndex,
                            relationship_type: rel_type.unwrap(),
                            strength: 0.8,
                        });
                    }
                }
            }
        }
    }

    fn determine_relationship_type(
        &self,
        from: &SymbolDefinition,
        to: &SymbolDefinition,
    ) -> Option<RelationshipType> {
        // Simplified relationship determination
        match (&from.kind, &to.kind) {
            (SymbolKind::Function, SymbolKind::Function) => Some(RelationshipType::Calls),
            (SymbolKind::Struct, SymbolKind::Trait) => Some(RelationshipType::Implements),
            (SymbolKind::Struct, SymbolKind::Enum) => Some(RelationshipType::Contains),
            _ => None,
        }
    }
}

/// AST visitor for semantic analysis
pub struct SemanticVisitor<'a> {
    context: &'a mut SemanticContext,
    symbol_index: SymbolIndex,
}

impl<'a> SemanticVisitor<'a> {
    pub fn new(context: &'a mut SemanticContext) -> Self {
        Self {
            context,
            symbol_index: 0,
        }
    }

    fn add_definition(
        &mut self,
        name: String,
        kind: SymbolKind,
        location: CodeLocation,
        visibility: Visibility,
    ) {
        self.context.definitions.push(SymbolDefinition {
            name,
            kind,
            location,
            visibility,
            documentation: None,
        });
        self.symbol_index += 1;
    }
}

impl<'a> Visit<'_> for SemanticVisitor<'a> {
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        // Analyze function definitions
        let name = node.sig.ident.to_string();
        let kind = SymbolKind::Function;
        let visibility = if matches!(node.vis, syn::Visibility::Public(_)) {
            Visibility::Public
        } else {
            Visibility::Private
        };

        let location = CodeLocation {
            file: "current".to_string(), // Would get from file path
            start_line: 0,               // Would get from span
            end_line: 0,
            start_column: 0,
            end_column: 0,
        };

        self.add_definition(name, kind, location, visibility);
    }

    fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
        // Analyze struct definitions
        let name = node.ident.to_string();
        let location = CodeLocation {
            file: "current".to_string(),
            start_line: 0,
            end_line: 0,
            start_column: 0,
            end_column: 0,
        };

        self.add_definition(name, SymbolKind::Struct, location, Visibility::Private);
    }

    fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        // Analyze enum definitions
        let name = node.ident.to_string();
        let location = CodeLocation {
            file: "current".to_string(),
            start_line: 0,
            end_line: 0,
            start_column: 0,
            end_column: 0,
        };

        self.add_definition(name, SymbolKind::Enum, location, Visibility::Private);
    }

    fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        // Analyze trait definitions
        let name = node.ident.to_string();
        let location = CodeLocation {
            file: "current".to_string(),
            start_line: 0,
            end_line: 0,
            start_column: 0,
            end_column: 0,
        };

        self.add_definition(name, SymbolKind::Trait, location, Visibility::Private);
    }
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            max_function_complexity: 10,
            enable_code_smell_detection: true,
            enable_relationship_analysis: true,
        }
    }
}
