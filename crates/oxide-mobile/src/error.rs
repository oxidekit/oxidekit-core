//! Mobile-specific error types.
//!
//! Provides error handling for mobile platform operations including build errors,
//! signing errors, and platform-specific failures.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for mobile operations.
pub type MobileResult<T> = Result<T, MobileError>;

/// Errors that can occur in mobile platform operations.
#[derive(Debug, Error)]
pub enum MobileError {
    /// Build configuration error.
    #[error("Build configuration error: {0}")]
    ConfigError(String),

    /// Target platform not supported.
    #[error("Target platform not supported: {0}")]
    UnsupportedPlatform(String),

    /// Build tool not found.
    #[error("Build tool not found: {tool} (hint: {hint})")]
    ToolNotFound {
        /// Name of the missing tool.
        tool: String,
        /// Installation hint.
        hint: String,
    },

    /// Build process failed.
    #[error("Build failed: {message}")]
    BuildFailed {
        /// Error message.
        message: String,
        /// Exit code if available.
        exit_code: Option<i32>,
    },

    /// Code signing error.
    #[error("Code signing error: {0}")]
    SigningError(String),

    /// Certificate not found.
    #[error("Certificate not found: {0}")]
    CertificateNotFound(String),

    /// Provisioning profile error.
    #[error("Provisioning profile error: {0}")]
    ProvisioningError(String),

    /// SDK not found.
    #[error("SDK not found for platform {platform}: {sdk}")]
    SdkNotFound {
        /// Target platform.
        platform: String,
        /// SDK identifier.
        sdk: String,
    },

    /// File system error.
    #[error("File system error at {path}: {message}")]
    FileSystemError {
        /// Path where error occurred.
        path: PathBuf,
        /// Error message.
        message: String,
    },

    /// Device connection error.
    #[error("Device connection error: {0}")]
    DeviceConnectionError(String),

    /// Gesture recognition error.
    #[error("Gesture recognition error: {0}")]
    GestureError(String),

    /// IME (Input Method Editor) error.
    #[error("IME error: {0}")]
    ImeError(String),

    /// Safe area calculation error.
    #[error("Safe area calculation error: {0}")]
    SafeAreaError(String),

    /// Generic IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl MobileError {
    /// Create a build failed error with an exit code.
    pub fn build_failed_with_code(message: impl Into<String>, exit_code: i32) -> Self {
        Self::BuildFailed {
            message: message.into(),
            exit_code: Some(exit_code),
        }
    }

    /// Create a build failed error without an exit code.
    pub fn build_failed(message: impl Into<String>) -> Self {
        Self::BuildFailed {
            message: message.into(),
            exit_code: None,
        }
    }

    /// Create a file system error.
    pub fn fs_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::FileSystemError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a tool not found error with installation hint.
    pub fn tool_not_found(tool: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::ToolNotFound {
            tool: tool.into(),
            hint: hint.into(),
        }
    }

    /// Create an SDK not found error.
    pub fn sdk_not_found(platform: impl Into<String>, sdk: impl Into<String>) -> Self {
        Self::SdkNotFound {
            platform: platform.into(),
            sdk: sdk.into(),
        }
    }

    /// Returns true if this is a recoverable error.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            MobileError::DeviceConnectionError(_) | MobileError::ImeError(_)
        )
    }

    /// Returns true if this is a configuration error.
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            MobileError::ConfigError(_)
                | MobileError::UnsupportedPlatform(_)
                | MobileError::SdkNotFound { .. }
        )
    }

    /// Returns true if this is a signing-related error.
    pub fn is_signing_error(&self) -> bool {
        matches!(
            self,
            MobileError::SigningError(_)
                | MobileError::CertificateNotFound(_)
                | MobileError::ProvisioningError(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = MobileError::ConfigError("invalid target".into());
        assert!(err.to_string().contains("invalid target"));
    }

    #[test]
    fn test_build_failed_with_code() {
        let err = MobileError::build_failed_with_code("compilation failed", 1);
        match err {
            MobileError::BuildFailed { message, exit_code } => {
                assert_eq!(message, "compilation failed");
                assert_eq!(exit_code, Some(1));
            }
            _ => panic!("Expected BuildFailed variant"),
        }
    }

    #[test]
    fn test_tool_not_found() {
        let err = MobileError::tool_not_found("xcodebuild", "Install Xcode from the App Store");
        match err {
            MobileError::ToolNotFound { tool, hint } => {
                assert_eq!(tool, "xcodebuild");
                assert!(hint.contains("Xcode"));
            }
            _ => panic!("Expected ToolNotFound variant"),
        }
    }

    #[test]
    fn test_is_recoverable() {
        assert!(MobileError::DeviceConnectionError("timeout".into()).is_recoverable());
        assert!(!MobileError::ConfigError("bad config".into()).is_recoverable());
    }

    #[test]
    fn test_is_config_error() {
        assert!(MobileError::ConfigError("bad config".into()).is_config_error());
        assert!(MobileError::UnsupportedPlatform("windows".into()).is_config_error());
        assert!(!MobileError::SigningError("bad cert".into()).is_config_error());
    }

    #[test]
    fn test_is_signing_error() {
        assert!(MobileError::SigningError("invalid signature".into()).is_signing_error());
        assert!(MobileError::CertificateNotFound("dev cert".into()).is_signing_error());
        assert!(!MobileError::BuildFailed {
            message: "failed".into(),
            exit_code: None
        }
        .is_signing_error());
    }
}
