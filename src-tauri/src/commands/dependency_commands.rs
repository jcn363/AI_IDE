use crate::{
    integration::{UnifiedCargoService, UnifiedCargoError, UnifiedGraph, UnifiedNode, UnifiedEdge, NodeInfo},
    license::LicenseComplianceChecker,
    security::VulnerabilityScanner,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn check_vulnerabilities(
    manifest_path: PathBuf,
) -> Result<Vec<crate::security::vulnerability_scanner::VulnerabilityReport>, String> {
    let service = UnifiedCargoService::new();
    service.analyze_vulnerabilities(&manifest_path).await
        .map_err(|e| format!("Failed to analyze vulnerabilities: {}", e))
}

#[tauri::command]
pub async fn check_license_compliance(
    license: String,
    project_path: PathBuf,
) -> Result<crate::license::compliance_checker::LicenseCompliance, String> {
    let service = UnifiedCargoService::new();
    service.check_license_compliance(&project_path).await
        .map_err(|e| format!("Failed to check license compliance: {}", e))
}

#[tauri::command]
pub async fn update_dependencies(
    manifest_path: PathBuf,
    dry_run: bool,
) -> Result<Vec<crate::dependency::updater::DependencyUpdate>, String> {
    let service = UnifiedCargoService::new();
    service.update_dependencies(&manifest_path, dry_run).await
        .map_err(|e| format!("Failed to update dependencies: {}", e))
}

#[tauri::command]
pub async fn check_dependency_updates(
    project_path: PathBuf,
) -> Result<Vec<crate::dependency::update_checker::DependencyInfo>, String> {
    let service = UnifiedCargoService::new();
    service.check_updates(&project_path).await
        .map_err(|e| format!("Failed to check updates: {}", e))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphNode {
    id: String,
    name: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    vulnerabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphLink {
    source: String,
    target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyGraphData {
    nodes: Vec<GraphNode>,
    links: Vec<GraphLink>,
}

#[tauri::command]
pub async fn get_dependency_graph(
    project_path: PathBuf,
) -> Result<DependencyGraphData, String> {
    let service = UnifiedCargoService::new();
    let unified_graph = service.get_dependency_graph(&project_path).await
        .map_err(|e| format!("Failed to get dependency graph: {}", e))?;

    // Convert from unified format back to existing format
    let nodes = unified_graph.nodes
        .into_iter()
        .map(|node| GraphNode {
            id: node.id.clone(),
            name: node.name,
            version: node.version,
            license: node.license,
            vulnerabilities: Vec::new(), // Will be populated from vulnerability scanner
        })
        .collect();

    // Convert edges to links format
    let links = unified_graph.edges
        .into_iter()
        .map(|edge| GraphLink {
            source: edge.from,
            target: edge.to,
            label: Some(edge.dep_type),
        })
        .collect();

    Ok(DependencyGraphData { nodes, links })
}

#[tauri::command]
pub async fn get_dependency_info(
    project_path: PathBuf,
) -> Result<Vec<crate::dependency::update_checker::DependencyInfo>, String> {
    let service = UnifiedCargoService::new();
    service.check_updates(&project_path).await
        .map_err(|e| format!("Failed to get dependency info: {}", e))
}
