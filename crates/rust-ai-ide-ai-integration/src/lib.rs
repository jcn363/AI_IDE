//! # Rust AI IDE AI Service Integration Layer (Phase 2.4)
//!
//! This crate provides the complete AI service integration system that bridges
//! AI capabilities with the LSP service and frontend user experience, building
//! upon all Phase 1-2.3 foundation components.
//!
//! ## Architecture Overview
//!
//! The integration layer consists of several key components:
//!
//! - **LSPAiBridge**: LSP service extension for real-time AI assistance
//! - **AITauriInterface**: Frontend interaction system for AI feedback
//! - **AITypesGenerator**: TypeScript interface generation from Rust types
//! - **AiUxOptimizer**: User experience optimization and adaptation
//! - **AiPerformanceRouter**: Smart routing based on AI capabilities and performance

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod bridge;
pub mod errors;
pub mod frontend;
pub mod layer;
pub mod metrics;
pub mod router;
pub mod types;

pub use layer::AIServiceIntegrationLayer;
pub use bridge::LSPAiBridge;
pub use frontend::AITauriInterface;
pub use router::AiPerformanceRouter;

/// Main integration layer that orchestrates all AI service components
///
/// This struct serves as the central hub for all AI integration functionality,
/// managing the interaction between LSP service, frontend, and backend AI components.
pub struct AIServiceIntegrationLayer {
    lsp_bridge: std::sync::Arc<bridge::LSPAiBridge>,
    frontend_interface: std::sync::Arc<frontend::AITauriInterface>,
    performance_router: std::sync::Arc<router::AiPerformanceRouter>,
}

impl AIServiceIntegrationLayer {
    /// Initialize the complete AI service integration layer
    ///
    /// # Errors
    /// Returns an error if any component fails to initialize
    pub async fn new() -> Result<Self, errors::IntegrationError> {
        let lsp_bridge = std::sync::Arc::new(
            bridge::LSPAiBridge::new().await.map_err(errors::IntegrationError::LspBridge)?
        );

        let frontend_interface = std::sync::Arc::new(
            frontend::AITauriInterface::new().await.map_err(errors::IntegrationError::Frontend)?
        );

        let performance_router = std::sync::Arc::new(
            router::AiPerformanceRouter::new().await.map_err(errors::IntegrationError::Router)?
        );

        Ok(Self {
            lsp_bridge,
            frontend_interface,
            performance_router,
        })
    }

    /// Get a clone of the LSP AI bridge component
    #[must_use]
    pub fn lsp_bridge(&self) -> std::sync::Arc<bridge::LSPAiBridge> {
        std::sync::Arc::clone(&self.lsp_bridge)
    }

    /// Get a clone of the frontend interface component
    #[must_use]
    pub fn frontend_interface(&self) -> std::sync::Arc<frontend::AITauriInterface> {
        std::sync::Arc::clone(&self.frontend_interface)
    }

    /// Get a clone of the performance router component
    #[must_use]
    pub fn performance_router(&self) -> std::sync::Arc<router::AiPerformanceRouter> {
        std::sync::Arc::clone(&self.performance_router)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_layer_creation() {
        let layer = AIServiceIntegrationLayer::new().await;
        assert!(layer.is_ok(), "Integration layer should initialize successfully");
    }

    #[tokio::test]
    async fn test_component_access() {
        let layer = AIServiceIntegrationLayer::new().await.unwrap();
        let _lsp_bridge = layer.lsp_bridge();
        let _frontend = layer.frontend_interface();
        let _router = layer.performance_router();
    }
}