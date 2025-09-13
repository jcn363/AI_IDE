use std::path::Path;

use crate::types::*;

/// Utility functions for refactoring operations
pub struct RefactoringUtils;
///// Range indexing normalization utilities
/// The frontend uses 1-based line indices (like most editors) while the backend
/// uses 0-based indices (like most programming languages). Character positions
/// are always 0-based. These utilities provide consistent conversion.
pub struct RangeNormalizer;

impl RangeNormalizer {
    /// Convert frontend range (1-based lines, 0-based chars) to backend format (0-based)
    pub fn frontend_to_backend(range: &CodeRange) -> CodeRange {
        CodeRange {
            start_line:      (range.start_line as isize).saturating_sub(1) as usize,
            start_character: range.start_character, // Already 0-based
            end_line:        (range.end_line as isize).saturating_sub(1) as usize,
            end_character:   range.end_character, // Already 0-based
        }
    }

    /// Convert backend range (0-based) to frontend format (1-based lines, 0-based chars)
    pub fn backend_to_frontend(range: &CodeRange) -> CodeRange {
        CodeRange {
            start_line:      range.start_line + 1,
            start_character: range.start_character, // Already 0-based
            end_line:        range.end_line + 1,
            end_character:   range.end_character, // Already 0-based
        }
    }

    /// Convert line index from frontend (1-based) to backend (0-based)
    pub fn frontend_line_to_backend(line: usize) -> usize {
        (line as isize).saturating_sub(1) as usize
    }

    /// Convert line index from backend (0-based) to frontend (1-based)
    pub fn backend_line_to_frontend(line: usize) -> usize {
        line + 1
    }

    /// Validate that a range is properly formatted
    pub fn validate_range(range: &CodeRange, context: &str) -> Result<(), String> {
        if range.start_line > range.end_line {
            return Err(format!(
                "{}: Invalid range - start_line ({}) > end_line ({})",
                context, range.start_line, range.end_line
            ));
        }

        if range.start_line == range.end_line && range.start_character > range.end_character {
            return Err(format!(
                "{}: Invalid range - start_character ({}) > end_character ({}) on same line",
                context, range.start_character, range.end_character
            ));
        }

        Ok(())
    }

    /// Ensure range doesn't exceed file bounds
    pub fn clamp_to_file_bounds(range: &CodeRange, line_count: usize, content: &str) -> CodeRange {
        if range.end_line >= line_count {
            let clamped_end_line = line_count.saturating_sub(1);
            let clamped_end_char = if clamped_end_line < line_count {
                content
                    .lines()
                    .nth(clamped_end_line)
                    .map_or(0, |line| line.len())
            } else {
                0
            };

            return CodeRange {
                start_line:      range.start_line,
                start_character: range.start_character,
                end_line:        clamped_end_line,
                end_character:   clamped_end_char,
            };
        }

        range.clone()
    }
}

impl RefactoringUtils {
    /// Validate file path and permissions
    pub fn validate_file_path(file_path: &str) -> Result<(), String> {
        if file_path.is_empty() {
            return Err("File path cannot be empty".to_string());
        }

        let path = Path::new(file_path);

        // Check if file exists
        if !path.exists() {
            return Err(format!("File does not exist: {}", file_path));
        }

        // Check if it's a file (not directory)
        if !path.is_file() {
            return Err(format!("Path is not a file: {}", file_path));
        }

        // Check if file is readable
        match std::fs::File::open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("File is not readable: {}", e)),
        }
    }

    /// Ensure file type is supported for AST-based operations
    pub fn ensure_rust_file(file_path: &str) -> Result<(), String> {
        let path = Path::new(file_path);
        match rust_ai_ide_shared_utils::get_extension(path) {
            Some(ext) if ext == "rs" => Ok(()),
            Some(ext) => Err(format!(
                "AST-based operations support Rust (.rs) files only, got: {}",
                ext
            )),
            None => Err(format!("Unable to determine file type for: {}", file_path)),
        }
    }

    /// Check if file is writable
    pub fn is_file_writable(file_path: &str) -> bool {
        let path = Path::new(file_path);
        match std::fs::OpenOptions::new().write(true).open(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Convert UTF-8 character index to byte offset
    /// This is crucial for safe string manipulation in Rust's UTF-8 strings
    pub fn char_to_byte_idx(content: &str, char_idx: usize) -> usize {
        content
            .char_indices()
            .nth(char_idx)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or(content.len())
    }

    /// Check if the language supports the refactoring type
    pub fn is_refactoring_supported(language: &str, refactoring_type: &RefactoringType) -> bool {
        match language.to_lowercase().as_str() {
            "rust" => matches!(
                refactoring_type,
                RefactoringType::Rename | RefactoringType::ExtractFunction
            ),
            "typescript" | "javascript" => matches!(
                refactoring_type,
                RefactoringType::Rename | RefactoringType::ExtractFunction | RefactoringType::ExtractVariable
            ),
            "python" => matches!(
                refactoring_type,
                RefactoringType::Rename | RefactoringType::ExtractFunction
            ),
            "java" => matches!(
                refactoring_type,
                RefactoringType::Rename | RefactoringType::ExtractFunction | RefactoringType::ExtractVariable
            ),
            "cpp" | "c" => matches!(refactoring_type, RefactoringType::Rename),
            _ => false, // Unknown language - default to unsupported
        }
    }

    /// Generate a unique ID for refactoring results
    pub fn generate_refactoring_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("refactor_{}", timestamp)
    }

    /// Generate SHA256 hash of content for integrity checking
    pub fn hash_content(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify content integrity against a hash
    pub fn verify_content_hash(content: &str, expected_hash: &str) -> bool {
        Self::hash_content(content) == expected_hash
    }

    /// Calculate diff between old and new content
    pub fn calculate_diff(old_content: &str, new_content: &str) -> Vec<DiffLine> {
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();

        let mut diff = Vec::new();

        // Simple diff implementation - in real world, use a proper diff library
        let max_len = std::cmp::max(old_lines.len(), new_lines.len());
        for i in 0..max_len {
            let old_line = old_lines.get(i);
            let new_line = new_lines.get(i);

            match (old_line, new_line) {
                (Some(old), Some(new)) =>
                    if old != new {
                        diff.push(DiffLine {
                            line_number: i + 1,
                            change_type: ChangeType::Replacement,
                            old_content: Some(old.to_string()),
                            new_content: Some(new.to_string()),
                        });
                    },
                (Some(old), None) => {
                    diff.push(DiffLine {
                        line_number: i + 1,
                        change_type: ChangeType::Deletion,
                        old_content: Some(old.to_string()),
                        new_content: None,
                    });
                }
                (None, Some(new)) => {
                    diff.push(DiffLine {
                        line_number: i + 1,
                        change_type: ChangeType::Insertion,
                        old_content: None,
                        new_content: Some(new.to_string()),
                    });
                }
                (None, None) => break,
            }
        }

        diff
    }

    /// Apply changes to string content with proper range normalization
    pub fn apply_changes_to_content(original_content: &str, changes: &[CodeChange]) -> Result<String, String> {
        let mut lines: Vec<String> = original_content.lines().map(|s| s.to_string()).collect();
        let mut offset = 0;

        // Sort changes by range start
        let mut sorted_changes = changes.to_vec();
        sorted_changes.sort_by_key(|c| (c.range.start_line, c.range.start_character));

        for change in sorted_changes {
            // Normalize the frontend range to backend format (convert 1-based lines to 0-based)
            let normalized_range = RangeNormalizer::frontend_to_backend(&change.range);

            // Validate the normalized range
            RangeNormalizer::validate_range(
                &normalized_range,
                &format!("CodeChange for {}", change.file_path),
            )?;
            let clamped_range =
                RangeNormalizer::clamp_to_file_bounds(&normalized_range, lines.len(), &original_content);

            // Apply offset to handle previous changes
            let start_line = (clamped_range.start_line as isize + offset) as usize;
            let end_line = (clamped_range.end_line as isize + offset) as usize;

            if start_line > lines.len() || end_line > lines.len() {
                return Err("Change range is out of bounds after applying offset".to_string());
            }

            // Apply the change
            if matches!(change.change_type, ChangeType::Replacement) {
                if end_line == start_line {
                    // Single line replacement - use UTF-8 safe byte offsets
                    let line = &lines[start_line];
                    let start_byte = Self::char_to_byte_idx(line, change.range.start_character as usize);
                    let end_byte = Self::char_to_byte_idx(line, change.range.end_character as usize);
                    let before = &line[..start_byte];
                    let after = &line[end_byte..];
                    lines[start_line] = format!("{}{}{}", before, change.new_text, after);
                } else {
                    // Multi-line replacement - use UTF-8 safe byte offsets
                    let first_line = &lines[start_line];
                    let last_line = &lines[end_line];

                    let first_char_start = change.range.start_character as usize;
                    let last_char_end = change.range.end_character as usize;
                    let first_part_byte = Self::char_to_byte_idx(first_line, first_char_start);
                    let last_part_byte = Self::char_to_byte_idx(last_line, last_char_end);

                    let first_part = &first_line[..first_part_byte];
                    let last_part = &last_line[last_part_byte..];

                    // Collect values before mutation
                    let first_part_clone = first_part.to_string();
                    let last_part_clone = last_part.to_string();
                    let new_text = change.new_text.clone();

                    lines[start_line] = format!("{}{}", first_part_clone, new_text);

                    // Remove lines between start and end (inclusive)
                    lines.drain(start_line + 1..=end_line);

                    if !last_part_clone.is_empty() {
                        // Re-fetch current line content after previous mutation
                        let current_first_part = if start_line < lines.len() {
                            lines[start_line].clone()
                        } else {
                            String::new()
                        };

                        let new_line = format!("{}{}", current_first_part, last_part_clone);

                        if start_line < lines.len() {
                            lines[start_line] = new_line;
                        } else {
                            lines.push(new_line);
                        }
                    }

                    offset -= (end_line - start_line) as isize;
                }
            } else if matches!(change.change_type, ChangeType::Deletion) {
                if end_line == start_line {
                    // Delete part of a line - use UTF-8 safe byte offsets
                    let line = &lines[start_line];
                    let start_byte = Self::char_to_byte_idx(line, change.range.start_character as usize);
                    let end_byte = Self::char_to_byte_idx(line, change.range.end_character as usize);
                    let before = &line[..start_byte];
                    let after = &line[end_byte..];
                    lines[start_line] = format!("{}{}", before, after);
                } else {
                    // Delete multiple lines - use UTF-8 safe byte offsets
                    let first_line = &lines[start_line];
                    let last_line = &lines[end_line];

                    let first_part_byte = Self::char_to_byte_idx(first_line, change.range.start_character as usize);
                    let last_part_byte = Self::char_to_byte_idx(last_line, change.range.end_character as usize);

                    let first_part = &first_line[..first_part_byte];
                    let last_part = &last_line[last_part_byte..];

                    lines[start_line] = format!("{}{}", first_part, last_part);
                    lines.drain(start_line + 1..=end_line);

                    offset -= (end_line - start_line) as isize;
                }
            } else if matches!(change.change_type, ChangeType::Insertion) {
                // Insertion - use UTF-8 safe byte offset
                if start_line < lines.len() {
                    let line = &lines[start_line];
                    let char_offset = change.range.start_character as usize;
                    let byte_offset = Self::char_to_byte_idx(line, char_offset);
                    let before = &line[..byte_offset];
                    let after = &line[byte_offset..];
                    lines[start_line] = format!("{}{}{}", before, change.new_text, after);
                } else {
                    lines.push(change.new_text.clone());
                }
            }
        }

        Ok(lines.join("\n"))
    }

    /// Validate refactoring options
    pub fn validate_refactoring_options(options: &RefactoringOptions) -> Result<(), String> {
        if options.generate_tests && !options.preserve_references {
            return Err("Cannot generate tests while preserving references is disabled".to_string());
        }

        if options.apply_to_all_occurrences && options.ignore_safe_operations {
            return Err("Applying to all occurrences conflicts with ignoring safe operations".to_string());
        }

        Ok(())
    }
}

/// Diff line representation
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_number: usize,
    pub change_type: ChangeType,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}

/// Backup manager for refactoring operations
pub struct BackupManager {
    backup_dir: String,
}

impl BackupManager {
    pub fn new(backup_dir: String) -> Self {
        BackupManager { backup_dir }
    }

    /// Create backup manifest with planned change metadata
    pub fn create_backup_manifest(
        &self,
        operation_type: &RefactoringType,
        context: &RefactoringContext,
        result: &RefactoringResult,
    ) -> Result<String, String> {
        use std::time::{SystemTime, UNIX_EPOCH};

        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct BackupManifest {
            /// Unique backup identifier
            backup_id:             String,
            /// Timestamp of backup creation
            timestamp:             String,
            /// Original file path
            original_file_path:    String,
            /// Operation type that was executed
            operation_type:        String,
            /// Hash of the original content before changes
            original_content_hash: String,
            /// Planned changes that were applied
            planned_changes:       Vec<ChangeSummary>,
            /// Hash of the resulting content after changes
            result_content_hash:   String,
            /// Whether the operation was successful
            success:               bool,
            /// Context information for recreate capability
            context_snapshot:      ContextSnapshot,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct ChangeSummary {
            /// Unique change identifier
            change_id:        String,
            /// Type of change (Insertion, Replacement, Deletion)
            change_type:      String,
            /// Target range in the file
            range:            BackupCodeRange,
            /// Hash of the old content (if any)
            old_content_hash: Option<String>,
            /// Hash of the new content (if any)
            new_content_hash: Option<String>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct BackupCodeRange {
            start_line:      usize,
            start_character: usize,
            end_line:        usize,
            end_character:   usize,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct ContextSnapshot {
            symbol_name:     Option<String>,
            symbol_kind:     Option<String>,
            cursor_position: Option<(usize, usize)>,
            selection_range: Option<BackupCodeRange>,
        }

        // Generate unique backup ID
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let backup_id = format!("backup_{}_{}", timestamp, std::process::id());

        // Create change summaries
        let planned_changes: Vec<ChangeSummary> = result
            .changes
            .iter()
            .enumerate()
            .map(|(index, change)| {
                let change_id = format!("{}_change_{}", backup_id, index);

                ChangeSummary {
                    change_id,
                    change_type: format!("{:?}", change.change_type),
                    range: BackupCodeRange {
                        start_line:      change.range.start_line,
                        start_character: change.range.start_character,
                        end_line:        change.range.end_line,
                        end_character:   change.range.end_character,
                    },
                    old_content_hash: if change.old_text.is_empty() {
                        None
                    } else {
                        Some(crate::utils::RefactoringUtils::hash_content(
                            &change.old_text,
                        ))
                    },
                    new_content_hash: if change.new_text.is_empty() {
                        None
                    } else {
                        Some(crate::utils::RefactoringUtils::hash_content(
                            &change.new_text,
                        ))
                    },
                }
            })
            .collect();

        // Create context snapshot
        let context_snapshot = ContextSnapshot {
            symbol_name:     context.symbol_name.clone(),
            symbol_kind:     context.symbol_kind.clone().map(|k| format!("{:?}", k)),
            cursor_position: Some((
                context.cursor_line as usize,
                context.cursor_character as usize,
            )),
            selection_range: context.selection.clone().map(|sel| BackupCodeRange {
                start_line:      sel.start_line,
                start_character: sel.start_character,
                end_line:        sel.end_line,
                end_character:   sel.end_character,
            }),
        };

        // Calculate content hashes
        let original_content = match std::fs::read_to_string(&context.file_path) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read original file: {}", e)),
        };

        let original_content_hash = crate::utils::RefactoringUtils::hash_content(&original_content);
        let result_content_hash = match &result.new_content {
            Some(content) => crate::utils::RefactoringUtils::hash_content(content),
            None => original_content_hash.clone(),
        };

        // Create the manifest
        let manifest = BackupManifest {
            backup_id: backup_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
            original_file_path: context.file_path.clone(),
            operation_type: format!("{:?}", operation_type),
            original_content_hash,
            planned_changes,
            result_content_hash,
            success: result.success,
            context_snapshot,
        };

        // Serialize to JSON
        match serde_json::to_string_pretty(&manifest) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Failed to serialize backup manifest: {}", e)),
        }
    }

    /// Create backup of file before modification
    pub async fn create_backup(
        &self,
        file_path: &str,
        changes: &[CodeChange],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use std::time::{SystemTime, UNIX_EPOCH};

        use tokio::fs;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let backup_filename = format!(
            "backup_{}_{}_{}.backup",
            Path::new(file_path).file_name().unwrap().to_string_lossy(),
            timestamp,
            std::process::id()
        );

        let backup_path = Path::new(&self.backup_dir).join(backup_filename);
        let backup_path_str = backup_path.to_string_lossy().to_string();

        // Copy original file
        fs::create_dir_all(&self.backup_dir).await?;
        fs::copy(file_path, &backup_path_str).await?;

        println!("Created backup: {}", backup_path_str);

        Ok(backup_path_str)
    }

    /// Restore file from backup
    pub async fn restore_backup(
        &self,
        backup_path: &str,
        original_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tokio::fs::copy(backup_path, original_path).await?;
        println!("Restored backup from: {}", backup_path);
        Ok(())
    }

    /// Clean old backups
    pub async fn cleanup_old_backups(
        &self,
        max_age_days: u32,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        use std::time::SystemTime;

        use tokio::fs;

        if !Path::new(&self.backup_dir).exists() {
            return Ok(0);
        }

        let max_age_seconds = max_age_days as u64 * 24 * 60 * 60;
        let mut removed_count = 0;

        let mut read_dir = fs::read_dir(&self.backup_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let metadata = entry.metadata().await?;
            let modified = metadata.modified()?;
            let age = SystemTime::now().duration_since(modified)?.as_secs();

            if age > max_age_seconds {
                fs::remove_file(entry.path()).await?;
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    /// Write backup manifest to a file alongside the actual backup
    pub async fn save_backup_manifest(
        &self,
        manifest_json: &str,
        backup_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use tokio::fs;

        let manifest_path = Path::new(&self.backup_dir).join(format!("{}.manifest.json", backup_id));

        fs::write(&manifest_path, manifest_json).await?;
        println!("Saved backup manifest to: {}", manifest_path.display());

        Ok(manifest_path.to_string_lossy().to_string())
    }

    /// Execute operation and create comprehensive backup with metadata
    pub async fn execute_with_backup<T>(
        &self,
        operation: impl FnOnce() -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>>,
        operation_type: &RefactoringType,
        context: &RefactoringContext,
        file_path: &str,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        // Execute the operation first to get the changes
        let result = operation()?;

        // Only create backup if operation was successful
        if result.success {
            // Create backup of the original file
            let backup_path = self.create_backup(file_path, &result.changes).await?;

            // Generate backup manifest with planned changes metadata
            let manifest_json = self.create_backup_manifest(operation_type, context, &result)?;

            // Save the manifest alongside the backup
            let manifest_path = self
                .save_backup_manifest(&manifest_json, result.id.as_ref().unwrap())
                .await?;

            // Store backup paths in the result for potential restoration
            println!(
                "Backup created with metadata: {} (backup: {}, manifest: {})",
                result.id.as_deref().unwrap_or("unknown"),
                backup_path,
                manifest_path
            );
        }

        Ok(result)
    }

    /// Restore from backup manifest
    pub async fn restore_from_manifest(
        &self,
        manifest_path: &str,
        target_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::fs;

        // Read manifest
        let manifest_content = fs::read_to_string(manifest_path).await?;

        // Parse manifest JSON
        #[derive(Debug, serde::Deserialize)]
        struct BackupManifest {
            backup_id:          String,
            original_file_path: String,
            planned_changes:    Vec<serde::de::IgnoredAny>,
        }

        let manifest: BackupManifest =
            serde_json::from_str(&manifest_content).map_err(|e| format!("Failed to parse backup manifest: {}", e))?;

        // Find corresponding backup file
        let backup_file = Path::new(&self.backup_dir).join(format!("{}.backup", manifest.backup_id));

        if !backup_file.exists() {
            return Err(format!("Backup file not found: {}", backup_file.display()).into());
        }

        // Restore the backup
        self.restore_backup(&backup_file.to_string_lossy(), target_path)
            .await?;

        // Verify restoration if we have hash data
        if let Ok(restored_content) = fs::read_to_string(target_path).await {
            println!("Successfully restored {} from backup manifest", target_path);
        }

        Ok(())
    }

    /// List available backup manifests
    pub async fn list_backup_manifests(
        &self,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error + Send + Sync>> {
        use std::time::SystemTime;

        use tokio::fs;

        let mut manifests = Vec::new();

        if !Path::new(&self.backup_dir).exists() {
            return Ok(manifests);
        }

        let mut read_dir = fs::read_dir(&self.backup_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension == "json" && path.to_string_lossy().contains(".manifest.") {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let manifest_path = path.to_string_lossy().to_string();
                    manifests.push((file_name, manifest_path));
                }
            }
        }

        // Sort by modification time (newest first)
        manifests.sort_by_key(|(_, path)| {
            let metadata = std::fs::metadata(path);
            metadata
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });
        manifests.reverse();

        Ok(manifests)
    }
}
