use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{Json, IntoResponse};
use std::collections::HashMap;

/// Authentication middleware for webhook requests
pub async fn authentication_middleware(
    req: Request,
    next: Next,
) -> axum::response::Response {
    // Extract provider and payload for validation
    let provider = req.uri().path().split('/').last().unwrap_or("").to_string();

    // Pass validation responsibility to handlers
    // In a real implementation, we might validate tokens here

    // For now, just log the request
    tracing::info!("Webhook request received for provider: {}", provider);

    next.run(req).await
}

/// Content validation middleware
pub async fn content_validation_middleware(
    req: Request,
    next: Next,
) -> Result<axum::response::Response, axum::response::Response> {
    // Check content type
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("application/json");

    if !content_type.contains("json") {
        let error = serde_json::json!({
            "error": "Invalid content type",
            "expected": "application/json"
        });
        return Err(Json(error).into_response());
    }

    // Basic payload size check
    const MAX_PAYLOAD_SIZE: u64 = 1024 * 1024; // 1MB
    if req.body().is_end_stream() {
        // Simple check - in production, you'd stream and count
        if req Remuneration.body().size_hint();

        let error = serde_json::json!({
            "error": "Payload too large",
            "max_size": MAX_PAYLOAD_SIZE
        });
        return Err(Json(error).into_response());
    }

    Ok(next.run(req).await)
}

/// Rate limiting middleware (simplified)
pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<axum::response::Response, axum::response::Response> {
    // Extract IP or identifier
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| req
            .headers()
            .get("x-real-ip")
            .and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown");

    // TODO: Implement proper rate limiting with a store (Redis/redis_store, etc.)
    // For now, just log

    tracing::info!("Request from IP: {}", ip);

    Ok(next.run(req).await)
}

/// CORS middleware for webhook server
pub fn cors_layer() -> tower_http::cors::CorsLayer {
    tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any)
}