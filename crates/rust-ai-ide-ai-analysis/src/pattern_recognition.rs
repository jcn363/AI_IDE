//! Language-Aware Pattern Recognition Module
//!
//! This module provides intelligent pattern recognition across multiple programming languages
//! to identify common programming patterns, anti-patterns, and architectural smells.

use std::collections::HashMap;

use rust_ai_ide_common::{IdeError, IdeResult};
use serde::{Deserialize, Serialize};

use crate::analysis::types::Severity;
use crate::multi_ast::{ASTNode, Language, MultiASTParser, UnifiedAST};

/// Pattern recognition engine for multi-language analysis
#[derive(Debug)]
pub struct LanguageAwarePatternRecognizer {
    /// AST parser for multiple languages
    ast_parser: MultiASTParser,
    /// Registered patterns for each language
    patterns: HashMap<Language, Vec<PatternDefinition>>,
    /// Pattern confidence thresholds
    thresholds: PatternThresholds,
}

/// Configuration for pattern recognition thresholds
#[derive(Debug, Clone)]
pub struct PatternThresholds {
    pub min_confidence: f64,
    pub high_confidence: f64,
    pub critical_confidence: f64,
}

impl Default for PatternThresholds {
    fn default() -> Self {
        Self {
            min_confidence: 0.3,
            high_confidence: 0.7,
            critical_confidence: 0.9,
        }
    }
}

/// Pattern definition for recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PatternCategory,
    pub severity: Severity,
    pub ast_pattern: ASTPattern,
    pub confidence_rule: ConfidenceRule,
}

/// Pattern categories for classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternCategory {
    CodeSmell,
    SecurityVulnerability,
    PerformanceIssue,
    MaintainabilityProblem,
    GoodPractice,
    AntiPattern,
}

/// Abstract syntax tree pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTPattern {
    pub node_types: Vec<String>,
    pub child_patterns: Vec<ChildPattern>,
    pub properties: HashMap<String, String>,
}

/// Child pattern specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildPattern {
    pub relation: RelationType,
    pub pattern: Box<ASTPattern>,
}

/// Relationship types between AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    DirectChild,
    Descendant,
    Sibling,
    Ancestor,
}

/// Confidence calculation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceRule {
    pub base_score: f64,
    pub multipliers: Vec<ConfidenceMultiplier>,
}

/// Confidence multipliers based on context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceMultiplier {
    pub condition: String,
    pub multiplier: f64,
}

/// Recognized pattern result
#[derive(Debug, Clone)]
pub struct RecognizedPattern {
    pub pattern: PatternDefinition,
    pub location: String,
    pub confidence: f64,
    pub context: HashMap<String, String>,
    pub suggestions: Vec<String>,
}

impl Default for LanguageAwarePatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageAwarePatternRecognizer {
    /// Create a new language-aware pattern recognizer
    pub fn new() -> Self {
        let mut recognizer = Self {
            ast_parser: MultiASTParser::new(),
            patterns: HashMap::new(),
            thresholds: Default::default(),
        };

        recognizer.initialize_standard_patterns();
        recognizer
    }

    /// Analyze code across multiple languages
    pub fn analyze_code(
        &mut self,
        content: &str,
        language: &Language,
        file_path: &str,
    ) -> IdeResult<Vec<RecognizedPattern>> {
        // Parse the code into unified AST
        let ast = self.ast_parser.parse(language, content).map_err(|e| {
            eprintln!("Failed to parse code: {}", e);
            IdeError::Analysis {
                message: format!("Parse error: {}", e),
                file_path: file_path.to_string(),
            }
        })?;

        // Get patterns for this language
        let language_patterns = self.patterns.get(language).cloned().unwrap_or_default();

        // Find patterns in the AST
        let mut recognized_patterns = Vec::new();

        for pattern in language_patterns {
            if let Some(matches) = self.find_pattern_matches(&ast, &pattern) {
                for match_context in matches {
                    let confidence =
                        self.calculate_confidence(&pattern.confidence_rule, &match_context);

                    if confidence >= self.thresholds.min_confidence {
                        let recognized = RecognizedPattern {
                            pattern: pattern.clone(),
                            location: format!(
                                "{}:{}",
                                file_path,
                                match_context
                                    .get("location")
                                    .unwrap_or(&"unknown".to_string())
                            ),
                            confidence,
                            context: match_context,
                            suggestions: self.generate_suggestions(&pattern),
                        };

                        recognized_patterns.push(recognized);
                    }
                }
            }
        }

        Ok(recognized_patterns)
    }

    /// Find pattern matches in AST
    fn find_pattern_matches(
        &self,
        ast: &UnifiedAST,
        pattern: &PatternDefinition,
    ) -> Option<Vec<HashMap<String, String>>> {
        let mut matches = Vec::new();

        // Walk the AST and look for pattern matches
        self.walk_ast_for_pattern(
            &ast.root_node,
            &pattern.ast_pattern,
            &mut matches,
            &mut HashMap::new(),
            0,
        );

        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }

    /// Walk AST looking for pattern matches
    fn walk_ast_for_pattern(
        &self,
        node: &ASTNode,
        pattern: &ASTPattern,
        matches: &mut Vec<HashMap<String, String>>,
        context: &mut HashMap<String, String>,
        depth: usize,
    ) {
        // Check if current node matches the pattern
        if self.node_matches_pattern(node, pattern) {
            let mut match_context = context.clone();

            // Add node-specific context
            match_context.insert("location".to_string(), format!("{}", depth));
            match_context.insert("node_type".to_string(), self.node_type_name(node));

            // Check if this is a complete match
            if self.is_complete_pattern_match(node, pattern) {
                matches.push(match_context);
            }
        }

        // Continue walking children
        self.walk_children_for_pattern(node, pattern, matches, context, depth + 1);
    }

    /// Walk specific children for pattern matching
    fn walk_children_for_pattern(
        &self,
        node: &ASTNode,
        pattern: &ASTPattern,
        matches: &mut Vec<HashMap<String, String>>,
        context: &mut HashMap<String, String>,
        depth: usize,
    ) {
        match node {
            ASTNode::Document { children, .. }
            | ASTNode::Statement {
                content: children, ..
            }
            | ASTNode::Expression {
                operands: children, ..
            } => {
                for child in children {
                    self.walk_ast_for_pattern(child, pattern, matches, context, depth);
                }
            }
            ASTNode::Function { body, .. } => {
                for child in body {
                    self.walk_ast_for_pattern(child, pattern, matches, context, depth);
                }
            }
            ASTNode::Class { methods, .. } => {
                for method in methods {
                    self.walk_ast_for_pattern(method, pattern, matches, context, depth);
                }
            }
            _ => {
                // Other node types don't have children to walk
            }
        }
    }

    /// Check if node matches pattern
    fn node_matches_pattern(&self, node: &ASTNode, pattern: &ASTPattern) -> bool {
        // Check node type
        let node_type = self.node_type_name(node);
        if !pattern.node_types.is_empty() && !pattern.node_types.contains(&node_type) {
            return false;
        }

        // Check properties
        for (key, expected_value) in &pattern.properties {
            match self.get_node_property(node, key) {
                Some(actual_value) if actual_value != *expected_value => return false,
                None => return false,
                _ => continue,
            }
        }

        true
    }

    /// Check if this is a complete pattern match
    fn is_complete_pattern_match(&self, _node: &ASTNode, _pattern: &ASTPattern) -> bool {
        // This is a simple implementation - in practice, this would be more sophisticated
        // to check all child patterns, relations, etc.
        true
    }

    /// Get human-readable node type name
    fn node_type_name(&self, node: &ASTNode) -> String {
        match node {
            ASTNode::Document { .. } => "document".to_string(),
            ASTNode::Function { .. } => "function".to_string(),
            ASTNode::Class { .. } => "class".to_string(),
            ASTNode::Variable { .. } => "variable".to_string(),
            ASTNode::Import { .. } => "import".to_string(),
            ASTNode::Statement { .. } => "statement".to_string(),
            ASTNode::Expression { .. } => "expression".to_string(),
            ASTNode::Comment { .. } => "comment".to_string(),
            ASTNode::Other { node_type, .. } => node_type.clone(),
        }
    }

    /// Get node property for matching
    fn get_node_property(&self, node: &ASTNode, property: &str) -> Option<String> {
        match (node, property) {
            (ASTNode::Function { name, .. }, "name") => Some(name.clone()),
            (ASTNode::Class { name, .. }, "name") => Some(name.clone()),
            (ASTNode::Variable { name, .. }, "name") => Some(name.clone()),
            _ => None,
        }
    }

    /// Calculate pattern confidence
    fn calculate_confidence(
        &self,
        rule: &ConfidenceRule,
        context: &HashMap<String, String>,
    ) -> f64 {
        let mut confidence = rule.base_score;

        for multiplier in &rule.multipliers {
            // Simple condition checking - in practice this would be more sophisticated
            if !multiplier.condition.is_empty() && context.contains_key(&multiplier.condition) {
                confidence *= multiplier.multiplier;
            }
        }

        confidence.min(1.0)
    }

    /// Generate suggestions for a pattern
    fn generate_suggestions(&self, pattern: &PatternDefinition) -> Vec<String> {
        match pattern.category {
            PatternCategory::CodeSmell => vec![
                format!("Consider refactoring this {} pattern", pattern.name),
                "Apply appropriate design patterns to improve code structure".to_string(),
                "Consider breaking down complex constructs into smaller, focused units".to_string(),
            ],
            PatternCategory::SecurityVulnerability => vec![
                format!(
                    "Address this {} security vulnerability immediately",
                    pattern.name
                ),
                "Implement proper input validation and sanitization".to_string(),
                "Review authorization and authentication mechanisms".to_string(),
            ],
            PatternCategory::PerformanceIssue => vec![
                format!("Optimize this {} performance issue", pattern.name),
                "Consider algorithm and data structure improvements".to_string(),
                "Implement caching where appropriate".to_string(),
            ],
            PatternCategory::MaintainabilityProblem => vec![
                format!("Improve maintainability of this {} pattern", pattern.name),
                "Add comprehensive documentation and comments".to_string(),
                "Consider applying SOLID principles".to_string(),
            ],
            PatternCategory::GoodPractice => vec![
                format!("This {} pattern follows good practices", pattern.name),
                "Consider this approach for similar scenarios".to_string(),
            ],
            PatternCategory::AntiPattern => vec![
                format!("Avoid this {} anti-pattern", pattern.name),
                "Replace with appropriate design pattern".to_string(),
                "Consider architectural refactoring".to_string(),
            ],
        }
    }

    /// Initialize standard patterns for all supported languages
    fn initialize_standard_patterns(&mut self) {
        // Long function pattern (applies to most languages)
        let long_function_pattern = PatternDefinition {
            id: "long_function".to_string(),
            name: "Long Function".to_string(),
            description: "Function with excessive number of lines indicating poor cohesion"
                .to_string(),
            category: PatternCategory::MaintainabilityProblem,
            severity: Severity::Warning,
            ast_pattern: ASTPattern {
                node_types: vec!["function".to_string()],
                child_patterns: vec![],
                properties: HashMap::new(),
            },
            confidence_rule: ConfidenceRule {
                base_score: 0.8,
                multipliers: vec![ConfidenceMultiplier {
                    condition: "line_count".to_string(),
                    multiplier: 1.2,
                }],
            },
        };

        // Large class pattern
        let large_class_pattern = PatternDefinition {
            id: "large_class".to_string(),
            name: "Large Class".to_string(),
            description: "Class with too many methods or fields indicating SRP violation"
                .to_string(),
            category: PatternCategory::AntiPattern,
            severity: Severity::Error,
            ast_pattern: ASTPattern {
                node_types: vec!["class".to_string()],
                child_patterns: vec![],
                properties: HashMap::new(),
            },
            confidence_rule: ConfidenceRule {
                base_score: 0.9,
                multipliers: vec![], // Would be based on method/field count
            },
        };

        // Primitive obsession pattern
        let primitive_obsession_pattern = PatternDefinition {
            id: "primitive_obsession".to_string(),
            name: "Primitive Obsession".to_string(),
            description: "Excessive use of primitive types instead of domain objects".to_string(),
            category: PatternCategory::CodeSmell,
            severity: Severity::Info,
            ast_pattern: ASTPattern {
                node_types: vec!["function".to_string(), "variable".to_string()],
                child_patterns: vec![],
                properties: HashMap::new(),
            },
            confidence_rule: ConfidenceRule {
                base_score: 0.6,
                multipliers: vec![],
            },
        };

        // God object pattern
        let god_object_pattern = PatternDefinition {
            id: "god_object".to_string(),
            name: "God Object".to_string(),
            description:
                "Class with too many responsibilities (violates Single Responsibility Principle)"
                    .to_string(),
            category: PatternCategory::AntiPattern,
            severity: Severity::Critical,
            ast_pattern: ASTPattern {
                node_types: vec!["class".to_string()],
                child_patterns: vec![],
                properties: HashMap::new(),
            },
            confidence_rule: ConfidenceRule {
                base_score: 0.95,
                multipliers: vec![],
            },
        };

        // Register patterns for all supported languages
        self.register_pattern(&Language::Rust, long_function_pattern.clone());
        self.register_pattern(&Language::TypeScript, long_function_pattern.clone());
        self.register_pattern(&Language::Python, long_function_pattern.clone());
        self.register_pattern(&Language::Go, long_function_pattern.clone());
        self.register_pattern(&Language::Java, long_function_pattern.clone());
        self.register_pattern(&Language::Cpp, long_function_pattern.clone());

        self.register_pattern(&Language::Rust, large_class_pattern.clone());
        self.register_pattern(&Language::TypeScript, large_class_pattern.clone());
        self.register_pattern(&Language::Python, large_class_pattern.clone());
        self.register_pattern(&Language::Java, large_class_pattern.clone());
        self.register_pattern(&Language::Cpp, large_class_pattern.clone());

        self.register_pattern(&Language::Rust, primitive_obsession_pattern.clone());
        self.register_pattern(&Language::TypeScript, primitive_obsession_pattern.clone());
        self.register_pattern(&Language::Python, primitive_obsession_pattern.clone());
        self.register_pattern(&Language::Java, primitive_obsession_pattern.clone());
        self.register_pattern(&Language::Cpp, primitive_obsession_pattern.clone());

        self.register_pattern(&Language::Rust, god_object_pattern.clone());
        self.register_pattern(&Language::TypeScript, god_object_pattern.clone());
        self.register_pattern(&Language::Python, god_object_pattern.clone());
        self.register_pattern(&Language::Java, god_object_pattern.clone());
        self.register_pattern(&Language::Cpp, god_object_pattern.clone());
    }

    /// Register a pattern for a specific language
    pub fn register_pattern(&mut self, language: &Language, pattern: PatternDefinition) {
        self.patterns
            .entry(language.clone())
            .or_insert_with(Vec::new)
            .push(pattern);
    }

    /// Get patterns for a specific language
    pub fn get_patterns_for_language(&self, language: &Language) -> Vec<&PatternDefinition> {
        self.patterns
            .get(language)
            .map(|patterns| patterns.iter().collect())
            .unwrap_or_default()
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<Language> {
        vec![
            Language::Rust,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Go,
            Language::Java,
            Language::Cpp,
        ]
    }

    /// Update pattern recognition thresholds
    pub fn set_thresholds(&mut self, thresholds: PatternThresholds) {
        self.thresholds = thresholds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_recognizer_creation() {
        let recognizer = LanguageAwarePatternRecognizer::new();
        assert!(!recognizer.supported_languages().is_empty());
    }

    #[test]
    fn test_analyze_typescript_function() {
        let mut recognizer = LanguageAwarePatternRecognizer::new();
        let code = "function veryLongFunctionName(parameter1: string, parameter2: number, parameter3: boolean) {\n    \
                    console.log('line 1');\n    console.log('line 2');\n    console.log('line 3');\n    \
                    console.log('line 4');\n    console.log('line 5');\n    console.log('line 6');\n    \
                    console.log('line 7');\n    console.log('line 8');\n    console.log('line 9');\n    \
                    console.log('line 10');\n    console.log('line 11');\n    console.log('line 12');\n    \
                    console.log('line 13');\n    console.log('line 14');\n    console.log('line 15');\n    \
                    console.log('line 16');\n    console.log('line 17');\n    console.log('line 18');\n    \
                    console.log('line 19');\n    console.log('line 20');\n    console.log('line 21');\n    \
                    console.log('line 22');\n    console.log('line 23');\n    console.log('line 24');\n    \
                    console.log('line 25');\n}";

        let result = recognizer.analyze_code(code, &Language::TypeScript, "test.ts");
        assert!(result.is_ok());
        // The long function pattern should be detected
        if let Ok(patterns) = result {
            assert!(!patterns.is_empty());
        }
    }
}
