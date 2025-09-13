//! Serialization logic for dependency graphs

use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use serde_json;

use super::models::*;

/// Supported export formats for the dependency graph
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportFormat {
    #[default]
    /// Export as DOT format (Graphviz)
    Dot,
    /// JSON format
    Json,
}

/// Export a dependency graph to the specified format
pub fn export_graph(graph: &DependencyGraph, format: ExportFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match format {
        ExportFormat::Dot => export_dot(graph),
        ExportFormat::Json => export_json(graph),
    }
}

fn export_dot(graph: &DependencyGraph) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    output.extend_from_slice(b"digraph dependencies {\n  node [shape=box, style=rounded];\n\n");

    // Add nodes
    for node_idx in graph.graph.node_indices() {
        if let Some(node) = graph.graph.node_weight(node_idx) {
            let label = format!("{}@{}", node.name, node.version);
            let shape = if node.is_workspace() {
                "doubleoctagon"
            } else {
                "box"
            };
            let style = if node.is_workspace() {
                "filled"
            } else {
                "rounded"
            };
            let color = if node.is_workspace() {
                "#d4f1f9"
            } else {
                "#ffffff"
            };

            output.extend_from_slice(
                format!(
                    "  \"{}:{}\" [label=\"{}\" shape={} style=\"{} \" fillcolor=\"{}\"]\n",
                    node.name, node.version, label, shape, style, color
                )
                .as_bytes(),
            );
        }
    }

    // Add edges
    for edge in graph.graph.edge_references() {
        if let (Some(source), Some(target)) = (
            graph.graph.node_weight(edge.source()),
            graph.graph.node_weight(edge.target()),
        ) {
            let style = match edge.weight().dep_type {
                DependencyType::Dev => "style=dashed",
                DependencyType::Build => "style=dotted",
                DependencyType::Workspace => "style=bold",
                _ => "",
            };

            output.extend_from_slice(
                format!(
                    "  \"{}:{}\" -> \"{}:{}\" {}\n",
                    source.name, source.version, target.name, target.version, style
                )
                .as_bytes(),
            );
        }
    }

    output.extend_from_slice(b"}\n");
    Ok(output)
}

fn export_json(graph: &DependencyGraph) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    #[derive(Serialize)]
    struct ExportNode {
        id:           String,
        name:         String,
        version:      String,
        is_workspace: bool,
    }

    #[derive(Serialize)]
    struct ExportEdge {
        source: String,
        target: String,
        r#type: &'static str,
    }

    let mut export = (Vec::new(), Vec::new());

    // Add nodes
    for node_idx in graph.graph.node_indices() {
        if let Some(node) = graph.graph.node_weight(node_idx) {
            export.0.push(ExportNode {
                id:           format!("{}:{}", node.name, node.version),
                name:         node.name.clone(),
                version:      node.version.clone(),
                is_workspace: node.is_workspace(),
            });
        }
    }

    // Add edges
    for edge in graph.graph.edge_references() {
        if let (Some(source), Some(target)) = (
            graph.graph.node_weight(edge.source()),
            graph.graph.node_weight(edge.target()),
        ) {
            let edge_type = match edge.weight().dep_type {
                DependencyType::Normal => "normal",
                DependencyType::Dev => "dev",
                DependencyType::Build => "build",
                DependencyType::Workspace => "workspace",
            };

            export.1.push(ExportEdge {
                source: format!("{}:{}", source.name, source.version),
                target: format!("{}:{}", target.name, target.version),
                r#type: edge_type,
            });
        }
    }

    Ok(serde_json::to_vec_pretty(&export)?)
}

pub use self::{export_graph, ExportFormat};
