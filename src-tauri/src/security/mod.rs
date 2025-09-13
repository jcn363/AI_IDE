//! Main security module coordinating all security infrastructure components
//!
//! This module provides a centralized SecurityManager that coordinates encryption,
//! supply chain security, secrets detection, SIEM integration, and security policy enforcement.
//! All components follow strict security programming rules including no plain text secrets,
//! path traversal protection, audit logging, and compliance with cargo-deny bans.

use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::errors::{IDEError, IDEResult};
use crate::command_templates::{CommandConfig, execute_command, spawn_background_task};

use encryption::{GLOBAL_ENCRYPTION_MANAGER, EncryptionManager};
use secrets_detection::GLOBAL_SECRETS_DETECTOR;
use siem_integration::{GLOBAL_SIEM_INTEGRATION, SecurityEvent, EventType, Severity as SIEMSeverity};
use supply_chain::GLOBAL_SCANNER;

// Core security modules
pub mod vulnerability_scanner;
pub mod rustsec_integration;
pub mod ai_security_analyzer;
pub mod owasp_scanner;
pub mod file_security;

// New security modules for enhanced security infrastructure
pub mod encryption;
pub mod supply_chain;
pub mod secrets_detection;
pub mod siem_integration;

/// Security configuration for the main security manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityManagerConfig {
    /// Enable comprehensive security monitoring
    pub enable_monitoring: bool,
    /// Enable automatic security policy enforcement
    pub enable_policy_enforcement: bool,
    /// Maximum security scan concurrency
    pub max_concurrent_scans: usize,
    /// Security event retention period in hours
    pub event_retention_hours: u64,
    /// Enable audit logging for all security operations
    pub enable_audit_logging: bool,
}

impl Default for SecurityManagerConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            enable_policy_enforcement: true,
            max_concurrent_scans: 4,
            event_retention_hours: 168, // 7 days
            enable_audit_logging: true,
        }
    }
}

/// Main security manager coordinating all security components
pub struct SecurityManager {
    /// Configuration for security operations
    config: SecurityManagerConfig,
    /// Security initialization status
    initialized: bool,
    /// Security policy cache
    policy_cache: HashMap<String, SecurityPolicy>,
}

/// Security policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub description: String,
    pub rules: Vec<SecurityRule>,
    pub severity: SecuritySeverity,
}

/// Security rule for policy enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub name: String,
    pub condition: String,
    pub action: SecurityAction,
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Security action to take when rule is violated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    Allow,
    Deny,
    Log,
    Alert,
    Block,
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new(SecurityManagerConfig::default())
    }
}

impl SecurityManager {
    /// Create a new security manager with the given configuration
    pub fn new(config: SecurityManagerConfig) -> Self {
        Self {
            config,
            initialized: false,
            policy_cache: HashMap::new(),
        }
    }

    /// Initialize all security services and components
    pub async fn initialize(&mut self) -> IDEResult<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("Initializing security infrastructure...");

        // Initialize encryption manager
        self.initialize_encryption().await
            .map_err(|e| IDEError::Security(format!("Failed to initialize encryption: {}", e)))?;

        // Initialize secrets detection
        self.initialize_secrets_detection().await
            .map_err(|e| IDEError::Security(format!("Failed to initialize secrets detection: {}", e)))?;

        // Initialize SIEM integration
        self.initialize_siem_integration().await
            .map_err(|e| IDEError::Security(format!("Failed to initialize SIEM: {}", e)))?;

        // Initialize supply chain security
        self.initialize_supply_chain_security().await
            .map_err(|e| IDEError::Security(format!("Failed to initialize supply chain security: {}", e)))?;

        // Load and enforce security policies
        self.load_security_policies().await
            .map_err(|e| IDEError::Security(format!("Failed to load security policies: {}", e)))?;

        self.initialized = true;

        // Log successful initialization
        self.audit_log("security_initialization", "Security infrastructure initialized successfully", SIEMSeverity::Low).await;

        log::info!("Security infrastructure initialized successfully");
        Ok(())
    }

    /// Initialize encryption manager
    async fn initialize_encryption(&self) -> Result<(), String> {
        // Encryption manager is already initialized via lazy_static
        log::info!("Encryption manager ready");
        Ok(())
    }

    /// Initialize secrets detection engine
    async fn initialize_secrets_detection(&self) -> Result<(), String> {
        // Secrets detector is already initialized via lazy_static
        log::info!("Secrets detection engine ready");
        Ok(())
    }

    /// Initialize SIEM integration
    async fn initialize_siem_integration(&self) -> Result<(), String> {
        use siem_integration::{init_global_siem, SIEMConfig, AlertThresholds};

        let siem_config = SIEMConfig {
            enable_realtime: self.config.enable_monitoring,
            max_events_buffer: 10000,
            retention_hours: self.config.event_retention_hours,
            anomaly_sensitivity: 0.8,
            alert_thresholds: AlertThresholds {
                failed_login_attempts: 5,
                suspicious_file_access: 10,
                high_entropy_detections: 3,
                vulnerability_findings: 5,
                compliance_violations: 2,
            },
            external_endpoints: Vec::new(),
        };

        init_global_siem(siem_config).await?;
        log::info!("SIEM integration initialized");
        Ok(())
    }

    /// Initialize supply chain security
    async fn initialize_supply_chain_security(&self) -> Result<(), String> {
        // Supply chain scanner is initialized via lazy_static
        log::info!("Supply chain security ready");
        Ok(())
    }

    /// Load security policies from configuration
    async fn load_security_policies(&mut self) -> Result<(), String> {
        // Default security policies - in real implementation, these would be loaded from config files
        let policies = vec![
            SecurityPolicy {
                name: "path_traversal_protection".to_string(),
                description: "Prevent path traversal attacks".to_string(),
                rules: vec![
                    SecurityRule {
                        name: "validate_file_paths".to_string(),
                        condition: "path_contains_parent_directory".to_string(),
                        action: SecurityAction::Deny,
                    },
                ],
                severity: SecuritySeverity::High,
            },
            SecurityPolicy {
                name: "secrets_detection".to_string(),
                description: "Detect and prevent secrets exposure".to_string(),
                rules: vec![
                    SecurityRule {
                        name: "scan_for_secrets".to_string(),
                        condition: "file_contains_potential_secret".to_string(),
                        action: SecurityAction::Alert,
                    },
                ],
                severity: SecuritySeverity::Critical,
            },
        ];

        for policy in policies {
            self.policy_cache.insert(policy.name.clone(), policy);
        }

        log::info!("Loaded {} security policies", self.policy_cache.len());
        Ok(())
    }

    /// Validate a file path for security (path traversal protection)
    pub fn validate_secure_path(&self, path: &Path) -> IDEResult<()> {
        use rust_ai_ide_common::validation::validate_secure_path;

        match validate_secure_path(path) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.audit_log_sync("path_validation_failed",
                    &format!("Path validation failed for: {}", path.display()),
                    SIEMSeverity::High);
                Err(IDEError::PathValidation(e.to_string()))
            }
        }
    }

    /// Perform comprehensive security scan on a directory
    pub async fn perform_security_scan(&self, directory: &Path) -> IDEResult<SecurityScanResult> {
        self.validate_secure_path(directory)?;

        log::info!("Starting comprehensive security scan on: {}", directory.display());

        let start_time = std::time::Instant::now();

        // Run secrets detection
        let secrets_result = self.scan_for_secrets(directory).await?;

        // Run vulnerability scanning
        let vuln_result = self.scan_for_vulnerabilities(directory).await?;

        // Run supply chain analysis
        let supply_chain_result = self.analyze_supply_chain(directory).await?;

        let duration = start_time.elapsed();

        let result = SecurityScanResult {
            directory: directory.to_path_buf(),
            secrets_findings: secrets_result,
            vulnerability_findings: vuln_result,
            supply_chain_findings: supply_chain_result,
            scan_duration_ms: duration.as_millis() as u64,
            timestamp: chrono::Utc::now(),
        };

        // Audit log the scan completion
        self.audit_log("security_scan_completed",
            &format!("Security scan completed for: {}", directory.display()),
            SIEMSeverity::Low).await;

        Ok(result)
    }

    /// Scan for secrets in the given directory
    async fn scan_for_secrets(&self, directory: &Path) -> IDEResult<secrets_detection::SecretsScanResult> {
        let detector = GLOBAL_SECRETS_DETECTOR.lock().await;
        match detector.scan_directory(directory).await {
            Ok(result) => Ok(result),
            Err(e) => {
                self.audit_log("secrets_scan_failed",
                    &format!("Secrets scan failed: {}", e),
                    SIEMSeverity::Medium).await;
                Err(IDEError::Security(format!("Secrets scan failed: {}", e)))
            }
        }
    }

    /// Scan for vulnerabilities in the given directory
    async fn scan_for_vulnerabilities(&self, directory: &Path) -> IDEResult<Vec<String>> {
        // Use rustsec integration for vulnerability scanning
        match rustsec_integration::RustsecScanner::new().scan_directory(directory).await {
            Ok(vulns) => Ok(vulns.into_iter().map(|v| v.package_name).collect()),
            Err(e) => {
                self.audit_log("vulnerability_scan_failed",
                    &format!("Vulnerability scan failed: {}", e),
                    SIEMSeverity::Medium).await;
                Err(IDEError::Security(format!("Vulnerability scan failed: {}", e)))
            }
        }
    }

    /// Analyze supply chain security
    async fn analyze_supply_chain(&self, directory: &Path) -> IDEResult<Vec<String>> {
        // Supply chain analysis would be implemented here
        // For now, return empty results
        Ok(Vec::new())
    }

    /// Encrypt data using the global encryption manager
    pub async fn encrypt_data(&self, data: &[u8]) -> Result<encryption::EncryptedPayload, String> {
        let manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.encrypt_payload(data)
    }

    /// Decrypt data using the global encryption manager
    pub async fn decrypt_data(&self, payload: &encryption::EncryptedPayload) -> Result<Vec<u8>, String> {
        let manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.decrypt_payload(payload)
    }

    /// Encrypt IPC payload for secure communication
    pub async fn encrypt_ipc_payload(&self, payload: &serde_json::Value) -> Result<String, String> {
        let manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.encrypt_ipc_payload(payload)
    }

    /// Decrypt IPC payload from secure communication
    pub async fn decrypt_ipc_payload(&self, encrypted_json: &str) -> Result<serde_json::Value, String> {
        let manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.decrypt_ipc_payload(encrypted_json)
    }

    /// Audit log a security event (async version)
    pub async fn audit_log(&self, event_type: &str, message: &str, severity: SIEMSeverity) {
        if !self.config.enable_audit_logging {
            return;
        }

        let event = SecurityEvent {
            id: format!("audit_{}", uuid::Uuid::new_v4()),
            timestamp: chrono::Utc::now(),
            event_type: EventType::SystemHealth, // Use appropriate event type
            severity,
            source: "security_manager".to_string(),
            user: None,
            resource: Some(event_type.to_string()),
            details: serde_json::json!({
                "message": message,
                "component": "security_manager"
            }),
            correlation_id: None,
            raw_data: None,
        };

        if let Err(e) = siem_integration::process_security_event(event).await {
            log::error!("Failed to audit log event: {}", e);
        }
    }

    /// Audit log a security event (synchronous version for use in non-async contexts)
    pub fn audit_log_sync(&self, event_type: &str, message: &str, severity: SIEMSeverity) {
        if !self.config.enable_audit_logging {
            return;
        }

        // For sync contexts, we'll spawn a background task
        let event_type = event_type.to_string();
        let message = message.to_string();
        let severity = severity;

        spawn_background_task(async move {
            let event = SecurityEvent {
                id: format!("audit_{}", uuid::Uuid::new_v4()),
                timestamp: chrono::Utc::now(),
                event_type: EventType::SystemHealth,
                severity,
                source: "security_manager".to_string(),
                user: None,
                resource: Some(event_type.clone()),
                details: serde_json::json!({
                    "message": message,
                    "component": "security_manager"
                }),
                correlation_id: None,
                raw_data: None,
            };

            if let Err(e) = siem_integration::process_security_event(event).await {
                log::error!("Failed to audit log event: {}", e);
            }
        }, "audit_log_sync");
    }

    /// Get security status information
    pub async fn get_security_status(&self) -> IDEResult<SecurityStatus> {
        let encryption_status = self.check_encryption_status().await;
        let siem_status = self.check_siem_status().await;
        let secrets_status = self.check_secrets_status().await;

        Ok(SecurityStatus {
            initialized: self.initialized,
            encryption_ready: encryption_status.is_ok(),
            siem_ready: siem_status.is_ok(),
            secrets_detection_ready: secrets_status.is_ok(),
            active_policies: self.policy_cache.len(),
            last_scan_time: None, // Would be tracked in real implementation
        })
    }

    /// Check encryption manager status
    async fn check_encryption_status(&self) -> Result<(), String> {
        let manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        // Test basic encryption functionality
        let test_data = b"test";
        let encrypted = manager.encrypt_payload(test_data)?;
        let decrypted = manager.decrypt_payload(&encrypted)?;
        if decrypted == test_data {
            Ok(())
        } else {
            Err("Encryption/decryption test failed".to_string())
        }
    }

    /// Check SIEM integration status
    async fn check_siem_status(&self) -> Result<(), String> {
        let siem = GLOBAL_SIEM_INTEGRATION.lock().await;
        if siem.is_some() {
            Ok(())
        } else {
            Err("SIEM not initialized".to_string())
        }
    }

    /// Check secrets detection status
    async fn check_secrets_status(&self) -> Result<(), String> {
        // Secrets detector is always ready via lazy_static
        Ok(())
    }

    /// Rotate encryption keys for forward secrecy
    pub async fn rotate_encryption_keys(&self) -> Result<(), String> {
        let mut manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.rotate_session_key()?;

        self.audit_log("key_rotation", "Encryption keys rotated successfully", SIEMSeverity::Low).await;
        log::info!("Encryption keys rotated successfully");
        Ok(())
    }
}

/// Security scan result combining all security checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    pub directory: std::path::PathBuf,
    pub secrets_findings: secrets_detection::SecretsScanResult,
    pub vulnerability_findings: Vec<String>,
    pub supply_chain_findings: Vec<String>,
    pub scan_duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Security status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
    pub initialized: bool,
    pub encryption_ready: bool,
    pub siem_ready: bool,
    pub secrets_detection_ready: bool,
    pub active_policies: usize,
    pub last_scan_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Global security manager instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_SECURITY_MANAGER: Arc<Mutex<SecurityManager>> =
        Arc::new(Mutex::new(SecurityManager::new(SecurityManagerConfig::default())));
}

// Re-exports for easier access
pub use vulnerability_scanner::{VulnerabilityScanner, VulnerabilityReport as VulnerabilityReportSimple};
pub use rustsec_integration::{RustsecScanner, VulnerabilityReport};
pub use ai_security_analyzer::{
    AISecurityAnalyzer,
    SecurityAnalysisResult,
    SecurityIssue,
    SecuritySeverity,
    analyze_code_security,
    integrate_with_rustsec
};

// OWASP Top 10 Scanner re-exports
pub use owasp_scanner::{
    OWASPScanner,
    OWASPCategory,
    OWASPVulnerability,
    OWASPScanResult,
    OWASPSummary,
};

// Supply Chain Security (Legacy - now enhanced)
pub use owasp_scanner::supply_chain::{
    SupplyChainScanner,
    DependencyAnalysis,
    LicenseCompliance,
    MalwareDetection,
};

// New enhanced security modules
pub use encryption::{EncryptionManager, EncryptedPayload, SessionKey, GLOBAL_ENCRYPTION_MANAGER};
pub use supply_chain::{
    SupplyChainScanner, SupplyChainScanResult, SupplyChainConfig,
    Vulnerability, LicenseIssue, MalwareDetection,
    SBOM, init_global_scanner, get_global_scanner,
};
pub use secrets_detection::{
    SecretsDetector, SecretsScanResult, SecretFinding, SecretType, Severity,
    GLOBAL_SECRETS_DETECTOR,
};
pub use siem_integration::{
    SIEMIntegration, SIEMConfig, SecurityEvent, EventType, Severity as SIEMSeverity,
    ComplianceReport, Alert, GLOBAL_SIEM_INTEGRATION,
    init_global_siem, process_security_event,
};

/// Initialize the global security manager
pub async fn init_global_security_manager(config: SecurityManagerConfig) -> IDEResult<()> {
    let mut manager = GLOBAL_SECURITY_MANAGER.lock().await;
    *manager = SecurityManager::new(config);
    manager.initialize().await
}

/// Get the global security manager instance
pub fn get_global_security_manager() -> Arc<Mutex<SecurityManager>> {
    Arc::clone(&GLOBAL_SECURITY_MANAGER)
}

// Import HashMap for the policy cache
use std::collections::HashMap;

// Import UUID for audit logging
use uuid;
