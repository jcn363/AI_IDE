//! File-based configuration source
//!
//! Supports loading configuration from TOML, YAML, and JSON files
//! with automatic format detection and security validation.

use async_trait::async_trait;
use notify::Watcher;
use std::path::PathBuf;
use tokio::fs;

use super::{ConfigSource, SourceUtils};
use crate::config::ConfigSourcePriority;
use crate::sources::ConfigFormat;

#[derive(Debug, Clone)]
pub struct FileSource {
    /// Configuration directories to search
    directories: Vec<PathBuf>,
    /// File format (auto-detect if None)
    format: Option<ConfigFormat>,
    /// Security validator reference
    security_validator: std::sync::Arc<crate::SecurityValidator>,
}

impl FileSource {
    /// Create a new file source with default directories
    pub fn new() -> Self {
        Self {
            directories: vec![
                PathBuf::from("./config"),
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("./config"))
                    .join("rust-ai-ide"),
            ],
            format: None,
            security_validator: std::sync::Arc::new(crate::SecurityValidator::new(
                crate::config::SecurityLevel::High,
            )),
        }
    }

    /// Create file source with custom directories
    pub fn with_directories(directories: Vec<PathBuf>) -> Self {
        let mut source = Self::new();
        source.directories = directories;
        source
    }

    /// Set preferred file format
    pub fn with_format(mut self, format: ConfigFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set security validator
    pub fn with_security_validator(
        mut self,
        validator: std::sync::Arc<crate::SecurityValidator>,
    ) -> Self {
        self.security_validator = validator;
        self
    }

    /// Find configuration file by name
    fn find_config_file(&self, name: &str) -> Option<PathBuf> {
        let files = SourceUtils::find_config_files(name, &self.directories);
        files.into_iter().next() // Return first found file
    }

    /// Detect format from file path
    fn detect_format(&self, path: &PathBuf) -> crate::IDEResult<ConfigFormat> {
        if let Some(format) = self.format {
            return Ok(format);
        }

        ConfigFormat::from_path(path).ok_or_else(|| {
            crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(
                "Could not detect configuration format from file extension",
            ))
        })
    }

    /// Load and parse configuration file
    async fn load_from_file<C>(&self, path: PathBuf) -> crate::IDEResult<Option<C>>
    where
        C: crate::Config,
    {
        // Security validation of path
        self.security_validator.validate_path(&path, None)?;

        // Read file content
        let content = fs::read_to_string(&path).await.map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            )))
        })?;

        // Detect or use specified format
        let format = self.detect_format(&path)?;

        // Optionally detect format from content if extension-based detection failed
        let format = if let Some(actual_format) = ConfigFormat::detect_from_content(&content) {
            if actual_format != format {
                tracing::warn!(
                    "File extension suggests {:?} but content appears to be {:?} for {}",
                    format,
                    actual_format,
                    path.display()
                );
            }
            actual_format
        } else {
            format
        };

        // Parse configuration
        let config = SourceUtils::parse_config::<C>(format, &content)?;

        Ok(Some(config))
    }
}

#[async_trait]
impl ConfigSource for FileSource {
    async fn load(&self, name: &str) -> crate::IDEResult<Option<serde_json::Value>> {
        let Some(path) = self.find_config_file(name) else {
            return Ok(None); // File not found
        };

        let content = std::fs::read_to_string(&path).map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            )))
        })?;

        let format = self.detect_format(&path).map_err(|e| {
            crate::RustAIError::Serialization(format!("Format detection error: {}", e))
        })?;

        let value: serde_json::Value = match format {
            ConfigFormat::Json => serde_json::from_str(&content).map_err(|e| {
                crate::RustAIError::Serialization(format!("JSON parsing error: {}", e))
            })?,
            ConfigFormat::Yaml => serde_yaml::from_str(&content).map_err(|e| {
                crate::RustAIError::Serialization(format!("YAML parsing error: {}", e))
            })?,
            ConfigFormat::Toml => {
                let toml_value: toml::Value = toml::from_str(&content).map_err(|e| {
                    crate::RustAIError::Serialization(format!("TOML parsing error: {}", e))
                })?;
                serde_json::to_value(toml_value).map_err(|e| {
                    crate::RustAIError::Serialization(format!(
                        "TOML to JSON conversion error: {}",
                        e
                    ))
                })?
            }
        };

        Ok(Some(value))
    }

    async fn save(&self, name: &str, config: &serde_json::Value) -> crate::IDEResult<()> {
        let format = self.format.unwrap_or(ConfigFormat::Json);
        let file_name = format!("{}.{:?}", name, format).to_lowercase();
        let path = self
            .directories
            .first()
            .ok_or_else(|| {
                crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(
                    "No configuration directories configured",
                ))
            })?
            .join(file_name);

        // Security validation of path
        self.security_validator.validate_path(&path, None)?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                    "Failed to create config directory {}: {}",
                    parent.display(),
                    e
                )))
            })?;
        }

        // Serialize and write
        let content = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(config).map_err(|e| {
                crate::RustAIError::Serialization(format!("JSON serialization error: {}", e))
            })?,
            ConfigFormat::Yaml => serde_yaml::to_string(config).map_err(|e| {
                crate::RustAIError::Serialization(format!("YAML serialization error: {}", e))
            })?,
            ConfigFormat::Toml => {
                let toml_value: toml::Value =
                    serde_json::from_value(config.clone()).map_err(|e| {
                        crate::RustAIError::Serialization(format!(
                            "JSON to TOML conversion error: {}",
                            e
                        ))
                    })?;
                toml::to_string_pretty(&toml_value).map_err(|e| {
                    crate::RustAIError::Serialization(format!("TOML serialization error: {}", e))
                })?
            }
        };

        fs::write(&path, content).await.map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to write config file {}: {}",
                path.display(),
                e
            )))
        })?;

        tracing::info!("Saved configuration to {}", path.display());
        Ok(())
    }

    fn can_save(&self) -> bool {
        !self.directories.is_empty()
    }

    fn priority(&self) -> ConfigSourcePriority {
        ConfigSourcePriority::File
    }

    fn description(&self) -> String {
        format!("File source (directories: {:?})", self.directories)
    }
}

// Configuration watching for hot reload
#[derive(Debug)]
pub struct ConfigWatcher {
    watcher: notify::RecommendedWatcher,
    watched_files: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<PathBuf>>>,
}

impl ConfigWatcher {
    /// Create new configuration file watcher
    pub fn new() -> crate::IDEResult<Self> {
        let (tx, _rx) = std::sync::mpsc::channel();
        let watcher = notify::recommended_watcher(tx).map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to create file watcher: {}",
                e
            )))
        })?;

        Ok(Self {
            watcher,
            watched_files: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
        })
    }

    /// Watch a configuration file for changes
    pub fn watch_file(&mut self, path: PathBuf) -> crate::IDEResult<()> {
        self.watcher
            .watch(&path, notify::RecursiveMode::NonRecursive)
            .map_err(|e| {
                crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                    "Failed to watch file {}: {}",
                    path.display(),
                    e
                )))
            })?;

        self.watched_files.lock().unwrap().insert(path);
        Ok(())
    }

    /// Stop watching a file
    pub fn unwatch_file(&mut self, path: &PathBuf) -> crate::IDEResult<()> {
        self.watcher.unwatch(path).map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to unwatch file {}: {}",
                path.display(),
                e
            )))
        })?;

        self.watched_files.lock().unwrap().remove(path);
        Ok(())
    }

    /// Get currently watched files
    pub fn watched_files(&self) -> Vec<PathBuf> {
        self.watched_files.lock().unwrap().iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_source_creation() {
        let source = FileSource::new();
        assert!(!source.directories.is_empty());
        assert_eq!(source.priority(), ConfigSourcePriority::File);
    }

    #[tokio::test]
    async fn test_format_detection() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("test.toml");

        let format = ConfigFormat::from_path(&temp_path);
        assert_eq!(format, Some(ConfigFormat::Toml));
    }

    #[tokio::test]
    async fn test_content_format_detection() {
        let json_content = r#"{"test": "value"}"#;
        let yaml_content = "test: value\n";
        let toml_content = "[test]\nvalue = \"test\"\n";

        assert_eq!(
            ConfigFormat::detect_from_content(json_content),
            Some(ConfigFormat::Json)
        );
        assert_eq!(
            ConfigFormat::detect_from_content(yaml_content),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::detect_from_content(toml_content),
            Some(ConfigFormat::Toml)
        );
    }

    #[tokio::test]
    async fn test_config_watcher() {
        let mut watcher = ConfigWatcher::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.toml");

        // Create empty test file
        tokio::fs::write(&test_file, "").await.unwrap();

        watcher.watch_file(test_file.clone()).unwrap();
        assert_eq!(watcher.watched_files().len(), 1);

        watcher.unwatch_file(&test_file).unwrap();
        assert_eq!(watcher.watched_files().len(), 0);
    }
}
