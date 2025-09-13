use std::collections::HashMap;

use syn::{visit, ItemUse, TypePath};

use super::super::types::{ArchitecturalFinding, CodeLocation, Severity};
use super::ArchitecturalVisitor;

/// Visitor that checks for violations of architectural layer dependencies
pub struct LayerDependencyVisitor<'a> {
    analyzer:      &'a ArchitecturalAnalyzer,
    current_layer: String,
    allowed_deps:  HashMap<String, Vec<String>>,
    violations:    Vec<ArchitecturalFinding>,
}

impl<'a> LayerDependencyVisitor<'a> {
    /// Create a new LayerDependencyVisitor
    pub fn new(analyzer: &'a ArchitecturalAnalyzer, current_layer: &str) -> Self {
        // Convert allowed_deps to use String for owned values
        let allowed_deps = analyzer
            .allowed_layer_dependencies
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect();

        Self {
            analyzer,
            current_layer: current_layer.to_string(),
            allowed_deps,
            violations: Vec::new(),
        }
    }

    /// Check if a dependency between layers is allowed
    fn check_dependency(&mut self, dep_type: &str, dep_path: &str, location: CodeLocation) {
        // Skip standard library and external dependencies
        if self.is_std_or_external(dep_path) {
            return;
        }

        // Find the most specific matching layer for the dependency
        if let Some((dep_layer, _)) = self.find_matching_layer(dep_path) {
            // Check if this dependency is allowed
            if !self.is_dependency_allowed(dep_layer) {
                self.record_violation(dep_type, dep_layer, location);
            }
        } else {
            // If we can't determine the layer, record it as an unknown dependency
            self.record_unknown_dependency(dep_path, location);
        }
    }

    /// Check if a path belongs to std or external crates
    fn is_std_or_external(&self, path: &str) -> bool {
        // Check for standard library paths
        if path.starts_with("std::") || path.starts_with("core::") {
            return true;
        }

        // Check for external crates (not in our codebase)
        if !path.starts_with("crate::") && !path.starts_with(&format!("{}::", self.analyzer.crate_name)) {
            return true;
        }

        false
    }

    /// Find the most specific matching layer for a given path
    fn find_matching_layer(&self, path: &str) -> Option<(&str, usize)> {
        let mut best_match = None;
        let mut best_score = 0;

        // Check each layer to find the best match
        for layer in self.allowed_deps.keys() {
            let layer_path = format!("crate::{}::", layer);
            if path.starts_with(&layer_path) {
                // The longer the match, the more specific it is
                let score = layer_path.len();
                if score > best_score {
                    best_score = score;
                    best_match = Some((layer.as_str(), score));
                }
            }
        }

        best_match
    }

    /// Check if a dependency is allowed based on the current layer's allowed dependencies
    fn is_dependency_allowed(&self, dep_layer: &str) -> bool {
        // A layer can always depend on itself
        if dep_layer == self.current_layer {
            return true;
        }

        // Check if the dependency is in the allowed list for the current layer
        self.allowed_deps
            .get(&self.current_layer)
            .map_or(false, |allowed| allowed.contains(&dep_layer.to_string()))
    }

    /// Record a dependency violation
    fn record_violation(&mut self, dep_type: &str, dep_layer: &str, location: CodeLocation) {
        self.violations.push(ArchitecturalFinding {
            id: format!("layer_violation_{}_{}", self.current_layer, dep_layer),
            message: format!(
                "Layer '{}' is not allowed to depend on layer '{}' via {}",
                self.current_layer, dep_layer, dep_type
            ),
            severity: Severity::Warning,
            location,
            suggestion: Some("Consider refactoring to respect the architectural boundaries.".to_string()),
            confidence: 0.9,
            rule_id: "ARCH_LAYER_VIOLATION".to_string(),
        });
    }

    /// Record an unknown dependency for potential future categorization
    fn record_unknown_dependency(&mut self, path: &str, location: CodeLocation) {
        self.violations.push(ArchitecturalFinding {
            id: format!("unknown_dependency_{}", path.replace("::", "_")),
            message: format!(
                "Unknown dependency '{}' in layer '{}'. Please categorize this dependency.",
                path, self.current_layer
            ),
            severity: Severity::Info,
            location,
            suggestion: Some("Add this path to the appropriate layer or update the layer configuration.".to_string()),
            confidence: 0.7,
            rule_id: "ARCH_UNKNOWN_DEPENDENCY".to_string(),
        });
    }
}

impl<'a> visit::Visit<'a> for LayerDependencyVisitor<'a> {
    fn visit_item_use(&mut self, i: &'a ItemUse) {
        // Record use statements as potential layer violations
        if let Some(use_path) = i.path() {
            let path_str = path_to_string(use_path);
            let location = CodeLocation {
                file_path: String::new(), // Will be filled in by the analyzer
                line:      i.span().start().line as u32,
                column:    i.span().start().column as u32,
            };
            self.check_dependency("use statement", &path_str, location);
        }

        // Continue visiting child nodes
        visit::visit_item_use(self, i);
    }

    fn visit_type_path(&mut self, ty: &'a TypePath) {
        // Record type references as potential layer violations
        let path_str = path_to_string(&ty.path);
        let location = CodeLocation {
            file_path: String::new(), // Will be filled in by the analyzer
            line:      ty.span().start().line as u32,
            column:    ty.span().start().column as u32,
        };
        self.check_dependency("type reference", &path_str, location);

        // Continue visiting child nodes
        visit::visit_type_path(self, ty);
    }
}

/// Helper function to convert a path to a string
fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    fn create_test_analyzer() -> ArchitecturalAnalyzer {
        let mut analyzer = ArchitecturalAnalyzer::new();

        // Set up test layer dependencies
        analyzer
            .allowed_layer_dependencies
            .insert("domain".to_string(), vec!["core".to_string()]);

        analyzer
            .allowed_layer_dependencies
            .insert("application".to_string(), vec![
                "domain".to_string(),
                "core".to_string(),
            ]);

        analyzer
            .allowed_layer_dependencies
            .insert("infrastructure".to_string(), vec![
                "application".to_string(),
                "domain".to_string(),
                "core".to_string(),
            ]);

        analyzer
    }

    #[test]
    fn test_allowed_dependency() {
        let analyzer = create_test_analyzer();
        let mut visitor = LayerDependencyVisitor::new(&analyzer, "application");

        // This should be allowed (application -> domain)
        let ty: TypePath = parse_quote!(crate::domain::some_type);
        visitor.visit_type_path(&ty);

        assert!(visitor.violations.is_empty());
    }

    #[test]
    fn test_violating_dependency() {
        let analyzer = create_test_analyzer();
        let mut visitor = LayerDependencyVisitor::new(&analyzer, "domain");

        // This should be a violation (domain should not depend on application)
        let ty: TypePath = parse_quote!(crate::application::some_type);
        visitor.visit_type_path(&ty);

        assert_eq!(visitor.violations.len(), 1);
        assert!(visitor.violations[0]
            .message
            .contains("not allowed to depend"));
    }

    #[test]
    fn test_unknown_dependency() {
        let analyzer = create_test_analyzer();
        let mut visitor = LayerDependencyVisitor::new(&analyzer, "domain");

        // This should be flagged as an unknown dependency
        let ty: TypePath = parse_quote!(crate::unknown::module::SomeType);
        visitor.visit_type_path(&ty);

        assert_eq!(visitor.violations.len(), 1);
        assert!(visitor.violations[0].message.contains("Unknown dependency"));
    }
}
