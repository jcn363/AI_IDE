//! Commands for build system functionality
//!
//! This module provides command handlers for the build system.

use crate::build::{BuildProfile, BuildStatus, BuildSystem};
use anyhow::Result;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Start a new build with the specified options
#[cfg(feature = "tauri")]
#[cfg_attr(feature = "tauri", tauri::command)]
pub async fn start_build(
    #[cfg(feature = "tauri")] app_handle: AppHandle,
    #[cfg(not(feature = "tauri"))] _app_handle: (),
    project_path: PathBuf,
    profile: String,
    features: Option<Vec<String>>,
    target: Option<String>,
) -> Result<String, String> {
    #[cfg(feature = "tauri")]
    let build_system = app_handle.state::<BuildSystem>();

    #[cfg(not(feature = "tauri"))]
    {
        // In non-Tauri context, we need to create a dummy AppHandle
        // This is a temporary solution for testing purposes
        use tauri::AppHandle;
        let app_handle = AppHandle::dummy();
        BuildSystem::new(app_handle)
    }

    let features_ref: Option<Vec<&str>> = features
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    build_system
        .start_build(
            &project_path,
            &profile,
            features_ref,
            target.as_deref(),
            None,
        )
        .await
        .map_err(|e| e.to_string())
}

/// Cancel the currently running build
#[cfg_attr(feature = "tauri", tauri::command)]
pub async fn cancel_build(
    #[cfg(feature = "tauri")] app_handle: AppHandle,
    #[cfg(not(feature = "tauri"))] _app_handle: (),
) -> Result<(), String> {
    #[cfg(feature = "tauri")]
    let build_system = app_handle.state::<BuildSystem>();

    #[cfg(not(feature = "tauri"))]
    {
        // In non-Tauri context, we need to create a dummy AppHandle
        // This is a temporary solution for testing purposes
        use tauri::AppHandle;
        let app_handle = AppHandle::dummy();
        BuildSystem::new(app_handle)
    }

    build_system.cancel_build().await.map_err(|e| e.to_string())
}

/// Get the current build status
#[cfg_attr(feature = "tauri", tauri::command)]
pub async fn get_build_status(
    #[cfg(feature = "tauri")] app_handle: AppHandle,
    #[cfg(not(feature = "tauri"))] _app_handle: (),
) -> Result<BuildStatus, String> {
    #[cfg(feature = "tauri")]
    let build_system = app_handle.state::<BuildSystem>();

    #[cfg(not(feature = "tauri"))]
    {
        // In non-Tauri context, we need to create a dummy AppHandle
        // This is a temporary solution for testing purposes
        use tauri::AppHandle;
        let app_handle = AppHandle::dummy();
        BuildSystem::new(app_handle)
    }

    Ok(build_system.get_build_status().await)
}

/// Get available build profiles
#[cfg_attr(feature = "tauri", tauri::command)]
pub async fn get_build_profiles(
    #[cfg(feature = "tauri")] _app_handle: AppHandle,
    #[cfg(not(feature = "tauri"))] _app_handle: (),
) -> Result<Vec<BuildProfile>, String> {
    // In a real implementation, this would read from Cargo.toml
    // For now, return default profiles
    Ok(vec![
        BuildProfile::default(),
        BuildProfile::release(),
        BuildProfile::bench(),
    ])
}

/// Initialize the build system for the Tauri application
#[cfg(feature = "tauri")]
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let build_system = BuildSystem::new(app.handle().clone());
    app.manage(build_system);

    Ok(())
}

/// Initialize build commands for Tauri
pub fn init_build_commands<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    // The commands are already registered via the #[tauri::command] attribute
    // This function is kept for future command initialization if needed
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        assert!(true);
    }
}
