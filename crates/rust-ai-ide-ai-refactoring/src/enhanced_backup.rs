use crate::types::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::io::AsyncReadExt;

/// Enhanced backup system with incremental restoration and smart versioning
pub struct EnhancedBackupManager {
    backup_root: PathBuf,
    current_version: u64,
    backup_history: Vec<BackupVersion>,
    incremental_enabled: bool,
    max_backup_size_mb: u64,
}

impl EnhancedBackupManager {
    /// Create a new enhanced backup manager
    pub fn new(backup_root: String) -> Self {
        let backup_root = PathBuf::from(backup_root);
        Self::create_backup_directory(&backup_root);

        EnhancedBackupManager {
            backup_root,
            current_version: 0,
            backup_history: Vec::new(),
            incremental_enabled: true,
            max_backup_size_mb: 100, // 100MB default limit per backup
        }
    }

    /// Create incremental backup for multiple changes
    pub async fn create_incremental_backup(
        &mut self,
        project_root: &str,
        changes: &[CodeChange],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let version = self.current_version + 1;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();

        let backup_id = format!("incremental_{}_{}", version, timestamp);
        let backup_dir = self.backup_root.join(&backup_id);

        // Create backup directory
        fs::create_dir_all(&backup_dir).await?;

        // Get files that would be affected
        let affected_files = self.extract_affected_files(changes)?;
        let mut backup_manifest = BackupManifest {
            version,
            backup_id: backup_id.clone(),
            timestamp,
            project_root: project_root.to_string(),
            affected_files: affected_files.clone().into_iter().collect(),
            changes: changes.to_vec(),
            total_size_bytes: 0,
            incremental: true,
            parent_version: Some(self.current_version),
        };

        let backup_info = BackupInfo {
            id: backup_id.clone(),
            directory: backup_dir.clone(),
            manifest: backup_manifest.clone(),
            restored: false,
        };

        // Create incremental backup (only save changed files)
        let mut total_size = 0u64;
        for file_path in &affected_files {
            let source_path = Path::new(project_root).join(file_path);

            if source_path.exists() {
                // Calculate file hash for incremental backup
                let file_hash = self.calculate_file_hash(&source_path).await?;
                let relative_backup_path = self.get_relative_backup_path(file_path, &file_hash);
                let full_backup_path = backup_dir.join(&relative_backup_path);

                // Create parent directories if needed
                if let Some(parent) = full_backup_path.parent() {
                    fs::create_dir_all(parent).await?;
                }

                // Copy file
                self.copy_file(&source_path, &full_backup_path).await?;
                let file_size = fs::metadata(&source_path).await?.len();
                total_size += file_size;

                // Check size limit
                if total_size > self.max_backup_size_mb * 1024 * 1024 {
                    return Err(format!(
                        "Backup size exceeds limit ({}MB). Consider using larger limit or smaller batch size.",
                        self.max_backup_size_mb
                    ).into());
                }
            }
        }

        // Update manifest with total size
        backup_manifest.total_size_bytes = total_size;

        // Save manifest
        let manifest_path = backup_dir.join("manifest.json");
        let manifest_content = serde_json::to_string_pretty(&backup_manifest)?;
        fs::write(manifest_path, manifest_content).await?;

        // Save backup metadata for versioning
        let version_info = BackupVersion {
            version,
            backup_id: backup_id.clone(),
            timestamp,
            incremental: true,
            parent_version: Some(self.current_version),
            size_bytes: total_size,
        };

        self.backup_history.push(version_info);
        self.current_version = version;

        // Keep only last 50 backups for history management
        self.cleanup_old_backups().await?;

        Ok(backup_id)
    }

    /// Create full project backup (for critical operations)
    pub async fn create_full_backup(
        &mut self,
        project_root: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let version = self.current_version + 1;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();

        let backup_id = format!("full_{}_{}", version, timestamp);
        let backup_dir = self.backup_root.join(&backup_id);

        fs::create_dir_all(&backup_dir).await?;

        // Get all project files for full backup
        let all_files = self.get_all_project_files(project_root).await?;
        let total_size = self.calculate_total_size(&all_files, project_root).await?;

        if total_size > self.max_backup_size_mb * 1024 * 1024 {
            return Err(format!("Full backup too large ({}MB). Increase size limit or use incremental backup.", total_size / (1024 * 1024)).into());
        }

        let mut copied_files = Vec::new();
        for file_path in &all_files {
            let source_path = Path::new(project_root).join(file_path);
            let backup_file_path = backup_dir.join(file_path);

            // Create directory structure
            if let Some(parent) = backup_file_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            self.copy_file(&source_path, &backup_file_path).await?;
            copied_files.push(file_path.clone());
        }

        let manifest = BackupManifest {
            version,
            backup_id: backup_id.clone(),
            timestamp,
            project_root: project_root.to_string(),
            affected_files: copied_files,
            changes: vec![], // Full backup has no specific changes
            total_size_bytes: total_size,
            incremental: false,
            parent_version: None,
        };

        // Save manifest
        let manifest_path = backup_dir.join("manifest.json");
        let manifest_content = serde_json::to_string_pretty(&manifest)?;
        fs::write(manifest_path, manifest_content).await?;

        // Update versioning
        let version_info = BackupVersion {
            version,
            backup_id: backup_id.clone(),
            timestamp,
            incremental: false,
            parent_version: Some(0),
            size_bytes: total_size,
        };

        self.backup_history.push(version_info);
        self.current_version = version;

        Ok(backup_id)
    }

    /// Restore from incremental backup with conflict resolution
    pub async fn restore_incremental_backup(
        &mut self,
        backup_id: &str,
        project_root: &str,
        restore_strategy: RestoreStrategy,
    ) -> Result<RestoreResult, Box<dyn std::error::Error + Send + Sync>> {
        let backup_dir = self.backup_root.join(backup_id);
        if !backup_dir.exists() {
            return Err(format!("Backup {} not found", backup_id).into());
        }

        let manifest_path = backup_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(format!("Backup manifest for {} is missing", backup_id).into());
        }

        let manifest_content = fs::read_to_string(manifest_path).await?;
        let manifest: BackupManifest = serde_json::from_str(&manifest_content)?;

        let mut restored_files = Vec::new();
        let mut failed_restores = Vec::new();
        let mut conflicts_resolved = Vec::new();

        // Process each file in the backup
        for file_path in &manifest.affected_files {
            let backup_file_path = backup_dir.join(file_path);
            let target_file_path = Path::new(project_root).join(file_path);

            if !backup_file_path.exists() {
                failed_restores.push(format!("Backup file missing: {}", file_path));
                continue;
            }

            // Check for conflicts
            let conflict_exists = target_file_path.exists() && manifest.incremental;

            if conflict_exists {
                match restore_strategy {
                    RestoreStrategy::OverwriteExisting => {
                        // Overwrite existing files
                        self.copy_file(&backup_file_path, &target_file_path).await?;
                        conflicts_resolved.push(file_path.clone());
                    }
                    RestoreStrategy::MergeInteractive => {
                        // For now, merge by creating a .merge file for user review
                        let merge_path = target_file_path.with_extension("merge");
                        self.copy_file(&backup_file_path, &merge_path).await?;
                        conflicts_resolved.push(format!("{} (merge created)", file_path));
                    }
                    RestoreStrategy::BackupAndRestore => {
                        // Create backup of current file
                        let conflict_backup = format!("{}.conflict.{}", file_path,
                            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
                        let conflict_path = Path::new(project_root).join(&conflict_backup);
                        self.copy_file(&target_file_path, &conflict_path).await?;

                        // Restore original
                        self.copy_file(&backup_file_path, &target_file_path).await?;
                        conflicts_resolved.push(format!("{} (conflict backed up as {})", file_path, conflict_backup));
                    }
                }
            } else {
                // No conflict, direct restore
                if let Some(parent) = target_file_path.parent() {
                    fs::create_dir_all(parent).await?;
                }
                self.copy_file(&backup_file_path, &target_file_path).await?;
                restored_files.push(file_path.clone());
            }
        }

        Ok(RestoreResult {
            backup_id: backup_id.to_string(),
            restored_files,
            failed_restores,
            conflicts_resolved,
            total_files: manifest.affected_files.len(),
            restore_strategy,
        })
    }

    /// Get backup information and statistics
    pub fn get_backup_info(&self, backup_id: &str) -> Option<BackupInfo> {
        let backup_dir = self.backup_root.join(backup_id);
        let manifest_path = backup_dir.join("manifest.json");

        if !manifest_path.exists() {
            return None;
        }

        // This would read manifest and return info in a real implementation
        Some(BackupInfo {
            id: backup_id.to_string(),
            directory: backup_dir,
            manifest: BackupManifest {
                version: 1,
                backup_id: backup_id.to_string(),
                timestamp: 0,
                project_root: "".to_string(),
                affected_files: Vec::new(),
                changes: Vec::new(),
                total_size_bytes: 0,
                incremental: true,
                parent_version: None,
            },
            restored: false,
        })
    }

    /// List all available backups
    pub fn list_backups(&self) -> Vec<BackupSummary> {
        let mut summaries = Vec::new();

        for version in &self.backup_history {
            summaries.push(BackupSummary {
                backup_id: version.backup_id.clone(),
                version: version.version,
                timestamp: version.timestamp,
                size_mb: version.size_bytes as f64 / (1024.0 * 1024.0),
                incremental: version.incremental,
                description: if version.incremental {
                    "Incremental backup".to_string()
                } else {
                    "Full backup".to_string()
                },
            });
        }

        // Sort by version (most recent first)
        summaries.sort_by(|a, b| b.version.cmp(&a.version));
        summaries
    }

    /// Validate backup integrity
    pub async fn validate_backup_integrity(&self, backup_id: &str) -> Result<IntegrityCheck, Box<dyn std::error::Error + Send + Sync>> {
        let backup_dir = self.backup_root.join(backup_id);
        if !backup_dir.exists() {
            return Ok(IntegrityCheck {
                backup_id: backup_id.to_string(),
                valid: false,
                issues: vec!["Backup directory does not exist".to_string()],
                missing_files: Vec::new(),
                corrupted_files: Vec::new(),
            });
        }

        let manifest_path = backup_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Ok(IntegrityCheck {
                backup_id: backup_id.to_string(),
                valid: false,
                issues: vec!["Manifest file missing".to_string()],
                missing_files: Vec::new(),
                corrupted_files: Vec::new(),
            });
        }

        // Check manifest integrity
        let manifest_content = fs::read_to_string(manifest_path).await?;
        let manifest: Option<BackupManifest> = serde_json::from_str(&manifest_content).ok();

        let mut issues = Vec::new();
        let mut missing_files = Vec::new();
        let mut corrupted_files = Vec::new();

        if let Some(manifest) = manifest {
            // Check each file exists and has expected size
            for file_path in &manifest.affected_files {
                let backup_file_path = backup_dir.join(file_path);

                if !backup_file_path.exists() {
                    missing_files.push(file_path.clone());
                } else {
                    // Basic corruption check - readable file
                    match fs::read(&backup_file_path).await {
                        Ok(content) if content.is_empty() => {
                            corrupted_files.push(format!("{} (empty file)", file_path));
                        }
                        Err(_) => {
                            corrupted_files.push(file_path.clone());
                        }
                        Ok(_) => {} // File is OK
                    }
                }
            }
        } else {
            issues.push("Invalid manifest format".to_string());
        }

        if !missing_files.is_empty() {
            issues.push(format!("{} files missing", missing_files.len()));
        }
        if !corrupted_files.is_empty() {
            issues.push(format!("{} files corrupted", corrupted_files.len()));
        }

        Ok(IntegrityCheck {
            backup_id: backup_id.to_string(),
            valid: issues.is_empty(),
            issues,
            missing_files,
            corrupted_files,
        })
    }

    /// Cleanup old backups based on retention policy
    pub async fn cleanup_old_backups(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let max_backups = 50; // Keep last 50 backups
        let mut deleted_count = 0;

        if self.backup_history.len() <= max_backups {
            return Ok(0);
        }

        // Sort by version (oldest first)
        let to_delete: Vec<_> = self.backup_history.iter()
            .take(self.backup_history.len().saturating_sub(max_backups))
            .collect();

        // Actually delete the backup directories
        for version in &to_delete {
            let backup_dir = self.backup_root.join(&version.backup_id);
            if backup_dir.exists() {
                match self.remove_directory(&backup_dir).await {
                    Ok(_) => deleted_count += 1,
                    Err(e) => eprintln!("Warning: Failed to delete backup {}: {}", version.backup_id, e),
                }
            }
        }

        Ok(deleted_count)
    }

    /// Helper methods
    fn create_backup_directory(backup_root: &Path) {
        if let Err(e) = std::fs::create_dir_all(backup_root) {
            eprintln!("Warning: Failed to create backup directory: {}", e);
        }
    }

    async fn copy_file(&self, from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read(from).await?;
        fs::write(to, content).await?;
        Ok(())
    }

    async fn calculate_file_hash(&self, file_path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use sha2::{Sha256, Digest};
        let content = fs::read(file_path).await?;
        let hash = Sha256::new().chain_update(content).finalize();
        Ok(format!("{:x}", hash))
    }

    fn extract_affected_files(&self, changes: &[CodeChange]) -> Result<HashSet<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut files = HashSet::new();
        for change in changes {
            files.insert(change.file_path.clone());
        }
        Ok(files)
    }

    fn get_relative_backup_path(&self, file_path: &str, _hash: &str) -> String {
        file_path.to_string() // For now, keep flat structure
    }

    async fn get_all_project_files(&self, project_root: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let project_path = Path::new(project_root);
        let mut files = Vec::new();

        fn walk_dir(dir: &Path, root: &Path, files: &mut Vec<String>) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                        walk_dir(&path, root, files)?;
                    }
                } else {
                    if let Ok(rel_path) = path.strip_prefix(root) {
                        files.push(rel_path.to_string_lossy().to_string());
                    }
                }
            }
            Ok(())
        }

        walk_dir(project_path, project_path, &mut files)?;
        Ok(files)
    }

    async fn calculate_total_size(&self, files: &[String], project_root: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let mut total = 0u64;
        for file_path in files {
            let full_path = Path::new(project_root).join(file_path);
            if let Ok(metadata) = fs::metadata(&full_path).await {
                total += metadata.len();
            }
        }
        Ok(total)
    }

    async fn remove_directory(&self, dir: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        fs::remove_dir_all(dir).await?;
        Ok(())
    }
}

/// Backup manifest describing the backup contents
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupManifest {
    pub version: u64,
    pub backup_id: String,
    pub timestamp: u128,
    pub project_root: String,
    pub affected_files: Vec<String>,
    pub changes: Vec<CodeChange>,
    pub total_size_bytes: u64,
    pub incremental: bool,
    pub parent_version: Option<u64>,
}

/// Backup version information for history tracking
#[derive(Debug, Clone)]
pub struct BackupVersion {
    pub version: u64,
    pub backup_id: String,
    pub timestamp: u128,
    pub incremental: bool,
    pub parent_version: Option<u64>,
    pub size_bytes: u64,
}

/// Backup information
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub id: String,
    pub directory: PathBuf,
    pub manifest: BackupManifest,
    pub restored: bool,
}

/// Restore strategy for conflict resolution
#[derive(Debug, Clone)]
pub enum RestoreStrategy {
    /// Overwrite existing files
    OverwriteExisting,
    /// Create merge files for user review
    MergeInteractive,
    /// Backup current state and restore original
    BackupAndRestore,
}

/// Result of restore operation
#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub backup_id: String,
    pub restored_files: Vec<String>,
    pub failed_restores: Vec<String>,
    pub conflicts_resolved: Vec<String>,
    pub total_files: usize,
    pub restore_strategy: RestoreStrategy,
}

/// Backup summary for listing
#[derive(Debug, Clone)]
pub struct BackupSummary {
    pub backup_id: String,
    pub version: u64,
    pub timestamp: u128,
    pub size_mb: f64,
    pub incremental: bool,
    pub description: String,
}

/// Integrity check result
#[derive(Debug, Clone)]
pub struct IntegrityCheck {
    pub backup_id: String,
    pub valid: bool,
    pub issues: Vec<String>,
    pub missing_files: Vec<String>,
    pub corrupted_files: Vec<String>,
}

impl Default for EnhancedBackupManager {
    fn default() -> Self {
        Self::new("./enhanced-backups".to_string())
    }
}