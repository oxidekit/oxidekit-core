//! Error types for the portability system.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for portability operations.
pub type PortabilityResult<T> = Result<T, PortabilityError>;

/// Errors that can occur during portability checking.
#[derive(Debug, Error)]
pub enum PortabilityError {
    /// Target not supported for the requested operation.
    #[error("Target '{0}' is not supported")]
    UnsupportedTarget(String),

    /// API is not portable to the requested target.
    #[error("API '{api}' is not portable to target '{target}': {reason}")]
    NotPortable {
        /// The API that is not portable
        api: String,
        /// The target that was requested
        target: String,
        /// Reason why it's not portable
        reason: String,
    },

    /// Invalid portability manifest.
    #[error("Invalid portability manifest: {0}")]
    InvalidManifest(String),

    /// Manifest file not found.
    #[error("Manifest not found at: {0}")]
    ManifestNotFound(PathBuf),

    /// IO error during portability checking.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Plugin has portability issues.
    #[error("Plugin '{plugin}' has {count} portability issues")]
    PluginPortabilityIssues {
        /// The plugin with issues
        plugin: String,
        /// Number of issues found
        count: usize,
    },

    /// Invalid target triple.
    #[error("Invalid target triple: {0}")]
    InvalidTargetTriple(String),

    /// Compile-time check failed.
    #[error("Compile-time portability check failed: {0}")]
    CompileCheckFailed(String),

    /// Feature not available on current target.
    #[error("Feature '{feature}' is not available on target '{target}'")]
    FeatureNotAvailable {
        /// The feature that is not available
        feature: String,
        /// The current target
        target: String,
    },

    /// Dependency has portability issues.
    #[error("Dependency '{dependency}' is not portable: {reason}")]
    DependencyNotPortable {
        /// The dependency with issues
        dependency: String,
        /// Reason why it's not portable
        reason: String,
    },
}

impl From<serde_json::Error> for PortabilityError {
    fn from(err: serde_json::Error) -> Self {
        PortabilityError::Serialization(err.to_string())
    }
}

impl From<toml::de::Error> for PortabilityError {
    fn from(err: toml::de::Error) -> Self {
        PortabilityError::Serialization(err.to_string())
    }
}

impl From<toml::ser::Error> for PortabilityError {
    fn from(err: toml::ser::Error) -> Self {
        PortabilityError::Serialization(err.to_string())
    }
}
