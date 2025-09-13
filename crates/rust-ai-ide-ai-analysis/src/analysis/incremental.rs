//! Incremental analysis capabilities

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Manages incremental analysis of changed files
#[derive(Debug)]
pub struct IncrementalAnalyzer {
    analyzed_files:     RwLock<HashSet<PathBuf>>,
    file_modifications: RwLock<HashSet<PathBuf>>,
}

impl IncrementalAnalyzer {
    /// Create a new incremental analyzer
    pub fn new() -> Self {
        Self {
            analyzed_files:     RwLock::new(HashSet::new()),
            file_modifications: RwLock::new(HashSet::new()),
        }
    }

    /// Mark a file as analyzed
    pub fn mark_analyzed(&self, path: &Path) {
        self.analyzed_files
            .write()
            .unwrap()
            .insert(path.to_path_buf());
    }

    /// Mark a file as modified
    pub fn mark_modified(&self, path: &Path) {
        self.file_modifications
            .write()
            .unwrap()
            .insert(path.to_path_buf());
    }

    /// Get files that need re-analysis (modified but already analyzed)
    pub fn get_files_needing_reanalysis(&self) -> Vec<PathBuf> {
        let analyzed = self.analyzed_files.read().unwrap();
        let modified = self.file_modifications.read().unwrap();

        modified.intersection(&analyzed).cloned().collect()
    }

    /// Clear all modification tracking
    pub fn clear_modifications(&self) {
        self.file_modifications.write().unwrap().clear();
    }

    /// Reset analysis tracking
    pub fn reset(&self) {
        self.analyzed_files.write().unwrap().clear();
        self.file_modifications.write().unwrap().clear();
    }

    /// Check if a file has been modified since last analysis
    pub fn is_modified_since_last_analysis(&self, path: &Path) -> bool {
        self.analyzed_files.read().unwrap().contains(path) && self.file_modifications.read().unwrap().contains(path)
    }
}

impl Default for IncrementalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Track file changes for incremental analysis
pub struct FileChangeTracker {
    modified_files: HashSet<PathBuf>,
    new_files:      HashSet<PathBuf>,
    deleted_files:  HashSet<PathBuf>,
}

impl FileChangeTracker {
    /// Create a new change tracker
    pub fn new() -> Self {
        Self {
            modified_files: HashSet::new(),
            new_files:      HashSet::new(),
            deleted_files:  HashSet::new(),
        }
    }

    /// Add a modified file
    pub fn add_modified(&mut self, path: PathBuf) {
        self.modified_files.insert(path);
    }

    /// Add a new file
    pub fn add_new(&mut self, path: PathBuf) {
        self.new_files.insert(path);
    }

    /// Add a deleted file
    pub fn add_deleted(&mut self, path: PathBuf) {
        self.deleted_files.insert(path);
    }

    /// Get all changed files
    pub fn all_changed_files(&self) -> HashSet<PathBuf> {
        let mut all = HashSet::new();
        all.extend(self.modified_files.iter().cloned());
        all.extend(self.new_files.iter().cloned());
        all.extend(self.deleted_files.iter().cloned());
        all
    }

    /// Clear all tracked changes
    pub fn clear(&mut self) {
        self.modified_files.clear();
        self.new_files.clear();
        self.deleted_files.clear();
    }
}

impl Default for FileChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}
