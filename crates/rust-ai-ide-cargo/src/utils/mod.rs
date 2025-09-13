//! Utility functions for Cargo integration

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use toml;

use crate::models::CargoDepVersion;

/// Parse dependencies from TOML value
pub fn parse_dependencies(toml_value: &toml::Value, section: &str) -> Result<HashMap<String, CargoDepVersion>> {
    let mut deps = HashMap::new();

    if let Some(deps_table) = toml_value.get(section) {
        if let Some(table) = deps_table.as_table() {
            for (name, value) in table {
                let dep = if value.is_str() {
                    CargoDepVersion::Simple(value.as_str().unwrap().to_string())
                } else if let Some(table) = value.as_table() {
                    let mut dep = CargoDepVersion::Detailed {
                        version:          table
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        path:             table.get("path").and_then(|v| v.as_str()).map(String::from),
                        git:              table.get("git").and_then(|v| v.as_str()).map(String::from),
                        branch:           table
                            .get("branch")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        tag:              table.get("tag").and_then(|v| v.as_str()).map(String::from),
                        features:         table.get("features").and_then(|v| v.as_array()).map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        }),
                        default_features: table.get("default-features").and_then(|v| v.as_bool()),
                    };

                    // If it's a simple version string, convert to Simple variant
                    if let CargoDepVersion::Detailed {
                        version: Some(version),
                        path: None,
                        git: None,
                        branch: None,
                        tag: None,
                        features: None,
                        default_features: None,
                    } = &dep
                    {
                        dep = CargoDepVersion::Simple(version.clone());
                    }

                    dep
                } else {
                    anyhow::bail!("Invalid dependency format for {}", name);
                };

                deps.insert(name.clone(), dep);
            }
        }
    }

    Ok(deps)
}

/// Find all Rust source files in a directory
pub fn find_rust_files(path: &Path) -> impl Iterator<Item = std::path::PathBuf> + 'static {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .map(|e| e.into_path())
}
