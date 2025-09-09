//! Streaming and cache management for diagnostic operations
//!
//! This module handles real-time diagnostic streaming, cache management,
//! and subscription-based diagnostic updates.

use crate::modules::shared::diagnostics::*;
use crate::commands::utils::*;
use crate::commands::utils::cache::*;
use crate::commands::utils::errors::*;
use std::time::{SystemTime, Duration};
use tauri::State;
use uuid;
use tokio::time;

/// Subscribe to real-time diagnostic updates
#[tauri::command]
pub async fn subscribe_to_diagnostics(
    request: DiagnosticStreamRequest,
    stream_state: State<'_, DiagnosticStreamState>,
) -> Result<String, String> {
    async_command!("Subscribing to diagnostics", {
        let stream_id = uuid::Uuid::new_v4().to_string();

        let stream = DiagnosticStream {
            id: stream_id.clone(),
            workspace_path: request.workspace_path.clone(),
            is_active: true,
            last_update: SystemTime::now(),
            subscribers: vec![request.subscriber_id],
        };

        {
            let mut stream_guard = stream_state.write().await;
            stream_guard.insert(stream_id.clone(), stream);
        }

        // Start background task for periodic updates if auto-refresh is enabled
        if let Some(interval) = request.auto_refresh_interval_seconds {
            let stream_state_clone = stream_state.inner().clone();
            let stream_id_clone = stream_id.clone();
            let workspace_path = request.workspace_path.clone();

            tokio::spawn(async move {
                let mut interval_timer = time::interval(Duration::from_secs(interval));

                loop {
                    interval_timer.tick().await;

                    // Check if stream is still active
                    {
                        let stream_guard = stream_state_clone.read().await;
                        if let Some(stream) = stream_guard.get(&stream_id_clone) {
                            if !stream.is_active {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    // Run diagnostics and emit update
                    if let Ok(diagnostics) = run_cargo_check(&workspace_path).await {
                        // In a real implementation, you would emit this to subscribers
                        log::debug!("Diagnostic update available for stream: {}", stream_id_clone);
                    }
                }
            });
        }

        Ok(stream_id)
    }).await
}

/// Unsubscribe from diagnostic updates
#[tauri::command]
pub async fn unsubscribe_from_diagnostics(
    stream_id: String,
    subscriber_id: String,
    stream_state: State<'_, DiagnosticStreamState>,
) -> Result<String, String> {
    async_command!("Unsubscribing from diagnostics", {
        let mut stream_guard = stream_state.write().await;

        if let Some(stream) = stream_guard.get_mut(&stream_id) {
            stream.subscribers.retain(|id| id != &subscriber_id);

            // If no more subscribers, deactivate the stream
            if stream.subscribers.is_empty() {
                stream.is_active = false;
            }

            Ok("Unsubscribed successfully".to_string())
        } else {
            Err("Stream not found".to_string())
        }
    }).await
}

/// Clear diagnostic cache
#[tauri::command]
pub async fn clear_diagnostic_cache(
    diagnostic_cache: State<'_, DiagnosticCacheState>,
    explanation_cache: State<'_, ExplanationCacheState>,
) -> Result<String, String> {
    async_command!("Clearing diagnostic caches", {
        {
            let mut cache_guard = diagnostic_cache.write().await;
            cache_guard.clear();
        }

        {
            let mut cache_guard = explanation_cache.write().await;
            cache_guard.clear();
        }

        Ok("Caches cleared successfully")
    }).await
}

/// Get cache statistics
#[tauri::command]
pub async fn get_cache_statistics(
    diagnostic_cache: State<'_, DiagnosticCacheState>,
    explanation_cache: State<'_, ExplanationCacheState>,
) -> Result<CacheStatistics, String> {
    let diagnostic_cache_guard = diagnostic_cache.read().await;
    let explanation_cache_guard = explanation_cache.read().await;

    let stats = CacheStatistics {
        diagnostic_cache_size: diagnostic_cache_guard.len(),
        diagnostic_cache_max_size: diagnostic_cache_guard.max_entries,
        explanation_cache_size: explanation_cache_guard.len(),
        explanation_cache_max_size: explanation_cache_guard.max_entries,
        diagnostic_cache_hit_ratio: 0.0, // Would need to track hits/misses
        explanation_cache_hit_ratio: 0.0, // Would need to track hits/misses
    };

    Ok(stats)
}

/// Cache statistics structure
#[derive(Debug, serde::Serialize)]
pub struct CacheStatistics {
    pub diagnostic_cache_size: usize,
    pub diagnostic_cache_max_size: usize,
    pub explanation_cache_size: usize,
    pub explanation_cache_max_size: usize,
    pub diagnostic_cache_hit_ratio: f32,
    pub explanation_cache_hit_ratio: f32,
}