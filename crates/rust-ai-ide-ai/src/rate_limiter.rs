//! Rate limiting for AI service calls
//!
//! This module provides rate limiting capabilities to protect external AI service APIs
//! from being overwhelmed by too many requests.

use std::num::NonZeroU32;
use std::sync::Arc;

use governor::clock::DefaultClock;
use governor::state::direct::NotKeyed;
use governor::state::InMemoryState;
use governor::RateLimiter;
use tokio::sync::Mutex;

/// AI service rate limiter
#[derive(Clone)]
pub struct AIRateLimiter {
    /// Rate limiter for text generation and analysis requests
    text_generation_limiter: Arc<Mutex<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,

    /// Rate limiter for completion requests
    completion_limiter: Arc<Mutex<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,

    /// Rate limiter for code analysis requests
    analysis_limiter: Arc<Mutex<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,

    /// Rate limiter for model management operations
    management_limiter: Arc<Mutex<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
}

impl Default for AIRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl AIRateLimiter {
    /// Create a new AI rate limiter with default rates
    pub fn new() -> Self {
        Self::with_rates(
            60,  // requests per minute for text generation
            120, // requests per minute for completions
            30,  // requests per minute for analysis
            10,  // requests per minute for management
        )
    }

    /// Create a new AI rate limiter with custom rates
    pub fn with_rates(
        text_generation_per_minute: u32,
        completion_per_minute: u32,
        analysis_per_minute: u32,
        management_per_minute: u32,
    ) -> Self {
        let clock = DefaultClock::default();

        Self {
            text_generation_limiter: Arc::new(Mutex::new(RateLimiter::direct_with_clock(
                governor::Quota::per_minute(
                    NonZeroU32::new(text_generation_per_minute).unwrap_or(NonZeroU32::new(1).unwrap()),
                )
                .allow_burst(NonZeroU32::new((text_generation_per_minute / 4).max(1)).unwrap()),
                clock.clone(),
            ))),
            completion_limiter:      Arc::new(Mutex::new(RateLimiter::direct_with_clock(
                governor::Quota::per_minute(
                    NonZeroU32::new(completion_per_minute).unwrap_or(NonZeroU32::new(1).unwrap()),
                )
                .allow_burst(NonZeroU32::new((completion_per_minute / 4).max(1)).unwrap()),
                clock.clone(),
            ))),
            analysis_limiter:        Arc::new(Mutex::new(RateLimiter::direct_with_clock(
                governor::Quota::per_minute(
                    NonZeroU32::new(analysis_per_minute).unwrap_or(NonZeroU32::new(1).unwrap()),
                )
                .allow_burst(NonZeroU32::new((analysis_per_minute / 4).max(1)).unwrap()),
                clock.clone(),
            ))),
            management_limiter:      Arc::new(Mutex::new(RateLimiter::direct_with_clock(
                governor::Quota::per_minute(
                    NonZeroU32::new(management_per_minute).unwrap_or(NonZeroU32::new(1).unwrap()),
                )
                .allow_burst(NonZeroU32::new(2).unwrap()),
                clock,
            ))),
        }
    }

    /// Check if a text generation request is allowed under rate limits
    pub async fn check_text_generation(&self) -> Result<(), RateLimitError> {
        let limiter = self.text_generation_limiter.lock().await;
        match limiter.check_n(NonZeroU32::new(1).unwrap()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::TextGenerationLimitExceeded),
        }
    }

    /// Check if a completion request is allowed under rate limits
    pub async fn check_completion(&self) -> Result<(), RateLimitError> {
        let limiter = self.completion_limiter.lock().await;
        match limiter.check_n(NonZeroU32::new(1).unwrap()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::CompletionLimitExceeded),
        }
    }

    /// Check if an analysis request is allowed under rate limits
    pub async fn check_analysis(&self) -> Result<(), RateLimitError> {
        let limiter = self.analysis_limiter.lock().await;
        match limiter.check_n(NonZeroU32::new(1).unwrap()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::AnalysisLimitExceeded),
        }
    }

    /// Check if a management request is allowed under rate limits
    pub async fn check_management(&self) -> Result<(), RateLimitError> {
        let limiter = self.management_limiter.lock().await;
        match limiter.check_n(NonZeroU32::new(1).unwrap()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::ManagementLimitExceeded),
        }
    }

    /// Get current rate limit statistics
    pub async fn get_stats(&self) -> RateLimitStats {
        let _text_gen_limiter = self.text_generation_limiter.lock().await;
        let _completion_limiter = self.completion_limiter.lock().await;
        let _analysis_limiter = self.analysis_limiter.lock().await;
        let _management_limiter = self.management_limiter.lock().await;

        // TODO: Extract actual quota information when governor API is available
        RateLimitStats {
            text_generation_requests_available: 50,  // Placeholder
            completion_requests_available:      100, // Placeholder
            analysis_requests_available:        25,  // Placeholder
            management_requests_available:      8,   // Placeholder
        }
    }

    /// Reset rate limits (useful for testing or administrative purposes)
    pub async fn reset(&mut self) {
        // Create new rate limiters to effectively reset
        let new_limiter = Self::new();
        self.text_generation_limiter = new_limiter.text_generation_limiter;
        self.completion_limiter = new_limiter.completion_limiter;
        self.analysis_limiter = new_limiter.analysis_limiter;
        self.management_limiter = new_limiter.management_limiter;
    }
}

/// Rate limiting errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum RateLimitError {
    #[error("Text generation rate limit exceeded")]
    TextGenerationLimitExceeded,

    #[error("Completion rate limit exceeded")]
    CompletionLimitExceeded,

    #[error("Analysis rate limit exceeded")]
    AnalysisLimitExceeded,

    #[error("Management rate limit exceeded")]
    ManagementLimitExceeded,
}

/// Rate limit statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitStats {
    pub text_generation_requests_available: u32,
    pub completion_requests_available:      u32,
    pub analysis_requests_available:        u32,
    pub management_requests_available:      u32,
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let mut limiter = AIRateLimiter::new();
        assert!(limiter.check_text_generation().await.is_ok());
    }

    #[tokio::test]
    async fn test_custom_rates() {
        let limiter = AIRateLimiter::with_rates(10, 20, 5, 2);
        assert!(limiter.check_text_generation().await.is_ok());
    }

    #[tokio::test]
    async fn test_stats() {
        let limiter = AIRateLimiter::new();
        let stats = limiter.get_stats().await;

        assert!(stats.text_generation_requests_available > 0);
        assert!(stats.completion_requests_available > 0);
        assert!(stats.analysis_requests_available > 0);
        assert!(stats.management_requests_available > 0);
    }

    #[tokio::test]
    async fn test_reset() {
        let limiter = AIRateLimiter::new();

        // Exhaust text generation limit (this will succeed initially but would be limited in real usage)
        for _ in 0..70 {
            let _ = limiter.check_text_generation().await;
        }

        limiter.reset().await;

        // Should work again after reset
        assert!(limiter.check_text_generation().await.is_ok());
    }
}
