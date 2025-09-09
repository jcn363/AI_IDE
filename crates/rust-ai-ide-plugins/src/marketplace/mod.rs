//! Plugin marketplace functionality for the Rust AI IDE plugin system.
//!
//! This module provides marketplace functionality including plugin discovery,
//! installation, signature verification, and registry management.

// Re-export main components
pub mod client;
pub mod registry;

// Re-export the main registry trait and implementation
pub use registry::{PluginRegistry, PluginRegistryImpl};

// Re-export client functionality
pub use client::{
    InstallResult, InstalledPlugin, MarketplaceClient, MarketplaceSearchResult, MarketplaceServer,
};
