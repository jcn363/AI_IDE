use shared_test_utils::*;
use std::path::Path;
use std::collections::HashMap;
use serde_json;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_temp_workspace_creation() {
        let workspace = TempWorkspace::new().unwrap();
        assert!(workspace.path().exists());
        // Workspace automatically cleans up
    }

    #[test]
    fn test_test_error_creation() {
        let error = TestError::Async("test error message".to_string());
        assert!(matches!(error, TestError::Async(_)));
    }

    #[test]
    fn test_validation_error_constructors() {
        let error = ValidationError::path_validation("invalid path message");
        assert!(matches!(error, ValidationError::PathValidation { .. }));
    }
}

#[cfg(test)]
mod filesystem_tests {
    use super::*;
    use shared_test_utils::filesystem::TestScenario;

    #[test]
    fn test_workspace_basic_project() {
        let workspace = TempWorkspace::new().unwrap();
        workspace.setup_basic_project().unwrap();

        assert!(workspace.file_exists(Path::new("Cargo.toml")));
        assert!(workspace.file_exists(Path::new("src/lib.rs")));
        assert!(!workspace.file_exists(Path::new("src/main.rs")));
    }

    #[test]
    fn test_workspace_scenarios() {
        let workspace = TempWorkspace::new().unwrap();
        workspace.setup_scenario(TestScenario::WithTests).unwrap();

        assert!(workspace.file_exists(Path::new("Cargo.toml")));
        assert!(workspace.file_exists(Path::new("src/lib.rs")));
        assert!(workspace.file_exists(Path::new("tests/lib_test.rs")));
    }

    #[test]
    fn test_workspace_file_operations() {
        let workspace = TempWorkspace::new().unwrap();

        workspace.create_file(Path::new("test.txt"), "hello world").unwrap();
        let content = workspace.read_file(Path::new("test.txt")).unwrap();
        assert_eq!(content, "hello world");
        assert!(workspace.file_exists(Path::new("test.txt")));
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_validate_content() {
        assert!(ValidationUtils::validate_content("hello world", &["hello"]).is_ok());
        assert!(ValidationUtils::validate_content("hello world", &["missing"]).is_err());
    }

    #[test]
    fn test_validate_path_security() {
        let workspace = TempWorkspace::new().unwrap();
        let test_file = Path::new("test.txt");
        workspace.create_file(test_file, "test").unwrap();

        assert!(ValidationUtils::validate_path_security(&workspace.path().join("test.txt")).is_ok());
        assert!(ValidationUtils::validate_path_security(Path::new("nonexistent")).is_err());
    }

    #[test]
    fn test_validate_test_setup() {
        let components = vec![Some("comp1"), Some("comp2"), None];
        let names = vec!["A", "B", "C"];

        assert!(ValidationUtils::validate_test_setup(&components, &names).is_err());

        let valid_components = vec![Some("comp1"), Some("comp2"), Some("comp3")];
        assert!(ValidationUtils::validate_test_setup(&valid_components, &names).is_ok());
    }
}

#[cfg(test)]
mod fixtures_tests {
    use super::*;
    use shared_test_utils::fixtures::FixturePresets;

    #[test]
    fn test_fixture_builder() {
        let workspace = TempWorkspace::new().unwrap();

        let fixture = TestFixtureBuilder::new()
            .with_file("config.json", r#"{"test": true}"#)
            .build(&workspace).unwrap();

        assert!(workspace.file_exists(Path::new("config.json")));
        let content = fixture.get_file_content(&Path::new("config.json").to_path_buf()).unwrap();
        assert_eq!(content, r#"{"test": true}"#);
    }

    #[test]
    fn test_cargo_workspace_fixture() {
        let workspace = TempWorkspace::new().unwrap();
        let fixture = FixturePresets::cargo_workspace(&["member1"]).build(&workspace).unwrap();

        assert!(workspace.file_exists(Path::new("Cargo.toml")));
        assert!(workspace.file_exists(Path::new("member1/Cargo.toml")));
    }

    #[test]
    fn test_rust_library_fixture() {
        let workspace = TempWorkspace::new().unwrap();
        let fixture = FixturePresets::rust_library().build(&workspace).unwrap();

        assert!(workspace.file_exists(Path::new("Cargo.toml")));
        assert!(workspace.file_exists(Path::new("src/lib.rs")));
        assert!(fixture.files().count() >= 2);
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    use shared_test_utils::command_tests::{MockCommand, CommandTestBuilder};

    #[test]
    fn test_mock_command() {
        let command = MockCommand::new("test", serde_json::json!({"input": true}))
            .with_result(serde_json::json!({"result": true}));

        assert_eq!(command.name, "test");
        assert!(command.result.is_ok());
    }

    #[test]
    fn test_command_test_runner() {
        let mut runner = CommandTestBuilder::new()
            .success_command("get_data", serde_json::json!({}), serde_json::json!({"data": "test"}))
            .build_runner();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result: serde_json::Value = rt.block_on(async {
            runner.execute_command("get_data", &serde_json::json!({})).await.unwrap()
        });

        assert_eq!(result["data"], "test");
    }

    #[test]
    fn test_command_error_handling() {
        let mut runner = CommandTestBuilder::new()
            .error_command("failing", serde_json::json!({}), "error msg")
            .build_runner();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result: Result<serde_json::Value, _> = rt.block_on(async {
            runner.execute_command("failing", &serde_json::json!({})).await
        });

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use shared_test_utils::integration::{IntegrationTestRunner, IntegrationPresets, IntegrationContext};

    #[test]
    fn test_integration_runner_setup() {
        let mut runner = IntegrationTestRunner::new().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            runner.setup(IntegrationPresets::minimal()).await.unwrap();
            assert!(runner.workspace().is_some());
            let context = runner.context().unwrap();
            assert_eq!(context.config.timeout_seconds, 30);
        });

        runner.cleanup().unwrap();
    }

    #[test]
    fn test_integration_context_state() {
        // Create a context directly without the runner to avoid runtime issues
        let config = IntegrationPresets::minimal();
        let workspace = TempWorkspace::new().unwrap();

        // Manually create the integration structure
        workspace.create_dir(Path::new("integration_data")).unwrap();
        workspace.create_file(Path::new("config.toml"), &format!(
            r#"cleanup_on_exit = {}
isolated_tests = {}
enable_logging = {}
timeout_seconds = {}
"#,
            config.cleanup_on_exit,
            config.isolated_tests,
            config.enable_logging,
            config.timeout_seconds
        )).unwrap();

        // Create context
        let mut context = IntegrationContext {
            test_dir: workspace.path().to_path_buf(),
            config,
            state: HashMap::new(),
        };

        // Test state management directly
        context.store_state("key", "value").unwrap();
        let retrieved: String = context.get_state("key").unwrap();
        assert_eq!(retrieved, "value");

        // Test scenario state
        context.state.insert("current_scenario".to_string(),
            serde_json::Value::String("manual_test".to_string()));

        assert_eq!(context.state["current_scenario"], "manual_test");

        // Test resource paths
        let resource_path = context.get_resource_path("test.txt");
        assert!(resource_path.ends_with("integration_data/test.txt"));
        assert!(!context.resource_exists("test.txt"));
    }
}

#[cfg(test)]
mod async_tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_async_timeout() {
        let result = with_timeout(async { String::from("success") }, Duration::from_millis(100)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_async_context() {
        let context = AsyncContext::with_timeout(Duration::from_secs(5));
        let result = context.execute(async { "test" }).await.unwrap();
        assert_eq!(result, "test");
    }

    #[tokio::test]
    async fn test_timeout_failure() {
        let result = with_timeout(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            "late"
        }, Duration::from_millis(50)).await;

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod macro_tests {
    use super::*;
    use shared_test_utils::fixtures::FixturePresets;
    use shared_test_utils::error::TestResult;

    #[test]
    fn test_setup_test_workspace_macro() {
        let workspace = setup_test_workspace!();
        assert!(workspace.path().exists());
        // Workspace automatically cleans up
    }

    #[test]
    fn test_assert_test_eq_macro() {
        assert_test_eq!(1 + 1, 2);
        assert_test_eq!(true, true, "Boolean test");
    }

    #[test]
    fn test_with_test_fixture_macro() {
        let (workspace, fixture) = with_test_fixture!(FixturePresets::rust_library());
        assert!(workspace.file_exists(Path::new("Cargo.toml")));
        let files = fixture.files().collect::<Vec<_>>();
        assert!(files.len() >= 2);
    }
}