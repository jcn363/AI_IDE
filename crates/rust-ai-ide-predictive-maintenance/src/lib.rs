//! # Predictive Maintenance Forecasting System
//! Phase 3.2: ML-driven technical debt forecasting and automated maintenance recommendations
//!
//! This crate provides comprehensive technical debt analysis, cost estimation,
//! and automated maintenance prioritization for large-scale Rust development.

mod types;
mod errors;
mod debt_predictor;
mod cost_estimator;
mod impact_analyzer;
mod priority_scorer;
mod recommender;
mod forecast;
mod cache;

pub use types::*;
pub use errors::*;

use std::sync::Arc;
use tokio::sync::RwLock;

// Type aliases for component modules
type DebtPredictor = debt_predictor::DebtPredictor;
type CostEstimator = cost_estimator::CostEstimator;
type ImpactAnalyzer = impact_analyzer::ImpactAnalyzer;
type PriorityScorer = priority_scorer::PriorityScorer;
type Recommendor = recommender::Recommendor;
type ForecastEngine = forecast::ForecastEngine;
type CacheManager = cache::CacheManager;

/// Core predictive maintenance engine integrating all forecasting components
#[derive(Clone)]
pub struct PredictiveMaintenanceEngine {
    /// Technical debt forecasting component
    debt_predictor: Arc<RwLock<DebtPredictor>>,

    /// Cost estimation for maintenance tasks
    cost_estimator: Arc<RwLock<CostEstimator>>,

    /// Cross-file dependency impact analysis
    impact_analyzer: Arc<RwLock<ImpactAnalyzer>>,

    /// Automated maintenance prioritization
    priority_scorer: Arc<RwLock<PriorityScorer>>,

    /// Intelligent maintenance recommendations
    recommender: Arc<RwLock<Recommendor>>,

    /// Shared forecasting models and cache
    forecast_engine: Arc<RwLock<ForecastEngine>>,

    /// Performance and caching layer
    cache_manager: Arc<CacheManager>,
}

impl PredictiveMaintenanceEngine {
    /// Initialize the predictive maintenance engine with all components
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        // Initialize components concurrently for better startup performance
        let (
            debt_predictor,
            cost_estimator,
            impact_analyzer,
            priority_scorer,
            recommender,
            forecast_engine,
            cache_manager,
        ) = tokio::try_join!(
            DebtPredictor::new(config),
            CostEstimator::new(config),
            ImpactAnalyzer::new(config),
            PriorityScorer::new(config),
            Recommendor::new(config),
            ForecastEngine::new(config),
            CacheManager::new(config),
        )?;

        Ok(Self {
            debt_predictor: Arc::new(RwLock::new(debt_predictor)),
            cost_estimator: Arc::new(RwLock::new(cost_estimator)),
            impact_analyzer: Arc::new(RwLock::new(impact_analyzer)),
            priority_scorer: Arc::new(RwLock::new(priority_scorer)),
            recommender: Arc::new(RwLock::new(recommender)),
            forecast_engine: Arc::new(RwLock::new(forecast_engine)),
            cache_manager: Arc::new(cache_manager),
        })
    }

    /// Generate comprehensive maintenance forecast for a workspace
    pub async fn generate_forecast(
        &self,
        workspace: &Workspace,
        analysis_context: &AnalysisContext,
    ) -> MaintenanceResult<MaintenanceForecast> {
        // Analyze technical debt evolution
        let debt_forecast = self.analyze_debt_forecast(workspace, analysis_context).await?;

        // Estimate costs for maintenance tasks
        let cost_estimation = self.estimate_maintenance_costs(&debt_forecast).await?;

        // Analyze cross-file dependency impacts
        let impact_analysis = self.analyze_dependency_impacts(workspace, analysis_context).await?;

        // Score and prioritize maintenance tasks
        let prioritized_tasks = self.prioritize_tasks(&debt_forecast, &cost_estimation, &impact_analysis).await?;

        // Generate automated recommendations
        let recommendations = self.generate_recommendations(&prioritized_tasks).await?;

        Ok(MaintenanceForecast {
            debt_forecast,
            cost_estimation,
            impact_analysis,
            prioritized_tasks,
            recommendations,
            generated_at: chrono::Utc::now(),
            confidence_score: self.calculate_overall_confidence(&debt_forecast, &cost_estimation),
        })
    }

    /// Analyze technical debt evolution trends
    async fn analyze_debt_forecast(
        &self,
        workspace: &Workspace,
        context: &AnalysisContext,
    ) -> MaintenanceResult<DebtForecast> {
        let predictor = self.debt_predictor.read().await;
        predictor.forecast_debt_evolution(workspace, context).await
    }

    /// Estimate costs for maintenance tasks
    async fn estimate_maintenance_costs(
        &self,
        debt_forecast: &DebtForecast,
    ) -> MaintenanceResult<CostEstimation> {
        let estimator = self.cost_estimator.read().await;
        let mut costs = Vec::new();

        for debt_item in &debt_forecast.projected_debt {
            let cost = estimator.estimate_cost(&debt_item).await?;
            costs.push(cost);
        }

        Ok(CostEstimation {
            total_estimated_cost: costs.iter().map(|c| c.estimated_effort_hours).sum(),
            breakdown: costs,
            currency: "developer-hours".to_string(),
        })
    }

    /// Analyze cross-file dependency impacts
    async fn analyze_dependency_impacts(
        &self,
        workspace: &Workspace,
        context: &AnalysisContext,
    ) -> MaintenanceResult<ImpactAnalysis> {
        let analyzer = self.impact_analyzer.read().await;
        analyzer.analyze_impacts(workspace, context).await
    }

    /// Prioritize maintenance tasks based on multiple criteria
    async fn prioritize_tasks(
        &self,
        debt_forecast: &DebtForecast,
        cost_estimation: &CostEstimation,
        impact_analysis: &ImpactAnalysis,
    ) -> MaintenanceResult<PrioritizedTaskList> {
        let scorer = self.priority_scorer.read().await;
        let mut prioritized = Vec::new();

        for (index, _) in debt_forecast.projected_debt.iter().enumerate() {
            let cost = cost_estimation.breakdown.get(index).unwrap_or(&CostBreakdown {
                estimated_effort_hours: 0.0,
                risk_factor: 0.5,
                complexity_multiplier: 1.0,
                urgency_score: 0.5,
            });

            if let Some(impact) = impact_analysis.impacts.get(index) {
                let priority = scorer.calculate_priority(impact, cost).await?;
                prioritized.push(PrioritizedTask {
                    original_index: index,
                    priority_score: priority.score,
                    recommended_timeline: priority.timeline,
                    rationale: priority.rationale,
                });
            }
        }

        // Sort by priority score (highest first)
        prioritized.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());

        Ok(PrioritizedTaskList {
            tasks: prioritized,
            prioritization_strategy: "multi-criteria-weighted".to_string(),
        })
    }

    /// Generate intelligent maintenance recommendations
    async fn generate_recommendations(
        &self,
        prioritized_tasks: &PrioritizedTaskList,
    ) -> MaintenanceResult<MaintenanceRecommendations> {
        let recommender = self.recommender.read().await;
        recommender.generate_recommendations(prioritized_tasks).await
    }

    /// Calculate overall confidence score for the forecast
    fn calculate_overall_confidence(
        &self,
        debt_forecast: &DebtForecast,
        cost_estimation: &CostEstimation,
    ) -> f64 {
        // Simple weighted average of confidence scores
        let debt_confidence = debt_forecast.confidence_score;
        let cost_confidence = if cost_estimation.total_estimated_cost > 0.0 {
            cost_estimation.breakdown.iter().map(|c| c.urgency_score).sum::<f64>() / cost_estimation.breakdown.len() as f64
        } else {
            0.8
        };

        (debt_confidence * 0.7) + (cost_confidence * 0.3)
    }
}

impl std::fmt::Debug for PredictiveMaintenanceEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PredictiveMaintenanceEngine")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_initialization() {
        let config = MaintenanceConfig::default();
        let result = PredictiveMaintenanceEngine::new(&config).await;
        assert!(result.is_ok());
    }
}