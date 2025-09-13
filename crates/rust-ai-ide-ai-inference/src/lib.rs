//! # Rust AI IDE AI Inference Crate
//!
//! This crate provides model loading, inference engine, and related utilities
//! for AI-powered features in the Rust AI IDE.
//!
//! ## Lazy Loading Support
//!
//! This crate implements lazy loading for performance optimization:
//! - Predictive completion models are loaded on-demand
//! - Natural language to code conversion is lazy-loaded
//! - Memory pooling for frequently allocated objects

use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

/// Lazy loading configuration for AI inference services
pub static LAZY_LOADING_CONFIG: Lazy<rust_ai_ide_lazy_loading::LazyLoadingConfig> = Lazy::new(|| {
    rust_ai_ide_lazy_loading::LazyLoadingConfig {
        max_concurrent_loads:          5,  // Lower concurrency for AI models
        load_timeout_seconds:          60, // Longer timeout for model loading
        memory_pool_limits:            rust_ai_ide_lazy_loading::MemoryPoolLimits {
            analysis_result_pool_max: 100,
            model_state_pool_max:     10, // Fewer model states due to memory usage
            max_memory_usage:         500 * 1024 * 1024, // 500MB limit for AI models
        },
        enable_performance_monitoring: true,
    }
});

/// Global lazy loader instance for AI inference services
pub static LAZY_LOADER: Lazy<Arc<rust_ai_ide_lazy_loading::LazyLoader>> = Lazy::new(|| {
    Arc::new(rust_ai_ide_lazy_loading::LazyLoader::new(
        LAZY_LOADING_CONFIG.clone(),
    ))
});

/// Global memory pool manager for AI objects
pub static MEMORY_POOL_MANAGER: Lazy<Arc<rust_ai_ide_lazy_loading::MemoryPoolManager>> = Lazy::new(|| {
    Arc::new(rust_ai_ide_lazy_loading::MemoryPoolManager::new(
        LAZY_LOADING_CONFIG
            .memory_pool_limits
            .analysis_result_pool_max,
        LAZY_LOADING_CONFIG.memory_pool_limits.model_state_pool_max,
        LAZY_LOADING_CONFIG.memory_pool_limits.max_memory_usage,
    ))
});

/// Initialize lazy loading for AI inference services
pub async fn init_lazy_loading() -> rust_ai_ide_lazy_loading::LazyResult<()> {
    // Initialize performance monitoring
    rust_ai_ide_lazy_loading::PerformanceMonitor::init().await?;

    // Register lazy components
    register_lazy_components().await?;

    tracing::info!("AI inference lazy loading initialized successfully");
    Ok(())
}

/// Register lazy-loadable components
async fn register_lazy_components() -> rust_ai_ide_lazy_loading::LazyResult<()> {
    let loader = LAZY_LOADER.clone();

    // Register predictive completion engine (heavy component)
    let predictive_completion_component =
        rust_ai_ide_lazy_loading::SimpleLazyComponent::new("predictive_completion", || async {
            // This would initialize the heavy predictive completion models
            // For now, just return success - full implementation would load models
            Ok(Arc::new(()) as Arc<dyn std::any::Any + Send + Sync>)
        });
    loader
        .register_component(Box::new(predictive_completion_component))
        .await?;

    // Register natural language to code converter
    let nlp_component = rust_ai_ide_lazy_loading::SimpleLazyComponent::new("natural_language_to_code", || async {
        // This would initialize NLP models
        Ok(Arc::new(()) as Arc<dyn std::any::Any + Send + Sync>)
    });
    loader.register_component(Box::new(nlp_component)).await?;

    Ok(())
}

/// Get performance report for AI inference lazy loading
pub async fn get_performance_report() -> rust_ai_ide_lazy_loading::PerformanceReport {
    if let Some(monitor) = rust_ai_ide_lazy_loading::PerformanceMonitor::global() {
        monitor.generate_performance_report().await
    } else {
        // Return empty report if monitoring not initialized
        rust_ai_ide_lazy_loading::PerformanceReport {
            startup_performance:    Default::default(),
            memory_usage_stats:     Default::default(),
            pool_performance_stats: Vec::new(),
            timestamp:              std::time::SystemTime::now(),
        }
    }
}

/// Core types used throughout the inference system
pub mod types;

/// Model inference engine and related functionality
pub mod inference;

/// Natural language to code conversion
pub mod natural_language_to_code;

/// Model loading and management
pub mod model_loader;

/// Generic loader implementations
pub mod loaders;

/// Model handle structures
pub mod model_handle;

/// Model registry for tracking loaded models
pub mod registry;

/// System resource monitoring
pub mod resource_monitor;

/// Predictive code completion
pub mod predictive_completion;

/// Resource type definitions
pub mod resource_types;

/// Unloading policies for memory management
pub mod unloading_policies;

// Re-exports for convenience
// Re-export commonly used items for convenience
pub use inference::{AnalysisType, CodeCompletionConfig, GenerationConfig, InferenceEngine, InferenceError};
pub use loaders::{LoaderConfig, LoaderFactory, ModelLoader as LoaderTrait, ResourceAwareLoader};
pub use model_loader::{
    ModelCapabilities, ModelHandle, ModelLoadConfig, ModelLoadError, ModelLoader, ModelLoaderTrait,
};
pub use natural_language_to_code::{NLToCodeConverter, NLToCodeInput, NLToCodeResult};
pub use predictive_completion::{
    CodingStyle, CompletionContext, CompletionSuggestion, CompletionType, ContextRelevance, SecurityContext,
    SymbolContext, SymbolInfo, UserProfile,
};
pub use types::*;
// Re-exports from types module
pub use types::{EditOperation, SecurityError, SecurityResult};
