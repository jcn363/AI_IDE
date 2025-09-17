//! Workspace-aware dependency management

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::*;
use crate::graph::*;
use crate::resolver::*;

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub root_path: PathBuf,
    pub member_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub package_json_path: Option<PathBuf>,
    pub cargo_toml_path: Option<PathBuf>,
    pub lockfile_paths: Vec<PathBuf>,
    pub override_paths: HashMap<String, PathBuf>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            member_patterns: vec!["*".to_string()],
            exclude_patterns: vec!["vendor".to_string(), "target".to_string()],
            package_json_path: Some(PathBuf::from("package.json")),
            cargo_toml_path: Some(PathBuf::from("Cargo.toml")),
            lockfile_paths: vec![
                PathBuf::from("Cargo.lock"),
                PathBuf::from("package-lock.json"),
                PathBuf::from("yarn.lock"),
                PathBuf::from("pnpm-lock.yaml"),
            ],
            override_paths: HashMap::new(),
        }
    }
}

/// Workspace member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest_type: ManifestType,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub status: PublicationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum ManifestType {
    Cargo,
    Npm,
    Yarn,
    Pnpm,
    Poetry,
    Pip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublicationStatus {
    Published { version: String },
    Unpublished,
    Private,
}

/// Workspace dependency resolver with awareness of member relationships
pub struct WorkspaceResolver {
    config: WorkspaceConfig,
    members: HashMap<String, WorkspaceMember>,
    dependency_graph: Arc<RwLock<DependencyGraph>>,
}

impl WorkspaceResolver {
    pub fn new(config: WorkspaceConfig, dependency_graph: Arc<RwLock<DependencyGraph>>) -> Self {
        Self {
            config,
            members: HashMap::new(),
            dependency_graph,
        }
    }

    /// Discover workspace members from filesystem
    pub async fn discover_members(&mut self) -> DependencyResult<()> {
        self.members.clear();

        if let Some(cargo_path) = &self.config.cargo_toml_path {
            if cargo_path.exists() {
                self.discover_cargo_members().await?;
            }
        }

        if let Some(package_path) = &self.config.package_json_path {
            if package_path.exists() {
                self.discover_npm_members().await?;
            }
        }

        // Update dependency graph with workspace members
        let mut graph = self.dependency_graph.write().await;
        let member_names: HashSet<String> = self.members.keys().cloned().collect();
        graph.workspace_members = member_names;

        Ok(())
    }

    async fn discover_cargo_members(&mut self) -> DependencyResult<()> {
        // In a real implementation, this would parse Cargo.toml files
        // and discover workspace members according to Cargo's workspace format

        // Placeholder: Add dummy workspace members
        let dummy_member = WorkspaceMember {
            name: "rust-ai-ide-dependency-graph".to_string(),
            path: PathBuf::from("crates/rust-ai-ide-dependency-graph"),
            manifest_path: PathBuf::from("crates/rust-ai-ide-dependency-graph/Cargo.toml"),
            manifest_type: ManifestType::Cargo,
            dependencies: vec!["petgraph".to_string(), "semver".to_string()],
            dependents: Vec::new(),
            status: PublicationStatus::Unpublished,
        };

        self.members.insert(dummy_member.name.clone(), dummy_member);
        Ok(())
    }

    async fn discover_npm_members(&mut self) -> DependencyResult<()> {
        // Placeholder for npm workspace discovery
        Ok(())
    }

    /// Resolve workspace-internal dependencies
    pub async fn resolve_workspace_dependencies(
        &self,
    ) -> DependencyResult<WorkspaceResolutionResult> {
        let mut internal_deps = HashMap::new();
        let mut external_deps = HashMap::new();
        let mut version_conflicts = Vec::new();

        let graph = self.dependency_graph.read().await;

        for (package_name, member) in &self.members {
            for dep in &member.dependencies {
                if self.members.contains_key(dep) {
                    // Internal workspace dependency
                    internal_deps
                        .entry(package_name.clone())
                        .or_insert_with(Vec::new)
                        .push(dep.clone());
                } else {
                    // External dependency
                    external_deps
                        .entry(package_name.clone())
                        .or_insert_with(Vec::new)
                        .push(dep.clone());
                }
            }
        }

        // Check for version conflicts in internal dependencies
        let conflicts = self.analyze_version_conflicts(&internal_deps)?;
        version_conflicts.extend(conflicts);

        let resolution_suggestions = self
            .generate_resolution_suggestions(&version_conflicts)
            .await?;

        Ok(WorkspaceResolutionResult {
            internal_dependencies: internal_deps,
            external_dependencies: external_deps,
            version_conflicts,
            resolution_suggestions,
        })
    }

    fn analyze_version_conflicts(
        &self,
        internal_deps: &HashMap<String, Vec<String>>,
    ) -> DependencyResult<Vec<VersionConflict>> {
        let mut conflicts = Vec::new();
        let mut dep_usage = HashMap::new();

        // Count how many times each dependency is used across workspace
        for deps in internal_deps.values() {
            for dep in deps {
                *dep_usage.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        // Find dependencies used in multiple packages
        for (dep, usage_count) in dep_usage {
            if usage_count > 1 {
                let mut packages_using = Vec::new();
                for (pkg, deps) in internal_deps {
                    if deps.contains(&dep) {
                        packages_using.push(pkg.clone());
                    }
                }

                if packages_using.len() > 1 {
                    conflicts.push(VersionConflict {
                        package: dep,
                        constraints: packages_using
                            .iter()
                            .enumerate()
                            .map(|(i, pkg)| {
                                PackageConstraint {
                                    source_package: pkg.clone(),
                                    version_req: "*".to_string(), // Placeholder
                                    depth: i,
                                }
                            })
                            .collect(),
                    });
                }
            }
        }

        Ok(conflicts)
    }

    async fn generate_resolution_suggestions(
        &self,
        conflicts: &[VersionConflict],
    ) -> DependencyResult<Vec<String>> {
        let mut suggestions = Vec::new();

        for conflict in conflicts {
            suggestions.push(format!(
                "Consider creating a workspace-shared dependency for {}",
                conflict.package
            ));
        }

        Ok(suggestions)
    }

    /// Sync workspace member metadata with dependency graph
    pub async fn sync_with_dependency_graph(&self) -> DependencyResult<()> {
        let mut graph = self.dependency_graph.write().await;

        // Add workspace members as nodes if not present
        for (name, member) in &self.members {
            if !graph.node_indices.contains_key(name) {
                let node = DependencyNode {
                    name: name.clone(),
                    version: None, // Would be populated from manifest
                    description: None,
                    repository: None,
                    license: None,
                    authors: Vec::new(),
                    keywords: Vec::new(),
                    categories: Vec::new(),
                    homepage: None,
                    documentation: None,
                    readme: None,
                    is_workspace_member: true,
                    source_url: None,
                    checksum: None,
                    yanked: false,
                    created_at: None,
                };

                graph.add_package(node)?;
            }
        }

        // Update workspace members set
        graph.workspace_members = self.members.keys().cloned().collect();

        Ok(())
    }

    /// Get workspace dependency paths
    pub fn get_workspace_paths(&self) -> Vec<PathBuf> {
        self.members
            .values()
            .map(|member| member.path.clone())
            .collect()
    }

    /// Check if a package is a workspace member
    pub fn is_workspace_member(&self, package_name: &str) -> bool {
        self.members.contains_key(package_name)
    }

    /// Get workspace member information
    pub fn get_workspace_member(&self, package_name: &str) -> Option<&WorkspaceMember> {
        self.members.get(package_name)
    }

    /// Get all workspace members
    pub fn get_all_members(&self) -> Vec<&WorkspaceMember> {
        self.members.values().collect()
    }

    /// Update workspace configuration
    pub async fn update_config(&mut self, new_config: WorkspaceConfig) -> DependencyResult<()> {
        self.config = new_config;
        // Re-discover members with new configuration
        self.discover_members().await?;
        self.sync_with_dependency_graph().await?;
        Ok(())
    }

    /// Get workspace statistics
    pub fn get_workspace_stats(&self) -> WorkspaceStats {
        let total_members = self.members.len();
        let published_count = self
            .members
            .values()
            .filter(|m| matches!(m.status, PublicationStatus::Published { .. }))
            .count();
        let unpublished_count = total_members - published_count;

        let mut manifest_types = HashSet::new();
        for member in self.members.values() {
            manifest_types.insert(&member.manifest_type);
        }

        WorkspaceStats {
            total_members,
            published_members: published_count,
            unpublished_members: unpublished_count,
            manifest_types: manifest_types.len(),
            workspace_root: self.config.root_path.clone(),
        }
    }
}

/// Workspace resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceResolutionResult {
    pub internal_dependencies: HashMap<String, Vec<String>>,
    pub external_dependencies: HashMap<String, Vec<String>>,
    pub version_conflicts: Vec<VersionConflict>,
    pub resolution_suggestions: Vec<String>,
}

/// Workspace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceStats {
    pub total_members: usize,
    pub published_members: usize,
    pub unpublished_members: usize,
    pub manifest_types: usize,
    pub workspace_root: PathBuf,
}

impl WorkspaceResolutionResult {
    pub fn has_conflicts(&self) -> bool {
        !self.version_conflicts.is_empty()
    }

    pub fn conflict_summary(&self) -> String {
        if self.version_conflicts.is_empty() {
            "No conflicts detected".to_string()
        } else {
            format!("{} conflicts detected", self.version_conflicts.len())
        }
    }

    pub fn suggestion_count(&self) -> usize {
        self.resolution_suggestions.len()
    }
}

/// Integrated workspace-aware dependency manager
pub struct WorkspaceAwareManager {
    resolver: WorkspaceResolver,
    graph: Arc<RwLock<DependencyGraph>>,
    resolution_strategy: ResolutionStrategy,
}

impl WorkspaceAwareManager {
    pub fn new(resolver: WorkspaceResolver, graph: Arc<RwLock<DependencyGraph>>) -> Self {
        Self {
            resolver,
            graph,
            resolution_strategy: ResolutionStrategy::WorkspaceAware,
        }
    }

    /// Perform workspace-aware dependency resolution
    pub async fn resolve_workspace_dependencies(
        &self,
    ) -> DependencyResult<WorkspaceResolutionResult> {
        self.resolver.resolve_workspace_dependencies().await
    }

    /// Update workspace members and sync with dependency graph
    pub async fn update_workspace(&mut self) -> DependencyResult<()> {
        self.resolver.discover_members().await?;
        self.resolver.sync_with_dependency_graph().await?;
        Ok(())
    }

    /// Get comprehensive workspace analysis
    pub async fn analyze_workspace(&self) -> DependencyResult<WorkspaceAnalysis> {
        let resolution_result = self.resolver.resolve_workspace_dependencies().await?;
        let stats = self.resolver.get_workspace_stats();
        let graph = self.graph.read().await;

        let workspace_members = self.resolver.get_all_members().len();
        let total_graph_nodes = graph.get_statistics().total_packages;

        let coverage = if total_graph_nodes > 0 {
            (workspace_members as f64 / total_graph_nodes as f64) * 100.0
        } else {
            0.0
        };

        Ok(WorkspaceAnalysis {
            resolution: resolution_result,
            stats,
            workspace_coverage_percent: coverage,
            graph_integrity_check: self.check_graph_integrity().await?,
        })
    }

    async fn check_graph_integrity(&self) -> DependencyResult<GraphIntegrityReport> {
        let graph = self.graph.read().await;
        let workspace_members: HashSet<String> = self.resolver.members.keys().cloned().collect();

        let mut missing_members = Vec::new();
        let mut extra_nodes = Vec::new();

        // Check if all workspace members are in the graph
        for member in &workspace_members {
            if !graph.node_indices.contains_key(member) {
                missing_members.push(member.clone());
            }
        }

        // Check if all graph nodes that should be workspace members are marked as such
        for node in graph.get_all_packages() {
            if node.is_workspace_member && !workspace_members.contains(&node.name) {
                extra_nodes.push(node.name.clone());
            }
        }

        Ok(GraphIntegrityReport {
            missing_workspace_members: missing_members.clone(),
            incorrect_workspace_marking: extra_nodes.clone(),
            is_integrity_maintained: missing_members.is_empty() && extra_nodes.is_empty(),
        })
    }
}

/// Comprehensive workspace analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAnalysis {
    pub resolution: WorkspaceResolutionResult,
    pub stats: WorkspaceStats,
    pub workspace_coverage_percent: f64,
    pub graph_integrity_check: GraphIntegrityReport,
}

/// Graph integrity report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphIntegrityReport {
    pub missing_workspace_members: Vec<String>,
    pub incorrect_workspace_marking: Vec<String>,
    pub is_integrity_maintained: bool,
}

impl WorkspaceAnalysis {
    pub fn is_healthy(&self) -> bool {
        !self.resolution.has_conflicts() && self.graph_integrity_check.is_integrity_maintained
    }

    pub fn health_score(&self) -> f64 {
        let mut score = 100.0;

        // Deduct for conflicts
        score -= self.resolution.version_conflicts.len() as f64 * 5.0;

        // Deduct for integrity issues
        let integrity_issues = self.graph_integrity_check.missing_workspace_members.len()
            + self.graph_integrity_check.incorrect_workspace_marking.len();
        score -= integrity_issues as f64 * 10.0;

        // Deduct for low workspace coverage
        if self.workspace_coverage_percent < 80.0 {
            score -= (100.0 - self.workspace_coverage_percent) * 0.5;
        }

        score.max(0.0)
    }
}
