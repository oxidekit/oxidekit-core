//! Standardized authentication contracts
//!
//! Provides the OxideKit standard auth model:
//! - Access tokens (short-lived)
//! - Refresh tokens (rotating)
//! - Standard endpoints (/auth/login, /auth/refresh, /auth/logout, etc.)
//! - Consistent error responses
//! - Token handling rules

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// Token configuration
    pub tokens: TokenConfig,
    /// Required endpoints
    pub endpoints: Vec<AuthEndpoint>,
    /// Auth flow configuration
    pub flow: AuthFlow,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tokens: TokenConfig::default(),
            endpoints: AuthEndpoint::standard_endpoints(),
            flow: AuthFlow::default(),
        }
    }
}

impl AuthConfig {
    /// Create a disabled auth configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create auth config with custom token settings
    pub fn with_tokens(mut self, tokens: TokenConfig) -> Self {
        self.tokens = tokens;
        self
    }

    /// Add an endpoint to the configuration
    pub fn with_endpoint(mut self, endpoint: AuthEndpoint) -> Self {
        self.endpoints.push(endpoint);
        self
    }
}

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Access token expiration in seconds
    pub access_token_expire_seconds: u64,
    /// Refresh token expiration in seconds
    pub refresh_token_expire_seconds: u64,
    /// JWT algorithm to use
    pub algorithm: TokenAlgorithm,
    /// Enable refresh token rotation
    pub rotate_refresh_tokens: bool,
    /// Detect refresh token reuse (security)
    pub detect_reuse: bool,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            access_token_expire_seconds: 900,        // 15 minutes
            refresh_token_expire_seconds: 604800,    // 7 days
            algorithm: TokenAlgorithm::HS256,
            rotate_refresh_tokens: true,
            detect_reuse: true,
        }
    }
}

impl TokenConfig {
    /// Get access token expiration as Duration
    pub fn access_token_duration(&self) -> Duration {
        Duration::from_secs(self.access_token_expire_seconds)
    }

    /// Get refresh token expiration as Duration
    pub fn refresh_token_duration(&self) -> Duration {
        Duration::from_secs(self.refresh_token_expire_seconds)
    }

    /// Set access token expiration in minutes
    pub fn with_access_token_minutes(mut self, minutes: u64) -> Self {
        self.access_token_expire_seconds = minutes * 60;
        self
    }

    /// Set refresh token expiration in days
    pub fn with_refresh_token_days(mut self, days: u64) -> Self {
        self.refresh_token_expire_seconds = days * 24 * 60 * 60;
        self
    }
}

/// JWT algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TokenAlgorithm {
    /// HMAC with SHA-256
    #[default]
    HS256,
    /// HMAC with SHA-384
    HS384,
    /// HMAC with SHA-512
    HS512,
    /// RSA with SHA-256
    RS256,
    /// RSA with SHA-384
    RS384,
    /// RSA with SHA-512
    RS512,
    /// ECDSA with P-256 and SHA-256
    ES256,
    /// ECDSA with P-384 and SHA-384
    ES384,
}

impl TokenAlgorithm {
    /// Get the algorithm name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenAlgorithm::HS256 => "HS256",
            TokenAlgorithm::HS384 => "HS384",
            TokenAlgorithm::HS512 => "HS512",
            TokenAlgorithm::RS256 => "RS256",
            TokenAlgorithm::RS384 => "RS384",
            TokenAlgorithm::RS512 => "RS512",
            TokenAlgorithm::ES256 => "ES256",
            TokenAlgorithm::ES384 => "ES384",
        }
    }

    /// Check if algorithm uses symmetric key
    pub fn is_symmetric(&self) -> bool {
        matches!(
            self,
            TokenAlgorithm::HS256 | TokenAlgorithm::HS384 | TokenAlgorithm::HS512
        )
    }
}

/// Standard authentication endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEndpoint {
    /// Endpoint path
    pub path: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Endpoint type
    pub endpoint_type: AuthEndpointType,
    /// Whether the endpoint requires authentication
    pub requires_auth: bool,
    /// Description
    pub description: String,
}

impl AuthEndpoint {
    /// Create a new auth endpoint
    pub fn new(
        path: impl Into<String>,
        method: HttpMethod,
        endpoint_type: AuthEndpointType,
        requires_auth: bool,
    ) -> Self {
        Self {
            path: path.into(),
            method,
            endpoint_type,
            requires_auth,
            description: endpoint_type.default_description().to_string(),
        }
    }

    /// Get the standard OxideKit auth endpoints
    pub fn standard_endpoints() -> Vec<Self> {
        vec![
            Self::new("/auth/login", HttpMethod::Post, AuthEndpointType::Login, false),
            Self::new("/auth/refresh", HttpMethod::Post, AuthEndpointType::Refresh, false),
            Self::new("/auth/logout", HttpMethod::Post, AuthEndpointType::Logout, true),
            Self::new("/auth/revoke", HttpMethod::Post, AuthEndpointType::Revoke, true),
            Self::new("/auth/me", HttpMethod::Get, AuthEndpointType::Me, true),
        ]
    }
}

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Types of authentication endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthEndpointType {
    /// Login endpoint - returns access and refresh tokens
    Login,
    /// Refresh endpoint - exchanges refresh token for new tokens
    Refresh,
    /// Logout endpoint - invalidates current session
    Logout,
    /// Revoke endpoint - invalidates all sessions for user
    Revoke,
    /// Me endpoint - returns current user info
    Me,
    /// Register endpoint - creates new user account
    Register,
    /// Password reset request
    PasswordResetRequest,
    /// Password reset confirm
    PasswordResetConfirm,
    /// Email verification
    VerifyEmail,
}

impl AuthEndpointType {
    /// Get the default description for this endpoint type
    pub fn default_description(&self) -> &'static str {
        match self {
            AuthEndpointType::Login => "Authenticate with credentials and receive tokens",
            AuthEndpointType::Refresh => "Exchange refresh token for new access token",
            AuthEndpointType::Logout => "Invalidate current session",
            AuthEndpointType::Revoke => "Revoke all sessions for the current user",
            AuthEndpointType::Me => "Get current authenticated user information",
            AuthEndpointType::Register => "Create a new user account",
            AuthEndpointType::PasswordResetRequest => "Request a password reset email",
            AuthEndpointType::PasswordResetConfirm => "Confirm password reset with token",
            AuthEndpointType::VerifyEmail => "Verify email address with token",
        }
    }
}

/// Authentication flow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthFlow {
    /// Allow password-based authentication
    pub password_auth: bool,
    /// Allow OAuth providers
    pub oauth_providers: Vec<OAuthProvider>,
    /// Allow API key authentication
    pub api_key_auth: bool,
    /// Require email verification
    pub require_email_verification: bool,
    /// Allow remember me (longer refresh tokens)
    pub allow_remember_me: bool,
    /// Maximum concurrent sessions per user (0 = unlimited)
    pub max_sessions: u32,
}

impl Default for AuthFlow {
    fn default() -> Self {
        Self {
            password_auth: true,
            oauth_providers: Vec::new(),
            api_key_auth: false,
            require_email_verification: false,
            allow_remember_me: true,
            max_sessions: 0,
        }
    }
}

/// Supported OAuth providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    GitHub,
    Apple,
    Microsoft,
    Facebook,
    Twitter,
    Custom(String),
}

impl OAuthProvider {
    /// Get the provider name
    pub fn name(&self) -> &str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::GitHub => "github",
            OAuthProvider::Apple => "apple",
            OAuthProvider::Microsoft => "microsoft",
            OAuthProvider::Facebook => "facebook",
            OAuthProvider::Twitter => "twitter",
            OAuthProvider::Custom(name) => name,
        }
    }
}

/// Standard error response for auth endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthError {
    /// Error code (machine-readable)
    pub code: AuthErrorCode,
    /// Error message (human-readable)
    pub message: String,
    /// Additional details (optional)
    pub details: Option<serde_json::Value>,
}

impl AuthError {
    /// Create a new auth error
    pub fn new(code: AuthErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Standard auth error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthErrorCode {
    /// Invalid credentials provided
    InvalidCredentials,
    /// Token has expired
    TokenExpired,
    /// Token is invalid or malformed
    InvalidToken,
    /// Refresh token was already used (security issue)
    TokenReused,
    /// Session has been revoked
    SessionRevoked,
    /// User account is not verified
    UnverifiedAccount,
    /// User account is locked
    AccountLocked,
    /// Too many login attempts
    TooManyAttempts,
    /// Missing required field
    MissingField,
    /// Invalid field format
    InvalidFormat,
    /// Internal server error
    InternalError,
}

impl AuthErrorCode {
    /// Get the HTTP status code for this error
    pub fn http_status(&self) -> u16 {
        match self {
            AuthErrorCode::InvalidCredentials => 401,
            AuthErrorCode::TokenExpired => 401,
            AuthErrorCode::InvalidToken => 401,
            AuthErrorCode::TokenReused => 401,
            AuthErrorCode::SessionRevoked => 401,
            AuthErrorCode::UnverifiedAccount => 403,
            AuthErrorCode::AccountLocked => 403,
            AuthErrorCode::TooManyAttempts => 429,
            AuthErrorCode::MissingField => 400,
            AuthErrorCode::InvalidFormat => 400,
            AuthErrorCode::InternalError => 500,
        }
    }
}

// These structs use camelCase field names intentionally to match the API contract
// The naming is enforced by OxideKit's naming policy (camelCase for JSON)

/// Standard login request schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct LoginRequest {
    /// User email
    pub email: String,
    /// User password
    pub password: String,
    /// Remember me flag (optional)
    #[serde(default)]
    pub rememberMe: bool,
}

/// Standard login response schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct LoginResponse {
    /// Access token (short-lived)
    pub accessToken: String,
    /// Refresh token (longer-lived)
    pub refreshToken: String,
    /// Token type (always "Bearer")
    pub tokenType: String,
    /// Seconds until access token expires
    pub expiresIn: u64,
}

/// Standard refresh request schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct RefreshRequest {
    /// The refresh token to exchange
    pub refreshToken: String,
}

/// Standard refresh response schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct RefreshResponse {
    /// New access token
    pub accessToken: String,
    /// New refresh token (rotated)
    pub refreshToken: String,
    /// Token type
    pub tokenType: String,
    /// Seconds until access token expires
    pub expiresIn: u64,
}

/// Standard user response schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct UserResponse {
    /// User ID
    pub id: String,
    /// User email
    pub email: String,
    /// Account creation timestamp
    pub createdAt: String,
    /// Last update timestamp
    pub updatedAt: String,
}

/// Generate OpenAPI schemas for auth endpoints
pub fn generate_auth_schemas() -> serde_json::Value {
    serde_json::json!({
        "LoginRequest": {
            "type": "object",
            "required": ["email", "password"],
            "properties": {
                "email": {
                    "type": "string",
                    "format": "email"
                },
                "password": {
                    "type": "string",
                    "minLength": 1
                },
                "rememberMe": {
                    "type": "boolean",
                    "default": false
                }
            }
        },
        "LoginResponse": {
            "type": "object",
            "required": ["accessToken", "refreshToken", "tokenType", "expiresIn"],
            "properties": {
                "accessToken": {
                    "type": "string"
                },
                "refreshToken": {
                    "type": "string"
                },
                "tokenType": {
                    "type": "string",
                    "enum": ["Bearer"]
                },
                "expiresIn": {
                    "type": "integer",
                    "description": "Seconds until access token expires"
                }
            }
        },
        "RefreshRequest": {
            "type": "object",
            "required": ["refreshToken"],
            "properties": {
                "refreshToken": {
                    "type": "string"
                }
            }
        },
        "RefreshResponse": {
            "type": "object",
            "required": ["accessToken", "refreshToken", "tokenType", "expiresIn"],
            "properties": {
                "accessToken": {
                    "type": "string"
                },
                "refreshToken": {
                    "type": "string"
                },
                "tokenType": {
                    "type": "string",
                    "enum": ["Bearer"]
                },
                "expiresIn": {
                    "type": "integer"
                }
            }
        },
        "UserResponse": {
            "type": "object",
            "required": ["id", "email", "createdAt", "updatedAt"],
            "properties": {
                "id": {
                    "type": "string"
                },
                "email": {
                    "type": "string",
                    "format": "email"
                },
                "createdAt": {
                    "type": "string",
                    "format": "date-time"
                },
                "updatedAt": {
                    "type": "string",
                    "format": "date-time"
                }
            }
        },
        "AuthError": {
            "type": "object",
            "required": ["code", "message"],
            "properties": {
                "code": {
                    "type": "string",
                    "enum": [
                        "INVALID_CREDENTIALS",
                        "TOKEN_EXPIRED",
                        "INVALID_TOKEN",
                        "TOKEN_REUSED",
                        "SESSION_REVOKED",
                        "UNVERIFIED_ACCOUNT",
                        "ACCOUNT_LOCKED",
                        "TOO_MANY_ATTEMPTS",
                        "MISSING_FIELD",
                        "INVALID_FORMAT",
                        "INTERNAL_ERROR"
                    ]
                },
                "message": {
                    "type": "string"
                },
                "details": {
                    "type": "object",
                    "additionalProperties": true
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert!(config.enabled);
        assert_eq!(config.endpoints.len(), 5);
    }

    #[test]
    fn test_auth_config_disabled() {
        let config = AuthConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_token_config_default() {
        let config = TokenConfig::default();
        assert_eq!(config.access_token_expire_seconds, 900);
        assert_eq!(config.refresh_token_expire_seconds, 604800);
        assert!(config.rotate_refresh_tokens);
    }

    #[test]
    fn test_token_config_duration() {
        let config = TokenConfig::default()
            .with_access_token_minutes(30)
            .with_refresh_token_days(14);

        assert_eq!(config.access_token_duration(), Duration::from_secs(1800));
        assert_eq!(
            config.refresh_token_duration(),
            Duration::from_secs(14 * 24 * 60 * 60)
        );
    }

    #[test]
    fn test_standard_endpoints() {
        let endpoints = AuthEndpoint::standard_endpoints();
        assert_eq!(endpoints.len(), 5);

        let paths: Vec<_> = endpoints.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"/auth/login"));
        assert!(paths.contains(&"/auth/refresh"));
        assert!(paths.contains(&"/auth/logout"));
        assert!(paths.contains(&"/auth/revoke"));
        assert!(paths.contains(&"/auth/me"));
    }

    #[test]
    fn test_auth_error_http_status() {
        assert_eq!(AuthErrorCode::InvalidCredentials.http_status(), 401);
        assert_eq!(AuthErrorCode::AccountLocked.http_status(), 403);
        assert_eq!(AuthErrorCode::TooManyAttempts.http_status(), 429);
        assert_eq!(AuthErrorCode::MissingField.http_status(), 400);
    }

    #[test]
    fn test_token_algorithm() {
        assert_eq!(TokenAlgorithm::HS256.as_str(), "HS256");
        assert!(TokenAlgorithm::HS256.is_symmetric());
        assert!(!TokenAlgorithm::RS256.is_symmetric());
    }

    #[test]
    fn test_oauth_provider_name() {
        assert_eq!(OAuthProvider::Google.name(), "google");
        assert_eq!(OAuthProvider::GitHub.name(), "github");
        assert_eq!(OAuthProvider::Custom("custom-provider".to_string()).name(), "custom-provider");
    }

    #[test]
    fn test_generate_auth_schemas() {
        let schemas = generate_auth_schemas();
        assert!(schemas.get("LoginRequest").is_some());
        assert!(schemas.get("LoginResponse").is_some());
        assert!(schemas.get("RefreshRequest").is_some());
        assert!(schemas.get("RefreshResponse").is_some());
        assert!(schemas.get("UserResponse").is_some());
        assert!(schemas.get("AuthError").is_some());
    }
}
