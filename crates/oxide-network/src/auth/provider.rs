//! Authentication provider traits and implementations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{AuthError, AuthState, NetworkResult};
use crate::http::HttpRequest;

use super::TokenPair;

/// Trait for authentication providers.
///
/// Providers handle the specifics of different auth mechanisms
/// (OAuth, JWT, API keys, etc.) while exposing a unified interface.
#[async_trait]
pub trait AuthProvider: Send + Sync + std::fmt::Debug {
    /// Get the provider's unique identifier.
    fn id(&self) -> &str;

    /// Get the provider type.
    fn provider_type(&self) -> AuthProviderType;

    /// Get the current authentication state.
    async fn state(&self) -> AuthState;

    /// Authenticate with the provider.
    ///
    /// The meaning of credentials depends on the provider type:
    /// - OAuth: May contain authorization code
    /// - JWT: Token to validate
    /// - API Key: The API key
    /// - Basic: username:password
    async fn authenticate(&self, credentials: AuthCredentials) -> NetworkResult<TokenPair>;

    /// Refresh the current token.
    async fn refresh(&self) -> NetworkResult<TokenPair>;

    /// Logout/invalidate the current session.
    async fn logout(&self) -> NetworkResult<()>;

    /// Apply authentication to an HTTP request.
    async fn apply_to_request(&self, request: HttpRequest) -> NetworkResult<HttpRequest>;

    /// Check if the provider can refresh tokens.
    fn can_refresh(&self) -> bool;

    /// Get the current token (if any).
    async fn current_token(&self) -> Option<TokenPair>;
}

/// Types of authentication providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthProviderType {
    /// OAuth 2.0 provider.
    OAuth2,
    /// JWT-based authentication.
    Jwt,
    /// API key authentication.
    ApiKey,
    /// Basic HTTP authentication.
    Basic,
    /// Custom/other authentication.
    Custom,
}

impl std::fmt::Display for AuthProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthProviderType::OAuth2 => write!(f, "OAuth 2.0"),
            AuthProviderType::Jwt => write!(f, "JWT"),
            AuthProviderType::ApiKey => write!(f, "API Key"),
            AuthProviderType::Basic => write!(f, "Basic Auth"),
            AuthProviderType::Custom => write!(f, "Custom"),
        }
    }
}

/// Credentials for authentication.
#[derive(Debug, Clone)]
pub enum AuthCredentials {
    /// Username and password.
    UsernamePassword {
        /// Username.
        username: String,
        /// Password.
        password: String,
    },
    /// OAuth authorization code.
    OAuthCode {
        /// Authorization code from OAuth flow.
        code: String,
        /// Redirect URI used in the flow.
        redirect_uri: String,
        /// Code verifier for PKCE.
        code_verifier: Option<String>,
    },
    /// Direct token (e.g., from SSO or refresh).
    Token {
        /// The token string.
        token: String,
    },
    /// API key.
    ApiKey {
        /// The API key.
        key: String,
        /// Header name to use (default: "X-API-Key").
        header_name: Option<String>,
    },
    /// Custom credentials.
    Custom(HashMap<String, String>),
}

impl AuthCredentials {
    /// Create username/password credentials.
    pub fn username_password(username: impl Into<String>, password: impl Into<String>) -> Self {
        AuthCredentials::UsernamePassword {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Create OAuth code credentials.
    pub fn oauth_code(
        code: impl Into<String>,
        redirect_uri: impl Into<String>,
        code_verifier: Option<String>,
    ) -> Self {
        AuthCredentials::OAuthCode {
            code: code.into(),
            redirect_uri: redirect_uri.into(),
            code_verifier,
        }
    }

    /// Create token credentials.
    pub fn token(token: impl Into<String>) -> Self {
        AuthCredentials::Token {
            token: token.into(),
        }
    }

    /// Create API key credentials.
    pub fn api_key(key: impl Into<String>) -> Self {
        AuthCredentials::ApiKey {
            key: key.into(),
            header_name: None,
        }
    }

    /// Create API key credentials with custom header name.
    pub fn api_key_with_header(key: impl Into<String>, header_name: impl Into<String>) -> Self {
        AuthCredentials::ApiKey {
            key: key.into(),
            header_name: Some(header_name.into()),
        }
    }
}

/// Configuration for OAuth 2.0 providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// Client ID.
    pub client_id: String,
    /// Client secret (optional for public clients).
    pub client_secret: Option<String>,
    /// Authorization endpoint URL.
    pub auth_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// User info endpoint URL (optional).
    pub userinfo_url: Option<String>,
    /// Revocation endpoint URL (optional).
    pub revoke_url: Option<String>,
    /// Redirect URI for the OAuth flow.
    pub redirect_uri: String,
    /// Scopes to request.
    pub scopes: Vec<String>,
    /// Whether to use PKCE (recommended).
    pub use_pkce: bool,
    /// Additional parameters for auth request.
    #[serde(default)]
    pub extra_auth_params: HashMap<String, String>,
}

impl OAuth2Config {
    /// Create a new OAuth2 config.
    pub fn new(
        client_id: impl Into<String>,
        auth_url: impl Into<String>,
        token_url: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: None,
            auth_url: auth_url.into(),
            token_url: token_url.into(),
            userinfo_url: None,
            revoke_url: None,
            redirect_uri: redirect_uri.into(),
            scopes: Vec::new(),
            use_pkce: true,
            extra_auth_params: HashMap::new(),
        }
    }

    /// Set the client secret.
    pub fn with_client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    /// Add scopes.
    pub fn with_scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scopes = scopes.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Set PKCE usage.
    pub fn with_pkce(mut self, use_pkce: bool) -> Self {
        self.use_pkce = use_pkce;
        self
    }
}

/// Configuration for JWT-based authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Expected issuer.
    pub issuer: Option<String>,
    /// Expected audience.
    pub audience: Option<String>,
    /// JWKS URI for key verification.
    pub jwks_uri: Option<String>,
    /// Secret for HMAC verification (not recommended for production).
    pub secret: Option<String>,
    /// Token refresh endpoint.
    pub refresh_url: Option<String>,
    /// Clock skew tolerance in seconds.
    pub clock_skew_secs: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            issuer: None,
            audience: None,
            jwks_uri: None,
            secret: None,
            refresh_url: None,
            clock_skew_secs: 60,
        }
    }
}

/// Configuration for API key authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Header name to use for the API key.
    pub header_name: String,
    /// Optional prefix for the key value.
    pub key_prefix: Option<String>,
    /// Whether to also accept the key as a query parameter.
    pub allow_query_param: bool,
    /// Query parameter name if allowed.
    pub query_param_name: Option<String>,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            header_name: "X-API-Key".to_string(),
            key_prefix: None,
            allow_query_param: false,
            query_param_name: None,
        }
    }
}

impl ApiKeyConfig {
    /// Create a config with a custom header name.
    pub fn with_header(header_name: impl Into<String>) -> Self {
        Self {
            header_name: header_name.into(),
            ..Default::default()
        }
    }

    /// Use Authorization header with Bearer prefix.
    pub fn bearer() -> Self {
        Self {
            header_name: "Authorization".to_string(),
            key_prefix: Some("Bearer ".to_string()),
            ..Default::default()
        }
    }
}

/// A simple API key provider implementation.
#[derive(Debug)]
pub struct ApiKeyProvider {
    id: String,
    config: ApiKeyConfig,
    api_key: tokio::sync::RwLock<Option<String>>,
}

impl ApiKeyProvider {
    /// Create a new API key provider.
    pub fn new(id: impl Into<String>, config: ApiKeyConfig) -> Self {
        Self {
            id: id.into(),
            config,
            api_key: tokio::sync::RwLock::new(None),
        }
    }

    /// Create a provider with a pre-set API key.
    pub fn with_key(id: impl Into<String>, config: ApiKeyConfig, key: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            config,
            api_key: tokio::sync::RwLock::new(Some(key.into())),
        }
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn provider_type(&self) -> AuthProviderType {
        AuthProviderType::ApiKey
    }

    async fn state(&self) -> AuthState {
        if self.api_key.read().await.is_some() {
            AuthState::Authenticated
        } else {
            AuthState::Unauthenticated
        }
    }

    async fn authenticate(&self, credentials: AuthCredentials) -> NetworkResult<TokenPair> {
        match credentials {
            AuthCredentials::ApiKey { key, .. } => {
                *self.api_key.write().await = Some(key.clone());
                Ok(TokenPair::new(key))
            }
            AuthCredentials::Token { token } => {
                *self.api_key.write().await = Some(token.clone());
                Ok(TokenPair::new(token))
            }
            _ => Err(AuthError::InvalidCredentials.into()),
        }
    }

    async fn refresh(&self) -> NetworkResult<TokenPair> {
        // API keys don't refresh
        Err(AuthError::InvalidCredentials.into())
    }

    async fn logout(&self) -> NetworkResult<()> {
        *self.api_key.write().await = None;
        Ok(())
    }

    async fn apply_to_request(&self, mut request: HttpRequest) -> NetworkResult<HttpRequest> {
        if let Some(key) = self.api_key.read().await.as_ref() {
            let value = if let Some(prefix) = &self.config.key_prefix {
                format!("{}{}", prefix, key)
            } else {
                key.clone()
            };
            request.headers.insert(self.config.header_name.clone(), value);
        }
        Ok(request)
    }

    fn can_refresh(&self) -> bool {
        false
    }

    async fn current_token(&self) -> Option<TokenPair> {
        self.api_key
            .read()
            .await
            .as_ref()
            .map(|k| TokenPair::new(k.clone()))
    }
}

/// A simple basic auth provider implementation.
#[derive(Debug)]
pub struct BasicAuthProvider {
    id: String,
    credentials: tokio::sync::RwLock<Option<(String, String)>>,
}

impl BasicAuthProvider {
    /// Create a new basic auth provider.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            credentials: tokio::sync::RwLock::new(None),
        }
    }

    /// Create a provider with pre-set credentials.
    pub fn with_credentials(
        id: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            credentials: tokio::sync::RwLock::new(Some((username.into(), password.into()))),
        }
    }
}

#[async_trait]
impl AuthProvider for BasicAuthProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn provider_type(&self) -> AuthProviderType {
        AuthProviderType::Basic
    }

    async fn state(&self) -> AuthState {
        if self.credentials.read().await.is_some() {
            AuthState::Authenticated
        } else {
            AuthState::Unauthenticated
        }
    }

    async fn authenticate(&self, credentials: AuthCredentials) -> NetworkResult<TokenPair> {
        match credentials {
            AuthCredentials::UsernamePassword { username, password } => {
                *self.credentials.write().await = Some((username.clone(), password.clone()));
                // Create a "token" from the encoded credentials
                let encoded = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    format!("{}:{}", username, password),
                );
                Ok(TokenPair::new(encoded).with_token_type("Basic"))
            }
            _ => Err(AuthError::InvalidCredentials.into()),
        }
    }

    async fn refresh(&self) -> NetworkResult<TokenPair> {
        // Basic auth doesn't refresh - just re-use existing credentials
        if let Some((username, password)) = self.credentials.read().await.as_ref() {
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                format!("{}:{}", username, password),
            );
            Ok(TokenPair::new(encoded).with_token_type("Basic"))
        } else {
            Err(AuthError::InvalidCredentials.into())
        }
    }

    async fn logout(&self) -> NetworkResult<()> {
        *self.credentials.write().await = None;
        Ok(())
    }

    async fn apply_to_request(&self, mut request: HttpRequest) -> NetworkResult<HttpRequest> {
        if let Some((username, password)) = self.credentials.read().await.as_ref() {
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                format!("{}:{}", username, password),
            );
            request
                .headers
                .insert("Authorization".to_string(), format!("Basic {}", encoded));
        }
        Ok(request)
    }

    fn can_refresh(&self) -> bool {
        false
    }

    async fn current_token(&self) -> Option<TokenPair> {
        self.credentials.read().await.as_ref().map(|(u, p)| {
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                format!("{}:{}", u, p),
            );
            TokenPair::new(encoded).with_token_type("Basic")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_key_provider() {
        let provider = ApiKeyProvider::new("test", ApiKeyConfig::default());

        assert_eq!(provider.state().await, AuthState::Unauthenticated);

        provider
            .authenticate(AuthCredentials::api_key("my-secret-key"))
            .await
            .unwrap();

        assert_eq!(provider.state().await, AuthState::Authenticated);

        let token = provider.current_token().await.unwrap();
        assert_eq!(token.access_token, "my-secret-key");
    }

    #[tokio::test]
    async fn test_basic_auth_provider() {
        let provider = BasicAuthProvider::new("test");

        provider
            .authenticate(AuthCredentials::username_password("user", "pass"))
            .await
            .unwrap();

        assert_eq!(provider.state().await, AuthState::Authenticated);

        let token = provider.current_token().await.unwrap();
        assert_eq!(token.token_type, "Basic");
    }
}
