//! Integration tests for the LSP client

use rust_ai_ide_lsp::client::{LSPClient, LSPClientConfig};
use std::path::PathBuf;
use tempfile::tempdir;
use test_log::test;
use tokio::time::{sleep, Duration};

#[test_log::test(tokio::test)]
async fn test_lsp_client_initialization() {
    // Skip if rust-analyzer is not in PATH
    if which::which("rust-analyzer").is_err() {
        eprintln!("Skipping test: rust-analyzer not found in PATH");
        return;
    }

    // Create a temporary directory for the test workspace
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let workspace_path = temp_dir.path().to_path_buf();

    // Create a simple Cargo.toml for testing
    let cargo_toml = workspace_path.join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .expect("Failed to write Cargo.toml");

    // Create a simple source file
    let src_dir = workspace_path.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src dir");
    std::fs::write(
        src_dir.join("main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}
"#,
    )
    .expect("Failed to write main.rs");

    // Configure the LSP client
    let config = LSPClientConfig {
        server_path: Some("rust-analyzer".to_string()),
        root_dir: Some(workspace_path.clone()),
        ..Default::default()
    };

    // Create and initialize the LSP client
    let client = LSPClient::with_config(config).expect("Failed to create LSP client");

    // Test connecting to the LSP server
    let result = client.connect().await;
    assert!(
        result.is_ok(),
        "Failed to connect to LSP server: {:?}",
        result
    );

    // Give the server some time to initialize
    sleep(Duration::from_secs(1)).await;

    // Test getting document symbols
    let document_uri = url::Url::from_file_path(src_dir.join("main.rs")).expect("Invalid file URI");
    let symbols = client.document_symbols(document_uri).await;

    // The test should at least not panic, even if we don't get symbols back
    assert!(
        symbols.is_ok(),
        "Failed to get document symbols: {:?}",
        symbols
    );

    // Clean up
    drop(temp_dir);
}
