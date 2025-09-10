//! Core memory optimization manager and configuration
//! This module provides the central orchestration for memory optimization,
//! leak detection, and performance monitoring.

mod manager;
mod config;

pub use manager::MemoryOptimizationManager;
pub use config::MemoryOptimizationConfig;