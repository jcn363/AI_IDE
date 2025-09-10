//! Rust AI IDE Plugin System
//!
//! This crate provides a comprehensive plugin system for the Rust AI IDE,
//! allowing extensions and third-party plugins to extend the editor's functionality.

pub mod dependency_resolver;
pub mod interfaces;
pub mod loader;
pub mod marketplace;
pub mod mission_control;
pub mod plugin_runtime;
pub mod marketplace_integration;
pub mod registry;

pub use dependency_resolver::*;
/// Re-export the main plugin system components for user convenience.
pub use interfaces::*;
pub use loader::PluginLoader;
pub use marketplace::{client::MarketplaceClient, registry::PluginRegistry as MarketplaceRegistry};
pub use registry::PluginRegistry;

/// Enhanced plugin initialization using Mission Control architecture
pub async fn init_enhanced_plugin_system() -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder for Mission Control initialization
    Ok(())
}

/// Initialize the enhanced plugin system with dependency resolution
pub async fn init_plugin_system_with_dependencies(
    available_plugins: Vec<PluginMetadata>,
) -> Result<(PluginRegistry, DependencyResolver), Box<dyn std::error::Error>> {
    let registry = PluginRegistry::new();
    let resolver = DependencyResolver::new();
    // resolver.add_available_plugins(available_plugins).await;

    Ok((registry, resolver))
}

/// Initialize the plugin system.
/// This function should be called during IDE startup.
pub async fn init_plugin_system() -> Result<PluginRegistry, Box<dyn std::error::Error>> {
    let mut registry = PluginRegistry::new();
    let loader = PluginLoader::new(&mut registry);
    loader.load_plugins_from_directory("./plugins").await?;
    Ok(registry)
}
