use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTask {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub run_in_parallel: bool,
    pub continue_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildHooks {
    pub pre_build: Vec<BuildTask>,
    pub post_build: Vec<BuildTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub hooks: BuildHooks,
    pub env: HashMap<String, String>,
    pub target: Option<String>,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            hooks: BuildHooks {
                pre_build: Vec::new(),
                post_build: Vec::new(),
            },
            env: HashMap::new(),
            target: None,
            features: Vec::new(),
            all_features: false,
            no_default_features: false,
        }
    }
}

impl BuildTask {
    pub fn execute(&self, project_path: &Path) -> Result<()> {
        let mut command = Command::new(&self.command);

        // Set working directory
        let working_dir = self
            .working_dir
            .as_ref()
            .map(|p| project_path.join(p))
            .unwrap_or_else(|| project_path.to_path_buf());

        command.current_dir(working_dir);

        // Set environment variables
        command.envs(&self.env);

        // Add command arguments
        command.args(&self.args);

        // Execute the command
        let status = command
            .status()
            .with_context(|| format!("Failed to execute build task: {}", self.name))?;

        if !status.success() && !self.continue_on_error {
            anyhow::bail!("Build task '{}' failed with status: {}", self.name, status);
        }

        Ok(())
    }
}

impl BuildHooks {
    pub async fn execute_pre_build(&self, project_path: &Path) -> Result<()> {
        self.execute_tasks(&self.pre_build, project_path).await
    }

    pub async fn execute_post_build(&self, project_path: &Path) -> Result<()> {
        self.execute_tasks(&self.post_build, project_path).await
    }

    async fn execute_tasks(&self, tasks: &[BuildTask], project_path: &Path) -> Result<()> {
        use tokio::task;

        let mut handles = Vec::new();

        for task in tasks {
            let task = task.clone();
            let project_path = project_path.to_path_buf();

            if task.run_in_parallel {
                let handle = task::spawn_blocking(move || task.execute(&project_path));
                handles.push(handle);
            } else {
                // Wait for all previous parallel tasks to complete
                for handle in handles.drain(..) {
                    handle.await??;
                }
                // Execute current task
                task.execute(&project_path)?;
            }
        }

        // Wait for any remaining parallel tasks
        for handle in handles {
            handle.await??;
        }

        Ok(())
    }
}
