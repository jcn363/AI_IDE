//! Language Server Protocol types

/// LSP client configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspClientConfig {
    pub server_executable: String,
    pub server_args:       Vec<String>,
    pub root_uri:          String,
    pub capabilities:      serde_json::Value,
}

/// LSP server response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspResponse {
    pub id:     u64,
    pub result: Option<serde_json::Value>,
    pub error:  Option<LspErrorResponse>,
}

/// LSP error response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspErrorResponse {
    pub code:    i32,
    pub message: String,
}
