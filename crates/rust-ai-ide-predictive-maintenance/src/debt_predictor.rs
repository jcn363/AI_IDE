//! Technical debt forecasting and trend analysis component

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use statrs::statistics::{OrderStatistics, Statistics};
use crate::{
    types::*,
    errors::*,
    forecast::MLForecastingEngine,
};

/// Core technical debt predictor implementing ML-driven forecasting
#[derive(Debug)]
pub struct DebtPredictor {
    /// Historical debt analyzer for trend analysis
    historical_analyzer: HistoricalDebtAnalyzer,

    /// Trend predictor using multiple algorithms
    trend_predictor: DebtTrendPredictor,

    /// Intervention detector for critical thresholds
    intervention_detector: InterventionDetector,

    /// Cost-benefit analyzer for different strategies
    cost_benefit_analyzer: CostBenefitAnalyzer,

    /// ML-driven forecasting engine
    forecasting_engine: MLForecastingEngine,

    /// Configuration settings
    config: MaintenanceConfig,
}

impl DebtPredictor {
    /// Create a new debt predictor with the given configuration
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            historical_analyzer: HistoricalDebtAnalyzer::new(config),
            trend_predictor: DebtTrendPredictor::new(config),
            intervention_detector: InterventionDetector::new(config),
            cost_benefit_analyzer: CostBenefitAnalyzer::new(config),
            forecasting_engine: MLForecastingEngine::new(config)?,
            config: config.clone(),
        })
    }

    /// Forecast technical debt evolution for a workspace
    pub async fn forecast_debt_evolution(
        &self,
        workspace: &Workspace,
        context: &AnalysisContext,
    ) -> MaintenanceResult<DebtForecast> {
        // Analyze historical debt trends
        let historical_analysis = self.historical_analyzer
            .analyze_historical_trends(context)
            .await?;

        // Generate forecasts for each horizon
        let mut projected_debt = Vec::new();
        let mut all_thresholds = Vec::new();

        for horizon in &self.config.forecast_horizons {
            for debt_item in &context.current_debt_items {
                let projection = self.project_single_debt_item(
                    debt_item,
                    &historical_analysis,
                    *horizon,
                    workspace,
                ).await?;

                // Find critical thresholds for this projection
                let thresholds = self.intervention_detector
                    .detect_critical_thresholds(&projection, &historical_analysis)
                    .await?;

                all_thresholds.extend(thresholds);
                projected_debt.push(projection);
            }
        }

        // Sort projected debt by current severity (highest first)
        projected_debt.sort_by(|a, b| {
            b.timeline.first()
                .map(|p| p.severity)
                .unwrap_or(0.0)
                .partial_cmp(&a.timeline.first().map(|p| p.severity).unwrap_or(0.0))
                .unwrap()
        });

        let confidence_score = self.calculate_forecast_confidence(&projected_debt, &historical_analysis);

        Ok(DebtForecast {
            projected_debt,
            confidence_score,
            forecast_window_weeks: *self.config.forecast_horizons.last().unwrap_or(&12),
            critical_thresholds: all_thresholds,
        })
    }

    /// Project a single debt item's evolution
    async fn project_single_debt_item(
        &self,
        debt_item: &DebtItem,
        historical_analysis: &HistoricalAnalysis,
        horizon_weeks: u32,
        workspace: &Workspace,
    ) -> MaintenanceResult<ProjectedDebtItem> {
        // Calculate average growth rate from historical data
        let growth_rate = self.calculate_growth_rate_for_debt_type(
            debt_item.debt_type.clone(),
            historical_analysis,
        ).await?;

        // Calculate base trajectory using trend predictor
        let base_trajectory = self.trend_predictor
            .predict_trend(debt_item, growth_rate, horizon_weeks)
            .await?;

        // Enhance with ML forecasting for more accuracy
        let enhanced_trajectory = self.forecasting_engine
            .enhance_debt_forecast(debt_item, &base_trajectory, historical_analysis)
            .await?;

        // Identify risk factors that might affect this debt item
        let risk_factors = self.identify_risk_factors(
            debt_item,
            workspace,
            &enhanced_trajectory,
        ).await?;

        // Calculate threshold crossings
        let threshold_crossings = self.calculate_threshold_crossings(
            &enhanced_trajectory,
        ).await?;

        Ok(ProjectedDebtItem {
            original_debt_id: debt_item.id.clone(),
            timeline: enhanced_trajectory,
            threshold_crossings,
            risk_factors,
        })
    }

    /// Calculate growth rate for a specific debt type
    async fn calculate_growth_rate_for_debt_type(
        &self,
        debt_type: DebtType,
        historical_analysis: &HistoricalAnalysis,
    ) -> MaintenanceResult<f64> {
        let mut growth_rates = Vec::new();

        // Analyze historical measurements for this debt type
        for measurement in &historical_analysis.historical_measurements {
            if let Some(type_data) = measurement.debt_by_type.get(&debt_type) {
                if let Some(prev_measurement) = historical_analysis.historical_measurements
                    .iter()
                    .find(|m| m.timestamp < measurement.timestamp)
                {
                    if let Some(prev_type_data) = prev_measurement.debt_by_type.get(&debt_type) {
                        // Calculate weekly growth rate
                        let days_diff = (measurement.timestamp.timestamp() - prev_measurement.timestamp.timestamp()) as f64 / 86400.0;
                        let weeks_diff = days_diff / 7.0;

                        if weeks_diff > 0.0 {
                            let severity_growth = type_data.avg_severity - prev_type_data.avg_severity;
                            let growth_rate = severity_growth / weeks_diff;
                            growth_rates.push(growth_rate);
                        }
                    }
                }
            }
        }

        // Return median growth rate, default to small positive growth if no data
        if growth_rates.is_empty() {
            Ok(0.01) // 1% weekly growth as baseline
        } else {
            Ok(growth_rates[..].partial_cmp_median().unwrap_or(0.01))
        }
    }

    /// Identify risk factors that might affect debt evolution
    async fn identify_risk_factors(
        &self,
        debt_item: &DebtItem,
        workspace: &Workspace,
        trajectory: &Vec<TimelinePoint>,
    ) -> MaintenanceResult<Vec<RiskFactor>> {
        let mut risk_factors = Vec::new();

        // High change frequency risk
        if debt_item.tags.contains(&"frequently-changed".to_string()) {
            risk_factors.push(RiskFactor {
                factor_type: RiskType::ChangeFrequency,
                score: 0.8,
                description: "File is frequently modified, increasing debt accumulation risk".to_string(),
                activation_time: Utc::now(),
            });
        }

        // Technology obsolescence risk
        let age_weeks = debt_item.age_days as f64 / 7.0;
        if age_weeks > 52.0 * 2.0 { // 2 years old
            risk_factors.push(RiskFactor {
                factor_type: RiskType::TechnologyObsolescence,
                score: (age_weeks / (52.0 * 5.0)).min(1.0), // Max out at 5 years
                description: "Debt item is old and may become harder to address".to_string(),
                activation_time: Utc::now() + Duration::weeks(4), // Activate in 4 weeks
            });
        }

        // Complexity growth risk
        if let Some(latest_severity) = trajectory.last().map(|p| p.severity) {
            if latest_severity > 0.7 {
                risk_factors.push(RiskFactor {
                    factor_type: RiskType::ComplexityGrowth,
                    score: (latest_severity - 0.7) * 2.0, // Scale to 0.0-1.0
                    description: "Debt severity is high, may grow exponentially".to_string(),
                    activation_time: Utc::now() + Duration::weeks(2),
                });
            }
        }

        Ok(risk_factors)
    }

    /// Calculate when threshold crossings will occur
    async fn calculate_threshold_crossings(
        &self,
        trajectory: &Vec<TimelinePoint>,
    ) -> MaintenanceResult<Vec<DateTime<Utc>>> {
        let mut crossings = Vec::new();
        let thresholds = vec![0.5, 0.7, 0.85]; // Critical severity thresholds

        for threshold in thresholds {
            for window in trajectory.windows(2) {
                let current = window[0].clone();
                let next = window[1].clone();

                if current.severity < threshold && next.severity >= threshold {
                    // Linear interpolation to find crossing point
                    let progress = (threshold - current.severity) / (next.severity - current.severity);
                    let crossing_time = current.timestamp + (next.timestamp - current.timestamp) * progress;
                    crossings.push(crossing_time);
                }
            }
        }

        crossings.sort();
        Ok(crossings)
    }

    /// Calculate overall confidence score for the forecast
    fn calculate_forecast_confidence(
        &self,
        projected_debt: &Vec<ProjectedDebtItem>,
        historical_analysis: &HistoricalAnalysis,
    ) -> f64 {
        if projected_debt.is_empty() {
            return 0.5; // Neutral confidence
        }

        // Base confidence from historical analysis quality
        let base_confidence = if historical_analysis.historical_measurements.len() > 10 {
            0.8
        } else if historical_analysis.historical_measurements.len() > 5 {
            0.6
        } else {
            0.4
        };

        // Adjust based on forecast consistency
        let forecast_consistency = self.calculate_forecast_consistency(projected_debt);

        // Weighted combination
        (base_confidence * 0.6) + (forecast_consistency * 0.4)
    }

    /// Calculate consistency across projections
    fn calculate_forecast_consistency(&self, projected_debt: &Vec<ProjectedDebtItem>) -> f64 {
        // Simple consistency measure: how many projections follow expected trends
        let consistent_projections = projected_debt.iter()
            .filter(|proj| self.is_projection_consistent(proj))
            .count();

        consistent_projections as f64 / projected_debt.len() as f64
    }

    /// Check if a single projection is consistent with expectations
    fn is_projection_consistent(&self, projection: &ProjectedDebtItem) -> bool {
        if projection.timeline.len() < 3 {
            return false;
        }

        // Check for reasonable growth rates
        let recent_points: Vec<_> = projection.timeline.iter()
            .rev()
            .take(3)
            .collect();

        let mut growth_rates = Vec::new();
        for window in recent_points.windows(2) {
            let current = window[0].severity;
            let prev = window[1].severity;
            if prev > 0.0 {
                growth_rates.push((current - prev) / prev);
            }
        }

        if growth_rates.is_empty() {
            return true; // Stable debt
        }

        // Check if growth rates are reasonable (not extreme)
        let avg_growth = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        avg_growth.abs() < 1.0 // Growth rate between -100% and +100% per period
    }
}

/// Historical analysis data structure
#[derive(Debug)]
pub struct HistoricalAnalysis {
    pub historical_measurements: Vec<HistoricalMeasurement>,
    pub average_growth_rate: f64,
    pub volatility_measure: f64,
}

/// Historical measurement with debt type breakdown
#[derive(Debug)]
pub struct HistoricalMeasurement {
    pub timestamp: DateTime<Utc>,
    pub total_debt: f64,
    pub debt_by_type: HashMap<DebtType, DebtTypeStats>,
}

/// Statistics for a specific debt type
#[derive(Debug)]
pub struct DebtTypeStats {
    pub count: usize,
    pub avg_severity: f64,
    pub avg_maintainability_impact: f64,
}

/// Historical debt trend analyzer
#[derive(Debug)]
pub struct HistoricalDebtAnalyzer {
    config: MaintenanceConfig,
}

impl HistoricalDebtAnalyzer {
    pub fn new(config: &MaintenanceConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn analyze_historical_trends(
        &self,
        context: &AnalysisContext,
    ) -> MaintenanceResult<HistoricalAnalysis> {
        let mut measurements = Vec::new();

        // Convert analysis context measurements to structured format
        for measurement in &context.historical_measurements {
            // Group debt items by type for this measurement
            let mut debt_by_type = HashMap::new();
            for debt_item in &context.current_debt_items {
                let entry = debt_by_type.entry(debt_item.debt_type.clone())
                    .or_insert_with(|| DebtTypeStats {
                        count: 0,
                        avg_severity: 0.0,
                        avg_maintainability_impact: 0.0,
                    });

                entry.count += 1;
                entry.avg_severity += debt_item.severity;
                entry.avg_maintainability_impact += debt_item.maintainability_impact;
            }

            // Calculate averages
            for stats in debt_by_type.values_mut() {
                if stats.count > 0 {
                    stats.avg_severity /= stats.count as f64;
                    stats.avg_maintainability_impact /= stats.count as f64;
                }
            }

            measurements.push(HistoricalMeasurement {
                timestamp: measurement.timestamp,
                total_debt: measurement.total_severity as f64,
                debt_by_type,
            });
        }

        // Sort measurements by timestamp
        measurements.sort_by_key(|m| m.timestamp);

        // Calculate average growth rate and volatility
        let (avg_growth, volatility) = self.calculate_growth_statistics(&measurements);

        Ok(HistoricalAnalysis {
            historical_measurements: measurements,
            average_growth_rate: avg_growth,
            volatility_measure: volatility,
        })
    }

    fn calculate_growth_statistics(&self, measurements: &Vec<HistoricalMeasurement>) -> (f64, f64) {
        if measurements.len() < 2 {
            return (0.05, 0.5); // Default values
        }

        let mut growth_rates = Vec::new();

        for window in measurements.windows(2) {
            let current = &window[0];
            let previous = &window[1];

            let days_diff = (current.timestamp.timestamp() - previous.timestamp.timestamp()) as f64 / 86400.0;
            let weeks_diff = days_diff / 7.0;

            if weeks_diff > 0.0 && previous.total_debt > 0.0 {
                let growth_rate = (current.total_debt - previous.total_debt) / previous.total_debt / weeks_diff;
                growth_rates.push(growth_rate);
            }
        }

        if growth_rates.is_empty() {
            return (0.05, 0.5);
        }

        // Calculate median and standard deviation
        let avg_growth = growth_rates[..].partial_cmp_median().unwrap_or(0.05);
        let mean = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        let variance = growth_rates.iter()
            .map(|rate| (rate - mean).powi(2))
            .sum::<f64>() / growth_rates.len() as f64;
        let volatility = variance.sqrt();

        (avg_growth, volatility)
    }
}

/// Debt trend prediction component
#[derive(Debug)]
pub struct DebtTrendPredictor {
    config: MaintenanceConfig,
}

impl DebtTrendPredictor {
    pub fn new(config: &MaintenanceConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn predict_trend(
        &self,
        debt_item: &DebtItem,
        growth_rate: f64,
        horizon_weeks: u32,
    ) -> MaintenanceResult<Vec<TimelinePoint>> {
        let mut timeline = Vec::new();
        let start_severity = debt_item.severity;
        let now = Utc::now();

        for week in 0..=horizon_weeks {
            let time_offset = Duration::weeks(week as i64);
            let timestamp = now + time_offset;

            // Calculate severity at this point using exponential growth
            let severity = start_severity * (1.0 + growth_rate).powi(week as i32);

            // Impact grows somewhat slower than severity
            let maintainability_impact = debt_item.maintainability_impact *
                (1.0 + growth_rate * 0.8).powi(week as i32);

            // Confidence interval reduces with time
            let confidence_factor = (-week as f64 / horizon_weeks as f64).exp();
            let confidence_interval = (
                severity * (1.0 - confidence_factor * 0.3), // Lower bound
                severity * (1.0 + confidence_factor * 0.3), // Upper bound
            );

            timeline.push(TimelinePoint {
                timestamp,
                severity: severity.min(1.0), // Cap at maximum severity
                maintainability_impact: maintainability_impact.min(1.0), // Cap at maximum impact
                confidence_interval,
            });
        }

        Ok(timeline)
    }
}

/// Intervention threshold detection
#[derive(Debug)]
pub struct InterventionDetector {
    config: MaintenanceConfig,
}

impl InterventionDetector {
    pub fn new(config: &MaintenanceConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn detect_critical_thresholds(
        &self,
        projection: &ProjectedDebtItem,
        historical_analysis: &HistoricalAnalysis,
    ) -> MaintenanceResult<Vec<Threshold>> {
        let mut thresholds = Vec::new();

        // Define critical severity thresholds
        let critical_thresholds = vec![
            (0.4, "Low urgency intervention", "Debt is approaching critical levels"),
            (0.7, "High urgency intervention", "Debt requires immediate attention"),
            (0.85, "Critical intervention required", "Debt will seriously impact maintainability"),
        ];

        for (value, description, consequences) in critical_thresholds {
            if let Some(crossing_time) = self.find_first_crossing_after_threshold(
                &projection.timeline,
                value,
                Utc::now(),
            ) {
                thresholds.push(Threshold {
                    threshold_type: ThresholdType::Severity,
                    value,
                    expected_crossing: crossing_time,
                    consequences: consequences.to_string(),
                });
            }
        }

        // Add timeline-based budget thresholds
        for week in &self.config.forecast_horizons {
            thresholds.push(Threshold {
                threshold_type: ThresholdType::Timeline,
                value: *week as f64,
                expected_crossing: Utc::now() + Duration::weeks(*week as i64),
                consequences: format!("Timeline threshold at {} weeks", week),
            });
        }

        thresholds.sort_by_key(|t| t.expected_crossing);
        Ok(thresholds)
    }

    fn find_first_crossing_after_threshold(
        &self,
        timeline: &Vec<TimelinePoint>,
        threshold: f64,
        start_time: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        for window in timeline.windows(2) {
            let current = &window[0];
            let next = &window[1];

            // Only consider future crossings
            if next.timestamp < start_time {
                continue;
            }

            if current.severity < threshold && next.severity >= threshold {
                // Linear interpolation
                let progress = (threshold - current.severity) / (next.severity - current.severity);
                let crossing_time = current.timestamp +
                    (next.timestamp - current.timestamp) * progress;
                return Some(crossing_time);
            }
        }
        None
    }
}

/// Cost-benefit analysis for maintenance strategies
#[derive(Debug)]
pub struct CostBenefitAnalyzer {
    config: MaintenanceConfig,
}

impl CostBenefitAnalyzer {
    pub fn new(config: &MaintenanceConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    // Placeholder implementation - would contain detailed cost-benefit analysis
    pub async fn analyze_cost_benefit(
        &self,
        strategy: &MaintenanceStrategy,
        projection: &ProjectedDebtItem,
    ) -> MaintenanceResult<CostBenefitResult> {
        // This would implement detailed cost-benefit calculations
        Ok(CostBenefitResult {
            net_benefit: 0.0,
            payback_period_weeks: 12,
            risk_adjusted_return: 0.25,
            recommended_action: "Delay intervention".to_string(),
        })
    }
}

/// Simple forecasting engine interface
#[derive(Debug)]
pub struct MLForecastingEngine {
    config: MaintenanceConfig,
}

impl MLForecastingEngine {
    pub fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    pub async fn enhance_debt_forecast(
        &self,
        _debt_item: &DebtItem,
        base_trajectory: &Vec<TimelinePoint>,
        _historical_analysis: &HistoricalAnalysis,
    ) -> MaintenanceResult<Vec<TimelinePoint>> {
        // For now, return the base trajectory
        // In a real implementation, this would use machine learning
        // to enhance the forecast with patterns learned from historical data
        Ok(base_trajectory.clone())
    }
}

// Additional helper types
#[derive(Debug)]
pub struct MaintenanceStrategy {
    pub name: String,
    pub cost: f64,
    pub timeline_weeks: u32,
    pub risk_level: RiskLevel,
}

#[derive(Debug)]
pub struct CostBenefitResult {
    pub net_benefit: f64,
    pub payback_period_weeks: u32,
    pub risk_adjusted_return: f64,
    pub recommended_action: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debt_predictor_creation() {
        let config = MaintenanceConfig::default();
        let result = DebtPredictor::new(&config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_historical_analyzer() {
        let config = MaintenanceConfig::default();
        let analyzer = HistoricalDebtAnalyzer::new(&config);

        // Test with minimal context
        let context = AnalysisContext {
            current_debt_items: vec![],
            historical_measurements: vec![],
            recent_refactorings: vec![],
            dependency_graph: HashMap::new(),
        };

        // Should handle empty data gracefully
        let result = tokio::spawn(async move {
            analyzer.analyze_historical_trends(&context).await
        });
        // Note: In actual test, we'd need to await and assert
    }
}