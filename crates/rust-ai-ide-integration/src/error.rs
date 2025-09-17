//! Error types for the integration bridge

use rust_ai_ide_errors::IDEError;
use thiserror::Error;

/// Errors that can occur in the collaboration-LSP integration bridge
#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("LSP communication failed: {message}")]
    LSPCommunication { message: String },

    #[error("Collaboration state synchronization failed: {message}")]
    CollaborationSync { message: String },

    #[error("CRDT operation translation failed: {message}")]
    CRDTTranslation { message: String },

    #[error("Conflict resolution failed: {message}")]
    ConflictResolution { message: String },

    #[error("Document state inconsistency detected: {document_uri}")]
    DocumentStateInconsistency { document_uri: String },

    #[error("Workspace synchronization failed: {workspace_id}")]
    WorkspaceSync { workspace_id: String },

    #[error("Timeout during integration operation: {operation}")]
    Timeout { operation: String },

    #[error("Invalid document state: {message}")]
    InvalidDocumentState { message: String },

    #[error("AI conflict resolution failed: {message}")]
    AIConflictResolution { message: String },

    #[error("Security validation failed: {message}")]
    SecurityValidation { message: String },

    #[error("Integration bridge initialization failed: {message}")]
    Initialization { message: String },

    #[error("Fallback mechanism activated: {reason}")]
    FallbackActivated { reason: String },

    #[error("Concurrent operation conflict: {operation}")]
    ConcurrentOperationConflict { operation: String },

    #[error("Bridge capacity exceeded: {resource}")]
    CapacityExceeded { resource: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Underlying IDE error: {0}")]
    IDEError(#[from] IDEError),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Other error: {message}")]
    Other { message: String },
}

/// Result type alias for integration operations
pub type IntegrationResult<T> = Result<T, IntegrationError>;

/// Helper trait for converting errors to integration errors with context
pub trait IntegrationResultExt<T> {
    fn with_integration_context(self, context: &str) -> IntegrationResult<T>;
}

impl<T, E> IntegrationResultExt<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn with_integration_context(self, context: &str) -> IntegrationResult<T> {
        self.map_err(|e| IntegrationError::Other {
            message: format!("{}: {}", context, e),
        })
    }
}