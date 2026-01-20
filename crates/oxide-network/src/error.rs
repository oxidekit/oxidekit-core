//! Error types for the networking system.

use std::fmt;
use thiserror::Error;

/// Result type for networking operations.
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Errors that can occur during networking operations.
#[derive(Error, Debug)]
pub enum NetworkError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// URL parsing failed.
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// JSON serialization/deserialization failed.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Request blocked by allowlist policy.
    #[error("Request to '{url}' blocked: domain not in allowlist")]
    BlockedByAllowlist {
        /// The URL that was blocked.
        url: String,
    },

    /// Request requires authentication.
    #[error("Authentication required")]
    AuthenticationRequired,

    /// Authentication failed (invalid credentials).
    #[error("Authentication failed: {message}")]
    AuthenticationFailed {
        /// Detailed error message.
        message: String,
    },

    /// Token expired and could not be refreshed.
    #[error("Token expired and refresh failed")]
    TokenExpired,

    /// Token refresh failed.
    #[error("Token refresh failed: {message}")]
    RefreshFailed {
        /// Detailed error message.
        message: String,
    },

    /// Authorization failed (insufficient permissions).
    #[error("Authorization failed: insufficient permissions for {resource}")]
    Forbidden {
        /// The resource access was denied for.
        resource: String,
    },

    /// Network is offline.
    #[error("Network unavailable")]
    Offline,

    /// Connection timeout.
    #[error("Connection timeout after {duration_secs}s")]
    Timeout {
        /// Timeout duration in seconds.
        duration_secs: u64,
    },

    /// Maximum retry attempts exceeded.
    #[error("Max retries ({max_retries}) exceeded")]
    MaxRetriesExceeded {
        /// Maximum number of retries attempted.
        max_retries: u32,
    },

    /// WebSocket error.
    #[error("WebSocket error: {message}")]
    WebSocketError {
        /// Detailed error message.
        message: String,
    },

    /// WebSocket connection closed unexpectedly.
    #[error("WebSocket connection closed: {reason}")]
    WebSocketClosed {
        /// Close reason.
        reason: String,
        /// Close code if available.
        code: Option<u16>,
    },

    /// Credential storage error.
    #[error("Credential storage error: {message}")]
    CredentialError {
        /// Detailed error message.
        message: String,
    },

    /// Certificate validation failed.
    #[error("Certificate validation failed: {message}")]
    CertificateError {
        /// Detailed error message.
        message: String,
    },

    /// Request was cancelled.
    #[error("Request cancelled")]
    Cancelled,

    /// Invalid configuration.
    #[error("Invalid configuration: {message}")]
    ConfigError {
        /// Detailed error message.
        message: String,
    },

    /// Rate limited by server.
    #[error("Rate limited: retry after {retry_after_secs:?}s")]
    RateLimited {
        /// Seconds to wait before retrying, if provided.
        retry_after_secs: Option<u64>,
    },

    /// Server error (5xx status codes).
    #[error("Server error: HTTP {status}")]
    ServerError {
        /// HTTP status code.
        status: u16,
        /// Optional response body.
        body: Option<String>,
    },

    /// Client error (4xx status codes).
    #[error("Client error: HTTP {status}")]
    ClientError {
        /// HTTP status code.
        status: u16,
        /// Optional response body.
        body: Option<String>,
    },

    /// Invalid auth provider state.
    #[error("Invalid auth state: {message}")]
    InvalidAuthState {
        /// Detailed error message.
        message: String,
    },

    /// Missing required capability.
    #[error("Missing required capability: {capability}")]
    MissingCapability {
        /// The capability that is missing.
        capability: String,
    },

    /// Generic IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl NetworkError {
    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            NetworkError::Offline
                | NetworkError::Timeout { .. }
                | NetworkError::RateLimited { .. }
                | NetworkError::ServerError { .. }
        )
    }

    /// Check if this error is an auth-related error.
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            NetworkError::AuthenticationRequired
                | NetworkError::AuthenticationFailed { .. }
                | NetworkError::TokenExpired
                | NetworkError::RefreshFailed { .. }
                | NetworkError::Forbidden { .. }
        )
    }

    /// Check if this error indicates the network is unavailable.
    pub fn is_offline(&self) -> bool {
        matches!(self, NetworkError::Offline)
    }

    /// Get a suggested retry delay for this error in seconds.
    pub fn suggested_retry_delay(&self) -> Option<u64> {
        match self {
            NetworkError::RateLimited { retry_after_secs } => *retry_after_secs,
            NetworkError::Offline => Some(5),
            NetworkError::Timeout { .. } => Some(2),
            NetworkError::ServerError { .. } => Some(3),
            _ => None,
        }
    }
}

/// Auth-specific error type for more granular auth handling.
#[derive(Error, Debug, Clone)]
pub enum AuthError {
    /// Invalid credentials provided.
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Token has expired.
    #[error("Token expired at {expired_at}")]
    TokenExpired {
        /// When the token expired.
        expired_at: chrono::DateTime<chrono::Utc>,
    },

    /// Token is not yet valid.
    #[error("Token not yet valid until {valid_from}")]
    TokenNotYetValid {
        /// When the token becomes valid.
        valid_from: chrono::DateTime<chrono::Utc>,
    },

    /// Token signature verification failed.
    #[error("Token signature invalid")]
    InvalidSignature,

    /// Missing required claims in token.
    #[error("Missing required claim: {claim}")]
    MissingClaim {
        /// The missing claim name.
        claim: String,
    },

    /// Invalid issuer in token.
    #[error("Invalid token issuer: expected {expected}, got {actual}")]
    InvalidIssuer {
        /// Expected issuer.
        expected: String,
        /// Actual issuer.
        actual: String,
    },

    /// Invalid audience in token.
    #[error("Invalid token audience")]
    InvalidAudience,

    /// OAuth state mismatch (CSRF protection).
    #[error("OAuth state mismatch - possible CSRF attack")]
    OAuthStateMismatch,

    /// OAuth code exchange failed.
    #[error("OAuth code exchange failed: {message}")]
    OAuthCodeExchangeFailed {
        /// Detailed error message.
        message: String,
    },

    /// API key invalid or revoked.
    #[error("API key invalid or revoked")]
    InvalidApiKey,

    /// User account locked.
    #[error("Account locked: {reason}")]
    AccountLocked {
        /// Lock reason.
        reason: String,
    },

    /// MFA required but not provided.
    #[error("Multi-factor authentication required")]
    MfaRequired,

    /// Session expired.
    #[error("Session expired")]
    SessionExpired,
}

impl From<AuthError> for NetworkError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => NetworkError::AuthenticationFailed {
                message: "Invalid credentials".to_string(),
            },
            AuthError::TokenExpired { .. } => NetworkError::TokenExpired,
            AuthError::InvalidSignature => NetworkError::AuthenticationFailed {
                message: "Token signature verification failed".to_string(),
            },
            AuthError::InvalidApiKey => NetworkError::AuthenticationFailed {
                message: "Invalid API key".to_string(),
            },
            AuthError::SessionExpired => NetworkError::TokenExpired,
            other => NetworkError::AuthenticationFailed {
                message: other.to_string(),
            },
        }
    }
}

/// Auth state for tracking authentication lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    /// Not authenticated.
    Unauthenticated,
    /// Authentication in progress.
    Authenticating,
    /// Successfully authenticated.
    Authenticated,
    /// Token refresh in progress.
    Refreshing,
    /// Authentication expired.
    Expired,
    /// Authentication failed.
    Failed(String),
    /// Account requires action (MFA, password reset, etc.).
    ActionRequired(AuthAction),
}

impl fmt::Display for AuthState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthState::Unauthenticated => write!(f, "Unauthenticated"),
            AuthState::Authenticating => write!(f, "Authenticating"),
            AuthState::Authenticated => write!(f, "Authenticated"),
            AuthState::Refreshing => write!(f, "Refreshing"),
            AuthState::Expired => write!(f, "Expired"),
            AuthState::Failed(msg) => write!(f, "Failed: {}", msg),
            AuthState::ActionRequired(action) => write!(f, "Action required: {:?}", action),
        }
    }
}

impl AuthState {
    /// Check if the auth state indicates the user is authenticated.
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated | AuthState::Refreshing)
    }

    /// Check if the auth state indicates auth is in progress.
    pub fn is_in_progress(&self) -> bool {
        matches!(self, AuthState::Authenticating | AuthState::Refreshing)
    }

    /// Check if the auth state indicates a failure or expiration.
    pub fn needs_reauth(&self) -> bool {
        matches!(
            self,
            AuthState::Unauthenticated | AuthState::Expired | AuthState::Failed(_)
        )
    }
}

/// Actions that may be required during authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthAction {
    /// Multi-factor authentication required.
    Mfa,
    /// Password reset required.
    PasswordReset,
    /// Email verification required.
    EmailVerification,
    /// Terms of service acceptance required.
    AcceptTerms,
    /// Account setup required.
    AccountSetup,
    /// Custom action with description.
    Custom(String),
}
