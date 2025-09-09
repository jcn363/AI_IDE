// Operational Transformation for collaborative text editing

use super::text_crdt::EditorOperation;
use std::collections::HashSet;

// Operation types expanded for OT
#[derive(Debug, Clone, PartialEq)]
pub struct OperationTransform {
    pub operation: EditorOperation,
    pub context: OperationContext,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationContext {
    pub client_id: String,
    pub timestamp: u64,
    pub depends_on: Vec<String>, // Operation IDs this operation depends on
    pub session_id: String,
}

// Conflict resolution strategies
#[derive(Debug, Clone)]
pub enum ConflictResolutionStrategy {
    LatestWins,     // Last client to modify wins
    PositionBased,  // Use position to determine precedence
    ClientPriority, // Specific client takes precedence
}

#[derive(Debug, Clone)]
pub struct OperationalTransformer {
    client_id: String,
    session_id: String,
    applied_operations: HashSet<String>,
    conflict_strategy: ConflictResolutionStrategy,
}

impl OperationalTransformer {
    pub fn new(client_id: String, session_id: String) -> Self {
        Self {
            client_id,
            session_id,
            applied_operations: HashSet::new(),
            conflict_strategy: ConflictResolutionStrategy::PositionBased,
        }
    }

    pub fn set_conflict_strategy(&mut self, strategy: ConflictResolutionStrategy) {
        self.conflict_strategy = strategy;
    }

    pub fn transform_operation(&mut self, operation: EditorOperation) -> OperationTransform {
        let context = OperationContext {
            client_id: self.client_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            depends_on: self.get_recent_operations(),
            session_id: self.session_id.clone(),
        };

        OperationTransform { operation, context }
    }

    pub fn merge_operations(
        &mut self,
        operation1: &EditorOperation,
        operation2: &EditorOperation,
    ) -> Option<EditorOperation> {
        match (operation1, operation2) {
            // Two concurrent inserts at same position
            (
                EditorOperation::Insert {
                    position: pos1,
                    content: content1,
                    ..
                },
                EditorOperation::Insert {
                    position: pos2,
                    content: content2,
                    ..
                },
            ) if pos1 == pos2 => Some(EditorOperation::Insert {
                position: *pos1,
                content: format!("{}", content1),
                op_id: self.generate_operation_id(),
            }),

            // Insert and delete that don't conflict
            (
                EditorOperation::Insert {
                    position: insert_pos,
                    ..
                },
                EditorOperation::Delete {
                    position: delete_pos,
                    length,
                    ..
                },
            ) => {
                if insert_pos >= &(delete_pos + length) {
                    Some(operation1.clone())
                } else if insert_pos <= delete_pos {
                    Some(operation2.clone())
                } else {
                    // Need to adjust insertion position
                    let new_pos = *delete_pos;
                    match operation1 {
                        EditorOperation::Insert { content, op_id, .. } => {
                            Some(EditorOperation::Insert {
                                position: new_pos,
                                content: content.clone(),
                                op_id: op_id.clone(),
                            })
                        }
                        _ => None,
                    }
                }
            }

            // Default: return first operation
            _ => Some(operation1.clone()),
        }
    }

    pub fn should_apply_operation(&self, context: &OperationContext) -> bool {
        // Check if all dependencies are already applied
        for dep in &context.depends_on {
            if !self.applied_operations.contains(dep) {
                return false;
            }
        }
        true
    }

    pub fn record_applied_operation(&mut self, op_id: &str) {
        self.applied_operations.insert(op_id.to_string());
    }

    fn get_recent_operations(&self) -> Vec<String> {
        // Return the last few operation IDs as dependencies (simplified)
        self.applied_operations.iter().take(5).cloned().collect()
    }

    fn generate_operation_id(&self) -> String {
        format!("{}-{}", self.client_id, self.applied_operations.len())
    }
}

// Operational Transformation Server for coordinating multiple clients
#[derive(Debug, Clone)]
pub struct OTServer {
    pending_operations: Vec<OperationTransform>,
    client_states: std::collections::HashMap<String, u64>, // client_id -> last_applied_operation
    operation_history: Vec<OperationTransform>,
}

impl OTServer {
    pub fn new() -> Self {
        Self {
            pending_operations: Vec::new(),
            client_states: std::collections::HashMap::new(),
            operation_history: Vec::new(),
        }
    }

    pub fn submit_operation(&mut self, transform: OperationTransform) -> bool {
        // Check if all dependencies are satisfied
        let transformer =
            OperationalTransformer::new("server".to_string(), transform.context.session_id.clone());

        if transformer.should_apply_operation(&transform.context) {
            let client_id = transform.context.client_id.clone();
            let timestamp = transform.context.timestamp;
            let session_id = transform.context.session_id.clone();

            self.operation_history.push(transform.clone());
            self.pending_operations.push(transform);

            // Update client state
            self.client_states.insert(client_id, timestamp);
            true
        } else {
            false
        }
    }

    pub fn get_pending_operations(&self) -> &[OperationTransform] {
        &self.pending_operations
    }

    pub fn clear_pending_operations(&mut self) {
        self.pending_operations.clear();
    }

    pub fn get_client_last_update(&self, client_id: &str) -> Option<u64> {
        self.client_states.get(client_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operational_transform() {
        let mut transformer =
            OperationalTransformer::new("client1".to_string(), "session1".to_string());

        let operation = EditorOperation::Insert {
            position: 5,
            content: "test".to_string(),
            op_id: "op1".to_string(),
        };

        let transform = transformer.transform_operation(operation);
        assert_eq!(transform.context.client_id, "client1");
        assert_eq!(transform.context.session_id, "session1");
        assert!(!transform.context.depends_on.is_empty()); // Should have generated operation ID
    }

    #[test]
    fn test_merge_concurrent_inserts() {
        let mut transformer =
            OperationalTransformer::new("client1".to_string(), "session1".to_string());

        let op1 = EditorOperation::Insert {
            position: 5,
            content: "hello".to_string(),
            op_id: "op1".to_string(),
        };

        let op2 = EditorOperation::Insert {
            position: 5,
            content: "world".to_string(),
            op_id: "op2".to_string(),
        };

        let merged = transformer.merge_operations(&op1, &op2);
        assert!(merged.is_some());
    }

    #[test]
    fn test_ot_server() {
        let mut server = OTServer::new();

        let transform = OperationTransform {
            operation: EditorOperation::Insert {
                position: 5,
                content: "test".to_string(),
                op_id: "op1".to_string(),
            },
            context: OperationContext {
                client_id: "client1".to_string(),
                timestamp: 12345,
                depends_on: vec![],
                session_id: "session1".to_string(),
            },
        };

        let success = server.submit_operation(transform.clone());
        assert!(success);

        let pending = server.get_pending_operations();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].operation.clone(), transform.operation);

        assert_eq!(server.get_client_last_update("client1"), Some(12345));
        assert_eq!(server.get_client_last_update("nonexistent"), None);
    }
}
