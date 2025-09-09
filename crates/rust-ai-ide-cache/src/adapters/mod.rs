//! Cache Adapters for Backward Compatibility
//!
//! This module provides adapter implementations that allow existing cache
//! interfaces to work with the unified cache system.

pub mod lsp_adapter;

pub use lsp_adapter::*;
