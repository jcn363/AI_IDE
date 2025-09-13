//! # Model Handle Module
//!
//! Model handle and related structures for tracking loaded models.

use std::path::PathBuf;
use std::time::Instant;

use chrono::{DateTime, Utc};

use crate::resource_types::{ModelSize, ModelType, ResourceUsage};

/// Model handle with clone support and resource tracking
#[derive(Debug, Clone)]
pub struct ModelHandle {
    pub id:             String,
    pub path:           PathBuf,
    pub size:           ModelSize,
    pub loaded_at:      Instant,
    pub model_type:     ModelType,
    pub resource_usage: ResourceUsage,
}

impl ModelHandle {
    /// Create a new model handle
    pub fn new(id: String, path: PathBuf, size: ModelSize, model_type: ModelType, memory_usage_bytes: u64) -> Self {
        let now = Utc::now();
        Self {
            id,
            path,
            size,
            loaded_at: Instant::now(),
            model_type,
            resource_usage: ResourceUsage {
                memory_usage_bytes,
                last_accessed: now,
                access_count: 0,
                load_timestamp: now,
            },
        }
    }

    /// Update the last accessed timestamp
    pub fn touch(&mut self) {
        self.resource_usage.last_accessed = Utc::now();
        self.resource_usage.access_count += 1;
    }

    /// Get the age since last access in hours
    pub fn age_since_last_access_hours(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.resource_usage.last_accessed)
            .num_hours()
    }

    /// Get the age since loading in hours
    pub fn age_since_load_hours(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.resource_usage.load_timestamp)
            .num_hours()
    }

    /// Check if model should be unloaded based on LRU policy
    pub fn should_unload_lru(&self, max_age_hours: u32) -> bool {
        self.age_since_last_access_hours() >= max_age_hours as i64
    }

    /// Check if model should be unloaded based on time-based policy
    pub fn should_unload_time_based(&self, max_age_hours: u32) -> bool {
        self.age_since_load_hours() >= max_age_hours as i64
    }

    /// Get memory usage in human readable format
    pub fn memory_usage_mb(&self) -> f64 {
        self.resource_usage.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Check if the model is still accessible (not removed)
    pub fn is_accessible(&self) -> bool {
        // Check if the path still exists
        self.path.exists() && self.path.is_file()
    }

    /// Get model metadata for display/logging
    pub fn metadata(&self) -> ModelMetadata {
        ModelMetadata {
            id:            self.id.clone(),
            model_type:    self.model_type,
            size:          self.size,
            memory_mb:     self.memory_usage_mb(),
            age_hours:     self.age_since_load_hours() as f64,
            access_count:  self.resource_usage.access_count,
            last_accessed: self.resource_usage.last_accessed,
        }
    }
}

/// Metadata for model information display
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub id:            String,
    pub model_type:    ModelType,
    pub size:          ModelSize,
    pub memory_mb:     f64,
    pub age_hours:     f64,
    pub access_count:  u64,
    pub last_accessed: DateTime<Utc>,
}

impl ModelMetadata {
    /// Format for logging
    pub fn format_for_log(&self) -> String {
        format!(
            "Model {} (type: {:?}, size: {:?}, memory: {:.1}MB, age: {:.1}h, accesses: {})",
            self.id, self.model_type, self.size, self.memory_mb, self.age_hours, self.access_count
        )
    }

    /// Check if this model is a good candidate for unloading based on metadata
    pub fn is_unloading_candidate(&self, max_age_hours: f64, max_memory_mb: f64) -> bool {
        self.age_hours >= max_age_hours || self.memory_mb >= max_memory_mb
    }
}

/// Builder pattern for creating model handles
#[derive(Debug)]
pub struct ModelHandleBuilder {
    pub id:                 Option<String>,
    pub path:               Option<PathBuf>,
    pub size:               Option<ModelSize>,
    pub model_type:         Option<ModelType>,
    pub memory_usage_bytes: Option<u64>,
}

impl ModelHandleBuilder {
    pub fn new() -> Self {
        Self {
            id:                 None,
            path:               None,
            size:               None,
            model_type:         None,
            memory_usage_bytes: None,
        }
    }

    pub fn id<T: Into<String>>(mut self, id: T) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn path<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn size(mut self, size: ModelSize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn model_type(mut self, model_type: ModelType) -> Self {
        self.model_type = Some(model_type);
        self
    }

    pub fn memory_usage_bytes(mut self, memory: u64) -> Self {
        self.memory_usage_bytes = Some(memory);
        self
    }

    pub fn build(self) -> Result<ModelHandle, String> {
        let id = self.id.ok_or("Model ID is required")?;
        let path = self.path.ok_or("Model path is required")?;
        let size = self.size.ok_or("Model size is required")?;
        let model_type = self.model_type.ok_or("Model type is required")?;
        let memory_usage_bytes = self.memory_usage_bytes.unwrap_or(1024 * 1024 * 100); // 100MB default

        Ok(ModelHandle::new(
            id,
            path,
            size,
            model_type,
            memory_usage_bytes,
        ))
    }
}

impl Default for ModelHandleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_model_handle_builder() {
        let handle = ModelHandleBuilder::new()
            .id("test_model")
            .path("/tmp/test")
            .size(ModelSize::Medium)
            .model_type(ModelType::CodeLlama)
            .memory_usage_bytes(1024 * 1024 * 500)
            .build()
            .expect("Failed to build model handle in inference test");

        assert_eq!(handle.id, "test_model");
        assert_eq!(handle.size, ModelSize::Medium);
        assert_eq!(handle.model_type, ModelType::CodeLlama);
        assert_eq!(handle.resource_usage.memory_usage_bytes, 1024 * 1024 * 500);
    }

    #[test]
    fn test_should_unload_policies() {
        let mut handle = ModelHandleBuilder::new()
            .id("test")
            .path("/tmp/test")
            .size(ModelSize::Small)
            .model_type(ModelType::CodeLlama)
            .build()
            .expect("Failed to build model handle in unload test");

        // Simulate old model by manually setting timestamp
        handle.resource_usage.load_timestamp = Utc::now() - chrono::Duration::hours(25);

        assert!(handle.should_unload_time_based(24));
        assert!(!handle.should_unload_time_based(48));
    }

    #[test]
    fn test_access_tracking() {
        let mut handle = ModelHandleBuilder::new()
            .id("test")
            .path("/tmp/test")
            .size(ModelSize::Small)
            .model_type(ModelType::CodeLlama)
            .build()
            .expect("Failed to build model handle in access tracking test");

        assert_eq!(handle.resource_usage.access_count, 0);

        handle.touch();
        assert_eq!(handle.resource_usage.access_count, 1);

        let metadata = handle.metadata();
        assert_eq!(metadata.access_count, 1);
    }
}
