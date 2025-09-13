//! Team management and project sharing features.
//!
//! Provides team creation, user invitations, project sharing, permissions management,
//! and collaborative workspace organization.

use std::collections::HashMap;
use std::sync::Arc;

use rust_ai_ide_common::validation::validate_secure_path;
use rust_ai_ide_security::audit_logger;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::*;

/// Team management service
pub struct TeamManagementService {
    teams:            Arc<RwLock<HashMap<String, Team>>>,
    team_memberships: Arc<RwLock<HashMap<String, Vec<String>>>>, // user_id -> team_ids
    invitations:      Arc<RwLock<HashMap<String, TeamInvitation>>>,
    project_sharing:  Arc<RwLock<HashMap<String, ProjectShare>>>,
}

/// Team information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub id:            String,
    pub name:          String,
    pub description:   String,
    pub owner_id:      String,
    pub created_at:    std::time::SystemTime,
    pub settings:      TeamSettings,
    pub member_count:  usize,
    pub project_count: usize,
}

/// Team settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamSettings {
    pub allow_public_projects:      bool,
    pub require_approval_for_joins: bool,
    pub default_member_role:        UserRole,
    pub max_members:                Option<usize>,
    pub max_projects:               Option<usize>,
}

/// Team invitation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamInvitation {
    pub invitation_id:      String,
    pub team_id:            String,
    pub invited_by:         String,
    pub invited_user_email: String,
    pub invited_user_id:    Option<String>,
    pub role:               UserRole,
    pub status:             InvitationStatus,
    pub created_at:         std::time::SystemTime,
    pub expires_at:         std::time::SystemTime,
    pub message:            Option<String>,
}

/// Invitation status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Cancelled,
}

/// Project sharing configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectShare {
    pub project_id:  String,
    pub shared_by:   String,
    pub team_id:     String,
    pub permissions: Vec<Permission>,
    pub share_type:  ShareType,
    pub created_at:  std::time::SystemTime,
    pub expires_at:  Option<std::time::SystemTime>,
    pub is_active:   bool,
}

/// Share types for projects
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareType {
    ReadOnly,
    ReadWrite,
    Admin,
    Custom,
}

/// Team member with role
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id:            String,
    pub username:           String,
    pub email:              String,
    pub role:               UserRole,
    pub joined_at:          std::time::SystemTime,
    pub last_active:        std::time::SystemTime,
    pub contribution_count: usize,
}

/// Project access log
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectAccessLog {
    pub user_id:    String,
    pub project_id: String,
    pub team_id:    String,
    pub action:     AccessAction,
    pub timestamp:  std::time::SystemTime,
    pub details:    String,
}

/// Access action types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AccessAction {
    Viewed,
    Edited,
    Shared,
    Downloaded,
    Deleted,
    PermissionChanged,
}

impl TeamManagementService {
    pub fn new() -> Self {
        Self {
            teams:            Arc::new(RwLock::new(HashMap::new())),
            team_memberships: Arc::new(RwLock::new(HashMap::new())),
            invitations:      Arc::new(RwLock::new(HashMap::new())),
            project_sharing:  Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new team
    pub async fn create_team(
        &self,
        name: String,
        description: String,
        owner_id: String,
        settings: Option<TeamSettings>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let team_id = format!("team_{}", uuid::Uuid::new_v4());
        let team = Team {
            id: team_id.clone(),
            name: name.clone(),
            description,
            owner_id: owner_id.clone(),
            created_at: std::time::SystemTime::now(),
            settings: settings.unwrap_or_default(),
            member_count: 1, // Owner is first member
            project_count: 0,
        };

        // Store team
        {
            let mut teams = self.teams.write().await;
            teams.insert(team_id.clone(), team);
        }

        // Add owner as first member
        {
            let mut memberships = self.team_memberships.write().await;
            memberships
                .entry(owner_id.clone())
                .or_insert_with(Vec::new)
                .push(team_id.clone());
        }

        audit_logger::log_event(
            "team_created",
            &format!("Team '{}' created by user {}", name, owner_id),
        );

        Ok(team_id)
    }

    /// Invite user to team
    pub async fn invite_user_to_team(
        &self,
        team_id: String,
        invited_by: String,
        invited_user_email: String,
        role: UserRole,
        message: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Validate that inviter has permission
        let teams = self.teams.read().await;
        let team = teams
            .get(&team_id)
            .ok_or_else(|| format!("Team {} not found", team_id))?;

        if team.owner_id != invited_by {
            // Check if inviter has invite permission
            // This would integrate with permission system
            return Err("Insufficient permissions to invite users".into());
        }

        let invitation_id = format!("inv_{}", uuid::Uuid::new_v4());
        let invitation = TeamInvitation {
            invitation_id: invitation_id.clone(),
            team_id: team_id.clone(),
            invited_by: invited_by.clone(),
            invited_user_email: invited_user_email.clone(),
            invited_user_id: None, // Would be populated if user exists
            role,
            status: InvitationStatus::Pending,
            created_at: std::time::SystemTime::now(),
            expires_at: std::time::SystemTime::now() + std::time::Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            message,
        };

        let mut invitations = self.invitations.write().await;
        invitations.insert(invitation_id.clone(), invitation);

        audit_logger::log_event(
            "team_invitation_sent",
            &format!(
                "User {} invited {} to team {}",
                invited_by, invited_user_email, team_id
            ),
        );

        Ok(invitation_id)
    }

    /// Accept team invitation
    pub async fn accept_invitation(
        &self,
        invitation_id: String,
        user_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut invitations = self.invitations.write().await;
        let invitation = invitations
            .get_mut(&invitation_id)
            .ok_or_else(|| format!("Invitation {} not found", invitation_id))?;

        if invitation.status != InvitationStatus::Pending {
            return Err("Invitation is not pending".into());
        }

        // Check if invitation has expired
        if std::time::SystemTime::now() > invitation.expires_at {
            invitation.status = InvitationStatus::Expired;
            return Err("Invitation has expired".into());
        }

        invitation.status = InvitationStatus::Accepted;
        invitation.invited_user_id = Some(user_id.clone());

        // Add user to team
        {
            let mut memberships = self.team_memberships.write().await;
            memberships
                .entry(user_id.clone())
                .or_insert_with(Vec::new)
                .push(invitation.team_id.clone());
        }

        // Update team member count
        {
            let mut teams = self.teams.write().await;
            if let Some(team) = teams.get_mut(&invitation.team_id) {
                team.member_count += 1;
            }
        }

        audit_logger::log_event(
            "team_invitation_accepted",
            &format!(
                "User {} accepted invitation to team {}",
                user_id, invitation.team_id
            ),
        );

        Ok(())
    }

    /// Share project with team
    pub async fn share_project_with_team(
        &self,
        project_id: String,
        shared_by: String,
        team_id: String,
        permissions: Vec<Permission>,
        share_type: ShareType,
        expires_at: Option<std::time::SystemTime>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate permissions
        let teams = self.teams.read().await;
        let team = teams
            .get(&team_id)
            .ok_or_else(|| format!("Team {} not found", team_id))?;

        // Check if user is team member or owner
        if team.owner_id != shared_by {
            let memberships = self.team_memberships.read().await;
            let user_teams = memberships.get(&shared_by).unwrap_or(&Vec::new());
            if !user_teams.contains(&team_id) {
                return Err("User is not a member of this team".into());
            }
        }

        let share = ProjectShare {
            project_id: project_id.clone(),
            shared_by: shared_by.clone(),
            team_id: team_id.clone(),
            permissions,
            share_type,
            created_at: std::time::SystemTime::now(),
            expires_at,
            is_active: true,
        };

        let mut project_sharing = self.project_sharing.write().await;
        project_sharing.insert(project_id.clone(), share);

        // Update team project count
        {
            let mut teams = self.teams.write().await;
            if let Some(team) = teams.get_mut(&team_id) {
                team.project_count += 1;
            }
        }

        audit_logger::log_event(
            "project_shared",
            &format!(
                "Project {} shared with team {} by user {}",
                project_id, team_id, shared_by
            ),
        );

        Ok(())
    }

    /// Get team members
    pub async fn get_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>, Box<dyn std::error::Error>> {
        let teams = self.teams.read().await;
        let team = teams
            .get(team_id)
            .ok_or_else(|| format!("Team {} not found", team_id))?;

        // This would normally query a database, but for now return mock data
        let mut members = Vec::new();

        // Add owner
        members.push(TeamMember {
            user_id:            team.owner_id.clone(),
            username:           format!("owner_{}", team.owner_id),
            email:              format!("owner@team.com"),
            role:               UserRole::Owner,
            joined_at:          team.created_at,
            last_active:        std::time::SystemTime::now(),
            contribution_count: 0,
        });

        // Add other members (would be from database)
        for i in 1..team.member_count {
            members.push(TeamMember {
                user_id:            format!("member_{}", i),
                username:           format!("member_{}", i),
                email:              format!("member{}@team.com", i),
                role:               UserRole::Editor,
                joined_at:          std::time::SystemTime::now(),
                last_active:        std::time::SystemTime::now(),
                contribution_count: 0,
            });
        }

        Ok(members)
    }

    /// Get user's teams
    pub async fn get_user_teams(&self, user_id: &str) -> Result<Vec<Team>, Box<dyn std::error::Error>> {
        let memberships = self.team_memberships.read().await;
        let user_team_ids = memberships.get(user_id).cloned().unwrap_or_default();

        let teams = self.teams.read().await;
        let mut user_teams = Vec::new();

        for team_id in user_team_ids {
            if let Some(team) = teams.get(&team_id) {
                user_teams.push(team.clone());
            }
        }

        Ok(user_teams)
    }

    /// Check if user can access project
    pub async fn can_user_access_project(
        &self,
        user_id: &str,
        project_id: &str,
        required_permission: &Permission,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let project_sharing = self.project_sharing.read().await;

        if let Some(share) = project_sharing.get(project_id) {
            // Check if share is still active
            if !share.is_active {
                return Ok(false);
            }

            // Check expiration
            if let Some(expires_at) = share.expires_at {
                if std::time::SystemTime::now() > expires_at {
                    return Ok(false);
                }
            }

            // Check if user is in the team
            let memberships = self.team_memberships.read().await;
            let user_teams = memberships.get(user_id).unwrap_or(&Vec::new());

            if user_teams.contains(&share.team_id) {
                return Ok(share.permissions.contains(required_permission));
            }
        }

        Ok(false)
    }

    /// Get pending invitations for user
    pub async fn get_pending_invitations(
        &self,
        user_email: &str,
    ) -> Result<Vec<TeamInvitation>, Box<dyn std::error::Error>> {
        let invitations = self.invitations.read().await;
        Ok(invitations
            .values()
            .filter(|inv| inv.invited_user_email == user_email && inv.status == InvitationStatus::Pending)
            .cloned()
            .collect())
    }
}

impl Default for TeamSettings {
    fn default() -> Self {
        Self {
            allow_public_projects:      false,
            require_approval_for_joins: true,
            default_member_role:        UserRole::Editor,
            max_members:                Some(50),
            max_projects:               Some(100),
        }
    }
}

impl std::fmt::Display for ShareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareType::ReadOnly => write!(f, "Read Only"),
            ShareType::ReadWrite => write!(f, "Read & Write"),
            ShareType::Admin => write!(f, "Admin"),
            ShareType::Custom => write!(f, "Custom"),
        }
    }
}

impl std::fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationStatus::Pending => write!(f, "Pending"),
            InvitationStatus::Accepted => write!(f, "Accepted"),
            InvitationStatus::Declined => write!(f, "Declined"),
            InvitationStatus::Expired => write!(f, "Expired"),
            InvitationStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}
