//! Utility functions for code analysis

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use syn::visit::Visit;
use syn::{File, Item, ItemFn, ItemMod, ItemStruct, ItemTrait, ItemImpl};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Creates a new progress bar with the given length and message
pub fn progress_bar(len: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} {msg}: [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Get the module path for a file
pub fn get_module_path(file_path: &Path) -> String {
    file_path
        .with_extension("")
        .display()
        .to_string()
        .replace(std::path::MAIN_SEPARATOR, "::")
}

/// Check if a path is a test file
pub fn is_test_file(file_path: &Path) -> bool {
    file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.ends_with("_test") || s.ends_with("_tests"))
        .unwrap_or(false)
}

/// Count the number of lines in a string
pub fn count_lines(content: &str) -> usize {
    content.lines().count()
}

/// Count the number of non-empty lines in a string
pub fn count_non_empty_lines(content: &str) -> usize {
    content.lines().filter(|line| !line.trim().is_empty()).count()
}

/// Count the number of comment lines in a string
pub fn count_comment_lines(content: &str) -> usize {
    content
        .lines()
        .filter(|line| line.trim().starts_with("//") || line.trim().starts_with("/*"))
        .count()
}

/// Get the cyclomatic complexity of a function
pub fn calculate_cyclomatic_complexity(func: &ItemFn) -> usize {
    struct ComplexityVisitor {
        complexity: usize,
    }

    impl<'ast> Visit<'ast> for ComplexityVisitor {
        fn visit_expr(&mut self, _node: &'ast syn::Expr) {
            println!("DEBUG: ComplexityVisitor processing expression with 'ast lifetime");
            self.complexity += 1;
            syn::visit::visit_expr(self, _node);
        }

        fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
            self.complexity += 1;
            syn::visit::visit_expr_if(self, node);
        }

        fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
            self.complexity += node.arms.len();
            syn::visit::visit_expr_match(self, node);
        }

        fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
            self.complexity += 1;
            syn::visit::visit_expr_loop(self, node);
        }

        fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
            self.complexity += 1;
            syn::visit::visit_expr_while(self, node);
        }

        fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
            self.complexity += 1;
            syn::visit::visit_expr_for_loop(self, node);
        }
    }

    let mut visitor = ComplexityVisitor { complexity: 1 };
    visitor.visit_item_fn(func);
    visitor.complexity
}

/// Get the maintainability index for a file
pub fn calculate_maintainability_index(
    lines_of_code: usize,
    cyclomatic_complexity: f64,
    halstead_volume: f64,
) -> f64 {
    let volume = halstead_volume.max(1.0);
    let loc = lines_of_code.max(1) as f64;
    let cm = cyclomatic_complexity.max(1.0);
    
    // Calculate maintainability index (0-100 scale)
    let mi = 171.0 - 5.2 * volume.ln() - 0.23 * cm - 16.2 * loc.ln();
    mi.max(0.0).min(100.0)
}

/// Get the Halstead metrics for a file
pub fn calculate_halstead_metrics(ast: &File) -> (f64, f64, f64) {
    struct Metrics {
        operators: std::collections::HashSet<String>,
        operands: std::collections::HashSet<String>,
        operator_count: usize,
        operand_count: usize,
    }

    impl<'ast> Visit<'ast> for Metrics {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            use syn::Expr;
            
            // Count operators
            match node {
                Expr::Binary(bin) => {
                    self.operators.insert(format!("{:?}", bin.op));
                    self.operator_count += 1;
                }
                Expr::Unary(unary) => {
                    self.operators.insert(format!("{:?}", unary.op));
                    self.operator_count += 1;
                }
                Expr::Assign(assign) => {
                    self.operators.insert("=".to_string());
                    self.operator_count += 1;
                }
                Expr::AssignOp(assign) => {
                    self.operators
                        .insert(format!("{}={:?}", assign.op, assign.op));
                    self.operator_count += 1;
                }
                _ => {}
            }

            // Count operands (identifiers and literals)
            match node {
                Expr::Path(expr_path) => {
                    if let Some(ident) = expr_path.path.get_ident() {
                        self.operands.insert(ident.to_string());
                        self.operand_count += 1;
                    }
                }
                Expr::Lit(lit) => {
                    self.operands.insert(format!("{:?}", lit.lit));
                    self.operand_count += 1;
                }
                _ => {}
            }

            syn::visit::visit_expr(self, node);
        }
    }

    let mut metrics = Metrics {
        operators: std::collections::HashSet::new(),
        operands: std::collections::HashSet::new(),
        operator_count: 0,
        operand_count: 0,
    };

    metrics.visit_file(ast);

    let n1 = metrics.operators.len() as f64;
    let n2 = metrics.operands.len() as f64;
    let N1 = metrics.operator_count as f64;
    let N2 = metrics.operand_count as f64;

    // Calculate Halstead metrics
    let vocabulary = n1 + n2;
    let length = N1 + N2;
    let volume = length * (vocabulary + 1.0).log2();
    let difficulty = (n1 / 2.0) * (N2 / n2);
    let effort = volume * difficulty;

    (volume, difficulty, effort)
}

/// Get the depth of inheritance for a type
pub fn get_inheritance_depth(ast: &File) -> usize {
    struct InheritanceVisitor {
        max_depth: usize,
        current_depth: usize,
    }

    impl<'ast> Visit<'ast> for InheritanceVisitor {
        fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
            if let Some((_, path, _)) = &node.trait_ {
                self.current_depth += 1;
                self.max_depth = self.max_depth.max(self.current_depth);
                syn::visit::visit_path(self, path);
                self.current_depth -= 1;
            }
            syn::visit::visit_item_impl(self, node);
        }
    }

    let mut visitor = InheritanceVisitor {
        max_depth: 0,
        current_depth: 0,
    };

    visitor.visit_file(ast);
    visitor.max_depth
}

/// Get the number of dependencies for a module
pub fn count_dependencies(ast: &File) -> usize {
    let mut deps = std::collections::HashSet::new();

    for item in &ast.items {
        match item {
            Item::Use(item_use) => {
                if let Some(ident) = item_use.path.get_ident() {
                    deps.insert(ident.to_string());
                }
            }
            Item::ExternCrate(item_extern) => {
                deps.insert(item_extern.ident.to_string());
            }
            _ => {}
        }
    }

    deps.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_count_lines() {
        let content = "line1\nline2\nline3";
        assert_eq!(count_lines(content), 3);
    }

    #[test]
    fn test_count_non_empty_lines() {
        let content = "line1\n\nline2\n  \nline3";
        assert_eq!(count_non_empty_lines(content), 3);
    }

    #[test]
    fn test_count_comment_lines() {
        let content = "// comment1\ncode1\n/* comment2 */\ncode2\n// comment3";
        assert_eq!(count_comment_lines(content), 3);
    }

    #[test]
    fn test_calculate_cyclomatic_complexity() {
        let func: ItemFn = parse_quote! {
            fn test(x: i32) -> i32 {
                if x > 0 {
                    return 1;
                } else if x < 0 {
                    return -1;
                }
                0
            }
        };
        assert_eq!(calculate_cyclomatic_complexity(&func), 3);
    }

    #[test]
    fn test_calculate_maintainability_index() {
        let mi = calculate_maintainability_index(100, 5.0, 1000.0);
        assert!(mi > 0.0 && mi <= 100.0);
    }
}
