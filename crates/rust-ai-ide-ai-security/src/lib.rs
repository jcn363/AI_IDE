//! # Wave 4: Secure AI Operations with Privacy-Preserving Computation 🔐🤖
//!
//! This crate implements advanced security features for AI operations building on Wave 3's
//! enterprise-grade security foundation. It provides:
//!
//! ## 🔒 Secure AI Features
//! - **Confidential AI Operations**: Encrypted model storage and inference
//! - **Privacy-Preserving Computation**: Differential privacy and zero-knowledge proofs
//! - **AI Audit Trails**: Complete decision lineage and explainability
//! - **Federated AI Security**: Secure distributed learning with data sovereignty
//! - **Quantum-Resistant Encryption**: Protection against future quantum threats
//! - **AI Compliance Automation**: Automated regulatory compliance for AI operations
//!
//! ## 🏗️ Architecture
//! ```
//! +---------------------+     +---------------------+     +---------------------+
//! |   Secure AI Engine  | --> |   Wave 3 Security   | --> |   AI Inference      |
//! +---------------------+     +---------------------+     +---------------------+
//!           |
//!           v
//! +---------------------+     +---------------------+     +---------------------+
//! | Privacy Preserving  | --> |  Federated Security | --> | AI Audit System    |
//! +---------------------+     +---------------------+     +---------------------+
//! ```
//!
//! ## 🚀 Usage
//! ```rust,no_run
//! use rust_ai_ide_ai_security::{SecureAIEngine, PrivacyConfig, AIComplianceConfig};
//!
//! // Initialize secure AI engine with Wave 3 integration
//! let mut secure_engine = SecureAIEngine::new_wave4(
//!     PrivacyConfig::high_privacy(),
//!     AIComplianceConfig::gdpr_compliant()
//! ).await?;
//!
//! // Perform secure AI inference with privacy guarantees
//! let result = secure_engine.infer_with_privacy(&model_request).await?;
//!
//! // Get complete audit trail
//! let audit_trail = secure_engine.get_audit_trail(&inference_id).await?;
//! ```

pub mod ai_audit;
pub mod ai_compliance;
pub mod encrypted_models;
pub mod federated_learning;
pub mod federated_security;
pub mod privacy_preservation;
pub mod quantum_resistant;
pub mod secure_ai_engine;
pub mod secure_inference;

// Wave 4 Re-exports
pub use ai_audit::{AIAuditEvent, AIAuditTrail, ExplainabilityReport};
pub use ai_compliance::{AIComplianceConfig, AIComplianceEngine};
pub use federated_learning::{FederatedLearningEngine, SecureAggregation};
pub use federated_security::FederatedSecurity;
pub use privacy_preservation::{PrivacyConfig, PrivacyGuard};
pub use quantum_resistant::{PostQuantumAIConfig, QuantumResistantAI};
pub use secure_ai_engine::{SecureAIEngine, Wave4Config};

// Core types that integrate with existing AI inference
use rust_ai_ide_ai_inference::{InferenceEngine, ModelLoadConfig};
use rust_ai_ide_security::{SecurityEngine, SecurityResult};

/// Core secure AI engine integrating Wave 3 security
pub struct SecureAIEngine {
    inference_engine: InferenceEngine,
    security_engine: SecurityEngine,
    privacy_guard: PrivacyGuard,
    audit_trail: AIAuditTrail,
    federated_security: Option<FederatedSecurity>,
    quantum_resistant: Option<QuantumResistantAI>,
}

impl SecureAIEngine {
    /// Initialize secure AI engine with Wave 4 enhancements
    pub async fn new_wave4(
        privacy_config: PrivacyConfig,
        compliance_config: AIComplianceConfig,
    ) -> SecurityResult<Self> {
        // Initialize core components
        let inference_engine = InferenceEngine::new().await?;
        let security_engine = rust_ai_ide_security::create_security_engine(
            rust_ai_ide_security::SecurityConfig::default(),
        )?;
        let privacy_guard = PrivacyGuard::new(privacy_config)?;
        let audit_trail = AIAuditTrail::new()?;

        Ok(Self {
            inference_engine,
            security_engine,
            privacy_guard,
            audit_trail,
            federated_security: None,
            quantum_resistant: None,
        })
    }

    /// Enable federated learning capabilities
    pub async fn enable_federated_learning(
        &mut self,
        participants: Vec<String>,
    ) -> SecurityResult<()> {
        self.federated_security = Some(FederatedSecurity::new(participants)?);
        Ok(())
    }

    /// Enable quantum-resistant encryption
    pub async fn enable_quantum_resistance(&mut self) -> SecurityResult<()> {
        self.quantum_resistant = Some(QuantumResistantAI::new()?);
        Ok(())
    }

    /// Secure AI inference with privacy guarantees
    pub async fn infer_with_privacy(
        &self,
        model_request: &secure_ai_engine::AIInferenceRequest,
    ) -> SecurityResult<secure_ai_engine::AISecureInferenceResult> {
        // 1. Check permissions (integrating Wave 3 RBAC/ABAC)
        let operation_context = model_request.to_operation_context();
        self.security_engine
            .secure_operation(&operation_context, || async {
                // 2. Apply privacy-preserving transformation
                let privacy_request = self.privacy_guard.apply_privacy(&model_request)?;

                // 3. Perform secure inference
                let inference_result = self
                    .inference_engine
                    .infer(&privacy_request.inner_request()?)
                    .await?;

                // 4. Generate audit trail
                let audit_id = self
                    .audit_trail
                    .create_audit_entry(&model_request, &inference_result)
                    .await?;

                Ok(secure_inference::AISecureInferenceResult::new(
                    inference_result,
                    audit_id,
                    self.privacy_guard.get_privacy_guarantees(),
                ))
            })
            .await??
            .inner_result()
    }

    /// Get audit trail for explainability
    pub async fn get_audit_trail(
        &self,
        inference_id: &str,
    ) -> SecurityResult<ExplainabilityReport> {
        self.audit_trail
            .generate_explainability_report(inference_id)
            .await
    }

    /// Secure federated learning training
    pub async fn secure_federated_training(
        &self,
        training_request: &federated_learning::FederatedTrainingRequest,
    ) -> SecurityResult<federated_learning::FederatedTrainingResult> {
        if let Some(federated) = &self.federated_security {
            federated.secure_training(training_request).await
        } else {
            Err(rust_ai_ide_security::SecurityError::ConfigurationError {
                config_error: "Federated learning not enabled".to_string(),
            })
        }
    }
}

/// Wave 4 configuration combining AI and security settings
#[derive(Debug, Clone)]
pub struct Wave4Config {
    pub ai_config: ModelLoadConfig,
    pub security_config: rust_ai_ide_security::SecurityConfig,
    pub privacy_config: PrivacyConfig,
    pub compliance_config: AIComplianceConfig,
    pub quantum_resistant: bool,
    pub federated_enabled: bool,
}

impl Default for Wave4Config {
    fn default() -> Self {
        Self {
            ai_config: ModelLoadConfig {
                quantization: Some(rust_ai_ide_ai_inference::Quantization::FP16),
                lora_adapters: vec![],
                memory_limit_mb: Some(4096),
                device: rust_ai_ide_ai_inference::ModelDevice::Auto,
                lazy_loading: true,
                enable_cache: true,
            },
            security_config: rust_ai_ide_security::SecurityConfig::default(),
            privacy_config: PrivacyConfig::balanced(),
            compliance_config: AIComplianceConfig::gdpr_compliant(),
            quantum_resistant: false,
            federated_enabled: false,
        }
    }
}
