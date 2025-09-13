//! Plugin registry implementation for marketplace signature verification.
//!
//! This module provides signature verification functionality for plugins in the marketplace,
//! ensuring authenticity and integrity of plugin downloads and installations.

use std::collections::HashMap;

use uuid::Uuid;

use crate::interfaces::PluginError;

/// Type alias for plugin IDs in the marketplace
pub type PluginId = Uuid;

/// Signature verification result
pub type VerificationResult = Result<(), PluginError>;

/// Trait for plugin registry operations dealing with marketplace verification
#[async_trait::async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Verify signature for plugin authenticity
    async fn verify_signature(&self, plugin_id: PluginId, signature: &str, bytes: &[u8]) -> VerificationResult;

    /// Add a public key for a plugin publisher
    async fn add_publisher_key(&self, plugin_id: PluginId, public_key: &str) -> Result<(), PluginError>;

    /// Remove a publisher key for a plugin
    async fn remove_publisher_key(&self, plugin_id: PluginId) -> Result<(), PluginError>;

    /// Check if a plugin is trusted (has valid public key)
    async fn is_trusted_publisher(&self, plugin_id: PluginId) -> bool;

    /// Get all trusted plugin IDs
    async fn get_trusted_plugins(&self) -> Vec<PluginId>;
}

/// Concrete implementation of PluginRegistry
pub struct PluginRegistryImpl {
    /// Map of plugin IDs to their public keys
    publisher_keys: HashMap<PluginId, String>,
}

impl PluginRegistryImpl {
    /// Create a new registry instance
    pub fn new() -> Self {
        Self {
            publisher_keys: HashMap::new(),
        }
    }

    /// Create a registry with initial trusted publishers
    pub fn with_trusted_publishers(publishers: HashMap<PluginId, String>) -> Self {
        Self {
            publisher_keys: publishers,
        }
    }
}

#[async_trait::async_trait]
impl PluginRegistry for PluginRegistryImpl {
    async fn verify_signature(&self, plugin_id: PluginId, signature: &str, bytes: &[u8]) -> VerificationResult {
        // Get the public key for this plugin
        let public_key = self
            .publisher_keys
            .get(&plugin_id)
            .ok_or_else(|| PluginError::Other(format!("No public key found for plugin {}", plugin_id)))?;

        // Verify the signature - this is a simplified implementation
        // In a real implementation, this would use proper cryptographic verification
        match verify_signature_with_key(public_key, signature, bytes) {
            Ok(true) => Ok(()),
            Ok(false) => Err(PluginError::Other(format!(
                "Signature verification failed for plugin {}",
                plugin_id
            ))),
            Err(e) => Err(PluginError::Other(format!(
                "Signature verification error: {}",
                e
            ))),
        }
    }

    async fn add_publisher_key(&self, plugin_id: PluginId, public_key: &str) -> Result<(), PluginError> {
        // This implementation doesn't modify the registry as it's read-only for verification
        // In practice, this would interact with a persistent store
        Err(PluginError::Other(
            "Registry modification not implemented in this version".to_string(),
        ))
    }

    async fn remove_publisher_key(&self, plugin_id: PluginId) -> Result<(), PluginError> {
        // This implementation doesn't modify the registry as it's read-only for verification
        // In practice, this would interact with a persistent store
        Err(PluginError::Other(
            "Registry modification not implemented in this version".to_string(),
        ))
    }

    async fn is_trusted_publisher(&self, plugin_id: PluginId) -> bool {
        self.publisher_keys.contains_key(&plugin_id)
    }

    async fn get_trusted_plugins(&self) -> Vec<PluginId> {
        self.publisher_keys.keys().cloned().collect()
    }
}

impl Default for PluginRegistryImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified signature verification function
/// In a real implementation, this would use proper cryptographic libraries
/// like ring or rust-crypto for proper signature verification
fn verify_signature_with_key(_public_key: &str, _signature: &str, _bytes: &[u8]) -> Result<bool, PluginError> {
    // This is a placeholder implementation
    // Real implementation would use cryptographic verification
    //
    // Example real implementation:
    // use ring::signature::{self, RSA_PKCS1_SHA256};
    //
    // match signature::UnparsedPublicKey::new(&RSA_PKCS1_SHA256, public_key_bytes)
    //     .verify(bytes, signature_bytes) {
    //         Ok(()) => Ok(true),
    //         Err(_) => Ok(false),
    //     }

    // For now, just return true for demo purposes
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_verification() {
        let registry = PluginRegistryImpl::new();
        let plugin_id = Uuid::new_v4();

        // Without a public key, verification should fail
        let result = registry.verify_signature(plugin_id, "sig", b"data").await;
        assert!(result.is_err());

        // With a public key, it should pass (in our simplified implementation)
        let mut with_keys = HashMap::new();
        with_keys.insert(plugin_id, "dummy-key".to_string());
        let registry_with_keys = PluginRegistryImpl::with_trusted_publishers(with_keys);

        let result = registry_with_keys
            .verify_signature(plugin_id, "sig", b"data")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trusted_publisher_check() {
        let plugin_id = Uuid::new_v4();
        let mut with_keys = HashMap::new();

        let registry = PluginRegistryImpl::new();
        assert!(!registry.is_trusted_publisher(plugin_id).await);

        with_keys.insert(plugin_id, "key".to_string());
        let registry_with_keys = PluginRegistryImpl::with_trusted_publishers(with_keys);
        assert!(registry_with_keys.is_trusted_publisher(plugin_id).await);
    }

    #[tokio::test]
    async fn test_get_trusted_plugins() {
        let plugin_id1 = Uuid::new_v4();
        let plugin_id2 = Uuid::new_v4();
        let mut with_keys = HashMap::new();
        with_keys.insert(plugin_id1, "key1".to_string());
        with_keys.insert(plugin_id2, "key2".to_string());

        let registry = PluginRegistryImpl::with_trusted_publishers(with_keys);
        let trusted = registry.get_trusted_plugins().await;

        assert_eq!(trusted.len(), 2);
        assert!(trusted.contains(&plugin_id1));
        assert!(trusted.contains(&plugin_id2));
    }
}
