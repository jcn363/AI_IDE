//! Simple integration test demonstrating shared-test-utils with rust-ai-ide-core

/// Basic integration test showing shared-test-utils functionality
#[test]
fn test_shared_utils_integration() {
    // This demonstrates that shared-test-utils is properly integrated
    // as a dev dependency and can be imported

    // Test basic functionality - workspace creation
    let workspace = shared_test_utils::TempWorkspace::new().unwrap();
    assert!(workspace.path().exists());

    // Create a test file using TempWorkspace
    workspace
        .create_file(
            std::path::Path::new("test_config.toml"),
            r#"[test]
name = "integration-test"
enabled = true"#,
        )
        .unwrap();

    // Verify the file was created and contains expected content
    assert!(workspace.file_exists(std::path::Path::new("test_config.toml")));
    let content = workspace
        .read_file(std::path::Path::new("test_config.toml"))
        .unwrap();
    assert!(content.contains("integration-test"));
    assert!(content.contains("enabled = true"));

    // Test content validation from ValidationUtils
    assert!(shared_test_utils::ValidationUtils::validate_content(&content, &["test"]).is_ok());

    // Demonstrate that the workspace is automatically cleaned up
    drop(workspace); // This should clean up the temporary directory
}
