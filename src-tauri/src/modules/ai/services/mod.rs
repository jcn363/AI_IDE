//! AI Services module
//!
//! This module provides consolidated AI service management including:
//! - Service discovery and registration via AIServiceRegistry
//! - Connection pooling for efficient resource management
//! - Unified interfaces for analysis, learning, and inference operations

pub mod common;
pub mod finetune;

// Re-export the main components for easy access
pub use common::{
    AIServiceRegistry, AIServiceTrait, PoolGuard, PoolStatus, PooledServiceConfig, WrappedAIService, GLOBAL_AI_REGISTRY,
};
// Re-export finetune functionality
pub use finetune::*;
