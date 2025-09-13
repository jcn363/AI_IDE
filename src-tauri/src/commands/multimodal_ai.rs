//! Multi-modal AI command handlers
//!
//! This module provides Tauri command handlers for multi-modal AI operations
//! including vision processing, audio analysis, and combined modality processing.

use std::sync::Arc;

use rust_ai_ide_ai_multimodal::{AnalysisRequest, ModalityType, MultiModalAiService};
use rust_ai_ide_common::validation::TauriInputSanitizer;
use serde_json::json;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

/// State for multi-modal AI service management
#[derive(Default)]
pub struct MultiModalState {
    /// Multi-modal AI service instance
    pub multimodal_service: Option<Arc<MultiModalAiService>>,
}

impl MultiModalState {
    /// Initialize the multi-modal service
    pub async fn initialize(&mut self) -> Result<(), tauri::Error> {
        let multimodal_service = MultiModalAiService::new().await.map_err(|e| {
            tauri::Error::Anyhow(anyhow::anyhow!(
                "Failed to initialize multimodal service: {}",
                e
            ))
        })?;

        self.multimodal_service = Some(Arc::new(multimodal_service));
        Ok(())
    }

    /// Get the multimodal service reference
    #[must_use]
    pub fn service(&self) -> Result<&Arc<MultiModalAiService>, tauri::Error> {
        self.multimodal_service
            .as_ref()
            .ok_or_else(|| tauri::Error::Anyhow(anyhow::anyhow!("Multimodal service not initialized")))
    }
}

/// Initialize multi-modal AI services on app startup
#[tauri::command]
pub async fn initialize_multimodal_ai(
    app_handle: AppHandle,
    multimodal_state: State<'_, Arc<Mutex<MultiModalState>>>,
) -> Result<(), tauri::Error> {
    log::info!("Initializing multi-modal AI services...");

    // Double-locking pattern for async state initialization
    let needs_initialization = {
        let state_guard = multimodal_state.lock().await;
        state_guard.multimodal_service.is_none()
    };

    if needs_initialization {
        log::debug!("Initializing multimodal service...");

        let mut state_guard = multimodal_state.lock().await;
        state_guard.initialize().await?;
    }

    log::info!("Multi-modal AI services initialized successfully");
    Ok(())
}

/// Process multi-modal analysis request
#[tauri::command]
pub async fn process_multimodal_analysis(
    request: serde_json::Value,
    multimodal_state: State<'_, Arc<Mutex<MultiModalState>>>,
    _sanitizer: State<'_, TauriInputSanitizer>,
) -> Result<serde_json::Value, tauri::Error> {
    let service = {
        let state_guard = multimodal_state.lock().await;
        Arc::clone(state_guard.service()?)
    };

    // Parse the request
    let modality_types: Vec<ModalityType> = request
        .get("modality_types")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| match s {
                    "text" => Some(ModalityType::Text),
                    "image" => Some(ModalityType::Image),
                    "audio" => Some(ModalityType::Audio),
                    "screenshot" => Some(ModalityType::Screenshot),
                    "diagram" => Some(ModalityType::Diagram),
                    "code" => Some(ModalityType::Code),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    let mut analysis_request = AnalysisRequest::new().with_modality(ModalityType::Text);

    // Add modalities
    for modality in modality_types {
        analysis_request = analysis_request.with_modality(modality);
    }

    // Add text content if provided
    if let Some(text) = request.get("text_content").and_then(|t| t.as_str()) {
        analysis_request = analysis_request.with_text_content(text.to_string());
    }

    // Add image data if provided
    if let Some(image_data) = request.get("image_data").and_then(|i| i.as_str()) {
        analysis_request = analysis_request.with_image_data(image_data.to_string());
        analysis_request = analysis_request.with_modality(ModalityType::Image);
    }

    // Add audio data if provided
    if let Some(audio_data) = request.get("audio_data").and_then(|a| a.as_str()) {
        analysis_request = analysis_request.with_audio_data(audio_data.to_string());
        analysis_request = analysis_request.with_modality(ModalityType::Audio);
    }

    // Process the analysis
    match service.process_request(analysis_request).await {
        Ok(response) => {
            // Convert the response to JSON-compatible format
            let results_json: Vec<serde_json::Value> = response
                .modality_results
                .into_iter()
                .map(|(modality, result)| {
                    json!({
                        "modality_type": modality.to_string(),
                        "success": result.success,
                        "confidence": result.confidence,
                        "data": match &result.data {
                            rust_ai_ide_ai_multimodal::ModalityData::Text { content, language, entities } => {
                                json!({
                                    "type": "text",
                                    "content": content,
                                    "language": language,
                                    "entities": entities.iter().map(|e| json!({
                                        "text": e.text,
                                        "entity_type": e.entity_type,
                                        "confidence": e.confidence
                                    })).collect::<Vec<_>>()
                                })
                            },
                            rust_ai_ide_ai_multimodal::ModalityData::Image { description, objects, ocr_text, scene } => {
                                json!({
                                    "type": "image",
                                    "description": description,
                                    "objects": objects.iter().map(|o| json!({
                                        "class": o.class,
                                        "confidence": o.confidence
                                    })).collect::<Vec<_>>(),
                                    "ocr_text": ocr_text,
                                    "scene": scene
                                })
                            },
                            rust_ai_ide_ai_multimodal::ModalityData::Audio { transcription, language, speakers, audio_events } => {
                                json!({
                                    "type": "audio",
                                    "transcription": transcription,
                                    "language": language,
                                    "speakers": speakers.iter().map(|s| json!({
                                        "speaker_id": s.speaker_id,
                                        "confidence": s.confidence
                                    })).collect::<Vec<_>>(),
                                    "audio_events": audio_events.iter().map(|e| json!({
                                        "event_type": e.event_type,
                                        "confidence": e.confidence
                                    })).collect::<Vec<_>>()
                                })
                            },
                            rust_ai_ide_ai_multimodal::ModalityData::Multimodal {} => {
                                json!({
                                    "type": "multimodal",
                                    "description": "Multi-modal combined result"
                                })
                            }
                        },
                        "processing_time_ms": result.processing_time_ms
                    })
                })
                .collect();

            Ok(json!({
                "success": response.success,
                "results": results_json,
                "processing_time": response.processing_duration_ms,
                "timestamp": response.timestamp.timestamp()
            }))
        }
        Err(e) => {
            log::error!("Multi-modal analysis failed: {}", e);
            Ok(json!({
                "success": false,
                "error": e.to_string(),
                "results": [],
                "processing_time": 0
            }))
        }
    }
}

/// Analyze image content
#[tauri::command]
pub async fn analyze_image(
    image_data: String,
    image_format: Option<String>,
    multimodal_state: State<'_, Arc<Mutex<MultiModalState>>>,
    _sanitizer: State<'_, TauriInputSanitizer>,
) -> Result<serde_json::Value, tauri::Error> {
    let service = {
        let state_guard = multimodal_state.lock().await;
        Arc::clone(state_guard.service()?)
    };

    let vision_processor = service.vision_processor();
    let analysis_request = AnalysisRequest::new()
        .with_modality(ModalityType::Image)
        .with_image_data(image_data)
        .as_capability_query(); // For basic analysis

    match service.process_request(analysis_request).await {
        Ok(response) =>
            if let Some(result) = response.modality_results.get(&ModalityType::Image) {
                match &result.data {
                    rust_ai_ide_ai_multimodal::ModalityData::Image {
                        description,
                        objects,
                        ocr_text,
                        ..
                    } => Ok(json!({
                        "success": result.success,
                        "description": description,
                        "objects": objects.iter().map(|o| json!({
                            "class": o.class,
                            "confidence": o.confidence,
                            "bbox": [o.bbox.x1, o.bbox.y1, o.bbox.x2, o.bbox.y2]
                        })).collect::<Vec<_>>(),
                        "ocr_text": ocr_text,
                        "confidence": result.confidence,
                        "processing_time_ms": result.processing_time_ms
                    })),
                    _ => Ok(json!({"error": "Unexpected result format"})),
                }
            } else {
                Ok(json!({"error": "No image result found"}))
            },
        Err(e) => Ok(json!({"error": format!("Image analysis failed: {}", e)})),
    }
}

/// Recognize voice command
#[tauri::command]
pub async fn recognize_voice_command(
    audio_data: String,
    language: Option<String>,
    multimodal_state: State<'_, Arc<Mutex<MultiModalState>>>,
    _sanitizer: State<'_, TauriInputSanitizer>,
) -> Result<serde_json::Value, tauri::Error> {
    let service = {
        let state_guard = multimodal_state.lock().await;
        Arc::clone(state_guard.service()?)
    };

    let analysis_request = AnalysisRequest::new()
        .with_modality(ModalityType::Audio)
        .with_audio_data(audio_data);

    if let Some(lang) = language {
        analysis_request
            .metadata
            .insert("language".to_string(), json!(lang));
    }

    match service.process_request(analysis_request).await {
        Ok(response) =>
            if let Some(result) = response.modality_results.get(&ModalityType::Audio) {
                match &result.data {
                    rust_ai_ide_ai_multimodal::ModalityData::Audio {
                        transcription,
                        language: detected_lang,
                        speakers,
                        ..
                    } => Ok(json!({
                        "success": result.success,
                        "transcription": transcription,
                        "language": detected_lang,
                        "speakers": speakers.len(),
                        "confidence": result.confidence,
                        "processing_time_ms": result.processing_time_ms
                    })),
                    _ => Ok(json!({"error": "Unexpected result format"})),
                }
            } else {
                Ok(json!({"error": "No audio result found"}))
            },
        Err(e) => Ok(json!({"error": format!("Voice recognition failed: {}", e)})),
    }
}

/// Get multi-modal service capabilities
#[tauri::command]
pub async fn get_multimodal_capabilities(
    multimodal_state: State<'_, Arc<Mutex<MultiModalState>>>,
    _sanitizer: State<'_, TauriInputSanitizer>,
) -> Result<serde_json::Value, tauri::Error> {
    let service = {
        let state_guard = multimodal_state.lock().await;
        Arc::clone(state_guard.service()?)
    };

    let capabilities = AnalysisRequest::new().as_capability_query();

    // Get metrics to understand current capabilities
    let metrics = service
        .metrics_collector()
        .get_metrics()
        .await
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("Failed to get metrics: {}", e)))?;

    Ok(json!({
        "supported_modalities": [
            "text",
            "image",
            "audio",
            "screenshot",
            "diagram",
            "code"
        ],
        "gpu_acceleration_available": true,
        "current_metrics": {
            "total_requests": metrics.request_count,
            "avg_confidence": metrics.avg_confidence,
            "avg_processing_time_ms": metrics.avg_processing_time
        },
        "max_input_sizes": {
            "image_mb": 10,
            "audio_mb": 50,
            "text_chars": 10000
        },
        "version": "0.1.0",
        "status": "operational"
    }))
}

/// Get performance metrics for multi-modal AI
#[tauri::command]
pub async fn get_multimodal_metrics(
    multimodal_state: State<'_, Arc<Mutex<MultiModalState>>>,
    _sanitizer: State<'_, TauriInputSanitizer>,
) -> Result<serde_json::Value, tauri::Error> {
    let service = {
        let state_guard = multimodal_state.lock().await;
        Arc::clone(state_guard.service()?)
    };

    let metrics = service
        .metrics_collector()
        .get_metrics()
        .await
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("Failed to get metrics: {}", e)))?;

    Ok(json!({
        "request_count": metrics.request_count,
        "total_processing_time_ms": metrics.total_processing_time,
        "avg_confidence": metrics.avg_confidence,
        "error_count": metrics.error_count,
        "avg_processing_time_ms": metrics.avg_processing_time,
        "cache_hit_rate": metrics.cache_hit_rate,
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// Register multimodal AI commands
pub fn register_commands(app: &mut tauri::App) -> Result<(), tauri::Error> {
    // Initialize multimodal service state
    let multimodal_state = MultiModalState::default();
    app.manage(Arc::new(Mutex::new(multimodal_state)));

    // Register input sanitizer (reused from AI commands)
    app.manage(TauriInputSanitizer::default());

    log::info!("Multi-modal AI commands registered successfully");
    Ok(())
}
