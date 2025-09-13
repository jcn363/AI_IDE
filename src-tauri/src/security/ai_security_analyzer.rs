//! AI Security Analyzer
//!
//! This module provides comprehensive security analysis for Rust codebases,
//! detecting various security vulnerabilities, unsafe patterns, and potential
//! attack vectors through static code analysis.

// Sub-modules
pub mod ai_visitor_base;
pub mod integration;
pub mod security_rules;
pub mod types;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use syn::{
    spanned::Spanned, visit::Visit, Expr, ExprCall, ExprMacro, ExprMethodCall, ExprPath, File, Lit,
    LitStr, Span,
};
use walkdir;

// Re-export types and implementations
pub use ai_visitor_base::*;
pub use integration::*;
pub use security_rules::*;
pub use types::*;
// Re-export base visitor macro for use in this module
pub use ai_visitor_base::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCategory {
    HardcodedSecrets,
    UnsafeCode,
    SqlInjection,
    PathTraversal,
    InsecureRandom,
    MemorySafety,
    CryptographicIssues,
    InputValidation,
    Dependencies,
    CommandInjection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub category: SecurityCategory,
    pub severity: SecuritySeverity,
    pub title: String,
    pub description: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub column: Option<usize>,
    pub code_snippet: Option<String>,
    pub remediation: String,
    pub confidence: f32, // 0.0 to 1.0
    pub cwe_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysisResult {
    pub issues: Vec<SecurityIssue>,
    pub summary: SecuritySummary,
    pub dependency_vulnerabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub total_issues: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub overall_score: f32, // 0.0 to 100.0
}

pub struct AISecurityAnalyzer {
    secret_patterns: Vec<SecretPattern>,
    security_rules: Vec<SecurityRule>,
}

#[derive(Debug, Clone)]
struct SecretPattern {
    name: String,
    pattern: Regex,
    severity: SecuritySeverity,
    confidence: f32,
}

#[derive(Debug, Clone)]
struct SecurityRule {
    name: String,
    category: SecurityCategory,
    pattern: Regex,
    severity: SecuritySeverity,
    description: String,
    remediation: String,
    cwe_id: Option<u32>,
}

// Manually implement BaseSecurityVisitor trait for all visitor structs
impl<'ast> BaseSecurityVisitor<'ast> for UnsafeCodeVisitor {
    fn get_issues(&self) -> &Vec<SecurityIssue> {
        &self.issues
    }

    fn get_issues_mut(&mut self) -> &mut Vec<SecurityIssue> {
        &mut self.issues
    }

    fn get_file_path(&self) -> &str {
        &self.file_path
    }
}

impl<'ast> BaseSecurityVisitor<'ast> for PanicUnwrapVisitor {
    fn get_issues(&self) -> &Vec<SecurityIssue> {
        &self.issues
    }

    fn get_issues_mut(&mut self) -> &mut Vec<SecurityIssue> {
        &mut self.issues
    }

    fn get_file_path(&self) -> &str {
        &self.file_path
    }
}

impl<'ast> BaseSecurityVisitor<'ast> for CommandInjectionVisitor {
    fn get_issues(&self) -> &Vec<SecurityIssue> {
        &self.issues
    }

    fn get_issues_mut(&mut self) -> &mut Vec<SecurityIssue> {
        &mut self.issues
    }

    fn get_file_path(&self) -> &str {
        &self.file_path
    }
}

impl<'ast> BaseSecurityVisitor<'ast> for ArithmeticOverflowVisitor {
    fn get_issues(&self) -> &Vec<SecurityIssue> {
        &self.issues
    }

    fn get_issues_mut(&mut self) -> &mut Vec<SecurityIssue> {
        &mut self.issues
    }

    fn get_file_path(&self) -> &str {
        &self.file_path
    }
}

impl<'ast> BaseSecurityVisitor<'ast> for ComplexitySecurityVisitor {
    fn get_issues(&self) -> &Vec<SecurityIssue> {
        &self.issues
    }

    fn get_issues_mut(&mut self) -> &mut Vec<SecurityIssue> {
        &mut self.issues
    }

    fn get_file_path(&self) -> &str {
        &self.file_path
    }
}

struct UnsafeCodeVisitor {
    issues: Vec<SecurityIssue>,
    file_path: String,
}

impl UnsafeCodeVisitor {
    fn new(file_path: String) -> Self {
        Self {
            issues: Vec::new(),
            file_path,
        }
    }
}

impl<'ast> Visit<'ast> for UnsafeCodeVisitor {
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        // Check for unsafe blocks
        if let Expr::Unsafe(block) = node {
            self.issues.push(SecurityIssue {
                category: SecurityCategory::UnsafeCode,
                severity: SecuritySeverity::High,
                title: "Unsafe block usage".to_string(),
                description: "Unsafe code block detected - manually verify safety".to_string(),
                file_path: self.file_path.clone(),
                line_number: Some(block.unsafe_token.span().line as usize),
                column: Some(block.unsafe_token.span().column as usize),
                code_snippet: Some(format!("{:?}", block)),
                remediation: "Ensure all unsafe code is properly documented and justified"
                    .to_string(),
                confidence: 0.9,
                cwe_id: Some(628), // CWE-628: Function Call with Incorrectly Specified Arguments
            });
        }

        // Continue visiting child nodes
        syn::visit::visit_expr(self, node);
    }
}

/// Visitor for detecting panic/unwrap calls in inappropriate contexts
struct PanicUnwrapVisitor {
    issues: Vec<SecurityIssue>,
    file_path: String,
    in_library_code: bool,
    in_test_code: bool,
}

impl PanicUnwrapVisitor {
    fn new(file_path: String) -> Self {
        let in_library_code = file_path.contains("/src/lib.rs")
            || file_path.contains("/src/")
                && !file_path.contains("/examples/")
                && !file_path.contains("/tests/");
        let in_test_code = file_path.contains("/tests/") || file_path.contains("test.rs");

        Self {
            issues: Vec::new(),
            file_path,
            in_library_code,
            in_test_code,
        }
    }
}

impl<'ast> Visit<'ast> for PanicUnwrapVisitor {
    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        if let Some(segment) = node.mac.path.segments.last() {
            let macro_name = segment.ident.to_string();

            match macro_name.as_str() {
                "panic" => {
                    if self.in_library_code {
                        self.issues.push(SecurityIssue {
                            category: SecurityCategory::MemorySafety,
                            severity: SecuritySeverity::High,
                            title: "Panic in library code".to_string(),
                            description: "panic! macro used in library code can cause denial of service".to_string(),
                            file_path: self.file_path.clone(),
                            line_number: None,
                            column: None,
                            code_snippet: None,
                            remediation: "Return Result<T, E> instead of panicking. Use panic! only in unrecoverable situations".to_string(),
                            confidence: 0.9,
                            cwe_id: Some(248), // CWE-248: Uncaught Exception
                        });
                    }
                }
                "unreachable" => {
                    if self.in_library_code {
                        self.issues.push(SecurityIssue {
                            category: SecurityCategory::MemorySafety,
                            severity: SecuritySeverity::Medium,
                            title: "Unreachable code in library".to_string(),
                            description: "unreachable! macro in library code may indicate logic errors".to_string(),
                            file_path: self.file_path.clone(),
                            line_number: None,
                            column: None,
                            code_snippet: None,
                            remediation: "Review logic to ensure unreachable code is truly unreachable or handle the case properly".to_string(),
                            confidence: 0.7,
                            cwe_id: Some(561), // CWE-561: Dead Code
                        });
                    }
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method_name = node.method.to_string();

        match method_name.as_str() {
            "unwrap" | "expect" => {
                if self.in_library_code && !self.in_test_code {
                    let severity = if method_name == "unwrap" {
                        SecuritySeverity::High
                    } else {
                        SecuritySeverity::Medium
                    };

                    self.issues.push(SecurityIssue {
                        category: SecurityCategory::MemorySafety,
                        severity,
                        title: format!("Use of {} in library code", method_name),
                        description: format!("{} can cause panics and denial of service in library code", method_name),
                        file_path: self.file_path.clone(),
                        line_number: None,
                        column: None,
                        code_snippet: None,
                        remediation: "Use pattern matching, if let, or proper error handling instead of unwrap/expect".to_string(),
                        confidence: 0.85,
                        cwe_id: Some(248), // CWE-248: Uncaught Exception
                    });
                }
            }
            "unwrap_unchecked" => {
                self.issues.push(SecurityIssue {
                    category: SecurityCategory::MemorySafety,
                    severity: SecuritySeverity::Critical,
                    title: "Use of unwrap_unchecked".to_string(),
                    description:
                        "unwrap_unchecked bypasses safety checks and can cause undefined behavior"
                            .to_string(),
                    file_path: self.file_path.clone(),
                    line_number: None,
                    column: None,
                    code_snippet: None,
                    remediation: "Use safe alternatives or ensure the preconditions are always met"
                        .to_string(),
                    confidence: 0.95,
                    cwe_id: Some(119), // CWE-119: Improper Restriction of Operations within the Bounds of a Memory Buffer
                });
            }
            _ => {}
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Visitor for detecting command injection vulnerabilities
struct CommandInjectionVisitor {
    issues: Vec<SecurityIssue>,
    file_path: String,
}

impl CommandInjectionVisitor {
    fn new(file_path: String) -> Self {
        Self {
            issues: Vec::new(),
            file_path,
        }
    }
}

impl<'ast> Visit<'ast> for CommandInjectionVisitor {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        // Check for std::process::Command::new() calls
        if let Expr::Path(expr_path) = &*node.func {
            if let Some(ident) = expr_path.path.get_ident() {
                if ident == "Command" {
                    self.analyze_command_usage(node);
                }
            }
        }

        // Check for dangerous functions like std::process::Command::output()
        if let Expr::Path(expr_path) = &*node.func {
            if let Some(segment) = expr_path.path.segments.last() {
                if segment.ident == "output"
                    || segment.ident == "spawn"
                    || segment.ident == "status"
                {
                    // This is an ExprCall, so we can analyze it
                    // We need to create a separate analysis for method calls on expressions
                    // For now, skip this since it requires more complex AST analysis
                }
            }
        }

        // Continue visiting child nodes
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        // Check for Command::new() calls
        if let Expr::Path(expr_path) = &*node.receiver {
            if let Some(segment) = expr_path.path.segments.last() {
                if segment.ident == "Command" {
                    self.analyze_command_usage_method(node);
                }
            }
        }

        // Check for dangerous method calls on Command objects
        if let Expr::Path(expr_path) = &*node.receiver {
            if let Some(segment) = expr_path.path.segments.last() {
                if segment.ident == "arg" || segment.ident == "args" {
                    self.check_command_arguments(node);
                }
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

impl CommandInjectionVisitor {
    fn analyze_command_usage(&mut self, node: &syn::ExprCall) {
        // Check if command arguments contain user input
        for arg in &node.args {
            if let Expr::Lit(lit) = arg {
                if let Lit::Str(lit_str) = &lit.lit {
                    if self.is_potentially_unsafe_input(&lit_str.value()) {
                        let issue = SecurityIssue {
                            category: SecurityCategory::CommandInjection,
                            severity: SecuritySeverity::High,
                            title: "Potential command injection vulnerability".to_string(),
                            description: "Command contains potentially unsafe user input".to_string(),
                            file_path: self.file_path.clone(),
                            line_number: Some(lit_str.span().line as usize),
                            column: Some(lit_str.span().column as usize),
                            code_snippet: Some(format!("{:?}", node)),
                            remediation: "Use std::process::Command with explicit arguments instead of string interpolation".to_string(),
                            confidence: 0.8,
                            cwe_id: Some(78),
                        };
                        self.issues.push(issue);
                    }
                }
            }
        }
    }

    fn analyze_command_usage_method(&mut self, node: &syn::ExprMethodCall) {
        // Check if method call contains user input
        for arg in &node.args {
            if let Expr::Lit(lit) = arg {
                if let Lit::Str(lit_str) = &lit.lit {
                    if self.is_potentially_unsafe_input(&lit_str.value()) {
                        let issue = SecurityIssue {
                            category: SecurityCategory::CommandInjection,
                            severity: SecuritySeverity::High,
                            title: "Potential command injection vulnerability".to_string(),
                            description: "Command contains potentially unsafe user input".to_string(),
                            file_path: self.file_path.clone(),
                            line_number: Some(lit_str.span().line as usize),
                            column: Some(lit_str.span().column as usize),
                            code_snippet: Some(format!("{:?}", node)),
                            remediation: "Use std::process::Command with explicit arguments instead of string interpolation".to_string(),
                            confidence: 0.8,
                            cwe_id: Some(78),
                        };
                        self.issues.push(issue);
                    }
                }
            }
        }
    }

    fn check_command_arguments(&mut self, node: &syn::ExprMethodCall) {
        // Check for dangerous method calls on Command objects
        if let Expr::Path(expr_path) = &*node.receiver {
            if let Some(segment) = expr_path.path.segments.last() {
                if segment.ident == "arg" || segment.ident == "args" {
                    for arg in &node.args {
                        if let Expr::Lit(lit) = arg {
                            if let Lit::Str(lit_str) = &lit.lit {
                                if self.is_potentially_unsafe_input(&lit_str.value()) {
                                    let issue = SecurityIssue {
                                        category: SecurityCategory::CommandInjection,
                                        severity: SecuritySeverity::High,
                                        title: "Potential command injection in argument".to_string(),
                                        description: "Command argument contains potentially unsafe user input".to_string(),
                                        file_path: self.file_path.clone(),
                                        line_number: Some(lit_str.span().line as usize),
                                        column: Some(lit_str.span().column as usize),
                                        code_snippet: Some(format!("{:?}", node)),
                                        remediation: "Validate and sanitize all command arguments".to_string(),
                                        confidence: 0.9,
                                        cwe_id: Some(78),
                                    };
                                    self.issues.push(issue);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check if a string contains potentially dangerous shell metacharacters
    fn is_potentially_unsafe_input(&self, input: &str) -> bool {
        let dangerous_patterns = [
            "&&", "||", ";", "|", "`", "$", "(", ")", "{", "}", "<", ">", "*", "?", "[", "]", "..",
            "/", "\\",
        ];

        dangerous_patterns.iter().any(|&p| input.contains(p))
    }
}

/// Placeholder visitor for arithmetic overflow analysis - TODO: implement
struct ArithmeticOverflowVisitor {
    issues: Vec<SecurityIssue>,
    file_path: String,
}

impl ArithmeticOverflowVisitor {
    fn new(file_path: String) -> Self {
        Self {
            issues: Vec::new(),
            file_path,
        }
    }
}

impl<'ast> Visit<'ast> for ArithmeticOverflowVisitor {
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        // TODO: Implement arithmetic overflow detection
        // For now, just delegate to visit child expressions
        syn::visit::visit_expr(self, node);
    }
}

/// Placeholder visitor for complexity security analysis - TODO: implement
struct ComplexitySecurityVisitor {
    issues: Vec<SecurityIssue>,
    file_path: String,
}

impl ComplexitySecurityVisitor {
    fn new(file_path: String) -> Self {
        Self {
            issues: Vec::new(),
            file_path,
        }
    }
}

impl<'ast> Visit<'ast> for ComplexitySecurityVisitor {
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        // TODO: Implement complexity security analysis
        // For now, just delegate to visit child expressions
        syn::visit::visit_expr(self, node);
    }
}

impl AISecurityAnalyzer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let secret_patterns = SecurityRulesFactory::initialize_secret_patterns()?;
        let security_rules = SecurityRulesFactory::initialize_security_rules()?;

        Ok(Self {
            secret_patterns,
            security_rules,
        })
    }

    // Security rules and patterns are now initialized via SecurityRulesFactory

    pub fn analyze_code(&self, code: &str, file_path: &str) -> SecurityAnalysisResult {
        let mut issues = Vec::new();

        // Detect hardcoded secrets
        issues.extend(self.detect_hardcoded_secrets(code, file_path));

        // Apply security rules
        issues.extend(self.apply_security_rules(code, file_path));

        // Parse and analyze AST for unsafe code and other patterns
        if let Ok(syntax_tree) = syn::parse_file(code) {
            issues.extend(self.analyze_ast(&syntax_tree, file_path));

            // NEW: Advanced AST analysis with new visitors
            issues.extend(self.analyze_panic_unwrap(&syntax_tree, file_path));
            issues.extend(self.analyze_command_injection(&syntax_tree, file_path));
            // TODO: Implement arithmetic_overflow and complexity_security visitors
            // issues.extend(self.analyze_arithmetic_overflow(&syntax_tree, file_path));
            // issues.extend(self.analyze_complexity_security(&syntax_tree, file_path));
        }

        // Get dependency vulnerabilities (would need manifest path)
        let dependency_vulnerabilities = Vec::new(); // Placeholder

        let summary = self.calculate_security_summary(&issues);

        SecurityAnalysisResult {
            issues,
            summary,
            dependency_vulnerabilities,
        }
    }

    pub fn analyze_workspace(
        &self,
        workspace_path: &Path,
    ) -> Result<SecurityAnalysisResult, Box<dyn std::error::Error>> {
        let mut all_issues = Vec::new();

        // Walk through all Rust files in the workspace
        for entry in walkdir::WalkDir::new(workspace_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            if let Ok(code) = std::fs::read_to_string(entry.path()) {
                let file_path = entry.path().to_string_lossy().to_string();
                let analysis = self.analyze_code(&code, &file_path);
                all_issues.extend(analysis.issues);
            }
        }

        let summary = self.calculate_security_summary(&all_issues);

        Ok(SecurityAnalysisResult {
            issues: all_issues,
            summary,
            dependency_vulnerabilities,
        })
    }

    fn detect_hardcoded_secrets(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for pattern in &self.secret_patterns {
            for mat in pattern.pattern.find_iter(code) {
                let line_number = code[..mat.start()].lines().count();

                issues.push(SecurityIssue {
                    category: SecurityCategory::HardcodedSecrets,
                    severity: pattern.severity.clone(),
                    title: format!("Hardcoded {} detected", pattern.name),
                    description: format!(
                        "A {} appears to be hardcoded in the source code",
                        pattern.name
                    ),
                    file_path: file_path.to_string(),
                    line_number: Some(line_number),
                    column: Some(mat.start()),
                    code_snippet: Some(mat.as_str().to_string()),
                    remediation:
                        "Move sensitive data to environment variables or secure configuration files"
                            .to_string(),
                    confidence: pattern.confidence,
                    cwe_id: Some(798), // CWE-798: Use of Hard-coded Credentials
                });
            }
        }

        issues
    }

    fn apply_security_rules(&self, code: &str, file_path: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for rule in &self.security_rules {
            for mat in rule.pattern.find_iter(code) {
                let line_number = code[..mat.start()].lines().count();

                issues.push(SecurityIssue {
                    category: rule.category.clone(),
                    severity: rule.severity.clone(),
                    title: rule.name.clone(),
                    description: rule.description.clone(),
                    file_path: file_path.to_string(),
                    line_number: Some(line_number),
                    column: Some(mat.start()),
                    code_snippet: Some(mat.as_str().to_string()),
                    remediation: rule.remediation.clone(),
                    confidence: 0.8,
                    cwe_id: rule.cwe_id,
                });
            }
        }

        issues
    }

    fn analyze_ast(&self, syntax_tree: &File, file_path: &str) -> Vec<SecurityIssue> {
        let mut visitor = UnsafeCodeVisitor::new(file_path.to_string());
        syn::visit::visit_file(&mut visitor, syntax_tree);
        visitor.get_issues().clone()
    }

    fn analyze_panic_unwrap(&self, syntax_tree: &File, file_path: &str) -> Vec<SecurityIssue> {
        let mut visitor = PanicUnwrapVisitor::new(file_path.to_string());
        syn::visit::visit_file(&mut visitor, syntax_tree);
        visitor.get_issues().clone()
    }

    fn analyze_command_injection(&self, syntax_tree: &File, file_path: &str) -> Vec<SecurityIssue> {
        let mut visitor = CommandInjectionVisitor::new(file_path.to_string());
        syn::visit::visit_file(&mut visitor, syntax_tree);
        visitor.get_issues().clone()
    }

    fn analyze_arithmetic_overflow(
        &self,
        syntax_tree: &File,
        file_path: &str,
    ) -> Vec<SecurityIssue> {
        let mut visitor = ArithmeticOverflowVisitor::new(file_path.to_string());
        syn::visit::visit_file(&mut visitor, syntax_tree);
        visitor.get_issues().clone()
    }

    fn analyze_complexity_security(
        &self,
        syntax_tree: &File,
        file_path: &str,
    ) -> Vec<SecurityIssue> {
        let mut visitor = ComplexitySecurityVisitor::new(file_path.to_string());
        syn::visit::visit_file(&mut visitor, syntax_tree);
        visitor.get_issues().clone()
    }

    fn calculate_security_summary(&self, issues: &[SecurityIssue]) -> SecuritySummary {
        let total_issues = issues.len();

        let (critical_count, high_count, medium_count, low_count, info_count) = issues.iter().fold(
            (0, 0, 0, 0, 0),
            |(crit, high, med, low, info), issue| match issue.severity {
                SecuritySeverity::Critical => (crit + 1, high, med, low, info),
                SecuritySeverity::High => (crit, high + 1, med, low, info),
                SecuritySeverity::Medium => (crit, high, med + 1, low, info),
                SecuritySeverity::Low => (crit, high, med, low + 1, info),
                SecuritySeverity::Info => (crit, high, med, low, info + 1),
            },
        );

        // Calculate overall security score (0-100, higher is better)
        let overall_score = if total_issues == 0 {
            100.0
        } else {
            let weighted_score =
                (critical_count * 10 + high_count * 5 + medium_count * 2 + low_count) as f32;
            let max_possible_score = total_issues as f32 * 10.0;
            ((max_possible_score - weighted_score) / max_possible_score * 100.0).max(0.0)
        };

        SecuritySummary {
            total_issues,
            critical_count,
            high_count,
            medium_count,
            low_count,
            info_count,
            overall_score,
        }
    }

    pub fn add_custom_rule(&mut self, rule: SecurityRule) {
        self.security_rules.push(rule);
    }

    pub fn add_custom_secret_pattern(&mut self, pattern: SecretPattern) {
        self.secret_patterns.push(pattern);
    }

    /// Convert SecurityIssue to analysis engine format for consistency
    pub fn to_analysis_finding(&self, issue: &SecurityIssue) -> AnalysisEngineFinding {
        AnalysisEngineFinding {
            message: issue.description.clone(),
            severity: self.map_security_severity(&issue.severity),
            category: AnalysisEngineCategory::Security,
            range: AnalysisEngineRange {
                start_line: issue.line_number.unwrap_or(1) as u32,
                start_column: issue.column.unwrap_or(1) as u32,
                end_line: issue.line_number.unwrap_or(1) as u32,
                end_column: (issue.column.unwrap_or(1) + 10) as u32,
            },
            suggestion: Some(issue.remediation.clone()),
            confidence: issue.confidence,
            rule_id: issue.title.clone(),
            cwe_id: issue.cwe_id,
        }
    }

    fn map_security_severity(&self, severity: &SecuritySeverity) -> AnalysisEngineSeverity {
        match severity {
            SecuritySeverity::Critical => AnalysisEngineSeverity::Error,
            SecuritySeverity::High => AnalysisEngineSeverity::Warning,
            SecuritySeverity::Medium => AnalysisEngineSeverity::Info,
            SecuritySeverity::Low => AnalysisEngineSeverity::Hint,
            SecuritySeverity::Info => AnalysisEngineSeverity::Hint,
        }
    }

    pub fn get_remediation_suggestions(&self, issue: &SecurityIssue) -> Vec<String> {
        let base_suggestions = vec![issue.remediation.clone()];

        let category_suggestions: Vec<&str> = match issue.category {
            SecurityCategory::HardcodedSecrets => vec![
                "Use environment variables with dotenv crate",
                "Consider using a secrets management service like HashiCorp Vault",
                "Use configuration files that are not committed to version control",
            ],
            SecurityCategory::UnsafeCode => vec![
                "Add comprehensive documentation explaining why unsafe is necessary",
                "Minimize the scope of unsafe blocks",
                "Consider using safe abstractions from crates like crossbeam or parking_lot",
            ],
            SecurityCategory::CryptographicIssues => vec![
                "Use the 'ring' crate for cryptographic operations",
                "Consider using 'rustls' for TLS implementations",
                "Use 'argon2' for password hashing",
                "Implement proper key rotation and management",
            ],
            SecurityCategory::InputValidation => vec![
                "Implement input sanitization and validation",
                "Use allowlists instead of blocklists for validation",
                "Consider using validated input types",
            ],
            SecurityCategory::MemorySafety => vec![
                "Use safe Rust alternatives when possible",
                "Add bounds checking for array/vector access",
                "Consider using smart pointers for memory management",
            ],
            SecurityCategory::PathTraversal => vec![
                "Use Path::canonicalize() to resolve paths safely",
                "Implement path validation against allowed directories",
                "Consider using a sandboxed file access approach",
            ],
            SecurityCategory::InsecureRandom => vec![
                "Use OsRng for cryptographically secure random numbers",
                "Consider using the 'rand' crate with secure generators",
                "Avoid predictable random number generation",
            ],
            SecurityCategory::Dependencies => vec![
                "Regularly update dependencies to latest secure versions",
                "Use cargo audit to check for known vulnerabilities",
                "Consider using dependency scanning tools in CI/CD",
            ],
            SecurityCategory::CommandInjection => vec![
                "Use command argument arrays instead of string concatenation",
                "Validate and sanitize all command arguments",
                "Consider using higher-level libraries instead of shell commands",
            ],
            SecurityCategory::SqlInjection => vec![
                "Use parameterized queries or prepared statements",
                "Avoid string concatenation for SQL queries",
                "Consider using an ORM for database access",
            ],
        };

        // Convert all suggestions to owned strings and combine with base suggestions
        let mut result = base_suggestions;
        result.extend(category_suggestions.into_iter().map(|s| s.to_string()));
        result
    }
}

/// Types for integration with analysis engine
#[derive(Debug, Clone)]
pub struct AnalysisEngineFinding {
    pub message: String,
    pub severity: AnalysisEngineSeverity,
    pub category: AnalysisEngineCategory,
    pub range: AnalysisEngineRange,
    pub suggestion: Option<String>,
    pub confidence: f32,
    pub rule_id: String,
    pub cwe_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum AnalysisEngineSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub enum AnalysisEngineCategory {
    Security,
}

#[derive(Debug, Clone)]
pub struct AnalysisEngineRange {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl Default for AISecurityAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to initialize AI Security Analyzer")
    }
}
