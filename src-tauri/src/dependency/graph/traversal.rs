//! Graph traversal algorithms for dependency graphs

use std::collections::{HashMap, HashSet};

use petgraph::visit::{Dfs, EdgeRef};
use petgraph::Direction;

use super::{DependencyEdge, DependencyGraph, DependencyNode};

/// Traverse the dependency graph in depth-first order
pub fn depth_first_traverse<'a>(
    graph: &'a DependencyGraph,
    start: &str,
    direction: Direction,
) -> impl Iterator<Item = (&'a DependencyNode, Vec<&'a DependencyEdge>)> + 'a {
    let start_idx = match graph.find_node(start).map(|(idx, _)| idx) {
        Some(idx) => idx,
        None => {
            tracing::warn!("Node not found during traversal: {}", start);
            return std::iter::empty().collect::<Vec<_>>().into_iter();
        }
    };

    let mut visited = HashSet::new();
    let mut stack = vec![(start_idx, Vec::new())];
    let mut result = Vec::new();

    while let Some((node_idx, path)) = stack.pop() {
        if !visited.insert(node_idx) {
            continue;
        }

        if let Some(node) = graph.graph().node_weight(node_idx) {
            // Clone the current path for each outgoing edge
            for edge in graph.graph().edges_directed(node_idx, direction) {
                let mut new_path = path.clone();
                new_path.push(edge.weight());
                stack.push((edge.target(), new_path));
            }

            result.push((node, path));
        }
    }

    result.into_iter()
}

/// Find all paths between two nodes in the graph
pub fn find_all_paths<'a>(
    graph: &'a DependencyGraph,
    from: &str,
    to: &str,
    direction: Direction,
) -> Vec<Vec<(&'a DependencyNode, &'a DependencyEdge)>> {
    let from_idx = match graph.find_node(from).map(|(idx, _)| idx) {
        Some(idx) => idx,
        None => {
            tracing::warn!("Source node not found for path finding: {}", from);
            return Vec::new();
        }
    };

    let to_idx = graph
        .find_node(to)
        .map(|(idx, _)| idx)
        .ok_or_else(|| format!("Node not found: {}", to));

    let to_idx = match to_idx {
        Ok(idx) => idx,
        Err(_) => return Vec::new(),
    };

    let mut paths = Vec::new();
    let mut visited = HashSet::new();
    let mut current_path = Vec::new();

    fn visit<'a>(
        graph: &'a DependencyGraph,
        current: petgraph::graph::NodeIndex,
        target: petgraph::graph::NodeIndex,
        direction: Direction,
        visited: &mut HashSet<petgraph::graph::NodeIndex>,
        current_path: &mut Vec<(&'a DependencyNode, &'a DependencyEdge)>,
        paths: &mut Vec<Vec<(&'a DependencyNode, &'a DependencyEdge)>>,
    ) {
        if current == target {
            paths.push(current_path.clone());
            return;
        }

        if !visited.insert(current) {
            return;
        }

        if let Some(node) = graph.graph().node_weight(current) {
            for edge in graph.graph().edges_directed(current, direction) {
                if let Some(next_node) = graph.graph().node_weight(edge.target()) {
                    current_path.push((next_node, edge.weight()));
                    visit(
                        graph,
                        edge.target(),
                        target,
                        direction,
                        visited,
                        current_path,
                        paths,
                    );
                    current_path.pop();
                }
            }
        }

        visited.remove(&current);
    }

    static DEFAULT_EDGE: DependencyEdge = DependencyEdge {
        dep_type:              crate::dependency::models::DependencyType::Normal,
        version_req:           String::new(),
        optional:              false,
        uses_default_features: true,
        features:              Vec::new(),
        target:                None,
    };

    if let Some(start_node) = graph.graph().node_weight(from_idx) {
        current_path.push((start_node, &DEFAULT_EDGE));
        visit(
            graph,
            from_idx,
            to_idx,
            direction,
            &mut visited,
            &mut current_path,
            &mut paths,
        );
    }

    paths
}

/// Find all dependencies of a node, including transitive dependencies
pub fn find_all_dependencies<'a>(
    graph: &'a DependencyGraph,
    node_name: &str,
    include_types: &[super::DependencyType],
) -> HashMap<String, &'a DependencyNode> {
    let mut result = HashMap::new();

    if let Some((start_idx, _)) = graph.find_node(node_name) {
        let mut dfs = Dfs::new(graph.graph(), start_idx);

        while let Some(node_idx) = dfs.next(graph.graph()) {
            if node_idx == start_idx {
                continue; // Skip the start node
            }

            if let Some(node) = graph.graph().node_weight(node_idx) {
                // Check if any incoming edge matches the included types
                let has_matching_edge = graph
                    .graph()
                    .edges_directed(node_idx, Direction::Incoming)
                    .any(|edge| include_types.is_empty() || include_types.contains(&edge.weight().dep_type));

                if has_matching_edge {
                    result.insert(node.name.clone(), node);
                }
            }
        }
    }

    result
}

/// Find all reverse dependencies (what depends on this node)
pub fn find_reverse_dependencies<'a>(
    graph: &'a DependencyGraph,
    node_name: &str,
) -> HashMap<String, &'a DependencyNode> {
    let mut result = HashMap::new();

    if let Some((target_idx, _)) = graph.find_node(node_name) {
        // We need to find all nodes that have a path to our target
        for node_idx in graph.graph().node_indices() {
            if node_idx == target_idx {
                continue; // Skip the target node itself
            }

            if let Some(node) = graph.graph().node_weight(node_idx) {
                // Check if there's any path from this node to our target
                let mut dfs = Dfs::new(graph.graph(), node_idx);
                if dfs.next(graph.graph()) == Some(target_idx) {
                    result.insert(node.name.clone(), node);
                }
            }
        }
    }

    result
}