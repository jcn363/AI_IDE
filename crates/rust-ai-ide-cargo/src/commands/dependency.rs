use crate::dependency::{DependencyInfo, DependencyManager, VersionAlignment};
///! Tauri commands for dependency management
// We use our own DependencyKind from the dependency module
use serde::Deserialize;
use std::sync::Arc;
use tauri::{Runtime, State};

// Testing utilities will be added when needed

/// State for managing dependencies
pub struct DependencyState {
    manager: Arc<DependencyManager>,
}

impl DependencyState {
    /// Creates a new DependencyState with a shared DependencyManager
    pub fn new(manager: Arc<DependencyManager>) -> Self {
        Self { manager }
    }
}

/// Parameters for adding a dependency
#[derive(Debug, Deserialize)]
pub struct AddDependencyParams {
    pub name: String,
    pub version: String,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub default_features: Option<bool>,
    pub kind: Option<String>,
}

/// Parameters for updating a dependency
#[derive(Debug, Deserialize)]
pub struct UpdateDependencyParams {
    pub name: String,
    pub version: String,
}

/// Loads all dependencies for the project
#[tauri::command]
pub async fn load_dependencies(
    state: State<'_, DependencyState>,
) -> Result<Vec<DependencyInfo>, String> {
    state
        .manager
        .load_dependencies()
        .await
        .map_err(|e| e.to_string())?;
    state
        .manager
        .get_dependencies()
        .await
        .into_iter()
        .map(|d| Ok(d))
        .collect()
}

/// Gets a specific dependency by name
#[tauri::command]
pub async fn get_dependency(
    name: String,
    state: State<'_, DependencyState>,
) -> Result<Option<DependencyInfo>, String> {
    Ok(state.manager.get_dependency(&name).await)
}

/// Adds a new dependency
#[tauri::command]
pub async fn add_dependency(
    params: AddDependencyParams,
    state: State<'_, DependencyState>,
) -> Result<(), String> {
    let kind = match params.kind.as_deref() {
        Some("dev") => crate::dependency::DependencyKind::Development,
        Some("build") => crate::dependency::DependencyKind::Build,
        _ => crate::dependency::DependencyKind::Normal,
    };

    let dep = DependencyInfo {
        name: params.name,
        version: params.version,
        features: params.features.unwrap_or_default(),
        optional: params.optional.unwrap_or(false),
        default_features: params.default_features.unwrap_or(true),
        target: None,
        kind,
        registry: None,
        source: None,
    };

    state
        .manager
        .add_dependency(dep)
        .await
        .map_err(|e| e.to_string())
}

/// Updates an existing dependency
#[tauri::command]
pub async fn update_dependency(
    params: UpdateDependencyParams,
    state: State<'_, DependencyState>,
) -> Result<(), String> {
    state
        .manager
        .update_dependency(&params.name, &params.version)
        .await
        .map_err(|e| e.to_string())
}

/// Removes a dependency
#[tauri::command]
pub async fn remove_dependency(
    name: String,
    state: State<'_, DependencyState>,
) -> Result<(), String> {
    state
        .manager
        .remove_dependency(&name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Analyzes workspace dependencies and suggests version alignments (general)
#[tauri::command(rename = "dependency_analyze_version_alignment")]
pub async fn dependency_analyze_version_alignment(
    state: State<'_, DependencyState>,
) -> Result<Vec<VersionAlignment>, String> {
    state
        .manager
        .analyze_version_alignment()
        .await
        .map_err(|e| e.to_string())
}

/// Applies version alignments to the workspace (general)
#[tauri::command(rename = "dependency_apply_version_alignment")]
pub async fn dependency_apply_version_alignment(
    alignments: Vec<VersionAlignment>,
    state: State<'_, DependencyState>,
) -> Result<(), String> {
    state
        .manager
        .apply_version_alignment(&alignments)
        .await
        .map_err(|e| e.to_string())
}

/// Initializes the dependency management system with a shared DependencyManager
pub fn init_dependency_management<R: Runtime>(
    app: &tauri::AppHandle<R>,
    manager: Arc<DependencyManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = DependencyState::new(manager);
    <tauri::AppHandle<R> as tauri::Manager<R>>::manage(app, state);
    Ok(())
}
