//! Project management commands and functionality
//!
//! This module contains commands for managing projects, dependencies,
//! build configurations, and other project-level operations.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::dependency::graph::{self, Edge, Graph, Node};
use crate::dependency::{DependencyInfo, DependencyUpdate, DependencyUpdateChecker, DependencyUpdater};
use crate::license::LicenseComplianceChecker;
use crate::security::VulnerabilityScanner;

#[tauri::command]
pub async fn check_vulnerabilities(
    manifest_path: PathBuf,
) -> Result<Vec<crate::security::vulnerability_scanner::VulnerabilityReport>, String> {
    let scanner = VulnerabilityScanner::new().map_err(|e| e.to_string())?;
    scanner.check_dependencies(&manifest_path);
    // In a real implementation, we would return the actual reports
    Ok(Vec::new())
}

#[tauri::command]
pub async fn check_license_compliance(
    license: String,
) -> Result<crate::license::compliance_checker::LicenseCompliance, String> {
    let checker = LicenseComplianceChecker::default();
    Ok(checker.check_license(&license))
}

#[tauri::command]
pub async fn update_dependencies(manifest_path: PathBuf, dry_run: bool) -> Result<Vec<DependencyUpdate>, String> {
    let updater = DependencyUpdater::new(manifest_path);
    updater
        .update_dependencies(dry_run)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_dependency_updates(project_path: PathBuf) -> Result<Vec<DependencyInfo>, String> {
    let checker = DependencyUpdateChecker::new(project_path);
    checker.check_updates()
}

#[derive(Serialize)]
pub struct ProjectDependencyInfo {
    name:            String,
    version:         String,
    license:         Option<String>,
    vulnerabilities: Vec<String>,
    updates:         Vec<DependencyUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphNode {
    id:              String,
    name:            String,
    version:         String,
    #[serde(skip_serializing_if = "Option::is_none")]
    license:         Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    vulnerabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphLink {
    source: String,
    target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label:  Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyGraphData {
    nodes: Vec<GraphNode>,
    links: Vec<GraphLink>,
}

#[tauri::command]
pub async fn get_dependency_graph(project_path: PathBuf) -> Result<DependencyGraphData, String> {
    // Build the dependency graph
    let graph = graph::GraphBuilder::new(project_path.clone())
        .build()
        .map_err(|e| format!("Failed to build dependency graph: {}", e))?;

    // Convert nodes
    let nodes = graph
        .nodes()
        .values()
        .map(|node| {
            GraphNode {
                id:              node.id().to_string(),
                name:            node.name().to_string(),
                version:         node.version().to_string(),
                license:         node.license().map(|l| l.to_string()),
                vulnerabilities: Vec::new(), // Will be populated from vulnerability scanner
            }
        })
        .collect();

    // Convert edges
    let links = graph
        .edges()
        .iter()
        .map(|edge| GraphLink {
            source: edge.source().to_string(),
            target: edge.target().to_string(),
            label:  edge.label().map(|s| s.to_string()),
        })
        .collect();

    Ok(DependencyGraphData { nodes, links })
}

#[tauri::command]
pub async fn get_project_dependency_info(manifest_path: PathBuf) -> Result<Vec<ProjectDependencyInfo>, String> {
    // This would combine all the above functionality
    // Implementation omitted for brevity
    Ok(Vec::new())
}
