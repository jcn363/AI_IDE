//! Incremental analysis module for efficient code analysis by only processing changed files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use super::AnalysisConfig;
use crate::analysis::cache::{self, CacheKey};

/// Tracks file modification times and analysis states for incremental processing.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IncrementalState {
    /// Maps file paths to their last known modification time and analysis state
    file_states: HashMap<PathBuf, FileState>,
    /// Global configuration hash for cache invalidation
    config_hash: String,
}

/// Represents the state of a file for incremental analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileState {
    /// Last modification time of the file when it was analyzed
    modified_time: u64,
    /// Size of the file when it was analyzed
    size:          u64,
    /// Hash of the file contents when it was analyzed
    content_hash:  String,
    /// Whether the file was successfully analyzed
    analyzed:      bool,
    /// Any dependencies this file has on other files
    dependencies:  Vec<PathBuf>,
}

impl IncrementalState {
    /// Creates a new incremental state with the given configuration hash
    pub fn new(config_hash: &str) -> Self {
        Self {
            file_states: HashMap::new(),
            config_hash: config_hash.to_string(),
        }
    }

    /// Loads the incremental state from the cache
    pub fn load(config: &AnalysisConfig) -> Result<Self> {
        let cache_key = CacheKey::new(
            Path::new("incremental_state"),
            "global_state",
            1,
            &config.get_hash().to_string(),
            &config,
        )?;

        if let Some(state) = cache::get_cached(&cache_key)? {
            debug!("Loaded incremental state from cache");
            Ok(state)
        } else {
            debug!("No cached incremental state found, starting fresh");
            Ok(Self::new(&config.get_hash().to_string()))
        }
    }

    /// Saves the incremental state to the cache
    pub fn save(&self, config: &AnalysisConfig) -> Result<()> {
        let cache_key = CacheKey::new(
            Path::new("incremental_state"),
            "global_state",
            1,
            &self.config_hash,
            &config,
        )?;

        cache::set_cached(&cache_key, self, Some(Duration::from_secs(86400 * 7)), None)
            .context("Failed to save incremental state")?;

        debug!("Saved incremental state to cache");
        Ok(())
    }

    /// Determines which files need to be analyzed based on changes
    pub fn get_changed_files<'a>(
        &mut self,
        root_dir: &Path,
        file_patterns: &[&str],
        config: &AnalysisConfig,
    ) -> Result<Vec<PathBuf>> {
        let mut changed_files = Vec::new();
        let mut file_patterns_regex = file_patterns
            .iter()
            .map(|p| regex::Regex::new(p).map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;

        // Always include all files if no patterns are specified
        if file_patterns_regex.is_empty() {
            file_patterns_regex.push(regex::Regex::new(".*")?);
        }

        for entry in WalkDir::new(root_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let rel_path = path.strip_prefix(root_dir).unwrap_or(path).to_path_buf();

            // Skip files that don't match any of the patterns
            let path_str = rel_path.to_string_lossy();
            if !file_patterns_regex.iter().any(|re| re.is_match(&path_str)) {
                trace!("Skipping file that doesn't match patterns: {:?}", path);
                continue;
            }

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(e) => {
                    debug!("Failed to get metadata for {}: {}", path.display(), e);
                    continue;
                }
            };

            let modified_time = FileTime::from_last_modification_time(&metadata).unix_seconds() as u64;
            let file_size = metadata.len();
            let content_hash = self.calculate_file_hash(path)?;

            let needs_analysis = match self.file_states.get(path) {
                Some(state) => {
                    // Check if file has changed
                    state.modified_time != modified_time ||
                    state.size != file_size ||
                    state.content_hash != content_hash ||
                    // Check if any dependencies have changed
                    self.have_dependencies_changed(path, config)?
                }
                None => true, // New file, needs analysis
            };

            if needs_analysis {
                debug!("File needs analysis: {:?}", path);
                changed_files.push(path.to_path_buf());
            }

            // Update the state with current file info
            self.file_states.insert(path.to_path_buf(), FileState {
                modified_time,
                size: file_size,
                content_hash,
                analyzed: !needs_analysis, // Mark as analyzed if not changed
                dependencies: Vec::new(),  // Will be updated during analysis
            });
        }

        Ok(changed_files)
    }

    /// Updates the analysis state for a file
    pub fn update_file_analysis(&mut self, path: &Path, mut dependencies: Vec<PathBuf>, success: bool) -> Result<()> {
        // Get file metadata
        let metadata =
            std::fs::metadata(path).with_context(|| format!("Failed to get metadata for file: {}", path.display()))?;

        let modified_time = FileTime::from_last_modification_time(&metadata).unix_seconds() as u64;
        let size = metadata.len();

        // Calculate content hash
        let content_hash = self.calculate_file_hash(path)?;

        // Normalize dependencies to ensure consistent paths
        dependencies = dependencies
            .into_iter()
            .map(|p| p.canonicalize().unwrap_or(p))
            .collect();

        // Update or insert the file state
        self.file_states.insert(path.to_path_buf(), FileState {
            modified_time,
            size,
            content_hash,
            analyzed: success,
            dependencies,
        });

        Ok(())
    }

    /// Checks if any dependencies of a file have changed
    fn have_dependencies_changed(&self, path: &Path, config: &AnalysisConfig) -> Result<bool> {
        if let Some(state) = self.file_states.get(path) {
            for dep_path in &state.dependencies {
                if let Some(dep_state) = self.file_states.get(dep_path) {
                    let current_mtime =
                        FileTime::from_last_modification_time(&std::fs::metadata(dep_path)?).unix_seconds() as u64;

                    if dep_state.modified_time < current_mtime {
                        debug!("Dependency changed: {}", dep_path.display());
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Checks if a file needs to be analyzed
    pub fn needs_analysis(&self, file_path: &str) -> bool {
        let path = Path::new(file_path);

        // If we don't have any state for this file, it needs analysis
        let Some(state) = self.file_states.get(path) else {
            return true;
        };

        // Check if the file has been modified since last analysis
        match std::fs::metadata(path) {
            Ok(metadata) => {
                let modified = FileTime::from_last_modification_time(&metadata).unix_seconds() as u64;
                if modified > state.modified_time {
                    return true;
                }

                // Check if file size has changed
                if metadata.len() != state.size {
                    return true;
                }

                // Check if content hash has changed
                match self.calculate_file_hash(path) {
                    Ok(hash) if hash != state.content_hash => return true,
                    Err(_) => return true, // If we can't calculate hash, re-analyze to be safe
                    _ => (),
                }
            }
            Err(_) => return true, // If we can't get metadata, re-analyze
        }

        // If we get here, the file itself hasn't changed, but we need to check dependencies
        match self.have_dependencies_changed(path, &AnalysisConfig::default()) {
            Ok(changed) => changed,
            Err(_) => true, // If we can't check dependencies, re-analyze to be safe
        }
    }

    /// Calculates a hash of the file contents for change detection
    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        use std::fs::File;
        use std::io::Read;

        use blake3;

        let mut file =
            File::open(path).with_context(|| format!("Failed to open file for hashing: {}", path.display()))?;

        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 8192];

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }
}

/// Performs incremental analysis on a directory
pub fn analyze_incrementally(
    root_dir: &Path,
    file_patterns: &[&str],
    config: &AnalysisConfig,
    analyzer: impl Fn(&Path) -> Result<Vec<PathBuf>>,
) -> Result<()> {
    info!("Starting incremental analysis in: {}", root_dir.display());

    // Load or create incremental state
    let mut state = IncrementalState::load(config)?;

    // Get list of changed files that need analysis
    let changed_files = state.get_changed_files(root_dir, file_patterns, config)?;

    info!("Found {} files that need analysis", changed_files.len());

    // Process each changed file
    for file_path in changed_files {
        info!("Analyzing: {}", file_path.display());

        match analyzer(&file_path) {
            Ok(dependencies) => {
                state.update_file_analysis(&file_path, dependencies, true)?;
            }
            Err(e) => {
                log::error!("Failed to analyze {}: {}", file_path.display(), e);
                state.update_file_analysis(&file_path, Vec::new(), false)?;
            }
        }
    }

    // Save the updated state
    state.save(config)?;

    info!("Incremental analysis completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_incremental_analysis() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let root_path = temp_dir.path();

        // Create some test files
        let file1 = root_path.join("test1.rs");
        let file2 = root_path.join("test2.rs");

        fs::write(&file1, "fn main() {}")?;
        fs::write(&file2, "fn helper() {}")?;

        // Create a test analyzer that just returns the file's dependencies
        let analyzer = |path: &Path| -> Result<Vec<PathBuf>> {
            if path == file1 {
                Ok(vec![file2.clone()])
            } else {
                Ok(Vec::new())
            }
        };

        // Run initial analysis
        let config = AnalysisConfig::default();
        analyze_incrementally(root_path, &[".*\\.rs"], &config, &analyzer)?;

        // Modify one of the files
        let mut file = fs::OpenOptions::new().append(true).open(&file1)?;
        writeln!(file, "// New comment")?;

        // Run analysis again - only the modified file should be analyzed
        analyze_incrementally(root_path, &[".*\\.rs"], &config, &analyzer)?;

        // Modify the dependency file
        let mut file = fs::OpenOptions::new().append(true).open(&file2)?;
        writeln!(file, "// Updated dependency")?;

        // Run analysis again - both files should be analyzed because of the dependency
        analyze_incrementally(root_path, &[".*\\.rs"], &config, &analyzer)?;

        Ok(())
    }
}
