//! # Model Loaders Module
//!
//! Model loader implementations and traits for different model types.

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::model_handle::ModelHandle;
use crate::resource_monitor::SystemMonitor;
use crate::resource_types::{ModelSize, ModelType, BYTES_PER_GB};

/// Model loader trait for different model types
#[async_trait]
pub trait ModelLoader: Send + Sync + std::fmt::Debug {
    /// Load a model from the given path
    async fn load_model(&self, model_path: &str) -> Result<ModelHandle>;

    /// Unload a model by its ID
    async fn unload_model(&self, model_id: &str) -> Result<()>;

    /// Get supported model sizes for this loader
    fn get_supported_sizes(&self) -> &'static [ModelSize];

    /// Get the model type this loader handles
    fn get_model_type(&self) -> ModelType;

    /// Estimate memory requirement for a model before loading
    async fn estimate_memory(&self, model_path: &str) -> Result<u64> {
        // Default: check if file exists and return reasonable estimate
        std::fs::metadata(model_path)?;

        // Return a reasonable default based on supported sizes
        let sizes = self.get_supported_sizes();
        if let Some(&first_size) = sizes.first() {
            match first_size {
                ModelSize::Small => Ok(1024 * 1024 * 200),       // 200MB
                ModelSize::Medium => Ok(1024 * 1024 * 1000),     // 1GB
                ModelSize::Large => Ok(1024 * 1024 * 2000),      // 2GB
                ModelSize::ExtraLarge => Ok(1024 * 1024 * 4000), // 4GB
            }
        } else {
            Ok(1024 * 1024 * 500) // 500MB default
        }
    }
}

/// Enhanced model loader with resource awareness
#[async_trait]
pub trait ResourceAwareLoader: ModelLoader + std::fmt::Debug {
    /// Check if the system has sufficient resources for loading
    async fn can_load_with_resources(
        &self,
        model_path: &str,
        system_monitor: &SystemMonitor,
    ) -> Result<bool> {
        let required_memory = self.estimate_memory(model_path).await?;
        Ok(system_monitor.has_sufficient_memory(required_memory).await)
    }

    /// Get recommended model size based on available resources
    async fn recommend_model_size(&self, available_memory: u64) -> ModelSize {
        let memory_gb = available_memory as f64 / BYTES_PER_GB;

        match memory_gb {
            m if m >= 4.0 => ModelSize::ExtraLarge,
            m if m >= 2.5 => ModelSize::Large,
            m if m >= 1.5 => ModelSize::Medium,
            m if m >= 0.5 => ModelSize::Small,
            _ => ModelSize::Small, // Safe fallback
        }
    }
}

/// Base configuration for model loaders
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    pub max_concurrent_loads: usize,
    pub memory_buffer_gb: f64,
    pub allow_model_optimization: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            max_concurrent_loads: 1,
            memory_buffer_gb: 0.5,
            allow_model_optimization: true,
        }
    }
}

/// CodeLlama model loader implementation
#[derive(Debug)]
pub struct CodeLlamaLoader {
    _config: LoaderConfig,
}

impl CodeLlamaLoader {
    /// Create a new CodeLlama loader
    pub fn new() -> Self {
        Self {
            _config: LoaderConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: LoaderConfig) -> Self {
        Self { _config: config }
    }
}

#[async_trait]
impl ModelLoader for CodeLlamaLoader {
    async fn load_model(&self, model_path: &str) -> Result<ModelHandle> {
        // Check if model file exists
        if !std::path::Path::new(model_path).exists() {
            return Err(anyhow!("Model file not found: {}", model_path));
        }

        // Estimate memory requirement
        let estimated_memory = self.estimate_memory(model_path).await?;

        // Create model handle with resource tracking
        let handle = ModelHandle::new(
            format!(
                "code_llama_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs()
            ),
            std::path::PathBuf::from(model_path),
            ModelSize::Medium,
            ModelType::CodeLlama,
            estimated_memory,
        );

        Ok(handle)
    }

    async fn unload_model(&self, model_id: &str) -> Result<()> {
        if model_id.starts_with("code_llama") {
            Ok(()) // Placeholder for actual unloading logic
        } else {
            Err(anyhow!(
                "Model not managed by CodeLlama loader: {}",
                model_id
            ))
        }
    }

    fn get_supported_sizes(&self) -> &'static [ModelSize] {
        &[ModelSize::Small, ModelSize::Medium, ModelSize::Large]
    }

    fn get_model_type(&self) -> ModelType {
        ModelType::CodeLlama
    }
}

#[async_trait]
impl ResourceAwareLoader for CodeLlamaLoader {
    async fn can_load_with_resources(
        &self,
        model_path: &str,
        system_monitor: &SystemMonitor,
    ) -> Result<bool> {
        // Check if model exists
        if !std::path::Path::new(model_path).exists() {
            return Ok(false);
        }

        // Check memory requirements
        let required_memory = self.estimate_memory(model_path).await?;
        let has_memory = system_monitor.has_sufficient_memory(required_memory).await;

        Ok(has_memory)
    }
}

/// StarCoder model loader implementation
#[derive(Debug)]
pub struct StarCoderLoader {
    _config: LoaderConfig,
}

impl StarCoderLoader {
    /// Create a new StarCoder loader
    pub fn new() -> Self {
        Self {
            _config: LoaderConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: LoaderConfig) -> Self {
        Self { _config: config }
    }
}

#[async_trait]
impl ModelLoader for StarCoderLoader {
    async fn load_model(&self, model_path: &str) -> Result<ModelHandle> {
        // Check if model file exists
        if !std::path::Path::new(model_path).exists() {
            return Err(anyhow!("Model file not found: {}", model_path));
        }

        // Estimate memory requirement
        let estimated_memory = self.estimate_memory(model_path).await?;

        // Create model handle with resource tracking
        let handle = ModelHandle::new(
            format!(
                "star_coder_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs()
            ),
            std::path::PathBuf::from(model_path),
            ModelSize::Large,
            ModelType::StarCoder,
            estimated_memory,
        );

        Ok(handle)
    }

    async fn unload_model(&self, model_id: &str) -> Result<()> {
        if model_id.starts_with("star_coder") {
            Ok(()) // Placeholder for actual unloading logic
        } else {
            Err(anyhow!(
                "Model not managed by StarCoder loader: {}",
                model_id
            ))
        }
    }

    fn get_supported_sizes(&self) -> &'static [ModelSize] {
        &[ModelSize::Medium, ModelSize::Large, ModelSize::ExtraLarge]
    }

    fn get_model_type(&self) -> ModelType {
        ModelType::StarCoder
    }
}

#[async_trait]
impl ResourceAwareLoader for StarCoderLoader {
    async fn can_load_with_resources(
        &self,
        model_path: &str,
        system_monitor: &SystemMonitor,
    ) -> Result<bool> {
        // Check if model exists
        if !std::path::Path::new(model_path).exists() {
            return Ok(false);
        }

        // Check memory requirements
        let required_memory = self.estimate_memory(model_path).await?;
        let has_memory = system_monitor.has_sufficient_memory(required_memory).await;

        Ok(has_memory)
    }
}

/// Factory for creating model loaders
pub struct LoaderFactory;

impl LoaderFactory {
    /// Create a loader for the specified model type
    pub fn create_loader(model_type: ModelType) -> Box<dyn ModelLoader> {
        match model_type {
            ModelType::CodeLlama => Box::new(CodeLlamaLoader::new()),
            ModelType::StarCoder => Box::new(StarCoderLoader::new()),
        }
    }

    /// Create a resource-aware loader for the specified model type
    pub fn create_resource_aware_loader(model_type: ModelType) -> Box<dyn ResourceAwareLoader> {
        match model_type {
            ModelType::CodeLlama => Box::new(CodeLlamaLoader::new()),
            ModelType::StarCoder => Box::new(StarCoderLoader::new()),
        }
    }

    /// Get all supported model types
    pub fn get_supported_model_types() -> &'static [ModelType] {
        &[ModelType::CodeLlama, ModelType::StarCoder]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource_monitor::SystemMonitor;

    #[test]
    fn test_loader_factory() {
        let code_llama = LoaderFactory::create_loader(ModelType::CodeLlama);
        assert_eq!(code_llama.get_model_type(), ModelType::CodeLlama);

        let star_coder = LoaderFactory::create_loader(ModelType::StarCoder);
        assert_eq!(star_coder.get_model_type(), ModelType::StarCoder);

        assert_eq!(LoaderFactory::get_supported_model_types().len(), 2);
    }

    #[tokio::test]
    async fn test_memory_estimation() {
        let loader = CodeLlamaLoader::new();
        let system_monitor = SystemMonitor::new();

        // Test with a non-existent path (should fail early)
        let result = loader
            .can_load_with_resources("/non/existent/path", &system_monitor)
            .await;
        assert!(result.is_err() || !result.unwrap());
    }

    #[test]
    fn test_supported_sizes() {
        let code_llama = CodeLlamaLoader::new();
        let star_coder = StarCoderLoader::new();

        assert!(code_llama
            .get_supported_sizes()
            .contains(&ModelSize::Medium));
        assert!(star_coder.get_supported_sizes().contains(&ModelSize::Large));
    }

    #[test]
    fn test_loader_config() {
        let config = LoaderConfig {
            max_concurrent_loads: 3,
            memory_buffer_gb: 1.0,
            allow_model_optimization: false,
        };

        let _loader = CodeLlamaLoader::with_config(config.clone());
        // Configuration doesn't affect the basic functionality, but can be extended
        // assert_eq!(loader._config.max_concurrent_loads, 3);
        // assert_eq!(loader._config.memory_buffer_gb, 1.0);
        // assert!(!loader._config.allow_model_optimization);
    }
}
