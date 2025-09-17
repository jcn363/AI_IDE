use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, Method, Response};
// Re-export unified RustAIError with convenient alias
pub use rust_ai_ide_errors::{IDEResult as HttpResult, RustAIError as HttpError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::time::timeout;

// Bridge for legacy HttpError compatibility during migration
pub type HttpErrorLegacy = rust_ai_ide_errors::RustAIError;

/// HTTP request builder with type safety
#[derive(Debug, Clone)]
pub struct HttpRequest<B = ()> {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<B>,
    pub query_params: HashMap<String, String>,
    pub timeout: Option<Duration>,
}

impl<B> HttpRequest<B> {
    /// Create a new GET request
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: Method::GET,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            timeout: None,
        }
    }

    /// Create a new POST request
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            method: Method::POST,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            timeout: None,
        }
    }

    /// Set header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set body
    pub fn body<B2>(self, body: B2) -> HttpRequest<B2> {
        HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: Some(body),
            query_params: self.query_params,
            timeout: self.timeout,
        }
    }

    /// Add query parameter
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    /// Set timeout
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
}

/// HttpResponse for type-safe response handling
#[derive(Debug)]
pub struct HttpResponse<T> {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: T,
}

/// Cache entry for HTTP responses
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub response: serde_json::Value,
    pub cached_at: std::time::SystemTime,
    pub ttl_seconds: u64,
}

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub base_url: Option<String>,
    pub user_agent: Option<String>,
    pub default_timeout: Duration,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub retry_max_delay_ms: u64,
    pub auth_token: Option<String>,
    pub enable_cache: bool,
    pub cache_ttl_seconds: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            user_agent: Some("RustAIIDECore/1.0".to_string()),
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay_ms: 1000,
            retry_max_delay_ms: 30000,
            auth_token: None,
            enable_cache: false,
            cache_ttl_seconds: 300, // 5 minutes
        }
    }
}

/// HttpClient with comprehensive features: caching, retry, auth, middleware
pub struct HttpClient {
    client: Client,
    config: HttpConfig,
    request_interceptors: Vec<RequestInterceptorEnum>,
    response_interceptors: Vec<ResponseInterceptorEnum>,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    call_count: std::sync::atomic::AtomicU64,
}

/// Request interceptor trait
#[async_trait::async_trait]
pub trait RequestInterceptor: Send + Sync {
    async fn intercept(
        &self,
        request: &mut HttpRequest<serde_json::Value>,
    ) -> Result<(), HttpError>;
}

/// Response interceptor trait
#[async_trait::async_trait]
pub trait ResponseInterceptor: Send + Sync {
    async fn intercept(
        &self,
        response: &mut HttpResponse<serde_json::Value>,
    ) -> Result<(), HttpError>;
}

/// Enum-based polymorphism for RequestInterceptor
pub enum RequestInterceptorEnum {
    Logging(LoggingInterceptor),
}

#[async_trait::async_trait]
impl RequestInterceptor for RequestInterceptorEnum {
    async fn intercept(
        &self,
        request: &mut HttpRequest<serde_json::Value>,
    ) -> Result<(), HttpError> {
        match self {
            RequestInterceptorEnum::Logging(interceptor) => {
                RequestInterceptor::intercept(interceptor, request).await
            }
        }
    }
}

/// Enum-based polymorphism for ResponseInterceptor
pub enum ResponseInterceptorEnum {
    Logging(LoggingInterceptor),
}

#[async_trait::async_trait]
impl ResponseInterceptor for ResponseInterceptorEnum {
    async fn intercept(
        &self,
        response: &mut HttpResponse<serde_json::Value>,
    ) -> Result<(), HttpError> {
        match self {
            ResponseInterceptorEnum::Logging(interceptor) => {
                ResponseInterceptor::intercept(interceptor, response).await
            }
        }
    }
}

/// Builder for HttpClient
pub struct HttpClientBuilder {
    config: HttpConfig,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        Self {
            config: HttpConfig::default(),
        }
    }

    pub fn config(mut self, config: HttpConfig) -> Self {
        self.config = config;
        self
    }

    pub fn base_url<S: Into<String>>(mut self, url: S) -> Self {
        self.config.base_url = Some(url.into());
        self
    }

    pub fn user_agent<S: Into<String>>(mut self, ua: S) -> Self {
        self.config.user_agent = Some(ua.into());
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.config.default_timeout = duration;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    pub fn auth_token<S: Into<String>>(mut self, token: S) -> Self {
        self.config.auth_token = Some(token.into());
        self
    }

    pub fn enable_cache(mut self, ttl_seconds: u64) -> Self {
        self.config.enable_cache = true;
        self.config.cache_ttl_seconds = ttl_seconds;
        self
    }

    pub async fn build(self) -> Result<HttpClient, HttpError> {
        let mut client_builder = Client::builder()
            .timeout(self.config.default_timeout)
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10);

        if let Some(ua) = &self.config.user_agent {
            client_builder = client_builder.user_agent(ua);
        }

        let client = client_builder
            .build()
            .map_err(|e| rust_ai_ide_errors::RustAIError::Network(e.to_string()))?;

        Ok(HttpClient {
            client,
            config: self.config,
            request_interceptors: Vec::new(),
            response_interceptors: Vec::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            call_count: std::sync::atomic::AtomicU64::new(0),
        })
    }
}

impl HttpClient {
    /// Create a new HttpClient with default configuration
    pub async fn new() -> Result<Self, HttpError> {
        HttpClientBuilder::new().build().await
    }

    /// Add a request interceptor
    pub fn add_request_interceptor(&mut self, interceptor: RequestInterceptorEnum) {
        self.request_interceptors.push(interceptor);
    }

    /// Add a response interceptor
    pub fn add_response_interceptor(&mut self, interceptor: ResponseInterceptorEnum) {
        self.response_interceptors.push(interceptor);
    }

    /// Check if response is cached
    async fn check_cache(&self, cache_key: &str) -> Option<HttpResponse<serde_json::Value>> {
        if !self.config.enable_cache {
            return None;
        }

        if let Some(entry) = self.cache.lock().await.get(cache_key) {
            if !entry.is_expired() {
                log::debug!("Cache hit for key: {}", cache_key);
                return Some(HttpResponse {
                    status: 200, // Cached responses always treated as successful
                    headers: HashMap::new(),
                    body: entry.response.clone(),
                });
            }
        }
        None
    }

    /// Store response in cache
    async fn store_cache(&self, cache_key: String, response: &HttpResponse<serde_json::Value>) {
        if !self.config.enable_cache {
            return;
        }

        let entry = CacheEntry {
            response: response.body.clone(),
            cached_at: std::time::SystemTime::now(),
            ttl_seconds: self.config.cache_ttl_seconds,
        };

        self.cache.lock().await.insert(cache_key, entry);
    }

    /// Execute HTTP request with full pipeline: interceptors -> cache -> retry
    pub async fn execute<B, T>(&self, request: HttpRequest<B>) -> Result<HttpResponse<T>, HttpError>
    where
        B: Serialize + Send + Sync,
        T: DeserializeOwned + Serialize,
    {
        // Apply request interceptors
        let mut json_request = HttpRequest {
            method: request.method,
            url: request.url,
            headers: request.headers,
            body: Some(
                request
                    .body
                    .map(|b| serde_json::to_value(b))
                    .transpose()
                    .map_err(|e| HttpError::Validation(format!("JSON serialization error: {}", e)))?
                    .unwrap_or(serde_json::Value::Null),
            ),
            query_params: request.query_params,
            timeout: request.timeout,
        };

        for interceptor in &self.request_interceptors {
            interceptor.intercept(&mut json_request).await?;
        }

        // Generate cache key
        let cache_key = self.generate_cache_key(&json_request);

        // Check cache
        if let Some(cached) = self.check_cache(&cache_key).await {
            return Ok(HttpResponse {
                status: cached.status,
                headers: cached.headers,
                body: serde_json::from_value(cached.body)
                    .map_err(|e| HttpError::Validation(format!("JSON parse error: {}", e)))?,
            });
        }

        // Execute with retry
        let result = self.execute_with_retry(json_request).await?;

        // Extract metadata before consuming result
        let status_code = result.status().as_u16();
        let headers = result
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // Parse response
        let response_body: T = result
            .json()
            .await
            .map_err(|e| HttpError::Validation(format!("Response body parse error: {}", e)))?;

        let http_response = HttpResponse {
            status: status_code,
            headers,
            body: response_body,
        };

        // Apply response interceptors
        let mut json_response = HttpResponse {
            status: http_response.status,
            headers: http_response.headers.clone(),
            body: serde_json::to_value(&http_response.body)
                .map_err(|e| HttpError::Protocol(format!("Response serialization error: {}", e)))?,
        };

        for interceptor in &self.response_interceptors {
            interceptor.intercept(&mut json_response).await?;
        }

        // Store in cache
        if self.config.enable_cache {
            self.store_cache(cache_key, &json_response).await;
        }

        Ok(http_response)
    }

    /// Generate cache key from request
    fn generate_cache_key(&self, request: &HttpRequest<serde_json::Value>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.method.hash(&mut hasher);
        request.url.hash(&mut hasher);
        for (k, v) in &request.query_params {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    /// Execute request with retry logic
    async fn execute_with_retry(
        &self,
        request: HttpRequest<serde_json::Value>,
    ) -> Result<Response, HttpError> {
        let call_id = self
            .call_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        for attempt in 0..=self.config.max_retries {
            log::debug!(
                "HTTP Request #{} [attempt {}]: {} {}",
                call_id,
                attempt + 1,
                request.method,
                request.url
            );

            match self.execute_single(&request).await {
                Ok(response) => {
                    if response.status().is_success() {
                        log::debug!(
                            "HTTP Response #{}: {} {}",
                            call_id,
                            response.status(),
                            response.url()
                        );
                        return Ok(response);
                    } else if response.status().is_server_error()
                        && attempt < self.config.max_retries
                    {
                        log::warn!(
                            "HTTP Error #{}: {} {} - retrying",
                            call_id,
                            response.status(),
                            response.status().canonical_reason().unwrap_or("Unknown")
                        );
                        self.delay_before_retry(attempt).await;
                        continue;
                    } else {
                        let status = response.status().as_u16() as u32;
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        return Err(HttpError::Network(format!("HTTP {}: {}", status, message)));
                    }
                }
                Err(HttpError::Network(e)) if attempt < self.config.max_retries => {
                    log::warn!("Network error #{}: {} - retrying", call_id, e);
                    self.delay_before_retry(attempt).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(HttpError::Network(
            "All retry attempts exhausted".to_string(),
        ))
    }

    /// Delay before retry with exponential backoff
    async fn delay_before_retry(&self, attempt: u32) {
        let delay = std::cmp::min(
            self.config.retry_delay_ms * (2u64.pow(attempt)),
            self.config.retry_max_delay_ms,
        );
        tokio::time::sleep(Duration::from_millis(delay)).await;
    }

    /// Execute single HTTP request
    async fn execute_single(
        &self,
        request: &HttpRequest<serde_json::Value>,
    ) -> Result<Response, HttpError> {
        // Build final URL with base_url if configured
        let final_url = if let Some(base_url) = &self.config.base_url {
            reqwest::Url::parse(base_url)
                .and_then(|base| base.join(&request.url))
                .map_err(|e| HttpError::Protocol(format!("URL parse error: {}", e)))?
        } else {
            reqwest::Url::parse(&request.url)
                .map_err(|e| HttpError::Protocol(format!("URL parse error: {}", e)))?
        };

        let mut req_builder = self.client.request(request.method.clone(), final_url);

        // Add headers
        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        // Add auth token if configured
        if let Some(token) = &self.config.auth_token {
            req_builder = req_builder.bearer_auth(token);
        }

        // Add query parameters
        for (key, value) in &request.query_params {
            req_builder = req_builder.query(&[(key, value)]);
        }

        // Add body if present
        if let Some(body) = &request.body {
            if !body.is_null() {
                req_builder = req_builder.json(body);
            }
        }

        // Set timeout override
        if let Some(timeout_duration) = request.timeout {
            req_builder = req_builder.timeout(timeout_duration);
        }

        // Execute request with timeout
        let request_timeout = request.timeout.unwrap_or(self.config.default_timeout);
        timeout(request_timeout, req_builder.send())
            .await
            .map_err(|_| HttpError::Timeout("Request timed out".to_string()))?
            .map_err(|e| HttpError::Network(format!("Network error: {}", e)))
    }

    /// Convenience method for GET requests
    pub async fn get<T>(&self, url: impl Into<String>) -> Result<HttpResponse<T>, HttpError>
    where
        T: DeserializeOwned + Serialize,
    {
        self.execute(HttpRequest::<()>::get(url)).await
    }

    /// Convenience method for POST requests
    pub async fn post<B, T>(
        &self,
        url: impl Into<String>,
        body: B,
    ) -> Result<HttpResponse<T>, HttpError>
    where
        B: Serialize + Send + Sync,
        T: DeserializeOwned + Serialize,
    {
        self.execute(HttpRequest::<B>::post(url).body(body)).await
    }

    /// Statistics about client usage
    pub fn stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert(
            "total_calls".to_string(),
            self.call_count.load(std::sync::atomic::Ordering::Relaxed),
        );
        stats
    }
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed().unwrap_or_default().as_secs() > self.ttl_seconds
    }
}

/// Logging interceptor for HTTP requests
#[derive(Clone)]
pub struct LoggingInterceptor {
    pub request_log_level: log::Level,
    pub response_log_level: log::Level,
}

impl Default for LoggingInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingInterceptor {
    pub fn new() -> Self {
        Self {
            request_log_level: log::Level::Info,
            response_log_level: log::Level::Info,
        }
    }
}

#[async_trait::async_trait]
impl RequestInterceptor for LoggingInterceptor {
    async fn intercept(
        &self,
        request: &mut HttpRequest<serde_json::Value>,
    ) -> Result<(), HttpError> {
        log::log!(
            self.request_log_level,
            "HTTP Request: {} {}",
            request.method,
            request.url
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl ResponseInterceptor for LoggingInterceptor {
    async fn intercept(
        &self,
        response: &mut HttpResponse<serde_json::Value>,
    ) -> Result<(), HttpError> {
        log::log!(
            self.response_log_level,
            "HTTP Response: {}",
            response.status
        );
        Ok(())
    }
}
