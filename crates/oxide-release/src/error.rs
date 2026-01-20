//! Error types for the release system

use thiserror::Error;

/// Release system errors
#[derive(Error, Debug)]
pub enum ReleaseError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Version parsing error
    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    /// Build error
    #[error("Build failed: {0}")]
    BuildFailed(String),

    /// Signing error
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// Signing identity not found
    #[error("Signing identity not found: {0}")]
    SigningIdentityNotFound(String),

    /// Signing certificate expired
    #[error("Signing certificate expired: {0}")]
    SigningCertificateExpired(String),

    /// Notarization error
    #[error("Notarization failed: {0}")]
    NotarizationFailed(String),

    /// Notarization timeout
    #[error("Notarization timed out after {0} seconds")]
    NotarizationTimeout(u64),

    /// Notarization rejected
    #[error("Notarization rejected by Apple: {0}")]
    NotarizationRejected(String),

    /// Packaging error
    #[error("Packaging failed: {0}")]
    PackagingFailed(String),

    /// Unsupported package format
    #[error("Unsupported package format: {0}")]
    UnsupportedPackageFormat(String),

    /// Changelog generation error
    #[error("Changelog generation failed: {0}")]
    ChangelogFailed(String),

    /// Git error
    #[error("Git error: {0}")]
    Git(String),

    /// GitHub API error
    #[error("GitHub API error: {0}")]
    GitHub(String),

    /// Publishing error
    #[error("Publishing failed: {0}")]
    PublishFailed(String),

    /// Manifest validation error
    #[error("Manifest validation failed: {0}")]
    ManifestValidation(String),

    /// Update metadata error
    #[error("Update metadata error: {0}")]
    UpdateMetadata(String),

    /// File system error
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// Checksum mismatch
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    /// Artifact not found
    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    /// Tool not found
    #[error("Required tool not found: {0}. Install it or add to PATH.")]
    ToolNotFound(String),

    /// Missing environment variable
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Doctor check failed
    #[error("Doctor check failed: {message}")]
    DoctorFailed {
        /// Error message
        message: String,
        /// Recommendations to fix
        recommendations: Vec<String>,
    },

    /// Dry run - operation skipped
    #[error("Operation skipped (dry run): {0}")]
    DryRun(String),

    /// Channel mismatch
    #[error("Channel mismatch: expected {expected}, got {actual}")]
    ChannelMismatch { expected: String, actual: String },

    /// Version already exists
    #[error("Version {0} already exists")]
    VersionExists(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Generic error with context
    #[error("{context}: {source}")]
    WithContext {
        /// Context message
        context: String,
        /// Source error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl ReleaseError {
    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a build error
    pub fn build(msg: impl Into<String>) -> Self {
        Self::BuildFailed(msg.into())
    }

    /// Create a signing error
    pub fn signing(msg: impl Into<String>) -> Self {
        Self::SigningFailed(msg.into())
    }

    /// Create a notarization error
    pub fn notarization(msg: impl Into<String>) -> Self {
        Self::NotarizationFailed(msg.into())
    }

    /// Create a packaging error
    pub fn packaging(msg: impl Into<String>) -> Self {
        Self::PackagingFailed(msg.into())
    }

    /// Create a GitHub error
    pub fn github(msg: impl Into<String>) -> Self {
        Self::GitHub(msg.into())
    }

    /// Create a publish error
    pub fn publish(msg: impl Into<String>) -> Self {
        Self::PublishFailed(msg.into())
    }

    /// Create a doctor failed error with recommendations
    pub fn doctor_failed(message: impl Into<String>, recommendations: Vec<String>) -> Self {
        Self::DoctorFailed {
            message: message.into(),
            recommendations,
        }
    }

    /// Add context to an error
    pub fn with_context<E>(context: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::WithContext {
            context: context.into(),
            source: Box::new(source),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Network(_)
                | Self::NotarizationTimeout(_)
                | Self::DryRun(_)
                | Self::ToolNotFound(_)
        )
    }

    /// Get suggestions for fixing this error
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::SigningIdentityNotFound(id) => vec![
                format!("Run 'security find-identity -v -p codesigning' to list available identities"),
                format!("Ensure your signing certificate for '{}' is installed in Keychain", id),
                String::from("Check that your Apple Developer account is active"),
            ],
            Self::SigningCertificateExpired(_) => vec![
                String::from("Renew your signing certificate in Apple Developer Portal"),
                String::from("Download and install the new certificate"),
                String::from("Run 'oxide doctor' to verify the new certificate"),
            ],
            Self::NotarizationFailed(_) => vec![
                String::from("Ensure your Apple ID credentials are correct"),
                String::from("Check that your app meets Apple's notarization requirements"),
                String::from("Review the notarization log for specific issues"),
            ],
            Self::NotarizationRejected(reason) => vec![
                format!("Review rejection reason: {}", reason),
                String::from("Ensure all binaries are properly signed"),
                String::from("Check for hardened runtime issues"),
            ],
            Self::ToolNotFound(tool) => vec![
                format!("Install '{}' using your package manager", tool),
                format!("macOS: brew install {}", tool),
                String::from("Ensure the tool is in your PATH"),
            ],
            Self::MissingEnvVar(var) => vec![
                format!("Set the {} environment variable", var),
                format!("Example: export {}=<value>", var),
                String::from("Consider using a .env file for local development"),
            ],
            Self::GitHub(msg) if msg.contains("401") => vec![
                String::from("Check that GITHUB_TOKEN is set correctly"),
                String::from("Ensure the token has 'repo' scope"),
                String::from("Regenerate the token if it has expired"),
            ],
            Self::DoctorFailed { recommendations, .. } => recommendations.clone(),
            _ => vec![],
        }
    }
}

impl From<git2::Error> for ReleaseError {
    fn from(err: git2::Error) -> Self {
        Self::Git(err.message().to_string())
    }
}

impl From<toml::de::Error> for ReleaseError {
    fn from(err: toml::de::Error) -> Self {
        Self::Config(format!("TOML parse error: {}", err))
    }
}

impl From<serde_json::Error> for ReleaseError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<semver::Error> for ReleaseError {
    fn from(err: semver::Error) -> Self {
        Self::InvalidVersion(err.to_string())
    }
}

/// Result type for release operations
pub type ReleaseResult<T> = Result<T, ReleaseError>;
