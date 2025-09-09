//! Pattern and Anti-Pattern Detectors
//!
//! This module provides the main detection interfaces and orchestrates
//! the pattern detection, ML scoring, and intelligent suggestions.

use crate::analysis::architectural::{anti_patterns::*, ml_scorer::*, patterns::*};
use crate::analysis::{AnalysisCategory, Severity};
use rust_ai_ide_common::{IdeResult, IdeError};
#[cfg(feature = "caching")]
use rust_ai_ide_cache::{Cache, InMemoryCache, CacheEntry};

#[cfg(not(feature = "caching"))]
use crate::analysis::architectural::dummy_cache::{DummyCache as InMemoryCache, DummyCacheEntry as CacheEntry};

// The Cache trait is now implemented by the actual cache implementations
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main AI-powered code analysis detector
pub struct AIDetector {
    /// ML-enhanced scoring system
    ml_scorer: Arc<RwLock<MLScorer>>,
    /// Anti-pattern detector
    anti_pattern_detector: Arc<RwLock<AntiPatternDetector>>,
    /// Pattern detector for architectural patterns
    pattern_detector: Arc<RwLock<PatternDetector>>,
    /// Analysis cache for performance optimization
    #[cfg(feature = "caching")]
    analysis_cache: Arc<dyn Cache<String, AnalysisResult> + Send + Sync>,
    
    /// Dummy cache implementation when caching is disabled
    #[cfg(not(feature = "caching"))]
    analysis_cache: Arc<InMemoryCache<String, AnalysisResult>>,
    /// Intelligent suggestion generator
    suggestion_generator: Arc<RwLock<IntelligentSuggestionGenerator>>,
}

/// Pattern detection engine
pub struct PatternDetector {
    /// Known architectural patterns
    architectural_patterns: HashMap<String, ArchitecturalPattern>,
}

/// Intelligent suggestion generator with ML enhancement
pub struct IntelligentSuggestionGenerator {
    /// Template-based suggestion generators
    suggestion_templates: HashMap<String, SuggestionTemplate>,
}

/// Template for generating contextual suggestions
#[derive(Debug, Clone)]
pub struct SuggestionTemplate {
    /// Condition that triggers this suggestion
    condition: SuggestionCondition,
    /// Base suggestion text template
    template: String,
    /// Expected benefits
    benefits: Vec<String>,
    /// Refactoring type
    refactoring_type: RefactoringType,
    /// Priority level
    priority: Priority,
}

/// Conditions for triggering suggestions
#[derive(Debug, Clone)]
pub enum SuggestionCondition {
    AntiPatternType(AntiPattern),
    PatternType(ArchitecturalPattern),
    ComplexityThreshold(f32),
    Custom(String),
}

/// Analysis result containing all detected patterns and issues
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub file_path: String,
    pub detected_patterns: Vec<DetectedPattern>,
    pub detected_anti_patterns: Vec<DetectedAntiPattern>,
    pub intelligence_suggestions: Vec<IntelligenceSuggestion>,
    pub analysis_metadata: AnalysisMetadata,
    pub performance_metrics: PerformanceMetrics,
}

/// Metadata about the analysis process
#[derive(Debug, Clone)]
pub struct AnalysisMetadata {
    pub analysis_duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub detector_version: String,
    pub features_used: Vec<String>,
}

/// Performance metrics for the analysis
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub cache_hit_rate: f32,
    pub ml_predictions_count: usize,
    pub patterns_detected: usize,
    pub anti_patterns_detected: usize,
}

impl AIDetector {
    /// Create a new AI detector with default configuration
    pub fn new() -> Self {
        Self::with_config(AIDetectorConfig::default())
    }

    /// Create detector with custom configuration
    pub fn with_config(config: AIDetectorConfig) -> Self {
        let ml_scorer = Arc::new(RwLock::new(MLScorer::with_config(config.ml_scorer_config)));
        let anti_pattern_detector = Arc::new(RwLock::new(AntiPatternDetector::new()));
        let pattern_detector = Arc::new(RwLock::new(PatternDetector::new()));
        let suggestion_generator = Arc::new(RwLock::new(IntelligentSuggestionGenerator::new()));

        #[cfg(feature = "caching")]
        let analysis_cache = {
            use rust_ai_ide_cache::InMemoryCache;
            Arc::new(InMemoryCache::new()) as Arc<dyn Cache<String, AnalysisResult> + Send + Sync>
        };

        #[cfg(not(feature = "caching"))]
        let analysis_cache = Arc::new(InMemoryCache::new());

        Self {
            ml_scorer,
            anti_pattern_detector,
            pattern_detector,
            analysis_cache,
            suggestion_generator,
        }
    }

    /// Perform comprehensive AI-powered code analysis
    pub async fn analyze_code(
        &self,
        content: &str,
        file_path: &str,
        request: AnalysisRequest,
    ) -> IdeResult<AnalysisResult> {
        let start_time = std::time::Instant::now();
        let cache_key = self.generate_cache_key(file_path, content, &request);

        // Check cache first
        if let Some(cached) = self.get_from_cache(&cache_key).await {
            if self.is_cache_valid(&cached, &request) {
                return Ok(cached);
            }
        }

        let mut analysis_result = AnalysisResult {
            file_path: file_path.to_string(),
            detected_patterns: Vec::new(),
            detected_anti_patterns: Vec::new(),
            intelligence_suggestions: Vec::new(),
            analysis_metadata: AnalysisMetadata {
                analysis_duration_ms: 0,
                timestamp: chrono::Utc::now(),
                detector_version: env!("CARGO_PKG_VERSION").to_string(),
                features_used: Vec::new(),
            },
            performance_metrics: PerformanceMetrics {
                cache_hit_rate: 0.0,
                ml_predictions_count: 0,
                patterns_detected: 0,
                anti_patterns_detected: 0,
            },
        };

        // Detect anti-patterns
        if request.detect_anti_patterns {
            let mut anti_patterns = self.anti_pattern_detector.write().await;
            let detected = anti_patterns.analyze_code(
                content,
                file_path,
                request.parse_tree.as_ref(),
            )?;

            // Score anti-patterns with ML
            let mut scored_anti_patterns = Vec::new();
            {
                let mut ml_scorer = self.ml_scorer.write().await;
                for mut anti_pattern in detected {
                    anti_pattern.confidence = ml_scorer.score_anti_pattern(&anti_pattern).await?;
                    scored_anti_patterns.push(anti_pattern);
                }
            }

            analysis_result.detected_anti_patterns = scored_anti_patterns;
            analysis_result.performance_metrics.anti_patterns_detected =
                analysis_result.detected_anti_patterns.len();
        }

        // Detect architectural patterns
        if request.detect_patterns {
            let patterns = self.pattern_detector.read().await;
            let detected_patterns = patterns.analyze_patterns(content, file_path, &request)?;

            // Score patterns with ML
            let mut scored_patterns = Vec::new();
            {
                let mut ml_scorer = self.ml_scorer.write().await;
                for mut pattern in detected_patterns {
                    pattern.confidence = ml_scorer.score_pattern(&pattern).await?;
                    scored_patterns.push(pattern);
                }
            }

            analysis_result.detected_patterns = scored_patterns;
            analysis_result.performance_metrics.patterns_detected =
                analysis_result.detected_patterns.len();
        }

        // Generate intelligent suggestions
        let suggestions = self.generate_suggestions(&analysis_result, &request).await?;
        analysis_result.intelligence_suggestions = suggestions;

        // Calculate analysis duration
        let duration = start_time.elapsed();
        analysis_result.analysis_metadata.analysis_duration_ms = duration.as_millis() as u64;

        // Cache the result
        let cache_entry = CacheEntry::new(analysis_result.clone());
        let _ = cache.insert(cache_key, analysis_result.clone(), None).await;

        Ok(analysis_result)
    }

    /// Generate intelligent suggestions based on analysis results
    pub async fn generate_suggestions(
        &self,
        analysis_result: &AnalysisResult,
        request: &AnalysisRequest,
    ) -> IdeResult<Vec<IntelligenceSuggestion>> {
        let mut suggestions = Vec::new();
        let suggestion_gen = self.suggestion_generator.read().await;

        // Generate suggestions from anti-patterns
        for anti_pattern in &analysis_result.detected_anti_patterns {
            if let Some(suggestion) = suggestion_gen.generate_from_anti_pattern(anti_pattern)? {
                suggestions.push(suggestion);
            }
        }

        // Generate suggestions from patterns
        for pattern in &analysis_result.detected_patterns {
            if let Some(suggestion) = suggestion_gen.generate_from_pattern(pattern)? {
                suggestions.push(suggestion);
            }
        }

        // Enhance suggestions with ML
        {
            let mut ml_scorer = self.ml_scorer.write().await;
            for suggestion in &mut suggestions {
                ml_scorer.enhance_suggestion(suggestion).await?;
            }
        }

        // Prioritize suggestions
        suggestions.sort_by(|a, b| {
            // Sort by priority first, then by confidence
            let priority_order = |p: &Priority| match p {
                Priority::Critical => 0,
                Priority::High => 1,
                Priority::Medium => 2,
                Priority::Low => 3,
                Priority::Info => 4,
            };

            let priority_cmp = priority_order(&a.priority).cmp(&priority_order(&b.priority));
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }

            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(suggestions)
    }

    /// Apply intelligent fix for a suggestion
    pub async fn apply_intelligent_fix(
        &self,
        suggestion: &IntelligenceSuggestion,
    ) -> IdeResult<Option<String>> {
        match &suggestion.automated_fix {
            Some(automated_fix) => {
                // For now, return a placeholder - would implement actual code transformation
                let transformed_code = self.generate_fix_code(suggestion, automated_fix)?;
                Ok(Some(transformed_code))
            }
            None => Ok(None),
        }
    }

    /// Get analysis performance metrics
    pub async fn get_performance_metrics(&self) -> IdeResult<AnalysisPerformanceStats> {
        #[cfg(feature = "caching")]
        let cache_stats = self.analysis_cache.stats().await;

        Ok(AnalysisPerformanceStats {
            #[cfg(feature = "caching")]
            cache_hit_rate: cache_stats.hit_ratio,
            #[cfg(not(feature = "caching"))]
            cache_hit_rate: 0.0,
            
            #[cfg(feature = "caching")]
            total_analyses: cache_stats.total_sets as usize,
            #[cfg(not(feature = "caching"))]
            total_analyses: 0,
            
            average_analysis_time_ms: 100.0, // Would calculate from actual data
            ml_predictions_processed: 0,     // Would track from ML scorer
            
            #[cfg(feature = "caching")]
            cache_size_entries: cache_stats.total_entries as usize,
            #[cfg(not(feature = "caching"))]
            cache_size_entries: 0,
            
            #[cfg(feature = "caching")]
            uptime_seconds: cache_stats.uptime_seconds,
            #[cfg(not(feature = "caching"))]
            uptime_seconds: 0,
        })
    }

    /// Generate cache key for analysis request
    fn generate_cache_key(&self, file_path: &str, content: &str, request: &AnalysisRequest) -> String {
        // Simple hash-based key generation
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        file_path.hash(&mut hasher);
        content.hash(&mut hasher);
        format!("analysis:{:x}", hasher.finish())
    }

    /// Get a value from the cache
    async fn get_from_cache(&self, key: &str) -> Option<AnalysisResult> {
        #[cfg(feature = "caching")]
        {
            self.analysis_cache.get(key).await
        }
        
        #[cfg(not(feature = "caching"))]
        {
            self.analysis_cache.get(key).cloned()
        }
    }
    
    /// Store a value in the cache
    async fn store_in_cache(&self, key: String, value: AnalysisResult) {
        #[cfg(feature = "caching")]
        {
            self.analysis_cache.set(key, value).await;
        }
        
        #[cfg(not(feature = "caching"))]
        {
            self.analysis_cache.insert(key, value);
        }
    }

    /// Check if cached result is still valid
    fn is_cache_valid(&self, cached: &AnalysisResult, request: &AnalysisRequest) -> bool {
        // Simple cache validation - could be enhanced with file modification times
        let cache_age = chrono::Utc::now().signed_duration_since(cached.analysis_metadata.timestamp);
        cache_age < chrono::Duration::minutes(30) // 30 minute cache validity
    }

    /// Generate code for automated fix (placeholder implementation)
    fn generate_fix_code(&self, _suggestion: &IntelligenceSuggestion, _fix: &AutomatedFix) -> IdeResult<String> {
        // This would implement the actual code transformation logic
        // For now, return a placeholder comment
        Ok("// AI-generated fix applied\n// TODO: Implement actual code transformation".to_string())
    }
}

/// Configuration for the AI detector
#[derive(Debug, Clone)]
pub struct AIDetectorConfig {
    /// Enable anti-pattern detection
    pub enable_anti_pattern_detection: bool,
    /// Enable pattern detection
    pub enable_pattern_detection: bool,
    /// Enable ML-based scoring
    pub enable_ml_scoring: bool,
    /// Enable caching of analysis results
    pub enable_caching: bool,
    
    /// Configuration for pattern detection
    pub pattern_config: PatternDetectorConfig,
    /// Configuration for ML scorer
    pub ml_scorer_config: MLScorerConfig,
    /// Configuration for anti-pattern detection
    pub anti_pattern_config: AntiPatternDetectorConfig,
    
    /// Cache configuration (only used if enable_caching is true)
    #[cfg(feature = "caching")]
    pub cache_config: Option<rust_ai_ide_cache::CacheConfig>,
    
    /// Private field to force users to use the builder pattern when not using the caching feature
    #[cfg(not(feature = "caching"))]
    _non_exhaustive: (),
}

/// Configuration for pattern detection
#[derive(Debug, Clone)]
pub struct PatternDetectorConfig {
    /// Minimum confidence for pattern detection
    pub min_confidence: f32,
    /// Maximum patterns to detect per file
    pub max_patterns_per_file: usize,
}

/// Configuration for ML scorer
#[derive(Debug, Clone)]
pub struct MLScorerConfig {
    /// Enable ML enhancements
    pub enable_ml: bool,
    /// Model directory path
    pub model_path: Option<String>,
}

/// Request for analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisRequest {
    /// File URI to analyze
    pub file_uri: String,
    /// Detect anti-patterns
    pub detect_anti_patterns: bool,
    /// Detect architectural patterns
    pub detect_patterns: bool,
    /// Generate suggestions
    pub generate_suggestions: bool,
    /// Include performance analysis
    pub performance_analysis: bool,
    /// Parse tree (if available)
    pub parse_tree: Option<String>,
    /// Analysis context
    pub context: Option<AnalysisContext>,
}

/// Analysis context information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisContext {
    /// Project root path
    pub project_root: Option<String>,
    /// Target language
    pub language: Option<String>,
    /// Framework in use
    pub framework: Option<String>,
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct AnalysisPerformanceStats {
    pub cache_hit_rate: f64,
    pub total_analyses: usize,
    pub average_analysis_time_ms: f64,
    pub ml_predictions_processed: usize,
    pub cache_size_entries: usize,
    pub uptime_seconds: u64,
}

/// Configuration for anti-pattern detection
#[derive(Debug, Clone)]
pub struct AntiPatternDetectorConfig {
    /// Minimum confidence score for anti-pattern detection (0.0 to 1.0)
    pub min_confidence: f32,
    /// Maximum number of anti-patterns to report per file
    pub max_patterns_per_file: usize,
    /// Whether to enable detailed reporting of anti-patterns
    pub enable_detailed_reports: bool,
}

impl Default for AntiPatternDetectorConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_patterns_per_file: 10,
            enable_detailed_reports: true,
        }
    }
}

impl Default for MLScorerConfig {
    fn default() -> Self {
        Self {
            enable_ml: true,
            model_path: None,
        }
    }
}

impl PatternDetector {
    fn new() -> Self {
        let mut architectural_patterns = HashMap::new();

        // Add known patterns
        architectural_patterns.insert("Singleton".to_string(), ArchitecturalPattern::Singleton);
        architectural_patterns.insert("Repository".to_string(), ArchitecturalPattern::Repository);
        architectural_patterns.insert("Factory".to_string(), ArchitecturalPattern::Factory);
        architectural_patterns.insert("Observer".to_string(), ArchitecturalPattern::Observer);

        Self {
            architectural_patterns,
        }
    }

    fn analyze_patterns(
        &self,
        content: &str,
        file_path: &str,
        request: &AnalysisRequest,
    ) -> IdeResult<Vec<DetectedPattern>> {
        let mut detected_patterns = Vec::new();

        // Simple pattern detection based on keywords and structure
        for (pattern_name, pattern_type) in &self.architectural_patterns {
            if let Some(detected) = self.detect_pattern(content, file_path, pattern_name, *pattern_type) {
                detected_patterns.push(detected);
            }
        }

        Ok(detected_patterns)
    }

    fn detect_pattern(
        &self,
        content: &str,
        file_path: &str,
        pattern_name: &str,
        pattern_type: ArchitecturalPattern,
    ) -> Option<DetectedPattern> {
        let pattern_keywords = self.get_pattern_keywords(&pattern_type);
        let keyword_matches = pattern_keywords.iter()
            .filter(|keyword| content.contains(&format!("\\b{}\\b", keyword)))
            .count();

        if keyword_matches >= pattern_keywords.len() / 2 {
            let location = CodeLocation {
                file_path: file_path.to_string(),
                start_line: 1,
                start_column: 0,
                end_line: content.lines().count() as u32,
                end_column: 0,
                function_name: None,
                class_name: None,
            };

            Some(DetectedPattern {
                pattern_type,
                confidence: 0.0, // Will be set by ML scorer
                location,
                context: PatternContext {
                    code_snippet: content[..200.min(content.len())].to_string(),
                    surrounding_context: content.to_string(),
                    structural_info: StructuralInfo {
                        lines_of_code: content.lines().count(),
                        cyclomatic_complexity: 1,
                        nesting_depth: 0,
                        method_count: content.matches("fn ").count(),
                        field_count: 0,
                        dependency_count: content.matches("use ").count(),
                    },
                    semantic_info: SemanticInfo {
                        symbols: Vec::new(),
                        references: Vec::new(),
                        definitions: Vec::new(),
                        usages: HashMap::new(),
                    },
                },
                metadata: HashMap::new(),
            })
        } else {
            None
        }
    }

    fn get_pattern_keywords(&self, pattern_type: &ArchitecturalPattern) -> Vec<String> {
        match pattern_type {
            ArchitecturalPattern::Singleton => vec!["static".to_string(), "instance".to_string(), "lazy".to_string()],
            ArchitecturalPattern::Repository => vec!["trait".to_string(), "impl".to_string(), "query".to_string()],
            ArchitecturalPattern::Factory => vec!["create".to_string(), "factory".to_string(), "build".to_string()],
            ArchitecturalPattern::Observer => vec!["subscribe".to_string(), "notify".to_string(), "observer".to_string()],
            _ => vec![],
        }
    }
}

impl IntelligentSuggestionGenerator {
    fn new() -> Self {
        let mut suggestion_templates = HashMap::new();

        // Add templates for anti-patterns
        suggestion_templates.insert(
            "LongMethod".to_string(),
            SuggestionTemplate {
                condition: SuggestionCondition::AntiPatternType(AntiPattern::LongMethod),
                template: "Consider breaking down this long method '{method_name}' into smaller, focused methods to improve maintainability".to_string(),
                benefits: vec![
                    "Improved code readability".to_string(),
                    "Easier maintenance".to_string(),
                    "Better testability".to_string(),
                    "Reduced complexity".to_string(),
                ],
                refactoring_type: RefactoringType::ExtractMethod,
                priority: Priority::Medium,
            }
        );

        suggestion_templates.insert(
            "LargeClass".to_string(),
            SuggestionTemplate {
                condition: SuggestionCondition::AntiPatternType(AntiPattern::LargeClass),
                template: "Class '{class_name}' is quite large. Consider extracting some responsibilities into separate classes".to_string(),
                benefits: vec![
                    "Single Responsibility Principle".to_string(),
                    "Improved maintainability".to_string(),
                    "Better testability".to_string(),
                    "Reduced coupling".to_string(),
                ],
                refactoring_type: RefactoringType::ExtractClass,
                priority: Priority::High,
            }
        );

        Self {
            suggestion_templates,
        }
    }

    fn generate_from_anti_pattern(&self, anti_pattern: &DetectedAntiPattern) -> IdeResult<Option<IntelligenceSuggestion>> {
        let template_key = format!("{:?}", anti_pattern.anti_pattern_type);
        if let Some(template) = self.suggestion_templates.get(&template_key) {
            let title = template.template
                .replace("{method_name}", anti_pattern.location.function_name.as_deref().unwrap_or("method"))
                .replace("{class_name}", anti_pattern.location.class_name.as_deref().unwrap_or("class"));

            let suggestion = IntelligenceSuggestion::new(
                SuggestionCategory::Maintainability,
                format!("Improve {}", anti_pattern.anti_pattern_type.description()),
                title,
                anti_pattern.confidence,
                template.priority.clone(),
                anti_pattern.location.clone(),
                template.refactoring_type.clone(),
            )
            .with_benefits(template.benefits.clone())
            .with_guidance("Use 'Extract Method' refactoring to break down large methods".to_string());

            Ok(Some(suggestion))
        } else {
            Ok(None)
        }
    }

    fn generate_from_pattern(&self, _pattern: &DetectedPattern) -> IdeResult<Option<IntelligenceSuggestion>> {
        // For now, no specific suggestions for detected patterns
        Ok(None)
    }
}

impl AnalysisRequest {
    /// Create a comprehensive analysis request
    pub fn comprehensive(file_uri: &str) -> Self {
        Self {
            file_uri: file_uri.to_string(),
            detect_anti_patterns: true,
            detect_patterns: true,
            generate_suggestions: true,
            performance_analysis: true,
            parse_tree: None,
            context: None,
        }
    }

    /// Create a quick analysis request
    pub fn quick(file_uri: &str) -> Self {
        Self {
            file_uri: file_uri.to_string(),
            detect_anti_patterns: true,
            detect_patterns: false,
            generate_suggestions: true,
            performance_analysis: false,
            parse_tree: None,
            context: None,
        }
    }
}