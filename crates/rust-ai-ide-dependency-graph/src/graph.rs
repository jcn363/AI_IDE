//! Core dependency graph structures and algorithms

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableGraph;
use petgraph::visit::{EdgeRef, IntoEdgesDirected};
use petgraph::Directed;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::*;

/// Types of dependency relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    Normal,
    Dev,
    Build,
    Workspace,
    Optional,
}

/// Information about a specific dependency requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRequirement {
    pub name:             String,
    pub version_req:      String,
    pub dep_type:         DependencyType,
    pub optional:         bool,
    pub features:         Vec<String>,
    pub default_features: bool,
    pub target:           Option<String>,
}

/// Represents a node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub name:                String,
    pub version:             Option<String>,
    pub description:         Option<String>,
    pub repository:          Option<String>,
    pub license:             Option<String>,
    pub authors:             Vec<String>,
    pub keywords:            Vec<String>,
    pub categories:          Vec<String>,
    pub homepage:            Option<String>,
    pub documentation:       Option<String>,
    pub readme:              Option<String>,
    pub is_workspace_member: bool,
    pub source_url:          Option<String>,
    pub checksum:            Option<String>,
    pub yanked:              bool,
    pub created_at:          Option<String>,
}

/// Edge representing a dependency relationship between packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub dep_type:           DependencyType,
    pub source_name:        String,
    pub target_name:        String,
    pub version_constraint: Option<String>,
    pub features_requested: Vec<String>,
    pub features_enabled:   Vec<String>,
    pub optional:           bool,
    pub req_depth:          usize,
}

/// Main dependency graph structure using petgraph for efficient graph operations
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub graph:             StableGraph<DependencyNode, DependencyEdge, Directed>,
    pub node_indices:      HashMap<String, NodeIndex>,
    pub root_package:      Option<String>,
    pub workspace_members: HashSet<String>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            graph:             StableGraph::new(),
            node_indices:      HashMap::new(),
            root_package:      None,
            workspace_members: HashSet::new(),
        }
    }

    /// Add a package node to the graph
    pub fn add_package(&mut self, package: DependencyNode) -> DependencyResult<NodeIndex> {
        let package_name = package.name.clone();

        // Check if package already exists
        if let Some(&existing_idx) = self.node_indices.get(&package_name) {
            // Update existing node with potentially newer information
            if let Some(node) = self.graph.node_weight_mut(existing_idx) {
                // Merge information, preferring non-None values
                if node.description.is_none() {
                    node.description = package.description;
                }
                if node.repository.is_none() {
                    node.repository = package.repository;
                }
                // Continue for other fields...
                node.authors.extend_from_slice(&package.authors);
                node.authors.sort();
                node.authors.dedup();
                node.keywords.extend_from_slice(&package.keywords);
                node.keywords.sort();
                node.keywords.dedup();
            }
            Ok(existing_idx)
        } else {
            // Add new node
            let node_idx = self.graph.add_node(package);
            self.node_indices.insert(package_name, node_idx);
            Ok(node_idx)
        }
    }

    /// Add a dependency edge between packages
    pub fn add_dependency(&mut self, source: &str, target: &str, edge: DependencyEdge) -> DependencyResult<EdgeIndex> {
        let source_idx = self
            .node_indices
            .get(source)
            .copied()
            .ok_or_else(|| DependencyError::PackageNotFound(source.to_string()))?;

        // Add target package if it doesn't exist
        if !self.node_indices.contains_key(target) {
            let dummy_node = DependencyNode {
                name:                target.to_string(),
                version:             None,
                description:         None,
                repository:          None,
                license:             None,
                authors:             Vec::new(),
                keywords:            Vec::new(),
                categories:          Vec::new(),
                homepage:            None,
                documentation:       None,
                readme:              None,
                is_workspace_member: false,
                source_url:          None,
                checksum:            None,
                yanked:              false,
                created_at:          None,
            };
            let target_idx = self.graph.add_node(dummy_node);
            self.node_indices.insert(target.to_string(), target_idx);
        }

        let target_idx = self.node_indices[target];

        // Check if edge already exists to avoid duplicates
        for edge_ref in self
            .graph
            .edges_directed(source_idx, petgraph::Direction::Outgoing)
        {
            if edge_ref.target() == target_idx {
                // Edge already exists, could update or return existing
                return Ok(edge_ref.id());
            }
        }

        Ok(self.graph.add_edge(source_idx, target_idx, edge))
    }

    /// Get dependencies of a package
    pub fn get_dependencies(&self, package_name: &str) -> DependencyResult<Vec<(String, DependencyEdge)>> {
        let node_idx = self
            .node_indices
            .get(package_name)
            .copied()
            .ok_or_else(|| DependencyError::PackageNotFound(package_name.to_string()))?;

        let mut dependencies = Vec::new();
        for edge_ref in self
            .graph
            .edges_directed(node_idx, petgraph::Direction::Outgoing)
        {
            if let Some(target_node) = self.graph.node_weight(edge_ref.target()) {
                dependencies.push((target_node.name.clone(), edge_ref.weight().clone()));
            }
        }

        Ok(dependencies)
    }

    /// Get dependants (reverse dependencies) of a package
    pub fn get_dependants(&self, package_name: &str) -> DependencyResult<Vec<(String, DependencyEdge)>> {
        let node_idx = self
            .node_indices
            .get(package_name)
            .copied()
            .ok_or_else(|| DependencyError::PackageNotFound(package_name.to_string()))?;

        let mut dependants = Vec::new();
        for edge_ref in self
            .graph
            .edges_directed(node_idx, petgraph::Direction::Incoming)
        {
            if let Some(source_node) = self.graph.node_weight(edge_ref.source()) {
                dependants.push((source_node.name.clone(), edge_ref.weight().clone()));
            }
        }

        Ok(dependants)
    }

    /// Get all packages in the graph
    pub fn get_all_packages(&self) -> Vec<&DependencyNode> {
        self.graph.node_weights().collect()
    }

    /// Check if two packages share a dependency
    pub fn has_shared_dependency(&self, package1: &str, package2: &str) -> DependencyResult<bool> {
        let deps1: HashSet<String> = self
            .get_dependencies(package1)?
            .into_iter()
            .map(|(name, _)| name)
            .collect();

        let deps2: HashSet<String> = self
            .get_dependencies(package2)?
            .into_iter()
            .map(|(name, _)| name)
            .collect();

        Ok(!deps1.is_disjoint(&deps2))
    }

    /// Get the depth of a package in the dependency tree
    pub fn get_dependency_depth(&self, package_name: &str) -> DependencyResult<usize> {
        if let Some(root) = &self.root_package {
            self.get_path_length(root, package_name)
        } else {
            Err(DependencyError::ResolutionError {
                package: package_name.to_string(),
                reason:  "No root package set".to_string(),
            })
        }
    }

    /// Get shortest path length between two packages
    pub fn get_path_length(&self, from: &str, to: &str) -> DependencyResult<usize> {
        let start_idx = self
            .node_indices
            .get(from)
            .copied()
            .ok_or_else(|| DependencyError::PackageNotFound(from.to_string()))?;

        let end_idx = self
            .node_indices
            .get(to)
            .copied()
            .ok_or_else(|| DependencyError::PackageNotFound(to.to_string()))?;

        // BFS to find shortest path
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut distances = HashMap::new();

        queue.push_back(start_idx);
        visited.insert(start_idx);
        distances.insert(start_idx, 0);

        while let Some(current) = queue.pop_front() {
            if current == end_idx {
                return Ok(distances[&current]);
            }

            for neighbor in self
                .graph
                .neighbors_directed(current, petgraph::Direction::Outgoing)
            {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                    distances.insert(neighbor, distances[&current] + 1);
                }
            }
        }

        Err(DependencyError::ResolutionError {
            package: to.to_string(),
            reason:  "No path found from source".to_string(),
        })
    }

    /// Check if the graph has cycles
    pub fn has_cycles(&self) -> bool {
        petgraph::algo::tarjan_scc(&self.graph)
            .into_iter()
            .any(|scc| scc.len() > 1)
    }

    /// Get all cycles in the graph
    pub fn get_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();

        for scc in petgraph::algo::tarjan_scc(&self.graph) {
            if scc.len() > 1 {
                let cycle: Vec<String> = scc
                    .into_iter()
                    .filter_map(|node_idx| {
                        self.graph
                            .node_weight(node_idx)
                            .map(|node| node.name.clone())
                    })
                    .collect();
                cycles.push(cycle);
            }
        }

        cycles
    }

    /// Perform topological sorting of the graph
    pub fn topological_sort(&self) -> DependencyResult<Vec<String>> {
        if self.has_cycles() {
            let cycles = self.get_cycles();
            return Err(DependencyError::CircularDependency(
                cycles.into_iter().flatten().collect(),
            ));
        }

        petgraph::algo::toposort(&self.graph, None)
            .map_err(|_| DependencyError::ResolutionError {
                package: "graph".to_string(),
                reason:  "Cycle detected during topological sort".to_string(),
            })
            .and_then(|nodes| {
                Ok(nodes
                    .into_iter()
                    .filter_map(|node_idx| {
                        self.graph
                            .node_weight(node_idx)
                            .map(|node| node.name.clone())
                    })
                    .collect())
            })
    }

    /// Apply a filter to get a subset of the graph
    pub fn filter_packages<F>(&self, predicate: F) -> Vec<&DependencyNode>
    where
        F: Fn(&DependencyNode) -> bool,
    {
        self.graph
            .node_weights()
            .filter(|node| predicate(node))
            .collect()
    }

    /// Set the root package for dependency depth calculations
    pub fn set_root_package(&mut self, package_name: String) -> DependencyResult<()> {
        if self.node_indices.contains_key(&package_name) {
            self.root_package = Some(package_name);
            Ok(())
        } else {
            Err(DependencyError::PackageNotFound(package_name))
        }
    }

    /// Get statistics about the graph
    pub fn get_statistics(&self) -> DependencyGraphStats {
        let nodes = self.graph.node_count();
        let edges = self.graph.edge_count();

        let mut workspace_members = 0;
        let mut total_dependencies = 0;

        for node in self.graph.node_weights() {
            if node.is_workspace_member {
                workspace_members += 1;
            }
        }

        for _edge in self.graph.edge_weights() {
            total_dependencies += 1;
        }

        DependencyGraphStats {
            total_packages: nodes,
            total_dependencies: edges,
            workspace_members,
            has_cycles: self.has_cycles(),
        }
    }

    /// Convert the graph to DOT format for visualization
    pub fn to_dot(&self) -> String {
        use petgraph::dot::{Config, Dot};

        format!(
            "{:?}",
            Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        )
    }

    /// Get packages by their dependency depth
    pub fn get_packages_by_depth(&self) -> DependencyResult<Vec<(String, usize)>> {
        let mut packages = Vec::new();

        for (package_name, &node_idx) in &self.node_indices {
            if let Some(root) = &self.root_package {
                match self.get_path_length(&root, &package_name) {
                    Ok(depth) => packages.push((package_name.clone(), depth)),
                    Err(_) => continue, // Package not reachable from root
                }
            } else {
                // If no root, use distance from first workspace member or assume depth 0
                packages.push((package_name.clone(), 0));
            }
        }

        packages.sort_by(|a, b| a.1.cmp(&b.1));
        Ok(packages)
    }
}

/// Statistics about the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphStats {
    pub total_packages:     usize,
    pub total_dependencies: usize,
    pub workspace_members:  usize,
    pub has_cycles:         bool,
}

/// Thread-safe dependency graph wrapper
#[derive(Clone)]
pub struct SharedDependencyGraph {
    graph: Arc<RwLock<DependencyGraph>>,
}

impl SharedDependencyGraph {
    /// Create a new shared dependency graph
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(DependencyGraph::new())),
        }
    }

    /// Read access to the graph
    pub async fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&DependencyGraph) -> R + Send,
        R: Send,
    {
        let guard = self.graph.read().await;
        f(&guard)
    }

    /// Write access to the graph
    pub async fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut DependencyGraph) -> R + Send,
        R: Send,
    {
        let mut guard = self.graph.write().await;
        f(&mut guard)
    }
}

impl Default for SharedDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}
