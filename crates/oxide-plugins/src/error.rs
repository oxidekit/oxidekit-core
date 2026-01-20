//! Error types for the plugin system.

use std::path::PathBuf;
use thiserror::Error;

use crate::namespace::PluginId;

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

/// Errors that can occur during plugin operations.
#[derive(Error, Debug)]
pub enum PluginError {
    /// Project directory not found
    #[error("Project not found at path: {0}")]
    ProjectNotFound(PathBuf),

    /// Plugin not found
    #[error("Plugin not found: {0}")]
    PluginNotFound(PluginId),

    /// Plugin already installed
    #[error("Plugin already installed: {0}")]
    AlreadyInstalled(PluginId),

    /// Plugin not installed
    #[error("Plugin not installed: {0}")]
    NotInstalled(PluginId),

    /// Invalid plugin ID format
    #[error("Invalid plugin ID: {0}")]
    InvalidPluginId(String),

    /// Invalid namespace
    #[error("Invalid namespace: {0}. Must be one of: ui, native, auth, db, data, tool, theme, design, icons, fonts")]
    InvalidNamespace(String),

    /// Invalid manifest
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),

    /// Manifest file not found
    #[error("Manifest file not found: {0}")]
    ManifestNotFound(PathBuf),

    /// Failed to parse manifest
    #[error("Failed to parse manifest: {0}")]
    ManifestParseError(String),

    /// Version constraint not satisfied
    #[error("Version constraint not satisfied: {0}")]
    VersionConstraintError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Capability not allowed
    #[error("Capability not allowed for plugin kind '{kind}': {capability}")]
    CapabilityNotAllowed {
        /// The plugin kind
        kind: String,
        /// The disallowed capability
        capability: String,
    },

    /// Trust level insufficient
    #[error("Trust level insufficient: requires {required:?}, has {actual:?}")]
    InsufficientTrustLevel {
        /// Required trust level
        required: crate::trust::TrustLevel,
        /// Actual trust level
        actual: crate::trust::TrustLevel,
    },

    /// Installation failed
    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    /// Git operation failed
    #[error("Git operation failed: {0}")]
    GitError(String),

    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Hash verification failed
    #[error("Hash verification failed for {path}: expected {expected}, got {actual}")]
    HashMismatch {
        /// Path to the file
        path: PathBuf,
        /// Expected hash
        expected: String,
        /// Actual hash
        actual: String,
    },

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    /// Sandbox error
    #[error("Sandbox error: {0}")]
    SandboxError(String),

    /// WASM compilation error
    #[error("WASM compilation error: {0}")]
    WasmCompilationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Lockfile error
    #[error("Lockfile error: {0}")]
    LockfileError(String),

    /// Dependency resolution error
    #[error("Dependency resolution error: {0}")]
    DependencyResolutionError(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Build script not allowed
    #[error("Build script not allowed for unverified plugins")]
    BuildScriptNotAllowed,

    /// Unsafe dependency detected
    #[error("Unsafe dependency detected: {0}")]
    UnsafeDependency(String),

    /// Plugin kind mismatch
    #[error("Plugin kind mismatch: expected {expected}, got {actual}")]
    KindMismatch {
        /// Expected kind
        expected: String,
        /// Actual kind
        actual: String,
    },
}

impl From<toml::de::Error> for PluginError {
    fn from(err: toml::de::Error) -> Self {
        PluginError::ManifestParseError(err.to_string())
    }
}

impl From<toml::ser::Error> for PluginError {
    fn from(err: toml::ser::Error) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}

impl From<semver::Error> for PluginError {
    fn from(err: semver::Error) -> Self {
        PluginError::VersionConstraintError(err.to_string())
    }
}
