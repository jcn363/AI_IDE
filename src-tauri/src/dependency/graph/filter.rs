//! Module for filtering dependency graphs

use super::{DependencyGraph, DependencyType, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashSet;

/// A filter for dependency graphs
#[derive(Debug, Clone, Default)]
pub struct DependencyFilter {
    include_types: HashSet<DependencyType>,
    exclude_types: HashSet<DependencyType>,
    include_pattern: Option<String>,
    exclude_pattern: Option<String>,
    max_depth: Option<usize>,
    direct_only: bool,
    workspace_only: bool,
    has_updates: Option<bool>,
    has_vulnerabilities: Option<bool>,
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

    fn matches_node(&self, node: &super::node::DependencyNode) -> bool {
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
        // if let Some(has_vulns) = self.has_vulnerabilities {
        //     if has_vulnerabilities(&node.name, &node.version) != has_vulns {
        //         return false;
        //     }
        // }

        true
    }

    fn apply_depth_filter(&self, graph: &mut DependencyGraph, max_depth: usize) {
        let root_idx = match graph.find_node(graph.root_package()) {
            Some((idx, _)) => idx,
            None => return,
        };

        let mut visited = HashSet::new();
        let mut to_visit = vec![(root_idx, 0)];
        let mut keep = HashSet::new();

        while let Some((node_idx, depth)) = to_visit.pop() {
            if depth > max_depth {
                continue;
            }

            if !visited.insert(node_idx) {
                continue;
            }

            keep.insert(node_idx);

            // Add all dependencies to the visit queue
            for edge in graph
                .graph()
                .edges_directed(node_idx, petgraph::Direction::Outgoing)
            {
                to_visit.push((edge.target(), depth + 1));
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
