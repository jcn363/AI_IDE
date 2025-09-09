//! Graph-based analysis for detecting architectural issues like circular dependencies.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

use petgraph::algo::kosaraju_scc as find_strongly_connected_components;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use proc_macro2::Span;
use syn::spanned::Spanned;

use crate::analysis::CodeLocation;

/// Represents a node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// The name of the module or item
    pub name: String,
    /// The source file path
    pub file_path: String,
    /// The source code span for precise location reporting
    pub span: Span,
}

impl PartialEq for DependencyNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.file_path == other.file_path
    }
}

impl Eq for DependencyNode {}

impl Hash for DependencyNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.file_path.hash(state);
    }
}

impl fmt::Display for DependencyNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.file_path)
    }
}

/// Represents a dependency between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// The kind of dependency (e.g., "uses", "implements", "references")
    pub kind: String,
    /// The source code span for precise location reporting
    pub span: Span,
}

/// A graph representing dependencies between modules and items
pub struct DependencyGraph {
    graph: DiGraph<DependencyNode, DependencyEdge>,
    node_indices: HashMap<String, NodeIndex>,
    node_map: HashMap<NodeIndex, DependencyNode>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add a node to the graph if it doesn't already exist
    pub fn add_node(&mut self, name: &str, file_path: &str, span: Span) -> NodeIndex {
        let key = format!("{}:{}", file_path, name);
        
        if let Some(&idx) = self.node_indices.get(&key) {
            return idx;
        }
        
        let node = DependencyNode {
            name: name.to_string(),
            file_path: file_path.to_string(),
            span,
        };
        
        let idx = self.graph.add_node(node.clone());
        self.node_indices.insert(key, idx);
        self.node_map.insert(idx, node);
        
        idx
    }

    /// Add a dependency edge between two nodes
    pub fn add_dependency(
        &mut self,
        from: &str,
        from_file: &str,
        to: &str,
        to_file: &str,
        kind: &str,
        span: Span,
    ) {
        let from_idx = self.add_node(from, from_file, span);
        let to_idx = self.add_node(to, to_file, span);
        
        // Only add the edge if it doesn't already exist
        if !self.graph.edges_connecting(from_idx, to_idx).any(|e| e.weight().kind == kind) {
            self.graph.add_edge(
                from_idx,
                to_idx,
                DependencyEdge {
                    kind: kind.to_string(),
                    span,
                },
            );
        }
    }

    /// Detect cycles in the dependency graph
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let sccs = find_strongly_connected_components(&self.graph);
        
        sccs.into_iter()
            .filter(|scc| scc.len() > 1) // Only include actual cycles (2+ nodes)
            .map(|scc| {
                scc.into_iter()
                    .filter_map(|idx| self.graph.node_weight(idx).map(|n| n.name.clone()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Find all paths between two nodes
    pub fn find_paths(&self, from: &str, to: &str) -> Vec<Vec<String>> {
        // Implementation of path finding would go here
        // This is a simplified version that just returns an empty vector
        Vec::new()
    }

    /// Get the location of a node
    pub fn get_node_location(&self, node_name: &str) -> Option<CodeLocation> {
        self.node_map.values()
            .find(|n| n.name == node_name)
            .map(|n| CodeLocation::from_span(&n.span))
    }
}

impl Serialize for DependencyGraph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DependencyGraph", 2)?;

        let nodes: Vec<DependencyNode> = self.graph.node_weights().cloned().collect();
        state.serialize_field("nodes", &nodes)?;

        let edges: Vec<(usize, usize, DependencyEdge)> = self.graph
            .edge_references()
            .map(|e| (e.source().index(), e.target().index(), e.weight().clone()))
            .collect();
        state.serialize_field("edges", &edges)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for DependencyGraph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DependencyGraphRaw {
            nodes: Vec<DependencyNode>,
            edges: Vec<(usize, usize, DependencyEdge)>,
        }

        let raw = DependencyGraphRaw::deserialize(deserializer)?;
        let mut graph = DiGraph::new();
        let node_indices_vec: Vec<NodeIndex> = raw.nodes.into_iter().map(|n| graph.add_node(n)).collect();

        for (source, target, weight) in raw.edges {
            if source >= node_indices_vec.len() || target >= node_indices_vec.len() {
                return Err(serde::de::Error::custom(format!(
                    "Invalid node index in edges: source={}, target={}, len={}",
                    source, target, node_indices_vec.len()
                )));
            }
            graph.add_edge(node_indices_vec[source], node_indices_vec[target], weight);
        }

        // Rebuild node_indices and node_map
        let mut node_indices = HashMap::new();
        let mut node_map = HashMap::new();
        for (i, &idx) in node_indices_vec.iter().enumerate() {
            if let Some(node) = graph.node_weight(idx) {
                let key = format!("{}:{}", node.file_path, node.name);
                node_indices.insert(key, idx);
                node_map.insert(idx, node.clone());
            }
        }

        Ok(DependencyGraph {
            graph,
            node_indices,
            node_map,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_add_node() {
        let mut graph = DependencyGraph::new();
        let span = parse_quote!(span);
        
        let idx1 = graph.add_node("module1", "path/to/file1.rs", span);
        let idx2 = graph.add_node("module2", "path/to/file2.rs", span);
        
        // Adding the same node again should return the same index
        let idx1_again = graph.add_node("module1", "path/to/file1.rs", span);
        assert_eq!(idx1, idx1_again);
        
        // Different nodes should have different indices
        assert_ne!(idx1, idx2);
    }

    #[test]
    fn test_detect_cycles() {
        let mut graph = DependencyGraph::new();
        let span = parse_quote!(span);
        
        // A -> B -> C -> A (cycle)
        graph.add_dependency("A", "file1.rs", "B", "file2.rs", "uses", span);
        graph.add_dependency("B", "file2.rs", "C", "file3.rs", "uses", span);
        graph.add_dependency("C", "file3.rs", "A", "file1.rs", "uses", span);
        
        let cycles = graph.detect_cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
        
        // Should contain all nodes in the cycle, order doesn't matter
        let cycle_nodes: HashSet<_> = cycles[0].iter().collect();
        assert!(cycle_nodes.contains(&"A".to_string()));
        assert!(cycle_nodes.contains(&"B".to_string()));
        assert!(cycle_nodes.contains(&"C".to_string()));
    }
}
