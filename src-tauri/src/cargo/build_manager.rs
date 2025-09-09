use super::build_task::{BuildConfig, BuildHooks};
use rust_ai_ide_core::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output: String,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct BuildManager {
    config: BuildConfig,
    config_path: Option<PathBuf>,
}

impl BuildManager {
    pub fn new() -> Self {
        Self {
            config: BuildConfig::default(),
            config_path: None,
        }
    }

    pub async fn load_config(&mut self, project_path: &Path) -> Result<()> {
        let config_path = project_path.join(".cargo/config.toml");
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            self.config = toml::from_str(&content)?;
            self.config_path = Some(config_path);
        }
        Ok(())
    }

    pub async fn save_config(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            let content = toml::to_string_pretty(&self.config)?;
            tokio::fs::write(path, content).await?;
        }
        Ok(())
    }

    pub async fn execute_build(
        &self,
        project_path: &Path,
        profile: &str,
        tx: mpsc::Sender<String>,
    ) -> Result<BuildResult> {
        let start = std::time::Instant::now();
        
        // Execute pre-build hooks
        if let Err(e) = self.config.hooks.execute_pre_build(project_path).await {
            tx.send(format!("Pre-build hook failed: {}\n", e)).await?;
            return Ok(BuildResult {
                success: false,
                output: format!("Pre-build hook failed: {}", e),
                duration: start.elapsed(),
            });
        }

        // Execute the actual build
        let build_result = self.execute_cargo_build(project_path, profile, tx.clone()).await;

        // Execute post-build hooks regardless of build success
        if let Err(e) = self.config.hooks.execute_post_build(project_path).await {
            tx.send(format!("Post-build hook failed: {}\n", e)).await?;
            return Ok(BuildResult {
                success: false,
                output: format!("Build completed but post-build hook failed: {}", e),
                duration: start.elapsed(),
            });
        }

        build_result
    }

    async fn execute_cargo_build(
        &self,
        project_path: &Path,
        profile: &str,
        tx: mpsc::Sender<String>,
    ) -> Result<BuildResult> {
        let mut command = Command::new("cargo");
        command.current_dir(project_path);
        
        // Set environment variables
        command.envs(&self.config.env);
        
        // Build command
        command.arg("build");
        
        // Set profile
        if profile != "debug" {
            command.arg("--profile").arg(profile);
        }
        
        // Set target if specified
        if let Some(target) = &self.config.target {
            command.arg("--target").arg(target);
        }
        
        // Handle features
        if self.config.all_features {
            command.arg("--all-features");
        } else if self.config.no_default_features {
            command.arg("--no-default-features");
        } else if !self.config.features.is_empty() {
            command.arg("--features").arg(self.config.features.join(","));
        }
        
        // Execute the command
        let output = command
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute cargo build: {}", e))?;
        
        // Send output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let error_str = String::from_utf8_lossy(&output.stderr);
        
        tx.send(output_str.to_string()).await?;
        if !error_str.is_empty() {
            tx.send(error_str.to_string()).await?;
        }
        
        Ok(BuildResult {
            success: output.status.success(),
            output: format!("{}\n{}", output_str, error_str),
            duration: start.elapsed(),
        })
    }
    
    pub fn get_config(&self) -> &BuildConfig {
        &self.config
    }
    
    pub fn get_config_mut(&mut self) -> &mut BuildConfig {
        &mut self.config
    }
}
