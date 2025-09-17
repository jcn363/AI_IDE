# Model Warmup Prediction System - Integration Troubleshooting Guide

This guide addresses integration issues between the Model Warmup Prediction System and external components including Tauri, EventBus, LSP services, and multi-model orchestration.

## Tauri Integration Issues

### IPC Communication Failures

#### Issue: Command Registration Problems

**Symptoms:**
- Tauri commands not available in frontend
- "Command not found" errors
- IPC bridge failures

**Resolution:**
```rust
// Ensure proper command registration in main.rs
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Warmup predictor commands
            get_warmup_prediction,
            start_warmup_session,
            stop_warmup_session,
            get_warmup_metrics,
            // Ensure all commands are listed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### Issue: State Management Conflicts

**Symptoms:**
- State not persisting between commands
- Double-locking deadlocks
- Memory leaks in Tauri state

**Resolution:**
```rust
// Proper state management in Tauri
struct AppState {
    predictor: Arc<Mutex<ModelWarmupPredictor>>,
    metrics: Arc<Mutex<ModelWarmupMetrics>>,
}

#[tauri::command]
async fn get_warmup_prediction(
    request: WarmupRequest,
    state: State<'_, AppState>,
) -> Result<WarmupPrediction, String> {
    // Use double-locking pattern to avoid deadlocks
    let predictor = state.predictor.lock().await;
    let prediction = predictor.predict_and_warm(&request).await
        .map_err(|e| e.to_string())?;

    // Release lock before returning
    drop(predictor);

    Ok(prediction)
}
```

### Frontend Integration Issues

#### Issue: TypeScript Type Mismatches

**Symptoms:**
- Type errors in frontend code
- API responses not matching expected types
- Build failures in web component

**Resolution:**
```typescript
// Proper typing in frontend
interface WarmupPrediction {
    predicted_models: ModelInfo[];
    confidence_score: number;
    estimated_warmup_time_ms: number;
    resource_requirements: ResourceRequirements;
}

interface ModelInfo {
    model_id: string;
    model_type: ModelType;
    priority: RequestPriority;
}

// Type-safe API calls
async function getWarmupPrediction(request: WarmupRequest): Promise<WarmupPrediction> {
    return await invoke('get_warmup_prediction', { request });
}
```

## EventBus Integration Issues

### Message Routing Problems

#### Issue: Event Delivery Failures

**Symptoms:**
- Events not reaching subscribers
- Message queue overflows
- Component communication breakdowns

**Resolution:**
```rust
use rust_ai_ide_warmup_predictor::infra::EventBus;

// Proper event bus configuration
let event_bus = EventBus::new().await?;

// Register event handlers
event_bus.register_handler("warmup_request", |event: WarmupRequestEvent| async move {
    println!("Received warmup request: {:?}", event);
    // Process event
    process_warmup_request(event.data).await?;
    Ok(())
}).await?;

// Ensure proper event publishing
event_bus.publish(WarmupEvent {
    event_type: "prediction_complete".to_string(),
    data: prediction_data,
    timestamp: Instant::now(),
}).await?;
```

#### Issue: Event Handler Registration Conflicts

**Symptoms:**
- Multiple handlers for same event type
- Handler priority conflicts
- Race conditions in event processing

**Resolution:**
```rust
// Use event handler priorities
event_bus.register_handler_with_priority(
    "prediction_update",
    100, // Higher priority
    |event| async move {
        // High-priority handler
        handle_critical_prediction_update(event).await
    }
).await?;

event_bus.register_handler_with_priority(
    "prediction_update",
    50, // Lower priority
    |event| async move {
        // Background processing handler
        handle_background_prediction_update(event).await
    }
).await?;
```

### Event Serialization Issues

#### Issue: Message Format Incompatibilities

**Symptoms:**
- Deserialization errors
- Event data corruption
- Interoperability issues between components

**Resolution:**
```rust
use serde::{Serialize, Deserialize};

// Consistent event structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WarmupEvent {
    pub event_type: String,
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub data: serde_json::Value, // Flexible data field
    pub metadata: HashMap<String, String>,
}

// Version-aware serialization
impl WarmupEvent {
    pub fn new(event_type: &str, data: impl Serialize) -> Self {
        Self {
            event_type: event_type.to_string(),
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            source: env!("CARGO_PKG_NAME").to_string(),
            data: serde_json::to_value(data).unwrap_or_default(),
            metadata: HashMap::new(),
        }
    }
}
```

## LSP Service Integration Issues

### Service Connection Problems

#### Issue: LSP Service Initialization Failures

**Symptoms:**
- AI features not available
- Model loading errors
- LSP communication timeouts

**Resolution:**
```rust
use rust_ai_ide_warmup_predictor::lsp::LSPClient;

// Proper LSP client initialization
let lsp_client = LSPClient::builder()
    .with_endpoint("http://localhost:3000/lsp")
    .with_timeout(Duration::from_secs(30))
    .with_retry_policy(RetryPolicy::exponential_backoff())
    .build()
    .await?;

// Initialize AI models through LSP
lsp_client.initialize_models(vec![
    ModelConfig {
        model_id: "code-completion".to_string(),
        model_type: ModelType::Completion,
        memory_limit_mb: 1024,
    }
]).await?;
```

#### Issue: Model Orchestration Conflicts

**Symptoms:**
- Model switching failures
- Resource conflicts between models
- Inconsistent model state

**Resolution:**
```rust
use rust_ai_ide_multi_model_orchestrator::MultiModelOrchestrator;

// Proper model orchestration integration
let orchestrator = MultiModelOrchestrator::new().await?;
let warmup_predictor = ModelWarmupPredictor::new().await?;

// Register warmup predictor with orchestrator
orchestrator.register_warmup_predictor(Arc::new(warmup_predictor)).await?;

// Configure model switching policies
orchestrator.set_model_switching_policy(ModelSwitchingPolicy {
    max_concurrent_models: 3,
    switching_timeout_ms: 5000,
    resource_aware_switching: true,
    predictive_preloading: true,
}).await?;
```

### LSP Protocol Issues

#### Issue: Protocol Version Mismatches

**Symptoms:**
- Protocol parsing errors
- Feature incompatibility
- Communication breakdowns

**Resolution:**
```rust
// LSP protocol version negotiation
const SUPPORTED_LSP_VERSIONS: &[&str] = &["3.17", "3.16", "3.15"];

let client_capabilities = ClientCapabilities {
    text_document: Some(TextDocumentClientCapabilities {
        synchronization: Some(TextDocumentSyncClientCapabilities {
            dynamic_registration: Some(true),
            will_save: Some(true),
            will_save_wait_until: Some(true),
            did_save: Some(true),
        }),
        completion: Some(CompletionCapability {
            dynamic_registration: Some(true),
            completion_item: Some(CompletionItemCapability {
                snippet_support: Some(true),
                commit_characters_support: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }),
    ..Default::default()
};
```

## Multi-Model Orchestration Integration

### Model Coordination Issues

#### Issue: Model Loading Race Conditions

**Symptoms:**
- Models loading simultaneously
- Resource exhaustion
- Inconsistent model availability

**Resolution:**
```rust
use tokio::sync::Semaphore;
use std::collections::HashMap;

// Model loading coordination
struct ModelCoordinator {
    semaphore: Arc<Semaphore>,
    loading_status: Arc<Mutex<HashMap<String, ModelLoadStatus>>>,
}

impl ModelCoordinator {
    pub fn new(max_concurrent_loads: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent_loads)),
            loading_status: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn load_model(&self, model_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _permit = self.semaphore.acquire().await?;

        // Mark as loading
        {
            let mut status = self.loading_status.lock().await;
            status.insert(model_id.to_string(), ModelLoadStatus::Loading);
        }

        // Load model
        match load_model_from_disk(model_id).await {
            Ok(model) => {
                let mut status = self.loading_status.lock().await;
                status.insert(model_id.to_string(), ModelLoadStatus::Loaded(model));
                Ok(())
            }
            Err(e) => {
                let mut status = self.loading_status.lock().await;
                status.insert(model_id.to_string(), ModelLoadStatus::Failed(e.to_string()));
                Err(e)
            }
        }
    }
}
```

#### Issue: Resource Allocation Conflicts

**Symptoms:**
- Models competing for GPU memory
- CPU resource contention
- Performance degradation

**Resolution:**
```rust
// Resource-aware model scheduling
struct ResourceAwareScheduler {
    gpu_memory_tracker: GPUMemoryTracker,
    cpu_resource_tracker: CPUResourceTracker,
    model_priorities: HashMap<String, Priority>,
}

impl ResourceAwareScheduler {
    pub async fn schedule_model_load(&self, model_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check resource availability
        let gpu_available = self.gpu_memory_tracker.available_memory_mb().await?;
        let cpu_available = self.cpu_resource_tracker.available_cores().await?;

        // Estimate resource requirements
        let requirements = self.estimate_model_requirements(model_id).await?;

        // Check if resources are sufficient
        if gpu_available < requirements.gpu_memory_mb ||
           cpu_available < requirements.cpu_cores as f32 {
            return Err("Insufficient resources for model loading".into());
        }

        // Allocate resources and load model
        self.allocate_resources(requirements).await?;
        load_and_initialize_model(model_id).await?;
        Ok(())
    }
}
```

### Orchestration Communication Issues

#### Issue: Inter-Model Communication Failures

**Symptoms:**
- Model coordination messages lost
- State synchronization issues
- Inconsistent model behavior

**Resolution:**
```rust
use tokio::sync::broadcast;

// Model communication channel
struct ModelCommunicationBus {
    sender: broadcast::Sender<ModelMessage>,
}

impl ModelCommunicationBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub async fn broadcast_model_state(&self, model_id: &str, state: ModelState) {
        let message = ModelMessage {
            sender: model_id.to_string(),
            message_type: MessageType::StateUpdate,
            payload: serde_json::to_value(state).unwrap(),
            timestamp: Utc::now(),
        };

        let _ = self.sender.send(message);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ModelMessage> {
        self.sender.subscribe()
    }
}
```

## Cross-Component Integration Issues

### State Synchronization Problems

#### Issue: Inconsistent Component State

**Symptoms:**
- Components showing different data
- Race conditions in state updates
- Data inconsistency across services

**Resolution:**
```rust
use tokio::sync::RwLock;

// Centralized state management
struct GlobalStateManager {
    component_states: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    state_watchers: Arc<Mutex<Vec<Box<dyn StateWatcher>>>>,
}

impl GlobalStateManager {
    pub async fn update_component_state(&self, component_id: &str, state: serde_json::Value) {
        // Update state
        {
            let mut states = self.component_states.write().await;
            states.insert(component_id.to_string(), state.clone());
        }

        // Notify watchers
        let watchers = self.state_watchers.lock().await;
        for watcher in watchers.iter() {
            watcher.on_state_change(component_id, &state).await;
        }
    }

    pub async fn get_component_state(&self, component_id: &str) -> Option<serde_json::Value> {
        let states = self.component_states.read().await;
        states.get(component_id).cloned()
    }
}
```

### Service Discovery Issues

#### Issue: Service Location Failures

**Symptoms:**
- Services unable to find each other
- Dynamic service registration issues
- Load balancing failures

**Resolution:**
```rust
use rust_ai_ide_warmup_predictor::discovery::ServiceRegistry;

// Service discovery implementation
struct ServiceRegistryImpl {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    health_checker: Arc<HealthChecker>,
}

impl ServiceRegistryImpl {
    pub async fn register_service(&self, service: ServiceInfo) -> Result<(), Box<dyn std::error::Error>> {
        // Validate service health
        self.health_checker.check_service(&service).await?;

        // Register service
        let mut services = self.services.write().await;
        services.insert(service.id.clone(), service);

        // Start health monitoring
        self.start_health_monitoring(&service.id).await?;

        Ok(())
    }

    pub async fn discover_service(&self, service_type: &str) -> Result<Vec<ServiceInfo>, Box<dyn std::error::Error>> {
        let services = self.services.read().await;
        let matching_services: Vec<ServiceInfo> = services.values()
            .filter(|service| service.service_type == service_type)
            .filter(|service| service.is_healthy)
            .cloned()
            .collect();

        Ok(matching_services)
    }
}
```

## Integration Testing Strategies

### Automated Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_system_integration() {
        // Start all components
        let tauri_app = start_tauri_app().await;
        let event_bus = start_event_bus().await;
        let lsp_service = start_lsp_service().await;
        let orchestrator = start_orchestrator().await;

        // Test complete workflow
        let request = create_test_warmup_request();
        let prediction = tauri_app.get_warmup_prediction(request).await.unwrap();

        // Verify all components participated
        assert!(prediction.predicted_models.len() > 0);
        assert!(prediction.confidence_score > 0.5);

        // Test event propagation
        let events = event_bus.get_recent_events("prediction_complete").await;
        assert!(events.len() > 0);

        // Cleanup
        shutdown_all_services().await;
    }
}
```

### Integration Monitoring

```rust
use rust_ai_ide_warmup_predictor::monitoring::IntegrationMonitor;

let monitor = IntegrationMonitor::new();

// Monitor component interactions
monitor.watch_component_interactions().await?;

// Track integration health
let health = monitor.get_integration_health().await?;
println!("Integration Health: {:.1}%", health.overall_health);

// Alert on integration issues
monitor.set_integration_alerts(|issue| async move {
    match issue.severity {
        Severity::Critical => send_critical_alert(issue).await?,
        Severity::Warning => log_warning(issue).await?,
        _ => {}
    }
    Ok(())
}).await?;
```

This integration troubleshooting guide provides comprehensive procedures for diagnosing and resolving integration issues between the Model Warmup Prediction System and external components. Proper integration is crucial for system reliability and performance.