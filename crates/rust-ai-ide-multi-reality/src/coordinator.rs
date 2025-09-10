//! Multi-Reality Coordinator - Central coordination hub for AR/VR functionality
//!
//! This module provides the `MultiRealityCoordinator` which serves as the central hub
//! for coordinating all AR/VR devices, immersive UI, AI assistance, and collaborative
//! development sessions. It manages the lifecycle, state synchronization, and
//! cross-reality coordination of the entire multi-reality system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{Mutex, RwLock};
use futures::future::join_all;
use rust_ai_ide_errors::IDEError;

use crate::types::*;
use crate::ar_engine::{ArEngine, ArEngineState};
use crate::vr_engine::{VrEngine, VrEngineState};
use crate::collaboration_manager::{CollaborationManager, CollaborationState};
use crate::device_orchestrator::{DeviceOrchestrator, DeviceOrchestratorState};
use crate::immersive_ui_controller::{ImmersiveUIController, UIState};
use crate::ai_integration_bridge::{AiIntegrationBridge, AIState};

/// Multi-Reality Coordinator - Core coordination system
///
/// This struct serves as the central hub for all multi-reality functionality.
/// It coordinates AR/VR engines, collaboration, device management, UI controllers,
/// and AI integration to provide a cohesive immersive development experience.
#[derive(Debug)]
pub struct MultiRealityCoordinator {
    /// Current configuration
    config: MultiRealityConfig,
    /// Current reality mode
    current_mode: RealityMode,
    /// Coordinator status
    coordinator_state: CoordinatorState,
    /// Spatial objects registry with locations
    spatial_objects: Arc<RwLock<HashMap<String, SpatialEntity>>>,
    /// Active immersive sessions
    active_sessions: Arc<RwLock<HashMap<String, ImmersiveSession>>>,
    /// AR engine instance
    ar_engine: Arc<Mutex<Option<ArEngine>>>,
    /// VR engine instance
    vr_engine: Arc<Mutex<Option<VrEngine>>>,
    /// Collaboration manager
    collaboration_manager: Arc<Mutex<Option<CollaborationManager>>>,
    /// Device orchestrator
    device_orchestrator: Arc<Mutex<Option<DeviceOrchestrator>>>,
    /// Immersive UI controller
    ui_controller: Arc<Mutex<Option<ImmersiveUIController>>>,
    /// AI integration bridge
    ai_bridge: Arc<Mutex<Option<AiIntegrationBridge>>>,
    /// Security sanitizer for inputs
    sanitizer: Arc<RwLock<Box<dyn Sanitizer + Send + Sync>>>,
    /// Performance monitor for optimization
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
    /// Event channel sender for system-wide events
    event_sender: tokio::sync::broadcast::Sender<ImmeriveEvent>,
}

/// Performance monitoring data
#[derive(Debug, Clone)]
struct PerformanceMonitor {
    /// CPU usage history
    cpu_history: Vec<f32>,
    /// GPU usage history
    gpu_history: Vec<f32>,
    /// Memory usage history
    memory_history: Vec<f32>,
    /// Frame rate history
    fps_history: Vec<f32>,
    /// Quality scaling factor (0.0 to 1.0)
    current_quality_factor: f32,
    /// Last performance check timestamp
    last_check: SystemTime,
}

/// State of the coordinator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordinatorState {
    /// Coordinator is uninitialized
    Uninitialized,
    /// Coordinator is being initialized
    Initializing,
    /// Coordinator is ready for use
    Ready,
    /// Coordinator is running with active sessions
    Active,
    /// Coordinator is in error state
    Error(String),
    /// Coordinator is being shut down
    ShuttingDown,
}

impl MultiRealityCoordinator {
    /// Create a new MultiRealityCoordinator with the given configuration
    ///
    /// This method initializes the coordinator but does not start any services.
    /// Use `initialize()` to fully set up the system.
    pub fn new(config: MultiRealityConfig) -> Self {
        let (event_sender, _) = tokio::sync::broadcast::channel(100);

        Self {
            config,
            current_mode: RealityMode::Desktop,
            coordinator_state: CoordinatorState::Uninitialized,
            spatial_objects: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            ar_engine: Arc::new(Mutex::new(None)),
            vr_engine: Arc::new(Mutex::new(None)),
            collaboration_manager: Arc::new(Mutex::new(None)),
            device_orchestrator: Arc::new(Mutex::new(None)),
            ui_controller: Arc::new(Mutex::new(None)),
            ai_bridge: Arc::new(Mutex::new(None)),
            sanitizer: Arc::new(RwLock::new(Box::new(DefaultSanitizer))),
            performance_monitor: Arc::new(RwLock::new(PerformanceMonitor {
                cpu_history: Vec::new(),
                gpu_history: Vec::new(),
                memory_history: Vec::new(),
                fps_history: Vec::new(),
                current_quality_factor: 1.0,
                last_check: SystemTime::now(),
            })),
            event_sender,
        }
    }

    /// Initialize the multi-reality coordinator and all its components
    ///
    /// This method performs the full initialization of the coordinator, setting up
    /// all engines, managers, and controllers based on the configuration.
    ///
    /// # Returns
    /// * `Ok(())` if initialization succeeds
    /// * `Err(Box<dyn std::error::Error + Send + Sync>)` if initialization fails
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.coordinator_state = CoordinatorState::Initializing;

        // Initialize AR engine if AR is enabled or mixed reality
        if should_initialize_ar(&self.config) {
            let ar_engine = self.initialize_ar_engine().await?;
            *self.ar_engine.lock().await = Some(ar_engine);
        }

        // Initialize VR engine if VR is enabled or mixed reality
        if should_initialize_vr(&self.config) {
            let vr_engine = self.initialize_vr_engine().await?;
            *self.vr_engine.lock().await = Some(vr_engine);
        }

        // Initialize collaboration manager
        let collaboration_manager = self.initialize_collaboration_manager().await?;
        *self.collaboration_manager.lock().await = Some(collaboration_manager);

        // Initialize device orchestrator
        let device_orchestrator = self.initialize_device_orchestrator().await?;
        *self.device_orchestrator.lock().await = Some(device_orchestrator);

        // Initialize UI controller
        let ui_controller = self.initialize_ui_controller().await?;
        *self.ui_controller.lock().await = Some(ui_controller);

        // Initialize AI integration bridge
        let ai_bridge = self.initialize_ai_bridge().await?;
        *self.ai_bridge.lock().await = Some(ai_bridge);

        // Set coordinator state to ready
        self.coordinator_state = CoordinatorState::Ready;

        Ok(())
    }

    /// Check if the coordinator is ready for use
    ///
    /// # Returns
    /// * `true` if the coordinator is ready and initialized
    /// * `false` if the coordinator is still initializing or in error state
    pub async fn is_ready(&self) -> bool {
        matches!(self.coordinator_state, CoordinatorState::Ready | CoordinatorState::Active)
    }

    /// Get the current reality mode
    ///
    /// # Returns
    /// * `RealityMode` - The current reality mode the system is operating in
    pub async fn get_current_reality(&self) -> RealityMode {
        self.current_mode
    }

    /// Switch to Augmented Reality mode
    ///
    /// This method switches the system to AR mode, initializing the AR engine
    /// if necessary and updating all components for AR operation.
    ///
    /// # Returns
    /// * `Ok(())` if the switch succeeds
    /// * `Err(Box<dyn std::error::Error + Send + Sync>)` if the switch fails
    pub async fn switch_to_ar_mode(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.switch_reality_mode(RealityMode::AR).await
    }

    /// Switch to Virtual Reality mode
    ///
    /// This method switches the system to VR mode, initializing the VR engine
    /// if necessary and updating all components for VR operation.
    ///
    /// # Returns
    /// * `Ok(())` if the switch succeeds
    /// * `Err(Box<dyn std::error::Error + Send + Sync>)` if the switch fails
    pub async fn switch_to_vr_mode(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.switch_reality_mode(RealityMode::VR).await
    }

    /// Switch to desktop mode
    ///
    /// This method switches the system back to traditional desktop mode,
    /// suspending AR/VR engines and returning to 2D interface operation.
    ///
    /// # Returns
    /// * `Ok(())` if the switch succeeds
    /// * `Err(Box<dyn std::error::Error + Send + Sync>)` if the switch fails
    pub async fn switch_to_desktop_mode(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.switch_reality_mode(RealityMode::Desktop).await
    }

    /// Internal method to switch reality modes
    async fn switch_reality_mode(&mut self, new_mode: RealityMode) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let previous_mode = self.current_mode;

        match (previous_mode, new_mode) {
            // Switching to AR mode
            (_, RealityMode::AR) if self.config.device_caps.supported_camera_types.contains(&CameraType::RGB) => {
                if let Some(ar_engine) = &*self.ar_engine.lock().await {
                    ar_engine.start_ar_session().await?;
                } else {
                    return Err("AR engine not initialized".into());
                }
            }

            // Switching to VR mode
            (_, RealityMode::VR) if !self.config.device_caps.supported_vr_headsets.is_empty() => {
                if let Some(vr_engine) = &*self.vr_engine.lock().await {
                    vr_engine.start_vr_session().await?;
                } else {
                    return Err("VR engine not initialized".into());
                }
            }

            // Switching to desktop mode (stop active sessions)
            (RealityMode::AR, RealityMode::Desktop) => {
                if let Some(ar_engine) = &*self.ar_engine.lock().await {
                    ar_engine.stop_ar_session().await?;
                }
            }

            (RealityMode::VR, RealityMode::Desktop) => {
                if let Some(vr_engine) = &*self.vr_engine.lock().await {
                    vr_engine.stop_vr_session().await?;
                }
            }

            // Same mode transition (no-op)
            _ if previous_mode == new_mode => return Ok(()),

            _ => return Err(format!("Unsupported mode transition: {:?} -> {:?}", previous_mode, new_mode).into()),
        }

        // Update current mode
        self.current_mode = new_mode;

        // Emit event for mode change
        let event = ImmeriveEvent::RealityModeChanged {
            previous_mode,
            new_mode,
            timestamp: SystemTime::now(),
        };

        let _ = self.event_sender.send(event);

        Ok(())
    }

    /// Update the spatial position of an object
    ///
    /// This method updates the spatial position of a registered object and
    /// synchronizes the change across all active sessions and devices.
    ///
    /// # Arguments
    /// * `object_id` - Unique identifier of the object
    /// * `position` - New spatial position for the object
    ///
    /// # Returns
    /// * `Ok(())` if the update succeeds
    /// * `Err(Box<dyn std::error::Error + Send + Sync>)` if the update fails
    pub async fn update_spatial_position(
        &self,
        object_id: &str,
        position: SpatialPosition,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Validate position
        types::utils::validate_spatial_position(&position)?;

        // Get or create spatial entity
        let mut spatial_objects = self.spatial_objects.write().await;
        let entity = spatial_objects.entry(object_id.to_string()).or_insert_with(|| SpatialEntity {
            id: object_id.to_string(),
            entity_type: SpatialEntityType::Generic,
            position: position.clone(),
            properties: HashMap::new(),
            last_updated: SystemTime::now(),
            visible: true,
        });

        entity.position = position.clone();
        entity.last_updated = SystemTime::now();

        drop(spatial_objects);

        // Emit position change event
        let event = ImmeriveEvent::PositionChanged {
            object_id: object_id.to_string(),
            new_position: position,
            timestamp: SystemTime::now(),
        };

        let _ = self.event_sender.send(event);

        Ok(())
    }

    /// Get the spatial position of an object
    ///
    /// This method retrieves the current spatial position of a registered object.
    ///
    /// # Arguments
    /// * `object_id` - Unique identifier of the object
    ///
    /// # Returns
    /// * `SpatialPosition` - Current position of the object
    pub async fn get_spatial_position(&self, object_id: &str) -> SpatialPosition {
        let spatial_objects = self.spatial_objects.read().await;
        spatial_objects
            .get(object_id)
            .map(|entity| entity.position.clone())
            .unwrap_or_default()
    }

    /// Subscribe to immersive events
    ///
    /// This method returns a receiver for immersive events emitted by the coordinator
    /// and its components, allowing components to listen for system-wide events.
    ///
    /// # Returns
    /// * `tokio::sync::broadcast::Receiver<ImmeriveEvent>` - Event receiver
    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<ImmeriveEvent> {
        self.event_sender.subscribe()
    }

    /// Process a spatial input (gesture, voice, or controller input)
    ///
    /// This method processes spatial input and dispatches it to the appropriate
    /// handlers based on the current reality mode and input type.
    ///
    /// # Arguments
    /// * `input` - Spatial input to process
    ///
    /// # Returns
    /// * `Ok(())` if the input is processed successfully
    /// * `Err(Box<dyn std::error::Error + Send + Sync>)` if processing fails
    pub async fn process_spatial_input(
        &self,
        input: SpatialInput,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Sanitize input for security
        let sanitizer = self.sanitizer.read().await;
        let sanitized_input = sanitizer.sanitize_spatial_input(&input)?;

        // Process based on input type and current reality mode
        match (self.current_mode, &sanitized_input) {
            (RealityMode::AR, SpatialInput::Gesture(gesture_type)) => {
                self.process_ar_gesture(*gesture_type, input.extract_position()).await
            }
            (RealityMode::VR, SpatialInput::Gesture(gesture_type)) => {
                self.process_vr_gesture(*gesture_type, input.extract_position()).await
            }
            (RealityMode::AR, SpatialInput::Voice(command)) => {
                self.process_ar_voice_command(command).await
            }
            (RealityMode::VR, SpatialInput::Voice(command)) => {
                self.process_vr_voice_command(command).await
            }
            (_, SpatialInput::ControllerInput(controller_input)) => {
                self.process_controller_input(controller_input.clone()).await
            }
            (_, SpatialInput::EyeGaze(eye_gaze)) => {
                self.process_eye_gaze(*eye_gaze).await
            }
        }
    }

    /// Get coordinator status and metrics
    ///
    /// This method returns comprehensive status information about the coordinator
    /// and its components, including performance metrics and active sessions.
    ///
    /// # Returns
    /// * `CoordinatorStatus` - Current status of the coordinator and all components
    pub async fn get_status(&self) -> CoordinatorStatus {
        CoordinatorStatus {
            state: self.coordinator_state.clone(),
            current_mode: self.current_mode,
            active_sessions_count: self.active_session_count().await,
            spatial_objects_count: self.spatial_object_count().await,
            ar_engine_status: self.get_ar_engine_status().await,
            vr_engine_status: self.get_vr_engine_status().await,
            performance_metrics: self.get_performance_metrics().await,
        }
    }

    // Private implementation methods

    /// Initialize AR engine
    async fn initialize_ar_engine(&self) -> Result<ArEngine, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation - in real system this would initialize with proper config
        Ok(ArEngine::new().await)
    }

    /// Initialize VR engine
    async fn initialize_vr_engine(&self) -> Result<VrEngine, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation - in real system this would initialize with proper config
        Ok(VrEngine::new().await)
    }

    /// Initialize collaboration manager
    async fn initialize_collaboration_manager(&self) -> Result<CollaborationManager, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(CollaborationManager::new().await)
    }

    /// Initialize device orchestrator
    async fn initialize_device_orchestrator(&self) -> Result<DeviceOrchestrator, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(DeviceOrchestrator::new().await)
    }

    /// Initialize UI controller
    async fn initialize_ui_controller(&self) -> Result<ImmersiveUIController, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(ImmersiveUIController::new().await)
    }

    /// Initialize AI bridge
    async fn initialize_ai_bridge(&self) -> Result<AiIntegrationBridge, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(AiIntegrationBridge::new().await)
    }

    /// Process AR gesture
    async fn process_ar_gesture(
        &self,
        gesture: GestureType,
        position: Option<SpatialPosition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(())
    }

    /// Process VR gesture
    async fn process_vr_gesture(
        &self,
        gesture: GestureType,
        position: Option<SpatialPosition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(())
    }

    /// Process AR voice command
    async fn process_ar_voice_command(
        &self,
        command: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(())
    }

    /// Process VR voice command
    async fn process_vr_voice_command(
        &self,
        command: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(())
    }

    /// Process controller input
    async fn process_controller_input(
        &self,
        input: ControllerInputType,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(())
    }

    /// Process eye gaze input
    async fn process_eye_gaze(
        &self,
        gaze: EyeGazeData,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(())
    }

    /// Get active session count
    async fn active_session_count(&self) -> usize {
        self.active_sessions.read().await.len()
    }

    /// Get spatial object count
    async fn spatial_object_count(&self) -> usize {
        self.spatial_objects.read().await.len()
    }

    /// Get AR engine status
    async fn get_ar_engine_status(&self) -> Option<ArEngineState> {
        if let Some(ar_engine) = &*self.ar_engine.lock().await {
            Some(ar_engine.get_state().await)
        } else {
            None
        }
    }

    /// Get VR engine status
    async fn get_vr_engine_status(&self) -> Option<VrEngineState> {
        if let Some(vr_engine) = &*self.vr_engine.lock().await {
            Some(vr_engine.get_state().await)
        } else {
            None
        }
    }

    /// Get performance metrics
    async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let monitor = self.performance_monitor.read().await;

        PerformanceMetrics {
            cpu_usage: monitor.cpu_history.last().copied().unwrap_or(0.0),
            gpu_usage: monitor.gpu_history.last().copied().unwrap_or(0.0),
            memory_usage: monitor.memory_history.last().copied().unwrap_or(0.0),
            frame_rate: monitor.fps_history.last().copied().unwrap_or(0.0),
            quality_factor: monitor.current_quality_factor,
        }
    }
}

/// Status summary of the multi-reality coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorStatus {
    /// Current coordinator state
    pub state: CoordinatorState,
    /// Current reality mode
    pub current_mode: RealityMode,
    /// Number of active sessions
    pub active_sessions_count: usize,
    /// Number of spatial objects
    pub spatial_objects_count: usize,
    /// AR engine status
    pub ar_engine_status: Option<ArEngineState>,
    /// VR engine status
    pub vr_engine_status: Option<VrEngineState>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics summary
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// GPU usage percentage (0-100)
    pub gpu_usage: f32,
    /// Memory usage percentage (0-100)
    pub memory_usage: f32,
    /// Frame rate (FPS)
    pub frame_rate: f32,
    /// Quality scaling factor (0.0-1.0)
    pub quality_factor: f32,
}

/// Initialize the MultiRealityCoordinator with given configuration
///
/// This is the main entry point for creating and initializing the coordinator.
/// It handles all the setup and coordination between different components.
///
/// # Arguments
/// * `config` - Configuration for the multi-reality system
///
/// # Returns
/// * `Result<MultiRealityCoordinator, Box<dyn std::error::Error + Send + Sync>>` -
///   Initialized coordinator or initialization error
pub async fn init_coordinator(
    config: MultiRealityConfig,
) -> Result<MultiRealityCoordinator, Box<dyn std::error::Error + Send + Sync>> {
    let mut coordinator = MultiRealityCoordinator::new(config);
    coordinator.initialize().await?;
    Ok(coordinator)
}

/// Helper: determine if AR should be initialized
fn should_initialize_ar(config: &MultiRealityConfig) -> bool {
    !config.device_caps.supported_camera_types.is_empty()
        || config.webxr_config.is_some()
        || matches!(config.current_reality_mode, RealityMode::AR | RealityMode::MixedReality)
}

/// Helper: determine if VR should be initialized
fn should_initialize_vr(config: &MultiRealityConfig) -> bool {
    !config.device_caps.supported_vr_headsets.is_empty()
        || config.webxr_config.is_some()
        || matches!(config.current_reality_mode, RealityMode::VR | RealityMode::MixedReality)
}

/// Default sanitizer implementation for multi-reality systems
#[derive(Debug, Clone)]
struct DefaultSanitizer;

impl Sanitizer for DefaultSanitizer {
    fn sanitize_spatial_input(&self, input: &SpatialInput) -> Result<SpatialInput, IDEError> {
        match input {
            SpatialInput::Gesture(gesture) => {
                Ok(SpatialInput::Gesture(*gesture)) // Gestures are typically safe
            }
            SpatialInput::Voice(command) if command.len() > 1000 => {
                Err(IDEError::InvalidSpatialInput("Voice command too long".into()))
            }
            SpatialInput::Voice(command) => {
                // Basic sanitization - remove potential injection characters
                let sanitized = command
                    .chars()
                    .filter(|c| c.is_ascii() && (*c).is_alphanumeric() || *c == ' ')
                    .collect::<String>();

                if sanitized.is_empty() {
                    Err(IDEError::InvalidSpatialInput("Voice command contains invalid characters".into()))
                } else {
                    Ok(SpatialInput::Voice(sanitized))
                }
            }
            SpatialInput::ControllerInput(_) => {
                Ok(input.clone()) // Controller inputs are typically safe
            }
            SpatialInput::EyeGaze(_) => {
                Ok(input.clone()) // Eye gaze data is typically safe
            }
        }
    }

    fn sanitize_session_config(&self, config: &MultiRealityConfig) -> Result<MultiRealityConfig, IDEError> {
        // Basic validation - ensure valid ranges
        if config.max_concurrent_sessions > 100 {
            return Err(IDEError::InvalidConfiguration("Too many concurrent sessions".into()));
        }

        Ok(config.clone())
    }

    fn sanitize_device_registration(&self, registration: &DeviceRegistration) -> Result<DeviceRegistration, IDEError> {
        // Basic validation - ensure device ID format
        if registration.device_id.is_empty() || registration.device_id.len() > 100 {
            return Err(IDEError::InvalidDeviceRegistration("Invalid device ID".into()));
        }

        Ok(registration.clone())
    }

    fn sanitize_event(&self, event: &ImmeriveEvent) -> Result<ImmeriveEvent, IDEError> {
        Ok(event.clone()) // Events are typically safe as they're internal
    }
}

// Extension traits for SpatialInput
impl SpatialInput {
    /// Extract position information from spatial input if available
    fn extract_position(&self) -> Option<SpatialPosition> {
        match self {
            SpatialInput::Gesture(_) => None, // Position comes separately
            SpatialInput::Voice(_) => None,
            SpatialInput::ControllerInput(_) => None,
            SpatialInput::EyeGaze(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let config = MultiRealityConfig::default();
        let coordinator = MultiRealityCoordinator::new(config);

        assert_eq!(coordinator.current_mode, RealityMode::Desktop);
        assert_eq!(coordinator.coordinator_state, CoordinatorState::Uninitialized);
    }

    #[tokio::test]
    async fn test_coordinator_initialization() {
        let config = MultiRealityConfig::default();
        let mut coordinator = MultiRealityCoordinator::new(config);

        // Should not be ready before initialization
        assert!(!coordinator.is_ready().await);

        // Initialization should succeed with placeholder implementations
        let result = coordinator.initialize().await;
        assert!(result.is_ok());

        // Should be ready after initialization
        assert!(coordinator.is_ready().await);
    }

    #[tokio::test]
    async fn test_reality_mode_switching() {
        let config = MultiRealityConfig::default();
        let mut coordinator = MultiRealityCoordinator::new(config.clone());
        coordinator.initialize().await.unwrap();

        // Test AR mode switch
        let result = coordinator.switch_to_ar_mode().await;
        if config.device_caps.supported_camera_types.contains(&CameraType::RGB) {
            assert!(result.is_ok());
            assert_eq!(coordinator.get_current_reality().await, RealityMode::AR);
        }

        // Switch back to desktop
        let result = coordinator.switch_to_desktop_mode().await;
        assert!(result.is_ok());
        assert_eq!(coordinator.get_current_reality().await, RealityMode::Desktop);
    }

    #[tokio::test]
    async fn test_spatial_position_update() {
        let config = MultiRealityConfig::default();
        let coordinator = MultiRealityCoordinator::new(config);
        let coordinator = std::sync::Arc::new(tokio::sync::Mutex::new(coordinator));

        // Update position
        let position = SpatialPosition { x: 1.0, y: 2.0, z: 3.0, rotation: None, scale: None };
        let result = coordinator.lock().await.update_spatial_position("test_object", position.clone()).await;
        assert!(result.is_ok());

        // Verify position was updated
        let retrieved = coordinator.lock().await.get_spatial_position("test_object").await;
        assert_eq!(retrieved.x, position.x);
        assert_eq!(retrieved.y, position.y);
        assert_eq!(retrieved.z, position.z);
    }

    #[test]
    fn test_default_sanitizer() {
        let sanitizer = DefaultSanitizer;

        // Test valid voice input
        let valid_input = SpatialInput::Voice("hello world".to_string());
        let result = sanitizer.sanitize_spatial_input(&valid_input);
        assert!(result.is_ok());

        // Test invalid voice input (too long)
        let long_input = SpatialInput::Voice("a".repeat(1001));
        let result = sanitizer.sanitize_spatial_input(&long_input);
        assert!(result.is_err());

        // Test voice input with special characters
        let special_input = SpatialInput::Voice("hello<script>alert('xss')</script>world".to_string());
        let result = sanitizer.sanitize_spatial_input(&special_input);
        if let Ok(SpatialInput::Voice(sanitized)) = result {
            assert!(!sanitized.contains('<') && !sanitized.contains('>'));
        } else {
            assert!(result.is_err());
        }
    }
}