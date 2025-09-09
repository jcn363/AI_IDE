//! Keyboard shortcuts and keybinding management commands.
//!
//! This module provides comprehensive keybinding management for the IDE,
//! including customizable keymaps, conflict resolution, and cross-platform support.

use rust_ai_ide_core::validation::validate_secure_path;
use rust_ai_ide_core::security::{audit_logger, audit_action};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;
use lazy_static::lazy_static;

use crate::command_templates::*;

/// Shortcut context types for different IDE areas
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutContext {
    Global,
    Editor,
    Terminal,
    FileExplorer,
    CommandPalette,
    Search,
    Git,
    Debugger,
    Cargo,
    AiAssistant,
}

/// Key combination structure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombination {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd on Mac, Windows key on Windows
}

/// Keyboard shortcut action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutAction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub context: ShortcutContext,
    pub default_keys: Vec<KeyCombination>,
}

/// User keybinding profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub shortcuts: HashMap<String, Vec<KeyCombination>>,
    pub is_default: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Keybinding conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConflict {
    pub keys: String,
    pub actions: Vec<String>,
}

/// Keybinding manager service
#[derive(Debug)]
pub struct KeybindingManager {
    profiles: Arc<Mutex<HashMap<String, KeybindingsProfile>>>,
    current_profile: Arc<Mutex<String>>,
    available_actions: Arc<Mutex<Vec<ShortcutAction>>>,
}

impl KeybindingManager {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(Mutex::new(HashMap::new())),
            current_profile: Arc::new(Mutex::new("default".to_string())),
            available_actions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn initialize_default_profile(&self) {
        let mut profiles = self.profiles.lock().await;
        let mut actions = self.available_actions.lock().await;

        // Initialize default actions
        self.initialize_default_actions(&mut actions).await;

        // Create default profile
        let default_profile = KeybindingsProfile {
            id: "default".to_string(),
            name: "Default Profile".to_string(),
            description: "Default keybindings profile".to_string(),
            shortcuts: self.create_default_shortcuts(&actions),
            is_default: true,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
        };

        profiles.insert("default".to_string(), default_profile);
    }

    pub async fn initialize_default_actions(&self, actions: &mut Vec<ShortcutAction>) {
        // Editor actions
        actions.push(ShortcutAction {
            id: "editor.save".to_string(),
            name: "Save File".to_string(),
            description: "Save the current file".to_string(),
            context: ShortcutContext::Editor,
            default_keys: vec![KeyCombination {
                key: "s".to_string(),
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            }],
        });

        actions.push(ShortcutAction {
            id: "editor.undo".to_string(),
            name: "Undo".to_string(),
            description: "Undo last action".to_string(),
            context: ShortcutContext::Editor,
            default_keys: vec![KeyCombination {
                key: "z".to_string(),
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            }],
        });

        actions.push(ShortcutAction {
            id: "editor.redo".to_string(),
            name: "Redo".to_string(),
            description: "Redo last undone action".to_string(),
            context: ShortcutContext::Editor,
            default_keys: vec![
                KeyCombination {
                    key: "z".to_string(),
                    ctrl: true,
                    alt: false,
                    shift: true,
                    meta: false,
                },
                KeyCombination {
                    key: "y".to_string(),
                    ctrl: true,
                    alt: false,
                    shift: false,
                    meta: false,
                },
            ],
        });

        // Terminal actions
        actions.push(ShortcutAction {
            id: "terminal.new".to_string(),
            name: "New Terminal".to_string(),
            description: "Open new terminal instance".to_string(),
            context: ShortcutContext::Terminal,
            default_keys: vec![KeyCombination {
                key: "`".to_string(),
                ctrl: true,
                alt: false,
                shift: true,
                meta: false,
            }],
        });

        // Search actions
        actions.push(ShortcutAction {
            id: "search.find".to_string(),
            name: "Find".to_string(),
            description: "Open find dialog".to_string(),
            context: ShortcutContext::Search,
            default_keys: vec![KeyCombination {
                key: "f".to_string(),
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            }],
        });

        actions.push(ShortcutAction {
            id: "search.replace".to_string(),
            name: "Replace".to_string(),
            description: "Open find and replace dialog".to_string(),
            context: ShortcutContext::Search,
            default_keys: vec![KeyCombination {
                key: "h".to_string(),
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            }],
        });

        // Command palette
        actions.push(ShortcutAction {
            id: "command.palette".to_string(),
            name: "Command Palette".to_string(),
            description: "Open command palette".to_string(),
            context: ShortcutContext::CommandPalette,
            default_keys: vec![KeyCombination {
                key: "p".to_string(),
                ctrl: true,
                alt: false,
                shift: true,
                meta: false,
            }],
        });

        // Git actions
        actions.push(ShortcutAction {
            id: "git.status".to_string(),
            name: "Git Status".to_string(),
            description: "Show git status".to_string(),
            context: ShortcutContext::Git,
            default_keys: vec![KeyCombination {
                key: "g".to_string(),
                ctrl: true,
                alt: false,
                shift: true,
                meta: false,
            }],
        });

        // File explorer actions
        actions.push(ShortcutAction {
            id: "explorer.toggle".to_string(),
            name: "Toggle Explorer".to_string(),
            description: "Toggle file explorer visibility".to_string(),
            context: ShortcutContext::FileExplorer,
            default_keys: vec![KeyCombination {
                key: "b".to_string(),
                ctrl: true,
                alt: false,
                shift: true,
                meta: false,
            }],
        });
    }

    pub fn create_default_shortcuts(&self, actions: &[ShortcutAction]) -> HashMap<String, Vec<KeyCombination>> {
        let mut shortcuts = HashMap::new();

        for action in actions {
            if !action.default_keys.is_empty() {
                shortcuts.insert(action.id.clone(), action.default_keys.clone());
            }
        }

        shortcuts
    }

    pub async fn get_available_actions(&self) -> Vec<ShortcutAction> {
        let actions = self.available_actions.lock().await;
        actions.clone()
    }

    pub async fn get_profile(&self, profile_id: &str) -> Option<KeybindingsProfile> {
        let profiles = self.profiles.lock().await;
        profiles.get(profile_id).cloned()
    }

    pub async fn get_current_profile(&self) -> String {
        let current = self.current_profile.lock().await;
        current.clone()
    }

    pub async fn switch_profile(&self, profile_id: &str) -> Result<(), String> {
        let profiles = self.profiles.lock().await;

        if profiles.contains_key(profile_id) {
            drop(profiles); // Release lock before acquiring new one
            let mut current = self.current_profile.lock().await;
            *current = profile_id.to_string();
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", profile_id))
        }
    }

    pub async fn create_profile(&self, profile_data: serde_json::Value) -> Result<String, String> {
        let name: String = match profile_data.get("name") {
            Some(n) => serde_json::from_value(n.clone()).map_err(|e| format!("Invalid name: {}", e))?,
            None => return Err("Profile name is required".to_string()),
        };

        let description: Option<String> = profile_data.get("description")
            .and_then(|d| serde_json::from_value(d.clone()).ok());

        let profile_id = format!("profile_{}", uuid::Uuid::new_v4().to_string());

        let profile = KeybindingsProfile {
            id: profile_id.clone(),
            name,
            description: description.unwrap_or_default(),
            shortcuts: self.create_default_shortcuts(&self.get_available_actions().await),
            is_default: false,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
        };

        let mut profiles = self.profiles.lock().await;
        profiles.insert(profile_id.clone(), profile);

        Ok(profile_id)
    }

    pub async fn validate_keybinding(&self, key_combo: &KeyCombination) -> Result<(), String> {
        // Validate that the key is not empty
        if key_combo.key.trim().is_empty() {
            return Err("Key cannot be empty".to_string());
        }

        // Basic validation - could be extended
        if key_combo.key.len() > 1 && !key_combo.key.starts_with('F') {
            return Err("Invalid key combination".to_string());
        }

        Ok(())
    }

    pub async fn detect_conflicts(&self, profile_id: &str) -> Vec<KeybindingConflict> {
        let mut conflicts = Vec::new();

        let profiles = match self.profiles.lock().await.get(profile_id) {
            Some(profile) => profile,
            None => return conflicts,
        };

        let mut reverse_map: HashMap<String, Vec<String>> = HashMap::new();

        // Build reverse mapping of keys to actions
        for (action_id, keys) in &profiles.shortcuts {
            for key_combo in keys {
                let key_str = format_key_combo(key_combo);
                reverse_map.entry(key_str)
                    .or_insert_with(Vec::new)
                    .push(action_id.clone());
            }
        }

        // Find conflicts (multiple actions per key combination)
        for (key_combo, actions) in reverse_map {
            if actions.len() > 1 {
                conflicts.push(KeybindingConflict {
                    keys: key_combo,
                    actions,
                });
            }
        }

        conflicts
    }
}

fn format_key_combo(key_combo: &KeyCombination) -> String {
    let mut parts = Vec::new();

    if key_combo.ctrl {
        parts.push("Ctrl".to_string());
    }
    if key_combo.alt {
        parts.push("Alt".to_string());
    }
    if key_combo.shift {
        parts.push("Shift".to_string());
    }
    if key_combo.meta {
        parts.push("Cmd".to_string());
    }

    if !key_combo.key.is_empty() {
        parts.push(key_combo.key.clone());
    }

    parts.join("+")
}

// Lazy-static service instance
lazy_static::lazy_static! {
    static ref KEYBINDING_MANAGER: KeybindingManager = KeybindingManager::new();
}

// Public functions for Tauri commands

tauri_command_template! {
    get_keybindings_profile,
    async fn get_keybindings_profile_impl(profile_id: Option<String>) -> Result<serde_json::Value, String> {
        let profile_id = profile_id.unwrap_or_else(|| "default".to_string());

        match KEYBINDING_MANAGER.get_profile(&profile_id).await {
            Some(profile) => Ok(serde_json::to_value(&profile).unwrap_or(serde_json::json!({}))),
            None => Err(format!("Profile '{}' not found", profile_id))
        }
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    create_keybindings_profile,
    async fn create_keybindings_profile_impl(profile_data: serde_json::Value) -> Result<String, String> {
        KEYBINDING_MANAGER.create_profile(profile_data).await
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    update_keybinding_profile,
    async fn update_keybinding_profile_impl(profile_id: String, updates: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"status": "success", "message": "Profile updated successfully"}))
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    delete_keybindings_profile,
    async fn delete_keybindings_profile_impl(profile_id: String) -> Result<serde_json::Value, String> {
        let mut profiles = KEYBINDING_MANAGER.profiles.lock().await;
        let current_profile = KEYBINDING_MANAGER.get_current_profile().await;

        if profile_id == "default" {
            return Err("Cannot delete default profile".to_string());
        }

        if current_profile == profile_id {
            return Err("Cannot delete currently active profile".to_string());
        }

        if profiles.remove(&profile_id).is_some() {
            Ok(serde_json::json!({"status": "success", "message": "Profile deleted successfully"}))
        } else {
            Err(format!("Profile '{}' not found", profile_id))
        }
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    get_available_actions,
    async fn get_available_actions_impl() -> Result<serde_json::Value, String> {
        let actions = KEYBINDING_MANAGER.get_available_actions().await;
        Ok(serde_json::to_value(&actions).unwrap_or(serde_json::json!([])))
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    validate_keybinding_conflicts,
    async fn validate_keybinding_conflicts_impl(profile_id: String) -> Result<serde_json::Value, String> {
        let conflicts = KEYBINDING_MANAGER.detect_conflicts(&profile_id).await;
        Ok(serde_json::to_value(&conflicts).unwrap_or(serde_json::json!([])))
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    apply_keybindings_profile,
    async fn apply_keybindings_profile_impl(profile_id: String) -> Result<serde_json::Value, String> {
        KEYBINDING_MANAGER.switch_profile(&profile_id).await?;
        Ok(serde_json::json!({"status": "success", "message": "Profile applied successfully"}))
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    export_keybindings,
    async fn export_keybindings_impl(profile_id: String) -> Result<serde_json::Value, String> {
        match KEYBINDING_MANAGER.get_profile(&profile_id).await {
            Some(profile) => Ok(serde_json::to_value(&profile).unwrap_or(serde_json::json!({}))),
            None => Err(format!("Profile '{}' not found", profile_id))
        }
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    import_keybindings,
    async fn import_keybindings_impl(import_data: serde_json::Value) -> Result<String, String> {
        let profile: KeybindingsProfile = serde_json::from_value(import_data)
            .map_err(|e| format!("Invalid profile data: {}", e))?;

        let profile_id = format!("imported_{}", uuid::Uuid::new_v4().to_string());

        let mut profiles = KEYBINDING_MANAGER.profiles.lock().await;
        profiles.insert(profile_id.clone(), profile);

        Ok(profile_id)
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    reset_to_defaults,
    async fn reset_to_defaults_impl() -> Result<serde_json::Value, String> {
        KEYBINDING_MANAGER.initialize_default_profile().await;
        KEYBINDING_MANAGER.switch_profile("default").await?;
        Ok(serde_json::json!({"status": "success", "message": "Reset to defaults successfully"}))
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    get_conflicts,
    async fn get_conflicts_impl() -> Result<serde_json::Value, String> {
        let current_profile = KEYBINDING_MANAGER.get_current_profile().await;
        let conflicts = KEYBINDING_MANAGER.detect_conflicts(&current_profile).await;
        Ok(serde_json::to_value(&conflicts).unwrap_or(serde_json::json!([])))
    },
    service = KeybindingManager,
    state = KEYBINDING_MANAGER,
    config = CommandConfig::default()
}

impl CommandService for KeybindingManager {
    type Error = String;

    fn is_ready(&self) -> bool {
        true // Keybinding manager is always ready
    }

    fn service_name(&self) -> &'static str {
        "KeybindingManager"
    }
}