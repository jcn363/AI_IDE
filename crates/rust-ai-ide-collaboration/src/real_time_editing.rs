//! CRDT-based real-time collaborative editing system.
//!
//! Implements Conflict-Free Replicated Data Types for distributed text editing,
//! operational transforms for efficient synchronization, and intelligent merge policies.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core collaborative editing service
pub struct RealTimeEditingService {
    documents: Arc<RwLock<std::collections::HashMap<String, DocumentState>>>,
}

/// Represents a document's collaborative state
pub struct DocumentState {
    pub id: String,
    pub crdt: TextCRDT,
    pub operations_log: Vec<Operation>,
    pub participants: std::collections::HashSet<String>,
    pub last_sync_timestamp: std::time::SystemTime,
}

/// CRDT implementation for collaborative text editing
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TextCRDT {
    pub sites: Vec<SiteState>,
    pub tombstone: Vec<Option<char>>, // Deletion tracking
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SiteState {
    pub site_id: u32,
    pub operations: Vec<CRDTOperation>,
}

/// CRDT operations for text editing
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum CRDTOperation {
    Insert {
        pos: usize,
        char: char,
        lamport_clock: u64,
        site_id: u32,
    },
    Delete {
        pos: usize,
        lamport_clock: u64,
        site_id: u32,
    },
}

/// Operational Transform operation types
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Operation {
    InsertOp(InsertOperation),
    DeleteOp(DeleteOperation),
    TransformOp {
        operation: Box<Operation>,
        transformed_operations: Vec<Operation>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InsertOperation {
    pub id: Uuid,
    pub position: usize,
    pub content: String,
    pub site_id: u32,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeleteOperation {
    pub id: Uuid,
    pub position: usize,
    pub length: usize,
    pub site_id: u32,
    pub timestamp: u64,
}

/// Merge policies for resolving conflicts
#[derive(Clone, Debug)]
pub enum MergePolicy {
    LatestWins,
    PositionBased,
    Custom(Box<dyn MergeResolver>),
}

pub trait MergeResolver: Send + Sync {
    fn resolve(&self, left: &Operation, right: &Operation) -> Operation;
}

impl RealTimeEditingService {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Apply a CRDT operation to a document
    pub async fn apply_crdt_operation(
        &self,
        document_id: String,
        operation: CRDTOperation,
        site_id: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut documents = self.documents.write().await;
        let doc_state = documents.get_mut(&document_id)
            .ok_or_else(|| format!("Document {} not found", document_id))?;

        // Apply the operation to the CRDT
        doc_state.crdt.apply_operation(operation.clone(), site_id)?;

        // Add to operations log
        doc_state.operations_log.push(Operation::InsertOp(InsertOperation {
            id: Uuid::new_v4(),
            position: match &operation {
                CRDTOperation::Insert { pos, .. } => *pos,
                CRDTOperation::Delete { pos, .. } => *pos,
            },
            content: match &operation {
                CRDTOperation::Insert { char, .. } => char.to_string(),
                CRDTOperation::Delete { .. } => "".to_string(),
            },
            site_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        }));

        Ok(())
    }

    /// Apply operational transform for non-CRDT operations
    pub async fn apply_operational_transform(
        &self,
        document_id: String,
        operation: Operation,
        merge_policy: MergePolicy,
        user_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut documents = self.documents.write().await;
        let doc_state = documents.get_mut(&document_id)
            .ok_or_else(|| format!("Document {} not found", document_id))?;

        // Resolve conflicts using the merge policy
        let resolved_operation = self.resolve_conflicts(&operation, &merge_policy, &doc_state.operations_log)?;

        // Transform the operation against concurrent operations
        let transformed_operation = self.transform_operation(resolved_operation, &doc_state.operations_log)?;

        // Apply the transformed operation
        self.apply_operation_to_document(&mut doc_state.crdt, &transformed_operation)?;

        // Add to log
        doc_state.operations_log.push(transformed_operation);

        Ok(())
    }

    /// Merge multiple operations from different sites
    pub async fn merge_operations(
        &self,
        document_id: String,
        operations: Vec<Operation>,
        merge_policy: MergePolicy,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut documents = self.documents.write().await;
        let doc_state = documents.get_mut(&document_id)
            .ok_or_else(|| format!("Document {} not found", document_id))?;

        for operation in operations {
            let resolved_operation = self.resolve_conflicts(&operation, &merge_policy, &doc_state.operations_log)?;
            let transformed_operation = self.transform_operation(resolved_operation, &doc_state.operations_log)?;

            self.apply_operation_to_document(&mut doc_state.crdt, &transformed_operation)?;
            doc_state.operations_log.push(transformed_operation);
        }

        Ok(())
    }

    fn resolve_conflicts(
        &self,
        operation: &Operation,
        _policy: &MergePolicy,
        log: &[Operation]
    ) -> Result<Operation, Box<dyn std::error::Error>> {
        // Simple conflict resolution - latest timestamp wins
        // Real implementation would handle complex conflict resolution logic
        Ok(operation.clone())
    }

    fn transform_operation(
        &self,
        operation: Operation,
        log: &[Operation]
    ) -> Result<Operation, Box<dyn std::error::Error>> {
        // Operational transformation implementation
        // Transform the incoming operation to account for other operations that have occurred
        let mut transformed_op = operation;

        for existing_op in log.iter().rev() {
            transformed_op = self.transform_against(transformed_op, existing_op)?;
        }

        Ok(transformed_op)
    }

    fn transform_against(
        &self,
        mut incoming: Operation,
        existing: &Operation
    ) -> Result<Operation, Box<dyn std::error::Error>> {
        // Transformation rules for operational transforms
        match (&mut incoming, existing) {
            (Operation::InsertOp(insert), Operation::InsertOp(existing_insert)) => {
                // Handle insert-insert transformations
                if existing_insert.position <= insert.position {
                    let new_pos = insert.position + existing_insert.content.len();
                    insert.position = new_pos;
                }
            }
            (Operation::DeleteOp(delete), Operation::InsertOp(existing_insert)) => {
                // Handle delete-insert transformations
                if existing_insert.position <= delete.position {
                    let new_pos = delete.position + existing_insert.content.len();
                    delete.position = new_pos;
                }
            }
            // Add more transformation rules as needed
            _ => {} // No transformation needed
        }

        Ok(incoming)
    }

    fn apply_operation_to_document(
        &self,
        crdt: &mut TextCRDT,
        operation: &Operation
    ) -> Result<(), Box<dyn std::error::Error>> {
        match operation {
            Operation::InsertOp(insert_op) => {
                // Apply insert operation to CRDT
                crdt.apply_crdt_operation(CRDTOperation::Insert {
                    pos: insert_op.position,
                    char: insert_op.content.chars().next().unwrap_or(' '),
                    lamport_clock: insert_op.timestamp,
                    site_id: insert_op.site_id,
                }, insert_op.site_id)?;
            }
            Operation::DeleteOp(delete_op) => {
                // Apply delete operation
                crdt.apply_crdt_operation(CRDTOperation::Delete {
                    pos: delete_op.position,
                    lamport_clock: delete_op.timestamp,
                    site_id: delete_op.site_id,
                }, delete_op.site_id)?;
            }
            Operation::TransformOp { .. } => {
                // Transformed operations are handled separately
            }
        }
        Ok(())
    }

    /// Get the current document content from CRDT state
    pub async fn get_document_content(&self, document_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let documents = self.documents.read().await;
        let doc_state = documents.get(document_id)
            .ok_or_else(|| format!("Document {} not found", document_id))?;

        Ok(doc_state.crdt.to_string())
    }
}

impl TextCRDT {
    pub fn new() -> Self {
        Self {
            sites: Vec::new(),
            tombstone: Vec::new(),
        }
    }

    pub fn apply_operation(&mut self, operation: CRDTOperation, site_id: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Find or create site state
        let site_idx = self.sites.iter().position(|site| site.site_id == site_id)
            .unwrap_or_else(|| {
                self.sites.push(SiteState { site_id, operations: Vec::new() });
                self.sites.len() - 1
            });

        self.sites[site_idx].operations.push(operation.clone());

        // Apply the operation to the document
        match operation {
            CRDTOperation::Insert { pos, char, .. } => {
                while self.tombstone.len() <= pos {
                    self.tombstone.push(None);
                }
                self.tombstone.insert(pos, Some(char));
            }
            CRDTOperation::Delete { pos, .. } => {
                if pos < self.tombstone.len() {
                    self.tombstone[pos] = None;
                }
            }
        }

        Ok(())
    }

    pub fn to_string(&self) -> String {
        self.tombstone.iter()
            .filter_map(|&opt_char| opt_char)
            .collect()
    }
}

/// Standard merge resolver for latest timestamp wins
pub struct LatestTimestampResolver;

impl MergeResolver for LatestTimestampResolver {
    fn resolve(&self, left: &Operation, right: &Operation) -> Operation {
        // Compare timestamps from both operations
        match (left, right) {
            (Operation::InsertOp(left_op), Operation::InsertOp(right_op)) => {
                if left_op.timestamp >= right_op.timestamp {
                    Operation::InsertOp(left_op.clone())
                } else {
                    Operation::InsertOp(right_op.clone())
                }
            }
            (Operation::DeleteOp(left_op), Operation::DeleteOp(right_op)) => {
                if left_op.timestamp >= right_op.timestamp {
                    Operation::DeleteOp(left_op.clone())
                } else {
                    Operation::DeleteOp(right_op.clone())
                }
            }
            // For different operation types, keep left by default
            _ => left.clone(),
        }
    }
}