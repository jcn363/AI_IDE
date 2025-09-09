//! Shared Configuration System
//! ============================
//!
//! This module provides a unified configuration system for AI services,
//! model loading, and inference settings across the entire Rust AI IDE workspace.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::ai::{AIProvider, ModelConfig, AnalysisConfig};
use super::error::{IDEResult, IDEError};
use rust_ai_ide_common::config::Config;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Core IDE configuration
    pub core: CoreConfig,
    /// AI systems configuration
    pub ai: AIConfig,
    /// Analysis configuration
    pub analysis: AnalysisConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Caching configuration
    pub cache: CacheConfig,
    /// Networking configuration
    pub network: NetworkConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Additional custom configurations
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            ai: AIConfig::default(),
            analysis: AnalysisConfig::default(),
            performance: PerformanceConfig::default(),
            cache: CacheConfig::default(),
            network: NetworkConfig::default(),
            logging: LoggingConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Config for AppConfig {
    const FILE_PREFIX: &'static str = "rust_ai_ide";
    const DESCRIPTION: &'static str = "Rust AI IDE Application Configuration";

    fn validate(&self) -> Result<Vec<String>, anyhow::Error> {
        self.validate_compat()
    }

    fn default_config() -> Self {
        Self::default()
    }

    fn schema() -> Option<serde_json::Value> {
        // Could implement JSON schema generation here
        None
    }
}

impl AppConfig {
    /// Load configuration from a file
    pub async fn from_file(path: PathBuf) -> IDEResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        // Extract extension before moving path
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());

        let contents = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| IDEError::from(e))?;

        match extension.as_ref().map(|s| s.as_str()) {
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str(&contents)
                    .map_err(|e| IDEError::Parse {
                        message: format!("YAML parsing error: {}", e)
                    })
            }
            Some("toml") => {
                toml::from_str(&contents)
                    .map_err(|e| IDEError::Parse {
                        message: format!("TOML parsing error: {}", e)
                    })
            }
            Some("json") => {
                serde_json::from_str(&contents)
                    .map_err(|e| IDEError::Parse {
                        message: format!("JSON parsing error: {}", e)
                    })
            }
            _ => {
                // Try JSON first, then TOML
                match serde_json::from_str(&contents) {
                    Ok(config) => Ok(config),
                    Err(_) => match toml::from_str(&contents) {
                        Ok(config) => Ok(config),
                        Err(_) => Err(IDEError::Parse {
                            message: "Unsupported configuration format".to_string()
                        })
                    }
                }
            }
        }
    }

    /// Save configuration to a file
    pub async fn save_to_file(&self, path: PathBuf, format: ConfigFormat) -> IDEResult<()> {
        let contents = match format {
            ConfigFormat::Yaml => serde_yaml::to_string(self)
                .map_err(|e| IDEError::Generic {
                    message: format!("YAML serialization error: {}", e)
                })?,
            ConfigFormat::Toml => toml::to_string(self)
                .map_err(|e| IDEError::Generic {
                    message: format!("TOML serialization error: {}", e)
                })?,
            ConfigFormat::Json => serde_json::to_string_pretty(self)
                .map_err(|e| IDEError::Generic {
                    message: format!("JSON serialization error: {}", e)
                })?,
        };

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| IDEError::from(e))?;
        }

        tokio::fs::write(path, contents)
            .await
            .map_err(|e| IDEError::from(e))
    }

    /// Validate the configuration
    pub fn validate(&self) -> IDEResult<()> {
        self.ai.validate()?;
        self.performance.validate()?;
        self.cache.validate()?;
        self.network.validate()?;
        self.logging.validate()?;
        Ok(())
    }

    /// Validate the configuration (Config trait implementation)
    fn validate_compat(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut errors = Vec::new();
        if let Err(e) = self.ai.validate() {
            errors.push(format!("AI config error: {}", e));
        }
        if let Err(e) = self.performance.validate() {
            errors.push(format!("Performance config error: {}", e));
        }
        if let Err(e) = self.cache.validate() {
            errors.push(format!("Cache config error: {}", e));
        }
        if let Err(e) = self.network.validate() {
            errors.push(format!("Network config error: {:?}", e));
        }
        if let Err(e) = self.logging.validate() {
            errors.push(format!("Logging config error: {:?}", e));
        }
        Ok(errors)
    }
}

/// Configuration file format
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Yaml,
    Toml,
    Json,
}

/// Core IDE configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Application name
    pub app_name: String,
    /// Application version
    pub app_version: String,
    /// Data directory
    pub data_directory: PathBuf,
    /// Config directory
    pub config_directory: PathBuf,
    /// Cache directory
    pub cache_directory: PathBuf,
    /// Log directory
    pub log_directory: PathBuf,
    /// Editor theme
    pub theme: String,
    /// Font preferences
    pub fonts: FontConfig,
    /// Editor settings
    pub editor: EditorConfig,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            app_name: "Rust AI IDE".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            data_directory: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("./data"))
                .join("rust-ai-ide"),
            config_directory: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("./config"))
                .join("rust-ai-ide"),
            cache_directory: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("./cache"))
                .join("rust-ai-ide"),
            log_directory: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("./logs")),
            theme: "dark".to_string(),
            fonts: FontConfig::default(),
            editor: EditorConfig::default(),
        }
    }
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Editor font family
    pub editor_font_family: String,
    /// Editor font size
    pub editor_font_size: f32,
    /// UI font family
    pub ui_font_family: String,
    /// UI font size
    pub ui_font_size: f32,
    /// Monospace font family
    pub monospace_font_family: String,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            editor_font_family: "JetBrains Mono".to_string(),
            editor_font_size: 14.0,
            ui_font_family: "System".to_string(),
            ui_font_size: 13.0,
            monospace_font_family: "JetBrains Mono".to_string(),
        }
    }
}

/// Editor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Tab size
    pub tab_size: u32,
    /// Insert spaces instead of tabs
    pub insert_spaces: bool,
    /// Word wrap
    pub word_wrap: bool,
    /// Minimap enabled
    pub minimap: bool,
    /// Line numbers
    pub line_numbers: bool,
    /// Automatic save delay (seconds)
    pub auto_save_delay: Option<u32>,
    /// Bracket matching
    pub bracket_matching: bool,
    /// Highlight current line
    pub highlight_current_line: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            insert_spaces: true,
            word_wrap: true,
            minimap: true,
            line_numbers: true,
            auto_save_delay: Some(60),
            bracket_matching: true,
            highlight_current_line: true,
        }
    }
}

/// AI systems configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Default AI provider
    pub default_provider: AIProvider,
    /// Available providers
    pub providers: Vec<AIProvider>,
    /// Active models
    pub active_models: Vec<ModelConfig>,
    /// API endpoints for remote providers
    pub endpoints: HashMap<AIProvider, String>,
    /// API keys (encrypted or environment-based)
    pub api_keys: HashMap<String, String>,
    /// Model management settings
    pub model_management: ModelManagementConfig,
    /// Inference settings
    pub inference: InferenceConfig,
}

impl Default for AIConfig {
    fn default() -> Self {
        let mut endpoints = HashMap::new();
        endpoints.insert(AIProvider::Mock, "http://localhost:8080".to_string());

        Self {
            default_provider: AIProvider::Mock,
            providers: vec![AIProvider::Mock],
            active_models: Vec::new(),
            endpoints,
            api_keys: HashMap::new(),
            model_management: ModelManagementConfig::default(),
            inference: InferenceConfig::default(),
        }
    }
}

impl AIConfig {
    pub fn validate(&self) -> IDEResult<()> {
        // Validate that default provider is in providers list
        if !self.providers.contains(&self.default_provider) {
            return Err(IDEError::Config {
                message: format!("Default provider {:?} not in providers list", self.default_provider)
            });
        }

        // Validate endpoints for providers
        for provider in &self.providers {
            match provider {
                AIProvider::OpenAI | AIProvider::Anthropic | AIProvider::Local { .. } => {
                    if !self.endpoints.contains_key(provider) {
                        return Err(IDEError::Config {
                            message: format!("Missing endpoint for provider {:?}", provider)
                        });
                    }
                }
                _ => {} // Mock provider doesn't need endpoint
            }
        }

        Ok(())
    }
}

/// Model management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManagementConfig {
    /// Maximum memory usage for models (MB)
    pub max_memory_mb: u64,
    /// Maximum number of concurrent models
    pub max_concurrent_models: usize,
    /// Model unloading policy
    pub unloading_policy: UnloadingPolicy,
    /// Enable model caching
    pub enable_caching: bool,
    /// Cache directory
    pub cache_directory: Option<PathBuf>,
}

impl Default for ModelManagementConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 2048,
            max_concurrent_models: 3,
            unloading_policy: UnloadingPolicy::LeastRecentlyUsed,
            enable_caching: true,
            cache_directory: Some(PathBuf::from("./model-cache")),
        }
    }
}

/// Model unloading policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnloadingPolicy {
    /// Never unload
    Never,
    /// Least recently used
    LeastRecentlyUsed,
    /// Least frequently used
    LeastFrequentlyUsed,
    /// Memory pressure based
    MemoryPressure,
}

/// Inference configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Default temperature
    pub default_temperature: f32,
    /// Default max tokens
    pub default_max_tokens: u32,
    /// Default top-p
    pub default_top_p: f32,
    /// Default top-k
    pub default_top_k: u32,
    /// Request timeout (seconds)
    pub request_timeout_seconds: u32,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Rate limiting
    pub rate_limiting: RateLimitConfig,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            default_temperature: 0.7,
            default_max_tokens: 2048,
            default_top_p: 0.9,
            default_top_k: 40,
            request_timeout_seconds: 300,
            retry: RetryConfig::default(),
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum attempts
    pub max_attempts: u32,
    /// Base delay between retries (milliseconds)
    pub base_delay_ms: u64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Burst limit
    pub burst_limit: u32,
    /// Backoff time on rate limit hit (milliseconds)
    pub backoff_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_limit: 10,
            backoff_ms: 60000,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum threads for analysis
    pub max_analysis_threads: usize,
    /// Maximum memory usage (MB)
    pub max_memory_mb: u64,
    /// Concurrency limit for AI operations
    pub ai_concurrency_limit: usize,
    /// IO thread pool size
    pub io_thread_pool_size: usize,
    /// Cache sizes
    pub cache_sizes: CacheSizesConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_analysis_threads: num_cpus::get(),
            max_memory_mb: 4096,
            ai_concurrency_limit: 4,
            io_thread_pool_size: num_cpus::get(),
            cache_sizes: CacheSizesConfig::default(),
        }
    }
}

impl PerformanceConfig {
    pub fn validate(&self) -> IDEResult<()> {
        if self.max_analysis_threads == 0 {
            return Err(IDEError::Config {
                message: "Max analysis threads must be > 0".to_string()
            });
        }

        if self.max_memory_mb < 256 {
            return Err(IDEError::Config {
                message: "Max memory must be at least 256 MB".to_string()
            });
        }

        Ok(())
    }
}

/// Cache sizes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSizesConfig {
    /// Analysis result cache size (entries)
    pub analysis_cache_size: usize,
    /// Model cache size (entries)
    pub model_cache_size: usize,
    /// LSP cache size (entries)
    pub lsp_cache_size: usize,
    /// File cache size (MB)
    pub file_cache_size_mb: usize,
}

impl Default for CacheSizesConfig {
    fn default() -> Self {
        Self {
            analysis_cache_size: 1000,
            model_cache_size: 100,
            lsp_cache_size: 500,
            file_cache_size_mb: 500,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Cache directory
    pub directory: PathBuf,
    /// Maximum cache size (MB)
    pub max_size_mb: u64,
    /// Cache compression
    pub compression: CompressionConfig,
    /// TTL for cache entries (seconds)
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("./cache"))
            .join("rust-ai-ide");

        Self {
            enabled: true,
            directory: dir,
            max_size_mb: 1024,
            compression: CompressionConfig::default(),
            ttl_seconds: 3600, // 1 hour
        }
    }
}

impl CacheConfig {
    pub fn validate(&self) -> IDEResult<()> {
        if self.enabled && self.max_size_mb < 64 {
            return Err(IDEError::Config {
                message: "Minimum cache size is 64 MB".to_string()
            });
        }

        Ok(())
    }
}

/// Cache compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
    /// Minimum size to compress (bytes)
    pub min_size_bytes: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::Gzip,
            min_size_bytes: 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Bz2,
    Xz,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// HTTP client timeout (seconds)
    pub timeout_seconds: u64,
    /// Maximum response size (MB)
    pub max_response_size_mb: u64,
    /// SSL/TLS configuration
    pub tls: TlsConfig,
    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
    /// Connection pool settings
    pub connection_pool: ConnectionPoolConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_response_size_mb: 100,
            tls: TlsConfig::default(),
            proxy: None,
            connection_pool: ConnectionPoolConfig::default(),
        }
    }
}

impl NetworkConfig {
    pub fn validate(&self) -> IDEResult<()> {
        if self.timeout_seconds < 5 {
            return Err(IDEError::Config {
                message: "Network timeout must be at least 5 seconds".to_string()
            });
        }

        Ok(())
    }
}

/// SSL/TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable SSL/TLS verification
    pub verify_ssl: bool,
    /// Certificate file path
    pub ca_cert_file: Option<PathBuf>,
    /// Client certificate file path
    pub client_cert_file: Option<PathBuf>,
    /// Client key file path
    pub client_key_file: Option<PathBuf>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify_ssl: true,
            ca_cert_file: None,
            client_cert_file: None,
            client_key_file: None,
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy URL
    pub url: String,
    /// Proxy username
    pub username: Option<String>,
    /// Proxy password
    pub password: Option<String>,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum connections per host
    pub max_connections_per_host: usize,
    /// Maximum idle connections
    pub max_idle_connections: usize,
    /// Connection timeout (seconds)
    pub connect_timeout_seconds: u64,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            max_idle_connections: 5,
            connect_timeout_seconds: 10,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Minimum log level
    pub level: LogLevel,
    /// Log format
    pub format: LogFormat,
    /// Output destinations
    pub outputs: Vec<LogOutput>,
    /// Maximum log file size (MB)
    pub max_log_file_size_mb: u64,
    /// Maximum number of log files to keep
    pub max_log_files: usize,
    /// Enable log compression
    pub compress_logs: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            outputs: vec![LogOutput::Stdout],
            max_log_file_size_mb: 100,
            max_log_files: 10,
            compress_logs: true,
        }
    }
}

impl LoggingConfig {
    pub fn validate(&self) -> IDEResult<()> {
        if self.outputs.is_empty() {
            return Err(IDEError::Config {
                message: "At least one log output must be configured".to_string()
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
    Pretty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(PathBuf),
}