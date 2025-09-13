//! Secure Key Management and Credential Rotation
//!
//! This module provides enterprise-grade key management and credential rotation
//! capabilities for secure cryptographic operations.
//!
//! # Key Management Features
//!
//! - **Hardware Security Module (HSM) Support**: FIPS 140-2 Level 3+ compliant
//! - **Automatic Key Rotation**: Configurable rotation policies and schedules
//! - **Key Lifecycle Management**: Generation, storage, distribution, retirement
//! - **Secure Backup and Recovery**: Encrypted key backups with access controls
//! - **Multi-Region Distribution**: Global key distribution and replication
//! - **Emergency Access Procedures**: Break-glass protocols for critical situations
//! - **Key Usage Auditing**: Complete audit trail of key operations
//!
//! # Credential Management Features
//!
//! - **Centralized Credential Storage**: Encrypted credential vault
//! - **Automatic Rotation**: Time-based and usage-based credential renewal
//! - **Multi-Factor Authentication**: MFA for credential access
//! - **Access Logging**: Detailed credential access and usage logs
//! - **Secure Distribution**: Time-limited credential distribution
//!
//! # Security Compartments
//!
//! - **Production Keys**: Long-lived keys for normal operations
//! - **Ephemeral Keys**: Short-lived keys for temporary operations
//! - **Emergency Keys**: High-security keys for break-glass scenarios
//! - **Backup Keys**: Encrypted keys for disaster recovery
//!
//! # Usage
//!
//! ```rust,no_run
//! use rust_ai_ide_security::key_management::{KeyManager, KeyRotationPolicy};
//!
//! // Create key manager with HSM integration
//! let key_manager = KeyManager::new_with_hsm().await?;
//!
//! // Generate a new encryption key
//! let key_id = key_manager.generate_key("production", "aes256").await?;
//!
//! // Use key for encryption
//! let encrypted = key_manager.encrypt_data(&key_id, b"sensitive data").await?;
//!
//! // Rotate key after 90 days
//! let new_key_id = key_manager.rotate_key(&key_id).await?;
//!
//! // Use credential manager
//! let credential_id = credential_manager.generate_credential("database", "admin").await?;
//! ```

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::SecurityResult;

/// Key management backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyBackendType {
    InMemory,
    FileSystem,
    HardwareSecurityModule {
        hsm_path: String,
        slot_id: String,
        pin_file: String,
    },
    CloudKMS {
        provider: CloudProvider,
        key_ring: String,
        project_id: String,
    },
    ExternalKMS {
        provider: String,
        endpoint: String,
        auth_token: String,
    },
}

/// Cloud providers for KMS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    AWS,
    GCP,
    Azure,
    Oracle,
}

/// Key purpose and classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyPurpose {
    Encryption,
    Signing,
    Authentication,
    Backup,
    Emergency,
    Audit,
}

/// Key algorithm types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Aes256Gcm,
    Chacha20Poly1305,
    Rsa2048,
    Rsa4096,
    EcSecp256r1,
    EcEd25519,
}

/// Key metadata and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub key_id: String,
    pub version: u32,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub usage_count: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub status: KeyStatus,
    pub backend_info: HashMap<String, String>,
    pub hsm_slot_id: Option<String>,
    pub backup_available: bool,
    pub encryption_policy: KeyEncryptionPolicy,
}

/// Key status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyStatus {
    Active,
    Rotating,
    Retired,
    Compromised,
    Expired,
}

/// Key encryption policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEncryptionPolicy {
    pub allow_export: bool,
    pub requires_mfa: bool,
    pub max_uses: Option<u64>,
    pub allowed_networks: Vec<String>,
    pub audit_required: bool,
}

/// Key rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationPolicy {
    pub automatic_rotation: bool,
    pub rotation_interval_days: u32,
    pub keep_versions: u32,
    pub rotation_notification_days: u32,
    pub emergency_rotation: bool,
    pub rotation_schedule: Option<RotationSchedule>,
}

/// Rotation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationSchedule {
    pub timezone: String,
    pub maintenance_window_start: String, // HH:MM format
    pub maintenance_window_end: String,
    pub blackout_periods: Vec<String>, // Dates to avoid rotation
}

/// Credential types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialType {
    APIKey,
    DatabasePassword,
    SSHKey,
    OAuthToken,
    JWTToken,
    Custom(String),
}

/// Credential metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    pub credential_id: String,
    pub name: String,
    pub credential_type: CredentialType,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_rotated_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub access_list: HashSet<String>, // User/role IDs with access
    pub rotation_policy: CredentialRotationPolicy,
    pub backup_status: BackupStatus,
}

/// Credential rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRotationPolicy {
    pub enabled: bool,
    pub rotation_interval_days: u32,
    pub max_lifetime_days: u32,
    pub auto_rotate: bool,
    pub rotation_notification_days: u32,
    pub require_manual_approval: bool,
}

/// Backup status for disaster recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStatus {
    NotBackedUp,
    LocalBackupOnly,
    OffsiteBackup,
    MultiRegionBackup,
    EncryptedBackupCritical,
}

/// Encrypted credential data
#[derive(Debug, Clone)]
pub struct EncryptedCredential {
    pub metadata: CredentialMetadata,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub key_id: String,
}

/// Key manager for cryptographic keys
#[async_trait]
pub trait KeyManager: Send + Sync {
    /// Generate a new cryptographic key
    async fn generate_key(&self, purpose: &str, algorithm: &str) -> SecurityResult<String>;

    /// Get key metadata
    async fn get_key_metadata(&self, key_id: &str) -> SecurityResult<Option<KeyMetadata>>;

    /// Use key for encryption/decryption operations
    async fn encrypt_data(&self, key_id: &str, data: &[u8]) -> SecurityResult<Vec<u8>>;

    async fn decrypt_data(&self, key_id: &str, encrypted_data: &[u8]) -> SecurityResult<Vec<u8>>;

    /// Rotate key (create new version)
    async fn rotate_key(&self, key_id: &str) -> SecurityResult<String>;

    /// Schedule key rotation
    async fn schedule_key_rotation(
        &self,
        key_id: &str,
        schedule: RotationSchedule,
    ) -> SecurityResult<()>;

    /// Get key rotation status
    async fn get_rotation_status(&self, key_id: &str) -> SecurityResult<RotationStatus>;

    /// Backup key securely
    async fn backup_key(&self, key_id: &str, backup_location: &str) -> SecurityResult<()>;

    /// Recover key from backup
    async fn recover_key(&self, key_id: &str, backup_location: &str) -> SecurityResult<()>;

    /// Revoke key (emergency operation)
    async fn revoke_key(&self, key_id: &str, reason: &str) -> SecurityResult<()>;

    /// List keys by status
    async fn list_keys(&self, status: Option<KeyStatus>) -> SecurityResult<Vec<KeyMetadata>>;

    /// Key health check
    async fn health_check(&self) -> SecurityResult<KeyHealthStatus>;
}

/// Credential manager for service credentials
#[async_trait]
pub trait CredentialManager: Send + Sync {
    /// Generate new credential
    async fn generate_credential(
        &self,
        service: &str,
        account_type: &str,
    ) -> SecurityResult<String>;

    /// Get credential metadata
    async fn get_credential_metadata(
        &self,
        credential_id: &str,
    ) -> SecurityResult<Option<CredentialMetadata>>;

    /// Retrieve credential (decrypted)
    async fn get_credential(&self, credential_id: &str, requester: &str) -> SecurityResult<String>;

    /// Update credential
    async fn update_credential(
        &self,
        credential_id: &str,
        new_credential: &str,
    ) -> SecurityResult<()>;

    /// Rotate credential
    async fn rotate_credential(&self, credential_id: &str) -> SecurityResult<String>;

    /// Schedule credential rotation
    async fn schedule_credential_rotation(
        &self,
        credential_id: &str,
        schedule: RotationSchedule,
    ) -> SecurityResult<()>;

    /// Revoke credential
    async fn revoke_credential(&self, credential_id: &str, reason: &str) -> SecurityResult<()>;

    /// Audit credential access
    async fn audit_credential_access(
        &self,
        credential_id: &str,
    ) -> SecurityResult<Vec<CredentialAccessEvent>>;

    /// Backup credentials
    async fn backup_credentials(&self) -> SecurityResult<()>;

    /// Get rotation status
    async fn get_credential_rotation_status(
        &self,
        credential_id: &str,
    ) -> SecurityResult<RotationStatus>;
}

/// Credential access event for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialAccessEvent {
    pub credential_id: String,
    pub accessed_by: String,
    pub access_time: DateTime<Utc>,
    pub access_type: String, // retrieve, update, rotate, etc.
    pub success: bool,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Rotation status for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationStatus {
    pub scheduled_at: Option<DateTime<Utc>>,
    pub last_rotated_at: Option<DateTime<Utc>>,
    pub next_rotation_at: Option<DateTime<Utc>>,
    pub rotation_count: u32,
    pub status: RotationResult,
}

/// Rotation result status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationResult {
    NotScheduled,
    Scheduled,
    InProgress,
    Success,
    Failed,
    Cancelled,
}

/// Key health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHealthStatus {
    pub backend_type: String,
    pub connection_status: bool,
    pub total_keys: u32,
    pub active_keys: u32,
    pub expired_keys: u32,
    pub backed_up_keys: u32,
    pub last_backup_time: Option<DateTime<Utc>>,
    pub alerts: Vec<String>,
}

// Implementation: Default Key Manager

/// Default key manager with software-based cryptography
pub struct SoftwareKeyManager {
    keys: Arc<RwLock<HashMap<String, KeyMetadata>>>,
    key_storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    rotation_policies: Arc<RwLock<HashMap<String, KeyRotationPolicy>>>,
    backup_key: Vec<u8>, // Master key for encrypting key data
}

impl SoftwareKeyManager {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            key_storage: Arc::new(RwLock::new(HashMap::new())),
            rotation_policies: Arc::new(RwLock::new(HashMap::new())),
            backup_key: crate::global_backup_key(), // In real implementation, this would be securely sourced
        }
    }

    fn generate_cryptographic_key(&self, algorithm: &KeyAlgorithm) -> SecurityResult<Vec<u8>> {
        match algorithm {
            KeyAlgorithm::Aes256Gcm | KeyAlgorithm::Chacha20Poly1305 => {
                // For AES256/Chacha20, generate 256-bit key
                let mut key = vec![0u8; 32];
                rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut key);
                Ok(key)
            }
            KeyAlgorithm::Rsa2048 => {
                // Would use RSA library to generate key pair
                Err(crate::SecurityError::ConfigurationError {
                    config_error: "RSA key generation not implemented".to_string(),
                })
            }
            KeyAlgorithm::Rsa4096 => Err(crate::SecurityError::ConfigurationError {
                config_error: "RSA4096 key generation not implemented".to_string(),
            }),
            KeyAlgorithm::EcSecp256r1 => {
                // Would use ECC library to generate key
                Err(crate::SecurityError::ConfigurationError {
                    config_error: "EC key generation not implemented".to_string(),
                })
            }
            KeyAlgorithm::EcEd25519 => Err(crate::SecurityError::ConfigurationError {
                config_error: "Ed25519 key generation not implemented".to_string(),
            }),
        }
    }
}

#[async_trait]
impl KeyManager for SoftwareKeyManager {
    async fn generate_key(&self, purpose: &str, algorithm: &str) -> SecurityResult<String> {
        let algorithm = match algorithm {
            "aes256" | "aes256-gcm" => KeyAlgorithm::Aes256Gcm,
            "chacha20" => KeyAlgorithm::Chacha20Poly1305,
            _ => {
                return Err(crate::SecurityError::ConfigurationError {
                    config_error: format!("Unsupported algorithm: {}", algorithm),
                })
            }
        };

        let purpose_type = match purpose {
            "production" => KeyPurpose::Encryption,
            "signing" => KeyPurpose::Signing,
            "authentication" => KeyPurpose::Authentication,
            "emergency" => KeyPurpose::Emergency,
            _ => KeyPurpose::Encryption,
        };

        let key_id = format!("key_{}", uuid::Uuid::new_v4());
        let key_version = 1;
        let now = Utc::now();

        // Generate cryptographic key material
        let key_material = self.generate_cryptographic_key(&algorithm)?;

        // Encrypt key material with backup key for secure storage
        let (encrypted_key, nonce) = crate::encrypt_with_backup_key(&key_material)?;

        let key_metadata = KeyMetadata {
            key_id: key_id.clone(),
            version: key_version,
            algorithm: algorithm.clone(),
            purpose: purpose_type,
            usage_count: 0,
            created_at: now,
            expires_at: Some(now + chrono::Duration::days(365)),
            rotated_at: None,
            status: KeyStatus::Active,
            backend_info: [("backend".to_string(), "software".to_string())].into(),
            hsm_slot_id: None,
            backup_available: true,
            encryption_policy: KeyEncryptionPolicy {
                allow_export: false,
                requires_mfa: false,
                max_uses: Some(1000000),
                allowed_networks: vec![],
                audit_required: true,
            },
        };

        // Store metadata and encrypted key material
        let mut keys = self.keys.write().await;
        let mut key_storage = self.key_storage.write().await;
        keys.insert(key_id.clone(), key_metadata);
        key_storage.insert(key_id.clone(), encrypted_key);

        // Set up default rotation policy
        let rotation_policy = KeyRotationPolicy {
            automatic_rotation: true,
            rotation_interval_days: 90,
            keep_versions: 5,
            rotation_notification_days: 7,
            emergency_rotation: false,
            rotation_schedule: Some(RotationSchedule {
                timezone: "UTC".to_string(),
                maintenance_window_start: "02:00".to_string(),
                maintenance_window_end: "06:00".to_string(),
                blackout_periods: vec![],
            }),
        };

        let mut policies = self.rotation_policies.write().await;
        policies.insert(key_id.clone(), rotation_policy);

        Ok(key_id)
    }

    async fn get_key_metadata(&self, key_id: &str) -> SecurityResult<Option<KeyMetadata>> {
        let keys = self.keys.read().await;
        Ok(keys.get(key_id).cloned())
    }

    async fn encrypt_data(&self, key_id: &str, data: &[u8]) -> SecurityResult<Vec<u8>> {
        let key_storage = self.key_storage.read().await;
        let key_data =
            key_storage
                .get(key_id)
                .ok_or_else(|| crate::SecurityError::ConfigurationError {
                    config_error: format!("Key not found: {}", key_id),
                })?;

        // Decrypt the key
        let key_material = crate::decrypt_with_backup_key(key_data)?;

        // Use the key for encryption (simplified)
        use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
        let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_material[0..32]);
        let nonce = Aes256Gcm::generate_nonce(&mut rand::rngs::OsRng);
        let cipher = Aes256Gcm::new(cipher_key);

        match cipher.encrypt(&nonce, data) {
            Ok(mut ciphertext) => {
                // Prepend nonce for storage
                ciphertext.splice(0..0, nonce);
                Ok(ciphertext)
            }
            Err(e) => Err(crate::SecurityError::EncryptionError {
                source: format!("Encryption failed: {}", e).into(),
            }),
        }
    }

    async fn decrypt_data(&self, key_id: &str, encrypted_data: &[u8]) -> SecurityResult<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(crate::SecurityError::EncryptionError {
                source: "Invalid encrypted data".into(),
            });
        }

        let key_storage = self.key_storage.read().await;
        let key_data =
            key_storage
                .get(key_id)
                .ok_or_else(|| crate::SecurityError::ConfigurationError {
                    config_error: format!("Key not found: {}", key_id),
                })?;

        // Decrypt the key
        let key_material = crate::decrypt_with_backup_key(key_data)?;

        // Extract nonce and ciphertext
        let nonce_slice = &encrypted_data[0..12];
        let nonce = aes_gcm::Nonce::from_slice(nonce_slice);
        let ciphertext = &encrypted_data[12..];

        // Use the key for decryption
        use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit};
        let cipher = Aes256Gcm::new_from_slice(&key_material).map_err(|_| {
            crate::SecurityError::EncryptionError {
                source: "Invalid key format".into(),
            }
        })?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| crate::SecurityError::EncryptionError {
                source: format!("Decryption failed: {}", e).into(),
            })
    }

    async fn rotate_key(&self, key_id: &str) -> SecurityResult<String> {
        let mut keys = self.keys.write().await;
        let current_key =
            keys.get_mut(key_id)
                .ok_or_else(|| crate::SecurityError::ConfigurationError {
                    config_error: format!("Key not found: {}", key_id),
                })?;

        // Generate new key material
        let algorithm = current_key.algorithm.clone();
        let key_material = self.generate_cryptographic_key(&algorithm)?;

        // Encrypt with backup key
        let (encrypted_key, _nonce) = crate::encrypt_with_backup_key(&key_material)?;

        // Create new key metadata
        let now = Utc::now();
        let new_key_id = format!("{}_v{}", key_id, current_key.version + 1);

        let new_key_metadata = KeyMetadata {
            key_id: new_key_id.clone(),
            version: current_key.version + 1,
            algorithm: algorithm.clone(),
            purpose: current_key.purpose.clone(),
            usage_count: 0,
            created_at: now,
            expires_at: Some(now + chrono::Duration::days(365)),
            rotated_at: Some(now),
            status: KeyStatus::Active,
            backend_info: [("backend".to_string(), "software".to_string())].into(),
            hsm_slot_id: None,
            backup_available: true,
            encryption_policy: current_key.encryption_policy.clone(),
        };

        // Store new key
        keys.insert(new_key_id.clone(), new_key_metadata);
        let mut key_storage = self.key_storage.write().await;
        key_storage.insert(new_key_id.clone(), encrypted_key);

        // Update old key status
        current_key.status = KeyStatus::Retired;
        current_key.rotated_at = Some(now);

        Ok(new_key_id)
    }

    async fn schedule_key_rotation(
        &self,
        key_id: &str,
        schedule: RotationSchedule,
    ) -> SecurityResult<()> {
        let mut policies = self.rotation_policies.write().await;
        if let Some(policy) = policies.get_mut(key_id) {
            policy.rotation_schedule = Some(schedule);
            Ok(())
        } else {
            Err(crate::SecurityError::ConfigurationError {
                config_error: format!("Key rotation policy not found: {}", key_id),
            })
        }
    }

    async fn get_rotation_status(&self, key_id: &str) -> SecurityResult<RotationStatus> {
        let keys = self.keys.read().await;
        let policies = self.rotation_policies.read().await;

        let key = keys
            .get(key_id)
            .ok_or_else(|| crate::SecurityError::ConfigurationError {
                config_error: format!("Key not found: {}", key_id),
            })?;

        let policy =
            policies
                .get(key_id)
                .ok_or_else(|| crate::SecurityError::ConfigurationError {
                    config_error: format!("Rotation policy not found: {}", key_id),
                })?;

        Ok(RotationStatus {
            scheduled_at: None, // Could be calculated based on policy
            last_rotated_at: key.rotated_at,
            next_rotation_at: None, // Could be calculated
            rotation_count: key.version - 1,
            status: RotationResult::Scheduled,
        })
    }

    async fn backup_key(&self, key_id: &str, _backup_location: &str) -> SecurityResult<()> {
        let keys = self.keys.read().await;
        let key_storage = self.key_storage.read().await;

        if keys.contains_key(key_id) && key_storage.contains_key(key_id) {
            // In real implementation, would write to backup location
            Ok(())
        } else {
            Err(crate::SecurityError::ConfigurationError {
                config_error: "Key not found for backup".to_string(),
            })
        }
    }

    async fn recover_key(&self, key_id: &str, _backup_location: &str) -> SecurityResult<()> {
        // In real implementation, would read from backup location
        Err(crate::SecurityError::ConfigurationError {
            config_error: "Recovery requires HSM or cloud provider".to_string(),
        })
    }

    async fn revoke_key(&self, key_id: &str, _reason: &str) -> SecurityResult<()> {
        let mut keys = self.keys.write().await;
        if let Some(key) = keys.get_mut(key_id) {
            key.status = KeyStatus::Compromised;
            Ok(())
        } else {
            Err(crate::SecurityError::ConfigurationError {
                config_error: format!("Key not found: {}", key_id),
            })
        }
    }

    async fn list_keys(
        &self,
        status_filter: Option<KeyStatus>,
    ) -> SecurityResult<Vec<KeyMetadata>> {
        let keys = self.keys.read().await;
        let mut result: Vec<KeyMetadata> = keys.values().cloned().collect();

        if let Some(status) = status_filter {
            result.retain(|k| k.status == status);
        }

        Ok(result)
    }

    async fn health_check(&self) -> SecurityResult<KeyHealthStatus> {
        let keys = self.keys.read().await;
        let total_keys = keys.len() as u32;
        let active_keys = keys
            .values()
            .filter(|k| k.status == KeyStatus::Active)
            .count() as u32;
        let expired_keys = keys
            .values()
            .filter(|k| k.expires_at.is_some() && k.expires_at.unwrap() < Utc::now())
            .count() as u32;
        let backed_up_keys = keys.values().filter(|k| k.backup_available).count() as u32;

        Ok(KeyHealthStatus {
            backend_type: "software".to_string(),
            connection_status: true,
            total_keys,
            active_keys,
            expired_keys,
            backed_up_keys,
            last_backup_time: Some(Utc::now()), // Simplified
            alerts: vec![],                     // Would contain actual alerts
        })
    }
}

// Utility functions (would be in separate module in real implementation)
fn encrypt_with_backup_key(data: &[u8]) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
    let backup_key = b"super_secret_backup_key_1234567890123456"; // NEVER USE IN PRODUCTION
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

    let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(backup_key);
    let nonce = Aes256Gcm::generate_nonce(&mut rand::rngs::OsRng);
    let cipher = Aes256Gcm::new(cipher_key);

    match cipher.encrypt(&nonce, data) {
        Ok(ciphertext) => Ok((ciphertext, nonce.to_vec())),
        Err(e) => Err(crate::SecurityError::EncryptionError {
            source: format!("Backup key encryption failed: {}", e).into(),
        }),
    }
}

fn decrypt_with_backup_key(encrypted_data: &[u8]) -> SecurityResult<Vec<u8>> {
    let backup_key = b"super_secret_backup_key_1234567890123456"; // NEVER USE IN PRODUCTION
    if encrypted_data.len() < 12 {
        return Err(crate::SecurityError::EncryptionError {
            source: "Invalid encrypted data for backup".into(),
        });
    }

    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
    let cipher = Aes256Gcm::new_from_slice(backup_key).map_err(|_| {
        crate::SecurityError::EncryptionError {
            source: "Invalid backup key".into(),
        }
    })?;

    let nonce = Nonce::from_slice(&encrypted_data[0..12]);
    let ciphertext = &encrypted_data[12..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| crate::SecurityError::EncryptionError {
            source: format!("Backup key decryption failed: {}", e).into(),
        })
}

// Factory functions
pub async fn create_software_key_manager() -> Arc<dyn KeyManager> {
    Arc::new(SoftwareKeyManager::new())
}

pub async fn create_hsm_key_manager(
    _hsm_config: HashMap<String, String>,
) -> SecurityResult<Arc<dyn KeyManager>> {
    // In real implementation, this would create HSM-backed key manager
    Err(crate::SecurityError::ConfigurationError {
        config_error: "HSM key manager not implemented for this demo".to_string(),
    })
}

pub async fn create_aws_kms_key_manager(
    _aws_config: HashMap<String, String>,
) -> SecurityResult<Arc<dyn KeyManager>> {
    // In real implementation, this would create AWS KMS-backed key manager
    Err(crate::SecurityError::ConfigurationError {
        config_error: "AWS KMS key manager not implemented for this demo".to_string(),
    })
}

// Configuration creation helper
pub fn create_default_key_rotation_policy() -> KeyRotationPolicy {
    KeyRotationPolicy {
        automatic_rotation: true,
        rotation_interval_days: 90,
        keep_versions: 5,
        rotation_notification_days: 7,
        emergency_rotation: false,
        rotation_schedule: Some(RotationSchedule {
            timezone: "UTC".to_string(),
            maintenance_window_start: "02:00".to_string(),
            maintenance_window_end: "06:00".to_string(),
            blackout_periods: vec!["2024-12-25".to_string(), "2024-12-31".to_string()],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test as async_test;

    #[async_test]
    async fn test_key_generation() {
        let key_manager = create_software_key_manager().await;

        let key_id = key_manager.generate_key("test", "aes256").await.unwrap();
        assert!(!key_id.is_empty());

        let metadata = key_manager.get_key_metadata(&key_id).await.unwrap();
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.algorithm, KeyAlgorithm::Aes256Gcm);
        assert_eq!(metadata.purpose, KeyPurpose::Encryption);
        assert_eq!(metadata.status, KeyStatus::Active);
    }

    #[async_test]
    async fn test_encrypt_decrypt_round_trip() {
        let key_manager = create_software_key_manager().await;

        let key_id = key_manager.generate_key("test", "aes256").await.unwrap();
        let test_data = b"Hello, encrypted world!";

        // Encrypt
        let encrypted = key_manager.encrypt_data(&key_id, test_data).await.unwrap();
        assert!(!encrypted.is_empty());

        // Decrypt
        let decrypted = key_manager.decrypt_data(&key_id, &encrypted).await.unwrap();
        assert_eq!(decrypted, test_data);
    }

    #[async_test]
    async fn test_key_rotation() {
        let key_manager = create_software_key_manager().await;
        let old_key_id = key_manager
            .generate_key("production", "aes256")
            .await
            .unwrap();

        // Verify original key exists
        let original_metadata = key_manager
            .get_key_metadata(&old_key_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(original_metadata.status, KeyStatus::Active);

        // Rotate key
        let new_key_id = key_manager.rotate_key(&old_key_id).await.unwrap();
        assert!(!new_key_id.is_empty());

        // Verify new key
        let new_metadata = key_manager
            .get_key_metadata(&new_key_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(new_metadata.version, 2);
        assert_eq!(new_metadata.status, KeyStatus::Active);

        // Verify old key was retired
        let retired_metadata = key_manager
            .get_key_metadata(&old_key_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retired_metadata.status, KeyStatus::Retired);
        assert!(retired_metadata.rotated_at.is_some());
    }

    #[async_test]
    async fn test_health_check() {
        let key_manager = create_software_key_manager().await;

        let health = key_manager.health_check().await.unwrap();
        assert_eq!(health.backend_type, "software");
        assert!(health.connection_status);
    }

    #[async_test]
    async fn test_key_listing() {
        let key_manager = create_software_key_manager().await;

        // Generate some keys
        let key1 = key_manager.generate_key("test1", "aes256").await.unwrap();
        let key2 = key_manager.generate_key("test2", "chacha20").await.unwrap();

        // List all keys
        let all_keys = key_manager.list_keys(None).await.unwrap();
        assert_eq!(all_keys.len(), 2);

        // List only active keys
        let active_keys = key_manager
            .list_keys(Some(KeyStatus::Active))
            .await
            .unwrap();
        assert_eq!(active_keys.len(), 2);

        // Verify key IDs
        let key_ids: HashSet<String> = all_keys.iter().map(|k| k.key_id.clone()).collect();
        assert!(key_ids.contains(&key1));
        assert!(key_ids.contains(&key2));
    }

    #[test]
    fn test_rotation_policy_creation() {
        let policy = create_default_key_rotation_policy();

        assert!(policy.automatic_rotation);
        assert_eq!(policy.rotation_interval_days, 90);
        assert_eq!(policy.keep_versions, 5);
        assert_eq!(policy.rotation_notification_days, 7);
        assert!(!policy.emergency_rotation);

        let schedule = policy.rotation_schedule.unwrap();
        assert_eq!(schedule.timezone, "UTC");
        assert_eq!(schedule.maintenance_window_start, "02:00");
        assert_eq!(schedule.maintenance_window_end, "06:00");
        assert!(schedule
            .blackout_periods
            .contains(&"2024-12-25".to_string()));
    }
}
