//! Progress tracking for refactoring operations

/// Progress tracker for operations
pub struct ProgressTracker;

impl ProgressTracker {
    pub fn new() -> Self {
        ProgressTracker
    }

    pub async fn start_operation(&self, _operation_type: String) -> String {
        "operation_id".to_string()
    }

    pub async fn complete_operation(&self, _operation_id: String) {}

    pub async fn fail_operation(&self, _operation_id: String, _error: &str) {}
}
