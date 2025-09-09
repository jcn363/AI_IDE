//! Plugin interfaces for the Rust AI IDE plugin system.

pub mod enhanced_metadata;
pub mod plugin;
pub mod plugin_capabilities;
pub mod plugin_context;
pub mod plugin_metadata;

pub use enhanced_metadata::{ConflictResolution, EnhancedPluginMetadata, PluginDependency};
/// Re-export common interfaces for ease of use.
pub use plugin::{plugin_error, Plugin, PluginError};
pub use plugin_capabilities::PluginCapabilities;
pub use plugin_context::{EditorInterface, PluginContext};
pub use plugin_metadata::PluginMetadata;
