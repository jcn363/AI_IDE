//! Test utilities for WebAuthn testing

use std::sync::Arc;

use mockall::predicate::*;
use mockall::Sequence;
// Re-export commonly used items
pub use mockall::*;
pub use webauthn_rs::prelude::*;
use webauthn_rs::prelude::{
    AttestationConveyancePreference, CreationChallengeResponse, Credential as WebAuthnCredential,
    CredentialID, PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse,
    ResidentKeyRequirement, UserVerificationRequirement,
};

use crate::security::audit::AuditLogger;
use crate::security::crypto::{CryptoOps, DataKeyManager, MasterKeyManager};
use crate::security::models::DataKey;
use crate::security::webauthn::{WebAuthnConfig, WebAuthnService};
use crate::security::UserContext;

// Mock implementations for testing
mock! {
    pub AuditLogger {}
    #[async_trait]
    impl AuditLogger for AuditLogger {
        async fn log(&self, event: &crate::security::audit::AuditEventContext) -> crate::security::SecurityResult<()>;
    }
}

mock! {
    pub CryptoOps {}
    #[async_trait]
    impl CryptoOps for CryptoOps {
        async fn encrypt(&self, data: &[u8], key: &[u8]) -> crate::security::SecurityResult<Vec<u8>>;
        async fn decrypt(&self, data: &[u8], key: &[u8]) -> crate::security::SecurityResult<Vec<u8>>>;
    }
}

mock! {
    pub MasterKeyManager {}
    #[async_trait]
    impl MasterKeyManager for MasterKeyManager {
        async fn get_master_key(&self) -> crate::security::SecurityResult<Vec<u8>>>;
        async fn rotate_master_key(&self) -> crate::security::SecurityResult<()>;
    }
}

mock! {
    pub DataKeyManager {}
    #[async_trait]
    impl DataKeyManager for DataKeyManager {
        async fn generate_data_key(&self) -> crate::security::SecurityResult<DataKey>;
        async fn decrypt_data_key(&self, key_id: &str) -> crate::security::SecurityResult<Vec<u8>>>;
    }
}

/// Create a test user
pub fn create_test_user() -> UserContext {
    UserContext {
        user_id: "test-user-123".to_string(),
        username: Some("testuser".to_string()),
        email: Some("test@example.com".to_string()),
        roles: vec!["user".to_string()],
        permissions: vec![
            "webauthn.register".to_string(),
            "webauthn.authenticate".to_string(),
        ],
    }
}

/// Create a test WebAuthn service with mocks
pub async fn create_test_service() -> (WebAuthnService, Arc<MockAuditLogger>) {
    let config = WebAuthnConfig {
        enabled: true,
        rp_name: "Test RP".to_string(),
        rp_id: "localhost".to_string(),
        rp_origin: "http://localhost:3000".to_string(),
        credential_timeout_ms: 60000,
        challenge_timeout_seconds: 300,
        max_credentials_per_user: 5,
        allow_software_keys: true,
        attestation_preference: AttestationConveyancePreference::None,
        user_verification: UserVerificationRequirement::Preferred,
        resident_key_requirement: ResidentKeyRequirement::Discouraged,
    };

    let audit_logger = Arc::new(MockAuditLogger::new());
    let crypto_ops = Arc::new(MockCryptoOps::new());
    let master_key_manager = Arc::new(MockMasterKeyManager::new());
    let data_key_manager = Arc::new(MockDataKeyManager::new());

    let service = WebAuthnService::new(
        config,
        audit_logger.clone(),
        crypto_ops,
        master_key_manager,
        data_key_manager,
    )
    .await
    .expect("Failed to create WebAuthn service");

    (service, audit_logger)
}

/// Create a test registration challenge
pub fn create_test_registration_challenge() -> (String, CreationChallengeResponse) {
    let challenge_id = "test-challenge-123".to_string();
    let ccr = CreationChallengeResponse {
        public_key: Default::default(),
    };
    (challenge_id, ccr)
}

/// Create a test authentication challenge
pub fn create_test_authentication_challenge() -> (String, RequestChallengeResponse) {
    let challenge_id = "test-auth-challenge-123".to_string();
    let rcr = RequestChallengeResponse {
        public_key: Default::default(),
    };
    (challenge_id, rcr)
}
