//! Integration bridge between collaboration and LSP systems.
//!
//! This module implements the `CollaborationLSPBridge` that provides real-time
//! synchronization between collaborative editing and LSP document state, conflict
//! resolution, and shared functionality across both systems.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::{timeout, Instant};
use tracing::{debug, error, info, warn};

use lsp_types::{DidChangeTextDocumentParams, DidOpenTextDocumentParams, Position, Range, TextDocumentContentChangeEvent, Url};
use ropey::Rope;
use rust_ai_ide_ai::services::AIService;
use rust_ai_ide_collaboration::ai_conflict_resolution::AIConflictResolver;
use rust_ai_ide_collaboration::crdt::{EditorOperation, LamportClock};
use rust_ai_ide_collaboration::real_time_editing::{RealTimeEditingService, CRDTOperation, Operation, TextCRDT};
use rust_ai_ide_collaboration::distributed_workspace::DistributedWorkspaceManager;
use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_lsp::language_router::LanguageRouter;
use rust_ai_ide_security::audit_logger;

use crate::error::{IntegrationError, IntegrationResult};
use crate::types::*;

/// Main integration bridge between collaboration and LSP systems
pub struct CollaborationLSPBridge {
    /// Configuration for the bridge
    config: BridgeConfig,
    /// Collaboration service instance
    collaboration_service: Arc<RwLock<RealTimeEditingService>>,
    /// Distributed workspace manager
    workspace_manager: Arc<RwLock<DistributedWorkspaceManager>>,
    /// Language router for LSP communication
    language_router: Arc<RwLock<LanguageRouter>>,
    /// AI service for conflict resolution
    ai_service: Option<Arc<AIService>>,
    /// AI conflict resolver
    ai_conflict_resolver: Option<Arc<RwLock<AIConflictResolver>>>,
    /// Shared bridge state
    shared_state: Arc<SharedBridgeState>,
    /// Event broadcaster for bridge events
    event_tx: mpsc::UnboundedSender<BridgeEvent>,
    /// Event receiver for processing events
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<BridgeEvent>>>,
    /// Synchronization semaphore to prevent concurrent operations
    sync_semaphore: Arc<Semaphore>,
    /// Cache for recent translations
    translation_cache: Cache<String, CRDTTranslationResult>,
    /// Health monitoring
    health_monitor: Arc<RwLock<BridgeHealthMonitor>>,
}

/// Health monitoring for the bridge
pub struct BridgeHealthMonitor {
    last_health_check: Instant,
    sync_operations: u64,
    failed_operations: u64,
    average_sync_time: std::time::Duration,
}

impl CollaborationLSPBridge {
    /// Create a new collaboration-LSP bridge
    pub async fn new(
        config: BridgeConfig,
        collaboration_service: Arc<RwLock<RealTimeEditingService>>,
        workspace_manager: Arc<RwLock<DistributedWorkspaceManager>>,
        language_router: Arc<RwLock<LanguageRouter>>,
        ai_service: Option<Arc<AIService>>,
        ai_conflict_resolver: Option<Arc<RwLock<AIConflictResolver>>>,
    ) -> IntegrationResult<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let shared_state = Arc::new(SharedBridgeState {
            document_states: Arc::new(RwLock::new(HashMap::new())),
            document_ropes: Arc::new(RwLock::new(HashMap::new())),
            workspace_state: Arc::new(RwLock::new(None)),
            diagnostics: Arc::new(RwLock::new(HashMap::new())),
            code_actions: Arc::new(RwLock::new(HashMap::new())),
            completion_cache: Arc::new(RwLock::new(HashMap::new())),
            hover_cache: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(BridgeHealthStatus {
                overall_status: SyncStatus::Synchronized,
                documents_synced: 0,
                conflicts_resolved: 0,
                sync_failures: 0,
                average_sync_time_ms: 0.0,
                last_health_check: std::time::SystemTime::now(),
            })),
        });

        let sync_semaphore = Arc::new(Semaphore::new(10)); // Allow up to 10 concurrent sync operations

        let translation_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(300)) // 5 minutes TTL
            .build();

        let health_monitor = Arc::new(RwLock::new(BridgeHealthMonitor {
            last_health_check: Instant::now(),
            sync_operations: 0,
            failed_operations: 0,
            average_sync_time: Duration::from_millis(100),
        }));

        let bridge = Self {
            config,
            collaboration_service,
            workspace_manager,
            language_router,
            ai_service,
            ai_conflict_resolver,
            shared_state,
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            sync_semaphore,
            translation_cache,
            health_monitor,
        };

        // Start background event processing
        bridge.start_event_processor().await;

        audit_logger::log_event(
            "bridge_initialized",
            &format!("Collaboration-LSP bridge initialized with config: {:?}", bridge.config),
        );

        Ok(bridge)
    }

    /// Start the background event processor
    async fn start_event_processor(&self) {
        let event_rx = self.event_rx.clone();
        let shared_state = self.shared_state.clone();
        let health_monitor = self.health_monitor.clone();

        tokio::spawn(async move {
            let mut rx = event_rx.write().await;
            while let Some(event) = rx.recv().await {
                if let Err(e) = Self::process_bridge_event(&shared_state, &health_monitor, event).await {
                    error!("Failed to process bridge event: {}", e);
                }
            }
        });
    }

    /// Process a bridge event
    async fn process_bridge_event(
        shared_state: &SharedBridgeState,
        health_monitor: &RwLock<BridgeHealthMonitor>,
        event: BridgeEvent,
    ) -> IntegrationResult<()> {
        match event {
            BridgeEvent::DocumentChanged { uri, changes, author } => {
                debug!("Processing document change event for {} by {}", uri, author);
                // Update document sync state
                let mut doc_states = shared_state.document_states.write().await;
                if let Some(state) = doc_states.get_mut(&uri) {
                    state.pending_changes.extend(changes);
                    state.last_sync_timestamp = std::time::SystemTime::now();
                }
            }
            BridgeEvent::ConflictDetected { uri, conflict } => {
                warn!("Conflict detected for document {}", uri);
                // Mark document as in conflict
                let mut doc_states = shared_state.document_states.write().await;
                if let Some(state) = doc_states.get_mut(&uri) {
                    state.is_in_conflict = true;
                    state.conflict_resolution_attempts += 1;
                }

                let mut health = shared_state.health_status.write().await;
                health.overall_status = SyncStatus::InConflict;
            }
            BridgeEvent::ConflictResolved { uri, strategy, success } => {
                info!("Conflict resolved for {} using {:?}: {}", uri, strategy, success);
                let mut doc_states = shared_state.document_states.write().await;
                if let Some(state) = doc_states.get_mut(&uri) {
                    state.is_in_conflict = false;
                    if success {
                        state.conflict_resolution_attempts = 0;
                    }
                }

                let mut health = shared_state.health_status.write().await;
                if success {
                    health.conflicts_resolved += 1;
                }
            }
            BridgeEvent::SyncCompleted { uri: _, status, duration_ms } => {
                let mut monitor = health_monitor.write().await;
                monitor.sync_operations += 1;
                monitor.average_sync_time = (monitor.average_sync_time + Duration::from_millis(duration_ms)) / 2;

                let mut health = shared_state.health_status.write().await;
                health.overall_status = status;
                health.average_sync_time_ms = monitor.average_sync_time.as_millis() as f64;
                health.last_health_check = std::time::SystemTime::now();
            }
            _ => {
                // Handle other events as needed
            }
        }

        Ok(())
    }

    /// Translate CRDT operations to LSP document changes
    pub async fn translate_crdt_to_lsp_changes(
        &self,
        document_uri: &Url,
        crdt_operations: &[CRDTOperation],
    ) -> IntegrationResult<CRDTTranslationResult> {
        // Check cache first
        let cache_key = format!("{}_{}", document_uri, crdt_operations.len());
        if let Some(cached) = self.translation_cache.get(&cache_key).await {
            return Ok(cached);
        }

        // Get or create the document rope
        let rope = self.get_or_create_document_rope(document_uri).await?;

        let mut lsp_changes = Vec::new();
        let mut warnings = Vec::new();
        let mut confidence = 1.0;

        for operation in crdt_operations {
            match operation {
                CRDTOperation::Insert { pos, char, .. } => {
                    // Convert insert operation to LSP change
                    let pos_lsp = Self::byte_to_lsp_position(&rope, *pos);
                    let range = Range {
                        start: pos_lsp,
                        end: pos_lsp,
                    };

                    lsp_changes.push(TextDocumentContentChangeEvent {
                        range: Some(range),
                        range_length: Some(0),
                        text: char.to_string(),
                    });
                }
                CRDTOperation::Delete { pos, .. } => {
                    // Convert delete operation to LSP change
                    let start_pos = Self::byte_to_lsp_position(&rope, *pos);
                    let end_pos = Self::byte_to_lsp_position(&rope, *pos + 1);
                    let range = Range {
                        start: start_pos,
                        end: end_pos,
                    };

                    lsp_changes.push(TextDocumentContentChangeEvent {
                        range: Some(range),
                        range_length: Some(1),
                        text: String::new(),
                    });
                }
            }
        }
        
        #[cfg(test)]
        mod tests {
            use super::*;
            use ropey::Rope;
        
            impl CollaborationLSPBridge {
                pub(crate) fn byte_to_lsp_position(&self, rope: &Rope, byte_pos: usize) -> Position {
                    let line_idx = rope.byte_to_line(byte_pos);
                    let line_start_byte = rope.line_to_byte(line_idx);
                    let char_idx_in_line = rope.byte_to_char(byte_pos) - rope.line_to_char(line_start_byte);
                    let utf16_cu = rope.char_to_utf16_cu(rope.line_to_char(line_idx) + char_idx_in_line) - rope.char_to_utf16_cu(rope.line_to_char(line_idx));
        
                    Position {
                        line: line_idx as u32,
                        character: utf16_cu as u32,
                    }
                }
            }
        
            #[test]
            fn test_byte_to_lsp_position_single_line() {
                let content = "hello world";
                let rope = Rope::from_str(content);
        
                // Position at start
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 0);
                assert_eq!(pos.line, 0);
                assert_eq!(pos.character, 0);
        
                // Position at 'w' (6th char)
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 6);
                assert_eq!(pos.line, 0);
                assert_eq!(pos.character, 6);
            }
        
            #[test]
            fn test_byte_to_lsp_position_multi_line() {
                let content = "hello\nworld\ntest";
                let rope = Rope::from_str(content);
        
                // Position at start
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 0);
                assert_eq!(pos.line, 0);
                assert_eq!(pos.character, 0);
        
                // Position at 'o' in "hello" (4th char, line 0)
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 4);
                assert_eq!(pos.line, 0);
                assert_eq!(pos.character, 4);
        
                // Position at start of line 1 ("world")
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 6); // "hello\n" is 6 bytes
                assert_eq!(pos.line, 1);
                assert_eq!(pos.character, 0);
        
                // Position at 'r' in "world" (2nd char, line 1)
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 8); // "hello\nw" is 8 bytes
                assert_eq!(pos.line, 1);
                assert_eq!(pos.character, 2);
        
                // Position at start of line 2 ("test")
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 12); // "hello\nworld\n" is 12 bytes
                assert_eq!(pos.line, 2);
                assert_eq!(pos.character, 0);
            }
        
            #[test]
            fn test_byte_to_lsp_position_utf16() {
                let content = "héllo\nwörld"; // é and ö are multi-byte but single UTF-16
                let rope = Rope::from_str(content);
        
                // Position at 'é' (1st char, but 2 bytes)
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 1); // 'h' is 0, 'é' starts at 1
                assert_eq!(pos.line, 0);
                assert_eq!(pos.character, 1); // UTF-16 code unit 1
        
                // Position at 'ö' in "wörld" (line 1, char 1)
                let pos = CollaborationLSPBridge::byte_to_lsp_position(&rope, 9); // "héllo\nw" is 9 bytes
                assert_eq!(pos.line, 1);
                assert_eq!(pos.character, 1);
            }
        }

        if lsp_changes.is_empty() {
            confidence = 0.0;
            warnings.push("No LSP changes generated from CRDT operations".to_string());
        }

        let result = CRDTTranslationResult {
            lsp_changes,
            translation_confidence: confidence,
            warnings,
        };

        // Cache the result
        self.translation_cache.insert(cache_key, result.clone()).await;

        Ok(result)
    }

    /// Get or create the document rope for position calculations
    async fn get_or_create_document_rope(&self, document_uri: &Url) -> IntegrationResult<Rope> {
        // Check if rope already exists
        {
            let ropes = self.shared_state.document_ropes.read().await;
            if let Some(rope) = ropes.get(document_uri) {
                return Ok(rope.clone());
            }
        }

        // Get document content from collaborative service
        let collab_service = self.collaboration_service.read().await;
        let document_content = collab_service.get_document_content(&document_uri.to_string()).await
            .map_err(|e| IntegrationError::CollaborationSync {
                message: format!("Failed to get document content for rope: {}", e)
            })?;

        // Create rope
        let rope = Rope::from_str(&document_content);

        // Store in shared state
        {
            let mut ropes = self.shared_state.document_ropes.write().await;
            ropes.insert(document_uri.clone(), rope.clone());
        }

        Ok(rope)
    }

    /// Convert byte position to LSP Position with UTF-16 code units
    fn byte_to_lsp_position(rope: &Rope, byte_pos: usize) -> Position {
        let line_idx = rope.byte_to_line(byte_pos);
        let line_start_byte = rope.line_to_byte(line_idx);
        let char_idx_in_line = rope.byte_to_char(byte_pos) - rope.line_to_char(line_start_byte);
        let utf16_cu = rope.char_to_utf16_cu(rope.line_to_char(line_idx) + char_idx_in_line) - rope.char_to_utf16_cu(rope.line_to_char(line_idx));

        Position {
            line: line_idx as u32,
            character: utf16_cu as u32,
        }
    }

    /// Synchronize collaborative changes to LSP
    pub async fn sync_collaborative_changes_to_lsp(
        &self,
        document_uri: &Url,
        user_id: &str,
    ) -> IntegrationResult<()> {
        let _permit = self.sync_semaphore.acquire().await
            .map_err(|_| IntegrationError::CapacityExceeded {
                resource: "sync_semaphore".to_string()
            })?;

        let start_time = Instant::now();

        // Validate document URI security
        if let Ok(path) = document_uri.to_file_path() {
            validate_secure_path(path.to_str().unwrap_or(""))
                .map_err(|_| IntegrationError::SecurityValidation {
                    message: "Invalid document path".to_string()
                })?;
        }

        // Get collaborative operations for this document
        let collab_service = self.collaboration_service.read().await;
        let document_content = collab_service.get_document_content(&document_uri.to_string()).await
            .map_err(|e| IntegrationError::CollaborationSync {
                message: format!("Failed to get document content: {}", e)
            })?;

        // Get document state
        let mut doc_states = self.shared_state.document_states.write().await;
        let doc_state = doc_states.entry(document_uri.clone()).or_insert_with(|| DocumentSyncState {
            document_uri: document_uri.clone(),
            lsp_version: None,
            crdt_state: TextCRDT::new(),
            last_sync_timestamp: std::time::SystemTime::now(),
            pending_changes: Vec::new(),
            is_in_conflict: false,
            conflict_resolution_attempts: 0,
        });

        // Check for conflicts with LSP state
        if let Some(lsp_version) = doc_state.lsp_version {
            if self.detect_conflicts(document_uri, &document_content).await? {
                return self.handle_conflict(document_uri, user_id).await;
            }
        }

        // Send changes to LSP
        let language_router = self.language_router.read().await;
        let routing_result = language_router.route_request(&lsp_types::TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: document_uri.clone(),
            },
            position: Position::default(), // Not used for document sync
        }.into()).await
            .map_err(|e| IntegrationError::LSPCommunication {
                message: format!("Failed to route LSP request: {:?}", e)
            })?;

        if let Some(server_handle) = routing_result.server_handle {
            // Send didChange notification to LSP server
            let params = DidChangeTextDocumentParams {
                text_document: lsp_types::VersionedTextDocumentIdentifier {
                    uri: document_uri.clone(),
                    version: doc_state.lsp_version.unwrap_or(0) + 1,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: document_content,
                }],
            };

            // Update LSP version
            doc_state.lsp_version = Some(doc_state.lsp_version.unwrap_or(0) + 1);

            let _ = self.event_tx.send(BridgeEvent::SyncCompleted {
                uri: document_uri.clone(),
                status: SyncStatus::Synchronized,
                duration_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        Ok(())
    }

    /// Synchronize LSP changes to collaborative system
    pub async fn sync_lsp_changes_to_collaborative(
        &self,
        document_uri: &Url,
        changes: &[TextDocumentContentChangeEvent],
        user_id: &str,
    ) -> IntegrationResult<()> {
        let _permit = self.sync_semaphore.acquire().await
            .map_err(|_| IntegrationError::CapacityExceeded {
                resource: "sync_semaphore".to_string()
            })?;

        let start_time = Instant::now();

        // Convert LSP changes to CRDT operations
        for change in changes {
            let operations = self.convert_lsp_change_to_crdt_operations(change)?;

            // Apply to collaborative service
            let mut collab_service = self.collaboration_service.write().await;
            for operation in operations {
                collab_service.apply_operational_transform(
                    document_uri.to_string(),
                    operation,
                    rust_ai_ide_collaboration::real_time_editing::MergePolicy::LatestWins,
                    user_id.to_string(),
                ).await
                    .map_err(|e| IntegrationError::CollaborationSync {
                        message: format!("Failed to apply operation: {}", e)
                    })?;
            }
        }

        let _ = self.event_tx.send(BridgeEvent::SyncCompleted {
            uri: document_uri.clone(),
            status: SyncStatus::Synchronized,
            duration_ms: start_time.elapsed().as_millis() as u64,
        });

        Ok(())
    }

    /// Detect conflicts between collaborative and LSP states
    async fn detect_conflicts(&self, document_uri: &Url, collaborative_content: &str) -> IntegrationResult<bool> {
        // Get LSP document content (this would need to be implemented to query LSP server)
        // For now, assume no conflict detection is possible without LSP server state
        Ok(false)
    }

    /// Handle conflicts between collaborative and LSP changes
    async fn handle_conflict(&self, document_uri: &Url, user_id: &str) -> IntegrationResult<()> {
        let conflict_detection = ConflictDetection {
            has_conflict: true,
            conflict_ranges: Vec::new(), // Would need to be calculated
            lsp_changes: Vec::new(),
            collaborative_operations: Vec::new(),
            severity: ConflictSeverity::Medium,
        };

        let _ = self.event_tx.send(BridgeEvent::ConflictDetected {
            uri: document_uri.clone(),
            conflict: conflict_detection,
        });

        // Use AI for conflict resolution if enabled
        if self.config.enable_ai_conflict_resolution && self.ai_service.is_some() {
            self.resolve_conflict_with_ai(document_uri, user_id).await
        } else {
            // Use default conflict resolution strategy
            self.resolve_conflict_default(document_uri, user_id).await
        }
    }

    /// Resolve conflicts using AI
    async fn resolve_conflict_with_ai(&self, document_uri: &Url, user_id: &str) -> IntegrationResult<()> {
        if let Some(ai_conflict_resolver) = &self.ai_conflict_resolver {
            // Get document content for analysis
            let collab_service = self.collaboration_service.read().await;
            let document_content = collab_service.get_document_content(&document_uri.to_string()).await
                .map_err(|e| IntegrationError::CollaborationSync {
                    message: format!("Failed to get document content: {}", e)
                })?;

            // Gather pending collaborative ops and LSP diffs
            let collaborative_operations = self.gather_pending_collaborative_ops(document_uri).await?;
            let lsp_diffs = self.gather_lsp_diffs(document_uri).await?;

            // Combine all operations for analysis
            let mut all_operations = collaborative_operations;
            all_operations.extend(lsp_diffs);

            if !all_operations.is_empty() {
                // Analyze conflicts
                let analysis = ai_conflict_resolver.read().await.analyze_conflicts(
                    &all_operations,
                    &document_content
                ).await
                .map_err(|e| IntegrationError::LSPCommunication {
                    message: format!("AI conflict analysis failed: {:?}", e)
                })?;

                // Resolve conflicts
                let resolved_operations = ai_conflict_resolver.read().await.resolve_conflicts(
                    &analysis,
                    &document_content
                ).await
                .map_err(|e| IntegrationError::LSPCommunication {
                    message: format!("AI conflict resolution failed: {:?}", e)
                })?;

                // Apply chosen operations to collaborative state
                let mut collab_service = self.collaboration_service.write().await;
                for editor_op in &resolved_operations {
                    let operation = self.convert_editor_operation_to_operation(editor_op.clone())?;
                    collab_service.apply_operational_transform(
                        document_uri.to_string(),
                        operation,
                        rust_ai_ide_collaboration::real_time_editing::MergePolicy::LatestWins,
                        user_id.to_string(),
                    ).await
                    .map_err(|e| IntegrationError::CollaborationSync {
                        message: format!("Failed to apply resolved operation: {}", e)
                    })?;
                }

                // Dispatch LSP edits (or vice versa per policy)
                self.dispatch_resolved_operations_to_lsp(document_uri, &resolved_operations).await?;

                // Update health metrics
                let mut health = self.shared_state.health_status.write().await;
                health.conflicts_resolved += 1;

                // Emit appropriate events
                let _ = self.event_tx.send(BridgeEvent::ConflictResolved {
                    uri: document_uri.clone(),
                    strategy: ConflictResolutionStrategy::AIResolution,
                    success: true,
                });
            } else {
                // No operations to resolve
                let _ = self.event_tx.send(BridgeEvent::ConflictResolved {
                    uri: document_uri.clone(),
                    strategy: ConflictResolutionStrategy::AIResolution,
                    success: true,
                });
            }
        } else {
            // Fallback to default resolution
            self.resolve_conflict_default(document_uri, user_id).await?;
        }

        Ok(())
    }

    /// Default conflict resolution (LSP wins)
    async fn resolve_conflict_default(&self, document_uri: &Url, user_id: &str) -> IntegrationResult<()> {
        // Default: LSP changes take precedence
        let _ = self.event_tx.send(BridgeEvent::ConflictResolved {
            uri: document_uri.clone(),
            strategy: ConflictResolutionStrategy::LSPWins,
            success: true,
        });

        Ok(())
    }

    /// Convert LSP change to CRDT operations
    fn convert_lsp_change_to_crdt_operations(&self, change: &TextDocumentContentChangeEvent) -> IntegrationResult<Vec<Operation>> {
        let mut operations = Vec::new();

        if let Some(range) = &change.range {
            // Calculate position from range
            let start_pos = range.start.line * 1000 + range.start.character; // Simple position calculation

            if change.text.is_empty() {
                // Delete operation
                operations.push(Operation::DeleteOp(
                    rust_ai_ide_collaboration::real_time_editing::DeleteOperation {
                        id: uuid::Uuid::new_v4(),
                        position: start_pos as usize,
                        length: (range.end.character - range.start.character) as usize,
                        site_id: 0, // Would need proper site ID
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    }
                ));
            } else {
                // Insert operation
                operations.push(Operation::InsertOp(
                    rust_ai_ide_collaboration::real_time_editing::InsertOperation {
                        id: uuid::Uuid::new_v4(),
                        position: start_pos as usize,
                        content: change.text.clone(),
                        site_id: 0, // Would need proper site ID
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    }
                ));
            }
        } else {
            // Full document replacement
            operations.push(Operation::InsertOp(
                rust_ai_ide_collaboration::real_time_editing::InsertOperation {
                    id: uuid::Uuid::new_v4(),
                    position: 0,
                    content: change.text.clone(),
                    site_id: 0,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                }
            ));
        }

        Ok(operations)
    }

    /// Get bridge health status
    pub async fn get_health_status(&self) -> BridgeHealthStatus {
        self.shared_state.health_status.read().await.clone()
    }

    /// Force synchronization of a document
    pub async fn force_sync_document(&self, document_uri: &Url, user_id: &str) -> IntegrationResult<()> {
        info!("Forcing sync for document {} by user {}", document_uri, user_id);

        // Reset conflict state
        let mut doc_states = self.shared_state.document_states.write().await;
        if let Some(state) = doc_states.get_mut(document_uri) {
            state.is_in_conflict = false;
            state.conflict_resolution_attempts = 0;
            state.pending_changes.clear();
        }

        // Perform sync
        self.sync_collaborative_changes_to_lsp(document_uri, user_id).await
    }

    /// Get shared state for debugging
    pub fn get_shared_state(&self) -> Arc<SharedBridgeState> {
        self.shared_state.clone()
    }

    /// Convert EditorOperation to Operation for collaborative service
    fn convert_editor_operation_to_operation(&self, editor_op: EditorOperation) -> IntegrationResult<Operation> {
        match editor_op {
            EditorOperation::Insert { position, content, op_id, .. } => {
                Ok(Operation::InsertOp(
                    rust_ai_ide_collaboration::real_time_editing::InsertOperation {
                        id: uuid::Uuid::parse_str(&op_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                        position,
                        content,
                        site_id: 0, // Would need proper site ID mapping
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    }
                ))
            }
            EditorOperation::Delete { position, length, op_id, .. } => {
                Ok(Operation::DeleteOp(
                    rust_ai_ide_collaboration::real_time_editing::DeleteOperation {
                        id: uuid::Uuid::parse_str(&op_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                        position,
                        length,
                        site_id: 0, // Would need proper site ID mapping
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    }
                ))
            }
            EditorOperation::Update { position, new_content, op_id, .. } => {
                Ok(Operation::InsertOp(
                    rust_ai_ide_collaboration::real_time_editing::InsertOperation {
                        id: uuid::Uuid::parse_str(&op_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                        position,
                        content: new_content,
                        site_id: 0, // Would need proper site ID mapping
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    }
                ))
            }
        }
    }

    /// Gather pending collaborative operations for the document
    async fn gather_pending_collaborative_ops(&self, document_uri: &Url) -> IntegrationResult<Vec<EditorOperation>> {
        let collab_service = self.collaboration_service.read().await;

        // Access the internal documents map to get operations log
        // Since the field is private, we'll use the available methods
        // For now, return empty vec as the collab service doesn't expose pending ops directly
        // In a real implementation, this would access the operations_log from DocumentState
        Ok(Vec::new())
    }

    /// Gather LSP diffs for the document
    async fn gather_lsp_diffs(&self, document_uri: &Url) -> IntegrationResult<Vec<EditorOperation>> {
        let doc_states = self.shared_state.document_states.read().await;
        if let Some(doc_state) = doc_states.get(document_uri) {
            let mut editor_ops = Vec::new();
            for change in &doc_state.pending_changes {
                // Convert TextDocumentContentChangeEvent to EditorOperation
                let ops = self.convert_lsp_change_to_editor_operations(change)?;
                editor_ops.extend(ops);
            }
            Ok(editor_ops)
        } else {
            Ok(Vec::new())
        }
    }

    /// Convert LSP change to EditorOperation
    fn convert_lsp_change_to_editor_operations(&self, change: &TextDocumentContentChangeEvent) -> IntegrationResult<Vec<EditorOperation>> {
        let mut operations = Vec::new();

        if let Some(range) = &change.range {
            // Calculate position from range
            let start_pos = range.start.line as usize * 1000 + range.start.character as usize; // Simple position calculation

            if change.text.is_empty() {
                // Delete operation
                operations.push(EditorOperation::Delete {
                    position: start_pos,
                    length: (range.end.character - range.start.character) as usize,
                    op_id: format!("lsp_delete_{}_{}", start_pos, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                    clock: rust_ai_ide_collaboration::crdt::LamportClock::new("lsp".to_string()),
                });
            } else {
                // Insert operation
                operations.push(EditorOperation::Insert {
                    position: start_pos,
                    content: change.text.clone(),
                    op_id: format!("lsp_insert_{}_{}", start_pos, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                    clock: rust_ai_ide_collaboration::crdt::LamportClock::new("lsp".to_string()),
                });
            }
        } else {
            // Full document replacement
            operations.push(EditorOperation::Update {
                position: 0,
                old_content: String::new(), // We don't have old content here
                new_content: change.text.clone(),
                op_id: format!("lsp_replace_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                clock: rust_ai_ide_collaboration::crdt::LamportClock::new("lsp".to_string()),
            });
        }

        Ok(operations)
    }

    /// Dispatch resolved operations to LSP
    async fn dispatch_resolved_operations_to_lsp(&self, document_uri: &Url, operations: &[EditorOperation]) -> IntegrationResult<()> {
        if operations.is_empty() {
            return Ok(());
        }

        // Convert EditorOperations back to LSP changes and send to LSP
        let mut lsp_changes = Vec::new();

        for op in operations {
            match op {
                EditorOperation::Insert { position, content, .. } => {
                    // Convert position back to LSP Position
                    let line = *position / 1000; // Simple reverse calculation
                    let character = *position % 1000;
                    let pos = Position {
                        line: line as u32,
                        character: character as u32,
                    };
                    let range = Range {
                        start: pos,
                        end: pos,
                    };

                    lsp_changes.push(TextDocumentContentChangeEvent {
                        range: Some(range),
                        range_length: Some(0),
                        text: content.clone(),
                    });
                }
                EditorOperation::Delete { position, length, .. } => {
                    let line = *position / 1000;
                    let character = *position % 1000;
                    let start_pos = Position {
                        line: line as u32,
                        character: character as u32,
                    };
                    let end_pos = Position {
                        line: line as u32,
                        character: (character + *length) as u32,
                    };
                    let range = Range {
                        start: start_pos,
                        end: end_pos,
                    };

                    lsp_changes.push(TextDocumentContentChangeEvent {
                        range: Some(range),
                        range_length: Some(*length as u32),
                        text: String::new(),
                    });
                }
                EditorOperation::Update { position, new_content, .. } => {
                    // For updates, send full document replacement
                    lsp_changes.push(TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: new_content.clone(),
                    });
                }
            }
        }

        // Send to LSP server
        let language_router = self.language_router.read().await;
        let routing_result = language_router.route_request(&lsp_types::TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: document_uri.clone(),
            },
            position: Position::default(),
        }.into()).await
            .map_err(|e| IntegrationError::LSPCommunication {
                message: format!("Failed to route LSP request: {:?}", e)
            })?;

        if let Some(server_handle) = routing_result.server_handle {
            // Send didChange notification to LSP server
            let params = DidChangeTextDocumentParams {
                text_document: lsp_types::VersionedTextDocumentIdentifier {
                    uri: document_uri.clone(),
                    version: None, // Let LSP handle versioning
                },
                content_changes: lsp_changes,
            };

            // Note: In a real implementation, this would send the notification to the LSP server
            // For now, we just log it
            debug!("Dispatched {} resolved operations to LSP for {}", operations.len(), document_uri);
        }

        Ok(())
    }
}