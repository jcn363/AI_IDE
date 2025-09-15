//! Main AI Module
//!
//! This module provides comprehensive AI functionality for the Rust IDE,
//! including service management, analysis, learning, and inference capabilities.

pub mod commands;
pub mod services;

// Re-export key components for backward compatibility
pub use commands::*;
// Re-export finetune functionality explicitly to avoid conflicts
pub use services::finetune;
pub use services::{
    AIServiceRegistry, AIServiceTrait, PoolGuard, PoolStatus, PooledServiceConfig,
    WrappedAIService, GLOBAL_AI_REGISTRY,
};
