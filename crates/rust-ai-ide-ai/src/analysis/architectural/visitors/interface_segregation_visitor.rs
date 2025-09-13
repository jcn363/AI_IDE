use syn::{visit, ItemTrait, TraitItem, Type};
use super::super::types::{ArchitecturalFinding, CodeLocation, Severity};
use super::ArchitecturalVisitor;

/// Visitor that checks for violations of the Interface Segregation Principle
pub struct InterfaceSegregationVisitor<'a> {
    analyzer: &'a ArchitecturalAnalyzer,
    violations: Vec<ArchitecturalFinding>,
}

impl<'a> InterfaceSegregationVisitor<'a> {
    /// Create a new InterfaceSegregationVisitor
    pub fn new(analyzer: &'a ArchitecturalAnalyzer) -> Self {
        Self {
            analyzer,
            violations: Vec::new(),
        }
    }

    /// Check if a type is an I/O-related type that might justify more methods
    fn is_io_type(ty: &Type) -> bool {
        // Check for common I/O types that might have many methods
        let type_str = format!("{:?}", ty);
        type_str.contains("File") ||
        type_str.contains("Tcp") ||
        type_str.contains("Udp") ||
        type_str.contains("Buf") ||
        type_str.contains("Read") ||
        type_str.contains("Write") ||
        type_str.contains("Stream") ||
        type_str.contains("Sink")
    }
}

impl<'a> visit::Visit<'a> for InterfaceSegregationVisitor<'a> {
    fn visit_item_trait(&mut self, node: &'a ItemTrait) {
        let method_count = node.items.iter()
            .filter(|item| matches!(item, TraitItem::Method(_)))
            .count();

        // Check if the trait has too many methods
        if method_count > self.analyzer.max_methods_per_interface {
            self.violations.push(ArchitecturalFinding {
                id: format!("too_many_methods_{}", node.ident),
                message: format!(
                    "Trait '{}' has {} methods, which exceeds the maximum of {}. Consider splitting it into smaller, more focused traits.",
                    node.ident, method_count, self.analyzer.max_methods_per_interface
                ),
                severity: Severity::Warning,
                location: CodeLocation {
                    file_path: String::new(), // Will be filled in by the analyzer
                    line: node.span().start().line as u32,
                    column: node.span().start().column as u32,
                },
                suggestion: Some("Split this trait into smaller, more focused traits that each handle a single responsibility.".to_string()),
                confidence: 0.8,
                rule_id: "ISP_VIOLATION_TOO_MANY_METHODS".to_string(),
            });
        }

        // Check for methods with too many parameters
        for item in &node.items {
            if let TraitItem::Method(method) = item {
                let param_count = method.sig.inputs.len();

                // Skip the &self parameter
                let non_self_params = if method.sig.inputs.iter().next()
                    .map_or(false, |input| {
                        matches!(input, syn::FnArg::Receiver(_))
                    }) {
                    param_count.saturating_sub(1)
                } else {
                    param_count
                };

                // Check for too many parameters
                if non_self_params > self.analyzer.max_parameters_per_method {
                    // Skip I/O related methods that might naturally have more parameters
                    let is_io_related = method.sig.inputs.iter().any(|input| {
                        if let syn::FnArg::Typed(ty) = input {
                            Self::is_io_type(&ty.ty)
                        } else {
                            false
                        }
                    });

                    if !is_io_related {
                        self.violations.push(ArchitecturalFinding {
                            id: format!("too_many_params_{}_{}", node.ident, method.sig.ident),
                            message: format!(
                                "Method '{}.{}' has {} parameters, which exceeds the maximum of {}. Consider using a parameter object or splitting the method.",
                                node.ident, method.sig.ident, non_self_params, self.analyzer.max_parameters_per_method
                            ),
                            severity: Severity::Warning,
                            location: CodeLocation {
                                file_path: String::new(), // Will be filled in by the analyzer
                                line: method.span().start().line as u32,
                                column: method.span().start().column as u32,
                            },
                            suggestion: Some("Consider using a parameter object or builder pattern to reduce the number of parameters.".to_string()),
                            confidence: 0.85,
                            rule_id: "ISP_VIOLATION_TOO_MANY_PARAMETERS".to_string(),
                        });
                    }
                }
            }
        }

        // Continue visiting child nodes
        visit::visit_item_trait(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_too_many_methods() {
        let analyzer = ArchitecturalAnalyzer {
            max_methods_per_interface: 5,
            max_parameters_per_method: 5,
            ..Default::default()
        };

        let mut visitor = InterfaceSegregationVisitor::new(&analyzer);

        // This trait has 6 methods, which exceeds the limit of 5
        let code = r#"
            pub trait TooLargeTrait {
                fn method1(&self);
                fn method2(&self);
                fn method3(&self);
                fn method4(&self);
                fn method5(&self);
                fn method6(&self); // This should trigger a violation
            }
        "#;

        let ast: syn::File = syn::parse_str(code).unwrap();
        visitor.visit_file(&ast);

        assert_eq!(visitor.violations.len(), 1);
        assert!(visitor.violations[0].message.contains("has 6 methods"));
    }

    #[test]
    test_too_many_parameters() {
        let analyzer = ArchitecturalAnalyzer {
            max_methods_per_interface: 10,
            max_parameters_per_method: 3,
            ..Default::default()
        };

        let mut visitor = InterfaceSegregationVisitor::new(&analyzer);

        // This method has 4 parameters, which exceeds the limit of 3
        let code = r#"
            pub trait SomeTrait {
                fn too_many_params(&self, a: i32, b: i32, c: i32, d: i32);
            }
        "#;

        let ast: syn::File = syn::parse_str(code).unwrap();
        visitor.visit_file(&ast);

        assert_eq!(visitor.violations.len(), 1);
        assert!(visitor.violations[0].message.contains("has 4 parameters"));
    }

    #[test]
    fn test_io_methods_exempt() {
        let analyzer = ArchitecturalAnalyzer {
            max_methods_per_interface: 10,
            max_parameters_per_method: 3,
            ..Default::default()
        };

        let mut visitor = InterfaceSegregationVisitor::new(&analyzer);

        // This method has 4 parameters, but they're I/O related so it should be allowed
        let code = r#"
            pub trait IoInterface {
                fn read(&mut self, buf: &mut [u8], offset: u64, len: usize) -> std::io::Result<usize>;
            }
        "#;

        let ast: syn::File = syn::parse_str(code).unwrap();
        visitor.visit_file(&ast);

        // Should not have any violations because it's I/O related
        assert!(visitor.violations.is_empty());
    }
}
