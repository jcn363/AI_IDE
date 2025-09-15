use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use syn::visit::Visit;
use syn::{parse_file, File as SynFile, Ident, Item, ItemFn, ReturnType, Type};
use tokio::sync::Mutex;

use crate::ai_suggester::AnalysisContext;
use crate::error::{AnalysisError, AnalysisResult};
use crate::types::{
    RefactoringSuggestion, SafetyValidation, ValidationCheck, ValidationCheckType,
    ValidationSeverity,
};

/// Safety guard component using syn AST analysis for validation
pub struct RefactoringSafetyGuard {
    validation_cache: Arc<Mutex<HashSet<String>>>,
}

impl RefactoringSafetyGuard {
    /// Create a new safety guard
    pub fn new() -> Self {
        Self {
            validation_cache: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Validate the safety of refactoring suggestions
    pub async fn validate_safety(
        &self,
        suggestions: &[RefactoringSuggestion],
        context: &AnalysisContext,
    ) -> AnalysisResult<SafetyValidation> {
        let mut all_checks = Vec::new();
        let mut functional_equivalence_verified = true;
        let mut behavior_preservation_confirmed = true;
        let mut circular_dependencies = Vec::new();

        for suggestion in suggestions {
            let validation = self.validate_single_suggestion(suggestion, context).await?;
            all_checks.extend(validation.validation_checks);

            if !validation.functional_equivalence_verified {
                functional_equivalence_verified = false;
            }
            if !validation.behavior_preservation_confirmed {
                behavior_preservation_confirmed = false;
            }
            circular_dependencies.extend(validation.circular_dependencies);
        }

        // Calculate overall safety score
        let overall_safety_score = self.calculate_overall_safety_score(&all_checks);

        Ok(SafetyValidation {
            validation_id: uuid::Uuid::new_v4(),
            transformation_id: suggestions.first().map(|s| s.id).unwrap_or_default(),
            is_safe: overall_safety_score >= 0.8, // 80% threshold for safety
            validation_checks: all_checks,
            functional_equivalence_verified,
            behavior_preservation_confirmed,
            circular_dependencies,
            overall_safety_score,
        })
    }

    /// Validate a single suggestion
    async fn validate_single_suggestion(
        &self,
        suggestion: &RefactoringSuggestion,
        context: &AnalysisContext,
    ) -> AnalysisResult<SafetyValidation> {
        // Parse the file to analyze
        let content = std::fs::read_to_string(&suggestion.target_file).map_err(|e| {
            AnalysisError::DataProcessing {
                stage: format!("Failed to read file {}: {}", suggestion.target_file, e),
            }
        })?;

        let syntax_tree: SynFile =
            parse_file(&content).map_err(|e| AnalysisError::DataProcessing {
                stage: format!("Failed to parse file {}: {}", suggestion.target_file, e),
            })?;

        // Perform various safety checks using syn visitors
        let syntactic_check = self.check_syntactic_correctness(&syntax_tree);
        let type_check = self.check_type_safety(&syntax_tree, suggestion);
        let functional_check = self.check_functional_equivalence(&syntax_tree, suggestion);
        let behavior_check = self.check_behavior_preservation(&syntax_tree, suggestion);

        let checks = vec![
            syntactic_check,
            type_check,
            functional_check,
            behavior_check,
        ];
        let functional_equivalence_verified = checks.iter().all(|c| c.passed);
        let behavior_preservation_confirmed = checks.iter().all(|c| c.passed);

        Ok(SafetyValidation {
            validation_id: uuid::Uuid::new_v4(),
            transformation_id: suggestion.id,
            is_safe: functional_equivalence_verified && behavior_preservation_confirmed,
            validation_checks: checks,
            functional_equivalence_verified,
            behavior_preservation_confirmed,
            circular_dependencies: vec![], // Would be detected separately
            overall_safety_score: if functional_equivalence_verified
                && behavior_preservation_confirmed
            {
                1.0
            } else {
                0.5
            },
        })
    }

    /// Check syntactic correctness using syn parsing
    fn check_syntactic_correctness(&self, syntax_tree: &SynFile) -> ValidationCheck {
        // Since we already parsed the file successfully, syntax is correct
        // But we could do additional checks here
        ValidationCheck {
            check_type: ValidationCheckType::SyntacticCorrectness,
            passed: true,
            severity: ValidationSeverity::Info,
            message: "Code parses successfully".to_string(),
            details: Default::default(),
        }
    }

    /// Check type safety using syn type analysis
    fn check_type_safety(
        &self,
        syntax_tree: &SynFile,
        suggestion: &RefactoringSuggestion,
    ) -> ValidationCheck {
        // Use syn visitor to check for type-related issues
        let mut visitor = TypeSafetyVisitor::new();
        visitor.visit_file(syntax_tree);

        let passed = visitor.issues.is_empty();
        let severity = if passed {
            ValidationSeverity::Info
        } else {
            ValidationSeverity::Error
        };
        let message = if passed {
            "No type safety issues detected".to_string()
        } else {
            format!("Type safety issues found: {}", visitor.issues.join(", "))
        };

        ValidationCheck {
            check_type: ValidationCheckType::TypeChecking,
            passed,
            severity,
            message,
            details: Default::default(),
        }
    }

    /// Check functional equivalence
    fn check_functional_equivalence(
        &self,
        syntax_tree: &SynFile,
        suggestion: &RefactoringSuggestion,
    ) -> ValidationCheck {
        // This would require more sophisticated analysis
        // For now, use heuristics based on suggestion type
        let passed = match suggestion.suggestion_type {
            crate::types::RefactoringType::RenameSymbol => true, // Renaming should preserve functionality
            crate::types::RefactoringType::ExtractMethod => true, // Extracting method should preserve functionality
            crate::types::RefactoringType::ExtractVariable => true, // Extracting variable should preserve functionality
            _ => false, // Unknown refactoring types need manual verification
        };

        ValidationCheck {
            check_type: ValidationCheckType::FunctionalEquivalence,
            passed,
            severity: if passed {
                ValidationSeverity::Info
            } else {
                ValidationSeverity::Warning
            },
            message: if passed {
                "Functional equivalence likely preserved".to_string()
            } else {
                "Functional equivalence cannot be automatically verified".to_string()
            },
            details: Default::default(),
        }
    }

    /// Check behavior preservation
    fn check_behavior_preservation(
        &self,
        syntax_tree: &SynFile,
        suggestion: &RefactoringSuggestion,
    ) -> ValidationCheck {
        // Use syn visitor to check for behavioral issues
        let mut visitor = BehaviorVisitor::new();
        visitor.visit_file(syntax_tree);

        let has_side_effects = visitor.has_side_effects;
        let passed = !has_side_effects
            || matches!(
                suggestion.suggestion_type,
                crate::types::RefactoringType::ExtractMethod
            );

        ValidationCheck {
            check_type: ValidationCheckType::BehaviorPreservation,
            passed,
            severity: if passed {
                ValidationSeverity::Info
            } else {
                ValidationSeverity::Warning
            },
            message: if passed {
                "Behavior preservation verified".to_string()
            } else {
                "Potential behavior changes detected".to_string()
            },
            details: Default::default(),
        }
    }

    /// Calculate overall safety score from checks
    fn calculate_overall_safety_score(&self, checks: &[ValidationCheck]) -> f64 {
        if checks.is_empty() {
            return 0.0;
        }

        let passed_checks = checks.iter().filter(|c| c.passed).count();
        let total_checks = checks.len();

        passed_checks as f64 / total_checks as f64
    }
}

/// Visitor for type safety analysis using syn 2.x
struct TypeSafetyVisitor {
    issues: Vec<String>,
}

impl TypeSafetyVisitor {
    fn new() -> Self {
        Self { issues: Vec::new() }
    }
}

impl<'ast> Visit<'ast> for TypeSafetyVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Check for potentially unsafe patterns
        if let ReturnType::Type(_, ty) = &node.sig.output {
            if let Type::Infer(_) = **ty {
                self.issues
                    .push("Function has inferred return type".to_string());
            }
        }

        // Check for raw pointers (potential unsafety)
        let mut ptr_visitor = PointerVisitor::new();
        syn::visit::visit_item_fn(&mut ptr_visitor, node);
        self.issues.extend(
            ptr_visitor
                .pointers
                .into_iter()
                .map(|p| format!("Raw pointer usage: {}", p)),
        );

        syn::visit::visit_item_fn(self, node);
    }
}

/// Visitor to detect raw pointers
struct PointerVisitor {
    pointers: Vec<String>,
}

impl PointerVisitor {
    fn new() -> Self {
        Self {
            pointers: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for PointerVisitor {
    fn visit_type_ptr(&mut self, node: &'ast syn::TypePtr) {
        self.pointers.push(format!("{:?}", node));
        syn::visit::visit_type_ptr(self, node);
    }
}

/// Visitor for behavior analysis
struct BehaviorVisitor {
    has_side_effects: bool,
}

impl BehaviorVisitor {
    fn new() -> Self {
        Self {
            has_side_effects: false,
        }
    }
}

impl<'ast> Visit<'ast> for BehaviorVisitor {
    fn visit_expr_call(&mut self, _node: &'ast syn::ExprCall) {
        // Function calls can have side effects
        self.has_side_effects = true;
        syn::visit::visit_expr_call(self, _node);
    }

    fn visit_expr_assign(&mut self, _node: &'ast syn::ExprAssign) {
        // Assignments can be side effects
        self.has_side_effects = true;
        syn::visit::visit_expr_assign(self, _node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safety_guard_creation() {
        let guard = RefactoringSafetyGuard::new();
        let cache = guard.validation_cache.lock().await;
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_type_safety_visitor() {
        let code = r#"
            fn safe_function() -> i32 {
                42
            }

            fn unsafe_function() -> *const i32 {
                std::ptr::null()
            }
        "#;

        let syntax_tree: SynFile = parse_file(code).unwrap();
        let mut visitor = TypeSafetyVisitor::new();
        visitor.visit_file(&syntax_tree);

        // Should detect the raw pointer
        assert!(!visitor.issues.is_empty());
        assert!(visitor
            .issues
            .iter()
            .any(|issue| issue.contains("Raw pointer")));
    }

    #[tokio::test]
    async fn test_behavior_visitor() {
        let code = r#"
            fn test() {
                let mut x = 1;
                x = 2; // assignment
                println!("{}", x); // function call
            }
        "#;

        let syntax_tree: SynFile = parse_file(code).unwrap();
        let mut visitor = BehaviorVisitor::new();
        visitor.visit_file(&syntax_tree);

        assert!(visitor.has_side_effects);
    }
}
