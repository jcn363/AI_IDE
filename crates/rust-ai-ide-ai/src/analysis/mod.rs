//! Analysis module organization
//!
//! This module provides a structured organization of analysis components:
//! - `types`: Core types and data structures for analysis
//! - `architectural`: Advanced architectural analysis patterns
//! - `security`: Enhanced security pattern detection
//! - `performance`: Performance-specific analyzers
//! - `metrics`: Code metrics calculation and analysis
//! - `cache`: Caching layer for analysis results
//! - `incremental`: Incremental analysis to only process changed files
//! - `utils`: Utility functions for analysis
//!
//! The analysis system is designed to be efficient, with built-in support for:
//! - Incremental analysis to only process changed files
//! - Caching of analysis results with TTL
//! - Parallel processing of independent files
//! - Dependency tracking for accurate change detection

pub mod types;
pub mod architectural;
pub use architectural::*;
pub use types::*;

#[path = "security/mod.rs"]
mod security_mod;
pub use security_mod as security;
pub mod performance;
pub mod metrics;
pub mod cache;
pub mod incremental;
// Re-export key types from submodules for convenience
pub use metrics::{
    CodeMetrics,
    MetricsCalculator,
};

pub use performance::{
    PerformanceAnalyzer,
    PerformanceIssue,
    PerformanceIssueSeverity,
};

pub use architectural::{
    CircularDependencyAnalyzer,
    LayerViolationDetector,
    InterfaceSegregationAnalyzer,
    DependencyInversionAnalyzer,
};

pub use security::{
    SecurityAnalyzer,
    AdvancedPatternDetector,
    CryptographicAnalyzer,
    InputValidationAnalyzer,
    ConcurrencySecurityAnalyzer,
    SecurityIssue,
    SecurityIssueType,
    Severity as SecuritySeverity,
};

pub use performance::{
    IteratorChainAnalyzer,
    MemoryAllocationAnalyzer,
    AsyncPerformanceAnalyzer,
    AlgorithmicComplexityAnalyzer,
};

pub use metrics::{
    CyclomaticComplexity,
    MaintainabilityIndex,
    HalsteadMetrics,
};


// Standard library imports
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

// External crate imports
use anyhow::{Context, Result};
use log::{debug, error, info, trace};
use num_cpus;
use rayon::prelude::*;
use regex;
use serde::{Deserialize, Serialize};
use syn;
use walkdir::WalkDir;

// Internal module imports
use crate::analysis::cache::{CacheKey, get_cached, set_cached, invalidate_cache};

// Types are defined in the types module and re-exported above
// This eliminates type conflicts and provides clean separation

// Re-export utility functions
pub use utils::{
    create_finding,
    extract_line_number,
    span_to_range,
    meets_confidence_threshold,
    merge_ranges,
};

// CodeLocation is now defined in the types module and re-exported

// Range, Severity, and AnalysisCategory are now defined in the types module

/// Main analysis result type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFinding {
    /// The type of the finding
    pub kind: String,
    /// Description of the finding
    pub message: String,
    /// The actual finding data as a JSON string
    pub data: String,
    /// Severity level
    pub severity: Severity,
    /// Category of the finding
    pub category: AnalysisCategory,
    /// Location in the source code
    pub location: String,
    /// Code range where the finding was detected
    pub range: Range,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Unique identifier for the rule that generated this finding
    pub rule_id: String,
}

impl From<CodeSmell> for AnalysisFinding {
    fn from(smell: CodeSmell) -> Self {
        let range = Range {
            start_line: smell.span.line,
            end_line: smell.span.line,
            start_col: smell.span.column,
            end_col: smell.span.column + 1, // Approximate end column
        };
        
        Self {
            kind: "code_smell".to_string(),
            message: smell.description,
            data: serde_json::to_string(&smell).unwrap_or_default(),
            severity: Severity::Warning,
            category: AnalysisCategory::CodeSmell,
            location: format!("{}:{}", smell.span.file, smell.span.line),
            range,
            suggestion: Some(smell.suggestion),
            confidence: 0.9, // High confidence for code smells
            rule_id: format!("CS{:04}", 1000 + smell.span.line as u32),
        }
    }
}

impl From<SecurityIssue> for AnalysisFinding {
    fn from(issue: SecurityIssue) -> Self {
        let range = Range {
            start_line: issue.span.line,
            end_line: issue.span.line,
            start_col: issue.span.column,
            end_col: issue.span.column + 1,
        };
        
        Self {
            kind: "security_issue".to_string(),
            message: issue.description,
            data: serde_json::to_string(&issue).unwrap_or_default(),
            severity: Severity::Error,
            category: AnalysisCategory::Security,
            location: format!("{}:{}", issue.span.file, issue.span.line),
            range,
            suggestion: issue.suggestion,
            confidence: 0.9, // High confidence for security issues
            rule_id: format!("SEC{:04}", 1000 + issue.span.line as u32),
        }
    }
}

impl From<PerformanceIssue> for AnalysisFinding {
    fn from(issue: PerformanceIssue) -> Self {
        let range = Range {
            start_line: issue.span.line,
            end_line: issue.span.line,
            start_col: issue.span.column,
            end_col: issue.span.column + 1,
        };
        
        Self {
            kind: "performance_issue".to_string(),
            message: issue.description,
            data: serde_json::to_string(&issue).unwrap_or_default(),
            severity: Severity::Warning,
            category: AnalysisCategory::Performance,
            location: format!("{}:{}", issue.span.file, issue.span.line),
            range,
            suggestion: issue.suggestion,
            confidence: 0.8, // Medium-high confidence for performance issues
            rule_id: format!("PERF{:04}", 1000 + issue.span.line as u32),
        }
    }
}

impl From<StyleViolation> for AnalysisFinding {
    fn from(violation: StyleViolation) -> Self {
        let range = Range {
            start_line: violation.span.line,
            end_line: violation.span.line,
            start_col: violation.span.column,
            end_col: violation.span.column + 1,
        };
        
        Self {
            kind: "style_violation".to_string(),
            message: violation.description,
            data: serde_json::to_string(&violation).unwrap_or_default(),
            severity: Severity::Info,
            category: AnalysisCategory::Style,
            location: format!("{}:{}", violation.span.file, violation.span.line),
            range,
            suggestion: Some(violation.suggestion),
            confidence: 0.95, // Very high confidence for style violations
            rule_id: format!("STYLE{:04}", 1000 + violation.span.line as u32),
        }
    }
}

/// Individual analysis finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFindingOld {
    /// Description of the finding
    pub message: String,
    /// Severity level
    pub severity: Severity,
    /// Category of the finding
    pub category: AnalysisCategory,
    /// Location in the code
    pub range: Range,
    /// Suggested fix (if any)
    pub suggestion: Option<String>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Unique identifier for the rule that triggered this finding
    pub rule_id: String,
}

/// Analysis preferences that control analyzer behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPreferences {
    /// Enable architectural analysis
    pub enable_architecture: bool,
    /// Enable security analysis
    pub enable_security: bool,
    /// Enable performance analysis
    pub enable_performance: bool,
    /// Enable metrics calculation
    pub enable_code_style: bool,
    /// Enable code smell analysis
    pub enable_code_smells: bool,
    /// Whether to use incremental analysis
    pub incremental_analysis: bool,
    /// Minimum confidence threshold for findings (0.0 to 1.0)
    pub min_confidence: f32,
    /// Whether to include suggestions in findings
    pub include_suggestions: bool,
}

impl Default for AnalysisPreferences {
    fn default() -> Self {
        Self {
            enable_architecture: true,
            enable_security: true,
            enable_performance: true,
            enable_code_style: true,
            enable_code_smells: true,
            incremental_analysis: true,
            min_confidence: 0.7,
            include_suggestions: true,
        }
    }
}

/// Trait for all analysis components to ensure consistent interface
#[typetag::serde(tag = "type")]
pub trait Analyzer: std::fmt::Debug + Send + Sync + 'static {
    /// The type of findings this analyzer produces
    type Finding: Finding + 'static;  
    /// Analyze the given code and return findings
    fn analyze(&self, ast: &syn::File, code: &str, file_path: &str) -> Result<Vec<Self::Finding>>;
    
    /// Get the name/identifier of this analyzer
    fn name(&self) -> &'static str;
    
    /// Get the category of analysis this analyzer performs
    fn category(&self) -> AnalysisCategory;
    
    /// Check if this analyzer is enabled based on preferences
    fn is_enabled(&self, preferences: &AnalysisPreferences) -> bool {
        match self.category() {
            AnalysisCategory::CodeSmell => preferences.enable_code_smells,
            AnalysisCategory::Security => preferences.enable_security,
            AnalysisCategory::Performance => preferences.enable_performance,
            AnalysisCategory::Style => preferences.enable_code_style,
            AnalysisCategory::Architecture => preferences.enable_architecture,
        }
    }
    
    /// Run the analyzer if it's enabled
    fn run_if_enabled(&self, ast: &syn::File, code: &str, file_path: &str, preferences: &AnalysisPreferences) -> Result<Vec<Self::Finding>> {
        if self.is_enabled(preferences) {
            self.analyze(ast, code, file_path)
        } else {
            Ok(Vec::new())
        }
    }
}

/// Configuration for analysis modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Whether caching is enabled
    #[serde(default = "default_true")]
    pub cache_enabled: bool,
    /// Time-to-live for cached results in seconds (None = no expiration)
    #[serde(default)]
    pub cache_ttl_seconds: Option<u64>,
    /// Whether to enable incremental analysis
    #[serde(default = "default_true")]
    pub incremental_analysis: bool,
    /// Maximum number of files to analyze in parallel
    #[serde(default = "default_max_parallel_files")]
    pub max_parallel_files: usize,
    /// File patterns to include in analysis
    #[serde(default = "default_include_patterns")]
    pub include_patterns: Vec<String>,
    /// Version of the analysis configuration (bump this to invalidate all caches)
    #[serde(default = "default_version")]
    pub version: u32,
    /// Minimum confidence threshold for findings (0.0 - 1.0)
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
    /// Custom analysis rules
    #[serde(default)]
    pub custom_rules: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool { true }
fn default_max_parallel_files() -> usize { num_cpus::get().max(1) }
fn default_include_patterns() -> Vec<String> { vec![r"\.rs$".to_string()] }
fn default_version() -> u32 { 1 }
fn default_min_confidence() -> f32 { 0.7 }

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_ttl_seconds: Some(3600 * 24 * 7), // 1 week
            version: 1,
            min_confidence: 0.7,
            incremental_analysis: true,
            max_parallel_files: num_cpus::get(),
            include_patterns: vec![r"\.rs$".to_string()],
            custom_rules: HashMap::new(),
        }
    }
}

impl AnalysisConfig {
    /// Generate a hash of the configuration for cache invalidation
    pub fn get_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.version.hash(&mut hasher);
        self.min_confidence.to_bits().hash(&mut hasher);
        self.cache_enabled.hash(&mut hasher);
        self.cache_ttl_seconds.hash(&mut hasher);
        self.incremental_analysis.hash(&mut hasher);
        self.max_parallel_files.hash(&mut hasher);
        
        // Sort keys for consistent hashing
        let mut keys: Vec<_> = self.custom_rules.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(&mut hasher);
            // Note: We don't hash the values as they might contain complex structures
            // that don't implement Hash. The version bump should be enough to
            // invalidate the cache when rules change.
        }
        
        hasher.finish()
    }
}

/// Registry for managing analysis components with incremental analysis support
#[derive(Debug)]
pub struct AnalysisRegistry {
    architectural_analyzers: Vec<Box<dyn Analyzer<Finding = AnalysisFinding> + Send + Sync>>,
    security_analyzers: Vec<Box<dyn Analyzer<Finding = SecurityIssue> + Send + Sync>>,
    performance_analyzers: Vec<Box<dyn Analyzer<Finding = PerformanceIssue> + Send + Sync>>,
    metrics_calculators: Vec<Box<dyn Analyzer<Finding = CodeMetrics> + Send + Sync>>,
    config: AnalysisConfig,
    incremental_state: Option<Arc<RwLock<incremental::IncrementalState>>>,
    preferences: AnalysisPreferences,
}

impl AnalysisRegistry {
    /// Creates a new analysis registry with default analyzers
    pub fn new() -> Result<Self> {
        let config = AnalysisConfig::default();
        let preferences = AnalysisPreferences::default();
        let config_hash = format!("{:x}", md5::compute(format!("{:?}", preferences)));
        let incremental_state = if preferences.incremental_analysis {
            Some(Arc::new(RwLock::new(incremental::IncrementalState::new(&config_hash))))
        } else {
            None
        };

        Ok(Self {
            architectural_analyzers: Vec::new(),
            security_analyzers: Vec::new(),
            performance_analyzers: Vec::new(),
            metrics_calculators: Vec::new(),
            config,
            incremental_state,
            preferences: AnalysisPreferences::default(),
        })
    }

    /// Register an architectural analyzer
    pub fn register_architectural_analyzer<A>(&mut self, analyzer: A)
    where
        A: Analyzer<Finding = CodeSmell> + Send + Sync + 'static,
    {
        self.architectural_analyzers.push(Box::new(analyzer));
    }

    /// Register a security analyzer
    pub fn register_security_analyzer<A>(&mut self, analyzer: A)
    where
        A: Analyzer<Finding = SecurityIssue> + Send + Sync + 'static,
    {
        self.security_analyzers.push(Box::new(analyzer));
    }

    /// Register a performance analyzer
    pub fn register_performance_analyzer<A>(&mut self, analyzer: A)
    where
        A: Analyzer<Finding = PerformanceIssue> + Send + Sync + 'static,
    {
        self.performance_analyzers.push(Box::new(analyzer));
    }

    /// Register a metrics calculator
    pub fn register_metrics_calculator<A>(&mut self, calculator: A)
    where
        A: Analyzer<Finding = CodeMetrics> + Send + Sync + 'static,
    {
        self.metrics_calculators.push(Box::new(calculator));
    }

    /// Run analysis on a single file with caching and incremental analysis
    pub fn analyze_file(
        &self,
        ast: &syn::File,
        code: &str,
        file_path: &str,
        preferences: &AnalysisPreferences,
    ) -> Result<(Vec<AnalysisFinding>, Vec<CodeMetrics>)> {
        let path = Path::new(file_path);
        
        // Check if we should skip analysis due to incremental mode
        if self.config.incremental_analysis {
            if let Some(state) = &self.incremental_state {
                if !state.read().unwrap().needs_analysis(file_path) {
                    debug!("Skipping unchanged file in incremental mode: {}", file_path);
                    return Ok((Vec::new(), Vec::new()));
                }
            }
        }

        // Check if we can use cached results
        if self.config.cache_enabled {
            let cache_key = self.generate_cache_key(code, file_path)?;
            if let Some(cached) = self.get_cached_results(&cache_key)? {
                debug!("Using cached analysis results for {}", file_path);
                
                // Update incremental state to mark this file as analyzed
                if self.config.incremental_analysis {
                    if let Some(state) = &self.incremental_state {
                        let _ = state.write().unwrap().update_file_analysis(path, Vec::new(), true);
                    }
                }
                
                return Ok(cached);
            }
        }

        // Run the actual analysis
        let (findings, metrics) = self.run_analysis(ast, code, file_path, preferences);

        // Update incremental state if enabled
        if self.config.incremental_analysis {
            if let Some(incr_state) = &self.incremental_state {
                if let Err(e) = incr_state.write().unwrap().update_file_analysis(
                    Path::new(file_path),
                    Vec::new(),
                    true,
                ) {
                    error!("Failed to update incremental state for {}: {}", file_path, e);
                }
            }
        }

        // Cache the results if enabled
        if self.config.cache_enabled {
            let cache_key = self.generate_cache_key(code, file_path)?;
            if let Err(e) = self.cache_results(&cache_key, &findings, &metrics) {
                error!("Failed to cache analysis results for {}: {}", file_path, e);
            }
        }

        Ok((findings, metrics))
    }

    /// Run analysis on a directory with incremental analysis
    pub fn analyze_directory(
        &self,
        dir_path: &str,
        preferences: &AnalysisPreferences,
    ) -> Result<Vec<(String, Vec<AnalysisFinding>, Vec<CodeMetrics>)>> {
        let mut results = Vec::new();
        
        // Get all files matching the include patterns
        let files = self.find_files_to_analyze(dir_path)?;
        
        // Filter files based on incremental analysis if enabled
        let files_to_analyze: Vec<_> = if self.config.incremental_analysis {
            if let Some(state) = &self.incremental_state {
                files.into_par_iter()
                    .filter(|path| {
                        let needs_analysis = state.read().unwrap().needs_analysis(&path.to_string_lossy());
                        if !needs_analysis {
                            debug!("Skipping unchanged file in incremental mode: {}", path.display());
                        }
                        needs_analysis
                    })
                    .collect()
            } else {
                files
            }
        } else {
            files
        };
        
        info!("Analyzing {} out of {} files ({} skipped due to incremental analysis)",
            files_to_analyze.len(),
            files.len(),
            files.len() - files_to_analyze.len()
        );
        
        // Process files in parallel with progress reporting
        let pb = utils::progress_bar(files_to_analyze.len() as u64, "Analyzing files");
        
        results = files_to_analyze
            .par_iter()
            .filter_map(|file_path| {
                pb.inc(1);
                match self.analyze_file_path(file_path, preferences) {
                    Ok((findings, metrics)) => {
                        // Update incremental state for successfully analyzed files
                        if self.config.incremental_analysis {
                            if let Some(state) = &self.incremental_state {
                                let _ = state.write().unwrap().update_file_analysis(file_path, Vec::new(), true);
                            }
                        }
                        Some((
                            file_path.to_string_lossy().into_owned(),
                            findings,
                            metrics,
                        ))
                    }
                    Err(e) => {
                        error!("Failed to analyze {}: {}", file_path.display(), e);
                        None
                    }
                }
            })
            .collect();
        
        pb.finish_with_message("Analysis complete");
        
        // Save incremental state if enabled
        if self.config.incremental_analysis {
            if let Some(state) = &self.incremental_state {
                if let Err(e) = state.write().unwrap().save(&self.config) {
                    error!("Failed to save incremental state: {}", e);
                }
            }
        }
        
        Ok(results)
    }

    /// Analyze a file at the given path
    fn analyze_file_path(
        &self,
        file_path: &Path,
        preferences: &AnalysisPreferences,
    ) -> Result<(Vec<AnalysisFinding>, Vec<CodeMetrics>)> {
        let code = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        
        let ast = syn::parse_file(&code)
            .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;
        
        self.analyze_file(&ast, &code, &file_path.to_string_lossy(), preferences)
    }

    /// Find all files that should be analyzed in the given directory
    fn find_files_to_analyze(&self, dir_path: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if self.should_analyze_file(path) {
                files.push(path.to_path_buf());
            }
        }
        
        Ok(files)
    }
    
    /// Check if a file should be analyzed based on include patterns
    fn should_analyze_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Check if the file matches any of the include patterns
        self.config.include_patterns.iter().any(|pattern| {
            regex::Regex::new(pattern)
                .map(|re| re.is_match(&path_str))
                .unwrap_or(false)
        })
    }

    /// Generate a cache key for the given file and code
    fn generate_cache_key(&self, code: &str, file_path: &str) -> Result<CacheKey> {
        let config_hash = self.config.get_hash();
        let cache_key = format!(
            "analysis_v{}_{:x}",
            self.config.version, config_hash
        );
        
        CacheKey::new(
            Path::new(file_path),
            &cache_key,
            self.config.version,
            code,
            &self.config,
        )
    }

    /// Get cached analysis results
    fn get_cached_results(
        &self,
        key: &CacheKey,
    ) -> Result<Option<(Vec<AnalysisFinding>, Vec<CodeMetrics>)>> {
        if !self.config.cache_enabled {
            return Ok(None);
        }
        
        // Check if the cache entry is valid
        if key.is_stale() {
            return Ok(None);
        }
        
        // Try to get from cache
        match cache::get_cached(key) {
            Ok(Some(result)) => {
                debug!("Cache hit for key: {}", key.key());
                Ok(Some(result))
            }
            Ok(None) => {
                trace!("Cache miss for key: {}", key.key());
                Ok(None)
            }
            Err(e) => {
                error!("Cache error for key {}: {}", key.key(), e);
                Ok(None)
            }
        }
    }

    /// Cache analysis results
    fn cache_results(
        &self,
        key: &CacheKey,
        findings: &[AnalysisFinding],
        metrics: &[CodeMetrics],
    ) -> Result<()> {
        if !self.config.cache_enabled {
            return Ok(());
        }
        
        let metadata = serde_json::json!({
            "version": self.config.version,
            "findings_count": findings.len(),
            "metrics_count": metrics.len(),
            "cached_at": chrono::Utc::now().to_rfc3339(),
        });
        
        cache::set_cached(
            key,
            &(findings, metrics),
            self.config.cache_ttl_seconds.map(Duration::from_secs),
            Some(metadata),
        )?;

        debug!("Cached results for {} with key {}", key.file_path().display(), key.key());
        
        Ok(())
    }

    /// Invalidate cache for files matching a pattern
    pub fn invalidate_cache(&self, pattern: &str) -> Result<usize> {
        debug!("Invalidating cache for pattern: {}", pattern);
        let count = cache::invalidate_cache(pattern)?;
        
        // Also clean up any expired entries
        let expired = cache::cleanup_expired()?;
        debug!("Cleaned up {} expired cache entries", expired);
        
        Ok(count)
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> Result<cache::CacheStats> {
        cache::get_stats()
    }
    
    /// Clear the entire cache
    pub fn clear_cache(&self) -> Result<usize> {
        self.invalidate_cache(".*")
    }
    
    /// Run all analyzers on the given code and return findings and metrics
    fn run_analysis(
        &self,
        ast: &syn::File,
        code: &str,
        file_path: &str,
        preferences: &AnalysisPreferences,
    ) -> (Vec<AnalysisFinding>, Vec<CodeMetrics>) {
        let mut all_findings = Vec::new();
        let mut all_metrics = Vec::new();
        
        // Run architectural analyzers
        if !self.architectural_analyzers.is_empty() {
            self.run_analyzers::<dyn Analyzer<Finding = AnalysisFinding> + Send + Sync>(
                &self.architectural_analyzers,
                ast,
                code,
                file_path,
                preferences,
                &mut all_findings,
            );
        }
        
        // Run security analyzers
        if !self.security_analyzers.is_empty() {
            let mut security_findings = Vec::new();
            self.run_analyzers::<dyn Analyzer<Finding = SecurityIssue> + Send + Sync>(
                &self.security_analyzers,
                ast,
                code,
                file_path,
                preferences,
                &mut security_findings,
            );
            all_findings.extend(security_findings.into_iter().map(Into::into));
        }
        
        // Run performance analyzers
        if !self.performance_analyzers.is_empty() {
            let mut performance_findings = Vec::new();
            self.run_analyzers::<dyn Analyzer<Finding = PerformanceIssue> + Send + Sync>(
                &self.performance_analyzers,
                ast,
                code,
                file_path,
                preferences,
                &mut performance_findings,
            );
            all_findings.extend(performance_findings.into_iter().map(Into::into));
        }
        
        // Run metrics calculators
        if !self.metrics_calculators.is_empty() {
            let mut metrics = Vec::new();
            self.run_analyzers::<dyn Analyzer<Finding = CodeMetrics> + Send + Sync>(
                &self.metrics_calculators,
                ast,
                code,
                file_path,
                preferences,
                &mut metrics,
            );
            all_metrics = metrics;
        }

        // Update incremental state if enabled
        if preferences.incremental_analysis {
            if let Some(incr_state) = &self.incremental_state {
                if let Err(e) = incr_state.write().unwrap().update_file_analysis(
                    Path::new(file_path),
                    Vec::new(),
                    true,
                ) {
                    error!("Failed to update incremental state for {}: {}", file_path, e);
                }
            }
        }

        (all_findings, all_metrics)
    }

    /// Run analyzers on the given code and return findings
    fn run_analyzers<F: Analyzer + ?Sized + Send + Sync + 'static>(
        &self,
        analyzers: &[Box<dyn Analyzer<Finding = F::Finding> + Send + Sync>],
        ast: &syn::File,
        code: &str,
        file_path: &str,
        preferences: &AnalysisPreferences,
        results: &mut Vec<F::Finding>,
    ) where
        F::Finding: Send + Sync + 'static,
    {
        for analyzer in analyzers {
            match analyzer.run_if_enabled(ast, code, file_path, preferences) {
                Ok(mut analyzer_findings) => {
                    results.append(&mut analyzer_findings);
                }
                Err(e) => {
                    // Log the error but continue with other analyzers
                    error!("Error running analyzer on {}: {}", file_path, e);
                }
            }
        }
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: AnalysisConfig) -> Result<()> {
        // If cache TTL changed, update the cache system
        if self.config.cache_ttl_seconds != config.cache_ttl_seconds {
            if let Some(ttl) = config.cache_ttl_seconds {
                cache::init_cache(None, Some(Duration::from_secs(ttl)))?;
            }
        }
        
        // If the config changed significantly, we might need to invalidate the cache
        if self.config.get_hash() != config.get_hash() {
            debug!("Configuration changed significantly, reloading incremental state");
            self.incremental_state = if config.incremental_analysis {
                Some(Arc::new(RwLock::new(incremental::IncrementalState::load(&config)?)))
            } else {
                None
            };
        }
        
        self.config = config;
        Ok(())
    }
}

impl Default for AnalysisRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create default AnalysisRegistry")
    }
}

impl AnalysisRegistry {
    /// Update analysis preferences
    pub fn update_preferences(&mut self, preferences: AnalysisPreferences) {
        self.preferences = preferences;
    }
    
        /// Update analysis provider
    pub fn update_provider(&mut self, provider: impl Analyzer<Finding = AnalysisFinding> + Send + Sync + 'static) {
        self.architectural_analyzers.push(Box::new(provider));
    }
}

/// Utility functions for analysis modules
pub mod utils {
    use crate::analysis::{AnalysisCategory, AnalysisFinding, Range, Severity};

    /// Create a standard analysis finding
    pub fn create_finding(
        message: String,
        severity: Severity,
        category: AnalysisCategory,
        range: Range,
        suggestion: Option<String>,
        confidence: f32,
        rule_id: String,
    ) -> AnalysisFinding {
        let location = format!("{}:{}", range.start_line, range.start_col);
        
        AnalysisFinding {
            kind: category.to_string().to_lowercase(),
            message,
            data: String::new(),
            severity,
            category,
            location,
            range,
            suggestion,
            confidence: confidence.clamp(0.0, 1.0),
            rule_id,
        }
    }
    
    /// Extract line number from syn span
    pub fn extract_line_number(span: &proc_macro2::Span) -> u32 {
        let source_text = span.unwrap().source_text().unwrap_or_default();
        source_text.lines().count() as u32
    }
    
    /// Create a range from syn span
    pub fn span_to_range(span: &proc_macro2::Span) -> Range {
        // Get the source text position from the span
        let source_text = span.unwrap().source_text().unwrap_or_default();
        
        // For now, return a default range since we can't get accurate positions
        // This will need to be updated if we need accurate positions
        Range {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: source_text.len() as u32 + 1, // Approximate end column
        }
    }
    
    /// Check if a finding meets the confidence threshold
    pub fn meets_confidence_threshold(finding: &AnalysisFinding, threshold: f32) -> bool {
        finding.confidence >= threshold
    }
    
    /// Merge overlapping ranges
    pub fn merge_ranges(ranges: &[Range]) -> Vec<Range> {
        if ranges.is_empty() {
            return Vec::new();
        }
        
        let mut sorted_ranges = ranges.to_vec();
        sorted_ranges.sort_by_key(|r| (r.start_line, r.start_col));
        
        let mut merged = vec![sorted_ranges[0].clone()];
        
        for range in sorted_ranges.iter().skip(1) {
            let last = merged.last_mut().unwrap();
            
            // Check if ranges overlap or are adjacent
            if range.start_line <= last.end_line + 1 {
                // Merge ranges
                last.end_line = last.end_line.max(range.end_line);
                last.end_col = if last.end_line == range.end_line {
                    last.end_col.max(range.end_col)
                } else {
                    range.end_col
                };
            } else {
                // No overlap, add new range
                merged.push(range.clone());
            }
        }
        
        merged
    }
}
