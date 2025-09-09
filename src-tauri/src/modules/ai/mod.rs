//! Main AI Module
//!
//! This module provides comprehensive AI functionality for the Rust IDE,
//! including service management, analysis, learning, and inference capabilities.

pub mod services;
pub mod commands;

// Re-export key components for backward compatibility
pub use commands::*;
pub use services::{
    AIServiceTrait,
    AIServiceRegistry,
    PoolGuard,
    PoolStatus,
    WrappedAIService,
    PooledServiceConfig,
    GLOBAL_AI_REGISTRY,
};
// Re-export finetune functionality explicitly to avoid conflicts
pub use services::finetune;