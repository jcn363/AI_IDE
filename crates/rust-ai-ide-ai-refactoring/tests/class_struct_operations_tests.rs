#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use rust_ai_ide_ai_refactoring::class_struct_operations::{
        ExtractInterfaceOperation, SplitClassOperation, MergeClassesOperation,
        ExtractClassOperation, EncapsulateFieldOperation, GenerateGettersSettersOperation
    };
    use rust_ai_ide_ai_refactoring::types::*;
    use rust_ai_ide_ai_refactoring::RefactoringOperation;

    // Helper function to create temporary Rust file with test content
    fn create_temp_rust_file(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file
    }

    #[tokio::test]
    async fn test_extract_interface_operation_name_and_description() {
        let operation = ExtractInterfaceOperation;
        assert_eq!(operation.name(), "Extract Interface");
        assert_eq!(operation.description(), "Extracts an interface from a class or struct");
        assert_eq!(operation.refactoring_type(), RefactoringType::ExtractInterface);
    }

    #[tokio::test]
    async fn test_extract_interface_operation_is_applicable() {
        let operation = ExtractInterfaceOperation;

        // Applicable for structs
        let context = RefactoringContext {
            file_path: "test.rs".to_string(),
            cursor_line: 1,
            cursor_character: 0,
            selection: None,
            symbol_name: Some("MyStruct".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
        };
        assert!(operation.is_applicable(&context, None).await.unwrap());

        // Applicable for classes
        let context_class = RefactoringContext {
            symbol_kind: Some(SymbolKind::Class),
            ..context
        };
        assert!(operation.is_applicable(&context_class, None).await.unwrap());

        // Not applicable for other types
        let context_function = RefactoringContext {
            symbol_kind: Some(SymbolKind::Function),
            ..context
        };
        assert!(!operation.is_applicable(&context_function, None).await.unwrap());
    }

    #[tokio::test]
    async fn test_extract_interface_operation_execute_success() {
        let operation = ExtractInterfaceOperation;

        // Create a temporary file with a simple struct
        let test_content = r#"
pub struct CalculatorService {
    value: i32,
}

impl CalculatorService {
    pub fn add(&self, x: i32, y: i32) -> i32 {
        x + y
    }

    pub fn multiply(&self, x: i32, y: i32) -> i32 {
        x * y
    }
}
"#;
        let temp_file = create_temp_rust_file(test_content);
        let file_path = temp_file.path().to_string_lossy().to_string();

        let context = RefactoringContext {
            file_path,
            cursor_line: 1,
            cursor_character: 0,
            selection: None,
            symbol_name: Some("CalculatorService".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
        };

        let options = RefactoringOptions {
            create_backup: true,
            generate_tests: false,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: Some(serde_json::json!({
                "interfaceName": "CalculatorInterface"
            })),
        };

        let result = operation.execute(&context, &options).await.unwrap();
        assert!(result.success);
        assert!(result.new_content.is_some());
        assert!(result.new_content.as_ref().unwrap().contains("trait CalculatorInterface"));
        assert!(result.new_content.as_ref().unwrap().contains("impl CalculatorInterface for CalculatorService"));
        assert!(!result.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_extract_interface_operation_execute_insufficient_methods() {
        let operation = ExtractInterfaceOperation;

        // Create a struct with only one public method
        let test_content = r#"
pub struct SimpleService {
    value: i32,
}

impl SimpleService {
    pub fn get_value(&self) -> i32 {
        self.value
    }
}
"#;
        let temp_file = create_temp_rust_file(test_content);
        let file_path = temp_file.path().to_string_lossy().to_string();

        let context = RefactoringContext {
            file_path,
            cursor_line: 1,
            cursor_character: 0,
            selection: None,
            symbol_name: Some("SimpleService".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
        };

        let options = RefactoringOptions::default();

        let result = operation.execute(&context, &options).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("needs at least 2 public methods"));
    }

    #[tokio::test]
    async fn test_extract_interface_operation_analyze() {
        let operation = ExtractInterfaceOperation;

        let context = RefactoringContext {
            file_path: "test.rs".to_string(),
            cursor_line: 1,
            cursor_character: 0,
            selection: None,
            symbol_name: Some("TestStruct".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
        };

        let analysis = operation.analyze(&context).await.unwrap();
        assert_eq!(analysis.confidence_score, 0.7);
        assert_eq!(analysis.potential_impact, RefactoringImpact::Medium);
        assert!(analysis.affected_files.contains(&"test.rs".to_string()));
        assert!(!analysis.breaking_changes.is_empty());
    }

    #[tokio::test]
    async fn test_split_class_operation_name_and_description() {
        let operation = SplitClassOperation;
        assert_eq!(operation.name(), "Split Class");
        assert_eq!(operation.description(), "Splits a large class into multiple smaller classes using composition");
        assert_eq!(operation.refactoring_type(), RefactoringType::SplitClass);
    }

    #[tokio::test]
    async fn test_split_class_operation_is_applicable() {
        let operation = SplitClassOperation;

        // Applicable for structs with name
        let context = RefactoringContext {
            file_path: "test.rs".to_string(),
            cursor_line: 1,
            cursor_character: 0,
            selection: None,
            symbol_name: Some("LargeStruct".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
        };
        assert!(operation.is_applicable(&context, None).await.unwrap());

        // Not applicable without name
        let context_no_name = RefactoringContext {
            symbol_name: None,
            ..context
        };
        assert!(!operation.is_applicable(&context_no_name, None).await.unwrap());

        // Not applicable for non-structs
        let context_function = RefactoringContext {
            symbol_kind: Some(SymbolKind::Function),
            ..context
        };
        assert!(!operation.is_applicable(&context_function, None).await.unwrap());
    }

    #[tokio::test]
    async fn test_split_class_operation_execute_insufficient_fields() {
        let operation = SplitClassOperation;

        // Create a struct with insufficient fields
        let test_content = r#"
pub struct SmallStruct {
    value: i32,
}
"#;
        let temp_file = create_temp_rust_file(test_content);
        let file_path = temp_file.path().to_string_lossy().to_string();

        let context = RefactoringContext {
            file_path,
            cursor_line: 1,
            cursor_character: 0,
            selection: None,
            symbol_name: Some("SmallStruct".to_string()),
            symbol_kind: Some(SymbolKind::Struct),
        };

        let options = RefactoringOptions::default();

        let result = operation.execute(&context, &options).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("only 1 fields"));
    }

    #[tokio::test]
    async fn test_merge_classes_operation_placeholder() {
        let operation = MergeClassesOperation;

        assert_eq!(operation.name(), "Merge Classes");
        assert_eq!(operation.description(), "Merges multiple classes into one");
        assert_eq!(operation.refactoring_type(), RefactoringType::MergeClasses);

        let context = RefactoringContext::default();
        let result = operation.execute(&context, &RefactoringOptions::default()).await.unwrap();
        assert!(result.success);
        assert!(result.warnings.iter().any(|w| w.contains("requires implementation")));
    }

    #[tokio::test]
    async fn test_extract_class_operation_placeholder() {
        let operation = ExtractClassOperation;

        assert_eq!(operation.name(), "Extract Class");
        assert_eq!(operation.description(), "Extracts a class from existing code");
        assert_eq!(operation.refactoring_type(), RefactoringType::ExtractClass);

        let context = RefactoringContext::default();
        let result = operation.execute(&context, &RefactoringOptions::default()).await.unwrap();
        assert!(result.success);
        assert!(result.warnings.iter().any(|w| w.contains("requires implementation")));
    }

    #[tokio::test]
    async fn test_encapsulate_field_operation_placeholder() {
        let operation = EncapsulateFieldOperation;

        assert_eq!(operation.name(), "Encapsulate Field");
        assert_eq!(operation.description(), "Encapsulates a field with getter/setter methods");
        assert_eq!(operation.refactoring_type(), RefactoringType::EncapsulateField);

        let context = RefactoringContext::default();
        let result = operation.execute(&context, &RefactoringOptions::default()).await.unwrap();
        assert!(result.success);
        assert!(result.warnings.iter().any(|w| w.contains("requires implementation")));
    }

    #[tokio::test]
    async fn test_generate_getters_setters_operation_placeholder() {
        let operation = GenerateGettersSettersOperation;

        assert_eq!(operation.name(), "Generate Getters/Setters");
        assert_eq!(operation.description(), "Generates getter and setter methods for fields");
        assert_eq!(operation.refactoring_type(), RefactoringType::GenerateGettersSetters);

        let context = RefactoringContext::default();
        let result = operation.execute(&context, &RefactoringOptions::default()).await.unwrap();
        assert!(result.success);
        assert!(result.warnings.iter().any(|w| w.contains("requires implementation")));
    }

    #[tokio::test]
    async fn test_merge_classes_operation_is_not_applicable() {
        let operation = MergeClassesOperation;
        let context = RefactoringContext::default();
        assert!(!operation.is_applicable(&context, None).await.unwrap());
    }

    #[tokio::test]
    async fn test_extract_class_operation_is_not_applicable() {
        let operation = ExtractClassOperation;
        let context = RefactoringContext::default();
        assert!(!operation.is_applicable(&context, None).await.unwrap());
    }

    #[tokio::test]
    async fn test_encapsulate_field_operation_is_not_applicable() {
        let operation = EncapsulateFieldOperation;
        let context = RefactoringContext::default();
        assert!(!operation.is_applicable(&context, None).await.unwrap());
    }

    #[tokio::test]
    async fn test_generate_getters_setters_operation_is_not_applicable() {
        let operation = GenerateGettersSettersOperation;
        let context = RefactoringContext::default();
        assert!(!operation.is_applicable(&context, None).await.unwrap());
    }
}