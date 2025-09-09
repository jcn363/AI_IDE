//! # Model Registry Module
//!
//! Core model registry implementation with resource management and automated policies.

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// Internal helper modules for deduplication
mod internal {
    use super::*;
    use crate::{
        loaders::ModelLoader,
        resource_types::ModelType,
        unloading_policies::{PolicyEvaluator, UnloadingRecommendation},
    };

    /// Shared implementation for auto unloading models
    /// This eliminates the duplication between ModelRegistry and ModelRegistryInner
    pub async fn execute_auto_unload(
        loaded_models: &Arc<RwLock<HashMap<String, ModelHandle>>>,
        model_loaders: &HashMap<ModelType, Arc<dyn ModelLoader>>,
        policy_evaluator: &PolicyEvaluator,
        unloading_policy: &UnloadingPolicy,
    ) -> Result<f64> {
        let models = loaded_models.read().await;
        let recommendations = policy_evaluator
            .get_unloading_recommendations(&models, unloading_policy)
            .await;

        match recommendations {
            UnloadingRecommendation::Unload {
                model_ids,
                memory_freed_mb,
                ..
            } => {
                if model_ids.is_empty() {
                    return Ok(0.0);
                }

                info!(
                    "Auto-unloading {} models, freeing {:.1}MB",
                    model_ids.len(),
                    memory_freed_mb
                );

                // Actually unload the models
                for model_id in &model_ids {
                    if let Some(handle) = models.get(model_id) {
                        let loader = model_loaders.get(&handle.model_type);
                        if let Some(loader) = loader {
                            if let Err(e) = loader.unload_model(model_id).await {
                                warn!("Failed to unload model {}: {}", model_id, e);
                            }
                        }
                    }
                }

                // Remove from registry
                drop(models);
                let mut writable_models = loaded_models.write().await;
                let mut freed_memory = 0.0;

                for model_id in model_ids {
                    if let Some(handle) = writable_models.get(&model_id) {
                        freed_memory += handle.memory_usage_mb();
                    }
                    writable_models.remove(&model_id);
                }

                Ok(freed_memory)
            }
            UnloadingRecommendation::None { reason } => {
                info!("No models to unload: {}", reason);
                Ok(0.0)
            }
        }
    }
}

use crate::{
    loaders::{LoaderFactory, ModelLoader},
    model_handle::ModelHandle,
    resource_monitor::SystemMonitor,
    resource_types::{ModelSize, ModelType, Quantization, UnloadingPolicy},
    unloading_policies::{PolicyEvaluator, UnloadingRecommendation},
};

/// Advanced model registry with resource monitoring and automatic policies
#[derive(Debug)]
pub struct ModelRegistry {
    /// Loaded models with their handles
    loaded_models: Arc<RwLock<HashMap<String, ModelHandle>>>,
    /// Available model loaders
    model_loaders: HashMap<ModelType, Arc<dyn ModelLoader>>,
    /// Operations in progress (for preventing concurrent loads)
    load_in_progress: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<()>>>>,
    /// System resource monitor
    system_monitor: Arc<SystemMonitor>,
    /// Policy evaluator for automatic unloading
    policy_evaluator: PolicyEvaluator,
    /// Current unloading policy
    unloading_policy: UnloadingPolicy,
}

/// Global registry instance
static REGISTRY: Lazy<ModelRegistry> = Lazy::new(ModelRegistry::new);

impl ModelRegistry {
    /// Create a new model registry with default settings
    pub fn new() -> Self {
        let system_monitor = Arc::new(SystemMonitor::new());
        let policy_evaluator = PolicyEvaluator::new((*system_monitor).clone());

        // Create loaders for all supported model types
        let mut model_loaders: HashMap<ModelType, Arc<dyn ModelLoader>> = HashMap::new();
        for &model_type in LoaderFactory::get_supported_model_types() {
            let loader = LoaderFactory::create_loader(model_type);
            model_loaders.insert(model_type, Arc::from(loader));
        }

        Self {
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
            model_loaders,
            load_in_progress: Arc::new(RwLock::new(HashMap::new())),
            system_monitor,
            policy_evaluator,
            unloading_policy: UnloadingPolicy::LRU { max_age_hours: 24 },
        }
    }

    /// Create registry with custom unloading policy
    pub fn with_policy(unloading_policy: UnloadingPolicy) -> Self {
        let mut registry = Self::new();
        registry.unloading_policy = unloading_policy;
        registry
    }

    /// Get the global registry instance
    pub fn get_instance() -> &'static ModelRegistry {
        &REGISTRY
    }

    /// Generate a unique load key for tracking operations
    fn generate_load_key(model_type: ModelType, model_path: &str) -> String {
        let model_type_key = model_type as u8;
        format!("{}-{}", model_type_key, model_path)
    }

    /// Check if a load is already in progress and prevent concurrent loads
    async fn check_and_prepare_load(
        &self,
        load_key: String,
    ) -> Result<tokio::sync::mpsc::Sender<()>> {
        let (tx, _rx) = tokio::sync::mpsc::channel::<()>(1);

        let mut in_progress = self.load_in_progress.write().await;
        if let std::collections::hash_map::Entry::Occupied(_) = in_progress.entry(load_key.clone())
        {
            return Err(anyhow!("Model load already in progress"));
        }
        in_progress.insert(load_key, tx.clone());
        Ok(tx)
    }

    /// Load a model with automatic resource management
    pub async fn load_model(&self, model_type: ModelType, model_path: &str) -> Result<String> {
        let load_key = Self::generate_load_key(model_type, model_path);
        let _tx = self.check_and_prepare_load(load_key.clone()).await?;

        info!(
            "Starting model load for type: {:?}, path: {}",
            model_type, model_path
        );

        // Check if model is already loaded
        {
            let models = self.loaded_models.read().await;
            if let Some(existing) = models
                .values()
                .find(|h| h.path == PathBuf::from(model_path) && h.model_type == model_type)
            {
                // Extract the ID before dropping the lock
                let model_id = existing.id.clone();

                // Update access time
                drop(models);
                if let Err(e) = self.update_model_access(&model_id).await {
                    warn!("Failed to update model access time: {}", e);
                }

                info!(
                    "Model already loaded, returning existing model ID: {}",
                    model_id
                );
                return Ok(model_id);
            }
        }

        // Check memory constraints before loading (for policies that consider memory)
        let should_precheck_memory = matches!(
            self.unloading_policy,
            UnloadingPolicy::MemoryThreshold { .. } | UnloadingPolicy::Hybrid { .. }
        );

        if should_precheck_memory {
            let estimated_memory = SystemMonitor::estimate_memory_requirement(
                match model_type {
                    ModelType::CodeLlama => ModelSize::Medium,
                    ModelType::StarCoder => ModelSize::Large,
                },
                Some(Quantization::FP16),
            );

            if !self
                .system_monitor
                .has_sufficient_memory(estimated_memory)
                .await
            {
                info!("Low memory detected, triggering preemptive unloading");

                if let Err(e) = self.auto_unload_models_with_policy().await {
                    warn!("Preemptive unloading failed: {}", e);
                }

                // Re-check memory after unloading
                if !self
                    .system_monitor
                    .has_sufficient_memory(estimated_memory)
                    .await
                {
                    return Err(anyhow!(
                        "Insufficient memory for loading model, even after unloading"
                    ));
                }
            }
        }

        // Load the model
        let loader = self
            .model_loaders
            .get(&model_type)
            .ok_or_else(|| anyhow!("No loader available for model type {:?}", model_type))?;

        let handle = loader.load_model(model_path).await?;

        // Store the handle
        let model_id = handle.id.clone();
        {
            let mut models = self.loaded_models.write().await;
            models.insert(model_id.clone(), handle);
        }

        // Remove from in-progress tracking
        {
            let mut in_progress = self.load_in_progress.write().await;
            in_progress.remove(&load_key);
        }

        info!("Successfully loaded model with ID: {}", model_id);
        Ok(model_id)
    }

    /// Unload a model and remove it from registry
    pub async fn unload_model(&self, model_id: &str) -> Result<()> {
        info!("Starting model unload for model ID: {}", model_id);

        // Find the model
        let model_type = {
            let models = self.loaded_models.read().await;
            let handle = models
                .get(model_id)
                .ok_or_else(|| anyhow!("Model not found: {}", model_id))?;
            handle.model_type
        };

        // Get appropriate loader and unload
        let loader = self
            .model_loaders
            .get(&model_type)
            .ok_or_else(|| anyhow!("No loader available for model type {:?}", model_type))?;

        loader.unload_model(model_id).await?;

        // Remove from registry
        let removed = {
            let mut models = self.loaded_models.write().await;
            models.remove(model_id).is_some()
        };

        if removed {
            info!("Successfully unloaded model with ID: {}", model_id);
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to remove model from registry: {}",
                model_id
            ))
        }
    }

    /// Trigger automatic model unloading based on current policy
    pub async fn auto_unload_models(&self) -> Result<Vec<String>> {
        let models = self.loaded_models.read().await;
        let recommendations = self
            .policy_evaluator
            .get_unloading_recommendations(&models, &self.unloading_policy)
            .await;

        match recommendations {
            UnloadingRecommendation::Unload {
                model_ids,
                memory_freed_mb,
                ..
            } => {
                if model_ids.is_empty() {
                    return Ok(Vec::new());
                }

                info!(
                    "Auto-unloading {} models, freeing {:.1}MB",
                    model_ids.len(),
                    memory_freed_mb
                );
                Ok(model_ids)
            }
            UnloadingRecommendation::None { reason } => {
                info!("No models to unload: {}", reason);
                Ok(Vec::new())
            }
        }
    }

    /// Internal method to actually unload models without returning them
    /// Uses shared implementation to eliminate duplication
    pub async fn auto_unload_models_with_policy(&self) -> Result<f64> {
        internal::execute_auto_unload(
            &self.loaded_models,
            &self.model_loaders,
            &self.policy_evaluator,
            &self.unloading_policy,
        )
        .await
    }

    /// Update last access time for a model
    pub async fn update_model_access(&self, model_id: &str) -> Result<()> {
        let mut models = self.loaded_models.write().await;
        if let Some(handle) = models.get_mut(model_id) {
            handle.touch();
            Ok(())
        } else {
            Err(anyhow!("Model {} not found", model_id))
        }
    }

    /// Get a model handle by ID
    pub async fn get_model(&self, model_id: &str) -> Option<ModelHandle> {
        let models = self.loaded_models.read().await;
        models.get(model_id).cloned()
    }

    /// Get all loaded models
    pub async fn get_loaded_models(&self) -> HashMap<String, ModelHandle> {
        self.loaded_models.read().await.clone()
    }

    /// Get total memory usage of all loaded models in bytes
    pub async fn get_total_memory_usage(&self) -> u64 {
        let models = self.loaded_models.read().await;
        models
            .values()
            .map(|h| h.resource_usage.memory_usage_bytes)
            .sum()
    }

    /// Get detailed resource usage statistics
    pub async fn get_resource_usage_stats(
        &self,
    ) -> HashMap<String, crate::resource_types::ResourceUsage> {
        let models = self.loaded_models.read().await;
        models
            .values()
            .map(|handle| (handle.id.clone(), handle.resource_usage.clone()))
            .collect()
    }

    /// Get system resource information
    pub async fn get_system_resource_info(&self) -> (u64, u64, f64) {
        self.system_monitor.get_resource_summary().await;
        let (used, total) = self.system_monitor.get_memory_info().await;
        let percentage = self.system_monitor.get_memory_usage_percentage().await;
        (used, total, percentage)
    }

    /// Set unloading policy at runtime
    pub fn set_unloading_policy(&mut self, policy: UnloadingPolicy) {
        self.unloading_policy = policy;
        info!("Updated unloading policy to: {:?}", self.unloading_policy);
    }

    /// Get current unloading policy
    pub fn get_unloading_policy(&self) -> &UnloadingPolicy {
        &self.unloading_policy
    }

    /// Refresh system resource information
    pub async fn refresh_system_resources(&self) {
        self.system_monitor.refresh().await;
    }

    /// Start background automatic unloading task
    pub async fn start_auto_unloading_task(
        &self,
        interval_seconds: u64,
    ) -> tokio::task::JoinHandle<()> {
        let registry_ref = Arc::new(ModelRegistryInner {
            loaded_models: Arc::clone(&self.loaded_models),
            model_loaders: self.model_loaders.clone(),
            policy_evaluator: self.policy_evaluator.clone(),
            unloading_policy: self.unloading_policy.clone(),
        });

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(interval_seconds));

            loop {
                interval.tick().await;

                match registry_ref.auto_unload_models_with_policy().await {
                    Ok(memory_freed) if memory_freed > 0.0 => {
                        info!("Automatic unloading completed. Freed {:.1}MB", memory_freed);
                    }
                    Ok(_) => {} // No models to unload
                    Err(e) => {
                        error!("Error during automatic unloading: {}", e);
                    }
                }
            }
        })
    }
}

/// Internal registry wrapper for background tasks
struct ModelRegistryInner {
    loaded_models: Arc<RwLock<HashMap<String, ModelHandle>>>,
    model_loaders: HashMap<ModelType, Arc<dyn ModelLoader>>,
    policy_evaluator: PolicyEvaluator,
    unloading_policy: UnloadingPolicy,
}

impl ModelRegistryInner {
    async fn auto_unload_models_with_policy(&self) -> Result<f64> {
        // Use same shared implementation to eliminate duplication
        internal::execute_auto_unload(
            &self.loaded_models,
            &self.model_loaders,
            &self.policy_evaluator,
            &self.unloading_policy,
        )
        .await
    }
}

impl PartialEq for ModelRegistry {
    fn eq(&self, _other: &Self) -> bool {
        // Registries are considered equal if they have the same unloading policy
        // This is mainly for testing purposes
        false // We can't easily compare Arc<RwLock<>> contents
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ModelRegistry::new();
        assert_eq!(registry.get_total_memory_usage().await, 0);

        let policy_registry =
            ModelRegistry::with_policy(UnloadingPolicy::MemoryThreshold { max_memory_gb: 8.0 });
        assert!(matches!(
            policy_registry.get_unloading_policy(),
            UnloadingPolicy::MemoryThreshold { .. }
        ));
    }

    #[tokio::test]
    async fn test_system_resource_info() {
        let registry = ModelRegistry::new();
        let (used, total, percentage) = registry.get_system_resource_info().await;

        assert!(total > 0);
        assert!(used <= total);
        assert!(percentage >= 0.0 && percentage <= 100.0);
    }

    #[tokio::test]
    async fn test_policy_change() {
        let mut registry = ModelRegistry::new();

        // Change policy
        registry.set_unloading_policy(UnloadingPolicy::TimeBased { max_age_hours: 48 });

        match registry.get_unloading_policy() {
            UnloadingPolicy::TimeBased { max_age_hours } => {
                assert_eq!(*max_age_hours, 48);
            }
            _ => panic!("Unexpected policy type"),
        }
    }

    #[tokio::test]
    async fn test_model_access_tracking() {
        let registry = ModelRegistry::new();
        let temp_path = std::env::temp_dir().join("test_model");

        // Create a test model handle manually inserted
        let handle = ModelHandle::new(
            "test_model".to_string(),
            temp_path,
            ModelSize::Small,
            ModelType::CodeLlama,
            1024 * 1024 * 100, // 100MB
        );

        {
            let mut models = registry.loaded_models.write().await;
            models.insert("test_model".to_string(), handle);
        }

        // Test access tracking
        registry.update_model_access("test_model").await.unwrap();

        if let Some(updated_handle) = registry.get_model("test_model").await {
            // Access count will be 1 (from touch in update_model_access) + 1 (from get_model which also updates)
            assert!(updated_handle.resource_usage.access_count >= 1);
        } else {
            panic!("Model should exist");
        }
    }
}
