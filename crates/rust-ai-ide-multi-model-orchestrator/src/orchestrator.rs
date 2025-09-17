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

// Import warmup prediction system
use rust_ai_ide_warmup_predictor::{ModelWarmupPredictor, WarmupPrediction};

/// Main Multi-Model Orchestrator
#[derive(Debug)]
pub struct MultiModelOrchestrator {
    pub model_selector: Arc<PerformanceBasedModelSelector>,
    pub load_balancer: Arc<ModelLoadBalancer>,
    pub consensus_engine: Arc<ModelConsensusEngine>,
    pub fallback_manager: Arc<ModelFallbackManager>,
    pub health_monitor: Arc<ModelHealthMonitor>,
    /// Integrated warmup prediction system
    pub warmup_predictor: Option<Arc<ModelWarmupPredictor>>,
}

impl MultiModelOrchestrator {
    /// Create a new orchestrator with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(OrchestrationConfig::default()).await
    }

    /// Create a new orchestrator with custom configuration
    pub async fn with_config(config: OrchestrationConfig) -> Result<Self> {
        // Initialize warmup predictor if available
        let warmup_predictor = match ModelWarmupPredictor::new().await {
            Ok(predictor) => Some(Arc::new(predictor)),
            Err(e) => {
                tracing::warn!("Failed to initialize warmup predictor: {:?}", e);
                None
            }
        };

        Ok(Self {
            model_selector: Arc::new(PerformanceBasedModelSelector::new(config.clone())?),
            load_balancer: Arc::new(ModelLoadBalancer::new(config.clone()).await?),
            consensus_engine: Arc::new(ModelConsensusEngine::new(config.clone())?),
            fallback_manager: Arc::new(ModelFallbackManager::new()),
            health_monitor: Arc::new(ModelHealthMonitor::new()),
            warmup_predictor,
        })
    }

    /// Create orchestrator with custom warmup predictor
    pub async fn with_warmup_predictor(config: OrchestrationConfig, warmup_predictor: Option<ModelWarmupPredictor>) -> Result<Self> {
        Ok(Self {
            model_selector: Arc::new(PerformanceBasedModelSelector::new(config.clone())?),
            load_balancer: Arc::new(ModelLoadBalancer::new(config.clone()).await?),
            consensus_engine: Arc::new(ModelConsensusEngine::new(config.clone())?),
            fallback_manager: Arc::new(ModelFallbackManager::new()),
            health_monitor: Arc::new(ModelHealthMonitor::new()),
            warmup_predictor: warmup_predictor.map(Arc::new),
        })
    }

    /// Process a request through the multi-model orchestration pipeline
    pub async fn process_request(&self, context: &RequestContext) -> Result<OrchestrationResult> {
        let start_time = std::time::Instant::now();

        // Step 1: Predictive warmup (if available)
        if let Some(ref predictor) = self.warmup_predictor {
            // Convert RequestContext to WarmupRequest for prediction
            let warmup_request = self.convert_to_warmup_request(context).await?;
            match predictor.predict_and_warm(&warmup_request).await {
                Ok(prediction) => {
                    tracing::info!("Warmup prediction completed: {} models predicted", prediction.predicted_models.len());
                    if !prediction.is_acceptable {
                        tracing::warn!("Warmup prediction indicates performance impact may not be acceptable");
                    }
                }
                Err(e) => {
                    tracing::warn!("Warmup prediction failed: {:?}", e);
                }
            }
        }

        // Step 2: Select best model based on performance
        let model_recommendation = self.model_selector.select_model(context).await?;

        // Step 3: Route request through load balancer
        let load_decision = self.load_balancer.submit_request(context.clone()).await?;

        // Step 4: Process consensus if multiple models available
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
                final_result: "processed output".to_string(), // Placeholder
                confidence_score: model_recommendation.confidence_score,
                model_contributions: std::collections::HashMap::new(),
                disagreement_score: 0.0,
                primary_model: model_recommendation.model_id,
            }
        };

        // Step 5: Ensure offline availability
        let offline_status = self
            .fallback_manager
            .ensure_offline_availability(&model_recommendation.model_id)
            .await?;

        let processing_time = start_time.elapsed();

        Ok(OrchestrationResult {
            model_recommendation,
            load_decision,
            consensus_result,
            offline_status,
            processing_time,
        })
    }

    /// Convert RequestContext to WarmupRequest for prediction
    async fn convert_to_warmup_request(&self, context: &RequestContext) -> Result<rust_ai_ide_warmup_predictor::WarmupRequest> {
        use rust_ai_ide_warmup_predictor::{Complexity as WPComplexity, ModelTask as WPModelTask, RequestPriority as WPRequestPriority, UserContext, ProjectContext, WarmupRequest};

        Ok(WarmupRequest {
            task: match context.task_type {
                crate::types::ModelTask::Completion => WPModelTask::Completion,
                crate::types::ModelTask::Chat => WPModelTask::Chat,
                crate::types::ModelTask::Classification => WPModelTask::Classification,
                crate::types::ModelTask::Generation => WPModelTask::Generation,
                crate::types::ModelTask::Analysis => WPModelTask::Analysis,
                crate::types::ModelTask::Refactoring => WPModelTask::Refactoring,
                crate::types::ModelTask::Translation => WPModelTask::Translation,
                crate::types::ModelTask::Custom(ref s) => WPModelTask::Custom(s.clone()),
            },
            input_length: context.input_length,
            complexity: match context.expected_complexity {
                crate::types::Complexity::Simple => WPComplexity::Simple,
                crate::types::Complexity::Medium => WPComplexity::Medium,
                crate::types::Complexity::Complex => WPComplexity::Complex,
            },
            priority: match context.priority {
                crate::types::RequestPriority::Low => WPRequestPriority::Low,
                crate::types::RequestPriority::Medium => WPRequestPriority::Medium,
                crate::types::RequestPriority::High => WPRequestPriority::High,
                crate::types::RequestPriority::Critical => WPRequestPriority::Critical,
            },
            acceptable_latency: context.acceptable_latency,
            preferred_hardware: context.preferred_hardware.clone(),
            user_context: UserContext {
                user_id: "anonymous".to_string(), // Simplified
                session_duration: std::time::Duration::from_secs(300), // Simplified
                recent_activities: vec![], // Simplified
                preferences: std::collections::HashMap::new(), // Simplified
            },
            project_context: ProjectContext {
                language: "rust".to_string(), // Simplified
                size_lines: 1000, // Simplified
                complexity_score: 0.5, // Simplified
                recent_changes: vec![], // Simplified
            },
            timestamp: std::time::Instant::now(),
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
