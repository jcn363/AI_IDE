//! Cloud Feature Flags for Progressive Deployment
//!
//! This crate provides a dynamic feature flag system that integrates with
//! Kubernetes ConfigMaps for runtime feature flag updates.

use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub mod errors;
pub mod provider;

pub use errors::FeatureFlagError;
pub use provider::FeatureFlagProvider;

/// Global feature flag registry with thread-safe access
static FEATURE_FLAGS: Lazy<Arc<FeatureFlagRegistry>> =
    Lazy::new(|| Arc::new(FeatureFlagRegistry::default()));

/// Feature flag value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeatureFlagValue {
    Boolean(bool),
    String(String),
    Number(f64),
    Json(serde_json::Value),
}

/// Feature flag with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub value: FeatureFlagValue,
    pub enabled: bool,
    pub rollout_percentage: Option<u8>,
    pub description: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

/// Feature flag registry for thread-safe access
pub struct FeatureFlagRegistry {
    flags: RwLock<DashMap<String, FeatureFlag>>,
    providers: RwLock<Vec<Arc<dyn FeatureFlagProvider>>>,
}

impl Default for FeatureFlagRegistry {
    fn default() -> Self {
        Self {
            flags: RwLock::new(DashMap::new()),
            providers: RwLock::new(Vec::new()),
        }
    }
}

/// Core feature flag manager
pub struct FeatureFlagManager {
    registry: Arc<FeatureFlagRegistry>,
}

impl FeatureFlagManager {
    /// Create a new feature flag manager
    pub fn new() -> Self {
        Self {
            registry: Arc::clone(&FEATURE_FLAGS),
        }
    }

    /// Initialize from ConfigMap data
    pub async fn load_from_configmap(&self, config_data: &str) -> Result<(), FeatureFlagError> {
        let parsed: std::collections::HashMap<String, String> = serde_json::from_str(config_data)
            .map_err(|e| {
            FeatureFlagError::ParseError(format!("Failed to parse configmap: {}", e))
        })?;

        let flags = self.registry.flags.write().await;

        for (key, value_str) in parsed {
            let (flag_name, rollout_percentage) = self.parse_flag_key(&key);

            // Parse value based on string content
            let value = self.parse_flag_value(&value_str);

            let flag = FeatureFlag {
                name: flag_name.clone(),
                value: value.clone(),
                enabled: self.calculate_enabled_state(&value, rollout_percentage),
                rollout_percentage,
                description: None,
                dependencies: None,
            };

            flags.insert(flag_name, flag);
        }

        info!("Loaded {} feature flags from ConfigMap", flags.len());
        Ok(())
    }

    /// Check if a feature is enabled
    pub async fn is_enabled(&self, flag_name: &str) -> bool {
        let flags = self.registry.flags.read().await;
        let result = if let Some(flag) = flags.get(flag_name) {
            flag.enabled
        } else {
            false
        };
        result
    }

    /// Get feature flag value
    pub async fn get_value(&self, flag_name: &str) -> Option<FeatureFlagValue> {
        let flags = self.registry.flags.read().await;
        flags.get(flag_name).map(|f| f.value.clone())
    }

    /// Get boolean flag value with default
    pub async fn get_bool(&self, flag_name: &str, default: bool) -> bool {
        if let Some(FeatureFlagValue::Boolean(val)) = self.get_value(flag_name).await {
            val
        } else {
            default
        }
    }

    /// Get string flag value with default
    pub async fn get_string(&self, flag_name: &str, default: &str) -> String {
        if let Some(FeatureFlagValue::String(val)) = self.get_value(flag_name).await {
            val
        } else {
            default.to_string()
        }
    }

    /// Update feature flags from providers
    pub async fn refresh_flags(&self) -> Result<(), FeatureFlagError> {
        let providers = self.registry.providers.read().await.clone();
        for provider in providers {
            if let Ok(updates) = provider.fetch_flags().await {
                for (key, value_str) in updates {
                    let (flag_name, rollout_percentage) = self.parse_flag_key(&key);
                    let value = self.parse_flag_value(&value_str);

                    let flags = self.registry.flags.write().await;
                    if let Some(mut flag) = flags.get_mut(&flag_name) {
                        flag.value = value.clone();
                        flag.enabled =
                            self.calculate_enabled_state(&flag.value, rollout_percentage);
                        flag.rollout_percentage = rollout_percentage;
                    } else {
                        let flag = FeatureFlag {
                            name: flag_name.clone(),
                            value: value.clone(),
                            enabled: self.calculate_enabled_state(&value, rollout_percentage),
                            rollout_percentage,
                            description: None,
                            dependencies: None,
                        };
                        flags.insert(flag_name, flag);
                    };
                }
            }
        }
        Ok(())
    }

    /// Register a provider
    pub async fn register_provider(
        &self,
        provider: Arc<dyn FeatureFlagProvider>,
    ) -> Result<(), FeatureFlagError> {
        let mut providers = self.registry.providers.write().await;
        providers.push(provider);
        Ok(())
    }

    /// Parse flag key for rollout percentage
    fn parse_flag_key(&self, key: &str) -> (String, Option<u8>) {
        if key.contains(':') {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                if let Ok(percentage) = parts[1].parse::<u8>() {
                    return (parts[0].to_string(), Some(percentage));
                }
            }
        }
        (key.to_string(), None)
    }

    /// Parse flag value from string
    fn parse_flag_value(&self, value_str: &str) -> FeatureFlagValue {
        // Try boolean first
        if let Ok(bool_val) = value_str.parse::<bool>() {
            return FeatureFlagValue::Boolean(bool_val);
        }

        // Try number
        if let Ok(num_val) = value_str.parse::<f64>() {
            return FeatureFlagValue::Number(num_val);
        }

        // Try JSON
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(value_str) {
            return FeatureFlagValue::Json(json_val);
        }

        // Default to string
        FeatureFlagValue::String(value_str.to_string())
    }

    /// Calculate if feature should be enabled based on rollout percentage
    fn calculate_enabled_state(
        &self,
        value: &FeatureFlagValue,
        rollout_percentage: Option<u8>,
    ) -> bool {
        // Check base value first
        let base_enabled = matches!(value, FeatureFlagValue::Boolean(true))
            || matches!(value, FeatureFlagValue::String(ref s) if s == "true");

        if !base_enabled {
            return false;
        }

        // Apply rollout percentage if specified
        if let Some(percentage) = rollout_percentage {
            // Simple user ID for percentage-based rollout (in real impl, use actual user ID)
            let user_id = std::process::id() as u8;
            let rollout_bucket = user_id % 100;
            return rollout_bucket < percentage;
        }

        true
    }
}

/// Global convenience functions
pub async fn is_enabled(flag_name: &str) -> bool {
    let manager = FeatureFlagManager::new();
    manager.is_enabled(flag_name).await
}

pub async fn get_bool(flag_name: &str, default: bool) -> bool {
    let manager = FeatureFlagManager::new();
    manager.get_bool(flag_name, default).await
}

pub async fn get_string(flag_name: &str, default: &str) -> String {
    let manager = FeatureFlagManager::new();
    manager.get_string(flag_name, default).await
}
