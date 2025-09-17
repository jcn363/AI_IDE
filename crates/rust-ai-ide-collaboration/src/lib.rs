//! Real-time collaboration system for Rust AI IDE.
//!
//! This crate implements CRDT-based collaborative editing with Lamport clocks,
//! AI-mediated conflict resolution, WebSocket communication with TLS 1.3,
//! performance monitoring with threshold-based alerting, and real-time communication infrastructure.

pub mod ai_coaching;
pub mod ai_conflict_resolution;
pub mod commands;
pub mod crdt;
pub mod distributed_workspace;
pub mod performance_monitoring;
pub mod presence;
pub mod real_time_editing;
pub mod session_management;
pub mod team_management;
pub mod websocket;

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub use distributed_workspace::*;
pub use presence::*;
use serde::{Deserialize, Serialize};
pub use session_management::*;
pub use team_management::*;

/// Core collaboration service that manages sessions and state
pub struct CollaborationService {
    state: Arc<RwLock<CollaborationState>>,
    distributed_workspace_manager: Option<Arc<DistributedWorkspaceManager>>,
    team_service: Option<Arc<TeamManagementService>>,
    real_time_editing_service: Option<Arc<real_time_editing::RealTimeEditingService>>,
    ai_coaching_service: Option<Arc<ai_coaching::AICoachingService>>,
    event_sender: mpsc::UnboundedSender<CollaborationEvent>,
    event_receiver: Arc<RwLock<mpsc::UnboundedReceiver<CollaborationEvent>>>,
}

/// Global state for the collaboration system
#[derive(Default)]
pub struct CollaborationState {
    pub sessions: Vec<CollaborationSession>,
    pub active_editing_state: std::collections::HashMap<String, EditingDocument>,
    pub workspace_states: std::collections::HashMap<String, WorkspaceState>,
    pub active_participants: std::collections::HashMap<String, Vec<String>>,
}

/// Events that can occur during collaboration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CollaborationEvent {
    /// Workspace-related events
    WorkspaceCreated { workspace_id: String, owner_id: String },
    WorkspaceJoined { workspace_id: String, user_id: String },
    WorkspaceLeft { workspace_id: String, user_id: String },
    WorkspaceOperation { workspace_id: String, operation: WorkspaceOperation },

    /// Editing-related events
    DocumentEdited { session_id: String, user_id: String, operation: crate::crdt::CRDTOperation },
    ConflictDetected { session_id: String, conflicts: Vec<String> },
    ConflictResolved { session_id: String, resolution: String },

    /// Team-related events
    TeamMemberAdded { team_id: String, user_id: String },
    TeamMemberRemoved { team_id: String, user_id: String },

    /// AI coaching events
    CoachingHint { session_id: String, hint: String },
    CodeSuggestion { session_id: String, suggestion: String },
    LearningOpportunity { session_id: String, opportunity: String },
}

/// Represents a single collaboration session
#[derive(Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub id:            String,
    pub participants:  Vec<String>,
    pub document_id:   String,
    pub last_activity: std::time::SystemTime,
}

/// Represents a document being collaboratively edited
#[derive(Clone, Serialize, Deserialize)]
pub struct EditingDocument {
    pub content:      String,
    pub crdt_state:   CRDTState,
    pub participants: Vec<String>,
}

impl CollaborationService {
    /// Create new collaboration service with all integrated components
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            state: Arc::new(RwLock::new(CollaborationState::default())),
            distributed_workspace_manager: None,
            team_service: None,
            real_time_editing_service: None,
            ai_coaching_service: None,
            event_sender: tx,
            event_receiver: Arc::new(RwLock::new(rx)),
        }
    }

    /// Initialize with integrated services
    pub async fn with_services(
        mut self,
        distributed_workspace_manager: Arc<DistributedWorkspaceManager>,
        team_service: Arc<TeamManagementService>,
        real_time_editing_service: Arc<real_time_editing::RealTimeEditingService>,
        ai_coaching_service: Arc<ai_coaching::AICoachingService>,
    ) -> Self {
        self.distributed_workspace_manager = Some(distributed_workspace_manager);
        self.team_service = Some(team_service);
        self.real_time_editing_service = Some(real_time_editing_service);
        self.ai_coaching_service = Some(ai_coaching_service);
        self
    }

    /// Create enhanced collaborative session with workspace integration
    pub async fn create_enhanced_session(
        &self,
        session_id: String,
        document_id: String,
        workspace_id: Option<String>,
        team_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.write().await;

        // Create the session
        state.sessions.push(CollaborationSession {
            id: session_id.clone(),
            participants: Vec::new(),
            document_id: document_id.clone(),
            last_activity: std::time::SystemTime::now(),
        });

        // Initialize document state with workspace integration
        state.active_editing_state.insert(
            document_id.clone(),
            EditingDocument {
                content: String::new(),
                crdt_state: CRDTState::default(),
                participants: Vec::new(),
            },
        );

        // Initialize workspace state if provided
        if let Some(workspace_id) = &workspace_id {
            if let Some(manager) = &self.distributed_workspace_manager {
                // Try to get workspace state, create if doesn't exist
                if manager.get_workspace_state(workspace_id, "system").await.is_err() {
                    manager.create_workspace(
                        format!("Workspace for {}", session_id),
                        "system".to_string(),
                        team_id.clone(),
                        None,
                    ).await?;
                }
            }
        }

        // Broadcast session creation event
        let _ = self.event_sender.send(CollaborationEvent::WorkspaceCreated {
            workspace_id: workspace_id.unwrap_or_else(|| format!("session_{}", session_id)),
            owner_id: "system".to_string(),
        });

        Ok(())
    }

    /// Apply workspace operation with real-time editing integration
    pub async fn apply_workspace_operation(
        &self,
        workspace_id: &str,
        operation: WorkspaceOperation,
        user_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(manager) = &self.distributed_workspace_manager {
            manager.apply_workspace_operation(
                workspace_id.to_string(),
                operation.clone(),
                user_id.to_string(),
            ).await?;

            // Broadcast the operation event
            let _ = self.event_sender.send(CollaborationEvent::WorkspaceOperation {
                workspace_id: workspace_id.to_string(),
                operation,
            });
        }

        Ok(())
    }

    /// Handle document editing with AI coaching integration
    pub async fn handle_document_edit(
        &self,
        session_id: &str,
        operation: crate::crdt::CRDTOperation,
        user_id: &str,
        context: &real_time_editing::DocumentState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Resolve document_id from sessions
        let state = self.state.read().await;
        let document_id = state.sessions.iter().find(|s| s.id == session_id).map(|s| s.document_id.clone()).unwrap_or_else(|| session_id.to_string());

        // Apply the edit operation
        let site_id = match &operation {
            CRDTOperation::Insert { site_id, .. } => *site_id,
            CRDTOperation::Delete { site_id, .. } => *site_id,
        };
        if let Some(editing_service) = &self.real_time_editing_service {
            editing_service.apply_crdt_operation(
                document_id,
                operation.clone(),
                site_id,
            ).await?;
        }

        // Trigger AI coaching suggestions
        if let Some(ai_service) = &self.ai_coaching_service {
            // Extract cursor position from operation
            let cursor_position = match &operation {
                CRDTOperation::Insert { pos, .. } | CRDTOperation::Delete { pos, .. } => (*pos as u32, 0),
            };
            // Convert CRDT operation to coaching context
            let code_context = ai_coaching::CodeContext {
                current_file: Some(document_id.clone()), // Use actual document ID
                visible_code: context.crdt.to_string(),
                cursor_position, // Accurate cursor position from operation
                programming_language: "rust".to_string(),
                project_context: "collaborative editing".to_string(),
            };

            let coaching_event = ai_service.provide_contextual_coaching(
                session_id,
                &code_context,
                &vec![], // user_actions - would be populated
            ).await?;

            // Convert to collaboration event
            match coaching_event {
                ai_coaching::AICoachingEvent::DynamicSuggestion(suggestion) => {
                    let _ = self.event_sender.send(CollaborationEvent::CodeSuggestion {
                        session_id: session_id.to_string(),
                        suggestion: suggestion.content,
                    });
                }
                ai_coaching::AICoachingEvent::ContextualHint(hint) => {
                    let _ = self.event_sender.send(CollaborationEvent::CoachingHint {
                        session_id: session_id.to_string(),
                        hint: hint.hint,
                    });
                }
                _ => {}
            }
        }

        // Broadcast edit event
        let _ = self.event_sender.send(CollaborationEvent::DocumentEdited {
            session_id: session_id.to_string(),
            user_id: user_id.to_string(),
            operation,
        });

        Ok(())
    }

    /// Join workspace with team validation
    pub async fn join_workspace(
        &self,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate team permissions if workspace belongs to a team
        if let Some(manager) = &self.distributed_workspace_manager {
            if let Ok(workspace) = manager.get_workspace_state(workspace_id, user_id).await {
                if let Some(team_id) = workspace.team_id {
                    if let Some(team_service) = &self.team_service {
                        if !team_service.can_user_access_project(
                            user_id,
                            &format!("workspace_{}", workspace_id),
                            &Permission::EditDocument,
                        ).await.unwrap_or(false) {
                            return Err("User not authorized to join this workspace".into());
                        }
                    }
                }
            }

            manager.join_workspace(workspace_id.to_string(), user_id.to_string()).await?;
        }

        // Update active participants
        let mut state = self.state.write().await;
        state.active_participants
            .entry(workspace_id.to_string())
            .or_insert_with(Vec::new)
            .push(user_id.to_string());

        // Broadcast join event
        let _ = self.event_sender.send(CollaborationEvent::WorkspaceJoined {
            workspace_id: workspace_id.to_string(),
            user_id: user_id.to_string(),
        });

        Ok(())
    }

    /// Leave workspace
    pub async fn leave_workspace(
        &self,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(manager) = &self.distributed_workspace_manager {
            manager.leave_workspace(workspace_id.to_string(), user_id.to_string()).await?;
        }

        // Update active participants
        let mut state = self.state.write().await;
        if let Some(participants) = state.active_participants.get_mut(workspace_id) {
            participants.retain(|p| p != user_id);
        }

        // Broadcast leave event
        let _ = self.event_sender.send(CollaborationEvent::WorkspaceLeft {
            workspace_id: workspace_id.to_string(),
            user_id: user_id.to_string(),
        });

        Ok(())
    }

    /// Get collaboration events (for real-time updates)
    pub async fn get_next_event(&self) -> Option<CollaborationEvent> {
        let mut receiver = self.event_receiver.write().await;
        receiver.recv().await
    }

    /// Backward compatibility method
    pub async fn create_session(
        &self,
        session_id: String,
        document_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.create_enhanced_session(session_id, document_id, None, None).await
    }
}

/// Basic CRDT state representation (placeholder for full implementation)
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CRDTState {
    pub operations: Vec<crate::crdt::CRDTOperation>,
}
