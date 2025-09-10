//! Tauri commands for plugin management

use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

use rust_ai_ide_plugins::{
    plugin_runtime::{PluginRuntime, PluginPermissions},
    marketplace_integration::MarketplaceIntegration
};
use rust_ai_ide_common::{
    IDEError, IDEErrorKind,
    validation::TauriInputSanitizer
};

// Command template macros
use crate::command_templates::{
    tauri_command_template,
    acquire_service_and_execute,
    execute_with_retry,
    CommandConfig,
    format_command_error,
};

/// App state for plugin services
#[derive(Clone)]
pub struct AppState {
    pub plugin_runtime: Arc<Mutex<PluginRuntime>>,
    pub marketplace: Arc<Mutex<MarketplaceIntegration>>,
}

impl AppState {
    pub async fn new() -> Self {
        let plugin_runtime = Arc::new(Mutex::new(PluginRuntime::new()));
        let marketplace = Arc::new(Mutex::new(MarketplaceIntegration::new()));
        Self {
            plugin_runtime,
            marketplace,
        }
    }
}

/// Command payload for listing installed plugins
#[derive(Debug, Serialize, Deserialize)]
pub struct ListPluginsPayload {
    pub include_metadata: bool,
}

/// Command payload for installing a plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct InstallPluginPayload {
    pub plugin_id: String,
    pub version: Option<String>,
    pub source_url: Option<String>,
}

/// Command payload for activating a plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivatePluginPayload {
    pub plugin_id: String,
}

/// Command payload for deactivating a plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct DeactivatePluginPayload {
    pub plugin_id: String,
}

/// Command payload for uninstalling a plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct UninstallPluginPayload {
    pub plugin_id: String,
}

/// Command payload for executing a plugin command
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutePluginCommandPayload {
    pub plugin_id: String,
    pub command: String,
    pub args: serde_json::Value,
}

/// Command payload for updating a plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePluginPayload {
    pub plugin_id: String,
    pub new_version: Option<String>,
}

/// Command payload for searching marketplace
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMarketplacePayload {
    pub query: String,
    pub category: Option<String>,
    pub limit: Option<u32>,
}

/// Command payload for rating a plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct RatePluginPayload {
    pub plugin_id: String,
    pub rating: f64,
    pub review: String,
}

/// Response for plugin operations
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginOperationResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// Command configuration
const COMMAND_CONFIG: CommandConfig = CommandConfig {
    enable_logging: true,
    log_level: log::Level::Info,
    enable_validation: true,
    async_timeout_secs: Some(30),
};

/// List all installed plugins
tauri_command_template! {
    list_installed_plugins,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let _sanitized_include_metadata = sanitizer
            .sanitize_boolean("include_metadata", &payload.include_metadata)
            .map_err(|e| format_command_error(e, "input validation"))?;

        // Acquire marketplace service
        let plugins = acquire_service_and_execute!(
            state.marketplace,
            MarketplaceIntegration,
            {
                let marketplace = state.marketplace.lock().await;
                marketplace.get_installed_plugins().await
            }
        )?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Found {} installed plugins", plugins.len()),
            data: Some(json!(plugins)),
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}

/// Search marketplace for plugins
tauri_command_template! {
    search_marketplace,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let sanitized_query = sanitizer
            .sanitize_string("query", &payload.query, 100, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        let sanitized_category = payload.category
            .as_ref()
            .map(|cat| sanitizer.sanitize_string("category", cat, 50, true))
            .transpose()
            .map_err(|e| format_command_error(e, "input validation"))?;

        // Use retry logic for marketplace API calls
        let plugins = execute_with_retry(
            || async {
                acquire_service_and_execute!(
                    app_state.marketplace,
                    MarketplaceIntegration,
                    {
                        let marketplace = app_state.marketplace.lock().await;
                        execute_with_retry(
                            || marketplace.browse_plugins(sanitized_category.as_deref(), Some(&sanitized_query)),
                            3,
                            "marketplace_browse",
                        ).await?
                    }
                )
            },
            2,
            "search_marketplace",
        ).await.map_err(|e| format_command_error(e, "marketplace search"))?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Found {} plugins matching '{}'", plugins.len(), sanitized_query),
            data: Some(json!(plugins)),
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}

/// Install a plugin
tauri_command_template! {
    install_plugin,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let sanitized_plugin_id = sanitizer
            .sanitize_string("plugin_id", &payload.plugin_id, 100, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        let sanitized_version = payload.version
            .as_ref()
            .map(|ver| sanitizer.sanitize_string("version", ver, 20, false))
            .transpose()
            .map_err(|e| format_command_error(e, "input validation"))?;

        // Validate plugin file path if provided
        if let Some(url) = &payload.source_url {
            let _sanitized_url = sanitizer
                .sanitize_url("source_url", url)
                .map_err(|e| format_command_error(e, "input validation"))?;
        }

        // Create default permissions for the plugin
        let permissions = PluginPermissions {
            can_execute: true,
            can_access_filesystem: true, // Should be configurable
            can_make_network_requests: false,
            can_interact_with_ui: false,
            memory_limit: 50 * 1024 * 1024, // 50MB
            execution_timeout: Duration::from_secs(30),
            allowed_domains: vec![],
            allowed_file_patterns: vec![],
        };

        // First download from marketplace, then load into runtime
        let install_path = execute_with_retry(
            || async {
                acquire_service_and_execute!(
                    app_state.marketplace,
                    MarketplaceIntegration,
                    {
                        let marketplace = app_state.marketplace.lock().await;
                        marketplace.download_plugin(&sanitized_plugin_id).await
                    }
                )
            },
            3,
            "download_plugin",
        ).await.map_err(|e| format_command_error(e, "plugin download"))?;

        // Load plugin into runtime
        let plugin_id = acquire_service_and_execute!(
            app_state.plugin_runtime,
            PluginRuntime,
            {
                let runtime = app_state.plugin_runtime.lock().await;
                runtime.load_plugin(&install_path, permissions).await?
            }
        )?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Plugin '{}' installed successfully", sanitized_plugin_id),
            data: Some(json!({
                "plugin_id": plugin_id,
                "install_path": install_path
            })),
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}

/// Activate/enable a plugin
tauri_command_template! {
    activate_plugin,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let sanitized_plugin_id = sanitizer
            .sanitize_string("plugin_id", &payload.plugin_id, 100, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        // Execute plugin (activation means it's already loaded and can execute)
        let input_data = json!({
            "command": "activate",
            "args": {}
        });

        let output = acquire_service_and_execute!(
            app_state.plugin_runtime,
            PluginRuntime,
            {
                let runtime = app_state.plugin_runtime.lock().await;
                let input_bytes = serde_json::to_vec(&input_data)
                    .map_err(|e| IDEError::new(IDEErrorKind::Serialization, "Failed to serialize input").with_source(e))?;
                runtime.execute_plugin(&sanitized_plugin_id, &input_bytes).await?
            }
        )?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Plugin '{}' activated successfully", sanitized_plugin_id),
            data: Some(json!({
                "output": String::from_utf8_lossy(&output).to_string()
            })),
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}

/// Deactivate/disable a plugin
tauri_command_template! {
    deactivate_plugin,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let sanitized_plugin_id = sanitizer
            .sanitize_string("plugin_id", &payload.plugin_id, 100, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        // Execute plugin deactivation command
        let input_data = json!({
            "command": "deactivate",
            "args": {}
        });

        let output = acquire_service_and_execute!(
            app_state.plugin_runtime,
            PluginRuntime,
            {
                let runtime = app_state.plugin_runtime.lock().await;
                let input_bytes = serde_json::to_vec(&input_data)
                    .map_err(|e| IDEError::new(IDEErrorKind::Serialization, "Failed to serialize input").with_source(e))?;
                runtime.execute_plugin(&sanitized_plugin_id, &input_bytes).await?
            }
        )?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Plugin '{}' deactivated successfully", sanitized_plugin_id),
            data: Some(json!({
                "output": String::from_utf8_lossy(&output).to_string()
            })),
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}

/// Uninstall a plugin
tauri_command_template! {
    uninstall_plugin,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let sanitized_plugin_id = sanitizer
            .sanitize_string("plugin_id", &payload.plugin_id, 100, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        // First unload from runtime
        acquire_service_and_execute!(
            app_state.plugin_runtime,
            PluginRuntime,
            {
                let runtime = app_state.plugin_runtime.lock().await;
                runtime.unload_plugin(&sanitized_plugin_id).await?
            }
        )?;

        // Then remove from marketplace state
        acquire_service_and_execute!(
            app_state.marketplace,
            MarketplaceIntegration,
            {
                let marketplace = app_state.marketplace.lock().await;
                marketplace.uninstall_plugin(&sanitized_plugin_id).await?
            }
        )?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Plugin '{}' uninstalled successfully", sanitized_plugin_id),
            data: None,
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}

/// Execute a command on a plugin
tauri_command_template! {
    execute_plugin_command,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let sanitized_plugin_id = sanitizer
            .sanitize_string("plugin_id", &payload.plugin_id, 100, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        let sanitized_command = sanitizer
            .sanitize_string("command", &payload.command, 50, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        // Create execution input
        let input_data = json!({
            "command": sanitized_command,
            "args": payload.args
        });

        let input_bytes = serde_json::to_vec(&input_data)
            .map_err(|e| format_command_error(e, "input serialization"))?;

        // Execute plugin
        let output = acquire_service_and_execute!(
            app_state.plugin_runtime,
            PluginRuntime,
            {
                let runtime = app_state.plugin_runtime.lock().await;
                runtime.execute_plugin(&sanitized_plugin_id, &input_bytes).await?
            }
        )?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Command '{}' executed on plugin '{}'", sanitized_command, sanitized_plugin_id),
            data: Some(json!({
                "output": String::from_utf8_lossy(&output).to_string()
            })),
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}

/// Update a plugin to a new version
tauri_command_template! {
    update_plugin,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let sanitized_plugin_id = sanitizer
            .sanitize_string("plugin_id", &payload.plugin_id, 100, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        // Update plugin using marketplace
        acquire_service_and_execute!(
            app_state.marketplace,
            MarketplaceIntegration,
            {
                let marketplace = app_state.marketplace.lock().await;
                marketplace.update_plugin(&sanitized_plugin_id).await?
            }
        )?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Plugin '{}' updated successfully", sanitized_plugin_id),
            data: None,
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}

/// Rate a plugin
tauri_command_template! {
    rate_plugin,
    {
        let sanitizer = TauriInputSanitizer::new();

        // Validate input
        let sanitized_plugin_id = sanitizer
            .sanitize_string("plugin_id", &payload.plugin_id, 100, false)
            .map_err(|e| format_command_error(e, "input validation"))?;

        let sanitized_rating = sanitizer
            .sanitize_number("rating", payload.rating, 0.0, 5.0)
            .map_err(|e| format_command_error(e, "input validation"))?;

        let sanitized_review = sanitizer
            .sanitize_string("review", &payload.review, 1000, true)
            .map_err(|e| format_command_error(e, "input validation"))?;

        // Submit rating using marketplace
        acquire_service_and_execute!(
            app_state.marketplace,
            MarketplaceIntegration,
            {
                let marketplace = app_state.marketplace.lock().await;
                marketplace.rate_plugin(&sanitized_plugin_id, sanitized_rating, &sanitized_review).await?
            }
        )?;

        let response = PluginOperationResponse {
            success: true,
            message: format!("Rating submitted for plugin '{}'", sanitized_plugin_id),
            data: None,
        };

        Ok(json!(response).to_string())
    },
    service = AppState,
    state = app_state,
    config = COMMAND_CONFIG
}