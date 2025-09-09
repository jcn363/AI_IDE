use lsp_types::Range;
use serde::{Deserialize, Serialize};

/// Hover information returned by the LSP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    /// The hover's content
    pub contents: String,
    /// An optional range is a range inside a text document
    /// that is used to visualize a hover, e.g. by changing the background color.
    pub range: Option<Range>,
}

impl HoverInfo {
    /// Create a new HoverInfo with the given contents
    pub fn new(contents: String) -> Self {
        Self {
            contents,
            range: None,
        }
    }

    /// Create a new HoverInfo with contents and range
    pub fn with_range(contents: String, range: Range) -> Self {
        Self {
            contents,
            range: Some(range),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range};

    #[test]
    fn test_hover_info() {
        let hover = HoverInfo::new("test content".to_string());
        assert_eq!(hover.contents, "test content");
        assert!(hover.range.is_none());

        let range = Range::new(Position::new(1, 2), Position::new(1, 6));
        let hover_with_range = HoverInfo::with_range("test".to_string(), range);
        assert_eq!(hover_with_range.range, Some(range));
    }
}
