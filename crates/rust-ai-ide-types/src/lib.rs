//! Shared types for Rust AI IDE frontend and backend

pub mod ai;
pub mod cargo;
pub mod diagnostics;
pub mod editor;
pub mod lsp;
pub mod performance;
pub mod project;
pub mod user_interface;

/// Core types that are serialized/deserialized between frontend and backend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub cargo_packages: Vec<CargoPackage>,
    pub settings: WorkspaceSettings,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub manifest_path: String,
    pub is_workspace_member: bool,
    pub edition: String,
    pub features: Vec<String>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_requirement: String,
    pub source: Option<String>,
    pub is_optional: bool,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceSettings {
    pub rust_analyzer: RustAnalyzerSettings,
    pub cargo: CargoSettings,
    pub editor: EditorSettings,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RustAnalyzerSettings {
    pub enable: bool,
    pub diagnostics: DiagnosticSettings,
    pub inlay_hints: InlayHintSettings,
    pub completion: CompletionSettings,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticSettings {
    pub enable: bool,
    pub check_on_save: bool,
    pub cargo_check_args: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InlayHintSettings {
    pub enable: bool,
    pub type_hints: bool,
    pub parameter_hints: bool,
    pub chaining_hints: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionSettings {
    pub enable: bool,
    pub autoshow: bool,
    pub import_insert_behavior: String,
    pub import_granularity: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CargoSettings {
    pub target_directory: Option<String>,
    pub package_filters: Vec<String>,
    pub check_args: Vec<String>,
    pub test_args: Vec<String>,
    pub coverage_args: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditorSettings {
    pub font_family: String,
    pub font_size: u16,
    pub tab_size: u8,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub line_numbers: bool,
    pub minimap: bool,
    pub theme: String,
}
