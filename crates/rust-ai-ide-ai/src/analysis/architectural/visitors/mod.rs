//! Contains visitors for different types of architectural analysis.

mod complexity_visitor;
mod dependency_visitor;
mod layer_visitor;
mod interface_segregation_visitor;

pub use complexity_visitor::ComplexityVisitor;
pub use dependency_visitor::DependencyVisitor;
pub use layer_visitor::LayerDependencyVisitor;
pub use interface_segregation_visitor::InterfaceSegregationVisitor;

use syn::File;
use super::ArchitecturalAnalyzer;
use super::types::{ArchitecturalFinding, CodeLocation, Severity};

/// Common trait for all architectural visitors
pub trait ArchitecturalVisitor {
    /// Analyze the given syntax tree and return findings
    fn analyze(&mut self, ast: &File) -> Vec<ArchitecturalFinding>;

    /// Get a reference to the analyzer configuration
    fn analyzer(&self) -> &ArchitecturalAnalyzer;

    /// Helper method to create a finding
    fn create_finding(
        &self,
        id: &str,
        message: String,
        line: u32,
        suggestion: Option<&str>,
        severity: Severity,
        confidence: f32,
        rule_id: &str,
    ) -> ArchitecturalFinding {
        ArchitecturalFinding {
            id: id.to_string(),
            message,
            severity,
            location: CodeLocation::new("", line, 0), // File path will be set by the analyzer
            suggestion: suggestion.map(|s| s.to_string()),
            confidence,
            rule_id: rule_id.to_string(),
        }
    }
}
