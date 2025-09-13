//! WebAuthn Command Handlers for Tauri with Rate Limiting
//!
//! This module provides Tauri command handlers for WebAuthn authentication operations,
//! implementing the complete WebAuthn ceremony flow with secure IPC communication
//! and integrated rate limiting protection.

use crate::command_templates::*;
use crate::security::rate_limit_middleware::{RateLimitMiddleware, RateLimitMiddlewareConfig};
use rust_ai_ide_common::validation::TauriInputSanitizer;
use rust_ai_ide_security::webauthn::*;
use rust_ai_ide_security::{AuthRateLimiter, EndpointType};
use rust_ai_ide_types::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use webauthn_rs::prelude::*;

// Command configuration for WebAuthn operations
const WEBAUTHN_COMMAND_CONFIG: CommandConfig = CommandConfig {
    enable_logging: true,
    log_level: log::Level::Info,
    enable_validation: true,
    async_timeout_secs: Some(30), // 30 second timeout for WebAuthn operations
};

// Input validation for WebAuthn commands
#[derive(Debug, Deserialize)]
pub struct StartRegistrationInput {
    pub user_display_name: String,
    pub user_name: String,
}

#[derive(Debug, Deserialize)]
pub struct FinishRegistrationInput {
    pub challenge_id: String,
    pub registration_response: RegisterPublicKeyCredential,
}

#[derive(Debug, Deserialize)]
pub struct StartAuthenticationInput {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct FinishAuthenticationInput {
    pub challenge_id: String,
    pub authentication_response: PublicKeyCredential,
}

#[derive(Debug, Deserialize)]
pub struct DeleteCredentialInput {
    pub credential_id: String,
}

// Sanitize and validate input data
fn sanitize_webauthn_input<T: for<'de> Deserialize<'de>>(input: T) -> Result<T, String> {
    TauriInputSanitizer::sanitize_input(input)
}

/// Start WebAuthn registration ceremony with rate limiting
///
/// Initiates the WebAuthn registration process by generating a challenge
/// and returning the necessary data for the frontend to create a credential.
/// Includes rate limiting protection against abuse.
#[tauri::command]
pub async fn webauthn_start_registration(
    state: State<'_, WebAuthnService>,
    rate_limiter: State<'_, Arc<AuthRateLimiter>>,
    user: UserContext,
    input: StartRegistrationInput,
) -> Result<String, String> {
    execute_command!("webauthn_start_registration", &WEBAUTHN_COMMAND_CONFIG, async move || {
        // Apply rate limiting first
        let rate_limit_result = rate_limiter.check_rate_limit(&user, EndpointType::WebauthnRegistration, None).await;

        match rate_limit_result {
            Ok((rate_limited, headers)) => {
                if rate_limited {
                    return Err(serde_json::json!({
                        "error": "Too Many Requests",
                        "message": "Rate limit exceeded for WebAuthn registration",
                        "code": "RATE_LIMIT_EXCEEDED"
                    }).to_string());
                }

                // Include rate limit headers in successful response if available
                let mut response_data = serde_json::Map::new();

                // Validate input
                let sanitized_input = sanitize_webauthn_input(input)?;

                // Acquire service and execute
                acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
                    let challenge_response = service.start_registration(
                        &user,
                        &sanitized_input.user_display_name,
                        &sanitized_input.user_name
                    ).await?;

                    let mut response_json = serde_json::to_value(&challenge_response)?;

                    // Add rate limit headers to response if enabled
                    if let Some(headers) = headers {
                        let header_map = headers.to_headers();
                        let mut headers_obj = serde_json::Map::new();
                        for (key, value) in header_map {
                            headers_obj.insert(key, serde_json::Value::String(value));
                        }
                        if let serde_json::Value::Object(ref mut obj) = response_json {
                            obj.insert("rateLimitHeaders".to_string(), serde_json::json!(headers_obj));
                        }
                    }

                    Ok(serde_json::to_string(&response_json)?)
                })
            }
            Err(e) => {
                log::warn!("Rate limit check failed, allowing command to proceed: {}", e);
                // Allow the command to proceed on rate limit service failure

                // Validate input
                let sanitized_input = sanitize_webauthn_input(input)?;

                // Acquire service and execute
                acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
                    let challenge_response = service.start_registration(
                        &user,
                        &sanitized_input.user_display_name,
                        &sanitized_input.user_name
                    ).await?;

                    Ok(serde_json::to_string(&challenge_response)?)
                })
            }
        }
    })
}

/// Complete WebAuthn registration ceremony with rate limiting
///
/// Finishes the WebAuthn registration by verifying the authenticator's response
/// and storing the credential securely. Includes rate limiting protection.
#[tauri::command]
pub async fn webauthn_finish_registration(
    state: State<'_, WebAuthnService>,
    rate_limiter: State<'_, Arc<AuthRateLimiter>>,
    user: UserContext,
    input: FinishRegistrationInput,
) -> Result<String, String> {
    execute_command!("webauthn_finish_registration", &WEBAUTHN_COMMAND_CONFIG, async move || {
        // Apply rate limiting first
        let rate_limit_result = rate_limiter.check_rate_limit(&user, EndpointType::WebauthnRegistration, None).await;

        match rate_limit_result {
            Ok((rate_limited, headers)) => {
                if rate_limited {
                    return Err(serde_json::json!({
                        "error": "Too Many Requests",
                        "message": "Rate limit exceeded for WebAuthn registration completion",
                        "code": "RATE_LIMIT_EXCEEDED"
                    }).to_string());
                }
            }
            Err(e) => {
                log::warn!("Rate limit check failed, allowing command to proceed: {}", e);
            }
        }

        // Validate input
        let sanitized_input = sanitize_webauthn_input(input)?;

        // Acquire service and execute
        acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
            let credential_id = service.finish_registration(
                &user,
                &sanitized_input.challenge_id,
                &sanitized_input.registration_response
            ).await?;

            let mut response = serde_json::json!({
                "credential_id": credential_id,
                "status": "success"
            });

            // Add rate limit headers to response if available
            if let Ok((_, Some(headers))) = rate_limit_result {
                let header_map = headers.to_headers();
                let mut headers_obj = serde_json::Map::new();
                for (key, value) in header_map {
                    headers_obj.insert(key, serde_json::Value::String(value));
                }
                if let serde_json::Value::Object(ref mut obj) = response {
                    obj.insert("rateLimitHeaders".to_string(), serde_json::json!(headers_obj));
                }
            }

            Ok(serde_json::to_string(&response)?)
        })
    })
}

/// Start WebAuthn authentication ceremony with rate limiting
///
/// Initiates the WebAuthn authentication process by generating a challenge
/// for the specified user. Includes rate limiting protection.
#[tauri::command]
pub async fn webauthn_start_authentication(
    state: State<'_, WebAuthnService>,
    rate_limiter: State<'_, Arc<AuthRateLimiter>>,
    input: StartAuthenticationInput,
) -> Result<String, String> {
    execute_command!("webauthn_start_authentication", &WEBAUTHN_COMMAND_CONFIG, async move || {
        // Create anonymous user context for rate limiting
        let user_context = UserContext {
            user_id: input.user_id.clone(),
            username: input.user_id.clone(),
            roles: vec![],
            permissions: vec![],
            session_id: None,
            mfa_verified: false,
        };

        // Apply rate limiting
        let rate_limit_result = rate_limiter.check_rate_limit(&user_context, EndpointType::WebauthnAuthentication, None).await;

        match rate_limit_result {
            Ok((rate_limited, headers)) => {
                if rate_limited {
                    return Err(serde_json::json!({
                        "error": "Too Many Requests",
                        "message": "Rate limit exceeded for WebAuthn authentication",
                        "code": "RATE_LIMIT_EXCEEDED"
                    }).to_string());
                }
            }
            Err(e) => {
                log::warn!("Rate limit check failed, allowing command to proceed: {}", e);
            }
        }

        // Validate input
        let sanitized_input = sanitize_webauthn_input(input)?;

        // Acquire service and execute
        acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
            let challenge_response = service.start_authentication(&sanitized_input.user_id).await?;

            let mut response_json = serde_json::to_value(&challenge_response)?;

            // Add rate limit headers to response if available
            if let Ok((_, Some(headers))) = rate_limit_result {
                let header_map = headers.to_headers();
                let mut headers_obj = serde_json::Map::new();
                for (key, value) in header_map {
                    headers_obj.insert(key, serde_json::Value::String(value));
                }
                if let serde_json::Value::Object(ref mut obj) = response_json {
                    obj.insert("rateLimitHeaders".to_string(), serde_json::json!(headers_obj));
                }
            }

            Ok(serde_json::to_string(&response_json)?)
        })
    })
}

/// Complete WebAuthn authentication ceremony with rate limiting
///
/// Finishes the WebAuthn authentication by verifying the authenticator's response
/// and returning the authenticated user ID. Includes rate limiting protection.
#[tauri::command]
pub async fn webauthn_finish_authentication(
    state: State<'_, WebAuthnService>,
    rate_limiter: State<'_, Arc<AuthRateLimiter>>,
    input: FinishAuthenticationInput,
) -> Result<String, String> {
    execute_command!("webauthn_finish_authentication", &WEBAUTHN_COMMAND_CONFIG, async move || {
        // Since we don't know the user_id yet at this point, we use a generic user context
        // The rate limiter will handle this as an anonymous request initially
        let temp_user_context = UserContext {
            user_id: "temp_webauthn_user".to_string(),
            username: "temp_webauthn_user".to_string(),
            roles: vec![],
            permissions: vec![],
            session_id: None,
            mfa_verified: false,
        };

        // Apply rate limiting
        let rate_limit_result = rate_limiter.check_rate_limit(&temp_user_context, EndpointType::WebauthnAuthentication, None).await;

        match rate_limit_result {
            Ok((rate_limited, _)) => {
                if rate_limited {
                    return Err(serde_json::json!({
                        "error": "Too Many Requests",
                        "message": "Rate limit exceeded for WebAuthn authentication completion",
                        "code": "RATE_LIMIT_EXCEEDED"
                    }).to_string());
                }
            }
            Err(e) => {
                log::warn!("Rate limit check failed, allowing command to proceed: {}", e);
            }
        }

        // Validate input
        let sanitized_input = sanitize_webauthn_input(input)?;

        // Acquire service and execute
        acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
            let user_id = service.finish_authentication(
                &sanitized_input.challenge_id,
                &sanitized_input.authentication_response
            ).await?;

            let mut response = serde_json::json!({
                "user_id": user_id,
                "authenticated": true
            });

            // Add rate limit headers to response if available
            if let Ok((_, Some(headers))) = rate_limit_result {
                let header_map = headers.to_headers();
                let mut headers_obj = serde_json::Map::new();
                for (key, value) in header_map {
                    headers_obj.insert(key, serde_json::Value::String(value));
                }
                if let serde_json::Value::Object(ref mut obj) = response {
                    obj.insert("rateLimitHeaders".to_string(), serde_json::json!(headers_obj));
                }
            }

            Ok(serde_json::to_string(&response)?)
        })
    })
}

/// List user's WebAuthn credentials
///
/// Returns a list of all WebAuthn credentials registered for the current user.
#[tauri::command]
pub async fn webauthn_list_credentials(
    state: State<'_, WebAuthnService>,
    user: UserContext,
) -> Result<String, String> {
    execute_command!("webauthn_list_credentials", &WEBAUTHN_COMMAND_CONFIG, async move || {
        // Acquire service and execute
        acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
            let credentials = service.list_credentials(&user).await?;

            Ok(serde_json::to_string(&credentials)?)
        })
    })
}

/// Delete a WebAuthn credential
///
/// Removes a specific WebAuthn credential from the user's account.
#[tauri::command]
pub async fn webauthn_delete_credential(
    state: State<'_, WebAuthnService>,
    user: UserContext,
    input: DeleteCredentialInput,
) -> Result<String, String> {
    execute_command!("webauthn_delete_credential", &WEBAUTHN_COMMAND_CONFIG, async move || {
        // Validate input
        let sanitized_input = sanitize_webauthn_input(input)?;

        // Acquire service and execute
        acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
            service.delete_credential(&user, &sanitized_input.credential_id).await?;

            Ok(serde_json::to_string(&serde_json::json!({
                "deleted": true,
                "credential_id": sanitized_input.credential_id
            }))?)
        })
    })
}

/// Get WebAuthn service status
///
/// Returns the current health status and configuration of the WebAuthn service.
#[tauri::command]
pub async fn webauthn_get_status(
    state: State<'_, WebAuthnService>,
) -> Result<String, String> {
    execute_command!("webauthn_get_status", &WEBAUTHN_COMMAND_CONFIG, async move || {
        // Acquire service and execute
        acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
            let status = service.health_status().await;

            Ok(serde_json::to_string(&serde_json::json!({
                "status": match status {
                    ComponentStatus::Healthy => "healthy",
                    ComponentStatus::Degraded => "degraded",
                    ComponentStatus::Unhealthy => "unhealthy",
                },
                "service": "WebAuthn Authentication Service"
            }))?)
        })
    })
}

/// Clean up expired WebAuthn challenges
///
/// Removes expired registration and authentication challenges to maintain system hygiene.
#[tauri::command]
pub async fn webauthn_cleanup_expired_challenges(
    state: State<'_, WebAuthnService>,
) -> Result<String, String> {
    execute_command!("webauthn_cleanup_expired_challenges", &WEBAUTHN_COMMAND_CONFIG, async move || {
        // Acquire service and execute
        acquire_service_and_execute!(state.webauthn_service, WebAuthnService, {
            let cleaned_count = service.cleanup_expired_challenges().await?;

            Ok(serde_json::to_string(&serde_json::json!({
                "cleaned_challenges": cleaned_count,
                "status": "success"
            }))?)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_ai_ide_common::validation::TauriInputSanitizer;

    #[test]
    fn test_input_sanitization() {
        let input = StartRegistrationInput {
            user_display_name: "Test User".to_string(),
            user_name: "testuser".to_string(),
        };

        let result = sanitize_webauthn_input(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_config() {
        assert_eq!(WEBAUTHN_COMMAND_CONFIG.enable_logging, true);
        assert_eq!(WEBAUTHN_COMMAND_CONFIG.enable_validation, true);
        assert_eq!(WEBAUTHN_COMMAND_CONFIG.async_timeout_secs, Some(30));
    }
}