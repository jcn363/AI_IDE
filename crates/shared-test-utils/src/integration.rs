use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use serde::{Serialize, Deserialize};
use crate::error::TestError;
use crate::filesystem::TempWorkspace;

/// Global state for integration tests
#[derive(Clone)]
pub struct IntegrationContext {
    pub test_dir: PathBuf,
    pub config: IntegrationConfig,
    pub state: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub cleanup_on_exit: bool,
    pub isolated_tests: bool,
    pub enable_logging: bool,
    pub timeout_seconds: u64,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        IntegrationConfig {
            cleanup_on_exit: true,
            isolated_tests: true,
            enable_logging: false,
            timeout_seconds: 60,
        }
    }
}

/// Main runner for integration tests
pub struct IntegrationTestRunner {
    context: Option<IntegrationContext>,
    workspace: Option<TempWorkspace>,
    runtime: Option<Runtime>,
}

impl IntegrationTestRunner {
    pub fn new() -> Result<Self, TestError> {
        Ok(IntegrationTestRunner {
            context: None,
            workspace: None,
            runtime: None,
        })
    }

    /// Initializes the integration test environment
    pub async fn setup(&mut self, config: IntegrationConfig) -> Result<(), TestError> {
        let workspace = TempWorkspace::new()?;

        // Create basic integration structure
        workspace.create_dir(Path::new("integration_data"))?;
        workspace.create_dir(Path::new("temp_resources"))?;
        workspace.create_dir(Path::new("logs"))?;

        // Setup basic configuration files
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
        ))?;

        // Initialize logging if enabled
        if config.enable_logging {
            workspace.create_file(Path::new("logs/integration.log"), "Integration test log\n")?;
        }

        self.context = Some(IntegrationContext {
            test_dir: workspace.path().to_path_buf(),
            config: config.clone(),
            state: HashMap::new(),
        });

        self.workspace = Some(workspace);

        // Setup Tokio runtime for async operations
        if self.runtime.is_none() {
            self.runtime = Some(Runtime::new()
                .map_err(|e| TestError::Async(format!("Failed to create runtime: {}", e)))?);
        }

        Ok(())
    }

    /// Runs an integration test scenario
    pub async fn run_scenario<T, F>(
        &mut self,
        scenario_name: &str,
        test_fn: F,
    ) -> Result<T, TestError>
    where
        F: FnOnce(&mut IntegrationContext) -> Result<T, TestError>,
        T: Send + 'static,
    {
        if self.context.is_none() {
            return Err(TestError::Validation(crate::error::ValidationError::invalid_setup(
                "Integration test not properly initialized. Call setup() first."
            )));
        }

        let mut context = self.context.clone().unwrap();

        // Execute test in runtime if available
        if let Some(runtime) = &self.runtime {
            runtime.block_on(async {
                context.state.insert("current_scenario".to_string(),
                    serde_json::Value::String(scenario_name.to_string()));

                test_fn(&mut context)
            })
        } else {
            context.state.insert("current_scenario".to_string(),
                serde_json::Value::String(scenario_name.to_string()));

            test_fn(&mut context)
        }
    }

    /// Cleans up the integration test environment
    pub fn cleanup(&mut self) -> Result<(), TestError> {
        if let Some(context) = &self.context {
            if context.config.cleanup_on_exit {
                // Clear state
                self.context.as_mut().unwrap().state.clear();

                // Workspace is automatically cleaned up when dropped
                self.workspace = None;
                self.context = None;
                self.runtime = None;
            }
        }

        Ok(())
    }

    /// Gets the current context (for advanced testing)
    pub fn context(&self) -> Option<&IntegrationContext> {
        self.context.as_ref()
    }

    /// Gets the workspace
    pub fn workspace(&self) -> Option<&TempWorkspace> {
        self.workspace.as_ref()
    }
}

impl Drop for IntegrationTestRunner {
    fn drop(&mut self) {
        // Auto-cleanup on drop
        let _ = self.cleanup();
    }
}

impl IntegrationContext {
    /// Stores state in the context
    pub fn store_state<T: serde::Serialize>(&mut self, key: &str, value: T) -> Result<(), TestError> {
        self.state.insert(key.to_string(),
            serde_json::to_value(value).map_err(|e| TestError::Serialization(e.to_string()))?);
        Ok(())
    }

    /// Retrieves state from context
    pub fn get_state<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, TestError> {
        if let Some(value) = self.state.get(key) {
            serde_json::from_value(value.clone())
                .map_err(|arg0: serde_json::Error| TestError::Serialization(arg0.to_string()))
        } else {
            Err(TestError::Validation(crate::error::ValidationError::invalid_setup(
                format!("State key '{}' not found", key)
            )))
        }
    }

    /// Gets the path to add a resource file to the integration directory
    pub fn get_resource_path(&self, name: &str) -> PathBuf {
        self.test_dir.join("integration_data").join(name)
    }

    /// Checks if a resource exists
    pub fn resource_exists(&self, name: &str) -> bool {
        self.get_resource_path(name).exists()
    }

    /// Gets the logs directory
    pub fn logs_dir(&self) -> &Path {
        &self.test_dir
    }
}

/// Predefined integration test setups
pub struct IntegrationPresets;

impl IntegrationPresets {
    /// Basic full-stack integration test setup
    pub fn full_stack() -> IntegrationConfig {
        IntegrationConfig {
            cleanup_on_exit: true,
            isolated_tests: true,
            enable_logging: true,
            timeout_seconds: 120,
        }
    }

    /// Minimal integration test setup
    pub fn minimal() -> IntegrationConfig {
        IntegrationConfig {
            cleanup_on_exit: true,
            isolated_tests: false,
            enable_logging: false,
            timeout_seconds: 30,
        }
    }

    /// Development-focused integration test with extended logging
    pub fn development() -> IntegrationConfig {
        IntegrationConfig {
            cleanup_on_exit: false, // Leave files for inspection
            isolated_tests: true,
            enable_logging: true,
            timeout_seconds: 300,
        }
    }
}