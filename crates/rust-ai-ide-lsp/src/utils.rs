//! Utility functions for the LSP client

use lsp_types::Uri;
use std::path::Path;

/// Convert a file path to an LSP URI
pub fn path_to_uri(path: impl AsRef<Path>) -> Result<Uri, String> {
    let path = path.as_ref();
    let url = url::Url::from_file_path(path)
        .map_err(|_| format!("Failed to convert path to URL: {:?}", path))?;
    let uri = url
        .as_str()
        .parse::<Uri>()
        .map_err(|_| format!("Failed to convert URL to URI: {:?}", path))?;

    Ok(uri)
}

/// Convert an LSP URI to a file path
pub fn uri_to_path(uri: &Uri) -> Result<std::path::PathBuf, String> {
    let url = url::Url::parse(uri.as_str())
        .map_err(|_| format!("Failed to parse URI as URL: {}", uri.as_str()))?;
    url.to_file_path()
        .map_err(|_| format!("Failed to convert URL to path: {}", uri.as_str()))
}
