use syn::{visit, TypePath, ItemImpl, ItemTrait};
use std::collections::HashSet;
use super::*;

/// Visitor that analyzes dependency-related architectural issues
pub struct DependencyVisitor<'a> {
    analyzer: &'a ArchitecturalAnalyzer,
    current_impl: Option<&'a ItemImpl>,
    current_trait: Option<&'a TypePath>,
    violations: Vec<DependencyInversionViolation>,
    allowed_concrete_deps: HashSet<String>,
}

impl<'a> DependencyVisitor<'a> {
    /// Create a new DependencyVisitor
    pub fn new(analyzer: &'a ArchitecturalAnalyzer) -> Self {
        let allowed_concrete_deps = analyzer.allowed_concrete_dependencies
            .iter()
            .cloned()
            .collect();
            
        Self {
            analyzer,
            current_impl: None,
            current_trait: None,
            violations: Vec::new(),
            allowed_concrete_deps,
        }
    }
    
    /// Check if a type path represents a concrete type that should be behind a trait
    fn check_concrete_type(&mut self, ty: &TypePath) {
        if self.current_impl.is_none() {
            return;
        }

        // Skip if it's a trait bound or self type
        if ty.path.segments.iter().any(|s| s.ident == "dyn" || s.ident == "Self") {
            return;
        }

        // Skip standard library types and primitive types
        let path_str = path_to_string(&ty.path);
        if self.is_std_or_primitive(&path_str) {
            return;
        }

        // Check if this is an allowed concrete dependency
        if !self.allowed_concrete_deps.contains(&path_str) {
            self.violations.push(DependencyInversionViolation {
                message: format!(
                    "Concrete type dependency on '{}' should be behind a trait", 
                    path_str
                ),
                location: CodeLocation::from_span(&ty.span()),
            });
        }
    }

    /// Check if a path is from the standard library or a primitive type
    fn is_std_or_primitive(&self, path: &str) -> bool {
        // Standard library paths
        if path.starts_with("std::") 
            || path.starts_with("core::")
            || path.starts_with("alloc::")
            || path.starts_with("collections::")
            || path.starts_with("collections::")
        {
            return true;
        }

        // Primitive types
        matches!(
            path,
            "bool"
                | "char"
                | "f32"
                | "f64"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "str"
                | "String"
                | "&str"
                | "()"
                | "!"
                | "Option"
                | "Result"
        )
    }
}

impl<'a> ArchitecturalVisitor for DependencyVisitor<'a> {
    fn analyze(&mut self, ast: &File) -> Vec<ArchitecturalFinding> {
        self.violations.clear();
        self.visit_file(ast);
        
        self.violations.drain(..).map(|violation| {
            self.create_finding(
                "dependency-inversion-violation",
                format!("Dependency Inversion Principle violation: {}", violation.message),
                violation.location.line,
                Some("Depend on abstractions (traits) rather than concrete implementations. Consider creating and using a trait that represents the required behavior."),
                Severity::Warning,
                0.8,
                "ARCH004",
            )
        }).collect()
    }
    
    fn analyzer(&self) -> &ArchitecturalAnalyzer {
        self.analyzer
    }
}

impl<'a> visit::Visit<'a> for DependencyVisitor<'a> {
    fn visit_item_impl(&mut self, node: &'a ItemImpl) {
        let old_impl = self.current_impl.replace(node);
        let old_trait = self.current_trait.take();

        if let Some((_, path, _)) = &node.trait_ {
            self.current_trait = Some(path);
        }

        visit::visit_item_impl(self, node);

        self.current_impl = old_impl;
        self.current_trait = old_trait;
    }

    fn visit_type_path(&mut self, ty: &'a TypePath) {
        self.check_concrete_type(ty);
        visit::visit_type_path(self, ty);
    }
}
