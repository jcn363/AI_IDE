use std::path::PathBuf;

use rust_ai_ide_common::read_file_to_string;
use serde::Serialize;
use toml::Table;

#[derive(Debug, Serialize)]
pub struct LockDependency {
    pub name:         String,
    pub version:      String,
    pub dependencies: Vec<String>,
    pub is_direct:    bool,
}

#[tauri::command]
pub async fn parse_cargo_lock(project_path: PathBuf) -> Result<Vec<LockDependency>, String> {
    let lock_path = project_path.join("Cargo.lock");
    if !lock_path.exists() {
        return Err("Cargo.lock not found".to_string());
    }

    let lock_content = read_file_to_string(&lock_path)
        .await
        .map_err(|e| format!("Failed to read Cargo.lock: {}", e))?;

    let lock_data: Table = toml::from_str(&lock_content).map_err(|e| format!("Failed to parse Cargo.lock: {}", e))?;

    let mut dependencies = Vec::new();

    // Get direct dependencies from Cargo.toml for reference
    let direct_deps = get_direct_dependencies(&project_path)
        .await
        .unwrap_or_default();

    if let Some(packages) = lock_data.get("package").and_then(|v| v.as_array()) {
        for pkg in packages {
            if let (Some(name), Some(version)) = (pkg.get("name"), pkg.get("version")) {
                let name_str = name.as_str().unwrap_or("").to_string();
                let version_str = version.as_str().unwrap_or("").to_string();

                let deps = pkg
                    .get("dependencies")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|d| d.as_str())
                            .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();

                dependencies.push(LockDependency {
                    name:         name_str.clone(),
                    version:      version_str,
                    dependencies: deps,
                    is_direct:    direct_deps.contains(&name_str),
                });
            }
        }
    }

    Ok(dependencies)
}

async fn get_direct_dependencies(project_path: &PathBuf) -> Result<Vec<String>, String> {
    let toml_path = project_path.join("Cargo.toml");
    if !toml_path.exists() {
        return Ok(Vec::new());
    }

    let toml_content = read_file_to_string(&toml_path)
        .await
        .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

    let cargo_toml: Table = toml::from_str(&toml_content).map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;

    let mut deps = Vec::new();

    // Check [dependencies] section
    if let Some(dependencies) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
        deps.extend(dependencies.keys().cloned());
    }

    // Check [dev-dependencies] section
    if let Some(dev_deps) = cargo_toml
        .get("dev-dependencies")
        .and_then(|d| d.as_table())
    {
        deps.extend(dev_deps.keys().cloned());
    }

    // Check [build-dependencies] section
    if let Some(build_deps) = cargo_toml
        .get("build-dependencies")
        .and_then(|d| d.as_table())
    {
        deps.extend(build_deps.keys().cloned());
    }

    // Check workspace dependencies
    if let Some(workspace) = cargo_toml.get("workspace").and_then(|w| w.as_table()) {
        if let Some(workspace_deps) = workspace.get("dependencies").and_then(|d| d.as_table()) {
            deps.extend(workspace_deps.keys().cloned());
        }
    }

    Ok(deps)
}
