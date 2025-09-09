// Conflict-free Replicated Data Types (CRDT) for text collaboration

use std::collections::HashMap;

// Operation types for text editing
#[derive(Debug, Clone, PartialEq)]
pub enum EditorOperation {
    Insert {
        position: usize,
        content: String,
        op_id: String,
    },
    Delete {
        position: usize,
        length: usize,
        op_id: String,
    },
    Update {
        position: usize,
        old_content: String,
        new_content: String,
        op_id: String,
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

// CRDT-based text document
#[derive(Debug, Clone)]
pub struct TextDocument {
    pub content: String,
    pub operation_count: u64,
    pub client_id: String,
    pub applied_operations: HashMap<String, EditorOperation>,
}

impl TextDocument {
    pub fn new(client_id: String) -> Self {
        Self {
            content: String::new(),
            operation_count: 0,
            client_id,
            applied_operations: HashMap::new(),
        }
    }

    pub fn with_content(client_id: String, initial_content: String) -> Self {
        Self {
            content: initial_content,
            operation_count: 0,
            client_id,
            applied_operations: HashMap::new(),
        }
    }

    pub fn apply_operation(&mut self, operation: EditorOperation) -> OperationResult {
        let result = match operation.clone() {
            EditorOperation::Insert {
                position,
                content,
                op_id,
            } => self.apply_insert(position, content, op_id),
            EditorOperation::Delete {
                position,
                length,
                op_id,
            } => self.apply_delete(position, length, op_id),
            EditorOperation::Update {
                position,
                old_content,
                new_content,
                op_id,
            } => self.apply_update(position, old_content, new_content, op_id),
        };

        self.operation_count += 1;
        result
    }

    fn apply_insert(&mut self, position: usize, content: String, op_id: String) -> OperationResult {
        // Prevent duplicate operations
        if self.applied_operations.contains_key(&op_id) {
            return OperationResult::new(false, self.content.clone());
        }

        if position <= self.content.len() {
            self.content.insert_str(position, &content);
            self.applied_operations.insert(
                op_id.clone(),
                EditorOperation::Insert {
                    position,
                    content,
                    op_id,
                },
            );

            OperationResult::new(true, self.content.clone())
        } else {
            OperationResult::new(false, self.content.clone())
        }
    }

    fn apply_delete(&mut self, position: usize, length: usize, op_id: String) -> OperationResult {
        // Prevent duplicate operations
        if self.applied_operations.contains_key(&op_id) {
            return OperationResult::new(false, self.content.clone());
        }

        if position <= self.content.len() && position + length <= self.content.len() {
            self.content.replace_range(position..position + length, "");
            self.applied_operations.insert(
                op_id.clone(),
                EditorOperation::Delete {
                    position,
                    length,
                    op_id,
                },
            );

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
                self.applied_operations.insert(
                    op_id.clone(),
                    EditorOperation::Update {
                        position,
                        old_content,
                        new_content,
                        op_id,
                    },
                );

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
