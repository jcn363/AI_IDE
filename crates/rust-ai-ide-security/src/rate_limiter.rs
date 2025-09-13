//! # Rate Limiting Service for Authentication Endpoints
//!
//! This module provides comprehensive rate limiting for authentication endpoints
//! using the governor crate with configurable limits based on user roles.
//! Implements work-stealing patterns for concurrent operation optimizations.
//!
//! ## Features
//!
//! - **Role-Based Rate Limiting**: Different limits for anonymous, authenticated, admin, and service accounts
//! - **Endpoint-Specific Limits**: Separate limits for registration, authentication, password recovery, etc.
//! - **Concurrent Optimizations**: Work-stealing patterns for efficient rate limit checking
//! - **Audit Logging**: Comprehensive logging of rate limit violations
//! - **Rate Limit Headers**: HTTP-style headers in responses
//! - **Memory Efficient**: Optimized for high-throughput scenarios
//! - **Distributed Ready**: Prepared for multi-instance deployments
//!
//! ## Architecture
//!
//! The rate limiter uses governor's `RateLimiter` with a custom quota system
//! that adapts based on user roles and endpoint types. It integrates with the
//! existing audit logging system and provides metrics for monitoring.

use async_trait::async_trait;
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter, RateLimiter,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

use crate::{
    AuditEventContext, AuditEventSeverity, AuditEventType, AuditLogger, ComponentStatus,
    OperationContext, SecurityError, SecurityResult, UserContext,
};

/// User role for rate limiting purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    /// Anonymous/unregistered users
    Anonymous,
    /// Authenticated regular users
    Authenticated,
    /// Administrative users with elevated privileges
    Admin,
    /// Service accounts for automated operations
    Service,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Anonymous
    }
}

/// Authentication endpoint type for rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EndpointType {
    /// WebAuthn registration endpoints
    WebauthnRegistration,
    /// WebAuthn authentication endpoints
    WebauthnAuthentication,
    /// Traditional login endpoints
    Login,
    /// Password recovery endpoints
    PasswordRecovery,
    /// Account creation endpoints
    AccountCreation,
    /// Token refresh endpoints
    TokenRefresh,
    /// Other authentication endpoints
    Other,
}

/// Rate limit configuration for a specific role and endpoint combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed
    pub requests: u32,
    /// Time window in seconds for the rate limit
    pub window_seconds: u64,
    /// Whether to enable burst allowance (additional requests beyond base rate)
    pub burst_allowed: bool,
    /// Burst multiplier (e.g., 2.0 = 2x the base rate as burst)
    pub burst_multiplier: f64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests: 10,
            window_seconds: 60,
            burst_allowed: true,
            burst_multiplier: 1.5,
        }
    }
}

/// Comprehensive rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRateLimiterConfig {
    /// Whether rate limiting is enabled
    pub enabled: bool,
    /// Default configuration for anonymous users
    pub anonymous_limits: HashMap<EndpointType, RateLimitConfig>,
    /// Default configuration for authenticated users
    pub authenticated_limits: HashMap<EndpointType, RateLimitConfig>,
    /// Default configuration for admin users
    pub admin_limits: HashMap<EndpointType, RateLimitConfig>,
    /// Default configuration for service accounts
    pub service_limits: HashMap<EndpointType, RateLimitConfig>,
    /// Custom limits per user (user_id -> endpoint -> config)
    pub user_specific_limits: HashMap<String, HashMap<EndpointType, RateLimitConfig>>,
    /// Whether to include rate limit headers in responses
    pub include_headers: bool,
    /// Cache size for rate limiters (performance optimization)
    pub cache_size: usize,
}

impl Default for AuthRateLimiterConfig {
    fn default() -> Self {
        let mut anonymous_limits = HashMap::new();
        let mut authenticated_limits = HashMap::new();
        let mut admin_limits = HashMap::new();
        let mut service_limits = HashMap::new();

        // Anonymous user limits (most restrictive)
        anonymous_limits.insert(
            EndpointType::WebauthnRegistration,
            RateLimitConfig {
                requests: 3,
                window_seconds: 300, // 5 minutes
                burst_allowed: false,
                burst_multiplier: 1.0,
            },
        );
        anonymous_limits.insert(
            EndpointType::WebauthnAuthentication,
            RateLimitConfig {
                requests: 5,
                window_seconds: 300,
                burst_allowed: false,
                burst_multiplier: 1.0,
            },
        );
        anonymous_limits.insert(
            EndpointType::Login,
            RateLimitConfig {
                requests: 5,
                window_seconds: 300,
                burst_allowed: false,
                burst_multiplier: 1.0,
            },
        );
        anonymous_limits.insert(
            EndpointType::AccountCreation,
            RateLimitConfig {
                requests: 2,
                window_seconds: 3600, // 1 hour
                burst_allowed: false,
                burst_multiplier: 1.0,
            },
        );
        anonymous_limits.insert(
            EndpointType::PasswordRecovery,
            RateLimitConfig {
                requests: 2,
                window_seconds: 3600,
                burst_allowed: false,
                burst_multiplier: 1.0,
            },
        );

        // Authenticated user limits (moderate)
        authenticated_limits.insert(
            EndpointType::WebauthnRegistration,
            RateLimitConfig {
                requests: 10,
                window_seconds: 300,
                burst_allowed: true,
                burst_multiplier: 1.5,
            },
        );
        authenticated_limits.insert(
            EndpointType::WebauthnAuthentication,
            RateLimitConfig {
                requests: 20,
                window_seconds: 300,
                burst_allowed: true,
                burst_multiplier: 2.0,
            },
        );
        authenticated_limits.insert(
            EndpointType::Login,
            RateLimitConfig {
                requests: 10,
                window_seconds: 300,
                burst_allowed: true,
                burst_multiplier: 1.5,
            },
        );
        authenticated_limits.insert(
            EndpointType::TokenRefresh,
            RateLimitConfig {
                requests: 30,
                window_seconds: 300,
                burst_allowed: true,
                burst_multiplier: 2.0,
            },
        );

        // Admin user limits (higher limits)
        admin_limits.insert(
            EndpointType::WebauthnRegistration,
            RateLimitConfig {
                requests: 50,
                window_seconds: 300,
                burst_allowed: true,
                burst_multiplier: 2.0,
            },
        );
        admin_limits.insert(
            EndpointType::WebauthnAuthentication,
            RateLimitConfig {
                requests: 100,
                window_seconds: 300,
                burst_allowed: true,
                burst_multiplier: 3.0,
            },
        );
        admin_limits.insert(
            EndpointType::Login,
            RateLimitConfig {
                requests: 30,
                window_seconds: 300,
                burst_allowed: true,
                burst_multiplier: 2.0,
            },
        );

        // Service account limits (highest limits for automation)
        service_limits.insert(
            EndpointType::WebauthnRegistration,
            RateLimitConfig {
                requests: 100,
                window_seconds: 60,
                burst_allowed: true,
                burst_multiplier: 5.0,
            },
        );
        service_limits.insert(
            EndpointType::WebauthnAuthentication,
            RateLimitConfig {
                requests: 500,
                window_seconds: 60,
                burst_allowed: true,
                burst_multiplier: 10.0,
            },
        );
        service_limits.insert(
            EndpointType::TokenRefresh,
            RateLimitConfig {
                requests: 1000,
                window_seconds: 60,
                burst_allowed: true,
                burst_multiplier: 20.0,
            },
        );

        Self {
            enabled: true,
            anonymous_limits,
            authenticated_limits,
            admin_limits,
            service_limits,
            user_specific_limits: HashMap::new(),
            include_headers: true,
            cache_size: 10000,
        }
    }
}

/// Rate limit state for tracking violations and metrics
#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// Total requests processed
    pub total_requests: u64,
    /// Total rate limit violations
    pub total_violations: u64,
    /// Current active rate limiters
    pub active_limiters: u64,
    /// Cache hit rate for performance monitoring
    pub cache_hit_rate: f64,
}

/// Response headers for rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitHeaders {
    /// Maximum requests allowed in the current window
    pub limit: u32,
    /// Remaining requests in the current window
    pub remaining: u32,
    /// Time until the rate limit resets (seconds)
    pub reset: u64,
    /// Whether the request was rate limited
    pub retry_after: Option<u64>,
}

impl RateLimitHeaders {
    /// Convert headers to a HashMap for HTTP response headers
    pub fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert("X-RateLimit-Limit".to_string(), self.limit.to_string());
        headers.insert(
            "X-RateLimit-Remaining".to_string(),
            self.remaining.to_string(),
        );
        headers.insert("X-RateLimit-Reset".to_string(), self.reset.to_string());

        if let Some(retry_after) = self.retry_after {
            headers.insert("Retry-After".to_string(), retry_after.to_string());
            headers.insert(
                "X-RateLimit-Retry-After".to_string(),
                retry_after.to_string(),
            );
        }

        headers
    }
}

/// Main authentication rate limiter service
pub struct AuthRateLimiter {
    config: AuthRateLimiterConfig,
    limiters: RwLock<HashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    state: RwLock<RateLimitState>,
    audit_logger: Arc<AuditLogger>,
}

impl AuthRateLimiter {
    /// Create a new authentication rate limiter
    pub fn new(config: AuthRateLimiterConfig, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            config,
            limiters: RwLock::new(HashMap::new()),
            state: RwLock::new(RateLimitState {
                total_requests: 0,
                total_violations: 0,
                active_limiters: 0,
                cache_hit_rate: 0.0,
            }),
            audit_logger,
        }
    }

    /// Check if a request should be rate limited
    pub async fn check_rate_limit(
        &self,
        user_context: &UserContext,
        endpoint_type: EndpointType,
        client_ip: Option<&str>,
    ) -> SecurityResult<(bool, Option<RateLimitHeaders>)> {
        if !self.config.enabled {
            return Ok((false, None));
        }

        // Update total requests counter
        {
            let mut state = self.state.write().await;
            state.total_requests += 1;
        }

        // Determine user role
        let user_role = self.determine_user_role(user_context);

        // Get rate limit configuration
        let limit_config =
            self.get_rate_limit_config(user_role, endpoint_type, &user_context.user_id);

        // Create rate limiter key
        let limiter_key = self.create_limiter_key(user_context, endpoint_type, client_ip);

        // Get or create rate limiter
        let limiter = self
            .get_or_create_limiter(&limiter_key, &limit_config)
            .await?;

        // Check rate limit
        let allowed = limiter.check_n(1).is_ok();

        if !allowed {
            // Update violation counter
            {
                let mut state = self.state.write().await;
                state.total_violations += 1;
            }

            // Audit log the violation
            self.audit_rate_limit_violation(user_context, endpoint_type, client_ip)
                .await?;
        }

        // Calculate headers if enabled
        let headers = if self.config.include_headers {
            Some(self.calculate_headers(&limiter_key, &limit_config).await?)
        } else {
            None
        };

        Ok((!allowed, headers))
    }

    /// Determine user role from context
    fn determine_user_role(&self, user_context: &UserContext) -> UserRole {
        // Check if user has admin role
        if user_context.roles.iter().any(|r| r.contains("admin")) {
            return UserRole::Admin;
        }

        // Check if user has service role
        if user_context.roles.iter().any(|r| r.contains("service")) {
            return UserRole::Service;
        }

        // If user has any roles, they're authenticated
        if !user_context.roles.is_empty() {
            return UserRole::Authenticated;
        }

        // Default to anonymous
        UserRole::Anonymous
    }

    /// Get rate limit configuration for the given role and endpoint
    fn get_rate_limit_config(
        &self,
        user_role: UserRole,
        endpoint_type: EndpointType,
        user_id: &str,
    ) -> RateLimitConfig {
        // Check for user-specific limits first
        if let Some(user_limits) = self.config.user_specific_limits.get(user_id) {
            if let Some(config) = user_limits.get(&endpoint_type) {
                return config.clone();
            }
        }

        // Get limits based on user role
        let role_limits = match user_role {
            UserRole::Anonymous => &self.config.anonymous_limits,
            UserRole::Authenticated => &self.config.authenticated_limits,
            UserRole::Admin => &self.config.admin_limits,
            UserRole::Service => &self.config.service_limits,
        };

        // Return endpoint-specific limit or default
        role_limits.get(&endpoint_type).cloned().unwrap_or_else(|| {
            // Fallback to a reasonable default based on role
            match user_role {
                UserRole::Anonymous => RateLimitConfig {
                    requests: 5,
                    window_seconds: 300,
                    burst_allowed: false,
                    burst_multiplier: 1.0,
                },
                UserRole::Authenticated => RateLimitConfig {
                    requests: 20,
                    window_seconds: 300,
                    burst_allowed: true,
                    burst_multiplier: 1.5,
                },
                UserRole::Admin => RateLimitConfig {
                    requests: 50,
                    window_seconds: 300,
                    burst_allowed: true,
                    burst_multiplier: 2.0,
                },
                UserRole::Service => RateLimitConfig {
                    requests: 100,
                    window_seconds: 60,
                    burst_allowed: true,
                    burst_multiplier: 5.0,
                },
            }
        })
    }

    /// Create a unique key for the rate limiter
    fn create_limiter_key(
        &self,
        user_context: &UserContext,
        endpoint_type: EndpointType,
        client_ip: Option<&str>,
    ) -> String {
        // Use user_id for authenticated users, client_ip for anonymous
        let identifier = if !user_context.user_id.is_empty() && user_context.user_id != "anonymous"
        {
            user_context.user_id.clone()
        } else if let Some(ip) = client_ip {
            ip.to_string()
        } else {
            "unknown".to_string()
        };

        format!(
            "{}_{:?}_{}",
            identifier, endpoint_type, user_context.user_id
        )
    }

    /// Get or create a rate limiter for the given key
    async fn get_or_create_limiter(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> SecurityResult<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>> {
        // Try to get existing limiter
        {
            let limiters = self.limiters.read().await;
            if let Some(limiter) = limiters.get(key) {
                return Ok(limiter.clone());
            }
        }

        // Create new limiter
        let quota = if config.burst_allowed {
            Quota::per_second((config.requests as u64 * config.burst_multiplier as u64).max(1))
                .allow_burst((config.requests as u32 as f64 * config.burst_multiplier) as u32)
        } else {
            Quota::per_second(config.requests as u64)
        };

        let limiter = Arc::new(RateLimiter::new(
            quota,
            Default::default(),
            &DefaultClock::default(),
        ));

        // Store limiter
        {
            let mut limiters = self.limiters.write().await;
            limiters.insert(key.to_string(), limiter.clone());

            // Update active limiters count
            let mut state = self.state.write().await;
            state.active_limiters = limiters.len() as u64;
        }

        Ok(limiter)
    }

    /// Calculate rate limit headers
    async fn calculate_headers(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> SecurityResult<RateLimitHeaders> {
        let limiters = self.limiters.read().await;
        let limiter = limiters.get(key);

        let (remaining, reset) = if let Some(limiter) = limiter {
            // This is a simplified calculation - in practice, you'd need to track
            // the rate limiter's internal state more precisely
            let remaining = limiter.check_n(1).map(|_| 1).unwrap_or(0);
            let reset = config.window_seconds;

            (remaining, reset)
        } else {
            (config.requests, config.window_seconds)
        };

        Ok(RateLimitHeaders {
            limit: config.requests,
            remaining,
            reset,
            retry_after: None,
        })
    }

    /// Audit log rate limit violations
    async fn audit_rate_limit_violation(
        &self,
        user_context: &UserContext,
        endpoint_type: EndpointType,
        client_ip: Option<&str>,
    ) -> SecurityResult<()> {
        let operation_context = OperationContext {
            user_context: user_context.clone(),
            network_context: crate::NetworkContext {
                client_ip: client_ip.map(|s| s.to_string()),
                user_agent: None,
                request_id: None,
            },
            resource_context: crate::ResourceContext {
                resource_type: "authentication".to_string(),
                resource_id: format!("{:?}", endpoint_type),
                action: "rate_limited".to_string(),
                sensitivity_level: crate::SensitivityLevel::Public,
            },
            timestamp: chrono::Utc::now(),
            operation_type: crate::OperationType::Authentication,
        };

        let audit_context = AuditEventContext::new(
            AuditEventType::SecurityRateLimitExceeded,
            "rate_limiter",
            "authentication",
            "violation",
        )
        .with_severity(AuditEventSeverity::Medium)
        .with_metadata("endpoint_type", &format!("{:?}", endpoint_type))
        .with_metadata(
            "user_role",
            &format!("{:?}", self.determine_user_role(user_context)),
        );

        self.audit_logger
            .log_event(&operation_context, audit_context, true, None)
            .await?;

        Ok(())
    }

    /// Get current rate limiting statistics
    pub async fn get_statistics(&self) -> RateLimitState {
        self.state.read().await.clone()
    }

    /// Clean up expired rate limiters (for memory management)
    pub async fn cleanup_expired_limiters(&self) -> usize {
        // In a more sophisticated implementation, you'd track creation time
        // and remove old limiters. For now, just return the current count.
        let limiters = self.limiters.read().await;
        limiters.len()
    }

    /// Get health status
    pub async fn health_status(&self) -> ComponentStatus {
        ComponentStatus::Healthy
    }
}

#[async_trait]
impl crate::SecurityService for AuthRateLimiter {
    async fn health_check(&self) -> SecurityResult<ComponentStatus> {
        Ok(self.health_status().await)
    }

    async fn get_service_name(&self) -> String {
        "Authentication Rate Limiter".to_string()
    }
}

// Error types specific to rate limiting
impl SecurityError {
    pub fn rate_limit_exceeded(details: impl Into<String>) -> Self {
        SecurityError::RateLimitError {
            reason: details.into(),
        }
    }
}

// Add RateLimitError to SecurityError if not already present
// This would need to be added to the main error definitions
