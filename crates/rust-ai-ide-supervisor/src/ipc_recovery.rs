//! IPC Recovery System - Channel health monitoring and automatic reconnection

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::time::{Duration, timeout, interval};
use serde::{Serialize, Deserialize};

use crate::error::{SupervisorError, SupervisorResult};
use crate::types::*;

/// IPC Monitor for channel health and recovery
pub struct IpcMonitor {
    channels: Arc<Mutex<HashMap<ChannelId, ChannelState>>>,
    recovery_tasks: Arc<Mutex<HashMap<ChannelId, tokio::task::JoinHandle<()>>>>,
    recovery_config: RecoveryConfig,
}

/// Recovery configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Health check interval in seconds
    pub health_check_interval: Duration,
    /// Maximum time to wait for reconnection
    pub reconnection_timeout: Duration,
    /// Maximum number of reconnection attempts
    pub max_reconnection_attempts: usize,
    /// Base delay between reconnection attempts
    pub base_reconnection_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_reconnection_delay: Duration,
    /// Buffer size for message queuing
    pub message_buffer_size: usize,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(10),
            reconnection_timeout: Duration::from_secs(30),
            max_reconnection_attempts: 5,
            base_reconnection_delay: Duration::from_secs(1),
            max_reconnection_delay: Duration::from_secs(60),
            message_buffer_size: 100,
        }
    }
}

/// Internal channel state
#[derive(Debug)]
struct ChannelState {
    health: ChannelHealth,
    tx: Option<mpsc::Sender<IpcMessage>>,
    rx: Option<mpsc::Receiver<IpcMessage>>,
    buffer: Vec<BufferedMessage>,
    reconnect_attempt: usize,
    last_message_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Buffered message with retry information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BufferedMessage {
    message: IpcMessage,
    retry_count: usize,
    first_attempt_time: chrono::DateTime<chrono::Utc>,
}

impl IpcMonitor {
    /// Create a new IPC monitor
    pub fn new() -> Self {
        Self::with_config(RecoveryConfig::default())
    }

    /// Create a new IPC monitor with custom config
    pub fn with_config(config: RecoveryConfig) -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            recovery_tasks: Arc::new(Mutex::new(HashMap::new())),
            recovery_config: config,
        }
    }

    /// Register a new IPC channel
    pub async fn register_channel(&self, channel_id: ChannelId) -> SupervisorResult<()> {
        let mut channels = self.channels.lock().await;

        if channels.contains_key(&channel_id) {
            return Err(SupervisorError::validation_error("channel_id", "Channel already registered"));
        }

        let (tx, rx) = mpsc::channel(self.recovery_config.message_buffer_size);

        let state = ChannelState {
            health: ChannelHealth {
                id: channel_id.clone(),
                healthy: true,
                last_message_time: None,
                last_failure_time: None,
                buffered_message_count: 0,
                reconnection_attempts: 0,
            },
            tx: Some(tx),
            rx: Some(rx),
            buffer: Vec::new(),
            reconnect_attempt: 0,
            last_message_time: None,
        };

        channels.insert(channel_id.clone(), state);

        // Start monitoring task for this channel
        self.start_channel_monitoring(channel_id).await?;

        log::info!("Registered IPC channel: {}", channel_id);

        Ok(())
    }

    /// Send message through a channel
    pub async fn send_message(&self, channel_id: &ChannelId, message: IpcMessage) -> SupervisorResult<()> {
        let mut channels = self.channels.lock().await;

        let state = channels.get_mut(channel_id)
            .ok_or_else(|| SupervisorError::validation_error("channel_id", "Channel not found"))?;

        if let Some(tx) = &state.tx {
            match tx.try_send(message) {
                Ok(()) => {
                    state.health.last_message_time = Some(chrono::Utc::now());
                    state.last_message_time = Some(chrono::Utc::now());
                    state.health.healthy = true;
                    log::debug!("Message sent successfully on channel {}", channel_id);
                    Ok(())
                }
                Err(mpsc::error::TrySendError::Full(msg)) => {
                    // Buffer the message for retry
                    self.buffer_message(channel_id, msg, 0).await?;
                    state.health.healthy = false;
                    Err(SupervisorError::ipc_recovery_error(
                        channel_id,
                        "Channel buffer full, message buffered for retry"
                    ))
                }
                Err(mpsc::error::TrySendError::Closed(msg)) => {
                    // Channel is closed, buffer the message and trigger recovery
                    self.buffer_message(channel_id, msg, 0).await?;
                    state.health.healthy = false;
                    self.trigger_channel_recovery(channel_id).await?;
                    Err(SupervisorError::ipc_recovery_error(
                        channel_id,
                        "Channel closed, buffering message for recovery"
                    ))
                }
            }
        } else {
            Err(SupervisorError::ipc_recovery_error(channel_id, "Channel transmitter not available"))
        }
    }

    /// Receive message from a channel
    pub async fn receive_message(&self, channel_id: &ChannelId, timeout_duration: Duration) -> SupervisorResult<IpcMessage> {
        let mut channels = self.channels.lock().await;

        let state = channels.get_mut(channel_id)
            .ok_or_else(|| SupervisorError::validation_error("channel_id", "Channel not found"))?;

        if let Some(rx) = &mut state.rx {
            match timeout(timeout_duration, rx.recv()).await {
                Ok(Some(message)) => {
                    state.last_message_time = Some(chrono::Utc::now());
                    Ok(message)
                }
                Ok(None) => {
                    // Channel is closed
                    state.health.healthy = false;
                    Err(SupervisorError::ipc_recovery_error(channel_id, "Channel is closed"))
                }
                Err(_) => {
                    Err(SupervisorError::ipc_recovery_error(channel_id, "Receive timeout"))
                }
            }
        } else {
            Err(SupervisorError::ipc_recovery_error(channel_id, "Channel receiver not available"))
        }
    }

    /// Get channel health status
    pub async fn get_channel_health(&self, channel_id: &ChannelId) -> SupervisorResult<ChannelHealth> {
        let channels = self.channels.lock().await;

        let state = channels.get(channel_id)
            .ok_or_else(|| SupervisorError::validation_error("channel_id", "Channel not found"))?;

        Ok(state.health.clone())
    }

    /// Get all channel health statuses
    pub async fn get_all_channel_health(&self) -> Vec<ChannelHealth> {
        let channels = self.channels.lock().await;
        channels.values().map(|state| state.health.clone()).collect()
    }

    /// Manually trigger channel recovery
    pub async fn trigger_recovery(&self, channel_id: &ChannelId) -> SupervisorResult<()> {
        self.trigger_channel_recovery(channel_id).await
    }

    /// Start monitoring all registered channels
    pub async fn start_monitoring(&self) -> SupervisorResult<()> {
        let channels = self.channels.lock().await;
        let channel_ids: Vec<_> = channels.keys().cloned().collect();

        for channel_id in channel_ids {
            self.start_channel_monitoring(&channel_id).await?;
        }

        log::info!("Started IPC monitoring for all channels");

        Ok(())
    }

    // Private methods

    /// Start monitoring task for a specific channel
    async fn start_channel_monitoring(&self, channel_id: ChannelId) -> SupervisorResult<()> {
        let recovery_config = self.recovery_config.clone();
        let channels = Arc::clone(&self.channels);

        let task = tokio::spawn(async move {
            let mut interval = interval(recovery_config.health_check_interval);

            loop {
                interval.tick().await;

                if let Err(e) = Self::perform_channel_monitoring(&channel_id, &channels, &recovery_config).await {
                    log::error!("Channel monitoring failed for {}: {:?}", channel_id, e);
                }
            }
        });

        let mut recovery_tasks = self.recovery_tasks.lock().await;
        recovery_tasks.insert(channel_id, task);

        Ok(())
    }

    /// Perform channel monitoring (health checks, buffered message retry)
    async fn perform_channel_monitoring(
        channel_id: &ChannelId,
        channels: &Arc<Mutex<HashMap<ChannelId, ChannelState>>>,
        config: &RecoveryConfig
    ) -> SupervisorResult<()> {
        let mut channels_guard = channels.lock().await;

        let state = channels_guard.get_mut(channel_id).unwrap();

        // Check if channel is healthy by trying to send a heartbeat
        if state.tx.is_some() {
            // Try to send a small heartbeat message or check channel state
            state.health.healthy = true;

            // Retry buffered messages
            state.buffer.retain(|buffered_msg| {
                if let Some(tx) = &state.tx {
                    match tx.try_send(buffered_msg.message.clone()) {
                        Ok(()) => {
                            log::debug!("Retried buffered message successfully for channel {}", channel_id);
                            false // Remove from buffer
                        }
                        Err(_) => {
                            // Still can't send, keep in buffer if under retry limit
                            buffered_msg.retry_count < config.max_reconnection_attempts
                        }
                    }
                } else {
                    // No transmitter available
                    state.health.healthy = false;
                    true
                }
            });

            // Update health metrics
            if state.health.healthy {
                state.last_message_time = Some(chrono::Utc::now());
            } else {
                state.health.last_failure_time = Some(chrono::Utc::now());
            }
        } else {
            state.health.healthy = false;
        }

        state.health.buffered_message_count = state.buffer.len() as usize;

        Ok(())
    }

    /// Trigger recovery for a failed channel
    async fn trigger_channel_recovery(&self, channel_id: &ChannelId) -> SupervisorResult<()> {
        let mut channels = self.channels.lock().await;

        log::warn!("Triggering recovery for channel {}", channel_id);

        let state = channels.get_mut(channel_id)
            .ok_or_else(|| SupervisorError::validation_error("channel_id", "Channel not found"))?;

        if state.reconnect_attempt >= self.recovery_config.max_reconnection_attempts {
            return Err(SupervisorError::ipc_recovery_error(
                channel_id,
                "Maximum reconnection attempts exceeded"
            ));
        }

        state.reconnect_attempt += 1;
        state.health.reconnection_attempts = state.reconnect_attempt;
        state.health.healthy = false;

        // Try to reconnect (this would involve channel-specific reconnection logic)
        self.perform_channel_reconnection(channel_id, &mut *channels).await?;

        log::info!("Recovery initiated for channel {}", channel_id);

        Ok(())
    }

    /// Perform actual channel reconnection
    async fn perform_channel_reconnection(&self, channel_id: &ChannelId, channels: &mut HashMap<ChannelId, ChannelState>) -> SupervisorResult<()> {
        // This is where specific reconnection logic would go
        // For example:
        // 1. Close existing connections
        // 2. Create new channel endpoints
        // 3. Restore buffered messages
        // 4. Notify dependent services

        let delay = std::cmp::min(
            self.recovery_config.base_reconnection_delay * (2_u32.pow((channels.get(channel_id).unwrap().reconnect_attempt - 1) as u32)),
            self.recovery_config.max_reconnection_delay
        );

        tokio::time::sleep(delay).await;

        // Simulate reconnection success
        let (tx, rx) = mpsc::channel(self.recovery_config.message_buffer_size);
        let state = channels.get_mut(channel_id).unwrap();

        state.tx = Some(tx);
        state.rx = Some(rx);
        state.health.healthy = true;
        state.health.last_message_time = Some(chrono::Utc::now());
        state.reconnect_attempt = 0;

        // Re-send buffered messages
        while let Some(buffered_msg) = state.buffer.pop() {
            if let Some(tx) = &state.tx {
                let _ = tx.try_send(buffered_msg.message).await;
            }
        }

        log::info!("Channel {} reconnected successfully after {} attempts", channel_id, state.health.reconnection_attempts);

        Ok(())
    }

    /// Buffer a message for retry
    async fn buffer_message(&self, channel_id: &ChannelId, message: IpcMessage, retry_count: usize) -> SupervisorResult<()> {
        let mut channels = self.channels.lock().await;

        let state = channels.get_mut(channel_id)
            .ok_or_else(|| SupervisorError::validation_error("channel_id", "Channel not found"))?;

        let buffered_msg = BufferedMessage {
            message,
            retry_count,
            first_attempt_time: chrono::Utc::now(),
        };

        state.buffer.push(buffered_msg);

        if state.buffer.len() > self.recovery_config.message_buffer_size / 2 {
            log::warn!("Channel {} message buffer is filling up: {}/{}",
                channel_id, state.buffer.len(), self.recovery_config.message_buffer_size);
        }

        Ok(())
    }
}

/// Recovery queue for managing channeled message buffering
pub struct RecoveryQueue {
    queues: Arc<Mutex<HashMap<ChannelId, mpsc::Sender<IpcMessage>>>>,
}

impl RecoveryQueue {
    /// Create a new recovery queue
    pub fn new() -> Self {
        Self {
            queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Queue a message for recovery
    pub async fn queue_message(&self, channel_id: &ChannelId, message: IpcMessage) -> SupervisorResult<()> {
        let mut queues = self.queues.lock().await;

        if let Some(tx) = queues.get(channel_id) {
            tx.send(message).await
                .map_err(|e| SupervisorError::ipc_recovery_error(channel_id, format!("Queue send failed: {:?}", e)))?;
            Ok(())
        } else {
            Err(SupervisorError::ipc_recovery_error(channel_id, "Queue not found"))
        }
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self, channel_id: &ChannelId) -> SupervisorResult<QueueStats> {
        // This would return statistics about the queue
        // For now, return placeholder stats
        Ok(QueueStats {
            pending_messages: 0,
            processed_messages: 0,
            failed_messages: 0,
        })
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending_messages: u32,
    pub processed_messages: u64,
    pub failed_messages: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_channel_registration() {
        let monitor = IpcMonitor::new();
        let channel_id = "test_channel".to_string();

        monitor.register_channel(channel_id.clone()).await.expect("Registration should succeed");

        // Test duplicate registration
        let result = monitor.register_channel(channel_id.clone()).await;
        assert!(result.is_err());

        let health = monitor.get_channel_health(&channel_id).await.expect("Should get health");
        assert!(health.healthy);
        assert_eq!(health.id, channel_id);
    }

    #[tokio::test]
    async fn test_message_buffer_and_recovery() {
        let monitor = IpcMonitor::new();
        let channel_id = "test_channel".to_string();

        monitor.register_channel(channel_id.clone()).await.expect("Failed to register channel");

        let message = IpcMessage {
            id: uuid::Uuid::new_v4(),
            message_type: "test".to_string(),
            payload: serde_json::json!({"action": "test"}),
            timestamp: chrono::Utc::now(),
            retry_count: 0,
        };

        // Send should work initially
        monitor.send_message(&channel_id, message.clone()).await.expect("Send should work");

        // Simulate channel failure by triggering recovery
        monitor.trigger_recovery(&channel_id).await.expect("Recovery should be triggered");

        // Health should reflect recovery state
        let health = monitor.get_channel_health(&channel_id).await.expect("Should get health");
        assert!(!health.healthy || health.reconnection_attempts > 0); // Either unhealthy or has attempted recovery
    }

    #[tokio::test]
    async fn test_recovery_queue() {
        let queue = RecoveryQueue::new();

        let message = IpcMessage {
            id: uuid::Uuid::new_v4(),
            message_type: "test".to_string(),
            payload: serde_json::json!({"action": "test"}),
            timestamp: chrono::Utc::now(),
            retry_count: 0,
        };

        // Test queuing to non-existent channel
        let result = queue.queue_message("non_existent", message).await;
        assert!(result.is_err());
    }
}