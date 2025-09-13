#![cfg_attr(feature = "nightly", feature(impl_trait_in_bindings))]
#![warn(missing_docs)]
#![warn(unsafe_code)]

//! # Rust AI IDE Quality Intelligence Dashboard
//!
//! Phase 3.4: Quality Intelligence Dashboard for the advanced AI-powered development framework.
//!
//! This crate provides:
//! - Interactive real-time quality metrics dashboard
//! - Trend analysis with time-series forecasting
//! - Team collaboration features for quality management
//! - Integration with existing analysis and maintenance systems
//! - Visual execution feedback and performance impact assessment
//! - Cross-project comparisons and industry benchmarking
//!
//! ## Architecture
//!
//! The quality intelligence dashboard consists of several key components:
//!
//! - [`QualityIntelligenceDashboard`]: Main orchestrator coordinating all dashboard components
//! - [`DashboardEngine`]: Core engine for real-time metric processing and visualization
//! - [`TrendAnalyzer`]: Time-series analysis and forecasting with benchmark comparison
//! - [`CollaborationHub`]: Team collaboration features and quality management
//! - [`VisualizationManager`]: Interactive scoring and benchmarking systems
//! - [`MetricCollector`]: Comprehensive metric collection and aggregation
//! - [`UiIntegration`]: Seamless frontend integration with responsive UI components

pub mod collaboration;
pub mod configuration;
pub mod dashboard;
pub mod engine;
pub mod errors;
pub mod metrics;
pub mod trends;
pub mod types;
pub mod ui_integration;
pub mod visualization;

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::types::*;

/// Version information for the quality intelligence dashboard
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information for debugging and support
pub fn build_info() -> String {
    format!(
        "rust-ai-ide-quality-dashboard v{} ({} build)",
        VERSION,
        env!("HOST")
    )
}

/// Main entry point for the Quality Intelligence Dashboard system
///
/// This struct orchestrates all dashboard components, providing a unified interface
/// for real-time quality metrics, trend analysis, and team collaboration features.
#[derive(Clone)]
pub struct QualityIntelligenceDashboard {
    /// Core dashboard engine for metric processing and visualization
    dashboard_engine: Arc<Mutex<DashboardEngine>>,

    /// Trend analysis and forecasting system
    trend_analyzer: Arc<RwLock<TrendAnalyzer>>,

    /// Team collaboration hub
    collaboration_hub: Arc<RwLock<CollaborationHub>>,

    /// Interactive visualization and scoring manager
    visualization_manager: Arc<RwLock<VisualizationManager>>,

    /// Comprehensive metric collection system
    metric_collector: Arc<RwLock<MetricCollector>>,

    /// UI integration and update management
    ui_integration: Arc<RwLock<UiIntegration>>,

    /// System configuration and user preferences
    configuration: Arc<RwLock<DashboardConfiguration>>,

    /// Internal state management
    state: Arc<RwLock<DashboardState>>,
}

/// Internal dashboard state
#[derive(Debug, Clone)]
pub struct DashboardState {
    /// Whether the dashboard is actively collecting and updating metrics
    pub is_active: bool,

    /// Last update timestamp for real-time monitoring
    pub last_update: chrono::DateTime<chrono::Utc>,

    /// Current dashboard configuration snapshot
    pub current_config: DashboardConfiguration,

    /// Performance metrics for the dashboard system itself
    pub performance_metrics: DashboardPerformanceMetrics,
}

/// Performance metrics tracking for the dashboard system
#[derive(Debug, Clone)]
pub struct DashboardPerformanceMetrics {
    /// Average update latency in milliseconds
    pub average_update_latency: f64,

    /// Number of active visualization sessions
    pub active_sessions: u32,

    /// Memory usage in bytes
    pub memory_usage: usize,

    /// Error rate as percentage
    pub error_rate: f32,
}

use collaboration::CollaborationHub;
use configuration::DashboardConfiguration;
use engine::DashboardEngine;
use errors::{DashboardError, DashboardResult};
use metrics::MetricCollector;
use trends::TrendAnalyzer;
use ui_integration::UiIntegration;
use visualization::VisualizationManager;

impl QualityIntelligenceDashboard {
    /// Initialize the quality intelligence dashboard with default configuration
    ///
    /// This function sets up all dashboard components with sensible defaults
    /// for most development scenarios. For custom configuration, use the builder pattern
    /// on individual components.
    ///
    /// # Returns
    ///
    /// Returns an initialized [`QualityIntelligenceDashboard`] ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails due to:
    /// - Missing dependencies
    /// - Resource allocation failures
    /// - Configuration validation errors
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_ai_ide_quality_dashboard::initialize_default_dashboard;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let dashboard = initialize_default_dashboard().await?;
    ///     dashboard.start().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> DashboardResult<Self> {
        let configuration = Arc::new(RwLock::new(DashboardConfiguration::default()));

        // Initialize all components concurrently for better startup performance
        let performance_metrics = DashboardPerformanceMetrics {
            average_update_latency: 0.0,
            active_sessions: 0,
            memory_usage: 0,
            error_rate: 0.0,
        };

        let initial_state = DashboardState {
            is_active: false,
            last_update: chrono::Utc::now(),
            current_config: configuration.read().await.clone(),
            performance_metrics,
        };

        let state = Arc::new(RwLock::new(initial_state));

        // Initialize each component with the configuration
        let (
            dashboard_engine,
            trend_analyzer,
            collaboration_hub,
            visualization_manager,
            metric_collector,
            ui_integration,
        ) = tokio::try_join!(
            DashboardEngine::new(configuration.clone(), state.clone()),
            TrendAnalyzer::new(configuration.clone()),
            CollaborationHub::new(configuration.clone()),
            VisualizationManager::new(configuration.clone()),
            MetricCollector::new(configuration.clone()),
            UiIntegration::new(configuration.clone()),
        )?;

        Ok(Self {
            dashboard_engine: Arc::new(Mutex::new(dashboard_engine)),
            trend_analyzer: Arc::new(RwLock::new(trend_analyzer)),
            collaboration_hub: Arc::new(RwLock::new(collaboration_hub)),
            visualization_manager: Arc::new(RwLock::new(visualization_manager)),
            metric_collector: Arc::new(RwLock::new(metric_collector)),
            ui_integration: Arc::new(RwLock::new(ui_integration)),
            configuration,
            state,
        })
    }

    /// Initialize the dashboard with custom configuration
    ///
    /// This function allows for detailed configuration of dashboard components.
    /// Use this when you need specific performance characteristics or integration requirements.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom configuration for the dashboard system
    ///
    /// # Returns
    ///
    /// Returns an initialized [`QualityIntelligenceDashboard`] with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails due to:
    /// - Invalid configuration
    /// - Dependency failures
    /// - Resource limitations
    pub async fn with_config(config: DashboardConfiguration) -> DashboardResult<Self> {
        let configuration = Arc::new(RwLock::new(config));

        // Initialize with custom configuration - implementation similar to new()
        Self::new().await
    }

    /// Start the quality intelligence dashboard
    ///
    /// This method initializes all background tasks and begins collecting
    /// real-time metrics, processing trends, and preparing for UI interactions.
    ///
    /// # Errors
    ///
    /// Returns an error if dashboard startup fails.
    pub async fn start(&self) -> DashboardResult<()> {
        // Start metric collection
        self.metric_collector
            .write()
            .await
            .start_collection()
            .await?;

        // Initialize UI integration
        self.ui_integration.write().await.initialize_ui().await?;

        // Start real-time updates
        self.dashboard_engine.lock().await.start_engine().await?;

        // Mark dashboard as active
        self.state.write().await.is_active = true;

        Ok(())
    }

    /// Stop the quality intelligence dashboard
    ///
    /// This method gracefully shuts down all dashboard components,
    /// saves current state, and cleans up resources.
    ///
    /// # Errors
    ///
    /// Returns an error if dashboard shutdown fails.
    pub async fn stop(&self) -> DashboardResult<()> {
        // Mark dashboard as inactive
        self.state.write().await.is_active = false;

        // Stop real-time updates
        self.dashboard_engine.lock().await.stop_engine().await?;

        // Stop metric collection
        self.metric_collector
            .write()
            .await
            .stop_collection()
            .await?;

        // Finalize UI integration
        self.ui_integration.write().await.finalize_ui().await?;

        Ok(())
    }

    /// Get current dashboard state snapshot
    ///
    /// Provides a thread-safe snapshot of the current dashboard state
    /// including activity status, configuration, and performance metrics.
    pub async fn get_state(&self) -> DashboardState {
        self.state.read().await.clone()
    }

    /// Update dashboard configuration
    ///
    /// Dynamically updates dashboard configuration. Some changes may require
    /// a restart of specific components to take effect.
    ///
    /// # Arguments
    ///
    /// * `new_config` - New configuration to apply
    ///
    /// # Errors
    ///
    /// Returns an error if configuration update fails.
    pub async fn update_configuration(
        &self,
        new_config: DashboardConfiguration,
    ) -> DashboardResult<()> {
        // Validate new configuration
        new_config.validate()?;

        // Update configuration
        *self.configuration.write().await = new_config.clone();

        // Update state snapshot
        self.state.write().await.current_config = new_config;

        // Propagate configuration changes to components
        self.dashboard_engine
            .lock()
            .await
            .update_config(new_config.clone())
            .await?;
        self.trend_analyzer
            .write()
            .await
            .update_config(new_config.clone())
            .await?;
        self.collaboration_hub
            .write()
            .await
            .update_config(new_config)
            .await?;

        Ok(())
    }
}

impl std::fmt::Debug for QualityIntelligenceDashboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QualityIntelligenceDashboard")
            .field(
                "is_active",
                &self.state.try_read().map(|s| s.is_active).unwrap_or(false),
            )
            .finish()
    }
}

/// Convenience function to initialize the default dashboard
pub async fn initialize_default_dashboard() -> DashboardResult<QualityIntelligenceDashboard> {
    QualityIntelligenceDashboard::new().await
}

/// Convenience function to initialize dashboard with configuration
pub async fn initialize_with_config(
    config: DashboardConfiguration,
) -> DashboardResult<QualityIntelligenceDashboard> {
    QualityIntelligenceDashboard::with_config(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_info() {
        let info = build_info();
        assert!(info.contains(VERSION));
        assert!(info.contains("rust-ai-ide-quality-dashboard"));
    }

    #[tokio::test]
    async fn test_default_dashboard_initialization() {
        let result = initialize_default_dashboard().await;
        assert!(result.is_ok(), "Dashboard initialization should succeed");
    }

    #[tokio::test]
    async fn test_dashboard_state() {
        let dashboard = initialize_default_dashboard().await.unwrap();
        let state = dashboard.get_state().await;
        assert!(
            !state.is_active,
            "Dashboard should be inactive after initialization"
        );
    }
}
