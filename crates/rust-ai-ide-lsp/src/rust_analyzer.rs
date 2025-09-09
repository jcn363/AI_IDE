//! Rust-analyzer specific LSP client functionality

use lsp_types::{
    ClientCapabilities, DidChangeWatchedFilesClientCapabilities,
    DocumentFormattingClientCapabilities, DocumentOnTypeFormattingClientCapabilities,
    DocumentRangeFormattingClientCapabilities, PublishDiagnosticsClientCapabilities,
    RenameClientCapabilities, TextDocumentClientCapabilities, WorkspaceClientCapabilities,
};
use lsp_types::{
    CodeActionClientCapabilities as CodeActionCapability,
    CompletionClientCapabilities as CompletionCapability,
    DocumentSymbolClientCapabilities as DocumentSymbolCapability,
    HoverClientCapabilities as HoverCapability,
    SignatureHelpClientCapabilities as SignatureHelpCapability,
};
use serde_json::{Map, Value};
use std::path::Path;

/// Generate rust-analyzer specific initialization options
pub fn rust_analyzer_config(root_path: &Path, config: &super::LSPClientConfig) -> Value {
    let mut ra_config = Map::new();

    // Check on save configuration
    let mut check_on_save = Map::new();
    check_on_save.insert("command".to_string(), Value::String("clippy".to_string()));
    check_on_save.insert(
        "extraArgs".to_string(),
        Value::Array(vec![
            Value::String("--all-targets".to_string()),
            Value::String("--all-features".to_string()),
            Value::String("--".to_string()),
            Value::String("-D".to_string()),
            Value::String("warnings".to_string()),
        ]),
    );

    // Cargo configuration
    let mut cargo = Map::new();
    cargo.insert("allFeatures".to_string(), Value::Bool(true));
    cargo.insert("loadOutDirsFromCheck".to_string(), Value::Bool(true));
    cargo.insert("runBuildScripts".to_string(), Value::Bool(true));

    // Proc macro configuration
    let mut proc_macro = Map::new();
    proc_macro.insert("enable".to_string(), Value::Bool(config.enable_proc_macro));

    // Inlay hints configuration
    let mut inlay_hints = Map::new();
    inlay_hints.insert("enable".to_string(), Value::Bool(config.enable_inlay_hints));

    let mut chaining_hints = Map::new();
    chaining_hints.insert("enable".to_string(), Value::Bool(true));
    inlay_hints.insert("chainingHints".to_string(), Value::Object(chaining_hints));

    let mut parameter_hints = Map::new();
    parameter_hints.insert("enable".to_string(), Value::Bool(true));
    inlay_hints.insert("parameterHints".to_string(), Value::Object(parameter_hints));

    let mut type_hints = Map::new();
    type_hints.insert("enable".to_string(), Value::Bool(true));
    inlay_hints.insert("typeHints".to_string(), Value::Object(type_hints));

    // Files configuration
    let mut files = Map::new();
    let watcher = if cfg!(target_os = "linux") {
        "client"
    } else {
        "notify"
    };
    files.insert("watcher".to_string(), Value::String(watcher.to_string()));

    // Lens configuration
    let mut lens = Map::new();
    lens.insert("enable".to_string(), Value::Bool(true));
    lens.insert("run".to_string(), Value::Bool(true));
    lens.insert("debug".to_string(), Value::Bool(true));
    lens.insert("implementations".to_string(), Value::Bool(true));
    lens.insert("references".to_string(), Value::Bool(true));
    lens.insert("references.adt".to_string(), Value::Bool(true));
    lens.insert("references.trait".to_string(), Value::Bool(true));
    lens.insert("references.method".to_string(), Value::Bool(true));

    // Build the final configuration
    ra_config.insert("checkOnSave".to_string(), Value::Object(check_on_save));
    ra_config.insert("cargo".to_string(), Value::Object(cargo));
    ra_config.insert("procMacro".to_string(), Value::Object(proc_macro));
    ra_config.insert("inlayHints".to_string(), Value::Object(inlay_hints));
    ra_config.insert("files".to_string(), Value::Object(files));
    ra_config.insert("lens".to_string(), Value::Object(lens));

    // Cache configuration
    let mut cache = Map::new();
    cache.insert("warmup".to_string(), Value::Bool(true));
    ra_config.insert("cache".to_string(), Value::Object(cache));

    // Diagnostics configuration
    let mut diagnostics = Map::new();
    diagnostics.insert("enable".to_string(), Value::Bool(true));
    diagnostics.insert("enableExperimental".to_string(), Value::Bool(true));
    ra_config.insert("diagnostics".to_string(), Value::Object(diagnostics));

    // Completion configuration
    let mut completion = Map::new();
    let mut autoimport = Map::new();
    autoimport.insert("enable".to_string(), Value::Bool(true));
    completion.insert("autoimport".to_string(), Value::Object(autoimport));
    ra_config.insert("completion".to_string(), Value::Object(completion));

    // Return the final configuration wrapped in a "rust-analyzer" object
    let mut config = Map::new();
    config.insert("rust-analyzer".to_string(), Value::Object(ra_config));

    // Add rootPath to the config
    config.insert(
        "rootPath".to_string(),
        Value::String(root_path.to_string_lossy().to_string()),
    );

    Value::Object(config)
}

/// Generate rust-analyzer specific client capabilities
pub fn rust_analyzer_capabilities() -> lsp_types::ClientCapabilities {
    ClientCapabilities {
        text_document: Some(TextDocumentClientCapabilities {
            completion: Some(CompletionCapability {
                dynamic_registration: None,
                completion_item: None,
                completion_item_kind: None,
                context_support: None,
                insert_text_mode: None,
                completion_list: None,
            }),
            hover: Some(HoverCapability::default()),
            signature_help: Some(SignatureHelpCapability::default()),
            document_symbol: Some(DocumentSymbolCapability {
                dynamic_registration: Some(true),
                ..Default::default()
            }),
            code_action: Some(CodeActionCapability {
                dynamic_registration: Some(true),
                code_action_literal_support: None,
                is_preferred_support: None,
                disabled_support: None,
                data_support: None,
                resolve_support: None,
                honors_change_annotations: None,
            }),
            formatting: Some(DocumentFormattingClientCapabilities {
                dynamic_registration: Some(true),
                ..Default::default()
            }),
            range_formatting: Some(DocumentRangeFormattingClientCapabilities {
                dynamic_registration: Some(true),
            }),
            on_type_formatting: Some(DocumentOnTypeFormattingClientCapabilities {
                dynamic_registration: Some(true),
            }),
            rename: Some(RenameClientCapabilities {
                dynamic_registration: Some(true),
                ..Default::default()
            }),
            publish_diagnostics: Some(PublishDiagnosticsClientCapabilities {
                related_information: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }),
        workspace: Some(WorkspaceClientCapabilities {
            configuration: Some(true),
            workspace_folders: Some(true),
            did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                dynamic_registration: Some(true),
                relative_pattern_support: Some(true),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn test_rust_analyzer_config() {
        let config = super::super::LSPClientConfig::default();
        let path = Path::new("/test/path");
        let json = rust_analyzer_config(path, &config);

        assert!(json.get("rust-analyzer").is_some());
        assert_eq!(json["rust-analyzer"]["procMacro"]["enable"], json!(true));
        assert_eq!(json["rust-analyzer"]["inlayHints"]["enable"], json!(true));
    }
}
