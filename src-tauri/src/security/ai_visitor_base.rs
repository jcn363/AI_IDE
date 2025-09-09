//! Base visitor functionality for security analysis
//!
//! This module provides the base trait and utilities for AST visitors used in
//! security analysis of Rust code.

use super::ai_security_analyzer::SecurityIssue;
use syn::visit::Visit;

/// Base trait for security analysis visitors
pub trait BaseVisitor<'ast>: Visit<'ast> {
    /// Get the collected security issues
    fn get_issues(&mut self) -> &mut Vec<SecurityIssue>;

    /// Add a new security issue
    fn add_issue(&mut self, issue: SecurityIssue) {
        self.get_issues().push(issue);
    }

    /// Get the file path being analyzed
    fn get_file_path(&self) -> &str;

    /// Visit a file with standard AST traversal
    fn visit_file_standard(&mut self, visitor: &mut impl Visit<'ast>, file: &'ast syn::File) {
        visitor.visit_file(file);
    }
}

/// Macro to implement the BaseVisitor trait for a visitor struct
/// This macro automatically implements Visit and provides default implementations
macro_rules! impl_base_visitor {
    ($visitor_type:ident) => {
        impl<'ast> BaseVisitor<'ast> for $visitor_type {
            fn get_issues(&mut self) -> &mut Vec<SecurityIssue> {
                &mut self.issues
            }

            fn get_file_path(&self) -> &str {
                &self.file_path
            }
        }

        // Pass through the Visit implementation if not already implemented
        impl<'ast> Visit<'ast> for $visitor_type {}
    };
}

pub(crate) use impl_base_visitor;

/// Alternative implementation that can be used instead of the macro
pub struct BaseVisitorImpl<T> {
    inner: T,
}

impl<T> BaseVisitorImpl<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}