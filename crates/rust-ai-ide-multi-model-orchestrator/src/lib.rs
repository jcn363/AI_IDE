/*! # Multi-Model Orchestration for Rust AI IDE
!
! This crate implements Phase 2.3 Multi-Model Orchestration features,
! building upon existing orchestration, quantization, and predictive quality infrastructure.
!
! ## Features
!
! - Performance-based model selection with real-time metrics
! - Intelligent load balancing across multiple model instances
! - Cross-model consensus mechanisms for improved accuracy
! - Offline model management with graceful degradation
! - LSP integration for model performance feedback
! - Tauri commands for frontend model switching UI
!
! ## Integration Points
!
! - **Phase 1 Orchestration**: Service discovery and health monitoring
! - **Phase 2.1 Quantization**: GGUF format support for efficient inference
! - **Phase 2.2 Predictive Quality**: Performance prediction models
! - **EventBus**: Real-time orchestration coordination
! - **LSP Service**: Model performance metrics collection
*/

pub mod model_selector;
pub mod load_balancer;
pub mod consensus_engine;
pub mod fallback_manager;
pub mod orchestrator;
pub mod health_monitor;
pub mod types;
pub mod config;
pub mod utils;

/// Main orchestrator struct that coordinates all multi-model orchestration components
pub use orchestrator::MultiModelOrchestrator;

/// Errors that can occur during multi-model orchestration
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationError {
    #[error("Model selection failed: {0}")]
    ModelSelectionError(String),

    #[error("Load balancing failed: {0}")]
    LoadBalancingError(String),

    #[error("Consensus calculation failed: {0}")]
    ConsensusError(String),

    #[error("Fallback mechanism failed: {0}")]
    FallbackError(String),

    #[error("Health monitoring failed: {0}")]
    HealthMonitoringError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

pub type Result<T> = std::result::Result<T, OrchestrationError>;