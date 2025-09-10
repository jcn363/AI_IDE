//! # Phase 4.1 Configuration Management
//!
//! This module handles all configuration aspects for the Advanced AI Applications system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main Phase 4 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase4Config {
    /// Development assistance configuration
    pub development_assistance: DevelopmentAssistanceConfig,

    /// Workflow orchestration configuration
    pub workflow_orchestration: WorkflowOrchestrationConfig,

    /// Code analysis configuration
    pub code_analysis: CodeAnalysisConfig,

    /// Testing configuration
    pub testing: TestingConfig,

    /// Collaboration configuration
    pub collaboration: CollaborationConfig,

    /// Performance configuration
    pub performance: PerformanceConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Caching configuration
    pub caching: CachingConfig,
}

impl Default for Phase4Config {
    fn default() -> Self {
        Self {
            development_assistance: DevelopmentAssistanceConfig::default(),
            workflow_orchestration: WorkflowOrchestrationConfig::default(),
            code_analysis: CodeAnalysisConfig::default(),
            testing: TestingConfig::default(),
            collaboration: CollaborationConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            caching: CachingConfig::default(),
        }
    }
}

/// Development assistance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentAssistanceConfig {
    /// Enable smart suggestions
    pub smart_suggestions: bool,

    /// Enable proactive improvements
    pub proactive_improvements: bool,

    /// Enable context-aware code generation
    pub context_aware_generation: bool,

    /// Maximum suggestions per request
    pub max_suggestions: usize,

    /// Confidence threshold for suggestions
    pub confidence_threshold: f32,

    /// Enable learning from user feedback
    pub learning_enabled: bool,
}

impl Default for DevelopmentAssistanceConfig {
    fn default() -> Self {
        Self {
            smart_suggestions: true,
            proactive_improvements: true,
            context_aware_generation: true,
            max_suggestions: 5,
            confidence_threshold: 0.7,
            learning_enabled: true,
        }
    }
}

/// Workflow orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOrchestrationConfig {
    /// Enable workflow orchestration
    pub enabled: bool,

    /// Maximum concurrent workflows
    pub max_concurrent_workflows: usize,

    /// Workflow timeout in seconds
    pub workflow_timeout_seconds: u64,

    /// Enable automatic workflow optimization
    pub auto_optimization: bool,

    /// Service discovery enabled
    pub service_discovery_enabled: bool,

    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,
}

impl Default for WorkflowOrchestrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_workflows: 10,
            workflow_timeout_seconds: 300, // 5 minutes
            auto_optimization: true,
            service_discovery_enabled: true,
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin allocation
    RoundRobin,

    /// Least connections strategy
    LeastConnections,

    /// Load-based allocation
    LoadBased,

    /// Performance-based allocation
    PerformanceBased,
}

/// Code analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisConfig {
    /// Enable semantic analysis
    pub semantic_analysis: bool,

    /// Enable pattern recognition
    pub pattern_recognition: bool,

    /// Enable dependency analysis
    pub dependency_analysis: bool,

    /// Enable performance profiling
    pub performance_profiling: bool,

    /// Analysis cache size
    pub cache_size_mb: usize,

    /// Analysis timeout in seconds
    pub analysis_timeout_seconds: u64,

    /// Enable parallel processing
    pub parallel_processing: bool,

    /// Maximum analysis depth
    pub max_analysis_depth: usize,
}

impl Default for CodeAnalysisConfig {
    fn default() -> Self {
        Self {
            semantic_analysis: true,
            pattern_recognition: true,
            dependency_analysis: true,
            performance_profiling: true,
            cache_size_mb: 512,
            analysis_timeout_seconds: 60,
            parallel_processing: true,
            max_analysis_depth: 10,
        }
    }
}

/// Testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingConfig {
    /// Enable automated test generation
    pub auto_test_generation: bool,

    /// Enable coverage optimization
    pub coverage_optimization: bool,

    /// Target test coverage percentage
    pub target_coverage_percent: f32,

    /// Enable mutation testing
    pub mutation_testing: bool,

    /// Testing timeout in seconds
    pub test_timeout_seconds: u64,

    /// Enable parallel test execution
    pub parallel_execution: bool,

    /// Maximum test processes
    pub max_test_processes: usize,

    /// Test generation strategies
    pub generation_strategies: Vec<TestGenerationStrategy>,
}

impl Default for TestingConfig {
    fn default() -> Self {
        Self {
            auto_test_generation: true,
            coverage_optimization: true,
            target_coverage_percent: 85.0,
            mutation_testing: false,
            test_timeout_seconds: 60,
            parallel_execution: true,
            max_test_processes: 4,
            generation_strategies: vec![
                TestGenerationStrategy::UnitTests,
                TestGenerationStrategy::IntegrationTests,
            ],
        }
    }
}

/// Test generation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestGenerationStrategy {
    /// Generate unit tests
    UnitTests,

    /// Generate integration tests
    IntegrationTests,

    /// Generate end-to-end tests
    EndToEndTests,

    /// Generate property-based tests
    PropertyBasedTests,

    /// Generate mutation tests
    MutationTests,
}

/// Collaboration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    /// Enable real-time collaboration
    pub real_time_enabled: bool,

    /// Maximum collaborators per session
    pub max_collaborators: usize,

    /// Collaboration session timeout
    pub session_timeout_minutes: u64,

    /// Enable shared insights
    pub shared_insights: bool,

    /// Enable collaborative workflows
    pub collaborative_workflows: bool,

    /// Collaboration features
    pub features: Vec<CollaborationFeature>,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            real_time_enabled: true,
            max_collaborators: 10,
            session_timeout_minutes: 120,
            shared_insights: true,
            collaborative_workflows: true,
            features: vec![
                CollaborationFeature::CodeReview,
                CollaborationFeature::SharedEditing,
            ],
        }
    }
}

/// Collaboration features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationFeature {
    /// Code review functionality
    CodeReview,

    /// Shared code editing
    SharedEditing,

    /// Pair debugging
    PairDebugging,

    /// Knowledge sharing
    KnowledgeSharing,

    /// Project insights sharing
    InsightsSharing,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Memory limit in MB
    pub memory_limit_mb: usize,

    /// CPU usage limit percentage
    pub cpu_limit_percent: f32,

    /// Enable resource monitoring
    pub resource_monitoring: bool,

    /// Background task priority
    pub background_task_priority: TaskPriority,

    /// Cache refresh interval in seconds
    pub cache_refresh_interval: u64,

    /// Enable performance profiling
    pub performance_profiling: bool,

    /// Profiling data retention days
    pub profiling_retention_days: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            memory_limit_mb: 2048,
            cpu_limit_percent: 80.0,
            resource_monitoring: true,
            background_task_priority: TaskPriority::Normal,
            cache_refresh_interval: 300, // 5 minutes
            performance_profiling: true,
            profiling_retention_days: 30,
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Low priority
    Low,

    /// Normal priority
    Normal,

    /// High priority
    High,

    /// Critical priority
    Critical,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security scanning
    pub security_scanning: bool,

    /// Vulnerability check frequency
    pub vulnerability_check_frequency: CheckFrequency,

    /// Audit logging enabled
    pub audit_logging: bool,

    /// Data encryption enabled
    pub data_encryption: bool,

    /// API rate limiting
    pub api_rate_limiting: bool,

    /// Security scan depth
    pub scan_depth: SecurityScanDepth,

    /// Require security approval for high-risk changes
    pub security_approval: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            security_scanning: true,
            vulnerability_check_frequency: CheckFrequency::Daily,
            audit_logging: true,
            data_encryption: true,
            api_rate_limiting: true,
            scan_depth: SecurityScanDepth::Comprehensive,
            security_approval: false,
        }
    }
}

/// Security check frequencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckFrequency {
    /// Check real-time
    RealTime,

    /// Check every 15 minutes
    Every15Minutes,

    /// Check hourly
    Hourly,

    /// Check daily
    Daily,

    /// Check weekly
    Weekly,
}

/// Security scan depths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityScanDepth {
    /// Basic scanning
    Basic,

    /// Comprehensive scanning
    Comprehensive,

    /// Enterprise-level scanning
    Enterprise,
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    /// Enable caching
    pub enabled: bool,

    /// Cache size in MB
    pub cache_size_mb: usize,

    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,

    /// Cache TTI in seconds
    pub cache_tti_seconds: u64,

    /// Distributed cache enabled
    pub distributed_cache: bool,

    /// Cache compression enabled
    pub cache_compression: bool,

    /// Cache monitoring enabled
    pub cache_monitoring: bool,
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_size_mb: 1024,
            cache_ttl_seconds: 3600, // 1 hour
            cache_tti_seconds: 1800, // 30 minutes
            distributed_cache: false,
            cache_compression: true,
            cache_monitoring: true,
        }
    }
}

/// Configuration management utilities
pub struct ConfigurationManager {
    /// Current configuration
    config: std::sync::RwLock<Phase4Config>,

    /// Configuration file path
    config_path: Option<String>,

    /// Configuration validation
    validation_rules: Vec<Box<dyn ValidationRule + Send + Sync>>,
}

impl ConfigurationManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: std::sync::RwLock::new(Phase4Config::default()),
            config_path: None,
            validation_rules: Vec::new(),
        }
    }

    /// Load configuration from file
    pub async fn load_from_file(&mut self, path: &str) -> crate::errors::Phase4Result<()> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| crate::errors::Phase4Error::Configuration(
                crate::errors::ConfigurationError::FileAccess(format!("Failed to read config file: {}", e))
            ))?;

        let config: Phase4Config = serde_json::from_str(&content)
            .map_err(|e| crate::errors::Phase4Error::Configuration(
                crate::errors::ConfigurationError::ParseError(format!("Failed to parse config: {}", e))
            ))?;

        self.validate_configuration(&config)?;
        *self.config.write().unwrap() = config;
        self.config_path = Some(path.to_string());

        Ok(())
    }

    /// Save configuration to file
    pub async fn save_to_file(&self) -> crate::errors::Phase4Result<()> {
        let config = self.config.read().unwrap().clone();
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| crate::errors::Phase4Error::Configuration(
                crate::errors::ConfigurationError::ParseError(format!("Failed to serialize config: {}", e))
            ))?;

        if let Some(path) = &self.config_path {
            tokio::fs::write(path, content).await
                .map_err(|e| crate::errors::Phase4Error::Configuration(
                    crate::errors::ConfigurationError::FileAccess(format!("Failed to write config file: {}", e))
                ))?;
        }

        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> Phase4Config {
        self.config.read().unwrap().clone()
    }

    /// Update configuration
    pub fn update_config(&self, new_config: Phase4Config) -> crate::errors::Phase4Result<()> {
        self.validate_configuration(&new_config)?;
        *self.config.write().unwrap() = new_config;
        Ok(())
    }

    /// Validate configuration
    fn validate_configuration(&self, config: &Phase4Config) -> crate::errors::Phase4Result<()> {
        // Basic validation rules
        if config.performance.memory_limit_mb == 0 {
            return Err(crate::errors::Phase4Error::Configuration(
                crate::errors::ConfigurationError::InvalidValue("Memory limit cannot be zero".to_string())
            ));
        }

        if config.workflow_orchestration.max_concurrent_workflows == 0 {
            return Err(crate::errors::Phase4Error::Configuration(
                crate::errors::ConfigurationError::InvalidValue("Max concurrent workflows cannot be zero".to_string())
            ));
        }

        if config.development_assistance.confidence_threshold < 0.0 || config.development_assistance.confidence_threshold > 1.0 {
            return Err(crate::errors::Phase4Error::Configuration(
                crate::errors::ConfigurationError::InvalidValue("Confidence threshold must be between 0.0 and 1.0".to_string())
            ));
        }

        // Run custom validation rules
        for rule in &self.validation_rules {
            rule.validate(config)?;
        }

        Ok(())
    }

    /// Add validation rule
    pub fn add_validation_rule(&mut self, rule: Box<dyn ValidationRule + Send + Sync>) {
        self.validation_rules.push(rule);
    }
}

/// Configuration validation rule trait
pub trait ValidationRule {
    fn validate(&self, config: &Phase4Config) -> crate::errors::Phase4Result<()>;
}

/// Security validation rule
pub struct SecurityValidationRule;

impl ValidationRule for SecurityValidationRule {
    fn validate(&self, config: &Phase4Config) -> crate::errors::Phase4Result<()> {
        // Ensure security settings are reasonable
        if config.security.security_scanning && config.security.vulnerability_check_frequency == CheckFrequency::Weekly {
            // This is a warning, but still valid
        }
        Ok(())
    }
}

/// Performance validation rule
pub struct PerformanceValidationRule;

impl ValidationRule for PerformanceValidationRule {
    fn validate(&self, config: &Phase4Config) -> crate::errors::Phase4Result<()> {
        // Validate performance settings
        if config.caching.enabled && config.caching.cache_size_mb == 0 {
            return Err(crate::errors::Phase4Error::Configuration(
                crate::errors::ConfigurationError::InvalidValue("Cache size must be greater than 0 when caching is enabled".to_string())
            ));
        }

        if config.performance.cpu_limit_percent > 100.0 {
            return Err(crate::errors::Phase4Error::Configuration(
                crate::errors::ConfigurationError::InvalidValue("CPU limit percentage cannot exceed 100%".to_string())
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Phase4Config::default();

        assert!(config.development_assistance.smart_suggestions);
        assert_eq!(config.workflow_orchestration.max_concurrent_workflows, 10);
        assert!(config.caching.enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Phase4Config::default();
        config.performance.memory_limit_mb = 0;

        let manager = ConfigurationManager::new();
        assert!(manager.validate_configuration(&config).is_err());
    }

    #[test]
    fn test_performance_validation_rule() {
        let rule = PerformanceValidationRule;

        // Valid config
        let mut valid_config = Phase4Config::default();
        assert!(rule.validate(&valid_config).is_ok());

        // Invalid config
        valid_config.caching.cache_size_mb = 0;
        assert!(rule.validate(&valid_config).is_err());
    }
}