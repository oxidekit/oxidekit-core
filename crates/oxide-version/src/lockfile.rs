//! Lockfile version tracking
//!
//! Tracks exact versions of installed components for reproducible builds.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::semver::Version;
use crate::compatibility::ComponentType;
use crate::error::VersionError;

/// Lockfile version for schema compatibility
pub const LOCKFILE_VERSION: &str = "1";

/// The lockfile format (oxide.lock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Lockfile schema version
    pub version: String,
    /// OxideKit core version
    pub oxidekit: LockfileEntry,
    /// Locked plugins
    #[serde(default)]
    pub plugins: HashMap<String, LockfileEntry>,
    /// Locked themes
    #[serde(default)]
    pub themes: HashMap<String, LockfileEntry>,
    /// Locked starters
    #[serde(default)]
    pub starters: HashMap<String, LockfileEntry>,
    /// Metadata
    #[serde(default)]
    pub metadata: LockfileMetadata,
}

impl Lockfile {
    /// Create a new lockfile with core version
    pub fn new(core_version: Version) -> Self {
        Self {
            version: LOCKFILE_VERSION.to_string(),
            oxidekit: LockfileEntry {
                version: core_version,
                source: None,
                checksum: None,
                dependencies: Vec::new(),
            },
            plugins: HashMap::new(),
            themes: HashMap::new(),
            starters: HashMap::new(),
            metadata: LockfileMetadata::default(),
        }
    }

    /// Add a plugin
    pub fn add_plugin(&mut self, name: impl Into<String>, entry: LockfileEntry) {
        self.plugins.insert(name.into(), entry);
        self.metadata.update();
    }

    /// Add a theme
    pub fn add_theme(&mut self, name: impl Into<String>, entry: LockfileEntry) {
        self.themes.insert(name.into(), entry);
        self.metadata.update();
    }

    /// Add a starter
    pub fn add_starter(&mut self, name: impl Into<String>, entry: LockfileEntry) {
        self.starters.insert(name.into(), entry);
        self.metadata.update();
    }

    /// Get a component by type and name
    pub fn get(&self, component_type: ComponentType, name: &str) -> Option<&LockfileEntry> {
        match component_type {
            ComponentType::Core => Some(&self.oxidekit),
            ComponentType::Plugin => self.plugins.get(name),
            ComponentType::Theme => self.themes.get(name),
            ComponentType::Starter => self.starters.get(name),
        }
    }

    /// Remove a component
    pub fn remove(&mut self, component_type: ComponentType, name: &str) -> Option<LockfileEntry> {
        let result = match component_type {
            ComponentType::Core => None, // Can't remove core
            ComponentType::Plugin => self.plugins.remove(name),
            ComponentType::Theme => self.themes.remove(name),
            ComponentType::Starter => self.starters.remove(name),
        };

        if result.is_some() {
            self.metadata.update();
        }

        result
    }

    /// Get all entries as a flat list
    pub fn all_entries(&self) -> Vec<(ComponentType, &str, &LockfileEntry)> {
        let mut entries = Vec::new();

        entries.push((ComponentType::Core, "oxidekit", &self.oxidekit));

        for (name, entry) in &self.plugins {
            entries.push((ComponentType::Plugin, name, entry));
        }

        for (name, entry) in &self.themes {
            entries.push((ComponentType::Theme, name, entry));
        }

        for (name, entry) in &self.starters {
            entries.push((ComponentType::Starter, name, entry));
        }

        entries
    }

    /// Check if the lockfile has any components (besides core)
    pub fn has_components(&self) -> bool {
        !self.plugins.is_empty() || !self.themes.is_empty() || !self.starters.is_empty()
    }

    /// Get total component count (including core)
    pub fn component_count(&self) -> usize {
        1 + self.plugins.len() + self.themes.len() + self.starters.len()
    }

    /// Parse from TOML string
    pub fn from_toml(content: &str) -> Result<Self, VersionError> {
        toml::from_str(content)
            .map_err(|e| VersionError::Lockfile(format!("Failed to parse lockfile: {}", e)))
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String, VersionError> {
        toml::to_string_pretty(self)
            .map_err(|e| VersionError::Lockfile(format!("Failed to serialize lockfile: {}", e)))
    }

    /// Load from file
    pub fn load(path: &Path) -> Result<Self, VersionError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| VersionError::Io(e))?;
        Self::from_toml(&content)
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<(), VersionError> {
        let content = self.to_toml()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Compare two lockfiles and find differences
    pub fn diff(&self, other: &Lockfile) -> LockfileDiff {
        let mut diff = LockfileDiff::new();

        // Compare core
        if self.oxidekit.version != other.oxidekit.version {
            diff.changed.push(LockfileChange {
                component_type: ComponentType::Core,
                name: "oxidekit".to_string(),
                from_version: Some(self.oxidekit.version.clone()),
                to_version: Some(other.oxidekit.version.clone()),
            });
        }

        // Compare plugins
        diff.compare_maps(ComponentType::Plugin, &self.plugins, &other.plugins);
        diff.compare_maps(ComponentType::Theme, &self.themes, &other.themes);
        diff.compare_maps(ComponentType::Starter, &self.starters, &other.starters);

        diff
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new(Version::new(0, 1, 0))
    }
}

/// A locked entry for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileEntry {
    /// Exact version
    pub version: Version,
    /// Source (registry, git, path)
    #[serde(default)]
    pub source: Option<String>,
    /// Checksum for verification
    #[serde(default)]
    pub checksum: Option<String>,
    /// Resolved dependencies
    #[serde(default)]
    pub dependencies: Vec<LockedDependency>,
}

impl LockfileEntry {
    /// Create a new entry with just a version
    pub fn new(version: Version) -> Self {
        Self {
            version,
            source: None,
            checksum: None,
            dependencies: Vec::new(),
        }
    }

    /// Create entry from registry
    pub fn from_registry(version: Version, registry: impl Into<String>) -> Self {
        Self {
            version,
            source: Some(registry.into()),
            checksum: None,
            dependencies: Vec::new(),
        }
    }

    /// Create entry from git
    pub fn from_git(version: Version, git_url: impl Into<String>) -> Self {
        Self {
            version,
            source: Some(git_url.into()),
            checksum: None,
            dependencies: Vec::new(),
        }
    }

    /// Set checksum
    pub fn with_checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = Some(checksum.into());
        self
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dep: LockedDependency) {
        self.dependencies.push(dep);
    }
}

/// A locked dependency reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedDependency {
    /// Name of the dependency
    pub name: String,
    /// Version
    pub version: Version,
    /// Type
    pub component_type: ComponentType,
}

impl LockedDependency {
    /// Create a new locked dependency
    pub fn new(name: impl Into<String>, version: Version, component_type: ComponentType) -> Self {
        Self {
            name: name.into(),
            version,
            component_type,
        }
    }
}

/// Lockfile metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LockfileMetadata {
    /// Last generated timestamp
    #[serde(default)]
    pub generated_at: Option<String>,
    /// OxideKit CLI version that generated this
    #[serde(default)]
    pub generator_version: Option<String>,
}

impl LockfileMetadata {
    /// Update timestamp
    pub fn update(&mut self) {
        self.generated_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

/// Lockfile version for compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileVersion {
    /// Schema version
    pub schema: String,
    /// Minimum CLI version required
    pub min_cli_version: Version,
}

impl LockfileVersion {
    /// Current lockfile version
    pub fn current() -> Self {
        Self {
            schema: LOCKFILE_VERSION.to_string(),
            min_cli_version: Version::new(0, 1, 0),
        }
    }

    /// Check if this lockfile version is compatible
    pub fn is_compatible(&self, cli_version: &Version) -> bool {
        cli_version >= &self.min_cli_version
    }
}

/// Differences between two lockfiles
#[derive(Debug, Clone, Default)]
pub struct LockfileDiff {
    /// Added components
    pub added: Vec<LockfileChange>,
    /// Removed components
    pub removed: Vec<LockfileChange>,
    /// Changed components
    pub changed: Vec<LockfileChange>,
}

impl LockfileDiff {
    /// Create empty diff
    pub fn new() -> Self {
        Self::default()
    }

    /// Compare two maps of components
    fn compare_maps(
        &mut self,
        component_type: ComponentType,
        old: &HashMap<String, LockfileEntry>,
        new: &HashMap<String, LockfileEntry>,
    ) {
        // Find removed and changed
        for (name, old_entry) in old {
            match new.get(name) {
                Some(new_entry) if old_entry.version != new_entry.version => {
                    self.changed.push(LockfileChange {
                        component_type,
                        name: name.clone(),
                        from_version: Some(old_entry.version.clone()),
                        to_version: Some(new_entry.version.clone()),
                    });
                }
                None => {
                    self.removed.push(LockfileChange {
                        component_type,
                        name: name.clone(),
                        from_version: Some(old_entry.version.clone()),
                        to_version: None,
                    });
                }
                _ => {} // Same version, no change
            }
        }

        // Find added
        for (name, new_entry) in new {
            if !old.contains_key(name) {
                self.added.push(LockfileChange {
                    component_type,
                    name: name.clone(),
                    from_version: None,
                    to_version: Some(new_entry.version.clone()),
                });
            }
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.changed.is_empty()
    }

    /// Format as a report
    pub fn to_report(&self) -> String {
        if !self.has_changes() {
            return "No changes.".to_string();
        }

        let mut output = String::new();

        if !self.added.is_empty() {
            output.push_str("Added:\n");
            for change in &self.added {
                output.push_str(&format!(
                    "  + {} {} ({})\n",
                    change.name,
                    change.to_version.as_ref().unwrap(),
                    change.component_type
                ));
            }
        }

        if !self.removed.is_empty() {
            output.push_str("Removed:\n");
            for change in &self.removed {
                output.push_str(&format!(
                    "  - {} {} ({})\n",
                    change.name,
                    change.from_version.as_ref().unwrap(),
                    change.component_type
                ));
            }
        }

        if !self.changed.is_empty() {
            output.push_str("Changed:\n");
            for change in &self.changed {
                output.push_str(&format!(
                    "  ~ {} {} -> {} ({})\n",
                    change.name,
                    change.from_version.as_ref().unwrap(),
                    change.to_version.as_ref().unwrap(),
                    change.component_type
                ));
            }
        }

        output
    }
}

/// A single change in the lockfile
#[derive(Debug, Clone)]
pub struct LockfileChange {
    /// Component type
    pub component_type: ComponentType,
    /// Component name
    pub name: String,
    /// Previous version (None if added)
    pub from_version: Option<Version>,
    /// New version (None if removed)
    pub to_version: Option<Version>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_creation() {
        let lockfile = Lockfile::new(Version::parse("0.5.0").unwrap());
        assert_eq!(lockfile.version, LOCKFILE_VERSION);
        assert_eq!(lockfile.oxidekit.version, Version::parse("0.5.0").unwrap());
    }

    #[test]
    fn test_add_components() {
        let mut lockfile = Lockfile::new(Version::parse("0.5.0").unwrap());

        lockfile.add_plugin(
            "icons",
            LockfileEntry::new(Version::parse("1.0.0").unwrap()),
        );

        lockfile.add_theme(
            "dark-mode",
            LockfileEntry::new(Version::parse("2.0.0").unwrap()),
        );

        assert_eq!(lockfile.component_count(), 3); // core + plugin + theme
        assert!(lockfile.has_components());
    }

    #[test]
    fn test_get_component() {
        let mut lockfile = Lockfile::new(Version::parse("0.5.0").unwrap());
        lockfile.add_plugin(
            "icons",
            LockfileEntry::new(Version::parse("1.0.0").unwrap()),
        );

        let plugin = lockfile.get(ComponentType::Plugin, "icons");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().version, Version::parse("1.0.0").unwrap());
    }

    #[test]
    fn test_toml_roundtrip() {
        let mut lockfile = Lockfile::new(Version::parse("0.5.0").unwrap());
        lockfile.add_plugin(
            "icons",
            LockfileEntry::from_registry(
                Version::parse("1.0.0").unwrap(),
                "https://registry.oxidekit.com",
            ),
        );

        let toml_str = lockfile.to_toml().unwrap();
        let parsed = Lockfile::from_toml(&toml_str).unwrap();

        assert_eq!(parsed.oxidekit.version, lockfile.oxidekit.version);
        assert!(parsed.plugins.contains_key("icons"));
    }

    #[test]
    fn test_diff() {
        let mut old = Lockfile::new(Version::parse("0.5.0").unwrap());
        old.add_plugin("icons", LockfileEntry::new(Version::parse("1.0.0").unwrap()));
        old.add_plugin("forms", LockfileEntry::new(Version::parse("1.0.0").unwrap()));

        let mut new = Lockfile::new(Version::parse("0.6.0").unwrap());
        new.add_plugin("icons", LockfileEntry::new(Version::parse("2.0.0").unwrap()));
        new.add_plugin("charts", LockfileEntry::new(Version::parse("1.0.0").unwrap()));

        let diff = old.diff(&new);

        assert!(diff.has_changes());
        assert_eq!(diff.added.len(), 1); // charts
        assert_eq!(diff.removed.len(), 1); // forms
        assert_eq!(diff.changed.len(), 2); // oxidekit + icons
    }
}
