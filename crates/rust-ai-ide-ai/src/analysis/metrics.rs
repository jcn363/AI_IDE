//! Code metrics calculation and analysis

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

use syn::visit::Visit;
use syn::{visit, Expr, File, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Stmt};

/// Comprehensive code metrics for a Rust file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    /// Lines of code
    pub lines_of_code: usize,
    /// Number of functions
    pub function_count: usize,
    /// Number of structs
    pub struct_count: usize,
    /// Number of traits
    pub trait_count: usize,
    /// Number of implementations
    pub impl_count: usize,
    /// Number of modules
    pub module_count: usize,
    /// Average cyclomatic complexity
    pub average_cyclomatic_complexity: f64,
    /// Maximum cyclomatic complexity
    pub max_cyclomatic_complexity: usize,
    /// Maintainability index (0-100)
    pub maintainability_index: f64,
    /// Halstead metrics
    pub halstead: HalsteadMetrics,
    /// Depth of inheritance
    pub inheritance_depth: usize,
    /// Number of dependencies
    pub dependency_count: usize,
    /// Number of comments
    pub comment_count: usize,
    /// Comment ratio (comments / total lines)
    pub comment_ratio: f64,
    /// Number of test functions
    pub test_function_count: usize,
    /// Number of documentation comments
    pub doc_comment_count: usize,
    /// Number of unsafe blocks
    pub unsafe_block_count: usize,
    /// Number of public items
    pub public_item_count: usize,
    /// Number of private items
    pub private_item_count: usize,
}

/// Halstead complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    /// Program vocabulary
    pub vocabulary: f64,
    /// Program length
    pub length: f64,
    /// Calculated program length
    pub calculated_length: f64,
    /// Volume
    pub volume: f64,
    /// Difficulty
    pub difficulty: f64,
    /// Effort
    pub effort: f64,
    /// Time required to program (seconds)
    pub time: f64,
    /// Number of delivered bugs
    pub bugs: f64,
}

impl Default for CodeMetrics {
    fn default() -> Self {
        Self {
            lines_of_code: 0,
            function_count: 0,
            struct_count: 0,
            trait_count: 0,
            impl_count: 0,
            module_count: 0,
            average_cyclomatic_complexity: 1.0,
            max_cyclomatic_complexity: 1,
            maintainability_index: 100.0,
            halstead: HalsteadMetrics::default(),
            inheritance_depth: 0,
            dependency_count: 0,
            comment_count: 0,
            comment_ratio: 0.0,
            test_function_count: 0,
            doc_comment_count: 0,
            unsafe_block_count: 0,
            public_item_count: 0,
            private_item_count: 0,
        }
    }
}

impl Default for HalsteadMetrics {
    fn default() -> Self {
        Self {
            vocabulary: 0.0,
            length: 0.0,
            calculated_length: 0.0,
            volume: 0.0,
            difficulty: 0.0,
            effort: 0.0,
            time: 0.0,
            bugs: 0.0,
        }
    }
}

/// Calculate metrics for a Rust file
pub fn calculate_metrics(syntax_tree: &File, source: &str) -> CodeMetrics {
    let mut metrics = CodeMetrics::default();

    // Count lines and comments
    metrics.lines_of_code = source.lines().count();
    metrics.comment_count = count_comment_lines(source);
    metrics.comment_ratio = if metrics.lines_of_code > 0 {
        metrics.comment_count as f64 / metrics.lines_of_code as f64
    } else {
        0.0
    };

    // Count doc comments
    metrics.doc_comment_count = count_doc_comment_lines(source);

    // Count items
    let mut visitor = MetricsVisitor::new();
    visitor.visit_file(syntax_tree);

    metrics.function_count = visitor.function_count;
    metrics.struct_count = visitor.struct_count;
    metrics.trait_count = visitor.trait_count;
    metrics.impl_count = visitor.impl_count;
    metrics.module_count = visitor.module_count;
    metrics.test_function_count = visitor.test_function_count;
    metrics.unsafe_block_count = visitor.unsafe_block_count;
    metrics.public_item_count = visitor.public_item_count;
    metrics.private_item_count = visitor.private_item_count;

    // Calculate cyclomatic complexity
    let total_cyclomatic = visitor.cyclomatic_complexity.iter().sum::<usize>();
    metrics.average_cyclomatic_complexity = if !visitor.cyclomatic_complexity.is_empty() {
        total_cyclomatic as f64 / visitor.cyclomatic_complexity.len() as f64
    } else {
        1.0
    };
    metrics.max_cyclomatic_complexity =
        visitor.cyclomatic_complexity.into_iter().max().unwrap_or(1);

    // Calculate Halstead metrics
    let (volume, difficulty, effort) = calculate_halstead_metrics(syntax_tree);

    metrics.halstead = HalsteadMetrics {
        vocabulary: 0.0,        // Will be calculated
        length: 0.0,            // Will be calculated
        calculated_length: 0.0, // Will be calculated
        volume,
        difficulty,
        effort,
        time: effort / 18.0,                   // Standard time calculation
        bugs: volume.powf(2.0 / 3.0) / 3000.0, // Standard bug calculation
    };

    // Calculate maintainability index
    metrics.maintainability_index = calculate_maintainability_index(
        metrics.lines_of_code,
        metrics.average_cyclomatic_complexity,
        metrics.halstead.volume,
    );

    metrics
}

/// Visitor for collecting metrics from the AST
struct MetricsVisitor {
    function_count: usize,
    struct_count: usize,
    trait_count: usize,
    impl_count: usize,
    module_count: usize,
    test_function_count: usize,
    unsafe_block_count: usize,
    public_item_count: usize,
    private_item_count: usize,
    cyclomatic_complexity: Vec<usize>,
    in_unsafe_block: bool,
}

impl MetricsVisitor {
    fn new() -> Self {
        Self {
            function_count: 0,
            struct_count: 0,
            trait_count: 0,
            impl_count: 0,
            module_count: 0,
            test_function_count: 0,
            unsafe_block_count: 0,
            public_item_count: 0,
            private_item_count: 0,
            cyclomatic_complexity: Vec::new(),
            in_unsafe_block: false,
        }
    }

    fn visit_visibility(&mut self, vis: &syn::Visibility) {
        if let syn::Visibility::Public(_) = vis {
            self.public_item_count += 1;
        } else {
            self.private_item_count += 1;
        }
    }
}

impl<'ast> Visit<'ast> for MetricsVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Check if it's a test function
        let is_test = node
            .attrs
            .iter()
            .any(|attr| attr.path.is_ident("test") || attr.path.is_ident("tokio::test"));

        if is_test {
            self.test_function_count += 1;
        } else {
            self.function_count += 1;

            // Calculate cyclomatic complexity for non-test functions
            let complexity = calculate_cyclomatic_complexity(node);
            self.cyclomatic_complexity.push(complexity);
        }

        // Visit visibility
        self.visit_visibility(&node.vis);

        // Continue visiting the function body
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.struct_count += 1;
        self.visit_visibility(&node.vis);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.trait_count += 1;
        self.visit_visibility(&node.vis);
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.impl_count += 1;
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.module_count += 1;
        self.visit_visibility(&node.vis);

        // Only visit the module if it's not a mod declaration without a block
        if node.content.is_some() {
            syn::visit::visit_item_mod(self, node);
        }
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.unsafe_block_count += 1;
        let was_in_unsafe = self.in_unsafe_block;
        self.in_unsafe_block = true;
        syn::visit::visit_expr_unsafe(self, node);
        self.in_unsafe_block = was_in_unsafe;
    }
}

/// Count the number of comment lines in a string
fn count_comment_lines(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.ends_with("*/")
        })
        .count()
}

/// Count the number of documentation comment lines in a string
fn count_doc_comment_lines(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("///") || trimmed.starts_with("//!") || trimmed.starts_with("/**")
        })
        .count()
}

/// Calculate Halstead metrics for a given syntax tree
fn calculate_halstead_metrics(syntax_tree: &syn::File) -> (f64, f64, f64) {
    // Calculate Halstead metrics
    let mut operators = 0;
    let mut operands = 0;
    let mut unique_operators = HashSet::new();
    let mut unique_operands = HashSet::new();

    for node in syntax_tree.items.iter() {
        match node {
            syn::Item::Fn(func) => {
                for arg in &func.sig.inputs {
                    match arg {
                        syn::FnArg::Typed(arg) => {
                            operands += 1;
                            unique_operands.insert(arg.pat.to_token_stream().to_string());
                        }
                        _ => {}
                    }
                }

                for statement in &func.block.stmts {
                    match statement {
                        syn::Stmt::Expr(expr) => match expr {
                            syn::Expr::Binary(bin_expr) => {
                                operators += 1;
                                unique_operators.insert(bin_expr.op.to_token_stream().to_string());
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let volume =
        (operators + operands) as f64 * (unique_operators.len() + unique_operands.len()) as f64;
    let difficulty =
        (unique_operators.len() as f64 / 2.0) * (operands as f64 / unique_operands.len() as f64);
    let effort = difficulty * volume;

    (volume, difficulty, effort)
}

/// Calculate maintainability index
fn calculate_maintainability_index(
    lines_of_code: usize,
    average_cyclomatic_complexity: f64,
    volume: f64,
) -> f64 {
    let mut index =
        171.0 - 5.2 * (average_cyclomatic_complexity).ln() - 0.23 * lines_of_code as f64;
    index += 16.2 * (lines_of_code as f64 / (lines_of_code + volume)) * 100.0;
    index.max(0.0).min(100.0)
}

/// Calculate cyclomatic complexity for a given function
fn calculate_cyclomatic_complexity(node: &syn::ItemFn) -> usize {
    let mut complexity = 1;

    for statement in &node.block.stmts {
        match statement {
            syn::Stmt::Expr(expr) => match expr {
                syn::Expr::Binary(bin_expr) => {
                    complexity += 1;
                }
                _ => {}
            },
            syn::Stmt::If(if_stmt) => {
                complexity += 1;
            }
            _ => {}
        }
    }

    complexity
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_calculate_metrics() {
        let source = r#"
        // This is a test module

        /// This is a doc comment
        pub struct TestStruct {
            pub field1: i32,
            field2: String,
        }

        impl TestStruct {
            pub fn new() -> Self {
                Self {
                    field1: 42,
                    field2: String::new(),
                }
            }

            pub fn do_something(&self, x: i32) -> i32 {
                if x > 0 {
                    x * 2
                } else {
                    x + 1
                }
            }
        }

        #[test]
        fn test_do_something() {
            let s = TestStruct::new();
            assert_eq!(s.do_something(2), 4);
        }
        "#;

        let syntax_tree: syn::File = syn::parse_str(source).unwrap();
        let metrics = calculate_metrics(&syntax_tree, source);

        assert_eq!(metrics.struct_count, 1);
        assert_eq!(metrics.function_count, 2); // new() and do_something()
        assert_eq!(metrics.test_function_count, 1);
        assert!(metrics.doc_comment_count > 0);
        assert_eq!(metrics.public_item_count, 2); // pub struct and pub field1
        assert_eq!(metrics.private_item_count, 1); // field2
        assert!(metrics.average_cyclomatic_complexity > 1.0);
        assert!(metrics.halstead.volume > 0.0);
    }
}
