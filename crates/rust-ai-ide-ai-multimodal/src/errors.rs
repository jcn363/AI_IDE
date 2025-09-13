//! Error handling for multi-modal AI services
//!
//! This module defines the error types and handling patterns for the multi-modal
//! AI processing system, following the project's error handling guidelines.

use std::fmt;

use thiserror::Error;

/// Main error type for multi-modal AI operations
#[derive(Error, Debug)]
pub enum MultimodalError {
    /// Vision processing error
    #[error("Vision processing error: {0}")]
    Vision(#[from] VisionError),

    /// Audio processing error
    #[error("Audio processing error: {0}")]
    Audio(#[from] AudioError),

    /// General processing error
    #[error("Processing error: {0}")]
    Processing(#[from] ProcessingError),

    /// Initialization error
    #[error("Initialization failed: {0}")]
    Initialization(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with context
    #[error("Generic error: {0}")]
    Generic(String),
}

/// Vision processing specific errors
#[derive(Error, Debug)]
pub enum VisionError {
    /// OpenCV initialization error
    #[error("OpenCV initialization failed: {0}")]
    OpencvInit(String),

    /// Image processing error
    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    /// OCR processing error
    #[error("OCR processing error: {0}")]
    OcrProcessing(String),

    /// Model inference error
    #[error("Vision model inference error: {0}")]
    ModelInference(String),

    /// Image format error
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),

    /// File not found
    #[error("Image file not found: {0}")]
    ImageNotFound(String),

    /// Permission denied
    #[error("Image access denied: {0}")]
    ImageAccessDenied(String),
}

/// Audio processing specific errors
#[derive(Error, Debug)]
pub enum AudioError {
    /// Audio device error
    #[error("Audio device error: {0}")]
    Device(String),

    /// Speech recognition error
    #[error("Speech recognition error: {0}")]
    SpeechRecognition(String),

    /// Audio format error
    #[error("Unsupported audio format: {0}")]
    UnsupportedFormat(String),

    /// Model loading error
    #[error("Audio model loading failed: {0}")]
    ModelLoading(String),

    /// Recording error
    #[error("Audio recording error: {0}")]
    Recording(String),

    /// Microphone access denied
    #[error("Microphone access denied: {0}")]
    MicrophoneAccessDenied(String),

    /// Playback error
    #[error("Audio playback error: {0}")]
    Playback(String),
}

/// General processing specific errors
#[derive(Error, Debug)]
pub enum ProcessingError {
    /// Multi-modal fusion error
    #[error("Multi-modal fusion error: {0}")]
    Fusion(String),

    /// Model loading error
    #[error("Model loading error: {0}")]
    ModelLoading(String),

    /// Caching error
    #[error("Cache operation error: {0}")]
    Caching(String),

    /// Resource allocation error
    #[error("Resource allocation error: {0}")]
    ResourceAllocation(String),

    /// Timeout error
    #[error("Processing timeout: {0}")]
    Timeout(String),

    /// Memory limit exceeded
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),

    /// GPU processing error
    #[error("GPU processing error: {0}")]
    GpuProcessing(String),

    /// Invalid input
    #[error("Invalid input data: {0}")]
    InvalidInput(String),

    /// Network error
    #[error("Network request failed: {0}")]
    NetworkError(String),
}

impl From<serde_json::Error> for MultimodalError {
    fn from(err: serde_json::Error) -> Self {
        MultimodalError::Generic(err.to_string())
    }
}

impl From<base64::DecodeError> for MultimodalError {
    fn from(err: base64::DecodeError) -> Self {
        MultimodalError::Generic(format!("Base64 decode error: {}", err))
    }
}

impl From<uuid::Error> for MultimodalError {
    fn from(err: uuid::Error) -> Self {
        MultimodalError::Generic(format!("UUID error: {}", err))
    }
}

impl From<regex::Error> for MultimodalError {
    fn from(err: regex::Error) -> Self {
        MultimodalError::Generic(format!("Regex error: {}", err))
    }
}

/// Extension trait for Result with MultimodalError
pub trait ResultExt<T> {
    /// Aggregate errors at function boundaries
    fn aggregate_error(self) -> Result<T, MultimodalError>;
}

impl<T> ResultExt<T> for Result<T, MultimodalError> {
    fn aggregate_error(self) -> Result<T, MultimodalError> {
        // In this implementation, we're just returning self as is
        // In practice, this would aggregate multiple errors if needed
        self
    }
}

/// Helper function to create a generic error
#[must_use]
pub fn generic_error(msg: impl Into<String>) -> MultimodalError {
    MultimodalError::Generic(msg.into())
}

/// Helper function to create an initialization error
#[must_use]
pub fn initialization_error(msg: impl Into<String>) -> MultimodalError {
    MultimodalError::Initialization(msg.into())
}

/// Helper function to create a configuration error
#[must_use]
pub fn configuration_error(msg: impl Into<String>) -> MultimodalError {
    MultimodalError::Configuration(msg.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_error_display() {
        let error = VisionError::ImageProcessing("format error".to_string());
        assert!(error.to_string().contains("Image processing error"));
    }

    #[test]
    fn test_audio_error_display() {
        let error = AudioError::Device("no device".to_string());
        assert!(error.to_string().contains("Audio device error"));
    }

    #[test]
    fn test_processing_error_display() {
        let error = ProcessingError::Timeout("60s".to_string());
        assert!(error.to_string().contains("Processing timeout"));
    }

    #[test]
    fn test_multimodal_error_from_vision() {
        let vision_err = VisionError::OpencvInit("failed".to_string());
        let multimodal_err: MultimodalError = vision_err.into();
        assert!(multimodal_err
            .to_string()
            .contains("Vision processing error"));
    }

    #[test]
    fn test_multimodal_error_from_audio() {
        let audio_err = AudioError::MicrophoneAccessDenied(String::new());
        let multimodal_err: MultimodalError = audio_err.into();
        assert!(multimodal_err
            .to_string()
            .contains("Audio processing error"));
    }

    #[test]
    fn test_helper_functions() {
        let generic = generic_error("test");
        assert!(matches!(generic, MultimodalError::Generic(_)));

        let init = initialization_error("test");
        assert!(matches!(init, MultimodalError::Initialization(_)));

        let config = configuration_error("test");
        assert!(matches!(config, MultimodalError::Configuration(_)));
    }
}
