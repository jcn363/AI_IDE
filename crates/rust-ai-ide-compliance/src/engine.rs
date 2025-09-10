//! Compliance Engine Architecture
//!
//! The central compliance engine that orchestrates all compliance verification,
//! policy enforcement, audit trails, and regulatory reporting for GDPR and HIPAA.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use async_trait::async_trait;

use crate::core::{
    ComplianceConfig, ComplianceError, ComplianceResult, AuditEntry, AuditSeverity,
    PolicyConfig, ComplianceStatus, ComplianceFramework,
};
use crate::audit::AuditTrailManager;
use crate::policy::PolicyEnforcementEngine;
use crate::risk::RiskAssessmentEngine;
use crate::data_management::{DataAnonymizationService, ConsentManager};

#[cfg(feature = "gdpr")]
use crate::gdpr::GdprProcessor;

#[cfg(feature = "hipaa")]
use crate::hipaa::HipaaProcessor;

/// Central compliance engine that manages all compliance operations
#[derive(Debug)]
pub struct ComplianceEngine {
    /// Engine configuration
    config: Arc<RwLock<ComplianceConfig>>,
    /// GDPR processor (if enabled)
    #[cfg(feature = "gdpr")]
    gdpr_processor: Arc<Mutex<Option<GdprProcessor>>>,
    /// HIPAA processor (if enabled)
    #[cfg(feature = "hipaa")]
    hipaa_processor: Arc<Mutex<Option<HipaaProcessor>>>,
    /// Audit trail manager
    audit_manager: Arc<Mutex<AuditTrailManager>>,
    /// Policy enforcement engine
    policy_engine: Arc<Mutex<PolicyEnforcementEngine>>,
    /// Risk assessment engine
    risk_engine: Arc<Mutex<RiskAssessmentEngine>>,
    /// Data anonymization service
    data_anonymizer: Arc<Mutex<DataAnonymizationService>>,
    /// Consent manager
    consent_manager: Arc<Mutex<ConsentManager>>,
    /// Engine initialization status
    initialized: Arc<RwLock<bool>>,
    /// Background task handles
    background_tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    /// Performance metrics
    metrics: Arc<Mutex<ComplianceMetrics>>,
}

impl ComplianceEngine {
    /// Create a new compliance engine with default configuration
    pub async fn new() -> ComplianceResult<Self> {
        Self::with_config(ComplianceConfig::default()).await
    }

    /// Create a new compliance engine with custom configuration
    pub async fn with_config(config: ComplianceConfig) -> ComplianceResult<Self> {
        let mut engine = Self {
            config: Arc::new(RwLock::new(config)),
            #[cfg(feature = "gdpr")]
            gdpr_processor: Arc::new(Mutex::new(None)),
            #[cfg(feature = "hipaa")]
            hipaa_processor: Arc::new(Mutex::new(None)),
            audit_manager: Arc::new(Mutex::new(AuditTrailManager::new().await?)),
            policy_engine: Arc::new(Mutex::new(PolicyEnforcementEngine::new().await?)),
            risk_engine: Arc::new(Mutex::new(RiskAssessmentEngine::new().await?)),
            data_anonymizer: Arc::new(Mutex::new(DataAnonymizationService::new().await?)),
            consent_manager: Arc::new(Mutex::new(ConsentManager::new().await?)),
            initialized: Arc::new(RwLock::new(false)),
            background_tasks: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(ComplianceMetrics::default())),
        };

        engine.initialize().await?;
        Ok(engine)
    }

    /// Initialize the compliance engine and all processors
    pub async fn initialize(&mut self) -> ComplianceResult<()> {
        let config = self.config.read().await;

        // Initialize GDP processor if enabled
        #[cfg(feature = "gdpr")]
        if config.gdpr_enabled {
            let processor = GdprProcessor::new(self.config.clone()).await?;
            *self.gdpr_processor.lock().await = Some(processor);
        }

        // Initialize HIPAA processor if enabled
        #[cfg(feature = "hipaa")]
        if config.hipaa_enabled {
            let processor = HipaaProcessor::new(self.config.clone()).await?;
            *self.hipaa_processor.lock().await = Some(processor);
        }

        // Initialize audit manager
        self.audit_manager.lock().await.initialize().await?;

        // Initialize policy engine
        let policy_config = config.clone().into();
        self.policy_engine.lock().await.initialize(&policy_config).await?;

        // Initialize risk assessment engine
        self.risk_engine.lock().await.initialize().await?;

        // Initialize data anonymization service
        self.data_anonymizer.lock().await.initialize().await?;

        // Initialize consent manager
        self.consent_manager.lock().await.initialize().await?;

        // Start background compliance monitoring
        self.start_background_monitoring().await?;

        *self.initialized.write().await = true;

        log::info!("Compliance engine initialized successfully");
        Ok(())
    }

    /// Shutdown the compliance engine gracefully
    pub async fn shutdown(&mut self) -> ComplianceResult<()> {
        log::info!("Shutting down compliance engine...");

        // Stop background tasks
        let tasks = std::mem::take(&mut *self.background_tasks.lock().await);
        for task in tasks {
            task.abort();
        }

        // Shutdown processors
        #[cfg(feature = "gdpr")]
        if let Some(processor) = self.gdpr_processor.lock().await.as_mut() {
            processor.shutdown().await?;
        }

        #[cfg(feature = "hipaa")]
        if let Some(processor) = self.hipaa_processor.lock().await.as_mut() {
            processor.shutdown().await?;
        }

        // Shutdown services
        self.audit_manager.lock().await.shutdown().await?;
        self.policy_engine.lock().await.shutdown().await?;
        self.risk_engine.lock().await.shutdown().await?;
        self.data_anonymizer.lock().await.shutdown().await?;
        self.consent_manager.lock().await.shutdown().await?;

        *self.initialized.write().await = false;

        log::info!("Compliance engine shutdown complete");
        Ok(())
    }

    /// Process data for compliance verification
    pub async fn process_data(&self, data: &[u8], context: &DataProcessingContext) -> ComplianceResult<DataProcessingResult> {
        self.ensure_initialized().await?;

        let mut result = DataProcessingResult {
            status: ComplianceStatus::Unknown,
            frameworks_checked: Vec::new(),
            violations: Vec::new(),
            recommendations: Vec::new(),
            audit_entries: Vec::new(),
        };

        // GDPR processing
        #[cfg(feature = "gdpr")]
        if let Some(gdpr) = &*self.gdpr_processor.lock().await {
            let gdpr_result = gdpr.process_data(data, context).await?;
            result.merge_framework_result(gdpr_result, ComplianceFramework::Gdpr);
        }

        // HIPAA processing
        #[cfg(feature = "hipaa")]
        if let Some(hipaa) = &*self.hipaa_processor.lock().await {
            let hipaa_result = hipaa.process_data(data, context).await?;
            result.merge_framework_result(hipaa_result, ComplianceFramework::Hipaa);
        }

        // Risk assessment
        let risk_result = self.risk_engine.lock().await
            .assess_risks(data, context).await?;
        result.add_recommendations(risk_result.recommendations);

        // Update metrics
        self.metrics.lock().await.record_processing(&result);

        // Audit the processing
        self.audit_data_processing(&result, context).await?;

        Ok(result)
    }

    /// Check compliance status for a specific framework
    pub async fn check_compliance_status(&self, framework: &ComplianceFramework) -> ComplianceResult<ComplianceStatus> {
        self.ensure_initialized().await?;

        match framework {
            ComplianceFramework::Gdpr => {
                #[cfg(feature = "gdpr")]
                if let Some(gdpr) = &*self.gdpr_processor.lock().await {
                    return gdpr.check_compliance_status().await;
                }
                #[cfg(not(feature = "gdpr"))]
                return Err(ComplianceError::ConfigurationError {
                    details: "GDPR support not enabled".to_string(),
                    section: Some("features".to_string()),
                });
            }
            ComplianceFramework::Hipaa => {
                #[cfg(feature = "hipaa")]
                if let Some(hipaa) = &*self.hipaa_processor.lock().await {
                    return hipaa.check_compliance_status().await;
                }
                #[cfg(not(feature = "hipaa"))]
                return Err(ComplianceError::ConfigurationError {
                    details: "HIPAA support not enabled".to_string(),
                    section: Some("features".to_string()),
                });
            }
            _ => Ok(ComplianceStatus::Unknown),
        }
    }

    /// Generate compliance report
    pub async fn generate_report(&self, framework: &ComplianceFramework) -> ComplianceResult<ComplianceReport> {
        self.ensure_initialized().await?;

        let mut report = ComplianceReport::new(framework);

        // Collect data from relevant processors
        match framework {
            ComplianceFramework::Gdpr => {
                #[cfg(feature = "gdpr")]
                if let Some(gdpr) = &*self.gdpr_processor.lock().await {
                    let gdpr_data = gdpr.generate_report().await?;
                    report.add_section("gdpr_compliance".to_string(), gdpr_data);
                }
            }
            ComplianceFramework::Hipaa => {
                #[cfg(feature = "hipaa")]
                if let Some(hipaa) = &*self.hipaa_processor.lock().await {
                    let hipaa_data = hipaa.generate_report().await?;
                    report.add_section("hipaa_compliance".to_string(), hipaa_data);
                }
            }
            _ => {}
        }

        // Add risk assessment data
        let risk_data = self.risk_engine.lock().await.generate_risk_report().await?;
        report.add_section("risk_assessment".to_string(), risk_data);

        // Add audit summary
        let audit_data = self.audit_manager.lock().await.generate_audit_summary().await?;
        report.add_section("audit_summary".to_string(), audit_data);

        report.metadata.generated_at = chrono::Utc::now();
        report.metadata.total_entries = report.sections.len();

        Ok(report)
    }

    /// Handle data breach notification
    pub async fn handle_data_breach(&self, breach: &DataBreachNotification) -> ComplianceResult<()> {
        self.ensure_initialized().await?;

        log::error!("Data breach detected: {}", breach.details);

        // Create audit entry for breach
        let audit_entry = AuditEntry::new(AuditSeverity::Critical, "breach", "data_breach_detected")
            .with_resource(&breach.affected_resource)
            .with_details(&breach.details)
            .with_metadata("breach_type".to_string(), breach.breach_type.clone())
            .with_metadata("affected_records".to_string(), breach.affected_records.to_string());

        self.audit_manager.lock().await.log_entry(audit_entry).await?;

        // GDPR breach notification
        #[cfg(feature = "gdpr")]
        if let Some(gdpr) = &*self.gdpr_processor.lock().await {
            gdpr.handle_breach_notification(breach).await?;
        }

        // HIPAA breach notification
        #[cfg(feature = "hipaa")]
        if let Some(hipaa) = &*self.hipaa_processor.lock().await {
            hipaa.handle_breach_notification(breach).await?;
        }

        // Send notifications
        self.send_breach_notifications(breach).await?;

        Ok(())
    }

    /// Update configuration
    pub async fn update_config(&mut self, new_config: ComplianceConfig) -> ComplianceResult<()> {
        *self.config.write().await = new_config;
        self.reinitialize_affected_components().await?;
        Ok(())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> ComplianceResult<ComplianceMetrics> {
        Ok(self.metrics.lock().await.clone())
    }
}

/// Private methods for internal operations
impl ComplianceEngine {
    async fn ensure_initialized(&self) -> ComplianceResult<()> {
        if !*self.initialized.read().await {
            return Err(ComplianceError::ConfigurationError {
                details: "Compliance engine not initialized".to_string(),
                section: Some("lifecycle".to_string()),
            });
        }
        Ok(())
    }

    async fn start_background_monitoring(&mut self) -> ComplianceResult<()> {
        // Start periodic compliance checks
        let compliance_checker = self.spawn_compliance_checker();
        self.background_tasks.lock().await.push(compliance_checker);

        // Start audit log maintenance
        let audit_maintenance = self.spawn_audit_maintenance();
        self.background_tasks.lock().await.push(audit_maintenance);

        log::info!("Background compliance monitoring started");
        Ok(())
    }

    fn spawn_compliance_checker(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        #[cfg(feature = "gdpr")]
        let gdpr_processor = self.gdpr_processor.clone();
        #[cfg(feature = "hipaa")]
        let hipaa_processor = self.hipaa_processor.clone();
        let audit_manager = self.audit_manager.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await; // Check every hour

                // Perform periodic compliance checks
                let _ = Self::perform_periodic_checks(
                    &config,
                    #[cfg(feature = "gdpr")]
                    &gdpr_processor,
                    #[cfg(feature = "hipaa")]
                    &hipaa_processor,
                    &audit_manager,
                ).await;
            }
        })
    }

    fn spawn_audit_maintenance(&self) -> tokio::task::JoinHandle<()> {
        let audit_manager = self.audit_manager.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await; // Daily cleanup

                // Clean up old audit logs
                let retention_days = config.read().await.audit.retention_days;
                if let Err(e) = audit_manager.lock().await.cleanup_old_entries(retention_days).await {
                    log::error!("Failed to cleanup old audit entries: {}", e);
                }
            }
        })
    }

    async fn perform_periodic_checks(
        _config: &Arc<RwLock<ComplianceConfig>>,
        #[cfg(feature = "gdpr")]
        _gdpr_processor: &Arc<Mutex<Option<GdprProcessor>>>,
        #[cfg(feature = "hipaa")]
        _hipaa_processor: &Arc<Mutex<Option<HipaaProcessor>>>,
        _audit_manager: &Arc<Mutex<AuditTrailManager>>,
    ) -> ComplianceResult<()> {
        // Implementation for periodic compliance checks
        Ok(())
    }

    async fn audit_data_processing(&self, result: &DataProcessingResult, context: &DataProcessingContext) -> ComplianceResult<()> {
        let audit_entry = AuditEntry::new(
            match result.status {
                ComplianceStatus::Compliant => AuditSeverity::Info,
                ComplianceStatus::Partial => AuditSeverity::Warning,
                ComplianceStatus::NonCompliant => AuditSeverity::Error,
                ComplianceStatus::Unknown => AuditSeverity::Info,
            },
            "data_processing",
            "compliance_check"
        )
        .with_resource(&context.resource_id)
        .with_user_id(context.user_id.clone())
        .with_metadata("frameworks_checked".to_string(),
            serde_json::to_string(&result.frameworks_checked).unwrap_or_default())
        .with_metadata("violations_count".to_string(), result.violations.len().to_string())
        .with_details(&format!("Processed {} frameworks, {} violations found",
            result.frameworks_checked.len(), result.violations.len()));

        self.audit_manager.lock().await.log_entry(audit_entry).await?;
        Ok(())
    }

    async fn send_breach_notifications(&self, breach: &DataBreachNotification) -> ComplianceResult<()> {
        // Implementation for sending breach notifications to authorities and affected parties
        log::warn!("Sending breach notifications for: {}", breach.details);
        Ok(())
    }

    async fn reinitialize_affected_components(&mut self) -> ComplianceResult<()> {
        // Reinitialize components affected by configuration changes
        log::info!("Reinitializing compliance components after config update");
        Ok(())
    }
}

/// Data processing context for compliance evaluation
#[derive(Debug, Clone)]
pub struct DataProcessingContext {
    pub resource_id: String,
    pub user_id: Option<String>,
    pub operation_type: String,
    pub location: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Default for DataProcessingContext {
    fn default() -> Self {
        Self {
            resource_id: "unknown".to_string(),
            user_id: None,
            operation_type: "unknown".to_string(),
            location: "unknown".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

/// Data processing result with compliance information
#[derive(Debug, Clone)]
pub struct DataProcessingResult {
    pub status: ComplianceStatus,
    pub frameworks_checked: Vec<ComplianceFramework>,
    pub violations: Vec<ComplianceViolation>,
    pub recommendations: Vec<String>,
    pub audit_entries: Vec<AuditEntry>,
}

impl DataProcessingResult {
    pub fn merge_framework_result(&mut self, result: FrameworkProcessingResult, framework: ComplianceFramework) {
        self.frameworks_checked.push(framework);
        self.violations.extend(result.violations);
        self.recommendations.extend(result.recommendations);
        self.status = self.status.merge(result.status);
    }

    pub fn add_recommendations(&mut self, recommendations: Vec<String>) {
        self.recommendations.extend(recommendations);
    }
}

/// Framework-specific processing result
#[derive(Debug, Clone)]
pub struct FrameworkProcessingResult {
    pub status: ComplianceStatus,
    pub violations: Vec<ComplianceViolation>,
    pub recommendations: Vec<String>,
}

/// Compliance violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub framework: ComplianceFramework,
    pub severity: AuditSeverity,
    pub violation_type: String,
    pub description: String,
    pub remediation: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Data breach notification structure
#[derive(Debug, Clone)]
pub struct DataBreachNotification {
    pub breach_id: String,
    pub breach_type: String,
    pub affected_resource: String,
    pub affected_records: usize,
    pub details: String,
    pub detection_timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: AuditSeverity,
}

/// Compliance report structure
#[derive(Debug, Clone)]
pub struct ComplianceReport {
    pub framework: ComplianceFramework,
    pub metadata: ReportMetadata,
    pub sections: HashMap<String, serde_json::Value>,
}

impl ComplianceReport {
    pub fn new(framework: &ComplianceFramework) -> Self {
        Self {
            framework: framework.clone(),
            metadata: ReportMetadata::new(),
            sections: HashMap::new(),
        }
    }

    pub fn add_section(&mut self, key: String, value: serde_json::Value) {
        self.sections.insert(key, value);
    }
}

/// Report metadata
#[derive(Debug, Clone)]
pub struct ReportMetadata {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub total_entries: usize,
    pub period_start: Option<chrono::DateTime<chrono::Utc>>,
    pub period_end: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ReportMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportMetadata {
    pub fn new() -> Self {
        Self {
            generated_at: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            total_entries: 0,
            period_start: None,
            period_end: None,
        }
    }
}

/// Compliance metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ComplianceMetrics {
    pub total_processings: u64,
    pub compliant_processings: u64,
    pub partial_compliant_processings: u64,
    pub non_compliant_processings: u64,
    pub violations_detected: u64,
    pub breach_notifications: u64,
    pub reports_generated: u64,
    pub average_processing_time_ms: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl ComplianceMetrics {
    pub fn record_processing(&mut self, result: &DataProcessingResult) {
        self.total_processings += 1;
        match result.status {
            ComplianceStatus::Compliant => self.compliant_processings += 1,
            ComplianceStatus::Partial => self.partial_compliant_processings += 1,
            ComplianceStatus::NonCompliant => self.non_compliant_processings += 1,
            ComplianceStatus::Unknown => {}
        }
        self.violations_detected += result.violations.len() as u64;
        self.last_updated = chrono::Utc::now();
    }

    pub fn record_breach_notification(&mut self) {
        self.breach_notifications += 1;
        self.last_updated = chrono::Utc::now();
    }

    pub fn record_report_generation(&mut self) {
        self.reports_generated += 1;
        self.last_updated = chrono::Utc::now();
    }
}

/// Utility traits for merging compliance statuses
trait MergeComplianceStatus {
    fn merge(self, other: Self) -> Self;
}

impl MergeComplianceStatus for ComplianceStatus {
    fn merge(self, other: Self) -> Self {
        use ComplianceStatus::*;
        match (self, other) {
            (Compliant, Compliant) => Compliant,
            (NonCompliant, _) | (_, NonCompliant) => NonCompliant,
            (Partial, _) | (_, Partial) => Partial,
            (Unknown, status) | (status, Unknown) => status,
        }
    }
}

/// Service trait for compliance processors
#[async_trait]
pub trait ComplianceProcessor: Send + Sync {
    /// Process data for compliance
    async fn process_data(&self, data: &[u8], context: &DataProcessingContext) -> ComplianceResult<FrameworkProcessingResult>;

    /// Check compliance status
    async fn check_compliance_status(&self) -> ComplianceResult<ComplianceStatus>;

    /// Generate compliance report section
    async fn generate_report(&self) -> ComplianceResult<serde_json::Value>;

    /// Handle breach notifications
    async fn handle_breach_notification(&self, breach: &DataBreachNotification) -> ComplianceResult<()>;

    /// Shutdown processor
    async fn shutdown(&self) -> ComplianceResult<()>;
}