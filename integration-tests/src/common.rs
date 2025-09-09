//! Common utilities and helpers for integration tests

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use shared_test_utils::IntegrationContext;
use shared_test_utils::integration::IntegrationTestRunner;
use rust_ai_ide_errors::RustAIError;

/// Common test data directory and sample files
pub const TEST_DATA_DIR: &str = "integration-test-data";
pub const SAMPLE_RUST_FILE: &str = r#"
use std::collections::HashMap;

pub struct TestAnalyzer {
    pub data: HashMap<String, String>,
}

impl TestAnalyzer {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn analyze(&self, input: &str) -> String {
        format!("Analyzed: {}", input)
    }

    pub async fn async_analyze(&self, input: &str) -> Result<String, RustAIError> {
        Ok(format!("Async analyzed: {}", input))
    }
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let analyzer = TestAnalyzer::new();
        assert_eq!(analyzer.analyze("test"), "Analyzed: test");
    }
}"#;

/// Integration test context extension with specialized helpers
#[derive(Clone)]
pub struct ExtendedIntegrationContext {
    pub base: IntegrationContext,
    pub test_workspace: PathBuf,
    pub mock_data: Arc<Mutex<std::collections::HashMap<String, serde_json::Value>>>,
}

impl ExtendedIntegrationContext {
    pub fn new(base: IntegrationContext) -> Self {
        let test_workspace = base.test_dir.join("workspace");
        Self {
            base,
            test_workspace,
            mock_data: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Create a sample Rust project in the test workspace
    pub fn create_sample_rust_project(&self, project_name: &str) -> Result<(), RustAIError> {
        let project_dir = self.test_workspace.join(project_name);
        std::fs::create_dir_all(&project_dir)?;

        // Create Cargo.toml
        let cargo_toml = format!(r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#, project_name);

        std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

        // Create src/main.rs
        std::fs::create_dir_all(project_dir.join("src"))?;
        std::fs::write(
            project_dir.join("src/main.rs"),
            SAMPLE_RUST_FILE.replace("TestAnalyzer", &format!("{}Analyzer", project_name))
        )?;

        Ok(())
    }

    /// Create mock LSP response data
    pub async fn store_mock_data(&self, key: &str, value: serde_json::Value) -> Result<(), RustAIError> {
        let mut mock_data = self.mock_data.lock().await;
        mock_data.insert(key.to_string(), value);
        Ok(())
    }

    /// Retrieve mock data
    pub async fn get_mock_data(&self, key: &str) -> Result<Option<serde_json::Value>, RustAIError> {
        let mock_data = self.mock_data.lock().await;
        Ok(mock_data.get(key).cloned())
    }

    /// Setup AI analysis mock responses
    pub async fn setup_ai_analysis_mocks(&self) -> Result<(), RustAIError> {
        let completions = serde_json::json!({
            "items": [
                {
                    "label": "vec![1, 2, 3]",
                    "kind": 6,
                    "detail": "CreateVec",
                    "documentation": "Creates a vector with the given elements"
                },
                {
                    "label": "String::new()",
                    "kind": 1,
                    "detail": "string.constructor",
                    "documentation": "Creates a new empty String"
                }
            ]
        });

        self.store_mock_data("completions", completions).await?;

        let diagnostics = serde_json::json!({
            "items": [
                {
                    "range": {
                        "start": {"line": 1, "character": 5},
                        "end": {"line": 1, "character": 10}
                    },
                    "severity": 1,
                    "message": "unused variable",
                    "source": "rust-ai-ide"
                }
            ]
        });

        self.store_mock_data("diagnostics", diagnostics).await?;

        Ok(())
    }

    /// Validate that a file exists and contains expected content
    pub async fn validate_file_content(&self, path: &Path, expected_content: &str) -> Result<bool, RustAIError> {
        if path.is_absolute() {
            return Ok(path.exists() && std::fs::read_to_string(path)?.contains(expected_content));
        } else {
            let full_path = self.test_workspace.join(path);
            Ok(full_path.exists() && std::fs::read_to_string(full_path)?.contains(expected_content))
        }
    }
}

/// Enhanced test runner with LSP and AI integration support
pub struct EnhancedIntegrationTestRunner {
    runner: IntegrationTestRunner,
    extended_context: Option<ExtendedIntegrationContext>,
}

impl EnhancedIntegrationTestRunner {
    pub fn new() -> Result<Self, RustAIError> {
        Ok(Self {
            runner: IntegrationTestRunner::new()?,
            extended_context: None,
        })
    }

    /// Initialize with enhanced context
    pub async fn setup_enhanced(&mut self, config: shared_test_utils::IntegrationContext) -> Result<(), RustAIError> {
        self.runner.setup(config.clone()).await?;

        if let Some(base_context) = self.runner.context() {
            let extended = ExtendedIntegrationContext::new(base_context.clone());
            self.extended_context = Some(extended);
        }

        Ok(())
    }

    /// Get the extended context
    pub fn extended_context(&self) -> Option<&ExtendedIntegrationContext> {
        self.extended_context.as_ref()
    }

    /// Run a test with enhanced context
    pub async fn run_with_enhanced_context<T, F>(
        &mut self,
        scenario_name: &str,
        test_fn: F,
    ) -> Result<T, RustAIError>
    where
        F: FnOnce(&mut ExtendedIntegrationContext) -> Result<T, RustAIError> + Send + 'static,
        T: Send + 'static,
    {
        let mut context = self.extended_context.clone()
            .ok_or_else(|| RustAIError::invalid_input("Enhanced context not initialized"))?;

        // Setup test scenario
        context.base.store_state("scenario", scenario_name.to_string())?;

        // Run the test function
        let result = test_fn(&mut context)?;

        Ok(result)
    }
}

/// Test scenario builders for common integration patterns
pub mod scenarios {
    use super::*;

    /// Builder for LSP integration scenarios
    pub struct LSPScenarioBuilder {
        pub project_files: Vec<(String, String)>,
        pub expected_operations: Vec<String>,
    }

    impl LSPScenarioBuilder {
        pub fn new() -> Self {
            Self {
                project_files: Vec::new(),
                expected_operations: Vec::new(),
            }
        }

        pub fn with_file(mut self, path: &str, content: &str) -> Self {
            self.project_files.push((path.to_string(), content.to_string()));
            self
        }

        pub fn expect_completion(mut self, position: (u32, u32)) -> Self {
            self.expected_operations.push(format!("completion: ({}, {})", position.0, position.1));
            self
        }

        pub fn expect_diagnostics(mut self) -> Self {
            self.expected_operations.push("diagnostics".to_string());
            self
        }

        pub fn build(self, context: &ExtendedIntegrationContext) -> Result<(), RustAIError> {
            // Create project files in test workspace
            let project_dir = context.test_workspace.join("lsp_test_project");
            std::fs::create_dir_all(&project_dir)?;

            for (relative_path, content) in self.project_files {
                let file_path = project_dir.join(relative_path);
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(file_path, content)?;
            }

            // Store expected operations in context
            context.base.store_state("expected_operations", self.expected_operations)?;

            Ok(())
        }
    }

    /// Builder for AI analysis scenarios
    pub struct AIScenarioBuilder {
        pub analysis_type: String,
        pub input_code: String,
        pub expected_issues: Vec<String>,
    }

    impl AIScenarioBuilder {
        pub fn new(analysis_type: &str) -> Self {
            Self {
                analysis_type: analysis_type.to_string(),
                input_code: String::new(),
                expected_issues: Vec::new(),
            }
        }

        pub fn with_code(mut self, code: &str) -> Self {
            self.input_code = code.to_string();
            self
        }

        pub fn expect_issue(mut self, issue: &str) -> Self {
            self.expected_issues.push(issue.to_string());
            self
        }

        pub async fn build(self, context: &mut ExtendedIntegrationContext) -> Result<(), RustAIError> {
            context.store_mock_data("analysis_type", serde_json::json!(self.analysis_type)).await?;
            context.store_mock_data("input_code", serde_json::json!(self.input_code)).await?;
            context.store_mock_data("expected_issues", serde_json::json!(self.expected_issues)).await?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sample_rust_file_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_file = temp_dir.path().join("sample.rs");
        std::fs::write(&project_file, SAMPLE_RUST_FILE).unwrap();

        let content = std::fs::read_to_string(&project_file).unwrap();
        assert!(content.contains("TestAnalyzer"));
        assert!(content.contains("struct"));
    }

    #[tokio::test]
    async fn test_enhanced_context_operations() {
        let mut runner = EnhancedIntegrationTestRunner::new().unwrap();
        let config = shared_test_utils::IntegrationContext::default();

        runner.setup_enhanced(config).await.unwrap();

        if let Some(context) = runner.extended_context() {
            context.setup_ai_analysis_mocks().await.unwrap();

            let data = context.get_mock_data("completions").await.unwrap();
            assert!(data.is_some());
        }
    }
}