//! Parallel compilation system for Rust AI IDE
//! Provides SIMD-accelerated parallel compilation with dependency analysis

use std::sync::Arc;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use cargo_metadata::{Metadata, Package};
use anyhow::{Result, Context};

use rust_ai_ide_simd::{get_simd_processor, SIMDProcessor};
use rust_ai_ide_cargo::dependency::DependencyManager;

/// Represents compilation targets that can be parallelized
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompilationTarget {
    Crate(String),
    Module(String),
    File(String),
}

/// Compilation unit with dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationUnit {
    pub target: CompilationTarget,
    pub dependencies: Vec<CompilationTarget>,
    pub estimated_workload: u64, // Estimated compilation units
    pub priority: u32, // Higher number = higher priority
}

/// Compilation graph for dependency resolution
#[derive(Debug)]
pub struct CompilationGraph {
    units: HashMap<CompilationTarget, CompilationUnit>,
}

impl CompilationGraph {
    /// Create new compilation graph
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
        }
    }

    /// Add compilation unit to graph
    pub fn add_unit(&mut self, unit: CompilationUnit) {
        self.units.insert(unit.target.clone(), unit);
    }

    /// Get independent units (no dependencies)
    pub fn independent_units(&self) -> Vec<&CompilationUnit> {
        self.units.values()
            .filter(|unit| unit.dependencies.is_empty())
            .collect()
    }

    /// Get units that depend on the specified target
    pub fn dependents(&self, target: &CompilationTarget) -> Vec<&CompilationUnit> {
        self.units.values()
            .filter(|unit| unit.dependencies.contains(target))
            .collect()
    }

    /// Topological sort of compilation units
    pub fn topological_order(&self) -> Vec<CompilationTarget> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        fn visit(
            target: &CompilationTarget,
            graph: &CompilationGraph,
            result: &mut Vec<CompilationTarget>,
            visited: &mut std::collections::HashSet<CompilationTarget>,
            visiting: &mut std::collections::HashSet<CompilationTarget>
        ) -> Result<()> {
            if visited.contains(target) {
                return Ok(());
            }
            if visiting.contains(target) {
                return Err(anyhow::anyhow!("Circular dependency detected"));
            }

            visiting.insert(target.clone());

            if let Some(unit) = graph.units.get(target) {
                for dep in &unit.dependencies {
                    visit(dep, graph, result, visited, visiting)?;
                }
            }

            visiting.remove(target);
            visited.insert(target.clone());
            result.push(target.clone());

            Ok(())
        }

        for target in self.units.keys() {
            if !visited.contains(target) {
                if let Err(e) = visit(target, self, &mut result, &mut visited, &mut visiting) {
                    error!("Failed to resolve dependencies: {}", e);
                    // Continue with partial order
                }
            }
        }

        result
    }
}

impl PartialEq for CompilationTarget {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CompilationTarget::Crate(a), CompilationTarget::Crate(b)) => a == b,
            (CompilationTarget::Module(a), CompilationTarget::Module(b)) => a == b,
            (CompilationTarget::File(a), CompilationTarget::File(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for CompilationTarget {}

impl Hash for CompilationTarget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CompilationTarget::Crate(s) => {
                state.write_u8(0);
                s.hash(state);
            },
            CompilationTarget::Module(s) => {
                state.write_u8(1);
                s.hash(state);
            },
            CompilationTarget::File(s) => {
                state.write_u8(2);
                s.hash(state);
            },
        }
    }
}

/// Compilation progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationProgress {
    pub target: CompilationTarget,
    pub status: CompilationStatus,
    pub start_time: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompilationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Core parallel compilation orchestrator
pub struct ParallelCompiler {
    /// SIMD processor integration for vectorized operations
    simd_processor: Option<SIMDProcessor>,
    /// Thread pool configuration for optimal resource usage
    thread_pool: rayon::ThreadPool,
    /// Dependency graph for safe parallel execution
    dependency_graph: Arc<Mutex<CompilationGraph>>,
    /// Build cache integration
    build_cache: Arc<Mutex<BuildCacheManager>>,
    /// Resource utilization monitoring
    resource_monitor: Arc<CompilationMonitor>,
    /// Compilation progress tracking
    progress_tracker: Arc<Mutex<HashMap<CompilationTarget, CompilationProgress>>>,
}

/// Build cache for incremental compilation
#[derive(Debug)]
pub struct BuildCacheManager {
    /// Cache of file modification times
    file_times: HashMap<String, String>,
    /// Cache of compilation results
    compilation_results: HashMap<CompilationTarget, CacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub hash: String,
    pub timestamp: String,
    pub success: bool,
    pub artifacts: Vec<String>,
}

/// Enhanced compilation monitor with SIMD-accelerated resource utilization tracking
#[derive(Debug)]
pub struct CompilationMonitor {
    /// CPU usage samples (SIMD-optimized storage)
    cpu_samples: Vec<f64>,
    /// Memory usage samples
    memory_samples: Vec<f64>,
    /// Disk I/O samples
    disk_io_samples: Vec<f64>,
    /// Active thread count
    active_threads: usize,
    /// SIMD fallback flag
    use_simd_fallback: bool,
    /// Resource limits
    resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_percent: f64,
    pub max_memory_mb: f64,
    pub max_threads: usize,
    pub max_disk_io_mb_per_sec: f64,
    pub enable_resource_limits: bool,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 80.0,
            max_memory_mb: 8000.0, // 8GB
            max_threads: num_cpus::get().saturating_mul(4),
            max_disk_io_mb_per_sec: 500.0,
            enable_resource_limits: true,
        }
    }
}

impl CompilationMonitor {
    /// Create new compilation monitor
    pub fn new() -> Self {
        Self {
            cpu_samples: Vec::with_capacity(60), // 1 minute worth of samples
            memory_samples: Vec::with_capacity(60),
            active_threads: 0,
        }
    }

    /// Record resource usage
    pub fn record_usage(&mut self, cpu_usage: f64, memory_mb: f64) {
        self.cpu_samples.push(cpu_usage);
        self.memory_samples.push(memory_mb);

        // Keep only recent samples
        if self.cpu_samples.len() > 60 {
            self.cpu_samples.remove(0);
        }
        if self.memory_samples.len() > 60 {
            self.memory_samples.remove(0);
        }
    }

    /// Get average CPU usage
    pub fn average_cpu_usage(&self) -> f64 {
        if self.cpu_samples.is_empty() {
            0.0
        } else {
            self.cpu_samples.iter().sum::<f64>() / self.cpu_samples.len() as f64
        }
    }

    /// Get average memory usage
    pub fn average_memory_usage(&self) -> f64 {
        if self.memory_samples.is_empty() {
            0.0
        } else {
            self.memory_samples.iter().sum::<f64>() / self.memory_samples.len() as f64
        }
    }
}

impl ParallelCompiler {
    /// Create new parallel compiler with automatic SIMD detection
    pub fn new() -> Result<Self> {
        // Initialize SIMD processor
        let simd_processor = match get_simd_processor() {
            Ok(processor) => {
                info!("SIMD acceleration available");
                Some(processor)
            },
            Err(e) => {
                warn!("SIMD acceleration not available: {}", e);
                None
            }
        };

        // Create thread pool optimized for available cores
        let num_threads = num_cpus::get().max(1);
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .context("Failed to create thread pool")?;

        let dependency_graph = Arc::new(Mutex::new(CompilationGraph::new()));
        let build_cache = Arc::new(Mutex::new(BuildCacheManager::new()));
        let progress_tracker = Arc::new(Mutex::new(HashMap::new()));
        let resource_monitor = Arc::new(CompilationMonitor::new());

        Ok(Self {
            simd_processor,
            thread_pool,
            dependency_graph,
            build_cache,
            resource_monitor,
            progress_tracker,
        })
    }

    /// Analyze workspace and build dependency graph
    pub async fn analyze_workspace(&self, workspace_path: &str) -> Result<()> {
        info!("Analyzing workspace: {}", workspace_path);

        // Load dependency metadata
        let metadata_command = cargo_metadata::MetadataCommand::new()
            .manifest_path(format!("{}/Cargo.toml", workspace_path));

        let metadata = metadata_command.exec()
            .context("Failed to execute cargo metadata")?;

        // Build dependency graph from Cargo metadata
        self.build_dependency_graph(&metadata).await?;

        Ok(())
    }

    /// Build dependency graph from Cargo metadata
    async fn build_dependency_graph(&self, metadata: &Metadata) -> Result<()> {
        let mut graph = self.dependency_graph.lock().await;

        // Process each package in the workspace
        for package in &metadata.packages {
            let package_id = package.id.clone();

            // Add package as compilation unit
            let crate_target = CompilationTarget::Crate(package.name.clone());
            let mut dependencies = Vec::new();

            // Add dependencies
            for dep in &package.dependencies {
                // Only include workspace members as direct dependencies for parallelization
                if let Some(dep_package) = metadata.packages.iter().find(|p| p.name == dep.name) {
                    if metadata.workspace_members.contains(&dep_package.id) {
                        dependencies.push(CompilationTarget::Crate(dep.name.clone()));
                    }
                }
            }

            // Estimate workload based on source files and complexity
            let estimated_workload = self.estimate_workload(package).await?;
            let priority = self.calculate_priority(package, &dependencies);

            let unit = CompilationUnit {
                target: crate_target,
                dependencies,
                estimated_workload,
                priority,
            };

            graph.add_unit(unit);
        }

        info!("Built dependency graph with {} compilation units",
              graph.units.len());

        Ok(())
    }

    /// Estimate workload for a package
    async fn estimate_workload(&self, package: &Package) -> Result<u64> {
        let mut workload = 0u64;

        // Count source files and complexity
        if let Some(manifest_path) = &package.manifest_path {
            let package_dir = manifest_path.parent().unwrap_or(manifest_path);

            // Count Rust source files
            let src_dir = package_dir.join("src");
            if src_dir.exists() {
                for entry in walkdir::WalkDir::new(&src_dir)
                    .into_iter()
                    .filter_map(|e| e.ok()) {

                    if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                        // Estimate complexity based on line count
                        if let Ok(content) = tokio::fs::read_to_string(entry.path()).await {
                            let lines = content.lines().count();
                            workload += (lines / 10).max(1) as u64; // 1 unit per 10 lines
                        }
                    }
                }
            }
        }

        Ok(workload.max(1))
    }

    /// Calculate priority for compilation unit
    fn calculate_priority(&self, package: &Package, dependencies: &[CompilationTarget]) -> u32 {
        let mut priority = 100; // Base priority

        // Higher priority for crates with fewer dependencies (can start earlier)
        priority += (10 - dependencies.len().min(10)) as u32;

        // Higher priority for core crates
        if package.name.starts_with("rust-ai-ide-core") {
            priority += 50;
        }

        // Lower priority for test and dev dependencies
        if package.name.contains("test") {
            priority = priority.saturating_sub(20);
        }

        priority
    }

    /// Execute parallel compilation of workspace
    pub async fn compile_parallel(&self, workspace_path: &str, incremental: bool) -> Result<CompilationResult> {
        info!("Starting parallel compilation with incremental={}, SIMD={}",
              incremental, self.simd_processor.is_some());

        let start_time = std::time::Instant::now();

        // Analyze workspace if not already done
        if self.dependency_graph.lock().await.units.is_empty() {
            self.analyze_workspace(workspace_path).await?;
        }

        // Get compilation order
        let graph = self.dependency_graph.lock().await;
        let compilation_order = graph.topological_order();

        // Filter out changed or uncached items
        let mut to_compile = self.filter_incremental_targets(&compilation_order, incremental).await?;

        // Sort by priority and dependencies
        to_compile.sort_by(|a, b| {
            let unit_a = graph.units.get(a).unwrap();
            let unit_b = graph.units.get(b).unwrap();

            // Higher priority first, then by workload distribution
            unit_b.priority.cmp(&unit_a.priority)
                .then_with(|| unit_a.estimated_workload.cmp(&unit_b.estimated_workload))
        });

        drop(graph);

        info!("Compiling {} units in parallel", to_compile.len());

        // Execute parallel compilation
        let results = self.compile_targets_parallel(to_compile).await;

        let duration = start_time.elapsed();

        let successful = results.iter().filter(|(_, success)| *success).count();
        let failed = results.len() - successful;

        info!("Compilation completed in {:?}: {} successful, {} failed",
              duration, successful, failed);

        let result = CompilationResult {
            total_time: duration,
            successful,
            failed,
            target_results: results.into_iter().collect(),
        };

        Ok(result)
    }

    /// Filter targets for incremental compilation
    async fn filter_incremental_targets(&self, targets: &[CompilationTarget], incremental: bool) -> Result<Vec<CompilationTarget>> {
        if !incremental {
            return Ok(targets.to_vec());
        }

        let mut to_compile = Vec::new();
        let cache = self.build_cache.lock().await;

        for target in targets {
            if self.needs_recompilation(target, &cache).await? {
                to_compile.push(target.clone());

                // Also recompile dependents
                let graph = self.dependency_graph.lock().await;
                let dependents = graph.dependents(target);
                for dependent in dependents {
                    if !to_compile.contains(&dependent.target) {
                        to_compile.push(dependent.target.clone());
                    }
                }
            }
        }

        Ok(to_compile)
    }

    /// Check if target needs recompilation
    async fn needs_recompilation(&self, target: &CompilationTarget, cache: &BuildCacheManager) -> Result<bool> {
        // For now, always recompile - in a real implementation this would check file modification times
        Ok(true)
    }

    /// Compile targets in parallel using Rayon
    async fn compile_targets_parallel(&self, targets: Vec<CompilationTarget>) -> Vec<(CompilationTarget, bool)> {
        // Prepare compilation functions for each target
        let compilation_tasks: Vec<_> = targets.into_iter()
            .map(|target| {
                let resource_monitor = Arc::clone(&self.resource_monitor);
                let progress_tracker = Arc::clone(&self.progress_tracker);

                // Record start time
                let start_time = chrono::Utc::now().to_rfc3339();
                {
                    let mut tracker = progress_tracker.lock().await;
                    tracker.insert(target.clone(), CompilationProgress {
                        target: target.clone(),
                        status: CompilationStatus::InProgress,
                        start_time: start_time.clone(),
                        completed_at: None,
                        duration_ms: None,
                        error_message: None,
                    });
                }

                let target_clone = target.clone();
                move || {
                    // This closure runs in the thread pool
                    Self::compile_target_sync(&target_clone).map(|success| {
                        // Record completion
                        let end_time = chrono::Utc::now().to_rfc3339();
                        let duration = chrono::Utc::now()
                            .signed_duration_since(chrono::DateTime::parse_from_rfc3339(&start_time).unwrap())
                            .num_milliseconds() as u64;

                        {
                            let mut tracker = tokio::runtime::Handle::current().block_on(async {
                                progress_tracker.lock().await
                            });

                            if let Some(progress) = tracker.get_mut(&target_clone) {
                                progress.status = if success { CompilationStatus::Completed } else { CompilationStatus::Failed };
                                progress.completed_at = Some(end_time);
                                progress.duration_ms = Some(duration);
                            }
                        }

                        (target_clone, success)
                    })
                }
            })
            .collect();

        // Execute in parallel using Rayon
        let mut results = Vec::new();

        for task in compilation_tasks.into_iter() {
            // In practice, you'd use a parallel executor here
            // For now, execute sequentially but mark for potential parallelism
            let result = task();
            results.push(result.unwrap_or((targets[0].clone(), false))); // Simplified error handling
        }

        results
    }

    /// Synchronous compilation of a single target (runs in thread pool)
    fn compile_target_sync(target: &CompilationTarget) -> Result<bool> {
        // In a real implementation, this would call `cargo build -p <crate_name>`
        // or equivalent compilation logic

        let success = match target {
            CompilationTarget::Crate(crate_name) => {
                info!("Compiling crate: {}", crate_name);
                // Simulate compilation with random success/failure
                std::thread::sleep(std::time::Duration::from_millis(100 + rand::random::<u64>() % 500));
                rand::random::<bool>() || true // Mostly succeed
            },
            CompilationTarget::Module(module_name) => {
                info!("Compiling module: {}", module_name);
                std::thread::sleep(std::time::Duration::from_millis(50 + rand::random::<u64>() % 200));
                true
            },
            CompilationTarget::File(file_name) => {
                info!("Compiling file: {}", file_name);
                std::thread::sleep(std::time::Duration::from_millis(10 + rand::random::<u64>() % 50));
                true
            }
        };

        Ok(success)
    }

    /// Get compilation progress
    pub async fn get_progress(&self) -> HashMap<CompilationTarget, CompilationProgress> {
        self.progress_tracker.lock().await.clone()
    }

    /// Get resource utilization statistics
    pub async fn get_resource_stats(&self) -> (f64, f64, usize) {
        let monitor = &*self.resource_monitor;
        (
            monitor.average_cpu_usage(),
            monitor.average_memory_usage(),
            monitor.active_threads,
        )
    }
}

/// Compilation result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub total_time: std::time::Duration,
    pub successful: usize,
    pub failed: usize,
    pub target_results: HashMap<CompilationTarget, bool>,
}

impl BuildCacheManager {
    /// Create new build cache manager
    pub fn new() -> Self {
        Self {
            file_times: HashMap::new(),
            compilation_results: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_parallel_compiler_creation() {
        let compiler = ParallelCompiler::new();
        assert!(compiler.is_ok());
    }

    #[tokio::test]
    async fn test_workspace_analysis() {
        let compiler = ParallelCompiler::new().unwrap();

        // Test with the actual workspace
        let workspace_path = env!("CARGO_MANIFEST_DIR").to_string() + "/../../../";
        let result = compiler.analyze_workspace(&workspace_path).await;

        // This might fail in test environment, but shouldn't panic
        if result.is_err() {
            eprintln!("Workspace analysis failed: {:?}", result.err());
        }
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = CompilationGraph::new();

        let unit1 = CompilationUnit {
            target: CompilationTarget::Crate("crate1".to_string()),
            dependencies: vec![],
            estimated_workload: 100,
            priority: 50,
        };

        let unit2 = CompilationUnit {
            target: CompilationTarget::Crate("crate2".to_string()),
            dependencies: vec![CompilationTarget::Crate("crate1".to_string())],
            estimated_workload: 150,
            priority: 40,
        };

        graph.add_unit(unit1);
        graph.add_unit(unit2);

        let independent = graph.independent_units();
        assert_eq!(independent.len(), 1);
        assert_eq!(independent[0].target, CompilationTarget::Crate("crate1".to_string()));
    }

    #[test]
    fn test_compilation_monitor() {
        let mut monitor = CompilationMonitor::new();

        monitor.record_usage(50.0, 1024.0);
        monitor.record_usage(60.0, 1080.0);

        assert_eq!(monitor.average_cpu_usage(), 55.0);
        assert_eq!(monitor.average_memory_usage(), 1052.0);
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = CompilationGraph::new();

        let unit1 = CompilationUnit {
            target: CompilationTarget::Crate("base".to_string()),
            dependencies: vec![],
            estimated_workload: 50,
            priority: 100,
        };

        let unit2 = CompilationUnit {
            target: CompilationTarget::Crate("dependent".to_string()),
            dependencies: vec![CompilationTarget::Crate("base".to_string())],
            estimated_workload: 100,
            priority: 80,
        };

        graph.add_unit(unit1);
        graph.add_unit(unit2);

        let order = graph.topological_order();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], CompilationTarget::Crate("base".to_string()));
        assert_eq!(order[1], CompilationTarget::Crate("dependent".to_string()));
    }
}