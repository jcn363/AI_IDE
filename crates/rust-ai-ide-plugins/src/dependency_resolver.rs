//! Plugin Dependency Resolution System
//!
//! This module provides advanced dependency resolution and conflict detection
//! for plugins, handling version compatibility, cyclic dependencies, and
//! automatic conflict resolution.

use crate::interfaces::EnhancedPluginMetadata;
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Dependency resolution result
#[derive(Debug, Clone)]
pub enum ResolutionResult {
    /// Successfully resolved dependencies
    Resolved(Vec<String>),
    /// Resolution failed with conflicts
    Conflicts(Vec<DependencyConflict>),
    /// Cyclic dependency detected
    Cyclic(Vec<String>),
    /// Missing dependencies found
    Missing(Vec<String>),
}

/// Types of dependency conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyConflict {
    /// Version incompatibility between required versions
    VersionIncompatibility {
        plugin_id: String,
        required_by: Vec<String>,
        conflicting_versions: HashSet<String>,
    },
    /// Direct conflict (plugin A conflicts with plugin B)
    DirectConflict {
        plugin_a: String,
        plugin_b: String,
        reason: String,
    },
    /// Cyclic dependency chain
    CyclicDependency(Vec<String>),
    /// Missing required dependency
    MissingDependency {
        requiring_plugin: String,
        required_plugin: String,
        version_req: VersionReq,
    },
}

/// Plugin dependency graph
#[derive(Debug)]
pub struct DependencyGraph {
    /// Graph representation of plugin dependencies
    graph: DiGraph<String, DependencyEdge>,
    /// Node index to plugin ID mapping
    node_to_plugin: HashMap<NodeIndex, String>,
    /// Plugin ID to node index mapping
    plugin_to_node: HashMap<String, String>,
    /// Plugin metadata cache
    metadata_cache: HashMap<String, EnhancedPluginMetadata>,
}

/// Edge data in dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Version requirement
    pub version_req: VersionReq,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Dependency type
    pub dep_type: DependencyType,
}

/// Types of dependencies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Hard dependency (must be present)
    Hard,
    /// Soft dependency (recommended but not required)
    Soft,
    /// Optional dependency (enhances functionality)
    Optional,
}

/// Plugin dependency resolver
pub struct DependencyResolver {
    /// Available plugins and their metadata
    available_plugins: Arc<RwLock<HashMap<String, EnhancedPluginMetadata>>>,
    /// Resolved dependency cache
    resolution_cache: Arc<RwLock<HashMap<String, ResolutionResult>>>,
    /// Current resolution depth (prevent deep recursion)
    max_resolution_depth: usize,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            available_plugins: Arc::new(RwLock::new(HashMap::new())),
            resolution_cache: Arc::new(RwLock::new(HashMap::new())),
            max_resolution_depth: 50, // Reasonable depth limit
        }
    }

    /// Create resolver with maximum resolution depth
    pub fn with_max_depth(max_depth: usize) -> Self {
        let mut resolver = Self::new();
        resolver.max_resolution_depth = max_depth;
        resolver
    }

    /// Add available plugins to the resolver
    pub async fn add_available_plugins(&self, plugins: Vec<EnhancedPluginMetadata>) {
        let mut available = self.available_plugins.write().await;
        for plugin in plugins {
            available.insert(plugin.core.id.to_string(), plugin);
        }
    }

    /// Resolve dependencies for a plugin
    pub async fn resolve_dependencies(&self, plugin_id: &str) -> ResolutionResult {
        // Check cache first
        if let Some(cached) = self.resolution_cache.read().await.get(plugin_id) {
            return cached.clone();
        }

        let available = self.available_plugins.read().await;
        let Some(plugin_metadata) = available.get(plugin_id) else {
            return ResolutionResult::Missing(vec![plugin_id.to_string()]);
        };

        let mut graph = DependencyGraph::new();
        let mut resolved = HashSet::new();
        let depth = 0;

        match self
            .resolve_plugin_dependencies(
                plugin_metadata,
                &mut graph,
                &mut resolved,
                &available,
                depth,
                &HashSet::new(),
            )
            .await
        {
            Ok(order) => {
                let mut cache = self.resolution_cache.write().await;
                cache.insert(
                    plugin_id.to_string(),
                    ResolutionResult::Resolved(order.clone()),
                );
                ResolutionResult::Resolved(order)
            }
            Err(conflicts) => {
                let mut cache = self.resolution_cache.write().await;
                cache.insert(
                    plugin_id.to_string(),
                    ResolutionResult::Conflicts(conflicts.clone()),
                );
                ResolutionResult::Conflicts(conflicts)
            }
        }
    }

    /// Resolve dependencies for multiple plugins
    pub async fn resolve_multiple(&self, plugin_ids: &[String]) -> ResolutionResult {
        let mut all_resolved = Vec::new();
        let mut all_conflicts = Vec::new();
        let mut all_missing = Vec::new();

        for plugin_id in plugin_ids {
            match self.resolve_dependencies(plugin_id).await {
                ResolutionResult::Resolved(order) => {
                    all_resolved.extend(order);
                }
                ResolutionResult::Conflicts(conflicts) => {
                    all_conflicts.extend(conflicts);
                }
                ResolutionResult::Cyclic(cycle) => {
                    all_conflicts.push(DependencyConflict::CyclicDependency(cycle));
                }
                ResolutionResult::Missing(missing) => {
                    all_missing.extend(missing);
                }
            }
        }

        if !all_conflicts.is_empty() {
            ResolutionResult::Conflicts(all_conflicts)
        } else if !all_missing.is_empty() {
            ResolutionResult::Missing(all_missing)
        } else {
            // Remove duplicates and maintain dependency order
            let mut unique = HashSet::with_capacity(all_resolved.len());
            all_resolved.retain(|id| unique.insert(id.clone()));
            ResolutionResult::Resolved(all_resolved)
        }
    }

    /// Check for conflicts between plugins
    pub async fn check_conflicts(&self, plugin_ids: &[String]) -> Vec<DependencyConflict> {
        let mut conflicts = Vec::new();
        let available = self.available_plugins.read().await;

        // Check direct conflicts first
        for i in 0..plugin_ids.len() {
            let plugin_a_id = &plugin_ids[i];
            if let Some(plugin_a) = available.get(plugin_a_id) {
                for j in (i + 1)..plugin_ids.len() {
                    let plugin_b_id = &plugin_ids[j];
                    if plugin_a.conflicts_with(plugin_b_id) {
                        conflicts.push(DependencyConflict::DirectConflict {
                            plugin_a: plugin_a_id.clone(),
                            plugin_b: plugin_b_id.clone(),
                            reason: "Direct conflict declared in plugin metadata".to_string(),
                        });
                        continue;
                    }

                    // Check reverse conflict
                    if let Some(plugin_b) = available.get(plugin_b_id) {
                        if plugin_b.conflicts_with(plugin_a_id) {
                            conflicts.push(DependencyConflict::DirectConflict {
                                plugin_a: plugin_b_id.clone(),
                                plugin_b: plugin_a_id.clone(),
                                reason: "Reverse conflict declared in plugin metadata".to_string(),
                            });
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// Suggest dependency resolution strategies for conflicts
    pub async fn suggest_resolutions(
        &self,
        conflicts: &[DependencyConflict],
    ) -> HashMap<String, Vec<String>> {
        let mut suggestions = HashMap::new();
        let available = self.available_plugins.read().await;

        for conflict in conflicts {
            match conflict {
                DependencyConflict::VersionIncompatibility {
                    plugin_id,
                    required_by,
                    ..
                } => {
                    let mut strategies = Vec::new();

                    if let Some(plugin) = available.get(plugin_id) {
                        // Suggest updating to compatible versions
                        strategies.push(format!(
                            "Update {} to version {} or later",
                            plugin_id,
                            plugin.versioning.version.to_string()
                        ));

                        // Suggest removing conflicting plugins
                        if !required_by.is_empty() {
                            strategies.push(format!(
                                "Remove one of the requiring plugins: {}",
                                required_by.join(", ")
                            ));
                        }
                    }

                    suggestions.insert(plugin_id.clone(), strategies);
                }
                DependencyConflict::DirectConflict {
                    plugin_a, plugin_b, ..
                } => {
                    suggestions.insert(
                        format!("{}-{}", plugin_a, plugin_b),
                        vec![
                            format!("Remove {} and use alternative functionality", plugin_a),
                            format!("Remove {} and use alternative functionality", plugin_b),
                            "Check for compatibility updates".to_string(),
                        ],
                    );
                }
                DependencyConflict::CyclicDependency(chain) => {
                    suggestions.insert(
                        "cyclic-dependency".to_string(),
                        vec![
                            format!("Break cyclic dependency chain: {}", chain.join(" -> ")),
                            "Refactor plugins to break circular references".to_string(),
                            "Use optional dependencies where possible".to_string(),
                        ],
                    );
                }
                DependencyConflict::MissingDependency {
                    requiring_plugin,
                    required_plugin,
                    ..
                } => {
                    suggestions.insert(
                        format!("{}-missing-{}", requiring_plugin, required_plugin),
                        vec![
                            format!("Install missing dependency: {}", required_plugin),
                            format!(
                                "Find alternative plugins that don't require {}",
                                required_plugin
                            ),
                            format!(
                                "Use a version of {} that has fewer dependencies",
                                required_plugin
                            ),
                        ],
                    );
                }
            }
        }

        suggestions
    }

    /// Internal method to resolve plugin dependencies recursively
    async fn resolve_plugin_dependencies(
        &self,
        plugin: &EnhancedPluginMetadata,
        graph: &mut DependencyGraph,
        resolved: &mut HashSet<String>,
        available: &HashMap<String, EnhancedPluginMetadata>,
        current_depth: usize,
        resolution_path: &HashSet<String>,
    ) -> Result<Vec<String>, Vec<DependencyConflict>> {
        if current_depth >= self.max_resolution_depth {
            return Err(vec![DependencyConflict::VersionIncompatibility {
                plugin_id: plugin.core.id.to_string(),
                required_by: Vec::new(),
                conflicting_versions: HashSet::new(),
            }]);
        }

        if resolved.contains(&plugin.core.id.to_string()) {
            return Ok(vec![]);
        }

        // Check for cyclic dependencies
        if resolution_path.contains(&plugin.core.id.to_string()) {
            let cycle: Vec<String> = resolution_path.iter().cloned().collect();
            return Err(vec![DependencyConflict::CyclicDependency(cycle)]);
        }

        let mut path_with_current = resolution_path.clone();
        path_with_current.insert(plugin.core.id.to_string());

        let mut ordered_dependencies = Vec::new();
        let mut conflicts = Vec::new();

        // Process each dependency
        for dependency in &plugin.dependencies {
            let dep_id = &dependency.plugin_id;

            // Check if dependency is available
            let Some(dep_plugin) = available.get(dep_id) else {
                conflicts.push(DependencyConflict::MissingDependency {
                    requiring_plugin: plugin.core.id.to_string(),
                    required_plugin: dep_id.clone(),
                    version_req: dependency.version_req.clone(),
                });
                continue;
            };

            // Check version compatibility
            if !dependency
                .version_req
                .matches(&dep_plugin.versioning.version)
            {
                conflicts.push(DependencyConflict::VersionIncompatibility {
                    plugin_id: dep_id.clone(),
                    required_by: vec![plugin.core.id.to_string()],
                    conflicting_versions: HashSet::from([
                        dependency.version_req.to_string(),
                        dep_plugin.versioning.version.to_string(),
                    ]),
                });
            }

            // Recursively resolve this dependency
            let depth = current_depth + 1;
            match Box::pin(async {
                self.resolve_plugin_dependencies(
                    dep_plugin,
                    graph,
                    resolved,
                    available,
                    depth,
                    &path_with_current,
                )
                .await
            })
            .await
            {
                Ok(sub_deps) => {
                    ordered_dependencies.extend(sub_deps);
                }
                Err(sub_conflicts) => {
                    conflicts.extend(sub_conflicts);
                }
            }
        }

        if conflicts.is_empty() {
            resolved.insert(plugin.core.id.to_string());
            ordered_dependencies.push(plugin.core.id.to_string());
            Ok(ordered_dependencies)
        } else {
            Err(conflicts)
        }
    }

    /// Clear the resolution cache
    pub async fn clear_cache(&self) {
        self.resolution_cache.write().await.clear();
    }

    /// Get currently available plugins
    pub async fn get_available_plugins(&self) -> Vec<String> {
        self.available_plugins
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_to_plugin: HashMap::new(),
            plugin_to_node: HashMap::new(),
            metadata_cache: HashMap::new(),
        }
    }

    /// Add a plugin to the graph
    pub fn add_plugin(&mut self, plugin_id: String, metadata: EnhancedPluginMetadata) -> NodeIndex {
        let node = self.graph.add_node(plugin_id.clone());
        self.node_to_plugin.insert(node, plugin_id.clone());
        self.plugin_to_node
            .insert(plugin_id.clone(), plugin_id.clone());
        self.metadata_cache.insert(plugin_id, metadata);
        node
    }

    /// Add a dependency between two plugins
    pub fn add_dependency(&mut self, from: &str, to: &str, edge: DependencyEdge) {
        let from_node = self.plugin_to_node.get(from).cloned();
        let to_node = self.plugin_to_node.get(to).cloned();

        if let (Some(from_plugin), Some(to_plugin)) = (from_node, to_node) {
            if let Some(from_idx) = self.get_node_from_plugin(&from_plugin) {
                if let Some(to_idx) = self.get_node_from_plugin(&to_plugin) {
                    self.graph.add_edge(from_idx, to_idx, edge);
                }
            }
        }
    }

    /// Get topological order of plugins (dependencies first)
    pub fn get_topological_order(&self) -> Result<Vec<String>, Vec<String>> {
        match toposort(&self.graph, None) {
            Ok(nodes) => Ok(nodes
                .into_iter()
                .filter_map(|node| self.node_to_plugin.get(&node))
                .cloned()
                .collect()),
            Err(cycle) => {
                let cycle_ids: Vec<String> = self
                    .graph
                    .node_indices()
                    .filter_map(|node| self.node_to_plugin.get(&node))
                    .cloned()
                    .collect();
                Err(cycle_ids)
            }
        }
    }

    /// Get direct dependencies of a plugin
    pub fn get_dependencies(&self, plugin_id: &str) -> Vec<String> {
        if let Some(node) = self.get_node_from_plugin(plugin_id) {
            self.graph
                .neighbors_directed(node, Direction::Outgoing)
                .filter_map(|neighbor| self.node_to_plugin.get(&neighbor))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if the graph has cycles
    pub fn has_cycles(&self) -> bool {
        toposort(&self.graph, None).is_err()
    }

    /// Get plugin ID from node index
    fn get_node_from_plugin(&self, plugin_id: &str) -> Option<NodeIndex> {
        // This is a simplified lookup - in practice you'd want a proper bidirectional map
        self.plugin_to_node.get(plugin_id).and_then(|id| {
            self.node_to_plugin
                .iter()
                .find(|(_, pid)| *pid == id)
                .map(|(node, _)| *node)
        })
    }
}

impl Serialize for DependencyGraph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DependencyGraph", 5)?;

        let nodes: Vec<String> = self.graph.node_weights().cloned().collect();
        state.serialize_field("nodes", &nodes)?;

        let edges: Vec<(usize, usize, DependencyEdge)> = self
            .graph
            .edge_references()
            .map(|e| (e.source().index(), e.target().index(), e.weight().clone()))
            .collect();
        state.serialize_field("edges", &edges)?;

        let node_to_plugin: HashMap<usize, String> = self
            .node_to_plugin
            .iter()
            .map(|(idx, id)| (idx.index(), id.clone()))
            .collect();
        state.serialize_field("node_to_plugin", &node_to_plugin)?;

        state.serialize_field("plugin_to_node", &self.plugin_to_node)?;
        state.serialize_field("metadata_cache", &self.metadata_cache)?;

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
            nodes: Vec<String>,
            edges: Vec<(usize, usize, DependencyEdge)>,
            node_to_plugin: HashMap<usize, String>,
            plugin_to_node: HashMap<String, String>,
            metadata_cache: HashMap<String, EnhancedPluginMetadata>,
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

        let node_to_plugin: HashMap<NodeIndex, String> = raw
            .node_to_plugin
            .into_iter()
            .filter_map(|(idx, id)| {
                if idx < node_indices_vec.len() {
                    Some((node_indices_vec[idx], id))
                } else {
                    None
                }
            })
            .collect();

        Ok(DependencyGraph {
            graph,
            node_to_plugin,
            plugin_to_node: raw.plugin_to_node,
            metadata_cache: raw.metadata_cache,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[tokio::test]
    async fn test_dependency_resolution() {
        let resolver = DependencyResolver::new();

        // Create test plugins with dependencies
        let base_plugin = EnhancedPluginMetadata::builder(
            "base-plugin".to_string(),
            "Base Plugin".to_string(),
            Version::parse("1.0.0").unwrap(),
        )
        .build();

        let dependent_plugin = EnhancedPluginMetadata::builder(
            "dependent-plugin".to_string(),
            "Dependent Plugin".to_string(),
            Version::parse("1.0.0").unwrap(),
        )
        .dependency(PluginDependency {
            plugin_id: "base-plugin".to_string(),
            version_req: VersionReq::parse("^1.0.0").unwrap(),
            optional: false,
            description: None,
            conflict_resolution: ConflictResolution::Auto,
        })
        .build();

        resolver
            .add_available_plugins(vec![base_plugin, dependent_plugin])
            .await;

        let result = resolver.resolve_dependencies("dependent-plugin").await;

        match result {
            ResolutionResult::Resolved(order) => {
                assert_eq!(order.len(), 2);
                assert_eq!(order[0], "base-plugin"); // Dependency should come first
                assert_eq!(order[1], "dependent-plugin");
            }
            _ => panic!("Expected resolved dependencies"),
        }
    }

    #[tokio::test]
    async fn test_missing_dependency() {
        let resolver = DependencyResolver::new();

        let plugin_with_missing_dep = EnhancedPluginMetadata::builder(
            "plugin".to_string(),
            "Test Plugin".to_string(),
            Version::parse("1.0.0").unwrap(),
        )
        .dependency(PluginDependency {
            plugin_id: "missing-plugin".to_string(),
            version_req: VersionReq::parse("*").unwrap(),
            optional: false,
            description: None,
            conflict_resolution: ConflictResolution::Auto,
        })
        .build();

        resolver
            .add_available_plugins(vec![plugin_with_missing_dep])
            .await;

        let result = resolver.resolve_dependencies("plugin").await;

        match result {
            ResolutionResult::Missing(missing) => {
                assert_eq!(missing.len(), 1);
                assert_eq!(missing[0], "missing-plugin");
            }
            _ => panic!("Expected missing dependency error"),
        }
    }
}
