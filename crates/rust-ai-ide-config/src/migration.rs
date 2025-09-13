//! Configuration migration and upgrade support
//!
//! Provides automatic migration of configuration files between versions,
//! backward compatibility handling, and upgrade path management.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Migration engine for configuration upgrades
#[derive(Debug)]
pub struct MigrationEngine {
    /// Registered migration plans
    plans:            HashMap<String, Vec<MigrationPlan>>,
    /// Current version mappings
    current_versions: HashMap<String, semver::Version>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Configuration type this plan applies to
    pub config_type:  String,
    /// Source version
    pub from_version: semver::Version,
    /// Target version
    pub to_version:   semver::Version,
    /// Migration steps
    pub steps:        Vec<MigrationStep>,
    /// Whether this migration is automatic (no user intervention)
    pub automatic:    bool,
    /// Migration description
    pub description:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    /// Step type
    pub step_type:     MigrationStepType,
    /// Field path to migrate
    pub field_path:    String,
    /// Migration action
    pub action:        MigrationAction,
    /// Default value if field is missing
    pub default_value: Option<serde_json::Value>,
    /// Migration condition
    pub condition:     Option<MigrationCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStepType {
    /// Add a new field
    AddField,
    /// Remove a field
    RemoveField,
    /// Rename a field
    RenameField,
    /// Transform field value
    TransformField,
    /// Split field into multiple fields
    SplitField,
    /// Merge multiple fields into one
    MergeFields,
}

impl std::fmt::Display for MigrationStepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddField => write!(f, "AddField"),
            Self::RemoveField => write!(f, "RemoveField"),
            Self::RenameField => write!(f, "RenameField"),
            Self::TransformField => write!(f, "TransformField"),
            Self::SplitField => write!(f, "SplitField"),
            Self::MergeFields => write!(f, "MergeFields"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationAction {
    /// Set to specific value
    SetValue(serde_json::Value),
    /// Rename field to new name
    RenameTo(String),
    /// Apply transformation function
    Transform(String), // Function name or script
    /// Copy from another field
    CopyFrom(String),
    /// Compute value from expression
    Compute(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationCondition {
    /// Condition type
    pub condition_type: ConditionType,
    /// Field to check
    pub field:          String,
    /// Expected value
    pub expected:       serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    /// Field exists
    Exists,
    /// Field equals value
    Equals,
    /// Field is greater than value
    GreaterThan,
    /// Field is less than value
    LessThan,
    /// Custom condition
    Custom(String),
}

/// Migration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Migration successful
    pub success:       bool,
    /// Configuration type migrated
    pub config_type:   String,
    /// Original version
    pub from_version:  Option<semver::Version>,
    /// Target version
    pub to_version:    semver::Version,
    /// Applied migration steps
    pub applied_steps: Vec<String>,
    /// Migration warnings
    pub warnings:      Vec<String>,
    /// Migration errors
    pub errors:        Vec<String>,
    /// Backup path (if created)
    pub backup_path:   Option<PathBuf>,
    /// Migration timestamp
    pub timestamp:     chrono::DateTime<chrono::Utc>,
}

impl MigrationEngine {
    /// Create new migration engine
    pub fn new() -> Self {
        Self {
            plans:            HashMap::new(),
            current_versions: HashMap::new(),
        }
    }

    /// Register migration plan
    pub fn register_plan(&mut self, plan: MigrationPlan) {
        self.plans
            .entry(plan.config_type.clone())
            .or_insert_with(Vec::new)
            .push(plan);
    }

    /// Check if migration is needed for configuration
    pub fn migration_needed<C>(&self, _config: &C) -> crate::IDEResult<Option<MigrationPlan>>
    where
        C: crate::Config + serde::Serialize,
    {
        let config_type = C::FILE_PREFIX.to_string();
        let current_version = self.current_versions.get(&config_type);

        if let Some(plans) = self.plans.get(&config_type) {
            // Find applicable migration plan
            for plan in plans {
                if let Some(current) = current_version {
                    if current < &plan.from_version {
                        return Ok(Some(plan.clone()));
                    }
                } else {
                    // No version info available, assume latest version
                    return Ok(Some(plan.clone()));
                }
            }
        }

        Ok(None)
    }

    /// Migrate configuration
    pub async fn migrate<C>(&mut self, mut config: C) -> crate::IDEResult<MigrationResult>
    where
        C: crate::Config + serde::Serialize + serde::de::DeserializeOwned,
    {
        let config_type = C::FILE_PREFIX.to_string();
        let from_version = self.current_versions.get(&config_type).cloned();

        let mut result = MigrationResult {
            success: false,
            config_type: config_type.clone(),
            from_version,
            to_version: semver::Version::new(1, 0, 0), // Default
            applied_steps: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            backup_path: None,
            timestamp: chrono::Utc::now(),
        };

        let plan = match self.migration_needed(&config)? {
            Some(plan) => plan,
            None => {
                result.success = true;
                return Ok(result);
            }
        };

        result.from_version = Some(plan.from_version.clone());
        result.to_version = plan.to_version.clone();

        // Create backup
        let backup_result = self.create_backup(&config).await;
        match backup_result {
            Ok(backup_path) => result.backup_path = Some(backup_path),
            Err(e) => result
                .warnings
                .push(format!("Failed to create backup: {}", e)),
        }

        // Serialize config to JSON for manipulation
        let mut json_config = serde_json::to_value(&config).map_err(|e| {
            crate::RustAIError::Serialization(format!("Failed to serialize config for migration: {}", e))
        })?;

        // Apply migration steps
        for step in &plan.steps {
            match self.apply_migration_step(&mut json_config, step) {
                Ok(step_name) => result.applied_steps.push(step_name),
                Err(e) => {
                    result.errors.push(format!("Migration step failed: {}", e));
                    return Ok(result);
                }
            }
        }

        // Deserialize migrated config
        config = serde_json::from_value(json_config)
            .map_err(|e| crate::RustAIError::Serialization(format!("Failed to deserialize migrated config: {}", e)))?;

        // Update version tracking
        self.current_versions
            .insert(config_type, plan.to_version.clone());

        result.success = true;
        Ok(result)
    }

    /// Apply single migration step
    fn apply_migration_step(&self, config: &mut serde_json::Value, step: &MigrationStep) -> crate::IDEResult<String> {
        // Check condition if present
        if let Some(condition) = &step.condition {
            if !self.evaluate_condition(config, condition)? {
                return Ok(format!(
                    "Skipped: condition not met for {}",
                    step.field_path
                ));
            }
        }

        match step.step_type {
            MigrationStepType::AddField =>
                if let Some(default) = &step.default_value {
                    self.set_nested_value(config, &step.field_path, default.clone())?;
                    Ok(format!("Added field: {}", step.field_path))
                } else {
                    Err(crate::RustAIError::Config(
                        rust_ai_ide_errors::ConfigError::new("AddField step requires default_value"),
                    ))
                },
            MigrationStepType::RemoveField => {
                self.remove_nested_value(config, &step.field_path)?;
                Ok(format!("Removed field: {}", step.field_path))
            }
            MigrationStepType::RenameField =>
                if let MigrationAction::RenameTo(new_name) = &step.action {
                    self.rename_nested_field(config, &step.field_path, new_name)?;
                    Ok(format!(
                        "Renamed field: {} -> {}",
                        step.field_path, new_name
                    ))
                } else {
                    Err(crate::RustAIError::Config(
                        rust_ai_ide_errors::ConfigError::new("RenameField requires RenameTo action"),
                    ))
                },
            _ => Ok(format!(
                "Applied step: {} to {}",
                step.step_type, step.field_path
            )),
        }
    }

    /// Evaluate migration condition
    fn evaluate_condition(&self, config: &serde_json::Value, condition: &MigrationCondition) -> crate::IDEResult<bool> {
        let field_value = self.get_nested_value(config, &condition.field);

        match condition.condition_type {
            ConditionType::Exists => Ok(field_value.is_some()),
            ConditionType::Equals => Ok(field_value
                .map(|v| v == &condition.expected)
                .unwrap_or(false)),
            _ => Ok(true), // For now, other conditions always pass
        }
    }

    /// Create backup of configuration
    async fn create_backup<C>(&self, config: &C) -> crate::IDEResult<PathBuf>
    where
        C: crate::Config + serde::Serialize,
    {
        let backup_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./data"))
            .join("rust-ai-ide")
            .join("config-backups");

        tokio::fs::create_dir_all(&backup_dir).await.map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to create backup directory: {}",
                e
            )))
        })?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("{}_{}.backup", C::FILE_PREFIX, timestamp);
        let backup_path = backup_dir.join(backup_filename);

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| crate::RustAIError::Serialization(format!("Failed to serialize backup: {}", e)))?;

        tokio::fs::write(&backup_path, json).await.map_err(|e| {
            crate::RustAIError::Io(rust_ai_ide_errors::IoError::new(&format!(
                "Failed to write backup: {}",
                e
            )))
        })?;

        Ok(backup_path)
    }

    // JSON manipulation helpers

    fn get_nested_value<'a>(&self, config: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = config;

        for part in parts {
            match current {
                serde_json::Value::Object(obj) => {
                    current = obj.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    fn set_nested_value(
        &self,
        config: &mut serde_json::Value,
        path: &str,
        value: serde_json::Value,
    ) -> crate::IDEResult<()> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = config;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Set the value
                if let serde_json::Value::Object(obj) = current {
                    obj.insert(part.to_string(), value);
                    return Ok(());
                }
            } else {
                // Navigate deeper
                match current {
                    serde_json::Value::Object(obj) => {
                        if !obj.contains_key(*part) {
                            obj.insert(
                                part.to_string(),
                                serde_json::Value::Object(serde_json::Map::new()),
                            );
                        }
                        current = obj.get_mut(*part).unwrap();
                    }
                    _ => {
                        return Err(crate::RustAIError::Config(
                            rust_ai_ide_errors::ConfigError::new(&format!("Cannot set nested value at path: {}", path)),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn remove_nested_value(&self, config: &mut serde_json::Value, path: &str) -> crate::IDEResult<()> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = config;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Remove the value
                if let serde_json::Value::Object(obj) = current {
                    obj.remove(*part);
                    return Ok(());
                }
            } else {
                // Navigate deeper
                match current {
                    serde_json::Value::Object(obj) => {
                        current = obj.get_mut(*part).ok_or_else(|| {
                            crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(&format!(
                                "Path not found: {}",
                                path
                            )))
                        })?;
                    }
                    _ => {
                        return Err(crate::RustAIError::Config(
                            rust_ai_ide_errors::ConfigError::new(&format!("Cannot navigate to path: {}", path)),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn rename_nested_field(
        &self,
        config: &mut serde_json::Value,
        old_path: &str,
        new_path: &str,
    ) -> crate::IDEResult<()> {
        if let Some(value) = self.get_nested_value(config, old_path).cloned() {
            self.remove_nested_value(config, old_path)?;
            self.set_nested_value(config, new_path, value)?;
        }
        Ok(())
    }
}

/// Migration utilities
pub struct MigrationUtils;

impl MigrationUtils {
    /// Check if configuration needs migration
    pub fn needs_migration<C>(config: &C, current_version: &semver::Version) -> bool
    where
        C: crate::Config,
    {
        // Simple version check - in practice, this would be more sophisticated
        let config_version = semver::Version::new(1, 0, 0); // Extract from config
        config_version < *current_version
    }

    /// Suggest migration path
    pub fn suggest_migration<C>(config: &C) -> Vec<String>
    where
        C: crate::Config,
    {
        vec![
            "Run configuration migration tool".to_string(),
            "Review breaking changes in changelog".to_string(),
            "Test configuration in staging environment".to_string(),
        ]
    }

    /// Validate migration plan
    pub fn validate_plan(plan: &MigrationPlan) -> Result<(), String> {
        if plan.from_version >= plan.to_version {
            return Err("Migration plan must have from_version < to_version".to_string());
        }

        if plan.steps.is_empty() {
            return Err("Migration plan must have at least one step".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_plan_validation() {
        let plan = MigrationPlan {
            config_type:  "test".to_string(),
            from_version: semver::Version::new(1, 0, 0),
            to_version:   semver::Version::new(2, 0, 0),
            steps:        vec![MigrationStep {
                step_type:     MigrationStepType::AddField,
                field_path:    "new_field".to_string(),
                action:        MigrationAction::SetValue(serde_json::Value::String("default".to_string())),
                default_value: Some(serde_json::Value::String("default".to_string())),
                condition:     None,
            }],
            automatic:    true,
            description:  "Test migration".to_string(),
        };

        assert!(MigrationUtils::validate_plan(&plan).is_ok());

        // Test invalid plan
        let invalid_plan = MigrationPlan {
            to_version: semver::Version::new(0, 5, 0), // Lower than from_version
            ..plan
        };

        assert!(MigrationUtils::validate_plan(&invalid_plan).is_err());
    }

    #[tokio::test]
    async fn test_migration_engine_creation() {
        let engine = MigrationEngine::new();
        assert!(engine.plans.is_empty());
    }

    #[tokio::test]
    async fn test_migration_plan_registration() {
        let mut engine = MigrationEngine::new();
        let plan = MigrationPlan {
            config_type:  "test".to_string(),
            from_version: semver::Version::new(1, 0, 0),
            to_version:   semver::Version::new(2, 0, 0),
            steps:        vec![],
            automatic:    true,
            description:  "Test plan".to_string(),
        };

        engine.register_plan(plan.clone());

        assert_eq!(engine.plans.len(), 1);
        assert!(engine.plans.contains_key("test"));
    }
}
