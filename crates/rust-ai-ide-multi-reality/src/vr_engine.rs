//! AI Integration Bridge for Spatial AI Assistance

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AIState {
    Uninitialized,
    Ready,
    Processing,
    Error(String),
}

#[derive(Debug)]
pub struct AiIntegrationBridge {
    state: Arc<Mutex<AIState>>,
}

impl AiIntegrationBridge {
    pub async fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AIState::Uninitialized)),
        }
    }
}