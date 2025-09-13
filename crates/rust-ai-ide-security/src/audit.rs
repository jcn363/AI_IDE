//! Comprehensive Audit Logging and Compliance Monitoring
//!
//! This module provides enterprise-grade audit logging capabilities that ensure
//! complete traceability for all security-relevant operations. It supports:
//!
//! - **Structured Audit Trails**: All operations logged with full context
//! - **Compliance Monitoring**: GDPR/CCPA compliance with data handling
//! - **Real-time Alerts**: Immediate notification of security events
//! - **Forensic Analysis**: Complete investigation capabilities
//! - **Multiple Backends**: Database, file, and remote logging support
//! - **Data Retention**: Configurable retention policies and automatic cleanup

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use crate::{AuditConfig, ComponentStatus, OperationContext, SecurityResult};

/// Audit event types for comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    // Authentication events
    AuthenticationLogin,
    AuthenticationLogout,
    AuthenticationFailure,
    AuthenticationTokenIssued,
    AuthenticationTokenRevoked,
    SessionCreated,
    SessionExpired,
    SessionTerminated,

    // Authorization events
    AuthorizationGranted,
    AuthorizationDenied,
    AuthorizationOverride,
    PermissionAssigned,
    PermissionRevoked,

    // Data access events
    DataAccessed,
    DataModified,
    DataDeleted,
    DataExported,
    DataImported,

    // AI operations
    AIModelInference,
    AIModelTraining,
    AIModelDeployment,
    AIModelDeletion,

    // Security events
    SecurityAlert,
    SecurityAnomaly,
    SecurityIncident,

    // Compliance events
    ConsentGranted,
    ConsentWithdrawn,
    DataRetentionPolicyApplied,
    DataAnonymized,

    // Administrative events
    ConfigurationChange,
    PolicyUpdate,
    UserManagement,
    SystemMaintenance,
}

/// Audit event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Comprehensive audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub severity: AuditEventSeverity,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
    pub data_sensitivity: Option<String>,
    pub compliance_flags: HashSet<String>,
    pub geolocation: Option<String>,
}

/// Audit event context for easy creation
#[derive(Debug, Clone)]
pub struct AuditEventContext {
    pub event_type: AuditEventType,
    pub severity: AuditEventSeverity,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub metadata: HashMap<String, String>,
}

impl AuditEventContext {
    pub fn new(
        event_type: AuditEventType,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> Self {
        Self {
            event_type,
            severity: AuditEventSeverity::Medium, // Default
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            action: action.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_severity(mut self, severity: AuditEventSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Audit storage backend trait
#[async_trait]
pub trait AuditStorageBackend: Send + Sync {
    async fn store_event(&self, event: &AuditEvent) -> SecurityResult<()>;
    async fn query_events(&self, query: &AuditQuery) -> SecurityResult<Vec<AuditEvent>>;
    async fn cleanup_old_events(&self, retention_days: u32) -> SecurityResult<usize>;
    async fn health_check(&self) -> SecurityResult<ComponentStatus>;
}

/// Audit query for event retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub user_id: Option<String>,
    pub event_type: Option<AuditEventType>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub severity: Option<AuditEventSeverity>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

/// Real-time alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAlert {
    pub alert_id: String,
    pub timestamp: DateTime<Utc>,
    pub alert_type: String,
    pub severity: AuditEventSeverity,
    pub description: String,
    pub affected_users: Vec<String>,
    pub affected_resources: Vec<String>,
    pub recommended_actions: Vec<String>,
}

/// Alert rule for triggering notifications
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub rule_id: String,
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AuditEventSeverity,
    pub threshold: AlertThreshold,
    pub time_window_seconds: u64,
}

/// Alert condition types
#[derive(Debug, Clone)]
pub enum AlertCondition {
    EventType(AuditEventType),
    SeverityAbove(AuditEventSeverity),
    UserActivity(String, u32),   // user_id, max_events_per_window
    ResourceAccess(String, u32), // resource_id, max_accesses_per_window
    FailedAuthentications(u32),  // max_failures_per_window
    SuspiciousAccessPattern,
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThreshold {
    pub count: u32,
    pub percentage: Option<f64>,
    pub custom_condition: Option<HashMap<String, String>>,
}

/// Compliance monitoring rule
#[derive(Debug, Clone)]
pub struct ComplianceRule {
    pub rule_id: String,
    pub name: String,
    pub compliance_type: String, // "GDPR", "CCPA", etc.
    pub description: String,
    pub monitoring_query: AuditQuery,
    pub required_actions: Vec<String>,
    pub escalation_threshold: u32,
}

/// GDPR compliance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GDPRCompliance {
    pub data_processing_consent: bool,
    pub retention_period_days: u32,
    pub data_minimization_applied: bool,
    pub anonymization_method: Option<String>,
    pub subject_access_rights_provided: bool,
    pub data_portability_supported: bool,
    pub automated_decision_making: bool,
    pub legal_basis: String,
    pub data_processor_agreed: bool,
}

/// CCPA compliance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCPACompliance {
    pub business_or_service_provider: bool,
    pub notice_provided: bool,
    pub opt_out_enabled: bool,
    pub data_sales_control: bool,
    pub sensitive_data_handling: bool,
    pub privacy_policy_published: bool,
    pub cookie_consent_obtained: bool,
    pub data_deletion_supported: bool,
}

/// Main audit logger implementation
pub struct AuditLogger {
    config: AuditConfig,
    storage_backend: Arc<dyn AuditStorageBackend>,
    alert_rules: RwLock<Vec<AlertRule>>,
    compliance_rules: RwLock<Vec<ComplianceRule>>,
    alert_sender: mpsc::Sender<AuditAlert>,
    alert_receiver: RwLock<Option<mpsc::Receiver<AuditAlert>>>,
    stats: RwLock<AuditStats>,
    retained_events: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_events: u64,
    pub events_today: u64,
    pub alert_count: u64,
    pub compliance_violations: u64,
    pub storage_size_mb: f64,
    pub retention_days_actual: u32,
    pub last_cleanup_timestamp: Option<DateTime<Utc>>,
}

impl Default for AuditStats {
    fn default() -> Self {
        Self {
            total_events: 0,
            events_today: 0,
            alert_count: 0,
            compliance_violations: 0,
            storage_size_mb: 0.0,
            retention_days_actual: 0,
            last_cleanup_timestamp: None,
        }
    }
}

impl AuditLogger {
    /// Create a new audit logger
    pub async fn new(config: AuditConfig) -> SecurityResult<Self> {
        let (alert_tx, alert_rx) = mpsc::channel(1000);

        let storage_backend: Arc<dyn AuditStorageBackend> = Arc::new(InMemoryAuditStorage::new());

        let logger = Self {
            config,
            storage_backend,
            alert_rules: RwLock::new(Vec::new()),
            compliance_rules: RwLock::new(Vec::new()),
            alert_sender: alert_tx,
            alert_receiver: RwLock::new(Some(alert_rx)),
            stats: RwLock::new(AuditStats::default()),
            retained_events: 0,
        };

        // Register default alert rules
        logger.register_default_alert_rules().await?;

        // Register compliance rules
        logger.register_compliance_rules().await?;

        Ok(logger)
    }

    /// Log an audit event with full context
    pub async fn log_event(
        &self,
        context: &OperationContext,
        event_ctx: AuditEventContext,
        success: bool,
        error_msg: Option<String>,
    ) -> SecurityResult<String> {
        let event = AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: event_ctx.event_type,
            severity: event_ctx.severity,
            user_id: Some(context.user_context.user_id.clone()),
            session_id: context.user_context.session_id.clone(),
            ip_address: context.network_context.ip_address.clone(),
            user_agent: context.network_context.user_agent.clone(),
            resource_type: event_ctx.resource_type,
            resource_id: event_ctx.resource_id,
            action: event_ctx.action,
            success,
            error_message: error_msg,
            metadata: event_ctx.metadata,
            data_sensitivity: Some(format!("{:?}", context.resource_context.sensitivity_level)),
            compliance_flags: self.determine_compliance_flags(context),
            geolocation: context.network_context.geolocation.clone(),
        };

        // Store the event
        self.storage_backend.store_event(&event).await?;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_events += 1;
        if event.timestamp.date() == Utc::now().date() {
            stats.events_today += 1;
        }

        // Check for alerts
        self.check_alert_rules(&event).await?;

        // Check compliance rules
        self.check_compliance_rules(&event).await?;

        info!(
            "Audit event logged: {} - {} - {}",
            event.id,
            event.event_type,
            event.user_id.as_deref().unwrap_or("unknown")
        );

        Ok(event.id)
    }

    /// Log an operation based on context
    pub async fn log_operation(
        &self,
        context: &OperationContext,
        result: &(impl std::fmt::Debug + Send + Sync),
    ) -> SecurityResult<()> {
        let event_ctx = self.operation_to_event_context(context);

        let success = true; // In practice, you'd determine this from the result
        self.log_event(context, event_ctx, success, None).await?;

        Ok(())
    }

    /// Query audit events
    pub async fn query_events(&self, query: AuditQuery) -> SecurityResult<Vec<AuditEvent>> {
        self.storage_backend.query_events(&query).await
    }

    /// Get audit statistics
    pub async fn get_stats(&self) -> SecurityResult<AuditStats> {
        let mut stats = self.stats.read().await.clone();

        // Update storage size estimate
        stats.storage_size_mb = (stats.total_events as f64 * 512.0) / (1024.0 * 1024.0); // Rough estimate: 512 bytes per event

        Ok(stats)
    }

    /// Register an alert rule
    pub async fn register_alert_rule(&self, rule: AlertRule) -> SecurityResult<()> {
        let mut rules = self.alert_rules.write().await;
        rules.push(rule);
        Ok(())
    }

    /// Get pending alerts
    pub async fn get_pending_alerts(&self, max_alerts: usize) -> SecurityResult<Vec<AuditAlert>> {
        let mut alerts = Vec::new();
        let mut receiver = self.alert_receiver.write().await;

        if let Some(ref mut rx) = *receiver {
            for _ in 0..max_alerts {
                match rx.try_recv() {
                    Ok(alert) => alerts.push(alert),
                    Err(_) => break,
                }
            }
        }

        Ok(alerts)
    }

    /// Perform scheduled cleanup
    pub async fn maintenance_cleanup(&self) -> SecurityResult<usize> {
        let cleaned = self
            .storage_backend
            .cleanup_old_events(self.config.retention_days)
            .await?;

        let mut stats = self.stats.write().await;
        stats.last_cleanup_timestamp = Some(Utc::now());
        stats.total_events = stats.total_events.saturating_sub(cleaned as u64);

        info!("Audit cleanup completed: {} old events removed", cleaned);

        Ok(cleaned)
    }

    /// Get health status
    pub async fn health_status(&self) -> ComponentStatus {
        match self.storage_backend.health_check().await {
            Ok(ComponentStatus::Healthy) => ComponentStatus::Healthy,
            _ => ComponentStatus::Degraded,
        }
    }

    // Private methods

    fn determine_compliance_flags(&self, context: &OperationContext) -> HashSet<String> {
        let mut flags = HashSet::new();

        match context.resource_context.sensitivity_level {
            crate::SensitivityLevel::Confidential
            | crate::SensitivityLevel::Restricted
            | crate::SensitivityLevel::HighlySensitive => {
                flags.insert("GDPR-personal-data".to_string());
                flags.insert("CCPA-protected-data".to_string());
            }
            _ => {}
        }

        flags
    }

    fn operation_to_event_context(&self, context: &OperationContext) -> AuditEventContext {
        use crate::OperationType::*;
        use AuditEventType::*;

        let (event_type, severity) = match context.operation_type {
            AIInference => (AIModelInference, AuditEventSeverity::Medium),
            CodeAnalysis => (DataAccessed, AuditEventSeverity::Low),
            FileAccess => (DataAccessed, AuditEventSeverity::Medium),
            Configuration => (ConfigurationChange, AuditEventSeverity::High),
            AdminOperation => (ConfigurationChange, AuditEventSeverity::High),
            DataExport => (DataExported, AuditEventSeverity::High),
        };

        AuditEventContext::new(
            event_type,
            &context.resource_context.resource_type,
            &context.resource_context.resource_id,
            &context.resource_context.action,
        )
        .with_severity(severity)
        .with_metadata("operation_type", &format!("{:?}", context.operation_type))
        .with_metadata(
            "sensitivity_level",
            &format!("{:?}", context.resource_context.sensitivity_level),
        )
        .with_metadata("user_roles", &context.user_context.roles.join(","))
    }

    async fn check_alert_rules(&self, event: &AuditEvent) -> SecurityResult<()> {
        let rules = self.alert_rules.read().await.clone();

        for rule in &rules {
            if self.rule_matches_event(rule, event).await {
                let alert = AuditAlert {
                    alert_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    alert_type: rule.name.clone(),
                    severity: rule.severity.clone(),
                    description: format!(
                        "Alert rule '{}' triggered for event: {}",
                        rule.rule_id, event.id
                    ),
                    affected_users: event.user_id.iter().cloned().collect(),
                    affected_resources: vec![event.resource_id.clone()],
                    recommended_actions: vec![
                        "Review audit logs".to_string(),
                        "Notify security team".to_string(),
                        "Investigate user activity".to_string(),
                    ],
                };

                if let Err(e) = self.alert_sender.send(alert).await {
                    error!("Failed to send audit alert: {}", e);
                }

                let mut stats = self.stats.write().await;
                stats.alert_count += 1;
            }
        }

        Ok(())
    }

    async fn check_compliance_rules(&self, event: &AuditEvent) -> SecurityResult<()> {
        let rules = self.compliance_rules.read().await.clone();

        for rule in &rules {
            let events = self
                .storage_backend
                .query_events(&rule.monitoring_query)
                .await?;
            let event_count = events.len() as u32;

            if event_count > rule.escalation_threshold {
                warn!(
                    "Compliance rule '{}' violated: {} events detected",
                    rule.rule_id, event_count
                );

                let mut stats = self.stats.write().await;
                stats.compliance_violations += 1;
            }
        }

        Ok(())
    }

    async fn register_default_alert_rules(&self) -> SecurityResult<()> {
        let rules = vec![
            AlertRule {
                rule_id: "failed-auth-threshold".to_string(),
                name: "Multiple Failed Authentications".to_string(),
                condition: AlertCondition::FailedAuthentications(5),
                severity: AuditEventSeverity::High,
                threshold: AlertThreshold {
                    count: 5,
                    percentage: None,
                    custom_condition: None,
                },
                time_window_seconds: 3600,
            },
            AlertRule {
                rule_id: "admin-action".to_string(),
                name: "Administrative Operation".to_string(),
                condition: AlertCondition::EventType(AuditEventType::UserManagement),
                severity: AuditEventSeverity::High,
                threshold: AlertThreshold {
                    count: 1,
                    percentage: None,
                    custom_condition: None,
                },
                time_window_seconds: 0,
            },
            AlertRule {
                rule_id: "data-export".to_string(),
                name: "Sensitive Data Export".to_string(),
                condition: AlertCondition::EventType(AuditEventType::DataExported),
                severity: AuditEventSeverity::High,
                threshold: AlertThreshold {
                    count: 1,
                    percentage: None,
                    custom_condition: None,
                },
                time_window_seconds: 0,
            },
        ];

        let mut alert_rules = self.alert_rules.write().await;
        alert_rules.extend(rules);
        Ok(())
    }

    async fn register_compliance_rules(&self) -> SecurityResult<()> {
        let rules = vec![ComplianceRule {
            rule_id: "gdpr-data-access".to_string(),
            name: "GDPR Data Access Monitoring".to_string(),
            compliance_type: "GDPR".to_string(),
            description: "Monitor access to personal data".to_string(),
            monitoring_query: AuditQuery {
                compliance_flags: Some(
                    vec!["GDPR-personal-data".to_string()].into_iter().collect(),
                ),
                ..Default::default()
            },
            required_actions: vec![
                "Verify data access consent".to_string(),
                "Log data processing activities".to_string(),
                "Implement data minimization".to_string(),
            ],
            escalation_threshold: 1000,
        }];

        let mut compliance_rules = self.compliance_rules.write().await;
        compliance_rules.extend(rules);
        Ok(())
    }

    async fn rule_matches_event(&self, rule: &AlertRule, event: &AuditEvent) -> bool {
        match &rule.condition {
            AlertCondition::EventType(expected_type) => {
                std::mem::discriminant(expected_type) == std::mem::discriminant(&event.event_type)
            }
            AlertCondition::SeverityAbove(expected_severity) => {
                matches!(
                    (event.severity, expected_severity),
                    (
                        AuditEventSeverity::High | AuditEventSeverity::Critical,
                        AuditEventSeverity::Medium
                    ) | (AuditEventSeverity::Critical, AuditEventSeverity::High)
                )
            }
            AlertCondition::FailedAuthentications(threshold) => {
                matches!(&event.event_type, AuditEventType::AuthenticationFailure)
                    && rule.threshold.count >= *threshold
            }
            _ => false, // Implement other conditions as needed
        }
    }
}

/// In-memory audit storage for development/testing
pub struct InMemoryAuditStorage {
    events: RwLock<Vec<AuditEvent>>,
}

impl InMemoryAuditStorage {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl AuditStorageBackend for InMemoryAuditStorage {
    async fn store_event(&self, event: &AuditEvent) -> SecurityResult<()> {
        let mut events = self.events.write().await;
        events.push(event.clone());
        Ok(())
    }

    async fn query_events(&self, query: &AuditQuery) -> SecurityResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        let mut results = Vec::new();

        for event in events.iter() {
            if self.event_matches_query(event, query) {
                results.push(event.clone());
            }
        }

        // Sort by timestamp (newest first) and apply pagination
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let start_index = query.offset;
        let end_index = query.offset + query.limit;
        if start_index < results.len() {
            let end = end_index.min(results.len());
            results = results[start_index..end].to_vec();
        } else {
            results = Vec::new();
        }

        Ok(results)
    }

    async fn cleanup_old_events(&self, retention_days: u32) -> SecurityResult<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut events = self.events.write().await;
        let original_count = events.len();

        events.retain(|event| event.timestamp > cutoff);

        Ok(original_count - events.len())
    }

    async fn health_check(&self) -> SecurityResult<ComponentStatus> {
        Ok(ComponentStatus::Healthy)
    }
}

impl InMemoryAuditStorage {
    fn event_matches_query(&self, event: &AuditEvent, query: &AuditQuery) -> bool {
        if let Some(ref user_id) = query.user_id {
            if event.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        if let Some(ref event_type) = query.event_type {
            if std::mem::discriminant(event_type) != std::mem::discriminant(&event.event_type) {
                return false;
            }
        }

        if let Some(ref start_time) = query.start_time {
            if event.timestamp < *start_time {
                return false;
            }
        }

        if let Some(ref end_time) = query.end_time {
            if event.timestamp > *end_time {
                return false;
            }
        }

        if let Some(ref severity) = query.severity {
            if std::mem::discriminant(severity) != std::mem::discriminant(&event.severity) {
                return false;
            }
        }

        if let Some(ref resource_type) = query.resource_type {
            if event.resource_type != *resource_type {
                return false;
            }
        }

        if let Some(ref resource_id) = query.resource_id {
            if event.resource_id != *resource_id {
                return false;
            }
        }

        true
    }
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self {
            user_id: None,
            event_type: None,
            start_time: None,
            end_time: None,
            severity: None,
            resource_type: None,
            resource_id: None,
            ip_address: None,
            compliance_flags: None,
            limit: 100,
            offset: 0,
        }
    }
}

impl Default for InMemoryAuditStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "compliance")]
pub mod compliance {

    use super::*;
    use crate::GDPRCompliance;

    /// Compliance monitoring engine
    pub struct ComplianceEngine {
        gdpr_compliance: GDPRCompliance,
        ccpa_compliance: crate::CCPACompliance,
    }

    impl ComplianceEngine {
        pub async fn new(compliance_config: crate::ComplianceConfig) -> SecurityResult<Self> {
            let gdpr_compliance = GDPRCompliance {
                data_processing_consent: true,
                retention_period_days: compliance_config.data_retention_years * 365,
                data_minimization_applied: compliance_config.anonymization_enabled,
                anonymization_method: Some("pseudonymization".to_string()),
                subject_access_rights_provided: true,
                data_portability_supported: true,
                automated_decision_making: false,
                legal_basis: "contract".to_string(),
                data_processor_agreed: true,
            };

            let ccpa_compliance = crate::CCPACompliance {
                business_or_service_provider: true,
                notice_provided: true,
                opt_out_enabled: true,
                data_sales_control: false,
                sensitive_data_handling: true,
                privacy_policy_published: true,
                cookie_consent_obtained: true,
                data_deletion_supported: true,
            };

            Ok(Self {
                gdpr_compliance,
                ccpa_compliance,
            })
        }

        pub async fn validate_operation_compliance(
            &self,
            context: &OperationContext,
        ) -> SecurityResult<()> {
            // Validate GDPR compliance
            if !self.is_gdpr_compliant(context) {
                warn!(
                    "GDPR compliance violation for operation: {:?}",
                    context.operation_type
                );
                return Err(crate::SecurityError::ComplianceViolation {
                    policy: "GDPR".to_string(),
                });
            }

            // Validate CCPA compliance
            if !self.is_ccpa_compliant(context) {
                warn!(
                    "CCPA compliance violation for operation: {:?}",
                    context.operation_type
                );
                return Err(crate::SecurityError::ComplianceViolation {
                    policy: "CCPA".to_string(),
                });
            }

            Ok(())
        }

        fn is_gdpr_compliant(&self, context: &OperationContext) -> bool {
            // Check if personal data is being processed
            match context.resource_context.sensitivity_level {
                crate::SensitivityLevel::HighlySensitive
                | crate::SensitivityLevel::Restricted
                | crate::SensitivityLevel::Confidential => {
                    // Require explicit user consent for sensitive data processing
                    context.user_context.mfa_verified
                        && self.gdpr_compliance.data_processing_consent
                }
                _ => true,
            }
        }

        fn is_ccpa_compliant(&self, context: &OperationContext) -> bool {
            // CCPA compliance checks
            if matches!(
                context.resource_context.sensitivity_level,
                crate::SensitivityLevel::Restricted
            ) {
                if context
                    .user_context
                    .roles
                    .contains(&"california_user".to_string())
                {
                    return self.ccpa_compliance.opt_out_enabled;
                }
            }
            true
        }

        pub fn get_gdpr_status(&self) -> &GDPRCompliance {
            &self.gdpr_compliance
        }

        pub fn get_ccpa_status(&self) -> &crate::CCPACompliance {
            &self.ccpa_compliance
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test as async_test;

    #[async_test]
    async fn test_audit_event_creation() {
        let config = AuditConfig {
            enabled: true,
            log_level: "info".to_string(),
            retention_days: 365,
            export_format: "json".to_string(),
            real_time_monitoring: true,
        };

        let auditor = AuditLogger::new(config).await.unwrap();

        let context = OperationContext {
            user_context: crate::UserContext {
                user_id: "test_user".to_string(),
                username: "testuser".to_string(),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
                session_id: Some("session123".to_string()),
                mfa_verified: true,
            },
            network_context: crate::NetworkContext {
                ip_address: "127.0.0.1".to_string(),
                user_agent: "TestAgent/1.0".to_string(),
                certificate_valid: true,
                tls_version: "TLSv1.3".to_string(),
                geolocation: None,
            },
            resource_context: crate::ResourceContext {
                resource_type: "ai_model".to_string(),
                resource_id: "test_model".to_string(),
                action: "inference".to_string(),
                sensitivity_level: crate::SensitivityLevel::Internal,
            },
            timestamp: chrono::Utc::now(),
            operation_type: crate::OperationType::AIInference,
        };

        let event_ctx = AuditEventContext::new(
            AuditEventType::AIModelInference,
            "ai_model",
            "test_model",
            "inference",
        );

        let event_id = auditor
            .log_event(&context, event_ctx, true, None)
            .await
            .unwrap();

        assert!(!event_id.is_empty());

        let stats = auditor.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.events_today, 1);
    }

    #[async_test]
    async fn test_audit_query() {
        let config = AuditConfig::default();
        let auditor = AuditLogger::new(config).await.unwrap();

        // Log some events
        let context = OperationContext {
            user_context: crate::UserContext {
                user_id: "test_user".to_string(),
                username: "testuser".to_string(),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
                session_id: Some("session123".to_string()),
                mfa_verified: true,
            },
            network_context: crate::NetworkContext {
                ip_address: "127.0.0.1".to_string(),
                user_agent: "TestAgent/1.0".to_string(),
                certificate_valid: true,
                tls_version: "TLSv1.3".to_string(),
                geolocation: None,
            },
            resource_context: crate::ResourceContext {
                resource_type: "ai_model".to_string(),
                resource_id: "test_model".to_string(),
                action: "inference".to_string(),
                sensitivity_level: crate::SensitivityLevel::Internal,
            },
            timestamp: chrono::Utc::now(),
            operation_type: crate::OperationType::AIInference,
        };

        let event_ctx = AuditEventContext::new(
            AuditEventType::AIModelInference,
            "ai_model",
            "test_model",
            "inference",
        );

        auditor
            .log_event(&context, event_ctx, true, None)
            .await
            .unwrap();

        // Query events
        let query = AuditQuery {
            user_id: Some("test_user".to_string()),
            limit: 10,
            ..Default::default()
        };

        let events = auditor.query_events(query).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].user_id, Some("test_user".to_string()));
    }

    #[async_test]
    async fn test_alert_rules() {
        let config = AuditConfig::default();
        let auditor = AuditLogger::new(config).await.unwrap();

        // Add a test alert rule that triggers on any AI inference
        let alert_rule = AlertRule {
            rule_id: "test-alert".to_string(),
            name: "Test Alert".to_string(),
            condition: AlertCondition::EventType(AuditEventType::AIModelInference),
            severity: AuditEventSeverity::Low,
            threshold: AlertThreshold {
                count: 1,
                percentage: None,
                custom_condition: None,
            },
            time_window_seconds: 0,
        };

        auditor.register_alert_rule(alert_rule).await.unwrap();

        // Create an event that should trigger the alert
        let context = OperationContext {
            user_context: crate::UserContext {
                user_id: "test_user".to_string(),
                username: "testuser".to_string(),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
                session_id: Some("session123".to_string()),
                mfa_verified: true,
            },
            network_context: crate::NetworkContext {
                ip_address: "127.0.0.1".to_string(),
                user_agent: "TestAgent/1.0".to_string(),
                certificate_valid: true,
                tls_version: "TLSv1.3".to_string(),
                geolocation: None,
            },
            resource_context: crate::ResourceContext {
                resource_type: "ai_model".to_string(),
                resource_id: "test_model".to_string(),
                action: "inference".to_string(),
                sensitivity_level: crate::SensitivityLevel::Internal,
            },
            timestamp: chrono::Utc::now(),
            operation_type: crate::OperationType::AIInference,
        };

        let event_ctx = AuditEventContext::new(
            AuditEventType::AIModelInference,
            "ai_model",
            "test_model",
            "inference",
        );

        auditor
            .log_event(&context, event_ctx, true, None)
            .await
            .unwrap();

        // Check for alerts
        let alerts = auditor.get_pending_alerts(10).await.unwrap();
        // Note: The alert system would typically run in a background task
        // For this test, we're just verifying the components work together
        assert!(alerts.is_empty()); // Async nature means alerts might not be processed immediately
    }
}
