//! Intelligent cache warm-up system for code generation
//!
//! This module provides comprehensive cache warming functionality that analyzes
//! existing codebases, identifies common patterns, and pre-generates templates
//! to accelerate code generation performance.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use futures_util::stream::{self, StreamExt};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use walkdir::WalkDir;

/// Cache entry with metadata for intelligent eviction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// Cached data
    pub data: T,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub accessed_at: DateTime<Utc>,
    /// Access count for LRU prioritization
    pub access_count: u64,
    /// Time-to-live in seconds
    pub ttl_seconds: Option<u64>,
    /// Pattern frequency score (higher = more common)
    pub pattern_score: f64,
    /// Development scenario relevance
    pub scenario_relevance: ScenarioRelevance,
}

/// Cache key for template patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatternKey {
    /// Pattern type (struct, function, trait, etc.)
    pub pattern_type: String,
    /// Language identifier
    pub language: String,
    /// Pattern signature hash
    pub signature_hash: String,
    /// Context features (async, generic, etc.)
    pub context_features: Vec<String>,
}

/// Cache key for scenario-based templates
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScenarioKey {
    /// Development scenario
    pub scenario: DevelopmentScenario,
    /// Language identifier
    pub language: String,
    /// Project type context
    pub project_context: ProjectContext,
}

/// Development scenarios for cache warming strategies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DevelopmentScenario {
    /// Web application development
    WebDevelopment,
    /// System/library development
    SystemDevelopment,
    /// API development
    ApiDevelopment,
    /// CLI tool development
    CliDevelopment,
    /// Testing-focused development
    TestDevelopment,
    /// Performance-critical development
    PerformanceDevelopment,
}

/// Project type context for intelligent cache loading
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectContext {
    /// Standard Rust library
    Library,
    /// Binary/application
    Binary,
    /// WebAssembly target
    Wasm,
    /// Embedded/no_std
    Embedded,
    /// Tauri application
    Tauri,
    /// Workspace with multiple crates
    Workspace,
}

/// Pattern frequency and relevance scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioRelevance {
    /// Relevance score for different scenarios (0.0 to 1.0)
    pub web_score: f64,
    pub system_score: f64,
    pub api_score: f64,
    pub cli_score: f64,
    pub test_score: f64,
    pub performance_score: f64,
}

/// Statistical analysis of code patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    /// Total patterns analyzed
    pub total_patterns: u64,
    /// Total lines analyzed across all files
    pub total_lines: u64,
    /// Pattern frequency distribution
    pub frequency_distribution: HashMap<String, u64>,
    /// Average pattern complexity
    pub avg_complexity: f64,
    /// Most common pattern types
    pub top_patterns: Vec<(String, u64)>,
    /// Language-specific statistics
    pub language_stats: HashMap<String, LanguageStats>,
}

/// Language-specific pattern statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    /// File count for this language
    pub file_count: u64,
    /// Total lines analyzed
    pub total_lines: u64,
    /// Pattern count by type
    pub patterns_by_type: HashMap<String, u64>,
}

/// Warm-up configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupConfig {
    /// Maximum cache size
    pub max_cache_size: u64,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum files to scan during warm-up
    pub max_files_to_scan: usize,
    /// Pattern frequency threshold for caching
    pub frequency_threshold: u64,
    /// Enable AI-powered pattern recognition
    pub enable_ai_recognition: bool,
    /// Development scenarios to prioritize
    pub prioritized_scenarios: Vec<DevelopmentScenario>,
    /// Project type context
    pub project_context: ProjectContext,
}

/// Main cache system with intelligent warm-up
pub struct IntelligentCache {
    /// Pattern-based cache
    pattern_cache: Cache<PatternKey, CacheEntry<String>>,
    /// Scenario-based cache
    scenario_cache: Cache<ScenarioKey, CacheEntry<Vec<String>>>,
    /// Pattern statistics
    statistics: Arc<RwLock<PatternStatistics>>,
    /// Warm-up configuration
    config: WarmupConfig,
    /// Cache warming state
    warming_state: Arc<Mutex<WarmingState>>,
    /// Template engine reference (for integration)
    template_engine: Option<Arc<crate::templates::TemplateEngine>>,
}

/// Current state of cache warming process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingState {
    /// Whether warm-up is in progress
    pub is_warming: bool,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f64,
    /// Files processed
    pub files_processed: usize,
    /// Patterns discovered
    pub patterns_discovered: usize,
    /// Cache entries created
    pub cache_entries_created: usize,
    /// Last warm-up timestamp
    pub last_warmup: Option<DateTime<Utc>>,
    /// Current warming phase
    pub current_phase: WarmingPhase,
}

/// Phases of the cache warming process
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarmingPhase {
    /// Initial scanning phase
    Scanning,
    /// Pattern analysis phase
    Analyzing,
    /// Template generation phase
    Generating,
    /// Cache population phase
    Populating,
    /// Optimization phase
    Optimizing,
    /// Completed
    Completed,
    /// Idle
    Idle,
}

impl Default for WarmingState {
    fn default() -> Self {
        Self {
            is_warming: false,
            progress: 0.0,
            files_processed: 0,
            patterns_discovered: 0,
            cache_entries_created: 0,
            last_warmup: None,
            current_phase: WarmingPhase::Idle,
        }
    }
}

impl IntelligentCache {
    /// Create a new intelligent cache system
    pub fn new(config: WarmupConfig) -> Self {
        let pattern_cache = Cache::builder()
            .max_capacity(config.max_cache_size)
            .time_to_live(Duration::from_secs(config.cache_ttl_seconds))
            .build();

        let scenario_cache = Cache::builder()
            .max_capacity(config.max_cache_size / 4) // Smaller for scenario cache
            .time_to_live(Duration::from_secs(config.cache_ttl_seconds * 2))
            .build();

        Self {
            pattern_cache,
            scenario_cache,
            statistics: Arc::new(RwLock::new(PatternStatistics::default())),
            config,
            warming_state: Arc::new(Mutex::new(WarmingState::default())),
            template_engine: None,
        }
    }

    /// Set template engine for integration
    pub fn with_template_engine(mut self, engine: Arc<crate::templates::TemplateEngine>) -> Self {
        self.template_engine = Some(engine);
        self
    }

    /// Perform intelligent cache warm-up
    pub async fn warmup(&self, project_root: &Path) -> Result<WarmingReport> {
        let mut state = self.warming_state.lock().await;
        if state.is_warming {
            return Err(anyhow!("Warm-up already in progress"));
        }

        state.is_warming = true;
        state.progress = 0.0;
        state.files_processed = 0;
        state.patterns_discovered = 0;
        state.cache_entries_created = 0;
        state.current_phase = WarmingPhase::Scanning;
        drop(state);

        let report = self.perform_warmup(project_root).await;

        let mut state = self.warming_state.lock().await;
        state.is_warming = false;
        state.last_warmup = Some(Utc::now());
        state.current_phase = WarmingPhase::Completed;
        drop(state);

        report
    }

    /// Internal warm-up implementation
    async fn perform_warmup(&self, project_root: &Path) -> Result<WarmingReport> {
        // Phase 1: Scan repository for source files
        self.update_phase(WarmingPhase::Scanning, 0.1).await;
        let source_files = self.scan_repository(project_root).await?;

        // Phase 2: Analyze patterns
        self.update_phase(WarmingPhase::Analyzing, 0.3).await;
        let pattern_analysis = self.analyze_patterns(&source_files).await?;

        // Phase 3: Generate templates
        self.update_phase(WarmingPhase::Generating, 0.6).await;
        let templates = self.generate_templates(&pattern_analysis).await?;

        // Phase 4: Populate cache
        self.update_phase(WarmingPhase::Populating, 0.8).await;
        let cache_entries = self.populate_cache(templates).await?;

        // Phase 5: Optimize and finalize
        self.update_phase(WarmingPhase::Optimizing, 0.95).await;
        self.optimize_cache().await?;

        self.update_phase(WarmingPhase::Completed, 1.0).await;

        Ok(WarmingReport {
            files_scanned: source_files.len(),
            patterns_discovered: pattern_analysis.total_patterns,
            cache_entries_created: cache_entries,
            statistics: pattern_analysis,
            timestamp: Utc::now(),
        })
    }

    /// Update warming phase and progress
    async fn update_phase(&self, phase: WarmingPhase, progress: f64) {
        let mut state = self.warming_state.lock().await;
        state.current_phase = phase;
        state.progress = progress;
    }

    /// Scan repository for source files
    async fn scan_repository(&self, project_root: &Path) -> Result<Vec<PathBuf>> {
        let mut source_files = Vec::new();
        let walker = WalkDir::new(project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| self.is_source_file(e.path()));

        for entry in walker {
            if source_files.len() >= self.config.max_files_to_scan {
                break;
            }

            source_files.push(entry.path().to_path_buf());

            let mut state = self.warming_state.lock().await;
            state.files_processed = source_files.len();
        }

        Ok(source_files)
    }

    /// Check if file is a source file we should analyze
    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("rs") | Some("ts") | Some("js") | Some("py") | Some("java"))
        } else {
            false
        }
    }

    /// Analyze code patterns from source files
    async fn analyze_patterns(&self, source_files: &[PathBuf]) -> Result<PatternStatistics> {
        let mut stats = PatternStatistics::default();
        let mut language_stats: HashMap<String, LanguageStats> = HashMap::new();

        // Process files in parallel with limited concurrency
        let stream = stream::iter(source_files)
            .map(|path| async move {
                self.analyze_single_file(&path).await
            })
            .buffer_unordered(10); // Limit concurrency

        let results: Vec<_> = stream.collect().await;

        for result in results {
            if let Ok(file_stats) = result {
                stats.total_patterns += file_stats.total_patterns;
                stats.total_lines += file_stats.total_lines;

                let lang_stats = language_stats
                    .entry(file_stats.language.clone())
                    .or_insert_with(|| LanguageStats {
                        file_count: 0,
                        total_lines: 0,
                        patterns_by_type: HashMap::new(),
                    });

                lang_stats.file_count += 1;
                lang_stats.total_lines += file_stats.total_lines;

                for (pattern_type, count) in file_stats.patterns_by_type {
                    *stats.frequency_distribution.entry(pattern_type.clone()).or_insert(0) += count;
                    *lang_stats.patterns_by_type.entry(pattern_type).or_insert(0) += count;
                }

                let mut state = self.warming_state.lock().await;
                state.patterns_discovered = stats.total_patterns as usize;
            }
        }

        stats.language_stats = language_stats;

        // Calculate top patterns
        let mut freq_vec: Vec<_> = stats.frequency_distribution.iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(a.1));
        stats.top_patterns = freq_vec.into_iter()
            .take(10)
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Calculate average complexity (simplified)
        stats.avg_complexity = if stats.total_patterns > 0 {
            (stats.total_lines as f64) / (stats.total_patterns as f64)
        } else {
            0.0
        };

        *self.statistics.write().await = stats.clone();

        Ok(stats)
    }

    /// Analyze a single file for patterns
    async fn analyze_single_file(&self, path: &Path) -> Result<FileAnalysisResult> {
        let content = tokio::fs::read_to_string(path).await?;
        let lines = content.lines().count();
        let language = self.detect_language(path);

        let mut patterns_by_type = HashMap::new();
        let mut total_patterns = 0u64;

        // Simple pattern detection (can be enhanced with AST parsing)
        if language == "rust" {
            total_patterns += self.count_rust_patterns(&content, &mut patterns_by_type);
        } else if language == "typescript" || language == "javascript" {
            total_patterns += self.count_js_patterns(&content, &mut patterns_by_type);
        }

        Ok(FileAnalysisResult {
            language,
            total_lines: lines as u64,
            total_patterns,
            patterns_by_type,
        })
    }

    /// Detect programming language from file path
    fn detect_language(&self, path: &Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("js") => "javascript".to_string(),
            Some("py") => "python".to_string(),
            Some("java") => "java".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Count Rust-specific patterns
    fn count_rust_patterns(&self, content: &str, patterns: &mut HashMap<String, u64>) -> u64 {
        let mut total = 0u64;

        // Count struct definitions
        let struct_count = content.matches("struct ").count() as u64;
        if struct_count > 0 {
            patterns.insert("struct".to_string(), struct_count);
            total += struct_count;
        }

        // Count function definitions
        let fn_count = content.matches("fn ").count() as u64;
        if fn_count > 0 {
            patterns.insert("function".to_string(), fn_count);
            total += fn_count;
        }

        // Count trait definitions
        let trait_count = content.matches("trait ").count() as u64;
        if trait_count > 0 {
            patterns.insert("trait".to_string(), trait_count);
            total += trait_count;
        }

        // Count async functions
        let async_fn_count = content.matches("async fn ").count() as u64;
        if async_fn_count > 0 {
            patterns.insert("async_function".to_string(), async_fn_count);
            total += async_fn_count;
        }

        // Count impl blocks
        let impl_count = content.matches("impl ").count() as u64;
        if impl_count > 0 {
            patterns.insert("impl".to_string(), impl_count);
            total += impl_count;
        }

        total
    }

    /// Count JavaScript/TypeScript patterns
    fn count_js_patterns(&self, content: &str, patterns: &mut HashMap<String, u64>) -> u64 {
        let mut total = 0u64;

        // Count function declarations
        let func_count = content.matches("function ").count() as u64;
        if func_count > 0 {
            patterns.insert("function".to_string(), func_count);
            total += func_count;
        }

        // Count class definitions
        let class_count = content.matches("class ").count() as u64;
        if class_count > 0 {
            patterns.insert("class".to_string(), class_count);
            total += class_count;
        }

        // Count arrow functions
        let arrow_count = content.matches("=> ").count() as u64;
        if arrow_count > 0 {
            patterns.insert("arrow_function".to_string(), arrow_count);
            total += arrow_count;
        }

        // Count async functions
        let async_count = content.matches("async ").count() as u64;
        if async_count > 0 {
            patterns.insert("async_function".to_string(), async_count);
            total += async_count;
        }

        total
    }

    /// Generate templates based on pattern analysis
    async fn generate_templates(&self, stats: &PatternStatistics) -> Result<Vec<TemplateGeneration>> {
        let mut templates = Vec::new();

        for (pattern_type, frequency) in &stats.frequency_distribution {
            if *frequency >= self.config.frequency_threshold {
                let template = self.generate_template_for_pattern(pattern_type, *frequency).await?;
                templates.push(template);
            }
        }

        Ok(templates)
    }

    /// Generate template for a specific pattern type
    async fn generate_template_for_pattern(&self, pattern_type: &str, frequency: u64) -> Result<TemplateGeneration> {
        let template_content = match pattern_type {
            "struct" => self.generate_struct_template().await,
            "function" => self.generate_function_template().await,
            "trait" => self.generate_trait_template().await,
            "impl" => self.generate_impl_template().await,
            "async_function" => self.generate_async_function_template().await,
            "class" => self.generate_class_template().await,
            "arrow_function" => self.generate_arrow_function_template().await,
            _ => format!("// Template for {} pattern\n", pattern_type),
        };

        Ok(TemplateGeneration {
            pattern_type: pattern_type.to_string(),
            template: template_content,
            frequency_score: frequency as f64 / 100.0, // Normalize score
            scenario_relevance: self.calculate_scenario_relevance(pattern_type),
        })
    }

    /// Generate struct template
    async fn generate_struct_template(&self) -> String {
        r#"/// Documentation for {{name}}
#[derive(Debug, Clone, Default)]
pub struct {{name}} {
    /// ID field
    pub id: String,
    /// Name field
    pub name: String,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl {{name}} {
    /// Create a new {{name}}
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: chrono::Utc::now(),
        }
    }
}"#.to_string()
    }

    /// Generate function template
    async fn generate_function_template(&self) -> String {
        r#"/// Documentation for {{name}}
pub fn {{name}}({{params}}) -> {{return_type}} {
    {{#if async}}todo!("Implement async {{name}}"){{/if}}
    {{#if sync}}todo!("Implement {{name}}"){{/if}}
}"#.to_string()
    }

    /// Generate trait template
    async fn generate_trait_template(&self) -> String {
        r#"/// Documentation for {{name}} trait
pub trait {{name}} {
    /// Method documentation
    fn {{method_name}}(&self) -> {{return_type}};

    {{#if has_default}}
    /// Default implementation
    fn {{default_method}}(&self) {
        // Default implementation
    }
    {{/if}}
}"#.to_string()
    }

    /// Generate impl template
    async fn generate_impl_template(&self) -> String {
        r#"impl {{trait_name}} for {{struct_name}} {
    fn {{method_name}}(&self) -> {{return_type}} {
        todo!("Implement {{method_name}} for {{struct_name}}")
    }
}"#.to_string()
    }

    /// Generate async function template
    async fn generate_async_function_template(&self) -> String {
        r#"/// Documentation for {{name}}
pub async fn {{name}}({{params}}) -> {{return_type}} {
    todo!("Implement async {{name}}")
}"#.to_string()
    }

    /// Generate class template (for JS/TS)
    async fn generate_class_template(&self) -> String {
        r#"/**
 * Documentation for {{name}} class
 */
export class {{name}} {
    /**
     * Constructor
     */
    constructor({{params}}) {
        {{#each params}}
        this.{{this}} = {{this}};
        {{/each}}
    }

    {{#if has_methods}}
    /**
     * Method documentation
     */
    {{method_name}}({{method_params}}) {
        // TODO: Implement method
    }
    {{/if}}
}"#.to_string()
    }

    /// Generate arrow function template
    async fn generate_arrow_function_template(&self) -> String {
        r#"/**
 * Documentation for {{name}} function
 */
export const {{name}} = ({{params}}) => {
    // TODO: Implement function
    return {{return_value}};
};"#.to_string()
    }

    /// Calculate scenario relevance for pattern
    fn calculate_scenario_relevance(&self, pattern_type: &str) -> ScenarioRelevance {
        match pattern_type {
            "struct" => ScenarioRelevance {
                web_score: 0.8,
                system_score: 0.9,
                api_score: 0.7,
                cli_score: 0.6,
                test_score: 0.5,
                performance_score: 0.8,
            },
            "async_function" => ScenarioRelevance {
                web_score: 0.9,
                system_score: 0.8,
                api_score: 0.9,
                cli_score: 0.7,
                test_score: 0.6,
                performance_score: 0.8,
            },
            "trait" => ScenarioRelevance {
                web_score: 0.6,
                system_score: 0.9,
                api_score: 0.8,
                cli_score: 0.5,
                test_score: 0.7,
                performance_score: 0.9,
            },
            "class" => ScenarioRelevance {
                web_score: 0.9,
                system_score: 0.6,
                api_score: 0.8,
                cli_score: 0.7,
                test_score: 0.8,
                performance_score: 0.7,
            },
            _ => ScenarioRelevance {
                web_score: 0.5,
                system_score: 0.5,
                api_score: 0.5,
                cli_score: 0.5,
                test_score: 0.5,
                performance_score: 0.5,
            },
        }
    }

    /// Populate cache with generated templates
    async fn populate_cache(&self, templates: Vec<TemplateGeneration>) -> Result<usize> {
        let mut entries_created = 0;

        for template in templates {
            // Create pattern-based cache entry
            let pattern_key = PatternKey {
                pattern_type: template.pattern_type.clone(),
                language: "rust".to_string(), // Default, can be parameterized
                signature_hash: format!("{:x}", template.pattern_type.len()), // Simple hash
                context_features: vec!["default".to_string()],
            };

            let pattern_entry = CacheEntry {
                data: template.template.clone(),
                created_at: Utc::now(),
                accessed_at: Utc::now(),
                access_count: 0,
                ttl_seconds: Some(self.config.cache_ttl_seconds),
                pattern_score: template.frequency_score,
                scenario_relevance: template.scenario_relevance.clone(),
            };

            self.pattern_cache.insert(pattern_key, pattern_entry).await;
            entries_created += 1;

            // Create scenario-based cache entries
            for scenario in &self.config.prioritized_scenarios {
                let scenario_key = ScenarioKey {
                    scenario: scenario.clone(),
                    language: "rust".to_string(),
                    project_context: self.config.project_context.clone(),
                };

                let scenario_entry = CacheEntry {
                    data: vec![template.template.clone()],
                    created_at: Utc::now(),
                    accessed_at: Utc::now(),
                    access_count: 0,
                    ttl_seconds: Some(self.config.cache_ttl_seconds * 2),
                    pattern_score: template.frequency_score,
                    scenario_relevance: template.scenario_relevance.clone(),
                };

                self.scenario_cache.insert(scenario_key, scenario_entry).await;
                entries_created += 1;
            }

            let mut state = self.warming_state.lock().await;
            state.cache_entries_created = entries_created;
        }

        Ok(entries_created)
    }

    /// Optimize cache based on usage patterns
    async fn optimize_cache(&self) -> Result<()> {
        // This would implement cache optimization strategies
        // For now, it's a placeholder for future optimization features
        Ok(())
    }

    /// Get cache warming state
    pub async fn get_warming_state(&self) -> WarmingState {
        self.warming_state.lock().await.clone()
    }

    /// Get pattern statistics
    pub async fn get_statistics(&self) -> PatternStatistics {
        self.statistics.read().await.clone()
    }

    /// Retrieve cached template by pattern
    pub async fn get_template(&self, key: &PatternKey) -> Option<String> {
        if let Some(entry) = self.pattern_cache.get(key).await {
            Some(entry.data)
        } else {
            None
        }
    }

    /// Retrieve cached templates by scenario
    pub async fn get_scenario_templates(&self, key: &ScenarioKey) -> Option<Vec<String>> {
        if let Some(entry) = self.scenario_cache.get(key).await {
            Some(entry.data)
        } else {
            None
        }
    }

    /// Clear all caches
    pub async fn clear_cache(&self) {
        self.pattern_cache.invalidate_all();
        self.scenario_cache.invalidate_all();

        let mut state = self.warming_state.lock().await;
        state.cache_entries_created = 0;
        state.patterns_discovered = 0;
    }

    /// Check if cache contains specific pattern
    pub async fn contains_pattern(&self, key: &PatternKey) -> bool {
        self.pattern_cache.contains_key(key)
    }

    /// Get cache metrics
    pub async fn get_cache_metrics(&self) -> CacheMetrics {
        CacheMetrics {
            pattern_cache_size: self.pattern_cache.entry_count(),
            scenario_cache_size: self.scenario_cache.entry_count(),
            pattern_cache_hits: 0, // Would need to track this
            pattern_cache_misses: 0,
            total_operations: 0,
        }
    }
}

/// Result of analyzing a single file
struct FileAnalysisResult {
    language: String,
    total_lines: u64,
    total_patterns: u64,
    patterns_by_type: HashMap<String, u64>,
}

/// Template generation result
struct TemplateGeneration {
    pattern_type: String,
    template: String,
    frequency_score: f64,
    scenario_relevance: ScenarioRelevance,
}

/// Report from cache warming process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingReport {
    /// Number of files scanned
    pub files_scanned: usize,
    /// Number of patterns discovered
    pub patterns_discovered: u64,
    /// Number of cache entries created
    pub cache_entries_created: usize,
    /// Pattern statistics
    pub statistics: PatternStatistics,
    /// Timestamp of warm-up completion
    pub timestamp: DateTime<Utc>,
}

/// Cache metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Current size of pattern cache
    pub pattern_cache_size: u64,
    /// Current size of scenario cache
    pub scenario_cache_size: u64,
    /// Pattern cache hits
    pub pattern_cache_hits: u64,
    /// Pattern cache misses
    pub pattern_cache_misses: u64,
    /// Total cache operations
    pub total_operations: u64,
}

impl Default for PatternStatistics {
    fn default() -> Self {
        Self {
            total_patterns: 0,
            total_lines: 0,
            frequency_distribution: HashMap::new(),
            avg_complexity: 0.0,
            top_patterns: Vec::new(),
            language_stats: HashMap::new(),
        }
    }
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            max_files_to_scan: 1000,
            frequency_threshold: 5,
            enable_ai_recognition: false,
            prioritized_scenarios: vec![
                DevelopmentScenario::WebDevelopment,
                DevelopmentScenario::SystemDevelopment,
                DevelopmentScenario::ApiDevelopment,
            ],
            project_context: ProjectContext::Library,
        }
    }
}

/// Simple template cache for performance optimization
#[derive(Clone)]
pub struct TemplateCache {
    templates: Arc<RwLock<HashMap<String, String>>>,
    is_warmed_up: Arc<RwLock<bool>>,
}

impl TemplateCache {
    /// Create a new template cache
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            is_warmed_up: Arc::new(RwLock::new(false)),
        }
    }

    /// Warm up the cache by preloading templates from the project root
    pub async fn warmup(&self, project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut warmed_up = self.is_warmed_up.write().await;
        if *warmed_up {
            return Ok(()); // Already warmed up
        }

        let mut templates = self.templates.write().await;
        let templates_dir = project_root.join("src").join("templates");

        if templates_dir.exists() {
            self.load_templates_from_directory(&templates_dir, &mut templates).await?;
        }

        *warmed_up = true;
        Ok(())
    }

    /// Load templates from directory recursively
    async fn load_templates_from_directory(
        &self,
        dir: &Path,
        templates: &mut HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                // Recursively load from subdirectories
                Box::pin(self.load_templates_from_directory(&path, templates)).await?;
            } else if let Some(extension) = path.extension() {
                if extension == "hbs" || extension == "rs" {
                    let file_name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    match tokio::fs::read_to_string(&path).await {
                        Ok(content) => {
                            templates.insert(file_name, content);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to read template {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a template by name
    pub async fn get_template(&self, name: &str) -> Option<String> {
        let templates = self.templates.read().await;
        templates.get(name).cloned()
    }

    /// Check if a template exists
    pub async fn has_template(&self, name: &str) -> bool {
        let templates = self.templates.read().await;
        templates.contains_key(name)
    }

    /// Get all template names
    pub async fn template_names(&self) -> Vec<String> {
        let templates = self.templates.read().await;
        templates.keys().cloned().collect()
    }

    /// Check if cache is warmed up
    pub async fn is_warmed_up(&self) -> bool {
        *self.is_warmed_up.read().await
    }
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}