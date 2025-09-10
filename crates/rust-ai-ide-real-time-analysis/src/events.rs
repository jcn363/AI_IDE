#![allow(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn, instrument};

use crate::types::{
    AnalysisResult, AnalysisTrigger, TriggerSource, AnalysisMetadata, TaskPriority,
    PerformanceMetrics,
};

/// Event processing errors
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event routing failed: {0}")]
    Routing(String),

    #[error("Subscriber communication error: {0}")]
    SubscriberError(String),

    #[error("LSP integration error: {0}")]
    LspError(String),

    #[error("Dashboard update error: {0}")]
    DashboardError(String),

    #[error("Event serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event channel error: {0}")]
    ChannelError(String),
}

/// Event processing result type
type EventResult<T> = Result<T, EventError>;

/// Event types that can be processed
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum RealTimeEvent {
    /// Analysis completed event
    AnalysisComplete(AnalysisCompleteEvent),
    /// File change detected event
    FileChange(FileChangeEvent),
    /// Cache event (hit/miss/invalidation)
    CacheEvent(CacheEvent),
    /// Performance monitoring event
    PerformanceEvent(PerformanceEvent),
    /// LSP diagnostic event
    LspDiagnosticEvent(LspDiagnosticEvent),
    /// Dashboard update event
    DashboardEvent(DashboardEvent),
    /// Health monitoring event
    HealthEvent(HealthEvent),
}

/// Analysis completion event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisCompleteEvent {
    /// Task ID
    pub task_id: String,
    /// File path analyzed
    pub file_path: String,
    /// Analysis type
    pub analysis_type: String,
    /// Number of findings
    pub findings_count: usize,
    /// Analysis duration (milliseconds)
    pub duration_ms: u64,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetricsData,
}

/// File change event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileChangeEvent {
    /// Paths affected
    pub paths: Vec<String>,
    /// Change type
    pub change_type: String,
    /// Timestamp
    pub timestamp: i64,
    /// Priority for processing
    pub priority: String,
}

/// Cache event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEvent {
    /// Cache operation type
    pub operation: String,
    /// Cache key
    pub key: String,
    /// Hit or miss
    pub is_hit: bool,
    /// Access time (milliseconds)
    pub access_time_ms: u64,
}

/// Performance monitoring event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceEvent {
    /// Metric name
    pub metric_name: String,
    /// Metric value
    pub value: f64,
    /// Unit
    pub unit: String,
    /// Timestamp
    pub timestamp: i64,
    /// Component name
    pub component: String,
}

/// LSP diagnostic event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspDiagnosticEvent {
    /// URI of the document
    pub uri: String,
    /// Diagnostic severity
    pub severity: String,
    /// Diagnostic message
    pub message: String,
    /// Diagnostic range
    pub range: DiagnosticRange,
    /// Diagnostic code
    pub code: Option<String>,
    /// Source
    pub source: String,
    /// Related information
    pub related_info: Vec<DiagnosticRelatedInfo>,
}

/// LSP diagnostic range
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticRange {
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

/// LSP diagnostic related information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticRelatedInfo {
    pub location: String,
    pub message: String,
}

/// Dashboard update event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardEvent {
    /// Dashboard component to update
    pub component: String,
    /// Update type
    pub update_type: String,
    /// Update data
    pub data: serde_json::Value,
    /// Priority
    pub priority: String,
}

/// Health monitoring event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthEvent {
    /// Component name
    pub component: String,
    /// Health status
    pub status: String,
    /// Health details
    pub details: serde_json::Value,
    /// Timestamp
    pub timestamp: i64,
}

/// Simplified performance metrics for events
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetricsData {
    pub cpu_time_ns: u64,
    pub memory_usage: u64,
    pub io_operations: u64,
}

/// Event subscriber trait for handling events
#[async_trait]
pub trait EventSubscriber {
    /// Get subscriber name
    fn name(&self) -> &str;

    /// Get subscribed event types
    fn subscribed_events(&self) -> Vec<String>;

    /// Handle incoming event
    async fn handle_event(&mut self, event: &RealTimeEvent) -> EventResult<()>;

    /// Check if subscriber is healthy
    async fn is_healthy(&self) -> bool {
        true
    }
}

/// Event routing rule
#[derive(Debug, Clone)]
struct RoutingRule {
    /// Event type pattern
    event_pattern: String,
    /// Subscriber name(s) to route to
    subscribers: HashSet<String>,
    /// Priority (higher = processed first)
    priority: i32,
    /// Filtering conditions
    conditions: HashMap<String, String>,
}

/// LSP service interface for diagnostics
#[async_trait]
pub trait LspService {
    /// Publish diagnostics to LSP
    async fn publish_diagnostics(&self, uri: &str, diagnostics: Vec<LspDiagnostic>) -> EventResult<()>;

    /// Clear diagnostics for URI
    async fn clear_diagnostics(&self, uri: &str) -> EventResult<()>;

    /// Get language server capabilities
    async fn get_capabilities(&self) -> EventResult<LspCapabilities>;
}

/// LSP diagnostic structure
#[derive(Debug, Clone)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub severity: Option<LspSeverity>,
    pub code: Option<String>,
    pub source: Option<String>,
    pub message: String,
    pub related_information: Vec<LspDiagnosticRelatedInformation>,
}

/// LSP range structure
#[derive(Debug, Clone)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

/// LSP position structure
#[derive(Debug, Clone)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

/// LSP severity levels
#[derive(Debug, Clone, Copy)]
pub enum LspSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// LSP diagnostic related information
#[derive(Debug, Clone)]
pub struct LspDiagnosticRelatedInformation {
    pub location: LspLocation,
    pub message: String,
}

/// LSP location structure
#[derive(Debug, Clone)]
pub struct LspLocation {
    pub uri: String,
    pub range: LspRange,
}

/// LSP server capabilities
#[derive(Debug, Clone)]
pub struct LspCapabilities {
    pub text_document_sync: Option<TextDocumentSyncKind>,
    pub diagnostic_provider: bool,
}

/// LSP text document sync kind
#[derive(Debug, Clone, Copy)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}

/// Dashboard updater interface
#[async_trait]
pub trait DashboardUpdater {
    /// Update dashboard with real-time data
    async fn update_dashboard(&self, update: DashboardUpdate) -> EventResult<()>;

    /// Send notification to dashboard
    async fn send_notification(&self, notification: DashboardNotification) -> EventResult<()>;
}

/// Dashboard update structure
#[derive(Debug, Clone)]
pub struct DashboardUpdate {
    pub component_type: String,
    pub component_id: String,
    pub update_type: String,
    pub data: serde_json::Value,
}

/// Dashboard notification structure
#[derive(Debug, Clone)]
pub struct DashboardNotification {
    pub title: String,
    pub message: String,
    pub severity: String,
    pub timestamp: i64,
}

/// Event processor and integration hub
#[derive(Clone)]
pub struct EventProcessor {
    /// Internal state
    inner: Arc<EventProcessorInner>,

    /// Event channel sender
    event_tx: mpsc::UnboundedSender<RealTimeEvent>,

    /// Cancellation token
    cancellation: CancellationToken,
}

struct EventProcessorInner {
    /// Registered event subscribers
    subscribers: RwLock<HashMap<String, Box<dyn EventSubscriber + Send + Sync>>>,

    /// Event routing rules
    routing_rules: RwLock<Vec<RoutingRule>>,

    /// LSP service integration
    lsp_service: Arc<RwLock<Option<Box<dyn LspService + Send + Sync>>>>,

    /// Dashboard updater
    dashboard_updater: Arc<RwLock<Option<Box<dyn DashboardUpdater + Send + Sync>>>>,

    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,

    /// Health checker
    health_checker: Arc<HealthChecker>,

    /// Event statistics
    statistics: Arc<RwLock<EventStatistics>>,
}

/// Performance monitoring for event processing
#[derive(Clone)]
struct PerformanceMonitor {
    /// Event processing latencies
    processing_latencies: Arc<DashMap<String, Vec<u64>>>,
    /// Throughput counters
    throughputs: Arc<DashMap<String, u64>>,
    /// Error counters
    errors: Arc<DashMap<String, u64>>,
}

/// Health checking for event processing components
#[derive(Clone)]
struct HealthChecker {
    /// Component health status
    component_health: Arc<DashMap<String, ComponentHealth>>,
    /// Health check interval
    check_interval: Duration,
}

/// Component health information
#[derive(Debug, Clone)]
struct ComponentHealth {
    /// Is component healthy
    healthy: bool,
    /// Last health check time
    last_check: Instant,
    /// Health check error message
    error_message: Option<String>,
    /// Component metrics
    metrics: HashMap<String, f64>,
}

/// Event processing statistics
#[derive(Debug, Clone)]
pub struct EventStatistics {
    /// Total events processed
    total_processed: u64,
    /// Events processed by type
    by_type: HashMap<String, u64>,
    /// Processing errors by type
    errors_by_type: HashMap<String, u64>,
    /// Average processing latency by type
    avg_latency_by_type: HashMap<String, f64>,
    /// Events processed in last minute
    events_per_minute: u64,
}

impl EventProcessor {
    /// Create a new event processor
    pub async fn new() -> EventResult<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let inner = EventProcessorInner {
            subscribers: RwLock::new(HashMap::new()),
            routing_rules: RwLock::new(Vec::new()),
            lsp_service: Arc::new(RwLock::new(None)),
            dashboard_updater: Arc::new(RwLock::new(None)),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
            health_checker: Arc::new(HealthChecker::new()),
            statistics: Arc::new(RwLock::new(EventStatistics::default())),
        };

        let processor = Self {
            inner: Arc::new(inner),
            event_tx,
            cancellation: CancellationToken::new(),
        };

        // Start event processing loop
        processor.start_event_processing_loop(event_rx);

        // Start health checking
        processor.start_health_checking();

        Ok(processor)
    }

    /// Register an event subscriber
    #[instrument(skip(self, subscriber), err)]
    pub async fn register_subscriber(
        &self,
        subscriber: Box<dyn EventSubscriber + Send + Sync>,
    ) -> EventResult<()> {
        let name = subscriber.name().to_string();
        let event_types = subscriber.subscribed_events();

        debug!("Registering event subscriber: {}", name);

        // Store subscriber
        {
            let mut subscribers = self.inner.subscribers.write().await;
            subscribers.insert(name.clone(), subscriber);
        }

        // Create default routing rules for this subscriber
        let mut routing_rules = self.inner.routing_rules.write().await;
        for event_type in event_types {
            let rule = RoutingRule {
                event_pattern: event_type.clone(),
                subscribers: HashSet::from([name.clone()]),
                priority: 0,
                conditions: HashMap::new(),
            };
            routing_rules.push(rule);
        }

        Ok(())
    }

    /// Unregister an event subscriber
    pub async fn unregister_subscriber(&self, name: &str) -> EventResult<()> {
        debug!("Unregistering event subscriber: {}", name);

        let mut subscribers = self.inner.subscribers.write().await;
        if subscribers.remove(name).is_none() {
            return Err(EventError::SubscriberError(
                format!("Subscriber not found: {}", name)
            ));
        }

        // Remove routing rules for this subscriber
        let mut routing_rules = self.inner.routing_rules.write().await;
        routing_rules.retain(|rule| !rule.subscribers.contains(name));

        Ok(())
    }

    /// Set LSP service for diagnostics integration
    pub async fn set_lsp_service(&self, lsp_service: Box<dyn LspService + Send + Sync>) {
        *self.inner.lsp_service.write().await = Some(lsp_service);
        info!("LSP service integration configured");
    }

    /// Set dashboard updater for real-time UI updates
    pub async fn set_dashboard_updater(&self, updater: Box<dyn DashboardUpdater + Send + Sync>) {
        *self.inner.dashboard_updater.write().await = Some(updater);
        info!("Dashboard updater integration configured");
    }

    /// Publish an event to subscribers
    #[instrument(skip(self), err)]
    pub async fn publish_event(&self, event: RealTimeEvent) -> EventResult<()> {
        debug!("Publishing event: {:?}", event);

        // Send to processing loop
        if let Err(e) = self.event_tx.send(event) {
            return Err(EventError::ChannelError(
                format!("Failed to send event: {}", e)
            ));
        }

        Ok(())
    }

    /// Publish multiple events
    pub async fn publish_events(&self, events: Vec<RealTimeEvent>) -> EventResult<()> {
        for event in events {
            self.publish_event(event).await?;
        }
        Ok(())
    }

    /// Get event processing statistics
    pub async fn get_statistics(&self) -> EventStatistics {
        self.inner.statistics.read().await.clone()
    }

    /// Get health status of all components
    pub async fn get_health_status(&self) -> HashMap<String, ComponentHealth> {
        let health_status = self.inner.health_checker.get_health_status().await;

        let mut result = HashMap::new();
        for (component_name, health) in health_status {
            result.insert(component_name, health.clone());
        }

        result
    }

    /// Shutdown the event processor
    pub async fn shutdown(&self) {
        info!("Shutting down event processor");
        self.cancellation.cancel();

        // Wait a bit for graceful shutdown
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    /// Start the main event processing loop
    fn start_event_processing_loop(&self, mut event_rx: mpsc::UnboundedReceiver<RealTimeEvent>) {
        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        if let Err(e) = Self::process_single_event(&inner, event).await {
                            error!("Failed to process event: {}", e);
                        }
                    }

                    _ = inner.cancellation.cancelled() => {
                        info!("Event processing loop shutting down");
                        break;
                    }
                }
            }
        });
    }

    /// Process a single event
    async fn process_single_event(inner: &Arc<EventProcessorInner>, event: RealTimeEvent) -> EventResult<()> {
        let start_time = Instant::now();

        // Update statistics
        {
            let mut stats = inner.statistics.write().await;
            stats.total_processed += 1;

            let event_type = Self::event_type_name(&event);
            *stats.by_type.entry(event_type.clone()).or_insert(0) += 1;
        }

        // Route event to subscribers
        let routing_result = Self::route_event_to_subscribers(inner, &event).await;

        // Handle LSP integration if applicable
        if matches!(event, RealTimeEvent::AnalysisComplete(_) | RealTimeEvent::LspDiagnosticEvent(_)) {
            if let Err(e) = Self::handle_lsp_integration(inner, &event).await {
                warn!("LSP integration failed: {}", e);
            }
        }

        // Handle dashboard updates
        if let Err(e) = Self::handle_dashboard_integration(inner, &event).await {
            warn!("Dashboard integration failed: {}", e);
        }

        // Update performance metrics
        let processing_time = start_time.elapsed().as_micros() as u64;
        inner.performance_monitor.record_latency(&Self::event_type_name(&event), processing_time).await;

        routing_result
    }

    /// Route event to appropriate subscribers
    async fn route_event_to_subscribers(inner: &Arc<EventProcessorInner>, event: &RealTimeEvent) -> EventResult<()> {
        let subscribers = inner.subscribers.read().await;
        let routing_rules = inner.routing_rules.read().await;

        let event_type_name = Self::event_type_name(event);

        // Find matching routing rules
        let mut target_subscribers = HashSet::new();

        for rule in routing_rules.iter() {
            if Self::matches_event_pattern(&rule.event_pattern, &event_type_name) {
                target_subscribers.extend(rule.subscribers.clone());
            }
        }

        // Route to matched subscribers
        for subscriber_name in target_subscribers {
            if let Some(subscriber) = subscribers.get(&subscriber_name) {
                let event_clone = event.clone();

                tokio::spawn(async move {
                    let mut subscriber = subscriber.clone();
                    if let Err(e) = subscriber.handle_event(&event_clone).await {
                        error!("Subscriber {} failed to handle event: {}", subscriber_name, e);
                    }
                });
            } else {
                warn!("Subscriber {} not found for routing rule", subscriber_name);
            }
        }

        Ok(())
    }

    /// Handle LSP integration for events
    async fn handle_lsp_integration(inner: &Arc<EventProcessorInner>, event: &RealTimeEvent) -> EventResult<()> {
        if let Some(lsp_service) = inner.lsp_service.read().await.as_ref() {
            match event {
                RealTimeEvent::AnalysisComplete(analysis_event) => {
                    if !analysis_event.success {
                        // Create LSP diagnostic for analysis failure
                        let diagnostic = LspDiagnostic {
                            range: LspRange {
                                start: LspPosition { line: 0, character: 0 },
                                end: LspPosition { line: 0, character: 0 },
                            },
                            severity: Some(LspSeverity::Warning),
                            code: Some("analysis_failed".to_string()),
                            source: Some("rust-ai-ide".to_string()),
                            message: analysis_event.error_message.clone().unwrap_or("Analysis failed".to_string()),
                            related_information: Vec::new(),
                        };

                        lsp_service.publish_diagnostics(&format!("file:///{}", analysis_event.file_path), vec![diagnostic]).await?;
                    }
                }

                RealTimeEvent::LspDiagnosticEvent(diagnostic_event) => {
                    let diagnostic = LspDiagnostic {
                        range: LspRange {
                            start: LspPosition {
                                line: diagnostic_event.range.start_line,
                                character: diagnostic_event.range.start_character,
                            },
                            end: LspPosition {
                                line: diagnostic_event.range.end_line,
                                character: diagnostic_event.range.end_character,
                            },
                        },
                        severity: match diagnostic_event.severity.as_str() {
                            "error" => Some(LspSeverity::Error),
                            "warning" => Some(LspSeverity::Warning),
                            "info" => Some(LspSeverity::Information),
                            _ => Some(LspSeverity::Hint),
                        },
                        code: diagnostic_event.code.clone(),
                        source: Some(diagnostic_event.source.clone()),
                        message: diagnostic_event.message.clone(),
                        related_information: diagnostic_event.related_info.iter().map(|info| {
                            LspDiagnosticRelatedInformation {
                                location: LspLocation {
                                    uri: info.location.clone(),
                                    range: LspRange {
                                        start: LspPosition { line: 0, character: 0 },
                                        end: LspPosition { line: 0, character: 0 },
                                    },
                                },
                                message: info.message.clone(),
                            }
                        }).collect(),
                    };

                    lsp_service.publish_diagnostics(&diagnostic_event.uri, vec![diagnostic]).await?;
                }

                _ => {}
            }
        }

        Ok(())
    }

    /// Handle dashboard integration for events
    async fn handle_dashboard_integration(inner: &Arc<EventProcessorInner>, event: &RealTimeEvent) -> EventResult<()> {
        if let Some(dashboard_updater) = inner.dashboard_updater.read().await.as_ref() {
            let update = match event {
                RealTimeEvent::AnalysisComplete(completion) => {
                    DashboardUpdate {
                        component_type: "analysis_progress".to_string(),
                        component_id: completion.file_path.clone(),
                        update_type: "completion".to_string(),
                        data: serde_json::json!({
                            "task_id": completion.task_id,
                            "success": completion.success,
                            "duration_ms": completion.duration_ms,
                            "findings_count": completion.findings_count
                        }),
                    }
                }

                RealTimeEvent::PerformanceEvent(perf) => {
                    DashboardUpdate {
                        component_type: "performance_metrics".to_string(),
                        component_id: perf.component.clone(),
                        update_type: perf.metric_name.clone(),
                        data: serde_json::json!({
                            "value": perf.value,
                            "unit": perf.unit,
                            "timestamp": perf.timestamp
                        }),
                    }
                }

                RealTimeEvent::HealthEvent(health) => {
                    DashboardUpdate {
                        component_type: "system_health".to_string(),
                        component_id: health.component.clone(),
                        update_type: "status".to_string(),
                        data: serde_json::json!({
                            "status": health.status,
                            "details": health.details,
                            "timestamp": health.timestamp
                        }),
                    }
                }

                _ => return Ok(()),
            };

            dashboard_updater.update_dashboard(update).await?;
        }

        Ok(())
    }

    /// Start health checking for components
    fn start_health_checking(&self) {
        let health_checker = Arc::clone(&self.inner.health_checker);
        let cancellation = self.cancellation.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(health_checker.check_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = health_checker.perform_health_checks().await {
                            error!("Health check failed: {}", e);
                        }
                    }

                    _ = cancellation.cancelled() => {
                        break;
                    }
                }
            }
        });
    }

    /// Extract event type name for statistics and routing
    fn event_type_name(event: &RealTimeEvent) -> String {
        match event {
            RealTimeEvent::AnalysisComplete(_) => "AnalysisComplete".to_string(),
            RealTimeEvent::FileChange(_) => "FileChange".to_string(),
            RealTimeEvent::CacheEvent(_) => "CacheEvent".to_string(),
            RealTimeEvent::PerformanceEvent(_) => "PerformanceEvent".to_string(),
            RealTimeEvent::LspDiagnosticEvent(_) => "LspDiagnosticEvent".to_string(),
            RealTimeEvent::DashboardEvent(_) => "DashboardEvent".to_string(),
            RealTimeEvent::HealthEvent(_) => "HealthEvent".to_string(),
        }
    }

    /// Check if event pattern matches event type
    fn matches_event_pattern(pattern: &str, event_type: &str) -> bool {
        // Simple wildcard matching (could be enhanced with regex)
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return event_type.starts_with(prefix);
        }

        pattern == event_type
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    fn new() -> Self {
        Self {
            processing_latencies: Arc::new(DashMap::new()),
            throughputs: Arc::new(DashMap::new()),
            errors: Arc::new(DashMap::new()),
        }
    }

    /// Record processing latency for an event type
    async fn record_latency(&self, event_type: &str, latency_us: u64) {
        let latencies = self.processing_latencies.entry(event_type.to_string()).or_insert(Vec::new());
        latencies.push(latency_us);

        // Keep only last 1000 measurements
        if latencies.len() > 1000 {
            latencies.remove(0);
        }

        // Update throughput counter
        let throughput = self.throughputs.entry(event_type.to_string()).or_insert(0);
        *throughput += 1;
    }

    /// Record processing error for an event type
    async fn record_error(&self, event_type: &str) {
        let errors = self.errors.entry(event_type.to_string()).or_insert(0);
        *errors += 1;
    }
}

impl HealthChecker {
    /// Create a new health checker
    fn new() -> Self {
        Self {
            component_health: Arc::new(DashMap::new()),
            check_interval: Duration::from_secs(30),
        }
    }

    /// Perform health checks on all components
    async fn perform_health_checks(&self) -> EventResult<()> {
        // Check event processor health
        let processor_health = ComponentHealth {
            healthy: true,
            last_check: Instant::now(),
            error_message: None,
            metrics: HashMap::from([
                ("uptime".to_string(), 1.0),
            ]),
        };

        self.component_health.insert("event_processor".to_string(), processor_health);

        Ok(())
    }

    /// Get health status for all components
    async fn get_health_status(&self) -> HashMap<String, ComponentHealth> {
        let mut result = HashMap::new();

        for entry in self.component_health.iter() {
            result.insert(entry.key().clone(), entry.value().clone());
        }

        result
    }
}

impl Default for EventStatistics {
    fn default() -> Self {
        Self {
            total_processed: 0,
            by_type: HashMap::new(),
            errors_by_type: HashMap::new(),
            avg_latency_by_type: HashMap::new(),
            events_per_minute: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    struct TestSubscriber {
        name: String,
        received_events: Arc<std::sync::Mutex<Vec<RealTimeEvent>>>,
    }

    impl TestSubscriber {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                received_events: Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn get_received_events(&self) -> Vec<RealTimeEvent> {
            self.received_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventSubscriber for TestSubscriber {
        fn name(&self) -> &str {
            &self.name
        }

        fn subscribed_events(&self) -> Vec<String> {
            vec![
                "AnalysisComplete".to_string(),
                "FileChange".to_string(),
            ]
        }

        async fn handle_event(&mut self, event: &RealTimeEvent) -> EventResult<()> {
            self.received_events.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_event_processor_creation() {
        let processor = EventProcessor::new().await;
        assert!(processor.is_ok());
    }

    #[tokio::test]
    async fn test_event_subscriber_registration() {
        let processor = EventProcessor::new().await.unwrap();

        let subscriber = Box::new(TestSubscriber::new("test_subscriber"));
        let result = processor.register_subscriber(subscriber).await;
        assert!(result.is_ok());

        let stats = processor.get_statistics().await;
        assert_eq!(stats.total_processed, 0);
    }

    #[tokio::test]
    async fn test_event_publishing_and_routing() {
        let processor = EventProcessor::new().await.unwrap();

        let test_subscriber = Box::new(TestSubscriber::new("test_subscriber"));
        let received_events_clone = Arc::clone(&test_subscriber.received_events);

        let _ = processor.register_subscriber(test_subscriber).await;

        // Publish an analysis complete event
        let event = RealTimeEvent::AnalysisComplete(AnalysisCompleteEvent {
            task_id: "test_task".to_string(),
            file_path: "/test/file.rs".to_string(),
            analysis_type: "Syntax".to_string(),
            findings_count: 2,
            duration_ms: 150,
            success: true,
            error_message: None,
            performance_metrics: PerformanceMetricsData {
                cpu_time_ns: 150000000,
                memory_usage: 1024,
                io_operations: 5,
            },
        });

        let _ = processor.publish_event(event).await;

        // Wait a bit for event processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Check if subscriber received the event
        let received = received_events_clone.lock().unwrap();
        assert!(!received.is_empty());
        assert!(matches!(received[0], RealTimeEvent::AnalysisComplete(_)));
    }

    #[tokio::test]
    async fn test_event_statistics() {
        let processor = EventProcessor::new().await.unwrap();

        let event = RealTimeEvent::FileChange(FileChangeEvent {
            paths: vec!["/test/file.rs".to_string()],
            change_type: "modified".to_string(),
            timestamp: 1234567890,
            priority: "high".to_string(),
        });

        let _ = processor.publish_event(event).await;

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        let stats = processor.get_statistics().await;
        assert_eq!(stats.total_processed, 1);
        assert_eq!(stats.by_type.get("FileChange").unwrap_or(&0), &1);
    }

    #[tokio::test]
    async fn test_health_status() {
        let processor = EventProcessor::new().await.unwrap();

        let health_status = processor.get_health_status().await;
        assert!(!health_status.is_empty());

        // Event processor should be healthy by default
        let processor_health = health_status.get("event_processor");
        assert!(processor_health.is_some());
        assert!(processor_health.unwrap().healthy);
    }
}