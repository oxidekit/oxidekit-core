//! Error types for the permissions system.

use std::net::IpAddr;
use std::path::PathBuf;
use thiserror::Error;

/// Result type for permission operations.
pub type PermissionResult<T> = Result<T, PermissionError>;

/// Errors that can occur during permission operations.
#[derive(Error, Debug)]
pub enum PermissionError {
    /// Permission denied for a capability
    #[error("Permission denied: capability '{capability}' is not granted")]
    CapabilityDenied {
        /// The denied capability
        capability: String,
    },

    /// Permission not declared in manifest
    #[error("Permission not declared in manifest: {0}")]
    PermissionNotDeclared(String),

    /// Network access denied
    #[error("Network access denied: domain '{domain}' is not in the allowlist")]
    NetworkDomainDenied {
        /// The denied domain
        domain: String,
    },

    /// Network access to private IP denied
    #[error("Network access to private IP range denied: {ip}")]
    PrivateIpDenied {
        /// The denied IP
        ip: IpAddr,
    },

    /// DNS resolution failed
    #[error("DNS resolution failed for domain '{domain}': {reason}")]
    DnsResolutionFailed {
        /// The domain that failed to resolve
        domain: String,
        /// The reason for failure
        reason: String,
    },

    /// Invalid manifest configuration
    #[error("Invalid manifest configuration: {0}")]
    InvalidManifest(String),

    /// Invalid capability format
    #[error("Invalid capability format: {0}")]
    InvalidCapability(String),

    /// Invalid network policy
    #[error("Invalid network policy: {0}")]
    InvalidNetworkPolicy(String),

    /// Attestation verification failed
    #[error("Attestation verification failed: {0}")]
    AttestationFailed(String),

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureInvalid(String),

    /// Hash mismatch during verification
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch {
        /// Expected hash
        expected: String,
        /// Actual hash
        actual: String,
    },

    /// Binary analysis failed
    #[error("Binary analysis failed: {0}")]
    BinaryAnalysisFailed(String),

    /// Verified build check failed
    #[error("Verified build check failed: {reason}")]
    VerifiedBuildFailed {
        /// The reason for failure
        reason: String,
        /// The specific check that failed
        check: String,
    },

    /// Forbidden API detected
    #[error("Forbidden API detected: {api} in {location}")]
    ForbiddenApiDetected {
        /// The forbidden API
        api: String,
        /// Where it was detected
        location: String,
    },

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Runtime enforcement error
    #[error("Runtime enforcement error: {0}")]
    RuntimeEnforcementError(String),

    /// Disclosure consent required
    #[error("User consent required for capability: {0}")]
    ConsentRequired(String),

    /// Marketplace badge verification failed
    #[error("Marketplace badge verification failed: {0}")]
    BadgeVerificationFailed(String),
}

impl From<toml::de::Error> for PermissionError {
    fn from(err: toml::de::Error) -> Self {
        PermissionError::InvalidManifest(err.to_string())
    }
}

impl From<toml::ser::Error> for PermissionError {
    fn from(err: toml::ser::Error) -> Self {
        PermissionError::SerializationError(err.to_string())
    }
}

impl From<serde_json::Error> for PermissionError {
    fn from(err: serde_json::Error) -> Self {
        PermissionError::SerializationError(err.to_string())
    }
}
