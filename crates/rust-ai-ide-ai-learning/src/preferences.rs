//! User preferences management for the learning system

use super::database::LearningDatabase;
use super::models::LearningPreferences;
use super::types::{AIResult, AIServiceError};

/// Preferences manager for learning system settings
#[derive(Debug)]
pub struct PreferencesManager {
    database: LearningDatabase,
}

impl PreferencesManager {
    /// Create a new preferences manager
    pub async fn new(database: LearningDatabase) -> Self {
        Self { database }
    }

    /// Load preferences from database with fallback to defaults
    pub async fn load(&self) -> LearningPreferences {
        match self.database.load_preferences().await {
            Ok(prefs) => {
                log::debug!("Loaded user preferences from database");
                prefs
            }
            Err(e) => {
                log::warn!(
                    "Failed to load preferences from database: {}. Using defaults.",
                    e
                );
                LearningPreferences::default()
            }
        }
    }

    /// Save preferences to database
    pub async fn save(&self, preferences: &LearningPreferences) -> AIResult<()> {
        log::debug!("Saving user preferences to database");
        self.database.save_preferences(preferences).await?;
        log::info!("Preferences saved successfully");
        Ok(())
    }

    /// Reset preferences to defaults
    pub async fn reset_to_defaults(&self) -> AIResult<LearningPreferences> {
        let defaults = LearningPreferences::default();
        self.save(&defaults).await?;
        log::info!("Preferences reset to defaults");
        Ok(defaults)
    }

    /// Update specific preference setting
    pub async fn update_setting<T>(&self, key: &str, value: T) -> AIResult<()>
    where
        T: ToString,
    {
        let mut current_prefs = self.load().await;

        match key {
            "enable_learning" => {
                current_prefs.enable_learning = value.to_string().parse().map_err(|_| {
                    AIServiceError::ValidationError("Invalid boolean value".to_string())
                })?;
            }
            "privacy_mode" => {
                // Would need to parse privacy mode from string
                log::warn!("Privacy mode updates should use update_privacy_mode method");
                return Err(AIServiceError::ValidationError(
                    "Use update_privacy_mode method for privacy mode".to_string(),
                ));
            }
            "confidence_threshold" => {
                current_prefs.confidence_threshold = value.to_string().parse().map_err(|_| {
                    AIServiceError::ValidationError("Invalid confidence threshold".to_string())
                })?;
            }
            "max_patterns_per_type" => {
                current_prefs.max_patterns_per_type = value.to_string().parse().map_err(|_| {
                    AIServiceError::ValidationError("Invalid max patterns value".to_string())
                })?;
            }
            "enable_community_sharing" => {
                current_prefs.enable_community_sharing =
                    value.to_string().parse().map_err(|_| {
                        AIServiceError::ValidationError("Invalid boolean value".to_string())
                    })?;
            }
            "use_community_patterns" => {
                current_prefs.use_community_patterns = value.to_string().parse().map_err(|_| {
                    AIServiceError::ValidationError("Invalid boolean value".to_string())
                })?;
            }
            "auto_apply_threshold" => {
                current_prefs.auto_apply_threshold = value.to_string().parse().map_err(|_| {
                    AIServiceError::ValidationError("Invalid auto apply threshold".to_string())
                })?;
            }
            _ => {
                return Err(AIServiceError::ValidationError(format!(
                    "Unknown preference key: {}",
                    key
                )));
            }
        }

        self.save(&current_prefs).await?;
        log::info!("Updated preference '{}' to '{}'", key, value.to_string());
        Ok(())
    }

    /// Validate preference values
    pub fn validate_preferences(prefs: &LearningPreferences) -> AIResult<()> {
        if !(0.0..=1.0).contains(&prefs.confidence_threshold) {
            return Err(AIServiceError::ValidationError(
                "Confidence threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if prefs.max_patterns_per_type == 0 {
            return Err(AIServiceError::ValidationError(
                "Max patterns per type must be greater than 0".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&prefs.auto_apply_threshold) {
            return Err(AIServiceError::ValidationError(
                "Auto apply threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Confidence threshold should be >= auto apply threshold
        if prefs.confidence_threshold < prefs.auto_apply_threshold {
            return Err(AIServiceError::ValidationError(
                "Confidence threshold must be greater than or equal to auto apply threshold"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Get privacy-sensitive preference settings
    pub fn is_privacy_respecting(&self, prefs: &LearningPreferences) -> bool {
        use super::types::PrivacyMode;

        match prefs.privacy_mode {
            PrivacyMode::OptOut | PrivacyMode::Anonymous => !prefs.enable_community_sharing,
            PrivacyMode::OptIn => {
                // In opt-in mode, community sharing should be explicitly enabled
                true // This would be checked at runtime
            }
        }
    }
}

/// Preference templates for different usage scenarios
pub mod templates {
    use super::super::models::LearningPreferences;
    use super::super::PrivacyMode;

    /// Template for maximum learning (risk-tolerant users)
    pub fn maximum_learning() -> LearningPreferences {
        LearningPreferences {
            enable_learning: true,
            privacy_mode: PrivacyMode::OptIn,
            confidence_threshold: 0.5,
            max_patterns_per_type: 200,
            enable_community_sharing: true,
            use_community_patterns: true,
            auto_apply_threshold: 0.8,
        }
    }

    /// Template for balanced learning and privacy
    pub fn balanced() -> LearningPreferences {
        LearningPreferences {
            enable_learning: true,
            privacy_mode: PrivacyMode::Anonymous,
            confidence_threshold: 0.7,
            max_patterns_per_type: 100,
            enable_community_sharing: false,
            use_community_patterns: true,
            auto_apply_threshold: 0.9,
        }
    }

    /// Template for privacy-first learning (conservative users)
    pub fn privacy_first() -> LearningPreferences {
        LearningPreferences {
            enable_learning: true,
            privacy_mode: PrivacyMode::OptOut,
            confidence_threshold: 0.8,
            max_patterns_per_type: 50,
            enable_community_sharing: false,
            use_community_patterns: false,
            auto_apply_threshold: 0.95,
        }
    }

    /// Template for development/testing (minimal restrictions)
    pub fn development() -> LearningPreferences {
        LearningPreferences {
            enable_learning: true,
            privacy_mode: PrivacyMode::OptIn,
            confidence_threshold: 0.3,
            max_patterns_per_type: 500,
            enable_community_sharing: false,
            use_community_patterns: true,
            auto_apply_threshold: 0.6,
        }
    }
}

/// Utilities for managing preference configurations
pub mod utils {
    use super::super::models::LearningPreferences;
    use super::super::PrivacyMode;

    /// Apply privacy mode implications to other settings
    pub fn apply_privacy_implications(prefs: &mut LearningPreferences) {
        match prefs.privacy_mode {
            PrivacyMode::OptOut => {
                // In opt-out mode, disable community features by default
                prefs.enable_community_sharing = false;
                prefs.use_community_patterns = false;
            }
            PrivacyMode::Anonymous => {
                // In anonymous mode, disable sharing but allow using community patterns
                prefs.enable_community_sharing = false;
                // use_community_patterns remains as set by user
            }
            PrivacyMode::OptIn => {
                // In opt-in mode, all community features are available
                // but disabled by default for safety
                prefs.enable_community_sharing = false;
                prefs.use_community_patterns = true;
            }
        }
    }

    /// Calculate effective privacy score (0.0 = most private, 1.0 = least private)
    pub fn calculate_privacy_score(prefs: &LearningPreferences) -> f32 {
        let mut score: f32 = 0.0;

        match prefs.privacy_mode {
            PrivacyMode::OptOut => score += 0.0,
            PrivacyMode::Anonymous => score += 0.5,
            PrivacyMode::OptIn => score += 1.0,
        }

        if prefs.enable_community_sharing {
            score += 0.2;
        }

        score.max(0.0).min(1.0)
    }

    /// Get recommendation message based on privacy settings
    pub fn get_privacy_recommendation(prefs: &LearningPreferences) -> &'static str {
        match prefs.privacy_mode {
            PrivacyMode::OptOut => "All learning data is kept private and local.",
            PrivacyMode::Anonymous => {
                "Learning data is anonymized before any sharing. Community patterns are available."
            }
            PrivacyMode::OptIn => {
                if prefs.enable_community_sharing {
                    "Patterns may be anonymously shared with the community. Community patterns are used to improve suggestions."
                } else {
                    "Community patterns are used but your patterns are not shared. This provides good balance between privacy and functionality."
                }
            }
        }
    }

    /// Validate preference transitions (ensure smooth upgrades)
    pub fn validate_transition(
        from: &LearningPreferences,
        to: &LearningPreferences,
    ) -> Result<(), String> {
        // Ensure confidence threshold doesn't drop too drastically
        if from.confidence_threshold - to.confidence_threshold > 0.3 {
            return Err(
                "Confidence threshold changes are limited to avoid sudden behavior changes"
                    .to_string(),
            );
        }

        // Prevent certain privacy mode reversals
        match (&from.privacy_mode, &to.privacy_mode) {
            (PrivacyMode::OptOut, PrivacyMode::OptIn) => {
                return Err(
                    "Cannot directly switch from OptOut to OptIn mode for safety".to_string(),
                );
            }
            _ => {} // All other transitions are allowed
        }

        Ok(())
    }
}
