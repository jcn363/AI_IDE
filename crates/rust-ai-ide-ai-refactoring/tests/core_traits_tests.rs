#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_trait::async_trait;
    use rust_ai_ide_ai_refactoring::core_traits::RefactoringOperation;
    use rust_ai_ide_ai_refactoring::types::*;

    // Mock implementation for testing
    struct MockRefactoringOperation {
        pub mock_name:              String,
        pub mock_description:       String,
        pub mock_refactoring_type:  RefactoringType,
        pub should_be_applicable:   bool,
        pub execute_should_succeed: bool,
        pub analyze_should_succeed: bool,
    }

    impl MockRefactoringOperation {
        fn new(
            name: String,
            description: String,
            refactoring_type: RefactoringType,
            applicable: bool,
            execute_success: bool,
            analyze_success: bool,
        ) -> Self {
            MockRefactoringOperation {
                mock_name:              name,
                mock_description:       description,
                mock_refactoring_type:  refactoring_type,
                should_be_applicable:   applicable,
                execute_should_succeed: execute_success,
                analyze_should_succeed: analyze_success,
            }
        }
    }

    #[async_trait]
    impl RefactoringOperation for MockRefactoringOperation {
        async fn execute(
            &self,
            _context: &RefactoringContext,
            _options: &RefactoringOptions,
        ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
            if self.execute_should_succeed {
                Ok(RefactoringResult {
                    id:            Some("test-id".to_string()),
                    success:       true,
                    changes:       vec![],
                    error_message: None,
                    warnings:      vec![],
                    new_content:   Some("mock content".to_string()),
                })
            } else {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Mock error",
                )))
            }
        }

        async fn analyze(
            &self,
            _context: &RefactoringContext,
        ) -> Result<RefactoringAnalysis, Box<dyn std::error::Error + Send + Sync>> {
            if self.analyze_should_succeed {
                Ok(RefactoringAnalysis {
                    is_safe:          true,
                    confidence_score: 0.9,
                    potential_impact: RefactoringImpact::Low,
                    affected_files:   vec!["test.rs".to_string()],
                    affected_symbols: vec!["test_symbol".to_string()],
                    breaking_changes: vec![],
                    suggestions:      vec![],
                    warnings:         vec![],
                })
            } else {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Analysis failed",
                )))
            }
        }

        async fn is_applicable(
            &self,
            _context: &RefactoringContext,
            _options: Option<&RefactoringOptions>,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(self.should_be_applicable)
        }

        fn refactoring_type(&self) -> RefactoringType {
            self.mock_refactoring_type.clone()
        }

        fn name(&self) -> &str {
            &self.mock_name
        }

        fn description(&self) -> &str {
            &self.mock_description
        }
    }

    #[test]
    fn test_mock_refactoring_operation_name() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );
        assert_eq!(operation.name(), "Test Operation");
    }

    #[test]
    fn test_mock_refactoring_operation_description() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );
        assert_eq!(operation.description(), "A test operation");
    }

    #[test]
    fn test_mock_refactoring_operation_refactoring_type() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::ExtractFunction,
            true,
            true,
            true,
        );
        assert_eq!(
            operation.refactoring_type(),
            RefactoringType::ExtractFunction
        );
    }

    #[tokio::test]
    async fn test_mock_refactoring_operation_execute_success() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );

        let context = RefactoringContext::default();
        let options = RefactoringOptions {
            create_backup:            true,
            generate_tests:           false,
            apply_to_all_occurrences: false,
            preserve_references:      true,
            ignore_safe_operations:   false,
            extra_options:            None,
        };

        let result = operation.execute(&context, &options).await.unwrap();
        assert!(result.success);
        assert_eq!(result.new_content, Some("mock content".to_string()));
        assert_eq!(result.id, Some("test-id".to_string()));
    }

    #[tokio::test]
    async fn test_mock_refactoring_operation_execute_failure() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            false,
            true,
        );

        let context = RefactoringContext::default();
        let options = RefactoringOptions::default();

        let result = operation.execute(&context, &options).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mock error"));
    }

    #[tokio::test]
    async fn test_mock_refactoring_operation_analyze_success() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );

        let context = RefactoringContext::default();
        let analysis = operation.analyze(&context).await.unwrap();

        assert!(analysis.is_safe);
        assert_eq!(analysis.confidence_score, 0.9);
        assert_eq!(analysis.potential_impact, RefactoringImpact::Low);
        assert_eq!(analysis.affected_files, vec!["test.rs"]);
        assert_eq!(analysis.affected_symbols, vec!["test_symbol"]);
        assert!(analysis.breaking_changes.is_empty());
        assert!(analysis.suggestions.is_empty());
        assert!(analysis.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_mock_refactoring_operation_analyze_failure() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            false,
        );

        let context = RefactoringContext::default();
        let result = operation.analyze(&context).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Analysis failed"));
    }

    #[tokio::test]
    async fn test_mock_refactoring_operation_is_applicable_true() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );

        let context = RefactoringContext::default();
        let applicable = operation.is_applicable(&context, None).await.unwrap();
        assert!(applicable);
    }

    #[tokio::test]
    async fn test_mock_refactoring_operation_is_applicable_false() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            false,
            true,
            true,
        );

        let context = RefactoringContext::default();
        let applicable = operation.is_applicable(&context, None).await.unwrap();
        assert!(!applicable);
    }

    #[tokio::test]
    async fn test_mock_refactoring_operation_is_applicable_with_options() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );

        let context = RefactoringContext::default();
        let options = RefactoringOptions::default();
        let applicable = operation
            .is_applicable(&context, Some(&options))
            .await
            .unwrap();
        assert!(applicable);
    }

    #[test]
    fn test_is_experimental_enabled_true() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );

        let mut extra_options = HashMap::new();
        extra_options.insert("experimental".to_string(), serde_json::json!(true));

        let options = RefactoringOptions {
            create_backup:            false,
            generate_tests:           false,
            apply_to_all_occurrences: false,
            preserve_references:      false,
            ignore_safe_operations:   false,
            extra_options:            Some(extra_options),
        };

        assert!(operation.is_experimental_enabled(&options));
    }

    #[test]
    fn test_is_experimental_enabled_false() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );

        let options = RefactoringOptions::default();
        assert!(!operation.is_experimental_enabled(&options));
    }

    #[test]
    fn test_is_experimental_enabled_false_with_wrong_value() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );

        let mut extra_options = HashMap::new();
        extra_options.insert("experimental".to_string(), serde_json::json!("not_boolean"));

        let options = RefactoringOptions {
            create_backup:            false,
            generate_tests:           false,
            apply_to_all_occurrences: false,
            preserve_references:      false,
            ignore_safe_operations:   false,
            extra_options:            Some(extra_options),
        };

        assert!(!operation.is_experimental_enabled(&options));
    }

    #[test]
    fn test_is_experimental_enabled_no_extra_options() {
        let operation = MockRefactoringOperation::new(
            "Test Operation".to_string(),
            "A test operation".to_string(),
            RefactoringType::Rename,
            true,
            true,
            true,
        );

        let options = RefactoringOptions {
            create_backup:            false,
            generate_tests:           false,
            apply_to_all_occurrences: false,
            preserve_references:      false,
            ignore_safe_operations:   false,
            extra_options:            None,
        };

        assert!(!operation.is_experimental_enabled(&options));
    }
}
