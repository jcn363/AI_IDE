use crate::types::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, error, debug, trace};

/// Comprehensive logging system for refactoring operations
pub struct RefactoringLogger {
    /// Log entries organized by session
    sessions: Arc<Mutex<HashMap<String, RefactoringSession>>>,
    /// Detailed operation logs
    operation_logs: Arc<Mutex<Vec<OperationLogEntry>>>,
    /// Performance metrics
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Error tracking
    error_tracker: Arc<Mutex<ErrorTracker>>,
}

impl RefactoringLogger {
    /// Create a new logger instance
    pub fn new() -> Self {
        RefactoringLogger {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            operation_logs: Arc::new(Mutex::new(Vec::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            error_tracker: Arc::new(Mutex::new(ErrorTracker::new())),
        }
    }

    /// Start a new refactoring session
    pub fn start_session(&self, session_type: SessionType, description: String) -> String {
        let session_id = format!("{}_{}", session_type.as_str(), Utc::now().timestamp_millis());

        info!("Started refactoring session: {} - {}", session_id, description);

        let session = RefactoringSession {
            id: session_id.clone(),
            session_type,
            description,
            start_time: Utc::now(),
            end_time: None,
            status: SessionStatus::Active,
            metadata: HashMap::new(),
            operations_count: 0,
            successful_operations: 0,
            failed_operations: 0,
        };

        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session_id.clone(), session);
        }

        session_id
    }

    /// End a refactoring session
    pub fn end_session(&self, session_id: &str, status: SessionStatus) {
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get_mut(session_id) {
                session.end_time = Some(Utc::now());
                session.status = status.clone();

                let duration = session.end_time.unwrap().signed_duration_since(session.start_time);
                info!(
                    "Ended refactoring session: {} - Status: {:?}, Duration: {}ms, Operations: {}/{} successful",
                    session_id, status, duration.num_milliseconds(),
                    session.successful_operations, session.operations_count
                );
            }
        }
    }

    /// Log operation start
    pub fn log_operation_start(&self, session_id: &str, refactoring_type: &RefactoringType, context: &RefactoringContext) {
        let entry = OperationLogEntry {
            session_id: session_id.to_string(),
            operation_type: refactoring_type.clone(),
            timestamp: Utc::now(),
            event_type: OperationEventType::Started,
            context: context.clone(),
            result: None,
            error: None,
            performance_data: None,
        };

        if let Ok(mut logs) = self.operation_logs.lock() {
            logs.push(entry);
        }

        debug!(
            "Operation started: {:?} on {}:{} in session {}",
            refactoring_type, context.file_path, context.cursor_line, session_id
        );

        // Update session counter
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get_mut(session_id) {
                session.operations_count += 1;
            }
        }
    }

    /// Log operation completion
    pub fn log_operation_complete(
        &self,
        session_id: &str,
        refactoring_type: &RefactoringType,
        context: &RefactoringContext,
        result: &RefactoringResult,
        duration_ms: f64,
    ) {
        let event_type = if result.success {
            OperationEventType::Completed
        } else {
            OperationEventType::Failed
        };

        let entry = OperationLogEntry {
            session_id: session_id.to_string(),
            operation_type: refactoring_type.clone(),
            timestamp: Utc::now(),
            event_type,
            context: context.clone(),
            result: Some(result.clone()),
            error: result.error_message.clone(),
            performance_data: Some(PerformanceData { duration_ms }),
        };

        if let Ok(mut logs) = self.operation_logs.lock() {
            logs.push(entry);
        }

        // Update session counters
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get_mut(session_id) {
                if result.success {
                    session.successful_operations += 1;
                } else {
                    session.failed_operations += 1;
                }
            }
        }

        if result.success {
            info!(
                "Operation completed successfully: {:?} on {} - {} changes in {:.2}ms",
                refactoring_type, context.file_path, result.changes.len(), duration_ms
            );
        } else {
            error!(
                "Operation failed: {:?} on {} - Error: {:?}",
                refactoring_type, context.file_path, result.error_message
            );
        }
    }

    /// Log error with context
    pub fn log_error(
        &self,
        session_id: &str,
        error_type: ErrorType,
        error: Box<dyn std::error::Error + Send + Sync>,
        context: Option<&RefactoringContext>,
    ) {
        if let Ok(mut tracker) = self.error_tracker.lock() {
            tracker.record_error(error_type.clone(), error.to_string());
        }

        if let Some(ctx) = context {
            error!(
                "Error in session {}: {:?} - File: {}, Error: {}",
                session_id, error_type, ctx.file_path, error
            );
        } else {
            error!(
                "Error in session {}: {:?} - Error: {}",
                session_id, error_type, error
            );
        }

        // Also create log entry
        let entry = OperationLogEntry {
            session_id: session_id.to_string(),
            operation_type: RefactoringType::Rename, // Default, not used for error entries
            timestamp: Utc::now(),
            event_type: OperationEventType::Failed,
            context: context.cloned().unwrap_or_default(),
            result: None,
            error: Some(error.to_string()),
            performance_data: None,
        };

        if let Ok(mut logs) = self.operation_logs.lock() {
            logs.push(entry);
        }
    }

    /// Log batch operation progress
    pub fn log_batch_progress(&self, session_id: &str, progress: &BatchProgressInfo) {
        info!(
            "Batch progress: {} - Completed: {}/{}, Success: {}/{}",
            session_id,
            progress.completed_operations,
            progress.total_operations,
            progress.successful_operations,
            progress.completed_operations
        );
    }

    /// Log performance metrics
    pub fn log_performance(&self, operation_type: &RefactoringType, metric: PerformanceMetric) {
        trace!(
            "Performance: {:?} - Duration: {}ms, Memory: {}kb",
            operation_type, metric.duration_ms, metric.memory_kb
        );

        if let Ok(mut metrics) = self.performance_metrics.lock() {
            metrics.record_metric(operation_type, metric);
        }
    }

    /// Get session summary
    pub fn get_session_summary(&self, session_id: &str) -> Option<SessionSummary> {
        let sessions = self.sessions.lock().ok()?;
        let operation_logs = self.operation_logs.lock().ok()?;

        let session = sessions.get(session_id)?;
        let session_logs: Vec<_> = operation_logs.iter()
            .filter(|log| log.session_id == session_id)
            .collect();

        let total_duration = if let Some(end) = session.end_time {
            end.signed_duration_since(session.start_time).num_milliseconds()
        } else {
            0
        };

        Some(SessionSummary {
            session: session.clone(),
            total_operations: session_logs.len(),
            successful_operations: session.successful_operations,
            failed_operations: session.failed_operations,
            total_duration_ms: total_duration,
            average_operation_time: if session_logs.is_empty() {
                0.0
            } else {
                session_logs.iter()
                    .filter_map(|log| log.performance_data.as_ref())
                    .map(|data| data.duration_ms)
                    .sum::<f64>() / session_logs.len() as f64
            },
        })
    }

    /// Get error summary for a session
    pub fn get_error_summary(&self, session_id: &str) -> Option<ErrorSummary> {
        let error_tracker = self.error_tracker.lock().ok()?;
        Some(error_tracker.get_error_summary(session_id))
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let metrics = self.performance_metrics.lock().unwrap();
        metrics.generate_summary()
    }

    /// Export logs to file
    pub async fn export_logs(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::fs;
        use serde_json;

        let sessions = self.sessions.lock().unwrap().clone();
        let operation_logs = self.operation_logs.lock().unwrap().clone();
        let performance_metrics = self.performance_metrics.lock().unwrap().clone();
        let error_tracker = self.error_tracker.lock().unwrap().clone();

        let export_data = LogExport {
            sessions,
            operation_logs,
            performance_metrics,
            error_tracker,
            export_time: Utc::now(),
        };

        let json = serde_json::to_string_pretty(&export_data)?;
        fs::write(file_path, json).await?;

        info!("Logs exported to: {}", file_path);
        Ok(())
    }

    /// Clear old logs (cleanup utility)
    pub fn clear_old_logs(&self, days_to_keep: i64) {
        let cutoff_time = Utc::now() - chrono::Duration::days(days_to_keep);

        // Clear old operation logs
        if let Ok(mut logs) = self.operation_logs.lock() {
            logs.retain(|log| log.timestamp > cutoff_time);
        }

        info!("Cleared logs older than {} days", days_to_keep);
    }
}

/// Types of refactoring sessions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SessionType {
    SingleOperation,
    BatchOperation,
    BulkOperation,
    LSPIntegrationTest,
    SafetyAnalysis,
}

impl SessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionType::SingleOperation => "single",
            SessionType::BatchOperation => "batch",
            SessionType::BulkOperation => "bulk",
            SessionType::LSPIntegrationTest => "lsp_test",
            SessionType::SafetyAnalysis => "safety_check",
        }
    }
}

/// Session status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Failed,
    Cancelled,
}

/// Refactoring session information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefactoringSession {
    pub id: String,
    pub session_type: SessionType,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub metadata: HashMap<String, String>,
    pub operations_count: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
}

/// Individual operation log entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationLogEntry {
    pub session_id: String,
    pub operation_type: RefactoringType,
    pub timestamp: DateTime<Utc>,
    pub event_type: OperationEventType,
    pub context: RefactoringContext,
    pub result: Option<RefactoringResult>,
    pub error: Option<String>,
    pub performance_data: Option<PerformanceData>,
}

/// Operation event types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OperationEventType {
    Started,
    Completed,
    Failed,
    Cancelled,
    SafetyCheck,
    Analysis,
}

/// Performance data for operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceData {
    pub duration_ms: f64,
}

/// Performance metrics tracker
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetrics {
    metrics: HashMap<RefactoringType, Vec<PerformanceMetric>>,
    system_metrics: Vec<SystemMetric>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        PerformanceMetrics {
            metrics: HashMap::new(),
            system_metrics: Vec::new(),
        }
    }

    pub fn record_metric(&mut self, operation_type: &RefactoringType, metric: PerformanceMetric) {
        self.metrics.entry(operation_type.clone())
            .or_insert(Vec::new())
            .push(metric);
    }

    pub fn generate_summary(&self) -> PerformanceSummary {
        let mut averages: HashMap<RefactoringType, OperationPerformance> = HashMap::new();

        for (operation_type, metrics) in &self.metrics {
            if metrics.is_empty() { continue; }

            let avg_duration = metrics.iter().map(|m| m.duration_ms).sum::<f64>() / metrics.len() as f64;
            let avg_memory = metrics.iter().map(|m| m.memory_kb).sum::<f64>() / metrics.len() as f64;
            let avg_cpu = metrics.iter().map(|m| m.cpu_percent).sum::<f64>() / metrics.len() as f64;

            averages.insert(operation_type.clone(), OperationPerformance {
                average_duration_ms: avg_duration,
                average_memory_kb: avg_memory,
                average_cpu_percent: avg_cpu,
                execution_count: metrics.len(),
            });
        }

        PerformanceSummary {
            operation_performance: averages,
            system_metrics: self.system_metrics.clone(),
            total_operations_logged: self.metrics.values().map(|v| v.len()).sum(),
        }
    }
}

/// Individual performance metric
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetric {
    pub duration_ms: f64,
    pub memory_kb: f64,
    pub cpu_percent: f64,
    pub timestamp: DateTime<Utc>,
}

/// System performance metric
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMetric {
    pub metric_type: SystemMetricType,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

/// System metric types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SystemMetricType {
    MemoryUsage,
    CPUUsage,
    DiskUsage,
    NetworkActivity,
}

/// Error tracker
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorTracker {
    errors: Vec<OperationError>,
}

impl ErrorTracker {
    pub fn new() -> Self {
        ErrorTracker {
            errors: Vec::new(),
        }
    }

    pub fn record_error(&mut self, error_type: ErrorType, message: String) {
        let error = OperationError {
            error_type,
            message,
            timestamp: Utc::now(),
            context: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        };
        self.errors.push(error);
    }

    pub fn get_error_summary(&self, session_id: &str) -> ErrorSummary {
        let session_errors: Vec<_> = self.errors.iter()
            .filter(|e| e.context.contains(session_id))
            .cloned()
            .collect();

        let error_counts = session_errors.iter()
            .fold(HashMap::new(), |mut acc, error| {
                *acc.entry(error.error_type.clone()).or_insert(0) += 1;
                acc
            });

        ErrorSummary {
            total_errors: session_errors.len(),
            error_counts,
            recent_errors: session_errors.iter().take(5).cloned().collect(),
        }
    }
}

/// Operation error
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationError {
    pub error_type: ErrorType,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub context: String,
}

/// Error types
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ErrorType {
    ASTParseError,
    SafetyCheckFailure,
    FileAccessError,
    TransformationError,
    ValidationError,
    LSPCommunicationError,
    UnknownError,
}

/// Session summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionSummary {
    pub session: RefactoringSession,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_duration_ms: i64,
    pub average_operation_time: f64,
}

/// Error summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorSummary {
    pub total_errors: usize,
    pub error_counts: HashMap<ErrorType, usize>,
    pub recent_errors: Vec<OperationError>,
}

/// Performance summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceSummary {
    pub operation_performance: HashMap<RefactoringType, OperationPerformance>,
    pub system_metrics: Vec<SystemMetric>,
    pub total_operations_logged: usize,
}

/// Operation performance metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationPerformance {
    pub average_duration_ms: f64,
    pub average_memory_kb: f64,
    pub average_cpu_percent: f64,
    pub execution_count: usize,
}

/// Batch progress information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchProgressInfo {
    pub total_operations: usize,
    pub completed_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub estimated_time_remaining: Option<f64>,
}

/// Exportable log data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogExport {
    pub sessions: HashMap<String, RefactoringSession>,
    pub operation_logs: Vec<OperationLogEntry>,
    pub performance_metrics: PerformanceMetrics,
    pub error_tracker: ErrorTracker,
    pub export_time: DateTime<Utc>,
}

impl Default for RefactoringLogger {
    fn default() -> Self {
        Self::new()
    }
}