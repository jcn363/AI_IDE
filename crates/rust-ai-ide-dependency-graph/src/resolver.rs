//! Dependency resolution algorithms and strategies

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rayon::prelude::*;
use semver::{Version, VersionReq};
use tokio::sync::RwLock;

use crate::error::*;
use crate::graph::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    Conservative,     // Prefer existing versions, minimize changes
    Aggressive,       // Use latest compatible versions
    LatestCompatible, // Use latest compatible versions for direct deps only
    WorkspaceAware,   // Consider workspace members first
}

pub struct DependencyResolver {
    graph:                Arc<RwLock<DependencyGraph>>,
    strategy:             ResolutionStrategy,
    max_parallel_fetches: usize,
}

impl DependencyResolver {
    pub fn new(graph: Arc<RwLock<DependencyGraph>>, strategy: ResolutionStrategy) -> Self {
        Self {
            graph,
            strategy,
            max_parallel_fetches: 10,
        }
    }

    pub fn with_parallel_fetches(mut self, max_fetches: usize) -> Self {
        self.max_parallel_fetches = max_fetches;
        self
    }

    /// Resolve dependency conflicts in the graph
    pub async fn resolve_conflicts(&self) -> DependencyResult<HashMap<String, String>> {
        let mut selected_versions = HashMap::new();

        match self.strategy {
            ResolutionStrategy::Conservative => {
                selected_versions = self.resolve_conservative().await?;
            }
            ResolutionStrategy::Aggressive => {
                selected_versions = self.resolve_aggressive().await?;
            }
            ResolutionStrategy::LatestCompatible => {
                selected_versions = self.resolve_latest_compatible().await?;
            }
            ResolutionStrategy::WorkspaceAware => {
                selected_versions = self.resolve_workspace_aware().await?;
            }
        }

        Ok(selected_versions)
    }

    async fn resolve_conservative(&self) -> DependencyResult<HashMap<String, String>> {
        let graph = self.graph.read().await;

        // Analyze version conflicts
        let conflicts = self.analyze_version_conflicts(&graph).await?;

        let mut selected_versions = HashMap::new();

        for (package, conflict) in conflicts {
            // Select the most common version that satisfies all constraints
            let selected_version = self.select_conservative_version(&conflict)?;
            selected_versions.insert(package, selected_version);
        }

        Ok(selected_versions)
    }

    async fn resolve_aggressive(&self) -> DependencyResult<HashMap<String, String>> {
        let graph = self.graph.read().await;

        // Analyze version conflicts
        let conflicts = self.analyze_version_conflicts(&graph).await?;

        let mut selected_versions = HashMap::new();

        for (package, conflict) in conflicts {
            // Select the latest compatible version
            let selected_version = self.select_aggressive_version(&conflict)?;
            selected_versions.insert(package, selected_version);
        }

        Ok(selected_versions)
    }

    async fn resolve_latest_compatible(&self) -> DependencyResult<HashMap<String, String>> {
        let mut graph = self.graph.write().await;

        let mut selected_versions = HashMap::new();

        // First pass: resolve direct dependencies
        for (_, node_idx) in &graph.node_indices {
            if let Some(node) = graph.graph.node_weight(*node_idx) {
                if self.is_direct_dependency(&graph, node).await {
                    let node_deps = graph.get_dependencies(&node.name)?;
                    for (dep_name, dep_edge) in node_deps {
                        if let Some(version_req) = &dep_edge.version_constraint {
                            let latest_version = self
                                .find_latest_compatible_version(&dep_name, version_req)
                                .await?;
                            selected_versions.insert(dep_name, latest_version);
                        }
                    }
                }
            }
        }

        Ok(selected_versions)
    }

    async fn resolve_workspace_aware(&self) -> DependencyResult<HashMap<String, String>> {
        let graph = self.graph.read().await;

        let mut selected_versions = HashMap::new();
        let workspace_members: HashSet<String> = graph.workspace_members.iter().cloned().collect();

        // First resolve workspace members
        for member in &workspace_members {
            if let Some(node_idx) = graph.node_indices.get(member) {
                if let Some(node) = graph.graph.node_weight(*node_idx) {
                    if let Some(version) = &node.version {
                        selected_versions.insert(member.clone(), version.clone());
                    }
                }
            }
        }

        // Then resolve external dependencies with workspace awareness
        let conflicts = self.analyze_version_conflicts(&graph).await?;

        for (package, conflict) in conflicts {
            if !workspace_members.contains(&package) {
                let selected_version = self.select_workspace_aware_version(&conflict)?;
                selected_versions.insert(package, selected_version);
            }
        }

        Ok(selected_versions)
    }

    /// Analyze version conflicts across the dependency graph
    async fn analyze_version_conflicts(
        &self,
        graph: &DependencyGraph,
    ) -> DependencyResult<HashMap<String, VersionConflict>> {
        let mut conflicts = HashMap::new();

        for (_, node_idx) in &graph.node_indices {
            if let Some(node) = graph.graph.node_weight(*node_idx) {
                let dependencies = graph.get_dependencies(&node.name)?;

                for (dep_name, dep_edge) in dependencies {
                    if let Some(version_req) = &dep_edge.version_constraint {
                        conflicts
                            .entry(dep_name.clone())
                            .or_insert_with(VersionConflict::new)
                            .add_constraint(node.name.clone(), version_req.clone(), dep_edge.req_depth);
                    }
                }
            }
        }

        Ok(conflicts
            .into_iter()
            .filter(|(_, conflict)| conflict.has_conflict())
            .collect())
    }

    fn select_conservative_version(&self, conflict: &VersionConflict) -> DependencyResult<String> {
        // Count version requirements and select the most common one
        let mut version_counts = HashMap::new();

        for constraint in &conflict.constraints {
            let version_req = VersionReq::parse(&constraint.version_req).map_err(|_| {
                DependencyError::ParseError(format!(
                    "Invalid version requirement: {}",
                    constraint.version_req
                ))
            })?;

            if let Ok(version) = self.select_matching_version(&version_req, &conflict.constraints) {
                *version_counts.entry(version).or_insert(0) += 1;
            }
        }

        let selected_version = version_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(version, _)| version)
            .unwrap_or_else(|| "1.0.0".to_string());

        Ok(selected_version)
    }

    fn select_aggressive_version(&self, conflict: &VersionConflict) -> DependencyResult<String> {
        // Find the highest compatible version for all constraints
        let all_versions = self.get_all_compatible_versions(conflict)?;
        let latest_version = all_versions
            .into_iter()
            .filter_map(|v| Version::parse(&v).ok())
            .max()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "1.0.0".to_string());

        Ok(latest_version)
    }

    fn select_workspace_aware_version(&self, conflict: &VersionConflict) -> DependencyResult<String> {
        // Prefer versions that are already used in the workspace
        self.select_conservative_version(conflict)
    }

    fn select_matching_version(
        &self,
        version_req: &VersionReq,
        constraints: &[PackageConstraint],
    ) -> Result<String, ()> {
        // This is a simplified implementation - in a real scenario,
        // you would fetch available versions from crates.io
        let candidates = vec!["1.0.0", "1.0.1", "1.1.0", "1.2.0", "2.0.0", "2.1.0"];

        for candidate in candidates {
            if let Ok(version) = Version::parse(candidate) {
                if version_req.matches(&version) {
                    return Ok(candidate.to_string());
                }
            }
        }

        Err(())
    }

    fn get_all_compatible_versions(&self, conflict: &VersionConflict) -> DependencyResult<Vec<String>> {
        let mut compatible_versions = HashSet::new();

        for constraint in &conflict.constraints {
            let version_req = VersionReq::parse(&constraint.version_req).map_err(|_| {
                DependencyError::ParseError(format!(
                    "Invalid version requirement: {}",
                    constraint.version_req
                ))
            })?;

            // Again, simplified - would fetch real versions in production
            let candidates = vec!["1.0.0", "1.0.1", "1.1.0", "1.2.0", "2.0.0", "2.1.0"];

            for candidate in candidates {
                if let Ok(version) = Version::parse(candidate) {
                    if version_req.matches(&version) {
                        compatible_versions.insert(candidate.to_string());
                    }
                }
            }
        }

        Ok(compatible_versions.into_iter().collect())
    }

    async fn find_latest_compatible_version(&self, package: &str, version_req: &str) -> DependencyResult<String> {
        let version_req = VersionReq::parse(version_req)
            .map_err(|_| DependencyError::ParseError(format!("Invalid version requirement: {}", version_req)))?;

        // Simplified - would fetch real latest version from registry
        self.select_matching_version(&version_req, &[])
            .map_err(|_| DependencyError::ResolutionError {
                package: package.to_string(),
                reason:  "No compatible version found".to_string(),
            })
    }

    async fn is_direct_dependency(&self, graph: &DependencyGraph, node: &DependencyNode) -> bool {
        if let Some(root) = &graph.root_package {
            // A dependency is direct if it's one level from root
            if let Ok(dependencies) = graph.get_dependencies(root) {
                dependencies
                    .iter()
                    .any(|(dep_name, _)| dep_name == &node.name)
            } else {
                false
            }
        } else {
            // If no root specified, assume all known dependencies are direct
            !graph.workspace_members.contains(&node.name)
        }
    }

    /// Perform parallel resolution of multiple packages
    pub async fn resolve_parallel(&self, packages: Vec<String>) -> DependencyResult<HashMap<String, String>> {
        let results: Vec<_> = packages
            .par_chunks(self.max_parallel_fetches)
            .map(|chunk| {
                // This is where you would perform parallel fetches/resolution
                // Using rayon for CPU-bound operations
                chunk
                    .iter()
                    .cloned()
                    .map(|pkg| (pkg, "1.0.0".to_string()))
                    .collect::<HashMap<_, _>>()
            })
            .collect();

        let mut final_results = HashMap::new();
        for result in results {
            final_results.extend(result);
        }

        Ok(final_results)
    }

    /// Apply resolved versions back to the graph
    pub async fn apply_resolved_versions(&self, resolved_versions: &HashMap<String, String>) -> DependencyResult<()> {
        let mut graph = self.graph.write().await;

        for (package_name, version) in resolved_versions {
            if let Some(&node_idx) = graph.node_indices.get(package_name) {
                if let Some(node) = graph.graph.node_weight_mut(node_idx) {
                    node.version = Some(version.clone());
                }
            }
        }

        Ok(())
    }
}

/// Representation of a version conflict between packages
#[derive(Debug, Clone)]
pub struct VersionConflict {
    pub package:     String,
    pub constraints: Vec<PackageConstraint>,
}

impl VersionConflict {
    pub fn new() -> Self {
        Self {
            package:     String::new(),
            constraints: Vec::new(),
        }
    }

    pub fn add_constraint(&mut self, source_package: String, version_req: String, depth: usize) -> &mut Self {
        self.constraints.push(PackageConstraint {
            source_package,
            version_req,
            depth,
        });
        self
    }

    pub fn has_conflict(&self) -> bool {
        let mut version_reqs = HashSet::new();
        for constraint in &self.constraints {
            version_reqs.insert(&constraint.version_req);
        }
        version_reqs.len() > 1
    }

    pub fn get_conflicting_packages(&self) -> Vec<String> {
        self.constraints
            .iter()
            .map(|c| c.source_package.clone())
            .collect()
    }
}

/// Constraint information for a package dependency
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PackageConstraint {
    pub source_package: String,
    pub version_req:    String,
    pub depth:          usize,
}

impl Ord for PackageConstraint {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by depth (shallow first) then by package name
        match self.depth.cmp(&other.depth) {
            Ordering::Equal => self.source_package.cmp(&other.source_package),
            other => other,
        }
    }
}

impl PartialOrd for PackageConstraint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
