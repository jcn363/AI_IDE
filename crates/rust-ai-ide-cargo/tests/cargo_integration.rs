//! Cargo integration with shared test utilities demonstration

use shared_test_utils::*;

/// Basic integration test showing TempWorkspace with Cargo project structure
#[test]
fn test_cargo_workspace_temp() {
    let workspace = TempWorkspace::new().unwrap();

    // Simulate Cargo workspace structure
    workspace.create_dir(std::path::Path::new("src")).unwrap();
    workspace
        .create_file(
            std::path::Path::new("Cargo.toml"),
            r#"[workspace]
resolver = "2"
members = ["member1", "member2"]
"#,
        )
        .unwrap();

    // Verify the structure was created
    assert!(workspace.file_exists(std::path::Path::new("Cargo.toml")));
    let cargo_content = workspace
        .read_file(std::path::Path::new("Cargo.toml"))
        .unwrap();
    assert!(cargo_content.contains("workspace"));
    assert!(cargo_content.contains("resolver = \"2\""));

    // Use validation utilities
    use shared_test_utils::ValidationUtils;
    assert!(ValidationUtils::validate_content(&cargo_content, &["workspace"]).is_ok());
}

/// Demonstrate async utilities in Cargo context
#[tokio::test]
async fn test_async_cargo_operations() {
    use std::time::Duration;

    // Test timeout functionality for Cargo operations
    let result = with_timeout(
        async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "Cargo check completed"
        },
        Duration::from_millis(100),
    )
    .await;

    assert!(result.is_ok());

    // Test async context
    let context = AsyncContext::with_timeout(Duration::from_secs(1));
    let operation_result = context.execute(async { "Cargo build successful" }).await;

    assert!(operation_result.is_ok());
}
