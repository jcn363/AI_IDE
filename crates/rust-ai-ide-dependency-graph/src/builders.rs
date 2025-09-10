//! Builder patterns and configuration for dependency graph operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::error::*;
use crate::graph::*;
use crate::resolver::*;
use crate::cache::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphConfig {
    pub max_parallel_fetches: usize,
    pub cache_enabled: bool,
    pub cache_ttl: Duration,
    pub enable_circular_dependency_detection: bool,
    pub max_dependency_depth: Option<usize>,
    pub default_resolution_strategy: ResolutionStrategy,
}

impl Default for DependencyGraphConfig {
    fn default() -> Self {
        Self {
            max_parallel_fetches: 10,
            cache_enabled: true,
            cache_ttl: Duration::from_secs(3600), // 1 hour
            enable_circular_dependency_detection: true,
            max_dependency_depth: Some(100),
            default_resolution_strategy: ResolutionStrategy::Conservative,
        }
    }
}

pub struct DependencyGraphServiceBuilder {
    config: DependencyGraphConfig,
    graph: Option<Arc<RwLock<DependencyGraph>>>,
    cache: Option<Arc<GraphCache>>,
}

impl Default for DependencyGraphServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraphServiceBuilder {
    pub fn new() -> Self {
        Self {
            config: DependencyGraphConfig::default(),
            graph: None,
            cache: None,
        }
    }

    pub fn with_config(mut self, config: DependencyGraphConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_graph(mut self, graph: DependencyGraph) -> Self {
        self.graph = Some(Arc::new(RwLock::new(graph)));
        self
    }

    pub fn with_cache(mut self, cache: Arc<GraphCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn enable_cache(mut self, enabled: bool) -> Self {
        self.config.cache_enabled = enabled;
        self
    }

    pub fn with_resolution_strategy(mut self, strategy: ResolutionStrategy) -> Self {
        self.config.default_resolution_strategy = strategy;
        self
    }

    pub fn with_max_parallel_fetches(mut self, max_fetches: usize) -> Self {
        self.config.max_parallel_fetches = max_fetches;
        self
    }

    pub async fn build(self) -> DependencyResult<DependencyGraphService> {
        let graph = self.graph.unwrap_or_else(|| Arc::new(RwLock::new(DependencyGraph::new())));
        let cache = self.cache.unwrap_or_else(|| Arc::new(GraphCache::new()));

        // Initialize cache warm-up if enabled
        if self.config.cache_enabled {
            let cached_graph = CachedDependencyGraph::new(graph.clone(), cache.clone());
            if let Err(e) = cached_graph.warmup_cache().await {
                tracing::warn!("Cache warm-up failed: {:?}", e);
            }
        }

        Ok(DependencyGraphService {
            graph,
            cache,
            config: self.config,
        })
    }
}

pub struct DependencyGraphService {
    pub graph: Arc<RwLock<DependencyGraph>>,
    pub cache: Arc<GraphCache>,
    pub config: DependencyGraphConfig,
}

impl DependencyGraphService {
    pub fn builder() -> DependencyGraphServiceBuilder {
        DependencyGraphServiceBuilder::new()
    }

    pub async fn create_resolver(&self) -> DependencyResolver {
        DependencyResolver::new(
            self.graph.clone(),
            self.config.default_resolution_strategy,
        ).with_parallel_fetches(self.config.max_parallel_fetches)
    }

    pub async fn get_graph_stats(&self) -> DependencyResult<DependencyGraphStats> {
        let graph = self.graph.read().await;
        Ok(graph.get_statistics())
    }

    pub async fn validate_graph(&self) -> DependencyResult<GraphValidationResult> {
        let graph = self.graph.read().await;

        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        if self.config.enable_circular_dependency_detection {
            if graph.has_cycles() {
                let cycles = graph.get_cycles();
                errors.push(ValidationError::CircularDependencies(cycles));
            }
        }

        if let Some(max_depth) = self.config.max_dependency_depth {
            let deep_packages: Vec<String> = graph.get_packages_by_depth().await?
                .into_iter()
                .filter(|(_, depth)| *depth > max_depth)
                .map(|(name, _)| name)
                .collect();

            if !deep_packages.is_empty() {
                warnings.push(ValidationWarning::DeepDependencies { packages: deep_packages, max_depth });
            }
        }

        Ok(GraphValidationResult { warnings, errors })
    }
}

#[derive(Debug, Clone)]
pub struct GraphValidationResult {
    pub warnings: Vec<ValidationWarning>,
    pub errors: Vec<ValidationError>,
}

impl GraphValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn get_summary(&self) -> ValidationSummary {
        ValidationSummary {
            total_warnings: self.warnings.len(),
            total_errors: self.errors.len(),
            is_valid: self.is_valid(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_warnings: usize,
    pub total_errors: usize,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub enum ValidationWarning {
    DeepDependencies { packages: Vec<String>, max_depth: usize },
    UnusedDependencies(Vec<String>),
    DeprecatedDependencies(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    CircularDependencies(Vec<Vec<String>>),
    MissingDependencies(Vec<String>),
    InvalidVersions(Vec<String>),
}

pub struct WorkspaceResolverBuilder {
    pub workspace_root: Option<String>,
    pub member_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_depth: Option<usize>,
}

impl Default for WorkspaceResolverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceResolverBuilder {
    pub fn new() -> Self {
        Self {
            workspace_root: None,
            member_patterns: vec!["*".to_string()],
            exclude_patterns: vec!["vendor".to_string(), "target".to_string()],
            max_depth: Some(5),
        }
    }

    pub fn with_workspace_root(mut self, root: String) -> Self {
        self.workspace_root = Some(root);
        self
    }

    pub fn with_member_patterns(mut self, patterns: Vec<String>) -> Self {
        self.member_patterns = patterns;
        self
    }

    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    pub fn with_max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn build(self) -> WorkspaceResolver {
        WorkspaceResolver {
            workspace_root: self.workspace_root,
            member_patterns: self.member_patterns,
            exclude_patterns: self.exclude_patterns,
            max_depth: self.max_depth,
        }
    }
}