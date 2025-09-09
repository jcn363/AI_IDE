//! Command history tracking for Cargo tasks

use super::{CargoTask, TaskResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::SystemTime;

/// Represents an entry in the command history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The task that was executed
    pub task: CargoTask,
    /// When the task was executed
    pub timestamp: DateTime<Utc>,
    /// Whether the task was successful
    pub success: bool,
    /// The duration of the task execution
    pub duration: Option<u64>,
    /// The end time of the task execution
    pub end_time: Option<SystemTime>,
}

impl HistoryEntry {
    /// Create a new history entry from a task result
    pub fn from_result(result: &TaskResult) -> Self {
        Self {
            task: result.task.clone(),
            timestamp: result.start_time.into(),
            success: result.success,
            duration: result.duration().map(|d| d.as_secs()),
            end_time: result.end_time,
        }
    }
}

/// Manages command history with a fixed capacity
pub struct CommandHistory {
    history: RwLock<VecDeque<HistoryEntry>>,
    capacity: usize,
}

impl CommandHistory {
    /// Create a new command history with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            history: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// Add a new entry to the history
    pub fn add_entry(&self, entry: HistoryEntry) {
        let mut history = self.history.write().unwrap();

        // Remove oldest entry if we've reached capacity
        if history.len() >= self.capacity {
            history.pop_back();
        }

        history.push_front(entry);
    }

    /// Get the most recent entries, with the most recent first
    pub fn get_recent(&self, limit: usize) -> Vec<HistoryEntry> {
        let history = self.history.read().unwrap();
        history.iter().take(limit).cloned().collect()
    }

    /// Find entries matching a predicate
    pub fn find<F>(&self, predicate: F) -> Vec<HistoryEntry>
    where
        F: Fn(&HistoryEntry) -> bool,
    {
        let history = self.history.read().unwrap();
        history.iter().filter(|e| predicate(e)).cloned().collect()
    }

    /// Clear the command history
    pub fn clear(&self) {
        let mut history = self.history.write().unwrap();
        history.clear();
    }

    /// Save the command history to a file
    pub fn save_to_file(&self, path: &PathBuf) -> std::io::Result<()> {
        let history = self.history.read().unwrap();
        let json = serde_json::to_string_pretty(&*history)?;
        std::fs::write(path, json)
    }

    /// Load the command history from a file
    pub fn load_from_file(path: &PathBuf) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let history: VecDeque<HistoryEntry> = serde_json::from_str(&json)?;
        let capacity = history.capacity();

        Ok(Self {
            history: RwLock::new(history),
            capacity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_task() -> CargoTask {
        CargoTask {
            command: "test".to_string(),
            args: vec!["--no-fail-fast".to_string()],
            working_dir: std::env::current_dir().unwrap(),
            release: false,
            env: vec![],
        }
    }

    #[test]
    fn test_history_operations() {
        let history = CommandHistory::new(5);
        let task = create_test_task();

        let result = TaskResult {
            task: task.clone(),
            exit_code: Some(0),
            stdout: "".to_string(),
            stderr: "".to_string(),
            start_time: SystemTime::now(),
            end_time: Some(SystemTime::now()),
            success: true,
        };

        let entry = HistoryEntry::from_result(&result);
        history.add_entry(entry);

        let entries = history.get_recent(1);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].task.command, "test");
    }
}
