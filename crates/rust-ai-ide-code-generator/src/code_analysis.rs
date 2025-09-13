//! Code analysis utilities for generation decision making

use super::*;

#[async_trait::async_trait]
pub trait CodeAnalyzer {
    async fn analyze(&self, code: &str) -> Result<CodeAnalysisResult, CodeGenerationError>;
}

pub struct CodeAnalysisResult {
    pub patterns:     Patterns,
    pub complexity:   ComplexityMetrics,
    pub dependencies: Vec<String>,
    pub suggestions:  Vec<String>,
}

pub struct Patterns {
    pub sync_functions:  usize,
    pub async_functions: usize,
    pub structs:         usize,
    pub traits:          usize,
    pub tests:           usize,
}

pub struct ComplexityMetrics {
    pub cyclomatic:         f64,
    pub nesting_depth:      usize,
    pub lines_per_function: f64,
    pub function_count:     usize,
}

pub struct CodeComplexityAnalyzer;

#[async_trait::async_trait]
impl CodeAnalyzer for CodeComplexityAnalyzer {
    async fn analyze(&self, code: &str) -> Result<CodeAnalysisResult, CodeGenerationError> {
        let ast = syn::parse_file(code).map_err(|e| CodeGenerationError::ParseError(e.to_string()))?;

        let patterns = self.analyze_patterns(&ast);
        let complexity = self.analyze_complexity(&ast);

        Ok(CodeAnalysisResult {
            patterns,
            complexity,
            dependencies: Vec::new(), // Would need dependency analysis
            suggestions: Vec::new(),  // Would need additional analysis
        })
    }
}

impl CodeComplexityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn analyze_patterns(&self, ast: &File) -> Patterns {
        let mut patterns = Patterns {
            sync_functions:  0,
            async_functions: 0,
            structs:         0,
            traits:          0,
            tests:           0,
        };

        struct PatternVisitor<'a> {
            patterns: &'a mut Patterns,
        }

        impl<'a, 'ast> syn::visit::Visit<'ast> for PatternVisitor<'a> {
            fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
                if node.sig.asyncness.is_some() {
                    self.patterns.async_functions += 1;
                } else {
                    self.patterns.sync_functions += 1;
                }
                if node
                    .attrs
                    .iter()
                    .any(|attr| attr.path().segments.iter().any(|seg| seg.ident == "test"))
                {
                    self.patterns.tests += 1;
                }
            }

            fn visit_item_struct(&mut self, _node: &'ast syn::ItemStruct) {
                self.patterns.structs += 1;
            }

            fn visit_item_trait(&mut self, _node: &'ast syn::ItemTrait) {
                self.patterns.traits += 1;
            }
        }

        let mut visitor = PatternVisitor {
            patterns: &mut patterns,
        };
        visitor.visit_file(ast);

        patterns
    }

    fn analyze_complexity(&self, ast: &File) -> ComplexityMetrics {
        let mut metrics = ComplexityMetrics {
            cyclomatic:         0.0,
            nesting_depth:      0,
            lines_per_function: 0.0,
            function_count:     0,
        };

        struct ComplexityVisitor<'a> {
            metrics:       &'a mut ComplexityMetrics,
            current_depth: usize,
        }

        impl<'a, 'ast> syn::visit::Visit<'ast> for ComplexityVisitor<'a> {
            fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
                self.metrics.cyclomatic += 1.0;
                let prev_depth = self.current_depth;
                self.current_depth += 1;
                visit::visit_expr_if(self, node);
                self.current_depth = prev_depth;
            }

            fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
                self.metrics.cyclomatic += 1.0;
                visit::visit_expr_for_loop(self, node);
            }

            fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
                self.metrics.cyclomatic += 1.0;
                visit::visit_expr_while(self, node);
            }

            fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
                self.metrics.function_count += 1;
                // Rough estimation of lines
                let body_lines = quote::quote!(#node).to_string().lines().count() as f64;
                if self.metrics.function_count > 0 {
                    self.metrics.lines_per_function =
                        (self.metrics.lines_per_function * (self.metrics.function_count - 1) as f64 + body_lines)
                            / self.metrics.function_count as f64;
                }
                visit::visit_item_fn(self, node);
            }
        }

        let mut visitor = ComplexityVisitor {
            metrics:       &mut metrics,
            current_depth: 0,
        };
        visitor.visit_file(ast);

        metrics.nesting_depth = visitor.current_depth;
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_code_analysis() {
        let analyzer = CodeComplexityAnalyzer::new();
        let code = r#"
            fn func1() {
                if true { }
            }

            fn func2() {
                for i in 0..10 {}
            }

            struct Test {}
        "#;

        let result = analyzer.analyze(code).await.unwrap();
        assert!(result.patterns.sync_functions >= 2);
        assert!(result.patterns.structs >= 1);
    }
}
