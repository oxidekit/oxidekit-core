//! Token types and utilities.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Represents an access token with optional refresh capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// The access token.
    pub access_token: String,
    /// Optional refresh token.
    pub refresh_token: Option<String>,
    /// Token type (usually "Bearer").
    pub token_type: String,
    /// When the access token expires (if known).
    pub expires_at: Option<DateTime<Utc>>,
    /// Scopes granted by this token.
    pub scopes: Vec<String>,
    /// Additional token metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl TokenPair {
    /// Create a new token pair.
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the refresh token.
    pub fn with_refresh_token(mut self, token: impl Into<String>) -> Self {
        self.refresh_token = Some(token.into());
        self
    }

    /// Set the token type.
    pub fn with_token_type(mut self, token_type: impl Into<String>) -> Self {
        self.token_type = token_type.into();
        self
    }

    /// Set the expiration time.
    pub fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set expiration from duration (seconds from now).
    pub fn with_expires_in(mut self, seconds: i64) -> Self {
        self.expires_at = Some(Utc::now() + Duration::seconds(seconds));
        self
    }

    /// Add scopes.
    pub fn with_scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scopes = scopes.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Utc::now() >= exp)
            .unwrap_or(false)
    }

    /// Check if the token is expiring soon (within threshold).
    pub fn is_expiring_soon(&self, threshold: Duration) -> bool {
        self.expires_at
            .map(|exp| Utc::now() + threshold >= exp)
            .unwrap_or(false)
    }

    /// Check if the token can be refreshed.
    pub fn can_refresh(&self) -> bool {
        self.refresh_token.is_some()
    }

    /// Get time until expiration.
    pub fn time_until_expiry(&self) -> Option<Duration> {
        self.expires_at.map(|exp| exp - Utc::now())
    }

    /// Get the authorization header value.
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }

    /// Check if the token has a specific scope.
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

/// JWT claims that can be extracted from a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (usually user ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    /// Issuer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// Audience.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<StringOrVec>,
    /// Expiration time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    /// Not before time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    /// Issued at time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    /// JWT ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    /// Custom claims.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl JwtClaims {
    /// Get expiration as DateTime.
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.exp.map(|ts| {
            DateTime::from_timestamp(ts, 0)
                .unwrap_or_else(|| Utc::now())
        })
    }

    /// Get issued at as DateTime.
    pub fn issued_at(&self) -> Option<DateTime<Utc>> {
        self.iat.map(|ts| {
            DateTime::from_timestamp(ts, 0)
                .unwrap_or_else(|| Utc::now())
        })
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        self.exp
            .map(|exp| Utc::now().timestamp() >= exp)
            .unwrap_or(false)
    }

    /// Check if the token is not yet valid.
    pub fn is_not_yet_valid(&self) -> bool {
        self.nbf
            .map(|nbf| Utc::now().timestamp() < nbf)
            .unwrap_or(false)
    }

    /// Get a custom claim value.
    pub fn get_claim<T: for<'de> Deserialize<'de>>(&self, name: &str) -> Option<T> {
        self.extra.get(name).and_then(|v| {
            serde_json::from_value(v.clone()).ok()
        })
    }
}

/// Helper type for audience claim which can be string or array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrVec {
    /// Single string value.
    String(String),
    /// Array of strings.
    Vec(Vec<String>),
}

impl StringOrVec {
    /// Check if the audience contains a specific value.
    pub fn contains(&self, value: &str) -> bool {
        match self {
            StringOrVec::String(s) => s == value,
            StringOrVec::Vec(v) => v.iter().any(|s| s == value),
        }
    }

    /// Get as a vector.
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            StringOrVec::String(s) => vec![s.clone()],
            StringOrVec::Vec(v) => v.clone(),
        }
    }
}

/// Decode JWT payload without verification (for extracting claims).
///
/// WARNING: This does NOT verify the signature. Use only for extracting
/// claims that will be validated by the server or when you have already
/// verified the token.
pub fn decode_jwt_payload_unverified(token: &str) -> Option<JwtClaims> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload = parts[1];
    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        payload,
    )
    .ok()?;

    serde_json::from_slice(&decoded).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_pair() {
        let token = TokenPair::new("access123")
            .with_refresh_token("refresh456")
            .with_expires_in(3600)
            .with_scopes(["read", "write"]);

        assert!(!token.is_expired());
        assert!(token.can_refresh());
        assert!(token.has_scope("read"));
        assert!(!token.has_scope("admin"));
        assert_eq!(
            token.authorization_header(),
            "Bearer access123"
        );
    }

    #[test]
    fn test_expired_token() {
        let token = TokenPair::new("expired")
            .with_expires_at(Utc::now() - Duration::hours(1));

        assert!(token.is_expired());
    }

    #[test]
    fn test_jwt_claims() {
        let claims = JwtClaims {
            sub: Some("user123".to_string()),
            iss: Some("https://auth.example.com".to_string()),
            aud: Some(StringOrVec::String("my-app".to_string())),
            exp: Some((Utc::now() + Duration::hours(1)).timestamp()),
            nbf: None,
            iat: Some(Utc::now().timestamp()),
            jti: None,
            extra: std::collections::HashMap::new(),
        };

        assert!(!claims.is_expired());
        assert!(!claims.is_not_yet_valid());
    }
}
