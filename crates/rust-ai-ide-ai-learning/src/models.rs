//! Core data structures for the learning system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A learned pattern from successful error resolutions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Unique identifier for the pattern
    pub id: String,
    /// Human-readable description of the pattern
    pub description: String,
    /// The original error message pattern
    pub error_pattern: String,
    /// Error code if available (e.g., E0308)
    pub error_code: Option<String>,
    /// Context patterns that help identify when this pattern applies
    pub context_patterns: Vec<String>,
    /// The successful fix that was applied
    pub fix_template: FixTemplate,
    /// Confidence score based on success rate (0.0 to 1.0)
    pub confidence: f32,
    /// Number of times this pattern was successfully applied
    pub success_count: u32,
    /// Total number of times this pattern was attempted
    pub attempt_count: u32,
    /// When this pattern was first learned
    pub created_at: DateTime<Utc>,
    /// When this pattern was last updated
    pub updated_at: DateTime<Utc>,
    /// Hash of the code context for similarity matching
    pub context_hash: String,
    /// Tags for categorizing patterns
    pub tags: Vec<String>,
    /// User who contributed this pattern (anonymized if privacy mode is enabled)
    pub contributor_id: Option<String>,
}

/// Template for applying a learned fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixTemplate {
    /// Template for the fix description
    pub description_template: String,
    /// Code change templates
    pub change_templates: Vec<ChangeTemplate>,
    /// Variables that can be substituted in the template
    pub variables: HashMap<String, String>,
    /// Conditions that must be met for this template to apply
    pub conditions: Vec<TemplateCondition>,
    /// Warnings to show when applying this template
    pub warnings: Vec<String>,
}

/// Template for a code change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTemplate {
    /// Pattern to match in the original code
    pub match_pattern: String,
    /// Replacement pattern with variables
    pub replacement_pattern: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Scope of the change
    pub scope: ChangeScope,
}

/// Type of code change in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// Replace matched text
    Replace,
    /// Insert before matched text
    InsertBefore,
    /// Insert after matched text
    InsertAfter,
    /// Delete matched text
    Delete,
    /// Wrap matched text
    Wrap { prefix: String, suffix: String },
}

/// Scope of a code change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeScope {
    /// Change affects only the error location
    Local,
    /// Change affects the current function
    Function,
    /// Change affects the current file
    File,
    /// Change affects multiple files
    Project,
}

/// Condition that must be met for a template to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCondition {
    /// Type of condition
    pub condition_type: ConditionType,
    /// Pattern to match
    pub pattern: String,
    /// Whether the condition must be true or false
    pub must_match: bool,
}

/// Type of template condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    /// Code contains pattern
    CodeContains,
    /// Error message contains pattern
    ErrorContains,
    /// File extension matches
    FileExtension,
    /// Dependency is present
    DependencyPresent,
    /// Rust edition matches
    RustEdition,
}

/// Pattern similarity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSimilarity {
    /// Overall similarity score (0.0 to 1.0)
    pub overall_score: f32,
    /// Error message similarity
    pub error_similarity: f32,
    /// Context similarity
    pub context_similarity: f32,
    /// Code structure similarity
    pub structure_similarity: f32,
    /// Matching learned pattern
    pub pattern: LearnedPattern,
}

/// User preferences for the learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPreferences {
    /// Whether to enable learning from successful fixes
    pub enable_learning: bool,
    /// Privacy mode for data collection
    pub privacy_mode: super::types::PrivacyMode,
    /// Minimum confidence threshold for learned patterns
    pub confidence_threshold: f32,
    /// Maximum number of patterns to store per error type
    pub max_patterns_per_type: u32,
    /// Whether to share patterns with the community (anonymized)
    pub enable_community_sharing: bool,
    /// Whether to use patterns from the community
    pub use_community_patterns: bool,
    /// Auto-apply fixes with confidence above this threshold
    pub auto_apply_threshold: f32,
}

/// Statistics about learned patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    /// Total number of learned patterns
    pub total_patterns: u32,
    /// Number of patterns that have been successfully applied
    pub successful_patterns: u32,
    /// Number of patterns updated in the last 30 days
    pub recent_patterns: u32,
    /// Overall success rate
    pub success_rate: f32,
}

impl Default for LearningPreferences {
    fn default() -> Self {
        Self {
            enable_learning: true,
            privacy_mode: super::types::PrivacyMode::OptIn,
            confidence_threshold: 0.7,
            max_patterns_per_type: 100,
            enable_community_sharing: false,
            use_community_patterns: true,
            auto_apply_threshold: 0.9,
        }
    }
}

impl LearnedPattern {
    /// Calculate the success rate of this pattern
    pub fn success_rate(&self) -> f32 {
        if self.attempt_count == 0 {
            0.0
        } else {
            self.success_count as f32 / self.attempt_count as f32
        }
    }

    /// Calculate effective confidence including recency and usage
    pub fn effective_confidence(&self) -> f32 {
        let base_confidence = self.confidence;
        let success_rate = self.success_rate();
        let recency_factor = self.calculate_recency_factor();
        let usage_factor = self.calculate_usage_factor();

        // Weighted combination of factors
        (base_confidence * 0.4)
            + (success_rate * 0.3)
            + (recency_factor * 0.15)
            + (usage_factor * 0.15)
    }

    /// Calculate recency factor (more recent patterns get higher scores)
    fn calculate_recency_factor(&self) -> f32 {
        let now = Utc::now();
        let days_since_update = (now - self.updated_at).num_days();

        // Decay factor: patterns lose relevance over time
        if days_since_update <= 7 {
            1.0
        } else if days_since_update <= 30 {
            0.8
        } else if days_since_update <= 90 {
            0.6
        } else if days_since_update <= 365 {
            0.4
        } else {
            0.2
        }
    }

    /// Calculate usage factor (more frequently used patterns get higher scores)
    fn calculate_usage_factor(&self) -> f32 {
        match self.attempt_count {
            0 => 0.0,
            1..=5 => 0.3,
            6..=20 => 0.6,
            21..=50 => 0.8,
            _ => 1.0,
        }
    }
}
