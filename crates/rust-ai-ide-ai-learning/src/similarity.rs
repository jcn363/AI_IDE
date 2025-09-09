//! Pattern similarity and matching algorithms for the learning system

use super::models::{LearnedPattern, PatternSimilarity};

/// Pattern similarity calculator
#[derive(Debug)]
pub struct SimilarityCalculator;

impl SimilarityCalculator {
    /// Calculate similarity between error context and learned pattern
    pub fn calculate_pattern_similarity(
        error_context: &str,
        pattern: &LearnedPattern,
    ) -> PatternSimilarity {
        let error_similarity =
            Self::calculate_text_similarity(error_context, &pattern.error_pattern);

        let context_similarity = if pattern.context_patterns.is_empty() {
            0.5 // Neutral score for patterns without context
        } else {
            let context_text = pattern.context_patterns.join(" ");
            Self::calculate_text_similarity(error_context, &context_text)
        };

        let structure_similarity = Self::calculate_structure_similarity(error_context, pattern);

        // Weighted combination of similarity factors
        let overall_score =
            (error_similarity * 0.5) + (context_similarity * 0.3) + (structure_similarity * 0.2);

        PatternSimilarity {
            overall_score,
            error_similarity,
            context_similarity,
            structure_similarity,
            pattern: pattern.clone(),
        }
    }

    /// Calculate text similarity using simple word-based metrics
    pub fn calculate_text_similarity(text1: &str, text2: &str) -> f32 {
        if text1.is_empty() || text2.is_empty() {
            return 0.0;
        }

        // Simple word-based similarity
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Calculate structural similarity based on code patterns
    pub fn calculate_structure_similarity(error_context: &str, pattern: &LearnedPattern) -> f32 {
        let mut similarity_score = 0.0;
        let mut total_checks = 0.0;

        // Check for common Rust keywords and patterns
        let rust_keywords = [
            "fn", "let", "mut", "impl", "struct", "enum", "trait", "match", "if", "for", "while",
        ];

        for keyword in &rust_keywords {
            total_checks += 1.0;
            let context_has = error_context.contains(keyword);
            let pattern_has = pattern.context_patterns.iter().any(|p| p.contains(keyword));

            if context_has == pattern_has {
                similarity_score += 1.0;
            }
        }

        // Check for code structure patterns
        total_checks += 1.0;
        let has_braces = error_context.contains('{') && error_context.contains('}');
        let pattern_has_braces = pattern
            .context_patterns
            .iter()
            .any(|p| p.contains('{') && p.contains('}'));
        if has_braces == pattern_has_braces {
            similarity_score += 1.0;
        }

        // Check for semicolons (statement-based patterns)
        total_checks += 1.0;
        let has_semicolons = error_context.contains(';');
        let pattern_has_semicolons = pattern.context_patterns.iter().any(|p| p.contains(';'));
        if has_semicolons == pattern_has_semicolons {
            similarity_score += 1.0;
        }

        if total_checks > 0.0 {
            similarity_score / total_checks
        } else {
            0.5
        }
    }

    /// Find best matching patterns from a list
    pub fn find_best_matches(
        error_context: &str,
        patterns: &[LearnedPattern],
        max_results: usize,
    ) -> Vec<PatternSimilarity> {
        let mut similarities: Vec<PatternSimilarity> = patterns
            .iter()
            .map(|pattern| Self::calculate_pattern_similarity(error_context, pattern))
            .filter(|s| s.overall_score > 0.3) // Only include reasonably similar patterns
            .collect();

        // Sort by similarity score
        similarities.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top results
        similarities.into_iter().take(max_results).collect()
    }

    /// Filter patterns by minimum confidence threshold
    pub fn filter_by_confidence(
        mut similarities: Vec<PatternSimilarity>,
        threshold: f32,
    ) -> Vec<PatternSimilarity> {
        similarities.retain(|s| s.overall_score >= threshold);
        similarities
    }

    /// Create similarity cache key for caching computed similarities
    pub fn create_cache_key(error_context: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        error_context.hash(&mut hasher);
        format!("similarity_{:x}", hasher.finish())
    }

    /// Advanced similarity calculation with semantic analysis
    pub fn calculate_semantic_similarity(text1: &str, text2: &str) -> f32 {
        let base_similarity = Self::calculate_text_similarity(text1, text2);

        // Add semantic bonuses for common programming patterns
        let mut semantic_bonus = 0.0;

        // Bonuses for matching error types
        let error_patterns = [
            ("borrow", "ownership", 0.2),
            ("lifetime", "ownership", 0.2),
            ("move", "ownership", 0.15),
            ("mutable", "ownership", 0.15),
            ("cannot", "error", 0.1),
            ("expected", "error", 0.1),
            ("found", "error", 0.1),
            ("type", "types", 0.1),
            ("trait", "traits", 0.15),
        ];

        for (pattern1, pattern2, bonus) in &error_patterns {
            let has_pattern1 = text1.contains(pattern1) && text2.contains(pattern1);
            let has_pattern2 = text1.contains(pattern2) && text2.contains(pattern2);
            if has_pattern1 || has_pattern2 {
                semantic_bonus += bonus;
            }
        }

        // Cap the total similarity at 1.0
        (base_similarity + semantic_bonus).min(1.0)
    }
}

/// Helper functions for pattern analysis
pub mod analysis {
    use super::*;
    use std::collections::HashMap;

    /// Analyze patterns for common themes
    pub fn analyze_pattern_themes(patterns: &[LearnedPattern]) -> HashMap<String, f32> {
        let mut themes = HashMap::new();
        let total_patterns = patterns.len() as f32;

        for pattern in patterns {
            for tag in &pattern.tags {
                *themes.entry(tag.clone()).or_insert(0.0) += 1.0;
            }
        }

        // Convert counts to percentages
        for count in themes.values_mut() {
            *count = *count / total_patterns;
        }

        themes
    }

    /// Group patterns by error type
    pub fn group_patterns_by_error_type(
        patterns: &[LearnedPattern],
    ) -> HashMap<String, Vec<&LearnedPattern>> {
        let mut grouped = HashMap::new();

        for pattern in patterns {
            if let Some(error_code) = &pattern.error_code {
                grouped
                    .entry(error_code.clone())
                    .or_insert_with(Vec::new)
                    .push(pattern);
            } else {
                grouped
                    .entry("unknown".to_string())
                    .or_insert_with(Vec::new)
                    .push(pattern);
            }
        }

        grouped
    }

    /// Calculate average confidence for pattern groups
    pub fn calculate_group_confidence(patterns: &[&LearnedPattern]) -> f32 {
        if patterns.is_empty() {
            return 0.0;
        }

        let total_confidence: f32 = patterns.iter().map(|p| p.effective_confidence()).sum();
        total_confidence / patterns.len() as f32
    }

    /// Find patterns with similar fix templates
    pub fn find_similar_fixes<'a>(
        patterns: &'a [LearnedPattern],
        target_pattern: &str,
    ) -> Vec<&'a LearnedPattern> {
        patterns
            .iter()
            .filter(|p| {
                SimilarityCalculator::calculate_text_similarity(
                    target_pattern,
                    &p.fix_template.description_template,
                ) > 0.5
            })
            .collect()
    }
}

// AI Learning Cache using Unified Cache System
//
// This module provides AI-optimized caching for similarity computations
// using the unified cache infrastructure from rust-ai-ide-cache.

use rust_ai_ide_cache::{key_utils, AiCacheExt, Cache, CacheConfig, CacheStats, InMemoryCache};
use rust_ai_ide_errors::IDEResult;
// serde is imported via the unified cache crate
use std::time::Duration;

/// Unified AI similarity cache implementation
pub struct SimilarityCache {
    unified_cache: InMemoryCache<String, serde_json::Value>,
    config: CacheConfig,
}

impl std::fmt::Debug for SimilarityCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimilarityCache")
            .field("config", &self.config)
            .finish()
    }
}

impl Clone for SimilarityCache {
    fn clone(&self) -> Self {
        // Create a new cache instance instead of cloning the existing one
        // This is necessary because InMemoryCache doesn't implement Clone
        let config = self.config.clone();
        let unified_cache = InMemoryCache::new(&config);
        Self {
            unified_cache,
            config,
        }
    }
}

impl SimilarityCache {
    /// Create a new AI similarity cache with optimized settings
    pub fn new() -> Self {
        let config = CacheConfig {
            max_entries: Some(5000), // Higher limit for AI computations
            default_ttl: Some(Duration::from_secs(1800)), // 30 minutes
            ..Default::default()
        };

        let unified_cache = InMemoryCache::new(&config);

        Self {
            unified_cache,
            config,
        }
    }

    /// Create cache with custom configuration
    pub fn new_with_config(config: CacheConfig) -> Self {
        let unified_cache = InMemoryCache::new(&config);
        Self {
            unified_cache,
            config,
        }
    }

    /// Get cached similarities by key
    pub async fn get(&self, key: &str) -> Option<Vec<PatternSimilarity>> {
        if let Ok(Some(value)) = self.unified_cache.get(&key.to_string()).await {
            // Deserialize from JSON
            serde_json::from_value(value).ok()
        } else {
            None
        }
    }

    /// Store similarities in cache with AI optimizations
    pub async fn insert(
        &mut self,
        key: String,
        similarities: Vec<PatternSimilarity>,
    ) -> IDEResult<()> {
        // Calculate token usage estimate for AI metrics
        let tokens_used = similarities.len() as u32;

        // Serialize to JSON for unified cache storage
        let json_value = serde_json::to_value(&similarities).map_err(|e| {
            rust_ai_ide_errors::RustAIError::Serialization(format!("Serialization error: {}", e))
        })?;

        // Use unified AI cache extension
        self.unified_cache
            .ai_store_inference(
                key,
                json_value,
                Some(tokens_used),
                Some(Duration::from_secs(1800)),
            )
            .await?;

        Ok(())
    }

    /// Clear all cached similarities
    pub async fn clear(&self) -> IDEResult<()> {
        self.unified_cache.clear().await
    }

    /// Invalidate cache entries related to a specific pattern
    pub async fn invalidate_pattern(&mut self, pattern_id: &str) -> IDEResult<()> {
        // Use basic cache clearing for now - in production, would need specialized invalidation
        self.unified_cache.clear().await?;
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.unified_cache.stats().await
    }

    /// Create similarity cache key with proper formatting
    pub fn create_cache_key(query: &str, method: &str) -> String {
        key_utils::structured_key("ai_similarity", &vec![query, method])
    }
}

/// Legacy compatibility functions for existing code
impl std::default::Default for SimilarityCache {
    fn default() -> Self {
        Self::new()
    }
}

// Provide synchronous wrapper methods for backward compatibility
impl SimilarityCache {
    /// Synchronous get method for backward compatibility
    pub fn get_sync(&self, key: &str) -> Option<&Vec<PatternSimilarity>> {
        // This is a placeholder - real implementation would use async runtime
        // For now, return None to avoid blocking
        None
    }

    /// Synchronous insert method for backward compatibility
    pub fn insert_sync(&mut self, key: String, similarities: Vec<PatternSimilarity>) {
        // This is a placeholder - real implementation would spawn async task
        // For now, we can't actually store due to sync requirement
        // In production, this would be refactored to be fully async
    }

    /// Synchronous clear method for backward compatibility
    pub fn clear_sync(&mut self) {
        // This is a placeholder - real implementation would spawn async task
        // For now, we can't actually clear due to sync requirement
    }
}
