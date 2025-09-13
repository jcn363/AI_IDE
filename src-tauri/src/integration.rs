//! Unified Cargo Integration Layer
//!
//! This module consolidates dependency resolution approaches from:
//! - cargo/service.rs (raw command execution, basic metadata)
//! - dependency/ (structured graph analysis, complex operations)
//! - commands/dependency_commands.rs (command interface layer)
//!
//! Provides a single, consistent API for all dependency operations.

use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cargo::service::CargoService;
use crate::dependency::{
    analysis::*, models::*, DependencyFilter, DependencyGraph, DependencyGraphBuilder,
    DependencyInfo, DependencyUpdate, DependencyUpdateChecker, DependencyUpdater, ExportFormat,
};
use crate::license::{LicenseCompliance, LicenseComplianceChecker};
use crate::security::{VulnerabilityReport, VulnerabilityScanner};

/// Unified cargo state that caches both raw and structured data
#[derive(Debug, Clone)]
pub struct CargoState {
    /// Raw metadata from cargo metadata command
    pub raw_metadata: Option<CargoMetadata>,
    /// Structured dependency graph
    pub graph: Option<DependencyGraph>,
    /// Last update timestamp
    pub last_updated: std::time::Instant,
}

/// Main unified Cargo integration service
#[derive(Debug, Clone)]
pub struct UnifiedCargoService {
    /// In-memory cache of project states
    states: Arc<RwLock<HashMap<PathBuf, CargoState>>>,
}

impl Default for UnifiedCargoService {
    fn default() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl UnifiedCargoService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create cached state for a project
    async fn get_or_load_project_state(
        &self,
        project_path: &Path,
    ) -> Result<CargoState, UnifiedCargoError> {
        let mut states = self.states.write().await;

        if let Some(state) = states.get(project_path) {
            // Check if cache is stale (5 seconds)
            if state.last_updated.elapsed().as_secs() < 5 {
                return Ok(state.clone());
            }
        }

        // Load fresh data
        let state = self.load_project_data(project_path).await?;
        states.insert(project_path.to_path_buf(), state.clone());
        Ok(state)
    }

    /// Load all project data (both raw and structured)
    async fn load_project_data(
        &self,
        project_path: &Path,
    ) -> Result<CargoState, UnifiedCargoError> {
        // Load raw metadata
        let raw_metadata =
            CargoService::get_metadata(project_path).map_err(UnifiedCargoError::CargoService)?;

        // Load structured graph
        let graph = DependencyGraphBuilder::new(project_path)
            .map_err(UnifiedCargoError::DependencyAnalysis)?
            .build()
            .map_err(UnifiedCargoError::DependencyAnalysis)?;

        Ok(CargoState {
            raw_metadata: Some(raw_metadata),
            graph: Some(graph),
            last_updated: std::time::Instant::now(),
        })
    }

    /// Invalidate cache for a project
    pub async fn invalidate_cache(&self, project_path: &Path) {
        let mut states = self.states.write().await;
        states.remove(&project_path.to_path_buf());
    }

    // ========================================
    // UNIFIED DEPENDENCY OPERATIONS
    // ========================================

    /// Get consolidated dependency information for a project
    pub async fn get_dependency_info(
        &self,
        project_path: &Path,
    ) -> Result<UnifiedDependencyInfo, UnifiedCargoError> {
        let state = self.get_or_load_project_state(project_path).await?;

        let raw_deps = state
            .raw_metadata
            .as_ref()
            .map(|m| m.packages.len())
            .unwrap_or(0);

        let graph_deps = state
            .graph
            .as_ref()
            .map(|g| g.graph.node_count())
            .unwrap_or(0);

        // Use graph-based analysis as primary source for richer data
        let licenses = if let Some(graph) = &state.graph {
            self.extract_licenses_from_graph(graph).await
        } else {
            HashMap::new()
        };

        Ok(UnifiedDependencyInfo {
            total_dependencies: std::cmp::max(raw_deps, graph_deps),
            direct_dependencies: self.get_direct_dependencies(&state)?,
            transitive_dependencies: self.get_transitive_dependencies(&state)?,
            licenses,
            security_issues: Vec::new(), // Will be populated by security analysis
            update_available: Vec::new(), // Will be populated by update checker
        })
    }

    /// Get dependency graph with both raw and structured data
    pub async fn get_dependency_graph(
        &self,
        project_path: &Path,
    ) -> Result<UnifiedGraph, UnifiedCargoError> {
        let state = self.get_or_load_project_state(project_path).await?;

        let nodes = if let Some(graph) = &state.graph {
            self.convert_graph_nodes_to_unified(&graph)?
        } else {
            Vec::new()
        };

        let edges = if let Some(graph) = &state.graph {
            self.convert_graph_edges_to_unified(&graph)?
        } else {
            Vec::new()
        };

        Ok(UnifiedGraph {
            nodes,
            edges,
            metadata: state.raw_metadata,
            build_info: None, // Could be extended for build performance data
        })
    }

    /// Unified dependency update operation
    pub async fn update_dependencies(
        &self,
        project_path: &Path,
        dry_run: bool,
    ) -> Result<Vec<DependencyUpdate>, UnifiedCargoError> {
        let updater = DependencyUpdater::new(project_path.as_ref());
        let updates = updater
            .update_dependencies(dry_run)
            .await
            .map_err(UnifiedCargoError::DependencyUpdater)?;

        // Invalidate cache after updates
        if !dry_run && !updates.is_empty() {
            self.invalidate_cache(project_path).await;
        }

        Ok(updates)
    }

    /// Check available updates using unified approach
    pub async fn check_updates(
        &self,
        project_path: &Path,
    ) -> Result<Vec<DependencyInfo>, UnifiedCargoError> {
        let checker = DependencyUpdateChecker::new(project_path.as_ref());
        checker
            .check_updates()
            .map_err(UnifiedCargoError::UpdateChecker)
    }

    /// Analyze vulnerabilities
    pub async fn analyze_vulnerabilities(
        &self,
        project_path: &Path,
    ) -> Result<Vec<VulnerabilityReport>, UnifiedCargoError> {
        let scanner = VulnerabilityScanner::new().map_err(UnifiedCargoError::SecurityScanner)?;
        Ok(scanner.check_dependencies(project_path))
    }

    /// Check license compliance
    pub async fn check_license_compliance(
        &self,
        _project_path: &Path,
    ) -> Result<LicenseCompliance, UnifiedCargoError> {
        let checker = LicenseComplianceChecker::default();
        // Simplified - would need actual license policy
        Ok(checker.check_license("MIT"))
    }

    /// Export graph in various formats
    pub async fn export_graph(
        &self,
        project_path: &Path,
        format: ExportFormat,
    ) -> Result<Vec<u8>, UnifiedCargoError> {
        let state = self.get_or_load_project_state(project_path).await?;
        if let Some(graph) = &state.graph {
            graph
                .export(format)
                .map_err(UnifiedCargoError::DependencyAnalysis)
        } else {
            Err(UnifiedCargoError::StateNotLoaded)
        }
    }

    /// Apply filters to dependency graph
    pub async fn filter_dependencies(
        &self,
        project_path: &Path,
        filter: DependencyFilter,
    ) -> Result<UnifiedGraph, UnifiedCargoError> {
        let state = self.get_or_load_project_state(project_path).await?;
        if let Some(graph) = &state.graph {
            let filtered_graph = filter.apply(graph);
            let nodes = self.convert_graph_nodes_to_unified(&filtered_graph)?;
            let edges = self.convert_graph_edges_to_unified(&filtered_graph)?;
            Ok(UnifiedGraph {
                nodes,
                edges,
                metadata: state.raw_metadata,
                build_info: None,
            })
        } else {
            Err(UnifiedCargoError::StateNotLoaded)
        }
    }

    /// Get dependency paths between packages
    pub async fn find_dependency_paths(
        &self,
        project_path: &Path,
        from: &str,
        to: &str,
    ) -> Result<Vec<Vec<NodeInfo>>, UnifiedCargoError> {
        let state = self.get_or_load_project_state(project_path).await?;
        if let Some(graph) = &state.graph {
            // Use graph analysis for path finding
            graph
                .find_paths(from, to)
                .map_err(UnifiedCargoError::DependencyAnalysis)
                .and_then(|paths| {
                    paths
                        .into_iter()
                        .map(|path| {
                            path.into_iter()
                                .filter_map(|idx| graph.graph.node_weight(idx))
                                .map(|node| NodeInfo {
                                    id: node.name.clone(),
                                    name: node.name.clone(),
                                    version: node.version.clone(),
                                    source: format!("{:?}", node.source),
                                })
                                .collect()
                        })
                        .collect::<Vec<Vec<NodeInfo>>>()
                        .pipe(Ok)
                })
        } else {
            Err(UnifiedCargoError::StateNotLoaded)
        }
    }

    // ========================================
    // HELPER METHODS
    // ========================================

    fn get_direct_dependencies(
        &self,
        state: &CargoState,
    ) -> Result<Vec<String>, UnifiedCargoError> {
        if let Some(graph) = &state.graph {
            Ok(graph.root_package_dependencies())
        } else {
            Ok(Vec::new())
        }
    }

    fn get_transitive_dependencies(
        &self,
        state: &CargoState,
    ) -> Result<Vec<String>, UnifiedCargoError> {
        if let Some(graph) = &state.graph {
            Ok(graph.all_dependencies())
        } else {
            Ok(Vec::new())
        }
    }

    async fn extract_licenses_from_graph(
        &self,
        graph: &DependencyGraph,
    ) -> HashMap<String, String> {
        let mut licenses = HashMap::new();
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                if let Some(license) = &node.license {
                    licenses.insert(node.name.clone(), license.clone());
                }
            }
        }
        licenses
    }

    fn convert_graph_nodes_to_unified(
        &self,
        graph: &DependencyGraph,
    ) -> Result<Vec<UnifiedNode>, UnifiedCargoError> {
        Ok(graph
            .graph
            .node_indices()
            .filter_map(|idx| graph.graph.node_weight(idx))
            .map(|node| UnifiedNode {
                id: node.name.clone(),
                name: node.name.clone(),
                version: node.version.clone(),
                license: node.license.clone(),
                source: format!("{:?}", node.source),
                features: node.features.clone(),
            })
            .collect())
    }

    fn convert_graph_edges_to_unified(
        &self,
        graph: &DependencyGraph,
    ) -> Result<Vec<UnifiedEdge>, UnifiedCargoError> {
        Ok(graph
            .graph
            .edge_indices()
            .filter_map(|edge_idx| graph.graph.edge_weight(edge_idx))
            .map(|edge| UnifiedEdge {
                from: String::new(), // Would need to map from node indices
                to: String::new(),
                dep_type: format!("{:?}", edge.dep_type),
                optional: edge.optional,
            })
            .collect())
    }
}

// ========================================
// UNIFIED DATA STRUCTURES
// ========================================

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedDependencyInfo {
    pub total_dependencies: usize,
    pub direct_dependencies: Vec<String>,
    pub transitive_dependencies: Vec<String>,
    pub licenses: HashMap<String, String>,
    pub security_issues: Vec<String>,
    pub update_available: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedGraph {
    pub nodes: Vec<UnifiedNode>,
    pub edges: Vec<UnifiedEdge>,
    pub metadata: Option<CargoMetadata>,
    pub build_info: Option<BuildInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedNode {
    pub id: String,
    pub name: String,
    pub version: String,
    pub license: Option<String>,
    pub source: String,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedEdge {
    pub from: String,
    pub to: String,
    pub dep_type: String,
    pub optional: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildInfo {
    pub build_time: u64,
    pub features_used: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source: String,
}

// Need to bridge to CargoMetadata structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CargoMetadata {
    pub packages: Vec<CargoPackage>,
    pub workspace_root: String,
    pub target_directory: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub manifest_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UnifiedCargoError {
    #[error("Cargo service error: {0}")]
    CargoService(String),
    #[error("Dependency analysis error: {0}")]
    DependencyAnalysis(String),
    #[error("Dependency updater error: {0}")]
    DependencyUpdater(String),
    #[error("Update checker error: {0}")]
    UpdateChecker(String),
    #[error("Security scanner error: {0}")]
    SecurityScanner(String),
    #[error("License compliance error: {0}")]
    LicenseCompliance(String),
    #[error("Project state not loaded")]
    StateNotLoaded,
    #[error("Project not found: {0}")]
    ProjectNotFound(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
