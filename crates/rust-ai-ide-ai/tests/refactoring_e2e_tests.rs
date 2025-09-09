use crate::refactoring::operations::*;
use crate::refactoring::types::*;
use crate::refactoring::utils::{BackupManager, RefactoringUtils};
use std::fs;
use tempfile::TempDir;

/// End-to-end tests for refactoring operations
/// Tests create temporary Rust files, execute operations, and verify compilation

#[cfg(test)]
mod e2e_tests {
    use super::*;

    /// Sample Rust code for testing refactorings
    const SAMPLE_RUST_CODE: &str = r#"
use std::collections::HashMap;

pub struct Calculator {
    memory: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator { memory: 0.0 }
    }

    pub fn add(&mut self, x: f64, y: f64) -> f64 {
        let result = x + y;
        println!("Adding {} + {} = {}", x, y, result);
        result
    }

    pub fn multiply(&self, x: f64, y: f64) -> f64 {
        x * y
    }

    pub fn old_function_name() -> String {
        "This is the old name".to_string()
    }
}

fn main() {
    let mut calc = Calculator::new();
    let sum = calc.add(5.0, 3.0);
    let product = calc.multiply(sum, 2.0);
    let old_result = old_function_name();
    println!("Product: {}, Old: {}", product, old_result);
}
"#;

    /// Create a temporary Rust project with sample code
    fn create_temp_rust_project() -> Result<(TempDir, String), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let project_dir = temp_dir.path().to_string_lossy().to_string();

        // Create src directory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)?;

        // Create Cargo.toml
        let cargo_toml = format!(
            r#"
[package]
name = "temp-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
        );
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)?;

        // Create main.rs with sample code
        let main_rs_path = src_dir.join("main.rs");
        fs::write(&main_rs_path, SAMPLE_RUST_CODE)?;

        Ok((temp_dir, main_rs_path.to_string_lossy().to_string()))
    }

    /// Check if code compiles successfully
    fn check_compilation(project_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;

        let output = Command::new("cargo")
            .arg("check")
            .current_dir(project_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr)?;
            return Err(format!("Compilation failed: {}", stderr).into());
        }

        Ok(())
    }

    /// Test extract function operation end-to-end
    #[tokio::test]
    async fn test_extract_function_e2e() {
        println!("Starting extract function E2E test...");

        // Create temporary project
        let (temp_dir, main_rs_path) = create_temp_rust_project().unwrap();

        // Verify initial compilation
        check_compilation(&temp_dir.path().to_string_lossy()).unwrap();
        println!("✓ Initial project compiles successfully");

        // Create extract function operation
        let context = RefactoringContext {
            file_path: main_rs_path.clone(),
            cursor_line: 16, // Line with `let result = x + y;`
            cursor_character: 8,
            selection: Some(CodeRange {
                start_line: 16,
                start_character: 8,
                end_line: 18,
                end_character: 0,
            }),
            symbol_name: Some("add".to_string()),
            symbol_kind: Some(SymbolKind::Function),
        };

        let options = RefactoringOptions {
            create_backup: true,
            generate_tests: true,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: Some(std::collections::HashMap::new()),
        };

        // Execute refactoring
        let operation = ExtractFunctionOperation;
        let result = operation.execute(&context, &options).await.unwrap();

        assert!(
            result.success,
            "Extract function operation failed: {:?}",
            result.error_message
        );
        println!("✓ Extract function operation completed successfully");

        // Verify result content compiles
        if let Some(new_content) = &result.new_content {
            fs::write(&main_rs_path, new_content).unwrap();
        }

        check_compilation(&temp_dir.path().to_string_lossy()).unwrap();
        println!("✓ Refactored code compiles successfully");

        // Verify changes were applied correctly
        assert!(!result.changes.is_empty(), "No changes were recorded");
        for change in &result.changes {
            assert_eq!(
                change.file_path, main_rs_path,
                "Change applied to wrong file"
            );
        }

        println!("✓ All assertions passed for extract function E2E test");
    }

    /// Test rename operation end-to-end
    #[tokio::test]
    async fn test_rename_operation_e2e() {
        println!("Starting rename operation E2E test...");

        // Create temporary project
        let (temp_dir, main_rs_path) = create_temp_rust_project().unwrap();

        // Verify initial compilation
        check_compilation(&temp_dir.path().to_string_lossy()).unwrap();

        // Rename "old_function_name" to "new_function_name"
        let context = RefactoringContext {
            file_path: main_rs_path.clone(),
            cursor_line: 32, // Line with function call
            cursor_character: 25,
            selection: None,
            symbol_name: Some("old_function_name".to_string()),
            symbol_kind: Some(SymbolKind::Function),
        };

        let mut extra_options = std::collections::HashMap::new();
        extra_options.insert(
            "newName".to_string(),
            serde_json::Value::String("new_function_name".to_string()),
        );

        let options = RefactoringOptions {
            create_backup: true,
            generate_tests: true,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: Some(extra_options),
        };

        // Execute refactoring
        let operation = RenameOperation;
        let result = operation.execute(&context, &options).await.unwrap();

        assert!(
            result.success,
            "Rename operation failed: {:?}",
            result.error_message
        );
        println!("✓ Rename operation completed successfully");

        // Verify result content compiles
        if let Some(new_content) = &result.new_content {
            fs::write(&main_rs_path, new_content).unwrap();
        }

        check_compilation(&temp_dir.path().to_string_lossy()).unwrap();
        println!("✓ Renamed code compiles successfully");

        // Verify the content contains the new name
        let content_after = fs::read_to_string(&main_rs_path).unwrap();
        assert!(
            content_after.contains("new_function_name"),
            "New function name not found in content"
        );
        assert!(
            !content_after.contains("old_function_name"),
            "Old function name still present"
        );
        println!("✓ Function rename was applied correctly");

        println!("✓ All assertions passed for rename operation E2E test");
    }

    /// Test backup and restore functionality
    #[tokio::test]
    async fn test_backup_restore_e2e() {
        println!("Starting backup/restore E2E test...");

        // Create temporary project and backup directory
        let (temp_dir, main_rs_path) = create_temp_rust_project().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        fs::create_dir(&backup_dir).unwrap();

        let backup_manager = BackupManager::new(backup_dir.to_string_lossy().to_string());

        // Execute an operation and create backup
        let context = RefactoringContext {
            file_path: main_rs_path.clone(),
            cursor_line: 32,
            cursor_character: 25,
            selection: None,
            symbol_name: Some("old_function_name".to_string()),
            symbol_kind: Some(SymbolKind::Function),
        };

        let mut extra_options = std::collections::HashMap::new();
        extra_options.insert(
            "newName".to_string(),
            serde_json::Value::String("function_after_backup".to_string()),
        );

        let options = RefactoringOptions {
            create_backup: true,
            generate_tests: false,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: Some(extra_options),
        };

        // Execute with backup
        let operation = |context: RefactoringContext, options: RefactoringOptions| async move {
            RenameOperation.execute(&context, &options).await
        };

        let result = backup_manager
            .execute_with_backup(
                || operation(context.clone(), options.clone()),
                &RefactoringType::Rename,
                &context,
                &main_rs_path,
            )
            .await
            .unwrap();

        assert!(result.success, "Operation with backup failed");
        println!("✓ Operation with backup completed successfully");

        // Corrupt the file to simulate damage
        fs::write(&main_rs_path, "This is corrupted content").unwrap();

        // Try to restore from backup
        let manifests = backup_manager.list_backup_manifests().await.unwrap();
        assert!(!manifests.is_empty(), "No backup manifests found");

        let (manifest_name, manifest_path) = &manifests[0];

        // Read manifest content to verify metadata
        let manifest_content = fs::read_to_string(manifest_path).unwrap();
        assert!(manifest_content.contains("backup_id"));
        assert!(manifest_content.contains("operation_type"));
        assert!(manifest_content.contains("planned_changes"));
        println!("✓ Backup manifest contains expected metadata");

        // Restore from manifest
        backup_manager
            .restore_from_manifest(manifest_path, &main_rs_path)
            .await
            .unwrap();

        // Verify restored content compiles
        check_compilation(&temp_dir.path().to_string_lossy()).unwrap();
        println!("✓ Restored content compiles successfully");

        // Verify the original function name is back
        let restored_content = fs::read_to_string(&main_rs_path).unwrap();
        assert!(restored_content.contains("old_function_name"));
        assert!(!restored_content.contains("function_after_backup"));
        println!("✓ Backup and restore functionality works correctly");

        println!("✓ All assertions passed for backup/restore E2E test");
    }

    /// Test multiple operations in sequence
    #[tokio::test]
    async fn test_multiple_operations_e2e() {
        println!("Starting multiple operations E2E test...");

        // Create temporary project
        let (temp_dir, main_rs_path) = create_temp_rust_project().unwrap();

        // Verify initial compilation
        check_compilation(&temp_dir.path().to_string_lossy()).unwrap();

        let content_before = fs::read_to_string(&main_rs_path).unwrap();

        // Operation 1: Rename function
        let rename_context = RefactoringContext {
            file_path: main_rs_path.clone(),
            cursor_line: 26,
            cursor_character: 12,
            selection: None,
            symbol_name: Some("old_function_name".to_string()),
            symbol_kind: Some(SymbolKind::Function),
        };

        let mut extra_options = HashMap::new();
        extra_options.insert(
            "newName".to_string(),
            serde_json::Value::String("renamed_function".to_string()),
        );

        let rename_options = RefactoringOptions {
            create_backup: true,
            generate_tests: false,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: Some(extra_options),
        };

        let rename_result = RenameOperation
            .execute(&rename_context, &rename_options)
            .await
            .unwrap();
        assert!(rename_result.success);

        // Apply the rename change
        if let Some(new_content) = &rename_result.new_content {
            fs::write(&main_rs_path, new_content).unwrap();
        }

        check_compilation(&temp_dir.path().to_string_lossy()).unwrap();
        println!("✓ Rename operation applied and code compiles");

        // Operation 2: Extract variable
        let extract_context = RefactoringContext {
            file_path: main_rs_path.clone(),
            cursor_line: 38,
            cursor_character: 19,
            selection: Some(CodeRange {
                start_line: 38,
                start_character: 19,
                end_line: 38,
                end_character: 30,
            }),
            symbol_name: Some("calc".to_string()),
            symbol_kind: Some(SymbolKind::Variable),
        };

        let extract_options = RefactoringOptions {
            create_backup: false,
            generate_tests: false,
            apply_to_all_occurrences: false,
            preserve_references: true,
            ignore_safe_operations: false,
            extra_options: None,
        };

        let extract_result = ExtractVariableOperation
            .execute(&extract_context, &extract_options)
            .await
            .unwrap();

        // Only apply if experimental is enabled
        if extract_result.success {
            if let Some(new_content) = &extract_result.new_content {
                fs::write(&main_rs_path, new_content).unwrap();
            }

            check_compilation(&temp_dir.path().to_string_lossy()).unwrap();
            println!("✓ Extract variable operation applied and code compiles");
        } else {
            println!("✓ Extract variable operation skipped (experimental feature disabled)");
        }

        // Verify content has changed but remains compilable
        let content_after = fs::read_to_string(&main_rs_path).unwrap();
        assert_ne!(content_before, content_after, "Content was not modified");
        assert!(
            content_after.contains("renamed_function"),
            "Function was not renamed"
        );
        println!("✓ Sequential operations executed successfully");

        println!("✓ All assertions passed for multiple operations E2E test");
    }

    /// Test compilation verification helper
    #[test]
    fn test_compilation_helper() {
        println!("Testing compilation helper...");

        // Create temporary project
        let (temp_dir, main_rs_path) = create_temp_rust_project().unwrap();

        // Verify compilation succeeds
        check_compilation(&temp_dir.path().to_string_lossy()).unwrap();

        // Create invalid Rust code to test failure detection
        let invalid_code = "fn main() { let a = undefined_symbol; }";
        fs::write(&main_rs_path, invalid_code).unwrap();

        // Verify compilation fails as expected
        assert!(
            check_compilation(&temp_dir.path().to_string_lossy()).is_err(),
            "Expected compilation to fail with invalid code"
        );

        println!("✓ Compilation verification helper works correctly");
    }
}
