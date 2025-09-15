//! # Rust AI IDE Workspace Optimizer
//!
//! This crate provides comprehensive workspace optimization capabilities for the large-scale
//! Rust AI IDE project with 67 crates. It implements various optimization strategies to
//! reduce build complexity and improve development workflow.
//!
//! ## Core Components
//!
//! - **DependencyAnalyzer**: Analyzes and detects circular dependencies between crates
//! - **ModularLoader**: Implements on-demand crate loading for optional components
//! - **BuildOptimizer**: Optimizes compilation through selective compilation and caching
//! - **WorkspaceHealthMonitor**: Monitors and reports workspace health metrics
//! - **ConsolidationTools**: Tools for reducing crate proliferation and dependencies
//!
//! ## Optimization Strategies
//!
//! - **Dependency Analysis**: Automated detection of circular dependencies and unused dependencies
//! - **Selective Compilation**: Compile only necessary crates for specific development tasks
//! - **Feature Flags**: Implement feature flags for optional functionality
//! - **Dynamic Loading**: Load heavy AI/ML components on-demand
//! - **Build Caching**: Leverage incremental compilation and build artifact caching
//! - **Parallel Compilation**: Optimize compilation parallelism across crates
//!
//! ## Safety Features
//!
//! - Non-destructive optimization recommendations
//! - Backup mechanisms for critical changes
//! - Gradual adoption with rollback capabilities
//! - Validation of optimization impacts
//! - Comprehensive error handling

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod analyzer;
pub mod build_optimizer;
pub mod consolidation_tools;
pub mod error;
pub mod health_monitor;
pub mod loader;
pub mod types;

// Re-export main types for convenience
pub use analyzer::DependencyAnalyzer;
pub use build_optimizer::BuildOptimizer;
pub use consolidation_tools::ConsolidationTools;
pub use error::{OptimizerError, OptimizerResult};
pub use health_monitor::WorkspaceHealthMonitor;
pub use loader::ModularLoader;
pub use types::*;

/// Version information for the workspace optimizer
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main workspace optimizer service
#[derive(Debug, Clone)]
pub struct WorkspaceOptimizer {
    /// Dependency analyzer for circular dependency detection
    pub analyzer:            DependencyAnalyzer,
    /// Modular loader for on-demand crate loading
    pub loader:              ModularLoader,
    /// Build optimizer for compilation optimization
    pub build_optimizer:     BuildOptimizer,
    /// Health monitor for workspace metrics
    pub health_monitor:      WorkspaceHealthMonitor,
    /// Consolidation tools for crate optimization
    pub consolidation_tools: ConsolidationTools,
}

impl WorkspaceOptimizer {
    /// Create a new workspace optimizer instance
    ///
    /// # Returns
    /// A new `WorkspaceOptimizer` instance with all components initialized
    pub async fn new() -> OptimizerResult<Self> {
        Ok(Self {
            analyzer:            DependencyAnalyzer::new().await?,
            loader:              ModularLoader::new().await?,
            build_optimizer:     BuildOptimizer::new().await?,
            health_monitor:      WorkspaceHealthMonitor::new().await?,
            consolidation_tools: ConsolidationTools::new().await?,
        })
    }

    /// Perform comprehensive workspace optimization
    ///
    /// # Returns
    /// Optimization results with metrics and recommendations
    pub async fn optimize_workspace(&self) -> OptimizerResult<OptimizationResults> {
        let mut results = OptimizationResults::default();

        // Analyze dependencies for circular references
        let dependency_analysis = self.analyzer.analyze_workspace().await?;
        results.dependency_analysis = Some(dependency_analysis);

        // Optimize build configuration
        let build_optimization = self.build_optimizer.optimize_build().await?;
        results.build_optimization = Some(build_optimization);

        // Monitor workspace health
        let health_metrics = self.health_monitor.collect_metrics().await?;
        results.health_metrics = Some(health_metrics);

        // Generate consolidation recommendations
        let consolidation_recommendations = self.consolidation_tools.generate_recommendations().await?;
        results.consolidation_recommendations = Some(consolidation_recommendations);

        Ok(results)
    }

    /// Apply optimization recommendations
    ///
    /// # Arguments
    /// * `recommendations` - The recommendations to apply
    ///
    /// # Returns
    /// Results of applying the optimizations
    pub async fn apply_optimizations(
        &self,
        recommendations: OptimizationResults,
    ) -> OptimizerResult<OptimizationResults> {
        let mut applied_results = OptimizationResults::default();

        // Apply dependency optimizations
        if let Some(ref analysis) = recommendations.dependency_analysis {
            let applied = self.analyzer.apply_optimizations(analysis.clone()).await?;
            applied_results.dependency_analysis = Some(applied);
        }

        // Apply build optimizations
        if let Some(ref build_opt) = recommendations.build_optimization {
            let applied = self
                .build_optimizer
                .apply_optimizations(build_opt.clone())
                .await?;
            applied_results.build_optimization = Some(applied);
        }

        // Apply consolidation optimizations
        if let Some(ref consolidation) = recommendations.consolidation_recommendations {
            let applied = self
                .consolidation_tools
                .apply_recommendations(consolidation.clone())
                .await?;
            applied_results.consolidation_recommendations = Some(applied);
        }

        Ok(applied_results)
    }

    /// Get current workspace health status
    ///
    /// # Returns
    /// Current health status and metrics
    pub async fn get_health_status(&self) -> OptimizerResult<HealthStatus> {
        self.health_monitor.get_status().await
    }
}

/// Global workspace optimizer instance using OnceCell for safe initialization
static WORKSPACE_OPTIMIZER: tokio::sync::OnceCell<WorkspaceOptimizer> = tokio::sync::OnceCell::const_new();

/// Initialize the workspace optimizer as a global service
///
/// # Returns
/// A global workspace optimizer instance
pub async fn init_workspace_optimizer() -> OptimizerResult<&'static WorkspaceOptimizer> {
    WORKSPACE_OPTIMIZER
        .get_or_init(|| async {
            WorkspaceOptimizer::new().await.unwrap() // Safe unwrap in initialization
        })
        .await;

    Ok(WORKSPACE_OPTIMIZER.get().unwrap()) // Safe unwrap after initialization
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workspace_optimizer_creation() {
        let optimizer = WorkspaceOptimizer::new().await;
        assert!(optimizer.is_ok());
    }

    #[tokio::test]
    async fn test_workspace_optimization() {
        let optimizer = WorkspaceOptimizer::new().await.unwrap();
        let results = optimizer.optimize_workspace().await;
        assert!(results.is_ok());
    }
}
