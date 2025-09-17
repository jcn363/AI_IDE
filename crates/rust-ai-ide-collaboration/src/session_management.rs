//! Enhanced user session management for collaboration system.
//!
//! Provides comprehensive session management including authentication, permissions,
//! user roles, and activity tracking for collaborative editing sessions.

use std::collections::HashMap;
use std::sync::Arc;

use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_security::audit_logger;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::*;

/// Enhanced collaboration service with comprehensive session management
pub struct EnhancedCollaborationService {
    pub collaboration_service: CollaborationService,
    pub session_manager: SessionManager,
    pub permission_manager: PermissionManager,
    pub activity_tracker: ActivityTracker,
}

/// Session manager for user authentication and session lifecycle
pub struct SessionManager {
    active_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    session_tokens: Arc<RwLock<HashMap<String, String>>>, // token -> session_id
}

/// User session with enhanced tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
    pub created_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

/// User roles in the collaboration system
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Owner,
    Admin,
    Editor,
    Viewer,
    Guest,
}

/// Permission types for collaborative actions
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    ReadDocument,
    EditDocument,
    DeleteDocument,
    InviteUsers,
    ManagePermissions,
    ViewAnalytics,
    ExportData,
    CreateSession,
    EndSession,
    ManageTeam,
}

/// Permission manager for access control
pub struct PermissionManager {
    role_permissions: HashMap<UserRole, Vec<Permission>>,
}

/// Activity tracker for session analytics
pub struct ActivityTracker {
    user_activities: Arc<RwLock<HashMap<String, Vec<UserActivity>>>>,
    session_activities: Arc<RwLock<HashMap<String, Vec<SessionActivity>>>>,
}

/// User activity record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserActivity {
    pub activity_type: ActivityType,
    pub timestamp: std::time::SystemTime,
    pub details: String,
    pub metadata: HashMap<String, String>,
}

/// Session activity record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionActivity {
    pub user_id: String,
    pub activity_type: ActivityType,
    pub timestamp: std::time::SystemTime,
    pub document_id: Option<String>,
    pub details: String,
}

/// Activity types for tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActivityType {
    SessionJoined,
    SessionLeft,
    DocumentEdited,
    PermissionChanged,
    UserInvited,
    DocumentShared,
    CoachingRequested,
    SuggestionApplied,
}

impl EnhancedCollaborationService {
    pub fn new() -> Self {
        Self {
            collaboration_service: CollaborationService::new(),
            session_manager: SessionManager::new(),
            permission_manager: PermissionManager::new(),
            activity_tracker: ActivityTracker::new(),
        }
    }

    /// Create authenticated session with user validation
    pub async fn create_authenticated_session(
        &self,
        user_id: String,
        username: String,
        document_id: String,
        user_role: UserRole,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Validate user permissions
        let permissions = self.permission_manager.get_role_permissions(&user_role);
        if !permissions.contains(&Permission::CreateSession) {
            return Err("Insufficient permissions to create session".into());
        }

        // Create session
        let session_id = format!("session_{}", uuid::Uuid::new_v4());
        let session = UserSession {
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            username: username.clone(),
            role: user_role,
            permissions: permissions.clone(),
            created_at: std::time::SystemTime::now(),
            last_activity: std::time::SystemTime::now(),
            ip_address: None,
            user_agent: None,
            is_active: true,
        };

        // Store session
        self.session_manager.store_session(session).await?;

        // Initialize collaboration session
        self.collaboration_service
            .create_session(session_id.clone(), document_id)
            .await?;

        // Log activity
        self.activity_tracker
            .track_user_activity(
                &user_id,
                ActivityType::SessionJoined,
                format!("Joined session {}", session_id),
                HashMap::from([
                    ("session_id".to_string(), session_id.clone()),
                    ("role".to_string(), format!("{:?}", user_role)),
                ]),
            )
            .await;

        audit_logger::log_event(
            "session_created",
            &format!(
                "User {} created session {} with role {:?}",
                username, session_id, user_role
            ),
        );

        Ok(session_id)
    }

    /// Validate session and user permissions
    pub async fn validate_session_and_permissions(
        &self,
        session_id: &str,
        user_id: &str,
        required_permission: &Permission,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let session = self
            .session_manager
            .get_session(session_id)
            .await?
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        if session.user_id != user_id {
            return Err("User not authorized for this session".into());
        }

        if !session.is_active {
            return Err("Session is not active".into());
        }

        if !session.permissions.contains(required_permission) {
            return Err(format!(
                "User lacks required permission: {:?}",
                required_permission
            ));
        }

        // Update last activity
        self.session_manager.update_activity(session_id).await?;

        Ok(())
    }

    /// End session with cleanup
    pub async fn end_session_with_cleanup(
        &self,
        session_id: &str,
        user_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate permissions
        self.validate_session_and_permissions(session_id, user_id, &Permission::EndSession)
            .await?;

        let session = self
            .session_manager
            .get_session(session_id)
            .await?
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        // End collaboration session
        self.collaboration_service.end_session(session_id)?;

        // Mark user session as inactive
        self.session_manager.deactivate_session(session_id).await?;

        // Log activity
        self.activity_tracker
            .track_user_activity(
                user_id,
                ActivityType::SessionLeft,
                format!("Left session {}", session_id),
                HashMap::from([("session_id".to_string(), session_id.to_string())]),
            )
            .await;

        audit_logger::log_event(
            "session_ended",
            &format!("User {} ended session {}", session.username, session_id),
        );

        Ok(())
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            session_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store_session(
        &self,
        session: UserSession,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session.session_id.clone(), session);
        Ok(())
    }

    pub async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Option<UserSession>, Box<dyn std::error::Error>> {
        let sessions = self.active_sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn update_activity(
        &self,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = std::time::SystemTime::now();
        }
        Ok(())
    }

    pub async fn deactivate_session(
        &self,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.is_active = false;
        }
        Ok(())
    }

    pub async fn get_active_sessions(
        &self,
    ) -> Result<Vec<UserSession>, Box<dyn std::error::Error>> {
        let sessions = self.active_sessions.read().await;
        Ok(sessions.values().filter(|s| s.is_active).cloned().collect())
    }
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();

        // Define permissions for each role
        role_permissions.insert(
            UserRole::Owner,
            vec![
                Permission::ReadDocument,
                Permission::EditDocument,
                Permission::DeleteDocument,
                Permission::InviteUsers,
                Permission::ManagePermissions,
                Permission::ViewAnalytics,
                Permission::ExportData,
                Permission::CreateSession,
                Permission::EndSession,
                Permission::ManageTeam,
            ],
        );

        role_permissions.insert(
            UserRole::Admin,
            vec![
                Permission::ReadDocument,
                Permission::EditDocument,
                Permission::DeleteDocument,
                Permission::InviteUsers,
                Permission::ManagePermissions,
                Permission::ViewAnalytics,
                Permission::CreateSession,
                Permission::EndSession,
                Permission::ManageTeam,
            ],
        );

        role_permissions.insert(
            UserRole::Editor,
            vec![
                Permission::ReadDocument,
                Permission::EditDocument,
                Permission::CreateSession,
                Permission::EndSession,
            ],
        );

        role_permissions.insert(
            UserRole::Viewer,
            vec![Permission::ReadDocument, Permission::ViewAnalytics],
        );

        role_permissions.insert(UserRole::Guest, vec![Permission::ReadDocument]);

        Self { role_permissions }
    }

    pub fn get_role_permissions(&self, role: &UserRole) -> Vec<Permission> {
        self.role_permissions.get(role).cloned().unwrap_or_default()
    }

    pub fn has_permission(&self, role: &UserRole, permission: &Permission) -> bool {
        self.role_permissions
            .get(role)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }
}

impl ActivityTracker {
    pub fn new() -> Self {
        Self {
            user_activities: Arc::new(RwLock::new(HashMap::new())),
            session_activities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn track_user_activity(
        &self,
        user_id: &str,
        activity_type: ActivityType,
        details: String,
        metadata: HashMap<String, String>,
    ) {
        let activity = UserActivity {
            activity_type,
            timestamp: std::time::SystemTime::now(),
            details,
            metadata,
        };

        let mut activities = self.user_activities.write().await;
        activities
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(activity);
    }

    pub async fn track_session_activity(
        &self,
        session_id: &str,
        user_id: &str,
        activity_type: ActivityType,
        document_id: Option<String>,
        details: String,
    ) {
        let activity = SessionActivity {
            user_id: user_id.to_string(),
            activity_type,
            timestamp: std::time::SystemTime::now(),
            document_id,
            details,
        };

        let mut activities = self.session_activities.write().await;
        activities
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(activity);
    }

    pub async fn get_user_activities(&self, user_id: &str) -> Vec<UserActivity> {
        let activities = self.user_activities.read().await;
        activities.get(user_id).cloned().unwrap_or_default()
    }

    pub async fn get_session_activities(&self, session_id: &str) -> Vec<SessionActivity> {
        let activities = self.session_activities.read().await;
        activities.get(session_id).cloned().unwrap_or_default()
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Viewer
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Owner => write!(f, "Owner"),
            UserRole::Admin => write!(f, "Admin"),
            UserRole::Editor => write!(f, "Editor"),
            UserRole::Viewer => write!(f, "Viewer"),
            UserRole::Guest => write!(f, "Guest"),
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::ReadDocument => write!(f, "Read Document"),
            Permission::EditDocument => write!(f, "Edit Document"),
            Permission::DeleteDocument => write!(f, "Delete Document"),
            Permission::InviteUsers => write!(f, "Invite Users"),
            Permission::ManagePermissions => write!(f, "Manage Permissions"),
            Permission::ViewAnalytics => write!(f, "View Analytics"),
            Permission::ExportData => write!(f, "Export Data"),
            Permission::CreateSession => write!(f, "Create Session"),
            Permission::EndSession => write!(f, "End Session"),
            Permission::ManageTeam => write!(f, "Manage Team"),
        }
    }
}
