//! Shared types for multi-modal AI processing
//!
//! This module defines the data types used throughout the multi-modal AI system,
//! including analysis requests, responses, and modality definitions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of modalities supported by the multi-modal AI system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModalityType {
    /// Text content analysis
    Text,
    /// Image or screenshot analysis
    Image,
    /// Audio or voice input
    Audio,
    /// Screenshot analysis
    Screenshot,
    /// Diagram recognition
    Diagram,
    /// Code snippet analysis
    Code,
    /// Mixed modalities combination
    Multimodal,
}

impl std::fmt::Display for ModalityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModalityType::Text => write!(f, "text"),
            ModalityType::Image => write!(f, "image"),
            ModalityType::Audio => write!(f, "audio"),
            ModalityType::Screenshot => write!(f, "screenshot"),
            ModalityType::Diagram => write!(f, "diagram"),
            ModalityType::Code => write!(f, "code"),
            ModalityType::Multimodal => write!(f, "multimodal"),
        }
    }
}

/// Main request structure for multi-modal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    /// Unique identifier for the request
    pub id:               Uuid,
    /// Types of modalities to analyze
    pub modality_types:   Vec<ModalityType>,
    /// Text content (for text modality)
    pub text_content:     Option<String>,
    /// Image data as base64 (for image modality)
    pub image_data:       Option<String>,
    /// Audio data as base64 (for audio modality)
    pub audio_data:       Option<String>,
    /// Additional metadata
    pub metadata:         HashMap<String, serde_json::Value>,
    /// Processing options
    pub options:          ProcessingOptions,
    /// Indicate if this is just a query for supported capabilities
    pub capability_query: bool,
}

impl AnalysisRequest {
    /// Create a new analysis request with default options
    #[must_use]
    pub fn new() -> Self {
        Self {
            id:               Uuid::new_v4(),
            modality_types:   Vec::new(),
            text_content:     None,
            image_data:       None,
            audio_data:       None,
            metadata:         HashMap::new(),
            options:          ProcessingOptions::default(),
            capability_query: false,
        }
    }

    /// Add a modality type
    #[must_use]
    pub fn with_modality(mut self, modality: ModalityType) -> Self {
        self.modality_types.push(modality);
        self
    }

    /// Set text content
    #[must_use]
    pub fn with_text_content(mut self, text: String) -> Self {
        self.text_content = Some(text);
        self
    }

    /// Set image data as base64
    #[must_use]
    pub fn with_image_data(mut self, data: String) -> Self {
        self.image_data = Some(data);
        self
    }

    /// Set audio data as base64
    #[must_use]
    pub fn with_audio_data(mut self, data: String) -> Self {
        self.audio_data = Some(data);
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Set as capability query
    #[must_use]
    pub fn as_capability_query(mut self) -> Self {
        self.capability_query = true;
        self
    }
}

impl Default for AnalysisRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Processing options for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    /// Enable GPU acceleration if available
    pub gpu_acceleration:     bool,
    /// Timeout in seconds
    pub timeout_seconds:      u32,
    /// Enable caching of results
    pub enable_caching:       bool,
    /// Minimum confidence threshold (0.0 to 1.0)
    pub confidence_threshold: f32,
    /// Enable detailed logging
    pub detailed_logging:     bool,
    /// Language for text processing
    pub language:             Option<String>,
    /// Expected image format (auto-detect if None)
    pub image_format:         Option<String>,
    /// Audio sample rate (auto-detect if None)
    pub audio_sample_rate:    Option<u32>,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            gpu_acceleration:     true,
            timeout_seconds:      30,
            enable_caching:       true,
            confidence_threshold: 0.7,
            detailed_logging:     false,
            language:             Some("en".to_string()),
            image_format:         None,
            audio_sample_rate:    None,
        }
    }
}

/// Main response structure from multi-modal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    /// Request ID that this response corresponds to
    pub request_id:             Uuid,
    /// Processing timestamp
    pub timestamp:              chrono::DateTime<chrono::Utc>,
    /// Overall confidence score (0.0 to 1.0)
    pub confidence_score:       f32,
    /// Results by modality
    pub modality_results:       HashMap<ModalityType, ModalityResult>,
    /// Combined multimodal result
    pub combined_result:        Option<CombinedResult>,
    /// Processing duration in milliseconds
    pub processing_duration_ms: u64,
    /// Whether the processing was successful
    pub success:                bool,
    /// Error message if any
    pub error_message:          Option<String>,
    /// Metadata about the processing
    pub processing_metadata:    HashMap<String, serde_json::Value>,
}

/// Result for a specific modality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityResult {
    /// Modality type
    pub modality_type:      ModalityType,
    /// Whether processing was successful for this modality
    pub success:            bool,
    /// Confidence score (0.0 to 1.0)
    pub confidence:         f32,
    /// Extracted data
    pub data:               ModalityData,
    /// Bounding boxes for image results
    pub bounding_boxes:     Vec<BoundingBox>,
    /// Processing time for this modality
    pub processing_time_ms: u64,
}

/// Different types of extracted data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModalityData {
    /// Text content
    Text {
        /// Full text content
        content:  String,
        /// Detected language
        language: String,
        /// Key phrases or entities
        entities: Vec<Entity>,
    },
    /// Image-based results
    Image {
        /// Image description
        description: String,
        /// Objects detected in the image
        objects:     Vec<Detection>,
        /// OCR text if applicable
        ocr_text:    Option<String>,
        /// Scene analysis
        scene:       Option<String>,
    },
    /// Audio-based results
    Audio {
        /// Transcribed text
        transcription: String,
        /// Language spoken
        language:      String,
        /// Speaker identification if available
        speakers:      Vec<SpeakerSegment>,
        /// Audio events or sounds detected
        audio_events:  Vec<AudioEvent>,
    },
    /// Multi-modal combined results
    Multimodal {},
}

/// Entity extracted from text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity text
    pub text:           String,
    /// Entity type (person, organization, etc.)
    pub entity_type:    String,
    /// Confidence score
    pub confidence:     f32,
    /// Position in the text
    pub start_position: usize,
    pub end_position:   usize,
}

/// Object detected in an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    /// Class name
    pub class:      String,
    /// Confidence score
    pub confidence: f32,
    /// Bounding box
    pub bbox:       BoundingBox,
}

/// Bounding box coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    /// X coordinate of top-left corner
    pub x1: f32,
    /// Y coordinate of top-left corner
    pub y1: f32,
    /// X coordinate of bottom-right corner
    pub x2: f32,
    /// Y coordinate of bottom-right corner
    pub y2: f32,
}

/// Speaker segment in audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerSegment {
    /// Speaker ID
    pub speaker_id: String,
    /// Start time in seconds
    pub start_time: f32,
    /// End time in seconds
    pub end_time:   f32,
    /// Confidence score
    pub confidence: f32,
}

/// Audio event detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEvent {
    /// Event type (speech, music, etc.)
    pub event_type: String,
    /// Confidence score
    pub confidence: f32,
    /// Start time in seconds
    pub start_time: f32,
    /// End time in seconds
    pub end_time:   f32,
}

/// Combined result for multi-modal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedResult {
    /// Overall fused understanding
    pub fused_understanding: String,
    /// Cross-modal relationships
    pub relationships:       Vec<CrossModalRelationship>,
    /// Higher-level insights
    pub insights:            Vec<String>,
    /// Recommendations
    pub recommendations:     Vec<String>,
}

/// Relationship between modalities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModalRelationship {
    /// Source modality
    pub source_modality:   ModalityType,
    /// Target modality
    pub target_modality:   ModalityType,
    /// Type of relationship
    pub relationship_type: String,
    /// Confidence score
    pub confidence:        f32,
    /// Description of the relationship
    pub description:       String,
}

/// Capabilities query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesResponse {
    /// Supported modalities
    pub supported_modalities: Vec<ModalityType>,
    /// Supported languages
    pub supported_languages:  Vec<String>,
    /// Maximum input size limits
    pub max_input_sizes:      HashMap<String, u64>,
    /// Whether GPU acceleration is available
    pub gpu_available:        bool,
    /// Version information
    pub version:              String,
}

/// Processing result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult<T> {
    /// Result data
    pub data:        Option<T>,
    /// Error message if any
    pub error:       Option<String>,
    /// Processing duration
    pub duration_ms: u64,
    /// Whether result is from cache
    pub cached:      bool,
}

impl<T> ProcessingResult<T> {
    /// Create a successful result
    #[must_use]
    pub fn success(data: T, duration_ms: u64, cached: bool) -> Self {
        Self {
            data: Some(data),
            error: None,
            duration_ms,
            cached,
        }
    }

    /// Create an error result
    #[must_use]
    pub fn error(error: String, duration_ms: u64) -> Self {
        Self {
            data: None,
            error: Some(error),
            duration_ms,
            cached: false,
        }
    }

    /// Check if result is successful
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.data.is_some()
    }

    /// Get the result data
    #[must_use]
    pub fn get_data(self) -> Option<T> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modality_type_display() {
        assert_eq!(ModalityType::Text.to_string(), "text");
        assert_eq!(ModalityType::Image.to_string(), "image");
        assert_eq!(ModalityType::Audio.to_string(), "audio");
    }

    #[test]
    fn test_analysis_request_builder() {
        let request = AnalysisRequest::new()
            .with_modality(ModalityType::Text)
            .with_text_content("test content".to_string())
            .with_metadata("test_key", serde_json::json!("test_value"));

        assert!(request.modality_types.contains(&ModalityType::Text));
        assert_eq!(request.text_content.as_ref().unwrap(), "test content");
        assert!(request.metadata.contains_key("test_key"));
    }

    #[test]
    fn test_processing_result() {
        let success_result = ProcessingResult::success("data", 100, false);
        assert!(success_result.is_success());
        assert!(success_result.cached == false);

        let error_result = ProcessingResult::error("test error".to_string(), 50);
        assert!(!error_result.is_success());
        assert_eq!(error_result.error.unwrap(), "test error");
    }

    #[test]
    fn test_modality_data_deserialization() {
        let text_data = ModalityData::Text {
            content:  "Hello world".to_string(),
            language: "en".to_string(),
            entities: Vec::new(),
        };

        assert!(matches!(text_data, ModalityData::Text { .. }));
    }
}
