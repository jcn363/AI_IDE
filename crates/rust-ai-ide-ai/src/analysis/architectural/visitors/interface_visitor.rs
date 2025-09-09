use syn::{visit, ItemTrait};
use super::*;

/// Represents an interface segregation violation
#[derive(Debug)]
pub struct InterfaceViolation {
    pub message: String,
    pub location: CodeLocation,
}

/// Visitor that analyzes interface-related architectural issues
pub struct InterfaceVisitor<'a> {
    analyzer: &'a ArchitecturalAnalyzer,
    violations: Vec<InterfaceViolation>,
}

impl<'a> InterfaceVisitor<'a> {
    /// Create a new InterfaceVisitor
    pub fn new(analyzer: &'a ArchitecturalAnalyzer) -> Self {
        Self {
            analyzer,
            violations: Vec::new(),
        }
    }
    
    /// Check if a trait has too many methods
    fn check_trait_size(&mut self, item: &ItemTrait) {
        let method_count = item.items.len();
        if method_count > self.analyzer.max_trait_methods {
            self.violations.push(InterfaceViolation {
                message: format!(
                    "Trait '{}' has too many methods ({} > {})",
                    item.ident, method_count, self.analyzer.max_trait_methods
                ),
                location: CodeLocation::from_span(&item.ident.span()),
            });
        }
    }
}

impl<'a> ArchitecturalVisitor for InterfaceVisitor<'a> {
    fn analyze(&mut self, ast: &File) -> Vec<ArchitecturalFinding> {
        self.violations.clear();
        self.visit_file(ast);
        
        self.violations.drain(..).map(|violation| {
            self.create_finding(
                "interface-segregation-violation",
                format!("Interface Segregation Principle violation: {}", violation.message),
                violation.location.line,
                Some("Split the trait into smaller, more focused interfaces that each serve a specific purpose."),
                Severity::Warning,
                0.7,
                "ARCH005",
            )
        }).collect()
    }
    
    fn analyzer(&self) -> &ArchitecturalAnalyzer {
        self.analyzer
    }
}

impl<'a> visit::Visit<'a> for InterfaceVisitor<'a> {
    fn visit_item_trait(&mut self, node: &'a ItemTrait) {
        self.check_trait_size(node);
        visit::visit_item_trait(self, node);
    }
}
