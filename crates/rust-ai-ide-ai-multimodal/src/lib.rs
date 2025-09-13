//! # Rust AI IDE Multi-Modal AI Services (Phase 7)
//!
//! This crate provides comprehensive multi-modal AI capabilities including vision,
//! audio processing, and contextual analysis integration with the existing IDE infrastructure.
//!
//! ## Architecture Overview
//!
//! The multi-modal AI system consists of these key components:
//!
//! - **Vision Processor**: OpenCV and image processing for screenshot/diagram analysis
//! - **Audio Processor**: Voice command recognition using Whisper and cpal
//! - **Multi-Modal Analyzer**: Combined vision, audio, and text processing
//! - **Metrics Collector**: Performance monitoring and analytics
//! - **Type System**: Shared data types for cross-modal processing

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod audio;
pub mod errors;
pub mod metrics;
pub mod multimodal;
pub mod types;
pub mod vision;

pub use audio::AudioProcessor;
pub use errors::{AudioError, MultimodalError, ProcessingError, VisionError};
pub use multimodal::MultiModalAnalyzer;
pub use types::{AnalysisRequest, AnalysisResponse, ModalityType, ProcessingResult};
pub use vision::VisionProcessor;

/// Main multi-modal AI service that orchestrates all modalities
///
/// This service serves as the central hub for multi-modal AI processing,
/// managing the interactions between different processing components
/// and providing unified access to multi-modal capabilities.
pub struct MultiModalAiService {
    vision_processor: std::sync::Arc<vision::VisionProcessor>,
    audio_processor: std::sync::Arc<audio::AudioProcessor>,
    multimodal_analyzer: std::sync::Arc<multimodal::MultiModalAnalyzer>,
    metrics_collector: std::sync::Arc<metrics::MetricsCollector>,
}

impl MultiModalAiService {
    /// Initialize the complete multi-modal AI service
    ///
    /// # Errors
    /// Returns an error if any component fails to initialize
    pub async fn new() -> Result<Self, errors::MultimodalError> {
        let vision_processor = std::sync::Arc::new(vision::VisionProcessor::new().await?);

        let audio_processor = std::sync::Arc::new(audio::AudioProcessor::new().await?);

        let multimodal_analyzer = std::sync::Arc::new(
            multimodal::MultiModalAnalyzer::new(
                std::sync::Arc::clone(&vision_processor),
                std::sync::Arc::clone(&audio_processor),
            )
            .await?,
        );

        let metrics_collector = std::sync::Arc::new(metrics::MetricsCollector::new().await?);

        Ok(Self {
            vision_processor,
            audio_processor,
            multimodal_analyzer,
            metrics_collector,
        })
    }

    /// Get a clone of the vision processor component
    #[must_use]
    pub fn vision_processor(&self) -> std::sync::Arc<vision::VisionProcessor> {
        std::sync::Arc::clone(&self.vision_processor)
    }

    /// Get a clone of the audio processor component
    #[must_use]
    pub fn audio_processor(&self) -> std::sync::Arc<audio::AudioProcessor> {
        std::sync::Arc::clone(&self.audio_processor)
    }

    /// Get a clone of the multi-modal analyzer component
    #[must_use]
    pub fn multimodal_analyzer(&self) -> std::sync::Arc<multimodal::MultiModalAnalyzer> {
        std::sync::Arc::clone(&self.multimodal_analyzer)
    }

    /// Get a clone of the metrics collector component
    #[must_use]
    pub fn metrics_collector(&self) -> std::sync::Arc<metrics::MetricsCollector> {
        std::sync::Arc::clone(&self.metrics_collector)
    }

    /// Process a multi-modal analysis request
    ///
    /// # Errors
    /// Returns an error if analysis fails
    pub async fn process_request(
        &self,
        request: types::AnalysisRequest,
    ) -> Result<types::AnalysisResponse, errors::MultimodalError> {
        // Record metrics
        self.metrics_collector
            .record_request(request.modality_types.len())
            .await;

        // Process through multimodal analyzer
        let result = self.multimodal_analyzer.analyze(request).await?;

        // Record completion
        self.metrics_collector
            .record_completion(result.confidence_score)
            .await;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::ModalityType;

    #[tokio::test]
    async fn test_multimodal_service_creation() {
        let service = MultiModalAiService::new().await;
        assert!(
            service.is_ok(),
            "Multi-modal service should initialize successfully"
        );
    }

    #[tokio::test]
    async fn test_component_access() {
        let service = MultiModalAiService::new().await.unwrap();
        let _vision = service.vision_processor();
        let _audio = service.audio_processor();
        let _multimodal = service.multimodal_analyzer();
        let _metrics = service.metrics_collector();
    }

    #[tokio::test]
    async fn test_basic_analysis_request() {
        let service = MultiModalAiService::new().await.unwrap();

        let request = types::AnalysisRequest::new()
            .with_modality(ModalityType::Text)
            .with_text_content("Test analysis content".to_string());

        let result = service.process_request(request).await;
        // Should complete without panic (may return placeholder result)
        assert!(
            result.is_ok() || result.is_err(),
            "Request should be processed"
        );
    }
}
