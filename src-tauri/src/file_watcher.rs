use anyhow::Context;
use log::{debug, error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc as tokio_mpsc;
use tokio::sync::oneshot;
use tokio::time::{self, Duration};

pub struct FileWatcher {
    _watcher: Arc<Mutex<RecommendedWatcher>>,
    _stop_tx: tokio::sync::mpsc::Sender<()>,
    _stop_signal_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    event_tx: Option<tokio_mpsc::UnboundedSender<Event>>,
}

impl FileWatcher {
    pub fn new(path: PathBuf, app_handle: AppHandle) -> anyhow::Result<Self> {
        debug!("Creating file watcher for {:?}", path);
        let (tx, rx) = std::sync::mpsc::channel();
        // Use tokio's mpsc for async/await compatibility
        let (stop_tx, mut stop_rx) = tokio::sync::mpsc::channel(1);

        // Create a new watcher
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

        // Wrap the watcher in an Arc to share ownership
        let watcher = Arc::new(std::sync::Mutex::new(watcher));

        // First, let's update the watch call to work with the Arc<Mutex<...>>
        let path_clone = path.clone();
        if let Ok(mut w) = watcher.lock() {
            w.watch(&path, RecursiveMode::NonRecursive)
                .with_context(|| format!("Failed to watch file {}", path_clone.display()))?;
        }
        debug!("Successfully started watching file: {:?}", path);

        // Create a channel for file events
        let (event_tx, mut event_rx) = tokio_mpsc::unbounded_channel();

        // Spawn a task to forward events from sync to async
        let event_tx_clone = event_tx.clone();
        let path_clone = path.clone();
        std::thread::spawn(move || {
            debug!("Started file watcher thread for {:?}", path_clone);
            for event in rx {
                if let Err(e) = event_tx_clone.send(event) {
                    error!("Failed to forward file event: {}", e);
                    break;
                }
            }
            debug!("File watcher sync thread exiting for {:?}", path_clone);
        });

        // Spawn a task to handle file change events
        let app_handle_clone = app_handle.clone();
        let path_clone = path.clone();
        tauri::async_runtime::spawn(async move {
            debug!("Started file change handler for {:?}", path_clone);
            while let Some(_) = event_rx.recv().await {
                if let Err(e) = app_handle_clone.emit("file-changed", &path_clone) {
                    error!("Failed to emit file-changed event: {}", e);
                }
                // Add a small delay to prevent rapid-fire events
                time::sleep(Duration::from_millis(100)).await;
            }
            debug!("File watcher event channel closed for {:?}", path_clone);
        });

        // Create a channel for the stop signal
        let (stop_signal_tx, stop_signal_rx) = oneshot::channel();
        let stop_signal_tx = Arc::new(Mutex::new(Some(stop_signal_tx)));

        let path_clone = path.clone();
        let watcher_clone = Arc::clone(&watcher);
        let event_tx_clone = event_tx.clone();

        tauri::async_runtime::spawn(async move {
            debug!("Started stop signal handler for {:?}", path_clone);
            // Wait for either a stop signal or the oneshot channel to close
            tokio::select! {
                Some(_) = stop_rx.recv() => {
                    debug!("Received stop signal, shutting down file watcher");
                }
                _ = stop_signal_rx => {
                    debug!("Received stop signal through oneshot, shutting down file watcher");
                }
            }

            // Clean up the watcher
            if let Ok(mut w) = watcher_clone.lock() {
                if let Err(e) = w.unwatch(&path_clone) {
                    error!("Failed to unwatch file {}: {}", path_clone.display(), e);
                }
                info!("Stopped file watcher for {:?}", path_clone);
            }

            // Close the channel to signal any listeners that we're done
            drop(event_tx_clone);
        });

        let watcher = Self {
            _watcher: watcher,
            _stop_tx: stop_tx,
            _stop_signal_tx: stop_signal_tx,
            event_tx: Some(event_tx),
        };

        // Spawn initial event handler
        if let Some(tx) = watcher.event_tx.as_ref() {
            if let Err(e) = tx.send(Event::default()) {
                error!("Failed to send initial event: {}", e);
            } else {
                debug!("Sent initial event for {:?}", path);
            }
        }

        Ok(watcher)
    }

    pub fn stop(self) {
        if let Ok(mut tx) = self._stop_signal_tx.lock() {
            if let Some(tx) = tx.take() {
                if let Err(e) = tx.send(()) {
                    error!("Failed to send stop signal: {:?}", e);
                }
            }
        }
        // The _stop_tx will be dropped when self is dropped
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        debug!("Dropping FileWatcher");
        if let Some(tx) = self.event_tx.take() {
            drop(tx);
        }
    }
}

// Get file checksum
pub fn get_file_checksum(path: &PathBuf) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}
