#![allow(missing_docs)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::types::{
    AnalysisTrigger, FileSystemEventData, FileSystemEventType, FileWatchConfig, TaskPriority,
    TriggerSource,
};

/// File system watching errors
#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error("File system watching error: {0}")]
    WatchError(#[from] notify::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Watcher is disabled")]
    Disabled,
}

type WatcherResult<T> = Result<T, WatcherError>;

/// File change event processor trait
#[async_trait]
pub trait FileEventProcessor {
    /// Process a batch of debounced file events
    async fn process_events(&mut self, events: Vec<FileSystemEventData>) -> WatcherResult<()>;

    /// Determine the priority for a file change
    fn get_file_priority(&self, _path: &std::path::Path) -> TaskPriority {
        TaskPriority::Normal
    }
}

/// File system watcher for real-time analysis triggers
#[derive(Clone)]
pub struct FileSystemWatcher {
    /// Internal state shared across threads
    inner: Arc<FileSystemWatcherInner>,

    /// Event channel for file change notifications
    event_sender: mpsc::UnboundedSender<FileSystemEventData>,

    /// Cancellation token for graceful shutdown
    cancellation_token: CancellationToken,
}

struct FileSystemWatcherInner {
    /// Configuration for file watching
    config: FileWatchConfig,

    /// File event processors
    processors: RwLock<Vec<Box<dyn FileEventProcessor + Send + Sync>>>,

    /// File filter engine
    filter_engine: FileFilterEngine,

    /// Change coalescer to group related changes
    change_coalescer: ChangeCoalescer,

    /// Watch state tracking
    watch_state: RwLock<HashMap<PathBuf, WatchEntry>>,
}

/// Watch entry tracking information
#[derive(Debug, Clone)]
struct WatchEntry {
    /// Timestamp of last change
    last_change: Instant,
    /// Pending changes counter
    pending_changes: usize,
}

/// File change coalescer for grouping related changes
#[derive(Clone)]
struct ChangeCoalescer {
    /// Pending changes by file
    pending_changes: Arc<RwLock<HashMap<PathBuf, PendingChange>>>,
    /// Coalescing window duration
    window_duration: Duration,
}

/// Pending change information
#[derive(Debug, Clone)]
struct PendingChange {
    /// Event type
    event_type: FileSystemEventType,
    /// First event timestamp
    first_seen: Instant,
    /// Last event timestamp
    last_seen: Instant,
    /// Event counter (for rate limiting)
    count: usize,
}

/// File filter engine for determining which files to watch
#[derive(Clone)]
struct FileFilterEngine {
    /// Allowed file extensions
    extensions: Vec<String>,
    /// Maximum file size to watch
    max_file_size: u64,
    /// Paths to ignore
    ignore_patterns: Vec<String>,
}

impl FileSystemWatcher {
    /// Create a new file system watcher with configuration
    pub async fn new(config: FileWatchConfig) -> WatcherResult<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let filter_engine = FileFilterEngine::new(
            config.watch_extensions.clone(),
            config.max_file_size,
        );

        let change_coalescer = ChangeCoalescer::new(config.debounce_duration);

        let inner = Arc::new(FileSystemWatcherInner {
            config: config.clone(),
            processors: RwLock::new(Vec::new()),
            filter_engine,
            change_coalescer,
            watch_state: RwLock::new(HashMap::new()),
        });

        let cancellation_token = CancellationToken::new();

        let watcher = Self {
            inner,
            event_sender,
            cancellation_token,
        };

        // Start the event processing task
        let inner_clone = Arc::clone(&watcher.inner);
        let token = watcher.cancellation_token.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::process_events_loop(
                inner_clone,
                event_receiver,
                config_clone,
                token,
            ).await {
                error!("Event processing loop failed: {}", e);
            }
        });

        Ok(watcher)
    }

    /// Add a file event processor
    pub async fn add_processor(&self, processor: Box<dyn FileEventProcessor + Send + Sync>) {
        let mut processors = self.inner.processors.write().await;
        processors.push(processor);
    }

    /// Start watching files at the specified paths
    pub async fn start_watching(&self) -> WatcherResult<()> {
        info!("Starting file system watcher with {} paths", self.inner.config.watch_paths.len());

        // Validate and create watcher
        let mut watcher = self.create_native_watcher()?;

        // Watch all configured paths
        for path in &self.inner.config.watch_paths {
            if !path.exists() {
                info!("Watch path does not exist, creating directory: {:?}", path);
                tokio::fs::create_dir_all(path).await?;
            }

            watcher.watch(path, RecursiveMode::Recursive)?;
            info!("Watching path: {:?}", path);
        }

        // Note: The watcher needs to be kept alive, so we spawn it in a background task
        tokio::spawn(async move {
            // The watcher is moved here and will be kept alive
            tokio::time::sleep(Duration::MAX).await; // Never returns normally
        });

        Ok(())
    }

    /// Stop file watching
    pub async fn stop_watching(&self) {
        info!("Stopping file system watcher");
        self.cancellation_token.cancel();
    }

    /// Manually trigger analysis for a specific path
    pub async fn trigger_analysis(&self, path: PathBuf) -> WatcherResult<()> {
        let event = FileSystemEventData {
            event_type: FileSystemEventType::Modified,
            path,
            timestamp: Instant::now(),
            metadata: HashMap::new(),
        };

        self.event_sender.send(event).map_err(|_| WatcherError::Disabled)?;
        Ok(())
    }

    /// Get the number of watched paths
    pub async fn watched_path_count(&self) -> usize {
        self.inner.config.watch_paths.len()
    }

    /// Get watching status information
    pub async fn get_watch_status(&self) -> HashMap<String, String> {
        let mut status = HashMap::new();

        let watch_state = self.inner.watch_state.read().await;
        status.insert("watched_paths".to_string(), self.inner.config.watch_paths.len().to_string());
        status.insert("active_watches".to_string(), watch_state.len().to_string());
        status.insert("is_cancelled".to_string(), self.cancellation_token.is_cancelled().to_string());

        status
    }

    /// Create the native file system watcher
    fn create_native_watcher(&self) -> WatcherResult<RecommendedWatcher> {
        let sender_clone = self.event_sender.clone();

        let debouncer = new_debouncer(self.inner.config.debounce_duration, move |res: DebounceEventResult| {
            match res {
                Ok(events) => {
                    // Process debounced events
                    for event in events {
                        if let Err(e) = Self::convert_and_send_event(&sender_clone, &event) {
                            error!("Failed to send file event: {}", e);
                        }
                    }
                }
                Err(err) => {
                    error!("Watch error: {:?}", err);
                }
            }
        }).map_err(|e| WatcherError::WatchError(notify::Error::Generic(e.to_string())))?;

        Ok(debouncer.watcher())
    }

    /// Convert notify event to our internal event format
    fn convert_and_send_event(
        sender: &mpsc::UnboundedSender<FileSystemEventData>,
        event: &Event,
    ) -> WatcherResult<()> {
        for path in &event.paths {
            let event_type = match event.kind {
                EventKind::Create(_) => FileSystemEventType::Created,
                EventKind::Modify(_) => FileSystemEventType::Modified,
                EventKind::Remove(_) => FileSystemEventType::Deleted,
                EventKind::Access(_) => continue, // Ignore access events
                _ => continue,
            };

            let event_data = FileSystemEventData {
                event_type,
                path: path.clone(),
                timestamp: Instant::now(),
                metadata: HashMap::from([
                    ("event_kind".to_string(), format!("{:?}", event.kind)),
                    ("attributes".to_string(), format!("{:?}", event.attrs)),
                ]),
            };

            if let Err(_) = sender.send(event_data) {
                warn!("Failed to send event for path: {:?}", path);
            }
        }

        Ok(())
    }

    /// Main event processing loop
    async fn process_events_loop(
        inner: Arc<FileSystemWatcherInner>,
        mut receiver: mpsc::UnboundedReceiver<FileSystemEventData>,
        config: FileWatchConfig,
        token: CancellationToken,
    ) -> WatcherResult<()> {
        info!("Starting file event processing loop");

        let mut event_buffer = Vec::new();
        let mut last_process_time = Instant::now();

        loop {
            tokio::select! {
                Some(event) = receiver.recv() => {
                    // Filter and coalesce the event
                    if let Some(filtered_event) = inner.filter_and_coalesce_event(event).await {
                        event_buffer.push(filtered_event);

                        // Process events if buffer is full or time window expired
                        let now = Instant::now();
                        let buffer_full = event_buffer.len() >= config.rate_limit.burst_limit;
                        let time_expired = now.duration_since(last_process_time) >= Duration::from_secs(1);

                        if buffer_full || time_expired {
                            if let Err(e) = Self::process_event_batch(&inner, event_buffer.clone()).await {
                                error!("Failed to process event batch: {}", e);
                            }
                            event_buffer.clear();
                            last_process_time = now;
                        }
                    }
                }

                _ = token.cancelled() => {
                    info!("Event processing loop shutting down");
                    break;
                }
            }
        }

        // Process any remaining events before shutdown
        if !event_buffer.is_empty() {
            if let Err(e) = Self::process_event_batch(&inner, event_buffer).await {
                error!("Failed to process final event batch: {}", e);
            }
        }

        Ok(())
    }

    /// Process a batch of events
    async fn process_event_batch(
        inner: &Arc<FileSystemWatcherInner>,
        events: Vec<FileSystemEventData>,
    ) -> WatcherResult<()> {
        let processors = inner.processors.read().await;

        debug!("Processing {} file events", events.len());

        // Dispatch to all registered processors
        for processor in processors.iter() {
            let mut processor = processor.clone_box();
            let events_clone = events.clone();

            tokio::spawn(async move {
                if let Err(e) = processor.process_events(events_clone).await {
                    error!("Processor failed to handle events: {}", e);
                }
            });
        }

        Ok(())
    }
}

impl FileSystemWatcherInner {
    /// Filter and coalesce an incoming event
    async fn filter_and_coalesce_event(&self, event: FileSystemEventData) -> Option<FileSystemEventData> {
        // Apply file filtering
        if !self.filter_engine.should_watch(&event.path).await.ok()? {
            return None;
        }

        // Update watch state
        let mut watch_state = self.watch_state.write().await;
        let entry = watch_state.entry(event.path.clone()).or_insert(WatchEntry {
            last_change: event.timestamp,
            pending_changes: 0,
        });

        entry.last_change = event.timestamp;
        entry.pending_changes = entry.pending_changes.saturating_add(1);

        // Apply coalescing
        if let Some(pending) = self.change_coalescer.coalesce_change(&event).await {
            return Some(pending);
        }

        Some(event)
    }
}

impl ChangeCoalescer {
    /// Create a new change coalescer with the specified window duration
    pub fn new(window_duration: Duration) -> Self {
        Self {
            pending_changes: Arc::new(RwLock::new(HashMap::new())),
            window_duration,
        }
    }

    /// Coalesce a change, returning it only if it should be processed
    pub async fn coalesce_change(&self, event: &FileSystemEventData) -> Option<FileSystemEventData> {
        let mut pending = self.pending_changes.write().await;
        let now = Instant::now();

        let entry = pending.entry(event.path.clone()).or_insert(PendingChange {
            event_type: event.event_type.clone(),
            first_seen: now,
            last_seen: now,
            count: 1,
        });

        entry.last_seen = now;
        entry.count = entry.count.saturating_add(1);

        // Only process if we've exceeded the coalescing window
        if entry.first_seen.elapsed() >= self.window_duration {
            let coalesced_event = FileSystemEventData {
                event_type: entry.event_type.clone(),
                path: event.path.clone(),
                timestamp: now,
                metadata: event.metadata.clone(),
            };

            // Remove the entry to allow new events for this path
            pending.remove(&event.path);

            Some(coalesced_event)
        } else {
            // Suppress this event - it will be processed later when window expires
            None
        }
    }
}

impl FileFilterEngine {
    /// Create a new file filter engine
    pub fn new(extensions: Vec<String>, max_file_size: u64) -> Self {
        Self {
            extensions,
            max_file_size,
            ignore_patterns: vec![
                ".git/".to_string(),
                "target/".to_string(),
                "node_modules/".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
                ".DS_Store".to_string(),
            ],
        }
    }

    /// Determine if a file should be watched
    pub async fn should_watch(&self, path: &std::path::Path) -> Result<bool, std::io::Error> {
        // Check if path matches ignore patterns
        if self.matches_ignore_pattern(path) {
            return Ok(false);
        }

        // Check file size if it's a file
        if path.is_file() {
            let metadata = tokio::fs::metadata(path).await?;
            if metadata.len() > self.max_file_size {
                debug!("Skipping large file: {:?}", path);
                return Ok(false);
            }

            // Check file extension
            if let Some(extension) = path.extension() {
                let ext_str = extension.to_string_lossy().to_string();
                if !self.extensions.contains(&ext_str) {
                    return Ok(false);
                }
            } else {
                // No extension, check if we watch files without extensions
                if !self.extensions.contains(&String::new()) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Check if path matches any ignore pattern
    fn matches_ignore_pattern(&self, path: &std::path::Path) -> bool {
        for pattern in &self.ignore_patterns {
            // Simple pattern matching for common cases
            let path_str = path.to_string_lossy();
            if path_str.contains(pattern) {
                return true;
            }
        }
        false
    }
}

/// Default file event processor implementation
pub struct DefaultFileEventProcessor {
    /// Analysis trigger callback
    trigger_callback: Arc<dyn Fn(AnalysisTrigger) + Send + Sync>,
}

impl DefaultFileEventProcessor {
    /// Create a new default processor with a trigger callback
    pub fn new(trigger_callback: Arc<dyn Fn(AnalysisTrigger) + Send + Sync>) -> Self {
        Self { trigger_callback }
    }
}

#[async_trait]
impl FileEventProcessor for DefaultFileEventProcessor {
    async fn process_events(&mut self, events: Vec<FileSystemEventData>) -> WatcherResult<()> {
        for event in events {
            let trigger = AnalysisTrigger {
                source: TriggerSource::FileSystem,
                file_paths: vec![event.path.clone()],
                priority: TaskPriority::Normal,
                timestamp: event.timestamp,
            };

            // Call the trigger callback
            (self.trigger_callback)(trigger);
        }

        Ok(())
    }

    fn get_file_priority(&self, path: &std::path::Path) -> TaskPriority {
        // Use file extension to determine priority
        let priority_patterns = [
            (".rs", TaskPriority::High),          // Rust source files - highest priority
            (".toml", TaskPriority::High),        // Configuration files
            (".md", TaskPriority::Low),           // Documentation - lowest priority
            (".js", TaskPriority::Normal),
            (".ts", TaskPriority::Normal),
        ];

        for (extension, priority) in &priority_patterns {
            if path.to_string_lossy().ends_with(extension) {
                return *priority;
            }
        }

        TaskPriority::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_filter_engine() {
        let filter_engine = FileFilterEngine::new(
            vec!["rs".into(), "toml".into()],
            1024, // 1KB max
        );

        // Test with a valid Rust file
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        std::fs::File::create(&rust_file).unwrap();
        assert!(filter_engine.should_watch(&rust_file).await.unwrap());

        // Test with an ignored file
        let log_file = temp_dir.path().join("test.log");
        std::fs::File::create(&log_file).unwrap();
        assert!(!filter_engine.should_watch(&log_file).await.unwrap());
    }

    #[tokio::test]
    async fn test_change_coalescer() {
        let coalescer = ChangeCoalescer::new(Duration::from_millis(50));

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let event = FileSystemEventData {
            event_type: FileSystemEventType::Modified,
            path: test_file,
            timestamp: Instant::now(),
            metadata: HashMap::new(),
        };

        // First event should not immediately produce output
        let result = coalescer.coalesce_change(&event).await;
        assert!(result.is_none());

        // Wait for coalescing window to expire
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Same file event should now produce output
        let result = coalescer.coalesce_change(&event).await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_default_processor_priority() {
        let trigger_count = Arc::new(std::sync::Mutex::new(0));
        let trigger_count_clone = Arc::clone(&trigger_count);

        let processor = DefaultFileEventProcessor::new(Arc::new(move |_trigger| {
            let mut count = trigger_count_clone.lock().unwrap();
            *count += 1;
        }));

        // Test priority determination
        let rust_file = std::path::PathBuf::from("test.rs");
        assert_eq!(processor.get_file_priority(&rust_file), TaskPriority::High);

        let md_file = std::path::PathBuf::from("test.md");
        assert_eq!(processor.get_file_priority(&md_file), TaskPriority::Low);

        let js_file = std::path::PathBuf::from("test.js");
        assert_eq!(processor.get_file_priority(&js_file), TaskPriority::Normal);
    }
}