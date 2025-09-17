//! Distributed workspace state management for collaborative IDE.
//!
//! Implements CRDT-based workspace coordination across multiple clients with
//! conflict-free operations for file system changes, permission-based access control,
//! workspace event broadcasting, and robust state persistence and recovery.

use std::collections::HashMap;
use std::sync::Arc;
use std::fs;
use std::path::Path;

use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_security::audit_logger;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use rusqlite::{Connection, params, Row};
use log;

use crate::crdt::operational_transform::{OperationTransform, OperationalTransformer};
use crate::crdt::{EditorOperation, CRDT};
use crate::session_management::{EnhancedCollaborationService, UserRole, Permission};
use crate::team_management::TeamManagementService;

/// Distributed workspace manager coordinating state across multiple clients
#[derive(Clone)]
pub struct DistributedWorkspaceManager {
    workspaces: Arc<RwLock<HashMap<String, WorkspaceState>>>,
    workspace_crdt: Arc<RwLock<HashMap<String, WorkspaceCRDT>>>,
    event_broadcaster: mpsc::UnboundedSender<WorkspaceEvent>,
    event_receiver: Arc<RwLock<mpsc::UnboundedReceiver<WorkspaceEvent>>>,
    team_service: Arc<TeamManagementService>,
    session_service: Arc<EnhancedCollaborationService>,
    operational_transformer: OperationalTransformer,
    persistence_enabled: bool,
    database_connection: Arc<RwLock<Option<Connection>>>,
}

/// CRDT implementation for workspace-level operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceCRDT {
    pub workspace_id: String,
    pub operations: Vec<EditorOperation>,
    pub lamport_clock: u64,
    pub site_id: u32,
    pub tombstone: Vec<Option<WorkspaceEntry>>,
}

/// Workspace entry representing files/directories in the workspace
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceEntry {
    pub path: String,
    pub entry_type: EntryType,
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
    pub modified_at: u64,
}

/// Type of workspace entry
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    File,
    Directory,
}

/// CRDT operations for workspace changes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkspaceOperation {
    AddEntry {
        path: String,
        entry_type: EntryType,
        lamport_clock: u64,
        site_id: u32,
    },
    RemoveEntry {
        path: String,
        lamport_clock: u64,
        site_id: u32,
    },
    RenameEntry {
        old_path: String,
        new_path: String,
        lamport_clock: u64,
        site_id: u32,
    },
    MoveEntry {
        from_path: String,
        to_path: String,
        lamport_clock: u64,
        site_id: u32,
    },
}

/// Workspace state with distributed coordination
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub workspace_id: String,
    pub name: String,
    pub owner_id: String,
    pub team_id: Option<String>,
    pub entries: HashMap<String, WorkspaceEntry>,
    pub participants: Vec<String>,
    pub settings: WorkspaceSettings,
    pub last_sync_timestamp: std::time::SystemTime,
    pub version: u64,
}

/// Workspace settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub allow_public_access: bool,
    pub max_file_size: u64,
    pub allowed_file_types: Vec<String>,
    pub auto_sync_enabled: bool,
}

/// Events broadcast to workspace participants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkspaceEvent {
    EntryAdded {
        workspace_id: String,
        path: String,
        entry_type: EntryType,
        user_id: String,
    },
    EntryRemoved {
        workspace_id: String,
        path: String,
        user_id: String,
    },
    EntryRenamed {
        workspace_id: String,
        old_path: String,
        new_path: String,
        user_id: String,
    },
    EntryMoved {
        workspace_id: String,
        from_path: String,
        to_path: String,
        user_id: String,
    },
    ParticipantJoined {
        workspace_id: String,
        user_id: String,
    },
    ParticipantLeft {
        workspace_id: String,
        user_id: String,
    },
    SettingsChanged {
        workspace_id: String,
        user_id: String,
    },
}

/// Operational transform for workspace structural changes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceOperationTransform {
    pub operation: EditorOperation,
    pub context: OperationContext,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationContext {
    pub workspace_id: String,
    pub user_id: String,
    pub timestamp: u64,
    pub session_id: String,
}

impl DistributedWorkspaceManager {
    /// Create new distributed workspace manager
    pub fn new(
        team_service: Arc<TeamManagementService>,
        session_service: Arc<EnhancedCollaborationService>,
        persistence_enabled: bool,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            workspace_crdt: Arc::new(RwLock::new(HashMap::new())),
            event_broadcaster: tx,
            event_receiver: Arc::new(RwLock::new(rx)),
            team_service,
            session_service,
            operational_transformer: OperationalTransformer::new(
                "workspace-manager".to_string(),
                "system-session".to_string(),
            ),
            persistence_enabled,
            database_connection: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize database connection and create schema
    pub async fn initialize_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.persistence_enabled {
            return Ok(());
        }

        let db_path = Path::new(".ide").join("workspaces.db");
        let conn = Connection::open(&db_path)?;

        // Create workspace_states table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS workspace_states (
                workspace_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                owner_id TEXT NOT NULL,
                team_id TEXT,
                settings TEXT NOT NULL,
                participants TEXT NOT NULL,
                last_sync_timestamp TEXT NOT NULL,
                version INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create workspace_entries table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS workspace_entries (
                workspace_id TEXT NOT NULL,
                path TEXT NOT NULL,
                entry_type TEXT NOT NULL,
                metadata TEXT,
                created_at INTEGER NOT NULL,
                modified_at INTEGER NOT NULL,
                PRIMARY KEY (workspace_id, path),
                FOREIGN KEY (workspace_id) REFERENCES workspace_states (workspace_id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create workspace_crdts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS workspace_crdts (
                workspace_id TEXT PRIMARY KEY,
                operations TEXT NOT NULL,
                lamport_clock INTEGER NOT NULL,
                site_id INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        {
            let mut db_conn = self.database_connection.write().await;
            *db_conn = Some(conn);
        }

        Ok(())
    }

    /// Create a new distributed workspace
    pub async fn create_workspace(
        &self,
        name: String,
        owner_id: String,
        team_id: Option<String>,
        settings: Option<WorkspaceSettings>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Validate user permissions
        if let Some(ref team_id) = team_id {
            let can_create = self.team_service.can_user_access_project(
                &owner_id,
                &format!("workspace_{}", team_id),
                &Permission::ManageTeam,
            ).await.unwrap_or(false);

            if !can_create {
                return Err("Insufficient permissions to create workspace in this team".into());
            }
        }

        let workspace_id = format!("workspace_{}", Uuid::new_v4());
        let workspace = WorkspaceState {
            workspace_id: workspace_id.clone(),
            name: name.clone(),
            owner_id: owner_id.clone(),
            team_id: team_id.clone(),
            entries: HashMap::new(),
            participants: vec![owner_id.clone()],
            settings: settings.unwrap_or_default(),
            last_sync_timestamp: std::time::SystemTime::now(),
            version: 0,
        };

        // Initialize CRDT state
        let crdt = WorkspaceCRDT {
            workspace_id: workspace_id.clone(),
            operations: Vec::new(),
            lamport_clock: 0,
            site_id: 0,
            tombstone: Vec::new(),
        };

        // Store workspace and CRDT state
        {
            let mut workspaces = self.workspaces.write().await;
            workspaces.insert(workspace_id.clone(), workspace);
        }

        {
            let mut crdts = self.workspace_crdt.write().await;
            crdts.insert(workspace_id.clone(), crdt);
        }

        // Broadcast workspace creation event
        let _ = self.event_broadcaster.send(WorkspaceEvent::ParticipantJoined {
            workspace_id: workspace_id.clone(),
            user_id: owner_id.clone(),
        });

        audit_logger::log_event(
            "workspace_created",
            &format!("Workspace '{}' created by user {}", name, owner_id),
        );

        Ok(workspace_id)
    }

    /// Apply CRDT operation to workspace
    pub async fn apply_workspace_operation(
        &self,
        workspace_id: String,
        operation: EditorOperation,
        user_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate user permissions
        self.validate_workspace_access(&workspace_id, &user_id, &Permission::EditDocument).await?;

        // Get CRDT state
        let mut crdts = self.workspace_crdt.write().await;
        let crdt = crdts.get_mut(&workspace_id)
            .ok_or_else(|| format!("Workspace {} not found", workspace_id))?;

        // Increment Lamport clock
        crdt.lamport_clock += 1;

        // Update operation with clock
        let operation = self.update_operation_clock(&operation, crdt.lamport_clock, crdt.site_id);

        // Apply operation to CRDT
        crdt.apply_operation(operation.clone())?;

        // Record operation
        crdt.operations.push(operation.clone());

        // Update workspace state
        self.apply_operation_to_workspace(&workspace_id, &operation, &user_id).await?;

        // Broadcast event
        self.broadcast_workspace_event(&workspace_id, &operation, &user_id).await;

        Ok(())
    }

    /// Apply operational transform for workspace structural changes
    pub async fn apply_workspace_transform(
        &self,
        workspace_id: String,
        operation: EditorOperation,
        user_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate access
        self.validate_workspace_access(&workspace_id, &user_id, &Permission::EditDocument).await?;

        // Create operation transform
        let transform = WorkspaceOperationTransform {
            operation: operation.clone(),
            context: OperationContext {
                workspace_id: workspace_id.clone(),
                user_id: user_id.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
                session_id: format!("session_{}", Uuid::new_v4()),
            },
        };

        // Transform operation against concurrent operations
        let transformed_operation = self.transform_workspace_operation(&workspace_id, operation).await?;

        // Apply transformed operation
        self.apply_workspace_operation(workspace_id, transformed_operation, user_id).await?;

        Ok(())
    }

    /// Synchronize workspace state from remote source
    pub async fn synchronize_workspace_state(
        &self,
        workspace_id: String,
        remote_operations: Vec<EditorOperation>,
        user_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate access
        self.validate_workspace_access(&workspace_id, &user_id, &Permission::EditDocument).await?;

        for operation in remote_operations {
            self.apply_workspace_operation(workspace_id.clone(), operation, user_id.clone()).await?;
        }

        // Update sync timestamp
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(&workspace_id) {
                workspace.last_sync_timestamp = std::time::SystemTime::now();
            }
        }

        Ok(())
    }

    /// Add participant to workspace
    pub async fn join_workspace(
        &self,
        workspace_id: String,
        user_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate team membership if workspace belongs to a team
        let workspaces = self.workspaces.read().await;
        let workspace = workspaces.get(&workspace_id)
            .ok_or_else(|| format!("Workspace {} not found", workspace_id))?;

        if let Some(ref team_id) = workspace.team_id {
            let can_access = self.team_service.can_user_access_project(
                &user_id,
                &format!("workspace_{}", team_id),
                &Permission::ReadDocument,
            ).await.unwrap_or(false);

            if !can_access {
                return Err("User not authorized to join this workspace".into());
            }
        }

        // Add participant
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(&workspace_id) {
                if !workspace.participants.contains(&user_id) {
                    workspace.participants.push(user_id.clone());
                }
            }
        }

        // Broadcast join event
        let _ = self.event_broadcaster.send(WorkspaceEvent::ParticipantJoined {
            workspace_id,
            user_id: user_id.clone(),
        });

        audit_logger::log_event(
            "workspace_joined",
            &format!("User {} joined workspace {}", user_id, workspace_id),
        );

        Ok(())
    }

    /// Remove participant from workspace
    pub async fn leave_workspace(
        &self,
        workspace_id: String,
        user_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Remove participant
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(&workspace_id) {
                workspace.participants.retain(|p| p != &user_id);
            }
        }

        // Broadcast leave event
        let _ = self.event_broadcaster.send(WorkspaceEvent::ParticipantLeft {
            workspace_id,
            user_id: user_id.clone(),
        });

        audit_logger::log_event(
            "workspace_left",
            &format!("User {} left workspace {}", user_id, workspace_id),
        );

        Ok(())
    }

    /// Get workspace state with validation
    pub async fn get_workspace_state(
        &self,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<WorkspaceState, Box<dyn std::error::Error>> {
        // Validate access
        self.validate_workspace_access(workspace_id, user_id, &Permission::ReadDocument).await?;

        let workspaces = self.workspaces.read().await;
        let workspace = workspaces.get(workspace_id)
            .ok_or_else(|| format!("Workspace {} not found", workspace_id))?;

        Ok(workspace.clone())
    }

    /// Validate workspace state consistency
    pub async fn validate_workspace_consistency(
        &self,
        workspace_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let crdts = self.workspace_crdt.read().await;
        let crdt = crdts.get(workspace_id)
            .ok_or_else(|| format!("Workspace CRDT {} not found", workspace_id))?;

        // Check CRDT consistency
        let mut reconstructed_entries = HashMap::new();

        for operation in &crdt.operations {
            match operation {
                EditorOperation::AddEntry { path, entry_type, .. } => {
                    if validate_secure_path(path).is_ok() {
                        // Convert string entry_type to EntryType enum
                        let entry_type_enum = match entry_type.as_str() {
                            "file" => EntryType::File,
                            "directory" => EntryType::Directory,
                            _ => EntryType::File, // Default to File for unrecognized types
                        };

                        reconstructed_entries.insert(path.clone(), WorkspaceEntry {
                            path: path.clone(),
                            entry_type: entry_type_enum,
                            metadata: HashMap::new(),
                            created_at: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)?
                                .as_secs(),
                            modified_at: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)?
                                .as_secs(),
                        });
                    }
                }
                EditorOperation::RemoveEntry { path, .. } => {
                    reconstructed_entries.remove(path);
                }
                EditorOperation::RenameEntry { old_path, new_path, .. } => {
                    if let Some(entry) = reconstructed_entries.remove(old_path) {
                        let mut new_entry = entry;
                        new_entry.path = new_path.clone();
                        reconstructed_entries.insert(new_path.clone(), new_entry);
                    }
                }
                EditorOperation::MoveEntry { from_path, to_path, .. } => {
                    if let Some(entry) = reconstructed_entries.remove(from_path) {
                        let mut new_entry = entry;
                        new_entry.path = to_path.clone();
                        reconstructed_entries.insert(to_path.clone(), new_entry);
                    }
                }
                // Ignore text operations for workspace consistency check
                EditorOperation::Insert { .. } | EditorOperation::Delete { .. } | EditorOperation::Update { .. } => {}
            }
        }

        // Compare with current workspace state
        let workspaces = self.workspaces.read().await;
        if let Some(workspace) = workspaces.get(workspace_id) {
            let consistent = reconstructed_entries.len() == workspace.entries.len() &&
                reconstructed_entries.keys().all(|k| workspace.entries.contains_key(k));
            Ok(consistent)
        } else {
            Ok(false)
        }
    }

    /// Persist workspace state to SQLite database
    pub async fn persist_workspace_state(&self, workspace_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.persistence_enabled {
            return Ok(());
        }

        let db_conn = self.database_connection.read().await;
        let conn = db_conn.as_ref()
            .ok_or_else(|| "Database not initialized".to_string())?;

        // Get workspace state
        let workspaces = self.workspaces.read().await;
        let workspace = workspaces.get(workspace_id)
            .ok_or_else(|| format!("Workspace {} not found", workspace_id))?;

        // Get CRDT state
        let crdts = self.workspace_crdt.read().await;
        let crdt = crdts.get(workspace_id)
            .ok_or_else(|| format!("Workspace CRDT {} not found", workspace_id))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Serialize data
        let settings_json = serde_json::to_string(&workspace.settings)?;
        let participants_json = serde_json::to_string(&workspace.participants)?;
        let operations_json = serde_json::to_string(&crdt.operations)?;
        let sync_timestamp = workspace.last_sync_timestamp
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Use transaction for atomic operations
        let tx = conn.unchecked_transaction()?;

        // Insert/update workspace state
        tx.execute(
            "INSERT OR REPLACE INTO workspace_states
             (workspace_id, name, owner_id, team_id, settings, participants, last_sync_timestamp, version, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                workspace.workspace_id,
                workspace.name,
                workspace.owner_id,
                workspace.team_id,
                settings_json,
                participants_json,
                sync_timestamp,
                workspace.version,
                now,
                now
            ],
        )?;

        // Clear existing entries for this workspace
        tx.execute(
            "DELETE FROM workspace_entries WHERE workspace_id = ?",
            params![workspace_id],
        )?;

        // Insert workspace entries
        for (path, entry) in &workspace.entries {
            let metadata_json = serde_json::to_string(&entry.metadata)?;
            tx.execute(
                "INSERT INTO workspace_entries
                 (workspace_id, path, entry_type, metadata, created_at, modified_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    workspace_id,
                    path,
                    match entry.entry_type {
                        EntryType::File => "file",
                        EntryType::Directory => "directory",
                    },
                    metadata_json,
                    entry.created_at,
                    entry.modified_at
                ],
            )?;
        }

        // Insert/update CRDT state
        tx.execute(
            "INSERT OR REPLACE INTO workspace_crdts
             (workspace_id, operations, lamport_clock, site_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                crdt.workspace_id,
                operations_json,
                crdt.lamport_clock,
                crdt.site_id,
                now,
                now
            ],
        )?;

        tx.commit()?;

        audit_logger::log_event(
            "workspace_persisted",
            &format!("Workspace {} state persisted to database", workspace_id),
        );

        Ok(())
    }

    /// Recover workspace state from SQLite database
    pub async fn recover_workspace_state(&self, workspace_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.persistence_enabled {
            return Ok(());
        }

        let db_conn = self.database_connection.read().await;
        let conn = db_conn.as_ref()
            .ok_or_else(|| "Database not initialized".to_string())?;

        // Load workspace state
        let mut stmt = conn.prepare(
            "SELECT name, owner_id, team_id, settings, participants, last_sync_timestamp, version
             FROM workspace_states WHERE workspace_id = ?",
        )?;

        let workspace_row = stmt.query_row(params![workspace_id], |row| {
            let settings_json: String = row.get(3)?;
            let participants_json: String = row.get(4)?;
            let sync_timestamp: u64 = row.get(5)?;

            Ok((
                row.get::<_, String>(0)?, // name
                row.get::<_, String>(1)?, // owner_id
                row.get::<_, Option<String>>(2)?, // team_id
                settings_json,
                participants_json,
                sync_timestamp,
                row.get::<_, u64>(6)?, // version
            ))
        });

        match workspace_row {
            Ok((name, owner_id, team_id, settings_json, participants_json, sync_timestamp, version)) => {
                // Deserialize settings and participants
                let settings: WorkspaceSettings = serde_json::from_str(&settings_json)?;
                let participants: Vec<String> = serde_json::from_str(&participants_json)?;
                let last_sync_timestamp = std::time::UNIX_EPOCH + std::time::Duration::from_secs(sync_timestamp);

                // Load workspace entries
                let mut entries = HashMap::new();
                let mut entries_stmt = conn.prepare(
                    "SELECT path, entry_type, metadata, created_at, modified_at
                     FROM workspace_entries WHERE workspace_id = ?",
                )?;

                let entries_iter = entries_stmt.query_map(params![workspace_id], |row| {
                    let metadata_json: String = row.get(2)?;
                    Ok((
                        row.get::<_, String>(0)?, // path
                        row.get::<_, String>(1)?, // entry_type
                        metadata_json,
                        row.get::<_, u64>(3)?, // created_at
                        row.get::<_, u64>(4)?, // modified_at
                    ))
                })?;

                for entry_result in entries_iter {
                    let (path, entry_type_str, metadata_json, created_at, modified_at) = entry_result?;
                    let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json)?;
                    let entry_type = match entry_type_str.as_str() {
                        "file" => EntryType::File,
                        "directory" => EntryType::Directory,
                        _ => EntryType::File,
                    };

                    entries.insert(path.clone(), WorkspaceEntry {
                        path,
                        entry_type,
                        metadata,
                        created_at,
                        modified_at,
                    });
                }

                let workspace = WorkspaceState {
                    workspace_id: workspace_id.to_string(),
                    name,
                    owner_id,
                    team_id,
                    entries,
                    participants,
                    settings,
                    last_sync_timestamp,
                    version,
                };

                // Load CRDT state
                let mut crdt_stmt = conn.prepare(
                    "SELECT operations, lamport_clock, site_id
                     FROM workspace_crdts WHERE workspace_id = ?",
                )?;

                let crdt_row = crdt_stmt.query_row(params![workspace_id], |row| {
                    let operations_json: String = row.get(0)?;
                    Ok((
                        operations_json,
                        row.get::<_, u64>(1)?, // lamport_clock
                        row.get::<_, u32>(2)?, // site_id
                    ))
                });

                match crdt_row {
                    Ok((operations_json, lamport_clock, site_id)) => {
                        let operations: Vec<EditorOperation> = serde_json::from_str(&operations_json)?;

                        let crdt = WorkspaceCRDT {
                            workspace_id: workspace_id.to_string(),
                            operations,
                            lamport_clock,
                            site_id,
                            tombstone: Vec::new(), // Will be reconstructed from operations
                        };

                        // Store recovered state
                        {
                            let mut workspaces = self.workspaces.write().await;
                            workspaces.insert(workspace_id.to_string(), workspace);
                        }

                        {
                            let mut crdts = self.workspace_crdt.write().await;
                            crdts.insert(workspace_id.to_string(), crdt);
                        }

                        audit_logger::log_event(
                            "workspace_recovered",
                            &format!("Workspace {} state recovered from database", workspace_id),
                        );
                    }
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        // CRDT not found, create empty one
                        let crdt = WorkspaceCRDT {
                            workspace_id: workspace_id.to_string(),
                            operations: Vec::new(),
                            lamport_clock: 0,
                            site_id: 0,
                            tombstone: Vec::new(),
                        };

                        {
                            let mut crdts = self.workspace_crdt.write().await;
                            crdts.insert(workspace_id.to_string(), crdt);
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Workspace not found in database
                return Err(format!("Workspace {} not found in database", workspace_id).into());
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    /// Start background periodic persistence task
    pub async fn start_periodic_persistence(&self, interval_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
        if !self.persistence_enabled {
            return Ok(());
        }

        let manager = Arc::downgrade(&Arc::new(self.clone()));

        tokio::spawn(async move {
            let interval = std::time::Duration::from_secs(interval_seconds);
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if let Some(manager) = manager.upgrade() {
                    // Get all workspace IDs
                    let workspace_ids: Vec<String> = {
                        let workspaces = manager.workspaces.read().await;
                        workspaces.keys().cloned().collect()
                    };

                    // Persist each workspace
                    for workspace_id in workspace_ids {
                        if let Err(e) = manager.persist_workspace_state(&workspace_id).await {
                            log::error!("Failed to persist workspace {}: {:?}", workspace_id, e);
                        }
                    }
                } else {
                    // Manager has been dropped, exit the task
                    break;
                }
            }
        });

        audit_logger::log_event(
            "periodic_persistence_started",
            &format!("Background persistence task started with {}s interval", interval_seconds),
        );

        Ok(())
    }

    /// Recover all workspaces from database on startup
    pub async fn recover_all_workspaces(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.persistence_enabled {
            return Ok(());
        }

        let db_conn = self.database_connection.read().await;
        let conn = db_conn.as_ref()
            .ok_or_else(|| "Database not initialized".to_string())?;

        let mut stmt = conn.prepare("SELECT workspace_id FROM workspace_states")?;
        let workspace_ids: Vec<String> = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        for workspace_id in workspace_ids {
            if let Err(e) = self.recover_workspace_state(&workspace_id).await {
                log::error!("Failed to recover workspace {}: {:?}", workspace_id, e);
            }
        }

        audit_logger::log_event(
            "all_workspaces_recovered",
            &format!("Recovered {} workspaces from database", workspace_ids.len()),
        );

        Ok(())
    }

    /// Save workspace state and CRDT to file
    #[cfg(feature = "workspace-persistence")]
    pub async fn save_workspace(&self, workspace_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.persistence_enabled {
            return Ok(());
        }

        let workspaces = self.workspaces.read().await;
        let workspace = workspaces.get(workspace_id)
            .ok_or_else(|| format!("Workspace {} not found", workspace_id))?;

        let crdts = self.workspace_crdt.read().await;
        let crdt = crdts.get(workspace_id)
            .ok_or_else(|| format!("Workspace CRDT {} not found", workspace_id))?;

        let data = serde_json::json!({
            "state": workspace,
            "crdt": crdt
        });

        let json = serde_json::to_string_pretty(&data)?;

        let dir_path = Path::new(".ide").join("workspaces");
        fs::create_dir_all(&dir_path)?;

        let file_path = dir_path.join(format!("{}.json", workspace_id));
        fs::write(&file_path, json)?;

        audit_logger::log_event(
            "workspace_saved",
            &format!("Workspace {} saved to file", workspace_id),
        );

        Ok(())
    }

    /// Load workspace state and CRDT from file
    #[cfg(feature = "workspace-persistence")]
    pub async fn load_workspace(&self, workspace_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.persistence_enabled {
            return Ok(());
        }

        let file_path = Path::new(".ide").join("workspaces").join(format!("{}.json", workspace_id));

        if !file_path.exists() {
            return Err(format!("Workspace file {} not found", file_path.display()).into());
        }

        let json = fs::read_to_string(&file_path)?;
        let data: serde_json::Value = serde_json::from_str(&json)?;

        let state: WorkspaceState = serde_json::from_value(data["state"].clone())?;
        let crdt: WorkspaceCRDT = serde_json::from_value(data["crdt"].clone())?;

        {
            let mut workspaces = self.workspaces.write().await;
            workspaces.insert(workspace_id.to_string(), state);
        }

        {
            let mut crdts = self.workspace_crdt.write().await;
            crdts.insert(workspace_id.to_string(), crdt);
        }

        audit_logger::log_event(
            "workspace_loaded",
            &format!("Workspace {} loaded from file", workspace_id),
        );

        Ok(())
    }

    // Private helper methods

    async fn validate_workspace_access(
        &self,
        workspace_id: &str,
        user_id: &str,
        required_permission: &Permission,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workspaces = self.workspaces.read().await;
        let workspace = workspaces.get(workspace_id)
            .ok_or_else(|| format!("Workspace {} not found", workspace_id))?;

        // Check if user is participant
        if !workspace.participants.contains(&user_id.to_string()) {
            return Err("User is not a participant in this workspace".into());
        }

        // Check team permissions if workspace belongs to a team
        if let Some(ref team_id) = workspace.team_id {
            let can_access = self.team_service.can_user_access_project(
                user_id,
                &format!("workspace_{}", team_id),
                required_permission,
            ).await.unwrap_or(false);

            if !can_access {
                return Err("Insufficient team permissions for workspace access".into());
            }
        }

        Ok(())
    }

    async fn apply_operation_to_workspace(
        &self,
        workspace_id: &str,
        operation: &EditorOperation,
        user_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces.get_mut(workspace_id)
            .ok_or_else(|| format!("Workspace {} not found", workspace_id))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        match operation {
            EditorOperation::AddEntry { path, entry_type, .. } => {
                if validate_secure_path(path).is_ok() {
                    // Convert string entry_type to EntryType enum
                    let entry_type_enum = match entry_type.as_str() {
                        "file" => EntryType::File,
                        "directory" => EntryType::Directory,
                        _ => EntryType::File, // Default to File for unrecognized types
                    };

                    workspace.entries.insert(path.clone(), WorkspaceEntry {
                        path: path.clone(),
                        entry_type: entry_type_enum,
                        metadata: HashMap::new(),
                        created_at: timestamp,
                        modified_at: timestamp,
                    });
                }
            }
            EditorOperation::RemoveEntry { path, .. } => {
                workspace.entries.remove(path);
            }
            EditorOperation::RenameEntry { old_path, new_path, .. } => {
                if let Some(entry) = workspace.entries.remove(old_path) {
                    let mut new_entry = entry;
                    new_entry.path = new_path.clone();
                    new_entry.modified_at = timestamp;
                    workspace.entries.insert(new_path.clone(), new_entry);
                }
            }
            EditorOperation::MoveEntry { from_path, to_path, .. } => {
                if let Some(entry) = workspace.entries.remove(from_path) {
                    let mut new_entry = entry;
                    new_entry.path = to_path.clone();
                    new_entry.modified_at = timestamp;
                    workspace.entries.insert(to_path.clone(), new_entry);
                }
            }
            // Text operations don't affect workspace state
            EditorOperation::Insert { .. } | EditorOperation::Delete { .. } | EditorOperation::Update { .. } => {}
        }

        workspace.version += 1;
        Ok(())
    }

    async fn transform_workspace_operation(
        &self,
        workspace_id: &str,
        operation: EditorOperation,
    ) -> Result<EditorOperation, Box<dyn std::error::Error>> {
        // Get recent operations for transformation context
        let crdts = self.workspace_crdt.read().await;
        let crdt = crdts.get(workspace_id)
            .ok_or_else(|| format!("Workspace CRDT {} not found", workspace_id))?;

        // Retrieve recent operations (last 50 for performance)
        let recent_operations: Vec<&WorkspaceOperation> = crdt.operations.iter().rev().take(50).collect();

        // Apply rename/move conflict rules and path remapping
        let mut transformed_operation = operation;

        for recent_op in recent_operations {
            match recent_op {
                WorkspaceOperation::RenameEntry { old_path, new_path, .. } => {
                    // Check if the operation affects paths under the renamed directory
                    transformed_operation = self.remap_operation_paths(
                        transformed_operation,
                        old_path,
                        new_path,
                    );
                }
                WorkspaceOperation::MoveEntry { from_path, to_path, .. } => {
                    // Check if the operation affects paths under the moved directory
                    transformed_operation = self.remap_operation_paths(
                        transformed_operation,
                        from_path,
                        to_path,
                    );
                }
                _ => {}
            }
        }

        // Use operational_transformer if available
        // Note: The OperationalTransformer is instantiated but may not have specific
        // workspace operation transformation methods. For workspace-specific OT,
        // we're handling path remapping manually above. If OperationalTransformer
        // supports generic operation transformation, it could be used here.
        // Currently, it's available but not utilized for workspace operations
        // as they require domain-specific conflict resolution (path remapping).

        Ok(transformed_operation)
    }

    /// Helper method to remap operation paths when parent directories are renamed/moved
    fn remap_operation_paths(
        &self,
        operation: EditorOperation,
        old_parent_path: &str,
        new_parent_path: &str,
    ) -> EditorOperation {
        match operation {
            EditorOperation::AddEntry { path, entry_type, lamport_clock, site_id, op_id, clock } => {
                let remapped_path = self.remap_path(&path, old_parent_path, new_parent_path);
                EditorOperation::AddEntry {
                    path: remapped_path,
                    entry_type,
                    lamport_clock,
                    site_id,
                    op_id,
                    clock,
                }
            }
            EditorOperation::RemoveEntry { path, lamport_clock, site_id, op_id, clock } => {
                let remapped_path = self.remap_path(&path, old_parent_path, new_parent_path);
                EditorOperation::RemoveEntry {
                    path: remapped_path,
                    lamport_clock,
                    site_id,
                    op_id,
                    clock,
                }
            }
            EditorOperation::RenameEntry { old_path, new_path, lamport_clock, site_id, op_id, clock } => {
                let remapped_old = self.remap_path(&old_path, old_parent_path, new_parent_path);
                let remapped_new = self.remap_path(&new_path, old_parent_path, new_parent_path);
                EditorOperation::RenameEntry {
                    old_path: remapped_old,
                    new_path: remapped_new,
                    lamport_clock,
                    site_id,
                    op_id,
                    clock,
                }
            }
            EditorOperation::MoveEntry { from_path, to_path, lamport_clock, site_id, op_id, clock } => {
                let remapped_from = self.remap_path(&from_path, old_parent_path, new_parent_path);
                let remapped_to = self.remap_path(&to_path, old_parent_path, new_parent_path);
                EditorOperation::MoveEntry {
                    from_path: remapped_from,
                    to_path: remapped_to,
                    lamport_clock,
                    site_id,
                    op_id,
                    clock,
                }
            }
            // Text operations don't need path remapping
            other => other,
        }
    }

    /// Helper method to remap a single path if it's under the old parent path
    fn remap_path(&self, path: &str, old_parent: &str, new_parent: &str) -> String {
        if path.starts_with(&format!("{}/", old_parent)) || path == old_parent {
            if path == old_parent {
                new_parent.to_string()
            } else {
                format!("{}/{}", new_parent, &path[(old_parent.len() + 1)..])
            }
        } else {
            path.to_string()
        }
    }

    /// Helper method to update operation with current CRDT clock values
    fn update_operation_clock(&self, operation: &EditorOperation, lamport_clock: u64, site_id: u32) -> EditorOperation {
        match operation {
            EditorOperation::AddEntry { path, entry_type, op_id, .. } => {
                EditorOperation::AddEntry {
                    path: path.clone(),
                    entry_type: entry_type.clone(),
                    lamport_clock,
                    site_id,
                    op_id: op_id.clone(),
                    clock: operation.clock().clone(),
                }
            }
            EditorOperation::RemoveEntry { path, op_id, .. } => {
                EditorOperation::RemoveEntry {
                    path: path.clone(),
                    lamport_clock,
                    site_id,
                    op_id: op_id.clone(),
                    clock: operation.clock().clone(),
                }
            }
            EditorOperation::RenameEntry { old_path, new_path, op_id, .. } => {
                EditorOperation::RenameEntry {
                    old_path: old_path.clone(),
                    new_path: new_path.clone(),
                    lamport_clock,
                    site_id,
                    op_id: op_id.clone(),
                    clock: operation.clock().clone(),
                }
            }
            EditorOperation::MoveEntry { from_path, to_path, op_id, .. } => {
                EditorOperation::MoveEntry {
                    from_path: from_path.clone(),
                    to_path: to_path.clone(),
                    lamport_clock,
                    site_id,
                    op_id: op_id.clone(),
                    clock: operation.clock().clone(),
                }
            }
            // For text operations, return as-is
            other => other.clone(),
        }
    }

    async fn broadcast_workspace_event(
        &self,
        workspace_id: &str,
        operation: &EditorOperation,
        user_id: &str,
    ) {
        let event = match operation {
            EditorOperation::AddEntry { path, entry_type, .. } => {
                // Convert string entry_type to EntryType enum
                let entry_type_enum = match entry_type.as_str() {
                    "file" => EntryType::File,
                    "directory" => EntryType::Directory,
                    _ => EntryType::File, // Default to File for unrecognized types
                };

                WorkspaceEvent::EntryAdded {
                    workspace_id: workspace_id.to_string(),
                    path: path.clone(),
                    entry_type: entry_type_enum,
                    user_id: user_id.to_string(),
                }
            }
            EditorOperation::RemoveEntry { path, .. } => {
                WorkspaceEvent::EntryRemoved {
                    workspace_id: workspace_id.to_string(),
                    path: path.clone(),
                    user_id: user_id.to_string(),
                }
            }
            EditorOperation::RenameEntry { old_path, new_path, .. } => {
                WorkspaceEvent::EntryRenamed {
                    workspace_id: workspace_id.to_string(),
                    old_path: old_path.clone(),
                    new_path: new_path.clone(),
                    user_id: user_id.to_string(),
                }
            }
            EditorOperation::MoveEntry { from_path, to_path, .. } => {
                WorkspaceEvent::EntryMoved {
                    workspace_id: workspace_id.to_string(),
                    from_path: from_path.clone(),
                    to_path: to_path.clone(),
                    user_id: user_id.to_string(),
                }
            }
            // Text operations don't generate workspace events
            EditorOperation::Insert { .. } | EditorOperation::Delete { .. } | EditorOperation::Update { .. } => return,
        };

        let _ = self.event_broadcaster.send(event);
    }
}

impl WorkspaceCRDT {
    /// Apply operation to CRDT state
    pub fn apply_operation(&mut self, operation: EditorOperation) -> Result<(), Box<dyn std::error::Error>> {
        match &operation {
            EditorOperation::AddEntry { path, entry_type, .. } => {
                // Ensure tombstone has space
                while self.tombstone.len() <= self.operations.len() {
                    self.tombstone.push(None);
                }

                // Convert string entry_type to EntryType enum
                let entry_type_enum = match entry_type.as_str() {
                    "file" => EntryType::File,
                    "directory" => EntryType::Directory,
                    _ => EntryType::File, // Default to File for unrecognized types
                };

                // Add entry marker with proper EntryType
                self.tombstone.push(Some(WorkspaceEntry {
                    path: path.clone(),
                    entry_type: entry_type_enum,
                    metadata: HashMap::new(),
                    created_at: 0,
                    modified_at: 0,
                }));
            }
            EditorOperation::RemoveEntry { .. } => {
                // Mark as deleted in tombstone
                if self.tombstone.len() > self.operations.len() {
                    self.tombstone[self.operations.len()] = None;
                }
            }
            EditorOperation::RenameEntry { .. } | EditorOperation::MoveEntry { .. } => {
                // These operations modify existing entries
                // Implementation would update the corresponding tombstone entry
            }
            // Text operations don't affect workspace CRDT
            EditorOperation::Insert { .. } | EditorOperation::Delete { .. } | EditorOperation::Update { .. } => {
                // No-op for workspace CRDT
            }
        }

        Ok(())
    }
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            allow_public_access: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_file_types: vec![
                "rs".to_string(),
                "toml".to_string(),
                "md".to_string(),
                "txt".to_string(),
                "json".to_string(),
            ],
            auto_sync_enabled: true,
        }
    }
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::File => write!(f, "File"),
            EntryType::Directory => write!(f, "Directory"),
        }
    }
}

impl std::fmt::Display for EditorOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorOperation::AddEntry { path, .. } => write!(f, "Add entry: {}", path),
            EditorOperation::RemoveEntry { path, .. } => write!(f, "Remove entry: {}", path),
            EditorOperation::RenameEntry { old_path, new_path, .. } => {
                write!(f, "Rename entry: {} -> {}", old_path, new_path)
            }
            EditorOperation::MoveEntry { from_path, to_path, .. } => {
                write!(f, "Move entry: {} -> {}", from_path, to_path)
            }
            EditorOperation::Insert { position, content, .. } => {
                write!(f, "Insert at {}: {}", position, content)
            }
            EditorOperation::Delete { position, length, .. } => {
                write!(f, "Delete at {} (length: {})", position, length)
            }
            EditorOperation::Update { position, old_content, new_content, .. } => {
                write!(f, "Update at {}: '{}' -> '{}'", position, old_content, new_content)
            }
        }
    }
}