//! Error types for the OxideKit crypto platform.
//!
//! Provides a comprehensive error taxonomy for all crypto operations,
//! with support for error redaction to prevent leaking sensitive information.

use std::fmt;
use thiserror::Error;

/// Result type for crypto operations.
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Comprehensive error types for crypto operations.
///
/// This error taxonomy covers all possible failure modes in crypto operations
/// while being careful not to leak sensitive information in error messages.
#[derive(Error, Debug)]
pub enum CryptoError {
    // ========================================================================
    // Encoding Errors
    // ========================================================================

    /// Invalid hex string encoding
    #[error("invalid hex encoding: {0}")]
    InvalidHex(String),

    /// Invalid base58 encoding
    #[error("invalid base58 encoding: {0}")]
    InvalidBase58(String),

    /// Invalid bech32 encoding
    #[error("invalid bech32 encoding: {0}")]
    InvalidBech32(String),

    /// Invalid data length for operation
    #[error("invalid data length: expected {expected}, got {actual}")]
    InvalidLength {
        /// Expected length
        expected: usize,
        /// Actual length
        actual: usize,
    },

    // ========================================================================
    // Key Management Errors
    // ========================================================================

    /// Invalid mnemonic phrase
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    /// Invalid derivation path
    #[error("invalid derivation path: {0}")]
    InvalidDerivationPath(String),

    /// Key not found in keystore
    #[error("key not found: {key_id}")]
    KeyNotFound {
        /// The key identifier (redacted)
        key_id: String,
    },

    /// Key is locked and requires unlock
    #[error("key is locked: unlock required")]
    KeyLocked,

    /// Key generation failed
    #[error("key generation failed")]
    KeyGenerationFailed,

    /// Keychain access denied
    #[error("keychain access denied")]
    KeychainAccessDenied,

    /// Keychain operation failed
    #[error("keychain operation failed: {0}")]
    KeychainError(String),

    // ========================================================================
    // Signing Errors
    // ========================================================================

    /// Signing operation failed
    #[error("signing failed")]
    SigningFailed,

    /// User cancelled signing
    #[error("signing cancelled by user")]
    SigningCancelled,

    /// Signing requires confirmation
    #[error("signing requires user confirmation")]
    SigningRequiresConfirmation,

    /// Invalid signature format
    #[error("invalid signature format")]
    InvalidSignature,

    // ========================================================================
    // Transaction Errors
    // ========================================================================

    /// Invalid transaction format
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Transaction simulation failed
    #[error("transaction simulation failed: {0}")]
    SimulationFailed(String),

    /// Insufficient funds for transaction
    #[error("insufficient funds")]
    InsufficientFunds,

    /// Gas estimation failed
    #[error("gas estimation failed: {0}")]
    GasEstimationFailed(String),

    /// Nonce error
    #[error("nonce error: {0}")]
    NonceError(String),

    // ========================================================================
    // Network/RPC Errors
    // ========================================================================

    /// RPC connection failed
    #[error("RPC connection failed: {endpoint}")]
    RpcConnectionFailed {
        /// The endpoint that failed (may be redacted)
        endpoint: String,
    },

    /// RPC request failed
    #[error("RPC request failed: {method}")]
    RpcRequestFailed {
        /// The RPC method that failed
        method: String,
    },

    /// Rate limit exceeded
    #[error("rate limit exceeded")]
    RateLimitExceeded,

    /// All providers failed
    #[error("all providers failed after {attempts} attempts")]
    AllProvidersFailed {
        /// Number of attempts made
        attempts: usize,
    },

    /// Network not allowed by policy
    #[error("network not allowed: {domain}")]
    NetworkNotAllowed {
        /// The blocked domain
        domain: String,
    },

    /// TLS required but not available
    #[error("TLS required for endpoint")]
    TlsRequired,

    // ========================================================================
    // Policy Errors
    // ========================================================================

    /// Policy violation
    #[error("policy violation: {rule}")]
    PolicyViolation {
        /// The rule that was violated
        rule: String,
    },

    /// Attestation verification failed
    #[error("attestation verification failed")]
    AttestationFailed,

    /// Required capability not available
    #[error("capability not available: {capability}")]
    CapabilityNotAvailable {
        /// The missing capability
        capability: String,
    },

    // ========================================================================
    // Chain-Specific Errors
    // ========================================================================

    /// Invalid chain ID
    #[error("invalid chain ID: {0}")]
    InvalidChainId(u64),

    /// Chain not supported
    #[error("chain not supported: {0}")]
    ChainNotSupported(String),

    /// Invalid address format
    #[error("invalid address format: {0}")]
    InvalidAddress(String),

    /// UTXO selection failed
    #[error("UTXO selection failed: {0}")]
    UtxoSelectionFailed(String),

    /// PSBT error
    #[error("PSBT error: {0}")]
    PsbtError(String),

    // ========================================================================
    // Node Operations Errors
    // ========================================================================

    /// Node binary not found
    #[error("node binary not found: {binary}")]
    NodeBinaryNotFound {
        /// The binary name
        binary: String,
    },

    /// Node health check failed
    #[error("node health check failed")]
    NodeHealthCheckFailed,

    /// Docker operation failed
    #[error("Docker operation failed: {0}")]
    DockerError(String),

    // ========================================================================
    // General Errors
    // ========================================================================

    /// Serialization error
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("deserialization error: {0}")]
    DeserializationError(String),

    /// Configuration error
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// Internal error (should not happen)
    #[error("internal error: {0}")]
    Internal(String),

    /// Feature not available
    #[error("feature not available: {0}")]
    FeatureNotAvailable(String),
}

impl CryptoError {
    /// Create a redacted version of this error suitable for logging.
    ///
    /// This removes any potentially sensitive information from the error
    /// message while preserving the error category.
    pub fn redacted(&self) -> RedactedError {
        RedactedError {
            category: self.category(),
            message: self.redacted_message(),
        }
    }

    /// Get the error category for classification.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::InvalidHex(_)
            | Self::InvalidBase58(_)
            | Self::InvalidBech32(_)
            | Self::InvalidLength { .. } => ErrorCategory::Encoding,

            Self::InvalidMnemonic(_)
            | Self::InvalidDerivationPath(_)
            | Self::KeyNotFound { .. }
            | Self::KeyLocked
            | Self::KeyGenerationFailed
            | Self::KeychainAccessDenied
            | Self::KeychainError(_) => ErrorCategory::KeyManagement,

            Self::SigningFailed
            | Self::SigningCancelled
            | Self::SigningRequiresConfirmation
            | Self::InvalidSignature => ErrorCategory::Signing,

            Self::InvalidTransaction(_)
            | Self::SimulationFailed(_)
            | Self::InsufficientFunds
            | Self::GasEstimationFailed(_)
            | Self::NonceError(_) => ErrorCategory::Transaction,

            Self::RpcConnectionFailed { .. }
            | Self::RpcRequestFailed { .. }
            | Self::RateLimitExceeded
            | Self::AllProvidersFailed { .. }
            | Self::NetworkNotAllowed { .. }
            | Self::TlsRequired => ErrorCategory::Network,

            Self::PolicyViolation { .. }
            | Self::AttestationFailed
            | Self::CapabilityNotAvailable { .. } => ErrorCategory::Policy,

            Self::InvalidChainId(_)
            | Self::ChainNotSupported(_)
            | Self::InvalidAddress(_)
            | Self::UtxoSelectionFailed(_)
            | Self::PsbtError(_) => ErrorCategory::Chain,

            Self::NodeBinaryNotFound { .. }
            | Self::NodeHealthCheckFailed
            | Self::DockerError(_) => ErrorCategory::NodeOps,

            Self::SerializationError(_)
            | Self::DeserializationError(_)
            | Self::ConfigError(_)
            | Self::Internal(_)
            | Self::FeatureNotAvailable(_) => ErrorCategory::General,
        }
    }

    /// Get a redacted error message.
    fn redacted_message(&self) -> String {
        match self {
            // Preserve non-sensitive information
            Self::InvalidLength { expected, actual } => {
                format!("invalid length: expected {expected}, got {actual}")
            }
            Self::KeyLocked => "key is locked".to_string(),
            Self::SigningCancelled => "signing cancelled".to_string(),
            Self::SigningRequiresConfirmation => "confirmation required".to_string(),
            Self::InsufficientFunds => "insufficient funds".to_string(),
            Self::RateLimitExceeded => "rate limited".to_string(),
            Self::AllProvidersFailed { attempts } => {
                format!("providers failed after {attempts} attempts")
            }
            Self::TlsRequired => "TLS required".to_string(),
            Self::AttestationFailed => "attestation failed".to_string(),
            Self::NodeHealthCheckFailed => "health check failed".to_string(),

            // Redact potentially sensitive information
            Self::InvalidMnemonic(_) => "invalid mnemonic".to_string(),
            Self::InvalidDerivationPath(_) => "invalid derivation path".to_string(),
            Self::KeyNotFound { .. } => "key not found".to_string(),
            Self::RpcConnectionFailed { .. } => "connection failed".to_string(),
            Self::NetworkNotAllowed { .. } => "network not allowed".to_string(),

            // Generic redaction for other errors
            _ => self.category().to_string(),
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RpcConnectionFailed { .. }
                | Self::RpcRequestFailed { .. }
                | Self::RateLimitExceeded
                | Self::NodeHealthCheckFailed
        )
    }

    /// Check if this error requires user action.
    pub fn requires_user_action(&self) -> bool {
        matches!(
            self,
            Self::KeyLocked
                | Self::SigningRequiresConfirmation
                | Self::SigningCancelled
                | Self::KeychainAccessDenied
                | Self::InsufficientFunds
        )
    }
}

/// Error category for classification and handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Encoding/decoding errors
    Encoding,
    /// Key management errors
    KeyManagement,
    /// Signing errors
    Signing,
    /// Transaction errors
    Transaction,
    /// Network/RPC errors
    Network,
    /// Policy errors
    Policy,
    /// Chain-specific errors
    Chain,
    /// Node operations errors
    NodeOps,
    /// General errors
    General,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encoding => write!(f, "encoding error"),
            Self::KeyManagement => write!(f, "key management error"),
            Self::Signing => write!(f, "signing error"),
            Self::Transaction => write!(f, "transaction error"),
            Self::Network => write!(f, "network error"),
            Self::Policy => write!(f, "policy error"),
            Self::Chain => write!(f, "chain error"),
            Self::NodeOps => write!(f, "node operations error"),
            Self::General => write!(f, "error"),
        }
    }
}

/// A redacted error suitable for logging.
///
/// This struct contains only non-sensitive information about an error,
/// making it safe to include in logs or diagnostics.
#[derive(Debug, Clone)]
pub struct RedactedError {
    /// The error category
    pub category: ErrorCategory,
    /// A redacted error message
    pub message: String,
}

impl fmt::Display for RedactedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.category, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let encoding_err = CryptoError::InvalidHex("bad".to_string());
        assert_eq!(encoding_err.category(), ErrorCategory::Encoding);

        let key_err = CryptoError::KeyLocked;
        assert_eq!(key_err.category(), ErrorCategory::KeyManagement);

        let network_err = CryptoError::RateLimitExceeded;
        assert_eq!(network_err.category(), ErrorCategory::Network);
    }

    #[test]
    fn test_error_redaction() {
        let err = CryptoError::InvalidMnemonic("secret phrase here".to_string());
        let redacted = err.redacted();

        assert!(!redacted.message.contains("secret"));
        assert_eq!(redacted.message, "invalid mnemonic");
    }

    #[test]
    fn test_retryable_errors() {
        assert!(CryptoError::RateLimitExceeded.is_retryable());
        assert!(CryptoError::RpcConnectionFailed {
            endpoint: "test".to_string()
        }.is_retryable());

        assert!(!CryptoError::KeyLocked.is_retryable());
        assert!(!CryptoError::InvalidSignature.is_retryable());
    }

    #[test]
    fn test_user_action_errors() {
        assert!(CryptoError::KeyLocked.requires_user_action());
        assert!(CryptoError::SigningRequiresConfirmation.requires_user_action());
        assert!(CryptoError::InsufficientFunds.requires_user_action());

        assert!(!CryptoError::RateLimitExceeded.requires_user_action());
    }
}
