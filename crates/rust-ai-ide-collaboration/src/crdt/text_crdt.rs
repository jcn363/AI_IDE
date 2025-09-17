// Conflict-free Replicated Data Types (CRDT) for text collaboration

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// CRDT trait for conflict-free replicated data types
pub trait CRDT {
    type Operation;
    type State;

    /// Apply an operation to the CRDT state
    fn apply_operation(&mut self, operation: Self::Operation) -> Result<(), Box<dyn std::error::Error>>;

    /// Merge state from another replica
    fn merge(&mut self, other: &Self::State) -> Result<(), Box<dyn std::error::Error>>;

    /// Get current state for synchronization
    fn get_state(&self) -> &Self::State;

    /// Check if operation can be applied (for validation)
    fn can_apply(&self, operation: &Self::Operation) -> bool;
}

// Operation type for creating new operations
#[derive(Debug, Clone)]
pub enum OperationType {
    Insert {
        position: usize,
        content: String,
    },
    Delete {
        position: usize,
        length: usize,
    },
    Update {
        position: usize,
        old_content: String,
        new_content: String,
    },
}

// Lamport Clock for causal ordering
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LamportClock {
    pub counter: u64,
    pub node_id: String,
}

impl LamportClock {
    pub fn new(node_id: String) -> Self {
        Self {
            counter: 0,
            node_id,
        }
    }

    pub fn increment(&mut self) -> Self {
        self.counter += 1;
        self.clone()
    }

    pub fn merge(&mut self, other: &LamportClock) {
        self.counter = self.counter.max(other.counter) + 1;
    }

    pub fn tick(&mut self) -> LamportClock {
        self.counter += 1;
        self.clone()
    }
}

impl PartialOrd for LamportClock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LamportClock {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.counter
            .cmp(&other.counter)
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

// Unified EditorOperation for all editing contexts (text and workspace)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EditorOperation {
    // Text editing operations
    Insert {
        position: usize,
        content: String,
        op_id: String,
        clock: LamportClock,
    },
    Delete {
        position: usize,
        length: usize,
        op_id: String,
        clock: LamportClock,
    },
    Update {
        position: usize,
        old_content: String,
        new_content: String,
        op_id: String,
        clock: LamportClock,
    },
    // Workspace operations
    AddEntry {
        path: String,
        entry_type: String, // "file" or "directory"
        lamport_clock: u64,
        site_id: u32,
        op_id: String,
        clock: LamportClock,
    },
    RemoveEntry {
        path: String,
        lamport_clock: u64,
        site_id: u32,
        op_id: String,
        clock: LamportClock,
    },
    RenameEntry {
        old_path: String,
        new_path: String,
        lamport_clock: u64,
        site_id: u32,
        op_id: String,
        clock: LamportClock,
    },
    MoveEntry {
        from_path: String,
        to_path: String,
        lamport_clock: u64,
        site_id: u32,
        op_id: String,
        clock: LamportClock,
    },
}

#[derive(Debug, Clone)]
pub struct OperationResult {
    pub success: bool,
    pub new_content: String,
    pub applied_operations: Vec<String>, // Operation IDs that were applied
}

impl OperationResult {
    pub fn new(success: bool, new_content: String) -> Self {
        Self {
            success,
            new_content,
            applied_operations: Vec::new(),
        }
    }

    pub fn add_applied_operation(&mut self, op_id: String) {
        self.applied_operations.push(op_id);
    }
}

// CRDT-based text document with Lamport clock synchronization
#[derive(Debug, Clone)]
pub struct TextDocument {
    pub content: String,
    pub operation_count: u64,
    pub client_id: String,
    pub clock: LamportClock,
    pub applied_operations: HashMap<String, EditorOperation>,
    pub operation_log: Vec<EditorOperation>, // Ordered log for causality
}

impl TextDocument {
    pub fn new(client_id: String) -> Self {
        Self {
            content: String::new(),
            operation_count: 0,
            client_id: client_id.clone(),
            clock: LamportClock::new(client_id),
            applied_operations: HashMap::new(),
            operation_log: Vec::new(),
        }
    }

    pub fn with_content(client_id: String, initial_content: String) -> Self {
        Self {
            content: initial_content,
            operation_count: 0,
            client_id: client_id.clone(),
            clock: LamportClock::new(client_id),
            applied_operations: HashMap::new(),
            operation_log: Vec::new(),
        }
    }

    pub fn apply_operation(&mut self, operation: EditorOperation) -> OperationResult {
        // Update our Lamport clock based on the incoming operation's clock
        self.clock.merge(&operation.clock());

        let result = match operation.clone() {
            EditorOperation::Insert {
                position,
                content,
                op_id,
                clock,
            } => self.apply_insert(position, content, op_id, clock),
            EditorOperation::Delete {
                position,
                length,
                op_id,
                clock,
            } => self.apply_delete(position, length, op_id, clock),
            EditorOperation::Update {
                position,
                old_content,
                new_content,
                op_id,
                clock,
            } => self.apply_update(position, old_content, new_content, op_id, clock),
        };

        if result.success {
            self.operation_count += 1;
            self.operation_log.push(operation.clone());
        }
        result
    }

    /// Generate a new operation with updated Lamport clock
    pub fn create_operation(
        &mut self,
        operation_type: OperationType,
        op_id: String,
    ) -> EditorOperation {
        let clock = self.clock.tick();

        match operation_type {
            OperationType::Insert { position, content } => EditorOperation::Insert {
                position,
                content,
                op_id,
                clock,
            },
            OperationType::Delete { position, length } => EditorOperation::Delete {
                position,
                length,
                op_id,
                clock,
            },
            OperationType::Update {
                position,
                old_content,
                new_content,
            } => EditorOperation::Update {
                position,
                old_content,
                new_content,
                op_id,
                clock,
            },
        }

        impl EditorOperation {
            /// Get the Lamport clock for this operation
            pub fn clock(&self) -> &LamportClock {
                match self {
                    EditorOperation::Insert { clock, .. } => clock,
                    EditorOperation::Delete { clock, .. } => clock,
                    EditorOperation::Update { clock, .. } => clock,
                    EditorOperation::AddEntry { clock, .. } => clock,
                    EditorOperation::RemoveEntry { clock, .. } => clock,
                    EditorOperation::RenameEntry { clock, .. } => clock,
                    EditorOperation::MoveEntry { clock, .. } => clock,
                }
            }

            /// Get the operation ID
            pub fn op_id(&self) -> &str {
                match self {
                    EditorOperation::Insert { op_id, .. } => op_id,
                    EditorOperation::Delete { op_id, .. } => op_id,
                    EditorOperation::Update { op_id, .. } => op_id,
                    EditorOperation::AddEntry { op_id, .. } => op_id,
                    EditorOperation::RemoveEntry { op_id, .. } => op_id,
                    EditorOperation::RenameEntry { op_id, .. } => op_id,
                    EditorOperation::MoveEntry { op_id, .. } => op_id,
                }
            }
        }
    }

    /// Get current Lamport clock
    pub fn get_clock(&self) -> &LamportClock {
        &self.clock
    }

    /// Get operation log for synchronization
    pub fn get_operation_log(&self) -> &[EditorOperation] {
        &self.operation_log
    }

    fn apply_insert(
        &mut self,
        position: usize,
        content: String,
        op_id: String,
        clock: LamportClock,
    ) -> OperationResult {
        // Prevent duplicate operations
        if self.applied_operations.contains_key(&op_id) {
            return OperationResult::new(false, self.content.clone());
        }

        if position <= self.content.len() {
            self.content.insert_str(position, &content);
            let operation = EditorOperation::Insert {
                position,
                content,
                op_id: op_id.clone(),
                clock,
            };
            self.applied_operations.insert(op_id, operation);

            OperationResult::new(true, self.content.clone())
        } else {
            OperationResult::new(false, self.content.clone())
        }
    }

    fn apply_delete(
        &mut self,
        position: usize,
        length: usize,
        op_id: String,
        clock: LamportClock,
    ) -> OperationResult {
        // Prevent duplicate operations
        if self.applied_operations.contains_key(&op_id) {
            return OperationResult::new(false, self.content.clone());
        }

        if position <= self.content.len() && position + length <= self.content.len() {
            self.content.replace_range(position..position + length, "");
            let operation = EditorOperation::Delete {
                position,
                length,
                op_id: op_id.clone(),
                clock,
            };
            self.applied_operations.insert(op_id, operation);

            OperationResult::new(true, self.content.clone())
        } else {
            OperationResult::new(false, self.content.clone())
        }
    }

    fn apply_update(
        &mut self,
        position: usize,
        old_content: String,
        new_content: String,
        op_id: String,
        clock: LamportClock,
    ) -> OperationResult {
        // Prevent duplicate operations
        if self.applied_operations.contains_key(&op_id) {
            return OperationResult::new(false, self.content.clone());
        }

        // Verify the content at position matches expected old content
        if position <= self.content.len() && position + old_content.len() <= self.content.len() {
            if self.content[position..position + old_content.len()] == old_content {
                self.content
                    .replace_range(position..position + old_content.len(), &new_content);
                let operation = EditorOperation::Update {
                    position,
                    old_content,
                    new_content,
                    op_id: op_id.clone(),
                    clock,
                };
                self.applied_operations.insert(op_id, operation);

                OperationResult::new(true, self.content.clone())
            } else {
                OperationResult::new(false, self.content.clone())
            }
        } else {
            OperationResult::new(false, self.content.clone())
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn get_operation_history(&self) -> &HashMap<String, EditorOperation> {
        &self.applied_operations
    }

    pub fn has_operation(&self, op_id: &str) -> bool {
        self.applied_operations.contains_key(op_id)
    }
}

impl CRDT for TextDocument {
    type Operation = EditorOperation;
    type State = String;

    fn apply_operation(&mut self, operation: Self::Operation) -> Result<(), Box<dyn std::error::Error>> {
        let result = TextDocument::apply_operation(self, operation);
        if result.success {
            Ok(())
        } else {
            Err("Failed to apply operation".into())
        }
    }

    fn merge(&mut self, other: &Self::State) -> Result<(), Box<dyn std::error::Error>> {
        // Simple merge strategy - keep the longer content (can be enhanced)
        if other.len() > self.content.len() {
            self.content = other.clone();
        }
        Ok(())
    }

    fn get_state(&self) -> &Self::State {
        &self.content
    }

    fn can_apply(&self, operation: &Self::Operation) -> bool {
        match operation {
            EditorOperation::Insert { position, .. } => *position <= self.content.len(),
            EditorOperation::Delete { position, length, .. } => *position <= self.content.len() && *position + *length <= self.content.len(),
            EditorOperation::Update { position, old_content, .. } => {
                *position <= self.content.len() && *position + old_content.len() <= self.content.len() &&
                &self.content[*position..*position + old_content.len()] == old_content
            },
            // Workspace operations always valid for now
            EditorOperation::AddEntry { .. } |
            EditorOperation::RemoveEntry { .. } |
            EditorOperation::RenameEntry { .. } |
            EditorOperation::MoveEntry { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_document_operations() {
        let mut doc = TextDocument::with_content("client1".to_string(), "Hello World".to_string());

        // Test insert operation
        let insert_op = EditorOperation::Insert {
            position: 5,
            content: " Beautiful".to_string(),
            op_id: "op1".to_string(),
        };

        let result = doc.apply_operation(insert_op);
        assert!(result.success);
        assert_eq!(result.new_content, "Hello Beautiful World");
        assert!(result.applied_operations.contains(&"op1".to_string()));
        assert!(doc.has_operation("op1"));
    }

    #[test]
    fn test_duplicate_operation_rejected() {
        let mut doc = TextDocument::new("client1".to_string());

        let op = EditorOperation::Insert {
            position: 0,
            content: "test".to_string(),
            op_id: "dup".to_string(),
        };

        // First application should succeed
        let result1 = doc.apply_operation(op.clone());
        assert!(result1.success);

        // Second application should fail
        let result2 = doc.apply_operation(op);
        assert!(!result2.success);
    }

    #[test]
    fn test_invalid_operation_positions() {
        let mut doc = TextDocument::with_content("client1".to_string(), "test".to_string());

        // Try to insert beyond document length
        let invalid_insert = EditorOperation::Insert {
            position: 10,
            content: "invalid".to_string(),
            op_id: "invalid1".to_string(),
        };

        let result = doc.apply_operation(invalid_insert);
        assert!(!result.success);
        assert!(!doc.has_operation("invalid1"));
    }
}
