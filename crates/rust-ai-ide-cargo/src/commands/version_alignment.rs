//! Commands for workspace version alignment

use crate::dependency::{DependencyManager, VersionAlignment};
use std::sync::Arc;
use tauri::{Manager, State};

/// State for version alignment commands
pub struct VersionAlignmentState {
    manager: Arc<DependencyManager>,
}

impl VersionAlignmentState {
    /// Creates a new VersionAlignmentState with a shared DependencyManager
    pub fn new(manager: Arc<DependencyManager>) -> Self {
        Self { manager }
    }
}

/// Analyzes workspace dependencies and suggests version alignments
#[tauri::command(rename = "version_alignment_analyze")]
pub async fn analyze_version_alignment(
    state: State<'_, VersionAlignmentState>,
) -> Result<Vec<VersionAlignment>, String> {
    state
        .manager
        .analyze_version_alignment()
        .await
        .map_err(|e| e.to_string())
}

/// Applies version alignments to the workspace
#[tauri::command(rename = "version_alignment_apply")]
pub async fn apply_version_alignment(
    alignments: Vec<VersionAlignment>,
    state: State<'_, VersionAlignmentState>,
) -> Result<(), String> {
    state
        .manager
        .apply_version_alignment(&alignments)
        .await
        .map_err(|e| e.to_string())
}

/// Initializes the version alignment system
pub fn init_version_alignment<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    manager: Arc<DependencyManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = VersionAlignmentState::new(manager);
    app.manage(state);
    Ok(())
}
