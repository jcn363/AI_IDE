use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::TestError;
use crate::filesystem::TempWorkspace;

/// Generic builder pattern for creating test fixtures
/// Supports multiple test contexts through type parameter T
#[derive(Clone, Debug)]
pub struct TestFixtureBuilder<T> {
    files:        HashMap<PathBuf, String>,
    directories:  Vec<PathBuf>,
    metadata:     HashMap<String, String>,
    context_data: Option<T>,
}

impl<T> TestFixtureBuilder<T> {
    /// Creates a new generic fixture builder
    pub fn new() -> Self {
        Self {
            files:        HashMap::new(),
            directories:  Vec::new(),
            metadata:     HashMap::new(),
            context_data: None,
        }
    }

    /// Sets the context data for this fixture
    pub fn with_context_data(mut self, context: T) -> Self {
        self.context_data = Some(context);
        self
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
    pub fn build(self, workspace: &TempWorkspace) -> Result<TestFixture<T>, TestError> {
        // Create directories
        for dir in &self.directories {
            workspace.create_dir(dir)?;
        }

        // Create files
        for (path, content) in &self.files {
            workspace.create_file(path, content)?;
        }

        Ok(TestFixture {
            files:        self.files,
            directories:  self.directories,
            metadata:     self.metadata,
            context_data: self.context_data,
        })
    }

    /// Builds the fixture without a workspace (for testing context only)
    pub fn build_context_only(self) -> TestFixture<T> {
        TestFixture {
            files:        self.files,
            directories:  self.directories,
            metadata:     self.metadata,
            context_data: self.context_data,
        }
    }
}

/// A completed generic test fixture
#[derive(Clone, Debug)]
pub struct TestFixture<T> {
    pub files:        HashMap<PathBuf, String>,
    pub directories:  Vec<PathBuf>,
    pub metadata:     HashMap<String, String>,
    pub context_data: Option<T>,
}

impl<T> TestFixture<T> {
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

    /// Gets the context data
    pub fn context_data(&self) -> Option<&T> {
        self.context_data.as_ref()
    }

    /// Takes ownership of context data
    pub fn take_context_data(&mut self) -> Option<T> {
        self.context_data.take()
    }

    /// Sets new context data
    pub fn set_context_data(&mut self, context: T) {
        self.context_data = Some(context);
    }
}

/// Builder for creating typed contexts
pub trait ContextBuilder<T> {
    fn build_context(&self) -> T;
}

/// Factory for creating fixture builders with pre-configured contexts
pub struct FixtureBuilderFactory {
    presets: HashMap<String, Box<dyn Fn() -> TestFixtureBuilder<serde_json::Value> + Send + Sync>>,
}

impl FixtureBuilderFactory {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
        }
    }

    pub fn register_preset<F>(&mut self, name: impl Into<String>, factory: F)
    where
        F: Fn() -> TestFixtureBuilder<serde_json::Value> + Send + Sync + 'static,
    {
        self.presets.insert(name.into(), Box::new(factory));
    }

    pub fn create_preset(&self, name: &str) -> Option<TestFixtureBuilder<serde_json::Value>> {
        self.presets.get(name).map(|factory| factory())
    }
}

/// Type-erased fixture for dynamic contexts
pub struct DynamicTestFixture {
    files:             HashMap<PathBuf, String>,
    directories:       Vec<PathBuf>,
    metadata:          HashMap<String, String>,
    context_type_name: String,
}

impl DynamicTestFixture {
    /// Creates a dynamic fixture from any typed fixture
    pub fn from_fixture<T>(fixture: TestFixture<T>) -> Self {
        Self {
            files:             fixture.files,
            directories:       fixture.directories,
            metadata:          fixture.metadata,
            context_type_name: std::any::type_name::<T>().to_string(),
        }
    }

    pub fn get_file_content(&self, path: &PathBuf) -> Option<&String> {
        self.files.get(path)
    }

    pub fn files(&self) -> impl Iterator<Item = &PathBuf> {
        self.files.keys()
    }

    pub fn directories(&self) -> impl Iterator<Item = &PathBuf> {
        self.directories.iter()
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn context_type_name(&self) -> &str {
        &self.context_type_name
    }
}

/// Default implementations for common types
impl<T> Default for TestFixtureBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FixtureBuilderFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macros for creating fixtures
#[macro_export]
macro_rules! fixture_builder {
    ($context_type:ty) => {
        TestFixtureBuilder::<$context_type>::new()
    };
    ($context_type:ty, $context_data:expr) => {
        TestFixtureBuilder::<$context_type>::new().with_context_data($context_data)
    };
}

#[macro_export]
macro_rules! quick_fixture {
    ($context_type:ty; $($key:expr => $value:expr),* $(;)?) => {
        {
            let mut builder = TestFixtureBuilder::<$context_type>::new();
            $(
                builder = builder.with_metadata($key, $value);
            )*
            builder
        }
    };
}

// Backward compatibility for existing TestFixtureBuilder
impl TestFixtureBuilder<()> {
    /// Convert to old-style non-generic fixture (for backward compatibility)
    pub fn as_legacy(self, workspace: &TempWorkspace) -> Result<crate::fixtures::TestFixture, TestError> {
        let typed_fixture = self.build(workspace)?;
        Ok(crate::fixtures::TestFixture {
            files:       typed_fixture.files,
            directories: typed_fixture.directories,
            metadata:    typed_fixture.metadata,
        })
    }
}
