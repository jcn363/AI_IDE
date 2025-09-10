//! Platform-specific battery monitoring implementations
//!
//! This module provides cross-platform battery state monitoring with
//! native integrations for Android and iOS, plus a mock implementation
//! for development and testing.

pub mod mock;

// Android-specific implementation
#[cfg(target_os = "android")]
pub mod android;

// iOS-specific implementation
#[cfg(target_os = "ios")]
pub mod ios;

// Re-export traits
pub use crate::battery_monitor::PlatformBatteryMonitor;