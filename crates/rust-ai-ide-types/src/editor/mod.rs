//! Editor-related types for Rust AI IDE

/// Editor cursor position
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditorCursor {
    pub line: u32,
    pub column: u32,
    pub selection: Option<std::ops::Range<usize>>,
}

/// Editor configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditorConfig {
    pub tab_size: u8,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub syntax_highlighting: bool,
}
