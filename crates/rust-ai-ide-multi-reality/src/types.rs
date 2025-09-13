//! Core types and data structures for the multi-reality coordination system
//!
//! This module defines all the fundamental types used throughout the multi-reality
//! system, including configurations, spatial positioning, device representations,
//! and event types.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use rust_ai_ide_errors::IDEError;
use serde::{Deserialize, Serialize};

/// Represents different reality modes supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealityMode {
    /// Traditional desktop mode (2D interface)
    Desktop,
    /// Augmented Reality mode (AR overlays on physical world)
    AR,
    /// Virtual Reality mode (fully immersive 3D environment)
    VR,
    /// Mixed Reality mode (combination of AR and VR)
    MixedReality,
}

/// Configuration for the multi-reality coordination system
///
/// This struct contains all configuration options for initializing and running
/// the multi-reality system, including performance settings, device capabilities,
/// and security parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRealityConfig {
    /// Maximum number of concurrent AR/VR sessions
    pub max_concurrent_sessions:      usize,
    /// Performance mode for adaptive quality scaling
    pub performance_mode:             PerformanceMode,
    /// Device capabilities configuration
    pub device_caps:                  DeviceCapabilities,
    /// Security configuration for multi-reality features
    pub security_config:              SecurityConfig,
    /// WebXR configuration for browser-based features
    pub webxr_config:                 Option<WebXRConfig>,
    /// Collaboration settings
    pub collaboration_config:         CollaborationConfig,
    /// Spatial audio configuration
    pub spatial_audio_enabled:        bool,
    /// Adaptive quality scaling thresholds
    pub quality_scaling_thresholds:   QualityScalingThresholds,
    /// Connection pooling settings
    pub connection_pool_size:         usize,
    /// Session persistence duration
    pub session_persistence_duration: Duration,
}

impl Default for MultiRealityConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sessions:      10,
            performance_mode:             PerformanceMode::Balanced,
            device_caps:                  DeviceCapabilities::default(),
            security_config:              SecurityConfig::default(),
            webxr_config:                 None,
            collaboration_config:         CollaborationConfig::default(),
            spatial_audio_enabled:        true,
            quality_scaling_thresholds:   QualityScalingThresholds::default(),
            connection_pool_size:         20,
            session_persistence_duration: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Performance modes for adaptive quality scaling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceMode {
    /// Maximum performance - quality may be reduced
    Performance,
    /// Balanced quality and performance
    Balanced,
    /// Maximum quality - performance may be reduced
    Quality,
}

/// Device capabilities and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Minimum GPU memory required (in MB)
    pub min_gpu_memory_mb:      usize,
    /// Minimum CPU cores required
    pub min_cpu_cores:          usize,
    /// Minimum RAM required (in MB)
    pub min_ram_mb:             usize,
    /// Supported camera types for AR
    pub supported_camera_types: Vec<CameraType>,
    /// Supported VR headset types
    pub supported_vr_headsets:  Vec<VrHeadsetType>,
    /// WebXR support required
    pub webxr_required:         bool,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            min_gpu_memory_mb:      1024, // 1GB
            min_cpu_cores:          4,
            min_ram_mb:             8192, // 8GB
            supported_camera_types: vec![CameraType::RGB, CameraType::Depth],
            supported_vr_headsets:  vec![
                VrHeadsetType::MetaQuest,
                VrHeadsetType::ValveIndex,
                VrHeadsetType::OculusRift,
            ],
            webxr_required:         false,
        }
    }
}

/// Camera types supported by AR devices
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CameraType {
    /// Standard RGB camera
    RGB,
    /// Depth-sensing camera
    Depth,
    /// Fish-eye camera for wide field of view
    Fisheye,
    /// Infrared camera for tracking
    Infrared,
}

/// VR headset types supported by the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VrHeadsetType {
    /// Meta Quest series
    MetaQuest,
    /// Valve Index
    ValveIndex,
    /// Oculus Rift series
    OculusRift,
    /// HTC Vive series
    HtcVive,
    /// Windows Mixed Reality
    WindowsMixedReality,
    /// Generic WebXR compatible device
    WebXRVrDevice,
}

/// Security configuration for multi-reality features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// End-to-end encryption enabled
    pub encryption_enabled:     bool,
    /// Encryption algorithm for secure communication
    pub encryption_algorithm:   EncryptionAlgorithm,
    /// Session key rotation interval
    pub key_rotation_interval:  Duration,
    /// Access control for multi-reality sessions
    pub access_control_enabled: bool,
    /// Audit logging enabled
    pub audit_logging_enabled:  bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled:     true,
            encryption_algorithm:   EncryptionAlgorithm::Aes256Gcm,
            key_rotation_interval:  Duration::from_secs(1800), // 30 minutes
            access_control_enabled: true,
            audit_logging_enabled:  true,
        }
    }
}

/// Encryption algorithms supported for secure communication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}

/// WebXR configuration for browser-based AR/VR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebXRConfig {
    /// Required WebXR features
    pub required_features: Vec<WebXRFeature>,
    /// Optional WebXR features
    pub optional_features: Vec<WebXRFeature>,
    /// XR session mode preference
    pub preferred_mode:    WebXRSessionMode,
    /// Frame rate target
    pub target_frame_rate: f32,
}

impl Default for WebXRConfig {
    fn default() -> Self {
        Self {
            required_features: vec![WebXRFeature::LocalFloor, WebXRFeature::HitTest],
            optional_features: vec![
                WebXRFeature::Anchors,
                WebXRFeature::HandTracking,
                WebXRFeature::Layers,
            ],
            preferred_mode:    WebXRSessionMode::ImmersiveVr,
            target_frame_rate: 72.0,
        }
    }
}

/// WebXR features for browser-based AR/VR
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebXRFeature {
    /// Local floor reference space
    LocalFloor,
    /// Hit testing for AR interactions
    HitTest,
    /// Anchors for persistent positioning
    Anchors,
    /// Hand tracking for gesture input
    HandTracking,
    /// Layers for performance optimization
    Layers,
    /// Light estimation
    LightEstimation,
    /// Depth sensing
    Depth,
}

/// WebXR session modes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebXRSessionMode {
    /// Inline AR (within web page)
    InlineAR,
    /// Immersive VR session
    ImmersiveVr,
    /// Immersive AR session
    ImmersiveAR,
}

/// Collaboration configuration for multi-user sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    /// Maximum participants per session
    pub max_participants:           usize,
    /// Session timeout duration
    pub session_timeout:            Duration,
    /// Audio conferencing enabled
    pub audio_conferencing_enabled: bool,
    /// Screen sharing enabled
    pub screen_sharing_enabled:     bool,
    /// File sharing enabled
    pub file_sharing_enabled:       bool,
    /// Real-time synchronization enabled
    pub realtime_sync_enabled:      bool,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            max_participants:           8,
            session_timeout:            Duration::from_secs(7200), // 2 hours
            audio_conferencing_enabled: true,
            screen_sharing_enabled:     true,
            file_sharing_enabled:       true,
            realtime_sync_enabled:      true,
        }
    }
}

/// Quality scaling thresholds for adaptive performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScalingThresholds {
    /// CPU usage threshold for quality reduction (percentage)
    pub cpu_threshold:        f32,
    /// GPU usage threshold for quality reduction (percentage)
    pub gpu_threshold:        f32,
    /// Memory usage threshold for quality reduction (percentage)
    pub memory_threshold:     f32,
    /// Frame rate threshold for quality reduction (FPS)
    pub frame_rate_threshold: f32,
    /// Quality level step size for adjustments
    pub quality_step:         f32,
}

impl Default for QualityScalingThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold:        80.0,
            gpu_threshold:        85.0,
            memory_threshold:     90.0,
            frame_rate_threshold: 60.0,
            quality_step:         0.1,
        }
    }
}

/// Spatial position in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialPosition {
    /// X coordinate in spatial space
    pub x:        f32,
    /// Y coordinate in spatial space (up/down)
    pub y:        f32,
    /// Z coordinate in spatial space (forward/backward)
    pub z:        f32,
    /// Rotation around Y axis (yaw) in degrees
    pub rotation: Option<f32>,
    /// Scale factor for object sizing
    pub scale:    Option<f32>,
}

impl Default for SpatialPosition {
    fn default() -> Self {
        Self {
            x:        0.0,
            y:        0.0,
            z:        0.0,
            rotation: Some(0.0),
            scale:    Some(1.0),
        }
    }
}

/// Input types for spatial interactions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialInput {
    /// Gesture-based input
    Gesture(GestureType),
    /// Voice command
    Voice(String),
    /// Controller input
    ControllerInput(ControllerInputType),
    /// Eye gaze tracking
    EyeGaze(EyeGazeData),
}

/// Gesture types for spatial interaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureType {
    /// Point gesture
    Point,
    /// Grab gesture
    Grab,
    /// Pinch gesture
    Pinch,
    /// Swipe gesture
    Swipe,
    /// Tap gesture
    Tap,
    /// Two-handed interaction
    TwoHand,
}

/// Controller input types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControllerInputType {
    /// Button press
    Button(String),
    /// Joystick movement
    Joystick(f32, f32),
    /// Trigger press
    Trigger(f32),
}

/// Eye gaze data for tracking
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EyeGazeData {
    /// Horizontal gaze position (normalized)
    pub x:          f32,
    /// Vertical gaze position (normalized)
    pub y:          f32,
    /// Confident gaze detection
    pub confidence: f32,
}

/// Immersive events for multi-reality interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ImmeriveEvent {
    /// Spatial positioning changed
    PositionChanged {
        object_id:    String,
        new_position: SpatialPosition,
        timestamp:    SystemTime,
    },

    /// Gesture input received
    GesturePerformed {
        gesture_type: GestureType,
        position:     SpatialPosition,
        timestamp:    SystemTime,
    },

    /// Voice command processed
    VoiceCommand {
        command:    String,
        confidence: f32,
        timestamp:  SystemTime,
    },

    /// Reality mode switched
    RealityModeChanged {
        previous_mode: RealityMode,
        new_mode:      RealityMode,
        timestamp:     SystemTime,
    },

    /// Collaboration session started
    SessionStarted {
        session_id:   String,
        participants: Vec<String>,
        timestamp:    SystemTime,
    },

    /// Collaboration session ended
    SessionEnded {
        session_id: String,
        reason:     SessionTerminationReason,
        timestamp:  SystemTime,
    },

    /// AI suggestion generated
    AISuggestion {
        suggestion_type: AISuggestionType,
        content:         String,
        confidence:      f32,
        timestamp:       SystemTime,
    },

    /// Performance metric updated
    PerformanceMetric {
        metric_type: PerformanceMetricType,
        value:       f32,
        timestamp:   SystemTime,
    },
}

/// Reasons for session termination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionTerminationReason {
    /// Session completed successfully
    Completed,
    /// Session timed out
    Timeout,
    /// Participant disconnected
    Disconnected,
    /// Error occurred
    Error,
}

/// AI suggestion types for immersive development
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AISuggestionType {
    /// Code completion suggestion
    CodeCompletion,
    /// Refactoring suggestion
    Refactoring,
    /// Debugging suggestion
    Debugging,
    /// Performance optimization
    Performance,
    /// Architectural recommendation
    Architecture,
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceMetricType {
    /// CPU usage percentage
    CpuUsage,
    /// GPU usage percentage
    GpuUsage,
    /// Memory usage percentage
    MemoryUsage,
    /// Frame rate (FPS)
    FrameRate,
    /// Latency (milliseconds)
    Latency,
}

/// Device registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistration {
    /// Unique device identifier
    pub device_id:         String,
    /// Device type
    pub device_type:       DeviceType,
    /// Device capabilities
    pub capabilities:      DeviceCapabilities,
    /// Device status
    pub status:            DeviceStatus,
    /// Last seen timestamp
    pub last_seen:         SystemTime,
    /// Connection settings
    pub connection_config: DeviceConnectionConfig,
}

/// Types of devices supported
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    /// VR headset
    VrHeadset(VrHeadsetType),
    /// AR glasses/spectacles
    ArGlasses(ArGlassesType),
    /// Smartphone with AR capabilities
    Smartphone,
    /// Desktop computer
    Desktop,
    /// Web-based XR device
    WebXrDevice,
}

/// AR glasses types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArGlassesType {
    /// Hololens
    Hololens,
    /// Magic Leap
    MagicLeap,
    /// Meta AR glasses
    MetaAR,
    /// Generic AR glasses
    Generic,
}

/// Device status information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    /// Device is online and available
    Online,
    /// Device is online but busy
    Busy,
    /// Device is offline
    Offline,
    /// Device is in maintenance mode
    Maintenance,
    /// Device error state
    Error(String),
}

/// Device connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConnectionConfig {
    /// Connection protocol (e.g., "usb", "bluetooth", "wifi")
    pub protocol:     String,
    /// Connection endpoint/address
    pub endpoint:     String,
    /// Authentication configuration
    pub auth_config:  DeviceAuthConfig,
    /// Connection timeout duration
    pub timeout:      Duration,
    /// Retry configuration
    pub retry_config: ConnectionRetryConfig,
}

/// Device authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthConfig {
    /// Authentication method
    pub method:      AuthMethod,
    /// Authentication token or credentials
    pub credentials: Option<String>,
    /// Certificate data for TLS connections
    pub certificate: Option<String>,
}

/// Authentication methods supported
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    /// No authentication required
    None,
    /// API key authentication
    ApiKey,
    /// OAuth2 authentication
    OAuth2,
    /// Token-based authentication
    Token,
    /// Certificate-based authentication
    Certificate,
}

/// Connection retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRetryConfig {
    /// Maximum number of retry attempts
    pub max_retries:        usize,
    /// Initial delay between retries
    pub initial_delay:      Duration,
    /// Maximum delay between retries
    pub max_delay:          Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f32,
}

impl Default for ConnectionRetryConfig {
    fn default() -> Self {
        Self {
            max_retries:        5,
            initial_delay:      Duration::from_millis(100),
            max_delay:          Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Immersive session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmersiveSession {
    /// Unique session identifier
    pub session_id:     String,
    /// Current reality mode
    pub reality_mode:   RealityMode,
    /// Session participants
    pub participants:   Vec<String>,
    /// Session start time
    pub start_time:     SystemTime,
    /// Session configuration
    pub config:         MultiRealityConfig,
    /// Session state
    pub state:          SessionState,
    /// Shared objects in the session
    pub shared_objects: HashMap<String, SpatialEntity>,
}

/// Session state information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is initializing
    Initializing,
    /// Session is active and running
    Active,
    /// Session is paused
    Paused,
    /// Session is being suspended
    Suspending,
    /// Session has ended
    Ended,
    /// Session is in error state
    Error(String),
}

/// Spatial entity representation for immersive environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialEntity {
    /// Unique entity identifier
    pub id:           String,
    /// Entity type
    pub entity_type:  SpatialEntityType,
    /// Current spatial position
    pub position:     SpatialPosition,
    /// Entity properties
    pub properties:   HashMap<String, String>,
    /// Last updated timestamp
    pub last_updated: SystemTime,
    /// Entity visibility setting
    pub visible:      bool,
}

/// Types of spatial entities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialEntityType {
    /// Code file representation
    CodeFile,
    /// AI suggestion overlay
    AISuggestion,
    /// Virtual participant avatar
    Avatar,
    /// Interactive UI element
    UIElement,
    /// Tool/gadget in 3D space
    Tool,
    /// Reference material or documentation
    Reference,
}

/// Sanitizer trait for secure data handling
///
/// This trait provides methods for sanitizing user inputs and ensuring
/// data security across multi-reality interactions.
pub trait Sanitizer {
    /// Sanitize spatial input data
    fn sanitize_spatial_input(&self, input: &SpatialInput) -> Result<SpatialInput, IDEError>;

    /// Sanitize session configuration
    fn sanitize_session_config(&self, config: &MultiRealityConfig) -> Result<MultiRealityConfig, IDEError>;

    /// Validate and sanitize device registration
    fn sanitize_device_registration(&self, registration: &DeviceRegistration) -> Result<DeviceRegistration, IDEError>;

    /// Sanitize immersive event data
    fn sanitize_event(&self, event: &ImmeriveEvent) -> Result<ImmeriveEvent, IDEError>;
}

/// Utility functions for type conversions and validation
pub mod utils {
    use super::*;

    /// Convert system time to milliseconds since epoch
    pub fn system_time_to_millis(time: SystemTime) -> Result<u64, IDEError> {
        time.duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .map_err(|_| IDEError::InvalidTimestamp("Time before UNIX epoch".into()))
    }

    /// Convert milliseconds to system time
    pub fn millis_to_system_time(millis: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(millis)
    }

    /// Validate spatial position bounds
    pub fn validate_spatial_position(position: &SpatialPosition) -> Result<(), IDEError> {
        if !position.x.is_finite() || !position.y.is_finite() || !position.z.is_finite() {
            return Err(IDEError::InvalidSpatialPosition(
                "Non-finite coordinates".into(),
            ));
        }

        if position.x.abs() > 10000.0 || position.y.abs() > 10000.0 || position.z.abs() > 10000.0 {
            return Err(IDEError::InvalidSpatialPosition(
                "Coordinates out of bounds".into(),
            ));
        }

        Ok(())
    }

    /// Calculate distance between two spatial positions
    pub fn spatial_distance(pos1: &SpatialPosition, pos2: &SpatialPosition) -> f32 {
        let dx = pos1.x - pos2.x;
        let dy = pos1.y - pos2.y;
        let dz = pos1.z - pos2.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Interpolate between two spatial positions
    pub fn interpolate_spatial_position(pos1: &SpatialPosition, pos2: &SpatialPosition, t: f32) -> SpatialPosition {
        let t = t.clamp(0.0, 1.0);
        SpatialPosition {
            x:        pos1.x + (pos2.x - pos1.x) * t,
            y:        pos1.y + (pos2.y - pos1.y) * t,
            z:        pos1.z + (pos2.z - pos1.z) * t,
            rotation: pos1.rotation.map(|r1| {
                let r2 = pos2.rotation.unwrap_or(0.0);
                normalize_angle(r1 + (angle_difference(r1, r2)) * t)
            }),
            scale:    pos1.scale.map(|s1| {
                let s2 = pos2.scale.unwrap_or(1.0);
                s1 + (s2 - s1) * t
            }),
        }
    }

    /// Normalize angle to 0-360 range
    fn normalize_angle(angle: f32) -> f32 {
        ((angle % 360.0) + 360.0) % 360.0
    }

    /// Calculate smallest angle difference between two angles
    fn angle_difference(angle1: f32, angle2: f32) -> f32 {
        let diff = normalize_angle(angle2) - normalize_angle(angle1);
        if diff > 180.0 {
            diff - 360.0
        } else if diff < -180.0 {
            diff + 360.0
        } else {
            diff
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_position_default() {
        let position = SpatialPosition::default();
        assert_eq!(position.x, 0.0);
        assert_eq!(position.y, 0.0);
        assert_eq!(position.z, 0.0);
        assert_eq!(position.rotation, Some(0.0));
        assert_eq!(position.scale, Some(1.0));
    }

    #[test]
    fn test_multi_reality_config_default() {
        let config = MultiRealityConfig::default();
        assert_eq!(config.max_concurrent_sessions, 10);
        assert_eq!(config.performance_mode, PerformanceMode::Balanced);
        assert_eq!(config.spatial_audio_enabled, true);
    }

    #[test]
    fn test_spatial_distance_calculation() {
        let pos1 = SpatialPosition {
            x:        0.0,
            y:        0.0,
            z:        0.0,
            rotation: None,
            scale:    None,
        };
        let pos2 = SpatialPosition {
            x:        3.0,
            y:        4.0,
            z:        0.0,
            rotation: None,
            scale:    None,
        };
        let distance = utils::spatial_distance(&pos1, &pos2);
        assert!((distance - 5.0).abs() < 0.001); // Should be 5 (3-4-5 triangle)
    }

    #[test]
    fn test_reality_mode_serialization() {
        let mode = RealityMode::AR;
        let serialized = serde_json::to_string(&mode).unwrap();
        assert!(serialized.contains("ar"));
        let deserialized: RealityMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, RealityMode::AR);
    }
}
