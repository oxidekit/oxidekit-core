//! Error types for the state management system.

use thiserror::Error;

/// Result type alias using StateError.
pub type StateResult<T> = Result<T, StateError>;

/// Errors that can occur during state operations.
#[derive(Debug, Error)]
pub enum StateError {
    /// State serialization failed.
    #[error("Failed to serialize state: {0}")]
    Serialization(#[from] serde_json::Error),

    /// State file I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// State not found in store.
    #[error("State not found: {id}")]
    NotFound {
        /// The ID of the missing state.
        id: String,
    },

    /// State version mismatch requiring migration.
    #[error("State version mismatch: found {found}, expected {expected}")]
    VersionMismatch {
        /// The version found in storage.
        found: u32,
        /// The expected version.
        expected: u32,
    },

    /// Migration failed.
    #[error("Migration failed from version {from} to {to}: {reason}")]
    MigrationFailed {
        /// Source version.
        from: u32,
        /// Target version.
        to: u32,
        /// Failure reason.
        reason: String,
    },

    /// No migration path exists.
    #[error("No migration path from version {from} to {to}")]
    NoMigrationPath {
        /// Source version.
        from: u32,
        /// Target version.
        to: u32,
    },

    /// Encryption error.
    #[cfg(feature = "encryption")]
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Decryption error.
    #[cfg(feature = "encryption")]
    #[error("Decryption error: {0}")]
    Decryption(String),

    /// Invalid encryption key.
    #[cfg(feature = "encryption")]
    #[error("Invalid encryption key")]
    InvalidKey,

    /// SQLite database error.
    #[cfg(feature = "sqlite")]
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Snapshot creation failed.
    #[error("Snapshot creation failed: {0}")]
    SnapshotFailed(String),

    /// Snapshot restoration failed.
    #[error("Snapshot restoration failed: {0}")]
    RestoreFailed(String),

    /// Invalid state type.
    #[error("Invalid state type: expected {expected}, got {got}")]
    TypeMismatch {
        /// Expected type name.
        expected: String,
        /// Actual type name.
        got: String,
    },

    /// State is locked by another operation.
    #[error("State is locked: {0}")]
    Locked(String),

    /// Synchronization error.
    #[error("Sync error: {0}")]
    SyncError(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Generic internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl StateError {
    /// Create a new NotFound error.
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound { id: id.into() }
    }

    /// Create a version mismatch error.
    pub fn version_mismatch(found: u32, expected: u32) -> Self {
        Self::VersionMismatch { found, expected }
    }

    /// Create a migration failed error.
    pub fn migration_failed(from: u32, to: u32, reason: impl Into<String>) -> Self {
        Self::MigrationFailed {
            from,
            to,
            reason: reason.into(),
        }
    }

    /// Create a type mismatch error.
    pub fn type_mismatch(expected: impl Into<String>, got: impl Into<String>) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            got: got.into(),
        }
    }
}
