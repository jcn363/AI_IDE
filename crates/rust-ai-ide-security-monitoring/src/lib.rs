//! Security Monitoring and SIEM integration for Rust AI IDE
//!
//! This crate provides comprehensive security monitoring capabilities including:
//! - Real-time security event collection and correlation
//! - SIEM integration and log aggregation
//! - Policy enforcement with instant decision making
//! - Compliance monitoring with automated reporting
//! - Anomaly detection using behavioral analytics

pub mod siem;
pub mod policy;
pub mod collectors;
pub mod correlation;
pub mod alerting;
pub mod analytics;

pub use siem::{SiemIntegrationManager, SiemGateway, EventCollector};
pub use policy::{RealtimePolicyEnforcer, PolicyEngine, DecisionPoints};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use moka::future::Cache;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub severity: EventSeverity,
    pub source: String,
    pub details: serde_json::Value,
    pub actor: Option<String>,
    pub resource: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Event processing error: {0}")]
    ProcessingError(String),

    #[error("Policy enforcement error: {0}")]
    PolicyError(String),

    #[error("Correlation engine error: {0}")]
    CorrelationError(String),
}

pub type Result<T> = std::result::Result<T, MonitoringError>;

/// Main security monitoring service
#[derive(Clone)]
pub struct SecurityMonitor {
    event_cache: Arc<Cache<String, Vec<SecurityEvent>>>,
    siem_manager: Arc<SiemIntegrationManager>,
    policy_enforcer: Arc<RealtimePolicyEnforcer>,
    correlation_engine: Arc<correlation::CorrelationEngine>,
    alerting_engine: Arc<alerting::AlertingEngine>,
}

impl SecurityMonitor {
    pub async fn new() -> Result<Self> {
        let event_cache = Arc::new(
            Cache::builder()
                .time_to_live(std::time::Duration::from_secs(3600))
                .build()
        );

        let siem_manager = Arc::new(SiemIntegrationManager::new().await?);
        let policy_enforcer = Arc::new(RealtimePolicyEnforcer::new().await?);
        let correlation_engine = Arc::new(correlation::CorrelationEngine::new());
        let alerting_engine = Arc::new(alerting::AlertingEngine::new());

        Ok(Self {
            event_cache,
            siem_manager,
            policy_enforcer,
            correlation_engine,
            alerting_engine,
        })
    }

    pub async fn process_security_event(&self, event: SecurityEvent) -> Result<()> {
        // Cache event
        let key = format!("event_{}", event.id);
        self.event_cache.insert(key, vec![event.clone()]).await;

        // Process through SIEM
        self.siem_manager.process_event(event.clone()).await?;

        // Enforce policies
        self.policy_enforcer.enforce_on_event(&event).await?;

        // Run correlation analysis
        let correlations = self.correlation_engine.analyze_event(&event).await?;
        if !correlations.is_empty() {
            // Handle correlated events
            self.alerting_engine.process_correlations(&correlations).await?;
        }

        Ok(())
    }

    pub async fn get_security_status(&self) -> Result<MonitoringStatus> {
        let recent_events = self.event_cache.iter().count();
        let policy_violations = self.policy_enforcer.get_violation_count().await?;

        Ok(MonitoringStatus {
            total_events_processed: recent_events as u64,
            policy_violations,
            last_update: Utc::now(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct MonitoringStatus {
    pub total_events_processed: u64,
    pub policy_violations: u64,
    pub last_update: DateTime<Utc>,
}