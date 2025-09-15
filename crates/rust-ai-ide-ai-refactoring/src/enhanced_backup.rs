//! Enhanced backup system for refactoring operations

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::Digest;
use tokio::sync::Mutex;

/// Enhanced backup manager for refactoring operations
pub struct EnhancedBackupManager {
    /// Backup storage directory
    backup_dir:            PathBuf,
    /// Active backups tracking
    active_backups:        Mutex<HashMap<String, BackupSession>>,
    /// Maximum backup versions to keep per file
    max_versions_per_file: usize,
}

/// Backup session information
#[derive(Debug, Clone)]
struct BackupSession {
    session_id:    String,
    original_file: String,
    backup_path:   PathBuf,
    timestamp:     std::time::SystemTime,
    file_hash:     String,
}

/// Backup metadata
#[derive(Debug, Clone)]
pub struct BackupMetadata {
    pub session_id:  String,
    pub file_path:   String,
    pub backup_path: String,
    pub timestamp:   std::time::SystemTime,
    pub size_bytes:  u64,
}

impl EnhancedBackupManager {
    pub fn new() -> Self {
        let backup_dir = std::env::temp_dir().join("rust_ai_ide_backups");
        fs::create_dir_all(&backup_dir).ok(); // Ignore errors if directory exists

        EnhancedBackupManager {
            backup_dir,
            active_backups: Mutex::new(HashMap::new()),
            max_versions_per_file: 10, // Keep 10 versions max
        }
    }

    /// Create a backup of the specified file
    pub async fn create_backup(&self, file_path: &str) -> Result<BackupMetadata, String> {
        let file_path = Path::new(file_path);
        if !file_path.exists() {
            return Err(format!("File does not exist: {}", file_path.display()));
        }

        // Generate unique session ID
        let session_id = format!(
            "backup_{}_{}",
            file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Create backup path
        let backup_path = self.backup_dir.join(&session_id);
        fs::create_dir_all(&backup_path).map_err(|e| format!("Failed to create backup directory: {}", e))?;

        let backup_file_path = backup_path.join(
            file_path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("backup")),
        );

        // Copy file to backup location
        fs::copy(file_path, &backup_file_path).map_err(|e| format!("Failed to copy file to backup: {}", e))?;

        // Calculate file hash for integrity checking
        let file_hash = self
            .calculate_file_hash(&backup_file_path)
            .unwrap_or_else(|_| "hash_error".to_string());

        // Get file size
        let metadata = fs::metadata(&backup_file_path).map_err(|e| format!("Failed to read backup metadata: {}", e))?;
        let size_bytes = metadata.len();

        let timestamp = std::time::SystemTime::now();

        let session = BackupSession {
            session_id: session_id.clone(),
            original_file: file_path.to_string_lossy().to_string(),
            backup_path: backup_file_path.clone(),
            timestamp,
            file_hash,
        };

        // Store backup session
        {
            let mut active_backups = self.active_backups.lock().await;
            active_backups.insert(session_id.clone(), session);
        }

        // Clean up old versions
        self.cleanup_old_versions(file_path).await;

        Ok(BackupMetadata {
            session_id,
            file_path: file_path.to_string_lossy().to_string(),
            backup_path: backup_file_path.to_string_lossy().to_string(),
            timestamp,
            size_bytes,
        })
    }

    /// Rollback to a specific backup
    pub async fn rollback(&self, session_id: &str) -> Result<(), String> {
        let active_backups = self.active_backups.lock().await;
        let session = active_backups
            .get(session_id)
            .ok_or_else(|| format!("Backup session not found: {}", session_id))?;

        // Verify backup integrity
        let current_hash = self.calculate_file_hash(&session.backup_path)?;
        if current_hash != session.file_hash {
            return Err(format!(
                "Backup integrity check failed for session: {}",
                session_id
            ));
        }

        // Restore the original file
        fs::copy(&session.backup_path, &session.original_file)
            .map_err(|e| format!("Failed to restore file from backup: {}", e))?;

        Ok(())
    }

    /// List available backups for a file
    pub async fn list_backups(&self, file_path: &str) -> Result<Vec<BackupMetadata>, String> {
        let active_backups = self.active_backups.lock().await;
        let mut backups = Vec::new();

        for session in active_backups.values() {
            if session.original_file == file_path {
                let metadata =
                    fs::metadata(&session.backup_path).map_err(|e| format!("Failed to read backup metadata: {}", e))?;

                backups.push(BackupMetadata {
                    session_id:  session.session_id.clone(),
                    file_path:   session.original_file.clone(),
                    backup_path: session.backup_path.to_string_lossy().to_string(),
                    timestamp:   session.timestamp,
                    size_bytes:  metadata.len(),
                });
            }
        }

        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    /// Delete a backup session
    pub async fn delete_backup(&self, session_id: &str) -> Result<(), String> {
        let mut active_backups = self.active_backups.lock().await;
        if let Some(session) = active_backups.remove(session_id) {
            // Remove backup files
            if session.backup_path.exists() {
                fs::remove_dir_all(session.backup_path.parent().unwrap_or(&session.backup_path))
                    .map_err(|e| format!("Failed to remove backup directory: {}", e))?;
            }
        }

        Ok(())
    }

    /// Clean up old backup versions for a file
    async fn cleanup_old_versions(&self, file_path: &Path) {
        let file_path_str = file_path.to_string_lossy().to_string();
        let active_backups = self.active_backups.lock().await;

        // Collect backups for this file
        let mut file_backups: Vec<_> = active_backups
            .values()
            .filter(|s| s.original_file == file_path_str)
            .collect();

        // Sort by timestamp (newest first)
        file_backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Remove old versions beyond limit
        if file_backups.len() > self.max_versions_per_file {
            let to_remove = &file_backups[self.max_versions_per_file..];
            for session in to_remove {
                if session.backup_path.exists() {
                    fs::remove_dir_all(session.backup_path.parent().unwrap_or(&session.backup_path)).ok();
                }
            }
        }
    }

    /// Calculate MD5 hash of a file for integrity checking
    fn calculate_file_hash(&self, file_path: &Path) -> Result<String, String> {
        use std::io::Read;

        let mut file = fs::File::open(file_path).map_err(|e| format!("Failed to open file for hashing: {}", e))?;

        let mut hasher = sha2::Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|e| format!("Failed to read file for hashing: {}", e))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get backup storage statistics
    pub async fn get_backup_stats(&self) -> Result<BackupStats, String> {
        let active_backups = self.active_backups.lock().await;
        let total_backups = active_backups.len();

        let mut total_size = 0u64;
        let mut files_backed_up = std::collections::HashSet::new();

        for session in active_backups.values() {
            if let Ok(metadata) = fs::metadata(&session.backup_path) {
                total_size += metadata.len();
                files_backed_up.insert(&session.original_file);
            }
        }

        Ok(BackupStats {
            total_backups,
            total_size_bytes: total_size,
            unique_files_backed_up: files_backed_up.len(),
            backup_directory: self.backup_dir.to_string_lossy().to_string(),
        })
    }
}

/// Backup storage statistics
#[derive(Debug, Clone)]
pub struct BackupStats {
    pub total_backups:          usize,
    pub total_size_bytes:       u64,
    pub unique_files_backed_up: usize,
    pub backup_directory:       String,
}
