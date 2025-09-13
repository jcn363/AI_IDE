//! Integration tests for rust-ai-ide-core using shared-test_utils

use std::path::Path;

use shared_test_utils::error::TestResult;
use shared_test_utils::fixtures::FixturePresets;
use shared_test_utils::*;

/// Integration test demonstrating TempWorkspace for file operations
#[test]
fn test_file_manager_with_temp_workspace() {
    let workspace = TempWorkspace::new().unwrap();

    // Create a simple test scenario using Path
    let config_path = std::path::Path::new("config.toml");
    let src_path = std::path::Path::new("src/lib.rs");

    workspace
        .create_file(
            config_path,
            r#"[project]
name = "test-project"
version = "0.1.0"
path = "src/lib.rs""#,
        )
        .unwrap();

    workspace.create_dir(std::path::Path::new("src")).unwrap();
    workspace
        .create_file(
            src_path,
            r#"pub fn hello() -> &'static str {
    "Hello from Rust AI IDE Core!"
}"#,
        )
        .unwrap();

    // Verify files exist using macro
    assert_test_file_exists!(workspace, config_path);
    assert_test_file_exists!(workspace, src_path);
    assert_file_contains!(workspace, src_path, "Hello from");

    // Test content validation
    assert!(ValidationUtils::validate_content("Hello from Rust AI IDE Core!", &["Hello"]).is_ok());
}

/// Integration test using test fixtures for consistent test setups
#[test]
fn test_core_with_fixtures() {
    let (workspace, fixture) = with_test_fixture!(FixturePresets::rust_library());

    // Use the fixture to set up test data
    assert!(workspace.file_exists(std::path::Path::new("Cargo.toml")));
    assert!(workspace.file_exists(std::path::Path::new("src/lib.rs")));

    // Validate fixture content
    let cargo_content = fixture
        .get_file_content(&std::path::Path::new("Cargo.toml").to_path_buf())
        .unwrap();
    assert!(cargo_content.contains("name"));
    assert!(cargo_content.contains("version"));
}

/// Async integration tests demonstrating timeout handling
#[tokio::test]
async fn test_async_operations_with_timeout() {
    // Test successful async operation
    let result = with_timeout(
        async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            "async_result"
        },
        tokio::time::Duration::from_millis(100),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "async_result");

    // Test concurrent operations with individual task creation
    async fn create_task(name: &str) -> Result<String, TestError> {
        Ok(format!("task_{}_result", name))
    }

    let concurrent_result = run_concurrent(vec![create_task("1"), create_task("2"), create_task("3")]).await;

    assert!(concurrent_result.is_ok());
    let result_value = concurrent_result.unwrap();
    assert!(result_value.contains("task_") && result_value.contains("_result"));
}

/// Integration test for error handling with shared utilities
#[test]
fn test_error_handling_integration() {
    let workspace = TempWorkspace::new().unwrap();

    // Test error propagation
    let result = std::fs::write(workspace.path().join("readonly_test"), "data");
    if let Err(e) = result {
        let test_error = TestError::Io(e);
        assert!(matches!(test_error, TestError::Io(_)));
    }

    // Test validation errors
    let validation_result = ValidationUtils::validate_path_security(Path::new("/nonexistent/path"));
    assert!(validation_result.is_err());
}

/// Performance-sensitive integration test with timeouts
#[tokio::test]
async fn test_performance_critical_operations() {
    let context = AsyncContext::with_timeout(tokio::time::Duration::from_secs(2));

    let result = context
        .execute(async {
            // Simulate some core operation that should complete quickly
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            "operation_completed"
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "operation_completed");
}

/// Integration test combining multiple shared-test-utils features
#[test]
fn test_full_integration_scenario() {
    // Set up workspace with fixture
    let (workspace, _fixture) = with_test_fixture!(FixturePresets::rust_library());

    // Test file operations
    assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
    assert_test_file_exists!(workspace, Path::new("src/lib.rs"));

    // Test content validation across multiple files
    let files = ["Cargo.toml", "src/lib.rs"];

    for file in &files {
        let exists = workspace.file_exists(Path::new(file));
        assert!(exists, "File {} should exist", file);
    }

    // Test error handling
    let nonexistent_result = workspace.read_file(Path::new("nonexistent.txt"));
    assert!(nonexistent_result.is_err());
}

/// Benchmark-style integration test using async utilities
#[tokio::test]
async fn test_concurrent_processing_integration() {
    use shared_test_utils::async_utils::AsyncContext;

    // Create individual tasks using function approach to avoid type issues
    async fn create_simple_task(name: &str) -> &str {
        format!("task{}_completed", name).leak() // leak to return static string
    }

    // Use AsyncContext for concurrent execution with timeout
    let context = AsyncContext::with_timeout(tokio::time::Duration::from_millis(500));

    // Test individual task execution
    let result1 = context.execute(create_simple_task("1")).await;
    let result2 = context.execute(create_simple_task("2")).await;
    let result3 = context.execute(create_simple_task("3")).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    // Verify all tasks completed successfully
    assert!(result1.unwrap().contains("completed"));
    assert!(result2.unwrap().contains("completed"));
    assert!(result3.unwrap().contains("completed"));
}

/// Demonstration of Tauri command testing integration
#[test]
fn test_command_integration_patterns() {
    use shared_test_utils::command_tests::{CommandTestBuilder, MockCommand};

    // Create mock command for testing interactions with core commands
    let command = MockCommand::new(
        "analyze_project",
        serde_json::json!({
            "path": "/test/project",
            "deep_analysis": true
        }),
    )
    .with_result(serde_json::json!({
        "files_analyzed": 15,
        "complexity_score": 7.2,
        "issues_found": 3
    }));

    assert_eq!(command.name, "analyze_project");

    // Test command runner setup
    let runner = CommandTestBuilder::new()
        .success_command(
            "get_project_info",
            serde_json::json!({}),
            serde_json::json!({"name": "test-project", "modules": 3}),
        )
        .error_command(
            "get_invalid_project",
            serde_json::json!({}),
            "Project not found",
        )
        .build_runner();

    assert_eq!(runner.called_commands().len(), 0);
}

/// Integration test demonstrating workspace validation
#[test]
fn test_workspace_validation_integration() {
    let workspace = TempWorkspace::new().unwrap();

    // Create a realistic project structure
    workspace.setup_basic_project().unwrap();

    // Test various validation scenarios
    let components = vec![
        Some("main"),
        Some("ui"),
        Some("backend"),
        None, // Missing optional component
    ];

    let names = vec!["Main", "UI", "Backend", "Optional"];

    // Should fail due to None component
    assert!(ValidationUtils::validate_test_setup(&components, &names).is_err());

    // Should succeed with all components present
    let valid_components = vec![Some("main"), Some("ui"), Some("backend"), Some("optional")];

    assert!(ValidationUtils::validate_test_setup(&valid_components, &names).is_ok());
}
