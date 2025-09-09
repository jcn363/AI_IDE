// Rust AI IDE Collaboration crate
// Provides CRDT-based collaborative editing features

pub mod crdt;

pub use crdt::*;

// Collaboration session management
#[derive(Debug, Clone)]
pub struct CollaborationSession {
    pub session_id: String,
    pub participants: Vec<String>,
    pub document_name: String,
    pub is_active: bool,
}

impl CollaborationSession {
    pub fn new(session_id: String, document_name: String) -> Self {
        Self {
            session_id,
            participants: Vec::new(),
            document_name,
            is_active: true,
        }
    }

    pub fn add_participant(&mut self, participant_id: String) {
        if !self.participants.contains(&participant_id) {
            self.participants.push(participant_id);
        }
    }

    pub fn remove_participant(&mut self, participant_id: &str) {
        self.participants.retain(|p| p != participant_id);
    }

    pub fn get_participant_count(&self) -> usize {
        self.participants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaboration_session() {
        let mut session =
            CollaborationSession::new("session1".to_string(), "document.rs".to_string());

        assert_eq!(session.session_id, "session1");
        assert_eq!(session.document_name, "document.rs");
        assert_eq!(session.get_participant_count(), 0);

        session.add_participant("user1".to_string());
        session.add_participant("user2".to_string());

        assert_eq!(session.get_participant_count(), 2);
        assert!(session.participants.contains(&"user1".to_string()));
        assert!(session.participants.contains(&"user2".to_string()));

        session.remove_participant("user1");
        assert_eq!(session.get_participant_count(), 1);
        assert!(!session.participants.contains(&"user1".to_string()));
        assert!(session.participants.contains(&"user2".to_string()));
    }
}
