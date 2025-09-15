use async_trait::async_trait;

use crate::{ComponentStatus, SecurityResult};

/// A trait defining the common interface for all security services
#[async_trait]
pub trait SecurityService: Send + Sync {
    /// Performs a health check of the service
    async fn health_check(&self) -> SecurityResult<ComponentStatus>;

    /// Returns the name of the service
    fn get_service_name(&self) -> String;
}
