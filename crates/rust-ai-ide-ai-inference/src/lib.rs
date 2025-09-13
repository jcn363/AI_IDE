//! # Rust AI IDE AI Inference Crate
//!
//! This crate provides model loading, inference engine, and related utilities
//! for AI-powered features in the Rust AI IDE.

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
pub use types::*;

// Re-exports from types module
pub use types::{EditOperation, SecurityError, SecurityResult};

// Re-export commonly used items for convenience
pub use inference::{
    AnalysisType, CodeCompletionConfig, GenerationConfig, InferenceEngine, InferenceError,
};
pub use loaders::{LoaderConfig, LoaderFactory, ModelLoader as LoaderTrait, ResourceAwareLoader};
pub use model_loader::{
    ModelCapabilities, ModelHandle, ModelLoadConfig, ModelLoadError, ModelLoader, ModelLoaderTrait,
};
pub use natural_language_to_code::{NLToCodeConverter, NLToCodeInput, NLToCodeResult};
pub use predictive_completion::{
    CodingStyle, CompletionContext, CompletionSuggestion, CompletionType, ContextRelevance,
    SecurityContext, SymbolContext, SymbolInfo, UserProfile,
};
