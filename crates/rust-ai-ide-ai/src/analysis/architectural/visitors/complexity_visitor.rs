use syn::{visit, Block, Expr, ImplItem, ItemFn, ItemImpl};

use super::super::types::{ArchitecturalFinding, CodeLocation, Severity};
use super::ArchitecturalVisitor;

/// Visitor that analyzes code complexity metrics
pub struct ComplexityVisitor<'a> {
    analyzer:     &'a ArchitecturalAnalyzer,
    findings:     Vec<ArchitecturalFinding>,
    current_file: String,
}

impl<'a> ComplexityVisitor<'a> {
    /// Create a new ComplexityVisitor
    pub fn new(analyzer: &'a ArchitecturalAnalyzer, file_path: &str) -> Self {
        Self {
            analyzer,
            findings: Vec::new(),
            current_file: file_path.to_string(),
        }
    }

    /// Calculate cyclomatic complexity of a function
    fn calculate_cyclomatic_complexity(&self, block: &Block) -> u32 {
        // Start with 1 for the function entry point
        let mut complexity = 1;

        // Visitor to count control flow constructs
        struct ComplexityCounter(u32);

        impl<'ast> visit::Visit<'ast> for ComplexityCounter {
            fn visit_expr(&mut self, node: &'ast Expr) {
                match node {
                    // Each of these adds to cyclomatic complexity
                    Expr::If(_)
                    | Expr::While(_)
                    | Expr::ForLoop(_)
                    | Expr::Loop(_)
                    | Expr::Match(_)
                    | Expr::Binary(syn::ExprBinary {
                        op: syn::BinOp::AndThen(_) | syn::BinOp::OrElse(_),
                        ..
                    }) => {
                        self.0 += 1;
                    }
                    _ => {}
                }
                visit::visit_expr(self, node);
            }
        }

        let mut counter = ComplexityCounter(0);
        visit::visit_block(&mut counter, block);

        complexity + counter.0
    }
}

impl ComplexityVisitor<'_> {
    /// Analyze the given syntax tree and return findings
    pub fn analyze(mut self, ast: &syn::File) -> Vec<ArchitecturalFinding> {
        self.visit_file(ast);
        self.findings
    }
}

impl<'a> visit::Visit<'a> for ComplexityVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a ItemFn) {
        let complexity = self.calculate_cyclomatic_complexity(&node.block);

        if complexity > self.analyzer.max_cyclomatic_complexity {
            self.findings.push(ArchitecturalFinding {
                id:         format!("high_complexity_{}", node.sig.ident),
                message:    format!(
                    "Function '{}' has a cyclomatic complexity of {} (max allowed is {})",
                    node.sig.ident, complexity, self.analyzer.max_cyclomatic_complexity
                ),
                severity:   Severity::Warning,
                location:   CodeLocation {
                    file_path: self.current_file.clone(),
                    line:      node.sig.span().line() as u32,
                    column:    node.sig.span().column() as u32,
                },
                suggestion: Some(
                    "Consider refactoring this function into smaller, more focused functions.".to_string(),
                ),
                confidence: 0.9,
                rule_id:    "HIGH_COMPLEXITY".to_string(),
            });
        }

        // Continue visiting child nodes
        visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'a ItemImpl) {
        // Check each method in the impl block
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                let complexity = self.calculate_cyclomatic_complexity(&method.block);

                if complexity > self.analyzer.max_cyclomatic_complexity {
                    self.findings.push(ArchitecturalFinding {
                        id:         format!("high_complexity_{}", method.sig.ident),
                        message:    format!(
                            "Method '{}' has a cyclomatic complexity of {} (max allowed is {})",
                            method.sig.ident, complexity, self.analyzer.max_cyclomatic_complexity
                        ),
                        severity:   Severity::Warning,
                        location:   CodeLocation {
                            file_path: self.current_file.clone(),
                            line:      method.span().line() as u32,
                            column:    method.span().column() as u32,
                        },
                        suggestion: Some(
                            "Consider refactoring this method into smaller, more focused methods.".to_string(),
                        ),
                        confidence: 0.9,
                        rule_id:    "HIGH_COMPLEXITY".to_string(),
                    });
                }
            }
        }

        // Continue visiting child nodes
        visit::visit_item_impl(self, node);
    }
}
