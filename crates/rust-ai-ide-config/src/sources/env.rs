//! Environment variable configuration source
//!
//! Loads configuration from environment variables with the secure RUST_AI_IDE_ prefix
//! and automatic type conversion with security validation.

use async_trait::async_trait;
use std::collections::HashMap;
use std::env;

use super::ConfigSource;
use crate::config::ConfigSourcePriority;

/// Environment variable configuration source
#[derive(Debug, Clone)]
pub struct EnvironmentSource {
    /// Environment variable prefix (defaults to "RUST_AI_IDE_")
    prefix: String,
    /// Security validator
    security_validator: std::sync::Arc<crate::SecurityValidator>,
}

impl EnvironmentSource {
    /// Create new environment source with default prefix
    pub fn new() -> Self {
        Self {
            prefix: "RUST_AI_IDE_".to_string(),
            security_validator: std::sync::Arc::new(crate::SecurityValidator::new(
                crate::config::SecurityLevel::High,
            )),
        }
    }

    /// Create environment source with custom prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        let mut source = Self::new();
        source.prefix = prefix.into();
        source
    }

    /// Create environment source with custom security validator
    pub fn with_security_validator(validator: std::sync::Arc<crate::SecurityValidator>) -> Self {
        let mut source = Self::new();
        source.security_validator = validator;
        source
    }

    /// Convert environment variable to configuration field
    ///
    /// Examples:
    /// - RUST_AI_IDE_API_KEY -> api_key
    /// - RUST_AI_IDE_DATABASE_URL -> database.url
    fn env_key_to_field(&self, env_key: &str) -> Option<String> {
        if !env_key.starts_with(&self.prefix) {
            return None;
        }

        let field = env_key[self.prefix.len()..].to_lowercase();
        let field = field.replace("_", ".");

        Some(field)
    }

    /// Convert configuration field to environment variable
    ///
    /// Examples:
    /// - api_key -> RUST_AI_IDE_API_KEY
    /// - database.url -> RUST_AI_IDE_DATABASE_URL
    fn field_to_env_key(&self, field: &str) -> String {
        let upper = field.to_uppercase();
        let env_key = upper.replace(".", "_");
        format!("{}{}", self.prefix, env_key)
    }

    /// Get all environment variables matching prefix
    fn get_matching_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        for (key, value) in env::vars() {
            if key.starts_with(&self.prefix) {
                vars.insert(key, value);
            }
        }

        vars
    }

    /// Convert string value to target type
    fn convert_value<T>(&self, value: &str) -> crate::IDEResult<T>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        // Security validation of value
        let sanitized = self.security_validator.sanitize_input(value, "env_value")?;

        sanitized.parse::<T>().map_err(|e| {
            crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(&format!(
                "Failed to parse environment value '{}': {}",
                sanitized, e
            )))
        })
    }

    /// Build configuration from environment variables as JSON value
    fn build_config_from_env_json(&self) -> crate::IDEResult<serde_json::Value> {
        let env_vars = self.get_matching_env_vars();

        if env_vars.is_empty() {
            return Err(crate::RustAIError::Config(
                rust_ai_ide_errors::ConfigError::new(&format!(
                    "No environment variables found with prefix {}",
                    self.prefix
                )),
            ));
        }

        // Convert to JSON-like structure
        let mut config_map = serde_json::Map::new();

        for (env_key, value) in env_vars {
            if let Some(field_path) = self.env_key_to_field(&env_key) {
                self.insert_value_into_map(&mut config_map, &field_path, &value)?;
            }
        }

        Ok(serde_json::Value::Object(config_map))
    }

    /// Insert environment value into nested map structure
    fn insert_value_into_map(
        &self,
        map: &mut serde_json::Map<String, serde_json::Value>,
        field_path: &str,
        value: &str,
    ) -> crate::IDEResult<()> {
        let parts: Vec<&str> = field_path.split('.').collect();

        if parts.is_empty() {
            return Ok(());
        }

        // Convert value to JSON first
        let json_value = match value.to_lowercase().as_str() {
            "true" => serde_json::Value::Bool(true),
            "false" => serde_json::Value::Bool(false),
            _ => {
                // Try to parse as number, otherwise keep as string
                if let Ok(num) = value.parse::<i64>() {
                    serde_json::Value::Number(num.into())
                } else if let Ok(num) = value.parse::<f64>() {
                    serde_json::Number::from_f64(num)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
                } else {
                    serde_json::Value::String(value.to_string())
                }
            }
        };

        // Insert value into the nested structure
        Self::insert_recursive(map, &parts, json_value);
        Ok(())
    }

    /// Helper function to recursively insert values into nested map structure
    fn insert_recursive(
        map: &mut serde_json::Map<String, serde_json::Value>,
        parts: &[&str],
        value: serde_json::Value,
    ) {
        let first_part = parts[0];

        if parts.len() == 1 {
            // Last part - set the value directly
            map.insert(first_part.to_string(), value);
        } else {
            // Need to create/get nested object and recurse
            if let Some(serde_json::Value::Object(ref mut nested_map)) = map.get_mut(first_part) {
                Self::insert_recursive(nested_map, &parts[1..], value);
            } else {
                // Create new nested object
                let mut new_map = serde_json::Map::new();
                Self::insert_recursive(&mut new_map, &parts[1..], value);
                map.insert(first_part.to_string(), serde_json::Value::Object(new_map));
            }
        }
    }
}

#[async_trait]
impl ConfigSource for EnvironmentSource {
    async fn load(&self, _name: &str) -> crate::IDEResult<Option<serde_json::Value>> {
        match self.build_config_from_env_json() {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                // Log warning but don't fail - environment config is optional
                tracing::warn!("Failed to load configuration from environment: {}", e);
                Ok(None)
            }
        }
    }

    async fn save(&self, _name: &str, _config: &serde_json::Value) -> crate::IDEResult<()> {
        // Environment variables are read-only
        Err(crate::RustAIError::Config(
            rust_ai_ide_errors::ConfigError::new("Environment variables are read-only"),
        ))
    }

    fn can_save(&self) -> bool {
        false
    }

    fn priority(&self) -> ConfigSourcePriority {
        ConfigSourcePriority::Environment
    }

    fn description(&self) -> String {
        format!("Environment variables (prefix: {})", self.prefix)
    }
}

/// Environment configuration utilities
pub struct EnvUtils;

impl EnvUtils {
    /// Check if environment variable is set
    pub fn is_env_set(key: &str) -> bool {
        env::var(key).is_ok()
    }

    /// Get environment variable with default
    pub fn get_env_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get typed environment variable
    pub fn get_env_typed<T>(key: &str) -> Option<Result<T, T::Err>>
    where
        T: std::str::FromStr,
    {
        env::var(key).ok().map(|s| s.parse::<T>())
    }

    /// List all environment variables with a prefix
    pub fn list_env_prefix(prefix: &str) -> Vec<String> {
        env::vars()
            .filter_map(|(key, _)| {
                if key.starts_with(prefix) {
                    Some(key)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_env_key_conversion() {
        let source = EnvironmentSource::new();

        assert_eq!(
            source.env_key_to_field("RUST_AI_IDE_API_KEY"),
            Some("api_key".to_string())
        );
        assert_eq!(
            source.env_key_to_field("RUST_AI_IDE_DATABASE_URL"),
            Some("database.url".to_string())
        );
        assert_eq!(source.env_key_to_field("OTHER_PREFIX_KEY"), None);
    }

    #[tokio::test]
    async fn test_field_key_conversion() {
        let source = EnvironmentSource::new();

        assert_eq!(source.field_to_env_key("api_key"), "RUST_AI_IDE_API_KEY");
        assert_eq!(
            source.field_to_env_key("database.url"),
            "RUST_AI_IDE_DATABASE_URL"
        );
    }

    #[tokio::test]
    async fn test_env_utils() {
        // Test with non-existent variable
        assert!(!EnvUtils::is_env_set("NON_EXISTENT_VAR"));
        assert_eq!(
            EnvUtils::get_env_or_default("NON_EXISTENT_VAR", "default"),
            "default"
        );

        // Test listing with prefix
        let vars = EnvUtils::list_env_prefix("NON_EXISTENT_PREFIX_");
        assert!(vars.is_empty());
    }

    #[tokio::test]
    async fn test_value_conversion() {
        let source = EnvironmentSource::new();

        assert_eq!(source.convert_value::<i32>("42").unwrap(), 42);
        assert_eq!(source.convert_value::<bool>("true").unwrap(), true);
        assert_eq!(source.convert_value::<String>("hello").unwrap(), "hello");
    }
}
