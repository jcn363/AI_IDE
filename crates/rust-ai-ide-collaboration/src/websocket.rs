// WebSocket server for real-time collaborative editing
// Implements secure WebSocket communication on port 3001 with TLS 1.3

use futures_util::{SinkExt, StreamExt};
use rust_ai_ide_common::validation::TauriInputSanitizer;
use rust_ai_ide_security::tls_config;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_hdr_async, WebSocketStream};

use crate::crdt::{EditorOperation, LamportClock, TextDocument};
use crate::session_management::{CollaborationSession, SessionManager};
use crate::CollaborationService;

/// WebSocket message types for collaborative editing
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CollaborationMessage {
    /// Join a collaborative session
    JoinSession {
        session_id: String,
        user_id: String,
        client_id: String,
    },
    /// Leave a collaborative session
    LeaveSession { session_id: String, user_id: String },
    /// Send an operation to other clients
    Operation {
        session_id: String,
        operation: EditorOperation,
        user_id: String,
    },
    /// Request synchronization with current state
    SyncRequest { session_id: String, user_id: String },
    /// Send synchronization data
    SyncResponse {
        session_id: String,
        operations: Vec<EditorOperation>,
        current_content: String,
    },
    /// Error message
    Error { message: String, code: String },
}

/// WebSocket server for collaborative editing
pub struct CollaborationWebSocketServer {
    sessions: Arc<RwLock<HashMap<String, SessionConnections>>>,
    collaboration_service: Arc<RwLock<CollaborationService>>,
    session_manager: Arc<RwLock<SessionManager>>,
    sanitizer: TauriInputSanitizer,
}

#[derive(Debug)]
struct SessionConnections {
    connections: HashMap<String, mpsc::UnboundedSender<Message>>,
    document: Arc<RwLock<TextDocument>>,
}

impl CollaborationWebSocketServer {
    pub fn new(
        collaboration_service: Arc<RwLock<CollaborationService>>,
        session_manager: Arc<RwLock<SessionManager>>,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            collaboration_service,
            session_manager,
            sanitizer: TauriInputSanitizer::new(),
        }
    }

    /// Start the WebSocket server on port 3001 with TLS 1.3
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = "0.0.0.0:3001";
        let listener = TcpListener::bind(addr).await?;
        log::info!("WebSocket server listening on {}", addr);

        // TLS configuration for secure connections (TLS 1.3)
        let tls_config = tls_config::create_tls_config()?;
        let acceptor =
            tokio_native_tls::TlsAcceptor::from(native_tls::TlsAcceptor::new(tls_config)?);

        loop {
            let (tcp_stream, _) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let sessions = self.sessions.clone();
            let collaboration_service = self.collaboration_service.clone();
            let session_manager = self.session_manager.clone();
            let sanitizer = self.sanitizer.clone();

            tokio::spawn(async move {
                match acceptor.accept(tcp_stream).await {
                    Ok(tls_stream) => {
                        if let Err(e) = Self::handle_connection(
                            tls_stream,
                            sessions,
                            collaboration_service,
                            session_manager,
                            sanitizer,
                        )
                        .await
                        {
                            log::error!("WebSocket connection error: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("TLS handshake error: {}", e);
                    }
                }
            });
        }
    }

    async fn handle_connection(
        tls_stream: tokio_native_tls::TlsStream<tokio::net::TcpStream>,
        sessions: Arc<RwLock<HashMap<String, SessionConnections>>>,
        collaboration_service: Arc<RwLock<CollaborationService>>,
        session_manager: Arc<RwLock<SessionManager>>,
        sanitizer: TauriInputSanitizer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let callback = |req: &Request, response: Response| {
            log::info!("WebSocket connection from: {:?}", req.uri());
            Ok(response)
        };

        let ws_stream = accept_hdr_async(tls_stream, callback).await?;
        let (sender, receiver) = mpsc::unbounded_channel();
        let (mut write, mut read) = ws_stream.split();

        // Handle incoming messages
        let sessions_clone = sessions.clone();
        let collaboration_service_clone = collaboration_service.clone();
        let session_manager_clone = session_manager.clone();
        let sanitizer_clone = sanitizer.clone();

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = Self::handle_message(
                            &text,
                            &sessions_clone,
                            &collaboration_service_clone,
                            &session_manager_clone,
                            &sanitizer_clone,
                        )
                        .await
                        {
                            log::error!("Error handling message: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {} // Ignore other message types
                }
            }
        });

        // Handle outgoing messages
        while let Some(message) = receiver.recv().await {
            if write.send(message).await.is_err() {
                break;
            }
        }

        Ok(())
    }

    async fn handle_message(
        text: &str,
        sessions: &Arc<RwLock<HashMap<String, SessionConnections>>>,
        collaboration_service: &Arc<RwLock<CollaborationService>>,
        session_manager: &Arc<RwLock<SessionManager>>,
        sanitizer: &TauriInputSanitizer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Sanitize input to prevent injection attacks
        let sanitized_text = sanitizer.sanitize_string(text)?;

        let message: CollaborationMessage = serde_json::from_str(&sanitized_text)?;

        match message {
            CollaborationMessage::JoinSession {
                session_id,
                user_id,
                client_id,
            } => {
                Self::handle_join_session(
                    session_id,
                    user_id,
                    client_id,
                    sessions,
                    collaboration_service,
                    session_manager,
                )
                .await?;
            }
            CollaborationMessage::LeaveSession {
                session_id,
                user_id,
            } => {
                Self::handle_leave_session(session_id, user_id, sessions, session_manager).await?;
            }
            CollaborationMessage::Operation {
                session_id,
                operation,
                user_id,
            } => {
                Self::handle_operation(
                    session_id,
                    operation,
                    user_id,
                    sessions,
                    collaboration_service,
                )
                .await?;
            }
            CollaborationMessage::SyncRequest {
                session_id,
                user_id,
            } => {
                Self::handle_sync_request(session_id, user_id, sessions, collaboration_service)
                    .await?;
            }
            _ => {
                log::warn!("Unhandled message type");
            }
        }

        Ok(())
    }

    async fn handle_join_session(
        session_id: String,
        user_id: String,
        client_id: String,
        sessions: &Arc<RwLock<HashMap<String, SessionConnections>>>,
        collaboration_service: &Arc<RwLock<CollaborationService>>,
        session_manager: &Arc<RwLock<SessionManager>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions_lock = sessions.write().await;

        // Create session if it doesn't exist
        if !sessions_lock.contains_key(&session_id) {
            let collaboration_state = collaboration_service.read().await;
            if let Some(editing_state) = collaboration_state
                .state
                .read()
                .await
                .active_editing_state
                .get(&session_id)
            {
                let session_connections = SessionConnections {
                    connections: HashMap::new(),
                    document: Arc::new(RwLock::new(editing_state.clone())),
                };
                sessions_lock.insert(session_id.clone(), session_connections);
            } else {
                // Create new session
                let mut collaboration_state = collaboration_service.write().await;
                collaboration_state
                    .create_session(session_id.clone(), format!("doc_{}", session_id))
                    .await?;
                let document = TextDocument::new(client_id.clone());
                let session_connections = SessionConnections {
                    connections: HashMap::new(),
                    document: Arc::new(RwLock::new(document)),
                };
                sessions_lock.insert(session_id.clone(), session_connections);
            }
        }

        // Add user to session
        if let Some(session_connections) = sessions_lock.get_mut(&session_id) {
            session_connections
                .connections
                .insert(user_id, mpsc::unbounded_channel().0);
        }

        log::info!("User {} joined session {}", user_id, session_id);
        Ok(())
    }

    async fn handle_leave_session(
        session_id: String,
        user_id: String,
        sessions: &Arc<RwLock<HashMap<String, SessionConnections>>>,
        session_manager: &Arc<RwLock<SessionManager>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions_lock = sessions.write().await;

        if let Some(session_connections) = sessions_lock.get_mut(&session_id) {
            session_connections.connections.remove(&user_id);

            // Remove session if empty
            if session_connections.connections.is_empty() {
                sessions_lock.remove(&session_id);
            }
        }

        log::info!("User {} left session {}", user_id, session_id);
        Ok(())
    }

    async fn handle_operation(
        session_id: String,
        operation: EditorOperation,
        user_id: String,
        sessions: &Arc<RwLock<HashMap<String, SessionConnections>>>,
        collaboration_service: &Arc<RwLock<CollaborationService>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sessions_lock = sessions.read().await;

        if let Some(session_connections) = sessions_lock.get(&session_id) {
            let mut document = session_connections.document.write().await;
            let result = document.apply_operation(operation.clone());

            // Broadcast operation to other clients in the session
            let operation_message = CollaborationMessage::Operation {
                session_id: session_id.clone(),
                operation,
                user_id,
            };
            let message_json = serde_json::to_string(&operation_message)?;

            for (client_user_id, sender) in &session_connections.connections {
                if client_user_id != &user_id {
                    let _ = sender.send(Message::Text(message_json.clone()));
                }
            }
        }

        Ok(())
    }

    async fn handle_sync_request(
        session_id: String,
        user_id: String,
        sessions: &Arc<RwLock<HashMap<String, SessionConnections>>>,
        collaboration_service: &Arc<RwLock<CollaborationService>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sessions_lock = sessions.read().await;

        if let Some(session_connections) = sessions_lock.get(&session_id) {
            let document = session_connections.document.read().await;

            let sync_response = CollaborationMessage::SyncResponse {
                session_id,
                operations: document.get_operation_log().to_vec(),
                current_content: document.get_content().to_string(),
            };

            let message_json = serde_json::to_string(&sync_response)?;

            if let Some(sender) = session_connections.connections.get(&user_id) {
                let _ = sender.send(Message::Text(message_json));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaboration_message_serialization() {
        let message = CollaborationMessage::JoinSession {
            session_id: "session123".to_string(),
            user_id: "user456".to_string(),
            client_id: "client789".to_string(),
        };

        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: CollaborationMessage = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            CollaborationMessage::JoinSession {
                session_id,
                user_id,
                client_id,
            } => {
                assert_eq!(session_id, "session123");
                assert_eq!(user_id, "user456");
                assert_eq!(client_id, "client789");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
