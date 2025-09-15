//! Real-time presence and collaboration features.
//!
//! Provides user presence tracking, cursor positions, active user monitoring,
//! and real-time collaboration indicators.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::*;

/// Presence service for tracking user activity and collaboration state
pub struct PresenceService {
    user_presence: Arc<RwLock<HashMap<String, UserPresence>>>,
    document_presence: Arc<RwLock<HashMap<String, Vec<String>>>>, // document_id -> [user_ids]
    cursor_positions: Arc<RwLock<HashMap<String, CursorPosition>>>,
}

/// User presence information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: String,
    pub username: String,
    pub status: PresenceStatus,
    pub last_seen: std::time::SystemTime,
    pub current_document: Option<String>,
    pub current_session: Option<String>,
    pub activity_type: ActivityType,
}

/// Presence status types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

/// Cursor position tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorPosition {
    pub user_id: String,
    pub document_id: String,
    pub line: usize,
    pub column: usize,
    pub selection_start: Option<(usize, usize)>, // line, column
    pub selection_end: Option<(usize, usize)>,   // line, column
    pub last_updated: std::time::SystemTime,
}

/// Activity feed entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityFeedEntry {
    pub user_id: String,
    pub username: String,
    pub activity_type: ActivityType,
    pub description: String,
    pub timestamp: std::time::SystemTime,
    pub document_id: Option<String>,
    pub session_id: Option<String>,
}

/// Collaboration indicators
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollaborationIndicators {
    pub active_users: Vec<String>,
    pub editing_users: Vec<String>,
    pub viewing_users: Vec<String>,
    pub recent_activities: Vec<ActivityFeedEntry>,
}

impl PresenceService {
    pub fn new() -> Self {
        Self {
            user_presence: Arc::new(RwLock::new(HashMap::new())),
            document_presence: Arc::new(RwLock::new(HashMap::new())),
            cursor_positions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update user presence
    pub async fn update_presence(
        &self,
        user_id: String,
        username: String,
        status: PresenceStatus,
        document_id: Option<String>,
        session_id: Option<String>,
        activity_type: ActivityType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let presence = UserPresence {
            user_id: user_id.clone(),
            username: username.clone(),
            status: status.clone(),
            last_seen: std::time::SystemTime::now(),
            current_document: document_id.clone(),
            current_session: session_id.clone(),
            activity_type: activity_type.clone(),
        };

        // Update user presence
        {
            let mut user_presence = self.user_presence.write().await;
            user_presence.insert(user_id.clone(), presence);
        }

        // Update document presence
        if let Some(doc_id) = &document_id {
            let mut document_presence = self.document_presence.write().await;
            document_presence
                .entry(doc_id.clone())
                .or_insert_with(Vec::new)
                .push(user_id.clone());
        }

        // Broadcast presence update via event bus
        self.broadcast_presence_update(&user_id, &status, &activity_type)
            .await;

        Ok(())
    }

    /// Update cursor position
    pub async fn update_cursor_position(
        &self,
        user_id: String,
        document_id: String,
        line: usize,
        column: usize,
        selection_start: Option<(usize, usize)>,
        selection_end: Option<(usize, usize)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cursor_pos = CursorPosition {
            user_id: user_id.clone(),
            document_id: document_id.clone(),
            line,
            column,
            selection_start,
            selection_end,
            last_updated: std::time::SystemTime::now(),
        };

        let mut cursor_positions = self.cursor_positions.write().await;
        cursor_positions.insert(user_id.clone(), cursor_pos);

        // Broadcast cursor update
        self.broadcast_cursor_update(&user_id, &document_id, line, column)
            .await;

        Ok(())
    }

    /// Get active users for a document
    pub async fn get_document_users(
        &self,
        document_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let document_presence = self.document_presence.read().await;
        Ok(document_presence
            .get(document_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Get collaboration indicators for a session
    pub async fn get_collaboration_indicators(
        &self,
        session_id: &str,
        document_id: &str,
    ) -> Result<CollaborationIndicators, Box<dyn std::error::Error>> {
        let user_presence = self.user_presence.read().await;
        let document_presence = self.document_presence.read().await;

        let active_users = document_presence
            .get(document_id)
            .cloned()
            .unwrap_or_default();

        let mut editing_users = Vec::new();
        let mut viewing_users = Vec::new();

        for user_id in &active_users {
            if let Some(presence) = user_presence.get(user_id) {
                match presence.activity_type {
                    ActivityType::DocumentEdited => editing_users.push(user_id.clone()),
                    _ => viewing_users.push(user_id.clone()),
                }
            }
        }

        // Get recent activities (simplified - would be from activity tracker)
        let recent_activities = Vec::new(); // Would populate from ActivityTracker

        Ok(CollaborationIndicators {
            active_users,
            editing_users,
            viewing_users,
            recent_activities,
        })
    }

    /// Get user presence
    pub async fn get_user_presence(
        &self,
        user_id: &str,
    ) -> Result<Option<UserPresence>, Box<dyn std::error::Error>> {
        let user_presence = self.user_presence.read().await;
        Ok(user_presence.get(user_id).cloned())
    }

    /// Get all online users
    pub async fn get_online_users(&self) -> Result<Vec<UserPresence>, Box<dyn std::error::Error>> {
        let user_presence = self.user_presence.read().await;
        Ok(user_presence
            .values()
            .filter(|p| p.status == PresenceStatus::Online)
            .cloned()
            .collect())
    }

    /// Remove user from presence (when they disconnect)
    pub async fn remove_user_presence(
        &self,
        user_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Update status to offline
        {
            let mut user_presence = self.user_presence.write().await;
            if let Some(presence) = user_presence.get_mut(user_id) {
                presence.status = PresenceStatus::Offline;
                presence.last_seen = std::time::SystemTime::now();
            }
        }

        // Remove from document presence
        {
            let mut document_presence = self.document_presence.write().await;
            for users in document_presence.values_mut() {
                users.retain(|u| u != user_id);
            }
        }

        // Remove cursor position
        {
            let mut cursor_positions = self.cursor_positions.write().await;
            cursor_positions.remove(user_id);
        }

        Ok(())
    }

    /// Get cursor positions for a document
    pub async fn get_document_cursors(
        &self,
        document_id: &str,
    ) -> Result<Vec<CursorPosition>, Box<dyn std::error::Error>> {
        let cursor_positions = self.cursor_positions.read().await;
        Ok(cursor_positions
            .values()
            .filter(|c| c.document_id == document_id)
            .cloned()
            .collect())
    }

    // Internal methods for broadcasting updates
    async fn broadcast_presence_update(
        &self,
        user_id: &str,
        status: &PresenceStatus,
        activity: &ActivityType,
    ) {
        // This would integrate with the event bus system
        log::debug!(
            "Presence update for user {}: {:?} - {:?}",
            user_id,
            status,
            activity
        );
        // EventBus::publish(Event::PresenceUpdate { user_id, status, activity });
    }

    async fn broadcast_cursor_update(
        &self,
        user_id: &str,
        document_id: &str,
        line: usize,
        column: usize,
    ) {
        // This would integrate with the event bus system
        log::debug!(
            "Cursor update for user {} in document {}: {}:{}",
            user_id,
            document_id,
            line,
            column
        );
        // EventBus::publish(Event::CursorUpdate { user_id, document_id, line, column });
    }
}

impl Default for PresenceStatus {
    fn default() -> Self {
        PresenceStatus::Offline
    }
}

impl std::fmt::Display for PresenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresenceStatus::Online => write!(f, "Online"),
            PresenceStatus::Away => write!(f, "Away"),
            PresenceStatus::Busy => write!(f, "Busy"),
            PresenceStatus::Offline => write!(f, "Offline"),
        }
    }
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::SessionJoined => write!(f, "Joined Session"),
            ActivityType::SessionLeft => write!(f, "Left Session"),
            ActivityType::DocumentEdited => write!(f, "Editing Document"),
            ActivityType::PermissionChanged => write!(f, "Changed Permissions"),
            ActivityType::UserInvited => write!(f, "Invited User"),
            ActivityType::DocumentShared => write!(f, "Shared Document"),
            ActivityType::CoachingRequested => write!(f, "Requested Coaching"),
            ActivityType::SuggestionApplied => write!(f, "Applied Suggestion"),
            ActivityType::CodeAnalyzed => write!(f, "Analyzed Code"),
        }
    }
}
