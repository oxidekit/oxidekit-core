//! # OxideKit Networking & Auth System
//!
//! A comprehensive networking library for OxideKit applications providing:
//!
//! - **HTTP Client**: Full-featured async HTTP client with retry logic, interceptors,
//!   and automatic auth handling
//! - **WebSocket**: Real-time communication with automatic reconnection and message queuing
//! - **Authentication**: Pluggable auth providers (OAuth 2.0, JWT, API keys, Basic auth)
//! - **Credential Storage**: Secure platform-native credential storage (Keychain, etc.)
//! - **Capabilities**: Fine-grained permission system for network access
//! - **Allowlist**: URL-based access control for plugins
//! - **Offline Support**: Network status detection, retry policies, and request queuing
//! - **CORS Elimination**: Target-aware networking that eliminates CORS pain
//! - **Dev Proxy**: First-class development proxy for web apps
//! - **Reverse Proxy**: Production reverse proxy config generation (Nginx, Caddy, Traefik)
//! - **Diagnostics**: CORS doctor and network configuration diagnostics
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use oxide_network::prelude::*;
//!
//! // Create an HTTP client
//! let client = HttpClient::builder()
//!     .base_url("https://api.example.com")
//!     .timeout(Duration::from_secs(30))
//!     .build()?;
//!
//! // Make a request
//! let response = client.get("/users").await?;
//! let users: Vec<User> = response.json()?;
//!
//! // With authentication
//! let auth_manager = AuthManager::new();
//! auth_manager.register_provider(ApiKeyProvider::with_key(
//!     "default",
//!     ApiKeyConfig::bearer(),
//!     "my-api-key",
//! )).await;
//!
//! let client = HttpClient::builder()
//!     .auth_manager(Arc::new(auth_manager))
//!     .build()?;
//!
//! let response = client
//!     .execute(HttpRequest::get("/protected")?.with_auth())
//!     .await?;
//! ```
//!
//! ## CORS Elimination
//!
//! OxideKit provides target-aware networking to eliminate CORS pain:
//!
//! - **Desktop apps**: No CORS (ever) - direct HTTP to APIs
//! - **Web/static apps**: Same-origin preferred; dev proxy for development
//! - **Mobile apps**: No CORS - native HTTP stack
//!
//! ```rust,ignore
//! use oxide_network::network_mode::{NetworkConfig, TargetPlatform};
//! use oxide_network::proxy::{ProxyConfig, ProxyTarget};
//!
//! // Configure for web development
//! let mut config = NetworkConfig::for_target(TargetPlatform::Web);
//! config.set_dev_mode(true);
//!
//! // Set up dev proxy
//! let proxy = ProxyConfig::new()
//!     .frontend_port(3000)
//!     .add_target(ProxyTarget::new("api", "http://localhost:8000").with_path("/api"));
//! ```
//!
//! ## Auth Lifecycle
//!
//! The auth system follows a well-defined lifecycle:
//!
//! 1. **Unauthenticated** - No credentials present
//! 2. **Authenticating** - Login in progress
//! 3. **Authenticated** - Valid credentials available
//! 4. **Refreshing** - Token refresh in progress
//! 5. **Expired** - Token expired, needs re-auth
//! 6. **Failed** - Auth failed with error
//!
//! Subscribe to auth state changes:
//!
//! ```rust,ignore
//! let mut rx = auth_manager.subscribe_state_changes();
//! while let Ok(change) = rx.recv().await {
//!     match change.current {
//!         AuthState::Authenticated => show_dashboard(),
//!         AuthState::Expired => redirect_to_login(),
//!         AuthState::Failed(msg) => show_error(&msg),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## Capability System
//!
//! Plugins must declare network capabilities to access network features:
//!
//! ```toml
//! # In plugin oxide.toml
//! [capabilities.native.network]
//! allowed_urls = ["api.example.com", "*.internal.com"]
//! allowed_methods = ["GET", "POST"]
//! allow_websocket = true
//! ```
//!
//! ## Features
//!
//! - `websocket` - WebSocket support via tokio-tungstenite
//! - `oauth` - OAuth 2.0 provider support
//! - `jwt` - JWT validation and generation
//! - `keychain` - Platform keychain integration
//! - `cert-pinning` - Certificate pinning support
//! - `full` - All features enabled

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod allowlist;
pub mod auth;
pub mod capability;
pub mod cors;
pub mod credentials;
pub mod diagnostics;
pub mod error;
pub mod http;
pub mod interceptor;
pub mod network_mode;
pub mod offline;
pub mod proxy;
pub mod reverse_proxy;

#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(not(feature = "websocket"))]
pub mod websocket;

// Re-exports for convenient access
pub use allowlist::{Allowlist, AllowlistMode, AllowlistPattern, PatternBuilder, PatternType};
pub use auth::{
    ApiKeyConfig, ApiKeyProvider, AuthCredentials, AuthManager, AuthManagerBuilder, AuthProvider,
    AuthProviderType, BasicAuthProvider, JwtConfig, OAuth2Config, TokenPair,
};
pub use capability::{CapabilityValidator, CapabilityViolation, NetworkCapability};
pub use cors::{BackendFramework, CorsConfig, CorsPreset, CorsDiagnostic};
pub use credentials::{Credential, CredentialManager, CredentialStore, CredentialType};
pub use diagnostics::{DiagnosticIssue, DiagnosticReport, NetworkDoctor, Severity, IssueCategory};
pub use error::{AuthAction, AuthError, AuthState, NetworkError, NetworkResult};
pub use http::{
    HttpClient, HttpClientBuilder, HttpClientConfig, HttpMethod, HttpRequest, HttpResponse,
    RequestBody, ResponseBuilder, RetryConfig,
};
pub use interceptor::{
    CacheInterceptor, ErrorTransformInterceptor, HeaderInterceptor, Interceptor, InterceptorChain,
    LoggingInterceptor, MetricsInterceptor, RequestMetrics,
};
pub use network_mode::{ApiEndpoint, NetworkConfig, NetworkConfigBuilder, NetworkMode, TargetPlatform};
pub use offline::{
    MonitorHandle, NetworkStatus, OfflineDetector, OfflineDetectorConfig, QueueError, QueuedRequest,
    RequestQueue, RetryPolicy,
};
pub use proxy::{DevProxy, ProxyConfig, ProxyLogEntry, ProxyLogLevel, ProxyTarget, PathRewrite};
pub use reverse_proxy::{GeneratedConfig, ProxyServer, ReverseProxyConfig, SecurityHeaders, SslConfig, Upstream};
pub use websocket::{WsClient, WsClientBuilder, WsClientConfig, WsConnectionState, WsEvent, WsMessage};

/// Convenient re-exports for common usage patterns.
pub mod prelude {
    pub use crate::allowlist::{Allowlist, PatternBuilder};
    pub use crate::auth::{
        ApiKeyConfig, ApiKeyProvider, AuthCredentials, AuthManager, AuthProvider, TokenPair,
    };
    pub use crate::capability::{CapabilityValidator, NetworkCapability};
    pub use crate::cors::{CorsConfig, CorsPreset, BackendFramework};
    pub use crate::credentials::CredentialManager;
    pub use crate::diagnostics::{NetworkDoctor, DiagnosticReport};
    pub use crate::error::{AuthState, NetworkError, NetworkResult};
    pub use crate::http::{HttpClient, HttpRequest, HttpResponse, RetryConfig};
    pub use crate::interceptor::{HeaderInterceptor, Interceptor, LoggingInterceptor};
    pub use crate::network_mode::{NetworkConfig, NetworkMode, TargetPlatform};
    pub use crate::offline::{NetworkStatus, OfflineDetector, RetryPolicy};
    pub use crate::proxy::{DevProxy, ProxyConfig, ProxyTarget};
    pub use crate::reverse_proxy::{ReverseProxyConfig, ProxyServer};
    pub use crate::websocket::{WsClient, WsClientBuilder, WsMessage};
}

/// Contracts for consistent auth behavior across admin panels.
///
/// These contracts define how authentication state should affect
/// UI, routes, and components in OxideKit applications.
pub mod contracts {
    use super::error::AuthState;

    /// Contract: Auth state to route access mapping.
    ///
    /// Defines which routes are accessible in each auth state.
    pub trait AuthRouteContract {
        /// Check if a route is accessible in the given auth state.
        fn is_route_accessible(&self, route: &str, state: &AuthState) -> bool;

        /// Get the redirect route for inaccessible routes.
        fn redirect_for_state(&self, state: &AuthState) -> Option<&str>;

        /// Routes that are always accessible regardless of auth state.
        fn public_routes(&self) -> &[&str];

        /// Routes that require authentication.
        fn protected_routes(&self) -> &[&str];
    }

    /// Contract: Auth state to UI visibility mapping.
    ///
    /// Defines which UI elements should be visible based on auth state.
    pub trait AuthUiContract {
        /// Check if a UI element should be visible.
        fn is_visible(&self, element_id: &str, state: &AuthState) -> bool;

        /// Check if a UI element should be enabled (not just visible).
        fn is_enabled(&self, element_id: &str, state: &AuthState) -> bool;

        /// Get placeholder/loading state for an element.
        fn placeholder_for_state(&self, element_id: &str, state: &AuthState) -> Option<&str>;
    }

    /// Contract: Permission to UI element mapping.
    ///
    /// Maps user permissions to specific UI capabilities.
    pub trait PermissionUiContract {
        /// Check if a permission allows access to a UI element.
        fn has_permission(&self, element_id: &str, permissions: &[String]) -> bool;

        /// Get required permissions for a UI element.
        fn required_permissions(&self, element_id: &str) -> &[&str];

        /// Check if any of the given permissions grants access.
        fn any_permission_grants(&self, element_id: &str, permissions: &[String]) -> bool;
    }

    /// Contract: Token refresh behavior.
    ///
    /// Defines how token refresh should be handled.
    pub trait TokenRefreshContract {
        /// How long before expiry to trigger refresh (seconds).
        fn refresh_threshold_secs(&self) -> u64 {
            300 // 5 minutes default
        }

        /// Maximum number of refresh attempts before failing.
        fn max_refresh_attempts(&self) -> u32 {
            3
        }

        /// Whether to queue requests during refresh.
        fn queue_during_refresh(&self) -> bool {
            true
        }

        /// Whether to retry requests after successful refresh.
        fn retry_after_refresh(&self) -> bool {
            true
        }
    }

    /// Contract: Auth failure handling.
    ///
    /// Defines how auth failures should be handled consistently.
    pub trait AuthFailureContract {
        /// Handle 401 Unauthorized response.
        fn on_unauthorized(&self) -> FailureAction;

        /// Handle 403 Forbidden response.
        fn on_forbidden(&self) -> FailureAction;

        /// Handle token expired error.
        fn on_token_expired(&self) -> FailureAction;

        /// Handle refresh failure.
        fn on_refresh_failed(&self) -> FailureAction;

        /// Handle network error during auth.
        fn on_network_error(&self) -> FailureAction;
    }

    /// Actions to take on auth failure.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum FailureAction {
        /// Attempt to refresh the token.
        RefreshToken,
        /// Redirect to login.
        RedirectToLogin,
        /// Show an error message.
        ShowError(String),
        /// Retry the request.
        RetryRequest,
        /// Do nothing.
        None,
    }

    /// Default implementation of auth contracts for admin panels.
    #[derive(Debug, Default)]
    pub struct DefaultAdminContracts;

    impl AuthRouteContract for DefaultAdminContracts {
        fn is_route_accessible(&self, route: &str, state: &AuthState) -> bool {
            match state {
                AuthState::Authenticated => true,
                AuthState::Refreshing => true, // Allow during refresh
                _ => self.public_routes().contains(&route),
            }
        }

        fn redirect_for_state(&self, state: &AuthState) -> Option<&str> {
            match state {
                AuthState::Unauthenticated | AuthState::Expired | AuthState::Failed(_) => {
                    Some("/login")
                }
                AuthState::ActionRequired(_) => Some("/account/action-required"),
                _ => None,
            }
        }

        fn public_routes(&self) -> &[&str] {
            &["/login", "/forgot-password", "/reset-password", "/public"]
        }

        fn protected_routes(&self) -> &[&str] {
            &["/dashboard", "/settings", "/admin", "/users"]
        }
    }

    impl TokenRefreshContract for DefaultAdminContracts {
        fn refresh_threshold_secs(&self) -> u64 {
            300 // 5 minutes
        }

        fn max_refresh_attempts(&self) -> u32 {
            3
        }

        fn queue_during_refresh(&self) -> bool {
            true
        }

        fn retry_after_refresh(&self) -> bool {
            true
        }
    }

    impl AuthFailureContract for DefaultAdminContracts {
        fn on_unauthorized(&self) -> FailureAction {
            FailureAction::RefreshToken
        }

        fn on_forbidden(&self) -> FailureAction {
            FailureAction::ShowError("You don't have permission to access this resource.".into())
        }

        fn on_token_expired(&self) -> FailureAction {
            FailureAction::RefreshToken
        }

        fn on_refresh_failed(&self) -> FailureAction {
            FailureAction::RedirectToLogin
        }

        fn on_network_error(&self) -> FailureAction {
            FailureAction::RetryRequest
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_basic_http_client() {
        let client = HttpClient::new().unwrap();
        assert_eq!(client.active_request_count().await, 0);
    }

    #[tokio::test]
    async fn test_allowlist_integration() {
        let allowlist = Allowlist::new()
            .allow_domain("api.example.com")
            .allow_wildcard_domain("*.internal.com");

        assert!(allowlist.is_allowed("https://api.example.com/users"));
        assert!(allowlist.is_allowed("https://foo.internal.com/data"));
        assert!(!allowlist.is_allowed("https://evil.com/hack"));
    }

    #[tokio::test]
    async fn test_auth_manager_integration() {
        let manager = AuthManager::new();

        let provider = ApiKeyProvider::new("test", ApiKeyConfig::default());
        manager.register_provider(provider).await;

        let providers = manager.list_providers().await;
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].0, "test");
    }

    #[tokio::test]
    async fn test_capability_validation() {
        let capability = NetworkCapability::minimal()
            .allow_url("api.example.com")
            .allow_method("GET")
            .allow_method("POST");

        let validator = CapabilityValidator::new(capability);

        assert!(validator.is_url_allowed("https://api.example.com/users"));
        assert!(validator.is_method_allowed("GET"));
        assert!(!validator.is_method_allowed("DELETE"));
    }

    #[test]
    fn test_contracts() {
        use contracts::*;

        let contracts = DefaultAdminContracts;

        assert!(contracts.is_route_accessible("/dashboard", &AuthState::Authenticated));
        assert!(!contracts.is_route_accessible("/dashboard", &AuthState::Unauthenticated));
        assert!(contracts.is_route_accessible("/login", &AuthState::Unauthenticated));

        assert_eq!(
            contracts.redirect_for_state(&AuthState::Unauthenticated),
            Some("/login")
        );

        assert_eq!(contracts.on_unauthorized(), FailureAction::RefreshToken);
        assert_eq!(contracts.on_refresh_failed(), FailureAction::RedirectToLogin);
    }

    // CORS Elimination & Proxy Tests

    #[test]
    fn test_network_mode_integration() {
        // Desktop apps don't need CORS
        let config = NetworkConfig::for_target(TargetPlatform::Desktop);
        assert_eq!(config.mode, NetworkMode::Direct);
        assert!(!config.requires_cors_handling());
        assert!(!config.needs_dev_proxy());

        // Web apps need proxy in dev mode
        let mut web_config = NetworkConfig::for_target(TargetPlatform::Web);
        web_config.set_dev_mode(true);
        assert!(web_config.needs_dev_proxy());
    }

    #[test]
    fn test_proxy_config_integration() {
        let proxy = ProxyConfig::new()
            .frontend_port(3000)
            .add_target(
                ProxyTarget::new("api", "http://localhost:8000")
                    .with_path("/api")
                    .strip_prefix(true),
            );

        assert!(proxy.find_target("/api/users").is_some());
        assert!(proxy.find_target("/other").is_none());

        let target = proxy.find_target("/api/users").unwrap();
        assert_eq!(target.build_url("/api/users"), "http://localhost:8000/users");
    }

    #[test]
    fn test_reverse_proxy_generation() {
        let config = ReverseProxyConfig::new("https://app.example.com")
            .add_upstream("api", "http://localhost:8000", "/api")
            .frontend_upstream("http://localhost:3000");

        // Generate configs for different servers
        let nginx = config.generate(ProxyServer::Nginx);
        assert!(nginx.contains("upstream api"));
        assert!(nginx.contains("location /api"));

        let caddy = config.generate(ProxyServer::Caddy);
        assert!(caddy.contains("handle_path /api/*"));

        let traefik = config.generate(ProxyServer::Traefik);
        assert!(traefik.contains("api-router:"));
    }

    #[test]
    fn test_cors_config_integration() {
        // Development CORS
        let dev_cors = CorsConfig::for_development("http://localhost:3000");
        assert!(dev_cors.origins.contains(&"http://localhost:3000".to_string()));
        assert!(dev_cors.allow_credentials);

        // Public API CORS
        let public_cors = CorsConfig::for_public_api();
        assert!(public_cors.allow_all_origins);
        assert!(!public_cors.allow_credentials);

        // Generate FastAPI config
        let fastapi = dev_cors.generate(BackendFramework::FastApi);
        assert!(fastapi.contains("CORSMiddleware"));
        assert!(fastapi.contains("allow_credentials=True"));
    }

    #[test]
    fn test_diagnostics_integration() {
        let doctor = NetworkDoctor::new();

        // Test desktop config (should pass)
        let desktop_config = NetworkConfig::for_target(TargetPlatform::Desktop);
        let report = doctor.diagnose(&desktop_config);
        assert!(!report.has_errors());

        // Test web config with cross-origin but no origins (should fail)
        let mut web_config = NetworkConfig::for_target(TargetPlatform::Web);
        web_config.mode = NetworkMode::CrossOrigin;
        let report = doctor.diagnose(&web_config);
        assert!(report.has_errors());
    }

    #[test]
    fn test_proxy_parsing() {
        let target = ProxyConfig::parse_proxy_arg("api=http://localhost:8000").unwrap();
        assert_eq!(target.name, "api");
        assert_eq!(target.target_url, "http://localhost:8000");

        let ws_target = ProxyConfig::parse_proxy_arg("ws=ws://localhost:8001").unwrap();
        assert!(ws_target.websocket);

        assert!(ProxyConfig::parse_proxy_arg("invalid").is_err());
    }

    #[test]
    fn test_quick_diagnostics() {
        use diagnostics::quick_checks;

        // Web app without proxy in dev mode with API calls
        let issues = quick_checks::check_web_app(true, false, true);
        assert!(!issues.is_empty());

        // Web app with proxy
        let issues = quick_checks::check_web_app(true, true, true);
        assert!(issues.is_empty());

        // Recommendations
        let rec = quick_checks::get_recommendation(TargetPlatform::Desktop, false);
        assert!(rec.contains("don't need CORS"));
    }
}
