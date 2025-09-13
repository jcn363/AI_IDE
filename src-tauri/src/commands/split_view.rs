//! Split View and Tab Management commands for IDE layout control
//!
//! This module provides comprehensive layout management for split views,
//! tabbed interfaces, and panel organization in the IDE.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use lazy_static::lazy_static;
use rust_ai_ide_core::security::{audit_action, audit_logger};
use rust_ai_ide_core::validation::validate_secure_path;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::command_templates::*;

/// Panel orientation enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

/// Split configuration for a panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitConfig {
    pub id:          String,
    pub orientation: SplitOrientation,
    pub size:        f32, // Ratio 0.0 to 1.0
    pub children:    Vec<PanelNode>,
}

/// Panel node representing the layout hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelNode {
    Split(SplitConfig),
    Leaf(PanelInfo),
}

/// Panel information for leaf nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelInfo {
    pub id:           String,
    pub content_type: PanelContentType,
    pub title:        Option<String>,
    pub is_active:    bool,
    pub tabs:         Vec<PanelTab>,
}

/// Content types for panels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PanelContentType {
    Editor,
    Terminal,
    FileExplorer,
    Output,
    Debug,
    Git,
    Cargo,
    Documentation,
}

/// Tab information for panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelTab {
    pub id:          String,
    pub title:       String,
    pub content_id:  String,
    pub is_modified: bool,
    pub is_pinned:   bool,
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub root_panel:     PanelNode,
    pub focused_panel:  String,
    pub layout_version: u32,
    pub last_updated:   u64,
}

/// Split view manager service
#[derive(Debug)]
pub struct SplitViewManager {
    current_layout: Arc<Mutex<LayoutConfig>>,
    saved_layouts:  Arc<Mutex<HashMap<String, LayoutConfig>>>,
}

impl SplitViewManager {
    pub fn new() -> Self {
        Self {
            current_layout: Arc::new(Mutex::new(LayoutConfig {
                root_panel:     PanelNode::Leaf(PanelInfo {
                    id:           "main".to_string(),
                    content_type: PanelContentType::Editor,
                    title:        Some("Main Editor".to_string()),
                    is_active:    true,
                    tabs:         Vec::new(),
                }),
                focused_panel:  "main".to_string(),
                layout_version: 1,
                last_updated:   std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            })),
            saved_layouts:  Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn split_panel(
        &self,
        panel_id: &str,
        orientation: SplitOrientation,
        size_ratio: f32,
    ) -> Result<(), String> {
        let mut layout = self.current_layout.lock().await;

        match self.find_and_split_panel(&mut layout.root_panel, panel_id, orientation, size_ratio) {
            Ok(_) => {
                layout.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                layout.layout_version += 1;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn find_and_split_panel(
        &self,
        node: &mut PanelNode,
        target_id: &str,
        orientation: SplitOrientation,
        size_ratio: f32,
    ) -> Result<(), String> {
        match node {
            PanelNode::Leaf(panel_info) =>
                if panel_info.id == target_id {
                    let new_panel_id = format!("{}_right", target_id);
                    let original_panel = panel_info.clone();
                    original_panel.id = format!("{}_left", target_id);

                    *node = PanelNode::Split(SplitConfig {
                        id:          format!("split_{}", target_id),
                        orientation: orientation.clone(),
                        size:        size_ratio,
                        children:    vec![
                            PanelNode::Leaf(original_panel),
                            PanelNode::Leaf(PanelInfo {
                                id:           new_panel_id,
                                content_type: PanelContentType::Editor,
                                title:        Some("New Panel".to_string()),
                                is_active:    false,
                                tabs:         Vec::new(),
                            }),
                        ],
                    });
                    Ok(())
                } else {
                    Err(format!("Panel '{}' not found", target_id))
                },
            PanelNode::Split(split_config) => {
                for child in &mut split_config.children {
                    if let Ok(_) = self.find_and_split_panel(child, target_id, orientation, size_ratio) {
                        return Ok(());
                    }
                }
                Err(format!("Panel '{}' not found", target_id))
            }
        }
    }

    pub async fn close_panel(&self, panel_id: &str) -> Result<(), String> {
        let mut layout = self.current_layout.lock().await;

        // Don't allow closing the last remaining panel
        let panel_count = self.count_panels(&layout.root_panel);
        if panel_count <= 1 {
            return Err("Cannot close the last remaining panel".to_string());
        }

        match self.find_and_remove_panel(&mut layout.root_panel, panel_id) {
            Ok(_) => {
                layout.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                layout.layout_version += 1;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn find_and_remove_panel(&self, node: &mut PanelNode, target_id: &str) -> Result<(), String> {
        match node {
            PanelNode::Split(split_config) => {
                let mut remove_index = None;

                for (i, child) in split_config.children.iter_mut().enumerate() {
                    if let PanelNode::Leaf(ref panel_info) = child {
                        if panel_info.id == target_id {
                            remove_index = Some(i);
                            break;
                        }
                    }
                }

                if let Some(index) = remove_index {
                    if split_config.children.len() > 2 {
                        // Remove just this child if more than 2 children
                        split_config.children.remove(index);
                    } else if split_config.children.len() == 2 {
                        // If only 2 children, remove the split entirely and promote the other child
                        let other_child = if index == 0 {
                            split_config.children.remove(1)
                        } else {
                            split_config.children.remove(0)
                        };
                        *node = other_child;
                    }
                    Ok(())
                } else {
                    // Recursively search in children
                    for child in &mut split_config.children {
                        if let PanelNode::Split(_) = child {
                            if let Ok(_) = self.find_and_remove_panel(child, target_id) {
                                return Ok(());
                            }
                        }
                    }
                    Err(format!("Panel '{}' not found", target_id))
                }
            }
            PanelNode::Leaf(panel_info) =>
                if panel_info.id == target_id {
                    Err("Cannot remove root level leaf panel".to_string())
                } else {
                    Err(format!("Panel '{}' not found", target_id))
                },
        }
    }

    fn count_panels(&self, node: &PanelNode) -> usize {
        match node {
            PanelNode::Leaf(_) => 1,
            PanelNode::Split(split_config) => split_config
                .children
                .iter()
                .map(|child| self.count_panels(child))
                .sum(),
        }
    }

    pub async fn add_tab(&self, panel_id: &str, tab: PanelTab) -> Result<(), String> {
        let mut layout = self.current_layout.lock().await;

        match self.find_panel_mut(&mut layout.root_panel, panel_id) {
            Some(panel_info) => {
                panel_info.tabs.push(tab);
                layout.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                layout.layout_version += 1;
                Ok(())
            }
            None => Err(format!("Panel '{}' not found", panel_id)),
        }
    }

    pub async fn remove_tab(&self, panel_id: &str, tab_id: &str) -> Result<(), String> {
        let mut layout = self.current_layout.lock().await;

        match self.find_panel_mut(&mut layout.root_panel, panel_id) {
            Some(panel_info) => {
                panel_info.tabs.retain(|tab| tab.id != tab_id);
                layout.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                layout.layout_version += 1;
                Ok(())
            }
            None => Err(format!("Panel '{}' not found", panel_id)),
        }
    }

    pub async fn set_focused_panel(&self, panel_id: &str) -> Result<(), String> {
        let mut layout = self.current_layout.lock().await;

        // Check if panel exists
        if self.find_panel(&layout.root_panel, panel_id).is_some() {
            layout.focused_panel = panel_id.to_string();
            layout.last_updated = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(())
        } else {
            Err(format!("Panel '{}' not found", panel_id))
        }
    }

    pub async fn get_layout(&self) -> LayoutConfig {
        self.current_layout.lock().await.clone()
    }

    pub async fn save_layout(&self, name: &str) -> Result<(), String> {
        let current = self.current_layout.lock().await.clone();
        let mut saved = self.saved_layouts.lock().await;
        saved.insert(name.to_string(), current);
        Ok(())
    }

    pub async fn load_layout(&self, name: &str) -> Result<(), String> {
        let saved = self.saved_layouts.lock().await;
        match saved.get(name) {
            Some(layout) => {
                let mut current = self.current_layout.lock().await;
                *current = layout.clone();
                current.layout_version += 1;
                current.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                Ok(())
            }
            None => Err(format!("Layout '{}' not found", name)),
        }
    }

    fn find_panel(&self, node: &PanelNode, panel_id: &str) -> Option<&PanelInfo> {
        match node {
            PanelNode::Leaf(panel_info) =>
                if panel_info.id == panel_id {
                    Some(panel_info)
                } else {
                    None
                },
            PanelNode::Split(split_config) => {
                for child in &split_config.children {
                    if let Some(panel) = self.find_panel(child, panel_id) {
                        return Some(panel);
                    }
                }
                None
            }
        }
    }

    fn find_panel_mut(&self, node: &mut PanelNode, panel_id: &str) -> Option<&mut PanelInfo> {
        match node {
            PanelNode::Leaf(panel_info) =>
                if panel_info.id == panel_id {
                    Some(panel_info)
                } else {
                    None
                },
            PanelNode::Split(split_config) => {
                for child in &mut split_config.children {
                    if let Some(panel) = self.find_panel_mut(child, panel_id) {
                        return Some(panel);
                    }
                }
                None
            }
        }
    }
}

// Lazy-static service instance
lazy_static! {
    static ref SPLIT_VIEW_MANAGER: SplitViewManager = SplitViewManager::new();
}

// Tauri commands

tauri_command_template! {
    split_panel,
    async fn split_panel_impl(panel_id: String, orientation: SplitOrientation, size_ratio: f32) -> Result<serde_json::Value, String> {
        SPLIT_VIEW_MANAGER
            .split_panel(&panel_id, orientation, size_ratio)
            .await?;

        Ok(serde_json::json!({"status": "success", "message": "Panel split successfully"}))
    },
    service = SplitViewManager,
    state = SPLIT_VIEW_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    close_panel,
    async fn close_panel_impl(panel_id: String) -> Result<serde_json::Value, String> {
        SPLIT_VIEW_MANAGER
            .close_panel(&panel_id)
            .await?;

        Ok(serde_json::json!({"status": "success", "message": "Panel closed successfully"}))
    },
    service = SplitViewManager,
    state = SPLIT_VIEW_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    add_tab_to_panel,
    async fn add_tab_to_panel_impl(panel_id: String, tab: PanelTab) -> Result<serde_json::Value, String> {
        SPLIT_VIEW_MANAGER
            .add_tab(&panel_id, tab)
            .await?;

        Ok(serde_json::json!({"status": "success", "message": "Tab added successfully"}))
    },
    service = SplitViewManager,
    state = SPLIT_VIEW_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    remove_tab_from_panel,
    async fn remove_tab_from_panel_impl(panel_id: String, tab_id: String) -> Result<serde_json::Value, String> {
        SPLIT_VIEW_MANAGER
            .remove_tab(&panel_id, &tab_id)
            .await?;

        Ok(serde_json::json!({"status": "success", "message": "Tab removed successfully"}))
    },
    service = SplitViewManager,
    state = SPLIT_VIEW_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    set_focused_panel,
    async fn set_focused_panel_impl(panel_id: String) -> Result<serde_json::Value, String> {
        SPLIT_VIEW_MANAGER.set_focused_panel(&panel_id).await?;

        Ok(serde_json::json!({"status": "success", "message": "Panel focused successfully"}))
    },
    service = SplitViewManager,
    state = SPLIT_VIEW_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    get_layout,
    async fn get_layout_impl() -> Result<serde_json::Value, String> {
        let layout = SPLIT_VIEW_MANAGER.get_layout().await;
        Ok(serde_json::json!({
            "status": "success",
            "layout": layout
        }))
    },
    service = SplitViewManager,
    state = SPLIT_VIEW_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    save_layout,
    async fn save_layout_impl(name: String) -> Result<serde_json::Value, String> {
        SPLIT_VIEW_MANAGER.save_layout(&name).await?;

        Ok(serde_json::json!({"status": "success", "message": "Layout saved successfully"}))
    },
    service = SplitViewManager,
    state = SPLIT_VIEW_MANAGER,
    config = CommandConfig::default()
}

tauri_command_template! {
    load_layout,
    async fn load_layout_impl(name: String) -> Result<serde_json::Value, String> {
        SPLIT_VIEW_MANAGER.load_layout(&name).await?;

        Ok(serde_json::json!({"status": "success", "message": "Layout loaded successfully"}))
    },
    service = SplitViewManager,
    state = SPLIT_VIEW_MANAGER,
    config = CommandConfig::default()
}

impl CommandService for SplitViewManager {
    type Error = String;

    fn is_ready(&self) -> bool {
        true // Split view manager is always ready
    }

    fn service_name(&self) -> &'static str {
        "SplitViewManager"
    }
}
