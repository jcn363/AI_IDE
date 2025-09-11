//! Serialization support for dependency graphs

use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io;
use std::path::Path;
use tokio::task;

use crate::error::*;
use crate::graph::*;

/// Supported serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    Yaml,
    Toml,
    Msgpack,
}

impl SerializationFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "msgpack" => Some(Self::Msgpack),
            _ => None,
        }
    }

    pub fn get_extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Msgpack => "msgpack",
        }
    }
}

/// Serializable representation of the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGraph {
    pub nodes: Vec<SerializableNode>,
    pub edges: Vec<SerializableEdge>,
    pub root_package: Option<String>,
    pub workspace_members: Vec<String>,
    pub metadata: GraphMetadata,
}

/// Metadata about the serialized graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_packages: usize,
    pub total_dependencies: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub format_version: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableNode {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub authors: Vec<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub readme: Option<String>,
    pub is_workspace_member: bool,
    pub source_url: Option<String>,
    pub checksum: Option<String>,
    pub yanked: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEdge {
    pub source_node: String,
    pub target_node: String,
    pub dep_type: String,
    pub version_constraint: Option<String>,
    pub features_requested: Vec<String>,
    pub features_enabled: Vec<String>,
    pub optional: bool,
    pub req_depth: usize,
}

/// Graph serializer with multiple format support
pub struct GraphSerializer {
    format: SerializationFormat,
}

impl GraphSerializer {
    pub fn new(format: SerializationFormat) -> Self {
        Self { format }
    }

    pub fn from_extension(extension: &str) -> DependencyResult<Self> {
        let format = SerializationFormat::from_extension(extension).ok_or_else(|| {
            DependencyError::ParseError(format!("Unsupported file extension: {}", extension))
        })?;
        Ok(Self::new(format))
    }

    pub async fn serialize_to_file(
        &self,
        graph: &DependencyGraph,
        path: &Path,
    ) -> DependencyResult<()> {
        let serializable = self.convert_to_serializable(graph).await?;
        let data = self.serialize_data(&serializable)?;
        self.write_to_file(path, &data).await?;
        Ok(())
    }

    pub async fn deserialize_from_file(&self, path: &Path) -> DependencyResult<DependencyGraph> {
        let data = self.read_from_file(path).await?;
        let serializable: SerializableGraph = self.deserialize_data(&data)?;
        let graph = self.convert_from_serializable(serializable)?;
        Ok(graph)
    }

    pub async fn serialize_to_string(&self, graph: &DependencyGraph) -> DependencyResult<String> {
        let serializable = self.convert_to_serializable(graph).await?;
        self.serialize_data(&serializable)
    }

    pub async fn deserialize_from_string(&self, data: &str) -> DependencyResult<DependencyGraph> {
        let serializable: SerializableGraph = self.deserialize_data(data)?;
        let graph = self.convert_from_serializable(serializable)?;
        Ok(graph)
    }

    fn serialize_data(&self, data: &SerializableGraph) -> DependencyResult<String> {
        match self.format {
            SerializationFormat::Json => serde_json::to_string_pretty(data).map_err(|e| {
                DependencyError::ParseError(format!("JSON serialization error: {}", e))
            }),
            SerializationFormat::Yaml => serde_yaml::to_string(data).map_err(|e| {
                DependencyError::ParseError(format!("YAML serialization error: {}", e))
            }),
            SerializationFormat::Toml => Ok(toml::to_string(data).map_err(|e| {
                DependencyError::ParseError(format!("TOML serialization error: {}", e))
            })?),
            SerializationFormat::Msgpack => Err(DependencyError::ParseError(
                "Msgpack serialization not implemented".to_string(),
            )),
        }
    }

    fn deserialize_data<'a, T: serde::Deserialize<'a>>(
        &self,
        data: &'a str,
    ) -> DependencyResult<T> {
        match self.format {
            SerializationFormat::Json => serde_json::from_str(data).map_err(|e| {
                DependencyError::ParseError(format!("JSON deserialization error: {}", e))
            }),
            SerializationFormat::Yaml => serde_yaml::from_str(data).map_err(|e| {
                DependencyError::ParseError(format!("YAML deserialization error: {}", e))
            }),
            SerializationFormat::Toml => toml::from_str(data).map_err(|e| {
                DependencyError::ParseError(format!("TOML deserialization error: {}", e))
            }),
            SerializationFormat::Msgpack => Err(DependencyError::ParseError(
                "Msgpack deserialization not implemented".to_string(),
            )),
        }
    }

    async fn convert_to_serializable(
        &self,
        graph: &DependencyGraph,
    ) -> DependencyResult<SerializableGraph> {
        let nodes: Vec<SerializableNode> = graph
            .get_all_packages()
            .iter()
            .map(|node| SerializableNode {
                name: node.name.clone(),
                version: node.version.clone(),
                description: node.description.clone(),
                repository: node.repository.clone(),
                license: node.license.clone(),
                authors: node.authors.clone(),
                keywords: node.keywords.clone(),
                categories: node.categories.clone(),
                homepage: node.homepage.clone(),
                documentation: node.documentation.clone(),
                readme: node.readme.clone(),
                is_workspace_member: node.is_workspace_member,
                source_url: node.source_url.clone(),
                checksum: node.checksum.clone(),
                yanked: node.yanked,
                created_at: node.created_at.clone(),
            })
            .collect();

        let mut edges = Vec::new();
        for (source_name, source_idx) in &graph.node_indices {
            for edge_ref in graph
                .graph
                .edges_directed(*source_idx, petgraph::Direction::Outgoing)
            {
                if let Some(source_node) = graph.graph.node_weight(edge_ref.source()) {
                    if let Some(target_node) = graph.graph.node_weight(edge_ref.target()) {
                        let edge = SerializableEdge {
                            source_node: source_node.name.clone(),
                            target_node: target_node.name.clone(),
                            dep_type: format!("{:?}", edge_ref.weight().dep_type),
                            version_constraint: edge_ref.weight().version_constraint.clone(),
                            features_requested: edge_ref.weight().features_requested.clone(),
                            features_enabled: edge_ref.weight().features_enabled.clone(),
                            optional: edge_ref.weight().optional,
                            req_depth: edge_ref.weight().req_depth,
                        };
                        edges.push(edge);
                    }
                }
            }
        }

        let metadata = GraphMetadata {
            total_packages: nodes.len(),
            total_dependencies: edges.len(),
            created_at: chrono::Utc::now(),
            format_version: "1.0".to_string(),
            source: "rust-ai-ide-dependency-graph".to_string(),
        };

        Ok(SerializableGraph {
            nodes,
            edges,
            root_package: graph.root_package.clone(),
            workspace_members: graph.workspace_members.iter().cloned().collect(),
            metadata,
        })
    }

    fn convert_from_serializable(
        &self,
        serializable: SerializableGraph,
    ) -> DependencyResult<DependencyGraph> {
        let mut graph = DependencyGraph::new();

        // Add nodes
        for node_data in serializable.nodes {
            let node = DependencyNode {
                name: node_data.name,
                version: node_data.version,
                description: node_data.description,
                repository: node_data.repository,
                license: node_data.license,
                authors: node_data.authors,
                keywords: node_data.keywords,
                categories: node_data.categories,
                homepage: node_data.homepage,
                documentation: node_data.documentation,
                readme: node_data.readme,
                is_workspace_member: node_data.is_workspace_member,
                source_url: node_data.source_url,
                checksum: node_data.checksum,
                yanked: node_data.yanked,
                created_at: node_data.created_at,
            };

            graph.add_package(node)?;
        }

        // Add edges
        for edge_data in serializable.edges {
            if let Some(&source_idx) = graph.node_indices.get(&edge_data.source_node) {
                if let Some(&target_idx) = graph.node_indices.get(&edge_data.target_node) {
                    let dep_type = match edge_data.dep_type.as_str() {
                        "Normal" => DependencyType::Normal,
                        "Dev" => DependencyType::Dev,
                        "Build" => DependencyType::Build,
                        "Workspace" => DependencyType::Workspace,
                        "Optional" => DependencyType::Optional,
                        _ => DependencyType::Normal,
                    };

                    let edge = DependencyEdge {
                        dep_type,
                        source_name: edge_data.source_node,
                        target_name: edge_data.target_node,
                        version_constraint: edge_data.version_constraint,
                        features_requested: edge_data.features_requested,
                        features_enabled: edge_data.features_enabled,
                        optional: edge_data.optional,
                        req_depth: edge_data.req_depth,
                    };

                    graph.graph.add_edge(source_idx, target_idx, edge);
                }
            }
        }

        // Set root package
        if let Some(root) = serializable.root_package {
            graph.set_root_package(root)?;
        }

        // Set workspace members
        graph.workspace_members = serializable.workspace_members.into_iter().collect();

        Ok(graph)
    }

    async fn write_to_file(&self, path: &Path, data: &str) -> DependencyResult<()> {
        task::spawn_blocking(move || {
            fs::write(path, data)
                .map_err(|e| DependencyError::IoError(format!("Failed to write file: {}", e)))
        })
        .await
        .map_err(|e| DependencyError::IoError(format!("Task join error: {}", e)))?
    }

    async fn read_from_file(&self, path: &Path) -> DependencyResult<String> {
        task::spawn_blocking(move || {
            fs::read_to_string(path)
                .map_err(|e| DependencyError::IoError(format!("Failed to read file: {}", e)))
        })
        .await
        .map_err(|e| DependencyError::IoError(format!("Task join error: {}", e)))?
    }
}

/// Graph export functionality with automatic format detection
pub struct GraphExporter;

impl GraphExporter {
    pub async fn export_to_file(graph: &DependencyGraph, path: &Path) -> DependencyResult<()> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        let serializer = GraphSerializer::from_extension(extension)?;
        serializer.serialize_to_file(graph, path).await
    }

    pub async fn export_to_string(
        graph: &DependencyGraph,
        format: SerializationFormat,
    ) -> DependencyResult<String> {
        let serializer = GraphSerializer::new(format);
        serializer.serialize_to_string(graph).await
    }
}

/// Graph import functionality
pub struct GraphImporter;

impl GraphImporter {
    pub async fn import_from_file(path: &Path) -> DependencyResult<DependencyGraph> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        let serializer = GraphSerializer::from_extension(extension)?;
        serializer.deserialize_from_file(path).await
    }

    pub async fn import_from_string(
        data: &str,
        format: SerializationFormat,
    ) -> DependencyResult<DependencyGraph> {
        let serializer = GraphSerializer::new(format);
        serializer.deserialize_from_string(data).await
    }
}
