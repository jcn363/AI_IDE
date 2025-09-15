//! LSP (Language Server Protocol) testing utilities and fixtures
//!
//! Provides mock LSP servers, client implementations, and testing patterns
//! for LSP-based functionality testing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::TestError;

/// LSP Message types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LSPRequest {
    pub id: Option<u64>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LSPResponse {
    pub id: Option<u64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<LSPError>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LSPNotification {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LSPError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Mock LSP server for testing
#[derive(Clone)]
pub struct MockLSPServer {
    requests: Arc<Mutex<Vec<LSPRequest>>>,
    responses: HashMap<String, serde_json::Value>,
    notifications: Arc<Mutex<Vec<LSPNotification>>>,
}

impl MockLSPServer {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            responses: HashMap::new(),
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a predefined response for a specific method
    pub fn add_response(&mut self, method: &str, response: serde_json::Value) {
        self.responses.insert(method.to_string(), response);
    }

    /// Handle an incoming LSP request
    pub fn handle_request(&self, request: LSPRequest) -> Result<Option<LSPResponse>, TestError> {
        self.requests.lock().unwrap().push(request.clone());

        if let Some(response_data) = self.responses.get(&request.method) {
            Ok(Some(LSPResponse {
                id: request.id,
                result: Some(response_data.clone()),
                error: None,
            }))
        } else {
            Ok(None)
        }
    }

    /// Send a notification from the server
    pub fn send_notification(&self, notification: LSPNotification) {
        self.notifications.lock().unwrap().push(notification);
    }

    /// Get all received requests
    pub fn received_requests(&self) -> Vec<LSPRequest> {
        self.requests.lock().unwrap().clone()
    }

    /// Get all sent notifications
    pub fn sent_notifications(&self) -> Vec<LSPNotification> {
        self.notifications.lock().unwrap().clone()
    }

    /// Verify that a specific request was received
    pub fn verify_request_received(&self, method: &str) -> Result<(), TestError> {
        let requests = self.requests.lock().unwrap();
        if requests.iter().any(|req| req.method == method) {
            Ok(())
        } else {
            Err(TestError::Validation(
                crate::ValidationError::invalid_setup(format!(
                    "Expected request '{}' was not received",
                    method
                )),
            ))
        }
    }
}

impl Default for MockLSPServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock LSP client for testing
pub struct MockLSPClient {
    server: Arc<MockLSPServer>,
    outbound_notifications: Arc<Mutex<Vec<LSPNotification>>>,
}

impl MockLSPClient {
    pub fn new(server: Arc<MockLSPServer>) -> Self {
        Self {
            server,
            outbound_notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Send a request to the server
    pub async fn send_request(
        &self,
        request: LSPRequest,
    ) -> Result<Option<LSPResponse>, TestError> {
        self.server.handle_request(request)
    }

    /// Send a notification to the server
    pub fn send_notification(&self, notification: LSPNotification) {
        self.outbound_notifications
            .lock()
            .unwrap()
            .push(notification.clone());
        // In a real implementation, this would send to the server
    }

    /// Receive a response from the server
    pub async fn receive_response(&self) -> Result<Option<LSPResponse>, TestError> {
        // Mock implementation - in reality, this would listen for responses
        Ok(None)
    }

    /// Receive a notification from the server
    pub fn receive_notification(&self) -> Option<LSPNotification> {
        let mut notifications = self.server.sent_notifications();
        notifications.pop()
    }
}

//// LSP test fixture with pre-configured server and client
pub struct LSPFixture {
    server: MockLSPServer,
    client: MockLSPClient,
}

impl LSPFixture {
    pub fn new() -> Self {
        let server = MockLSPServer::new();
        let temp_server = server.clone(); // Clone for client
        let client = MockLSPClient::new(Arc::new(temp_server));

        Self { server, client }
    }

    /// Setup common LSP initialization sequence
    pub async fn setup_with_init(&mut self) -> Result<(), TestError> {
        // Mock initialize request/response
        let init_request = LSPRequest {
            id: Some(1),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "processId": null,
                "rootPath": "/test/workspace",
                "capabilities": {}
            })),
        };

        let init_response = serde_json::json!({
            "capabilities": {
                "textDocumentSync": 1,
                "hoverProvider": true,
                "definitionProvider": true
            }
        });

        self.server.add_response("initialize", init_response);

        let response = self.client.send_request(init_request).await?;
        assert!(response.is_some(), "Initialize response should be present");

        // Send initialized notification
        let initialized_notification = LSPNotification {
            method: "initialized".to_string(),
            params: Some(serde_json::json!({})),
        };

        self.client.send_notification(initialized_notification);

        Ok(())
    }

    pub fn server(&self) -> &MockLSPServer {
        &self.server
    }

    pub fn client(&self) -> &MockLSPClient {
        &self.client
    }

    pub fn server_mut(&mut self) -> &mut MockLSPServer {
        &mut self.server
    }
}

impl Default for LSPFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined LSP fixture builders
pub struct LSPFixtures;

impl LSPFixtures {
    /// Create a basic LSP fixture with minimal setup
    pub fn basic() -> LSPFixture {
        LSPFixture::new()
    }

    /// Create a fixture with full initialization sequence
    pub fn initialized() -> LSPFixture {
        let mut fixture = LSPFixture::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(fixture.setup_with_init()).unwrap();
        fixture
    }

    /// Create a fixture with common Rust language server capabilities
    pub fn rust_language_server() -> LSPFixture {
        let mut fixture = Self::initialized();

        // Add responses directly since we have mutable access
        fixture.server.add_response(
            "textDocument/completion",
            serde_json::json!({
                "isIncomplete": false,
                "items": [
                    {
                        "label": "println!",
                        "kind": 3,
                        "detail": "macro",
                        "documentation": "Prints to stdout"
                    }
                ]
            }),
        );

        fixture.server.add_response(
            "textDocument/definition",
            serde_json::json!({
                "uri": "file:///test/src/lib.rs",
                "range": {
                    "start": {"line": 10, "character": 5},
                    "end": {"line": 10, "character": 15}
                }
            }),
        );

        fixture
    }

    /// Create a fixture with Cargo-specific capabilities
    pub fn cargo_server() -> LSPFixture {
        let mut fixture = Self::initialized();

        fixture.server.add_response(
            "cargo/check",
            serde_json::json!({
                "success": true,
                "diagnostics": []
            }),
        );

        fixture.server.add_response(
            "cargo/build",
            serde_json::json!({
                "success": true,
                "output": "Finished dev [unoptimized + debuginfo]"
            }),
        );

        fixture
    }
}

/// LSP message builder for creating test messages
pub struct LSPMessageBuilder;

impl LSPMessageBuilder {
    /// Create an initialize request
    pub fn initialize_request(root_path: &str) -> LSPRequest {
        LSPRequest {
            id: Some(1),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "processId": null,
                "rootPath": root_path,
                "capabilities": {
                    "workspace": {
                        "configuration": true
                    },
                    "textDocument": {
                        "publishDiagnostics": {
                            "relatedInformation": true
                        }
                    }
                }
            })),
        }
    }

    /// Create a text document open notification
    pub fn did_open_notification(uri: &str, language_id: &str, text: &str) -> LSPNotification {
        LSPNotification {
            method: "textDocument/didOpen".to_string(),
            params: Some(serde_json::json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": text
                }
            })),
        }
    }

    /// Create a completion request
    pub fn completion_request(uri: &str, line: u32, character: u32) -> LSPRequest {
        LSPRequest {
            id: Some(2),
            method: "textDocument/completion".to_string(),
            params: Some(serde_json::json!({
                "textDocument": {
                    "uri": uri
                },
                "position": {
                    "line": line,
                    "character": character
                }
            })),
        }
    }

    /// Create a hover request
    pub fn hover_request(uri: &str, line: u32, character: u32) -> LSPRequest {
        LSPRequest {
            id: Some(3),
            method: "textDocument/hover".to_string(),
            params: Some(serde_json::json!({
                "textDocument": {
                    "uri": uri
                },
                "position": {
                    "line": line,
                    "character": character
                }
            })),
        }
    }

    /// Create a definition request
    pub fn definition_request(uri: &str, line: u32, character: u32) -> LSPRequest {
        LSPRequest {
            id: Some(4),
            method: "textDocument/definition".to_string(),
            params: Some(serde_json::json!({
                "textDocument": {
                    "uri": uri
                },
                "position": {
                    "line": line,
                    "character": character
                }
            })),
        }
    }
}

/// LSP assertion utilities for test verification
pub struct LSPAssertions;

impl LSPAssertions {
    pub fn assert_request_received(server: &MockLSPServer, method: &str) -> Result<(), TestError> {
        server.verify_request_received(method)
    }

    pub fn assert_response_contains(
        response: &LSPResponse,
        key: &str,
        expected: &serde_json::Value,
    ) -> Result<(), TestError> {
        if let Some(result) = &response.result {
            if let Some(value) = result.get(key) {
                if value == expected {
                    Ok(())
                } else {
                    Err(TestError::Validation(
                        crate::ValidationError::invalid_setup(format!(
                            "Expected {} for key '{}', got {}",
                            expected, key, value
                        )),
                    ))
                }
            } else {
                Err(TestError::Validation(
                    crate::ValidationError::invalid_setup(format!(
                        "Key '{}' not found in response",
                        key
                    )),
                ))
            }
        } else {
            Err(TestError::Validation(
                crate::ValidationError::invalid_setup("No result in response"),
            ))
        }
    }

    pub fn assert_notification_sent(client: &MockLSPClient, method: &str) -> Result<(), TestError> {
        let notifications = client.outbound_notifications.lock().unwrap();
        if notifications.iter().any(|n| n.method == method) {
            Ok(())
        } else {
            Err(TestError::Validation(
                crate::ValidationError::invalid_setup(format!(
                    "Expected notification '{}' was not sent",
                    method
                )),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_lsp_server() {
        let mut server = MockLSPServer::new();

        server.add_response("test/method", serde_json::json!("test response"));

        let request = LSPRequest {
            id: Some(1),
            method: "test/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).unwrap().unwrap();
        assert_eq!(response.result.unwrap(), "test response");
    }

    #[test]
    fn test_lsp_message_builder() {
        let init_request = LSPMessageBuilder::initialize_request("/test/path");

        assert_eq!(init_request.method, "initialize");
        assert_eq!(init_request.id, Some(1));
        assert!(init_request.params.is_some());
    }

    #[test]
    fn test_lsp_fixture() {
        let mut fixture = LSPFixtures::basic();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(fixture.setup_with_init()).unwrap();

        let requests = fixture.server().received_requests();
        assert!(!requests.is_empty());
        assert_eq!(requests[0].method, "initialize");
    }

    #[test]
    fn test_lsp_assertions() {
        let mut server = MockLSPServer::new();
        let request = LSPRequest {
            id: Some(1),
            method: "test".to_string(),
            params: None,
        };

        server.handle_request(request).unwrap();
        assert!(LSPAssertions::assert_request_received(&server, "test").is_ok());
    }
}
