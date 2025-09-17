//! Common types for the integration bridge

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use lsp_types::{Diagnostic, CodeAction, CompletionItem, Hover, Position, Range, TextDocumentContentChangeEvent, Url};
use ropey::Rope;
use rust_ai_ide_collaboration::real_time_editing::{CRDTOperation, Operation, TextCRDT};
use rust_ai_ide_collaboration::distributed_workspace::WorkspaceState;

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub enable_ai_conflict_resolution: bool,
    pub max_sync_delay_ms: u64,
    pub conflict_resolution_timeout_ms: u64,
    pub diagnostics_sync_enabled: bool,
    pub code_actions_sync_enabled: bool,
    pub completion_sync_enabled: bool,
    pub hover_sync_enabled: bool,
    pub workspace_sync_enabled: bool,
}

/// Document synchronization state
#[derive(Debug, Clone)]
pub struct DocumentSyncState {
    pub document_uri: Url,
    pub lsp_version: Option<i32>,
    pub crdt_state: TextCRDT,
    pub last_sync_timestamp: std::time::SystemTime,
    pub pending_changes: Vec<TextDocumentContentChangeEvent>,
    pub is_in_conflict: bool,
    pub conflict_resolution_attempts: u32,
}

/// Collaborative diagnostic with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeDiagnostic {
    pub diagnostic: Diagnostic,
    pub author: String,
    pub timestamp: u64,
    pub session_id: String,
    pub votes: HashMap<String, DiagnosticVote>,
}

/// Vote on a diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticVote {
    Agree,
    Disagree,
    Unsure,
}

/// Collaborative code action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeCodeAction {
    pub action: CodeAction,
    pub author: String,
    pub timestamp: u64,
    pub applied_count: u32,
    pub rejected_count: u32,
}

/// Completion context for collaborative sharing
#[derive(Debug, Clone)]
pub struct CollaborativeCompletionContext {
    pub position: Position,
    pub trigger_character: Option<String>,
    pub items: Vec<CompletionItem>,
    pub author: String,
    pub session_id: String,
}

/// Hover information sharing
#[derive(Debug, Clone)]
pub struct CollaborativeHoverContext {
    pub position: Position,
    pub hover: Hover,
    pub author: String,
    pub session_id: String,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    /// LSP changes take precedence
    LSPWins,
    /// Collaborative changes take precedence
    CollaborationWins,
    /// Use AI to resolve conflicts
    AIResolution,
    /// Manual resolution required
    Manual,
    /// Merge changes if possible
    Merge,
}

/// Conflict detection result
#[derive(Debug, Clone)]
pub struct ConflictDetection {
    pub has_conflict: bool,
    pub conflict_ranges: Vec<Range>,
    pub lsp_changes: Vec<TextDocumentContentChangeEvent>,
    pub collaborative_operations: Vec<Operation>,
    pub severity: ConflictSeverity,
}

/// Conflict severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Synchronized,
    Syncing,
    OutOfSync,
    InConflict,
    Failed,
}

/// Bridge health status
#[derive(Debug, Clone)]
pub struct BridgeHealthStatus {
    pub overall_status: SyncStatus,
    pub documents_synced: usize,
    pub conflicts_resolved: usize,
    pub sync_failures: usize,
    pub average_sync_time_ms: f64,
    pub last_health_check: std::time::SystemTime,
}

/// Event types for bridge communication
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    DocumentChanged {
        uri: Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        author: String,
    },
    ConflictDetected {
        uri: Url,
        conflict: ConflictDetection,
    },
    ConflictResolved {
        uri: Url,
        strategy: ConflictResolutionStrategy,
        success: bool,
    },
    DiagnosticsUpdated {
        uri: Url,
        diagnostics: Vec<CollaborativeDiagnostic>,
    },
    CodeActionApplied {
        uri: Url,
        action: CollaborativeCodeAction,
    },
    SyncCompleted {
        uri: Url,
        status: SyncStatus,
        duration_ms: u64,
    },
}

/// Shared state between collaboration and LSP systems
#[derive(Debug)]
pub struct SharedBridgeState {
    pub document_states: Arc<RwLock<HashMap<Url, DocumentSyncState>>>,
    pub document_ropes: Arc<RwLock<HashMap<Url, Rope>>>,
    pub workspace_state: Arc<RwLock<Option<WorkspaceState>>>,
    pub diagnostics: Arc<RwLock<HashMap<Url, Vec<CollaborativeDiagnostic>>>>,
    pub code_actions: Arc<RwLock<HashMap<Url, Vec<CollaborativeCodeAction>>>>,
    pub completion_cache: Arc<RwLock<HashMap<String, Vec<CollaborativeCompletionContext>>>>,
    pub hover_cache: Arc<RwLock<HashMap<String, Vec<CollaborativeHoverContext>>>>,
    pub health_status: Arc<RwLock<BridgeHealthStatus>>,
}

/// CRDT operation translation result
#[derive(Debug, Clone)]
pub struct CRDTTranslationResult {
    pub lsp_changes: Vec<TextDocumentContentChangeEvent>,
    pub translation_confidence: f64,
    pub warnings: Vec<String>,
}

/// Workspace synchronization context
#[derive(Debug, Clone)]
pub struct WorkspaceSyncContext {
    pub workspace_id: String,
    pub last_sync_version: u64,
    pub pending_operations: Vec<rust_ai_ide_collaboration::distributed_workspace::WorkspaceOperation>,
    pub participants: Vec<String>,
}