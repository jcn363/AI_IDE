use syn::visit::Visit;
use syn::{ItemImpl, TypePath};

use super::super::types::CodeLocation;

/// Analyzer for the Dependency Inversion Principle
///
/// This analyzer checks for violations where high-level modules depend on low-level
/// modules directly instead of depending on abstractions.
pub struct DependencyInversionAnalyzer {
    /// List of violations found during analysis
    violations:         Vec<DependencyInversionViolation>,
    /// List of known abstractions (traits)
    known_abstractions: Vec<String>,
}

/// Represents a violation of the Dependency Inversion Principle
#[derive(Debug)]
pub struct DependencyInversionViolation {
    /// The concrete type that should be behind an interface
    pub concrete_type: String,
    /// Location of the violation
    pub location:      CodeLocation,
}

impl DependencyInversionAnalyzer {
    /// Create a new DependencyInversionAnalyzer
    pub fn new() -> Self {
        Self {
            violations:         Vec::new(),
            known_abstractions: Vec::new(),
        }
    }

    /// Add a known abstraction (trait) to the analyzer
    pub fn with_abstraction(mut self, trait_name: &str) -> Self {
        self.known_abstractions.push(trait_name.to_string());
        self
    }

    /// Analyze an implementation block for dependency inversion violations
    pub fn analyze_impl(&mut self, item_impl: &ItemImpl) {
        let mut visitor = DependencyInversionVisitor {
            analyzer:           self,
            current_impl_trait: item_impl.trait_.as_ref().map(|(_, path, _)| path).cloned(),
        };
        visitor.visit_item_impl(item_impl);
    }

    /// Get all violations found during analysis
    pub fn violations(&self) -> &[DependencyInversionViolation] {
        &self.violations
    }
}

struct DependencyInversionVisitor<'a> {
    analyzer:           &'a mut DependencyInversionAnalyzer,
    current_impl_trait: Option<syn::Path>,
}

impl<'ast> Visit<'ast> for DependencyInversionVisitor<'_> {
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        // Check if this is a trait implementation
        if self.current_impl_trait.is_none() {
            // For concrete implementations, check all type references
            syn::visit::visit_item_impl(self, node);
        }
    }

    fn visit_type_path(&mut self, ty: &'ast TypePath) {
        // Skip if this is part of a trait path
        if self
            .current_impl_trait
            .as_ref()
            .map_or(false, |trait_path| is_path_part_of(trait_path, &ty.path))
        {
            return;
        }

        // Skip standard library types and primitives
        if is_std_or_primitive(&ty.path) {
            return;
        }

        // Check if this is a concrete type that should be behind an interface
        let type_name = path_to_string(&ty.path);
        if !self
            .analyzer
            .known_abstractions
            .iter()
            .any(|abstraction| type_name.starts_with(abstraction))
        {
            // This is a potential violation
            let location = CodeLocation {
                file_path: String::new(), // Will be filled in by the caller
                line:      ty.span().start().line as u32,
                column:    ty.span().start().column as u32,
            };

            self.analyzer.violations.push(DependencyInversionViolation {
                concrete_type: type_name,
                location,
            });
        }
    }
}

fn is_path_part_of(base_path: &syn::Path, test_path: &syn::Path) -> bool {
    // Simple check if test_path starts with base_path
    if base_path.segments.len() > test_path.segments.len() {
        return false;
    }

    base_path
        .segments
        .iter()
        .zip(&test_path.segments)
        .all(|(a, b)| a.ident == b.ident)
}

fn is_std_or_primitive(path: &syn::Path) -> bool {
    // Check for standard library paths and primitives
    if let Some(segment) = path.segments.first() {
        let ident = segment.ident.to_string();
        matches!(
            ident.as_str(),
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
                | "str"
                | "String"
                | "Vec"
                | "Option"
                | "Result"
                | "Box"
                | "Arc"
                | "Rc"
                | "Cell"
                | "RefCell"
                | "Mutex"
                | "RwLock"
                | "HashMap"
                | "HashSet"
        )
    } else {
        false
    }
}

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

    #[test]
    fn test_dependency_inversion() {
        let code = r#"
            struct Service {
                repository: ConcreteRepository,
            }

            struct ConcreteRepository;
        "#;

        let file = syn::parse_file(code).unwrap();
        let mut analyzer = DependencyInversionAnalyzer::new().with_abstraction("dyn Repository");

        for item in &file.items {
            if let syn::Item::Struct(s) = item {
                let mut visitor = DependencyInversionVisitor {
                    analyzer:           &mut analyzer,
                    current_impl_trait: None,
                };
                visitor.visit_item_struct(s);
            }
        }

        assert_eq!(analyzer.violations().len(), 1);
        assert_eq!(analyzer.violations()[0].concrete_type, "ConcreteRepository");
    }
}
