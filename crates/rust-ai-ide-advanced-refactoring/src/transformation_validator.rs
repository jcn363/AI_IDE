use std::sync::Arc;

use async_trait::async_trait;
use syn::visit::Visit;
use syn::{parse_file, File as SynFile};
use tokio::sync::Mutex;

use crate::error::{AnalysisError, AnalysisResult};
use crate::types::{RefactoringTransformation, ValidationResult as ValResult};

/// Transformation validator using syn AST analysis
pub struct TransformationValidator {
    ai_service:       Arc<dyn crate::ai_suggester::AiService>,
    validation_cache: Arc<Mutex<std::collections::HashMap<String, bool>>>,
}

impl TransformationValidator {
    /// Create a new transformation validator
    pub fn new(ai_service: Arc<dyn crate::ai_suggester::AiService>) -> Self {
        Self {
            ai_service,
            validation_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Validate a transformation using syn AST analysis
    pub async fn validate_transformation(
        &self,
        transformation: &RefactoringTransformation,
    ) -> AnalysisResult<ValResult<()>> {
        // Check cache first
        let cache_key = self.create_cache_key(transformation);
        {
            let cache = self.validation_cache.lock().await;
            if let Some(cached_result) = cache.get(&cache_key) {
                return if *cached_result {
                    Ok(ValResult::Valid(()))
                } else {
                    Ok(ValResult::Invalid(vec![
                        "Cached validation failure".to_string()
                    ]))
                };
            }
        }

        // Read and parse the file
        let content =
            std::fs::read_to_string(&transformation.file_path).map_err(|e| AnalysisError::DataProcessing {
                stage: format!("Failed to read file {}: {}", transformation.file_path, e),
            })?;

        let syntax_tree: SynFile = parse_file(&content).map_err(|e| AnalysisError::DataProcessing {
            stage: format!("Failed to parse file {}: {}", transformation.file_path, e),
        })?;

        // Perform validation checks
        let validation_result = self
            .perform_validation_checks(&syntax_tree, transformation)
            .await;

        // Cache the result
        let is_valid = matches!(validation_result, ValResult::Valid(_));
        {
            let mut cache = self.validation_cache.lock().await;
            cache.insert(cache_key, is_valid);
        }

        Ok(validation_result)
    }

    /// Validate multiple transformations
    pub async fn validate_transformations(
        &self,
        transformations: &[RefactoringTransformation],
    ) -> AnalysisResult<Vec<ValResult<()>>> {
        let mut results = Vec::new();

        for transformation in transformations {
            let result = self.validate_transformation(transformation).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Perform comprehensive validation checks
    async fn perform_validation_checks(
        &self,
        syntax_tree: &SynFile,
        transformation: &RefactoringTransformation,
    ) -> ValResult<()> {
        let mut errors = Vec::new();

        // Check 1: Syntactic validity
        if let Err(e) = self.check_syntactic_validity(syntax_tree) {
            errors.push(format!("Syntactic error: {}", e));
        }

        // Check 2: Type safety
        if let Err(e) = self.check_type_safety(syntax_tree, transformation) {
            errors.push(format!("Type safety error: {}", e));
        }

        // Check 3: Semantic correctness
        if let Err(e) = self
            .check_semantic_correctness(syntax_tree, transformation)
            .await
        {
            errors.push(format!("Semantic error: {}", e));
        }

        // Check 4: Transformation consistency
        if let Err(e) = self.check_transformation_consistency(transformation) {
            errors.push(format!("Transformation consistency error: {}", e));
        }

        if errors.is_empty() {
            ValResult::Valid(())
        } else {
            ValResult::Invalid(errors)
        }
    }

    /// Check syntactic validity (parsing already succeeded, but we can do more)
    fn check_syntactic_validity(&self, _syntax_tree: &SynFile) -> Result<(), String> {
        // Basic checks passed since we parsed successfully
        // Could add more sophisticated checks here
        Ok(())
    }

    /// Check type safety using syn visitor
    fn check_type_safety(
        &self,
        syntax_tree: &SynFile,
        _transformation: &RefactoringTransformation,
    ) -> Result<(), String> {
        let mut visitor = TypeValidator::new();
        visitor.visit_file(syntax_tree);

        if visitor.has_issues() {
            Err(format!("Type safety issues: {:?}", visitor.get_issues()))
        } else {
            Ok(())
        }
    }

    /// Check semantic correctness (may involve AI analysis)
    async fn check_semantic_correctness(
        &self,
        syntax_tree: &SynFile,
        transformation: &RefactoringTransformation,
    ) -> Result<(), String> {
        // Use AI service for semantic analysis if needed
        let mut visitor = SemanticValidator::new();
        visitor.visit_file(syntax_tree);

        if visitor.has_potential_issues() {
            // Could use AI service for deeper analysis here
            // For now, just report potential issues
            Err("Potential semantic issues detected".to_string())
        } else {
            Ok(())
        }
    }

    /// Check transformation consistency
    fn check_transformation_consistency(&self, transformation: &RefactoringTransformation) -> Result<(), String> {
        // Check that the transformation makes sense
        if transformation.original_text.is_empty() {
            return Err("Original text cannot be empty".to_string());
        }

        if transformation.transformed_text.is_empty() {
            return Err("Transformed text cannot be empty".to_string());
        }

        // Check line/column numbers are reasonable
        if transformation.line_number == 0 {
            return Err("Line number must be positive".to_string());
        }

        Ok(())
    }

    /// Create cache key for validation results
    fn create_cache_key(&self, transformation: &RefactoringTransformation) -> String {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        transformation.file_path.hash(&mut hasher);
        transformation.line_number.hash(&mut hasher);
        transformation.column_number.hash(&mut hasher);
        transformation.original_text.hash(&mut hasher);
        transformation.transformed_text.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}

/// Type validator using syn visitor
struct TypeValidator {
    issues: Vec<String>,
}

impl TypeValidator {
    fn new() -> Self {
        Self { issues: Vec::new() }
    }

    fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    fn get_issues(&self) -> &[String] {
        &self.issues
    }
}

impl<'ast> Visit<'ast> for TypeValidator {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        // Check for potentially unsafe calls
        // This is a simplified example
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_type_ptr(&mut self, node: &'ast syn::TypePtr) {
        // Flag raw pointers as potential issues
        self.issues.push("Raw pointer usage detected".to_string());
        syn::visit::visit_type_ptr(self, node);
    }
}

/// Semantic validator using syn visitor
struct SemanticValidator {
    potential_issues: Vec<String>,
}

impl SemanticValidator {
    fn new() -> Self {
        Self {
            potential_issues: Vec::new(),
        }
    }

    fn has_potential_issues(&self) -> bool {
        !self.potential_issues.is_empty()
    }
}

impl<'ast> Visit<'ast> for SemanticValidator {
    fn visit_expr_assign(&mut self, _node: &'ast syn::ExprAssign) {
        // Assignments can change behavior
        self.potential_issues
            .push("Assignment operation detected".to_string());
        syn::visit::visit_expr_assign(self, _node);
    }

    fn visit_expr_call(&mut self, _node: &'ast syn::ExprCall) {
        // Function calls can have side effects
        self.potential_issues
            .push("Function call detected".to_string());
        syn::visit::visit_expr_call(self, _node);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    struct MockAiService;

    impl crate::ai_suggester::AiService for MockAiService {}

    #[tokio::test]
    async fn test_validator_creation() {
        let ai_service = Arc::new(MockAiService);
        let validator = TransformationValidator::new(ai_service);
        let cache = validator.validation_cache.lock().await;
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_transformation_validation() {
        let ai_service = Arc::new(MockAiService);
        let validator = TransformationValidator::new(ai_service);

        let transformation = RefactoringTransformation {
            id:               uuid::Uuid::new_v4(),
            suggestion_id:    uuid::Uuid::new_v4(),
            operation_type:   crate::types::TransformationOperation::ReplaceText,
            file_path:        "test.rs".to_string(),
            line_number:      1,
            column_number:    0,
            original_text:    "old".to_string(),
            transformed_text: "new".to_string(),
            dependencies:     vec![],
            rollback_steps:   vec![],
            validation_hash:  String::new(),
        };

        // This will fail because the file doesn't exist, but tests the structure
        let result = validator.validate_transformation(&transformation).await;
        assert!(result.is_err()); // File doesn't exist
    }

    #[test]
    fn test_type_validator() {
        let code = r#"
            fn test() {
                let ptr: *const i32 = std::ptr::null();
            }
        "#;

        let syntax_tree: SynFile = parse_file(code).unwrap();
        let mut validator = TypeValidator::new();
        validator.visit_file(&syntax_tree);

        assert!(validator.has_issues());
        assert!(validator
            .get_issues()
            .iter()
            .any(|issue| issue.contains("Raw pointer")));
    }
}
