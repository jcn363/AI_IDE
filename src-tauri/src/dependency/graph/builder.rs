use super::*;
use cargo_metadata::{DependencyKind, Metadata, Package, PackageId};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Builder for constructing a dependency graph from Cargo metadata
#[derive(Debug)]
pub struct DependencyGraphBuilder {
    metadata: Metadata,
    root_package: String,
}

impl DependencyGraphBuilder {
    /// Create a new builder for the project at the given path
    pub fn new(project_path: &Path) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(project_path.join("Cargo.toml"))
            .exec()?;

        let root_package = metadata
            .root_package()
            .ok_or_else(|| anyhow::anyhow!("No root package found"))?
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

        // First pass: add all packages as nodes
        for package in &self.metadata.packages {
            let node = DependencyNode::from_package(package);
            let idx = graph.add_node(node);
            node_indices.insert(package.id.repr.clone(), idx);
        }

        // Second pass: add edges between dependencies
        for package in &self.metadata.packages {
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
        let deps = self
            .metadata
            .resolve
            .as_ref()
            .and_then(|r| r.nodes.iter().find(|n| n.id == package.id))
            .map(|n| &n.dependencies)
            .map_or::<&[cargo_metadata::PackageId], _>(&[], |v| v.as_slice());

        for dep_id in deps {
            if let Some(&target_idx) = node_indices.get(&dep_id.repr) {
                let dep_pkg = self
                    .metadata
                    .packages
                    .iter()
                    .find(|p| &p.id == dep_id)
                    .ok_or_else(|| anyhow::anyhow!("Dependency not found: {}", dep_id))?;

                let dep_info = package
                    .dependencies
                    .iter()
                    .find(|d| d.name == dep_pkg.name.to_string())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Dependency info not found: {}", dep_pkg.name)
                    })?;

                let edge = DependencyEdge {
                    dep_type: if dep_info.kind == DependencyKind::Development {
                        DependencyType::Dev
                    } else if dep_info.kind == DependencyKind::Build {
                        DependencyType::Build
                    } else {
                        DependencyType::Normal
                    },
                    version_req: dep_info.req.to_string(),
                    optional: dep_info.optional,
                    uses_default_features: dep_info.uses_default_features,
                    features: dep_info.features.clone(),
                    target: dep_info.target.clone().map(|t| t.to_string()),
                };

                graph.add_edge(source_idx, target_idx, edge);
            }
        }

        Ok(())
    }
}
