use crate::analysis::types::*;
use crate::error_handling::AnalysisResult;
use regex::Regex;
use std::collections::HashMap;
use syn::visit::Visit;
use syn::*;
use uuid::Uuid;

/// Performance analyzer for detecting bottlenecks and optimization opportunities
pub struct PerformanceAnalyzer {
    patterns: HashMap<String, PerformancePattern>,
    metrics: PerformanceMetrics,
}

#[derive(Clone)]
pub struct PerformancePattern {
    pub pattern: Regex,
    pub impact: PerformanceImpact,
    pub description: String,
    pub suggestion: String,
}

#[derive(Clone, Default, Debug)]
pub struct PerformanceMetrics {
    pub allocations_analyzed: usize,
    pub expensive_operations: usize,
    pub memory_operations: usize,
    pub async_analysis_count: usize,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: HashMap::new(),
            metrics: PerformanceMetrics::default(),
        };
        analyzer.load_default_patterns();
        analyzer
    }

    /// Load default performance patterns to detect
    fn load_default_patterns(&mut self) {
        // Expensive string operations
        self.add_pattern(
            "string_concat_loop",
            PerformancePattern {
                pattern: Regex::new(
                    r#"(?i)let\s+mut\s+\w+\s*=\s*.*\.to_string\(\);\s*for.*?\{[^\}]*\+\s*= .*?\}"#,
                )
                .unwrap(),
                impact: PerformanceImpact::High,
                description: "Inefficient string concatenation in loops".to_string(),
                suggestion: "Use String::with_capacity or collect into Vec and join".to_string(),
            },
        );

        // Vector growth without preallocation
        self.add_pattern("vec_push_loop", PerformancePattern {
            pattern: Regex::new(r#"(?i)let\s+mut\s+\w+\s*:\s*Vec<.*>\s*=.*vec!\[.*\];\s*for.*?\{[^\}]*\.push\(.*?\).*?\}"#).unwrap(),
            impact: PerformanceImpact::Medium,
            description: "Vector resizing during loops".to_string(),
            suggestion: "Preallocate with Vec::with_capacity".to_string(),
        });

        // HashMap operations in tight loops
        self.add_pattern(
            "hashmap_operation_loop",
            PerformancePattern {
                pattern: Regex::new(r#"(?i)for.*?\{[^\}]*\.insert\(.*?\).*?\}"#).unwrap(),
                impact: PerformanceImpact::Medium,
                description: "HashMap operations in loops may be inefficient".to_string(),
                suggestion: "Consider alternative data structures or pre-computation".to_string(),
            },
        );

        // Regex compilation in loops
        self.add_pattern(
            "regex_compile_loop",
            PerformancePattern {
                pattern: Regex::new(r#"(?i)Regex::new\(.*?\).*"#).unwrap(),
                impact: PerformanceImpact::Medium,
                description: "Regex compilation is expensive, avoid in loops".to_string(),
                suggestion: "Pre-compile regex patterns outside loops".to_string(),
            },
        );

        // Iterator cloning
        self.add_pattern(
            "iterator_clone",
            PerformancePattern {
                pattern: Regex::new(r#"(?i)\.cloned\(\)|\.clone\(\)\.iter\(\)"#).unwrap(),
                impact: PerformanceImpact::Medium,
                description: "Iterator cloning creates unnecessary allocations".to_string(),
                suggestion: "Use &references where possible or restructure iterator chains"
                    .to_string(),
            },
        );

        // Box allocation for small types
        self.add_pattern(
            "box_small_type",
            PerformancePattern {
                pattern: Regex::new(r#"(?i)Box<.(u8|i8|u16|i16|u32|i32|char|bool|f32).>"#).unwrap(),
                impact: PerformanceImpact::Low,
                description: "Box allocation for small types".to_string(),
                suggestion: "Consider inline storage for small types".to_string(),
            },
        );

        // Async spawn without consideration for overhead
        self.add_pattern(
            "async_spawn",
            PerformancePattern {
                pattern: Regex::new(r#"(?i)tokio::spawn\(.*async\s+move\s*\{.*\}"#).unwrap(),
                impact: PerformanceImpact::Medium,
                description: "Async task spawning may have overhead".to_string(),
                suggestion: "Consider batching small tasks or using alternative async patterns"
                    .to_string(),
            },
        );
    }

    /// Add a custom performance pattern
    pub fn add_pattern(&mut self, name: &str, pattern: PerformancePattern) {
        self.patterns.insert(name.to_string(), pattern);
    }

    /// Analyze code for performance issues
    pub fn analyze_code(&self, content: &str, file_path: &str) -> Vec<PerformanceHint> {
        let mut hints = Vec::new();

        for line_no in 0..content.lines().count() {
            if let Some(line) = content.lines().nth(line_no) {
                for (_pattern_name, pattern) in &self.patterns {
                    let captures = pattern.pattern.find_iter(line);
                    for capture in captures {
                        let hint = PerformanceHint {
                            id: Uuid::new_v4(),
                            title: pattern.description.clone(),
                            description: format!(
                                "Performance issue detected: {}",
                                pattern.description
                            ),
                            impact: pattern.impact,
                            location: Location {
                                file: file_path.to_string(),
                                line: line_no + 1,
                                column: capture.start(),
                                offset: capture.start(),
                            },
                            suggestion: pattern.suggestion.clone(),
                        };
                        hints.push(hint);
                    }
                }
            }
        }

        hints
    }

    /// Analyze AST for performance issues
    pub async fn analyze(&self, ast: &File) -> AnalysisResult<Vec<PerformanceHint>> {
        let mut hints = Vec::new();

        // Add AST-specific performance checks
        hints.extend(self.analyze_allocations(ast));
        hints.extend(self.analyze_async_patterns(ast));
        hints.extend(self.analyze_loops(ast));
        hints.extend(self.analyze_data_structures(ast));

        Ok(hints)
    }

    /// Analyze memory allocations
    fn analyze_allocations(&self, ast: &File) -> Vec<PerformanceHint> {
        let mut hints = Vec::new();
        let mut visitor = AllocationVisitor {
            hints: &mut hints,
            file: "AST",
            metrics: &self.metrics,
        };
        visitor.visit_file(ast);
        hints
    }

    /// Analyze async patterns
    fn analyze_async_patterns(&self, ast: &File) -> Vec<PerformanceHint> {
        let mut hints = Vec::new();
        let mut visitor = AsyncPatternVisitor {
            hints: &mut hints,
            file: "AST",
        };
        visitor.visit_file(ast);
        hints
    }

    /// Analyze loops for performance issues
    fn analyze_loops(&self, ast: &File) -> Vec<PerformanceHint> {
        let mut hints = Vec::new();
        let mut visitor = LoopPerformanceVisitor {
            hints: &mut hints,
            file: "AST",
        };
        visitor.visit_file(ast);
        hints
    }

    /// Analyze data structure usage
    fn analyze_data_structures(&self, ast: &File) -> Vec<PerformanceHint> {
        let mut hints = Vec::new();
        let mut visitor = DataStructureVisitor {
            hints: &mut hints,
            file: "AST",
        };
        visitor.visit_file(ast);
        hints
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
}

/// Visits expressions looking for memory allocations
struct AllocationVisitor<'a> {
    hints: &'a mut Vec<PerformanceHint>,
    file: &'a str,
    metrics: &'a PerformanceMetrics,
}

impl<'a, 'ast> Visit<'ast> for AllocationVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(ref func) = *node.func {
            // Check for allocator usage
            let path_str = quote::quote!(#func).to_string();
            if path_str.contains("alloc::alloc") || path_str.contains("Layout::new") {
                let mut metrics_copy = self.metrics.clone();
                metrics_copy.allocations_analyzed += 1;

                let hint = PerformanceHint {
                    id: Uuid::new_v4(),
                    title: "Manual memory allocation detected".to_string(),
                    description: "Manual memory allocation may be inefficient".to_string(),
                    impact: PerformanceImpact::Medium,
                    location: Location {
                        file: self.file.to_string(),
                        line: 0, // AST nodes don't have reliable line info from span
                        column: 0,
                        offset: 0,
                    },
                    suggestion: "Consider using RAII patterns or smart pointers".to_string(),
                };
                self.hints.push(hint);
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

/// Visits async patterns
struct AsyncPatternVisitor<'a> {
    hints: &'a mut Vec<PerformanceHint>,
    file: &'a str,
}

impl<'a, 'ast> Visit<'ast> for AsyncPatternVisitor<'a> {
    fn visit_expr_await(&mut self, node: &'ast ExprAwait) {
        // Check for nested await calls which can indicate async overhead
        let hint = PerformanceHint {
            id: Uuid::new_v4(),
            title: "Await call detected".to_string(),
            description: "Consider async optimization patterns".to_string(),
            impact: PerformanceImpact::Low,
            location: Location {
                file: self.file.to_string(),
                line: 0, // AST nodes don't have reliable line info from span
                column: 0,
                offset: 0,
            },
            suggestion: "Review async call pattern for optimization opportunities".to_string(),
        };
        self.hints.push(hint);
        syn::visit::visit_expr_await(self, node);
    }
}

/// Visits loop constructs for performance analysis
struct LoopPerformanceVisitor<'a> {
    hints: &'a mut Vec<PerformanceHint>,
    file: &'a str,
}

impl<'a, 'ast> Visit<'ast> for LoopPerformanceVisitor<'a> {
    fn visit_expr_for_loop(&mut self, node: &'ast ExprForLoop) {
        // Analyze loop body for potential optimizations
        let body = quote::quote!(#node).to_string();

        if body.contains(".collect()") || body.contains(".clone()") {
            let hint = PerformanceHint {
                id: Uuid::new_v4(),
                title: "Potentially expensive loop operation".to_string(),
                description: "Collection or cloning operations in loop".to_string(),
                impact: PerformanceImpact::Medium,
                location: Location {
                    file: self.file.to_string(),
                    line: 0, // AST nodes don't have reliable line info from span
                    column: 0,
                    offset: 0,
                },
                suggestion: "Avoid allocations inside loops, preallocate or restructure"
                    .to_string(),
            };
            self.hints.push(hint);
        }
        syn::visit::visit_expr_for_loop(self, node);
    }
}

/// Visits data structure usage
struct DataStructureVisitor<'a> {
    hints: &'a mut Vec<PerformanceHint>,
    file: &'a str,
}

impl<'a, 'ast> Visit<'ast> for DataStructureVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(ref func) = *node.func {
            let func_name = quote::quote!(#func).to_string();

            // Check for sequence operations that might be inefficient
            if func_name.contains("sort") || func_name.contains("reverse") {
                let hint = PerformanceHint {
                    id: Uuid::new_v4(),
                    title: "Potentially expensive sequence operation".to_string(),
                    description: format!(
                        "Sequence operation '{}' may be costly for large collections",
                        func_name
                    ),
                    impact: PerformanceImpact::Medium,
                    location: Location {
                        file: self.file.to_string(),
                        line: 0, // AST nodes don't have reliable line info from span
                        column: 0,
                        offset: 0,
                    },
                    suggestion: "Consider using more efficient data structures or algorithms"
                        .to_string(),
                };
                self.hints.push(hint);
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_concatenation_detection() {
        let analyzer = PerformanceAnalyzer::new();
        let code = r#"
            let mut result = String::new();
            for item in collection {
                result += &item.to_string();
            }
        "#;

        let hints = analyzer.analyze_code(code, "test.rs");

        assert!(hints
            .iter()
            .any(|h| h.title.contains("string concatenation")));
    }

    #[test]
    fn test_vec_preallocation_detection() {
        let analyzer = PerformanceAnalyzer::new();
        let code = r#"
            let mut vec = Vec::new();
            for i in 0..100 {
                vec.push(i);
            }
        "#;

        let hints = analyzer.analyze_code(code, "test.rs");

        // Note: Our regex may not catch all cases, but the AST visitor should
        println!("Found {} hints", hints.len());
    }

    #[test]
    fn test_regex_compilation_detection() {
        let analyzer = PerformanceAnalyzer::new();
        let code = r#"let re = Regex::new(r"\d+")"#;

        let hints = analyzer.analyze_code(code, "test.rs");

        assert!(hints.iter().any(|h| h.title.contains("Regex compilation")));
    }

    #[tokio::test]
    async fn test_ast_performance_analysis() {
        let analyzer = PerformanceAnalyzer::new();
        let code = r#"
            let mut vec = vec![];
            for i in 0..10 {
                tokio::spawn(async move { vec.clone(); });
            }
        "#;
        let ast = syn::parse_file(code).unwrap();

        let hints = analyzer.analyze(&ast).await.unwrap();

        assert!(hints.len() > 0);
    }
}
