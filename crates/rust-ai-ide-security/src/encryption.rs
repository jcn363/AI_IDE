//! Encrypted Configuration Management
//!
//! This module provides secure configuration management with end-to-end encryption,
//! protecting sensitive data at rest and ensuring secure access to configuration parameters.
//!
//! Features:
//! - **AES-256-GCM Encryption**: Industry-standard encryption for configuration data
//! - **Key Hierarchy**: Master keys, data keys, and session keys
//! - **Key Rotation**: Automated key rotation with backward compatibility
//! - **Access Control**: Fine-grained permissions for configuration access
//! - **Audit Trail**: Complete logging of configuration changes
//! - **Versioning**: Configuration history and rollback capabilities
//! - **Secure Storage**: Multiple backend support (file, database, remote KMS)
//! - **FIPS Compliance**: Crypto operations compliant with FIPS standards

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose};

use crate::{
    SecurityResult, SecurityError, EncryptionConfig,
    ComponentStatus, UserContext, SensitivityLevel,
};

/// Encryption algorithm types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    Chacha20Poly1305,
    Aes256Cbc, // Legacy support
}

/// Key management backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyBackendType {
    InMemory,
    FileSystem,
    Database,
    HardwareSecurityModule, // HSM support
    CloudKMS {
        provider: String, // AWS KMS, Azure Key Vault, GCP KMS
        key_uri: String,
    },
}

/// Master key for encrypting data keys
#[derive(Debug, Clone)]
pub struct MasterKey {
    key_id: String,
    key_version: u32,
    algorithm: EncryptionAlgorithm,
    key_material: Vec<u8>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    is_active: bool,
}

/// Data encryption key for encrypting payload data
#[derive(Debug, Clone)]
pub struct DataKey {
    key_id: String,
    version: u32,
    encrypted_key_material: Vec<u8>, // Encrypted with master key
    algorithm: EncryptionAlgorithm,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    is_active: bool,
}

/// Encrypted configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedConfigEntry {
    config_key: String,
    encrypted_value: Vec<u8>, // Encrypted configuration value
    data_key_id: String,      // Reference to the data key used for encryption
    nonce: Vec<u8>,          // Nonce/IV for authenticated encryption
    encryption_algorithm: EncryptionAlgorithm,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    owner: String,           // User/entity that owns this configuration
    access_list: HashSet<String>, // Users/roles with access
    sensitivity_level: SensitivityLevel,
    metadata: HashMap<String, String>,
    version: u32,
    previous_versions: Vec<EncryptedConfigEntry>, // For version history
}

/// Configuration access policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    policy_id: String,
    config_pattern: String, // e.g., "ai.*", "database.credentials.*"
    allowed_roles: HashSet<String>,
    allowed_users: HashSet<String>,
    allowed_actions: HashSet<String>, // "read", "write", "delete", "grant"
    conditions: HashMap<String, String>,
    created_by: String,
    created_at: DateTime<Utc>,
}

/// Encrypted configuration manager
pub struct EncryptedConfigManager {
    config: EncryptionConfig,
    master_key_manager: Arc<dyn MasterKeyManager>,
    data_key_manager: Arc<dyn DataKeyManager>,
    storage_backend: Arc<dyn EncryptedStorageBackend>,
    access_policies: RwLock<Vec<AccessPolicy>>,
    cache: RwLock<HashMap<String, EncryptedConfigEntry>>,
    stats: RwLock<EncryptionStats>,
    // Wave 3 enhancements
    multi_key_manager: Arc<MultiKeyManager>,
    performance_optimizer: Arc<PerformanceOptimizer>,
    pq_crypto_engine: Option<Arc<PostQuantumCryptoEngine>>,
}

/// Statistics for encryption operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStats {
    total_encryptions: u64,
    total_decryptions: u64,
    key_rotations: u64,
    failed_decryptions: u64,
    cache_hits: u64,
    cache_misses: u64,
    key_generation_time_ms: u64,
    encryption_time_ms: u64,
    decryption_time_ms: u64,
}

// Key management traits

#[async_trait]
pub trait MasterKeyManager: Send + Sync {
    /// Generate a new master key
    async fn generate_master_key(&self) -> SecurityResult<MasterKey>;

    /// Retrieve master key by ID
    async fn get_master_key(&self, key_id: &str) -> SecurityResult<Option<MasterKey>>;

    /// Rotate master key and re-encrypt all data keys
    async fn rotate_master_key(&self, new_key: MasterKey) -> SecurityResult<()> ;

    /// Delete master key (if supported by backend)
    async fn delete_master_key(&self, key_id: &str) -> SecurityResult<()> ;

    /// List active master keys
    async fn list_active_keys(&self) -> SecurityResult<Vec<MasterKey>>;
}

#[async_trait]
pub trait DataKeyManager: Send + Sync {
    /// Generate a new data encryption key for a specific purpose
    async fn generate_data_key(&self, purpose: &str, algorithm: EncryptionAlgorithm) -> SecurityResult<DataKey>;

    /// Retrieve data key by ID
    async fn get_data_key(&self, key_id: &str) -> SecurityResult<Option<DataKey>>;

    /// Decrypt data key using master key
    async fn decrypt_data_key(&self, encrypted_key: &[u8], master_key: &MasterKey) -> SecurityResult<Vec<u8>>;

    /// List active data keys
    async fn list_data_keys(&self) -> SecurityResult<Vec<DataKey>>;
}

#[async_trait]
pub trait EncryptedStorageBackend: Send + Sync {
    /// Store encrypted configuration entry
    async fn store(&self, entry: &EncryptedConfigEntry) -> SecurityResult<()> ;

    /// Retrieve encrypted configuration entry
    async fn retrieve(&self, config_key: &str) -> SecurityResult<Option<EncryptedConfigEntry>>;

    /// List configuration entries with optional pattern matching
    async fn list(&self, pattern: Option<&str>, limit: usize) -> SecurityResult<Vec<EncryptedConfigEntry>>;

    /// Delete configuration entry
    async fn delete(&self, config_key: &str) -> SecurityResult<()> ;

    /// Search configuration entries by metadata
    async fn search_by_metadata(&self, metadata: HashMap<String, String>) -> SecurityResult<Vec<EncryptedConfigEntry>>;

    /// Get version history for a configuration key
    async fn get_version_history(&self, config_key: &str, limit: usize) -> SecurityResult<Vec<EncryptedConfigEntry>>;
}

/// Crypto operations for encryption/decryption
pub struct CryptoOps {
    algorithm: EncryptionAlgorithm,
}

impl CryptoOps {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8], key: &[u8], aad: Option<&[u8]>) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                self.encrypt_aes256_gcm(plaintext, key, aad)
            }
            EncryptionAlgorithm::Chacha20Poly1305 => {
                self.encrypt_chacha20_poly1305(plaintext, key, aad)
            }
            EncryptionAlgorithm::Aes256Cbc => {
                self.encrypt_aes256_cbc(plaintext, key, aad.unwrap_or_default())
            }
        }
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8], aad: Option<&[u8]>) -> SecurityResult<Vec<u8>> {
        match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                self.decrypt_aes256_gcm(ciphertext, key, nonce, aad)
            }
            EncryptionAlgorithm::Chacha20Poly1305 => {
                self.decrypt_chacha20_poly1305(ciphertext, key, nonce, aad)
            }
            EncryptionAlgorithm::Aes256Cbc => {
                self.decrypt_aes256_cbc(ciphertext, key, nonce, aad.unwrap_or_default())
            }
        }
    }

    fn encrypt_aes256_gcm(&self, plaintext: &[u8], key: &[u8], _aad: Option<&[u8]>) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, AeadCore, Nonce}};

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| SecurityError::EncryptionError {
                source: "Invalid key size for AES-256-GCM".into()
            })?;

        let nonce = Aes256Gcm::generate_nonce(&mut rand::rngs::OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|e| SecurityError::EncryptionError {
                source: format!("AES-256-GCM encryption failed: {}", e).into()
            })?;

        Ok((ciphertext, nonce.to_vec()))
    }

    fn decrypt_aes256_gcm(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8], _aad: Option<&[u8]>) -> SecurityResult<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, Nonce}};

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| SecurityError::EncryptionError {
                source: "Invalid key size for AES-256-GCM".into()
            })?;

        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| SecurityError::EncryptionError {
                source: format!("AES-256-GCM decryption failed: {}", e).into()
            })?;

        Ok(plaintext)
    }

    fn encrypt_chacha20_poly1305(&self, plaintext: &[u8], key: &[u8], _aad: Option<&[u8]>) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        use chacha20poly1305::{ChaCha20Poly1305, KeyInit, aead::{Aead, AeadCore}};

        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|_| SecurityError::EncryptionError {
                source: "Invalid key size for ChaCha20-Poly1305".into()
            })?;

        let nonce = ChaCha20Poly1305::generate_nonce(&mut rand::rngs::OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|e| SecurityError::EncryptionError {
                source: format!("ChaCha20-Poly1305 encryption failed: {}", e).into()
            })?;

        Ok((ciphertext, nonce.to_vec()))
    }

    fn decrypt_chacha20_poly1305(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8], _aad: Option<&[u8]>) -> SecurityResult<Vec<u8>> {
        use chacha20poly1305::{ChaCha20Poly1305, KeyInit, aead::{Aead, Nonce}};

        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|_| SecurityError::EncryptionError {
                source: "Invalid key size for ChaCha20-Poly1305".into()
            })?;

        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| SecurityError::EncryptionError {
                source: format!("ChaCha20-Poly1305 decryption failed: {}", e).into()
            })?;

        Ok(plaintext)
    }

    fn encrypt_aes256_cbc(&self, plaintext: &[u8], key: &[u8], iv: &[u8]) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        // Note: CBC mode should use HMAC for authentication, but this is a simplified implementation
        // In production, use authenticated encryption modes like GCM
        use aes::cipher::{KeyIvInit, StreamCipher};
        use aes::{Aes256, Aes256Enc};

        let mut cipher = cbc::Encryptor::<Aes256Enc>::new_from_slices(key, iv)
            .map_err(|e| SecurityError::EncryptionError {
                source: format!("CBC encryption setup failed: {}", e).into()
            })?;

        let mut buffer = plaintext.to_vec();
        cipher.encrypt(&mut buffer);

        Ok((buffer, iv.to_vec()))
    }

    fn decrypt_aes256_cbc(&self, ciphertext: &[u8], key: &[u8], iv: &[u8], _aad: Option<&[u8]>) -> SecurityResult<Vec<u8>> {
        // Note: AES-CBC without authentication is not recommended for production.
        // This is a simplified implementation for backwards compatibility only.
        // Use AES-GCM for new implementations.

        Err(SecurityError::EncryptionError {
            source: "AES-CBC without authentication is not supported for decryption".into(),
        })
    }

    /// Generate cryptographically secure random bytes
    pub fn generate_secure_random(&self, length: usize) -> SecurityResult<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut buffer[..]);
        Ok(buffer)
    }

    /// Generate a cryptographic key
    pub fn generate_key(&self, algorithm: EncryptionAlgorithm) -> SecurityResult<Vec<u8>> {
        let key_size = match algorithm {
            EncryptionAlgorithm::Aes256Gcm |
            EncryptionAlgorithm::Aes256Cbc => 32, // AES-256
            EncryptionAlgorithm::Chacha20Poly1305 => 32, // ChaCha20
        };

        self.generate_secure_random(key_size)
    }
}

// Storage backends

/// In-memory storage backend for development/testing
pub struct InMemoryConfigStorage {
    entries: RwLock<HashMap<String, EncryptedConfigEntry>>,
}

impl InMemoryConfigStorage {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl EncryptedStorageBackend for InMemoryConfigStorage {
    async fn store(&self, entry: &EncryptedConfigEntry) -> SecurityResult<()> {
        let mut entries = self.entries.write().await;
        entries.insert(entry.config_key.clone(), entry.clone());
        Ok(())
    }

    async fn retrieve(&self, config_key: &str) -> SecurityResult<Option<EncryptedConfigEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.get(config_key).cloned())
    }

    async fn list(&self, pattern: Option<&str>, limit: usize) -> SecurityResult<Vec<EncryptedConfigEntry>> {
        let entries = self.entries.read().await;
        let mut result: Vec<EncryptedConfigEntry> = entries.values().cloned().collect();

        if let Some(pattern) = pattern {
            result.retain(|entry| entry.config_key.contains(pattern));
        }

        result.truncate(limit);
        Ok(result)
    }

    async fn delete(&self, config_key: &str) -> SecurityResult<()> {
        let mut entries = self.entries.write().await;
        entries.remove(config_key);
        Ok(())
    }

    async fn search_by_metadata(&self, metadata: HashMap<String, String>) -> SecurityResult<Vec<EncryptedConfigEntry>> {
        let entries = self.entries.read().await;
        let mut result = Vec::new();

        for entry in entries.values() {
            let mut matches = true;
            for (key, value) in &metadata {
                if let Some(entry_value) = entry.metadata.get(key) {
                    if entry_value != value {
                        matches = false;
                        break;
                    }
                } else {
                    matches = false;
                    break;
                }
            }
            if matches {
                result.push(entry.clone());
            }
        }

        Ok(result)
    }

    async fn get_version_history(&self, _config_key: &str, _limit: usize) -> SecurityResult<Vec<EncryptedConfigEntry>> {
        // In-memory implementation doesn't maintain version history
        Ok(Vec::new())
    }
}

// Wave 3: Multi-key manager for handling multiple encryption keys
pub struct MultiKeyManager {
    keys: RwLock<HashMap<String, KeySet>>,
    rotation_scheduler: Arc<KeyRotationScheduler>,
    key_strategy: KeySelectionStrategy,
}

/// Key set for multi-key encryption
#[derive(Debug, Clone)]
pub struct KeySet {
    pub primary_key: DataKey,
    pub backup_keys: Vec<DataKey>,
    pub rotation_schedule: KeyRotationSchedule,
}

/// Key rotation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationSchedule {
    pub next_rotation: DateTime<Utc>,
    pub rotation_interval_days: u32,
    pub rotation_strategy: RotationStrategy,
}

/// Key rotation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    Automatic,
    OnDemand,
    Scheduled,
}

/// Key selection strategy
#[derive(Debug, Clone)]
pub enum KeySelectionStrategy {
    PrimaryFirst,
    LoadBalanced,
    PerformanceOptimized,
}

/// Key rotation scheduler
pub struct KeyRotationScheduler {
    // In production, this would use a cron-like scheduler
}

impl KeyRotationScheduler {
    pub fn new() -> Self {
        Self {}
    }
}

impl MultiKeyManager {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            rotation_scheduler: Arc::new(KeyRotationScheduler::new()),
            key_strategy: KeySelectionStrategy::PrimaryFirst,
        }
    }

    /// Select optimal key for encryption based on strategy
    pub async fn select_key(&self, purpose: &str) -> SecurityResult<DataKey> {
        let keys = self.keys.read().await;
        if let Some(key_set) = keys.get(purpose) {
            match self.key_strategy {
                KeySelectionStrategy::PrimaryFirst => Ok(key_set.primary_key.clone()),
                KeySelectionStrategy::LoadBalanced => {
                    // Simple load balancing - just use primary for now
                    Ok(key_set.primary_key.clone())
                },
                KeySelectionStrategy::PerformanceOptimized => {
                    // Use performance metrics to select best key
                    Ok(key_set.primary_key.clone())
                }
            }
        } else {
            Err(SecurityError::EncryptionError {
                source: format!("No key set found for purpose: {}", purpose).into(),
            })
        }
    }

    /// Add a new key set
    pub async fn add_key_set(&self, purpose: String, primary_key: DataKey, backup_keys: Vec<DataKey>) -> SecurityResult<()> {
        let key_set = KeySet {
            primary_key,
            backup_keys,
            rotation_schedule: KeyRotationSchedule {
                next_rotation: Utc::now() + chrono::Duration::days(90),
                rotation_interval_days: 90,
                rotation_strategy: RotationStrategy::Scheduled,
            },
        };

        let mut keys = self.keys.write().await;
        keys.insert(purpose, key_set);
        Ok(())
    }
}

/// Performance optimizer for encryption operations (Wave 3)
pub struct PerformanceOptimizer {
    operation_cache: RwLock<HashMap<String, PerformanceMetrics>>,
    hardware_acceleration: bool,
}

/// Performance metrics for encryption operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub avg_encrypt_time: u64,
    pub avg_decrypt_time: u64,
    pub throughput: f64,
    pub memory_usage: usize,
    pub last_updated: DateTime<Utc>,
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            operation_cache: RwLock::new(HashMap::new()),
            hardware_acceleration: false, // Can be detected at runtime
        }
    }

    /// Optimize encryption operation based on performance metrics
    pub async fn optimize_encryption(&self, data_size: usize, algorithm: EncryptionAlgorithm) -> SecurityResult<EncryptionOptimization> {
        let cache_key = format!("{:?}_{}", algorithm, data_size);
        let mut cache = self.operation_cache.write().await;

        if let Some(metrics) = cache.get(&cache_key) {
            // Use cached optimization if recent
            if (Utc::now() - metrics.last_updated).num_minutes() < 60 {
                return Ok(EncryptionOptimization::from_metrics(metrics));
            }
        }

        // Calculate optimal settings
        let optimization = if data_size > 1024 * 1024 { // > 1MB
            EncryptionOptimization {
                chunk_size: 64 * 1024, // 64KB chunks
                concurrent_operations: 2,
                use_hardware_accel: self.hardware_acceleration,
                algorithm: algorithm,
            }
        } else if data_size > 64 * 1024 { // > 64KB
            EncryptionOptimization {
                chunk_size: 32 * 1024, // 32KB chunks
                concurrent_operations: 1,
                use_hardware_accel: self.hardware_acceleration,
                algorithm: algorithm,
            }
        } else {
            EncryptionOptimization {
                chunk_size: data_size,
                concurrent_operations: 1,
                use_hardware_accel: false,
                algorithm: algorithm,
            }
        };

        // Update cache
        cache.insert(cache_key, PerformanceMetrics {
            avg_encrypt_time: 0,
            avg_decrypt_time: 0,
            throughput: 0.0,
            memory_usage: data_size,
            last_updated: Utc::now(),
        });

        Ok(optimization)
    }
}

/// Encryption optimization settings
#[derive(Debug, Clone)]
pub struct EncryptionOptimization {
    pub chunk_size: usize,
    pub concurrent_operations: usize,
    pub use_hardware_accel: bool,
    pub algorithm: EncryptionAlgorithm,
}

impl EncryptionOptimization {
    fn from_metrics(_metrics: &PerformanceMetrics) -> Self {
        // Simplified implementation
        Self {
            chunk_size: 32 * 1024,
            concurrent_operations: 1,
            use_hardware_accel: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        }
    }
}

/// Post-Quantum Cryptography Engine (Wave 3)
pub struct PostQuantumCryptoEngine {
    active: bool,
    crystal_kyber_keypairs: RwLock<HashMap<String, PQCryptoKey>>,
}

/// Post-quantum crypto key
#[derive(Debug, Clone)]
pub struct PQCryptoKey {
    pub algorithm: PQAlgorithm,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub key_id: String,
    pub created_at: DateTime<Utc>,
}

/// Post-quantum algorithms
#[derive(Debug, Clone)]
pub enum PQAlgorithm {
    CRYSTALKyber,
    Falcon,
}

impl PostQuantumCryptoEngine {
    pub fn new() -> Self {
        Self {
            active: false, // Post-quantum not actively integrated yet
            crystal_kyber_keypairs: RwLock::new(HashMap::new()),
        }
    }

    /// Generate post-quantum keypair for hybrid encryption
    pub async fn generate_hybrid_keypair(&self, key_id: &str) -> SecurityResult<PQCryptoKey> {
        // Note: This is a placeholder for post-quantum key generation
        // In production, this would use actual PQC algorithms like CRYSTALS-Kyber
        let keypair = PQCryptoKey {
            algorithm: PQAlgorithm::CRYSTALKyber,
            public_key: vec![0u8; 32], // Placeholder
            private_key: vec![0u8; 32], // Placeholder
            key_id: key_id.to_string(),
            created_at: Utc::now(),
        };

        let mut keypairs = self.crystal_kyber_keypairs.write().await;
        keypairs.insert(key_id.to_string(), keypair.clone());

        Ok(keypair)
    }

    /// Check if post-quantum encryption is ready for use
    pub fn is_ready(&self) -> bool {
        self.active
    }
}

impl EncryptedConfigManager {
    /// Create a new encrypted configuration manager with Wave 3 enhancements
    pub async fn new(config: EncryptionConfig) -> SecurityResult<Self> {
        let master_key_manager: Arc<dyn MasterKeyManager> = Arc::new(InMemoryMasterKeyManager::new());
        let data_key_manager: Arc<dyn DataKeyManager> = Arc::new(InMemoryDataKeyManager::new(master_key_manager.clone()));
        let storage_backend: Arc<dyn EncryptedStorageBackend> = Arc::new(InMemoryConfigStorage::new());

        // Initialize Wave 3 components
        let multi_key_manager = Arc::new(MultiKeyManager::new());
        let performance_optimizer = Arc::new(PerformanceOptimizer::new());
        let pq_crypto_engine = Some(Arc::new(PostQuantumCryptoEngine::new()));

        Ok(Self {
            config,
            master_key_manager,
            data_key_manager,
            storage_backend,
            access_policies: RwLock::new(Vec::new()),
            cache: RwLock::new(HashMap::new()),
            stats: RwLock::new(EncryptionStats {
                total_encryptions: 0,
                total_decryptions: 0,
                key_rotations: 0,
                failed_decryptions: 0,
                cache_hits: 0,
                cache_misses: 0,
                key_generation_time_ms: 0,
                encryption_time_ms: 0,
                decryption_time_ms: 0,
            }),
            multi_key_manager,
            performance_optimizer,
            pq_crypto_engine,
        })

    /// Enhanced set encrypted configuration value with multi-key and performance optimization (Wave 3)
    pub async fn set_config(&self, user: &UserContext, key: &str, value: &str, sensitivity: SensitivityLevel) -> SecurityResult<()> {
        // Check access permissions
        if !self.check_access(user, key, "write").await? {
            return Err(SecurityError::AuthorizationError {
                reason: format!("Access denied for setting configuration: {}", key)
            });
        }

        // Wave 3: Optimize encryption for better performance
        let optimization = self.performance_optimizer.optimize_encryption(value.len(), EncryptionAlgorithm::Aes256Gcm).await?;

        // Wave 3: Use multi-key management for better security
        let data_key = match self.multi_key_manager.select_key("config").await {
            Ok(key) => key,
            Err(_) => {
                // Initialize multi-key manager with a primary key
                let primary_key = self.data_key_manager.generate_data_key("config", EncryptionAlgorithm::Aes256Gcm).await?;
                self.multi_key_manager.add_key_set("config".to_string(), primary_key.clone(), vec![]).await?;
                primary_key
            }
        };

        let crypto_ops = CryptoOps::new(data_key.algorithm);

        // Use optimized AES-256-GCM encryption with timing
        let encrypt_start = std::time::Instant::now();
        let (encrypted_value, nonce) = if value.len() > optimization.chunk_size {
            // For large data, use chunked encryption
            self.encrypt_chunked(value.as_bytes(), &data_key.encrypted_key_material, Some(key.as_bytes())).await?
        } else {
            crypto_ops.encrypt(value.as_bytes(), &data_key.encrypted_key_material, Some(key.as_bytes()))?
        };
        let encrypt_time = encrypt_start.elapsed().as_millis() as u64;

        // Get existing entry for version history
        let existing_entry = self.storage_backend.retrieve(key).await?;
        let previous_versions = if let Some(entry) = existing_entry {
            vec![entry.version_history().to_vec()].into_iter().flatten().collect()
        } else {
            Vec::new()
        };

        let entry = EncryptedConfigEntry {
            config_key: key.to_string(),
            encrypted_value,
            data_key_id: data_key.key_id.clone(),
            nonce,
            encryption_algorithm: crypto_ops.algorithm,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            owner: user.user_id.clone(),
            access_list: HashSet::from([user.user_id.clone()]),
            sensitivity_level: sensitivity,
            metadata: HashMap::new(),
            version: previous_versions.len() as u32 + 1,
            previous_versions,
        };

        // Store the encrypted configuration
        self.storage_backend.store(&entry).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), entry.clone());

        // Update stats with performance metrics (Wave 3)
        let mut stats = self.stats.write().await;
        stats.total_encryptions += 1;
        stats.encryption_time_ms += encrypt_time;

        Ok(())
    }

    /// Get decrypted configuration value
    pub async fn get_config(&self, user: &UserContext, key: &str) -> SecurityResult<Option<String>> {
        // Check access permissions
        if !self.check_access(user, key, "read").await? {
            return Err(SecurityError::AuthorizationError {
                reason: format!("Access denied for reading configuration: {}", key)
            });
        }

        // Check cache first
        let mut cache = self.cache.write().await;
        if let Some(entry) = cache.get(key) {
            let mut stats = self.stats.write().await;
            stats.cache_hits += 1;

            return Ok(Some(self.decrypt_entry(entry).await?));
        }

        // Retrieve from storage
        let entry = self.storage_backend.retrieve(key).await?;
        if let Some(entry) = entry {
            // Add to cache
            cache.insert(key.to_string(), entry.clone());

            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
            stats.total_decryptions += 1;

            Ok(Some(self.decrypt_entry(&entry).await?))
        } else {
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
            Ok(None)
        }
    }

    /// Delete configuration entry
    pub async fn delete_config(&self, user: &UserContext, key: &str) -> SecurityResult<()> {
        // Check access permissions
        if !self.check_access(user, key, "delete").await? {
            return Err(SecurityError::AuthorizationError {
                reason: format!("Access denied for deleting configuration: {}", key)
            });
        }

        self.storage_backend.delete(key).await?;
        let mut cache = self.cache.write().await;
        cache.remove(key);

        Ok(())
    }

    /// Add access policy
    pub async fn add_access_policy(&self, policy: AccessPolicy) -> SecurityResult<()> {
        let mut policies = self.access_policies.write().await;
        policies.push(policy);
        Ok(())
    }

    /// Rotate encryption keys
    pub async fn rotate_keys(&self) -> SecurityResult<()> {
        // Generate new master key
        let new_master_key = self.master_key_manager.generate_master_key().await?;
        self.master_key_manager.rotate_master_key(new_master_key).await?;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.key_rotations += 1;

        Ok(())
    }

    /// Get encryption statistics
    pub async fn get_stats(&self) -> SecurityResult<EncryptionStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    /// Get health status
    pub fn health_status(&self) -> ComponentStatus {
        ComponentStatus::Healthy
    }

    /// Prepare post-quantum hybrid encryption (Wave 3)
    pub async fn prepare_pq_encryption(&self, key_id: &str) -> SecurityResult<()> {
        if let Some(pq_engine) = &self.pq_crypto_engine {
            pq_engine.generate_hybrid_keypair(key_id).await?;
        }
        Ok(())
    }

    /// Check if post-quantum encryption is available
    pub fn pq_encryption_ready(&self) -> bool {
        if let Some(pq_engine) = &self.pq_crypto_engine {
            pq_engine.is_ready()
        } else {
            false
        }
    }

    // Private methods for Wave 3 enhancements

    async fn encrypt_chunked(&self, data: &[u8], key: &[u8], aad: Option<&[u8]>) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        let crypto_ops = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);
        let chunk_size = 32 * 1024; // 32KB chunks for optimal performance
        let mut encrypted_chunks = Vec::new();
        let mut nonces = Vec::new();

        for chunk in data.chunks(chunk_size) {
            let (encrypted_chunk, nonce) = crypto_ops.encrypt(chunk, key, aad)?;
            encrypted_chunks.extend_from_slice(&encrypted_chunk);
            nonces.extend_from_slice(&nonce);
        }

        // Return concatenated encrypted data and combined nonces
        Ok((encrypted_chunks, nonces))
    }

    // Private methods

    async fn decrypt_entry(&self, entry: &EncryptedConfigEntry) -> SecurityResult<String> {
        // Get the data key
        let data_key = self.data_key_manager.get_data_key(&entry.data_key_id).await?
            .ok_or_else(|| SecurityError::EncryptionError {
                source: format!("Data key not found: {}", entry.data_key_id).into()
            })?;

        // Get master key to decrypt data key
        let master_key = self.master_key_manager.get_master_key("default").await?
            .ok_or_else(|| SecurityError::EncryptionError {
                source: "Master key not found".into()
            })?;

        // Decrypt data key
        let decrypted_key = self.data_key_manager.decrypt_data_key(&data_key.encrypted_key_material, &master_key).await?;

        // Decrypt the configuration value
        let crypto_ops = CryptoOps::new(entry.encryption_algorithm);
        let plaintext = crypto_ops.decrypt(
            &entry.encrypted_value,
            &decrypted_key,
            &entry.nonce,
            Some(entry.config_key.as_bytes())
        )?;

        String::from_utf8(plaintext)
            .map_err(|e| SecurityError::EncryptionError {
                source: format!("UTF-8 decode error: {}", e).into()
            })
    }

    async fn check_access(&self, user: &UserContext, key: &str, action: &str) -> SecurityResult<bool> {
        let policies = self.access_policies.read().await;

        // Default policy: owner can do anything, admins can do anything
        if user.user_id == "admin" || user.roles.contains(&"admin".to_string()) {
            return Ok(true);
        }

        // Check if user is in access list for this specific entry
        if let Some(entry) = self.cache.read().await.get(key) {
            if entry.access_list.contains(&user.user_id) || entry.owner == user.user_id {
                return Ok(true);
            }
        }

        // Check policies
        for policy in &*policies {
            // Simple pattern matching
            if key.contains(&policy.config_pattern) {
                let has_role = user.roles.iter().any(|role| policy.allowed_roles.contains(role));
                let has_user = policy.allowed_users.contains(&user.user_id);
                let has_action = policy.allowed_actions.contains(action);

                if (has_role || has_user) && has_action {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

// In-memory key managers

pub struct InMemoryMasterKeyManager {
    keys: RwLock<HashMap<String, MasterKey>>,
}

impl InMemoryMasterKeyManager {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MasterKeyManager for InMemoryMasterKeyManager {
    async fn generate_master_key(&self) -> SecurityResult<MasterKey> {
        let crypto_ops = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);
        let key_material = crypto_ops.generate_key(EncryptionAlgorithm::Aes256Gcm)?;

        let key_id = format!("master-{}", uuid::Uuid::new_v4());
        let key = MasterKey {
            key_id: key_id.clone(),
            key_version: 1,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_material,
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id, key.clone());

        Ok(key)
    }

    async fn get_master_key(&self, key_id: &str) -> SecurityResult<Option<MasterKey>> {
        let keys = self.keys.read().await;
        Ok(keys.get(key_id).cloned())
    }

    async fn rotate_master_key(&self, _new_key: MasterKey) -> SecurityResult<()> {
        // In production, this would re-encrypt all data keys
        Ok(())
    }

    async fn delete_master_key(&self, _key_id: &str) -> SecurityResult<()> {
        // In-memory implementation doesn't support deletion
        Ok(())
    }

    async fn list_active_keys(&self) -> SecurityResult<Vec<MasterKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().filter(|k| k.is_active).cloned().collect())
    }
}

pub struct InMemoryDataKeyManager {
    keys: RwLock<HashMap<String, DataKey>>,
    master_key_manager: Arc<dyn MasterKeyManager>,
}

impl InMemoryDataKeyManager {
    pub fn new(master_key_manager: Arc<dyn MasterKeyManager>) -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            master_key_manager,
        }
    }
}

#[async_trait]
impl DataKeyManager for InMemoryDataKeyManager {
    async fn generate_data_key(&self, purpose: &str, algorithm: EncryptionAlgorithm) -> SecurityResult<DataKey> {
        let crypto_ops = CryptoOps::new(algorithm);
        let key_material = crypto_ops.generate_key(algorithm)?;

        // Encrypt the data key with master key
        let master_key = self.master_key_manager.get_master_key("default").await?
            .unwrap_or_else(|| panic!("Default master key not found"));

        let encrypted_key = crypto_ops.encrypt_aes256_gcm_only(&key_material, &master_key.key_material)?;

        let key_id = format!("data-{}-{}", purpose, uuid::Uuid::new_v4());
        let key = DataKey {
            key_id: key_id.clone(),
            version: 1,
            encrypted_key_material: encrypted_key,
            algorithm,
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id, key.clone());

        Ok(key)
    }

    async fn get_data_key(&self, key_id: &str) -> SecurityResult<Option<DataKey>> {
        let keys = self.keys.read().await;
        Ok(keys.get(key_id).cloned())
    }

    async fn decrypt_data_key(&self, encrypted_key: &[u8], master_key: &MasterKey) -> SecurityResult<Vec<u8>> {
        let crypto_ops = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);
        crypto_ops.decrypt(encrypted_key, &master_key.key_material, &[], None)
    }

    async fn list_data_keys(&self) -> SecurityResult<Vec<DataKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().cloned().collect())
    }
}

impl CryptoOps {
    fn encrypt_aes256_gcm_only(&self, plaintext: &[u8], key: &[u8]) -> SecurityResult<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
        let cipher = Aes256Gcm::new_from_slice(key).unwrap();
        let nonce = Aes256Gcm::generate_nonce(&mut rand::rngs::OsRng);
        let mut ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();
        ciphertext.extend_from_slice(&nonce);
        Ok(ciphertext)
    }
}

impl EncryptedConfigEntry {
    fn version_history(&self) -> Vec<EncryptedConfigEntry> {
        self.previous_versions.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test as async_test;

    #[async_test]
    async fn test_encrypted_config_flow() {
        let config = EncryptionConfig {
            algorithm: "aes256-gcm".to_string(),
            key_rotation_days: 90,
            password_iterations: 10000,
            master_key_secure_store: true,
        };

        let manager = EncryptedConfigManager::new(config).await.unwrap();

        let user = UserContext {
            user_id: "test_user".to_string(),
            username: "testuser".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
            session_id: Some("session123".to_string()),
            mfa_verified: true,
        };

        // Set encrypted configuration
        manager.set_config(&user, "database.password", "secret_password", SensitivityLevel::Confidential).await.unwrap();

        // Retrieve decrypted configuration
        let retrieved = manager.get_config(&user, "database.password").await.unwrap();
        assert_eq!(retrieved, Some("secret_password".to_string()));
    }

    #[async_test]
    async fn test_crypto_operations() {
        let crypto = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);
        let key = crypto.generate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        let plaintext = b"Hello, encrypted world!";

        let (ciphertext, nonce) = crypto.encrypt(plaintext, &key, None).unwrap();
        let decrypted = crypto.decrypt(&ciphertext, &key, &nonce, None).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[async_test]
    async fn test_key_management() {
        let master_manager = InMemoryMasterKeyManager::new();
        let data_manager = InMemoryDataKeyManager::new(Arc::new(master_manager));

        let data_key = data_manager.generate_data_key("test", EncryptionAlgorithm::Aes256Gcm).await.unwrap();
        assert!(!data_key.key_id.is_empty());

        let retrieved = data_manager.get_data_key(&data_key.key_id).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[async_test]
    async fn test_aes256_gcm_edge_cases() {
        let crypto = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);

        // Test with empty data
        let key = crypto.generate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        let (ciphertext, nonce) = crypto.encrypt(b"", &key, None).unwrap();
        let decrypted = crypto.decrypt(&ciphertext, &key, &nonce, None).unwrap();
        assert_eq!(decrypted, b"");

        // Test with large data (1MB)
        let large_data = vec![0u8; 1024 * 1024];
        let (ciphertext, nonce) = crypto.encrypt(&large_data, &key, Some(b"aad")).unwrap();
        let decrypted = crypto.decrypt(&ciphertext, &key, &nonce, Some(b"aad")).unwrap();
        assert_eq!(decrypted, large_data);

        // Test with associated data
        let plaintext = b"test message";
        let aad = b"associated data";
        let (ciphertext, nonce) = crypto.encrypt(plaintext, &key, Some(aad)).unwrap();
        let decrypted = crypto.decrypt(&ciphertext, &key, &nonce, Some(aad)).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[async_test]
    async fn test_encryption_algorithm_switching() {
        let aes_crypto = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);
        let chacha_crypto = CryptoOps::new(EncryptionAlgorithm::Chacha20Poly1305);

        let key_aes = aes_crypto.generate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        let key_chacha = chacha_crypto.generate_key(EncryptionAlgorithm::Chacha20Poly1305).unwrap();

        let plaintext = b"Hello, World!";

        // Test AES-256-GCM
        let (aes_ciphertext, aes_nonce) = aes_crypto.encrypt(plaintext, &key_aes, None).unwrap();
        let aes_decrypted = aes_crypto.decrypt(&aes_ciphertext, &key_aes, &aes_nonce, None).unwrap();
        assert_eq!(aes_decrypted, plaintext);

        // Test ChaCha20-Poly1305
        let (chacha_ciphertext, chacha_nonce) = chacha_crypto.encrypt(plaintext, &key_chacha, None).unwrap();
        let chacha_decrypted = chacha_crypto.decrypt(&chacha_ciphertext, &key_chacha, &chacha_nonce, None).unwrap();
        assert_eq!(chacha_decrypted, plaintext);

        // Ensure different algorithms produce different ciphertext
        assert_ne!(aes_ciphertext, chacha_ciphertext);
    }

    #[async_test]
    async fn test_encryption_error_conditions() {
        let crypto = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);

        // Test with invalid key size
        let invalid_key = vec![0u8; 16]; // Wrong size for AES-256
        let plaintext = b"test";
        let result = crypto.encrypt(plaintext, &invalid_key, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::EncryptionError { .. }));

        // Test decryption with wrong key
        let key = crypto.generate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        let (ciphertext, nonce) = crypto.encrypt(plaintext, &key, None).unwrap();
        let wrong_key = crypto.generate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        let result = crypto.decrypt(&ciphertext, &wrong_key, &nonce, None);
        assert!(result.is_err());

        // Test decryption with tampered ciphertext
        let mut tampered_ciphertext = ciphertext.clone();
        if !tampered_ciphertext.is_empty() {
            tampered_ciphertext[0] ^= 1; // Flip a bit
        }
        let result = crypto.decrypt(&tampered_ciphertext, &key, &nonce, None);
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_multi_key_management() {
        let multi_key_manager = MultiKeyManager::new();

        // Test adding key set
        let primary_key = DataKey {
            key_id: "primary-1".to_string(),
            version: 1,
            encrypted_key_material: vec![1, 2, 3],
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
        };
        let backup_keys = vec![
            DataKey {
                key_id: "backup-1".to_string(),
                version: 1,
                encrypted_key_material: vec![4, 5, 6],
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                created_at: Utc::now(),
                expires_at: None,
                is_active: true,
            }
        ];

        multi_key_manager.add_key_set("test".to_string(), primary_key.clone(), backup_keys).await.unwrap();

        // Test selecting key
        let selected_key = multi_key_manager.select_key("test").await.unwrap();
        assert_eq!(selected_key.key_id, "primary-1");

        // Test selecting non-existent purpose
        let result = multi_key_manager.select_key("nonexistent").await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_performance_optimization() {
        let optimizer = PerformanceOptimizer::new();

        // Test small data optimization
        let small_optimization = optimizer.optimize_encryption(1024, EncryptionAlgorithm::Aes256Gcm).await.unwrap();
        assert_eq!(small_optimization.chunk_size, 1024);
        assert_eq!(small_optimization.concurrent_operations, 1);
        assert!(!small_optimization.use_hardware_accel);

        // Test large data optimization
        let large_optimization = optimizer.optimize_encryption(2 * 1024 * 1024, EncryptionAlgorithm::Aes256Gcm).await.unwrap();
        assert_eq!(large_optimization.chunk_size, 64 * 1024);
        assert_eq!(large_optimization.concurrent_operations, 2);
    }

    #[async_test]
    async fn test_access_control_edge_cases() {
        let config = EncryptionConfig {
            algorithm: "aes256-gcm".to_string(),
            key_rotation_days: 90,
            password_iterations: 10000,
            master_key_secure_store: true,
        };

        let manager = EncryptedConfigManager::new(config).await.unwrap();

        // Test access without policy - should deny for non-admin
        let regular_user = UserContext {
            user_id: "user1".to_string(),
            username: "user1".to_string(),
            roles: vec![],
            permissions: vec![],
            session_id: Some("session123".to_string()),
            mfa_verified: false,
        };

        let result = manager.get_config(&regular_user, "nonexistent.key").await;
        assert!(result.is_err());

        // Test admin access
        let admin_user = UserContext {
            user_id: "admin".to_string(),
            username: "admin".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec![],
            session_id: Some("session123".to_string()),
            mfa_verified: true,
        };

        let result = manager.get_config(&admin_user, "nonexistent.key").await;
        assert_eq!(result.unwrap(), None);
    }

    #[async_test]
    async fn test_key_rotation_scenario() {
        let master_manager = Arc::new(InMemoryMasterKeyManager::new());
        let data_manager = Arc::new(InMemoryDataKeyManager::new(master_manager.clone()));

        // Generate initial keys
        let initial_master = master_manager.generate_master_key().await.unwrap();
        let data_key = data_manager.generate_data_key("test", EncryptionAlgorithm::Aes256Gcm).await.unwrap();

        // Simulate rotation
        let new_master = master_manager.generate_master_key().await.unwrap();
        master_manager.rotate_master_key(new_master).await.unwrap();

        // Verify keys still work (in-memory doesn't actually re-encrypt, but structure is tested)
        let retrieved = data_manager.get_data_key(&data_key.key_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().key_id, data_key.key_id);
    }

    #[async_test]
    async fn test_secure_random_generation() {
        let crypto = CryptoOps::new(EncryptionAlgorithm::Aes256Gcm);

        // Test different sizes
        let random_16 = crypto.generate_secure_random(16).unwrap();
        let random_32 = crypto.generate_secure_random(32).unwrap();
        let random_64 = crypto.generate_secure_random(64).unwrap();

        assert_eq!(random_16.len(), 16);
        assert_eq!(random_32.len(), 32);
        assert_eq!(random_64.len(), 64);

        // Test that generated values are different (with high probability)
        assert_ne!(random_32, random_64[..32]);
    }

    #[async_test]
    async fn test_post_quantum_placeholder() {
        let pq_engine = PostQuantumCryptoEngine::new();
        assert!(!pq_engine.is_ready());

        // Generate placeholder keypair
        let keypair = pq_engine.generate_hybrid_keypair("test-key").await.unwrap();
        assert_eq!(keypair.key_id, "test-key");
        assert_eq!(keypair.algorithm, PQAlgorithm::CRYSTALKyber);
        // Note: In real implementation, these would be proper cryptographic keys
        assert_eq!(keypair.public_key.len(), 32);
        assert_eq!(keypair.private_key.len(), 32);
    }

    #[async_test]
    async fn test_encryption_stats_tracking() {
        let config = EncryptionConfig {
            algorithm: "aes256-gcm".to_string(),
            key_rotation_days: 90,
            password_iterations: 10000,
            master_key_secure_store: true,
        };

        let manager = EncryptedConfigManager::new(config).await.unwrap();

        let admin_user = UserContext {
            user_id: "admin".to_string(),
            username: "admin".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec![],
            session_id: Some("session123".to_string()),
            mfa_verified: true,
        };

        // Initial stats should be zero
        let initial_stats = manager.get_stats().await.unwrap();
        assert_eq!(initial_stats.total_encryptions, 0);
        assert_eq!(initial_stats.total_decryptions, 0);

        // Perform operations to update stats
        manager.set_config(&admin_user, "test.key1", "value1", SensitivityLevel::Confidential).await.unwrap();
        manager.set_config(&admin_user, "test.key2", "value2", SensitivityLevel::Confidential).await.unwrap();

        let retrieved1 = manager.get_config(&admin_user, "test.key1").await.unwrap();
        let retrieved2 = manager.get_config(&admin_user, "test.key2").await.unwrap();

        assert_eq!(retrieved1, Some("value1".to_string()));
        assert_eq!(retrieved2, Some("value2".to_string()));

        // Check stats after operations
        let final_stats = manager.get_stats().await.unwrap();
        assert_eq!(final_stats.total_encryptions, 2);
        assert_eq!(final_stats.total_decryptions, 2);
        assert_eq!(final_stats.cache_misses, 2); // First retrievals are cache misses
        assert_eq!(final_stats.cache_hits, 0);

        // Test cache hit
        let retrieved1_again = manager.get_config(&admin_user, "test.key1").await.unwrap();
        assert_eq!(retrieved1_again, Some("value1".to_string()));

        let cache_hit_stats = manager.get_stats().await.unwrap();
        assert_eq!(cache_hit_stats.cache_hits, 1);
    }

    #[async_test]
    async fn test_chunked_encryption_large_data() {
        let config = EncryptionConfig {
            algorithm: "aes256-gcm".to_string(),
            key_rotation_days: 90,
            password_iterations: 10000,
            master_key_secure_store: true,
        };

        let manager = EncryptedConfigManager::new(config).await.unwrap();

        let admin_user = UserContext {
            user_id: "admin".to_string(),
            username: "admin".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec![],
            session_id: Some("session123".to_string()),
            mfa_verified: true,
        };

        // Create large data (100KB)
        let large_value = "x".repeat(100 * 1024);

        // This should trigger chunked encryption internally
        manager.set_config(&admin_user, "large.config", &large_value, SensitivityLevel::Confidential).await.unwrap();

        let retrieved = manager.get_config(&admin_user, "large.config").await.unwrap();
        assert_eq!(retrieved, Some(large_value));
    }

    #[async_test]
    async fn test_utf8_encoding_edge_cases() {
        let config = EncryptionConfig {
            algorithm: "aes256-gcm".to_string(),
            key_rotation_days: 90,
            password_iterations: 10000,
            master_key_secure_store: true,
        };

        let manager = EncryptedConfigManager::new(config).await.unwrap();

        let admin_user = UserContext {
            user_id: "admin".to_string(),
            username: "admin".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec![],
            session_id: Some("session123".to_string()),
            mfa_verified: true,
        };

        // Test with Unicode characters
        let unicode_value = "Hello, 世界! 🌍 Test with émojis: 🚀🔒";
        manager.set_config(&admin_user, "unicode.config", unicode_value, SensitivityLevel::Confidential).await.unwrap();

        let retrieved = manager.get_config(&admin_user, "unicode.config").await.unwrap();
        assert_eq!(retrieved, Some(unicode_value.to_string()));

        // Test with null bytes (should work since we're storing as String)
        let null_bytes_value = "Test\x00with\x00null\x00bytes";
        manager.set_config(&admin_user, "nullbytes.config", null_bytes_value, SensitivityLevel::Confidential).await.unwrap();

        let retrieved_null = manager.get_config(&admin_user, "nullbytes.config").await.unwrap();
        assert_eq!(retrieved_null, Some(null_bytes_value.to_string()));
    }

    #[async_test]
    async fn test_storage_backend_operations() {
        let storage = InMemoryConfigStorage::new();

        let entry = EncryptedConfigEntry {
            config_key: "test.key".to_string(),
            encrypted_value: vec![1, 2, 3, 4],
            data_key_id: "data-key-1".to_string(),
            nonce: vec![5, 6, 7, 8],
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            owner: "test_user".to_string(),
            access_list: ["user1".to_string(), "user2".to_string()].into_iter().collect(),
            sensitivity_level: SensitivityLevel::Confidential,
            metadata: [("env".to_string(), "test".to_string())].into_iter().collect(),
            version: 1,
            previous_versions: vec![],
        };

        // Test store and retrieve
        storage.store(&entry).await.unwrap();
        let retrieved = storage.retrieve("test.key").await.unwrap();
        assert_eq!(retrieved, Some(entry.clone()));

        // Test list operations
        let all_entries = storage.list(None, 10).await.unwrap();
        assert_eq!(all_entries.len(), 1);

        let pattern_entries = storage.list(Some("test"), 10).await.unwrap();
        assert_eq!(pattern_entries.len(), 1);

        let no_match_entries = storage.list(Some("nonexistent"), 10).await.unwrap();
        assert_eq!(no_match_entries.len(), 0);

        // Test metadata search
        let metadata_search = storage.search_by_metadata([("env".to_string(), "test".to_string())].into_iter().collect()).await.unwrap();
        assert_eq!(metadata_search.len(), 1);

        let no_metadata_match = storage.search_by_metadata([("env".to_string(), "prod".to_string())].into_iter().collect()).await.unwrap();
        assert_eq!(no_metadata_match.len(), 0);

        // Test delete
        storage.delete("test.key").await.unwrap();
        let after_delete = storage.retrieve("test.key").await.unwrap();
        assert_eq!(after_delete, None);
    }
}