use rust_ai_ide_cargo::{
    build::{BuildStatus, BuildError, ErrorLevel},
    dependency::{DependencyInfo, DependencyKind, DependencyManager},
    models::BuildMetrics,
};
use tempfile::tempdir;

#[tokio::test]
async fn test_dependency_management() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().join("test_project");
    std::fs::create_dir_all(&project_path).unwrap();

    // Create a simple Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"
        [package]
        name = "test_project"
        version = "0.1.0"
        edition = "2021"
        
        [dependencies]
        serde = { version = "1.0", features = ["derive"] }
        
        [dev-dependencies]
        tempfile = "3.0"
        
        [build-dependencies]
        cc = "1.0"
    "#,
    )
    .unwrap();

    // Initialize the dependency manager
    let manager = DependencyManager::new(project_path.to_str().unwrap());

    // Test loading dependencies
    manager
        .load_dependencies()
        .await
        .expect("Failed to load dependencies");

    // Test getting all dependencies
    let deps = manager.get_dependencies().await;
    assert!(!deps.is_empty(), "Should have loaded dependencies");

    // Test getting a specific dependency
    let serde_dep = manager.get_dependency("serde").await;
    assert!(serde_dep.is_some(), "Should find serde dependency");

    if let Some(serde) = serde_dep {
        assert_eq!(serde.kind, DependencyKind::Normal);
        assert!(serde.features.contains(&"derive".to_string()));
    }

    // Test adding a new dependency
    let new_dep = DependencyInfo {
        name: "tokio".to_string(),
        version: "1.0".to_string(),
        features: vec!["full".to_string()],
        optional: false,
        default_features: true,
        target: None,
        kind: DependencyKind::Normal,
        registry: None,
        source: None,
    };

    manager
        .add_dependency(new_dep)
        .await
        .expect("Failed to add dependency");

    // Test updating a dependency
    manager
        .update_dependency("serde", "2.0")
        .await
        .expect("Failed to update dependency");

    // Verify the update
    let updated_serde = manager.get_dependency("serde").await.unwrap();
    assert_eq!(updated_serde.version, "2.0");

    // Test removing a dependency
    manager
        .remove_dependency("tempfile")
        .await
        .expect("Failed to remove dependency");
    assert!(manager.get_dependency("tempfile").await.is_none());

    // Simulate build status updates
    let building = BuildStatus::Building {
        progress: 0.5,
        current_target: Some("test_crate".to_string()),
        jobs_running: 1,
        jobs_total: 2,
    };

    let success = BuildStatus::Success {
        duration: 5000.0,
        metrics: BuildMetrics {
            warning_count: 0,
            ..Default::default()
        },
    };

    let failed = BuildStatus::Failed {
        error: "Build failed".to_string(),
        duration: 10000.0,
        error_details: vec![
            BuildError {
                message: "Build failed".to_string(),
                file: None,
                line: None,
                column: None,
                code: None,
                level: ErrorLevel::Error,
            },
        ],
    };

    let cancelled = BuildStatus::Cancelled;

    // Verify BuildStatus variants have correct state
    assert!(building.is_building());
    assert!(success.is_success());
    assert!(failed.is_failed());
    assert!(cancelled.is_cancelled());
}
