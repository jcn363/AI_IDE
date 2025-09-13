// Cursor synchronization for collaborative editing

use std::collections::HashMap;

// Cursor position in the document
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorPosition {
    pub line:   usize,
    pub column: usize,
    pub offset: usize, // Absolute character offset from document start
}

impl CursorPosition {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    pub fn from_offset(offset: usize) -> Self {
        // Simplified: assume each line has a fixed length
        let line = offset / 80; // 80 chars per line as a simple heuristic
        let column = offset % 80;
        Self::new(line, column, offset)
    }

    pub fn update_after_insert(&mut self, position: usize, length: usize) {
        if self.offset >= position {
            self.offset += length;
        }
        // Recalculate line/column based on new offset
        *self = Self::from_offset(self.offset);
    }

    pub fn update_after_delete(&mut self, position: usize, length: usize) {
        if self.offset > position + length {
            self.offset -= length;
        } else if self.offset > position {
            self.offset = position;
        }
        // Recalculate line/column based on new offset
        *self = Self::from_offset(self.offset);
    }
}

// Cursor synchronization event
#[derive(Debug, Clone)]
pub struct CursorSync {
    pub client_id:       String,
    pub position:        CursorPosition,
    pub selection_start: Option<CursorPosition>,
    pub selection_end:   Option<CursorPosition>,
    pub timestamp:       u64,
}

impl CursorSync {
    pub fn new(client_id: String, position: CursorPosition) -> Self {
        Self {
            client_id,
            position,
            selection_start: None,
            selection_end: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn with_selection(
        client_id: String,
        position: CursorPosition,
        selection_start: CursorPosition,
        selection_end: CursorPosition,
    ) -> Self {
        Self {
            client_id,
            position,
            selection_start: Some(selection_start),
            selection_end: Some(selection_end),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn is_selection(&self) -> bool {
        self.selection_start.is_some() && self.selection_end.is_some()
    }

    pub fn get_selection_length(&self) -> Option<usize> {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            Some(end.offset.saturating_sub(start.offset))
        } else {
            None
        }
    }
}

// Cursor synchronization manager
#[derive(Debug)]
pub struct CursorSyncManager {
    client_cursors:      HashMap<String, CursorSync>,
    document_operations: Vec<DocumentOperation>,
}

impl CursorSyncManager {
    pub fn new() -> Self {
        Self {
            client_cursors:      HashMap::new(),
            document_operations: Vec::new(),
        }
    }

    pub fn update_cursor(&mut self, sync: CursorSync) {
        self.client_cursors.insert(sync.client_id.clone(), sync);

        // Mark cursors that might be affected by operations
        let operations = self.document_operations.clone();
        for (_client_id, cursor_sync) in self.client_cursors.iter_mut() {
            for operation in &operations {
                match operation {
                    DocumentOperation::Insert { position, length } => {
                        cursor_sync.position.update_after_insert(*position, *length);
                    }
                    DocumentOperation::Delete { position, length } => {
                        cursor_sync.position.update_after_delete(*position, *length);
                    }
                }
            }
        }
    }

    pub fn get_cursor(&self, client_id: &str) -> Option<&CursorSync> {
        self.client_cursors.get(client_id)
    }

    pub fn get_all_cursors(&self) -> &HashMap<String, CursorSync> {
        &self.client_cursors
    }

    pub fn remove_client(&mut self, client_id: &str) -> bool {
        self.client_cursors.remove(client_id).is_some()
    }

    pub fn record_operation(&mut self, operation: DocumentOperation) {
        self.document_operations.push(operation);
    }

    pub fn clear_operations(&mut self) {
        self.document_operations.clear();
    }

    pub fn get_nearby_cursors(&self, position: CursorPosition, distance: usize) -> Vec<&CursorSync> {
        self.client_cursors
            .values()
            .filter(|cursor_sync| {
                let cursor_pos = cursor_sync.position;
                // Check if cursor is within the specified distance
                let line_diff = cursor_pos.line.abs_diff(position.line);
                let column_diff = cursor_pos.column.abs_diff(position.column);
                let offset_diff = cursor_pos.offset.abs_diff(position.offset);

                line_diff <= distance && column_diff <= distance && offset_diff <= distance
            })
            .collect()
    }

    pub fn find_cursors_in_selection(&self, start: CursorPosition, end: CursorPosition) -> Vec<&CursorSync> {
        self.client_cursors
            .values()
            .filter(|cursor_sync| {
                cursor_sync.position.offset >= start.offset && cursor_sync.position.offset <= end.offset
            })
            .collect()
    }

    pub fn clear_all(&mut self) {
        self.client_cursors.clear();
        self.document_operations.clear();
    }
}

// Document operations that affect cursor positions
#[derive(Debug, Clone)]
pub enum DocumentOperation {
    Insert { position: usize, length: usize },
    Delete { position: usize, length: usize },
}

impl DocumentOperation {
    pub fn insert(position: usize, length: usize) -> Self {
        Self::Insert { position, length }
    }

    pub fn delete(position: usize, length: usize) -> Self {
        Self::Delete { position, length }
    }

    pub fn affects_position(&self, cursor_position: usize) -> bool {
        match self {
            Self::Insert { position, length } => cursor_position >= *position,
            Self::Delete { position, length } => cursor_position >= *position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position() {
        let mut pos = CursorPosition::new(1, 5, 85);

        // Test insert update
        pos.update_after_insert(80, 5);
        assert_eq!(pos.offset, 90);
        assert_eq!(pos.line, 1); // Offset 90 / 80 chars per line = 1
        assert_eq!(pos.column, 10); // Offset 90 % 80 chars per line = 10

        // Test delete update
        pos.update_after_delete(85, 10);
        assert_eq!(pos.offset, 80);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_cursor_sync() {
        let position = CursorPosition::new(5, 10, 400);
        let sync = CursorSync::new("client1".to_string(), position);

        assert_eq!(sync.client_id, "client1");
        assert_eq!(sync.position, position);
        assert!(!sync.is_selection());
        assert!(sync.get_selection_length().is_none());

        let selection_sync = CursorSync::with_selection(
            "client1".to_string(),
            position,
            CursorPosition::new(5, 10, 400),
            CursorPosition::new(5, 15, 405),
        );

        assert!(selection_sync.is_selection());
        assert_eq!(selection_sync.get_selection_length(), Some(5));
    }

    #[test]
    fn test_cursor_sync_manager() {
        let mut manager = CursorSyncManager::new();

        let sync = CursorSync::new("client1".to_string(), CursorPosition::new(5, 10, 400));
        manager.update_cursor(sync);

        let retrieved = manager.get_cursor("client1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().client_id, "client1");

        // Add an insert operation and update cursor
        manager.record_operation(DocumentOperation::insert(390, 20));
        let sync2 = CursorSync::new("client1".to_string(), CursorPosition::new(5, 10, 400));
        manager.update_cursor(sync2);

        // Cursor should have shifted due to the insert operation
        let updated_cursor = manager.get_cursor("client1");
        assert!(updated_cursor.is_some());
        // The cursor should have moved to accommodate the insert

        assert!(manager.remove_client("client1"));
        assert!(manager.get_cursor("client1").is_none());
    }

    #[test]
    fn test_neaby_cursor_search() {
        let mut manager = CursorSyncManager::new();

        // Add multiple cursors
        let cursors = vec![
            ("client1", CursorPosition::new(5, 10, 400)),
            ("client2", CursorPosition::new(5, 15, 405)),
            ("client3", CursorPosition::new(10, 0, 800)),
        ];

        for (client_id, position) in cursors {
            let sync = CursorSync::new(client_id.to_string(), position);
            manager.update_cursor(sync);
        }

        let search_pos = CursorPosition::new(5, 12, 402);
        let nearby = manager.get_nearby_cursors(search_pos, 10);

        assert_eq!(nearby.len(), 2); // client1 and client2 should be nearby
        let client_ids: Vec<&str> = nearby.iter().map(|sync| sync.client_id.as_str()).collect();
        assert!(client_ids.contains(&"client1"));
        assert!(client_ids.contains(&"client2"));
        assert!(!client_ids.contains(&"client3"));
    }
}
