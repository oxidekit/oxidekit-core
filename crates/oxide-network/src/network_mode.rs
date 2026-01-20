//! Target-aware network modes for OxideKit applications.
//!
//! This module provides target-aware network configuration that eliminates
//! CORS pain by making networking boring:
//! - Desktop apps: no CORS (ever) - direct HTTP to APIs
//! - Web/static apps: same-origin preferred; proxy required for dev
//! - Mobile apps: no CORS (native HTTP stack)
//!
//! # Example
//!
//! ```rust
//! use oxide_network::network_mode::{NetworkMode, TargetPlatform, NetworkConfig};
//!
//! // Create config for desktop app
//! let config = NetworkConfig::for_target(TargetPlatform::Desktop);
//! assert!(!config.requires_cors_handling());
//! assert!(!config.needs_dev_proxy());
//!
//! // Create config for web app in dev mode
//! let mut config = NetworkConfig::for_target(TargetPlatform::Web);
//! config.set_dev_mode(true);
//! assert!(config.needs_dev_proxy());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// The target platform for the OxideKit application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TargetPlatform {
    /// Desktop application (Windows, macOS, Linux).
    /// No CORS restrictions - direct HTTP to any API.
    #[default]
    Desktop,

    /// Web application served by OxideKit's dev server or bundled.
    /// Subject to browser CORS policy - requires same-origin or proxy.
    Web,

    /// Static site deployment (served by external host).
    /// Subject to browser CORS policy - requires reverse proxy or edge routing.
    Static,

    /// Mobile application (iOS, Android).
    /// No CORS restrictions - native HTTP stack.
    Mobile,
}

impl TargetPlatform {
    /// Check if this platform is browser-based (subject to CORS).
    pub fn is_browser_based(&self) -> bool {
        matches!(self, TargetPlatform::Web | TargetPlatform::Static)
    }

    /// Check if this platform supports direct API access without CORS.
    pub fn supports_direct_api_access(&self) -> bool {
        matches!(self, TargetPlatform::Desktop | TargetPlatform::Mobile)
    }

    /// Get the recommended network mode for this platform.
    pub fn recommended_mode(&self) -> NetworkMode {
        match self {
            TargetPlatform::Desktop => NetworkMode::Direct,
            TargetPlatform::Mobile => NetworkMode::Direct,
            TargetPlatform::Web => NetworkMode::SameOrigin,
            TargetPlatform::Static => NetworkMode::SameOrigin,
        }
    }
}

impl fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetPlatform::Desktop => write!(f, "desktop"),
            TargetPlatform::Web => write!(f, "web"),
            TargetPlatform::Static => write!(f, "static"),
            TargetPlatform::Mobile => write!(f, "mobile"),
        }
    }
}

impl std::str::FromStr for TargetPlatform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "desktop" => Ok(TargetPlatform::Desktop),
            "web" => Ok(TargetPlatform::Web),
            "static" => Ok(TargetPlatform::Static),
            "mobile" => Ok(TargetPlatform::Mobile),
            _ => Err(format!(
                "Invalid target platform '{}'. Valid options: desktop, web, static, mobile",
                s
            )),
        }
    }
}

/// Network mode determining how API requests are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    /// Direct HTTP to APIs (no CORS handling needed).
    /// Used by desktop and mobile apps.
    #[default]
    Direct,

    /// Same-origin mode - API served from same origin as frontend.
    /// Eliminates CORS by design.
    SameOrigin,

    /// Proxied mode - dev proxy forwards API requests.
    /// Frontend at localhost:3000 proxies /api/* to backend.
    Proxied,

    /// Cross-origin mode - requires CORS configuration on backend.
    /// Not recommended; use only when other options are impossible.
    CrossOrigin,
}

impl NetworkMode {
    /// Check if this mode requires CORS handling on the backend.
    pub fn requires_cors(&self) -> bool {
        matches!(self, NetworkMode::CrossOrigin)
    }

    /// Check if this mode uses a development proxy.
    pub fn uses_proxy(&self) -> bool {
        matches!(self, NetworkMode::Proxied)
    }

    /// Get a human-readable description of this mode.
    pub fn description(&self) -> &'static str {
        match self {
            NetworkMode::Direct => "Direct HTTP to APIs (no CORS)",
            NetworkMode::SameOrigin => "Same-origin mode (CORS eliminated by design)",
            NetworkMode::Proxied => "Proxied mode (dev proxy forwards requests)",
            NetworkMode::CrossOrigin => "Cross-origin mode (requires CORS config)",
        }
    }
}

impl fmt::Display for NetworkMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkMode::Direct => write!(f, "direct"),
            NetworkMode::SameOrigin => write!(f, "same_origin"),
            NetworkMode::Proxied => write!(f, "proxied"),
            NetworkMode::CrossOrigin => write!(f, "cross_origin"),
        }
    }
}

/// API endpoint configuration for network routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    /// The name/alias for this endpoint (e.g., "api", "auth", "graphql").
    pub name: String,
    /// The target URL for this endpoint.
    pub url: String,
    /// Path prefix to match for this endpoint (e.g., "/api").
    pub path_prefix: String,
    /// Whether to strip the path prefix when forwarding.
    pub strip_prefix: bool,
    /// Custom headers to add to requests.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Whether this endpoint supports WebSocket connections.
    #[serde(default)]
    pub websocket: bool,
}

impl ApiEndpoint {
    /// Create a new API endpoint configuration.
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            path_prefix: format!("/{}", &name),
            name,
            url: url.into(),
            strip_prefix: true,
            headers: HashMap::new(),
            websocket: false,
        }
    }

    /// Set the path prefix for this endpoint.
    pub fn with_path_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.path_prefix = prefix.into();
        self
    }

    /// Set whether to strip the prefix when forwarding.
    pub fn with_strip_prefix(mut self, strip: bool) -> Self {
        self.strip_prefix = strip;
        self
    }

    /// Add a custom header to requests.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Enable WebSocket support for this endpoint.
    pub fn with_websocket(mut self, enabled: bool) -> Self {
        self.websocket = enabled;
        self
    }
}

/// Network configuration for an OxideKit application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// The target platform.
    pub target: TargetPlatform,
    /// The network mode (derived from target by default).
    pub mode: NetworkMode,
    /// Whether this is a development build.
    pub dev_mode: bool,
    /// API endpoints to configure.
    #[serde(default)]
    pub endpoints: Vec<ApiEndpoint>,
    /// The frontend origin (for dev mode).
    pub frontend_origin: Option<String>,
    /// The production origin (for reverse proxy config).
    pub production_origin: Option<String>,
    /// Allowed origins for CORS (if cross-origin mode is used).
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            target: TargetPlatform::Desktop,
            mode: NetworkMode::Direct,
            dev_mode: false,
            endpoints: Vec::new(),
            frontend_origin: None,
            production_origin: None,
            allowed_origins: Vec::new(),
        }
    }
}

impl NetworkConfig {
    /// Create a new network configuration for the given target.
    pub fn for_target(target: TargetPlatform) -> Self {
        Self {
            target,
            mode: target.recommended_mode(),
            ..Default::default()
        }
    }

    /// Set development mode.
    pub fn set_dev_mode(&mut self, dev: bool) {
        self.dev_mode = dev;
        // In dev mode, web/static targets use proxy
        if dev && self.target.is_browser_based() && self.mode == NetworkMode::SameOrigin {
            self.mode = NetworkMode::Proxied;
        }
    }

    /// Add an API endpoint.
    pub fn add_endpoint(&mut self, endpoint: ApiEndpoint) {
        self.endpoints.push(endpoint);
    }

    /// Check if this configuration requires CORS handling.
    pub fn requires_cors_handling(&self) -> bool {
        self.mode.requires_cors()
    }

    /// Check if this configuration needs a development proxy.
    pub fn needs_dev_proxy(&self) -> bool {
        self.dev_mode && self.target.is_browser_based()
    }

    /// Check if this configuration needs reverse proxy guidance for production.
    pub fn needs_reverse_proxy_guidance(&self) -> bool {
        !self.dev_mode && self.target.is_browser_based()
    }

    /// Get warnings for potential CORS issues.
    pub fn get_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Warn about cross-origin mode
        if self.mode == NetworkMode::CrossOrigin {
            warnings.push(
                "Cross-origin mode requires CORS configuration on your backend. \
                Consider using same-origin or proxied mode instead."
                    .to_string(),
            );
        }

        // Warn about web/static without proxy in dev
        if self.dev_mode
            && self.target.is_browser_based()
            && self.mode != NetworkMode::Proxied
            && !self.endpoints.is_empty()
        {
            warnings.push(format!(
                "Target '{}' in dev mode without proxy may cause CORS errors. \
                Run with --proxy option or configure endpoints for same-origin.",
                self.target
            ));
        }

        // Warn about missing production origin for static targets
        if !self.dev_mode
            && self.target == TargetPlatform::Static
            && self.production_origin.is_none()
        {
            warnings.push(
                "Static deployment without production_origin set. \
                Configure production_origin for reverse proxy guidance."
                    .to_string(),
            );
        }

        warnings
    }

    /// Validate the configuration and return errors if invalid.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for incompatible mode/target combinations
        if self.target.supports_direct_api_access() && self.mode == NetworkMode::CrossOrigin {
            errors.push(format!(
                "Cross-origin mode is unnecessary for {} target. Use direct mode.",
                self.target
            ));
        }

        // Check endpoints have valid URLs
        for endpoint in &self.endpoints {
            if !endpoint.url.starts_with("http://") && !endpoint.url.starts_with("https://") {
                errors.push(format!(
                    "Endpoint '{}' has invalid URL '{}'. URLs must start with http:// or https://",
                    endpoint.name, endpoint.url
                ));
            }
        }

        // Check allowed origins format
        for origin in &self.allowed_origins {
            if !origin.starts_with("http://") && !origin.starts_with("https://") && origin != "*" {
                errors.push(format!(
                    "Invalid allowed origin '{}'. Origins must be URLs or '*'.",
                    origin
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Builder for creating network configurations from oxide.toml.
#[derive(Debug, Default)]
pub struct NetworkConfigBuilder {
    target: Option<TargetPlatform>,
    mode: Option<NetworkMode>,
    dev_mode: bool,
    endpoints: Vec<ApiEndpoint>,
    frontend_origin: Option<String>,
    production_origin: Option<String>,
    allowed_origins: Vec<String>,
}

impl NetworkConfigBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target platform.
    pub fn target(mut self, target: TargetPlatform) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the network mode explicitly.
    pub fn mode(mut self, mode: NetworkMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Enable development mode.
    pub fn dev_mode(mut self, dev: bool) -> Self {
        self.dev_mode = dev;
        self
    }

    /// Add an API endpoint.
    pub fn endpoint(mut self, endpoint: ApiEndpoint) -> Self {
        self.endpoints.push(endpoint);
        self
    }

    /// Set the frontend origin for dev mode.
    pub fn frontend_origin(mut self, origin: impl Into<String>) -> Self {
        self.frontend_origin = Some(origin.into());
        self
    }

    /// Set the production origin.
    pub fn production_origin(mut self, origin: impl Into<String>) -> Self {
        self.production_origin = Some(origin.into());
        self
    }

    /// Add an allowed CORS origin.
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// Build the network configuration.
    pub fn build(self) -> NetworkConfig {
        let target = self.target.unwrap_or_default();
        let mode = self.mode.unwrap_or_else(|| {
            if self.dev_mode && target.is_browser_based() {
                NetworkMode::Proxied
            } else {
                target.recommended_mode()
            }
        });

        NetworkConfig {
            target,
            mode,
            dev_mode: self.dev_mode,
            endpoints: self.endpoints,
            frontend_origin: self.frontend_origin,
            production_origin: self.production_origin,
            allowed_origins: self.allowed_origins,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_platform_browser_based() {
        assert!(!TargetPlatform::Desktop.is_browser_based());
        assert!(TargetPlatform::Web.is_browser_based());
        assert!(TargetPlatform::Static.is_browser_based());
        assert!(!TargetPlatform::Mobile.is_browser_based());
    }

    #[test]
    fn test_target_platform_direct_access() {
        assert!(TargetPlatform::Desktop.supports_direct_api_access());
        assert!(!TargetPlatform::Web.supports_direct_api_access());
        assert!(!TargetPlatform::Static.supports_direct_api_access());
        assert!(TargetPlatform::Mobile.supports_direct_api_access());
    }

    #[test]
    fn test_network_mode_cors() {
        assert!(!NetworkMode::Direct.requires_cors());
        assert!(!NetworkMode::SameOrigin.requires_cors());
        assert!(!NetworkMode::Proxied.requires_cors());
        assert!(NetworkMode::CrossOrigin.requires_cors());
    }

    #[test]
    fn test_config_for_desktop() {
        let config = NetworkConfig::for_target(TargetPlatform::Desktop);
        assert_eq!(config.target, TargetPlatform::Desktop);
        assert_eq!(config.mode, NetworkMode::Direct);
        assert!(!config.requires_cors_handling());
        assert!(!config.needs_dev_proxy());
    }

    #[test]
    fn test_config_for_web_dev() {
        let mut config = NetworkConfig::for_target(TargetPlatform::Web);
        config.set_dev_mode(true);
        assert_eq!(config.mode, NetworkMode::Proxied);
        assert!(config.needs_dev_proxy());
    }

    #[test]
    fn test_config_builder() {
        let config = NetworkConfigBuilder::new()
            .target(TargetPlatform::Web)
            .dev_mode(true)
            .endpoint(ApiEndpoint::new("api", "http://localhost:8000"))
            .frontend_origin("http://localhost:3000")
            .build();

        assert_eq!(config.target, TargetPlatform::Web);
        assert_eq!(config.mode, NetworkMode::Proxied);
        assert!(config.dev_mode);
        assert_eq!(config.endpoints.len(), 1);
    }

    #[test]
    fn test_api_endpoint() {
        let endpoint = ApiEndpoint::new("api", "http://localhost:8000")
            .with_path_prefix("/v1/api")
            .with_websocket(true)
            .with_header("X-Custom", "value");

        assert_eq!(endpoint.name, "api");
        assert_eq!(endpoint.path_prefix, "/v1/api");
        assert!(endpoint.websocket);
        assert_eq!(endpoint.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = NetworkConfig::for_target(TargetPlatform::Desktop);
        assert!(config.validate().is_ok());

        // Invalid: cross-origin on desktop
        let mut config = NetworkConfig::for_target(TargetPlatform::Desktop);
        config.mode = NetworkMode::CrossOrigin;
        assert!(config.validate().is_err());

        // Invalid: bad endpoint URL
        let mut config = NetworkConfig::for_target(TargetPlatform::Web);
        config.endpoints.push(ApiEndpoint::new("api", "localhost:8000"));
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_warnings() {
        let mut config = NetworkConfig::for_target(TargetPlatform::Web);
        config.mode = NetworkMode::CrossOrigin;
        let warnings = config.get_warnings();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("Cross-origin"));
    }

    #[test]
    fn test_target_platform_parsing() {
        assert_eq!("desktop".parse::<TargetPlatform>().unwrap(), TargetPlatform::Desktop);
        assert_eq!("web".parse::<TargetPlatform>().unwrap(), TargetPlatform::Web);
        assert_eq!("static".parse::<TargetPlatform>().unwrap(), TargetPlatform::Static);
        assert_eq!("mobile".parse::<TargetPlatform>().unwrap(), TargetPlatform::Mobile);
        assert!("invalid".parse::<TargetPlatform>().is_err());
    }
}
