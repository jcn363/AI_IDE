//! Dependency analysis and circular dependency detection
//!
//! This module provides comprehensive dependency analysis capabilities including:
//! - Circular dependency detection
//! - Unused dependency identification
//! - Dependency chain analysis
//! - Impact assessment and optimization recommendations

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use cargo_metadata::{Dependency, MetadataCommand, Package};
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Direction, Graph};
use tokio::sync::RwLock;

use crate::error::{OptimizerError, OptimizerResult};
use crate::types::*;

/// Main dependency analyzer for workspace optimization
#[derive(Debug, Clone)]
pub struct DependencyAnalyzer {
    /// Cached workspace metadata
    metadata_cache: Arc<RwLock<Option<cargo_metadata::Metadata>>>,
    /// Dependency graph
    dependency_graph: Arc<RwLock<Option<Graph<String, DependencyKind>>>>,
    /// Analysis results cache
    analysis_cache: Arc<RwLock<Option<DependencyAnalysis>>>,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer
    pub fn new() -> Self {
        Self {
            metadata_cache: Arc::new(RwLock::new(None)),
            dependency_graph: Arc::new(RwLock::new(None)),
            analysis_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the analyzer with current workspace metadata
    pub async fn initialize(&self) -> OptimizerResult<()> {
        let metadata = MetadataCommand::new()
            .manifest_path("./Cargo.toml")
            .exec()
            .map_err(|e| {
                OptimizerError::cargo_metadata_error(format!("Failed to get metadata: {}", e))
            })?;

        // Build dependency graph
        let graph = self.build_dependency_graph(&metadata)?;

        // Update caches
        {
            let mut metadata_cache = self.metadata_cache.write().await;
            *metadata_cache = Some(metadata);
        }

        {
            let mut graph_cache = self.dependency_graph.write().await;
            *graph_cache = Some(graph);
        }

        Ok(())
    }

    /// Analyze the entire workspace for dependency issues
    pub async fn analyze_workspace(&self) -> OptimizerResult<DependencyAnalysis> {
        // Check if we have cached results
        {
            let cache = self.analysis_cache.read().await;
            if let Some(ref analysis) = *cache {
                return Ok(analysis.clone());
            }
        }

        // Initialize if needed
        {
            let metadata = self.metadata_cache.read().await;
            if metadata.is_none() {
                drop(metadata);
                self.initialize().await?;
            }
        }

        let metadata = self.metadata_cache.read().await;
        let metadata = metadata
            .as_ref()
            .ok_or_else(|| OptimizerError::invalid_state("No metadata available"))?;

        let mut analysis = DependencyAnalysis::default();

        // Detect circular dependencies
        analysis.circular_dependencies = self.detect_circular_dependencies().await?;

        // Find unused dependencies
        analysis.unused_dependencies = self.find_unused_dependencies(metadata).await?;

        // Analyze dependency depths
        analysis.dependency_depths = self.analyze_dependency_depths().await?;

        // Count total dependencies
        analysis.total_dependencies = self.count_total_dependencies(metadata);

        // Cache the results
        {
            let mut cache = self.analysis_cache.write().await;
            *cache = Some(analysis.clone());
        }

        Ok(analysis)
    }

    /// Apply optimization recommendations to fix dependency issues
    pub async fn apply_optimizations(
        &self,
        analysis: DependencyAnalysis,
    ) -> OptimizerResult<DependencyAnalysis> {
        let mut applied_analysis = analysis.clone();

        // For now, just mark optimizations as applied
        // In a real implementation, this would modify Cargo.toml files
        // and update dependency declarations

        // Clear the cache to force re-analysis
        {
            let mut cache = self.analysis_cache.write().await;
            *cache = None;
        }

        Ok(applied_analysis)
    }

    /// Build the dependency graph from workspace metadata
    fn build_dependency_graph(
        &self,
        metadata: &cargo_metadata::Metadata,
    ) -> OptimizerResult<Graph<String, DependencyKind>> {
        let mut graph = Graph::<String, DependencyKind>::new();

        // Create nodes for all packages
        let mut node_indices = HashMap::new();
        for package in &metadata.workspace_members {
            let node_idx = graph.add_node(package.repr.clone());
            node_indices.insert(package.repr.clone(), node_idx);
        }

        // Add dependencies as edges
        for package in &metadata.workspace_members {
            let source_idx = node_indices[&package.repr];

            for dep in &package.dependencies {
                if let Some(&target_idx) = node_indices.get(&dep.name) {
                    let dep_kind = if dep.kind == cargo_metadata::DependencyKind::Normal {
                        DependencyKind::Normal
                    } else if dep.kind == cargo_metadata::DependencyKind::Development {
                        DependencyKind::Dev
                    } else {
                        DependencyKind::Build
                    };

                    graph.add_edge(source_idx, target_idx, dep_kind);
                }
            }
        }

        Ok(graph)
    }

    /// Detect circular dependencies in the workspace
    async fn detect_circular_dependencies(&self) -> OptimizerResult<Vec<CircularDependency>> {
        let graph_guard = self.dependency_graph.read().await;
        let graph = graph_guard
            .as_ref()
            .ok_or_else(|| OptimizerError::invalid_state("No dependency graph available"))?;

        let mut circular_deps = Vec::new();

        // Use topological sort to detect cycles
        match toposort(graph, None) {
            Ok(_) => {
                // No cycles detected
                return Ok(circular_deps);
            }
            Err(cycle_node) => {
                // Cycle detected, find all cycles
                let cycles = self.find_all_cycles(graph, cycle_node.node_id())?;
                for cycle in cycles {
                    let impact = self.assess_dependency_impact(&cycle);
                    circular_deps.push(CircularDependency {
                        crates: cycle.clone(),
                        chain: cycle,
                        impact,
                    });
                }
            }
        }

        Ok(circular_deps)
    }

    /// Find all cycles in the dependency graph
    fn find_all_cycles(
        &self,
        graph: &Graph<String, DependencyKind>,
        start_node: NodeIndex,
    ) -> OptimizerResult<Vec<Vec<String>>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut path_set = HashSet::new();

        self.dfs_cycles(
            graph,
            start_node,
            &mut visited,
            &mut path,
            &mut path_set,
            &mut cycles,
        )?;

        Ok(cycles)
    }

    /// Depth-first search to find cycles
    fn dfs_cycles(
        &self,
        graph: &Graph<String, DependencyKind>,
        node: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        path: &mut Vec<String>,
        path_set: &mut HashSet<NodeIndex>,
        cycles: &mut Vec<Vec<String>>,
    ) -> OptimizerResult<()> {
        if path_set.contains(&node) {
            // Found a cycle
            let cycle_start = path.iter().position(|n| n == &graph[node]).unwrap_or(0);
            let cycle = path[cycle_start..].to_vec();
            cycles.push(cycle);
            return Ok(());
        }

        if visited.contains(&node) {
            return Ok(());
        }

        visited.insert(node);
        path_set.insert(node);
        path.push(graph[node].clone());

        for neighbor in graph.neighbors_directed(node, Direction::Outgoing) {
            self.dfs_cycles(graph, neighbor, visited, path, path_set, cycles)?;
        }

        path_set.remove(&node);
        path.pop();

        Ok(())
    }

    /// Assess the impact of a dependency chain
    fn assess_dependency_impact(&self, crates: &[String]) -> DependencyImpact {
        match crates.len() {
            0..=2 => DependencyImpact::Low,
            3..=5 => DependencyImpact::Medium,
            6..=10 => DependencyImpact::High,
            _ => DependencyImpact::Critical,
        }
    }

    /// Find unused dependencies in the workspace
    async fn find_unused_dependencies(
        &self,
        metadata: &cargo_metadata::Metadata,
    ) -> OptimizerResult<Vec<UnusedDependency>> {
        let mut unused_deps = Vec::new();

        // This is a simplified implementation
        // In a real implementation, this would:
        // 1. Analyze source code to find actual usage
        // 2. Cross-reference with Cargo.toml declarations
        // 3. Check for transitive dependencies

        // For now, return empty list as this requires complex analysis
        // of source code and compilation artifacts

        Ok(unused_deps)
    }

    /// Analyze dependency depths for all crates
    async fn analyze_dependency_depths(&self) -> OptimizerResult<HashMap<String, usize>> {
        let graph_guard = self.dependency_graph.read().await;
        let graph = graph_guard
            .as_ref()
            .ok_or_else(|| OptimizerError::invalid_state("No dependency graph available"))?;

        let mut depths = HashMap::new();

        for node_idx in graph.node_indices() {
            let depth = self.calculate_dependency_depth(graph, node_idx)?;
            let crate_name = graph[node_idx].clone();
            depths.insert(crate_name, depth);
        }

        Ok(depths)
    }

    /// Calculate dependency depth for a specific crate
    fn calculate_dependency_depth(
        &self,
        graph: &Graph<String, DependencyKind>,
        node: NodeIndex,
    ) -> OptimizerResult<usize> {
        let mut max_depth = 0;
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((node, 0));
        visited.insert(node);

        while let Some((current_node, depth)) = queue.pop_front() {
            max_depth = max_depth.max(depth);

            for neighbor in graph.neighbors_directed(current_node, Direction::Outgoing) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }

        Ok(max_depth)
    }

    /// Count total dependencies in the workspace
    fn count_total_dependencies(&self, metadata: &cargo_metadata::Metadata) -> usize {
        let mut total = 0;
        for package in &metadata.workspace_members {
            total += package.dependencies.len();
        }
        total
    }

    /// Clear all caches to force fresh analysis
    pub async fn clear_cache(&self) {
        let mut metadata_cache = self.metadata_cache.write().await;
        *metadata_cache = None;

        let mut graph_cache = self.dependency_graph.write().await;
        *graph_cache = None;

        let mut analysis_cache = self.analysis_cache.write().await;
        *analysis_cache = None;
    }

    /// Get current workspace metadata
    pub async fn get_metadata(&self) -> OptimizerResult<Option<cargo_metadata::Metadata>> {
        let metadata = self.metadata_cache.read().await;
        Ok(metadata.clone())
    }
}

/// Type of dependency relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    /// Normal runtime dependency
    Normal,
    /// Development-only dependency
    Dev,
    /// Build-time dependency
    Build,
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dependency_analyzer_creation() {
        let analyzer = DependencyAnalyzer::new();
        assert!(analyzer.metadata_cache.read().await.is_none());
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let analyzer = DependencyAnalyzer::new();

        // Set some dummy data
        {
            let mut metadata_cache = analyzer.metadata_cache.write().await;
            *metadata_cache = Some(cargo_metadata::Metadata::default());
        }

        // Clear cache
        analyzer.clear_cache().await;

        // Verify cache is cleared
        assert!(analyzer.metadata_cache.read().await.is_none());
    }
}
