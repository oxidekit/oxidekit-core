//! HTTP request types and builders.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

/// HTTP methods supported by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// GET request
    Get,
    /// POST request
    Post,
    /// PUT request
    Put,
    /// PATCH request
    Patch,
    /// DELETE request
    Delete,
    /// HEAD request
    Head,
    /// OPTIONS request
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

impl From<HttpMethod> for reqwest::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
        }
    }
}

/// Body content for HTTP requests.
#[derive(Debug, Clone)]
pub enum RequestBody {
    /// No body.
    None,
    /// JSON body (will be serialized).
    Json(serde_json::Value),
    /// Raw bytes.
    Bytes(Vec<u8>),
    /// Form data (URL encoded).
    Form(HashMap<String, String>),
    /// Multipart form data.
    Multipart(Vec<MultipartField>),
    /// Plain text.
    Text(String),
}

impl Default for RequestBody {
    fn default() -> Self {
        RequestBody::None
    }
}

/// A field in a multipart form request.
#[derive(Debug, Clone)]
pub struct MultipartField {
    /// Field name.
    pub name: String,
    /// Field value.
    pub value: MultipartValue,
}

/// Value type for multipart form fields.
#[derive(Debug, Clone)]
pub enum MultipartValue {
    /// Text value.
    Text(String),
    /// File content with filename and mime type.
    File {
        /// Original filename.
        filename: String,
        /// MIME type.
        content_type: String,
        /// File content.
        data: Vec<u8>,
    },
}

/// HTTP request representation.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// Unique request ID for tracking.
    pub id: Uuid,
    /// HTTP method.
    pub method: HttpMethod,
    /// Request URL.
    pub url: Url,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// Request body.
    pub body: RequestBody,
    /// Request timeout.
    pub timeout: Option<Duration>,
    /// Retry configuration.
    pub retry_config: RetryConfig,
    /// Whether to follow redirects.
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow.
    pub max_redirects: u32,
    /// Custom metadata for interceptors.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Whether auth should be attached automatically.
    pub require_auth: bool,
    /// Specific auth provider to use (if not default).
    pub auth_provider: Option<String>,
    /// Whether to skip allowlist checking (requires capability).
    pub bypass_allowlist: bool,
}

impl HttpRequest {
    /// Create a new GET request.
    pub fn get(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Self::new(HttpMethod::Get, url)
    }

    /// Create a new POST request.
    pub fn post(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Self::new(HttpMethod::Post, url)
    }

    /// Create a new PUT request.
    pub fn put(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Self::new(HttpMethod::Put, url)
    }

    /// Create a new PATCH request.
    pub fn patch(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Self::new(HttpMethod::Patch, url)
    }

    /// Create a new DELETE request.
    pub fn delete(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Self::new(HttpMethod::Delete, url)
    }

    /// Create a new request with the given method and URL.
    pub fn new(method: HttpMethod, url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Ok(Self {
            id: Uuid::new_v4(),
            method,
            url: Url::parse(url.as_ref())?,
            headers: HashMap::new(),
            body: RequestBody::None,
            timeout: None,
            retry_config: RetryConfig::default(),
            follow_redirects: true,
            max_redirects: 10,
            metadata: HashMap::new(),
            require_auth: false,
            auth_provider: None,
            bypass_allowlist: false,
        })
    }

    /// Set a header value.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set multiple headers.
    pub fn headers(mut self, headers: impl IntoIterator<Item = (String, String)>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Set JSON body.
    pub fn json<T: Serialize>(mut self, body: &T) -> Result<Self, serde_json::Error> {
        self.body = RequestBody::Json(serde_json::to_value(body)?);
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    /// Set raw JSON value as body.
    pub fn json_value(mut self, body: serde_json::Value) -> Self {
        self.body = RequestBody::Json(body);
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self
    }

    /// Set form body (URL encoded).
    pub fn form(mut self, data: HashMap<String, String>) -> Self {
        self.body = RequestBody::Form(data);
        self.headers.insert(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        );
        self
    }

    /// Set text body.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.body = RequestBody::Text(text.into());
        self.headers
            .insert("Content-Type".to_string(), "text/plain".to_string());
        self
    }

    /// Set raw bytes body.
    pub fn bytes(mut self, data: Vec<u8>, content_type: impl Into<String>) -> Self {
        self.body = RequestBody::Bytes(data);
        self.headers
            .insert("Content-Type".to_string(), content_type.into());
        self
    }

    /// Set request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set retry configuration.
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Disable retries.
    pub fn no_retry(mut self) -> Self {
        self.retry_config = RetryConfig::none();
        self
    }

    /// Enable auth requirement.
    pub fn with_auth(mut self) -> Self {
        self.require_auth = true;
        self
    }

    /// Use a specific auth provider.
    pub fn with_auth_provider(mut self, provider: impl Into<String>) -> Self {
        self.require_auth = true;
        self.auth_provider = Some(provider.into());
        self
    }

    /// Disable redirect following.
    pub fn no_redirects(mut self) -> Self {
        self.follow_redirects = false;
        self
    }

    /// Set maximum redirects to follow.
    pub fn max_redirects(mut self, max: u32) -> Self {
        self.max_redirects = max;
        self
    }

    /// Add custom metadata for interceptors.
    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set bearer token auth header.
    pub fn bearer_token(mut self, token: impl AsRef<str>) -> Self {
        self.headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", token.as_ref()),
        );
        self
    }

    /// Set basic auth header.
    pub fn basic_auth(mut self, username: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        let credentials = format!("{}:{}", username.as_ref(), password.as_ref());
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, credentials);
        self.headers.insert(
            "Authorization".to_string(),
            format!("Basic {}", encoded),
        );
        self
    }

    /// Add query parameter to URL.
    pub fn query(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.url
            .query_pairs_mut()
            .append_pair(key.as_ref(), value.as_ref());
        self
    }

    /// Add multiple query parameters.
    pub fn query_pairs(mut self, pairs: impl IntoIterator<Item = (String, String)>) -> Self {
        {
            let mut query = self.url.query_pairs_mut();
            for (k, v) in pairs {
                query.append_pair(&k, &v);
            }
        }
        self
    }
}

/// Configuration for automatic request retries.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub backoff_multiplier: f64,
    /// Whether to retry on timeout errors.
    pub retry_on_timeout: bool,
    /// Whether to retry on network errors.
    pub retry_on_network_error: bool,
    /// HTTP status codes that should trigger a retry.
    pub retry_status_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_on_network_error: true,
            retry_status_codes: vec![408, 429, 500, 502, 503, 504],
        }
    }
}

impl RetryConfig {
    /// Create a retry config with no retries.
    pub fn none() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Create a retry config with specified max retries.
    pub fn with_max_retries(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Calculate the delay for a given retry attempt (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        let delay = Duration::from_millis(delay_ms as u64);
        std::cmp::min(delay, self.max_delay)
    }

    /// Check if a status code should trigger a retry.
    pub fn should_retry_status(&self, status: u16) -> bool {
        self.retry_status_codes.contains(&status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let request = HttpRequest::get("https://api.example.com/users")
            .unwrap()
            .header("Accept", "application/json")
            .query("page", "1")
            .query("limit", "10")
            .timeout(Duration::from_secs(30))
            .with_auth();

        assert_eq!(request.method, HttpMethod::Get);
        assert!(request.url.as_str().contains("page=1"));
        assert!(request.require_auth);
    }

    #[test]
    fn test_retry_backoff() {
        let config = RetryConfig::default();

        let delay0 = config.delay_for_attempt(0);
        let delay1 = config.delay_for_attempt(1);
        let delay2 = config.delay_for_attempt(2);

        assert_eq!(delay0.as_millis(), 100);
        assert_eq!(delay1.as_millis(), 200);
        assert_eq!(delay2.as_millis(), 400);
    }

    #[test]
    fn test_json_body() {
        let data = serde_json::json!({
            "name": "Test",
            "value": 42
        });

        let request = HttpRequest::post("https://api.example.com/data")
            .unwrap()
            .json_value(data);

        assert!(matches!(request.body, RequestBody::Json(_)));
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }
}
