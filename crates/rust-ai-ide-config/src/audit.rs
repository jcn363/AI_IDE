//! Audit trail system with encrypted logging and change tracking
//!
//! Provides secure, tamper-evident logging of all configuration operations
//! with encryption, hashing, and change tracking capabilities.

use std::collections::HashMap;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Encrypted audit trail for configuration changes
#[derive(Debug)]
pub struct AuditTrail {
    /// Encrypted audit log entries
    entries:        RwLock<Vec<EncryptedEntry>>,
    /// Encryption key (derived from system secrets)
    encryption_key: Key<Aes256Gcm>,
    /// Audit configuration
    config:         AuditConfig,
    /// Hash of last entry for tamper detection
    last_hash:      RwLock<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled:            bool,
    /// Maximum number of audit entries to keep
    pub max_entries:        usize,
    /// Encryption enabled
    pub encryption_enabled: bool,
    /// Hash algorithm for tamper detection
    pub hash_algorithm:     HashAlgorithm,
    /// Audit log retention period (days)
    pub retention_days:     u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled:            true,
            max_entries:        10000,
            encryption_enabled: true,
            hash_algorithm:     HashAlgorithm::Sha256,
            retention_days:     90,
        }
    }
}

/// Hash algorithm for tamper detection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

/// Encrypted audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEntry {
    /// Encryption nonce
    pub nonce:          Vec<u8>,
    /// Encrypted audit data
    pub encrypted_data: Vec<u8>,
    /// Timestamp
    pub timestamp:      DateTime<Utc>,
    /// Hash of this entry (for chain integrity)
    pub entry_hash:     String,
}

/// Clear-text audit event data (before encryption)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event type
    pub event_type:       AuditEventType,
    /// Configuration name
    pub config_name:      String,
    /// Source of the change
    pub source:           String,
    /// User/system identifier
    pub actor:            String,
    /// Timestamp of event
    pub timestamp:        DateTime<Utc>,
    /// Change details
    pub changes:          Vec<ConfigChange>,
    /// Security context
    pub security_context: SecurityContext,
    /// Event metadata
    pub metadata:         HashMap<String, String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Configuration loaded
    ConfigLoad,
    /// Configuration saved
    ConfigSave,
    /// Configuration validated
    ConfigValidation,
    /// Security violation detected
    SecurityViolation,
    /// Hot reload triggered
    HotReload,
    /// Cache operation
    CacheOperation,
    /// Migration performed
    Migration,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigLoad => write!(f, "ConfigLoad"),
            Self::ConfigSave => write!(f, "ConfigSave"),
            Self::ConfigValidation => write!(f, "ConfigValidation"),
            Self::SecurityViolation => write!(f, "SecurityViolation"),
            Self::HotReload => write!(f, "HotReload"),
            Self::CacheOperation => write!(f, "CacheOperation"),
            Self::Migration => write!(f, "Migration"),
        }
    }
}

/// Configuration change details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    /// Field path that changed
    pub field_path:  String,
    /// Previous value (sanitized)
    pub old_value:   Option<String>,
    /// New value (sanitized)
    pub new_value:   Option<String>,
    /// Change type
    pub change_type: ChangeType,
}

/// Types of configuration changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// Field added
    Added,
    /// Field modified
    Modified,
    /// Field removed
    Removed,
    /// Field unchanged (for validation events)
    Unchanged,
}

/// Security context for audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Authentication method
    pub auth_method:  String,
    /// User roles/permissions
    pub roles:        Vec<String>,
    /// IP address (when applicable)
    pub ip_address:   Option<String>,
    /// Session ID
    pub session_id:   Option<String>,
    /// Threat level assessment
    pub threat_level: Option<i32>,
}

impl AuditTrail {
    /// Create new audit trail
    pub async fn new() -> crate::IDEResult<Self> {
        Self::new_with_config(AuditConfig::default()).await
    }

    /// Create audit trail with custom configuration
    pub async fn new_with_config(config: AuditConfig) -> crate::IDEResult<Self> {
        // Generate encryption key from system entropy
        // In production, this would use a proper KMS or key derivation
        let mut key_bytes = [0u8; 32];
        ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(), &mut key_bytes).map_err(|_| {
            crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(
                "Failed to generate encryption key",
            ))
        })?;
        let encryption_key = Key::<Aes256Gcm>::from_slice(&key_bytes);

        let trail = Self {
            entries: RwLock::new(Vec::new()),
            encryption_key: *encryption_key,
            config,
            last_hash: RwLock::new("genesis".to_string()),
        };

        // Initialize with genesis entry
        trail.record_genesis().await?;

        Ok(trail)
    }

    /// Create disabled audit trail
    pub fn disabled() -> Self {
        Self {
            entries:        RwLock::new(Vec::new()),
            encryption_key: *Key::<Aes256Gcm>::from_slice(&[0u8; 32]),
            config:         AuditConfig {
                enabled:            false,
                max_entries:        0,
                encryption_enabled: false,
                hash_algorithm:     HashAlgorithm::Sha256,
                retention_days:     0,
            },
            last_hash:      RwLock::new("disabled".to_string()),
        }
    }

    /// Record a configuration load event
    pub async fn record_load(&self, config_name: &str, source_count: usize, source: &str) -> crate::IDEResult<()> {
        let event = AuditEvent {
            event_type:       AuditEventType::ConfigLoad,
            config_name:      config_name.to_string(),
            source:           source.to_string(),
            actor:            self.get_current_actor().await,
            timestamp:        Utc::now(),
            changes:          vec![],
            security_context: self.get_security_context().await,
            metadata:         {
                let mut meta = HashMap::new();
                meta.insert("source_count".to_string(), source_count.to_string());
                meta
            },
        };

        self.record_event(event).await
    }

    /// Record a configuration save event
    pub async fn record_save(&self, config_name: &str, source: &str) -> crate::IDEResult<()> {
        let event = AuditEvent {
            event_type:       AuditEventType::ConfigSave,
            config_name:      config_name.to_string(),
            source:           source.to_string(),
            actor:            self.get_current_actor().await,
            timestamp:        Utc::now(),
            changes:          vec![],
            security_context: self.get_security_context().await,
            metadata:         HashMap::new(),
        };

        self.record_event(event).await
    }

    /// Record a security violation
    pub async fn record_security_violation(
        &self,
        violation: crate::SecurityViolation,
        field: &str,
        message: &str,
    ) -> crate::IDEResult<()> {
        let event = AuditEvent {
            event_type:       AuditEventType::SecurityViolation,
            config_name:      "security".to_string(),
            source:           "security_validator".to_string(),
            actor:            self.get_current_actor().await,
            timestamp:        Utc::now(),
            changes:          vec![],
            security_context: self.get_security_context().await,
            metadata:         {
                let mut meta = HashMap::new();
                meta.insert("violation_type".to_string(), format!("{:?}", violation));
                meta.insert("field".to_string(), field.to_string());
                meta.insert("message".to_string(), message.to_string());
                meta
            },
        };

        self.record_event(event).await
    }

    /// Record configuration changes
    pub async fn record_changes(&self, config_name: &str, changes: Vec<ConfigChange>) -> crate::IDEResult<()> {
        let event = AuditEvent {
            event_type: AuditEventType::ConfigValidation,
            config_name: config_name.to_string(),
            source: "change_tracker".to_string(),
            actor: self.get_current_actor().await,
            timestamp: Utc::now(),
            changes,
            security_context: self.get_security_context().await,
            metadata: HashMap::new(),
        };

        self.record_event(event).await
    }

    /// Record generic audit event
    pub async fn record_event(&self, event: AuditEvent) -> crate::IDEResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Serialize event data
        let event_data = serde_json::to_vec(&event)
            .map_err(|e| crate::RustAIError::Serialization(format!("Failed to serialize audit event: {}", e)))?;

        // Generate nonce
        let mut nonce_bytes = [0u8; 12];
        ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(), &mut nonce_bytes).map_err(|_| {
            crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(
                "Failed to generate nonce",
            ))
        })?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt data
        let cipher = Aes256Gcm::new(&self.encryption_key);
        let encrypted_data = cipher.encrypt(nonce, event_data.as_slice()).map_err(|_| {
            crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(
                "Failed to encrypt audit data",
            ))
        })?;

        // Generate entry hash for tamper detection
        let last_hash = self.last_hash.read().await.clone();
        let entry_hash = self.generate_entry_hash(&encrypted_data, &nonce_bytes, &last_hash);

        // Create encrypted entry
        let entry = EncryptedEntry {
            nonce: nonce_bytes.to_vec(),
            encrypted_data,
            timestamp: Utc::now(),
            entry_hash: entry_hash.clone(),
        };

        // Store entry
        {
            let mut entries = self.entries.write().await;
            entries.push(entry);

            // Maintain max entries limit
            if entries.len() > self.config.max_entries {
                let to_remove = entries.len() - self.config.max_entries;
                entries.drain(0..to_remove);
            }
        }

        // Update last hash
        *self.last_hash.write().await = entry_hash;

        tracing::debug!(
            "Recorded audit event: {} for config {}",
            event.event_type,
            event.config_name
        );
        Ok(())
    }

    /// Verify audit trail integrity
    pub async fn verify_integrity(&self) -> crate::IDEResult<bool> {
        let entries = self.entries.read().await;
        let mut current_hash = "genesis".to_string();

        for entry in entries.iter() {
            let computed_hash = self.generate_entry_hash(&entry.encrypted_data, &entry.nonce, &current_hash);

            if computed_hash != entry.entry_hash {
                tracing::error!(
                    "Audit trail integrity violation detected at {}",
                    entry.timestamp
                );
                return Ok(false);
            }

            current_hash = computed_hash;
        }

        Ok(true)
    }

    /// Get audit statistics
    pub async fn get_stats(&self) -> AuditStats {
        let entries = self.entries.read().await;
        let integrity_ok = self.verify_integrity().await.unwrap_or(false);

        AuditStats {
            total_entries:      entries.len(),
            oldest_entry:       entries.first().map(|e| e.timestamp),
            newest_entry:       entries.last().map(|e| e.timestamp),
            integrity_verified: integrity_ok,
        }
    }

    /// Search audit events
    pub async fn search_events(&self, filter: AuditSearchFilter) -> crate::IDEResult<Vec<AuditEvent>> {
        let entries = self.entries.read().await;
        let mut results = Vec::new();

        let cipher = Aes256Gcm::new(&self.encryption_key);

        for entry in entries.iter() {
            let nonce = Nonce::from_slice(&entry.nonce);
            let decrypted_data = cipher
                .decrypt(nonce, entry.encrypted_data.as_slice())
                .map_err(|_| {
                    crate::RustAIError::Config(rust_ai_ide_errors::ConfigError::new(
                        "Failed to decrypt audit data",
                    ))
                })?;

            let event: AuditEvent = serde_json::from_slice(&decrypted_data)
                .map_err(|e| crate::RustAIError::Serialization(format!("Failed to deserialize audit event: {}", e)))?;

            if filter.matches(&event) {
                results.push(event);
            }
        }

        Ok(results)
    }

    /// Clean up old audit entries
    pub async fn cleanup_old_entries(&self) -> crate::IDEResult<usize> {
        let mut entries = self.entries.write().await;
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        let initial_count = entries.len();
        entries.retain(|entry| entry.timestamp > cutoff);

        let removed = initial_count - entries.len();
        Ok(removed)
    }

    // Helper methods

    async fn record_genesis(&self) -> crate::IDEResult<()> {
        let genesis_event = AuditEvent {
            event_type:       AuditEventType::ConfigLoad,
            config_name:      "system".to_string(),
            source:           "genesis".to_string(),
            actor:            "system".to_string(),
            timestamp:        Utc::now(),
            changes:          vec![],
            security_context: SecurityContext {
                auth_method:  "system".to_string(),
                roles:        vec!["system".to_string()],
                ip_address:   None,
                session_id:   None,
                threat_level: None,
            },
            metadata:         {
                let mut meta = HashMap::new();
                meta.insert("genesis".to_string(), "true".to_string());
                meta
            },
        };

        self.record_event(genesis_event).await
    }

    async fn get_current_actor(&self) -> String {
        // In production, this would get the actual user/session
        "system".to_string()
    }

    async fn get_security_context(&self) -> SecurityContext {
        // In production, this would get actual security context
        SecurityContext {
            auth_method:  "system".to_string(),
            roles:        vec!["system".to_string()],
            ip_address:   None,
            session_id:   None,
            threat_level: None,
        }
    }

    fn generate_entry_hash(&self, encrypted_data: &[u8], nonce: &[u8], prev_hash: &str) -> String {
        let mut combined = Vec::new();
        combined.extend_from_slice(encrypted_data);
        combined.extend_from_slice(nonce);
        combined.extend_from_slice(prev_hash.as_bytes());

        ring::digest::digest(&ring::digest::SHA256, &combined)
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

/// Audit search filter
#[derive(Debug, Clone)]
pub struct AuditSearchFilter {
    pub config_name: Option<String>,
    pub event_type:  Option<AuditEventType>,
    pub actor:       Option<String>,
    pub start_time:  Option<DateTime<Utc>>,
    pub end_time:    Option<DateTime<Utc>>,
}

impl AuditSearchFilter {
    pub fn new() -> Self {
        Self {
            config_name: None,
            event_type:  None,
            actor:       None,
            start_time:  None,
            end_time:    None,
        }
    }

    pub fn config_name(mut self, name: impl Into<String>) -> Self {
        self.config_name = Some(name.into());
        self
    }

    pub fn event_type(mut self, event_type: AuditEventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn matches(&self, event: &AuditEvent) -> bool {
        if let Some(ref name) = self.config_name {
            if event.config_name != *name {
                return false;
            }
        }

        if let Some(ref event_type) = self.event_type {
            if std::mem::discriminant(&event.event_type) != std::mem::discriminant(event_type) {
                return false;
            }
        }

        if let Some(ref start) = self.start_time {
            if event.timestamp < *start {
                return false;
            }
        }

        if let Some(ref end) = self.end_time {
            if event.timestamp > *end {
                return false;
            }
        }

        true
    }
}

/// Audit trail statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries:      usize,
    pub oldest_entry:       Option<DateTime<Utc>>,
    pub newest_entry:       Option<DateTime<Utc>>,
    pub integrity_verified: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_trail_creation() {
        let trail = AuditTrail::new().await.unwrap();
        assert!(trail.get_stats().await.total_entries >= 1); // Genesis entry
    }

    #[tokio::test]
    async fn test_audit_recording() {
        let trail = AuditTrail::new().await.unwrap();

        trail.record_load("test_config", 1, "test").await.unwrap();
        trail.record_save("test_config", "test").await.unwrap();

        let stats = trail.get_stats().await;
        assert!(stats.total_entries >= 3); // Genesis + load + save
    }

    #[tokio::test]
    async fn test_integrity_verification() {
        let trail = AuditTrail::new().await.unwrap();

        trail.record_load("test_config", 1, "test").await.unwrap();
        let integrity_ok = trail.verify_integrity().await.unwrap();

        assert!(integrity_ok);
    }

    #[tokio::test]
    async fn test_audit_search() {
        let trail = AuditTrail::new().await.unwrap();

        trail.record_load("test_config", 1, "test").await.unwrap();

        let filter = AuditSearchFilter::new().config_name("test_config");
        let results = trail.search_events(filter).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].config_name, "test_config");
    }
}
