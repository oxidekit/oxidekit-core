//! Request/Response interceptors for OxideKit networking.
//!
//! Interceptors allow modification of requests before they are sent
//! and responses after they are received. Common use cases:
//! - Adding authentication headers
//! - Logging/metrics
//! - Request/response transformation
//! - Error handling

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, trace};

use crate::error::NetworkResult;
use crate::http::{HttpRequest, HttpResponse};

/// Trait for request/response interceptors.
///
/// Interceptors are called in order for requests (first registered = first called)
/// and in reverse order for responses (last registered = first called).
#[async_trait]
pub trait Interceptor: Send + Sync + std::fmt::Debug {
    /// Get the interceptor's name (for logging/debugging).
    fn name(&self) -> &str;

    /// Intercept and potentially modify a request before it's sent.
    ///
    /// Return `Ok(request)` to continue with the (potentially modified) request,
    /// or `Err(error)` to abort the request.
    async fn intercept_request(&self, request: HttpRequest) -> NetworkResult<HttpRequest> {
        Ok(request)
    }

    /// Intercept and potentially modify a response after it's received.
    ///
    /// Return `Ok(response)` to continue with the (potentially modified) response,
    /// or `Err(error)` to signal an error.
    async fn intercept_response(&self, response: HttpResponse) -> NetworkResult<HttpResponse> {
        Ok(response)
    }

    /// Called when a request fails with an error.
    ///
    /// Can be used for logging, metrics, or error transformation.
    async fn on_error(&self, request: &HttpRequest, error: &crate::error::NetworkError) {
        // Default: do nothing
        let _ = (request, error);
    }
}

/// Chain of interceptors.
#[derive(Debug)]
pub struct InterceptorChain {
    interceptors: Vec<Arc<dyn Interceptor>>,
}

impl InterceptorChain {
    /// Create a new interceptor chain.
    pub fn new(interceptors: Vec<Arc<dyn Interceptor>>) -> Self {
        Self { interceptors }
    }

    /// Create an empty interceptor chain.
    pub fn empty() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Add an interceptor to the chain.
    pub fn add(&mut self, interceptor: impl Interceptor + 'static) {
        self.interceptors.push(Arc::new(interceptor));
    }

    /// Run all request interceptors in order.
    pub async fn intercept_request(&self, mut request: HttpRequest) -> NetworkResult<HttpRequest> {
        for interceptor in &self.interceptors {
            trace!(
                interceptor = %interceptor.name(),
                request_id = %request.id,
                "Running request interceptor"
            );
            request = interceptor.intercept_request(request).await?;
        }
        Ok(request)
    }

    /// Run all response interceptors in reverse order.
    pub async fn intercept_response(
        &self,
        mut response: HttpResponse,
    ) -> NetworkResult<HttpResponse> {
        for interceptor in self.interceptors.iter().rev() {
            trace!(
                interceptor = %interceptor.name(),
                request_id = %response.request_id,
                "Running response interceptor"
            );
            response = interceptor.intercept_response(response).await?;
        }
        Ok(response)
    }

    /// Notify interceptors of an error.
    pub async fn on_error(&self, request: &HttpRequest, error: &crate::error::NetworkError) {
        for interceptor in &self.interceptors {
            interceptor.on_error(request, error).await;
        }
    }
}

/// Logging interceptor that logs request/response details.
#[derive(Debug, Clone)]
pub struct LoggingInterceptor {
    name: String,
    /// Whether to log request bodies.
    log_request_body: bool,
    /// Whether to log response bodies.
    log_response_body: bool,
    /// Maximum body length to log.
    max_body_length: usize,
}

impl Default for LoggingInterceptor {
    fn default() -> Self {
        Self {
            name: "logging".to_string(),
            log_request_body: false,
            log_response_body: false,
            max_body_length: 1024,
        }
    }
}

impl LoggingInterceptor {
    /// Create a new logging interceptor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable request body logging.
    pub fn with_request_body(mut self) -> Self {
        self.log_request_body = true;
        self
    }

    /// Enable response body logging.
    pub fn with_response_body(mut self) -> Self {
        self.log_response_body = true;
        self
    }

    /// Set maximum body length to log.
    pub fn with_max_body_length(mut self, length: usize) -> Self {
        self.max_body_length = length;
        self
    }
}

#[async_trait]
impl Interceptor for LoggingInterceptor {
    fn name(&self) -> &str {
        &self.name
    }

    async fn intercept_request(&self, request: HttpRequest) -> NetworkResult<HttpRequest> {
        debug!(
            request_id = %request.id,
            method = %request.method,
            url = %request.url,
            headers = ?request.headers.keys().collect::<Vec<_>>(),
            "Outgoing request"
        );

        if self.log_request_body {
            if let crate::http::RequestBody::Json(ref body) = request.body {
                let body_str = serde_json::to_string(body).unwrap_or_default();
                let truncated = if body_str.len() > self.max_body_length {
                    format!("{}... (truncated)", &body_str[..self.max_body_length])
                } else {
                    body_str
                };
                debug!(request_id = %request.id, body = %truncated, "Request body");
            }
        }

        Ok(request)
    }

    async fn intercept_response(&self, response: HttpResponse) -> NetworkResult<HttpResponse> {
        debug!(
            request_id = %response.request_id,
            status = response.status,
            duration_ms = response.duration.as_millis() as u64,
            "Incoming response"
        );

        if self.log_response_body {
            let body_str = response.text_lossy();
            let truncated = if body_str.len() > self.max_body_length {
                format!("{}... (truncated)", &body_str[..self.max_body_length])
            } else {
                body_str
            };
            debug!(request_id = %response.request_id, body = %truncated, "Response body");
        }

        Ok(response)
    }

    async fn on_error(&self, request: &HttpRequest, error: &crate::error::NetworkError) {
        tracing::error!(
            request_id = %request.id,
            method = %request.method,
            url = %request.url,
            error = %error,
            "Request failed"
        );
    }
}

/// Header injection interceptor.
#[derive(Debug, Clone)]
pub struct HeaderInterceptor {
    name: String,
    headers: std::collections::HashMap<String, String>,
}

impl HeaderInterceptor {
    /// Create a new header interceptor.
    pub fn new() -> Self {
        Self {
            name: "headers".to_string(),
            headers: std::collections::HashMap::new(),
        }
    }

    /// Add a header to inject.
    pub fn add_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Add multiple headers.
    pub fn add_headers(
        mut self,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (k, v) in headers {
            self.headers.insert(k.into(), v.into());
        }
        self
    }
}

impl Default for HeaderInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Interceptor for HeaderInterceptor {
    fn name(&self) -> &str {
        &self.name
    }

    async fn intercept_request(&self, mut request: HttpRequest) -> NetworkResult<HttpRequest> {
        for (key, value) in &self.headers {
            request.headers.entry(key.clone()).or_insert_with(|| value.clone());
        }
        Ok(request)
    }
}

/// Metrics collection interceptor.
pub struct MetricsInterceptor {
    name: String,
    /// Callback for recording metrics.
    on_request_complete: Option<Box<dyn Fn(RequestMetrics) + Send + Sync>>,
}

impl std::fmt::Debug for MetricsInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsInterceptor")
            .field("name", &self.name)
            .field("has_callback", &self.on_request_complete.is_some())
            .finish()
    }
}

/// Metrics collected for a request.
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    /// Request ID.
    pub request_id: uuid::Uuid,
    /// HTTP method.
    pub method: String,
    /// Request URL (host + path).
    pub url: String,
    /// Response status code.
    pub status: Option<u16>,
    /// Request duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the request succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

impl MetricsInterceptor {
    /// Create a new metrics interceptor.
    pub fn new() -> Self {
        Self {
            name: "metrics".to_string(),
            on_request_complete: None,
        }
    }

    /// Set the callback for recording metrics.
    pub fn with_callback(mut self, callback: impl Fn(RequestMetrics) + Send + Sync + 'static) -> Self {
        self.on_request_complete = Some(Box::new(callback));
        self
    }
}

impl Default for MetricsInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Interceptor for MetricsInterceptor {
    fn name(&self) -> &str {
        &self.name
    }

    async fn intercept_response(&self, response: HttpResponse) -> NetworkResult<HttpResponse> {
        if let Some(callback) = &self.on_request_complete {
            let metrics = RequestMetrics {
                request_id: response.request_id,
                method: "UNKNOWN".to_string(), // Would need to be passed through metadata
                url: response.final_url.clone(),
                status: Some(response.status),
                duration_ms: response.duration.as_millis() as u64,
                success: response.is_success(),
                error: None,
            };
            callback(metrics);
        }
        Ok(response)
    }

    async fn on_error(&self, request: &HttpRequest, error: &crate::error::NetworkError) {
        if let Some(callback) = &self.on_request_complete {
            let metrics = RequestMetrics {
                request_id: request.id,
                method: request.method.to_string(),
                url: request.url.to_string(),
                status: None,
                duration_ms: 0,
                success: false,
                error: Some(error.to_string()),
            };
            callback(metrics);
        }
    }
}

// Note: We can't implement Debug for dyn Fn directly due to orphan rules.
// The MetricsInterceptor handles its own Debug implementation.

/// Error transformation interceptor.
///
/// Can be used to transform or wrap errors based on response content.
#[derive(Debug)]
pub struct ErrorTransformInterceptor {
    name: String,
}

impl ErrorTransformInterceptor {
    /// Create a new error transform interceptor.
    pub fn new() -> Self {
        Self {
            name: "error-transform".to_string(),
        }
    }
}

impl Default for ErrorTransformInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Interceptor for ErrorTransformInterceptor {
    fn name(&self) -> &str {
        &self.name
    }

    async fn intercept_response(&self, response: HttpResponse) -> NetworkResult<HttpResponse> {
        // Transform known error responses into appropriate errors
        match response.status {
            401 => {
                // Check for specific auth error patterns in response body
                if let Ok(body) = response.text() {
                    if body.contains("token_expired") || body.contains("jwt expired") {
                        return Err(crate::error::NetworkError::TokenExpired);
                    }
                }
            }
            429 => {
                // Rate limited - extract retry-after if possible
                let retry_after = response.retry_after();
                return Err(crate::error::NetworkError::RateLimited {
                    retry_after_secs: retry_after,
                });
            }
            _ => {}
        }

        Ok(response)
    }
}

/// Caching interceptor for conditional requests.
#[derive(Debug)]
pub struct CacheInterceptor {
    name: String,
    // Would include cache storage in real implementation
}

impl CacheInterceptor {
    /// Create a new cache interceptor.
    pub fn new() -> Self {
        Self {
            name: "cache".to_string(),
        }
    }
}

impl Default for CacheInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Interceptor for CacheInterceptor {
    fn name(&self) -> &str {
        &self.name
    }

    async fn intercept_request(&self, request: HttpRequest) -> NetworkResult<HttpRequest> {
        // In a real implementation, would look up cached response
        // and add If-None-Match/If-Modified-Since headers
        trace!(
            request_id = %request.id,
            "Cache interceptor: would check for cached response"
        );
        Ok(request)
    }

    async fn intercept_response(&self, response: HttpResponse) -> NetworkResult<HttpResponse> {
        // In a real implementation, would cache the response
        // based on Cache-Control headers
        if response.status == 304 {
            trace!(
                request_id = %response.request_id,
                "Cache interceptor: would return cached response for 304"
            );
        }
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::ResponseBuilder;

    #[tokio::test]
    async fn test_interceptor_chain() {
        let chain = InterceptorChain::new(vec![
            Arc::new(LoggingInterceptor::new()),
            Arc::new(HeaderInterceptor::new().add_header("X-Custom", "value")),
        ]);

        let request = HttpRequest::get("https://example.com").unwrap();
        let request = chain.intercept_request(request).await.unwrap();

        assert_eq!(
            request.headers.get("X-Custom"),
            Some(&"value".to_string())
        );
    }

    #[tokio::test]
    async fn test_header_interceptor() {
        let interceptor = HeaderInterceptor::new()
            .add_header("Authorization", "Bearer token")
            .add_header("X-Request-ID", "123");

        let request = HttpRequest::get("https://example.com").unwrap();
        let request = interceptor.intercept_request(request).await.unwrap();

        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert_eq!(
            request.headers.get("X-Request-ID"),
            Some(&"123".to_string())
        );
    }

    #[tokio::test]
    async fn test_metrics_interceptor() {
        use std::sync::atomic::{AtomicU64, Ordering};

        let call_count = Arc::new(AtomicU64::new(0));
        let call_count_clone = call_count.clone();

        let interceptor = MetricsInterceptor::new().with_callback(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let response = ResponseBuilder::new()
            .status(200)
            .url("https://example.com")
            .build();

        interceptor.intercept_response(response).await.unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
}
