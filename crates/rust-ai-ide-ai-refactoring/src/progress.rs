use crate::types::*;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Comprehensive progress tracking system for refactoring operations
pub struct ProgressTracker {
    /// Active operation trackers
    active_trackers: Arc<Mutex<HashMap<String, OperationTracker>>>,
    /// Completed operation history
    completed_operations: Arc<Mutex<VecDeque<CompletedOperation>>>,
    /// Progress update channel
    progress_channel: broadcast::Sender<ProgressUpdate>,
    /// Global progress stats
    global_stats: Arc<Mutex<ProgressStats>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        ProgressTracker {
            active_trackers: Arc::new(Mutex::new(HashMap::new())),
            completed_operations: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            progress_channel: tx,
            global_stats: Arc::new(Mutex::new(ProgressStats::new())),
        }
    }

    /// Subscribe to progress updates
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressUpdate> {
        self.progress_channel.subscribe()
    }

    /// Start tracking an operation
    pub fn start_operation(&self, operation_id: String, operation_type: &RefactoringType, context: &RefactoringContext) -> OperationHandle {
        let tracker = OperationTracker::new(operation_id.clone(), operation_type.clone(), context.clone());

        if let Ok(mut trackers) = self.active_trackers.lock() {
            trackers.insert(operation_id.clone(), tracker.clone());
        }

        // Update global stats
        if let Ok(mut stats) = self.global_stats.lock() {
            stats.active_operations += 1;
            stats.total_operations_started += 1;
        }

        // Send initial progress update
        let _ = self.progress_channel.send(ProgressUpdate {
            operation_id: operation_id.clone(),
            phase: ProgressPhase::Started,
            progress_percent: 0.0,
            current_step: "Initializing operation".to_string(),
            estimated_time_remaining: None,
            details: Some(format!("Starting {:?} operation", operation_type)),
        });

        OperationHandle {
            operation_id,
            progress_tracker: self.clone(),
        }
    }

    /// Update operation progress
    pub fn update_progress(&self, operation_id: &str, progress_percent: f64, current_step: String, details: Option<String>) {
        if let Ok(mut trackers) = self.active_trackers.lock() {
            if let Some(tracker) = trackers.get_mut(operation_id) {
                tracker.current_progress = progress_percent;
                tracker.current_step = current_step.clone();
                tracker.last_update = std::time::Instant::now();

                // Calculate ETA if we have enough data
                let eta = if tracker.progress_history.len() >= 2 {
                    self.calculate_eta(tracker)
                } else {
                    tracker.progress_history.push(ProgressPoint {
                        progress: progress_percent,
                        timestamp: std::time::Instant::now(),
                    });
                    None
                };

                let update = ProgressUpdate {
                    operation_id: operation_id.to_string(),
                    phase: tracker.phase.clone(),
                    progress_percent,
                    current_step,
                    estimated_time_remaining: eta,
                    details,
                };

                let _ = self.progress_channel.send(update);
            }
        }
    }

    /// Advance operation to next phase
    pub fn advance_phase(&self, operation_id: &str, new_phase: ProgressPhase, description: String) {
        if let Ok(mut trackers) = self.active_trackers.lock() {
            if let Some(tracker) = trackers.get_mut(operation_id) {
                tracker.phase = new_phase.clone();
                tracker.phase_descriptions.insert(new_phase.clone(), description.clone());

                let update = ProgressUpdate {
                    operation_id: operation_id.to_string(),
                    phase: new_phase,
                    progress_percent: tracker.current_progress,
                    current_step: description,
                    estimated_time_remaining: self.calculate_eta(tracker),
                    details: Some("Phase transition".to_string()),
                };

                let _ = self.progress_channel.send(update);
            }
        }
    }

    /// Mark operation as completed
    pub fn complete_operation(&self, operation_id: &str, success: bool, final_message: Option<String>) {
        let tracker = if let Ok(mut trackers) = self.active_trackers.lock() {
            trackers.remove(operation_id).unwrap_or_else(|| {
                // Create a minimal tracker for completed operations we missed
                OperationTracker::new(operation_id.to_string(), RefactoringType::Rename, RefactoringContext {
                    file_path: "unknown".to_string(),
                    cursor_line: 0,
                    cursor_character: 0,
                    selection: None,
                    symbol_name: None,
                    symbol_kind: None,
                })
            })
        } else {
            return;
        };

        let duration = tracker.start_time.elapsed();
        let completed_op = CompletedOperation {
            tracker,
            success,
            duration,
            final_message: final_message.clone(),
        };

        // Store in completed operations (with size limit)
        if let Ok(mut completed) = self.completed_operations.lock() {
            if completed.len() >= 1000 {
                completed.pop_front();
            }
            completed.push_back(completed_op);
        }

        // Update global stats
        if let Ok(mut stats) = self.global_stats.lock() {
            stats.active_operations = stats.active_operations.saturating_sub(1);
            if success {
                stats.successful_operations += 1;
            } else {
                stats.failed_operations += 1;
            }
            stats.average_operation_time = self.calculate_average_operation_time();
        }

        let phase = if success { ProgressPhase::Completed } else { ProgressPhase::Failed };

        let update = ProgressUpdate {
            operation_id: operation_id.to_string(),
            phase,
            progress_percent: 100.0,
            current_step: final_message.unwrap_or_else(|| format!("Operation {}", if success { "completed successfully" } else { "failed" })),
            estimated_time_remaining: None,
            details: Some(format!("Duration: {:.2}s", duration.as_secs_f64())),
        };

        let _ = self.progress_channel.send(update);
    }

    /// Cancel an operation
    pub fn cancel_operation(&self, operation_id: &str, reason: String) {
        self.complete_operation(operation_id, false, Some(format!("Operation cancelled: {}", reason)));
    }

    /// Get current progress for an operation
    pub fn get_progress(&self, operation_id: &str) -> Option<ProgressUpdate> {
        let trackers = self.active_trackers.lock().ok()?;
        let tracker = trackers.get(operation_id)?;

        Some(ProgressUpdate {
            operation_id: operation_id.to_string(),
            phase: tracker.phase.clone(),
            progress_percent: tracker.current_progress,
            current_step: tracker.current_step.clone(),
            estimated_time_remaining: self.calculate_eta(tracker),
            details: Some("Current progress".to_string()),
        })
    }

    /// Get progress summary for all active operations
    pub fn get_active_operations_summary(&self) -> ActiveOperationsSummary {
        let trackers = self.active_trackers.lock().unwrap();
        let global_stats = self.global_stats.lock().unwrap();

        let mut operations = Vec::new();
        for (id, tracker) in &*trackers {
            operations.push(OperationProgress {
                operation_id: id.clone(),
                operation_type: tracker.operation_type.clone(),
                current_progress: tracker.current_progress,
                current_phase: tracker.phase.clone(),
                current_step: tracker.current_step.clone(),
                estimated_time_remaining: self.calculate_eta(tracker),
                elapsed_time: tracker.start_time.elapsed().as_secs_f64(),
            });
        }

        ActiveOperationsSummary {
            active_operations: operations,
            global_stats: global_stats.clone(),
        }
    }

    /// Get operation history with pagination
    pub fn get_operation_history(&self, limit: usize, offset: usize) -> Vec<CompletedOperation> {
        let completed = self.completed_operations.lock().unwrap();
        completed.iter()
            .rev() // Most recent first
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get global statistics
    pub fn get_global_stats(&self) -> ProgressStats {
        self.global_stats.lock().unwrap().clone()
    }

    /// Calculate ETA based on progress history
    fn calculate_eta(&self, tracker: &OperationTracker) -> Option<f64> {
        if tracker.progress_history.len() < 3 {
            return None;
        }

        // Calculate average progress rate
        let total_duration = tracker.start_time.elapsed().as_secs_f64();
        if total_duration < 1.0 || tracker.current_progress <= 0.0 {
            return None;
        }

        let progress_rate = tracker.current_progress / total_duration;
        let remaining_progress = 100.0 - tracker.current_progress;
        let estimated_seconds = remaining_progress / progress_rate;

        // If estimated time is unreasonable (>1 hour), don't show it
        if estimated_seconds > 3600.0 {
            None
        } else {
            Some(estimated_seconds)
        }
    }

    /// Calculate average operation time from history
    fn calculate_average_operation_time(&self) -> f64 {
        let completed = self.completed_operations.lock().unwrap();
        if completed.is_empty() {
            return 0.0;
        }

        let total_time: f64 = completed.iter()
            .map(|op| op.duration.as_secs_f64())
            .sum();
        total_time / completed.len() as f64
    }

    /// Get operation tracker (internal use)
    fn clone(&self) -> ProgressTracker {
        ProgressTracker {
            active_trackers: self.active_trackers.clone(),
            completed_operations: self.completed_operations.clone(),
            progress_channel: self.progress_channel.clone(),
            global_stats: self.global_stats.clone(),
        }
    }
}

/// Handle for managing an active operation
pub struct OperationHandle {
    operation_id: String,
    progress_tracker: ProgressTracker,
}

impl OperationHandle {
    /// Update progress
    pub fn update_progress(&self, progress_percent: f64, current_step: String, details: Option<String>) {
        self.progress_tracker.update_progress(&self.operation_id, progress_percent, current_step, details);
    }

    /// Advance to next phase
    pub fn advance_phase(&self, new_phase: ProgressPhase, description: String) {
        self.progress_tracker.advance_phase(&self.operation_id, new_phase, description);
    }

    /// Complete operation
    pub fn complete(&self, success: bool, final_message: Option<String>) {
        self.progress_tracker.complete_operation(&self.operation_id, success, final_message);
    }

    /// Cancel operation
    pub fn cancel(&self, reason: String) {
        self.progress_tracker.cancel_operation(&self.operation_id, reason);
    }
}

/// Individual operation tracker
#[derive(Debug, Clone)]
pub struct OperationTracker {
    pub operation_id: String,
    pub operation_type: RefactoringType,
    pub context: RefactoringContext,
    pub start_time: std::time::Instant,
    pub last_update: std::time::Instant,
    pub phase: ProgressPhase,
    pub current_progress: f64,
    pub current_step: String,
    pub progress_history: Vec<ProgressPoint>,
    pub phase_descriptions: HashMap<ProgressPhase, String>,
}

impl OperationTracker {
    fn new(operation_id: String, operation_type: RefactoringType, context: RefactoringContext) -> Self {
        OperationTracker {
            operation_id,
            operation_type,
            context,
            start_time: std::time::Instant::now(),
            last_update: std::time::Instant::now(),
            phase: ProgressPhase::Started,
            current_progress: 0.0,
            current_step: "Initializing".to_string(),
            progress_history: Vec::new(),
            phase_descriptions: HashMap::new(),
        }
    }
}

/// Progress point in history
#[derive(Debug, Clone)]
pub struct ProgressPoint {
    pub progress: f64,
    pub timestamp: std::time::Instant,
}

/// Progress phases
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ProgressPhase {
    Started,
    SafetyCheck,
    Analysis,
    Transformation,
    Validation,
    Backup,
    Recovery,
    Completed,
    Failed,
}

/// Progress update event
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub operation_id: String,
    pub phase: ProgressPhase,
    pub progress_percent: f64,
    pub current_step: String,
    pub estimated_time_remaining: Option<f64>,
    pub details: Option<String>,
}

/// Completed operation record
#[derive(Debug, Clone)]
pub struct CompletedOperation {
    pub tracker: OperationTracker,
    pub success: bool,
    pub duration: std::time::Duration,
    pub final_message: Option<String>,
}

/// Progress point in history
#[derive(Debug, Clone)]
pub struct OperationProgress {
    pub operation_id: String,
    pub operation_type: RefactoringType,
    pub current_progress: f64,
    pub current_phase: ProgressPhase,
    pub current_step: String,
    pub estimated_time_remaining: Option<f64>,
    pub elapsed_time: f64,
}

/// Summary of all active operations
#[derive(Debug, Clone)]
pub struct ActiveOperationsSummary {
    pub active_operations: Vec<OperationProgress>,
    pub global_stats: ProgressStats,
}

/// Global progress statistics
#[derive(Debug, Clone)]
pub struct ProgressStats {
    pub total_operations_started: usize,
    pub active_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub average_operation_time: f64,
    pub created_at: std::time::Instant,
}

impl ProgressStats {
    fn new() -> Self {
        ProgressStats {
            total_operations_started: 0,
            active_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            average_operation_time: 0.0,
            created_at: std::time::Instant::now(),
        }
    }
}

/// Batch operation progress tracker
pub struct BatchProgressTracker {
    context: String,
    total_operations: usize,
    completed_operations: usize,
    successful_operations: usize,
    failed_operations: usize,
    start_time: std::time::Instant,
}

impl BatchProgressTracker {
    pub fn new(context: String, total_operations: usize) -> Self {
        BatchProgressTracker {
            context,
            total_operations,
            completed_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn operation_completed(&mut self, success: bool) {
        self.completed_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }
    }

    pub fn get_progress(&self) -> f64 {
        if self.total_operations == 0 {
            return 100.0;
        }
        (self.completed_operations as f64 / self.total_operations as f64) * 100.0
    }

    pub fn remaining_operations(&self) -> usize {
        self.total_operations.saturating_sub(self.completed_operations)
    }

    pub fn estimated_time_remaining(&self) -> Option<f64> {
        if self.completed_operations == 0 {
            return None;
        }

        let elapsed = self.start_time.elapsed().as_secs_f64();
        let operations_per_second = self.completed_operations as f64 / elapsed;
        let remaining_time = self.remaining_operations() as f64 / operations_per_second;
        Some(remaining_time)
    }

    pub fn is_complete(&self) -> bool {
        self.completed_operations >= self.total_operations
    }

    pub fn success_rate(&self) -> f64 {
        if self.completed_operations == 0 {
            return 0.0;
        }
        self.successful_operations as f64 / self.completed_operations as f64
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}