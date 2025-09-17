//! Multi-Model Orchestrator Main Struct
//!
//! This module provides the primary interface for the multi-model orchestration system.

use std::sync::Arc;

use async_trait::async_trait;

use crate::consensus_engine::ModelConsensusEngine;
use crate::fallback_manager::ModelFallbackManager;
use crate::health_monitor::ModelHealthMonitor;
use crate::load_balancer::ModelLoadBalancer;
use crate::model_selector::PerformanceBasedModelSelector;
use crate::types::{
    ConsensusResult, LoadDecision, ModelRecommendation, OrchestrationConfig, RequestContext,
};
use crate::{OrchestrationError, Result};

/// Main Multi-Model Orchestrator
#[derive(Debug)]
pub struct MultiModelOrchestrator {
    pub model_selector: Arc<PerformanceBasedModelSelector>,
    pub load_balancer: Arc<ModelLoadBalancer>,
    pub consensus_engine: Arc<ModelConsensusEngine>,
    pub fallback_manager: Arc<ModelFallbackManager>,
    pub health_monitor: Arc<ModelHealthMonitor>,
}

impl MultiModelOrchestrator {
    /// Create a new orchestrator with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(OrchestrationConfig::default()).await
    }

    /// Create a new orchestrator with custom configuration
    pub async fn with_config(config: OrchestrationConfig) -> Result<Self> {
        Ok(Self {
            model_selector: Arc::new(PerformanceBasedModelSelector::new(config.clone())?),
            load_balancer: Arc::new(ModelLoadBalancer::new(config.clone()).await?),
            consensus_engine: Arc::new(ModelConsensusEngine::new(config.clone())?),
            fallback_manager: Arc::new(ModelFallbackManager::new()),
            health_monitor: Arc::new(ModelHealthMonitor::new()),
        })
    }

    /// Process a request through the multi-model orchestration pipeline
    pub async fn process_request(&self, context: &RequestContext) -> Result<OrchestrationResult> {
        // Step 1: Select best model based on performance
        let model_recommendation = self.model_selector.select_model(context).await?;

        // Step 2: Route request through load balancer
        let load_decision = self.load_balancer.submit_request(context.clone()).await?;

        // Step 3: Process consensus if multiple models available
        // This would collect outputs from multiple models (placeholder)
        let consensus_result = if load_decision.load_factor > 0.8 {
            // High load - use consensus approach
            self.consensus_engine
                .process_consensus(
                    std::collections::HashMap::new(), // Placeholder for actual model outputs
                    context,
                )
                .await?
        } else {
            // Use selected model directly
            ConsensusResult {
                final_result: "placeholder output".to_string(), // Placeholder
                confidence_score: model_recommendation.confidence_score,
                model_contributions: std::collections::HashMap::new(),
                disagreement_score: 0.0,
                primary_model: model_recommendation.model_id,
            }
        };

        // Step 4: Ensure offline availability
        let offline_status = self
            .fallback_manager
            .ensure_offline_availability(&model_recommendation.model_id)
            .await?;

        Ok(OrchestrationResult {
            model_recommendation,
            load_decision,
            consensus_result,
            offline_status,
            processing_time: std::time::Duration::from_millis(100), // Placeholder
        })
    }

    /// Get orchestrator health status
    pub async fn get_health_status(&self) -> OrchestratorHealth {
        OrchestratorHealth {
            model_selector_health: true,   // Placeholder
            load_balancer_health: true,    // Placeholder
            consensus_engine_health: true, // Placeholder
            fallback_manager_health: true, // Placeholder
            overall_health: true,          // Placeholder
        }
    }
}

/// Result of orchestration processing
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    pub model_recommendation: ModelRecommendation,
    pub load_decision: LoadDecision,
    pub consensus_result: ConsensusResult,
    pub offline_status: crate::types::OfflineStatus,
    pub processing_time: std::time::Duration,
}

/// Orchestrator health status
#[derive(Debug, Clone)]
pub struct OrchestratorHealth {
    pub model_selector_health: bool,
    pub load_balancer_health: bool,
    pub consensus_engine_health: bool,
    pub fallback_manager_health: bool,
    pub overall_health: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrchestrationConfigBuilder;
    use crate::types::{Complexity, ModelTask, RequestPriority};

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = MultiModelOrchestrator::new().await.unwrap();
        assert!(orchestrator.get_health_status().await.overall_health);
    }

    #[tokio::test]
    async fn test_request_processing() {
        let orchestrator = MultiModelOrchestrator::new().await.unwrap();

        let context = RequestContext {
            task_type: ModelTask::Completion,
            input_length: 100,
            priority: RequestPriority::Medium,
            expected_complexity: Complexity::Medium,
            acceptable_latency: std::time::Duration::from_secs(5),
            preferred_hardware: None,
        };

        // Note: This will fail due to no actual models - but tests the flow
        let result = orchestrator.process_request(&context).await;

        // Expected to fail without actual model setup
        assert!(result.is_err());
    }
}
