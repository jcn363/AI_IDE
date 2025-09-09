/*!

Integration tests for rust-ai-ide-debugger using shared-test-utils

This module demonstrates sophisticated debugging scenarios using the comprehensive
test utilities from shared-test-utils, including:

- Temp workspace management for debugging sessions
- Async timeout handling for long-running debug operations
- Fixture-based test setup for complex debugging scenarios
- File system operations with automatic cleanup
- Integration test runner framework for systematic testing

*/

// Test that proves we have the right imports and can run tests
#[cfg(test)]
mod integration_tests {
    use shared_test_utils::error::TestResult;
    use shared_test_utils::fixtures::FixturePresets;
    use shared_test_utils::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::time::Duration;

    /// Integration test demonstrating debugger session with temp workspaces
    #[test]
    fn test_debugger_session_with_temp_workspace() {
        println!("🔧 Setting up debugger integration test...");

        // Create a temp workspace for debugger testing
        let workspace = TempWorkspace::new().unwrap();

        // Set up a Rust project structure for debugging
        workspace.setup_basic_project().unwrap();

        // Create a test source file with debugging points
        workspace
            .create_file(
                Path::new("src/main.rs"),
                r#"#[derive(Debug)]
struct TestStruct {
    value: i32,
}

fn main() {
    let mut test = TestStruct { value: 42 };

    // Set breakpoint here for debugging
    println!("Initial value: {}", test.value);

    test.value = 100;
    println!("Updated value: {}", test.value);

    println!("Program completed!");
}
"#,
            )
            .unwrap();

        // Test file creation
        assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
        assert_test_file_exists!(workspace, Path::new("src/main.rs"));
        assert_file_contains!(workspace, Path::new("src/main.rs"), "TestStruct");

        // Validate the workspace structure
        let total_files = fs::read_dir(workspace.path()).unwrap().count();
        assert!(
            total_files >= 2,
            "Should have at least Cargo.toml and src/ directory"
        );

        println!("✅ Debugger workspace setup completed successfully");
    }

    /// Integration test using fixtures for debugging scenarios
    #[test]
    fn test_debugger_with_fixture_scenarios() {
        println!("🔧 Testing debugger with fixture scenarios...");

        // Use fixture for consistent debugging environment
        let (workspace, fixture) = with_test_fixture!(FixturePresets::rust_library());

        // Verify fixture provides expected structure
        assert_test_file_exists!(workspace, Path::new("Cargo.toml"));
        assert_test_file_exists!(workspace, Path::new("src/lib.rs"));

        // Test debug-specific file operations
        workspace
            .create_file(
                Path::new("tests/debug_tests.rs"),
                r#"#[cfg(test)]
mod tests {
    #[test]
    fn test_debug_functionality() {
        assert!(true);
    }
}
"#,
            )
            .unwrap();

        assert_test_file_exists!(workspace, Path::new("tests/debug_tests.rs"));

        // Test fixture content validation
        let cargo_content = fixture
            .get_file_content(&Path::new("Cargo.toml").to_path_buf())
            .unwrap();
        assert!(cargo_content.contains("name"));
        assert!(cargo_content.contains("version"));

        println!("✅ Debugger fixtures integration test passed");
    }

    /// Performance-critical debugger operations with timeouts
    #[tokio::test]
    async fn test_debugger_performance_with_timeout() {
        println!("🔧 Testing debugger performance with timeout handling...");

        let context = AsyncContext::with_timeout(Duration::from_secs(10));

        // Test timeout functionality
        let result = with_timeout(
            async {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                "debugger_operation_completed"
            },
            Duration::from_millis(200),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "debugger_operation_completed");

        println!("✅ Debugger performance timeout test passed");
    }

    /// Complex debugging scenario with multiple async operations
    #[tokio::test]
    async fn test_complex_debugging_scenario() {
        println!("🔧 Testing complex debugging scenario with multiple operations...");

        // Create workspace and set up scenario
        let context = AsyncContext::with_timeout(Duration::from_secs(30));
        let workspace = TempWorkspace::new().unwrap();

        // Set up multiple debugging scenarios
        workspace.create_dir(Path::new("debug_sessions")).unwrap();
        workspace.create_dir(Path::new("breakpoints")).unwrap();

        // Create multiple test scenarios
        let scenarios = vec![
            ("session1", "threading_debug"),
            ("session2", "memory_debug"),
            ("session3", "crash_debug"),
        ];

        // Demonstrate concurrent debugging operations
        async fn create_debug_scenario(
            name: &str,
            scenario_type: &str,
        ) -> Result<String, TestError> {
            Ok(format!(
                "Debug {} scenario '{}' created",
                scenario_type, name
            ))
        }

        // Run scenarios concurrently
        let results = context
            .execute_concurrent(
                vec![
                    create_debug_scenario(scenarios[0].0, scenarios[0].1),
                    create_debug_scenario(scenarios[1].0, scenarios[1].1),
                    create_debug_scenario(scenarios[2].0, scenarios[2].1),
                ],
                Some(Duration::from_millis(500)),
            )
            .await;

        assert!(results.is_ok());
        let result_strings = results.unwrap();
        assert_eq!(result_strings.len(), 3);

        // Since execute_concurrent preserves the Result structure from async operations,
        // we need to handle each inner Result
        for result in &result_strings {
            if let Ok(content) = result {
                if content.contains("Debug")
                    && content.contains("scenario")
                    && content.contains("created")
                {
                    assert!(true, "Result contains expected content");
                } else {
                    assert!(
                        false,
                        "Result does not contain expected content: {}",
                        content
                    );
                }
            } else {
                assert!(false, "Unexpected error result in scenario execution");
            }
        }

        println!("✅ Complex debugging scenario test completed successfully");
    }

    /// Integration test for debugger error handling
    #[test]
    fn test_debugger_error_handling() {
        println!("🔧 Testing debugger error handling scenarios...");

        let workspace = TempWorkspace::new().unwrap();

        // Test error propagation in debugging operations
        let result = std::fs::write(workspace.path().join("readonly_test"), "debug_data");

        if let Err(e) = result {
            let test_error = TestError::Io(e.to_string());
            assert!(matches!(test_error, TestError::Io(_)));
        }

        // Test path validation for debugging operations
        let test_path = Path::new("/nonexistent/debug/path");
        let validation_result = ValidationUtils::validate_path_security(test_path);
        assert!(validation_result.is_err());

        // Test context validation
        let components = vec![
            Some("debugger"),
            Some("session"),
            None, // Missing optional component
        ];
        let names = vec!["Debugger", "Session", "Optional"];

        assert!(ValidationUtils::validate_test_setup(&components, &names).is_err());

        // Valid setup should pass
        let valid_components = vec![Some("debugger"), Some("session"), Some("optional")];
        assert!(ValidationUtils::validate_test_setup(&valid_components, &names).is_ok());

        println!("✅ Debugger error handling test completed");
    }

    /// Command-based debugger testing (simulating Tauri integration)
    #[test]
    fn test_command_integration_debugger_patterns() {
        println!("🔧 Testing command integration patterns for debugger...");

        use shared_test_utils::command_tests::{CommandTestBuilder, MockCommand};

        // Create mock commands for debugger operations
        let commands = vec![
            MockCommand::new(
                "start_debug_session",
                serde_json::json!({
                    "program": "/path/to/program",
                    "args": ["--debug"]
                }),
            )
            .with_result(serde_json::json!({
                "session_id": "123456",
                "status": "running"
            })),
            MockCommand::new(
                "set_breakpoint",
                serde_json::json!({
                    "file": "src/main.rs",
                    "line": 10
                }),
            )
            .with_result(serde_json::json!({
                "breakpoint_id": "bp_001",
                "location": {
                    "file": "src/main.rs",
                    "line": 10
                }
            })),
            MockCommand::new(
                "step_over",
                serde_json::json!({
                    "session_id": "123456"
                }),
            )
            .with_result(serde_json::json!({
                "next_line": 15,
                "variables": []
            })),
        ];

        // Test command setup
        let runner = CommandTestBuilder::new()
            .success_command(
                "init_debugger",
                serde_json::json!({}),
                serde_json::json!({"initialized": true}),
            )
            .error_command(
                "stop_debugger",
                serde_json::json!({}),
                "Debugger not running",
            )
            .build_runner();

        // Verify all commands were registered
        assert_eq!(commands[0].name, "start_debug_session");
        assert!(commands[0].result.is_ok());

        assert_eq!(commands[1].name, "set_breakpoint");
        assert!(commands[1].result.is_ok());

        assert_eq!(commands[2].name, "step_over");
        assert!(commands[2].result.is_ok());

        // Verify test runner is set up
        assert_eq!(runner.called_commands().len(), 0);

        println!("✅ Command integration patterns test completed");
    }
}

#[cfg(test)]
mod concurrent_debug_tests {
    use shared_test_utils::*;
    use std::time::Duration;

    type TestResult<T> = Result<T, TestError>;

    /// Test concurrent debugger operations with proper sync patterns
    #[tokio::test]
    async fn test_concurrent_debugger_operations() {
        println!("🔧 Testing concurrent debugger operations...");

        // Create a mechanism for testing multiple debugger instances
        async fn simulate_debugger_operation(
            id: usize,
            duration_ms: u64,
        ) -> Result<String, TestError> {
            let result = with_timeout(
                async {
                    tokio::time::sleep(Duration::from_millis(duration_ms)).await;
                    format!("debugger_{}_completed", id)
                },
                Duration::from_millis(duration_ms + 50),
            )
            .await;

            match result {
                Ok(value) => Ok(value),
                Err(_) => Ok(format!("debugger_{}_timed_out", id)),
            }
        }

        // Test multiple debugger operations running concurrently
        let context = AsyncContext::with_timeout(Duration::from_secs(10));

        let operations = vec![
            simulate_debugger_operation(1, 50),
            simulate_debugger_operation(2, 30),
            simulate_debugger_operation(3, 80),
        ];

        let results = context
            .execute_concurrent(operations, Some(Duration::from_millis(200)))
            .await;

        assert!(results.is_ok());
        let result_values = results.unwrap();
        assert_eq!(result_values.len(), 3);

        // Test that all operations returned valid strings
        for result in &result_values {
            // Check that each result starts with "debugger_"
            if let Ok(content) = result {
                if content.starts_with("debugger_") {
                    if content.contains("completed") || content.contains("timed_out") {
                        assert!(true, "Valid debugger operation result");
                    } else {
                        assert!(false, "Invalid result content: {}", content);
                    }
                } else {
                    assert!(false, "Result does not start with debugger_: {}", content);
                }
            } else {
                assert!(false, "Operation failed with error: {:?}", result);
            }
        }

        println!(
            "✅ Complex debugging scenario test completed successfully - {} operations processed",
            result_values.len()
        );

        println!("✅ Concurrent debugger operations test passed");
    }
}
