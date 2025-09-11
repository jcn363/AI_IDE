//! Base Visitor trait for AI Security Analyzer
//!
//! This module provides a base trait that standardizes the common functionality
//! across different security analysis visitors in the AI Security Analyzer.
//! It eliminates duplicate visit_file() implementations and provides consistent
//! behavior for collecting security issues.

use crate::security::*;
use syn::visit::Visit;

/// Base trait for all security analysis visitors
/// This trait provides common functionality and ensures consistent behavior
pub trait BaseSecurityVisitor<'ast> {
    /// Get the collected security issues
    fn get_issues(&self) -> &Vec<SecurityIssue>;

    /// Get mutable reference to issues for internal modification
    fn get_issues_mut(&mut self) -> &mut Vec<SecurityIssue>;

    /// Get the file path being analyzed
    fn get_file_path(&self) -> &str;

    /// Add a security issue to the collection
    fn add_issue(&mut self, issue: SecurityIssue) {
        self.get_issues_mut().push(issue);
    }

    /// Check if this visitor is analyzing library code
    fn is_library_code(&self) -> bool {
        let file_path = self.get_file_path();
        file_path.contains("/src/lib.rs")
            || file_path.contains("/src/")
                && !file_path.contains("/examples/")
                && !file_path.contains("/tests/")
    }

    /// Check if this visitor is analyzing test code
    fn is_test_code(&self) -> bool {
        let file_path = self.get_file_path();
        file_path.contains("/tests/") || file_path.contains("test.rs")
    }

    /// Standard visit_file method that dispatches to child nodes
    /// This eliminates the need for each visitor to implement this manually
    fn visit_file_standard<V: Visit<'ast>>(&mut self, visitor: &mut V, node: &'ast syn::File) {
        syn::visit::visit_file(visitor, node);
    }
}

/// Helper macro to implement the BaseSecurityVisitor trait for any visitor type
/// This reduces boilerplate code and ensures consistency
#[macro_export]
macro_rules! impl_base_visitor {
    ($visitor_type:ty) => {
        impl<'ast> BaseSecurityVisitor<'ast> for $visitor_type {
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
    };
}

/// Mixin for creating standard security issue builders
/// This provides common patterns for creating SecurityIssue instances
pub trait SecurityIssueBuilder {
    /// Create a security issue with standard field initialization
    fn build_issue(
        category: SecurityCategory,
        severity: SecuritySeverity,
        title: String,
        description: String,
        file_path: &str,
        remediation: String,
        confidence: f32,
        cwe_id: Option<u32>,
    ) -> SecurityIssue {
        SecurityIssue {
            category,
            severity,
            title,
            description,
            file_path: file_path.to_string(),
            line_number: None,
            column: None,
            code_snippet: None,
            remediation,
            confidence,
            cwe_id,
        }
    }

    /// Create a security issue with location information
    fn build_issue_with_location(
        category: SecurityCategory,
        severity: SecuritySeverity,
        title: String,
        description: String,
        file_path: &str,
        line_number: Option<usize>,
        column: Option<usize>,
        code_snippet: Option<String>,
        remediation: String,
        confidence: f32,
        cwe_id: Option<u32>,
    ) -> SecurityIssue {
        SecurityIssue {
            category,
            severity,
            title,
            description,
            file_path: file_path.to_string(),
            line_number,
            column,
            code_snippet,
            remediation,
            confidence,
            cwe_id,
        }
    }
}

// Blanket implement SecurityIssueBuilder for all types that can build issues
impl<T> SecurityIssueBuilder for T {}
