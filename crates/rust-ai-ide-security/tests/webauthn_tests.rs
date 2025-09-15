//! Integration tests for WebAuthn functionality

mod test_utils;

use std::sync::Arc;

use test_utils::*;
use webauthn_rs::prelude::{
    AttestationConveyancePreference, PublicKeyCredential, RegisterPublicKeyCredential, ResidentKeyRequirement,
    UserVerificationRequirement,
};

#[tokio::test]
async fn test_webauthn_service_creation() {
    let config = WebAuthnConfig {
        enabled:                   true,
        rp_name:                   "Test RP".to_string(),
        rp_id:                     "localhost".to_string(),
        rp_origin:                 "http://localhost:3000".to_string(),
        credential_timeout_ms:     60000,
        challenge_timeout_seconds: 300,
        max_credentials_per_user:  5,
        allow_software_keys:       true,
        attestation_preference:    AttestationConveyancePreference::None,
        user_verification:         UserVerificationRequirement::Preferred,
        resident_key_requirement:  ResidentKeyRequirement::Discouraged,
    };

    // Test configuration validation
    assert_eq!(config.rp_name, "Test RP");
    assert_eq!(config.rp_id, "localhost");
    assert_eq!(config.max_credentials_per_user, 5);
}

#[tokio::test]
async fn test_start_registration_flow() {
    let (mut service, mock_logger) = create_test_service().await;
    let user = create_test_user();

    // Setup mock expectations
    let mut seq = Sequence::new();
    mock_logger
        .expect_log()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Ok(()));

    // Start registration
    let result = service
        .start_registration(&user, "Test User", "testuser")
        .await;

    assert!(result.is_ok());
    let ccr = result.unwrap();
    assert!(!ccr.public_key.challenge.is_empty());
    assert_eq!(ccr.public_key.rp.name, "Test RP");
}

#[tokio::test]
async fn test_list_credentials_empty() {
    let (service, _) = create_test_service().await;
    let user = create_test_user();

    // Test listing credentials for a new user
    let credentials = service.list_credentials(&user).await.unwrap();
    assert!(credentials.is_empty());
}

#[tokio::test]
async fn test_credential_management() {
    let (mut service, mock_logger) = create_test_service().await;
    let user = create_test_user();

    // Setup mock expectations
    let mut seq = Sequence::new();
    mock_logger
        .expect_log()
        .times(3) // For start_registration, finish_registration, and delete_credential
        .in_sequence(&mut seq)
        .returning(|_| Ok(()));

    // Start registration
    let ccr = service
        .start_registration(&user, "Test User", "testuser")
        .await
        .unwrap();

    // Create a mock registration response
    let mut response = RegisterPublicKeyCredential::default();
    response.raw_id = vec![1, 2, 3];
    response.id = "test-credential".to_string();

    // Complete registration
    let credential_id = service
        .finish_registration(&user, &ccr.public_key.challenge, &response)
        .await
        .unwrap();

    // Verify credential was added
    let credentials = service.list_credentials(&user).await.unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].credential_id, credential_id);

    // Delete the credential
    service
        .delete_credential(&user, &credential_id)
        .await
        .unwrap();

    // Verify credential was removed
    let credentials = service.list_credentials(&user).await.unwrap();
    assert!(credentials.is_empty());
}

#[tokio::test]
async fn test_authentication_flow() {
    let (mut service, mock_logger) = create_test_service().await;
    let user = create_test_user();

    // Setup mock expectations
    let mut seq = Sequence::new();
    mock_logger
        .expect_log()
        .times(2) // For start_authentication and finish_authentication
        .in_sequence(&mut seq)
        .returning(|_| Ok(()));

    // Start authentication
    let rcr = service.start_authentication(&user.user_id).await.unwrap();
    assert!(!rcr.public_key.challenge.is_empty());

    // Create a mock authentication response
    let mut response = PublicKeyCredential::default();
    response.raw_id = vec![1, 2, 3];
    response.id = "test-credential".to_string();

    // Complete authentication
    let result = service
        .finish_authentication(&rcr.public_key.challenge, &response)
        .await;

    // Note: This will fail because we don't have a real credential to authenticate with
    // In a real test, we would need to properly set up the credential first
    assert!(result.is_err());
}
