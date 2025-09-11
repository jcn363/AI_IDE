//! Batch refactoring operations

use crate::types::*;

/// Batch refactoring operation result
#[derive(Debug, Clone)]
pub struct BatchRefactoringResult {
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub results: Vec<RefactoringOperationResult>,
    pub errors: Vec<String>,
}

/// Batch operation configuration
#[derive(Debug, Clone)]
pub struct BatchOperation {
    pub refactoring_type: RefactoringType,
    pub context: RefactoringContext,
    pub options: RefactoringOptions,
    pub dependencies: Vec<String>,
}

/// Backup strategy for batch operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupStrategy {
    NoBackup,
    SingleBackup,
    PerOperationBackup,
    GitBackup,
}

/// Batch refactoring configuration
#[derive(Debug, Clone)]
pub struct BatchRefactoring {
    pub operations: Vec<BatchOperation>,
    pub validate_independently: bool,
    pub stop_on_first_error: bool,
    pub backup_strategy: BackupStrategy,
}

impl Default for BatchRefactoring {
    fn default() -> Self {
        BatchRefactoring {
            operations: Vec::new(),
            validate_independently: true,
            stop_on_first_error: false,
            backup_strategy: BackupStrategy::NoBackup,
        }
    }
}
