//! Core memory optimization manager and configuration
//! This module provides the central orchestration for memory optimization,
//! leak detection, and performance monitoring.

mod config;
mod manager;

pub use config::MemoryOptimizationConfig;
pub use manager::MemoryOptimizationManager;
