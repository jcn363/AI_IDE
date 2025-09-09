use crate::analysis::ConflictSeverity;
use crate::operations::RefactoringOperationFactory;
use crate::types::*;
use crate::utils::BackupManager;

/// Handles batch refactoring operations
pub struct BatchRefactoringHandler {
    validator: BatchValidator,
}

impl BatchRefactoringHandler {
    pub fn new() -> Self {
        BatchRefactoringHandler {
            validator: BatchValidator::new(),
        }
    }

    /// Execute a batch of refactoring operations
    pub async fn execute_batch(
        &self,
        batch: &BatchRefactoring,
        config: &RefactoringConfiguration,
    ) -> Result<BatchResult, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        let mut created_backups: Vec<String> = Vec::new();

        // Validate batch operations
        let validation = self.validator.validate_batch(batch).await?;

        if !validation.is_valid && batch.validate_independently {
            return Err(
                format!("Batch validation failed: {}", validation.errors.join(", ")).into(),
            );
        }

        let mut results = Vec::new();
        let mut rollback_operations = Vec::new();
        let mut success_count = 0;

        // Execute operations
        for operation in &batch.operations {
            // Check if we should stop on first error
            if batch.stop_on_first_error && !validation.is_valid {
                break;
            }

            match self.execute_single_operation(operation, config).await {
                Ok(result) => {
                    results.push(BatchOperationResult {
                        operation: operation.clone(),
                        result: Some(result.clone()),
                        error: None,
                        status: OperationStatus::Success,
                    });

                    if result.success {
                        success_count += 1;

                        // Store for potential rollback
                        if matches!(batch.backup_strategy, BackupStrategy::PerOperationBackup) {
                            rollback_operations.push(RollbackOperation {
                                operation: operation.clone(),
                                original_changes: result.changes.clone(),
                            });
                        }
                    } else {
                        results.last_mut().unwrap().status = OperationStatus::Failed;
                    }
                }
                Err(error) => {
                    results.push(BatchOperationResult {
                        operation: operation.clone(),
                        result: None,
                        error: Some(error.to_string()),
                        status: OperationStatus::Failed,
                    });

                    if batch.stop_on_first_error {
                        break;
                    }
                }
            }
        }

        // Create backups based on strategy
        match batch.backup_strategy {
            BackupStrategy::SingleBackup => {
                if success_count > 0 {
                    match self.create_batch_backup(&batch, &results).await {
                        Ok(backup_paths) => {
                            created_backups.extend(backup_paths);
                        }
                        Err(e) => {
                            println!("Warning: Failed to create unified backup: {}", e);
                        }
                    }
                }
            }
            BackupStrategy::PerOperationBackup => {
                // Create individual backups for successful operations
                for result in &results {
                    if result.status == OperationStatus::Success {
                        if let Some(refaction_result) = &result.result {
                            rollback_operations.push(RollbackOperation {
                                operation: result.operation.clone(),
                                original_changes: refaction_result.changes.clone(),
                            });
                        }
                    }
                }
            }
            BackupStrategy::NoBackup => {
                // No backup needed
            }
            BackupStrategy::GitBackup => {
                // Future enhancement: integrate with git for backup
                println!("Git backup strategy not yet implemented");
            }
        }

        Ok(BatchResult {
            operation_results: results,
            total_operations: batch.operations.len(),
            successful_operations: success_count,
            failed_operations: batch.operations.len() - success_count,
            backup_created: !created_backups.is_empty(),
            rollback_operations,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Execute a single operation within a batch
    async fn execute_single_operation(
        &self,
        operation: &BatchOperation,
        config: &RefactoringConfiguration,
    ) -> Result<RefactoringResult, Box<dyn std::error::Error + Send + Sync>> {
        let refactoring_op =
            RefactoringOperationFactory::create_operation(&operation.refactoring_type)?;

        // Merge batch options with operation options
        let merged_options = Self::merge_options(&config.default_options, &operation.options);

        refactoring_op
            .execute(&operation.context, &merged_options)
            .await
    }

    /// Create backup for batch operation using BackupManager
    async fn create_batch_backup(
        &self,
        batch: &BatchRefactoring,
        results: &[BatchOperationResult],
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::utils::BackupManager;

        println!(
            "Creating batch backup for {} operations",
            batch.operations.len()
        );

        // Collect all changes from successful operations
        let all_changes: Vec<_> = results
            .iter()
            .filter(|r| r.status == OperationStatus::Success)
            .filter_map(|r| r.result.as_ref())
            .flat_map(|r| r.changes.iter())
            .cloned()
            .collect();

        if all_changes.is_empty() {
            return Ok(Vec::new());
        }

        // Group changes by file to avoid duplicate backups
        let mut files_to_backup: std::collections::HashMap<String, Vec<CodeChange>> =
            std::collections::HashMap::new();
        for change in &all_changes {
            files_to_backup
                .entry(change.file_path.clone())
                .or_insert(Vec::new())
                .push(change.clone());
        }

        println!("Backing up {} files", files_to_backup.len());

        let backup_manager = BackupManager::new(".refactor-backups".to_string());
        let mut backup_paths = Vec::new();

        // Create backups sequentially to ensure consistency
        for (file_path, changes) in files_to_backup {
            match backup_manager.create_backup(&file_path, &changes).await {
                Ok(backup_path) => {
                    backup_paths.push(backup_path);
                }
                Err(e) => {
                    println!("Warning: Failed to backup {}: {}", file_path, e);
                    // Continue with other backups even if one fails
                }
            }
        }

        // Create batch manifest for rollback information
        if !backup_paths.is_empty() {
            self.create_batch_manifest(batch, &backup_paths, results)
                .await?;
        }

        Ok(backup_paths)
    }

    /// Create manifest file describing the batch operation for rollback purposes
    async fn create_batch_manifest(
        &self,
        batch: &BatchRefactoring,
        backup_paths: &[String],
        results: &[BatchOperationResult],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        use tokio::fs;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let manifest_path = format!(
            ".refactor-backups/batch_manifest_{}_{}.json",
            timestamp,
            std::process::id()
        );
        let manifest_data = serde_json::json!({
            "timestamp": timestamp,
            "total_operations": batch.operations.len(),
            "backup_paths": backup_paths,
            "operations": batch.operations,
            "results": results.iter().filter(|r| r.status == OperationStatus::Success)
                .map(|r| serde_json::json!({
                    "operation": r.operation,
                    "success": r.result.as_ref().map(|res| res.success).unwrap_or(false),
                    "changes_count": r.result.as_ref().map(|res| res.changes.len()).unwrap_or(0)
                }))
                .collect::<Vec<_>>()
        });

        let content = serde_json::to_string_pretty(&manifest_data)?;
        fs::write(&manifest_path, content).await?;

        Ok(())
    }

    /// Execute batch with dependency ordering for safe batch operations
    pub async fn execute_batch_with_dependencies(
        &self,
        batch: &BatchRefactoring,
        config: &RefactoringConfiguration,
    ) -> Result<BatchResult, Box<dyn std::error::Error + Send + Sync>> {
        // First validate and order operations based on dependencies
        let validation = self.validator.validate_batch(batch).await?;
        let ordered_operations = self.order_operations_by_dependencies(&batch.operations)?;

        // Create a new batch with ordered operations
        let ordered_batch = BatchRefactoring {
            operations: ordered_operations,
            validate_independently: batch.validate_independently,
            stop_on_first_error: batch.stop_on_first_error,
            backup_strategy: batch.backup_strategy.clone(),
        };

        // Execute with dependency ordering
        self.execute_batch(&ordered_batch, config).await
    }

    /// Order operations based on their dependencies using topological sort
    fn order_operations_by_dependencies(
        &self,
        operations: &[BatchOperation],
    ) -> Result<Vec<BatchOperation>, Box<dyn std::error::Error + Send + Sync>> {
        use std::collections::{HashMap, VecDeque};

        let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut incoming_edges: HashMap<String, usize> = HashMap::new();
        let mut operation_map: HashMap<String, &BatchOperation> = HashMap::new();

        // Build dependency graph (operation -> list of operations that depend on it)
        for operation in operations {
            let op_key = format!(
                "{:?}_{:?}",
                operation.refactoring_type, operation.context.file_path
            );
            operation_map.insert(op_key.clone(), operation);

            // Initialize if not present
            dependency_graph.entry(op_key.clone()).or_insert(Vec::new());
            incoming_edges.entry(op_key.clone()).or_default();

            // Add dependencies (operations this operation depends on)
            for dependency in &operation.dependencies {
                // Find the operation this dependency refers to
                if let Some(depending_op) = operations.iter().find(|op| {
                    format!("{:?}", op.refactoring_type) == *dependency
                        || format!("{:?}_{:?}", op.refactoring_type, op.context.file_path)
                            == *dependency
                }) {
                    let depending_key = format!(
                        "{:?}_{:?}",
                        depending_op.refactoring_type, depending_op.context.file_path
                    );
                    dependency_graph
                        .entry(depending_key)
                        .or_insert(Vec::new())
                        .push(op_key.clone());
                    *incoming_edges.entry(op_key.clone()).or_default() += 1;
                }
            }
        }

        // Topological sort using Kahn's algorithm
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut ordered = Vec::new();

        // Start with operations that have no dependencies
        for (op_key, &count) in &incoming_edges {
            if count == 0 {
                queue.push_back(op_key.clone());
            }
        }

        while let Some(op_key) = queue.pop_front() {
            if let Some(operation) = operation_map.get(&op_key) {
                ordered.push((*operation).clone());
            }

            // Decrease the count of operations that depend on this operation
            if let Some(dependents) = dependency_graph.get(&op_key) {
                for dependent in dependents {
                    let count = incoming_edges.get_mut(dependent).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        // Check for cycles (if not all operations were included in the ordering)
        if ordered.len() != operations.len() {
            return Err("Circular dependency detected in batch operations".into());
        }

        Ok(ordered)
    }

    /// Execute batch with conflict resolution
    pub async fn execute_batch_with_conflicts(
        &self,
        batch: &BatchRefactoring,
        config: &RefactoringConfiguration,
    ) -> Result<BatchResult, Box<dyn std::error::Error + Send + Sync>> {
        // Analyze potential conflicts before execution
        let analysis = crate::analysis::RefactoringAnalyzer::new();
        let batch_analysis = analysis.analyze_batch(batch).await?;

        if !batch_analysis.recommended_batch_safe {
            println!(
                "Warning: Batch contains conflicts. Operations may not be safe to run together:"
            );
            for conflict in &batch_analysis.conflicts {
                println!(
                    "  - {} <-> {}: {}",
                    conflict.operation1, conflict.operation2, conflict.description
                );
            }
        }

        // Proceed with execution if conflicts exist but severity is low to medium
        let has_critical_conflict = batch_analysis
            .conflicts
            .iter()
            .any(|c| matches!(c.severity, ConflictSeverity::Critical));

        if has_critical_conflict {
            return Err("Critical conflicts detected in batch operations. Refusing to execute unsafe batch.".into());
        }

        // Execute with dependency ordering
        self.execute_batch_with_dependencies(batch, config).await
    }

    /// Get backup information for a file
    pub async fn get_backup_info(
        &self,
        original_file: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let backup_path = self
            .find_backup_for_file(
                &crate::utils::BackupManager::new(".refactor-backups".to_string()),
                original_file,
            )
            .await;
        Ok(backup_path)
    }

    /// List all backup files
    pub async fn list_backups(
        &self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use std::path::Path;
        use tokio::fs;

        let backup_dir = Path::new(".refactor-backups");
        let mut backups = Vec::new();

        if backup_dir.exists() {
            let mut read_dir = fs::read_dir(backup_dir).await?;
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                if let Some(filename) = entry.path().file_name().and_then(|n| n.to_str()) {
                    backups.push(filename.to_string());
                }
            }
        }

        Ok(backups)
    }

    /// Verify backup integrity
    pub async fn verify_backup_integrity(
        &self,
        backup_path: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        use std::path::Path;
        use tokio::fs;

        let path = Path::new(backup_path);
        if !path.exists() {
            return Ok(false);
        }

        // Basic integrity check - file exists and is readable
        match fs::read_to_string(path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Create recovery plan for failed batch operations
    pub async fn create_recovery_plan(
        &self,
        failed_operations: &[BatchOperationResult],
    ) -> Result<
        std::collections::HashMap<String, Vec<String>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        use std::collections::HashMap;

        let mut recovery_plan: HashMap<String, Vec<String>> = HashMap::new();

        for failed_operation in failed_operations
            .iter()
            .filter(|r| r.status == OperationStatus::Failed)
        {
            let operation_type = format!("{:?}", failed_operation.operation.refactoring_type);
            recovery_plan
                .entry(operation_type)
                .or_insert(Vec::new())
                .push(
                    failed_operation
                        .error
                        .clone()
                        .unwrap_or_else(|| "Unknown error".to_string()),
                );
        }

        Ok(recovery_plan)
    }

    /// Check if backup should be created based on strategy
    fn should_create_backup(&self, strategy: &BackupStrategy) -> bool {
        !matches!(strategy, BackupStrategy::NoBackup)
    }

    /// Merge default options with operation-specific options, deep-merging extra_options
    fn merge_options(
        default: &RefactoringOptions,
        operation: &RefactoringOptions,
    ) -> RefactoringOptions {
        // Deep merge extra_options if both contain values
        let extra_options = match (&default.extra_options, &operation.extra_options) {
            (Some(default_extra), Some(operation_extra)) => {
                let mut merged = std::collections::HashMap::new();
                // First copy all from default extra_options
                for (key, value) in default_extra {
                    merged.insert(key.clone(), value.clone());
                }
                // Then override with operation-specific values
                for (key, value) in operation_extra {
                    merged.insert(key.clone(), value.clone());
                }
                Some(merged)
            }
            (Some(default_extra), None) => Some(default_extra.clone()),
            (None, Some(operation_extra)) => Some(operation_extra.clone()),
            (None, None) => None,
        };

        RefactoringOptions {
            create_backup: operation.create_backup || default.create_backup,
            generate_tests: operation.generate_tests || default.generate_tests,
            apply_to_all_occurrences: operation.apply_to_all_occurrences
                || default.apply_to_all_occurrences,
            preserve_references: operation.preserve_references || default.preserve_references,
            ignore_safe_operations: operation.ignore_safe_operations
                || default.ignore_safe_operations,
            extra_options,
        }
    }

    /// Rollback operations if needed with proper error handling and partial rollback support
    pub async fn rollback_batch(
        &self,
        operations: &[RollbackOperation],
    ) -> Result<BatchRollbackResult, Box<dyn std::error::Error + Send + Sync>> {
        use crate::utils::BackupManager;
        use std::collections::HashMap;

        println!("Rolling back {} operations", operations.len());

        let backup_manager = BackupManager::new(".refactor-backups".to_string());
        let mut rollback_results = Vec::new();
        let mut successful_rollbacks = 0;
        let mut failed_rollbacks = 0;

        // Group operations by backup file to avoid duplicate restores
        let mut backup_to_operations: HashMap<String, Vec<&RollbackOperation>> = HashMap::new();
        for operation in operations {
            // For now, we need to reconstruct the backup path from operation metadata
            // In a real implementation, we'd store the backup path in the RollbackOperation
            for change in &operation.original_changes {
                let backup_key = format!("{}_{}", change.file_path, std::process::id());
                backup_to_operations
                    .entry(backup_key)
                    .or_insert(Vec::new())
                    .push(operation);
            }
        }

        // Rollback in reverse order to handle dependencies properly
        let mut sorted_backups: Vec<_> = backup_to_operations.keys().collect();
        sorted_backups.sort_by(|a, b| b.cmp(a)); // Reverse alphabetical sort as rough dependency order

        for backup_file in sorted_backups {
            if let Some(file_operations) = backup_to_operations.get(backup_file) {
                for operation in file_operations {
                    // Try to rollback files from this operation
                    for change in &operation.original_changes {
                        let backup_path = self
                            .find_backup_for_file(&backup_manager, &change.file_path)
                            .await;

                        match backup_path {
                            Some(path) => {
                                match backup_manager
                                    .restore_backup(&path, &change.file_path)
                                    .await
                                {
                                    Ok(()) => {
                                        successful_rollbacks += 1;
                                        rollback_results.push(BatchOperationResult {
                                            operation: operation.operation.clone(),
                                            result: Some(RefactoringResult {
                                                id: None,
                                                success: true,
                                                changes: vec![], // Empty as we're restoring original state
                                                error_message: None,
                                                warnings: vec![
                                                    "File rolled back successfully".to_string()
                                                ],
                                                new_content: None,
                                            }),
                                            error: None,
                                            status: OperationStatus::Success,
                                        });
                                    }
                                    Err(e) => {
                                        failed_rollbacks += 1;
                                        println!("Failed to rollback {}: {}", change.file_path, e);
                                        rollback_results.push(BatchOperationResult {
                                            operation: operation.operation.clone(),
                                            result: None,
                                            error: Some(format!("Rollback failed: {}", e)),
                                            status: OperationStatus::Failed,
                                        });
                                    }
                                }
                            }
                            None => {
                                let error_msg =
                                    format!("No backup found for file: {}", change.file_path);
                                failed_rollbacks += 1;
                                println!("Warning: {}", error_msg);
                                rollback_results.push(BatchOperationResult {
                                    operation: operation.operation.clone(),
                                    result: None,
                                    error: Some(error_msg),
                                    status: OperationStatus::Failed,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Cleanup backup files after successful rollback
        if failed_rollbacks == 0 {
            let _ = backup_manager.cleanup_old_backups(0).await; // Clean all backups if all rollbacks succeeded
        }

        Ok(BatchRollbackResult {
            operation_results: rollback_results,
            successful_rollbacks,
            failed_rollbacks,
            total_operations: operations.len(),
            is_complete_rollback: failed_rollbacks == 0,
        })
    }

    /// Find backup file for a given original file path
    async fn find_backup_for_file(
        &self,
        backup_manager: &BackupManager,
        original_path: &str,
    ) -> Option<String> {
        use std::path::Path;
        use tokio::fs;

        let backup_dir = Path::new(".refactor-backups");
        if !backup_dir.exists() {
            return None;
        }

        // List backup files and find one that matches the original file
        if let Ok(mut read_dir) = fs::read_dir(backup_dir).await {
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Match backup files by prefix and file extension
                    if let Some(original_filename) = Path::new(original_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                    {
                        if filename.starts_with(&format!("backup_{}", original_filename)) {
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        None
    }
}

/// Validator for batch operations
pub struct BatchValidator;

impl BatchValidator {
    pub fn new() -> Self {
        BatchValidator
    }

    /// Validate batch operations before execution
    pub async fn validate_batch(
        &self,
        batch: &BatchRefactoring,
    ) -> Result<BatchValidation, Box<dyn std::error::Error + Send + Sync>> {
        let mut is_valid = true;
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for duplicate operations
        let mut seen_operations = std::collections::HashSet::new();
        for operation in &batch.operations {
            let key = format!(
                "{:?}@{:?}",
                operation.refactoring_type, operation.context.file_path
            );

            if seen_operations.contains(&key) {
                is_valid = false;
                errors.push(format!("Duplicate operation detected: {}", key));
            } else {
                seen_operations.insert(key);
            }
        }

        // Check dependencies
        for operation in &batch.operations {
            for dependency in &operation.dependencies {
                let has_dependency = batch
                    .operations
                    .iter()
                    .any(|op| format!("{:?}", op.refactoring_type) == *dependency);

                if !has_dependency {
                    warnings.push(format!(
                        "Dependency '{}' not found for operation {:?}",
                        dependency, operation.refactoring_type
                    ));
                }
            }
        }

        Ok(BatchValidation {
            is_valid,
            errors,
            warnings,
        })
    }
}

/// Rollback operation information
#[derive(Debug, Clone)]
pub struct RollbackOperation {
    pub operation: BatchOperation,
    pub original_changes: Vec<CodeChange>,
}

/// Result of a batch operation
#[derive(Debug, Clone)]
pub struct BatchOperationResult {
    pub operation: BatchOperation,
    pub result: Option<RefactoringResult>,
    pub error: Option<String>,
    pub status: OperationStatus,
}

/// Status of operation execution
#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Pending,
    Success,
    Failed,
}

/// Result of batch refactoring execution
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub operation_results: Vec<BatchOperationResult>,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub backup_created: bool,
    pub rollback_operations: Vec<RollbackOperation>,
    pub execution_time_ms: u64,
}

/// Validation result for batch operations
#[derive(Debug, Clone)]
pub struct BatchValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Result of rollback operation
#[derive(Debug, Clone)]
pub struct BatchRollbackResult {
    pub operation_results: Vec<BatchOperationResult>,
    pub successful_rollbacks: usize,
    pub failed_rollbacks: usize,
    pub total_operations: usize,
    pub is_complete_rollback: bool,
}
