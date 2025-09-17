use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use anyhow::{anyhow, Result};
use cargo_metadata::{DependencyKind, Package};
use petgraph::algo::all_simple_paths;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Dfs, EdgeRef};
use petgraph::Direction;

/// Analysis algorithms for dependency graphs
use super::models::*;
use crate::dependency::graph::traversal;

impl DependencyGraph {
    /// Get a reference to the underlying graph
    pub fn graph(&self) -> &DiGraph<DependencyNode, DependencyEdge> {
        &self.graph
    }

    /// Get a mutable reference to the underlying graph
    pub fn graph_mut(&mut self) -> &mut DiGraph<DependencyNode, DependencyEdge> {
        &mut self.graph
    }

    /// Get the root package name
    pub fn root_package(&self) -> &str {
        &self.root_package
    }

    /// Find a node by package name
    pub fn find_node(&self, name: &str) -> Option<(NodeIndex, &DependencyNode)> {
        self.node_indices
            .get(name)
            .and_then(|&idx| self.graph.node_weight(idx).map(|n| (idx, n)))
    }

    /// Find all paths between two dependencies
    pub fn find_paths(&self, from: &str, to: &str) -> Result<Vec<Vec<NodeIndex>>> {
        let from_idx = self
            .node_indices
            .get(from)
            .ok_or_else(|| anyhow!("Dependency '{}' not found in graph", from))?;

        let to_idx = self
            .node_indices
            .get(to)
            .ok_or_else(|| anyhow!("Dependency '{}' not found in graph", to))?;

        // Find all simple paths (no cycles) from 'from' to 'to'
        use petgraph::algo::all_simple_paths;
        let paths: Vec<Vec<NodeIndex>> = all_simple_paths(
            &self.graph,
            *from_idx,
            *to_idx,
            0,    // No minimum length
            None, // No maximum length
        )
        .collect();

        Ok(paths)
    }

    /// Get direct dependencies of the root package as a list of package names
    pub fn root_package_dependencies(&self) -> Vec<String> {
        let mut deps = Vec::new();

        if let Some((root_idx, _)) = self.find_node(&self.root_package) {
            // Get all outgoing edges (dependencies) from the root package
            for edge in self.graph.edges_directed(root_idx, Direction::Outgoing) {
                if let Some(target_node) = self.graph.node_weight(edge.target()) {
                    deps.push(target_node.name.clone());
                }
            }
        }

        deps
    }

    /// Get all transitive dependencies as a list of package names
    pub fn all_dependencies(&self) -> Vec<String> {
        let mut all_deps = HashSet::new();

        if let Some((root_idx, _)) = self.find_node(&self.root_package) {
            let mut dfs = Dfs::new(&self.graph, root_idx);

            while let Some(node_idx) = dfs.next(&self.graph) {
                if node_idx == root_idx {
                    continue; // Skip the root package itself
                }

                if let Some(node) = self.graph.node_weight(node_idx) {
                    all_deps.insert(node.name.clone());
                }
            }
        }

        // Convert HashSet to sorted Vec for deterministic output
        let mut result: Vec<String> = all_deps.into_iter().collect();
        result.sort();
        result
    }
}

// Removed duplicate DependencyEdge implementation - now using the canonical one from graph::edge

/// Builder for constructing a dependency graph from Cargo metadata
#[derive(Debug)]
pub struct DependencyGraphBuilder {
    metadata:     cargo_metadata::Metadata,
    root_package: String,
}

// DependencyGraphBuilder intentionally does not implement Default
// Use DependencyGraphBuilder::new(project_path) instead

impl DependencyGraphBuilder {
    /// Create a new builder for the project at the given path
    pub fn new(project_path: &Path) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(project_path.join("Cargo.toml"))
            .exec()?;

        let root_package = metadata
            .root_package()
            .ok_or_else(|| anyhow!("No root package found"))?
            .name
            .to_string();

        Ok(Self {
            metadata,
            root_package,
        })
    }

    /// Build the dependency graph
    pub fn build(self) -> Result<DependencyGraph> {
        let mut graph = DiGraph::new();
        let mut node_indices = HashMap::new();
        let packages = self.metadata.packages.clone(); // Extract packages to avoid borrowing conflict

        // First pass: add all packages as nodes
        for package in &packages {
            let node = DependencyNode::from_package(package);
            let idx = graph.add_node(node);
            node_indices.insert(package.id.repr.clone(), idx);
        }

        // Second pass: add edges between dependencies
        for package in &packages {
            if let Some(&source_idx) = node_indices.get(&package.id.repr) {
                self.add_dependencies(&mut graph, &node_indices, package, source_idx)?;
            }
        }

        Ok(DependencyGraph {
            graph,
            node_indices: node_indices.into_iter().map(|(k, v)| (k, v)).collect(),
            root_package: self.root_package,
        })
    }

    fn add_dependencies(
        &self,
        graph: &mut DiGraph<DependencyNode, DependencyEdge>,
        node_indices: &HashMap<String, NodeIndex>,
        package: &Package,
        source_idx: NodeIndex,
    ) -> Result<()> {
        static EMPTY_DEPS: Vec<cargo_metadata::PackageId> = Vec::new();
        let deps = self
            .metadata
            .resolve
            .as_ref()
            .and_then(|r| r.nodes.iter().find(|n| n.id == package.id))
            .map(|n| &n.dependencies)
            .unwrap_or(&EMPTY_DEPS);

        for dep_id in deps {
            if let Some(&target_idx) = node_indices.get(&dep_id.repr) {
                let dep_pkg = self
                    .metadata
                    .packages
                    .iter()
                    .find(|p| &p.id == dep_id)
                    .ok_or_else(|| anyhow!("Dependency not found: {}", dep_id))?;

                let dep_info = package
                    .dependencies
                    .iter()
                    .find(|d| d.name == dep_pkg.name.to_string())
                    .ok_or_else(|| anyhow!("Dependency info not found: {}", dep_pkg.name))?;

                let edge = DependencyEdge {
                    dep_type:              if dep_info.kind == DependencyKind::Development {
                        DependencyType::Dev
                    } else if dep_info.kind == DependencyKind::Build {
                        DependencyType::Build
                    } else {
                        DependencyType::Normal
                    },
                    version_req:           dep_info.req.to_string(),
                    optional:              dep_info.optional,
                    uses_default_features: dep_info.uses_default_features,
                    features:              dep_info.features.clone(),
                    target:                dep_info.target.clone().map(|t| t.to_string()),
                };

                graph.add_edge(source_idx, target_idx, edge);
            }
        }

        Ok(())
    }
}

impl DependencyFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Include only dependencies of the specified types
    pub fn include_types(mut self, types: &[DependencyType]) -> Self {
        self.include_types = types.iter().cloned().collect();
        self
    }

    /// Exclude dependencies of the specified types
    pub fn exclude_types(mut self, types: &[DependencyType]) -> Self {
        self.exclude_types = types.iter().cloned().collect();
        self
    }

    /// Filter by name pattern (supports glob)
    pub fn include_pattern(mut self, pattern: &str) -> Self {
        self.include_pattern = Some(pattern.to_string());
        self
    }

    /// Exclude by name pattern (supports glob)
    pub fn exclude_pattern(mut self, pattern: &str) -> Self {
        self.exclude_pattern = Some(pattern.to_string());
        self
    }

    /// Set maximum dependency depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Show only direct dependencies
    pub fn direct_only(mut self, direct: bool) -> Self {
        self.direct_only = direct;
        self
    }

    /// Show only workspace members
    pub fn workspace_only(mut self, workspace: bool) -> Self {
        self.workspace_only = workspace;
        self
    }

    /// Filter by update availability
    pub fn has_updates(mut self, has_updates: bool) -> Self {
        self.has_updates = Some(has_updates);
        self
    }

    /// Filter by vulnerability status
    pub fn has_vulnerabilities(mut self, has_vulns: bool) -> Self {
        self.has_vulnerabilities = Some(has_vulns);
        self
    }

    /// Apply the filter to a dependency graph
    pub fn apply(&self, graph: &DependencyGraph) -> DependencyGraph {
        let mut result = graph.clone();
        let mut to_remove = Vec::new();

        // Apply filters to nodes
        for node_idx in graph.graph().node_indices() {
            if let Some((_, node)) = graph.graph().node_weight(node_idx).map(|n| (node_idx, n)) {
                if !self.matches_node(node) {
                    to_remove.push(node_idx);
                }
            }
        }

        // Remove nodes that don't match the filter
        for node_idx in to_remove {
            result.graph_mut().remove_node(node_idx);
        }

        // If we're filtering by depth, we need to do a BFS from the root
        if let Some(max_depth) = self.max_depth {
            self.apply_depth_filter(&mut result, max_depth);
        }

        result
    }

    fn matches_node(&self, node: &DependencyNode) -> bool {
        // Check type filters
        if !self.include_types.is_empty()
            && !node
                .dependencies
                .iter()
                .any(|d| self.include_types.contains(&d.dep_type))
        {
            return false;
        }

        if node
            .dependencies
            .iter()
            .any(|d| self.exclude_types.contains(&d.dep_type))
        {
            return false;
        }

        // Check name patterns
        if let Some(pattern) = &self.include_pattern {
            if !glob_match::glob_match(pattern, &node.name) {
                return false;
            }
        }

        if let Some(pattern) = &self.exclude_pattern {
            if glob_match::glob_match(pattern, &node.name) {
                return false;
            }
        }

        // Check other flags
        if self.direct_only && !node.is_direct() {
            return false;
        }

        if self.workspace_only && !node.is_workspace() {
            return false;
        }

        if let Some(has_updates) = self.has_updates {
            if node.latest_version.is_some() != has_updates {
                return false;
            }
        }

        // Note: Vulnerability check would be implemented with an external service
        true
    }

    fn apply_depth_filter(&self, graph: &mut DependencyGraph, max_depth: usize) {
        let root_idx = match graph.find_node(graph.root_package()) {
            Some((idx, _)) => idx,
            None => return,
        };

        let mut visited = HashSet::new();
        let mut to_visit = VecDeque::new();
        to_visit.push_back((root_idx, 0));
        let mut keep = HashSet::new();

        while let Some((node_idx, depth)) = to_visit.pop_front() {
            if depth > max_depth {
                continue;
            }

            if !visited.insert(node_idx) {
                continue;
            }

            keep.insert(node_idx);

            // Add all dependencies to the visit queue
            for edge in graph.graph().edges_directed(node_idx, Direction::Outgoing) {
                to_visit.push_back((edge.target(), depth + 1));
            }
        }

        // Remove nodes not in the keep set
        let to_remove: Vec<_> = graph
            .graph()
            .node_indices()
            .filter(|&idx| !keep.contains(&idx))
            .collect();

        for idx in to_remove {
            graph.graph_mut().remove_node(idx);
        }
    }
}

// Re-export traversal functions
pub use traversal::{depth_first_traverse, find_all_dependencies, find_all_paths, find_reverse_dependencies};