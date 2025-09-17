//! Compaction scheduler for optimal timing
//!
//! This module provides intelligent scheduling for memory compaction operations,
//! determining the best times to perform compaction based on system state and patterns.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use crate::InfraResult;

/// Intelligent scheduler for compaction operations
#[derive(Debug)]
pub struct CompactionScheduler {
    /// Scheduler configuration
    config: SchedulerConfig,

    /// Scheduler state
    state: Arc<RwLock<SchedulerState>>,

    /// Scheduled compaction queue
    queue: Arc<RwLock<Vec<ScheduledCompaction>>>,

    /// Notification channel for scheduling decisions
    notification_tx: mpsc::UnboundedSender<SchedulerNotification>,
}

/// Configuration for the compaction scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Base scheduling interval
    pub base_interval: Duration,

    /// Maximum queue size
    pub max_queue_size: usize,

    /// Priority threshold for immediate scheduling
    pub priority_threshold: f64,

    /// Low activity period detection (hours)
    pub low_activity_window: u64,

    /// High activity period detection (hours)
    pub high_activity_window: u64,

    /// Adaptive scheduling enabled
    pub adaptive_scheduling: bool,

    /// Emergency scheduling enabled
    pub emergency_scheduling: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            base_interval: Duration::from_secs(300), // 5 minutes
            max_queue_size: 10,
            priority_threshold: 0.8,
            low_activity_window: 2,  // 2 hours
            high_activity_window: 1, // 1 hour
            adaptive_scheduling: true,
            emergency_scheduling: true,
        }
    }
}

/// Internal state of the scheduler
#[derive(Debug)]
struct SchedulerState {
    /// Next scheduled compaction time
    next_compaction: Option<Instant>,

    /// Last compaction execution time
    last_execution: Option<Instant>,

    /// Current scheduling mode
    mode: SchedulingMode,

    /// Activity pattern analysis
    activity_pattern: ActivityPattern,

    /// Scheduling cycle count
    cycle_count: usize,

    /// Emergency mode active
    emergency_mode: bool,

    /// Scheduler enabled
    enabled: bool,
}

/// Scheduled compaction operation
#[derive(Debug, Clone)]
struct ScheduledCompaction {
    /// Scheduled execution time
    scheduled_time: Instant,

    /// Priority level (0.0-1.0)
    priority: f64,

    /// Compaction strategy to use
    strategy: super::large_workspace_compactor::CompactionStrategy,

    /// Reason for scheduling
    reason: SchedulingReason,

    /// Estimated duration
    estimated_duration: Duration,

    /// System state at scheduling time
    system_state: SystemStateSnapshot,
}

/// Activity pattern analysis
#[derive(Debug, Clone)]
struct ActivityPattern {
    /// Peak activity hours
    peak_hours: Vec<u32>,

    /// Low activity hours
    low_activity_hours: Vec<u32>,

    /// Average activity level by hour
    hourly_activity: [f64; 24],

    /// Pattern confidence level
    confidence: f64,
}

/// System state snapshot for scheduling decisions
#[derive(Debug, Clone)]
struct SystemStateSnapshot {
    /// CPU usage
    cpu_usage: f64,

    /// Memory pressure
    memory_pressure: f64,

    /// Fragmentation ratio
    fragmentation_ratio: f64,

    /// Active process count
    active_processes: usize,

    /// I/O activity level
    io_activity: f64,
}

/// Scheduling mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingMode {
    /// Fixed interval scheduling
    Fixed,

    /// Adaptive scheduling based on patterns
    Adaptive,

    /// Conservative scheduling for stability
    Conservative,

    /// Aggressive scheduling for performance
    Aggressive,

    /// Emergency scheduling for critical situations
    Emergency,
}

/// Reason for scheduling a compaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingReason {
    /// Regular interval
    RegularInterval,

    /// High fragmentation detected
    HighFragmentation,

    /// Memory pressure threshold
    MemoryPressure,

    /// Low system activity
    LowActivity,

    /// Emergency situation
    Emergency,

    /// User requested
    UserRequested,

    /// Pattern-based optimal timing
    OptimalTiming,
}

/// Scheduler notification for external systems
#[derive(Debug, Clone)]
pub enum SchedulerNotification {
    /// Compaction scheduled
    CompactionScheduled {
        scheduled_time: Instant,
        priority: f64,
        reason: SchedulingReason,
    },

    /// Compaction cancelled
    CompactionCancelled {
        reason: String,
    },

    /// Scheduling mode changed
    ModeChanged {
        old_mode: SchedulingMode,
        new_mode: SchedulingMode,
        reason: String,
    },

    /// Emergency mode activated
    EmergencyActivated {
        trigger_reason: String,
    },
}

/// Status information for the scheduler
#[derive(Debug, Clone)]
pub struct SchedulerStatus {
    /// Next scheduled compaction
    pub next_compaction: Option<Instant>,

    /// Current scheduling mode
    pub mode: SchedulingMode,

    /// Queue size
    pub queue_size: usize,

    /// Emergency mode active
    pub emergency_mode: bool,

    /// Scheduler enabled
    pub enabled: bool,

    /// Activity pattern confidence
    pub pattern_confidence: f64,

    /// Scheduling cycle count
    pub cycle_count: usize,
}

impl CompactionScheduler {
    /// Create a new compaction scheduler
    pub fn new() -> Self {
        let (tx, _) = mpsc::unbounded_channel();

        Self {
            config: SchedulerConfig::default(),
            state: Arc::new(RwLock::new(SchedulerState {
                next_compaction: None,
                last_execution: None,
                mode: SchedulingMode::Adaptive,
                activity_pattern: ActivityPattern {
                    peak_hours: Vec::new(),
                    low_activity_hours: Vec::new(),
                    hourly_activity: [0.0; 24],
                    confidence: 0.0,
                },
                cycle_count: 0,
                emergency_mode: false,
                enabled: true,
            })),
            queue: Arc::new(RwLock::new(Vec::new())),
            notification_tx: tx,
        }
    }

    /// Schedule a compaction operation
    pub async fn schedule_compaction(
        &self,
        priority: f64,
        strategy: super::large_workspace_compactor::CompactionStrategy,
        reason: SchedulingReason,
        system_state: SystemStateSnapshot,
    ) -> InfraResult<Instant> {
        let mut state = self.state.read().await;

        // Check if emergency scheduling is needed
        if self.config.emergency_scheduling && self.should_schedule_emergency(&system_state).await {
            let scheduled_time = Instant::now() + Duration::from_secs(1); // Schedule immediately
            drop(state);

            self.add_to_queue(ScheduledCompaction {
                scheduled_time,
                priority: 1.0,
                strategy,
                reason: SchedulingReason::Emergency,
                estimated_duration: Duration::from_millis(500),
                system_state,
            }).await;

            self.notify_scheduler(SchedulerNotification::EmergencyActivated {
                trigger_reason: "Emergency compaction triggered".to_string(),
            }).await;

            return Ok(scheduled_time);
        }

        // Calculate optimal scheduling time
        let scheduled_time = self.calculate_optimal_time(priority, &reason, &system_state).await;

        drop(state);

        // Add to queue
        self.add_to_queue(ScheduledCompaction {
            scheduled_time,
            priority,
            strategy,
            reason,
            estimated_duration: self.estimate_duration(&strategy, priority).await,
            system_state,
        }).await;

        // Send notification
        self.notify_scheduler(SchedulerNotification::CompactionScheduled {
            scheduled_time,
            priority,
            reason,
        }).await;

        Ok(scheduled_time)
    }

    /// Check if compaction should be scheduled
    pub async fn should_schedule_compaction(&self) -> bool {
        let state = self.state.read().await;
        let queue = self.queue.read().await;

        // Check emergency conditions
        if state.emergency_mode {
            return true;
        }

        // Check queue size
        if queue.len() >= self.config.max_queue_size {
            return false; // Queue is full
        }

        // Check timing
        if let Some(next_compaction) = state.next_compaction {
            if Instant::now() >= next_compaction {
                return true;
            }
        }

        false
    }

    /// Get next scheduled compaction
    pub async fn get_next_scheduled(&self) -> Option<ScheduledCompaction> {
        let queue = self.queue.read().await;
        queue.first().cloned()
    }

    /// Cancel scheduled compaction
    pub async fn cancel_scheduled(&self, reason: String) -> InfraResult<()> {
        let mut queue = self.queue.write().await;

        if !queue.is_empty() {
            queue.remove(0); // Remove next scheduled

            self.notify_scheduler(SchedulerNotification::CompactionCancelled {
                reason,
            }).await;
        }

        Ok(())
    }

    /// Update activity pattern with new data
    pub async fn update_activity_pattern(&self, hour: u32, activity_level: f64) {
        let mut state = self.state.write().await;

        if hour < 24 {
            state.activity_pattern.hourly_activity[hour as usize] = activity_level;
        }

        // Recalculate patterns
        self.recalculate_patterns(&mut state).await;
    }

    /// Force emergency scheduling
    pub async fn force_emergency(&self, reason: String) -> InfraResult<()> {
        let mut state = self.state.write().await;
        state.emergency_mode = true;
        state.mode = SchedulingMode::Emergency;

        self.notify_scheduler(SchedulerNotification::EmergencyActivated {
            trigger_reason: reason,
        }).await;

        Ok(())
    }

    /// Get scheduler status
    pub async fn get_status(&self) -> SchedulerStatus {
        let state = self.state.read().await;
        let queue = self.queue.read().await;

        SchedulerStatus {
            next_compaction: state.next_compaction,
            mode: state.mode,
            queue_size: queue.len(),
            emergency_mode: state.emergency_mode,
            enabled: state.enabled,
            pattern_confidence: state.activity_pattern.confidence,
            cycle_count: state.cycle_count,
        }
    }

    /// Add compaction to queue
    async fn add_to_queue(&self, compaction: ScheduledCompaction) {
        let mut queue = self.queue.write().await;

        // Insert in priority order (highest priority first)
        let insert_pos = queue.partition_point(|c| c.priority >= compaction.priority);
        queue.insert(insert_pos, compaction);

        // Maintain queue size limit
        if queue.len() > self.config.max_queue_size {
            queue.pop(); // Remove lowest priority
        }

        // Update next compaction time
        if let Some(first) = queue.first() {
            let mut state = self.state.write().await;
            state.next_compaction = Some(first.scheduled_time);
        }
    }

    /// Calculate optimal scheduling time
    async fn calculate_optimal_time(
        &self,
        priority: f64,
        reason: &SchedulingReason,
        system_state: &SystemStateSnapshot,
    ) -> Instant {
        let now = Instant::now();

        // Emergency scheduling
        if matches!(reason, SchedulingReason::Emergency) {
            return now + Duration::from_secs(1);
        }

        // High priority scheduling
        if priority >= self.config.priority_threshold {
            return now + Duration::from_secs(30);
        }

        // Pattern-based optimal timing
        if self.config.adaptive_scheduling {
            if let Some(optimal_time) = self.find_optimal_window(system_state).await {
                return optimal_time;
            }
        }

        // Default scheduling
        now + self.config.base_interval
    }

    /// Find optimal scheduling window based on activity patterns
    async fn find_optimal_window(&self, system_state: &SystemStateSnapshot) -> Option<Instant> {
        let state = self.state.read().await;
        let current_hour = self.get_current_hour();

        // Find lowest activity hour
        let mut min_activity = f64::INFINITY;
        let mut optimal_hour = current_hour;

        for (hour, &activity) in state.activity_pattern.hourly_activity.iter().enumerate() {
            if activity < min_activity {
                min_activity = activity;
                optimal_hour = hour as u32;
            }
        }

        // If optimal hour is in the future, schedule for then
        if optimal_hour > current_hour {
            let hours_diff = optimal_hour - current_hour;
            return Some(Instant::now() + Duration::from_secs(hours_diff as u64 * 3600));
        }

        // If optimal hour is in the past, schedule for next occurrence
        if optimal_hour < current_hour {
            let hours_until_next = (24 - current_hour) + optimal_hour;
            return Some(Instant::now() + Duration::from_secs(hours_until_next as u64 * 3600));
        }

        None
    }

    /// Check if emergency scheduling is needed
    async fn should_schedule_emergency(&self, system_state: &SystemStateSnapshot) -> bool {
        system_state.memory_pressure > 0.9
            || system_state.fragmentation_ratio > 0.8
            || system_state.cpu_usage > 0.95
    }

    /// Estimate compaction duration
    async fn estimate_duration(
        &self,
        strategy: &super::large_workspace_compactor::CompactionStrategy,
        priority: f64,
    ) -> Duration {
        let base_duration = match strategy {
            super::large_workspace_compactor::CompactionStrategy::Incremental => Duration::from_millis(50),
            super::large_workspace_compactor::CompactionStrategy::Conservative => Duration::from_millis(25),
            super::large_workspace_compactor::CompactionStrategy::Aggressive => Duration::from_millis(200),
            super::large_workspace_compactor::CompactionStrategy::Emergency => Duration::from_millis(500),
            super::large_workspace_compactor::CompactionStrategy::VirtualMemory => Duration::from_millis(100),
            super::large_workspace_compactor::CompactionStrategy::LargeScale => Duration::from_millis(1000),
        };

        // Adjust based on priority (higher priority = longer allowed duration)
        let adjustment = 1.0 + (priority * 2.0);
        Duration::from_millis((base_duration.as_millis() as f64 * adjustment) as u64)
    }

    /// Recalculate activity patterns
    async fn recalculate_patterns(&self, state: &mut SchedulerState) {
        // Find peak hours (top 25% activity)
        let mut activities: Vec<(usize, f64)> = state.activity_pattern.hourly_activity
            .iter()
            .enumerate()
            .map(|(i, &a)| (i, a))
            .collect();

        activities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let peak_threshold = activities.len() / 4;
        state.activity_pattern.peak_hours = activities.iter()
            .take(peak_threshold)
            .map(|(hour, _)| *hour as u32)
            .collect();

        // Find low activity hours (bottom 25% activity)
        activities.reverse();
        state.activity_pattern.low_activity_hours = activities.iter()
            .take(peak_threshold)
            .map(|(hour, _)| *hour as u32)
            .collect();

        // Calculate confidence based on activity variation
        let mean = state.activity_pattern.hourly_activity.iter().sum::<f64>() / 24.0;
        let variance = state.activity_pattern.hourly_activity.iter()
            .map(|a| (a - mean).powi(2))
            .sum::<f64>() / 24.0;

        state.activity_pattern.confidence = 1.0 - (variance.sqrt() / mean).min(1.0);
    }

    /// Get current hour (0-23)
    fn get_current_hour(&self) -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let hours_since_epoch = now.as_secs() / 3600;
        (hours_since_epoch % 24) as u32
    }

    /// Send scheduler notification
    async fn notify_scheduler(&self, notification: SchedulerNotification) {
        let _ = self.notification_tx.send(notification);
    }

    /// Export scheduler status for monitoring
    pub async fn export_status(&self) -> serde_json::Value {
        let status = self.get_status().await;
        let state = self.state.read().await;
        let queue = self.queue.read().await;

        serde_json::json!({
            "scheduler": {
                "next_compaction_seconds": status.next_compaction.map(|t| t.elapsed().as_secs()).unwrap_or(0),
                "mode": format!("{:?}", status.mode),
                "queue_size": status.queue_size,
                "emergency_mode": status.emergency_mode,
                "enabled": status.enabled,
                "pattern_confidence": status.pattern_confidence,
                "cycle_count": status.cycle_count
            },
            "activity_pattern": {
                "peak_hours": state.activity_pattern.peak_hours,
                "low_activity_hours": state.activity_pattern.low_activity_hours,
                "hourly_activity": state.activity_pattern.hourly_activity,
                "confidence": state.activity_pattern.confidence
            },
            "queue": queue.iter().take(3).map(|c| {
                serde_json::json!({
                    "scheduled_in_seconds": c.scheduled_time.saturating_duration_since(Instant::now()).as_secs(),
                    "priority": c.priority,
                    "strategy": format!("{:?}", c.strategy),
                    "reason": format!("{:?}", c.reason),
                    "estimated_duration_ms": c.estimated_duration.as_millis()
                })
            }).collect::<Vec<_>>(),
            "config": {
                "base_interval_seconds": self.config.base_interval.as_secs(),
                "max_queue_size": self.config.max_queue_size,
                "priority_threshold": self.config.priority_threshold,
                "adaptive_scheduling": self.config.adaptive_scheduling
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = CompactionScheduler::new();
        let status = scheduler.get_status().await;

        assert!(status.enabled);
        assert!(!status.emergency_mode);
        assert_eq!(status.queue_size, 0);
    }

    #[tokio::test]
    async fn test_compaction_scheduling() {
        let scheduler = CompactionScheduler::new();

        let system_state = SystemStateSnapshot {
            cpu_usage: 0.5,
            memory_pressure: 0.6,
            fragmentation_ratio: 0.4,
            active_processes: 10,
            io_activity: 0.3,
        };

        let scheduled_time = scheduler.schedule_compaction(
            0.7,
            super::large_workspace_compactor::CompactionStrategy::Incremental,
            SchedulingReason::RegularInterval,
            system_state,
        ).await.unwrap();

        assert!(scheduled_time > Instant::now());

        let status = scheduler.get_status().await;
        assert_eq!(status.queue_size, 1);
    }

    #[tokio::test]
    async fn test_emergency_scheduling() {
        let scheduler = CompactionScheduler::new();

        let emergency_state = SystemStateSnapshot {
            cpu_usage: 0.5,
            memory_pressure: 0.95, // Emergency level
            fragmentation_ratio: 0.4,
            active_processes: 10,
            io_activity: 0.3,
        };

        let scheduled_time = scheduler.schedule_compaction(
            0.9,
            super::large_workspace_compactor::CompactionStrategy::Emergency,
            SchedulingReason::Emergency,
            emergency_state,
        ).await.unwrap();

        // Should be scheduled very soon (within 1 second)
        assert!(scheduled_time < Instant::now() + Duration::from_secs(2));

        let status = scheduler.get_status().await;
        assert_eq!(status.mode, SchedulingMode::Emergency);
    }
}