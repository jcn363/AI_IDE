//! # Rate Limiting Middleware for Tauri Commands
//!
//! This module provides middleware functionality to integrate rate limiting
//! with existing Tauri commands, specifically targeting authentication endpoints.
//!
//! ## Features
//!
//! - **Command-Level Rate Limiting**: Apply rate limits to specific Tauri commands
//! - **Automatic Integration**: Wrap existing commands without modifying their logic
//! - **Header Injection**: Automatically add rate limit headers to responses
//! - **Error Handling**: Consistent error responses for rate-limited requests
//! - **Concurrent Safety**: Thread-safe operation in multi-command environments

use std::collections::HashMap;
use std::sync::Arc;

use rust_ai_ide_common::validation::TauriInputSanitizer;
use rust_ai_ide_security::{AuthRateLimiter, EndpointType, RateLimitHeaders, SecurityError, UserContext};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::command_templates::*;

/// Configuration for rate limiting middleware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitMiddlewareConfig {
    /// Whether the middleware is enabled
    pub enabled:                  bool,
    /// Commands that should be rate limited
    pub rate_limited_commands:    Vec<String>,
    /// Command to endpoint type mapping
    pub command_endpoint_mapping: HashMap<String, EndpointType>,
}

impl Default for RateLimitMiddlewareConfig {
    fn default() -> Self {
        let mut command_endpoint_mapping = HashMap::new();

        // Map WebAuthn commands to endpoint types
        command_endpoint_mapping.insert(
            "webauthn_start_registration".to_string(),
            EndpointType::WebauthnRegistration,
        );
        command_endpoint_mapping.insert(
            "webauthn_finish_registration".to_string(),
            EndpointType::WebauthnRegistration,
        );
        command_endpoint_mapping.insert(
            "webauthn_start_authentication".to_string(),
            EndpointType::WebauthnAuthentication,
        );
        command_endpoint_mapping.insert(
            "webauthn_finish_authentication".to_string(),
            EndpointType::WebauthnAuthentication,
        );
        command_endpoint_mapping.insert(
            "webauthn_list_credentials".to_string(),
            EndpointType::WebauthnAuthentication,
        );
        command_endpoint_mapping.insert(
            "webauthn_delete_credential".to_string(),
            EndpointType::WebauthnAuthentication,
        );

        Self {
            enabled: true,
            rate_limited_commands: vec![
                "webauthn_start_registration".to_string(),
                "webauthn_finish_registration".to_string(),
                "webauthn_start_authentication".to_string(),
                "webauthn_finish_authentication".to_string(),
                "webauthn_list_credentials".to_string(),
                "webauthn_delete_credential".to_string(),
            ],
            command_endpoint_mapping,
        }
    }
}

/// Rate limit middleware for Tauri commands
pub struct RateLimitMiddleware {
    config:       RateLimitMiddlewareConfig,
    rate_limiter: Arc<AuthRateLimiter>,
}

impl RateLimitMiddleware {
    /// Create a new rate limit middleware
    pub fn new(config: RateLimitMiddlewareConfig, rate_limiter: Arc<AuthRateLimiter>) -> Self {
        Self {
            config,
            rate_limiter,
        }
    }

    /// Check if a command should be rate limited
    pub fn should_rate_limit(&self, command_name: &str) -> bool {
        self.config.enabled
            && self
                .config
                .rate_limited_commands
                .contains(&command_name.to_string())
    }

    /// Get the endpoint type for a command
    pub fn get_endpoint_type(&self, command_name: &str) -> Option<EndpointType> {
        self.config
            .command_endpoint_mapping
            .get(command_name)
            .cloned()
    }

    /// Apply rate limiting to a command execution
    pub async fn apply_rate_limit(
        &self,
        command_name: &str,
        user_context: &UserContext,
        client_ip: Option<&str>,
    ) -> Result<Option<RateLimitHeaders>, String> {
        if !self.should_rate_limit(command_name) {
            return Ok(None);
        }

        let endpoint_type = self
            .get_endpoint_type(command_name)
            .ok_or_else(|| format!("No endpoint type mapping for command: {}", command_name))?;

        // Check rate limit
        match self
            .rate_limiter
            .check_rate_limit(user_context, endpoint_type, client_ip)
            .await
        {
            Ok((rate_limited, headers)) => {
                if rate_limited {
                    return Err(self.create_rate_limit_error(headers));
                }
                Ok(headers)
            }
            Err(e) => Err(format!("Rate limit check failed: {}", e)),
        }
    }

    /// Create a standardized rate limit error response
    fn create_rate_limit_error(&self, headers: Option<RateLimitHeaders>) -> String {
        let mut error_response = serde_json::json!({
            "error": "Too Many Requests",
            "message": "Rate limit exceeded. Please try again later.",
            "code": "RATE_LIMIT_EXCEEDED"
        });

        if let Some(headers) = headers {
            if let Some(retry_after) = headers.retry_after {
                error_response["retry_after"] = serde_json::json!(retry_after);
            }

            // Add rate limit headers to the error response
            let header_map = headers.to_headers();
            let mut headers_obj = serde_json::Map::new();
            for (key, value) in header_map {
                headers_obj.insert(key, serde_json::Value::String(value));
            }
            error_response["rate_limit_headers"] = serde_json::json!(headers_obj);
        }

        serde_json::to_string(&error_response)
            .unwrap_or_else(|_| r#"{"error": "Rate limit error serialization failed"}"#.to_string())
    }

    /// Apply rate limiting and return headers for successful requests
    pub async fn check_and_get_headers(
        &self,
        command_name: &str,
        user_context: &UserContext,
        client_ip: Option<&str>,
    ) -> Result<Option<RateLimitHeaders>, String> {
        self.apply_rate_limit(command_name, user_context, client_ip)
            .await
    }
}

/// Enhanced command wrapper that includes rate limiting
#[macro_export]
macro_rules! rate_limited_command {
    (
        $command_name:ident,
        $endpoint_type:expr,
        $async_fn:block,
        service = $service_type:ty,
        state = $state_ident:ident,
        rate_limiter = $rate_limiter:ident,
        config = $config:expr
    ) => {
        #[tauri::command]
        pub async fn $command_name(
            $state_ident: State<'_, $service_type>,
            $rate_limiter: State<'_, std::sync::Arc<rust_ai_ide_security::AuthRateLimiter>>,
            user: UserContext,
            $($arg:ident: $arg_type:ty),*
        ) -> Result<String, String> {
            execute_command!(stringify!($command_name), &$config, async move || {
                // Apply rate limiting
                let rate_limit_result = $rate_limiter.check_rate_limit(&user, $endpoint_type, None).await;

                match rate_limit_result {
                    Ok((rate_limited, _)) => {
                        if rate_limited {
                            return Err(serde_json::json!({
                                "error": "Too Many Requests",
                                "message": "Rate limit exceeded for this endpoint",
                                "code": "RATE_LIMIT_EXCEEDED",
                                "endpoint_type": stringify!($endpoint_type)
                            }).to_string());
                        }
                    }
                    Err(e) => {
                        log::error!("Rate limit check failed for command {}: {}", stringify!($command_name), e);
                        // Allow the command to proceed on rate limit service failure
                    }
                }

                // Execute the original command logic
                $async_fn
            })
        }
    };
}

/// Wrapper for commands that need both service access and rate limiting
#[macro_export]
macro_rules! rate_limited_service_command {
    (
        $command_name:ident,
        $endpoint_type:expr,
        $async_fn:block,
        service = $service_type:ty,
        state = $state_ident:ident,
        rate_limiter = $rate_limiter:ident,
        config = $config:expr
    ) => {
        #[tauri::command]
        pub async fn $command_name(
            $state_ident: State<'_, $service_type>,
            $rate_limiter: State<'_, std::sync::Arc<rust_ai_ide_security::AuthRateLimiter>>,
            user: UserContext,
            $($arg:ident: $arg_type:ty),*
        ) -> Result<String, String> {
            execute_command!(stringify!($command_name), &$config, async move || {
                // Apply rate limiting first
                let rate_limit_result = $rate_limiter.check_rate_limit(&user, $endpoint_type, None).await;

                match rate_limit_result {
                    Ok((rate_limited, headers)) => {
                        if rate_limited {
                            let mut error_response = serde_json::json!({
                                "error": "Too Many Requests",
                                "message": format!("Rate limit exceeded for {}", stringify!($endpoint_type)),
                                "code": "RATE_LIMIT_EXCEEDED"
                            });

                            if let Some(headers) = headers {
                                let header_map = headers.to_headers();
                                let mut headers_obj = serde_json::Map::new();
                                for (key, value) in header_map {
                                    headers_obj.insert(key, serde_json::Value::String(value));
                                }
                                error_response["rate_limit_info"] = serde_json::json!(headers_obj);
                            }

                            return Err(error_response.to_string());
                        }
                    }
                    Err(e) => {
                        log::warn!("Rate limit check failed, allowing command to proceed: {}", e);
                    }
                }

                // Execute the service command
                acquire_service_and_execute!($state_ident, $service_type, $async_fn)
            })
        }
    };
}

/// Helper function to extract client IP from Tauri request (when available)
pub fn extract_client_ip(_app_handle: &AppHandle) -> Option<String> {
    // In a desktop application, we might not have direct access to client IP
    // This could be extended to extract from WebSocket connections or similar
    // For now, return None as desktop apps typically don't expose client IPs
    None
}

/// Initialize rate limiting middleware with default configuration
pub fn create_default_rate_limit_middleware(rate_limiter: Arc<AuthRateLimiter>) -> RateLimitMiddleware {
    RateLimitMiddleware::new(RateLimitMiddlewareConfig::default(), rate_limiter)
}

#[cfg(test)]
mod tests {
    use rust_ai_ide_security::{AuditLogger, AuthRateLimiterConfig};

    use super::*;

    #[tokio::test]
    async fn test_middleware_configuration() {
        let config = RateLimitMiddlewareConfig::default();
        assert!(config.enabled);
        assert!(!config.rate_limited_commands.is_empty());
        assert!(!config.command_endpoint_mapping.is_empty());
    }

    #[tokio::test]
    async fn test_should_rate_limit() {
        let rate_limiter = Arc::new(AuthRateLimiter::new(
            AuthRateLimiterConfig::default(),
            Arc::new(AuditLogger::new().unwrap()),
        ));

        let middleware = create_default_rate_limit_middleware(rate_limiter);

        assert!(middleware.should_rate_limit("webauthn_start_registration"));
        assert!(middleware.should_rate_limit("webauthn_finish_authentication"));
        assert!(!middleware.should_rate_limit("some_other_command"));
    }

    #[tokio::test]
    async fn test_endpoint_type_mapping() {
        let rate_limiter = Arc::new(AuthRateLimiter::new(
            AuthRateLimiterConfig::default(),
            Arc::new(AuditLogger::new().unwrap()),
        ));

        let middleware = create_default_rate_limit_middleware(rate_limiter);

        let endpoint_type = middleware.get_endpoint_type("webauthn_start_registration");
        assert_eq!(endpoint_type, Some(EndpointType::WebauthnRegistration));

        let invalid_endpoint = middleware.get_endpoint_type("non_existent_command");
        assert_eq!(invalid_endpoint, None);
    }
}
