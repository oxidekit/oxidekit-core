//! Development proxy configuration for OxideKit applications.
//!
//! This module provides a first-class development proxy that eliminates CORS
//! issues during development by forwarding API requests from the frontend
//! origin to the backend origin.
//!
//! # Features
//!
//! - Path-based routing with prefix matching
//! - Path rewriting (strip prefix when forwarding)
//! - WebSocket proxying support
//! - Request logging and debugging
//! - Multiple backend targets
//! - Custom header injection
//!
//! # Example
//!
//! ```rust
//! use oxide_network::proxy::{DevProxy, ProxyTarget, ProxyConfig};
//!
//! // Configure a dev proxy
//! let config = ProxyConfig::new()
//!     .frontend_port(3000)
//!     .add_target(
//!         ProxyTarget::new("api", "http://localhost:8000")
//!             .with_path("/api")
//!             .strip_prefix(true)
//!     )
//!     .add_target(
//!         ProxyTarget::new("ws", "ws://localhost:8001")
//!             .with_path("/ws")
//!             .websocket(true)
//!     );
//!
//! // The proxy would forward:
//! // GET /api/users -> http://localhost:8000/users
//! // WS /ws/events -> ws://localhost:8001/events
//! ```
//!
//! # CLI Integration
//!
//! ```bash
//! oxide dev --proxy api=http://localhost:8000
//! oxide dev --proxy api=https://dev-api.example.com
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// A proxy target configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTarget {
    /// Name/alias for this target (e.g., "api", "auth", "graphql").
    pub name: String,
    /// The backend URL to forward requests to.
    pub target_url: String,
    /// Path prefix to match (e.g., "/api").
    pub path_prefix: String,
    /// Whether to strip the path prefix when forwarding.
    pub strip_prefix: bool,
    /// Whether this target handles WebSocket connections.
    pub websocket: bool,
    /// Timeout for proxied requests.
    pub timeout: Duration,
    /// Custom headers to add to proxied requests.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Whether to preserve the Host header from the original request.
    pub preserve_host: bool,
    /// Rewrite rules for path transformation.
    #[serde(default)]
    pub rewrites: Vec<PathRewrite>,
}

impl ProxyTarget {
    /// Create a new proxy target.
    pub fn new(name: impl Into<String>, target_url: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            path_prefix: format!("/{}", &name),
            name,
            target_url: target_url.into(),
            strip_prefix: true,
            websocket: false,
            timeout: Duration::from_secs(30),
            headers: HashMap::new(),
            preserve_host: false,
            rewrites: Vec::new(),
        }
    }

    /// Set the path prefix for matching.
    pub fn with_path(mut self, prefix: impl Into<String>) -> Self {
        self.path_prefix = prefix.into();
        self
    }

    /// Configure prefix stripping.
    pub fn strip_prefix(mut self, strip: bool) -> Self {
        self.strip_prefix = strip;
        self
    }

    /// Enable WebSocket support.
    pub fn websocket(mut self, enabled: bool) -> Self {
        self.websocket = enabled;
        self
    }

    /// Set request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add a custom header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Preserve the original Host header.
    pub fn preserve_host(mut self, preserve: bool) -> Self {
        self.preserve_host = preserve;
        self
    }

    /// Add a path rewrite rule.
    pub fn add_rewrite(mut self, rewrite: PathRewrite) -> Self {
        self.rewrites.push(rewrite);
        self
    }

    /// Check if a path matches this target.
    pub fn matches(&self, path: &str) -> bool {
        path.starts_with(&self.path_prefix) || path == self.path_prefix.trim_end_matches('/')
    }

    /// Transform a request path for this target.
    pub fn transform_path(&self, path: &str) -> String {
        let mut result = if self.strip_prefix && path.starts_with(&self.path_prefix) {
            path[self.path_prefix.len()..].to_string()
        } else {
            path.to_string()
        };

        // Ensure path starts with /
        if !result.starts_with('/') {
            result = format!("/{}", result);
        }

        // Apply rewrite rules
        for rewrite in &self.rewrites {
            if let Some(new_path) = rewrite.apply(&result) {
                result = new_path;
            }
        }

        result
    }

    /// Build the full target URL for a request path.
    pub fn build_url(&self, path: &str) -> String {
        let transformed = self.transform_path(path);
        let base = self.target_url.trim_end_matches('/');
        format!("{}{}", base, transformed)
    }
}

/// Path rewrite rule for transforming request paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRewrite {
    /// Pattern to match (supports simple wildcards).
    pub pattern: String,
    /// Replacement string (can use $1, $2, etc. for captures).
    pub replacement: String,
}

impl PathRewrite {
    /// Create a new path rewrite rule.
    pub fn new(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
        }
    }

    /// Apply this rewrite rule to a path.
    pub fn apply(&self, path: &str) -> Option<String> {
        // Simple pattern matching with * wildcard
        if self.pattern.contains('*') {
            let parts: Vec<&str> = self.pattern.split('*').collect();
            if parts.len() == 2 {
                let (prefix, suffix) = (parts[0], parts[1]);
                if path.starts_with(prefix) && path.ends_with(suffix) {
                    let middle_start = prefix.len();
                    let middle_end = path.len() - suffix.len();
                    if middle_start <= middle_end {
                        let captured = &path[middle_start..middle_end];
                        let result = self.replacement.replace("$1", captured);
                        return Some(result);
                    }
                }
            }
        } else if path == self.pattern {
            return Some(self.replacement.clone());
        }
        None
    }
}

/// Logging level for proxy requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProxyLogLevel {
    /// No logging.
    None,
    /// Log errors only.
    Error,
    /// Log warnings and errors.
    Warn,
    /// Log basic request info.
    #[default]
    Info,
    /// Log detailed request/response info.
    Debug,
    /// Log everything including headers and bodies.
    Trace,
}

impl fmt::Display for ProxyLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyLogLevel::None => write!(f, "none"),
            ProxyLogLevel::Error => write!(f, "error"),
            ProxyLogLevel::Warn => write!(f, "warn"),
            ProxyLogLevel::Info => write!(f, "info"),
            ProxyLogLevel::Debug => write!(f, "debug"),
            ProxyLogLevel::Trace => write!(f, "trace"),
        }
    }
}

/// Configuration for the development proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Port for the frontend dev server.
    pub frontend_port: u16,
    /// Host for the frontend dev server.
    pub frontend_host: String,
    /// Proxy targets.
    pub targets: Vec<ProxyTarget>,
    /// Logging level.
    pub log_level: ProxyLogLevel,
    /// Whether to enable colorized output.
    pub colorize: bool,
    /// Default timeout for proxied requests.
    pub default_timeout: Duration,
    /// Whether to follow redirects.
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow.
    pub max_redirects: u8,
    /// Whether to verify SSL certificates (disable for self-signed certs in dev).
    pub verify_ssl: bool,
    /// Global headers to add to all proxied requests.
    #[serde(default)]
    pub global_headers: HashMap<String, String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            frontend_port: 3000,
            frontend_host: "localhost".to_string(),
            targets: Vec::new(),
            log_level: ProxyLogLevel::Info,
            colorize: true,
            default_timeout: Duration::from_secs(30),
            follow_redirects: true,
            max_redirects: 5,
            verify_ssl: false, // Dev mode default
            global_headers: HashMap::new(),
        }
    }
}

impl ProxyConfig {
    /// Create a new proxy configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the frontend port.
    pub fn frontend_port(mut self, port: u16) -> Self {
        self.frontend_port = port;
        self
    }

    /// Set the frontend host.
    pub fn frontend_host(mut self, host: impl Into<String>) -> Self {
        self.frontend_host = host.into();
        self
    }

    /// Add a proxy target.
    pub fn add_target(mut self, target: ProxyTarget) -> Self {
        self.targets.push(target);
        self
    }

    /// Set the logging level.
    pub fn log_level(mut self, level: ProxyLogLevel) -> Self {
        self.log_level = level;
        self
    }

    /// Enable or disable SSL verification.
    pub fn verify_ssl(mut self, verify: bool) -> Self {
        self.verify_ssl = verify;
        self
    }

    /// Add a global header.
    pub fn with_global_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.global_headers.insert(key.into(), value.into());
        self
    }

    /// Parse a CLI proxy argument (e.g., "api=http://localhost:8000").
    pub fn parse_proxy_arg(arg: &str) -> Result<ProxyTarget, String> {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid proxy argument '{}'. Expected format: name=url",
                arg
            ));
        }

        let name = parts[0].trim();
        let url = parts[1].trim();

        if name.is_empty() {
            return Err("Proxy name cannot be empty".to_string());
        }

        if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("ws://") && !url.starts_with("wss://") {
            return Err(format!(
                "Invalid proxy URL '{}'. Must start with http://, https://, ws://, or wss://",
                url
            ));
        }

        let websocket = url.starts_with("ws://") || url.starts_with("wss://");

        Ok(ProxyTarget::new(name, url).websocket(websocket))
    }

    /// Find the target that matches a given path.
    pub fn find_target(&self, path: &str) -> Option<&ProxyTarget> {
        // Find the most specific match (longest prefix)
        self.targets
            .iter()
            .filter(|t| t.matches(path))
            .max_by_key(|t| t.path_prefix.len())
    }

    /// Get the frontend origin URL.
    pub fn frontend_origin(&self) -> String {
        format!("http://{}:{}", self.frontend_host, self.frontend_port)
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.targets.is_empty() {
            errors.push("No proxy targets configured".to_string());
        }

        for target in &self.targets {
            if target.name.is_empty() {
                errors.push("Proxy target name cannot be empty".to_string());
            }
            if target.target_url.is_empty() {
                errors.push(format!("Proxy target '{}' has empty URL", target.name));
            }
        }

        // Check for duplicate path prefixes
        let mut seen_prefixes = std::collections::HashSet::new();
        for target in &self.targets {
            if !seen_prefixes.insert(&target.path_prefix) {
                errors.push(format!(
                    "Duplicate path prefix '{}' in proxy configuration",
                    target.path_prefix
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Generate a summary of the proxy configuration.
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("Dev Proxy Configuration"),
            format!("  Frontend: {}", self.frontend_origin()),
            format!("  Targets:"),
        ];

        for target in &self.targets {
            let ws_marker = if target.websocket { " [WS]" } else { "" };
            lines.push(format!(
                "    {} -> {}{}",
                target.path_prefix, target.target_url, ws_marker
            ));
            if target.strip_prefix {
                lines.push(format!("      (prefix stripped)"));
            }
        }

        lines.join("\n")
    }
}

/// A logged proxy request for debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyLogEntry {
    /// Timestamp of the request.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// HTTP method.
    pub method: String,
    /// Original request path.
    pub original_path: String,
    /// Proxied URL.
    pub proxied_url: String,
    /// Response status code (if received).
    pub status: Option<u16>,
    /// Request duration in milliseconds.
    pub duration_ms: u64,
    /// Whether this was a WebSocket upgrade.
    pub websocket: bool,
    /// Error message if the request failed.
    pub error: Option<String>,
}

impl ProxyLogEntry {
    /// Create a new log entry for a starting request.
    pub fn new(method: &str, original_path: &str, proxied_url: &str, websocket: bool) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            method: method.to_string(),
            original_path: original_path.to_string(),
            proxied_url: proxied_url.to_string(),
            status: None,
            duration_ms: 0,
            websocket,
            error: None,
        }
    }

    /// Mark the request as completed.
    pub fn complete(mut self, status: u16, duration_ms: u64) -> Self {
        self.status = Some(status);
        self.duration_ms = duration_ms;
        self
    }

    /// Mark the request as failed.
    pub fn failed(mut self, error: impl Into<String>, duration_ms: u64) -> Self {
        self.error = Some(error.into());
        self.duration_ms = duration_ms;
        self
    }

    /// Format the log entry for display.
    pub fn format(&self, colorize: bool) -> String {
        let status_str = match &self.status {
            Some(s) if *s < 400 => {
                if colorize {
                    format!("\x1b[32m{}\x1b[0m", s)
                } else {
                    s.to_string()
                }
            }
            Some(s) if *s < 500 => {
                if colorize {
                    format!("\x1b[33m{}\x1b[0m", s)
                } else {
                    s.to_string()
                }
            }
            Some(s) => {
                if colorize {
                    format!("\x1b[31m{}\x1b[0m", s)
                } else {
                    s.to_string()
                }
            }
            None => {
                if let Some(err) = &self.error {
                    if colorize {
                        format!("\x1b[31mERR: {}\x1b[0m", err)
                    } else {
                        format!("ERR: {}", err)
                    }
                } else {
                    "---".to_string()
                }
            }
        };

        let ws_marker = if self.websocket { " [WS]" } else { "" };

        format!(
            "[{}] {} {} -> {} {} {}ms{}",
            self.timestamp.format("%H:%M:%S"),
            self.method,
            self.original_path,
            self.proxied_url,
            status_str,
            self.duration_ms,
            ws_marker
        )
    }
}

/// DevProxy represents a running development proxy instance.
/// This is the main interface for the proxy system.
#[derive(Debug)]
pub struct DevProxy {
    config: ProxyConfig,
    log_entries: Vec<ProxyLogEntry>,
}

impl DevProxy {
    /// Create a new DevProxy with the given configuration.
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config,
            log_entries: Vec::new(),
        }
    }

    /// Get the proxy configuration.
    pub fn config(&self) -> &ProxyConfig {
        &self.config
    }

    /// Get logged entries.
    pub fn log_entries(&self) -> &[ProxyLogEntry] {
        &self.log_entries
    }

    /// Clear log entries.
    pub fn clear_logs(&mut self) {
        self.log_entries.clear();
    }

    /// Add a log entry.
    pub fn add_log(&mut self, entry: ProxyLogEntry) {
        self.log_entries.push(entry);
    }

    /// Route a request path to the appropriate target.
    pub fn route(&self, path: &str) -> Option<(&ProxyTarget, String)> {
        self.config.find_target(path).map(|target| {
            let url = target.build_url(path);
            (target, url)
        })
    }

    /// Check if a path should be proxied.
    pub fn should_proxy(&self, path: &str) -> bool {
        self.config.find_target(path).is_some()
    }

    /// Get headers to add to a proxied request.
    pub fn get_proxy_headers(&self, target: &ProxyTarget) -> HashMap<String, String> {
        let mut headers = self.config.global_headers.clone();
        headers.extend(target.headers.clone());
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_target_matching() {
        let target = ProxyTarget::new("api", "http://localhost:8000").with_path("/api");

        assert!(target.matches("/api"));
        assert!(target.matches("/api/users"));
        assert!(target.matches("/api/users/123"));
        assert!(!target.matches("/auth"));
        assert!(!target.matches("/"));
    }

    #[test]
    fn test_proxy_target_path_transform() {
        let target = ProxyTarget::new("api", "http://localhost:8000")
            .with_path("/api")
            .strip_prefix(true);

        assert_eq!(target.transform_path("/api/users"), "/users");
        assert_eq!(target.transform_path("/api"), "/");

        let target_no_strip = ProxyTarget::new("api", "http://localhost:8000")
            .with_path("/api")
            .strip_prefix(false);

        assert_eq!(target_no_strip.transform_path("/api/users"), "/api/users");
    }

    #[test]
    fn test_proxy_target_build_url() {
        let target = ProxyTarget::new("api", "http://localhost:8000")
            .with_path("/api")
            .strip_prefix(true);

        assert_eq!(target.build_url("/api/users"), "http://localhost:8000/users");
        assert_eq!(target.build_url("/api/users/123"), "http://localhost:8000/users/123");
    }

    #[test]
    fn test_path_rewrite() {
        let rewrite = PathRewrite::new("/old/*", "/new/$1");
        assert_eq!(rewrite.apply("/old/path"), Some("/new/path".to_string()));
        assert_eq!(rewrite.apply("/old/path/deep"), Some("/new/path/deep".to_string()));
        assert_eq!(rewrite.apply("/other/path"), None);

        let exact_rewrite = PathRewrite::new("/exact", "/replaced");
        assert_eq!(exact_rewrite.apply("/exact"), Some("/replaced".to_string()));
        assert_eq!(exact_rewrite.apply("/exact/more"), None);
    }

    #[test]
    fn test_proxy_config_parse_arg() {
        let target = ProxyConfig::parse_proxy_arg("api=http://localhost:8000").unwrap();
        assert_eq!(target.name, "api");
        assert_eq!(target.target_url, "http://localhost:8000");
        assert!(!target.websocket);

        let ws_target = ProxyConfig::parse_proxy_arg("ws=ws://localhost:8001").unwrap();
        assert!(ws_target.websocket);

        assert!(ProxyConfig::parse_proxy_arg("invalid").is_err());
        assert!(ProxyConfig::parse_proxy_arg("name=invalid-url").is_err());
    }

    #[test]
    fn test_proxy_config_find_target() {
        let config = ProxyConfig::new()
            .add_target(ProxyTarget::new("api", "http://localhost:8000").with_path("/api"))
            .add_target(ProxyTarget::new("auth", "http://localhost:8001").with_path("/auth"))
            .add_target(ProxyTarget::new("api-v2", "http://localhost:8002").with_path("/api/v2"));

        // Should find api-v2 (more specific)
        let target = config.find_target("/api/v2/users").unwrap();
        assert_eq!(target.name, "api-v2");

        // Should find api
        let target = config.find_target("/api/users").unwrap();
        assert_eq!(target.name, "api");

        // Should find auth
        let target = config.find_target("/auth/login").unwrap();
        assert_eq!(target.name, "auth");

        // No match
        assert!(config.find_target("/other").is_none());
    }

    #[test]
    fn test_proxy_config_validation() {
        let empty_config = ProxyConfig::new();
        assert!(empty_config.validate().is_err());

        let valid_config = ProxyConfig::new()
            .add_target(ProxyTarget::new("api", "http://localhost:8000"));
        assert!(valid_config.validate().is_ok());

        let duplicate_config = ProxyConfig::new()
            .add_target(ProxyTarget::new("api1", "http://localhost:8000").with_path("/api"))
            .add_target(ProxyTarget::new("api2", "http://localhost:8001").with_path("/api"));
        assert!(duplicate_config.validate().is_err());
    }

    #[test]
    fn test_dev_proxy_routing() {
        let config = ProxyConfig::new()
            .add_target(ProxyTarget::new("api", "http://localhost:8000").with_path("/api"));

        let proxy = DevProxy::new(config);

        assert!(proxy.should_proxy("/api/users"));
        assert!(!proxy.should_proxy("/static/file.js"));

        let (target, url) = proxy.route("/api/users").unwrap();
        assert_eq!(target.name, "api");
        assert_eq!(url, "http://localhost:8000/users");
    }

    #[test]
    fn test_proxy_log_entry() {
        let entry = ProxyLogEntry::new("GET", "/api/users", "http://localhost:8000/users", false)
            .complete(200, 150);

        assert_eq!(entry.status, Some(200));
        assert_eq!(entry.duration_ms, 150);
        assert!(entry.error.is_none());

        let failed_entry = ProxyLogEntry::new("POST", "/api/data", "http://localhost:8000/data", false)
            .failed("Connection refused", 5);

        assert!(failed_entry.status.is_none());
        assert!(failed_entry.error.is_some());
    }

    #[test]
    fn test_proxy_config_summary() {
        let config = ProxyConfig::new()
            .frontend_port(3000)
            .add_target(ProxyTarget::new("api", "http://localhost:8000").with_path("/api"))
            .add_target(ProxyTarget::new("ws", "ws://localhost:8001").with_path("/ws").websocket(true));

        let summary = config.summary();
        assert!(summary.contains("Frontend: http://localhost:3000"));
        assert!(summary.contains("/api -> http://localhost:8000"));
        assert!(summary.contains("[WS]"));
    }
}
