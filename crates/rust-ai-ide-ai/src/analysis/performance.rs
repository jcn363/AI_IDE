use anyhow::Result;
use serde::{Deserialize, Serialize};
use syn::visit::Visit;
use syn::{Expr, File};

use super::{AnalysisCategory, AnalysisFinding, AnalysisPreferences, Analyzer, Range, Severity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    pub file_path:  String,
    pub line:       usize,
    pub column:     usize,
    pub message:    String,
    pub suggestion: String,
    pub severity:   PerformanceIssueSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceIssueSeverity {
    Low,
    Medium,
    High,
}

/// Performance analyzer that identifies potential performance bottlenecks
#[derive(Debug)]
pub struct PerformanceAnalyzer {
    pub issues:   Vec<PerformanceIssue>,
    current_file: String,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn analyze_file(&mut self, file_path: &str, content: &str) -> Vec<PerformanceIssue> {
        self.issues.clear();

        if let Ok(ast) = syn::parse_file(content) {
            let mut visitor = PerformanceVisitor::new(file_path);
            visitor.visit_file(&ast);
            self.issues = visitor.issues;
        }

        self.issues.clone()
    }
}

impl Analyzer for PerformanceAnalyzer {
    type Finding = AnalysisFinding;

    fn analyze(&self, _ast: &File, code: &str, file_path: &str) -> Result<Vec<Self::Finding>> {
        let mut analyzer = PerformanceAnalyzer::new();
        let issues = analyzer.analyze_file(file_path, code);

        Ok(issues
            .into_iter()
            .map(|issue| {
                let severity = match issue.severity {
                    PerformanceIssueSeverity::High => Severity::Error,
                    PerformanceIssueSeverity::Medium => Severity::Warning,
                    PerformanceIssueSeverity::Low => Severity::Info,
                };

                let range = Range {
                    start_line: issue.line as u32,
                    start_col:  issue.column as u32,
                    end_line:   issue.line as u32,
                    end_col:    issue.column as u32 + 10, // Arbitrary end column
                };

                crate::analysis::AnalysisFinding {
                    message: issue.message,
                    severity,
                    category: AnalysisCategory::Performance,
                    range,
                    suggestion: Some(issue.suggestion),
                    confidence: 0.8,
                    rule_id: "performance".to_string(),
                }
            })
            .collect())
    }

    fn name(&self) -> &'static str {
        "performance_analyzer"
    }

    fn category(&self) -> AnalysisCategory {
        AnalysisCategory::Performance
    }

    fn is_enabled(&self, preferences: &AnalysisPreferences) -> bool {
        preferences.enable_performance
    }
}

struct PerformanceVisitor {
    issues:       Vec<PerformanceIssue>,
    current_file: String,
}

impl PerformanceVisitor {
    fn new(file_path: &str) -> Self {
        Self {
            issues:       Vec::new(),
            current_file: file_path.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for PerformanceVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if let Some(segment) = node.method.path.segments.last() {
            let method_name = segment.ident.to_string();

            // Check for inefficient patterns
            match method_name.as_str() {
                "collect" => self.check_inefficient_collect(node),
                "clone" => self.check_unnecessary_clone(node),
                _ => {}
            }
        }

        // Continue visiting the rest of the AST
        syn::visit::visit_expr_method_call(self, node);
    }
}

impl PerformanceVisitor {
    fn check_inefficient_collect(&mut self, node: &syn::ExprMethodCall) {
        if let Expr::MethodCall(prev_call) = &*node.receiver {
            if let Some(prev_segment) = prev_call.method.path.segments.last() {
                let prev_method = prev_segment.ident.to_string();

                if prev_method == "map" {
                    self.issues.push(PerformanceIssue {
                        file_path:  self.current_file.clone(),
                        line:       node.method.span().line() as usize,
                        column:     node.method.span().column() as usize,
                        message:    "Inefficient iterator chain: map().collect()".to_string(),
                        suggestion: "Consider using filter_map() or another more efficient operation".to_string(),
                        severity:   PerformanceIssueSeverity::Medium,
                    });
                }
            }
        }
    }

    fn check_unnecessary_clone(&mut self, node: &syn::ExprMethodCall) {
        if let Expr::MethodCall(prev_call) = &*node.receiver {
            if let Some(prev_segment) = prev_call.method.path.segments.last() {
                let prev_method = prev_segment.ident.to_string();

                if prev_method == "iter" || prev_method == "into_iter" {
                    self.issues.push(PerformanceIssue {
                        file_path:  self.current_file.clone(),
                        line:       node.method.span().line() as usize,
                        column:     node.method.span().column() as usize,
                        message:    "Unnecessary clone() in iterator chain".to_string(),
                        suggestion: "Consider removing clone() if possible".to_string(),
                        severity:   PerformanceIssueSeverity::Low,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inefficient_collect() {
        let code = r#"
        fn test() {
            let v = vec![1, 2, 3];
            let _: Vec<_> = v.iter().map(|x| x * 2).collect();
        }
        "#;

        let mut analyzer = PerformanceAnalyzer::new();
        let issues = analyzer.analyze_file("test.rs", code);
        assert!(!issues.is_empty());
        assert_eq!(
            issues[0].message,
            "Inefficient iterator chain: map().collect()"
        );
    }

    #[test]
    fn test_unnecessary_clone() {
        let code = r#"
        fn test() {
            let v = vec![1, 2, 3];
            let _: Vec<_> = v.iter().clone().collect();
        }
        "#;

        let mut analyzer = PerformanceAnalyzer::new();
        let issues = analyzer.analyze_file("test.rs", code);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].message, "Unnecessary clone() in iterator chain");
    }
}
