//! Data structures for dependency analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{EdgeRef, NodeRef};
use cargo_metadata::Package;

/// Types of dependencies in the graph
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    #[default]
    /// Regular dependency
    Normal,
    /// Development dependency (dev-dependencies)
    Dev,
    /// Build dependency (build-dependencies)
    Build,
    /// Workspace member
    Workspace,
}

/// Information about a specific version of a dependency
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub registry: Option<String>,
}

/// Information about a dependency's source
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Source {
    #[default]
    /// Unknown source
    Unknown,
    /// From crates.io
    CratesIo,
    /// From a git repository
    Git {
        url: String,
        rev: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
    },
    /// From a local path
    Path(PathBuf),
    /// From a custom registry
    Registry(String),
    /// Workspace member
    Workspace,
}

/// Information about a dependency relationship
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Name of the dependent package
    pub name: String,
    /// Version requirement
    pub version_req: String,
    /// Type of dependency
    pub dep_type: DependencyType,
    /// Whether this is an optional dependency
    pub optional: bool,
    /// Whether default features are used
    pub uses_default_features: bool,
    /// Enabled features
    pub features: Vec<String>,
    /// Target platform (if specified)
    pub target: Option<String>,
}

/// Represents a node in the dependency graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyNode {
    /// Name of the package
    pub name: String,
    /// Current version
    pub version: String,
    /// Latest version available
    pub latest_version: Option<String>,
    /// Description of the package
    pub description: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// License information
    pub license: Option<String>,
    /// Source of the dependency
    pub source: Source,
    /// Whether this is a workspace member
    pub is_workspace_member: bool,
    /// Path to the package (if local)
    pub path: Option<PathBuf>,
    /// Features defined in this package
    pub features: Vec<String>,
    /// Default features
    pub default_features: bool,
    /// Dependencies of this package
    pub dependencies: Vec<DependencyInfo>,
    /// Reverse dependencies (who depends on this package)
    pub reverse_dependencies: Vec<DependencyInfo>,
}

impl DependencyNode {
    /// Check if this is a direct dependency
    pub fn is_direct(&self) -> bool {
        self.dependencies.iter().any(|d| d.dep_type == DependencyType::Normal)
    }

    /// Check if this is a development dependency
    pub fn is_dev(&self) -> bool {
        self.dependencies.iter().all(|d| d.dep_type == DependencyType::Dev)
    }

    /// Check if this is a build dependency
    pub fn is_build(&self) -> bool {
        self.dependencies.iter().all(|d| d.dep_type == DependencyType::Build)
    }

    /// Check if this is a workspace member
    pub fn is_workspace(&self) -> bool {
        self.is_workspace_member
    }

    /// Create a new dependency node from a Cargo package
    pub fn from_package(package: &Package) -> Self {
        let source = if package.source.is_none() {
            if package.manifest_path.starts_with("/") {
                Source::Path(
                    package.manifest_path.parent()
                        .map(|p| p.as_std_path().to_path_buf())
                        .unwrap_or_default()
                )
            } else {
                Source::Workspace
            }
        } else if let Some(src) = &package.source {
            if src.is_crates_io() {
                Source::CratesIo
            } else if let Some(git_ref) = src.repr.strip_prefix("git+") {
                Source::Git {
                    url: git_ref.to_string(),
                    rev: None,
                    branch: None,
                    tag: None,
                }
            } else {
                Source::Unknown
            }
        } else {
            Source::Unknown
        };

        Self {
            name: package.name.to_string(),
            version: package.version.to_string(),
            latest_version: None,
            description: package.description.clone(),
            repository: package.repository.clone().map(|s| s.to_string()),
            license: package.license.clone(),
            source,
            is_workspace_member: false, // Will be updated later
            path: package.manifest_path.parent().map(|p| p.as_std_path().to_path_buf()),
            features: package.features.keys().cloned().collect(),
            default_features: true, // Will be updated from dependencies
            dependencies: Vec::new(),
            reverse_dependencies: Vec::new(),
        }
    }
}

// DependencyEdge is now consolidated in dependency::graph::edge to avoid conflicts

/// Represents a complete dependency graph for a Rust project
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub graph: DiGraph<DependencyNode, DependencyEdge>,
    pub node_indices: HashMap<String, NodeIndex>,
    pub root_package: String,
}

impl Serialize for DependencyGraph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DependencyGraph", 4)?;

        let nodes: Vec<DependencyNode> = self.graph.node_weights().cloned().collect();
        state.serialize_field("nodes", &nodes)?;

        let edges: Vec<(usize, usize, DependencyEdge)> = self.graph.edge_references()
            .map(|e| (e.source().index(), e.target().index(), e.weight().clone()))
            .collect();
        state.serialize_field("edges", &edges)?;

        // Serialize node_indices as HashMap<String, usize> using NodeIndex.index()
        let node_indices_serialized: HashMap<String, usize> = self.node_indices.iter()
            .map(|(k, v)| (k.clone(), v.index()))
            .collect();
        state.serialize_field("node_indices", &node_indices_serialized)?;

        state.serialize_field("root_package", &self.root_package)?;
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
            node_indices: HashMap<String, usize>,
            root_package: String,
        }

        let raw = DependencyGraphRaw::deserialize(deserializer)?;
        let mut graph = DiGraph::new();
        let node_indices_vec: Vec<NodeIndex> = raw.nodes.into_iter().map(|n| graph.add_node(n)).collect();

        for (source, target, weight) in raw.edges {
            if source >= node_indices_vec.len() || target >= node_indices_vec.len() {
                return Err(serde::de::Error::custom(format!("Invalid node index in edges: source={}, target={}, len={}", source, target, node_indices_vec.len())));
            }
            graph.add_edge(node_indices_vec[source], node_indices_vec[target], weight);
        }

        let node_indices: HashMap<String, NodeIndex> = raw.node_indices.into_iter()
            .filter_map(|(k, idx)| {
                if idx < node_indices_vec.len() {
                    Some((k, node_indices_vec[idx]))
                } else {
                    None
                }
            })
            .collect();

        Ok(DependencyGraph {
            graph,
            node_indices,
            root_package: raw.root_package,
        })
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self {
            graph: DiGraph::default(),
            node_indices: HashMap::default(),
            root_package: String::default(),
        }
    }
}

/// A filter for dependency graphs
#[derive(Debug, Clone, Default)]
pub struct DependencyFilter {
    pub include_types: std::collections::HashSet<DependencyType>,
    pub exclude_types: std::collections::HashSet<DependencyType>,
    pub include_pattern: Option<String>,
    pub exclude_pattern: Option<String>,
    pub max_depth: Option<usize>,
    pub direct_only: bool,
    pub workspace_only: bool,
    pub has_updates: Option<bool>,
    pub has_vulnerabilities: Option<bool>,
}

/// Supported export formats for the dependency graph
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportFormat {
    #[default]
    /// Export as DOT format (Graphviz)
    Dot,
    /// JSON format
    Json,
}