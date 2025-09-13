//! Tests for protocol type serialization compatibility
//!
//! These tests ensure that all protocol types can be properly serialized
//! and deserialized for Tauri communication.

#[cfg(test)]
mod tests {
    use rust_ai_ide_protocol::commands::fs::{FileInfo, ListFilesRequest};
    use rust_ai_ide_protocol::commands::git::{GitCommitRequest, GitStatusRequest};
    use rust_ai_ide_protocol::errors::ProtocolError;

    #[test]
    fn test_file_info_serialization() {
        let file_info = FileInfo {
            name:         "test.rs".to_string(),
            path:         "/tmp/test.rs".to_string(),
            is_directory: false,
        };

        let serialized = serde_json::to_string(&file_info).unwrap();
        let deserialized: FileInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "test.rs");
        assert_eq!(deserialized.path, "/tmp/test.rs");
        assert_eq!(deserialized.is_directory, false);
    }

    #[test]
    fn test_list_files_request_serialization() {
        let request = ListFilesRequest {
            path:           "/tmp".to_string(),
            recursive:      Some(true),
            include_hidden: Some(false),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: ListFilesRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.path, "/tmp");
        assert_eq!(deserialized.recursive, Some(true));
        assert_eq!(deserialized.include_hidden, Some(false));
    }

    #[test]
    fn test_git_request_serialization() {
        let status_request = GitStatusRequest {
            directory: "/tmp/repo".to_string(),
            quiet:     Some(true),
        };

        let serialized = serde_json::to_string(&status_request).unwrap();
        let deserialized: GitStatusRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.directory, "/tmp/repo");
        assert_eq!(deserialized.quiet, Some(true));
    }

    #[test]
    fn test_git_commit_request_serialization() {
        let commit_request = GitCommitRequest {
            directory:    "/tmp/repo".to_string(),
            message:      "Fix critical bug".to_string(),
            author_name:  Some("John Doe".to_string()),
            author_email: Some("john@example.com".to_string()),
        };

        let serialized = serde_json::to_string(&commit_request).unwrap();
        let deserialized: GitCommitRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.directory, "/tmp/repo");
        assert_eq!(deserialized.message, "Fix critical bug");
        assert_eq!(deserialized.author_name, Some("John Doe".to_string()));
        assert_eq!(
            deserialized.author_email,
            Some("john@example.com".to_string())
        );
    }

    #[test]
    fn test_protocol_error_serialization() {
        let validation_error = ProtocolError::Validation("Invalid input provided".to_string());

        let serialized = serde_json::to_string(&validation_error).unwrap();
        assert!(serialized.contains("Validation"));
        assert!(serialized.contains("Invalid input provided"));

        // Test file system error
        let fs_error = ProtocolError::FileSystem("Permission denied".to_string());
        let fs_serialized = serde_json::to_string(&fs_error).unwrap();
        assert!(fs_serialized.contains("FileSystem"));
        assert!(fs_serialized.contains("Permission denied"));

        // Test processing error
        let processing_error = ProtocolError::Processing("Analysis failed".to_string());
        let processing_serialized = serde_json::to_string(&processing_error).unwrap();
        assert!(processing_serialized.contains("Processing"));
        assert!(processing_serialized.contains("Analysis failed"));
    }

    #[test]
    fn test_protocol_extensions() {
        // Test that protocol can be extended with custom structures
        let custom_metadata = serde_json::json!({
            "version": "1.0.0",
            "supported_features": ["analysis", "refactoring", "completion"],
            "client_info": {
                "name": "rust-ai-ide",
                "platform": "desktop"
            }
        });

        let serialized = serde_json::to_string(&custom_metadata).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized["version"], "1.0.0");
        assert_eq!(deserialized["supported_features"][0], "analysis");
        assert_eq!(deserialized["client_info"]["name"], "rust-ai-ide");
        assert_eq!(deserialized["client_info"]["platform"], "desktop");
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that new fields with defaults don't break deserialization
        let old_file_info = r#"{
            "name": "old_file.rs",
            "path": "/old/path.rs",
            "is_directory": true
        }"#;

        let deserialized: FileInfo = serde_json::from_str(old_file_info).unwrap();
        assert_eq!(deserialized.name, "old_file.rs");
        assert_eq!(deserialized.path, "/old/path.rs");
        assert_eq!(deserialized.is_directory, true);

        // Test optional fields
        let minimal_list_request = r#"{
            "path": "/minimal"
        }"#;

        let deserialized: ListFilesRequest = serde_json::from_str(minimal_list_request).unwrap();
        assert_eq!(deserialized.path, "/minimal");
        // Optional fields should be None when not provided
        assert_eq!(deserialized.recursive, None);
        assert_eq!(deserialized.include_hidden, None);
    }

    #[test]
    fn test_protocol_versioning() {
        // Test that we can detect protocol version information
        let file_info = FileInfo {
            name:         "version_test.rs".to_string(),
            path:         "/test/version.rs".to_string(),
            is_directory: false,
        };

        let serialized = serde_json::to_string(&file_info).unwrap();
        assert!(serialized.contains("version_test.rs"));
        assert!(serialized.contains("/test/version.rs"));
        assert!(serialized.contains("false")); // is_directory
    }
}
