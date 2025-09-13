//! Language server implementations for web technologies (HTML, CSS, SQL)

use lsp_types::{
    ClientCapabilities, CodeAction, CodeActionContext, CodeActionParams, CodeActionProviderCapability,
    CodeActionResponse, CodeLens, CodeLensParams, Color, ColorInformation, ColorPresentation,
    ColorPresentationParams, Command, CompletionItem, CompletionOptions, CompletionParams,
    CompletionResponse, Diagnostic, DocumentColorParams, DocumentFormattingParams,
    DocumentHighlight, DocumentHighlightParams, DocumentSymbol, DocumentSymbolParams, FormattingOptions,
    Hover, HoverParams, InitializeParams, InitializeResult, Location, Position, Range,
    ReferenceContext, ReferenceParams, RenameFilesParams, RenameParams, ServerCapabilities, SymbolKind,
    TextDocumentPositionParams, TextEdit, Url, WorkDoneProgressParams, WorkspaceEdit,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::client::LSPError;
use crate::language_server::{
    GenericLspServer, LanguageInitializationOptions, LanguageServerConfig, LanguageServerKind,
};

/// HTML specific completion items
fn get_html_completion_items() -> Vec<CompletionItem> {
    vec![
        CompletionItem::new_simple("div".to_string(), "<div>\n    $0\n</div>".to_string()),
        CompletionItem::new_simple("span".to_string(), "<span>$1</span>$0".to_string()),
        CompletionItem::new_simple("a".to_string(), "<a href=\"$1\">$2</a>$0".to_string()),
        CompletionItem::new_simple("img".to_string(), "<img src=\"$1\" alt=\"$2\">$0".to_string()),
        CompletionItem::new_simple("button".to_string(), "<button>$1</button>$0".to_string()),
    ]
}

/// CSS specific completion items
fn get_css_completion_items() -> Vec<CompletionItem> {
    vec![
        CompletionItem::new_simple("color".to_string(), "color: $1;$0".to_string()),
        CompletionItem::new_simple("display".to_string(), "display: $1;$0".to_string()),
        CompletionItem::new_simple("margin".to_string(), "margin: $1;$0".to_string()),
        CompletionItem::new_simple("padding".to_string(), "padding: $1;$0".to_string()),
        CompletionItem::new_simple("font-size".to_string(), "font-size: $1;$0".to_string()),
    ]
}

/// SQL specific completion items
fn get_sql_completion_items() -> Vec<CompletionItem> {
    vec![
        CompletionItem::new_simple("SELECT".to_string(), "SELECT $1 FROM $2 WHERE $3;$0".to_string()),
        CompletionItem::new_simple("INSERT".to_string(), "INSERT INTO $1 ($2) VALUES ($3);$0".to_string()),
        CompletionItem::new_simple("UPDATE".to_string(), "UPDATE $1 SET $2 WHERE $3;$0".to_string()),
        CompletionItem::new_simple("DELETE".to_string(), "DELETE FROM $1 WHERE $2;$0".to_string()),
        CompletionItem::new_simple("CREATE TABLE".to_string(), "CREATE TABLE $1 (\n    id INT PRIMARY KEY,\n    $2\n);$0".to_string()),
    ]
}

// Common server implementation
macro_rules! impl_web_language_server {
    ($name:ident, $kind:expr, $exts:expr, $langs:expr) => {
        #[derive(Debug)]
        pub struct $name {
            config: LanguageServerConfig,
            client_capabilities: ClientCapabilities,
            server_capabilities: Option<ServerCapabilities>,
            initialized: bool,
        }

        impl $name {
            pub fn new(config: LanguageServerConfig) -> Self {
                Self {
                    config,
                    client_capabilities: ClientCapabilities::default(),
                    server_capabilities: None,
                    initialized: false,
                }
            }
        }

        #[async_trait::async_trait]
        impl GenericLspServer for $name {
            fn language_kind(&self) -> &LanguageServerKind {
                &$kind
            }

            fn client_capabilities(&self) -> &ClientCapabilities {
                &self.client_capabilities
            }

            fn supported_extensions(&self) -> &[String] {
                $exts
            }

            fn supports_language(&self, language_id: &str, _file_path: Option<&str>) -> bool {
                $langs.contains(&language_id.to_lowercase().as_str())
            }

            fn supports_request(&self, method: &str) -> bool {
                matches!(
                    method,
                    "textDocument/completion"
                        | "textDocument/hover"
                        | "textDocument/formatting"
                        | "textDocument/documentSymbol"
                        | "textDocument/definition"
                        | "textDocument/references"
                        | "textDocument/rename"
                        | "textDocument/codeAction"
                        | "textDocument/codeLens"
                        | "textDocument/documentHighlight"
                        | "textDocument/colorPresentation"
                        | "textDocument/documentColor"
                )
            }

            async fn initialize(
                &mut self,
                options: &LanguageInitializationOptions,
            ) -> Result<InitializeResult, LSPError> {
                self.client_capabilities = options.client_capabilities.clone();
                self.initialized = true;

                let capabilities = ServerCapabilities {
                    document_formatting_provider: Some(lsp_types::OneOf::Left(true)),
                    completion_provider: Some(CompletionOptions {
                        resolve_provider: Some(true),
                        trigger_characters: Some(vec![" ".to_string(), ".".to_string()]),
                        ..Default::default()
                    }),
                    hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
                    document_symbol_provider: Some(lsp_types::OneOf::Left(true)),
                    ..Default::default()
                };

                self.server_capabilities = Some(capabilities.clone());
                Ok(InitializeResult {
                    capabilities,
                    server_info: None,
                })
            }

            async fn shutdown(&mut self) -> Result<(), LSPError> {
                self.initialized = false;
                Ok(())
            }

            async fn send_request(
                &self,
                method: &str,
                params: Value,
            ) -> Result<Value, LSPError> {
                match method {
                    "textDocument/completion" => {
                        let _params: CompletionParams = serde_json::from_value(params)?;
                        let items = match self.language_kind() {
                            LanguageServerKind::Html => get_html_completion_items(),
                            LanguageServerKind::Css => get_css_completion_items(),
                            LanguageServerKind::Sql => get_sql_completion_items(),
                            _ => vec![],
                        };
                        Ok(serde_json::to_value(CompletionResponse::Array(items))?)
                    }
                    "textDocument/hover" => {
                        let _params: TextDocumentPositionParams = serde_json::from_value(params)?;
                        let contents = lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                            kind: lsp_types::MarkupKind::Markdown,
                            value: "Documentation for symbol".to_string(),
                        });
                        Ok(serde_json::to_value(Hover {
                            contents,
                            range: None,
                        })?)
                    }
                    "textDocument/formatting" => {
                        let _params: DocumentFormattingParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(Vec::<TextEdit>::new())?)
                    }
                    "textDocument/documentSymbol" => {
                        let _params: DocumentSymbolParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(Vec::<DocumentSymbol>::new())?)
                    }
                    "textDocument/definition" => {
                        let _params: TextDocumentPositionParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(None::<Location>)?)
                    }
                    "textDocument/references" => {
                        let _params: ReferenceParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(Vec::<Location>::new())?)
                    }
                    "textDocument/rename" => {
                        let _params: RenameParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(WorkspaceEdit::default())?)
                    }
                    "textDocument/codeAction" => {
                        let _params: CodeActionParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(CodeActionResponse::default())?)
                    }
                    "textDocument/codeLens" => {
                        let _params: CodeLensParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(Vec::<CodeLens>::new())?)
                    }
                    "textDocument/documentHighlight" => {
                        let _params: DocumentHighlightParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(Vec::<DocumentHighlight>::new())?)
                    }
                    "textDocument/colorPresentation" => {
                        let _params: ColorPresentationParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(Vec::<ColorPresentation>::new())?)
                    }
                    "textDocument/documentColor" => {
                        let _params: DocumentColorParams = serde_json::from_value(params)?;
                        Ok(serde_json::to_value(Vec::<ColorInformation>::new())?)
                    }
                    _ => Err(LSPError::MethodNotSupported(method.to_string())),
                }
            }

            async fn send_notification(&self, _method: &str, _params: Value) -> Result<(), LSPError> {
                Ok(())
            }

            fn is_initialized(&self) -> bool {
                self.initialized
            }

            fn server_capabilities(&self) -> Option<&ServerCapabilities> {
                self.server_capabilities.as_ref()
            }

            fn get_initialization_options(&self, _root_path: &PathBuf) -> Value {
                match self.language_kind() {
                    LanguageServerKind::Html => json!({
                        "html": {
                            "suggest": {},
                            "format": {},
                            "hover": {
                                "documentation": true,
                                "references": true
                            }
                        }
                    }),
                    LanguageServerKind::Css => json!({
                        "css": {
                            "validate": true,
                            "lint": {},
                            "completion": {
                                "triggerPropertyValueCompletion": true,
                                "completePropertyWithSemicolon": true
                            }
                        },
                        "scss": {
                            "validate": true,
                            "lint": {}
                        },
                        "less": {
                            "validate": true,
                            "lint": {}
                        }
                    }),
                    LanguageServerKind::Sql => json!({
                        "sql": {
                            "format": {
                                "indentSize": 4,
                                "useTabs": false,
                                "keywordCase": "upper",
                                "identifierCase": "lower"
                            },
                            "suggest": {
                                "keywords": true,
                                "snippets": true
                            }
                        }
                    }),
                    _ => json!({}),
                }
            }
        }
    };
}

// HTML Language Server
impl_web_language_server!(
    HtmlLanguageServer,
    LanguageServerKind::Html,
    &["html".to_string(), "htm".to_string(), "xhtml".to_string()],
    ["html", "xhtml", "vue-html", "php", "erb"]
);

// CSS Language Server
impl_web_language_server!(
    CssLanguageServer,
    LanguageServerKind::Css,
    &["css".to_string(), "scss".to_string(), "sass".to_string(), "less".to_string()],
    ["css", "scss", "sass", "less", "postcss"]
);

// SQL Language Server
impl_web_language_server!(
    SqlLanguageServer,
    LanguageServerKind::Sql,
    &[
        "sql".to_string(),
        "mysql".to_string(),
        "pgsql".to_string(),
        "postgres".to_string(),
        "psql".to_string()
    ],
    ["sql", "mysql", "postgresql", "postgres", "pgsql", "psql"]
);
