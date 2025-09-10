//! Factory for creating web language servers (HTML, CSS, SQL)

use std::path::PathBuf;
use std::sync::Arc;

use lsp_types::ClientCapabilities;
use serde_json::Value;

use crate::client::LSPError;
use crate::language_server::{
    GenericLspServer, LanguageServerConfig, LanguageServerFactory, LanguageServerKind,
};

use super::web_language_servers::{CssLanguageServer, HtmlLanguageServer, SqlLanguageServer};

/// Factory for creating web language servers
#[derive(Debug, Default)]
pub struct WebLanguageServerFactory {
    /// Cache for created servers
    server_cache: std::sync::RwLock<std::collections::HashMap<LanguageServerKind, Arc<dyn GenericLspServer>>>,
}

impl WebLanguageServerFactory {
    /// Create a new web language server factory
    pub fn new() -> Self {
        Self {
            server_cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl LanguageServerFactory for WebLanguageServerFactory {
    async fn create_server(
        &self,
        config: &LanguageServerConfig,
        _root_path: Option<PathBuf>,
    ) -> Result<Box<dyn GenericLspServer>, LSPError> {
        let server: Box<dyn GenericLspServer> = match config.language {
            LanguageServerKind::Html => Box::new(HtmlLanguageServer::new(config.clone())),
            LanguageServerKind::Css => Box::new(CssLanguageServer::new(config.clone())),
            LanguageServerKind::Sql => Box::new(SqlLanguageServer::new(config.clone())),
            _ => return Err(LSPError::UnsupportedLanguage(config.language.clone())),
        };

        // Cache the server if not already cached
        let mut cache = self.server_cache.write().unwrap();
        cache.insert(config.language.clone(), Arc::new(Box::new(server.as_ref()) as Box<dyn GenericLspServer>));

        Ok(server)
    }

    fn supports_language(&self, kind: &LanguageServerKind) -> bool {
        matches!(
            kind,
            LanguageServerKind::Html | LanguageServerKind::Css | LanguageServerKind::Sql
        )
    }

    fn factory_name(&self) -> &'static str {
        "web-language-server-factory"
    }

    fn is_available(&self) -> bool {
        // Web language servers are always available as they're built-in
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_server::LanguageServerConfig;
    use lsp_types::ClientCapabilities;

    #[tokio::test]
    async fn test_create_html_server() {
        let factory = WebLanguageServerFactory::new();
        let config = LanguageServerConfig {
            language: LanguageServerKind::Html,
            server_path: PathBuf::new(),
            args: vec![],
            file_extensions: vec!["html".to_string(), "htm".to_string(), "xhtml".to_string()],
            initialization_options: None,
            client_capabilities: ClientCapabilities::default(),
            supported_requests: vec!["textDocument/completion".to_string()],
            enable_tracing: false,
            max_request_timeout: 5000,
            enable_caching: true,
        };

        let server = factory.create_server(&config, None).await;
        assert!(server.is_ok());
        assert_eq!(
            server.unwrap().language_kind(),
            &LanguageServerKind::Html
        );
    }

    #[tokio::test]
    async fn test_supports_language() {
        let factory = WebLanguageServerFactory::new();
        assert!(factory.supports_language(&LanguageServerKind::Html));
        assert!(factory.supports_language(&LanguageServerKind::Css));
        assert!(factory.supports_language(&LanguageServerKind::Sql));
        assert!(!factory.supports_language(&LanguageServerKind::Rust));
    }
}
