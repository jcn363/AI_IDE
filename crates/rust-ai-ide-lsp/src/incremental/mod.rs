//! Incremental Analysis Framework for Rust AI IDE
//!
//! This module provides comprehensive incremental analysis capabilities for large multi-language codebases:
//!
//! # Features
//! - **Change-aware parsing**: Detects and processes only changed files and their dependencies
//! - **Dependency tracking**: Builds and maintains dependency graphs across languages
//! - **Selective re-analysis**: Re-analyzes only affected code when changes occur
//! - **Parallel processing**: Processes different file types concurrently
//! - **Smart caching**: Caches analysis results with intelligent invalidation
//! - **Performance monitoring**: Tracks analysis performance and optimization metrics
//!
//! # Architecture Overview
//!
//! The incremental analysis system consists of several key components:
//!
//! - **Change Tracker**: Monitors file system changes and git operations
//! - **Dependency Analyzer**: Builds and maintains inter-file and cross-language dependencies
//! - **Analysis Cache**: Stores analysis results with change-based invalidation
//! - **Parallel Processor**: Coordinates concurrent analysis of different file types
//! - **Analysis Engine**: Performs actual code analysis and produces results
//!
//! # Usage
//!
//! ```rust,no_run
//! use rust_ai_ide_lsp::incremental::{IncrementalAnalyzer, IncrementalConfig};
//!
//! let config = IncrementalConfig::default();
//! let analyzer = IncrementalAnalyzer::new(config).await?;
//!
//! // Analyze a workspace incrementally
//! let results = analyzer.analyze_workspace().await?;
//! println!("Analyzed {} files, found {} issues", results.processed_files, results.total_issues);
//! ```

pub mod change_tracker;
pub mod dependency_analyzer;
pub mod analysis_cache;
pub mod parallel_processor;
pub mod analysis_engine;
pub mod file_hash;
pub mod analysis_result;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

use crate::diagnostics::{Diagnostic, CodeSuggestion};
use change_tracker::{ChangeTracker, FileChange};

/// Configuration for incremental analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalConfig {
    /// Maximum number of concurrent analysis tasks
    pub max_concurrent_tasks: usize,
    /// Analysis cache size limit in MB
    pub cache_size_mb: usize,
    /// Enable dependency analysis across languages
    pub enable_cross_language_deps: bool,
    /// Minimum time between analysis runs (ms)
    pub analysis_debounce_ms: u64,
    /// File types to analyze (empty means all)
    pub supported_extensions: Vec<String>,
    /// Enable change-aware analysis
    pub enable_change_tracking: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Dependency analysis depth
    pub max_dependency_depth: usize,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 8,
            cache_size_mb: 1024,
            enable_cross_language_deps: true,
            analysis_debounce_ms: 500,
            supported_extensions: vec![
                "rs".to_string(), "py".to_string(), "js".to_string(), "ts".to_string(),
                "go".to_string(), "java".to_string(), "cpp".to_string(), "c".to_string(),
                "h".to_string(), "hpp".to_string(), "cc".to_string(),
            ],
            enable_change_tracking: true,
            cache_ttl_seconds: 3600,
            max_dependency_depth: 3,
        }
    }
}

/// Result of an incremental analysis run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalAnalysisResult {
    /// Number of files processed
    pub processed_files: usize,
    /// Number of files that were re-analyzed (not from cache)
    pub re_analyzed_files: usize,
    /// Total issues found across all files
    pub total_issues: usize,
    /// Analysis duration in milliseconds
    pub duration_ms: u64,
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,
    /// Files with issues
    pub files_with_issues: HashMap<PathBuf, Vec<Diagnostic>>,
    /// Code suggestions generated
    pub suggestions: HashMap<PathBuf, Vec<CodeSuggestion>>,
    /// Dependencies discovered
    pub dependencies: HashMap<PathBuf, Vec<PathBuf>>,
}

/// File analysis state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysisState {
    /// Last known hash of the file content
    pub content_hash: String,
    /// Last analysis timestamp
    pub last_analyzed: chrono::DateTime<chrono::Utc>,
    /// Dependencies of this file
    pub dependencies: Vec<PathBuf>,
    /// Files that depend on this file
    pub dependents: Vec<PathBuf>,
    /// Cached analysis result (if available)
    pub cached_result: Option<FileAnalysisResult>,
}

/// Result of analyzing a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysisResult {
    /// Path of the analyzed file
    pub file_path: PathBuf,
    /// Language detected
    pub language: String,
    /// Diagnostics found
    pub diagnostics: Vec<Diagnostic>,
    /// Code suggestions
    pub suggestions: Vec<CodeSuggestion>,
    /// Analysis duration
    pub analysis_time_ms: u64,
    /// Whether this was from cache
    pub from_cache: bool,
}

/// Incremental analyzer main struct
pub struct IncrementalAnalyzer {
    config: IncrementalConfig,
    change_tracker: Arc<ChangeTracker>,
    analysis_cache: analysis_cache::AnalysisCache,
    dependency_analyzer: dependency_analyzer::DependencyAnalyzer,
    analysis_engine: analysis_engine::AnalysisEngine,
    parallel_processor: parallel_processor::ParallelProcessor,
    file_states: Arc<RwLock<HashMap<PathBuf, FileAnalysisState>>>,
    workspace_root: PathBuf,
}

impl IncrementalAnalyzer {
    /// Create a new incremental analyzer
    pub async fn new(workspace_root: PathBuf, config: IncrementalConfig) -> Result<Self, String> {
        let change_tracker = Arc::new(ChangeTracker::new(&workspace_root).await?);
        let analysis_cache = analysis_cache::AnalysisCache::new(config.cache_size_mb).await?;
        let dependency_analyzer = dependency_analyzer::DependencyAnalyzer::new(
            config.enable_cross_language_deps,
            config.max_dependency_depth,
        );
        let analysis_engine = analysis_engine::AnalysisEngine::new(config.max_concurrent_tasks);
        let parallel_processor = parallel_processor::ParallelProcessor::new(config.max_concurrent_tasks);

        Ok(Self {
            config,
            change_tracker,
            analysis_cache,
            dependency_analyzer,
            analysis_engine,
            parallel_processor,
            file_states: Arc::new(RwLock::new(HashMap::new())),
            workspace_root,
        })
    }

    /// Perform incremental analysis of the entire workspace
    pub async fn analyze_workspace(&self) -> Result<IncrementalAnalysisResult, String> {
        let start_time = std::time::Instant::now();

        info!("Starting incremental workspace analysis: {}", self.workspace_root.display());

        // Get changed files since last analysis
        let changed_files = if self.config.enable_change_tracking {
            self.change_tracker.get_changed_files().await?
        } else {
            // Fallback: check all supported files
            self.get_all_supported_files().await?
        };

        debug!("Found {} changed files", changed_files.len());

        // Analyze affected files (changed files + their dependents)
        let affected_files = self.get_affected_files(&changed_files).await?;

        debug!("Analyzing {} affected files", affected_files.len());

        // Process files in parallel with dependency awareness
        let analysis_results = self.parallel_processor
            .process_files_incrementally(
                affected_files,
                &self.analysis_engine,
                &self.analysis_cache,
                &self.file_states,
            )
            .await?;

        // Update dependency graph with new results
        self.update_dependency_graph(&analysis_results).await?;

        // Update file states
        self.update_file_states(&analysis_results).await?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Calculate statistics
        let total_issues = analysis_results.values()
            .map(|result| result.diagnostics.len())
            .sum();

        let re_analyzed_files = analysis_results.values()
            .filter(|result| !result.from_cache)
            .count();

        let cache_hit_rate = if !analysis_results.is_empty() {
            1.0 - (re_analyzed_files as f64 / analysis_results.len() as f64)
        } else {
            1.0
        };

        let result = IncrementalAnalysisResult {
            processed_files: analysis_results.len(),
            re_analyzed_files,
            total_issues,
            duration_ms,
            cache_hit_rate,
            files_with_issues: self.extract_files_with_issues(&analysis_results),
            suggestions: self.extract_suggestions(&analysis_results),
            dependencies: self.extract_dependencies(&analysis_results),
        };

        info!(
            "Incremental analysis completed: {} files processed, {} issues found, {:.1}% cache hit rate",
            result.processed_files, result.total_issues, result.cache_hit_rate * 100.0
        );

        Ok(result)
    }

    /// Analyze a specific file and its dependencies
    pub async fn analyze_file(&self, file_path: PathBuf) -> Result<FileAnalysisResult, String> {
        let changed_files = vec![file_path];
        let affected_files = self.get_affected_files(&changed_files).await?;

        let results = self.parallel_processor
            .process_files_incrementally(
                affected_files,
                &self.analysis_engine,
                &self.analysis_cache,
                &self.file_states,
            )
            .await?;

        // Return the result for the specific file
        results.into_values().next()
            .ok_or_else(|| format!("No analysis result for requested file"))
    }

    /// Get analysis statistics
    pub async fn get_analysis_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        let file_states = self.file_states.read().await;

        stats.insert("total_files".to_string(), file_states.len().into());
        stats.insert("cached_files".to_string(),
            file_states.values()
                .filter(|state| state.cached_result.is_some())
                .count()
                .into()
        );

        stats.insert("dependencies_tracked".to_string(),
            file_states.values()
                .map(|state| state.dependencies.len())
                .sum::<usize>()
                .into()
        );

        stats
    }

    /// Force re-analysis of all files (clear cache)
    pub async fn force_full_analysis(&self) -> Result<(), String> {
        info!("Forcing full analysis - clearing all caches");

        self.analysis_cache.clear().await?;
        self.file_states.write().await.clear();

        // Reset change tracker
        self.change_tracker.reset().await?;

        Ok(())
    }

    // Helper methods
    async fn get_all_supported_files(&self) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();

        async fn walk_dir(
            dir: &std::path::Path,
            config: &IncrementalConfig,
            files: &mut Vec<PathBuf>,
        ) -> Result<(), String> {
            let mut entries = tokio::fs::read_dir(dir).await
                .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| format!("Failed to read directory entry: {}", e))? {

                let path = entry.path();

                if path.is_dir() {
                    // Skip common exclude directories
                    if !path.ends_with("target") && !path.ends_with(".git") && !path.ends_with("node_modules") {
                        walk_dir(&path, config, files).await?;
                    }
                } else if path.is_file() {
                    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                        if config.supported_extensions.contains(&extension.to_string()) {
                            files.push(path);
                        }
                    }
                }
            }

            Ok(())
        }

        walk_dir(&self.workspace_root, &self.config, &mut files).await?;
        Ok(files)
    }

    async fn get_affected_files(&self, changed_files: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
        let mut affected_files = changed_files.to_vec();
        let file_states = self.file_states.read().await;

        // Add all dependents of changed files
        for changed_file in changed_files {
            if let Some(state) = file_states.get(changed_file) {
                for dependent in &state.dependents {
                    if !affected_files.contains(dependent) {
                        affected_files.push(dependent.clone());
                    }
                }
            }
        }

        Ok(affected_files)
    }

    async fn update_dependency_graph(&self, results: &HashMap<PathBuf, FileAnalysisResult>) -> Result<(), String> {
        for (file_path, result) in results {
            self.dependency_analyzer.update_file_dependencies(
                file_path.clone(),
                &result.diagnostics,
                &result.suggestions,
            ).await?;
        }
        Ok(())
    }

    async fn update_file_states(&self, results: &HashMap<PathBuf, FileAnalysisResult>) -> Result<(), String> {
        let mut file_states = self.file_states.write().await;

        for (file_path, result) in results {
            let state = file_states.entry(file_path.clone()).or_insert_with(|| FileAnalysisState {
                content_hash: file_hash::calculate_file_hash(file_path).unwrap_or_default(),
                last_analyzed: chrono::Utc::now(),
                dependencies: Vec::new(),
                dependents: Vec::new(),
                cached_result: None,
            });

            state.last_analyzed = chrono::Utc::now();
            state.cached_result = Some(result.clone());
        }

        Ok(())
    }

    fn extract_files_with_issues(&self, results: &HashMap<PathBuf, FileAnalysisResult>) -> HashMap<PathBuf, Vec<Diagnostic>> {
        results.iter()
            .filter(|(_, result)| !result.diagnostics.is_empty())
            .map(|(path, result)| (path.clone(), result.diagnostics.clone()))
            .collect()
    }

    fn extract_suggestions(&self, results: &HashMap<PathBuf, FileAnalysisResult>) -> HashMap<PathBuf, Vec<CodeSuggestion>> {
        results.iter()
            .filter(|(_, result)| !result.suggestions.is_empty())
            .map(|(path, result)| (path.clone(), result.suggestions.clone()))
            .collect()
    }

    fn extract_dependencies(&self, results: &HashMap<PathBuf, FileAnalysisResult>) -> HashMap<PathBuf, Vec<PathBuf>> {
        // This would be populated from dependency analysis
        results.iter()
            .map(|(path, _)| (path.clone(), Vec::new()))
            .collect()
    }
}

/// Builder for zero-copy analysis result construction
#[derive(Default)]
pub struct ZeroCopyAnalysisBuilder {
    file_path: Option<PathBuf>,
    language: Option<String>,
    diagnostics: Vec<super::diagnostics::CodeAnalysis>,
    suggestions: Vec<super::CodeSuggestion>,
    analysis_time_ms: u64,
    from_cache: bool,
}

impl ZeroCopyAnalysisBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    pub fn language<S: Into<String>>(mut self, lang: S) -> Self {
        self.language = Some(lang.into());
        self
    }

    pub fn add_diagnostic(mut self, diagnostic: super::diagnostics::CodeAnalysis) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    pub fn add_suggestion(mut self, suggestion: super::CodeSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn analysis_time_ms(mut self, time: u64) -> Self {
        self.analysis_time_ms = time;
        self
    }

    pub fn from_cache(mut self, cached: bool) -> Self {
        self.from_cache = cached;
        self
    }

    pub fn build_zero_copy(self) -> Result<FileAnalysisResult, String> {
        let file_path = self.file_path.ok_or("file_path is required")?;
        let language = self.language.unwrap_or("unknown".to_string());

        Ok(FileAnalysisResult {
            file_path,
            language,
            diagnostics: self.diagnostics, // Vec moves, no copy
            suggestions: self.suggestions, // Vec moves, no copy
            analysis_time_ms: self.analysis_time_ms,
            from_cache: self.from_cache,
        })
    }
}

/// Factory function for creating optimized incremental analyzers
pub async fn create_incremental_analyzer(
    workspace_root: PathBuf,
    enable_gpu: bool,
    enable_distributed: bool,
) -> Result<IncrementalAnalyzer, String> {
    let mut config = IncrementalConfig::default();

    // Optimize configuration based on available resources
    if enable_gpu {
        config.max_concurrent_tasks = 16; // Higher parallelism with GPU
    }

    if enable_distributed {
        config.cache_size_mb = 2048; // Larger cache for distributed setup
        config.max_concurrent_tasks = 12;
    }

    IncrementalAnalyzer::new(workspace_root, config).await
}

/// Factory function for creating zero-copy enabled incremental analyzers
pub async fn create_zero_copy_incremental_analyzer(
    workspace_root: PathBuf,
    enable_zero_copy_cache: bool,
) -> Result<ZeroCopyIncrementalAnalyzer, String> {
    let config = IncrementalConfig {
        max_concurrent_tasks: 8,
        cache_size_mb: 1024,
        enable_cross_language_deps: true,
        analysis_debounce_ms: 200, // Faster analysis for zero-copy
        supported_extensions: vec![
            "rs".to_string(), "py".to_string(), "js".to_string(), "ts".to_string(),
            "go".to_string(), "java".to_string(), "cpp".to_string(), "c".to_string(),
            "h".to_string(), "hpp".to_string(), "cc".to_string(),
        ],
        enable_change_tracking: true,
        cache_ttl_seconds: 1800, // Shorter TTL for zero-copy
        max_dependency_depth: 2, // Reduced for performance
    };

    ZeroCopyIncrementalAnalyzer::new(workspace_root, config, enable_zero_copy_cache).await
}

/// Zero-copy version of IncrementalAnalyzer
pub struct ZeroCopyIncrementalAnalyzer {
    config: IncrementalConfig,
    change_tracker: Arc<change_tracker::ChangeTracker>,
    zero_copy_cache: Arc<analysis_cache::ZeroCopyAnalysisCache>,
    analysis_engine: analysis_engine::AnalysisEngine,
    parallel_processor: parallel_processor::ParallelProcessor,
    file_states: Arc<RwLock<HashMap<PathBuf, FileAnalysisState>>>,
    workspace_root: PathBuf,
}

impl ZeroCopyIncrementalAnalyzer {
    pub async fn new(
        workspace_root: PathBuf,
        config: IncrementalConfig,
        enable_zero_copy_cache: bool,
    ) -> Result<Self, String> {
        let change_tracker = Arc::new(change_tracker::ChangeTracker::new(&workspace_root).await?);

        // Use zero-copy cache configuration
        let cache_config = analysis_cache::LspCacheConfig {
            base_config: rust_ai_ide_cache::CacheConfig::default(),
            enable_file_validation: true,
            max_file_cache_age_seconds: config.cache_ttl_seconds,
            analysis_ttl_seconds: config.cache_ttl_seconds / 2,
        };

        let zero_copy_cache = if enable_zero_copy_cache {
            Arc::new(analysis_cache::ZeroCopyAnalysisCache::new(cache_config.clone()))
        } else {
            // Fallback to regular cache wrapped in zero-copy interface
            // This is just for compatibility - in practice you'd use the regular cache directly
            Arc::new(analysis_cache::ZeroCopyAnalysisCache::new(cache_config))
        };

        let analysis_engine = analysis_engine::AnalysisEngine::new(config.max_concurrent_tasks);
        let parallel_processor = parallel_processor::ParallelProcessor::new(config.max_concurrent_tasks);

        Ok(Self {
            config,
            change_tracker,
            zero_copy_cache,
            analysis_engine,
            parallel_processor,
            file_states: Arc::new(RwLock::new(HashMap::new())),
            workspace_root,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_incremental_config_default() {
        let config = IncrementalConfig::default();
        assert_eq!(config.max_concurrent_tasks, 8);
        assert_eq!(config.cache_size_mb, 1024);
        assert!(config.enable_cross_language_deps);
        assert!(config.supported_extensions.contains(&"rs".to_string()));
    }

    #[tokio::test]
    async fn test_incremental_analyzer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();

        // Create a test Rust file
        fs::write(workspace_root.join("test.rs"), "fn main() {}").unwrap();

        let config = IncrementalConfig::default();
        let analyzer = IncrementalAnalyzer::new(workspace_root, config).await;

        // Should succeed even without LSP servers running
        assert!(analyzer.is_ok());
    }

    #[tokio::test]
    async fn test_get_supported_files() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();

        // Create test files
        fs::write(workspace_root.join("test.rs"), "fn main() {}").unwrap();
        fs::write(workspace_root.join("test.py"), "print('hello')").unwrap();
        fs::write(workspace_root.join("test.txt"), "hello world").unwrap();

        let analyzer = IncrementalAnalyzer::new(workspace_root.clone(), IncrementalConfig::default()).await.unwrap();
        let files = analyzer.get_all_supported_files().await.unwrap();

        // Should find .rs and .py but not .txt
        assert_eq!(files.len(), 2);
        let extensions: Vec<String> = files.iter()
            .filter_map(|f| f.extension().and_then(|e| e.to_str()).map(|s| s.to_string()))
            .collect();
        assert!(extensions.contains(&"rs".to_string()));
        assert!(extensions.contains(&"py".to_string()));
    }
}