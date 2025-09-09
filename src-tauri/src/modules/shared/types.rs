//! Shared types used across the IDE modules

use serde::{Deserialize, Serialize};

/// Terminal event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalEvent {
    /// Terminal session ID
    pub id: String,
    /// Stream type (stdout/stderr)
    pub stream_type: String,
    /// Output line content
    pub line: String,
}