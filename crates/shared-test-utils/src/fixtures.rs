use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::TestError;
use crate::filesystem::TempWorkspace;

/// Builder pattern for creating test fixtures
#[derive(Clone, Debug, Default)]
pub struct TestFixtureBuilder {
    pub files:       HashMap<PathBuf, String>,
    pub directories: Vec<PathBuf>,
    pub metadata:    HashMap<String, String>,
}

impl TestFixtureBuilder {
    /// Creates a new fixture builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file to the fixture
    pub fn with_file(mut self, path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        self.files.insert(path.into(), content.into());
        self
    }

    /// Adds multiple files from an iterator
    pub fn with_files<I, P, S>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = (P, S)>,
        P: Into<PathBuf>,
        S: Into<String>,
    {
        for (path, content) in files {
            self.files.insert(path.into(), content.into());
        }
        self
    }

    /// Adds a directory to the fixture
    pub fn with_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.directories.push(path.into());
        self
    }

    /// Adds metadata to the fixture
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Builds the fixture in the given workspace
    pub fn build(self, workspace: &TempWorkspace) -> Result<TestFixture, TestError> {
        // Create directories
        for dir in &self.directories {
            workspace.create_dir(dir)?;
        }

        // Create files
        for (path, content) in &self.files {
            workspace.create_file(path, content)?;
        }

        Ok(TestFixture {
            files:       self.files,
            directories: self.directories,
            metadata:    self.metadata,
        })
    }
}

/// A completed test fixture
#[derive(Clone, Debug)]
pub struct TestFixture {
    pub files:       HashMap<PathBuf, String>,
    pub directories: Vec<PathBuf>,
    pub metadata:    HashMap<String, String>,
}

impl TestFixture {
    /// Gets the content of a file in the fixture
    pub fn get_file_content(&self, path: &PathBuf) -> Option<&String> {
        self.files.get(path)
    }

    /// Lists all files in the fixture
    pub fn files(&self) -> impl Iterator<Item = &PathBuf> {
        self.files.keys()
    }

    /// Lists all directories in the fixture
    pub fn directories(&self) -> impl Iterator<Item = &PathBuf> {
        self.directories.iter()
    }

    /// Gets metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Predefined fixture builders for common scenarios
/// Provides backward compatibility and integration with generic builder
pub struct FixturePresets;

impl FixturePresets {
    /// Basic Rust library fixture
    pub fn rust_library() -> TestFixtureBuilder {
        TestFixtureBuilder::new()
            .with_file(
                "Cargo.toml",
                r#"[package]
name = "test-lib"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
"#,
            )
            .with_file(
                "src/lib.rs",
                "pub fn hello() -> &'static str { \"Hello!\" }",
            )
            .with_directory("src")
    }

    /// Cargo workspace fixture
    pub fn cargo_workspace(members: &[&str]) -> TestFixtureBuilder {
        let mut builder = TestFixtureBuilder::new().with_file(
            "Cargo.toml",
            format!(
                r#"[workspace]
resolver = "2"
members = [{}]

[workspace.package]
version = "0.1.0"
edition = "2021"
publish = false
"#,
                members
                    .iter()
                    .map(|m| format!("\"{}\"", m))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );

        for member in members {
            builder = builder
                .with_directory(format!("{}/src", member))
                .with_file(
                    format!("{}/Cargo.toml", member),
                    format!(
                        r#"[package]
name = "{}"
version.workspace = true
edition.workspace = true
publish.workspace = true
"#,
                        member
                    ),
                )
                .with_file(
                    format!("{}/src/lib.rs", member),
                    format!("// {} library code", member),
                );
        }

        builder
    }

    /// JSON configuration fixture
    pub fn json_config() -> TestFixtureBuilder {
        TestFixtureBuilder::new().with_file(
            "config.json",
            r#"{
  "app": {
    "name": "test-app",
    "version": "1.0.0"
  },
  "features": ["feature1", "feature2"]
}"#,
        )
    }

    /// Multi-module Rust project
    pub fn multi_module() -> TestFixtureBuilder {
        TestFixtureBuilder::new()
            .with_file(
                "Cargo.toml",
                r#"[package]
name = "multi-module"
version = "0.1.0"
edition = "2021"
            .with_directory("src")
    }
}

/// Context types for different test scenarios
#[derive(Clone, Debug)]
pub struct MessageContext {
    pub message_type: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, Default)]
pub struct CargoWorkspaceContext {
    pub config: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct WebAppContext {
    pub framework: String,
    pub routes: Vec<String>,
    pub static_files: Vec<PathBuf>,
}

/// Adapter for converting between generic and legacy fixtures
pub struct FixtureAdapter;

impl FixtureAdapter {
    /// Convert a generic fixture to legacy format
    pub fn to_legacy<T>(fixture: crate::builder::TestFixture<T>) -> Result<TestFixture, TestError> {
        Ok(TestFixture {
            files: fixture.files,
            directories: fixture.directories,
            metadata: fixture.metadata,
        })
    }

    /// Convert legacy fixture to generic format
    pub fn from_legacy<T>(fixture: TestFixture, context: Option<T>) -> crate::builder::TestFixture<T> {
        crate::builder::TestFixture {
            files: fixture.files,
            directories: fixture.directories,
            metadata: fixture.metadata,
            context_data: context,
        }
    }

    /// Migrate existing test code to use new generic fixtures
    pub fn migrate_to_generic<T>(
        setup_code: &str,
        context_factory: impl Fn() -> T
    ) -> String {
        // Basic string replacement for migration
        let mut result = setup_code.replace("TestFixtureBuilder::new()", "fixture_builder!(())");
        result = result.replace("use crate::fixtures::TestFixtureBuilder", "use crate::builder::TestFixtureBuilder");
        result = result.replace("use crate::fixtures", "use crate::fixtures; use crate::builder");

        // Add context creation if needed
        if setup_code.contains("with_context_data") {
            result = format!("{}\nlet context = context_factory();", result);
        }

        result
    }
}

/// Extension trait for backward compatibility
pub trait TestFixtureExt {
    /// Convert legacy fixture to generic builder
    fn to_generic_builder<T>(self) -> crate::builder::TestFixtureBuilder<T>;
}

impl TestFixtureExt for TestFixtureBuilder {
    fn to_generic_builder<T>(self) -> crate::builder::TestFixtureBuilder<T> {
        let mut new_builder = crate::builder::TestFixtureBuilder::<T>::new();
        for (path, content) in self.files {
            new_builder = new_builder.with_file(path, content);
        }
        for dir in self.directories {
            new_builder = new_builder.with_directory(dir);
        }
        for (key, value) in self.metadata {
            new_builder = new_builder.with_metadata(key, value);
        }
        new_builder
    }
}

/// Factory for creating context-aware fixtures
pub struct ContextFixtureFactory;

impl ContextFixtureFactory {
    pub fn new() -> Self {
        Self
    }

    /// Create fixture for message-based tests
    pub fn message_fixture(&self, message_type: &str) -> crate::builder::TestFixtureBuilder<MessageContext> {
        crate::builder::TestFixtureBuilder::new()
            .with_context_data(MessageContext {
                message_type: message_type.to_string(),
                metadata: HashMap::new(),
            })
    }

    /// Create fixture for web application tests
    pub fn web_app_fixture(&self, framework: &str) -> crate::builder::TestFixtureBuilder<WebAppContext> {
        crate::builder::TestFixtureBuilder::new()
            .with_context_data(WebAppContext {
                framework: framework.to_string(),
                routes: vec![],
                static_files: vec![],
            })
    }

    /// Create fixture for Cargo workspace tests
    pub fn cargo_workspace_fixture(&self) -> crate::builder::TestFixtureBuilder<CargoWorkspaceContext> {
        crate::builder::TestFixtureBuilder::new()
            .with_context_data(CargoWorkspaceContext::default())
    }
}

impl Default for ContextFixtureFactory {
    fn default() -> Self {
        Self::new()
    }
}
"#,
            )
            .with_file(
                "src/lib.rs",
                r#"pub mod utils;
pub mod math;

pub use utils::*;
"#,
            )
            .with_file(
                "src/utils.rs",
                r#"pub fn helper() -> String {
    "helper".to_string()
}
"#,
            )
            .with_file(
                "src/math.rs",
                r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
            )
            .with_directory("src")
    }
}
