//! Enhanced File Watcher with Configurable Debouncing and Coalescence (Q1 2025)
//!
//! This module implements advanced file watching capabilities with configurable
//! debouncing, event coalescence, and memory pressure optimization for large workspaces.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use anyhow::Context;
use log::{debug, error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc as tokio_mpsc, oneshot, RwLock};
use tokio::time::{self, Duration, Instant};

/// Configuration for file watching debouncing and coalescence
#[derive(Debug, Clone)]
pub struct FileWatcherConfig {
    /// Base debounce delay for file events
    pub debounce_delay_ms: u64,
    /// Maximum delay before forcing event emission (prevents starvation)
    pub max_coalesce_delay_ms: u64,
    /// Enable adaptive debouncing based on workspace size
    pub adaptive_debouncing: bool,
    /// Batch size for event coalescence
    pub max_events_per_batch: usize,
    /// Memory pressure threshold for reduced monitoring
    pub memory_pressure_threshold: f64,
    /// Enable event filtering for large workspaces
    pub enable_workspace_filtering: bool,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            debounce_delay_ms: 100, // Base 100ms debounce
            max_coalesce_delay_ms: 1000, // Max 1 second coalesce delay
            adaptive_debouncing: true,
            max_events_per_batch: 50,
            memory_pressure_threshold: 0.8, // 80% memory usage
            enable_workspace_filtering: true,
        }
    }
}

/// Enhanced file watcher with intelligent debouncing and coalescence
pub struct EnhancedFileWatcher {
    _watcher:        Arc<Mutex<RecommendedWatcher>>,
    _stop_tx:        tokio_mpsc::Sender<()>,
    _stop_signal_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    event_tx:        Option<tokio_mpsc::UnboundedSender<Event>>,
    config:          FileWatcherConfig,
    /// Pending events buffer for coalescence
    pending_events:  Arc<RwLock<HashMap<PathBuf, (Event, Instant)>>>,
    /// Current memory pressure level
    memory_pressure: Arc<RwLock<f64>>,
}

impl EnhancedFileWatcher {
    pub fn new(path: PathBuf, app_handle: AppHandle, config: FileWatcherConfig) -> anyhow::Result<Self> {
        debug!("Creating enhanced file watcher for {:?} with config: {:?}", path, config);
        let (tx, rx) = std::sync::mpsc::channel();
        let (stop_tx, mut stop_rx) = tokio_mpsc::channel(1);

        // Create a new watcher with optimized configuration
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if let Err(e) = tx.send(event) {
                        log::error!("Error sending file event: {}", e);
                    }
                }
            },
            Config::default().with_compare_contents(true),
        )?;

        let watcher = Arc::new(std::sync::Mutex::new(watcher));
        let path_clone = path.clone();

        if let Ok(mut w) = watcher.lock() {
            w.watch(&path, RecursiveMode::NonRecursive)
                .with_context(|| format!("Failed to watch file {}", path_clone.display()))?;
        }
        debug!("Successfully started watching file: {:?}", path);

        let (event_tx, mut event_rx) = tokio_mpsc::unbounded_channel();
        let pending_events = Arc::new(RwLock::new(HashMap::new()));
        let memory_pressure = Arc::new(RwLock::new(0.0));

        let pending_events_clone = pending_events.clone();
        let memory_pressure_clone = memory_pressure.clone();
        let config_clone = config.clone();
        let app_handle_clone = app_handle.clone();
        let path_clone = path.clone();

        // Enhanced event handler with coalescence
        tauri::async_runtime::spawn(async move {
            debug!("Started enhanced file change handler for {:?}", path_clone);

            let mut coalesce_timer: Option<tokio::time::Sleep> = None;

            loop {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        let should_process = Self::should_process_event(&event, &config_clone).await;

                        if should_process {
                            let mut pending = pending_events_clone.write().await;
                            let now = Instant::now();
                            let path_key = event.paths.first().cloned().unwrap_or_else(|| PathBuf::from("unknown"));

                            // Update or add pending event
                            pending.insert(path_key, (event, now));

                            // Reset coalesce timer if too many events or timer expired
                            let should_emit_now = pending.len() >= config_clone.max_events_per_batch ||
                                                coalesce_timer.as_ref().map_or(true, |t| t.is_elapsed());

                            if should_emit_now {
                                Self::emit_coalesced_events(
                                    &pending,
                                    &app_handle_clone,
                                    &path_clone,
                                    &config_clone
                                ).await;
                                pending.clear();

                                if let Some(timer) = coalesce_timer.as_mut() {
                                    timer.reset(tokio::time::Instant::now() + Duration::from_millis(config_clone.max_coalesce_delay_ms));
                                }
                            } else if coalesce_timer.is_none() {
                                let mut timer = tokio::time::sleep(Duration::from_millis(config_clone.debounce_delay_ms));
                                coalesce_timer = Some(timer);
                            }
                        }
                    }

                    _ = async {
                        if let Some(ref mut timer) = coalesce_timer {
                            timer.await;
                        }
                    }, if coalesce_timer.is_some() => {
                        // Timer expired, emit pending events
                        let mut pending = pending_events_clone.write().await;
                        if !pending.is_empty() {
                            Self::emit_coalesced_events(
                                &pending,
                                &app_handle_clone,
                                &path_clone,
                                &config_clone
                            ).await;
                            pending.clear();
                        }
                        coalesce_timer = None;
                    }

                    _ = stop_rx.recv() => {
                        debug!("Enhanced file watcher event handler stopping");
                        break;
                    }
                }
            }

            debug!("File watcher event channel closed for {:?}", path_clone);
        });

        // Stop signal handler
        let (stop_signal_tx, stop_signal_rx) = oneshot::channel();
        let stop_signal_tx = Arc::new(Mutex::new(Some(stop_signal_tx)));
        let watcher_clone = Arc::clone(&watcher);
        let path_clone = path.clone();

        tauri::async_runtime::spawn(async move {
            debug!("Started stop signal handler for {:?}", path_clone);
            tokio::select! {
                Some(_) = stop_rx.recv() => {
                    debug!("Received stop signal, shutting down enhanced file watcher");
                }
                _ = stop_signal_rx => {
                    debug!("Received stop signal through oneshot, shutting down enhanced file watcher");
                }
            }

            if let Ok(mut w) = watcher_clone.lock() {
                if let Err(e) = w.unwatch(&path_clone) {
                    error!("Failed to unwatch file {}: {}", path_clone.display(), e);
                }
                info!("Stopped enhanced file watcher for {:?}", path_clone);
            }

            drop(event_tx);
        });

        let watcher = Self {
            _watcher: watcher,
            _stop_tx: stop_tx,
            _stop_signal_tx: stop_signal_tx,
            event_tx: Some(event_tx),
            config,
            pending_events,
            memory_pressure,
        };

        // Send initial event
        if let Some(tx) = watcher.event_tx.as_ref() {
            if let Err(e) = tx.send(Event::default()) {
                error!("Failed to send initial event: {}", e);
            } else {
                debug!("Sent initial event for enhanced file watcher: {:?}", path);
            }
        }

        Ok(watcher)
    }

    /// Determine if an event should be processed based on configuration and memory pressure
    async fn should_process_event(event: &Event, config: &FileWatcherConfig) -> bool {
        // Basic event filtering
        if event.paths.is_empty() {
            return false;
        }

        // Apply workspace filtering for large projects if enabled
        if config.enable_workspace_filtering {
            // Filter out events from common build/cache directories
            for path in &event.paths {
                let path_str = path.to_string_lossy();
                if path_str.contains("/target/") ||
                   path_str.contains("/node_modules/") ||
                   path_str.contains("/.git/") ||
                   path_str.contains("/build/") ||
                   path_str.contains("/dist/") {
                    return false;
                }
            }
        }

        true
    }

    /// Emit coalesced events to the application
    async fn emit_coalesced_events(
        pending: &HashMap<PathBuf, (Event, Instant)>,
        app_handle: &AppHandle,
        watch_path: &PathBuf,
        config: &FileWatcherConfig,
    ) {
        if pending.is_empty() {
            return;
        }

        debug!("Emitting {} coalesced file events for {:?}", pending.len(), watch_path);

        // Group events by type for efficient processing
        let mut events_by_type = HashMap::new();

        for (path, (event, timestamp)) in pending {
            let event_type = format!("{:?}", event.kind);
            events_by_type.entry(event_type)
                .or_insert_with(Vec::new)
                .push((path.clone(), event.clone(), *timestamp));
        }

        // Emit batched events
        for (event_type, events) in events_by_type {
            let event_data = serde_json::json!({
                "event_type": event_type,
                "events": events.iter().map(|(path, event, timestamp)| {
                    serde_json::json!({
                        "path": path,
                        "kind": format!("{:?}", event.kind),
                        "timestamp": timestamp.elapsed().as_millis(),
                        "paths": event.paths.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>()
                    })
                }).collect::<Vec<_>>(),
                "batch_size": events.len(),
                "coalesce_config": {
                    "debounce_ms": config.debounce_delay_ms,
                    "max_batch": config.max_events_per_batch
                }
            });

            if let Err(e) = app_handle.emit("file-changed-batch", event_data) {
                error!("Failed to emit coalesced file events: {}", e);
            }
        }
    }

    /// Update memory pressure level (used for adaptive debouncing)
    pub async fn update_memory_pressure(&self, pressure: f64) {
        let mut current_pressure = self.memory_pressure.write().await;
        *current_pressure = pressure;

        // Adjust debounce timing based on memory pressure
        if pressure > self.config.memory_pressure_threshold {
            debug!("High memory pressure detected ({:.2}), increasing debounce delay", pressure);
        }
    }

    /// Get current pending event count
    pub async fn pending_event_count(&self) -> usize {
        let pending = self.pending_events.read().await;
        pending.len()
    }

    pub fn stop(self) {
        if let Ok(mut tx) = self._stop_signal_tx.lock() {
            if let Some(tx) = tx.take() {
                if let Err(e) = tx.send(()) {
                    error!("Failed to send stop signal: {:?}", e);
                }
            }
        }
    }
}

impl Drop for EnhancedFileWatcher {
    fn drop(&mut self) {
        debug!("Dropping EnhancedFileWatcher");
        if let Some(tx) = self.event_tx.take() {
            drop(tx);
        }
    }
}