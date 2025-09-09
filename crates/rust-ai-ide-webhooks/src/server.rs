use axum::{routing::{post, any}, Router, middleware, extract::State};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::types::{WebhookPayload, EventHandlerResponse};
use crate::handlers::default_webhook_handler;

/// Webhook server state
pub struct ServerState {
    pub providers: RwLock<HashMap<String, Arc<dyn crate::WebhookProvider>>>,
    pub stats: RwLock<std::collections::HashMap<String, u64>>,
}

/// Webhook server implementation
pub struct WebhookServer {
    port: u16,
    state: Arc<ServerState>,
    server_handle: Option<JoinHandle<()>>,
    router: Option<Router>,
}

impl WebhookServer {
    /// Create a new webhook server
    pub async fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let state = Arc::new(ServerState {
            providers: RwLock::new(HashMap::new()),
            stats: RwLock::new(HashMap::new()),
        });

        let router = Self::create_router(state.clone());

        Ok(Self {
            port,
            state,
            server_handle: None,
            router: Some(router),
        })
    }

    /// Create the HTTP router for webhook handling
    fn create_router(state: Arc<ServerState>) -> Router {
        Router::new()
            .route("/webhook/:provider_name", post(webhook_post_handler))
            .route("/webhook/health", any(health_check))
            .layer(middleware::from_fn(logging_middleware))
            .with_state(state)
    }

    /// Register a webhook handler for a specific provider
    pub async fn register_handler(&self, provider_name: String, provider: Arc<dyn crate::WebhookProvider>) {
        let mut providers = self.state.providers.write().await;
        providers.insert(provider_name, provider);
    }

    /// Start the webhook server
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let router = self.router.take().ok_or("Router already taken")?;
        let listener = TcpListener::bind(addr).await?;
        let handle = tokio::spawn(async move {
            let app = router;
            axum::serve(listener, app).await.unwrap();
        });

        self.server_handle = Some(handle);
        tracing::info!("Webhook server started on port {}", self.port);
        Ok(())
    }

    /// Stop the webhook server
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            tracing::info!("Webhook server stopped");
        }
        Ok(())
    }

    /// Get server statistics
    pub async fn get_stats(&self) -> std::collections::HashMap<String, u64> {
        let stats = self.state.stats.read().await;
        stats.clone()
    }
}

/// HTTP handler for webhook POST requests
async fn webhook_post_handler(
    State(state): State<Arc<ServerState>>,
    axum::extract::Path(provider_name): axum::extract::Path<String>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Result<axum::Json<EventHandlerResponse>, (axum::http::StatusCode, String)> {
    let providers = state.providers.read().await;

    if let Some(provider) = providers.get(&provider_name) {
        // Convert headers to hashmap (simplified)
        let headers = HashMap::new(); // TODO: Extract actual headers

        let webhook_payload = WebhookPayload {
            id: uuid::Uuid::new_v4().to_string(),
            event: provider_name.clone(),
            payload,
            headers,
            signature: None, // TODO: Extract signature
        };

        match provider.process_payload(webhook_payload).await {
            Ok(_) => {
                // Update stats
                let mut stats = state.stats.write().await;
                *stats.entry(provider_name.clone()).or_insert(0) += 1;

                Ok(axum::Json(EventHandlerResponse {
                    success: true,
                    message: "Webhook processed successfully".to_string(),
                    data: None,
                }))
            }
            Err(e) => {
                tracing::error!("Webhook processing error: {}", e);
                Err((
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Processing error: {}", e),
                ))
            }
        }
    } else {
        Err((
            axum::http::StatusCode::NOT_FOUND,
            format!("Provider {} not found", provider_name),
        ))
    }
}

/// Health check handler
async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "rust-ai-ide-webhooks"
    }))
}

/// Logging middleware
async fn logging_middleware(
    req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let elapsed = start.elapsed();

    let status = response.status();
    tracing::info!("{} {} {} - {}ms", method, uri, status, elapsed.as_millis());

    response
}

impl Drop for WebhookServer {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}