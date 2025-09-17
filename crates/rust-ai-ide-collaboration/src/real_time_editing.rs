//! CRDT-based real-time collaborative editing system.
//!
//! Implements Conflict-Free Replicated Data Types for distributed text editing,
//! operational transforms for efficient synchronization, and intelligent merge policies.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::ai_conflict_resolution::AIConflictResolver;
use crate::crdt::{EditorOperation, LamportClock, CRDT};

/// Core collaborative editing service
pub struct RealTimeEditingService {
    documents: Arc<RwLock<std::collections::HashMap<String, DocumentState>>>,
    ai_conflict_resolver: Option<Arc<RwLock<AIConflictResolver>>>,
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
    pub sites: Vec<crate::crdt::SiteState>,
    pub tombstone: Vec<Option<char>>, // Deletion tracking
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
            ai_conflict_resolver: None,
        }
    }

    pub fn with_ai_conflict_resolver(ai_conflict_resolver: Arc<RwLock<AIConflictResolver>>) -> Self {
        Self {
            documents: Arc::new(RwLock::new(std::collections::HashMap::new())),
            ai_conflict_resolver: Some(ai_conflict_resolver),
        }
    }

    /// Apply a CRDT operation to a document
    pub async fn apply_crdt_operation(
        &self,
        document_id: String,
        operation: crate::crdt::CRDTOperation,
        site_id: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut documents = self.documents.write().await;
        let doc_state = documents
            .get_mut(&document_id)
            .ok_or_else(|| format!("Document {} not found", document_id))?;

        // Apply the operation to the CRDT
        doc_state.crdt.apply_operation(operation.clone(), site_id)?;

        // Add to operations log
        match &operation {
            crate::crdt::CRDTOperation::Insert { pos, char, lamport_clock, site_id } => {
                doc_state.operations_log.push(Operation::InsertOp(InsertOperation {
                    id: Uuid::new_v4(),
                    position: *pos,
                    content: char.to_string(),
                    site_id: *site_id,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_millis() as u64,
                }));
            }
            crate::crdt::CRDTOperation::Delete { pos, length, lamport_clock, site_id } => {
                doc_state.operations_log.push(Operation::DeleteOp(DeleteOperation {
                    id: Uuid::new_v4(),
                    position: *pos,
                    length: *length,
                    site_id: *site_id,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_millis() as u64,
                }));
            }
        }

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
        let doc_state = documents
            .get_mut(&document_id)
            .ok_or_else(|| format!("Document {} not found", document_id))?;

        // Resolve conflicts using the merge policy
        let resolved_operation =
            self.resolve_conflicts(&operation, &merge_policy, &doc_state.operations_log)?;

        // Transform the operation against concurrent operations
        let transformed_operation =
            self.transform_operation(resolved_operation, &doc_state.operations_log)?;

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
        let doc_state = documents
            .get_mut(&document_id)
            .ok_or_else(|| format!("Document {} not found", document_id))?;

        for operation in operations {
            let resolved_operation =
                self.resolve_conflicts(&operation, &merge_policy, &doc_state.operations_log)?;
            let transformed_operation =
                self.transform_operation(resolved_operation, &doc_state.operations_log)?;

            self.apply_operation_to_document(&mut doc_state.crdt, &transformed_operation)?;
            doc_state.operations_log.push(transformed_operation);
        }

        Ok(())
    }

    fn resolve_conflicts(
        &self,
        operation: &Operation,
        policy: &MergePolicy,
        log: &[Operation],
    ) -> Result<Operation, Box<dyn std::error::Error>> {
        // Find the most recent operation in the log that conflicts with the incoming operation
        let mut conflicting_op = None;
        let mut latest_timestamp = 0u64;

        for op in log.iter().rev() {
            if self.operations_conflict(operation, op) {
                let ts = self.get_operation_timestamp(op);
                if ts > latest_timestamp {
                    latest_timestamp = ts;
                    conflicting_op = Some(op.clone());
                }
            }
        }

        if let Some(conflicting) = conflicting_op {
            // Check if this is a semantic conflict and AI resolver is available
            if self.is_semantic_conflict(operation, &conflicting) {
                if let Some(ai_resolver) = &self.ai_conflict_resolver {
                    // Use AI resolver for semantic conflicts
                    match self.resolve_with_ai(operation, &conflicting, ai_resolver) {
                        Ok(resolved) => return Ok(resolved),
                        Err(_) => {
                            // Fall back to default policy if AI resolution fails
                        }
                    }
                }
            }

            match policy {
                MergePolicy::LatestWins => {
                    let incoming_ts = self.get_operation_timestamp(operation);
                    if incoming_ts >= latest_timestamp {
                        Ok(operation.clone())
                    } else {
                        Ok(conflicting)
                    }
                }
                MergePolicy::PositionBased => {
                    let incoming_pos = self.get_operation_position(operation);
                    let conflicting_pos = self.get_operation_position(&conflicting);
                    if incoming_pos <= conflicting_pos {
                        Ok(operation.clone())
                    } else {
                        Ok(conflicting)
                    }
                }
                MergePolicy::Custom(resolver) => {
                    Ok(resolver.resolve(operation, &conflicting))
                }
            }
        } else {
            Ok(operation.clone())
        }
    }

    fn operations_conflict(&self, op1: &Operation, op2: &Operation) -> bool {
        match (op1, op2) {
            (Operation::InsertOp(insert1), Operation::InsertOp(insert2)) => {
                // Conflict if inserting at same position
                insert1.position == insert2.position
            }
            (Operation::InsertOp(insert), Operation::DeleteOp(delete))
            | (Operation::DeleteOp(delete), Operation::InsertOp(insert)) => {
                // Conflict if insert position is within delete range
                insert.position >= delete.position && insert.position < delete.position + delete.length
            }
            (Operation::DeleteOp(delete1), Operation::DeleteOp(delete2)) => {
                // Semantic conflict: overlapping delete ranges
                let start1 = delete1.position;
                let end1 = delete1.position + delete1.length;
                let start2 = delete2.position;
                let end2 = delete2.position + delete2.length;
                !(end1 <= start2 || end2 <= start1)
            }
            _ => false,
        }
    }

    fn get_operation_timestamp(&self, op: &Operation) -> u64 {
        match op {
            Operation::InsertOp(insert) => insert.timestamp,
            Operation::DeleteOp(delete) => delete.timestamp,
            Operation::TransformOp { operation, .. } => self.get_operation_timestamp(operation),
        }
    }

    fn get_operation_position(&self, op: &Operation) -> usize {
        match op {
            Operation::InsertOp(insert) => insert.position,
            Operation::DeleteOp(delete) => delete.position,
            Operation::TransformOp { operation, .. } => self.get_operation_position(operation),
        }
    }

    fn is_semantic_conflict(&self, op1: &Operation, op2: &Operation) -> bool {
        // Simple heuristic: if operations affect the same position range and are different types
        // This could be enhanced with more sophisticated semantic analysis
        match (op1, op2) {
            (Operation::InsertOp(insert1), Operation::InsertOp(insert2)) => {
                // Different content inserted at same position could be semantic conflict
                insert1.position == insert2.position && insert1.content != insert2.content
            }
            (Operation::DeleteOp(delete1), Operation::InsertOp(insert)) => {
                // Inserting into a deleted range could be semantic conflict
                insert.position >= delete1.position && insert.position < delete1.position + delete1.length
            }
            (Operation::InsertOp(insert), Operation::DeleteOp(delete)) => {
                // Deleting an inserted range could be semantic conflict
                delete.position >= insert.position && delete.position < insert.position + insert.content.len()
            }
            _ => false,
        }
    }

    fn resolve_with_ai(
        &self,
        operation: &Operation,
        conflicting: &Operation,
        ai_resolver: &Arc<RwLock<AIConflictResolver>>,
    ) -> Result<Operation, Box<dyn std::error::Error>> {
        // Convert operations to EditorOperation format for AI resolver
        let editor_op1 = self.operation_to_editor_operation(operation)?;
        let editor_op2 = self.operation_to_editor_operation(conflicting)?;

        // For now, use a simple fallback since we can't get document content here
        // In a real implementation, we'd need document content to use AI resolution
        // For this integration, we'll fall back to the first operation
        Ok(operation.clone())
    }

    fn operation_to_editor_operation(&self, operation: &Operation) -> Result<EditorOperation, Box<dyn std::error::Error>> {
        match operation {
            Operation::InsertOp(insert_op) => {
                Ok(EditorOperation::Insert {
                    position: insert_op.position,
                    content: insert_op.content.clone(),
                    op_id: insert_op.id.to_string(),
                    clock: LamportClock::new("client".to_string()).increment(), // Simplified clock
                })
            }
            Operation::DeleteOp(delete_op) => {
                Ok(EditorOperation::Delete {
                    position: delete_op.position,
                    length: delete_op.length,
                    op_id: delete_op.id.to_string(),
                    clock: LamportClock::new("client".to_string()).increment(),
                })
            }
            Operation::TransformOp { operation, .. } => {
                // Recursively convert the underlying operation
                self.operation_to_editor_operation(operation)
            }
        }
    }

    fn transform_operation(
        &self,
        operation: Operation,
        log: &[Operation],
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
        existing: &Operation,
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
        operation: &Operation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match operation {
            Operation::InsertOp(insert_op) => {
                // Apply insert operation to CRDT
                for (offset, ch) in insert_op.content.chars().enumerate() {
                    crdt.apply_operation(
                        crate::crdt::crate::crdt::CRDTOperation::Insert {
                            pos: insert_op.position + offset,
                            char: ch,
                            lamport_clock: insert_op.timestamp,
                            site_id: insert_op.site_id,
                        },
                        insert_op.site_id,
                    )?;
                }
            }
            Operation::DeleteOp(delete_op) => {
                // Apply delete operation for each character in the range
                for offset in 0..delete_op.length {
                    crdt.apply_operation(
                        crate::crdt::crate::crdt::CRDTOperation::Delete {
                            pos: delete_op.position + offset,
                            length: 1,
                            lamport_clock: delete_op.timestamp,
                            site_id: delete_op.site_id,
                        },
                        delete_op.site_id,
                    )?;
                }
            }
            Operation::TransformOp { .. } => {
                // Transformed operations are handled separately
            }
        }
        Ok(())
    }

    /// Get the current document content from CRDT state
    pub async fn get_document_content(
        &self,
        document_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let documents = self.documents.read().await;
        let doc_state = documents
            .get(document_id)
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

    pub fn apply_operation(
        &mut self,
        operation: crate::crdt::CRDTOperation,
        site_id: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Find or create site state
        let site_idx = self
            .sites
            .iter()
            .position(|site| site.site_id == site_id)
            .unwrap_or_else(|| {
                self.sites.push(SiteState {
                    site_id,
                    operations: Vec::new(),
                });
                self.sites.len() - 1
            });

        self.sites[site_idx].operations.push(operation.clone());

        // Apply the operation to the document
        match operation {
            crate::crdt::CRDTOperation::Insert { pos, char, .. } => {
                while self.tombstone.len() <= pos {
                    self.tombstone.push(None);
                }
                self.tombstone.insert(pos, Some(char));
            }
            crate::crdt::CRDTOperation::Delete { pos, length, .. } => {
                for i in 0..length {
                    let delete_pos = pos + i;
                    if delete_pos < self.tombstone.len() {
                        self.tombstone[delete_pos] = None;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn to_string(&self) -> String {
        self.tombstone
            .iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_character_delete() {
        let mut crdt = TextCRDT::new();

        // Insert some text first
        crdt.apply_operation(crate::crdt::crate::crdt::CRDTOperation::Insert {
            pos: 0,
            char: 'H',
            lamport_clock: 1,
            site_id: 1,
        }, 1).unwrap();

        crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
            pos: 1,
            char: 'e',
            lamport_clock: 2,
            site_id: 1,
        }, 1).unwrap();

        crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
            pos: 2,
            char: 'l',
            lamport_clock: 3,
            site_id: 1,
        }, 1).unwrap();

        crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
            pos: 3,
            char: 'l',
            lamport_clock: 4,
            site_id: 1,
        }, 1).unwrap();

        crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
            pos: 4,
            char: 'o',
            lamport_clock: 5,
            site_id: 1,
        }, 1).unwrap();

        assert_eq!(crdt.to_string(), "Hello");

        // Delete single character at position 1
        crdt.apply_operation(crate::crdt::CRDTOperation::Delete {
            pos: 1,
            length: 1,
            lamport_clock: 6,
            site_id: 1,
        }, 1).unwrap();

        assert_eq!(crdt.to_string(), "Hllo");
    }

    #[test]
    fn test_multi_character_delete() {
        let mut crdt = TextCRDT::new();

        // Insert some text first
        for (i, ch) in "Hello World".chars().enumerate() {
            crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
                pos: i,
                char: ch,
                lamport_clock: i as u64 + 1,
                site_id: 1,
            }, 1).unwrap();
        }

        assert_eq!(crdt.to_string(), "Hello World");

        // Delete "lo Wo" (positions 3-7, length 5)
        crdt.apply_operation(crate::crdt::CRDTOperation::Delete {
            pos: 3,
            length: 5,
            lamport_clock: 12,
            site_id: 1,
        }, 1).unwrap();

        assert_eq!(crdt.to_string(), "Helrld");
    }

    #[test]
    fn test_delete_beyond_document_bounds() {
        let mut crdt = TextCRDT::new();

        // Insert some text first
        for (i, ch) in "Hi".chars().enumerate() {
            crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
                pos: i,
                char: ch,
                lamport_clock: i as u64 + 1,
                site_id: 1,
            }, 1).unwrap();
        }

        assert_eq!(crdt.to_string(), "Hi");

        // Try to delete more characters than exist
        crdt.apply_operation(crate::crdt::CRDTOperation::Delete {
            pos: 1,
            length: 10,
            lamport_clock: 3,
            site_id: 1,
        }, 1).unwrap();

        // Should only delete the available characters (position 1 = 'i')
        assert_eq!(crdt.to_string(), "H");
    }

    #[test]
    fn test_delete_entire_document() {
        let mut crdt = TextCRDT::new();

        // Insert some text first
        for (i, ch) in "Test".chars().enumerate() {
            crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
                pos: i,
                char: ch,
                lamport_clock: i as u64 + 1,
                site_id: 1,
            }, 1).unwrap();
        }

        assert_eq!(crdt.to_string(), "Test");

        // Delete entire document
        crdt.apply_operation(crate::crdt::CRDTOperation::Delete {
            pos: 0,
            length: 4,
            lamport_clock: 5,
            site_id: 1,
        }, 1).unwrap();

        assert_eq!(crdt.to_string(), "");
    }

    #[test]
    fn test_delete_operations_log_length() {
        let mut service = RealTimeEditingService::new();
        let document_id = "test_doc".to_string();

        // Create a document state
        {
            let mut documents = service.documents.try_write().unwrap();
            documents.insert(document_id.clone(), DocumentState {
                id: document_id.clone(),
                crdt: TextCRDT::new(),
                operations_log: Vec::new(),
                participants: std::collections::HashSet::new(),
                last_sync_timestamp: std::time::SystemTime::now(),
            });
        }

        // Apply a multi-character delete operation
        service.apply_crdt_operation(document_id.clone(), crate::crdt::CRDTOperation::Delete {
            pos: 0,
            length: 3,
            lamport_clock: 1,
            site_id: 1,
        }, 1).unwrap();

        // Check that the logged operation preserves the length
        let documents = service.documents.try_read().unwrap();
        let doc_state = documents.get(&document_id).unwrap();
        assert_eq!(doc_state.operations_log.len(), 1);

        match &doc_state.operations_log[0] {
            Operation::DeleteOp(delete_op) => {
                assert_eq!(delete_op.length, 3);
            }
            _ => panic!("Expected DeleteOp"),
        }
    }

    #[test]
    fn test_apply_operation_to_document_multi_delete() {
        let mut service = RealTimeEditingService::new();
        let document_id = "test_doc".to_string();

        // Create a document state with some content
        {
            let mut documents = service.documents.try_write().unwrap();
            let mut doc_state = DocumentState {
                id: document_id.clone(),
                crdt: TextCRDT::new(),
                operations_log: Vec::new(),
                participants: std::collections::HashSet::new(),
                last_sync_timestamp: std::time::SystemTime::now(),
            };

            // Insert "Hello World"
            for (i, ch) in "Hello World".chars().enumerate() {
                doc_state.crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
                    pos: i,
                    char: ch,
                    lamport_clock: i as u64 + 1,
                    site_id: 1,
                }, 1).unwrap();
            }

            documents.insert(document_id.clone(), doc_state);
        }

        // Apply a delete operation through operational transform
        let delete_op = Operation::DeleteOp(DeleteOperation {
            id: Uuid::new_v4(),
            position: 5,
            length: 6, // Delete " World"
            site_id: 1,
            timestamp: 1000,
        });

        {
            let mut documents = service.documents.try_write().unwrap();
            let doc_state = documents.get_mut(&document_id).unwrap();
            service.apply_operation_to_document(&mut doc_state.crdt, &delete_op).unwrap();
        }

        // Verify the content after deletion
        let documents = service.documents.try_read().unwrap();
        let doc_state = documents.get(&document_id).unwrap();
        assert_eq!(doc_state.crdt.to_string(), "Hello");
    }

    #[test]
    fn test_zero_length_delete() {
        let mut crdt = TextCRDT::new();

        // Insert some text first
        crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
            pos: 0,
            char: 'H',
            lamport_clock: 1,
            site_id: 1,
        }, 1).unwrap();

        crdt.apply_operation(crate::crdt::CRDTOperation::Insert {
            pos: 1,
            char: 'i',
            lamport_clock: 2,
            site_id: 1,
        }, 1).unwrap();

        assert_eq!(crdt.to_string(), "Hi");

        // Delete with length 0 (should not change anything)
        crdt.apply_operation(crate::crdt::CRDTOperation::Delete {
            pos: 1,
            length: 0,
            lamport_clock: 3,
            site_id: 1,
        }, 1).unwrap();

        assert_eq!(crdt.to_string(), "Hi");
    }
}

/// Conversion helpers between old and new operation types
impl RealTimeEditingService {
    /// Convert CRDTOperation to EditorOperation
    pub fn crdt_operation_to_editor_operation(&self, crdt_op: crate::crdt::CRDTOperation, site_id: u32, lamport_clock: u64) -> EditorOperation {
        match crdt_op {
            crate::crdt::CRDTOperation::Insert { pos, char, .. } => {
                EditorOperation::Insert {
                    position: pos,
                    content: char.to_string(),
                    op_id: format!("crdt_{}_{}_{}", site_id, lamport_clock, pos),
                    clock: LamportClock::new("client".to_string()),
                }
            }
            crate::crdt::CRDTOperation::Delete { pos, length, .. } => {
                EditorOperation::Delete {
                    position: pos,
                    length,
                    op_id: format!("crdt_{}_{}_{}", site_id, lamport_clock, pos),
                    clock: LamportClock::new("client".to_string()),
                }
            }
        }
    }

    /// Convert Operation to EditorOperation (legacy infallible version - deprecated)
    #[deprecated(note = "Use the fallible version instead")]
    pub fn operation_to_editor_operation_infallible(&self, operation: Operation) -> EditorOperation {
        match operation {
            Operation::InsertOp(insert_op) => {
                EditorOperation::Insert {
                    position: insert_op.position,
                    content: insert_op.content,
                    op_id: insert_op.id.to_string(),
                    clock: LamportClock::new("client".to_string()),
                }
            }
            Operation::DeleteOp(delete_op) => {
                EditorOperation::Delete {
                    position: delete_op.position,
                    length: delete_op.length,
                    op_id: delete_op.id.to_string(),
                    clock: LamportClock::new("client".to_string()),
                }
            }
            Operation::TransformOp { operation, .. } => {
                // Recursively convert the underlying operation
                self.operation_to_editor_operation_infallible(*operation)
            }
        }
    }

    /// Convert EditorOperation to CRDTOperation (for backward compatibility)
    pub fn editor_operation_to_crdt_operation(&self, editor_op: &EditorOperation) -> Option<crate::crdt::CRDTOperation> {
        match editor_op {
            EditorOperation::Insert { position, content, .. } => {
                if let Some(ch) = content.chars().next() {
                    Some(crate::crdt::CRDTOperation::Insert {
                        pos: *position,
                        char: ch,
                        lamport_clock: editor_op.clock().counter,
                        site_id: 0, // Simplified
                    })
                } else {
                    None
                }
            }
            EditorOperation::Delete { position, length, .. } => {
                Some(crate::crdt::CRDTOperation::Delete {
                    pos: *position,
                    length: *length,
                    lamport_clock: editor_op.clock().counter,
                    site_id: 0, // Simplified
                })
            }
            // Workspace operations don't convert to CRDT operations
            _ => None,
        }
    }
}
