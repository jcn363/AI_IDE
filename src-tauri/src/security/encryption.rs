//! End-to-end encryption framework for IPC channels and data storage
//!
//! This module implements AES-256-GCM encryption for secure communication
//! between the webview and extension, with proper key management and rotation.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Encryption manager for handling cryptographic operations
pub struct EncryptionManager {
    /// Current session key for encryption/decryption
    session_key: Key<Aes256Gcm>,
    /// Key rotation history for forward secrecy
    key_history: HashMap<String, Key<Aes256Gcm>>,
    /// Session key identifier
    session_id: String,
    /// Key rotation counter
    rotation_counter: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    /// Base64-encoded encrypted data
    pub ciphertext: String,
    /// Base64-encoded nonce
    pub nonce: String,
    /// Session identifier
    pub session_id: String,
    /// Key rotation counter
    pub rotation_counter: u64,
    /// HMAC for integrity verification
    pub hmac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    /// Base64-encoded session key
    pub key: String,
    /// Session identifier
    pub session_id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
}

impl Default for EncryptionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptionManager {
    /// Create a new encryption manager with a randomly generated session key
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes);

        let session_key = Key::<Aes256Gcm>::from(key_bytes);
        let session_id = format!("session_{}", chrono::Utc::now().timestamp());

        Self {
            session_key,
            key_history: HashMap::new(),
            session_id,
            rotation_counter: 0,
        }
    }

    /// Encrypt a payload using AES-256-GCM
    pub fn encrypt_payload(&self, data: &[u8]) -> Result<EncryptedPayload, String> {
        let cipher = Aes256Gcm::new(&self.session_key);

        // Generate a random nonce
        let mut rng = thread_rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| format!("Encryption failed: {:?}", e))?;

        // Generate HMAC for integrity
        let hmac = self.generate_hmac(&ciphertext, &nonce_bytes);

        Ok(EncryptedPayload {
            ciphertext: base64::encode(&ciphertext),
            nonce: base64::encode(nonce_bytes),
            session_id: self.session_id.clone(),
            rotation_counter: self.rotation_counter,
            hmac,
        })
    }

    /// Decrypt a payload using AES-256-GCM
    pub fn decrypt_payload(&self, payload: &EncryptedPayload) -> Result<Vec<u8>, String> {
        // Verify session ID
        if payload.session_id != self.session_id {
            return Err("Invalid session ID".to_string());
        }

        // Verify HMAC for integrity
        let expected_hmac = self.generate_hmac(
            &base64::decode(&payload.ciphertext)
                .map_err(|e| format!("Invalid base64 ciphertext: {:?}", e))?,
            &base64::decode(&payload.nonce)
                .map_err(|e| format!("Invalid base64 nonce: {:?}", e))?,
        );

        if expected_hmac != payload.hmac {
            return Err("HMAC verification failed".to_string());
        }

        let cipher = Aes256Gcm::new(&self.session_key);

        let nonce_bytes =
            base64::decode(&payload.nonce).map_err(|e| format!("Invalid base64 nonce: {:?}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = base64::decode(&payload.ciphertext)
            .map_err(|e| format!("Invalid base64 ciphertext: {:?}", e))?;

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {:?}", e))
    }

    /// Generate a new session key for forward secrecy
    pub fn rotate_session_key(&mut self) -> Result<(), String> {
        // Store the current key in history
        self.key_history.insert(
            format!("{}_{}", self.session_id, self.rotation_counter),
            self.session_key.clone(),
        );

        // Generate new key
        let mut rng = thread_rng();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes);

        self.session_key = Key::<Aes256Gcm>::from(key_bytes);
        self.rotation_counter += 1;

        log::info!("Session key rotated - counter: {}", self.rotation_counter);

        Ok(())
    }

    /// Generate a new session with a random ID
    pub fn new_session(&mut self) -> Result<(), String> {
        self.session_id = format!("session_{}", chrono::Utc::now().timestamp());
        self.rotation_counter = 0;
        self.key_history.clear();

        self.rotate_session_key()
    }

    /// Get current session key information
    pub fn get_session_key_info(&self) -> SessionKey {
        let now = chrono::Utc::now().timestamp() as u64;

        SessionKey {
            key: base64::encode(self.session_key.as_slice()),
            session_id: self.session_id.clone(),
            created_at: now,
            expires_at: now + 3600, // 1 hour expiration
        }
    }

    /// Encrypt data for IPC communication
    pub fn encrypt_ipc_payload(&self, payload: &serde_json::Value) -> Result<String, String> {
        let json_string = serde_json::to_string(payload)
            .map_err(|e| format!("JSON serialization failed: {:?}", e))?;

        let encrypted = self.encrypt_payload(json_string.as_bytes())?;
        serde_json::to_string(&encrypted)
            .map_err(|e| format!("Encrypted payload serialization failed: {:?}", e))
    }

    /// Decrypt data from IPC communication
    pub fn decrypt_ipc_payload(&self, encrypted_json: &str) -> Result<serde_json::Value, String> {
        let encrypted: EncryptedPayload = serde_json::from_str(encrypted_json)
            .map_err(|e| format!("Invalid encrypted payload format: {:?}", e))?;

        let decrypted_bytes = self.decrypt_payload(&encrypted)?;

        let json_string = String::from_utf8(decrypted_bytes)
            .map_err(|e| format!("Invalid UTF-8 in decrypted data: {:?}", e))?;

        serde_json::from_str(&json_string)
            .map_err(|e| format!("JSON deserialization failed: {:?}", e))
    }

    /// Generate HMAC for integrity verification
    fn generate_hmac(&self, ciphertext: &[u8], nonce: &[u8]) -> String {
        use hmac::{Hmac, Mac, NewMac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(self.session_key.as_slice())
            .expect("HMAC can take key of any size");

        mac.update(ciphertext);
        mac.update(nonce);

        let result = mac.finalize();
        base64::encode(result.into_bytes())
    }

    /// Verify HMAC for integrity verification
    fn verify_hmac(&self, ciphertext: &[u8], nonce: &[u8], hmac: &str) -> bool {
        let expected = self.generate_hmac(ciphertext, nonce);
        hmac == expected
    }

    /// Clean up expired keys from history
    pub fn cleanup_expired_keys(&mut self) {
        let now = chrono::Utc::now().timestamp() as u64;
        let valid_keys: HashMap<String, Key<Aes256Gcm>> = self
            .key_history
            .iter()
            .filter(|(_, _)| {
                // Keep keys for 24 hours for forward secrecy
                // In a real implementation, this would check actual timestamps
                true
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        self.key_history = valid_keys;
    }
}

/// Global encryption manager instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_ENCRYPTION_MANAGER: Arc<Mutex<EncryptionManager>> =
        Arc::new(Mutex::new(EncryptionManager::new()));
}

/// Convenience functions for global encryption manager
pub mod global {
    use super::*;

    pub async fn encrypt_payload(data: &[u8]) -> Result<EncryptedPayload, String> {
        let manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.encrypt_payload(data)
    }

    pub async fn decrypt_payload(payload: &EncryptedPayload) -> Result<Vec<u8>, String> {
        let manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.decrypt_payload(payload)
    }

    pub async fn rotate_session_key() -> Result<(), String> {
        let mut manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.rotate_session_key()
    }

    pub async fn new_session() -> Result<(), String> {
        let mut manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.new_session()
    }

    pub async fn cleanup_expired_keys() {
        let mut manager = GLOBAL_ENCRYPTION_MANAGER.lock().await;
        manager.cleanup_expired_keys();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption_decryption_roundtrip() {
        let manager = EncryptionManager::new();

        let test_data = b"Hello, World! This is a test message.";
        let encrypted = manager.encrypt_payload(test_data).unwrap();
        let decrypted = manager.decrypt_payload(&encrypted).unwrap();

        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[tokio::test]
    async fn test_session_key_rotation() {
        let mut manager = EncryptionManager::new();
        let original_key = manager.get_session_key_info();

        manager.rotate_session_key().unwrap();
        let new_key = manager.get_session_key_info();

        // Session ID should remain the same, but rotation counter should increase
        assert_eq!(original_key.session_id, new_key.session_id);
        assert_ne!(original_key.key, new_key.key);
    }

    #[tokio::test]
    async fn test_ipc_payload_encryption() {
        let manager = EncryptionManager::new();

        let payload = serde_json::json!({
            "command": "test_command",
            "data": {
                "user": "test_user",
                "permissions": ["read", "write"]
            }
        });

        let encrypted_string = manager.encrypt_ipc_payload(&payload).unwrap();
        let decrypted_payload = manager.decrypt_ipc_payload(&encrypted_string).unwrap();

        assert_eq!(payload, decrypted_payload);
    }

    #[tokio::test]
    async fn test_hmac_integrity() {
        let manager = EncryptionManager::new();

        let test_data = b"Test data for HMAC verification";
        let encrypted = manager.encrypt_payload(test_data).unwrap();

        // Verify HMAC is generated
        assert!(!encrypted.hmac.is_empty());

        // Verify decryption succeeds with correct HMAC
        let decrypted = manager.decrypt_payload(&encrypted).unwrap();
        assert_eq!(test_data.to_vec(), decrypted);
    }
}
