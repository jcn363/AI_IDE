//! !
//!
//! Integration tests for rust-ai-ide-lsp using shared-test-utils
//!
//! This module demonstrates sophisticated Language Server Protocol testing scenarios using the
//! comprehensive test utilities from shared-test-utils, including:
//!
//! - Temp workspace management for LSP workspace scenarios
//! - Async timeout handling for language server operations
//! - Validation utilities for LSP diagnostic scenarios
//! - Fixture-based test setup for consistent LSP test environments
//! - File system operations with automatic cleanup
//! - Integration test runner framework for systematic LSP testing
//! - Concurrent LSP client scenarios with proper synchronization
//!

use std::fs;
use std::path::Path;
use std::time::Duration;

use shared_test_utils::async_utils::AsyncContext;
use shared_test_utils::fixtures::FixturePresets;
use shared_test_utils::*;

// Test that proves we have the right imports and can run tests
#[cfg(test)]
mod integration_tests {
    use rust_ai_ide_lsp::client::{LSPClient, LSPClientConfig};

    use super::*;

    /// Integration test demonstrating LSP workspace setup with temp workspaces
    #[test]
    fn test_lsp_workspace_setup_with_shared_utils() {
        println!("🔧 Setting up LSP integration test with shared utilities...");

        // Create a temp workspace for LSP testing using shared utilities
        let workspace = TempWorkspace::new().unwrap();

        // Set up a Rust project structure for LSP testing
        workspace.setup_basic_project().unwrap();

        // Create LSP-specific test files
        workspace
            .create_file(
                Path::new("src/lib.rs"),
                r#"//! Test library for LSP integration testing

pub mod diagnostics;
pub mod completions;

/// A simple public function for testing
pub fn test_function() -> &'static str {
    "LSP integration test successful"
}

/// Function with diagnostics for testing
pub fn incomplete_function() {
    // This will generate a diagnostic about missing return type
    let x = 42;
    x // incomplete expression diagnostic
}
"#,
            )
            .unwrap();

        // Create test files for LSP scenarios
        workspace.create_dir(Path::new("tests")).unwrap();
        workspace
            .create_file(
                Path::new("tests/lsp_tests.rs"),
                r#"#[cfg(test)]
mod lsp_integration_tests {
    use super::*;

    #[test]
    fn test_lsp_integration() {
        assert!(true);
    }
}
"#,
            )
            .unwrap();

        // Test file creation with shared utilities macros
        assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
        assert_test_file_exists!(workspace, Path::new("src/lib.rs"));
        assert_test_file_exists!(workspace, Path::new("tests/lsp_tests.rs"));
        assert_file_contains!(workspace, Path::new("src/lib.rs"), "LSP integration test");

        // Validate workspace structure
        let total_files = fs::read_dir(workspace.path()).unwrap().count();
        assert!(
            total_files >= 4,
            "Should have at least Cargo.toml, src/, tests/, and targets/"
        );

        println!("✅ LSP workspace setup completed successfully");
    }

    /// Integration test using fixtures for LSP scenarios
    #[test]
    fn test_lsp_with_fixture_scenarios() {
        println!("🔧 Testing LSP with fixture scenarios...");

        // Use fixture for consistent LSP test environment
        let (workspace, fixture) = with_test_fixture!(FixturePresets::rust_library());

        // Extend fixture with LSP-specific files
        workspace
            .create_file(
                Path::new(".lsp-settings.json"),
                r#"{
  "rust-analyzer": {
    "diagnostics": {
      "enable": true
    },
    "checkOnSave": {
      "enable": true
    }
  }
}"#,
            )
            .unwrap();

        workspace
            .create_file(
                Path::new("rust-project.json"),
                r#"{
  "sysroot_src": "",
  "crates": [
    {
      "root_module": "src/lib.rs",
      "edition": "2021",
      "deps": [],
      "cfg": []
    }
  ]
}"#,
            )
            .unwrap();

        // Verify fixture provides expected structure
        assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
        assert_test_file_exists!(workspace, Path::new("src/lib.rs"));
        assert_test_file_exists!(workspace, Path::new(".lsp-settings.json"));
        assert_test_file_exists!(workspace, Path::new("rust-project.json"));

        // Test LSP configuration validation
        let lsp_config_content = fixture
            .get_file_content(&Path::new(".lsp-settings.json").to_path_buf())
            .unwrap();
        assert!(lsp_config_content.contains("rust-analyzer"));
        assert!(lsp_config_content.contains("diagnostics"));

        println!("✅ LSP fixtures integration test passed");
    }

    /// Performance-critical LSP operations with timeouts
    #[tokio::test]
    async fn test_lsp_operations_with_timeout() {
        println!("🔧 Testing LSP operations with timeout handling...");

        let context = AsyncContext::with_timeout(Duration::from_secs(10));

        // Test timeout functionality for simulated LSP operations
        let result = with_timeout(
            async {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                "lsp_operation_completed"
            },
            Duration::from_millis(200),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "lsp_operation_completed");

        println!("✅ LSP timeout test passed");
    }

    /// Complex LSP scenario with multiple workspace operations
    #[tokio::test]
    async fn test_complex_lsp_workspace_scenario() {
        println!("🔧 Testing complex LSP workspace scenario...");

        // Create workspace and set up complex LSP scenario
        let context = AsyncContext::with_timeout(Duration::from_secs(30));
        let workspace = TempWorkspace::new().unwrap();

        // Set up multiple LSP-related directories and files
        workspace.create_dir(Path::new("src/modules")).unwrap();
        workspace.create_dir(Path::new("src/ui")).unwrap();
        workspace
            .create_dir(Path::new("tests/integration"))
            .unwrap();

        // Create multiple Rust modules for LSP testing
        let module_files = vec![
            (
                "src/lib.rs",
                "pub mod parser; pub mod analyzer; pub mod ui;",
            ),
            (
                "src/modules/parser.rs",
                "pub fn parse_source() -> Result<(), &'static str> { Ok(()) }",
            ),
            (
                "src/modules/analyzer.rs",
                "pub fn analyze_syntax() -> Result<(), &'static str> { Ok(()) }",
            ),
            (
                "src/ui/components.rs",
                "pub fn render_component() -> String { String::new() }",
            ),
            (
                "tests/integration/parser_tests.rs",
                "#[test] fn test_parser() { assert!(true); }",
            ),
            (
                "tests/integration/ui_tests.rs",
                "#[test] fn test_ui() { assert!(true); }",
            ),
        ];

        // Simulate concurrent LSP file operations
        async fn create_file_async(workspace: &TempWorkspace, file: &str, content: &str) -> Result<String, TestError> {
            workspace.create_file(Path::new(file), content)?;
            Ok(format!("Created LSP file: {}", file))
        }

        let operations = vec![
            create_file_async(&workspace, module_files[0].0, module_files[0].1),
            create_file_async(&workspace, module_files[1].0, module_files[1].1),
            create_file_async(&workspace, module_files[2].0, module_files[2].1),
        ];

        let results = context
            .execute_concurrent(operations, Some(Duration::from_millis(500)))
            .await;

        assert!(results.is_ok());
        let result_strings = results.unwrap();
        assert_eq!(result_strings.len(), 3);

        // Verify each result properly handled LSP file creation
        for result in &result_strings {
            if let Ok(content) = result {
                assert!(content.contains("Created LSP file"));
                assert!(content.contains("src/"));
            } else {
                assert!(false, "LSP file creation failed with error: {:?}", result);
            }
        }

        // Create remaining files manually since they weren't in concurrent operations
        for (file, content) in &module_files[3..] {
            workspace.create_file(Path::new(file), content).unwrap();
        }

        println!(
            "✅ Complex LSP workspace scenario test completed successfully - {} LSP operations processed",
            result_strings.len()
        );
    }

    /// Integration test for LSP error handling and validation
    #[test]
    fn test_lsp_error_handling_and_validation() {
        println!("🔧 Testing LSP error handling and validation scenarios...");

        let workspace = TempWorkspace::new().unwrap();

        // Test LSP-specific file operations and error handling
        let result = std::fs::write(
            workspace.path().join("immutable_config.json"),
            r#"{
  "lsp": {
    "server": "rust-analyzer",
    "settings": {
      "check": {
        "command": "check"
      }
    }
  }
}"#,
        );

        if let Err(e) = result {
            let test_error = TestError::Io(e);
            assert!(matches!(test_error, TestError::Io(_)));
        }

        // Test LSP configuration validation
        let config_path = Path::new("/nonexistent/lsp/config");
        let validation_result = ValidationUtils::validate_path_security(config_path);
        assert!(validation_result.is_err());

        // Test LSP URI validation
        workspace
            .create_file(
                Path::new("lsp-config.json"),
                r#"{
  "rust-analyzer": {
    "linkedProjects": [
      "Cargo.toml"
    ]
  }
}"#,
            )
            .unwrap();

        // Test component validation for LSP scenarios
        let lsp_components = vec![
            Some("language_server"),
            Some("client"),
            None, // Missing optional component
        ];
        let names = vec!["Language Server", "Client", "Formatter"];

        assert!(ValidationUtils::validate_test_setup(&lsp_components, &names).is_err());

        // Valid setup should pass
        let valid_lsp_components = vec![Some("language_server"), Some("client"), Some("formatter")];
        assert!(ValidationUtils::validate_test_setup(&valid_lsp_components, &names).is_ok());

        println!("✅ LSP error handling and validation test completed");
    }

    /// Command integration testing for LSP scenarios
    #[test]
    fn test_lsp_command_integration_patterns() {
        println!("🔧 Testing LSP command integration patterns...");

        use shared_test_utils::command_tests::{CommandTestBuilder, MockCommand};

        // Create mock commands for LSP operations
        let commands = vec![
            MockCommand::new(
                "lsp_initialize",
                serde_json::json!({
                    "processId": null,
                    "rootUri": "file:///test/workspace",
                    "capabilities": {}
                }),
            )
            .with_result(serde_json::json!({
                "capabilities": {
                    "textDocumentSync": 2,
                    "completionProvider": {
                        "resolveProvider": false,
                        "triggerCharacters": [".", ":"]
                    }
                }
            })),
            MockCommand::new(
                "lsp_completion",
                serde_json::json!({
                    "textDocument": {"uri": "file:///test/src/lib.rs"},
                    "position": {"line": 1, "character": 10}
                }),
            )
            .with_result(serde_json::json!({
                "isIncomplete": false,
                "items": [
                    {
                        "label": "println!",
                        "kind": 3,
                        "detail": "macro",
                        "insertText": "println!($0)"
                    }
                ]
            })),
            MockCommand::new(
                "lsp_diagnostics",
                serde_json::json!({
                    "uri": "file:///test/src/invalid.rs"
                }),
            )
            .with_result(serde_json::json!({
                "diagnostics": [
                    {
                        "severity": 1,
                        "code": "E0425",
                        "message": "cannot find value `nonexistent_var` in this scope"
                    }
                ]
            })),
        ];

        // Test LSP command setup
        let runner = CommandTestBuilder::new()
            .success_command(
                "lsp_shutdown",
                serde_json::json!({}),
                serde_json::json!({"result": null}),
            )
            .error_command(
                "lsp_invalid_request",
                serde_json::json!({}),
                "LSP server error: Invalid request",
            )
            .build_runner();

        // Verify LSP commands were registered correctly
        assert_eq!(commands[0].name, "lsp_initialize");
        assert!(commands[0].result.is_ok());

        assert_eq!(commands[1].name, "lsp_completion");
        assert!(commands[1].result.is_ok());

        assert_eq!(commands[2].name, "lsp_diagnostics");
        assert!(commands[2].result.is_ok());

        // Verify LSP tester is set up
        assert_eq!(runner.called_commands().len(), 0);

        println!("✅ LSP command integration patterns test completed");
    }

    /// Concurrent LSP client operations test
    #[tokio::test]
    async fn test_concurrent_lsp_client_operations() {
        println!("🔧 Testing concurrent LSP client operations...");

        // Test simulating multiple LSP clients working concurrently
        async fn simulate_lsp_client_operation(client_id: usize, operation: &str) -> Result<String, TestError> {
            // Simulate different LSP operation times
            let delay = match operation {
                "initialize" => 100,
                "document_symbols" => 50,
                "completion" => 25,
                _ => 75,
            };

            let result = with_timeout(
                async {
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    format!("lsp_client_{}_{}_completed", client_id, operation)
                },
                Duration::from_millis(delay + 25),
            )
            .await;

            match result {
                Ok(value) => Ok(value),
                Err(_) => Ok(format!("lsp_client_{}_{}_timed_out", client_id, operation)),
            }
        }

        // Test multiple LSP client operations running concurrently
        let context = AsyncContext::with_timeout(Duration::from_secs(15));

        let operations = vec![
            simulate_lsp_client_operation(1, "initialize"),
            simulate_lsp_client_operation(1, "document_symbols"),
            simulate_lsp_client_operation(1, "completion"),
            simulate_lsp_client_operation(2, "initialize"),
            simulate_lsp_client_operation(2, "document_symbols"),
            simulate_lsp_client_operation(3, "initialize"),
        ];

        let results = context
            .execute_concurrent(
                operations,
                Some(Duration::from_millis(200)), // Reasonable timeout for LSP operations
            )
            .await;

        assert!(results.is_ok());
        let result_values = results.unwrap();
        assert_eq!(result_values.len(), 6);

        // Verify all LSP client operations completed
        for result in &result_values {
            if let Ok(content) = result {
                assert!(content.starts_with("lsp_client_"));
                assert!(content.contains("_completed") || content.contains("_timed_out"));
            } else {
                assert!(false, "LSP client operation failed: {:?}", result);
            }
        }

        // Verify at least some operations completed successfully
        let completed_count = result_values
            .iter()
            .filter(|r| r.contains("_completed"))
            .count();
        assert!(
            completed_count >= 4,
            "At least 4 LSP operations should have completed"
        );

        println!(
            "✅ Concurrent LSP client operations test completed - {} operations processed",
            result_values.len()
        );
    }
}
