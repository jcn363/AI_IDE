//! Implementation of the architectural analyzer

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use syn::File;

use crate::analysis::{AnalysisFinding, AnalysisPreferences, Analyzer};

/// Configuration for architectural analysis
#[derive(Debug, Clone)]
#[typetag::serde]
pub struct ArchitecturalAnalyzer {
    /// Maximum allowed cyclomatic complexity
    pub max_cyclomatic_complexity:   u32,
    /// Maximum allowed inheritance depth
    pub max_inheritance_depth:       usize,
    /// Allowed architectural layers
    pub allowed_layers:              Vec<String>,
    /// Enable or disable circular dependency checking
    pub check_circular_dependencies: bool,
    /// Enable or disable layer dependency enforcement
    pub enforce_layer_dependencies:  bool,
    /// Enable or disable dependency inversion checking
    pub check_dependency_inversion:  bool,
    /// Enable or disable interface segregation checking
    pub check_interface_segregation: bool,
    /// Maximum module size in lines of code
    pub max_module_size:             usize,
    /// Maximum number of public items allowed per module
    pub max_public_items:            usize,
}

impl ArchitecturalAnalyzer {
    /// Create a new architectural analyzer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum allowed cyclomatic complexity
    pub fn with_max_cyclomatic_complexity(mut self, value: u32) -> Self {
        self.max_cyclomatic_complexity = value;
        self
    }

    /// Set the maximum allowed inheritance depth
    pub fn with_max_inheritance_depth(mut self, value: usize) -> Self {
        self.max_inheritance_depth = value;
        self
    }

    /// Add an allowed architectural layer
    pub fn with_allowed_layer(mut self, layer: impl Into<String>) -> Self {
        let layer = layer.into();
        if !self.allowed_layers.contains(&layer) {
            self.allowed_layers.push(layer);
        }
        self
    }

    /// Enable or disable circular dependency checking
    pub fn with_circular_dependency_checking(mut self, enabled: bool) -> Self {
        self.check_circular_dependencies = enabled;
        self
    }

    /// Enable or disable layer dependency enforcement
    pub fn with_layer_dependency_enforcement(mut self, enabled: bool) -> Self {
        self.enforce_layer_dependencies = enabled;
        self
    }

    /// Enable or disable dependency inversion checking
    pub fn with_dependency_inversion_checking(mut self, enabled: bool) -> Self {
        self.check_dependency_inversion = enabled;
        self
    }

    /// Enable or disable interface segregation checking
    pub fn with_interface_segregation_checking(mut self, enabled: bool) -> Self {
        self.check_interface_segregation = enabled;
        self
    }

    /// Set the maximum module size in lines of code
    pub fn with_max_module_size(mut self, size: usize) -> Self {
        self.max_module_size = size;
        self
    }

    /// Set the maximum number of public items allowed per module
    pub fn with_max_public_items(mut self, count: usize) -> Self {
        self.max_public_items = count;
        self
    }

    /// Get the architectural layer of a module from its path
    pub fn get_module_layer(&self, file_path: &str) -> &str {
        let path = Path::new(file_path);

        // Try to find the first component that matches an allowed layer
        for component in path.components() {
            if let Some(component_str) = component.as_os_str().to_str() {
                if self.allowed_layers.iter().any(|l| l == component_str) {
                    return component_str;
                }
            }
        }

        // Default to the first allowed layer if no match found
        &self.allowed_layers[0]
    }

    // Private helper methods for analysis
    fn check_circular_dependencies(&self, _ast: &File, _file_path: &str) -> Vec<ArchitecturalFinding> {
        // Implementation for circular dependency checking
        vec![]
    }

    fn check_layer_violations(&self, _ast: &File, _file_path: &str) -> Vec<ArchitecturalFinding> {
        // Implementation for layer violation checking
        vec![]
    }

    fn check_dependency_inversion_violations(&self, _ast: &File, _file_path: &str) -> Vec<ArchitecturalFinding> {
        // Implementation for dependency inversion checking
        vec![]
    }

    fn check_interface_segregation_violations(&self, _ast: &File, _file_path: &str) -> Vec<ArchitecturalFinding> {
        // Implementation for interface segregation checking
        vec![]
    }
}

impl Analyzer for ArchitecturalAnalyzer {
    type Finding = ArchitecturalFinding;

    fn analyze(&self, ast: &File, _code: &str, file_path: &str) -> Vec<Self::Finding> {
        let mut findings = Vec::new();

        // Run all enabled analysis passes
        if self.check_circular_dependencies {
            findings.extend(self.check_circular_dependencies(ast, file_path));
        }

        if self.enforce_layer_dependencies {
            findings.extend(self.check_layer_violations(ast, file_path));
        }

        if self.check_dependency_inversion {
            findings.extend(self.check_dependency_inversion_violations(ast, file_path));
        }

        if self.check_interface_segregation {
            findings.extend(self.check_interface_segregation_violations(ast, file_path));
        }

        findings
    }

    fn is_enabled(&self, _preferences: &AnalysisPreferences) -> bool {
        // This analyzer is always enabled by default
        true
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_analyzer_configuration() {
        let analyzer = ArchitecturalAnalyzer::new()
            .with_max_cyclomatic_complexity(15)
            .with_max_inheritance_depth(2)
            .with_allowed_layer("test_layer");

        assert_eq!(analyzer.max_cyclomatic_complexity, 15);
        assert_eq!(analyzer.max_inheritance_depth, 2);
        assert!(analyzer.allowed_layers.contains(&"test_layer".to_string()));
    }

    #[test]
    fn test_module_layer_detection() {
        let analyzer = ArchitecturalAnalyzer::new()
            .with_allowed_layer("domain")
            .with_allowed_layer("application");

        assert_eq!(
            analyzer.get_module_layer("/path/to/domain/mod.rs"),
            "domain"
        );

        assert_eq!(
            analyzer.get_module_layer("/path/to/unknown/mod.rs"),
            "domain" // Default to first allowed layer
        );
    }
}
