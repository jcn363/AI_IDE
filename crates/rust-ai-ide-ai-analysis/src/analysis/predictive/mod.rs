// Module declarations
pub mod health;
pub mod metrics;
pub mod performance;
pub mod recommendations;
pub mod vulnerability;

// Core types for predictive analysis

/// Predictive configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PredictiveConfig {
    /// Enable vulnerability prediction
    pub enable_vulnerability_prediction: bool,
    /// Enable performance forecasting
    pub enable_performance_forecasting: bool,
    /// Enable code health scoring
    pub enable_health_scoring: bool,
    /// Enable automated recommendations
    pub enable_recommendations: bool,
    /// Prediction confidence threshold (0.0-1.0)
    pub confidence_threshold: f32,
    /// Historical data window for trend analysis
    pub historical_window_days: u32,
}

impl Default for PredictiveConfig {
    fn default() -> Self {
        Self {
            enable_vulnerability_prediction: true,
            enable_performance_forecasting: true,
            enable_health_scoring: true,
            enable_recommendations: true,
            confidence_threshold: 0.7,
            historical_window_days: 30,
        }
    }
}

// Re-export submodules
pub use health::*;
pub use performance::*;
pub use recommendations::*;
pub use vulnerability::*;

/// Predictive Quality Intelligence Engine
///
/// This is the main orchestrator for Phase 3 predictive analysis capabilities.
/// It integrates vulnerability prediction, performance forecasting, code health
/// scoring, and automated maintenance recommendations into a unified system.
#[derive(Debug)]
pub struct PredictiveQualityEngine {
    config: PredictiveConfig,
    vulnerability_predictor: VulnerabilityPredictor,
    performance_forecaster: PerformanceForecaster,
    health_scorer: HealthScorer,
    recommendation_engine: RecommendationEngine,
}

impl PredictiveQualityEngine {
    /// Create a new predictive quality engine with the given configuration
    pub fn new(config: PredictiveConfig) -> Self {
        Self {
            vulnerability_predictor: VulnerabilityPredictor::new(),
            performance_forecaster: PerformanceForecaster::new(),
            health_scorer: HealthScorer::new(),
            recommendation_engine: RecommendationEngine::new(),
            config,
        }
    }

    /// Perform comprehensive predictive analysis on a codebase
    pub async fn analyze_project(
        &self,
        project_path: &str,
        historical_data: Option<&HistoricalData>,
    ) -> Result<PredictiveAnalysisReport, PredictiveError> {
        let mut vulnerabilities = Vec::new();
        let mut performance_bottlenecks = Vec::new();
        let mut health_scores = Vec::new();
        let mut recommendations = Vec::new();

        // Vulnerability prediction
        if self.config.enable_vulnerability_prediction {
            vulnerabilities = self
                .vulnerability_predictor
                .predict_vulnerabilities(project_path, historical_data)
                .await
                .map_err(|e| {
                    PredictiveError::AnalysisFailed(format!(
                        "Vulnerability prediction failed: {}",
                        e
                    ))
                })?;
        }

        // Performance forecasting
        if self.config.enable_performance_forecasting {
            performance_bottlenecks = self
                .performance_forecaster
                .forecast_bottlenecks(project_path, historical_data)
                .await
                .map_err(|e| {
                    PredictiveError::AnalysisFailed(format!(
                        "Performance forecasting failed: {}",
                        e
                    ))
                })?;
        }

        // Code health scoring
        if self.config.enable_health_scoring {
            health_scores = self
                .health_scorer
                .score_project_health(project_path)
                .await
                .map_err(|e| {
                    PredictiveError::AnalysisFailed(format!("Health scoring failed: {}", e))
                })?;
        }

        // Generate recommendations
        if self.config.enable_recommendations {
            recommendations = self
                .recommendation_engine
                .generate_recommendations(
                    &vulnerabilities,
                    &performance_bottlenecks,
                    &health_scores,
                )
                .await?;
        }

        Ok(PredictiveAnalysisReport {
            vulnerabilities,
            performance_bottlenecks,
            health_scores,
            recommendations,
            confidence: self.config.confidence_threshold,
            generated_at: chrono::Utc::now(),
        })
    }
}

/// Predictive analysis report
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PredictiveAnalysisReport {
    /// Predicted vulnerabilities
    pub vulnerabilities: Vec<PredictedVulnerability>,
    /// Performance bottlenecks forecast
    pub performance_bottlenecks: Vec<PerformanceBottleneckForecast>,
    /// Code health scores
    pub health_scores: Vec<HealthScore>,
    /// Maintenance recommendations
    pub recommendations: Vec<MaintenanceRecommendation>,
    /// Overall confidence threshold used
    pub confidence: f32,
    /// When the report was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Historical data for trend analysis
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HistoricalData {
    /// Previous analysis reports
    pub reports: Vec<AnalysisReport>,
    /// Commit history data
    pub commit_history: Vec<CommitData>,
    /// Code metrics over time
    pub metrics_history: Vec<MetricsSnapshot>,
}

/// Simplified analysis report for historical data
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AnalysisReport {
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Vulnerabilities found
    pub vulnerabilities_found: u32,
    /// Performance issues detected
    pub performance_issues: u32,
}

/// Commit data for trend analysis
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CommitData {
    /// Commit timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Files changed
    pub files_changed: Vec<String>,
    /// Lines added
    pub lines_added: u32,
    /// Lines deleted
    pub lines_deleted: u32,
}

/// Metrics snapshot for trend analysis
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MetricsSnapshot {
    /// Snapshot timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Cyclomatic complexity average
    pub avg_cyclomatic_complexity: f64,
    /// Maintainability index
    pub maintainability_index: f64,
    /// Total lines of code
    pub total_loc: usize,
}

/// Predictive analysis errors
#[derive(Debug, thiserror::Error)]
#[error("Predictive analysis error")]
pub enum PredictiveError {
    #[error("ML model not available: {model}")]
    ModelNotAvailable { model: String },

    #[error("Insufficient historical data for prediction")]
    InsufficientHistoricalData,

    #[error("Prediction confidence too low")]
    LowConfidence,

    #[error("IO error during analysis: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),
}
