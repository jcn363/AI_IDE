//! # Lazy Loading Infrastructure for AI Inference and LSP Services
//!
//! This crate provides lazy loading mechanisms and memory pooling for performance optimization
//! of AI inference and LSP services in the Rust AI IDE.
//!
//! ## Key Features
//!
//! ### Lazy Loading
//! - Thread-safe lazy initialization using `once_cell`
//! - Async lazy loading with proper error handling
//! - Component registration and discovery
//! - Startup time optimization
//!
//! ### Memory Pooling
//! - Object pooling for frequently allocated objects (analysis results, model states)
//! - LRU-based eviction policies
//! - Memory usage monitoring and limits
//! - Zero-copy optimizations where possible

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub mod error;
pub mod lazy_loader;
pub mod memory_pool;
pub mod performance_monitor;

pub use error::*;
pub use lazy_loader::*;
pub use memory_pool::*;
// Re-export commonly used types
pub use once_cell::sync::Lazy;
pub use performance_monitor::*;

/// Configuration for lazy loading behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyLoadingConfig {
    /// Maximum number of concurrent lazy loads
    pub max_concurrent_loads:          usize,
    /// Timeout for lazy loading operations (in seconds)
    pub load_timeout_seconds:          u64,
    /// Memory pool size limits
    pub memory_pool_limits:            MemoryPoolLimits,
    /// Whether to enable performance monitoring
    pub enable_performance_monitoring: bool,
}

impl Default for LazyLoadingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_loads:          10,
            load_timeout_seconds:          30,
            memory_pool_limits:            MemoryPoolLimits::default(),
            enable_performance_monitoring: true,
        }
    }
}

/// Memory pool size limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolLimits {
    /// Maximum size for analysis result pool (in items)
    pub analysis_result_pool_max: usize,
    /// Maximum size for model state pool (in items)
    pub model_state_pool_max:     usize,
    /// Maximum memory usage for pools (in bytes)
    pub max_memory_usage:         usize,
}

impl Default for MemoryPoolLimits {
    fn default() -> Self {
        Self {
            analysis_result_pool_max: 1000,
            model_state_pool_max:     50,
            max_memory_usage:         100 * 1024 * 1024, // 100MB
        }
    }
}

/// Global lazy loading registry
pub type LazyRegistry = Arc<RwLock<HashMap<String, Box<dyn LazyComponent>>>>;

/// Trait for components that can be lazily loaded
#[async_trait]
pub trait LazyComponent: Send + Sync {
    /// Get the component name
    fn name(&self) -> &str;

    /// Check if component is loaded
    fn is_loaded(&self) -> bool;

    /// Load the component asynchronously
    async fn load(&mut self) -> Result<(), LazyLoadingError>;

    /// Get memory usage of the component (in bytes)
    fn memory_usage(&self) -> usize;

    /// Unload the component to free memory
    async fn unload(&mut self) -> Result<(), LazyLoadingError>;

    /// Clone the component as a boxed trait object
    fn clone_box(&self) -> Box<dyn LazyComponent>;
}

/// Trait for objects that can be pooled
pub trait Poolable: Send + Sync + Sized {
    /// Get the size of the object in bytes
    fn size_bytes(&self) -> usize;

    /// Reset the object to its initial state for reuse
    fn reset(&mut self);
}

/// Initialize the lazy loading system with default configuration
pub async fn init_lazy_loading() -> Result<LazyRegistry, LazyLoadingError> {
    init_lazy_loading_with_config(LazyLoadingConfig::default()).await
}

/// Initialize the lazy loading system with custom configuration
pub async fn init_lazy_loading_with_config(config: LazyLoadingConfig) -> Result<LazyRegistry, LazyLoadingError> {
    let registry = Arc::new(RwLock::new(HashMap::new()));

    // Initialize performance monitoring if enabled
    if config.enable_performance_monitoring {
        PerformanceMonitor::init().await?;
    }

    Ok(registry)
}

/// Macro for creating lazy-loaded components
#[macro_export]
macro_rules! lazy_component {
    ($name:expr, $init:expr) => {
        static LAZY_COMPONENT: once_cell::sync::Lazy<std::sync::Arc<tokio::sync::Mutex<Option<_>>>> =
            once_cell::sync::Lazy::new(|| std::sync::Arc::new(tokio::sync::Mutex::new(None)));

        pub async fn get_component() -> Result<std::sync::Arc<_>, $crate::LazyLoadingError> {
            let mut component = LAZY_COMPONENT.lock().await;
            if component.is_none() {
                *component = Some(std::sync::Arc::new($init));
            }
            Ok(component.as_ref().unwrap().clone())
        }

        pub async fn is_component_loaded() -> bool {
            LAZY_COMPONENT.lock().await.is_some()
        }
    };
}

/// Macro for creating memory-pooled objects
#[macro_export]
macro_rules! pooled_object {
    ($pool_name:ident, $type:ty, $max_size:expr) => {
        static OBJECT_POOL: once_cell::sync::Lazy<std::sync::Arc<tokio::sync::Mutex<$crate::ObjectPool<$type>>>> =
            once_cell::sync::Lazy::new(|| {
                std::sync::Arc::new(tokio::sync::Mutex::new($crate::ObjectPool::new($max_size)))
            });

        pub async fn acquire_pooled_object(
        ) -> Result<std::sync::Arc<tokio::sync::Mutex<$type>>, $crate::LazyLoadingError> {
            let pool = OBJECT_POOL.lock().await;
            pool.acquire().await
        }

        pub async fn release_pooled_object(
            obj: std::sync::Arc<tokio::sync::Mutex<$type>>,
        ) -> Result<(), $crate::LazyLoadingError> {
            let pool = OBJECT_POOL.lock().await;
            pool.release(obj).await
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lazy_loading_initialization() {
        let registry = init_lazy_loading().await.unwrap();
        assert!(registry.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_default_config() {
        let config = LazyLoadingConfig::default();
        assert_eq!(config.max_concurrent_loads, 10);
        assert_eq!(config.load_timeout_seconds, 30);
        assert!(config.enable_performance_monitoring);
    }
}
