//! Maintenance cost estimation component for predictive forecasting

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Duration;

use crate::errors::*;
use crate::types::*;

/// Core maintenance cost estimator
#[derive(Debug)]
pub struct CostEstimator {
    /// Complexity-to-cost mapper for estimating effort
    complexity_mapper: ComplexityToCostMapper,

    /// Effort estimation engine
    effort_estimator: EffortEstimator,

    /// Resource allocation forecaster
    resource_forecaster: ResourceForecaster,

    /// Debt risk assessment component
    risk_assessor: DebtRiskAssessor,

    /// Cost optimization engine
    cost_optimizer: CostOptimizationEngine,

    /// Configuration settings
    config: MaintenanceConfig,
}

impl CostEstimator {
    /// Create a new cost estimator with the given configuration
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            complexity_mapper:   ComplexityToCostMapper::new(config).await?,
            effort_estimator:    EffortEstimator::new(config).await?,
            resource_forecaster: ResourceForecaster::new(config).await?,
            risk_assessor:       DebtRiskAssessor::new(config).await?,
            cost_optimizer:      CostOptimizationEngine::new(config).await?,
            config:              config.clone(),
        })
    }

    /// Estimate cost for a single debt item (main interface)
    pub async fn estimate_cost(&self, debt_item: &DebtItem) -> MaintenanceResult<CostBreakdown> {
        // Calculate base effort estimate
        let base_effort = self
            .complexity_mapper
            .map_complexity_to_effort(debt_item)
            .await?;

        // Apply risk factors
        let risk_factor = self.risk_assessor.assess_risk(debt_item).await?;

        // Calculate complexity multiplier
        let complexity_multiplier = self.calculate_complexity_multiplier(debt_item);

        // Calculate adjusted effort
        let adjusted_effort = base_effort * risk_factor * complexity_multiplier;

        // Calculate urgency score (0.0-1.0)
        let urgency_score = self.calculate_urgency_score(debt_item, adjusted_effort);

        // Generate detailed cost components
        let components = self
            .generate_cost_components(
                base_effort,
                adjusted_effort,
                risk_factor,
                complexity_multiplier,
                debt_item,
            )
            .await?;

        Ok(CostBreakdown {
            estimated_effort_hours: adjusted_effort,
            risk_factor,
            complexity_multiplier,
            urgency_score,
            components,
        })
    }

    /// Calculate complexity multiplier based on debt characteristics
    fn calculate_complexity_multiplier(&self, debt_item: &DebtItem) -> f64 {
        let mut multiplier = 1.0;

        // Adjust based on age
        let age_weeks = debt_item.age_days as f64 / 7.0;
        if age_weeks > 52.0 {
            multiplier *= 1.3; // 30% increase for old debt
        } else if age_weeks > 26.0 {
            multiplier *= 1.15; // 15% increase for somewhat old debt
        }

        // Adjust based on debt type
        multiplier *= match debt_item.debt_type {
            DebtType::Security => 1.5,      // Security issues are more complex
            DebtType::Architecture => 1.4,  // Architecture changes are complex
            DebtType::Performance => 1.3,   // Performance optimizations can be tricky
            DebtType::Test => 1.2,          // Test debt often has ripple effects
            DebtType::Documentation => 0.8, // Documentation is usually easier
            _ => 1.0,
        };

        // Adjust based on file location (common patterns)
        if debt_item.file_path.contains("src/main.rs") || debt_item.file_path.contains("lib.rs") {
            multiplier *= 1.25; // Core files are more risky to modify
        }

        // Adjust for frequently modified files
        if debt_item.tags.contains(&"frequently-modified".to_string())
            || debt_item.tags.contains(&"hot-spot".to_string())
        {
            multiplier *= 1.1; // Small increase for active files
        }

        // Cap the multiplier at 3.0 to avoid extreme estimates
        multiplier.min(3.0).max(0.5)
    }

    /// Calculate urgency score based on various factors
    fn calculate_urgency_score(&self, debt_item: &DebtItem, estimated_effort: f64) -> f64 {
        // Base score from severity
        let mut score = debt_item.severity;

        // Boost score based on age (older debt becomes more urgent)
        let age_boost = (debt_item.age_days as f64 / 365.0 * 0.2).min(0.2);
        score += age_boost;

        // Boost score based on maintainability impact
        score += debt_item.maintainability_impact * 0.3;

        // Adjust based on effort (easier tasks can be done sooner)
        let effort_boost = if estimated_effort < 4.0 {
            0.1 // Small tasks can be quick wins
        } else if estimated_effort > 16.0 {
            -0.1 // Large tasks need careful planning
        } else {
            0.0
        };
        score += effort_boost;

        // Ensure score is between 0.0 and 1.0
        score.max(0.0).min(1.0)
    }

    /// Generate detailed cost components breakdown
    async fn generate_cost_components(
        &self,
        base_effort: f64,
        adjusted_effort: f64,
        risk_factor: f64,
        complexity_multiplier: f64,
        debt_item: &DebtItem,
    ) -> MaintenanceResult<Vec<CostComponent>> {
        let mut components = Vec::new();

        // Analysis phase
        components.push(CostComponent {
            component_name:        "Analysis and Planning".to_string(),
            base_effort_hours:     base_effort * 0.3,
            adjusted_effort_hours: adjusted_effort * 0.3,
            justification:         "Understanding the debt and planning the fix".to_string(),
        });

        // Implementation phase
        let implementation_effort = adjusted_effort * 0.5;
        components.push(CostComponent {
            component_name:        "Implementation".to_string(),
            base_effort_hours:     base_effort * 0.5,
            adjusted_effort_hours: implementation_effort,
            justification:         format!(
                "Actual code changes to address the {} debt",
                debt_item.debt_type
            ),
        });

        // Testing phase
        let testing_multiplier = self.calculate_testing_multiplier(debt_item);
        let testing_effort = adjusted_effort * 0.15 * testing_multiplier;
        components.push(CostComponent {
            component_name:        "Testing and Validation".to_string(),
            base_effort_hours:     base_effort * 0.15,
            adjusted_effort_hours: testing_effort,
            justification:         "Ensuring the fix doesn't break existing functionality".to_string(),
        });

        // Documentation phase (if needed)
        if matches!(debt_item.debt_type, DebtType::Documentation)
            || debt_item
                .description
                .to_lowercase()
                .contains("documentation")
        {
            components.push(CostComponent {
                component_name:        "Documentation Updates".to_string(),
                base_effort_hours:     base_effort * 0.05,
                adjusted_effort_hours: adjusted_effort * 0.05,
                justification:         "Updating documentation to reflect changes".to_string(),
            });
        }

        // Risk mitigation (additional time for high-risk changes)
        if risk_factor > 1.5 {
            let risk_mitigation_effort = adjusted_effort * 0.1 * (risk_factor - 1.0);
            components.push(CostComponent {
                component_name:        "Risk Mitigation".to_string(),
                base_effort_hours:     0.0, // Risk mitigation is over base estimate
                adjusted_effort_hours: risk_mitigation_effort,
                justification:         "Additional time for careful handling of high-risk changes".to_string(),
            });
        }

        // Review and integration
        components.push(CostComponent {
            component_name:        "Review and Integration".to_string(),
            base_effort_hours:     base_effort * 0.1,
            adjusted_effort_hours: adjusted_effort * 0.1,
            justification:         "Code review and integration with existing codebase".to_string(),
        });

        Ok(components)
    }

    /// Calculate testing effort multiplier based on debt characteristics
    fn calculate_testing_multiplier(&self, debt_item: &DebtItem) -> f64 {
        let mut multiplier = 1.0;

        // Higher testing for critical debt
        match debt_item.debt_type {
            DebtType::Security | DebtType::Performance => multiplier *= 1.5,
            DebtType::Architecture => multiplier *= 1.3,
            _ => {}
        }

        // Higher testing for high severity
        if debt_item.severity > 0.8 {
            multiplier *= 1.2;
        }

        // Higher testing for core components
        if debt_item.tags.contains(&"core".to_string()) || debt_item.tags.contains(&"critical-path".to_string()) {
            multiplier *= 1.25;
        }

        multiplier
    }
}

/// Complexity-to-cost mapping engine
#[derive(Debug)]
pub struct ComplexityToCostMapper {
    /// Cached complexity mappings for performance
    mappings: HashMap<String, f64>,

    /// Configuration
    config: MaintenanceConfig,
}

impl ComplexityToCostMapper {
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            mappings: Self::load_default_mappings(),
            config:   config.clone(),
        })
    }

    /// Map debt item complexity to estimated effort hours
    pub async fn map_complexity_to_effort(&self, debt_item: &DebtItem) -> MaintenanceResult<f64> {
        // Use debt type as primary mapping key
        let debt_type_key = format!("{:?}", debt_item.debt_type);

        // Get base estimate from mappings or use default
        let base_estimate = self.mappings.get(&debt_type_key).copied().unwrap_or(4.0); // 4 hours default

        // Adjust based on severity
        let severity_multiplier = 1.0 + (debt_item.severity * 0.5);
        let severity_adjusted = base_estimate * severity_multiplier;

        // Adjust based on maintainability impact
        let impact_multiplier = 1.0 + (debt_item.maintainability_impact * 0.3);
        let impact_adjusted = severity_adjusted * impact_multiplier;

        Ok(impact_adjusted)
    }

    /// Load default complexity-to-effort mappings
    fn load_default_mappings() -> HashMap<String, f64> {
        let mut mappings = HashMap::new();

        // Typical effort estimates in hours for different debt types
        mappings.insert("Complexity".to_string(), 8.0);
        mappings.insert("Duplication".to_string(), 4.0);
        mappings.insert("OutdatedDependency".to_string(), 6.0);
        mappings.insert("DeadCode".to_string(), 2.0);
        mappings.insert("Documentation".to_string(), 3.0);
        mappings.insert("Security".to_string(), 12.0);
        mappings.insert("Performance".to_string(), 10.0);
        mappings.insert("Architecture".to_string(), 16.0);
        mappings.insert("Test".to_string(), 6.0);

        mappings
    }
}

/// Effort estimation engine
#[derive(Debug)]
pub struct EffortEstimator {
    config: MaintenanceConfig,
}

impl EffortEstimator {
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    // Placeholder for effort estimation logic
    pub async fn estimate_effort_distribution(&self, debt_items: &[DebtItem]) -> MaintenanceResult<EffortDistribution> {
        let total_effort: f64 = debt_items
            .iter()
            .map(|item| item.estimated_effort_hours)
            .sum();

        let average_effort = if debt_items.is_empty() {
            0.0
        } else {
            total_effort / debt_items.len() as f64
        };

        let max_effort = debt_items
            .iter()
            .map(|item| item.estimated_effort_hours)
            .fold(0.0, |acc, x| acc.max(x));

        let min_effort = debt_items
            .iter()
            .map(|item| item.estimated_effort_hours)
            .fold(
                f64::INFINITY,
                |acc, x| {
                    if x > 0.0 {
                        acc.min(x)
                    } else {
                        acc
                    }
                },
            );

        Ok(EffortDistribution {
            total_effort,
            average_effort,
            max_effort,
            min_effort: if min_effort == f64::INFINITY {
                0.0
            } else {
                min_effort
            },
            task_count: debt_items.len(),
        })
    }
}

/// Resource allocation forecaster
#[derive(Debug)]
pub struct ResourceForecaster {
    config: MaintenanceConfig,
}

impl ResourceForecaster {
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Forecast resource needs for maintenance effort
    pub async fn forecast_resources(
        &self,
        effort_distribution: &EffortDistribution,
        timeline_days: u32,
    ) -> MaintenanceResult<ResourceForecast> {
        let total_person_days = effort_distribution.total_effort / 8.0; // Assuming 8-hour workdays
        let available_person_days = timeline_days as f64 * 1.0; // Assuming 1 FTE

        let capacity_utilization = total_person_days / available_person_days * 100.0;

        Ok(ResourceForecast {
            required_person_days: total_person_days,
            available_person_days,
            capacity_utilization: capacity_utilization.min(100.0),
            estimated_duration_days: (total_person_days / 1.0).ceil() as u32, // Assuming 1 FTE
            bottlenecks_identified: capacity_utilization > 85.0,
        })
    }
}

/// Debt risk assessment engine
#[derive(Debug)]
pub struct DebtRiskAssessor {
    config: MaintenanceConfig,
}

impl DebtRiskAssessor {
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Assess overall risk for a debt item
    pub async fn assess_risk(&self, debt_item: &DebtItem) -> MaintenanceResult<f64> {
        let mut risk_score = 1.0;

        // Risk based on severity
        risk_score *= 1.0 + (debt_item.severity * 0.5);

        // Risk based on age (older debt is riskier to fix)
        let age_weeks = debt_item.age_days as f64 / 7.0;
        risk_score *= 1.0 + (age_weeks / 52.0).min(1.0) * 0.3;

        // Risk based on type
        risk_score *= match debt_item.debt_type {
            DebtType::Security => 2.0,
            DebtType::Architecture => 1.5,
            DebtType::Performance => 1.3,
            DebtType::Test => 1.2,
            DebtType::Complexity => 1.1,
            _ => 1.0,
        };

        // Risk based on location (core files are riskier)
        if debt_item.file_path.contains("main.rs")
            || debt_item.file_path.contains("lib.rs")
            || debt_item.tags.contains(&"critical".to_string())
        {
            risk_score *= 1.25;
        }

        Ok(risk_score)
    }

    /// Assess timeline risk for different timescales
    pub async fn assess_timeline_risk(
        &self,
        debt_item: &DebtItem,
        delayed_weeks: u32,
    ) -> MaintenanceResult<TimelineRisk> {
        let base_risk = self.assess_risk(debt_item).await?;

        // Risk increases over time for delayed fixes
        let delay_multiplier = 1.0 + (delayed_weeks as f64 * 0.01);
        let delayed_risk = base_risk * delay_multiplier;

        Ok(TimelineRisk {
            immediate_risk: base_risk,
            delayed_risk,
            risk_increase_percent: (delayed_risk - base_risk) / base_risk * 100.0,
            optimal_fix_window: self.calculate_optimal_fix_window(debt_item),
        })
    }

    fn calculate_optimal_fix_window(&self, debt_item: &DebtItem) -> u32 {
        // Optimal fix window in weeks based on severity
        let base_window = match debt_item.severity {
            s if s > 0.8 => 1, // Fix within 1 week for critical debt
            s if s > 0.6 => 2, // Fix within 2 weeks for high severity
            s if s > 0.4 => 4, // Fix within 4 weeks for medium severity
            _ => 8,            // Fix within 8 weeks for low severity
        };

        // Adjust based on debt type
        let type_modifier = match debt_item.debt_type {
            DebtType::Security => 0.5,
            DebtType::Performance | DebtType::Architecture => 0.75,
            _ => 1.0,
        };

        (base_window as f64 * type_modifier) as u32
    }
}

/// Cost optimization engine
#[derive(Debug)]
pub struct CostOptimizationEngine {
    config: MaintenanceConfig,
}

impl CostOptimizationEngine {
    pub async fn new(config: &MaintenanceConfig) -> MaintenanceResult<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Optimize cost allocation across multiple debt items
    pub async fn optimize_costs(&self, cost_breakdowns: &[CostBreakdown]) -> MaintenanceResult<CostOptimization> {
        let total_cost: f64 = cost_breakdowns
            .iter()
            .map(|cb| cb.estimated_effort_hours)
            .sum();

        // Simple optimization: prioritize by cost-effectiveness
        let mut optimization_tips = Vec::new();

        // Identify quick wins (low effort, high impact)
        for (index, breakdown) in cost_breakdowns.iter().enumerate() {
            if breakdown.estimated_effort_hours < 4.0 && breakdown.urgency_score > 0.7 {
                optimization_tips.push(format!(
                    "Quick win: Item {} ({} hours) has high urgency ({:.1}%)",
                    index,
                    breakdown.estimated_effort_hours,
                    breakdown.urgency_score * 100.0
                ));
            }
        }

        // Identify high-risk items that need attention
        for (index, breakdown) in cost_breakdowns.iter().enumerate() {
            if breakdown.risk_factor > 1.5 {
                optimization_tips.push(format!(
                    "High-risk: Item {} has {:.1}% higher risk - consider early intervention",
                    index,
                    (breakdown.risk_factor - 1.0) * 100.0
                ));
            }
        }

        Ok(CostOptimization {
            total_optimized_cost: total_cost,
            estimated_savings_percentage: 10.0, // Conservative estimate
            optimization_tips,
            recommended_sequence: self.calculate_optimal_sequence(cost_breakdowns),
        })
    }

    fn calculate_optimal_sequence(&self, breakdowns: &[CostBreakdown]) -> Vec<usize> {
        // Sort by a combination of urgency and effort
        let mut indices: Vec<usize> = (0..breakdowns.len()).collect();

        indices.sort_by(|a, b| {
            let cost_a = &breakdowns[*a];
            let cost_b = &breakdowns[*b];

            // Prioritize high urgency with low effort first
            let score_a = cost_a.urgency_score - (cost_a.estimated_effort_hours / 100.0);
            let score_b = cost_b.urgency_score - (cost_b.estimated_effort_hours / 100.0);

            score_b.partial_cmp(&score_a).unwrap() // Higher score first
        });

        indices
    }
}

// Additional helper structures

/// Effort distribution across tasks
#[derive(Debug, Clone)]
pub struct EffortDistribution {
    pub total_effort:   f64,
    pub average_effort: f64,
    pub max_effort:     f64,
    pub min_effort:     f64,
    pub task_count:     usize,
}

/// Resource forecast for maintenance effort
#[derive(Debug, Clone)]
pub struct ResourceForecast {
    pub required_person_days:    f64,
    pub available_person_days:   f64,
    pub capacity_utilization:    f64,
    pub estimated_duration_days: u32,
    pub bottlenecks_identified:  bool,
}

/// Timeline risk assessment
#[derive(Debug, Clone)]
pub struct TimelineRisk {
    pub immediate_risk:        f64,
    pub delayed_risk:          f64,
    pub risk_increase_percent: f64,
    pub optimal_fix_window:    u32,
}

/// Cost optimization results
#[derive(Debug, Clone)]
pub struct CostOptimization {
    pub total_optimized_cost:         f64,
    pub estimated_savings_percentage: f64,
    pub optimization_tips:            Vec<String>,
    pub recommended_sequence:         Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cost_estimator_creation() {
        let config = MaintenanceConfig::default();
        let result = CostEstimator::new(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_basic_cost_estimation() {
        let config = MaintenanceConfig::default();
        let estimator = CostEstimator::new(&config).await.unwrap();

        let debt_item = DebtItem {
            id:                     "test-1".to_string(),
            file_path:              "src/main.rs".to_string(),
            start_line:             1,
            end_line:               10,
            debt_type:              DebtType::Complexity,
            severity:               0.7,
            description:            "High cyclic complexity".to_string(),
            estimated_effort_hours: 8.0,
            maintainability_impact: 0.6,
            age_days:               30,
            tags:                   vec!["core".to_string()],
        };

        let result = estimator.estimate_cost(&debt_item).await;
        assert!(result.is_ok());

        let breakdown = result.unwrap();
        assert!(breakdown.estimated_effort_hours > 0.0);
        assert!(breakdown.risk_factor >= 1.0);
        assert!(breakdown.complexity_multiplier >= 1.0);
        assert!(breakdown.urgency_score >= 0.0 && breakdown.urgency_score <= 1.0);
        assert!(!breakdown.components.is_empty());
    }

    #[tokio::test]
    async fn test_complexity_multiplier_calculation() {
        let config = MaintenanceConfig::default();
        let estimator = CostEstimator::new(&config).await.unwrap();

        let mut debt_item = DebtItem {
            id:                     "test-2".to_string(),
            file_path:              "src/main.rs".to_string(),
            start_line:             1,
            end_line:               10,
            debt_type:              DebtType::Security,
            severity:               0.8,
            description:            "Security vulnerability".to_string(),
            estimated_effort_hours: 12.0,
            maintainability_impact: 0.8,
            age_days:               365, // Very old debt
            tags:                   vec!["critical".to_string()],
        };

        let multiplier = estimator.calculate_complexity_multiplier(&debt_item);
        assert!(multiplier > 1.0); // Should be high for security debt in core file
        assert!(multiplier <= 3.0); // Should not exceed maximum
    }
}
