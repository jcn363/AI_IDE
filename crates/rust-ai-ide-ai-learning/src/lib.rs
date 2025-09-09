//! # Rust AI IDE Learning System
//!
//! This crate provides a comprehensive learning system for storing and retrieving successful error resolution patterns.
//!
//! ## Features
//!
//! - **SQLite-based persistence** with efficient database operations
//! - **Pattern matching algorithms** for finding similar error contexts
//! - **User preference management** with privacy controls
//! - **Statistical analysis** and performance reporting
//! - **Privacy-aware data handling** with configurable privacy modes
//!
//! ## Architecture
//!
//! The learning system is organized into several submodules for maintainability:
//!
//! - `models` - Core data structures and types
//! - `database` - Database operations and persistence
//! - `similarity` - Pattern matching and similarity algorithms
//! - `preferences` - User preference management and validation
//! - `statistics` - Analytics and reporting functionality
//! - `system` - Main orchestration layer tying everything together
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rust_ai_ide_ai_learning::{LearningSystem, LearningPreferences};
//!
//! #[tokio::test]
//! async fn example_usage() {
//!     // Create learning system
//!     let learning = LearningSystem::new().await.unwrap();
//!
//!     // Update preferences
//!     let mut prefs = LearningPreferences::default();
//!     prefs.enable_learning = true;
//!     learning.update_preferences(prefs).await.unwrap();
//!
//!     // Get similar patterns
//!     let similar_patterns = learning.get_similar_patterns("unused variable").await.unwrap();
//!
//!     // Get statistics
//!     let stats = learning.get_pattern_statistics().await.unwrap();
//! }
//! ```

// Declare submodules
pub mod database;
pub mod models;
pub mod preferences;
pub mod similarity;
pub mod statistics;
pub mod system;
pub mod types;

// Re-export main types and interfaces for convenient access
pub use models::{
    ChangeScope, ChangeTemplate, ChangeType, ConditionType, FixTemplate, LearnedPattern,
    LearningPreferences, PatternSimilarity, PatternStatistics, TemplateCondition,
};

pub use database::LearningDatabase;
pub use preferences::{
    templates as preference_templates, utils as preference_utils, PreferencesManager,
};
pub use similarity::SimilarityCalculator;
pub use statistics::{analysis as stats_analysis, LearningStatistics};
pub use system::LearningSystem;
pub use types::PrivacyMode;

// Re-export similarity analysis utilities
pub use similarity::analysis::{
    analyze_pattern_themes, find_similar_fixes, group_patterns_by_error_type,
};
pub use similarity::SimilarityCache;

// Re-export preference templates for easy access
pub use preferences::templates::{balanced, development, maximum_learning, privacy_first};

// Re-export preference utilities
pub use preferences::utils::{
    apply_privacy_implications, calculate_privacy_score, get_privacy_recommendation,
};

// Re-export statistical analysis functions
pub use statistics::analysis::{analyze_trends, generate_insights};

/// Version information for the learning system
pub const LEARNING_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default cache configuration
pub const DEFAULT_SIMILARITY_CACHE_SIZE: usize = 1000;

/// Default pattern matching parameters
pub mod constants {
    /// Minimum similarity score required for pattern matching
    pub const MIN_SIMILARITY_SCORE: f32 = 0.3;
    /// Default context lines to consider for pattern matching
    pub const DEFAULT_CONTEXT_LINES: usize = 3;
    /// Maximum number of similar patterns to return
    pub const MAX_SIMILAR_PATTERNS: usize = 10;
}

/// Helper functions for common learning operations
pub mod helpers {
    use super::*;

    /// Create a learning system with custom database path
    pub async fn create_learning_system(
        db_path: std::path::PathBuf,
    ) -> Result<LearningSystem, Box<dyn std::error::Error>> {
        Ok(LearningSystem::new_with_path(Some(db_path)).await?)
    }

    /// Quick pattern search with default parameters
    pub async fn find_patterns_quick(
        system: &LearningSystem,
        error_context: &str,
    ) -> Result<Vec<LearnedPattern>, Box<dyn std::error::Error>> {
        let patterns = system.get_similar_patterns(error_context).await?;
        Ok(patterns
            .into_iter()
            .take(constants::MAX_SIMILAR_PATTERNS)
            .collect())
    }

    /// Export patterns to JSON with automatic anonymization
    pub async fn export_patterns_safe(
        system: &LearningSystem,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let json = system.export_patterns().await?;
        Ok(json)
    }

    /// Get learning system status
    pub fn get_system_status(system: &LearningSystem) -> SystemStatus {
        let prefs = system.get_preferences();
        SystemStatus {
            learning_enabled: prefs.enable_learning,
            privacy_mode: prefs.privacy_mode.clone(),
            confidence_threshold: prefs.confidence_threshold,
            version: LEARNING_VERSION.to_string(),
        }
    }
}

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    /// Whether learning is currently enabled
    pub learning_enabled: bool,
    /// Current privacy mode
    pub privacy_mode: PrivacyMode,
    /// Current confidence threshold
    pub confidence_threshold: f32,
    /// System version
    pub version: String,
}

impl std::fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Learning System v{} - ", self.version)?;
        write!(
            f,
            "Learning: {}, ",
            if self.learning_enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        )?;
        write!(f, "Privacy: {}, ", self.privacy_mode)?;
        write!(f, "Confidence: {:.2}", self.confidence_threshold)
    }
}

/// Initialize the learning system with default configuration
pub async fn initialize() -> Result<LearningSystem, Box<dyn std::error::Error>> {
    let mut system = LearningSystem::new().await?;

    // Set up default preferences
    let defaults = preference_templates::balanced();
    system.update_preferences(defaults).await?;

    Ok(system)
}

/// Cleanup utility for testing
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use tempfile::TempDir;

    /// Create a temporary learning system for testing
    pub async fn create_temp_learning_system() -> (LearningSystem, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_learning.db");
        let system = LearningSystem::new_with_path(Some(db_path)).await.unwrap();
        (system, temp_dir)
    }

    /// Clear all test data
    pub async fn clear_test_data(system: &mut LearningSystem) {
        system.clear_all_patterns().await.unwrap();
    }
}
