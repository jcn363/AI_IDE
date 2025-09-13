//! Test module for Cargo integration

use std::fs::File;
use std::io::Write;

use rust_ai_ide_cargo::workspace::CargoManager;
use tempfile::tempdir;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_workspace_management() {
        // Create a temporary workspace
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path();

        // Create a workspace Cargo.toml
        let mut cargo_toml = File::create(workspace_root.join("Cargo.toml")).unwrap();
        writeln!(
            cargo_toml,
            r#"
            [workspace]
            members = ["crate1", "crate2"]
            "#
        )
        .unwrap();

        // Create member crates
        for name in &["crate1", "crate2"] {
            let crate_dir = workspace_root.join(name);
            std::fs::create_dir_all(&crate_dir).unwrap();

            let mut member_cargo = File::create(crate_dir.join("Cargo.toml")).unwrap();
            writeln!(
                member_cargo,
                r#"
                [package]
                name = "{}"
                version = "0.1.0"
                edition = "2021"
                "#,
                name
            )
            .unwrap();

            // Create a simple source file
            std::fs::create_dir_all(crate_dir.join("src")).unwrap();
            let mut lib_rs = File::create(crate_dir.join("src/lib.rs")).unwrap();
            writeln!(
                lib_rs,
                r#"
                pub fn hello() {{
                    println!("Hello from {}");
                }}
                "#,
                name
            )
            .unwrap();
        }

        // Test workspace initialization
        let mut manager = CargoManager::new();
        manager.initialize_workspace(workspace_root).await.unwrap();

        // Verify workspace members
        let members = manager.get_workspace_members();
        assert_eq!(members.len(), 2, "Expected exactly 2 workspace members");

        // Test workspace tree
        let tree = manager.get_workspace_tree().unwrap();
        assert!(tree.contains("Workspace Root"));
        assert!(tree.contains("crate1"));
        assert!(tree.contains("crate2"));

        // Test find references
        use rust_ai_ide_cargo::refactor::find_references;
        let references = find_references(&manager, "Hello").await.unwrap();
        assert!(
            !references.is_empty(),
            "Expected to find at least one reference to 'Hello'"
        );

        // Test workspace replace (dry run)
        use rust_ai_ide_cargo::refactor::workspace_replace;
        let count = workspace_replace(&manager, "Hello", "Hi", true)
            .await
            .unwrap();
        assert!(
            count >= 2,
            "Expected at least 2 occurrences of 'Hello' (one in each crate)"
        );
    }
}
