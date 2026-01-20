//! Plugin lockfile for reproducible installations.
//!
//! The `extensions.lock` file records the exact versions and sources of
//! all installed plugins, ensuring reproducible builds.

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::error::{PluginError, PluginResult};
use crate::installation::InstallSource;
use crate::manifest::PluginManifest;
use crate::namespace::PluginId;

/// The lockfile for tracking installed plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Lockfile format version.
    pub version: u32,
    /// When the lockfile was last updated.
    pub updated_at: DateTime<Utc>,
    /// Installed plugins.
    #[serde(default)]
    pub plugins: HashMap<String, LockEntry>,
}

impl Lockfile {
    /// Current lockfile format version.
    pub const VERSION: u32 = 1;

    /// Create a new empty lockfile.
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            updated_at: Utc::now(),
            plugins: HashMap::new(),
        }
    }

    /// Load a lockfile from disk.
    pub fn load<P: AsRef<Path>>(path: P) -> PluginResult<Self> {
        let content = fs::read_to_string(path)?;
        let lockfile: Self = toml::from_str(&content)?;

        // Check version compatibility
        if lockfile.version > Self::VERSION {
            return Err(PluginError::LockfileError(format!(
                "Lockfile version {} is newer than supported version {}",
                lockfile.version,
                Self::VERSION
            )));
        }

        Ok(lockfile)
    }

    /// Save the lockfile to disk.
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> PluginResult<()> {
        self.updated_at = Utc::now();
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Add an entry to the lockfile.
    pub fn add_entry(&mut self, entry: LockEntry) {
        self.plugins.insert(entry.id.clone(), entry);
    }

    /// Remove an entry from the lockfile.
    pub fn remove_entry(&mut self, plugin_id: &PluginId) {
        self.plugins.remove(plugin_id.full_name());
    }

    /// Get an entry by plugin ID.
    pub fn get_entry(&self, plugin_id: &str) -> Option<&LockEntry> {
        self.plugins.get(plugin_id)
    }

    /// Check if a plugin is in the lockfile.
    pub fn contains(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }

    /// Get all entries.
    pub fn entries(&self) -> impl Iterator<Item = &LockEntry> {
        self.plugins.values()
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if the lockfile is empty.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Verify all entries have valid hashes.
    pub fn verify_integrity(&self) -> Vec<&LockEntry> {
        self.plugins
            .values()
            .filter(|e| e.hash.is_none())
            .collect()
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

/// A lockfile entry for a single plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    /// Plugin ID.
    pub id: String,
    /// Resolved version.
    pub version: String,
    /// Installation source type.
    pub source: LockSource,
    /// Hash of the installed plugin.
    pub hash: Option<String>,
    /// When this entry was created/updated.
    pub locked_at: DateTime<Utc>,
    /// Dependencies of this plugin (for the full dependency closure).
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// The source recorded in the lockfile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LockSource {
    /// From the registry.
    Registry {
        /// Resolved version.
        version: String,
    },
    /// From a Git repository.
    Git {
        /// Repository URL.
        url: String,
        /// Resolved commit SHA.
        commit: String,
        /// Original ref (for reference).
        #[serde(rename = "ref")]
        git_ref: String,
    },
    /// From a local path.
    Path {
        /// Path to the plugin.
        path: String,
    },
}

impl LockEntry {
    /// Create a new lock entry from an installation.
    pub fn from_install(
        plugin_id: &PluginId,
        source: &InstallSource,
        manifest: &PluginManifest,
    ) -> PluginResult<Self> {
        let lock_source = match source {
            InstallSource::Registry { version } => LockSource::Registry {
                version: version.clone().unwrap_or_else(|| manifest.plugin.version.to_string()),
            },
            InstallSource::Git { url, git_ref } => LockSource::Git {
                url: url.clone(),
                commit: git_ref.clone(), // In real impl, this would be the resolved commit
                git_ref: git_ref.clone(),
            },
            InstallSource::Path { path } => LockSource::Path {
                path: path.to_string_lossy().to_string(),
            },
        };

        Ok(Self {
            id: plugin_id.full_name().to_string(),
            version: manifest.plugin.version.to_string(),
            source: lock_source,
            hash: None, // Hash should be calculated after installation
            locked_at: Utc::now(),
            dependencies: manifest.dependencies.plugins
                .iter()
                .map(|d| d.id.clone())
                .collect(),
        })
    }

    /// Set the hash for this entry.
    pub fn with_hash(mut self, hash: String) -> Self {
        self.hash = Some(hash);
        self
    }

    /// Check if this entry is from a local path.
    pub fn is_local(&self) -> bool {
        matches!(self.source, LockSource::Path { .. })
    }

    /// Check if this entry is from Git.
    pub fn is_git(&self) -> bool {
        matches!(self.source, LockSource::Git { .. })
    }

    /// Check if this entry is from the registry.
    pub fn is_registry(&self) -> bool {
        matches!(self.source, LockSource::Registry { .. })
    }

    /// Get the source URL or path.
    pub fn source_location(&self) -> String {
        match &self.source {
            LockSource::Registry { version } => format!("registry@{}", version),
            LockSource::Git { url, commit, .. } => format!("{}@{}", url, &commit[..8.min(commit.len())]),
            LockSource::Path { path } => format!("path:{}", path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_lockfile_new() {
        let lockfile = Lockfile::new();
        assert_eq!(lockfile.version, Lockfile::VERSION);
        assert!(lockfile.plugins.is_empty());
    }

    #[test]
    fn test_lockfile_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("extensions.lock");

        let mut lockfile = Lockfile::new();
        lockfile.add_entry(LockEntry {
            id: "ui.tables".to_string(),
            version: "1.0.0".to_string(),
            source: LockSource::Registry {
                version: "1.0.0".to_string(),
            },
            hash: Some("abc123".to_string()),
            locked_at: Utc::now(),
            dependencies: vec![],
        });

        lockfile.save(&path).unwrap();

        let loaded = Lockfile::load(&path).unwrap();
        assert_eq!(loaded.plugins.len(), 1);
        assert!(loaded.contains("ui.tables"));
    }

    #[test]
    fn test_lock_entry_source_location() {
        let registry = LockEntry {
            id: "test".to_string(),
            version: "1.0.0".to_string(),
            source: LockSource::Registry {
                version: "1.0.0".to_string(),
            },
            hash: None,
            locked_at: Utc::now(),
            dependencies: vec![],
        };
        assert_eq!(registry.source_location(), "registry@1.0.0");

        let git = LockEntry {
            id: "test".to_string(),
            version: "1.0.0".to_string(),
            source: LockSource::Git {
                url: "github.com/test/plugin".to_string(),
                commit: "abc123def456".to_string(),
                git_ref: "v1.0.0".to_string(),
            },
            hash: None,
            locked_at: Utc::now(),
            dependencies: vec![],
        };
        assert_eq!(git.source_location(), "github.com/test/plugin@abc123de");
    }
}
