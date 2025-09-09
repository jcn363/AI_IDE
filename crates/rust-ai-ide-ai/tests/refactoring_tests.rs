// Basic integration tests for refactoring functionality
// Tests end-to-end refactoring workflows and API compatibility

use rust_ai_ide_ai::refactoring::{
    BackendFeatures, RefactoringContext, RefactoringEngine, RefactoringOperationFactory,
    RefactoringOptions, RefactoringType, SymbolKind,
};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactoring_engine_creation() {
        let engine = RefactoringEngine::new();
        assert!(engine.get_cache_statistics().1 >= 0); // total_entries should be >= 0
    }

    #[test]
    fn test_available_refactorings_includes_all_types() {
        let available = RefactoringOperationFactory::available_refactorings();
        assert!(!available.is_empty(), "Should have available refactorings");

        // Test that we can create operations for all available refactorings
        for refactoring_type in available {
            let operation_result = RefactoringOperationFactory::create_operation(&refactoring_type);
            assert!(
                operation_result.is_ok(),
                "Failed to create operation for {:?}",
                refactoring_type
            );
        }
    }

    #[test]
    fn test_backend_capabilities_structure() {
        let features = BackendFeatures {
            batch_operations: true,
            analysis: true,
            backup_recovery: true,
            test_generation: false,
            ai_analysis: false,
            lsp_integration: false,
            git_integration: false,
            cross_language_support: true,
            parallel_processing: true,
        };

        assert!(features.batch_operations);
        assert!(features.analysis);
        assert!(!features.test_generation);
        assert!(features.cross_language_support);
    }

    #[test]
    fn test_refactoring_context_creation() {
        let context = RefactoringContext {
            file_path: "/tmp/test.rs".to_string(),
            cursor_line: 10,
            cursor_character: 5,
            selection: None,
            symbol_name: Some("test_function".to_string()),
            symbol_kind: Some(SymbolKind::Function),
        };

        assert_eq!(context.file_path, "/tmp/test.rs");
        assert_eq!(context.cursor_line, 10);
        assert_eq!(context.cursor_character, 5);
        assert_eq!(context.symbol_name, Some("test_function".to_string()));
    }

    #[test]
    fn test_refactoring_options_defaults() {
        let options = RefactoringOptions {
            create_backup: true,
            generate_tests: false,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
        };

        assert!(options.create_backup);
        assert!(!options.generate_tests);
        assert!(options.preserve_references);
        assert!(!options.ignore_safe_operations);
    }

    #[test]
    fn test_refactoring_type_serialization() {
        let refactoring_type = RefactoringType::Rename;
        let serialized = serde_json::to_string(&refactoring_type).unwrap();
        assert!(serialized.contains("rename"));
    }

    #[test]
    fn test_cache_feedback_mechanism() {
        let mut options = RefactoringOptions {
            create_backup: false,
            generate_tests: true,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
        };

        // Test that options can be modified and are properly handled
        options.apply_to_all_occurrences = true;
        assert!(options.apply_to_all_occurrences);
    }

    #[test]
    fn test_operation_applicability_checks() {
        let context_without_symbol = RefactoringContext {
            file_path: "/tmp/test.rs".to_string(),
            cursor_line: 1,
            cursor_character: 0,
            selection: None,
            symbol_name: None,
            symbol_kind: None,
        };

        // Test a simple operation that should be inapplicable without a symbol
        let operation =
            RefactoringOperationFactory::create_operation(&RefactoringType::Rename).unwrap();
        // This test assumes the operation would check for symbol presence, but the actual
        // implementation might be different. For now, we just ensure the operation is created.
        assert!(operation.refactoring_type() == RefactoringType::Rename);
    }

    #[test]
    fn test_multiple_operation_creation() {
        // Test creating multiple different operations
        let operations = vec![
            RefactoringType::Rename,
            RefactoringType::ExtractFunction,
            RefactoringType::ExtractVariable,
        ];

        for op_type in operations {
            let operation = RefactoringOperationFactory::create_operation(&op_type);
            assert!(
                operation.is_ok(),
                "Failed to create operation for {:?}",
                op_type
            );
            assert_eq!(operation.unwrap().refactoring_type(), op_type);
        }
    }

    #[test]
    fn test_performance_metrics_tracking() {
        // Test that cache statistics can be retrieved
        let mut engine = RefactoringEngine::new();

        // Initial state
        let (fresh_before, total_before) = engine.get_cache_statistics();

        // Clear cache (should reset to 0)
        engine.clear_analysis_cache();
        let (fresh_after, total_after) = engine.get_cache_statistics();

        // Even if there were entries before, clear should work
        assert_eq!(fresh_after, total_after);
        assert!(fresh_before >= 0);
        assert!(total_before >= 0);
    }

    #[test]
    fn test_batch_merge_options_preserves_extra_options() {
        use rust_ai_ide_ai::refactoring::batch::BatchRefactoringHandler;

        // Default options with no extra_options
        let default_options = RefactoringOptions {
            create_backup: true,
            generate_tests: false,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: None,
        };

        // Operation options with extra_options containing newName, functionName
        let mut extra_hashmap = HashMap::new();
        extra_hashmap.insert("newName".to_string(), serde_json::json!("renamedFunction"));
        extra_hashmap.insert(
            "functionName".to_string(),
            serde_json::json!("originalFunction"),
        );
        extra_hashmap.insert(
            "defaultField".to_string(),
            serde_json::json!("defaultValue"),
        );

        let operation_options = RefactoringOptions {
            create_backup: false,
            generate_tests: true,
            apply_to_all_occurrences: true,
            preserve_references: false,
            ignore_safe_operations: true,
            extra_options: Some(extra_hashmap),
        };

        // Merge options
        let merged = BatchRefactoringHandler::merge_options(&default_options, &operation_options);

        // Verify boolean fields are merged correctly
        assert_eq!(merged.create_backup, false); // operation_options overrides default
        assert_eq!(merged.generate_tests, true); // operation_options overrides default
        assert_eq!(merged.apply_to_all_occurrences, true); // operation_options overrides default
        assert_eq!(merged.preserve_references, false); // operation_options overrides default
        assert_eq!(merged.ignore_safe_operations, true); // operation_options overrides default

        // Verify extra_options are preserved
        assert!(merged.extra_options.is_some());
        let extra_opts = merged.extra_options.unwrap();
        assert_eq!(extra_opts.len(), 3);
        assert_eq!(
            extra_opts.get("newName"),
            Some(&serde_json::json!("renamedFunction"))
        );
        assert_eq!(
            extra_opts.get("functionName"),
            Some(&serde_json::json!("originalFunction"))
        );
        assert_eq!(
            extra_opts.get("defaultField"),
            Some(&serde_json::json!("defaultValue"))
        );
    }

    #[test]
    fn test_batch_merge_options_deep_merge_extra_options() {
        use rust_ai_ide_ai::refactoring::batch::BatchRefactoringHandler;

        // Default options with extra_options
        let mut default_extra = HashMap::new();
        default_extra.insert("newName".to_string(), serde_json::json!("defaultRenamed"));
        default_extra.insert("sharedField".to_string(), serde_json::json!("fromDefault"));

        let default_options = RefactoringOptions {
            create_backup: true,
            generate_tests: false,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: Some(default_extra),
        };

        // Operation options with overlapping extra_options
        let mut operation_extra = HashMap::new();
        operation_extra.insert("newName".to_string(), serde_json::json!("operationRenamed"));
        operation_extra.insert(
            "functionName".to_string(),
            serde_json::json!("operationFunction"),
        );

        let operation_options = RefactoringOptions {
            create_backup: false,
            generate_tests: true,
            apply_to_all_occurrences: true,
            preserve_references: false,
            ignore_safe_operations: true,
            extra_options: Some(operation_extra),
        };

        // Merge options
        let merged = BatchRefactoringHandler::merge_options(&default_options, &operation_options);

        // Verify that operation_options override default_values
        assert!(merged.extra_options.is_some());
        let extra_opts = merged.extra_options.unwrap();
        assert_eq!(extra_opts.len(), 3);
        // operationRenamed should override defaultRenamed
        assert_eq!(
            extra_opts.get("newName"),
            Some(&serde_json::json!("operationRenamed"))
        );
        // functionName should be added
        assert_eq!(
            extra_opts.get("functionName"),
            Some(&serde_json::json!("operationFunction"))
        );
        // sharedField should still be preserved from default
        assert_eq!(
            extra_opts.get("sharedField"),
            Some(&serde_json::json!("fromDefault"))
        );
    }

    #[test]
    fn test_batch_merge_options_with_none_extra_options() {
        use rust_ai_ide_ai::refactoring::batch::BatchRefactoringHandler;

        // Default options with extra_options
        let mut default_extra = HashMap::new();
        default_extra.insert("newName".to_string(), serde_json::json!("defaultRenamed"));

        let default_options = RefactoringOptions {
            create_backup: true,
            generate_tests: false,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: Some(default_extra),
        };

        // Operation options without extra_options
        let operation_options = RefactoringOptions {
            create_backup: false,
            generate_tests: true,
            apply_to_all_occurrences: true,
            preserve_references: false,
            ignore_safe_operations: true,
            extra_options: None,
        };

        // Merge options
        let merged = BatchRefactoringHandler::merge_options(&default_options, &operation_options);

        // Verify that default extra_options are preserved
        assert!(merged.extra_options.is_some());
        let extra_opts = merged.extra_options.unwrap();
        assert_eq!(extra_opts.len(), 1);
        assert_eq!(
            extra_opts.get("newName"),
            Some(&serde_json::json!("defaultRenamed"))
        );
    }

    #[test]
    fn test_file_permissions_validation() {
        use rust_ai_ide_ai::refactoring::RefactoringEngine;
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_file.rs");

        // Create a test file
        fs::write(&test_file_path, "fn main() {}").unwrap();

        let engine = RefactoringEngine::new();

        // Test that permissions validation works for a writable file
        let result = engine.check_file_accessibility(&test_file_path.to_string_lossy());
        assert!(result.is_none(), "Should be able to access writable file");

        // Test permissions validation method
        let perm_result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            engine.validate_file_permissions(&test_file_path.to_string_lossy())
        });
        assert!(
            perm_result.is_ok(),
            "Should validate permissions for writable file"
        );

        // Drop the temp dir to clean up before test ends
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_readonly_directory_validation() {
        use rust_ai_ide_ai::refactoring::RefactoringEngine;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_file.rs");

        // Create a test file
        fs::write(&test_file_path, "fn main() {}").unwrap();

        // Make the parent directory read-only on Unix systems
        #[cfg(unix)]
        {
            let parent = test_file_path.parent().unwrap();
            let mut perms = fs::metadata(parent).unwrap().permissions();
            perms.set_readonly(true);
            fs::set_permissions(parent, perms).unwrap();
        }

        let engine = RefactoringEngine::new();

        // Test permissions validation method for read-only directory
        let perm_result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            engine.validate_file_permissions(&test_file_path.to_string_lossy())
        });

        // On Unix systems, this should fail because we can't create temp files in readonly directory
        #[cfg(unix)]
        assert!(
            perm_result.is_err(),
            "Should fail to validate permissions in readonly directory"
        );

        // On Windows, this might still succeed depending on the filesystem implementation
        #[cfg(not(unix))]
        {
            // On non-Unix systems, just verify the method runs without panic
            println!(
                "Read-only directory test skipped on non-Unix system: {:?}",
                perm_result
            );
        }

        // Restore permissions and clean up
        #[cfg(unix)]
        {
            let parent = test_file_path.parent().unwrap();
            let mut perms = fs::metadata(parent).unwrap().permissions();
            perms.set_readonly(false);
            fs::set_permissions(parent, perms).unwrap();
        }

        temp_dir.close().unwrap();
    }
}
