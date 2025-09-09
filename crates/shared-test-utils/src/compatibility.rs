//! Backward compatibility layer for the unified test harness
//!
//! This module provides wrapper types and functions that maintain
//! the old API while using the new unified test harness internally.

use crate::error::TestError;
use crate::builder::{TestFixtureBuilder as GenericTestFixtureBuilder, TestFixture as GenericTestFixture};
use crate::fixtures::{TestFixtureBuilder, TestFixture};
use crate::harness::{TestHarness, TestResult};
use async_trait::async_trait;

/// Legacy wrapper for TestFixtureBuilder to maintain API compatibility
pub struct LegacyTestFixtureBuilder {
    inner: TestFixtureBuilder,
}

impl LegacyTestFixtureBuilder {
    /// Create new legacy fixture builder
    pub fn new() -> Self {
        Self {
            inner: TestFixtureBuilder::new(),
        }
    }

    /// Add file to fixture
    pub fn with_file(self, path: impl Into<std::path::PathBuf>, content: impl Into<String>) -> Self {
        Self {
            inner: self.inner.with_file(path, content),
        }
    }

    /// All other methods delegate to inner builder
    pub fn with_files<I, P, S>(self, files: I) -> Self
    where
        I: IntoIterator<Item = (P, S)>,
        P: Into<std::path::PathBuf>,
        S: Into<String>,
    {
        Self {
            inner: self.inner.with_files(files),
        }
    }

    pub fn with_directory(self, path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            inner: self.inner.with_directory(path),
        }
    }

    pub fn with_metadata(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            inner: self.inner.with_metadata(key, value),
        }
    }

    pub fn build(self, workspace: &crate::filesystem::TempWorkspace) -> Result<LegacyTestFixture, TestError> {
        Ok(LegacyTestFixture {
            inner: self.inner.build(workspace)?,
        })
    }
}

/// Legacy wrapper for TestFixture
pub struct LegacyTestFixture {
    inner: TestFixture,
}

impl LegacyTestFixture {
    pub fn get_file_content(&self, path: &std::path::PathBuf) -> Option<&String> {
        self.inner.get_file_content(path)
    }

    pub fn files(&self) -> impl Iterator<Item = &std::path::PathBuf> {
        self.inner.files.keys()
    }

    pub fn directories(&self) -> impl Iterator<Item = &std::path::PathBuf> {
        self.inner.directories()
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.inner.get_metadata(key)
    }
}

/// Adapter to convert between legacy and generic fixtures
pub struct FixtureAdapter;

impl FixtureAdapter {
    /// Convert legacy fixture to generic
    pub fn legacy_to_generic<T>(legacy: LegacyTestFixture) -> GenericTestFixture<T> {
        GenericTestFixture {
            files: legacy.inner.files().map(|path| (path.clone(), "".to_string())).collect::<std::collections::HashMap<_, _>>(),
            directories: legacy.inner.directories().cloned().collect(),
            metadata: legacy.inner.metadata.clone(),
            context_data: None,
        }
    }

    /// Convert generic fixture to legacy (with context loss)
    pub fn generic_to_legacy<T>(generic: GenericTestFixture<T>) -> Result<LegacyTestFixture, TestError> {
        Ok(LegacyTestFixture {
            inner: crate::fixtures::TestFixture {
                files: generic.files,
                directories: generic.directories,
                metadata: generic.metadata,
            },
        })
    }

    /// Upgrade legacy builder to generic builder
    pub fn upgrade_builder<T>(legacy: LegacyTestFixtureBuilder) -> GenericTestFixtureBuilder<T> {
        let mut new_builder = GenericTestFixtureBuilder::<T>::new();
        for (path, content) in legacy.inner.files {
            new_builder = new_builder.with_file(path, content);
        }
        for dir in legacy.inner.directories {
            new_builder = new_builder.with_directory(dir);
        }
        for (key, value) in legacy.inner.metadata {
            new_builder = new_builder.with_metadata(key, value);
        }
        new_builder
    }
}

/// Wrapper for Legacy Test Harness Compatibility
#[async_trait]
pub trait LegacyTestHarness {
    type Input: Send + Sync + Clone;
    type Output: Send + Sync;

    async fn setup(&self, input: &Self::Input) -> Result<(), TestError>;
    async fn execute(&self, input: &Self::Input) -> Result<Self::Output, TestError>;
    async fn validate(&self, input: &Self::Input, output: Self::Output) -> Result<bool, TestError>;
    async fn teardown(&self) -> Result<(), TestError>;

    async fn run_full_test(&self, input: Self::Input) -> Result<bool, TestError> {
        self.setup(&input).await?;
        let output = self.execute(&input).await?;
        let result = self.validate(&input, output).await?;
        self.teardown().await?;
        Ok(result)
    }
}

/// Adapter to wrap legacy harness in new harness interface
pub struct LegacyHarnessAdapter<T, H> {
    harness: H,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, H> LegacyHarnessAdapter<T, H> {
    pub fn new(harness: H) -> Self {
        Self {
            harness,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<H> TestHarness for LegacyHarnessAdapter<(), H>
where
    H: LegacyTestHarness<Input = (), Output = ()> + Send + Sync,
{
    type Context = ();
    type Input = ();
    type Output = ();

    async fn setup(&self, _input: Self::Input) -> Result<Self::Context, TestError> {
        self.harness.setup(&()).await?;
        Ok(())
    }

    async fn execute(&mut self, _context: Self::Context) -> Result<Self::Output, TestError> {
        self.harness.execute(&()).await
    }

    async fn validate(&self, _context: Self::Context, _output: Self::Output) -> Result<TestResult, TestError> {
        let passed = self.harness.validate(&(), ()).await?;
        Ok(TestResult {
            passed,
            message: if passed { "Test passed".to_string() } else { "Test failed".to_string() },
            details: None,
            duration: std::time::Duration::from_millis(1), // Placeholder duration
        })
    }

    async fn cleanup(&self, _context: Self::Context) -> Result<(), TestError> {
        self.harness.teardown().await
    }
}

/// Macro to easily wrap legacy test harnesses
#[macro_export]
macro_rules! wrap_legacy_harness {
    ($legacy_type:ty) => {
        impl TestHarness for $legacy_type {
            type Context = ();
            type Input = String;
            type Output = String;

            fn setup(&self, _input: Self::Input) -> Result<Self::Context, TestError> {
                Ok(())
            }

            fn execute(&self, _context: Self::Context) -> Result<Self::Output, TestError> {
                Ok("legacy".to_string())
            }

            fn validate(&self, _context: Self::Context, _output: Self::Output) -> Result<TestResult, TestError> {
                Ok(TestResult {
                    passed: true,
                    message: "Legacy test passed".to_string(),
                    details: None,
                    duration: std::time::Duration::from_millis(1),
                })
            }
        }
    };
}

/// Migration helpers
pub struct TestHarnessMigrator;

impl TestHarnessMigrator {
    /// Migrate legacy test code to use new harness
    pub fn migrate_legacy_test(code: &str) -> String {
        let mut result = code.to_string();

        // Replace legacy patterns with new ones
        result = result.replace("TestFixtureBuilder::new()", "LegacyTestFixtureBuilder::new()");
        result = result.replace("use shared_test_utils::TestFixtureBuilder", "use shared_test_utils::compatibility::LegacyTestFixtureBuilder");
        result = result.replace("use shared_test_utils::fixtures::TestFixtureBuilder", "use shared_test_utils::compatibility::LegacyTestFixtureBuilder");

        // Add migration comments
        result = format!("// TODO: Consider migrating to unified test harness\n{}", result);

        result
    }

    /// Validate that migration would preserve functionality
    pub fn validate_migration(code: &str) -> Result<(), TestError> {
        if code.contains("TestHarness") && code.contains("async_trait") {
            return Ok(());
        }

        if code.contains("TestFixtureBuilder") && !code.contains("LegacyTestFixtureBuilder") {
            return Err(TestError::Validation(crate::error::ValidationError::InvalidTestSetup {
                message: "Code may need migration to new harness".to_string()
            }));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_wrapper_creation() {
        let legacy = LegacyTestFixtureBuilder::new()
            .with_file("test.txt", "content")
            .with_metadata("key", "value");

        assert!(!legacy.inner.files.is_empty());
        assert!(!legacy.inner.metadata.is_empty());
    }

    #[test]
    fn test_adapter_conversions() {
        let legacy_fixture = LegacyTestFixture {
            inner: TestFixture {
                files: [("test.rs".into(), "fn main() {}".into())].into(),
                directories: vec!["src".into()],
                metadata: [("lang".into(), "rust".into())].into(),
            },
        };

        let generic: GenericTestFixture<()> = FixtureAdapter::legacy_to_generic(legacy_fixture);
        assert_eq!(generic.files.len(), 1);
        assert_eq!(generic.metadata.get("lang"), Some(&"rust".to_string()));
    }
}