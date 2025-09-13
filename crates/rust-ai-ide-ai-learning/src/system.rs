//! Main learning system implementation that orchestrates all subsystems

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use chrono::Utc;
use uuid::Uuid;

// Import our submodules
use super::database::LearningDatabase;
use super::models::{LearnedPattern, LearningPreferences, PatternSimilarity};
use super::preferences::PreferencesManager;
use super::similarity::SimilarityCalculator;
use super::statistics::analysis::generate_insights;
use super::statistics::LearningStatistics;
use super::types::db_types::{ChangeType as DBChangeType, ErrorPattern, FixSuggestion};
use super::types::{AIResult, AIServiceError, PrivacyMode};
use super::SimilarityCache;

/// Main learning system that coordinates all learning functionality
#[derive(Debug)]
pub struct LearningSystem {
    /// Database manager for persistence
    database:              LearningDatabase,
    /// Preferences manager for user settings
    preferences_manager:   PreferencesManager,
    /// Similarity calculator for pattern matching
    // Prepared for direct similarity calculations and advanced pattern matching
    similarity_calculator: SimilarityCalculator,
    /// Cache for similarity computations
    similarity_cache:      Arc<RwLock<SimilarityCache>>,
    /// In-memory cache for frequently used patterns
    pattern_cache:         Arc<RwLock<HashMap<String, Vec<LearnedPattern>>>>,
    /// User preferences (cached for performance)
    preferences:           LearningPreferences,
    /// User identifier (anonymized if privacy mode is enabled)
    user_id:               String,
}

impl LearningSystem {
    /// Create a new learning system with specified database path
    pub async fn new_with_path(db_path: Option<PathBuf>) -> AIResult<Self> {
        let database = LearningDatabase::new(db_path).await?;
        let preferences_manager = PreferencesManager::new(database.clone()).await;

        let preferences = preferences_manager.load().await;

        Ok(Self {
            database,
            preferences_manager,
            similarity_calculator: SimilarityCalculator,
            similarity_cache: Arc::new(RwLock::new(SimilarityCache::new())),
            pattern_cache: Arc::new(RwLock::new(HashMap::new())),
            preferences,
            user_id: Self::generate_user_id(),
        })
    }

    /// Create a new learning system using default database path
    pub async fn new() -> AIResult<Self> {
        Self::new_with_path(None).await
    }

    /// Update learning preferences
    pub async fn update_preferences(&mut self, preferences: LearningPreferences) -> AIResult<()> {
        // Validate preferences before updating
        super::preferences::PreferencesManager::validate_preferences(&preferences)?;

        self.preferences = preferences.clone();
        self.preferences_manager.save(&preferences).await?;

        // Clear caches when preferences change
        self.pattern_cache.write().unwrap().clear();
        if let Ok(cache) = self.similarity_cache.write() {
            if let Err(e) = cache.clear().await {
                eprintln!(
                    "Failed to clear similarity cache on preferences update: {:?}",
                    e
                );
            }
        }

        Ok(())
    }

    /// Get current preferences
    pub fn get_preferences(&self) -> &LearningPreferences {
        &self.preferences
    }

    /// Record a successful fix application
    pub async fn record_successful_fix(&self, error_pattern: ErrorPattern, fix: FixSuggestion) -> AIResult<()> {
        if !self.preferences.enable_learning {
            return Ok(());
        }

        // Check privacy settings
        let contributor_id = match self.preferences.privacy_mode {
            PrivacyMode::OptOut => return Ok(()),
            PrivacyMode::Anonymous => None,
            PrivacyMode::OptIn => Some(self.user_id.clone()),
        };

        // Create or update learned pattern
        let pattern_id = self.generate_pattern_id(&error_pattern, &fix);

        if let Ok(Some(_existing_pattern)) = self.database.get_pattern_by_id(&pattern_id).await {
            // Update existing pattern
            self.database
                .update_pattern_success(&pattern_id, true, &self.user_id)
                .await?;
        } else {
            // Create new learned pattern
            let learned_pattern = self
                .create_learned_pattern(error_pattern, fix, contributor_id)
                .await?;

            self.database.store_pattern(&learned_pattern).await?;
        }

        // Clear relevant caches
        self.invalidate_cache(&pattern_id);

        Ok(())
    }

    /// Record a failed fix application
    pub async fn record_failed_fix(&self, error_pattern: ErrorPattern, fix: FixSuggestion) -> AIResult<()> {
        if !self.preferences.enable_learning {
            return Ok(());
        }

        let pattern_id = self.generate_pattern_id(&error_pattern, &fix);

        if let Some(_existing_pattern) = self.database.get_pattern_by_id(&pattern_id).await? {
            // Update existing pattern with failure
            self.database
                .update_pattern_success(&pattern_id, false, &self.user_id)
                .await?;
        }

        // Clear relevant caches
        self.invalidate_cache(&pattern_id);

        Ok(())
    }

    /// Get similar patterns for a given error context
    pub async fn get_similar_patterns(&self, error_context: &str) -> AIResult<Vec<LearnedPattern>> {
        // Check cache first
        if let Ok(cache) = self.pattern_cache.read() {
            if let Some(cached_patterns) = cache.get(error_context) {
                return Ok(cached_patterns.clone());
            }
        }

        let similarities = self.find_similar_patterns(error_context).await?;

        // Filter by confidence threshold
        let patterns: Vec<LearnedPattern> = similarities
            .into_iter()
            .filter(|s| s.overall_score >= self.preferences.confidence_threshold)
            .map(|s| s.pattern)
            .collect();

        // Cache the results
        if let Ok(mut cache) = self.pattern_cache.write() {
            cache.insert(error_context.to_string(), patterns.clone());
        }

        Ok(patterns)
    }

    /// Find patterns similar to the given error context
    pub async fn find_similar_patterns(&self, error_context: &str) -> AIResult<Vec<PatternSimilarity>> {
        // Check cache first
        let cache_key = SimilarityCalculator::create_cache_key(error_context);

        // Use unified cache async API
        if let Ok(cache) = self.similarity_cache.read() {
            if let Some(cached_similarities) = cache.get(&cache_key).await {
                return Ok(cached_similarities);
            }
        }

        // Get all patterns from database
        let all_patterns = self.database.get_all_patterns().await?;
        let similarities = SimilarityCalculator::find_best_matches(
            error_context,
            &all_patterns,
            10, // Max results
        );

        // Cache the results using unified cache async API
        if let Ok(mut cache) = self.similarity_cache.write() {
            if let Err(e) = cache.insert(cache_key, similarities.clone()).await {
                // Log error but don't fail the operation
                eprintln!("Failed to cache similarity results: {:?}", e);
            }
        }

        Ok(similarities)
    }

    /// Get learned patterns by error type
    pub async fn get_patterns_by_error_type(&self, error_code: &str) -> AIResult<Vec<LearnedPattern>> {
        let max_count = self.preferences.max_patterns_per_type;
        self.database
            .get_patterns_by_error_type(error_code, max_count)
            .await
    }

    /// Get pattern statistics
    pub async fn get_pattern_statistics(&self) -> AIResult<LearningStatistics> {
        LearningStatistics::calculate(&self.database).await
    }

    /// Clear all learned patterns (with user confirmation)
    pub async fn clear_all_patterns(&mut self) -> AIResult<()> {
        self.database.clear_all_patterns().await?;

        // Clear pattern cache (synchronous)
        self.pattern_cache.write().unwrap().clear();

        // Clear similarity cache using unified system
        if let Ok(cache) = self.similarity_cache.write() {
            cache.clear().await?;
        }

        Ok(())
    }

    /// Export patterns for backup or sharing
    pub async fn export_patterns(&self) -> AIResult<String> {
        self.database
            .export_patterns(self.preferences.privacy_mode)
            .await
    }

    /// Import patterns from backup or community
    pub async fn import_patterns(&mut self, patterns_json: &str) -> AIResult<usize> {
        // Parse and import patterns
        let count = self
            .database
            .import_patterns_from_json(patterns_json)
            .await?;

        // Clear caches after import
        self.pattern_cache.write().unwrap().clear();
        if let Ok(cache) = self.similarity_cache.write() {
            cache.clear().await?;
        }

        Ok(count)
    }

    /// Generate a unique pattern ID from error pattern and fix
    fn generate_pattern_id(&self, error_pattern: &ErrorPattern, fix: &FixSuggestion) -> String {
        let combined = format!("{}_{}", error_pattern.message_pattern, fix.title);
        format!("pattern_{:x}", self.hash_string(&combined))
    }

    /// Create a learned pattern from an error pattern and fix
    async fn create_learned_pattern(
        &self,
        error_pattern: ErrorPattern,
        fix: FixSuggestion,
        contributor_id: Option<String>,
    ) -> AIResult<LearnedPattern> {
        use super::models::{ChangeType as ModelChangeType, FixTemplate};

        let fix_template = FixTemplate {
            description_template: fix.description.clone(),
            change_templates:     fix
                .changes
                .iter()
                .map(|change| {
                    use super::models::{ChangeScope, ChangeTemplate as CT};

                    CT {
                        match_pattern:       change.original_text.clone(),
                        replacement_pattern: change.new_text.clone(),
                        change_type:         match change.change_type {
                            DBChangeType::Insert => ModelChangeType::InsertAfter,
                            DBChangeType::Delete => ModelChangeType::Delete,
                            DBChangeType::Replace => ModelChangeType::Replace,
                            DBChangeType::Move => ModelChangeType::Replace,
                        },
                        scope:               ChangeScope::Local, // Default scope
                    }
                })
                .collect(),
            variables:            HashMap::new(),
            conditions:           vec![],
            warnings:             fix.warnings.clone(),
        };

        let context_hash = format!(
            "{:x}",
            self.hash_string(&error_pattern.context_patterns.join("\n"))
        );
        let tags = self.extract_tags(&error_pattern, &fix);

        Ok(LearnedPattern {
            id: self.generate_pattern_id(&error_pattern, &fix),
            description: format!("Learned pattern: {}", fix.title),
            error_pattern: error_pattern.message_pattern,
            error_code: error_pattern.error_code,
            context_patterns: error_pattern.context_patterns,
            fix_template,
            confidence: fix.confidence,
            success_count: 1,
            attempt_count: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            context_hash,
            tags,
            contributor_id,
        })
    }

    /// Extract tags from error pattern and fix
    fn extract_tags(&self, error_pattern: &ErrorPattern, fix: &FixSuggestion) -> Vec<String> {
        let mut tags = Vec::new();

        // Add error code as tag if available
        if let Some(error_code) = &error_pattern.error_code {
            tags.push(error_code.clone());
        }

        // Add tags based on error message content
        if error_pattern.message_pattern.contains("borrow") {
            tags.push("borrowing".to_string());
        }
        if error_pattern.message_pattern.contains("lifetime") {
            tags.push("lifetimes".to_string());
        }
        if error_pattern.message_pattern.contains("trait") {
            tags.push("traits".to_string());
        }
        if error_pattern.message_pattern.contains("type") {
            tags.push("types".to_string());
        }
        if error_pattern.message_pattern.contains("unused") {
            tags.push("unused".to_string());
        }

        // Add tags based on fix type
        if fix.title.contains("import") || fix.title.contains("use") {
            tags.push("imports".to_string());
        }
        if fix.title.contains("mut") || fix.title.contains("mutable") {
            tags.push("mutability".to_string());
        }
        if fix.title.contains("clone") {
            tags.push("cloning".to_string());
        }

        tags
    }

    /// Generate a hash for a string (for pattern IDs and context hashing)
    fn hash_string(&self, input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }

    /// Generate a unique user ID
    fn generate_user_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Invalidate cache entries related to a pattern
    fn invalidate_cache(&self, pattern_id: &str) {
        // Clear pattern cache (synchronous)
        self.pattern_cache.write().unwrap().clear();

        // Invalidate similarity cache using unified system
        // Note: Using sync clear to avoid lifetime issues with async spawn
        if let Ok(cache) = self.similarity_cache.write() {
            if let Err(e) = futures::executor::block_on(cache.clear()) {
                eprintln!("Failed to clear similarity cache: {:?}", e);
            }
        }
    }

    /// Get learning insights and recommendations
    pub async fn get_insights(&self) -> AIResult<Vec<String>> {
        let stats = self.get_pattern_statistics().await?;
        Ok(generate_insights(&stats))
    }

    /// Shutdown the learning system (cleanup resources)
    pub async fn shutdown(self) -> AIResult<()> {
        // Clear caches
        self.pattern_cache.write().unwrap().clear();
        if let Ok(cache) = self.similarity_cache.write() {
            cache.clear().await?;
        }

        // Ensure all pending writes are completed
        self.database.shutdown().await
    }
}

// Import helper methods for the database to support the new features
impl LearningDatabase {
    /// Import patterns from JSON string
    pub async fn import_patterns_from_json(&self, patterns_json: &str) -> AIResult<usize> {
        use super::models::LearnedPattern;

        let patterns: Vec<LearnedPattern> =
            serde_json::from_str(patterns_json).map_err(|e| AIServiceError::SerializationError(e.to_string()))?;

        self.import_patterns(&patterns).await
    }

    /// Shutdown the database connection
    pub async fn shutdown(&self) -> AIResult<()> {
        // In a real implementation, you might need to close connections
        // For now, this is a placeholder
        Ok(())
    }
}
