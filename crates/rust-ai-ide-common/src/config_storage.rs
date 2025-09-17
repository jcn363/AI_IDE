/// ! Platform-agnostic configuration and data storage
/// Provides unified interface for storing and retrieving configuration data across platforms

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::errors::IdeError;
use crate::fs::{ensure_directory, read_file_to_string, write_string_to_file};
use crate::platform::{get_app_data_dir, get_cache_dir};
use crate::security::{AuditLogger, SecureStorage};

/// Configuration storage manager
pub struct ConfigStorage {
    app_name: String,
    config_dir: PathBuf,
    cache_dir: PathBuf,
    secure_storage: SecureStorage,
    config_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl ConfigStorage {
    /// Create a new configuration storage instance
    pub async fn new(app_name: &str) -> Result<Self, IdeError> {
        let config_dir = get_app_data_dir(app_name).map_err(|e| IdeError::Io {
            message: format!("Failed to get app data directory: {}", e),
        })?;

        let cache_dir = get_cache_dir(app_name).map_err(|e| IdeError::Io {
            message: format!("Failed to get cache directory: {}", e),
        })?;

        // Ensure directories exist
        ensure_directory(&config_dir).await?;
        ensure_directory(&cache_dir).await?;

        Ok(Self {
            app_name: app_name.to_string(),
            config_dir,
            cache_dir,
            secure_storage: SecureStorage,
            config_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store a configuration value
    pub async fn set_config(&self, key: &str, value: &str) -> Result<(), IdeError> {
        let file_path = self.config_dir.join(format!("{}.json", key));

        // Update cache
        {
            let mut cache = self.config_cache.write().await;
            cache.insert(key.to_string(), value.to_string());
        }

        // Write to file
        write_string_to_file(&file_path, value).await?;

        AuditLogger::log_security_event("CONFIG_UPDATE", &format!("Updated config key: {}", key));
        Ok(())
    }

    /// Retrieve a configuration value
    pub async fn get_config(&self, key: &str) -> Result<Option<String>, IdeError> {
        // Check cache first
        {
            let cache = self.config_cache.read().await;
            if let Some(value) = cache.get(key) {
                return Ok(Some(value.clone()));
            }
        }

        // Read from file
        let file_path = self.config_dir.join(format!("{}.json", key));
        if file_path.exists() {
            let content = read_file_to_string(&file_path).await?;
            // Update cache
            {
                let mut cache = self.config_cache.write().await;
                cache.insert(key.to_string(), content.clone());
            }
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Store a secure configuration value
    pub async fn set_secure_config(&self, key: &str, value: &[u8]) -> Result<(), IdeError> {
        SecureStorage::store_secure_data(key, value).await?;
        AuditLogger::log_security_event("SECURE_CONFIG_UPDATE", &format!("Updated secure config key: {}", key));
        Ok(())
    }

    /// Retrieve a secure configuration value
    pub async fn get_secure_config(&self, key: &str) -> Result<Option<Vec<u8>>, IdeError> {
        let data = SecureStorage::retrieve_secure_data(key).await?;
        Ok(Some(data))
    }

    /// Store configuration object with serialization
    pub async fn set_config_object<T: Serialize>(&self, key: &str, value: &T) -> Result<(), IdeError> {
        let json = serde_json::to_string_pretty(value).map_err(|e| IdeError::Serialization {
            message: format!("Failed to serialize config object: {}", e),
        })?;

        self.set_config(key, &json).await
    }

    /// Retrieve and deserialize configuration object
    pub async fn get_config_object<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, IdeError> {
        if let Some(json) = self.get_config(key).await? {
            let value = serde_json::from_str(&json).map_err(|e| IdeError::Deserialization {
                message: format!("Failed to deserialize config object: {}", e),
            })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Remove a configuration value
    pub async fn remove_config(&self, key: &str) -> Result<(), IdeError> {
        let file_path = self.config_dir.join(format!("{}.json", key));

        // Remove from cache
        {
            let mut cache = self.config_cache.write().await;
            cache.remove(key);
        }

        // Remove file if it exists
        if file_path.exists() {
            tokio::fs::remove_file(&file_path).await.map_err(|e| IdeError::Io {
                message: format!("Failed to remove config file: {}", e),
            })?;
        }

        AuditLogger::log_security_event("CONFIG_REMOVE", &format!("Removed config key: {}", key));
        Ok(())
    }

    /// Remove a secure configuration value
    pub async fn remove_secure_config(&self, key: &str) -> Result<(), IdeError> {
        SecureStorage::delete_secure_data(key).await?;
        AuditLogger::log_security_event("SECURE_CONFIG_REMOVE", &format!("Removed secure config key: {}", key));
        Ok(())
    }

    /// List all configuration keys
    pub async fn list_config_keys(&self) -> Result<Vec<String>, IdeError> {
        use crate::fs::read_dir_filtered;

        let entries = read_dir_filtered(&self.config_dir, |entry| {
            entry.path().extension().and_then(|ext| ext.to_str()) == Some("json")
        }).await?;

        let keys: Vec<String> = entries
            .iter()
            .filter_map(|path| {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        Ok(keys)
    }

    /// Clear all configuration data
    pub async fn clear_config(&self) -> Result<(), IdeError> {
        let keys = self.list_config_keys().await?;
        for key in keys {
            self.remove_config(&key).await?;
        }

        // Clear cache
        {
            let mut cache = self.config_cache.write().await;
            cache.clear();
        }

        AuditLogger::log_security_event("CONFIG_CLEAR", "Cleared all configuration data");
        Ok(())
    }

    /// Get configuration directory path
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Export configuration to a backup file
    pub async fn export_config<P: AsRef<Path>>(&self, export_path: P) -> Result<(), IdeError> {
        use std::collections::HashMap;

        let keys = self.list_config_keys().await?;
        let mut config_data = HashMap::new();

        for key in keys {
            if let Some(value) = self.get_config(&key).await? {
                config_data.insert(key, value);
            }
        }

        let json = serde_json::to_string_pretty(&config_data).map_err(|e| IdeError::Serialization {
            message: format!("Failed to serialize config data: {}", e),
        })?;

        write_string_to_file(export_path, &json).await?;
        AuditLogger::log_security_event("CONFIG_EXPORT", "Exported configuration data");
        Ok(())
    }

    /// Import configuration from a backup file
    pub async fn import_config<P: AsRef<Path>>(&self, import_path: P) -> Result<(), IdeError> {
        use std::collections::HashMap;

        let json = read_file_to_string(import_path).await?;
        let config_data: HashMap<String, String> = serde_json::from_str(&json).map_err(|e| IdeError::Deserialization {
            message: format!("Failed to deserialize config data: {}", e),
        })?;

        for (key, value) in config_data {
            self.set_config(&key, &value).await?;
        }

        AuditLogger::log_security_event("CONFIG_IMPORT", "Imported configuration data");
        Ok(())
    }
}

/// Global configuration registry for managing multiple config storages
pub struct ConfigRegistry {
    storages: Arc<RwLock<HashMap<String, Arc<ConfigStorage>>>>,
}

impl ConfigRegistry {
    /// Create a new configuration registry
    pub fn new() -> Self {
        Self {
            storages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a configuration storage
    pub async fn register_storage(&self, name: &str, storage: Arc<ConfigStorage>) {
        let mut storages = self.storages.write().await;
        storages.insert(name.to_string(), storage);
    }

    /// Get a configuration storage by name
    pub async fn get_storage(&self, name: &str) -> Option<Arc<ConfigStorage>> {
        let storages = self.storages.read().await;
        storages.get(name).cloned()
    }

    /// List all registered storage names
    pub async fn list_storages(&self) -> Vec<String> {
        let storages = self.storages.read().await;
        storages.keys().cloned().collect()
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration migration helper
pub struct ConfigMigration {
    storage: Arc<ConfigStorage>,
}

impl ConfigMigration {
    /// Create a new migration helper
    pub fn new(storage: Arc<ConfigStorage>) -> Self {
        Self { storage }
    }

    /// Migrate configuration from old key to new key
    pub async fn migrate_key(&self, old_key: &str, new_key: &str) -> Result<(), IdeError> {
        if let Some(value) = self.storage.get_config(old_key).await? {
            self.storage.set_config(new_key, &value).await?;
            self.storage.remove_config(old_key).await?;
            log::info!("Migrated config key '{}' to '{}'", old_key, new_key);
        }
        Ok(())
    }

    /// Apply a series of migrations
    pub async fn apply_migrations(&self, migrations: Vec<(&str, &str)>) -> Result<(), IdeError> {
        for (old_key, new_key) in migrations {
            self.migrate_key(old_key, new_key).await?;
        }
        Ok(())
    }
}

/// Configuration validation helper
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate configuration value against a schema
    pub fn validate_config_value(value: &str, schema: &ConfigSchema) -> Result<(), IdeError> {
        // Basic validation - could be extended with more sophisticated schema validation
        match schema {
            ConfigSchema::String { min_length, max_length, .. } => {
                let len = value.len();
                if let Some(min) = min_length {
                    if len < *min {
                        return Err(IdeError::Validation {
                            field: "config_value".to_string(),
                            reason: format!("Value too short: {} < {}", len, min),
                        });
                    }
                }
                if let Some(max) = max_length {
                    if len > *max {
                        return Err(IdeError::Validation {
                            field: "config_value".to_string(),
                            reason: format!("Value too long: {} > {}", len, max),
                        });
                    }
                }
            }
            ConfigSchema::Number { min, max, .. } => {
                if let Ok(num) = value.parse::<f64>() {
                    if let Some(min_val) = min {
                        if num < *min_val {
                            return Err(IdeError::Validation {
                                field: "config_value".to_string(),
                                reason: format!("Value too small: {} < {}", num, min_val),
                            });
                        }
                    }
                    if let Some(max_val) = max {
                        if num > *max_val {
                            return Err(IdeError::Validation {
                                field: "config_value".to_string(),
                                reason: format!("Value too large: {} > {}", num, max_val),
                            });
                        }
                    }
                } else {
                    return Err(IdeError::Validation {
                        field: "config_value".to_string(),
                        reason: "Invalid number format".to_string(),
                    });
                }
            }
            ConfigSchema::Boolean => {
                if !matches!(value.to_lowercase().as_str(), "true" | "false" | "1" | "0") {
                    return Err(IdeError::Validation {
                        field: "config_value".to_string(),
                        reason: "Invalid boolean value".to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Configuration schema for validation
pub enum ConfigSchema {
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
        integer_only: bool,
    },
    Boolean,
}