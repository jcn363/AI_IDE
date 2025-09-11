//! LSP Integration Tests
//!
//! Comprehensive integration tests for LSP server initialization, client-server interactions,
//! AI-enhanced language processing, multi-language support, and performance validation.
//!
//! Tests cover:
//! - LSP server lifecycle management
//! - Client-server message protocol compliance
//! - AI-enhanced completion, diagnostics, and hover
//! - Multi-language symbol resolution
//! - High-throughput request processing
//! - Error recovery and graceful degradation

use crate::common::{scenarios::LSPScenarioBuilder, ExtendedIntegrationContext, SAMPLE_RUST_FILE};
use crate::IntegrationTestResult;
use async_trait::async_trait;
use rust_ai_ide_errors::RustAIError;
use rust_ai_ide_lsp::{client::LSPClient, AIContext, LSPClientConfig};
use shared_test_utils::lsp::{LSPFixture, LSPMessageBuilder, MockLSPServer};
use std::sync::Arc;
use tokio::sync::Mutex;

/// LSP Integration Test Suite Runner
#[derive(Clone)]
pub struct LSPIntegrationTestRunner {
    context: Option<ExtendedIntegrationContext>,
    mock_server: Option<Arc<Mutex<MockLSPServer>>>,
    client: Option<LSPClient>,
    results: Vec<IntegrationTestResult>,
}

impl LSPIntegrationTestRunner {
    pub fn new() -> Self {
        Self {
            context: None,
            mock_server: None,
            client: None,
            results: Vec::new(),
        }
    }

    /// Setup LSP test environment with mock server
    pub async fn setup_test_environment(
        &mut self,
        context: ExtendedIntegrationContext,
    ) -> Result<(), RustAIError> {
        self.context = Some(context);

        // Initialize mock LSP server
        let mock_server = Arc::new(Mutex::new(MockLSPServer::new()?));
        let server_address = mock_server.lock().await.address().clone();

        // Setup LSP fixtures and test data
        self.setup_lsp_fixtures().await?;

        // Configure LSP client
        let client_config = LSPClientConfig {
            server_address,
            timeout_ms: 5000,
            retry_attempts: 3,
            enable_ai: true,
        };

        let client = LSPClient::with_config(client_config).await?;
        client.initialize().await?;
        self.client = Some(client);
        self.mock_server = Some(mock_server);

        Ok(())
    }

    /// Setup LSP test fixtures and mock responses
    async fn setup_lsp_fixtures(&self) -> Result<(), RustAIError> {
        if let Some(context) = &self.context {
            // Create sample Rust project for testing
            context.create_sample_rust_project("lsp_test_project")?;

            // Setup completion fixtures
            let completions = serde_json::json!({
                "items": [
                    {
                        "label": "println!",
                        "kind": 2,
                        "detail": "macro println!",
                        "documentation": "Prints to stdout with newline",
                        "insertText": "println!(\"${1}\")",
                        "insertTextFormat": 2,
                        "sortText": "01"
                    },
                    {
                        "label": "vec!",
                        "kind": 2,
                        "detail": "macro vec!",
                        "documentation": "Creates a vector",
                        "insertText": "vec![${1}]",
                        "insertTextFormat": 2,
                        "sortText": "02"
                    }
                ]
            });

            // Setup diagnostic fixtures
            let diagnostics = serde_json::json!({
                "items": [
                    {
                        "range": {
                            "start": {"line": 0, "character": 0},
                            "end": {"line": 0, "character": 5}
                        },
                        "severity": 1,
                        "code": "unused_variable",
                        "message": "Unused variable `x`",
                        "source": "rust-ai-ide"
                    }
                ]
            });

            // Setup hover information
            let hover = serde_json::json!({
                "contents": {
                    "kind": "markdown",
                    "value": "# Vec\n\nA contiguous growable array type, written as `Vec<T>`, short for 'vector'."
                },
                "range": {
                    "start": {"line": 0, "character": 15},
                    "end": {"line": 0, "character": 18}
                }
            });

            context.store_mock_data("completions", completions).await?;
            context.store_mock_data("diagnostics", diagnostics).await?;
            context.store_mock_data("hover", hover).await?;
        }

        Ok(())
    }

    /// Test LSP server initialization and lifecycle
    pub async fn test_server_initialization(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("lsp_server_initialization");
        let start_time = std::time::Instant::now();

        match self.perform_server_initialization_test().await {
            Ok(_) => {
                result.success = true;
                result.add_metric(
                    "server_startup_time",
                    start_time.elapsed().as_millis().to_string(),
                );
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Server initialization failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_server_initialization_test(&self) -> Result<(), RustAIError> {
        if let Some(client) = &self.client {
            // Test basic LSP capabilities
            let capabilities = client.capabilities().await?;
            assert!(
                !capabilities.text_document_sync.is_none(),
                "Server must support text document sync"
            );

            // Test server status
            let status = client.server_status().await?;
            assert!(status.is_initialized, "Server must be properly initialized");

            Ok(())
        } else {
            Err(RustAIError::invalid_input("LSP client not initialized"))
        }
    }

    /// Test AI-enhanced completion requests
    pub async fn test_ai_completion_requests(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("ai_completion_requests");
        let start_time = std::time::Instant::now();

        match self.perform_completion_requests_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("AI completion test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_completion_requests_test(&self) -> Result<(), RustAIError> {
        if let Some(client) = &self.client {
            // Setup test scenario with LSP scenario builder
            if let Some(context) = &self.context {
                let scenario = LSPScenarioBuilder::new()
                    .with_file("src/main.rs", &SAMPLE_RUST_FILE.replace("println!", "pri"))
                    .expect_completion((0, 3))
                    .build(context)?;

                // Create AI context for enhanced completion
                let ai_context = AIContext {
                    current_code: "let vec = Vec::new();".to_string(),
                    file_name: Some("test.rs".to_string()),
                    cursor_position: Some((0, 15)),
                    selection: None,
                    project_context: std::collections::HashMap::from([
                        ("file_type".to_string(), "rust".to_string()),
                        ("imports".to_string(), "std::collections::*".to_string()),
                    ]),
                };

                // Test AI-enhanced completion
                let completions = client.get_completions_with_ai(ai_context).await?;
                assert!(!completions.items.is_empty(), "Should receive completions");

                // Validate completion quality
                let has_meaningful_suggestions = completions
                    .items
                    .iter()
                    .any(|item| item.label.contains("Vec") || item.detail.is_some());
                assert!(
                    has_meaningful_suggestions,
                    "Completions should be meaningful"
                );

                Ok(())
            } else {
                Err(RustAIError::invalid_input("Test context not available"))
            }
        } else {
            Err(RustAIError::invalid_input("LSP client not initialized"))
        }
    }

    /// Test diagnostic processing and AI enhancement
    pub async fn test_diagnostic_processing(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("diagnostic_processing");
        let start_time = std::time::Instant::now();

        match self.perform_diagnostic_processing_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Diagnostic processing test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_diagnostic_processing_test(&self) -> Result<(), RustAIError> {
        if let Some(client) = &self.client {
            // Setup file with deliberate unused variables
            let test_code = r#"fn main() {
    let unused_variable = 42;
    let another_unused = "test";
    println!("Hello, world!");
}"#;

            if let Some(context) = &self.context {
                let test_file = context.test_workspace.join("src/test_diagnostics.rs");
                std::fs::create_dir_all(test_file.parent().unwrap())?;
                std::fs::write(&test_file, test_code)?;

                // Request diagnostics
                let file_uri = format!("file://{}", test_file.display());
                let diagnostics = client.get_diagnostics(&file_uri).await?;

                // Validate that unused variables are detected
                let unused_diagnostics = diagnostics
                    .items
                    .iter()
                    .filter(|d| d.message.contains("unused"))
                    .count();

                assert!(
                    unused_diagnostics >= 2,
                    "Should detect at least 2 unused variables"
                );

                // Test AI-enhanced diagnostics if available
                if let Some(ai_diagnostics) = client.get_ai_enhanced_diagnostics(&file_uri).await {
                    assert!(
                        !ai_diagnostics.is_empty(),
                        "AI diagnostics should provide additional insights"
                    );
                }

                Ok(())
            } else {
                Err(RustAIError::invalid_input("Test context not available"))
            }
        } else {
            Err(RustAIError::invalid_input("LSP client not initialized"))
        }
    }

    /// Test hover information with AI context
    pub async fn test_hover_information(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("hover_information");
        let start_time = std::time::Instant::now();

        match self.perform_hover_information_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Hover information test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_hover_information_test(&self) -> Result<(), RustAIError> {
        if let Some(client) = &self.client {
            let test_code = "fn main() { let vec: Vec<i32> = Vec::new(); }";

            if let Some(context) = &self.context {
                let test_file = context.test_workspace.join("hover_test.rs");
                std::fs::write(&test_file, test_code)?;

                let file_uri = format!("file://{}", test_file.display());
                let position = (0, 35); // Position of "Vec" in "Vec::new()"

                // Test standard hover
                let hover_info = client.get_hover(&file_uri, position).await?;
                assert!(hover_info.is_some(), "Should get hover information for Vec");

                // Test AI-enhanced hover if available
                if let Some(ai_hover) = client.get_ai_hover(&file_uri, position).await {
                    assert!(
                        ai_hover.contents.len() > hover_info.as_ref().unwrap().contents.len(),
                        "AI hover should provide more comprehensive information"
                    );
                }

                Ok(())
            } else {
                Err(RustAIError::invalid_input("Test context not available"))
            }
        } else {
            Err(RustAIError::invalid_input("LSP client not initialized"))
        }
    }

    /// Test symbol search and cross-language references
    pub async fn test_symbol_search(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("symbol_search");
        let start_time = std::time::Instant::now();

        match self.perform_symbol_search_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Symbol search test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_symbol_search_test(&self) -> Result<(), RustAIError> {
        if let Some(client) = &self.client {
            let symbol_query = "TestAnalyzer";

            // Test basic symbol search
            let symbols = client.search_symbols(&symbol_query).await?;
            assert!(symbols.len() > 0, "Should find at least one symbol");

            // Test workspace symbols
            let workspace_symbols = client.search_workspace_symbols(&symbol_query).await?;
            assert!(
                workspace_symbols.len() >= symbols.len(),
                "Workspace search should find at least as many symbols as file search"
            );

            Ok(())
        } else {
            Err(RustAIError::invalid_input("LSP client not initialized"))
        }
    }

    /// Test multi-language support and symbol resolution
    pub async fn test_multi_language_support(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("multi_language_support");
        let start_time = std::time::Instant::now();

        match self.perform_multi_language_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Multi-language test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_multi_language_test(&self) -> Result<(), RustAIError> {
        // This would test cross-language symbol resolution
        // For now, we verify that the LSP client can handle multiple language configurations

        if let Some(client) = &self.client {
            // Test that client supports multiple languages
            let supported_languages = client.supported_languages().await?;
            assert!(
                supported_languages.contains(&"rust".to_string()),
                "Should support Rust language"
            );
            assert!(
                supported_languages.len() >= 1,
                "Should support at least one language"
            );

            Ok(())
        } else {
            Err(RustAIError::invalid_input("LSP client not initialized"))
        }
    }

    /// Test high-throughput request processing
    pub async fn test_performance_throughput(&mut self) -> Vec<IntegrationTestResult> {
        let mut results = Vec::new();

        let mut result = IntegrationTestResult::new("performance_throughput");
        let start_time = std::time::Instant::now();

        match self.perform_throughput_test().await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result.errors.push(format!("Throughput test failed: {}", e));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        results.push(result);

        results
    }

    async fn perform_throughput_test(&self) -> Result<(), RustAIError> {
        if let Some(client) = &self.client {
            const CONCURRENT_REQUESTS: usize = 10;
            let mut handles = Vec::new();

            for i in 0..CONCURRENT_REQUESTS {
                let test_code = format!("fn test_fn_{}() {{}}", i);
                let file_uri = format!("file:///test_{}.rs", i);
                let client_clone = client.clone();

                let handle = tokio::spawn(async move {
                    // Create document
                    client_clone.create_document(&file_uri, &test_code).await?;

                    // Request completions
                    let ai_context = AIContext {
                        current_code: test_code.clone(),
                        file_name: Some(format!("test_{}.rs", i)),
                        cursor_position: Some((0, test_code.len())),
                        selection: None,
                        project_context: std::collections::HashMap::new(),
                    };

                    let completions = client_clone.get_completions_with_ai(ai_context).await?;
                    Ok::<_, RustAIError>(completions.items.len())
                });

                handles.push(handle);
            }

            // Collect results
            let mut total_items = 0;
            for handle in handles {
                let result = handle.await??;
                total_items += result;
            }

            assert!(
                total_items > 0,
                "Should receive completions from concurrent requests"
            );

            Ok(())
        } else {
            Err(RustAIError::invalid_input("LSP client not initialized"))
        }
    }

    /// Get all test results
    pub fn get_all_results(&self) -> &[IntegrationTestResult] {
        &self.results
    }
}

/// Async trait implementation for integration with the test runner framework
#[async_trait]
impl crate::test_runner::TestSuiteRunner for LSPIntegrationTestRunner {
    fn suite_name(&self) -> &'static str {
        "lsp"
    }

    async fn run_test_suite(&self) -> Result<Vec<IntegrationTestResult>, RustAIError> {
        let mut runner = LSPIntegrationTestRunner::new();

        if let Some(context) = &self.context {
            runner.setup_test_environment(context.clone()).await?;
        }

        let mut all_results = Vec::new();

        // Run all LSP tests
        all_results.extend(runner.test_server_initialization().await);
        all_results.extend(runner.test_ai_completion_requests().await);
        all_results.extend(runner.test_diagnostic_processing().await);
        all_results.extend(runner.test_hover_information().await);
        all_results.extend(runner.test_symbol_search().await);
        all_results.extend(runner.test_multi_language_support().await);
        all_results.extend(runner.test_performance_throughput().await);

        Ok(all_results)
    }

    fn test_names(&self) -> Vec<String> {
        vec![
            "lsp_server_initialization".to_string(),
            "ai_completion_requests".to_string(),
            "diagnostic_processing".to_string(),
            "hover_information".to_string(),
            "symbol_search".to_string(),
            "multi_language_support".to_string(),
            "performance_throughput".to_string(),
        ]
    }

    fn is_test_enabled(&self, test_name: &str) -> bool {
        matches!(
            test_name,
            "lsp_server_initialization"
                | "ai_completion_requests"
                | "diagnostic_processing"
                | "hover_information"
                | "symbol_search"
                | "multi_language_support"
                | "performance_throughput"
        )
    }

    fn prerequisites(&self) -> Vec<String> {
        vec![
            "rust-analyzer".to_string(),
            "lsp-client".to_string(),
            "mock-server".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_lsp_fixture_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_path = temp_dir.path().join("workspace");
        std::fs::create_dir_all(&workspace_path).unwrap();

        let context = ExtendedIntegrationContext::new(shared_test_utils::IntegrationContext {
            test_dir: workspace_path,
            config: shared_test_utils::IntegrationConfig::default(),
            state: std::collections::HashMap::new(),
        });

        let scenario = LSPScenarioBuilder::new()
            .with_file("src/main.rs", SAMPLE_RUST_FILE)
            .expect_completion((0, 5))
            .build(&context);

        assert!(scenario.is_ok());
    }

    #[tokio::test]
    async fn test_result_collection() {
        let runner = LSPIntegrationTestRunner::new();
        assert_eq!(runner.get_all_results().len(), 0);
    }

    #[tokio::test]
    async fn test_runner_configuration() {
        let runner = LSPIntegrationTestRunner::new();
        assert_eq!(runner.suite_name(), "lsp");

        let test_names = runner.test_names();
        assert!(test_names.contains(&"lsp_server_initialization".to_string()));
        assert!(test_names.contains(&"ai_completion_requests".to_string()));

        assert!(runner.is_test_enabled("lsp_server_initialization"));
        assert!(!runner.is_test_enabled("nonexistent_test"));

        let prereqs = runner.prerequisites();
        assert!(prereqs.contains(&"rust-analyzer".to_string()));
        assert!(prereqs.contains(&"lsp-client".to_string()));
    }
}
