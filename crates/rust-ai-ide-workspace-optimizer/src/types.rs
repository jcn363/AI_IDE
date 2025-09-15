//! Common types and data structures for workspace optimization
//!
//! This module defines all the core types used throughout the workspace optimizer,
//! including optimization results, health metrics, dependency analysis results, and more.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result type for optimizer operations
pub type OptimizerResult<T> = Result<T, OptimizerError>;

/// Performance metrics for build operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    /// Total build time
    pub build_time:        Duration,
    /// Memory usage during build (MB)
    pub memory_usage_mb:   f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Number of crates compiled
    pub crates_compiled:   usize,
    /// Incremental compilation ratio
    pub incremental_ratio: f64,
    /// Timestamp of measurement
    pub timestamp:         DateTime<Utc>,
}

impl Default for BuildMetrics {
    fn default() -> Self {
        Self {
            build_time:        Duration::default(),
            memory_usage_mb:   0.0,
            cpu_usage_percent: 0.0,
            crates_compiled:   0,
            incremental_ratio: 0.0,
            timestamp:         Utc::now(),
        }
    }
}

/// Dependency analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// Circular dependencies found
    pub circular_dependencies: Vec<CircularDependency>,
    /// Unused dependencies
    pub unused_dependencies:   Vec<UnusedDependency>,
    /// Dependency chain depth analysis
    pub dependency_depths:     HashMap<String, usize>,
    /// Total number of dependencies
    pub total_dependencies:    usize,
    /// Analysis timestamp
    pub timestamp:             DateTime<Utc>,
}

impl Default for DependencyAnalysis {
    fn default() -> Self {
        Self {
            circular_dependencies: Vec::new(),
            unused_dependencies:   Vec::new(),
            dependency_depths:     HashMap::new(),
            total_dependencies:    0,
            timestamp:             Utc::now(),
        }
    }
}

/// Circular dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    /// The crates involved in the circular dependency
    pub crates: Vec<String>,
    /// The dependency chain
    pub chain:  Vec<String>,
    /// Impact assessment
    pub impact: DependencyImpact,
}

/// Unused dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedDependency {
    /// The crate that has the unused dependency
    pub crate_name:      String,
    /// The unused dependency name
    pub dependency_name: String,
    /// Last used timestamp (if available)
    pub last_used:       Option<DateTime<Utc>>,
}

/// Impact assessment for dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyImpact {
    /// Low impact - can be resolved easily
    Low,
    /// Medium impact - requires careful consideration
    Medium,
    /// High impact - significant changes required
    High,
    /// Critical impact - may break functionality
    Critical,
}

/// Optimization results container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResults {
    /// Dependency analysis results
    pub dependency_analysis:           Option<DependencyAnalysis>,
    /// Build optimization results
    pub build_optimization:            Option<BuildOptimization>,
    /// Health metrics
    pub health_metrics:                Option<HealthMetrics>,
    /// Consolidation recommendations
    pub consolidation_recommendations: Option<ConsolidationRecommendations>,
    /// Overall optimization score (0-100)
    pub optimization_score:            f64,
    /// Timestamp of optimization
    pub timestamp:                     DateTime<Utc>,
}

impl Default for OptimizationResults {
    fn default() -> Self {
        Self {
            dependency_analysis:           None,
            build_optimization:            None,
            health_metrics:                None,
            consolidation_recommendations: None,
            optimization_score:            0.0,
            timestamp:                     Utc::now(),
        }
    }
}

/// Build optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptimization {
    /// Recommended build profile changes
    pub profile_recommendations: Vec<ProfileRecommendation>,
    /// Feature flag optimizations
    pub feature_optimizations:   Vec<FeatureOptimization>,
    /// Compilation order optimizations
    pub compilation_order:       Vec<String>,
    /// Cache effectiveness metrics
    pub cache_effectiveness:     f64,
    /// Parallel compilation improvements
    pub parallel_improvements:   Vec<ParallelImprovement>,
}

impl Default for BuildOptimization {
    fn default() -> Self {
        Self {
            profile_recommendations: Vec::new(),
            feature_optimizations:   Vec::new(),
            compilation_order:       Vec::new(),
            cache_effectiveness:     0.0,
            parallel_improvements:   Vec::new(),
        }
    }
}

/// Profile recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRecommendation {
    /// Profile name
    pub profile_name:         String,
    /// Recommended changes
    pub changes:              HashMap<String, String>,
    /// Expected improvement
    pub expected_improvement: Duration,
}

/// Feature optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureOptimization {
    /// Crate name
    pub crate_name:   String,
    /// Feature name
    pub feature_name: String,
    /// Optimization action
    pub action:       FeatureAction,
    /// Impact assessment
    pub impact:       f64,
}

/// Feature optimization action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureAction {
    /// Enable the feature
    Enable,
    /// Disable the feature
    Disable,
    /// Make feature optional
    MakeOptional,
    /// Split feature into smaller features
    Split,
}

/// Parallel compilation improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelImprovement {
    /// Improvement description
    pub description:     String,
    /// Expected time savings
    pub time_savings:    Duration,
    /// Affected crates
    pub affected_crates: Vec<String>,
}

/// Health metrics for workspace monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Overall health score (0-100)
    pub overall_score:       f64,
    /// Build health metrics
    pub build_health:        BuildHealth,
    /// Dependency health metrics
    pub dependency_health:   DependencyHealth,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Alert conditions
    pub alerts:              Vec<HealthAlert>,
    /// Timestamp
    pub timestamp:           DateTime<Utc>,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            overall_score:       100.0,
            build_health:        BuildHealth::default(),
            dependency_health:   DependencyHealth::default(),
            performance_metrics: PerformanceMetrics::default(),
            alerts:              Vec::new(),
            timestamp:           Utc::now(),
        }
    }
}

/// Build health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildHealth {
    /// Build success rate (0-100)
    pub success_rate:       f64,
    /// Average build time
    pub average_build_time: Duration,
    /// Build stability score
    pub stability_score:    f64,
    /// Compilation warnings count
    pub warnings_count:     usize,
    /// Compilation errors count
    pub errors_count:       usize,
}

impl Default for BuildHealth {
    fn default() -> Self {
        Self {
            success_rate:       100.0,
            average_build_time: Duration::from_secs(0),
            stability_score:    100.0,
            warnings_count:     0,
            errors_count:       0,
        }
    }
}

/// Dependency health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    /// Number of circular dependencies
    pub circular_dependencies_count:    usize,
    /// Number of unused dependencies
    pub unused_dependencies_count:      usize,
    /// Average dependency depth
    pub average_dependency_depth:       f64,
    /// Outdated dependencies count
    pub outdated_dependencies_count:    usize,
    /// Security vulnerabilities count
    pub security_vulnerabilities_count: usize,
}

impl Default for DependencyHealth {
    fn default() -> Self {
        Self {
            circular_dependencies_count:    0,
            unused_dependencies_count:      0,
            average_dependency_depth:       0.0,
            outdated_dependencies_count:    0,
            security_vulnerabilities_count: 0,
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Memory usage (MB)
    pub memory_usage_mb:   f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Disk I/O operations per second
    pub disk_iops:         f64,
    /// Network I/O (if applicable)
    pub network_iops:      Option<f64>,
    /// Active threads count
    pub active_threads:    usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            memory_usage_mb:   0.0,
            cpu_usage_percent: 0.0,
            disk_iops:         0.0,
            network_iops:      None,
            active_threads:    0,
        }
    }
}

/// Health alert for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Alert level
    pub level:              AlertLevel,
    /// Alert message
    pub message:            String,
    /// Affected component
    pub component:          String,
    /// Recommended action
    pub recommended_action: String,
    /// Timestamp
    pub timestamp:          DateTime<Utc>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert requiring immediate attention
    Critical,
}

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Current status
    pub status:        SystemStatus,
    /// Detailed metrics
    pub metrics:       HealthMetrics,
    /// Active alerts
    pub active_alerts: Vec<HealthAlert>,
    /// Last updated timestamp
    pub last_updated:  DateTime<Utc>,
}

/// System status indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemStatus {
    /// System is healthy
    Healthy,
    /// System has minor issues
    Warning,
    /// System has significant issues
    Error,
    /// System is in critical condition
    Critical,
}

/// Consolidation recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationRecommendations {
    /// Crates that can be consolidated
    pub consolidatable_crates:      Vec<CrateConsolidation>,
    /// Feature flag optimizations
    pub feature_flag_optimizations: Vec<FeatureOptimization>,
    /// Dependency cleanup recommendations
    pub dependency_cleanup:         Vec<DependencyCleanup>,
    /// Estimated time savings
    pub estimated_time_savings:     Duration,
    /// Risk assessment
    pub risk_assessment:            ConsolidationRisk,
}

impl Default for ConsolidationRecommendations {
    fn default() -> Self {
        Self {
            consolidatable_crates:      Vec::new(),
            feature_flag_optimizations: Vec::new(),
            dependency_cleanup:         Vec::new(),
            estimated_time_savings:     Duration::from_secs(0),
            risk_assessment:            ConsolidationRisk::Low,
        }
    }
}

/// Crate consolidation recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateConsolidation {
    /// Primary crate to consolidate into
    pub primary_crate:          String,
    /// Crates to be merged
    pub merge_crates:           Vec<String>,
    /// Estimated effort (hours)
    pub estimated_effort_hours: f64,
    /// Complexity score (0-100)
    pub complexity_score:       f64,
    /// Benefits description
    pub benefits:               String,
}

/// Dependency cleanup recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCleanup {
    /// Crate name
    pub crate_name:             String,
    /// Dependencies to remove
    pub dependencies_to_remove: Vec<String>,
    /// Dependencies to update
    pub dependencies_to_update: Vec<String>,
    /// Estimated impact
    pub estimated_impact:       f64,
}

/// Consolidation risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsolidationRisk {
    /// Low risk - safe to proceed
    Low,
    /// Medium risk - requires testing
    Medium,
    /// High risk - significant changes required
    High,
    /// Critical risk - may break functionality
    Critical,
}

/// Configuration for workspace optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Enable dependency analysis
    pub enable_dependency_analysis: bool,
    /// Enable build optimization
    pub enable_build_optimization:  bool,
    /// Enable health monitoring
    pub enable_health_monitoring:   bool,
    /// Enable consolidation tools
    pub enable_consolidation_tools: bool,
    /// Maximum memory usage (MB)
    pub max_memory_mb:              u64,
    /// Maximum build time
    pub max_build_time:             Duration,
    /// Alert thresholds
    pub alert_thresholds:           AlertThresholds,
    /// Feature flags for optimization strategies
    pub feature_flags:              HashMap<String, bool>,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_dependency_analysis: true,
            enable_build_optimization:  true,
            enable_health_monitoring:   true,
            enable_consolidation_tools: false,
            max_memory_mb:              2048,                     // 2GB
            max_build_time:             Duration::from_secs(300), // 5 minutes
            alert_thresholds:           AlertThresholds::default(),
            feature_flags:              HashMap::new(),
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum build time threshold
    pub max_build_time_threshold:        Duration,
    /// Maximum memory usage threshold (MB)
    pub max_memory_threshold_mb:         f64,
    /// CPU usage warning threshold (%)
    pub cpu_warning_threshold_percent:   f64,
    /// Circular dependencies warning threshold
    pub circular_deps_warning_threshold: usize,
    /// Unused dependencies warning threshold
    pub unused_deps_warning_threshold:   usize,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_build_time_threshold:        Duration::from_secs(600), // 10 minutes
            max_memory_threshold_mb:         1024.0,                   // 1GB
            cpu_warning_threshold_percent:   90.0,
            circular_deps_warning_threshold: 5,
            unused_deps_warning_threshold:   10,
        }
    }
}
