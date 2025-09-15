//! Code refactoring functionality

use anyhow::Result;

use crate::utils;
use crate::workspace::CargoManager;

/// Perform a workspace-wide search and replace
pub async fn workspace_replace(
    manager: &CargoManager,
    search: &str,
    replace: &str,
    dry_run: bool,
) -> Result<usize> {
    let mut count = 0;
    let mut files_modified = 0;

    for member in manager.get_workspace_members() {
        for path in utils::find_rust_files(member) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.contains(search) {
                    let new_content = content.replace(search, replace);
                    if !dry_run {
                        std::fs::write(&path, new_content)?;
                    }
                    count += content.matches(search).count();
                    files_modified += 1;
                }
            }
        }
    }

    if dry_run {
        log::info!(
            "Would replace {} occurrences of '{}' in {} files",
            count,
            search,
            files_modified
        );
    } else {
        log::info!(
            "Replaced {} occurrences of '{}' in {} files",
            count,
            search,
            files_modified
        );
    }

    Ok(count)
}

/// Rename a symbol across the entire workspace
pub async fn rename_symbol(
    manager: &CargoManager,
    old_name: &str,
    new_name: &str,
    dry_run: bool,
) -> Result<usize> {
    // First find all references to ensure we can safely rename
    let references = find_references(manager, old_name).await?;

    if references.is_empty() {
        return Ok(0);
    }

    // Perform the actual replacement
    workspace_replace(manager, old_name, new_name, dry_run).await
}

/// Find all references to a symbol across the workspace
pub async fn find_references(
    manager: &CargoManager,
    symbol: &str,
) -> Result<Vec<(std::path::PathBuf, Vec<usize>)>> {
    use std::fs;
    use std::path::PathBuf;

    use url::Url;

    let mut results = Vec::new();

    for member in manager.get_workspace_members() {
        // Parse the URL
        let member_str = member.to_string_lossy();
        let path = if let Ok(url) = Url::parse(&member_str) {
            // If it's a valid URL, convert it to a file path
            url.to_file_path().unwrap_or_else(|_| member.clone())
        } else if member_str.starts_with("path+file://") {
            // Handle the custom path+file:// format
            let path_part = member_str.trim_start_matches("path+file://");
            let path_part = path_part.split('#').next().unwrap_or(path_part);
            PathBuf::from(path_part)
        } else {
            // Fall back to the original path
            member.clone()
        };

        println!("  Resolved path: {:?} (exists: {})", path, path.exists());

        // Check if the path exists and is a directory
        if !path.exists() {
            println!("  Path does not exist");
            continue;
        }

        // Debug: List all files in the member directory
        if let Ok(entries) = fs::read_dir(&path) {
            println!("  Directory contents of {:?}:", path);
            for entry in entries.filter_map(Result::ok) {
                let entry_path = entry.path();
                let file_type = if entry_path.is_dir() { "dir" } else { "file" };
                println!(
                    "    {}: {:?} (exists: {})",
                    file_type,
                    entry_path,
                    entry_path.exists()
                );

                // If it's a directory, list its contents
                if entry_path.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&entry_path) {
                        for sub_entry in sub_entries.filter_map(Result::ok) {
                            let sub_path = sub_entry.path();
                            let sub_type = if sub_path.is_dir() { "dir" } else { "file" };
                            println!(
                                "      {}: {:?} (exists: {})",
                                sub_type,
                                sub_path,
                                sub_path.exists()
                            );
                        }
                    }
                }
            }
        } else {
            println!("  Could not read directory: {:?}", path);
        }

        // Check if src directory exists and list its contents
        let src_dir = path.join("src");
        println!(
            "  Checking for src directory at: {:?} (exists: {})",
            src_dir,
            src_dir.exists()
        );

        if src_dir.exists() {
            if let Ok(entries) = fs::read_dir(&src_dir) {
                println!("  Source directory contents:");
                for entry in entries.filter_map(Result::ok) {
                    let entry_path = entry.path();
                    let file_type = if entry_path.is_dir() { "dir" } else { "file" };
                    println!(
                        "    {}: {:?} (exists: {})",
                        file_type,
                        entry_path,
                        entry_path.exists()
                    );

                    // If it's a file, try to read it
                    if entry_path.is_file() {
                        println!(
                            "      File content: {:?}",
                            fs::read_to_string(&entry_path).unwrap_or_default()
                        );
                    }
                }
            }
        }

        // Find and search Rust files
        for entry in walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "rs")
            })
        {
            let file_path = entry.path();
            println!("  Checking file: {:?}", file_path);

            if let Ok(content) = fs::read_to_string(file_path) {
                println!("    File content: {:?}", content);

                let line_numbers: Vec<usize> = content
                    .lines()
                    .enumerate()
                    .filter(|(i, line)| {
                        let found = line.contains(symbol);
                        if found {
                            println!("      Found '{}' at line {}: {}", symbol, i + 1, line);
                        }
                        found
                    })
                    .map(|(i, _)| i + 1)
                    .collect();

                if !line_numbers.is_empty() {
                    results.push((file_path.to_owned(), line_numbers));
                }
            } else {
                println!("    Could not read file: {:?}", file_path);
            }
        }
    }

    Ok(results)
}

/// Get dependency graph of the workspace
pub async fn get_dependency_graph(
    manager: &CargoManager,
) -> Result<std::collections::HashMap<String, Vec<String>>> {
    use serde_json;
    let mut graph = std::collections::HashMap::new();

    for member in manager.get_workspace_members() {
        if let Ok(output) = tokio::process::Command::new("cargo")
            .current_dir(member)
            .args(["metadata", "--format-version=1", "--no-deps"])
            .output()
            .await
        {
            if output.status.success() {
                if let Ok(metadata) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    if let Some(packages) = metadata["packages"].as_array() {
                        for package in packages {
                            if let (Some(pkg_name), Some(deps)) =
                                (package["name"].as_str(), package["dependencies"].as_array())
                            {
                                let dependencies: Vec<String> = deps
                                    .iter()
                                    .filter_map(|d| d["name"].as_str().map(String::from))
                                    .collect();
                                graph.insert(pkg_name.to_string(), dependencies);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(graph)
}
