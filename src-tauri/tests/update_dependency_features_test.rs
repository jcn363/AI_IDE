
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use crate::rust_ai_ide_cargo::cargo_set_dependency_features;
// Note: This test could also be migrated to use rust_ai_ide_common::test_utils.
// For demonstration, we're showing that both approaches work side by side.

#[tokio::test]
async fn test_update_dependency_features() {
    // Create a temporary directory for testing
    let dir = tempdir().unwrap();
    let manifest_path = dir.path().join("Cargo.toml");
    
    // Create a simple Cargo.toml for testing
    let mut file = File::create(&manifest_path).unwrap();
    writeln!(
        file, 
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = {{ version = \"1.0\", features = [\"derive\"] }}"
    ).unwrap();

    // Test updating features
    cargo_set_dependency_features(
        manifest_path.to_str().unwrap().to_string(),
        "serde".to_string(),
        vec!["derive".to_string(), "rc".to_string()],
        Some(false)
    ).await.unwrap();
    
    // Add a small delay to ensure the file is written
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Verify the changes
    let content = fs::read_to_string(&manifest_path).unwrap();
    assert!(content.contains("serde = { version = \"1.0\", features = [\"derive\", \"rc\"], default-features = false }"));

    // Clean up
    drop(file);
    dir.close().unwrap();
}
