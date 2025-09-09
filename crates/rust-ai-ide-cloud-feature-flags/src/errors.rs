use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeatureFlagError {
    #[error("Feature flag parsing error: {0}")]
    ParseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Feature flag not found: {0}")]
    NotFoundError(String),
}
