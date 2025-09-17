/// ! Cross-platform plugin compatibility layer
/// Provides unified interface for plugins across Windows, macOS, and Linux

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::errors::IdeError;
use crate::platform::{Platform, PlatformFileSystem};

/// Plugin compatibility descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCompatibility {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Supported platforms
    pub platforms: Vec<String>,
    /// Platform-specific requirements
    pub requirements: HashMap<String, PlatformRequirements>,
    /// Plugin capabilities
    pub capabilities: Vec<String>,
}

/// Platform-specific requirements for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRequirements {
    /// Minimum OS version
    pub min_version: Option<String>,
    /// Required system libraries
    pub libraries: Vec<String>,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Architecture requirements
    pub architectures: Vec<String>,
}

/// Cross-platform plugin manager
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, PluginCompatibility>>>,
    platform: Platform,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            platform: Platform::name().to_string(),
        }
    }

    /// Register a plugin with compatibility information
    pub async fn register_plugin(&self, plugin: PluginCompatibility) -> Result<(), IdeError> {
        // Validate platform compatibility
        if !plugin.platforms.contains(&self.platform) {
            return Err(IdeError::Validation {
                field: "platform".to_string(),
                reason: format!("Plugin '{}' does not support platform '{}'", plugin.name, self.platform),
            });
        }

        // Check platform-specific requirements
        if let Some(reqs) = plugin.requirements.get(&self.platform) {
            Self::validate_requirements(reqs).await?;
        }

        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin.name.clone(), plugin);
        Ok(())
    }

    /// Get plugin compatibility information
    pub async fn get_plugin(&self, name: &str) -> Option<PluginCompatibility> {
        let plugins = self.plugins.read().await;
        plugins.get(name).cloned()
    }

    /// List all registered plugins
    pub async fn list_plugins(&self) -> Vec<PluginCompatibility> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    /// Check if a plugin is compatible with the current platform
    pub async fn is_compatible(&self, plugin_name: &str) -> bool {
        if let Some(plugin) = self.get_plugin(plugin_name).await {
            plugin.platforms.contains(&self.platform)
        } else {
            false
        }
    }

    /// Validate platform-specific requirements
    async fn validate_requirements(reqs: &PlatformRequirements) -> Result<(), IdeError> {
        // Check minimum OS version
        if let Some(min_version) = &reqs.min_version {
            if !Self::check_os_version(min_version) {
                return Err(IdeError::Validation {
                    field: "os_version".to_string(),
                    reason: format!("OS version {} or higher required", min_version),
                });
            }
        }

        // Check required libraries
        for lib in &reqs.libraries {
            if !Self::check_library_available(lib) {
                return Err(IdeError::Validation {
                    field: "library".to_string(),
                    reason: format!("Required library '{}' not found", lib),
                });
            }
        }

        // Check permissions (this is a placeholder - actual permission checking would be platform-specific)
        for permission in &reqs.permissions {
            if !Self::check_permission(permission) {
                return Err(IdeError::Validation {
                    field: "permission".to_string(),
                    reason: format!("Required permission '{}' not granted", permission),
                });
            }
        }

        Ok(())
    }

    /// Check if the current OS version meets minimum requirements
    fn check_os_version(min_version: &str) -> bool {
        // This is a placeholder - actual implementation would check OS version
        // For now, we'll assume compatibility
        true
    }

    /// Check if a required library is available
    fn check_library_available(library: &str) -> bool {
        // Platform-specific library checking
        #[cfg(target_os = "windows")]
        {
            // On Windows, check if DLL exists in system paths
            Self::check_windows_library(library)
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, check if dylib exists
            Self::check_macos_library(library)
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, check if shared library exists
            Self::check_linux_library(library)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            // For other platforms, assume available
            true
        }
    }

    #[cfg(target_os = "windows")]
    fn check_windows_library(library: &str) -> bool {
        // Check common Windows library locations
        let paths = [
            "C:\\Windows\\System32",
            "C:\\Windows\\SysWOW64",
            "C:\\Windows\\WinSxS",
        ];

        for path in &paths {
            let lib_path = Path::new(path).join(format!("{}.dll", library));
            if lib_path.exists() {
                return true;
            }
        }
        false
    }

    #[cfg(target_os = "macos")]
    fn check_macos_library(library: &str) -> bool {
        // Check common macOS library locations
        let paths = [
            "/usr/lib",
            "/usr/local/lib",
            "/Library/Frameworks",
        ];

        for path in &paths {
            let lib_path = Path::new(path).join(format!("lib{}.dylib", library));
            if lib_path.exists() {
                return true;
            }
        }
        false
    }

    #[cfg(target_os = "linux")]
    fn check_linux_library(library: &str) -> bool {
        // Check common Linux library locations
        let paths = [
            "/usr/lib",
            "/usr/lib64",
            "/usr/local/lib",
            "/lib",
            "/lib64",
        ];

        for path in &paths {
            let lib_path = Path::new(path).join(format!("lib{}.so", library));
            if lib_path.exists() {
                return true;
            }
        }
        false
    }

    /// Check if a required permission is granted
    fn check_permission(permission: &str) -> bool {
        // This is a placeholder for actual permission checking
        // Real implementation would check OS-specific permissions
        match permission {
            "filesystem" | "network" => true, // Basic permissions usually granted
            _ => false, // Other permissions would need specific checking
        }
    }
}

/// Plugin isolation manager
pub struct PluginIsolation {
    sandbox_enabled: bool,
    allowed_paths: Vec<PathBuf>,
}

impl PluginIsolation {
    /// Create a new plugin isolation manager
    pub fn new() -> Self {
        Self {
            sandbox_enabled: Self::should_enable_sandbox(),
            allowed_paths: Vec::new(),
        }
    }

    /// Check if sandboxing should be enabled for the current platform
    fn should_enable_sandbox() -> bool {
        #[cfg(target_os = "macos")]
        {
            // macOS has strong sandboxing support
            true
        }

        #[cfg(target_os = "windows")]
        {
            // Windows has AppContainer but limited plugin sandboxing
            false
        }

        #[cfg(target_os = "linux")]
        {
            // Linux has namespaces and seccomp for sandboxing
            true
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            false
        }
    }

    /// Add an allowed path for plugin access
    pub fn allow_path(&mut self, path: PathBuf) {
        self.allowed_paths.push(path);
    }

    /// Check if a path is allowed for plugin access
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        if !self.sandbox_enabled {
            return true; // No sandboxing enabled
        }

        self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
    }

    /// Get platform-specific sandbox configuration
    pub fn get_sandbox_config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();

        #[cfg(target_os = "macos")]
        {
            config.insert("type".to_string(), "seatbelt".to_string());
            config.insert("profile".to_string(), "plugin_sandbox".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            config.insert("type".to_string(), "appcontainer".to_string());
            config.insert("capabilities".to_string(), "internetClient".to_string());
        }

        #[cfg(target_os = "linux")]
        {
            config.insert("type".to_string(), "namespaces".to_string());
            config.insert("isolation".to_string(), "user,mount,pid".to_string());
        }

        config
    }
}

/// Plugin WebAssembly runtime compatibility
pub struct WasmRuntime {
    supported_features: Vec<String>,
}

impl WasmRuntime {
    /// Create a new WASM runtime compatibility checker
    pub fn new() -> Self {
        Self {
            supported_features: Self::get_supported_features(),
        }
    }

    /// Get supported WebAssembly features for the current platform
    fn get_supported_features() -> Vec<String> {
        let mut features = vec![
            "wasm32".to_string(),
            "multi_value".to_string(),
            "bulk_memory".to_string(),
        ];

        #[cfg(target_os = "windows")]
        {
            // Windows-specific WASM features
            features.push("windows_api".to_string());
        }

        #[cfg(target_os = "macos")]
        {
            // macOS-specific WASM features
            features.push("grand_central_dispatch".to_string());
        }

        #[cfg(target_os = "linux")]
        {
            // Linux-specific WASM features
            features.push("epoll".to_string());
        }

        features
    }

    /// Check if a WASM feature is supported
    pub fn supports_feature(&self, feature: &str) -> bool {
        self.supported_features.contains(&feature.to_string())
    }

    /// Get all supported features
    pub fn get_features(&self) -> &[String] {
        &self.supported_features
    }
}