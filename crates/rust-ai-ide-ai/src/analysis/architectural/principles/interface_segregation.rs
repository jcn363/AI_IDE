use super::super::types::CodeLocation;
use syn::{visit::Visit, ItemTrait};

/// Analyzer for the Interface Segregation Principle
///
/// This analyzer checks for traits that have too many methods or methods with too many
/// parameters, which might indicate that the trait is doing too much.
pub struct InterfaceSegregationAnalyzer {
    /// Maximum number of methods allowed in a trait before it's considered a violation
    max_methods: usize,
    /// Maximum number of parameters allowed in a method before it's considered a violation
    max_parameters: usize,
    /// List of violations found during analysis
    violations: Vec<InterfaceSegregationViolation>,
}

/// Represents a violation of the Interface Segregation Principle
#[derive(Debug)]
pub struct InterfaceSegregationViolation {
    /// Name of the trait with too many methods
    pub trait_name: String,
    /// Number of methods in the trait
    pub method_count: usize,
    /// Location of the trait definition
    pub location: CodeLocation,
}

impl Default for InterfaceSegregationAnalyzer {
    fn default() -> Self {
        Self {
            max_methods: 10,
            max_parameters: 5,
            violations: Vec::new(),
        }
    }
}

impl InterfaceSegregationAnalyzer {
    /// Create a new InterfaceSegregationAnalyzer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of methods allowed in a trait
    pub fn with_max_methods(mut self, max: usize) -> Self {
        self.max_methods = max;
        self
    }

    /// Set the maximum number of parameters allowed in a method
    pub fn with_max_parameters(mut self, max: usize) -> Self {
        self.max_parameters = max;
        self
    }

    /// Analyze a trait definition for interface segregation violations
    pub fn analyze_trait(&mut self, item_trait: &ItemTrait) {
        let method_count = item_trait
            .items
            .iter()
            .filter(|item| matches!(item, syn::TraitItem::Method(_)))
            .count();

        if method_count > self.max_methods {
            let violation = InterfaceSegregationViolation {
                trait_name: item_trait.ident.to_string(),
                method_count,
                location: CodeLocation {
                    file_path: String::new(), // Will be filled in by the caller
                    line: item_trait.span().start().line as u32,
                    column: item_trait.span().start().column as u32,
                },
            };
            self.violations.push(violation);
        }

        // Check for methods with too many parameters
        for item in &item_trait.items {
            if let syn::TraitItem::Method(method) = item {
                let param_count = method.sig.inputs.len();
                if param_count > self.max_parameters {
                    let violation = InterfaceSegregationViolation {
                        trait_name: format!("{}::{}(...)", item_trait.ident, method.sig.ident),
                        method_count: param_count,
                        location: CodeLocation {
                            file_path: String::new(), // Will be filled in by the caller
                            line: method.span().start().line as u32,
                            column: method.span().start().column as u32,
                        },
                    };
                    self.violations.push(violation);
                }
            }
        }
    }

    /// Get all violations found during analysis
    pub fn violations(&self) -> &[InterfaceSegregationViolation] {
        &self.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_interface_segregation() {
        let code = r#"
            trait TooLargeTrait {
                fn method1(&self);
                fn method2(&self);
                fn method3(&self);
                fn method4(&self);
                fn method5(&self);
                fn method6(&self);
                fn method7(&self);
                fn method8(&self);
                fn method9(&self);
                fn method10(&self);
                fn method11(&self); // This should trigger the violation
            }
            
            trait TooManyParameters {
                fn too_many_params(&self, a: i32, b: i32, c: i32, d: i32, e: i32, f: i32);
            }
        "#;

        let file = syn::parse_file(code).unwrap();
        let mut analyzer = InterfaceSegregationAnalyzer::new()
            .with_max_methods(10)
            .with_max_parameters(5);

        for item in &file.items {
            if let syn::Item::Trait(t) = item {
                analyzer.analyze_trait(t);
            }
        }

        assert_eq!(analyzer.violations().len(), 2);

        // Check that we found the too-large trait
        assert!(analyzer
            .violations()
            .iter()
            .any(|v| v.trait_name == "TooLargeTrait" && v.method_count > 10));

        // Check that we found the method with too many parameters
        assert!(analyzer.violations().iter().any(|v| v
            .trait_name
            .starts_with("TooManyParameters::too_many_params")));
    }
}
