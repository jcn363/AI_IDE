//! CRDT (Conflict-Free Replicated Data Types) implementation for collaborative editing.
//!
//! This module provides the core CRDT operations and data structures used
//! throughout the collaboration system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CRDT operation types for collaborative text editing
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
        length: usize,
        lamport_clock: u64,
        site_id: u32,
    },
}

/// Site state containing operations for a specific collaborator
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SiteState {
    pub site_id: u32,
    pub operations: Vec<CRDTOperation>,
}

/// CRDT implementation for collaborative text editing
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TextCRDT {
    pub sites: Vec<SiteState>,
    pub tombstone: Vec<Option<char>>, // Deletion tracking
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
        operation: CRDTOperation,
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
            CRDTOperation::Insert { pos, char, .. } => {
                while self.tombstone.len() <= pos {
                    self.tombstone.push(None);
                }
                self.tombstone.insert(pos, Some(char));
            }
            CRDTOperation::Delete { pos, length, .. } => {
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

/// Lamport clock for operation ordering
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LamportClock {
    pub node_id: String,
    pub counter: u64,
}

impl LamportClock {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            counter: 0,
        }
    }

    pub fn increment(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    pub fn update(&mut self, other: u64) {
        self.counter = std::cmp::max(self.counter, other) + 1;
    }
}

/// Editor operation types (used by AI conflict resolution)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EditorOperation {
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
    Move {
        from_position: usize,
        to_position: usize,
        length: usize,
        op_id: String,
        clock: LamportClock,
    },
}

impl EditorOperation {
    pub fn clock(&self) -> &LamportClock {
        match self {
            EditorOperation::Insert { clock, .. } => clock,
            EditorOperation::Delete { clock, .. } => clock,
            EditorOperation::Update { clock, .. } => clock,
            EditorOperation::Move { clock, .. } => clock,
        }
    }
}

/// CRDT state representation for documents
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CRDTState {
    pub operations: Vec<CRDTOperation>,
}

/// Conversion helpers between CRDT and Editor operations
pub mod conversions {
    use super::*;

    pub fn crdt_to_editor_operation(
        crdt_op: CRDTOperation,
        site_id: u32,
        lamport_clock: u64,
    ) -> EditorOperation {
        match crdt_op {
            CRDTOperation::Insert { pos, char, .. } => {
                EditorOperation::Insert {
                    position: pos,
                    content: char.to_string(),
                    op_id: format!("crdt_{}_{}_{}", site_id, lamport_clock, pos),
                    clock: LamportClock::new("client".to_string()),
                }
            }
            CRDTOperation::Delete { pos, length, .. } => {
                EditorOperation::Delete {
                    position: pos,
                    length,
                    op_id: format!("crdt_{}_{}_{}", site_id, lamport_clock, pos),
                    clock: LamportClock::new("client".to_string()),
                }
            }
        }
    }

    pub fn editor_to_crdt_operation(editor_op: &EditorOperation) -> Option<CRDTOperation> {
        match editor_op {
            EditorOperation::Insert { position, content, .. } => {
                if let Some(ch) = content.chars().next() {
                    Some(CRDTOperation::Insert {
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
                Some(CRDTOperation::Delete {
                    pos: *position,
                    length: *length,
                    lamport_clock: editor_op.clock().counter,
                    site_id: 0, // Simplified
                })
            }
            EditorOperation::Update { position, new_content, .. } => {
                // Convert update to insert operation with new content
                if let Some(ch) = new_content.chars().next() {
                    Some(CRDTOperation::Insert {
                        pos: *position,
                        char: ch,
                        lamport_clock: editor_op.clock().counter,
                        site_id: 0,
                    })
                } else {
                    None
                }
            }
            EditorOperation::Move { .. } => {
                // Move operations are not directly convertible to CRDT operations
                None
            }
        }
    }
}