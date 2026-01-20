//! Network capabilities and permissions for OxideKit.
//!
//! Defines the `native.network` capability that plugins must declare
//! to access network functionality. Enforces principle of least privilege.

use serde::{Deserialize, Serialize};

/// Network capability configuration.
///
/// Plugins must declare this capability to access network features.
/// The capability defines what network operations are allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCapability {
    /// Allowed URL patterns (domains, paths).
    #[serde(default)]
    pub allowed_urls: Vec<String>,

    /// Allowed HTTP methods.
    #[serde(default = "default_allowed_methods")]
    pub allowed_methods: Vec<String>,

    /// Whether WebSocket connections are allowed.
    #[serde(default)]
    pub allow_websocket: bool,

    /// Whether offline/background requests are allowed.
    #[serde(default)]
    pub allow_background: bool,

    /// Maximum concurrent connections.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Maximum request body size in bytes.
    #[serde(default = "default_max_body_size")]
    pub max_request_body_size: usize,

    /// Maximum response body size in bytes.
    #[serde(default = "default_max_response_size")]
    pub max_response_body_size: usize,

    /// Allowed content types.
    #[serde(default)]
    pub allowed_content_types: Vec<String>,

    /// Whether to allow custom headers.
    #[serde(default = "default_true")]
    pub allow_custom_headers: bool,

    /// Blocked headers (cannot be set by plugin).
    #[serde(default = "default_blocked_headers")]
    pub blocked_headers: Vec<String>,

    /// Whether to allow credentials (cookies, auth headers).
    #[serde(default)]
    pub allow_credentials: bool,

    /// Whether to allow bypassing certificate validation (DANGEROUS).
    #[serde(default)]
    pub allow_insecure: bool,

    /// Custom capability extensions.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
}

fn default_allowed_methods() -> Vec<String> {
    vec!["GET".to_string(), "POST".to_string()]
}

fn default_max_connections() -> u32 {
    10
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_max_response_size() -> usize {
    50 * 1024 * 1024 // 50MB
}

fn default_true() -> bool {
    true
}

fn default_blocked_headers() -> Vec<String> {
    vec![
        "Host".to_string(),
        "Connection".to_string(),
        "Content-Length".to_string(),
        "Transfer-Encoding".to_string(),
    ]
}

impl Default for NetworkCapability {
    fn default() -> Self {
        Self {
            allowed_urls: Vec::new(),
            allowed_methods: default_allowed_methods(),
            allow_websocket: false,
            allow_background: false,
            max_connections: default_max_connections(),
            max_request_body_size: default_max_body_size(),
            max_response_body_size: default_max_response_size(),
            allowed_content_types: Vec::new(),
            allow_custom_headers: true,
            blocked_headers: default_blocked_headers(),
            allow_credentials: false,
            allow_insecure: false,
            extensions: std::collections::HashMap::new(),
        }
    }
}

impl NetworkCapability {
    /// Create a new network capability with minimal permissions.
    pub fn minimal() -> Self {
        Self {
            allowed_methods: Vec::new(), // Start with no methods allowed
            ..Default::default()
        }
    }

    /// Create a network capability with full permissions (use with caution).
    pub fn full() -> Self {
        Self {
            allowed_urls: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "PATCH".to_string(),
                "DELETE".to_string(),
                "HEAD".to_string(),
                "OPTIONS".to_string(),
            ],
            allow_websocket: true,
            allow_background: true,
            max_connections: 100,
            max_request_body_size: 100 * 1024 * 1024,
            max_response_body_size: 100 * 1024 * 1024,
            allowed_content_types: Vec::new(), // Empty = all allowed
            allow_custom_headers: true,
            blocked_headers: default_blocked_headers(),
            allow_credentials: true,
            allow_insecure: false, // Still don't allow insecure by default
            extensions: std::collections::HashMap::new(),
        }
    }

    /// Add an allowed URL pattern.
    pub fn allow_url(mut self, pattern: impl Into<String>) -> Self {
        self.allowed_urls.push(pattern.into());
        self
    }

    /// Add allowed URL patterns.
    pub fn allow_urls(mut self, patterns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.allowed_urls.extend(patterns.into_iter().map(|p| p.into()));
        self
    }

    /// Add an allowed HTTP method.
    pub fn allow_method(mut self, method: impl Into<String>) -> Self {
        self.allowed_methods.push(method.into().to_uppercase());
        self
    }

    /// Enable WebSocket support.
    pub fn with_websocket(mut self) -> Self {
        self.allow_websocket = true;
        self
    }

    /// Enable background requests.
    pub fn with_background(mut self) -> Self {
        self.allow_background = true;
        self
    }

    /// Enable credentials (cookies, auth).
    pub fn with_credentials(mut self) -> Self {
        self.allow_credentials = true;
        self
    }

    /// Set maximum connections.
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
}

/// Validator for network capability enforcement.
#[derive(Debug)]
pub struct CapabilityValidator {
    capability: NetworkCapability,
}

impl CapabilityValidator {
    /// Create a new validator with the given capability.
    pub fn new(capability: NetworkCapability) -> Self {
        Self { capability }
    }

    /// Check if a URL is allowed.
    pub fn is_url_allowed(&self, url: &str) -> bool {
        if self.capability.allowed_urls.is_empty() {
            return false;
        }

        // Check for wildcard
        if self.capability.allowed_urls.iter().any(|p| p == "*") {
            return true;
        }

        // Parse URL
        let parsed = match url::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let host = parsed.host_str().unwrap_or("");

        for pattern in &self.capability.allowed_urls {
            if self.matches_pattern(url, host, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a URL pattern matches.
    fn matches_pattern(&self, url: &str, host: &str, pattern: &str) -> bool {
        // Simple pattern matching
        // Supports:
        // - Exact match: "https://api.example.com"
        // - Domain match: "example.com" (matches any subdomain)
        // - Wildcard subdomain: "*.example.com"
        // - Path prefix: "https://api.example.com/v1/*"

        if pattern == "*" {
            return true;
        }

        // If pattern looks like a URL, do full match
        if pattern.starts_with("http://") || pattern.starts_with("https://") {
            if pattern.ends_with("*") {
                let prefix = &pattern[..pattern.len() - 1];
                return url.starts_with(prefix);
            }
            return url == pattern || url.starts_with(&format!("{}/", pattern));
        }

        // Domain-based matching
        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            return host.ends_with(suffix) || host == &pattern[2..];
        }

        // Exact domain match
        host == pattern || host.ends_with(&format!(".{}", pattern))
    }

    /// Check if an HTTP method is allowed.
    pub fn is_method_allowed(&self, method: &str) -> bool {
        let method_upper = method.to_uppercase();
        self.capability
            .allowed_methods
            .iter()
            .any(|m| m == &method_upper)
    }

    /// Check if a header is allowed to be set.
    pub fn is_header_allowed(&self, header: &str) -> bool {
        if !self.capability.allow_custom_headers {
            return false;
        }

        let header_lower = header.to_lowercase();
        !self
            .capability
            .blocked_headers
            .iter()
            .any(|h| h.to_lowercase() == header_lower)
    }

    /// Check if request body size is within limits.
    pub fn is_request_body_size_allowed(&self, size: usize) -> bool {
        size <= self.capability.max_request_body_size
    }

    /// Check if response body size is within limits.
    pub fn is_response_body_size_allowed(&self, size: usize) -> bool {
        size <= self.capability.max_response_body_size
    }

    /// Check if WebSocket is allowed.
    pub fn is_websocket_allowed(&self) -> bool {
        self.capability.allow_websocket
    }

    /// Check if credentials are allowed.
    pub fn is_credentials_allowed(&self) -> bool {
        self.capability.allow_credentials
    }

    /// Get the maximum allowed connections.
    pub fn max_connections(&self) -> u32 {
        self.capability.max_connections
    }

    /// Validate a request against the capability.
    pub fn validate_request(&self, request: &crate::http::HttpRequest) -> Result<(), CapabilityViolation> {
        // Check URL
        if !self.is_url_allowed(request.url.as_str()) {
            return Err(CapabilityViolation::UrlNotAllowed {
                url: request.url.to_string(),
            });
        }

        // Check method
        if !self.is_method_allowed(&request.method.to_string()) {
            return Err(CapabilityViolation::MethodNotAllowed {
                method: request.method.to_string(),
            });
        }

        // Check headers
        for header in request.headers.keys() {
            if !self.is_header_allowed(header) {
                return Err(CapabilityViolation::HeaderNotAllowed {
                    header: header.clone(),
                });
            }
        }

        // Check body size
        let body_size = match &request.body {
            crate::http::RequestBody::None => 0,
            crate::http::RequestBody::Json(v) => serde_json::to_vec(v).map(|b| b.len()).unwrap_or(0),
            crate::http::RequestBody::Bytes(b) => b.len(),
            crate::http::RequestBody::Text(t) => t.len(),
            crate::http::RequestBody::Form(f) => {
                f.iter().map(|(k, v)| k.len() + v.len()).sum()
            }
            crate::http::RequestBody::Multipart(fields) => {
                fields.iter().map(|f| {
                    f.name.len() + match &f.value {
                        crate::http::MultipartValue::Text(t) => t.len(),
                        crate::http::MultipartValue::File { data, .. } => data.len(),
                    }
                }).sum()
            }
        };

        if !self.is_request_body_size_allowed(body_size) {
            return Err(CapabilityViolation::RequestBodyTooLarge {
                size: body_size,
                max: self.capability.max_request_body_size,
            });
        }

        Ok(())
    }
}

/// Violations of network capability constraints.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CapabilityViolation {
    /// URL is not in the allowlist.
    #[error("URL not allowed: {url}")]
    UrlNotAllowed {
        /// The disallowed URL.
        url: String,
    },

    /// HTTP method is not allowed.
    #[error("HTTP method not allowed: {method}")]
    MethodNotAllowed {
        /// The disallowed method.
        method: String,
    },

    /// Header is not allowed.
    #[error("Header not allowed: {header}")]
    HeaderNotAllowed {
        /// The disallowed header.
        header: String,
    },

    /// Request body exceeds size limit.
    #[error("Request body too large: {size} bytes (max: {max})")]
    RequestBodyTooLarge {
        /// Actual size.
        size: usize,
        /// Maximum allowed.
        max: usize,
    },

    /// Response body exceeds size limit.
    #[error("Response body too large: {size} bytes (max: {max})")]
    ResponseBodyTooLarge {
        /// Actual size.
        size: usize,
        /// Maximum allowed.
        max: usize,
    },

    /// WebSocket not allowed.
    #[error("WebSocket connections not allowed")]
    WebSocketNotAllowed,

    /// Too many connections.
    #[error("Too many connections: {current} (max: {max})")]
    TooManyConnections {
        /// Current count.
        current: u32,
        /// Maximum allowed.
        max: u32,
    },

    /// Credentials not allowed.
    #[error("Credentials (cookies, auth) not allowed")]
    CredentialsNotAllowed,
}

/// Predefined capability profiles for common use cases.
pub mod profiles {
    use super::NetworkCapability;

    /// Read-only access to specific APIs.
    pub fn api_read(domains: &[&str]) -> NetworkCapability {
        NetworkCapability::minimal()
            .allow_urls(domains.iter().map(|d| d.to_string()))
            .allow_method("GET")
            .allow_method("HEAD")
    }

    /// Read-write access to specific APIs.
    pub fn api_readwrite(domains: &[&str]) -> NetworkCapability {
        NetworkCapability::minimal()
            .allow_urls(domains.iter().map(|d| d.to_string()))
            .allow_method("GET")
            .allow_method("POST")
            .allow_method("PUT")
            .allow_method("PATCH")
            .allow_method("DELETE")
    }

    /// WebSocket access to specific endpoints.
    pub fn websocket(domains: &[&str]) -> NetworkCapability {
        NetworkCapability::minimal()
            .allow_urls(domains.iter().map(|d| d.to_string()))
            .with_websocket()
    }

    /// Full network access (use sparingly).
    pub fn unrestricted() -> NetworkCapability {
        NetworkCapability::full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_pattern_matching() {
        let cap = NetworkCapability::minimal()
            .allow_url("api.example.com")
            .allow_url("*.internal.com")
            .allow_url("https://secure.example.com/api/*");

        let validator = CapabilityValidator::new(cap);

        // Exact domain match
        assert!(validator.is_url_allowed("https://api.example.com"));
        assert!(validator.is_url_allowed("https://api.example.com/users"));

        // Wildcard subdomain
        assert!(validator.is_url_allowed("https://foo.internal.com"));
        assert!(validator.is_url_allowed("https://bar.internal.com/path"));

        // Path prefix
        assert!(validator.is_url_allowed("https://secure.example.com/api/v1"));
        assert!(!validator.is_url_allowed("https://secure.example.com/other"));

        // Not allowed
        assert!(!validator.is_url_allowed("https://evil.com"));
    }

    #[test]
    fn test_method_validation() {
        let cap = NetworkCapability::minimal()
            .allow_method("GET")
            .allow_method("POST");

        let validator = CapabilityValidator::new(cap);

        assert!(validator.is_method_allowed("GET"));
        assert!(validator.is_method_allowed("get")); // Case insensitive
        assert!(validator.is_method_allowed("POST"));
        assert!(!validator.is_method_allowed("DELETE"));
    }

    #[test]
    fn test_header_validation() {
        let cap = NetworkCapability::default();
        let validator = CapabilityValidator::new(cap);

        assert!(validator.is_header_allowed("Authorization"));
        assert!(validator.is_header_allowed("X-Custom-Header"));
        assert!(!validator.is_header_allowed("Host"));
        assert!(!validator.is_header_allowed("Connection"));
    }

    #[test]
    fn test_profiles() {
        let read_only = profiles::api_read(&["api.example.com"]);
        let validator = CapabilityValidator::new(read_only);

        assert!(validator.is_url_allowed("https://api.example.com/users"));
        assert!(validator.is_method_allowed("GET"));
        assert!(!validator.is_method_allowed("POST"));
    }
}
