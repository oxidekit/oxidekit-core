//! Persistence tier definitions.
//!
//! Persistence tiers define how state is stored, its lifetime, and security requirements.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Persistence tier for state storage.
///
/// Each tier has different characteristics for storage location, lifetime,
/// and security requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PersistenceTier {
    /// In-memory only, lost on application restart.
    ///
    /// Use for:
    /// - UI state (scroll position, focus)
    /// - Animation state
    /// - Temporary calculations
    /// - Cached derived data
    #[default]
    Volatile,

    /// Persisted to local file storage.
    ///
    /// Use for:
    /// - User preferences
    /// - Application settings
    /// - Cached data
    /// - Non-sensitive user data
    Local,

    /// Encrypted local storage with application-level key.
    ///
    /// Use for:
    /// - API tokens
    /// - Session data
    /// - Sensitive settings
    Secure,

    /// Full encryption with user-provided key derivation.
    ///
    /// Use for:
    /// - Wallet seed phrases
    /// - Private keys
    /// - Admin credentials
    /// - Highly sensitive user data
    Encrypted,

    /// Designed for cloud synchronization.
    ///
    /// Use for:
    /// - User profile data
    /// - Cross-device settings
    /// - Bookmark/favorites data
    Syncable,
}

impl PersistenceTier {
    /// Returns true if this tier persists data across restarts.
    pub fn is_persistent(&self) -> bool {
        !matches!(self, Self::Volatile)
    }

    /// Returns true if this tier requires encryption.
    pub fn requires_encryption(&self) -> bool {
        matches!(self, Self::Secure | Self::Encrypted)
    }

    /// Returns true if this tier uses user-provided key derivation.
    pub fn requires_user_key(&self) -> bool {
        matches!(self, Self::Encrypted)
    }

    /// Returns true if this tier supports synchronization.
    pub fn is_syncable(&self) -> bool {
        matches!(self, Self::Syncable)
    }

    /// Get the recommended storage location for this tier.
    pub fn storage_location(&self) -> StorageLocation {
        match self {
            Self::Volatile => StorageLocation::Memory,
            Self::Local => StorageLocation::DataDir,
            Self::Secure => StorageLocation::DataDir,
            Self::Encrypted => StorageLocation::DataDir,
            Self::Syncable => StorageLocation::DataDir,
        }
    }

    /// Get the recommended file extension for this tier.
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Volatile => "json",
            Self::Local => "json",
            Self::Secure => "enc",
            Self::Encrypted => "vault",
            Self::Syncable => "sync.json",
        }
    }

    /// Parse a tier from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "volatile" => Some(Self::Volatile),
            "local" => Some(Self::Local),
            "secure" => Some(Self::Secure),
            "encrypted" => Some(Self::Encrypted),
            "syncable" => Some(Self::Syncable),
            _ => None,
        }
    }
}

impl fmt::Display for PersistenceTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Volatile => write!(f, "volatile"),
            Self::Local => write!(f, "local"),
            Self::Secure => write!(f, "secure"),
            Self::Encrypted => write!(f, "encrypted"),
            Self::Syncable => write!(f, "syncable"),
        }
    }
}

/// Storage location for persisted state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageLocation {
    /// In-memory only.
    Memory,
    /// Application data directory.
    DataDir,
    /// Application cache directory.
    CacheDir,
    /// Custom path.
    Custom,
}

impl StorageLocation {
    /// Get the base path for this storage location.
    pub fn base_path(&self, app_name: &str) -> Option<std::path::PathBuf> {
        match self {
            Self::Memory => None,
            Self::DataDir => directories::ProjectDirs::from("dev", "oxidekit", app_name)
                .map(|dirs| dirs.data_dir().to_path_buf()),
            Self::CacheDir => directories::ProjectDirs::from("dev", "oxidekit", app_name)
                .map(|dirs| dirs.cache_dir().to_path_buf()),
            Self::Custom => None,
        }
    }
}

/// Tier configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    /// The persistence tier.
    pub tier: PersistenceTier,
    /// Optional custom storage path.
    pub custom_path: Option<std::path::PathBuf>,
    /// Sync interval for syncable tier (in seconds).
    pub sync_interval_secs: Option<u64>,
    /// Maximum age for cached data (in seconds).
    pub max_age_secs: Option<u64>,
    /// Whether to compress stored data.
    pub compress: bool,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            tier: PersistenceTier::default(),
            custom_path: None,
            sync_interval_secs: None,
            max_age_secs: None,
            compress: false,
        }
    }
}

impl TierConfig {
    /// Create a new tier configuration.
    pub fn new(tier: PersistenceTier) -> Self {
        Self {
            tier,
            ..Default::default()
        }
    }

    /// Create a volatile tier configuration.
    pub fn volatile() -> Self {
        Self::new(PersistenceTier::Volatile)
    }

    /// Create a local tier configuration.
    pub fn local() -> Self {
        Self::new(PersistenceTier::Local)
    }

    /// Create a secure tier configuration.
    pub fn secure() -> Self {
        Self::new(PersistenceTier::Secure)
    }

    /// Create an encrypted tier configuration.
    pub fn encrypted() -> Self {
        Self::new(PersistenceTier::Encrypted)
    }

    /// Create a syncable tier configuration with sync interval.
    pub fn syncable(sync_interval_secs: u64) -> Self {
        Self {
            tier: PersistenceTier::Syncable,
            sync_interval_secs: Some(sync_interval_secs),
            ..Default::default()
        }
    }

    /// Set a custom storage path.
    pub fn with_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.custom_path = Some(path.into());
        self
    }

    /// Enable compression.
    pub fn with_compression(mut self) -> Self {
        self.compress = true;
        self
    }

    /// Set maximum age for cached data.
    pub fn with_max_age(mut self, secs: u64) -> Self {
        self.max_age_secs = Some(secs);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_properties() {
        assert!(!PersistenceTier::Volatile.is_persistent());
        assert!(PersistenceTier::Local.is_persistent());
        assert!(PersistenceTier::Secure.requires_encryption());
        assert!(PersistenceTier::Encrypted.requires_user_key());
        assert!(PersistenceTier::Syncable.is_syncable());
    }

    #[test]
    fn test_tier_from_str() {
        assert_eq!(PersistenceTier::from_str("local"), Some(PersistenceTier::Local));
        assert_eq!(PersistenceTier::from_str("ENCRYPTED"), Some(PersistenceTier::Encrypted));
        assert_eq!(PersistenceTier::from_str("invalid"), None);
    }

    #[test]
    fn test_tier_display() {
        assert_eq!(format!("{}", PersistenceTier::Local), "local");
        assert_eq!(format!("{}", PersistenceTier::Encrypted), "encrypted");
    }
}
