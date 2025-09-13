//! # Rust AI IDE Orchestration Layer
//!
//! This crate provides a unified service orchestration framework for the RUST_AI_IDE project.
//! It coordinates cross-crate communications, manages service lifecycles, and ensures
//! proper integration with existing AI/ML services and LSP infrastructure.
//!
//! ## Features
//!
//! - Service registry with automatic discovery and health monitoring
//! - Asynchronous message processing with tokio integration
//! - Cross-crate coordination extending existing EventBus patterns
//! - Tauri command consolidation following existing macro patterns
//! - Integration with existing LSP service and AI infrastructure
//!

pub mod commands;
pub mod error;
pub mod health_monitor;
pub mod lifecycle_manager;
pub mod message_router;
pub mod orchestrator;
pub mod service_registry;
pub mod types;

pub use error::OrchestrationError;
pub use health_monitor::HealthMonitor;
pub use lifecycle_manager::LifecycleManager;
pub use message_router::MessageRouter;
pub use orchestrator::ServiceOrchestrator;
pub use service_registry::ServiceRegistry;
pub use types::*;

// Re-export commonly used items from dependencies
pub use serde::{Deserialize, Serialize};
pub use tokio::sync::{mpsc, oneshot};
pub use tracing::{debug, error, info, warn};
