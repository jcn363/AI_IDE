use std::collections::{HashMap, HashSet};

use syn::visit::Visit;
use syn::*;

use crate::analysis::types::*;
use crate::error_handling::AnalysisResult;

/// Architecture analyzer for design pattern detection and recommendations
pub struct ArchitectureAnalyzer {
    patterns: HashMap<String, DesignPattern>,
}

#[derive(Clone, Debug)]
pub struct DesignPattern {
    pub name:            String,
    pub category:        PatternCategory,
    pub description:     String,
    pub benefits:        Vec<String>,
    pub indicators:      Vec<String>,
    pub detection_rules: Vec<DetectionRule>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternCategory {
    Creational,
    Structural,
    Behavioral,
    Architectural,
    Enterprise,
    Concurrency,
}

#[derive(Clone, Debug)]
pub struct DetectionRule {
    pub rule_type:  RuleType,
    pub patterns:   Vec<String>,
    pub confidence: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RuleType {
    StructPattern(String),
    TraitsPattern(String),
    FunctionPattern(String),
    ModulePattern(String),
    GenericPattern(String),
}

impl ArchitectureAnalyzer {
    /// Create a new architecture analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: HashMap::new(),
        };
        analyzer.load_standard_patterns();
        analyzer
    }

    /// Load standard design patterns
    fn load_standard_patterns(&mut self) {
        // Builder Pattern
        let builder_pattern = DesignPattern {
            name:            "Builder".to_string(),
            category:        PatternCategory::Creational,
            description:     "Separate construction of complex objects from their representation".to_string(),
            benefits:        vec![
                "Flexible object construction".to_string(),
                "Step-by-step object creation".to_string(),
                "Immutable objects".to_string(),
            ],
            indicators:      vec![
                "build()".to_string(),
                "with_* methods".to_string(),
                "separate builder struct".to_string(),
            ],
            detection_rules: vec![DetectionRule {
                rule_type:  RuleType::FunctionPattern("build".to_string()),
                patterns:   vec!["\\bbuild\\(".to_string()],
                confidence: 0.8,
            }],
        };
        self.patterns.insert("builder".to_string(), builder_pattern);

        // Strategy Pattern
        let strategy_pattern = DesignPattern {
            name:            "Strategy".to_string(),
            category:        PatternCategory::Behavioral,
            description:     "Define family of algorithms, encapsulate each, make them interchangeable".to_string(),
            benefits:        vec![
                "Runtime algorithm selection".to_string(),
                "Open-closed principle".to_string(),
                "Client isolation from implementation".to_string(),
            ],
            indicators:      vec![
                "Trait with algorithm methods".to_string(),
                "Enum or Struct implementing the trait".to_string(),
                "Runtime algorithm selection".to_string(),
            ],
            detection_rules: vec![DetectionRule {
                rule_type:  RuleType::TraitsPattern("dynamic algorithm".to_string()),
                patterns:   vec!["Box<dyn".to_string(), "&dyn".to_string()],
                confidence: 0.7,
            }],
        };
        self.patterns
            .insert("strategy".to_string(), strategy_pattern);

        // Command Pattern
        let command_pattern = DesignPattern {
            name:            "Command".to_string(),
            category:        PatternCategory::Behavioral,
            description:     "Encapsulate request as object, paramaterize clients".to_string(),
            benefits:        vec![
                "Queue operations".to_string(),
                "Undo operations".to_string(),
                "Decouple sender from receiver".to_string(),
            ],
            indicators:      vec![
                "execute".to_string(),
                "undo".to_string(),
                "command queue".to_string(),
            ],
            detection_rules: vec![DetectionRule {
                rule_type:  RuleType::FunctionPattern("execute".to_string()),
                patterns:   vec!["execute\\(".to_string()],
                confidence: 0.6,
            }],
        };
        self.patterns.insert("command".to_string(), command_pattern);

        // Factory Pattern
        let factory_pattern = DesignPattern {
            name:            "Factory".to_string(),
            category:        PatternCategory::Creational,
            description:     "Provide interface for object creation, defer instantiation to subclasses".to_string(),
            benefits:        vec![
                "Flexibility in object creation".to_string(),
                "Hide object creation complexity".to_string(),
                "Centralized creation logic".to_string(),
            ],
            indicators:      vec![
                "create_".to_string(),
                "factory methods".to_string(),
                "static constructor".to_string(),
            ],
            detection_rules: vec![DetectionRule {
                rule_type:  RuleType::FunctionPattern("create".to_string()),
                patterns:   vec!["create_\\w+\\(".to_string()],
                confidence: 0.7,
            }],
        };
        self.patterns.insert("factory".to_string(), factory_pattern);

        // Observer Pattern
        let observer_pattern = DesignPattern {
            name:            "Observer".to_string(),
            category:        PatternCategory::Behavioral,
            description:     "Define one-to-many dependency between objects".to_string(),
            benefits:        vec![
                "Loose coupling".to_string(),
                "Event-driven architecture".to_string(),
                "Dynamic subscription".to_string(),
            ],
            indicators:      vec![
                "subscribe".to_string(),
                "notify".to_string(),
                "observer trait".to_string(),
            ],
            detection_rules: vec![DetectionRule {
                rule_type:  RuleType::FunctionPattern("notify".to_string()),
                patterns:   vec!["notify\\(".to_string(), "subscribe\\(".to_string()],
                confidence: 0.8,
            }],
        };
        self.patterns
            .insert("observer".to_string(), observer_pattern);

        // Repository Pattern (Enterprise)
        let repository_pattern = DesignPattern {
            name:            "Repository".to_string(),
            category:        PatternCategory::Enterprise,
            description:     "Mediate between domain/business models and data access".to_string(),
            benefits:        vec![
                "Data access abstraction".to_string(),
                "Testable data operations".to_string(),
                "Centralized data logic".to_string(),
            ],
            indicators:      vec![
                "repository trait".to_string(),
                "find_by".to_string(),
                "save".to_string(),
                "repository pattern".to_string(),
            ],
            detection_rules: vec![DetectionRule {
                rule_type:  RuleType::TraitsPattern("repository".to_string()),
                patterns:   vec!["Repository".to_string()],
                confidence: 0.9,
            }],
        };
        self.patterns
            .insert("repository".to_string(), repository_pattern);
    }

    /// Analyze codebase for architectural patterns
    pub async fn analyze(&self, ast: &File) -> AnalysisResult<Vec<ArchitectureSuggestion>> {
        let mut suggestions = Vec::new();

        // Detect existing patterns
        let detected_patterns = self.detect_patterns(ast).await?;

        // Analyze code coupling and cohesion
        let coupling_analysis = self.analyze_coupling(ast)?;
        let cohesion_analysis = self.analyze_cohesion(ast)?;

        // Generate suggestions based on analysis
        suggestions.extend(self.generate_coupling_suggestions(&coupling_analysis)?);
        suggestions.extend(self.generate_cohesion_suggestions(&cohesion_analysis)?);
        suggestions.extend(self.generate_pattern_suggestions(&detected_patterns));

        Ok(suggestions)
    }

    /// Detect implemented design patterns
    async fn detect_patterns(&self, ast: &File) -> AnalysisResult<HashMap<String, f64>> {
        let mut detected_patterns = HashMap::new();

        // Initialize scores for all patterns
        for pattern_name in self.patterns.keys() {
            detected_patterns.insert(pattern_name.clone(), 0.0);
        }

        // Analyze the AST
        let mut visitor = PatternDetectionVisitor::new(self.patterns.clone());
        visitor.visit_file(ast);

        // Calculate confidence scores with enhanced weighting
        for (pattern_name, pattern) in &self.patterns {
            let mut total_score = 0.0;
            let mut total_weight = 0.0;

            for rule in &pattern.detection_rules {
                if visitor
                    .matches
                    .contains(&(pattern_name.clone(), rule.rule_type.clone()))
                {
                    // Use weighted scoring based on rule confidence
                    let weight = rule.confidence;
                    let adjusted_score = match rule.confidence {
                        confidence if confidence >= 0.8 => confidence * 1.3,
                        confidence if confidence >= 0.6 => confidence * 1.1,
                        confidence => confidence * 0.8,
                    };

                    total_score += adjusted_score * weight;
                    total_weight += weight;
                }
            }

            let weighted_score = if total_weight > 0.0 {
                total_score / total_weight
            } else {
                0.0
            };

            // Cap at 0.95 to leave room for context-based adjustments
            let final_score = weighted_score.min(0.95);
            detected_patterns.insert(pattern_name.clone(), final_score);
        }

        Ok(detected_patterns)
    }

    /// Analyze coupling between modules
    fn analyze_coupling(&self, ast: &File) -> AnalysisResult<CouplingAnalysis> {
        let mut visitor = CouplingAnalysisVisitor::new();
        visitor.visit_file(ast);
        Ok(CouplingAnalysis {
            inter_module_references: visitor.inter_module_refs,
            intra_module_references: visitor.intra_module_refs,
            external_dependencies:   visitor.external_deps.len(),
        })
    }

    /// Analyze cohesion within modules
    fn analyze_cohesion(&self, ast: &File) -> AnalysisResult<CohesionAnalysis> {
        let mut visitor = CohesionAnalysisVisitor::new();
        visitor.visit_file(ast);
        Ok(CohesionAnalysis {
            module_functions: visitor.module_functions,
            function_calls:   visitor.function_calls,
        })
    }

    /// Generate suggestions based on coupling analysis
    fn generate_coupling_suggestions(
        &self,
        coupling: &CouplingAnalysis,
    ) -> AnalysisResult<Vec<ArchitectureSuggestion>> {
        let mut suggestions = Vec::new();

        if coupling.external_dependencies > 10 {
            suggestions.push(ArchitectureSuggestion {
                pattern:              "Dependency Injection Container".to_string(),
                confidence:           0.8,
                location:             Location {
                    file:   "AST".to_string(),
                    line:   1,
                    column: 0,
                    offset: 0,
                },
                description:          "High external dependencies suggest DI container for better decoupling"
                    .to_string(),
                benefits:             vec![
                    "Better testability".to_string(),
                    "Reduced coupling".to_string(),
                    "Cleaner architecture".to_string(),
                ],
                implementation_steps: vec![
                    "Create a DI container struct".to_string(),
                    "Register services in the container".to_string(),
                    "Inject dependencies through constructor injection".to_string(),
                ],
            });
        }

        if coupling.inter_module_references > coupling.intra_module_references * 2 {
            suggestions.push(ArchitectureSuggestion {
                pattern:              "Microservices".to_string(),
                confidence:           0.7,
                location:             Location {
                    file:   "AST".to_string(),
                    line:   1,
                    column: 0,
                    offset: 0,
                },
                description:          "High inter-module coupling suggests microservices architecture".to_string(),
                benefits:             vec![
                    "Independent deployments".to_string(),
                    "Technology independence".to_string(),
                    "Scalability".to_string(),
                ],
                implementation_steps: vec![
                    "Identify service boundaries".to_string(),
                    "Define API contracts".to_string(),
                    "Implement service communication".to_string(),
                    "Set up service discovery".to_string(),
                ],
            });
        }

        Ok(suggestions)
    }

    /// Generate suggestions based on cohesion analysis
    fn generate_cohesion_suggestions(
        &self,
        cohesion: &CohesionAnalysis,
    ) -> AnalysisResult<Vec<ArchitectureSuggestion>> {
        let mut suggestions = Vec::new();

        // Analyze if there are functions with low cohesion
        for (module_name, functions) in &cohesion.module_functions {
            let total_calls = cohesion.function_calls.get(module_name).unwrap_or(&0);

            if *total_calls < 2 && functions.len() > 5 {
                suggestions.push(ArchitectureSuggestion {
                    pattern:              "Extract Module".to_string(),
                    confidence:           0.6,
                    location:             Location {
                        file:   format!("{}.rs", module_name),
                        line:   1,
                        column: 0,
                        offset: 0,
                    },
                    description:          format!(
                        "Module '{}' has many functions but low internal cohesion. Consider extracting into smaller \
                         modules",
                        module_name
                    ),
                    benefits:             vec![
                        "Better organization".to_string(),
                        "Improved maintainability".to_string(),
                        "Clearer responsibilities".to_string(),
                    ],
                    implementation_steps: vec![
                        "Identify logical groupings".to_string(),
                        "Create new module files".to_string(),
                        "Move related functions".to_string(),
                        "Update imports and exports".to_string(),
                    ],
                });
            }
        }

        Ok(suggestions)
    }

    /// Generate suggestions based on detected patterns with enhanced ranking
    fn generate_pattern_suggestions(&self, detected_patterns: &HashMap<String, f64>) -> Vec<ArchitectureSuggestion> {
        let mut suggestions = Vec::new();

        for (pattern_name, confidence) in detected_patterns {
            if *confidence > 0.6 {
                // Lower threshold for more suggestions, ranking will handle relevance
                if let Some(pattern) = self.patterns.get(pattern_name) {
                    let category_boost = match pattern.category {
                        PatternCategory::Enterprise => 0.05,
                        PatternCategory::Architectural => 0.03,
                        PatternCategory::Concurrency => 0.02,
                        PatternCategory::Behavioral | PatternCategory::Structural => 0.01,
                        PatternCategory::Creational => 0.0,
                    };

                    let adjusted_confidence = (*confidence + category_boost).min(1.0);

                    let (description, implementation_steps) = if *confidence > 0.9 {
                        // Very strong detection
                        (
                            format!(
                                "Strong detection of '{}' pattern usage. Consider documenting or ensuring proper \
                                 implementation.",
                                pattern.name
                            ),
                            vec![
                                "Document the pattern usage".to_string(),
                                "Ensure pattern invariant compliance".to_string(),
                                "Review for proper implementation".to_string(),
                            ],
                        )
                    } else if *confidence > 0.75 {
                        // Moderate to strong detection
                        (
                            format!(
                                "Detected '{}' pattern usage with good confidence. May benefit from pattern \
                                 refinement.",
                                pattern.name
                            ),
                            vec![
                                "Review pattern implementation".to_string(),
                                "Consider pattern consistency".to_string(),
                                "Document pattern usage if not already".to_string(),
                            ],
                        )
                    } else {
                        // Moderate detection
                        (
                            format!(
                                "Possible '{}' pattern usage detected. Consider refactoring if architecturally \
                                 beneficial.",
                                pattern.name
                            ),
                            vec![
                                "Evaluate current architecture".to_string(),
                                "Consider refactoring to full pattern implementation".to_string(),
                                "Assess architectural impact".to_string(),
                            ],
                        )
                    };

                    suggestions.push(ArchitectureSuggestion {
                        pattern: pattern.name.clone(),
                        confidence: adjusted_confidence,
                        location: Location {
                            file:   "AST".to_string(),
                            line:   1,
                            column: 0,
                            offset: 0,
                        },
                        description,
                        benefits: pattern.benefits.clone(),
                        implementation_steps,
                    });
                }
            }
        }

        // Sort suggestions by confidence descending for better ranking
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        suggestions
    }

    /// Get available patterns for reference
    pub fn get_available_patterns(&self) -> Vec<&DesignPattern> {
        self.patterns.values().collect()
    }

    /// Add custom design pattern
    pub fn add_pattern(&mut self, name: &str, pattern: DesignPattern) {
        self.patterns.insert(name.to_string(), pattern);
    }
}

/// Data structures for architectural analysis
#[derive(Clone, Debug)]
pub struct CouplingAnalysis {
    pub inter_module_references: usize,
    pub intra_module_references: usize,
    pub external_dependencies:   usize,
}

#[derive(Clone, Debug)]
pub struct CohesionAnalysis {
    pub module_functions: HashMap<String, Vec<String>>,
    pub function_calls:   HashMap<String, usize>,
}

/// Visitor for pattern detection in AST
struct PatternDetectionVisitor {
    patterns: HashMap<String, DesignPattern>,
    matches:  HashSet<(String, RuleType)>,
}

impl PatternDetectionVisitor {
    fn new(patterns: HashMap<String, DesignPattern>) -> Self {
        Self {
            patterns,
            matches: HashSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for PatternDetectionVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let fn_name = node.sig.ident.to_string();

        // Check function patterns
        for (pattern_name, pattern) in &self.patterns {
            for rule in &pattern.detection_rules {
                match &rule.rule_type {
                    RuleType::FunctionPattern(pattern) =>
                        if fn_name.contains(pattern) {
                            self.matches
                                .insert((pattern_name.clone(), rule.rule_type.clone()));
                        },
                    _ => {}
                }
            }
        }

        syn::visit::visit_item_fn(self, node);
    }

    // Removed visit_trait_item_method as TraitItemMethod doesn't exist in syn 2.0
    // fn visit_trait_item_method(&mut self, node: &'ast TraitItemMethod) {
    //     let method_sig = quote::quote!(#node).to_string();
    //
    //     for (pattern_name, pattern) in &self.patterns {
    //         for rule in &pattern.detection_rules {
    //             match &rule.rule_type {
    //                 RuleType::TraitsPattern(pattern) => {
    //                     if method_sig.contains(pattern) {
    //                         self.matches.insert((pattern_name.clone(), rule.rule_type.clone()));
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }
    //
    //     syn::visit::visit_trait_item_method(self, node);
    // }
}

/// Visitor for coupling analysis
struct CouplingAnalysisVisitor {
    inter_module_refs: usize,
    intra_module_refs: usize,
    external_deps:     HashSet<String>,
}

impl CouplingAnalysisVisitor {
    fn new() -> Self {
        Self {
            inter_module_refs: 0,
            intra_module_refs: 0,
            external_deps:     HashSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for CouplingAnalysisVisitor {
    fn visit_use_path(&mut self, node: &'ast syn::UsePath) {
        // Simplified coupling analysis - would need more sophisticated analysis for real implementation
        self.external_deps.insert(quote::quote!(#node).to_string());
        syn::visit::visit_use_path(self, node);
    }
}

/// Visitor for cohesion analysis
struct CohesionAnalysisVisitor {
    module_functions: HashMap<String, Vec<String>>,
    function_calls:   HashMap<String, usize>,
    current_module:   String,
}

impl CohesionAnalysisVisitor {
    fn new() -> Self {
        Self {
            module_functions: HashMap::new(),
            function_calls:   HashMap::new(),
            current_module:   "main".to_string(), // Default module
        }
    }
}

impl<'ast> Visit<'ast> for CohesionAnalysisVisitor {
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.current_module = node.ident.to_string();
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let module_functions = self
            .module_functions
            .entry(self.current_module.clone())
            .or_insert_with(Vec::new);

        module_functions.push(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        let call_count = self
            .function_calls
            .entry(self.current_module.clone())
            .or_insert(0);
        *call_count += 1;
        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detection() {
        let analyzer = ArchitectureAnalyzer::new();

        // Test that patterns were loaded
        assert!(analyzer.patterns.contains_key("builder"));
        assert!(analyzer.patterns.contains_key("strategy"));
        assert!(analyzer.patterns.contains_key("command"));
    }

    #[test]
    fn test_pattern_categories() {
        let analyzer = ArchitectureAnalyzer::new();

        if let Some(builder) = analyzer.patterns.get("builder") {
            assert_eq!(builder.category, PatternCategory::Creational);
        }

        if let Some(strategy) = analyzer.patterns.get("strategy") {
            assert_eq!(strategy.category, PatternCategory::Behavioral);
        }
    }

    #[tokio::test]
    async fn test_architecture_analysis() {
        let analyzer = ArchitectureAnalyzer::new();
        let code = r#"
            trait Builder<T> {
                fn build(self) -> T;
                fn with_name(mut self, name: String) -> Self;
            }

            struct MyBuilder {
                name: Option<String>,
            }

            impl Builder<i32> for MyBuilder {
                fn build(self) -> i32 { 42 }
                fn with_name(mut self, name: String) -> Self { self }
            }

            pub fn create_builder() -> MyBuilder {
                MyBuilder { name: None }
            }
        "#;

        if let Ok(ast) = syn::parse_file(code) {
            let result = analyzer.analyze(&ast).await;
            assert!(result.is_ok());

            let suggestions = result.unwrap();
            println!("Found {} architectural suggestions", suggestions.len());
            // We expect at least some suggestions based on the code structure
        }
    }
}
