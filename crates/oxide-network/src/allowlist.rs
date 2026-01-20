//! Network URL allowlist for OxideKit.
//!
//! Provides domain and URL-based access control for network requests.
//! Plugins can only make requests to URLs that match their declared allowlist.

use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// URL allowlist configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allowlist {
    /// Allowed domain patterns.
    #[serde(default)]
    patterns: Vec<AllowlistPattern>,
    /// Mode for the allowlist.
    #[serde(default)]
    mode: AllowlistMode,
    /// Whether to log blocked requests.
    #[serde(default = "default_true")]
    log_blocked: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Allowlist {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            mode: AllowlistMode::Strict,
            log_blocked: true,
        }
    }
}

/// Mode for allowlist operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AllowlistMode {
    /// Only explicitly allowed URLs are permitted.
    #[default]
    Strict,
    /// All URLs are allowed by default (use blocklist instead).
    Permissive,
    /// Audit mode - log but don't block.
    Audit,
}

/// A pattern in the allowlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowlistPattern {
    /// The pattern string.
    pub pattern: String,
    /// Type of pattern.
    pub pattern_type: PatternType,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this pattern allows WebSocket.
    #[serde(default)]
    pub allow_websocket: bool,
    /// Specific paths allowed (empty = all paths).
    #[serde(default)]
    pub allowed_paths: Vec<String>,
}

/// Type of allowlist pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    /// Exact domain match.
    #[default]
    Domain,
    /// Domain with wildcard subdomain support (*.example.com).
    WildcardDomain,
    /// Full URL prefix match.
    UrlPrefix,
    /// Regular expression pattern.
    Regex,
}

impl Allowlist {
    /// Create a new empty allowlist.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an allowlist that allows everything (permissive mode).
    pub fn allow_all() -> Self {
        Self {
            patterns: Vec::new(),
            mode: AllowlistMode::Permissive,
            log_blocked: false,
        }
    }

    /// Create an allowlist in audit mode.
    pub fn audit() -> Self {
        Self {
            patterns: Vec::new(),
            mode: AllowlistMode::Audit,
            log_blocked: true,
        }
    }

    /// Set the allowlist mode.
    pub fn with_mode(mut self, mode: AllowlistMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add a domain to the allowlist.
    pub fn allow_domain(mut self, domain: impl Into<String>) -> Self {
        self.patterns.push(AllowlistPattern {
            pattern: domain.into(),
            pattern_type: PatternType::Domain,
            description: None,
            allow_websocket: false,
            allowed_paths: Vec::new(),
        });
        self
    }

    /// Add a wildcard domain to the allowlist (e.g., *.example.com).
    pub fn allow_wildcard_domain(mut self, domain: impl Into<String>) -> Self {
        self.patterns.push(AllowlistPattern {
            pattern: domain.into(),
            pattern_type: PatternType::WildcardDomain,
            description: None,
            allow_websocket: false,
            allowed_paths: Vec::new(),
        });
        self
    }

    /// Add a URL prefix to the allowlist.
    pub fn allow_url_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.patterns.push(AllowlistPattern {
            pattern: prefix.into(),
            pattern_type: PatternType::UrlPrefix,
            description: None,
            allow_websocket: false,
            allowed_paths: Vec::new(),
        });
        self
    }

    /// Add a regex pattern to the allowlist.
    pub fn allow_regex(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(AllowlistPattern {
            pattern: pattern.into(),
            pattern_type: PatternType::Regex,
            description: None,
            allow_websocket: false,
            allowed_paths: Vec::new(),
        });
        self
    }

    /// Add a pattern with full configuration.
    pub fn add_pattern(mut self, pattern: AllowlistPattern) -> Self {
        self.patterns.push(pattern);
        self
    }

    /// Check if a URL is allowed.
    pub fn is_allowed(&self, url: &str) -> bool {
        let result = self.check_url(url);

        if !result && self.log_blocked {
            match self.mode {
                AllowlistMode::Strict => {
                    warn!(url = %url, "Request blocked by allowlist");
                }
                AllowlistMode::Audit => {
                    debug!(url = %url, "Request would be blocked (audit mode)");
                }
                AllowlistMode::Permissive => {}
            }
        }

        match self.mode {
            AllowlistMode::Strict => result,
            AllowlistMode::Permissive => true,
            AllowlistMode::Audit => true, // Allow but log
        }
    }

    /// Check if a WebSocket URL is allowed.
    pub fn is_websocket_allowed(&self, url: &str) -> bool {
        match self.mode {
            AllowlistMode::Permissive => return true,
            AllowlistMode::Audit => return true,
            AllowlistMode::Strict => {}
        }

        // Parse URL
        let parsed = match url::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let host = parsed.host_str().unwrap_or("");
        let path = parsed.path();

        for pattern in &self.patterns {
            if !pattern.allow_websocket {
                continue;
            }

            if self.matches_pattern(&parsed, host, path, pattern) {
                return true;
            }
        }

        false
    }

    /// Internal URL check without mode consideration.
    fn check_url(&self, url: &str) -> bool {
        if self.patterns.is_empty() && self.mode == AllowlistMode::Strict {
            return false;
        }

        // Parse URL
        let parsed = match url::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let host = parsed.host_str().unwrap_or("");
        let path = parsed.path();

        for pattern in &self.patterns {
            if self.matches_pattern(&parsed, host, path, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a URL matches a pattern.
    fn matches_pattern(
        &self,
        url: &url::Url,
        host: &str,
        path: &str,
        pattern: &AllowlistPattern,
    ) -> bool {
        // First check if the pattern matches
        let domain_matches = match pattern.pattern_type {
            PatternType::Domain => {
                host == pattern.pattern || host.ends_with(&format!(".{}", pattern.pattern))
            }
            PatternType::WildcardDomain => {
                let suffix = pattern.pattern.trim_start_matches("*.");
                host == suffix || host.ends_with(&format!(".{}", suffix))
            }
            PatternType::UrlPrefix => url.as_str().starts_with(&pattern.pattern),
            PatternType::Regex => {
                match Regex::new(&pattern.pattern) {
                    Ok(re) => re.is_match(url.as_str()),
                    Err(_) => false,
                }
            }
        };

        if !domain_matches {
            return false;
        }

        // Check path restrictions if any
        if !pattern.allowed_paths.is_empty() {
            let path_allowed = pattern.allowed_paths.iter().any(|allowed_path| {
                if allowed_path.ends_with("*") {
                    let prefix = &allowed_path[..allowed_path.len() - 1];
                    path.starts_with(prefix)
                } else {
                    path == allowed_path || path.starts_with(&format!("{}/", allowed_path))
                }
            });

            if !path_allowed {
                return false;
            }
        }

        true
    }

    /// Get all patterns.
    pub fn patterns(&self) -> &[AllowlistPattern] {
        &self.patterns
    }

    /// Get the allowlist mode.
    pub fn mode(&self) -> AllowlistMode {
        self.mode
    }

    /// Merge with another allowlist.
    pub fn merge(mut self, other: Allowlist) -> Self {
        self.patterns.extend(other.patterns);
        self
    }
}

/// Builder for allowlist patterns.
#[derive(Debug, Default)]
pub struct PatternBuilder {
    pattern: String,
    pattern_type: PatternType,
    description: Option<String>,
    allow_websocket: bool,
    allowed_paths: Vec<String>,
}

impl PatternBuilder {
    /// Create a new pattern builder for a domain.
    pub fn domain(domain: impl Into<String>) -> Self {
        Self {
            pattern: domain.into(),
            pattern_type: PatternType::Domain,
            ..Default::default()
        }
    }

    /// Create a new pattern builder for a wildcard domain.
    pub fn wildcard_domain(domain: impl Into<String>) -> Self {
        Self {
            pattern: domain.into(),
            pattern_type: PatternType::WildcardDomain,
            ..Default::default()
        }
    }

    /// Create a new pattern builder for a URL prefix.
    pub fn url_prefix(prefix: impl Into<String>) -> Self {
        Self {
            pattern: prefix.into(),
            pattern_type: PatternType::UrlPrefix,
            ..Default::default()
        }
    }

    /// Create a new pattern builder for a regex.
    pub fn regex(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            pattern_type: PatternType::Regex,
            ..Default::default()
        }
    }

    /// Add a description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Allow WebSocket connections.
    pub fn with_websocket(mut self) -> Self {
        self.allow_websocket = true;
        self
    }

    /// Restrict to specific paths.
    pub fn paths(mut self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.allowed_paths = paths.into_iter().map(|p| p.into()).collect();
        self
    }

    /// Build the pattern.
    pub fn build(self) -> AllowlistPattern {
        AllowlistPattern {
            pattern: self.pattern,
            pattern_type: self.pattern_type,
            description: self.description,
            allow_websocket: self.allow_websocket,
            allowed_paths: self.allowed_paths,
        }
    }
}

/// Predefined allowlists for common use cases.
pub mod presets {
    use super::*;

    /// Allowlist for common CDNs.
    pub fn cdns() -> Allowlist {
        Allowlist::new()
            .allow_wildcard_domain("*.cloudflare.com")
            .allow_wildcard_domain("*.cloudfront.net")
            .allow_wildcard_domain("*.akamaihd.net")
            .allow_wildcard_domain("*.jsdelivr.net")
            .allow_wildcard_domain("*.unpkg.com")
    }

    /// Allowlist for common API endpoints.
    pub fn common_apis() -> Allowlist {
        Allowlist::new()
            .allow_domain("api.github.com")
            .allow_domain("api.stripe.com")
            .allow_domain("api.openai.com")
    }

    /// Allowlist for Google services.
    pub fn google() -> Allowlist {
        Allowlist::new()
            .allow_wildcard_domain("*.googleapis.com")
            .allow_wildcard_domain("*.google.com")
            .allow_domain("accounts.google.com")
    }

    /// Allowlist for AWS services.
    pub fn aws() -> Allowlist {
        Allowlist::new()
            .allow_wildcard_domain("*.amazonaws.com")
            .allow_wildcard_domain("*.aws.amazon.com")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_matching() {
        let allowlist = Allowlist::new()
            .allow_domain("api.example.com")
            .allow_domain("example.org");

        assert!(allowlist.is_allowed("https://api.example.com/users"));
        assert!(allowlist.is_allowed("https://api.example.com"));
        assert!(allowlist.is_allowed("https://example.org/path"));
        assert!(!allowlist.is_allowed("https://evil.com"));
        assert!(!allowlist.is_allowed("https://notexample.com"));
    }

    #[test]
    fn test_wildcard_domain() {
        let allowlist = Allowlist::new()
            .allow_wildcard_domain("*.example.com");

        assert!(allowlist.is_allowed("https://api.example.com"));
        assert!(allowlist.is_allowed("https://www.example.com"));
        assert!(allowlist.is_allowed("https://sub.api.example.com"));
        assert!(allowlist.is_allowed("https://example.com")); // Base domain also matches
        assert!(!allowlist.is_allowed("https://notexample.com"));
    }

    #[test]
    fn test_url_prefix() {
        let allowlist = Allowlist::new()
            .allow_url_prefix("https://api.example.com/v1/");

        assert!(allowlist.is_allowed("https://api.example.com/v1/users"));
        assert!(allowlist.is_allowed("https://api.example.com/v1/posts/123"));
        assert!(!allowlist.is_allowed("https://api.example.com/v2/users"));
        assert!(!allowlist.is_allowed("https://api.example.com/"));
    }

    #[test]
    fn test_regex_pattern() {
        let allowlist = Allowlist::new()
            .allow_regex(r"https://[a-z]+\.example\.com/api/.*");

        assert!(allowlist.is_allowed("https://api.example.com/api/users"));
        assert!(allowlist.is_allowed("https://www.example.com/api/posts"));
        assert!(!allowlist.is_allowed("https://123.example.com/api/users"));
        assert!(!allowlist.is_allowed("https://api.example.com/other"));
    }

    #[test]
    fn test_path_restrictions() {
        let pattern = PatternBuilder::domain("api.example.com")
            .paths(["/v1/*", "/health"])
            .build();

        let allowlist = Allowlist::new().add_pattern(pattern);

        assert!(allowlist.is_allowed("https://api.example.com/v1/users"));
        assert!(allowlist.is_allowed("https://api.example.com/health"));
        assert!(!allowlist.is_allowed("https://api.example.com/v2/users"));
        assert!(!allowlist.is_allowed("https://api.example.com/other"));
    }

    #[test]
    fn test_permissive_mode() {
        let allowlist = Allowlist::allow_all();

        assert!(allowlist.is_allowed("https://anything.com"));
        assert!(allowlist.is_allowed("https://evil.com"));
    }

    #[test]
    fn test_websocket_restriction() {
        let pattern = PatternBuilder::domain("ws.example.com")
            .with_websocket()
            .build();

        let allowlist = Allowlist::new()
            .allow_domain("api.example.com")
            .add_pattern(pattern);

        // Regular requests allowed for both
        assert!(allowlist.is_allowed("https://api.example.com/users"));
        assert!(allowlist.is_allowed("wss://ws.example.com/stream"));

        // But WebSocket only allowed for ws.example.com
        assert!(allowlist.is_websocket_allowed("wss://ws.example.com/stream"));
        assert!(!allowlist.is_websocket_allowed("wss://api.example.com/stream"));
    }
}
