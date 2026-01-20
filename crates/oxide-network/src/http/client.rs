//! HTTP client implementation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

use crate::allowlist::Allowlist;
use crate::auth::AuthManager;
use crate::error::{NetworkError, NetworkResult};
use crate::interceptor::{Interceptor, InterceptorChain};
use crate::offline::OfflineDetector;

use super::{HttpRequest, HttpResponse, RequestBody, RetryConfig};

/// Configuration for the HTTP client.
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Base URL to prepend to relative paths.
    pub base_url: Option<String>,
    /// Default timeout for requests.
    pub default_timeout: Duration,
    /// Default headers to add to all requests.
    pub default_headers: HashMap<String, String>,
    /// User agent string.
    pub user_agent: String,
    /// Whether to enable compression.
    pub compression: bool,
    /// Maximum concurrent requests.
    pub max_concurrent_requests: usize,
    /// Connection pool idle timeout.
    pub pool_idle_timeout: Duration,
    /// Default retry configuration.
    pub default_retry: RetryConfig,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            default_timeout: Duration::from_secs(30),
            default_headers: HashMap::new(),
            user_agent: format!("OxideKit/{}", env!("CARGO_PKG_VERSION")),
            compression: true,
            max_concurrent_requests: 100,
            pool_idle_timeout: Duration::from_secs(90),
            default_retry: RetryConfig::default(),
        }
    }
}

/// Builder for configuring the HTTP client.
#[derive(Debug, Default)]
pub struct HttpClientBuilder {
    config: HttpClientConfig,
    allowlist: Option<Allowlist>,
    auth_manager: Option<Arc<AuthManager>>,
    interceptors: Vec<Arc<dyn Interceptor>>,
    offline_detector: Option<Arc<OfflineDetector>>,
}

impl HttpClientBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = Some(url.into());
        self
    }

    /// Set the default timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }

    /// Add a default header.
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.default_headers.insert(name.into(), value.into());
        self
    }

    /// Set the user agent.
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.config.user_agent = agent.into();
        self
    }

    /// Enable or disable compression.
    pub fn compression(mut self, enabled: bool) -> Self {
        self.config.compression = enabled;
        self
    }

    /// Set the URL allowlist.
    pub fn allowlist(mut self, allowlist: Allowlist) -> Self {
        self.allowlist = Some(allowlist);
        self
    }

    /// Set the auth manager.
    pub fn auth_manager(mut self, manager: Arc<AuthManager>) -> Self {
        self.auth_manager = Some(manager);
        self
    }

    /// Add an interceptor.
    pub fn interceptor(mut self, interceptor: impl Interceptor + 'static) -> Self {
        self.interceptors.push(Arc::new(interceptor));
        self
    }

    /// Set the offline detector.
    pub fn offline_detector(mut self, detector: Arc<OfflineDetector>) -> Self {
        self.offline_detector = Some(detector);
        self
    }

    /// Set the default retry configuration.
    pub fn default_retry(mut self, config: RetryConfig) -> Self {
        self.config.default_retry = config;
        self
    }

    /// Build the HTTP client.
    pub fn build(self) -> NetworkResult<HttpClient> {
        let inner_client = reqwest::Client::builder()
            .timeout(self.config.default_timeout)
            .user_agent(&self.config.user_agent)
            .pool_idle_timeout(self.config.pool_idle_timeout)
            .gzip(self.config.compression)
            .brotli(self.config.compression)
            .build()
            .map_err(NetworkError::HttpError)?;

        Ok(HttpClient {
            inner: inner_client,
            config: self.config,
            allowlist: self.allowlist.map(Arc::new),
            auth_manager: self.auth_manager,
            interceptors: Arc::new(InterceptorChain::new(self.interceptors)),
            offline_detector: self.offline_detector,
            active_requests: Arc::new(RwLock::new(0)),
        })
    }
}

/// HTTP client for making network requests.
///
/// Features:
/// - Request/response interceptors
/// - Automatic retry with exponential backoff
/// - Auth integration
/// - URL allowlist enforcement
/// - Offline detection
#[derive(Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    config: HttpClientConfig,
    allowlist: Option<Arc<Allowlist>>,
    auth_manager: Option<Arc<AuthManager>>,
    interceptors: Arc<InterceptorChain>,
    offline_detector: Option<Arc<OfflineDetector>>,
    active_requests: Arc<RwLock<usize>>,
}

impl HttpClient {
    /// Create a new HTTP client with default configuration.
    pub fn new() -> NetworkResult<Self> {
        HttpClientBuilder::new().build()
    }

    /// Create a builder for configuring the client.
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    /// Execute an HTTP request.
    pub async fn execute(&self, mut request: HttpRequest) -> NetworkResult<HttpResponse> {
        // Check offline status
        if let Some(detector) = &self.offline_detector {
            if detector.is_offline() {
                return Err(NetworkError::Offline);
            }
        }

        // Check allowlist
        if let Some(allowlist) = &self.allowlist {
            if !request.bypass_allowlist && !allowlist.is_allowed(request.url.as_str()) {
                return Err(NetworkError::BlockedByAllowlist {
                    url: request.url.to_string(),
                });
            }
        }

        // Apply base URL if request URL is relative
        if let Some(base_url) = &self.config.base_url {
            if request.url.scheme().is_empty() || !request.url.has_host() {
                let base = url::Url::parse(base_url)?;
                request.url = base.join(request.url.path())?;
            }
        }

        // Add default headers
        for (key, value) in &self.config.default_headers {
            request
                .headers
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }

        // Apply auth if required
        if request.require_auth {
            if let Some(auth_manager) = &self.auth_manager {
                request = auth_manager.apply_auth(request).await?;
            } else {
                warn!("Auth required but no auth manager configured");
            }
        }

        // Run request interceptors
        let request = self.interceptors.intercept_request(request).await?;

        // Execute with retry logic
        let response = self.execute_with_retry(request).await?;

        // Run response interceptors
        let response = self.interceptors.intercept_response(response).await?;

        Ok(response)
    }

    /// Execute a request with retry logic.
    async fn execute_with_retry(&self, request: HttpRequest) -> NetworkResult<HttpResponse> {
        let retry_config = if request.retry_config.max_retries > 0 {
            &request.retry_config
        } else {
            &self.config.default_retry
        };

        let mut _last_error: Option<NetworkError> = None;
        let mut attempt = 0;

        loop {
            match self.execute_single(&request).await {
                Ok(response) => {
                    // Check if we should retry based on status code
                    if retry_config.should_retry_status(response.status)
                        && attempt < retry_config.max_retries
                    {
                        let delay = response
                            .retry_after()
                            .map(Duration::from_secs)
                            .unwrap_or_else(|| retry_config.delay_for_attempt(attempt));

                        warn!(
                            request_id = %request.id,
                            status = response.status,
                            attempt = attempt + 1,
                            delay_ms = delay.as_millis() as u64,
                            "Retrying request due to status code"
                        );

                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }
                    return Ok(response);
                }
                Err(err) => {
                    if !err.is_retryable() || attempt >= retry_config.max_retries {
                        return Err(err);
                    }

                    let delay = err
                        .suggested_retry_delay()
                        .map(Duration::from_secs)
                        .unwrap_or_else(|| retry_config.delay_for_attempt(attempt));

                    warn!(
                        request_id = %request.id,
                        error = %err,
                        attempt = attempt + 1,
                        delay_ms = delay.as_millis() as u64,
                        "Retrying request due to error"
                    );

                    _last_error = Some(err);
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    /// Execute a single request without retry.
    async fn execute_single(&self, request: &HttpRequest) -> NetworkResult<HttpResponse> {
        // Track active requests
        {
            let mut count = self.active_requests.write().await;
            if *count >= self.config.max_concurrent_requests {
                warn!("Max concurrent requests reached");
            }
            *count += 1;
        }

        let start = Instant::now();
        let result = self.do_execute(request).await;

        // Decrement active requests
        {
            let mut count = self.active_requests.write().await;
            *count = count.saturating_sub(1);
        }

        match result {
            Ok(mut response) => {
                response.duration = start.elapsed();
                debug!(
                    request_id = %request.id,
                    method = %request.method,
                    url = %request.url,
                    status = response.status,
                    duration_ms = response.duration.as_millis() as u64,
                    "Request completed"
                );
                Ok(response)
            }
            Err(err) => {
                error!(
                    request_id = %request.id,
                    method = %request.method,
                    url = %request.url,
                    error = %err,
                    duration_ms = start.elapsed().as_millis() as u64,
                    "Request failed"
                );

                // Update offline detector
                if let Some(detector) = &self.offline_detector {
                    detector.record_failure();
                }

                Err(err)
            }
        }
    }

    /// Internal request execution.
    async fn do_execute(&self, request: &HttpRequest) -> NetworkResult<HttpResponse> {
        let method: reqwest::Method = request.method.into();
        let mut builder = self.inner.request(method, request.url.clone());

        // Add headers
        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        // Set timeout
        if let Some(timeout) = request.timeout {
            builder = builder.timeout(timeout);
        }

        // Add body
        builder = match &request.body {
            RequestBody::None => builder,
            RequestBody::Json(value) => builder.json(value),
            RequestBody::Bytes(data) => builder.body(data.clone()),
            RequestBody::Text(text) => builder.body(text.clone()),
            RequestBody::Form(data) => builder.form(data),
            RequestBody::Multipart(fields) => {
                let mut form = reqwest::multipart::Form::new();
                for field in fields {
                    match &field.value {
                        super::MultipartValue::Text(text) => {
                            form = form.text(field.name.clone(), text.clone());
                        }
                        super::MultipartValue::File {
                            filename,
                            content_type,
                            data,
                        } => {
                            let part = reqwest::multipart::Part::bytes(data.clone())
                                .file_name(filename.clone())
                                .mime_str(content_type)
                                .map_err(|e| NetworkError::ConfigError {
                                    message: e.to_string(),
                                })?;
                            form = form.part(field.name.clone(), part);
                        }
                    }
                }
                builder.multipart(form)
            }
        };

        // Execute request
        let response = builder.send().await.map_err(|e| {
            if e.is_timeout() {
                NetworkError::Timeout {
                    duration_secs: request
                        .timeout
                        .unwrap_or(self.config.default_timeout)
                        .as_secs(),
                }
            } else if e.is_connect() {
                NetworkError::Offline
            } else {
                NetworkError::HttpError(e)
            }
        })?;

        // Update offline detector on success
        if let Some(detector) = &self.offline_detector {
            detector.record_success();
        }

        // Convert to our response type
        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.bytes().await.map_err(NetworkError::HttpError)?;

        Ok(HttpResponse::new(
            request.id,
            status,
            headers,
            body.to_vec(),
            final_url,
            Duration::ZERO, // Will be set by caller
        ))
    }

    /// Execute a GET request.
    pub async fn get(&self, url: impl AsRef<str>) -> NetworkResult<HttpResponse> {
        let request = HttpRequest::get(url)?;
        self.execute(request).await
    }

    /// Execute a POST request with JSON body.
    pub async fn post<T: serde::Serialize>(
        &self,
        url: impl AsRef<str>,
        body: &T,
    ) -> NetworkResult<HttpResponse> {
        let request = HttpRequest::post(url)?.json(body)?;
        self.execute(request).await
    }

    /// Execute a PUT request with JSON body.
    pub async fn put<T: serde::Serialize>(
        &self,
        url: impl AsRef<str>,
        body: &T,
    ) -> NetworkResult<HttpResponse> {
        let request = HttpRequest::put(url)?.json(body)?;
        self.execute(request).await
    }

    /// Execute a PATCH request with JSON body.
    pub async fn patch<T: serde::Serialize>(
        &self,
        url: impl AsRef<str>,
        body: &T,
    ) -> NetworkResult<HttpResponse> {
        let request = HttpRequest::patch(url)?.json(body)?;
        self.execute(request).await
    }

    /// Execute a DELETE request.
    pub async fn delete(&self, url: impl AsRef<str>) -> NetworkResult<HttpResponse> {
        let request = HttpRequest::delete(url)?;
        self.execute(request).await
    }

    /// Get the number of currently active requests.
    pub async fn active_request_count(&self) -> usize {
        *self.active_requests.read().await
    }
}

impl std::fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient")
            .field("config", &self.config)
            .field("has_allowlist", &self.allowlist.is_some())
            .field("has_auth_manager", &self.auth_manager.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = HttpClient::new().unwrap();
        assert_eq!(client.active_request_count().await, 0);
    }

    #[tokio::test]
    async fn test_client_with_base_url() {
        let client = HttpClient::builder()
            .base_url("https://api.example.com")
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        assert!(client.config.base_url.is_some());
    }
}
