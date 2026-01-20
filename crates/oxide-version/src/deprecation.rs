//! Deprecation warnings system
//!
//! Provides structured deprecation warnings for APIs, features, and components.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::semver::Version;

/// Deprecation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeprecationLevel {
    /// Soft deprecation: still works, but not recommended
    Soft,
    /// Hard deprecation: will be removed soon, shows warning
    Hard,
    /// Removed: no longer available, shows error
    Removed,
}

impl Default for DeprecationLevel {
    fn default() -> Self {
        DeprecationLevel::Soft
    }
}

impl std::fmt::Display for DeprecationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeprecationLevel::Soft => write!(f, "deprecated"),
            DeprecationLevel::Hard => write!(f, "deprecated (will be removed)"),
            DeprecationLevel::Removed => write!(f, "removed"),
        }
    }
}

/// A deprecation notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deprecation {
    /// What is deprecated (API, feature, component name)
    pub subject: String,
    /// Deprecation level
    pub level: DeprecationLevel,
    /// Version when deprecated
    pub deprecated_in: Version,
    /// Version when it will be/was removed
    pub removal_version: Option<Version>,
    /// Reason for deprecation
    pub reason: String,
    /// Migration path or replacement
    pub migration: Option<String>,
    /// Link to documentation
    pub docs_url: Option<String>,
}

impl Deprecation {
    /// Create a new soft deprecation
    pub fn soft(subject: impl Into<String>, deprecated_in: Version, reason: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            level: DeprecationLevel::Soft,
            deprecated_in,
            removal_version: None,
            reason: reason.into(),
            migration: None,
            docs_url: None,
        }
    }

    /// Create a hard deprecation with removal version
    pub fn hard(
        subject: impl Into<String>,
        deprecated_in: Version,
        removal_version: Version,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            level: DeprecationLevel::Hard,
            deprecated_in,
            removal_version: Some(removal_version),
            reason: reason.into(),
            migration: None,
            docs_url: None,
        }
    }

    /// Create a removed notice
    pub fn removed(
        subject: impl Into<String>,
        deprecated_in: Version,
        removed_in: Version,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            level: DeprecationLevel::Removed,
            deprecated_in,
            removal_version: Some(removed_in),
            reason: reason.into(),
            migration: None,
            docs_url: None,
        }
    }

    /// Set migration path
    pub fn with_migration(mut self, migration: impl Into<String>) -> Self {
        self.migration = Some(migration.into());
        self
    }

    /// Set documentation URL
    pub fn with_docs(mut self, docs_url: impl Into<String>) -> Self {
        self.docs_url = Some(docs_url.into());
        self
    }

    /// Check if this deprecation is relevant for a given version
    pub fn is_active_for(&self, version: &Version) -> bool {
        match self.level {
            DeprecationLevel::Soft | DeprecationLevel::Hard => {
                version >= &self.deprecated_in
            }
            DeprecationLevel::Removed => {
                if let Some(ref removal) = self.removal_version {
                    version >= removal
                } else {
                    false
                }
            }
        }
    }

    /// Format as a warning message
    pub fn to_warning(&self) -> DeprecationWarning {
        DeprecationWarning {
            subject: self.subject.clone(),
            level: self.level,
            message: self.format_message(),
            migration: self.migration.clone(),
            docs_url: self.docs_url.clone(),
        }
    }

    /// Format the deprecation message
    fn format_message(&self) -> String {
        let mut msg = format!(
            "'{}' is {} since version {}: {}",
            self.subject, self.level, self.deprecated_in, self.reason
        );

        if let Some(ref removal) = self.removal_version {
            if self.level != DeprecationLevel::Removed {
                msg.push_str(&format!(" (will be removed in {})", removal));
            } else {
                msg.push_str(&format!(" (removed in {})", removal));
            }
        }

        msg
    }
}

/// A deprecation warning to display to users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationWarning {
    /// What is deprecated
    pub subject: String,
    /// Level
    pub level: DeprecationLevel,
    /// Warning message
    pub message: String,
    /// Migration path if available
    pub migration: Option<String>,
    /// Documentation URL
    pub docs_url: Option<String>,
}

impl DeprecationWarning {
    /// Format as colored terminal output
    pub fn to_terminal(&self) -> String {
        let mut output = String::new();

        // Header with severity
        let prefix = match self.level {
            DeprecationLevel::Soft => "DEPRECATED",
            DeprecationLevel::Hard => "WARNING: DEPRECATED",
            DeprecationLevel::Removed => "ERROR: REMOVED",
        };

        output.push_str(&format!("{}: {}\n", prefix, self.message));

        if let Some(ref migration) = self.migration {
            output.push_str(&format!("  Migration: {}\n", migration));
        }

        if let Some(ref docs) = self.docs_url {
            output.push_str(&format!("  Documentation: {}\n", docs));
        }

        output
    }
}

impl std::fmt::Display for DeprecationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_terminal())
    }
}

/// Registry of deprecations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeprecationRegistry {
    /// Deprecations by subject
    deprecations: HashMap<String, Deprecation>,
}

impl DeprecationRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            deprecations: HashMap::new(),
        }
    }

    /// Register a deprecation
    pub fn register(&mut self, deprecation: Deprecation) {
        self.deprecations.insert(deprecation.subject.clone(), deprecation);
    }

    /// Check if a subject is deprecated
    pub fn is_deprecated(&self, subject: &str) -> bool {
        self.deprecations.contains_key(subject)
    }

    /// Get deprecation for a subject
    pub fn get(&self, subject: &str) -> Option<&Deprecation> {
        self.deprecations.get(subject)
    }

    /// Get all deprecations active for a version
    pub fn active_for(&self, version: &Version) -> Vec<&Deprecation> {
        self.deprecations
            .values()
            .filter(|d| d.is_active_for(version))
            .collect()
    }

    /// Get all hard deprecations (items that will be removed)
    pub fn hard_deprecations(&self) -> Vec<&Deprecation> {
        self.deprecations
            .values()
            .filter(|d| d.level == DeprecationLevel::Hard)
            .collect()
    }

    /// Get all removed items
    pub fn removed_items(&self) -> Vec<&Deprecation> {
        self.deprecations
            .values()
            .filter(|d| d.level == DeprecationLevel::Removed)
            .collect()
    }

    /// Generate warnings for a given version
    pub fn warnings_for(&self, version: &Version) -> Vec<DeprecationWarning> {
        self.active_for(version)
            .into_iter()
            .map(|d| d.to_warning())
            .collect()
    }

    /// Check if any removals affect a version upgrade
    pub fn check_upgrade(&self, from: &Version, to: &Version) -> Vec<DeprecationWarning> {
        let mut warnings = Vec::new();

        for deprecation in self.deprecations.values() {
            // Check if something was deprecated between versions
            if deprecation.deprecated_in > *from && deprecation.deprecated_in <= *to {
                warnings.push(deprecation.to_warning());
            }

            // Check if something will be removed in the target version
            if let Some(ref removal) = deprecation.removal_version {
                if removal > from && removal <= to {
                    warnings.push(deprecation.to_warning());
                }
            }
        }

        // Sort by severity (most severe first)
        warnings.sort_by(|a, b| b.level.cmp(&a.level));

        warnings
    }

    /// Get total count
    pub fn len(&self) -> usize {
        self.deprecations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.deprecations.is_empty()
    }
}

/// Builder for creating deprecation registries
pub struct DeprecationRegistryBuilder {
    registry: DeprecationRegistry,
}

impl DeprecationRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: DeprecationRegistry::new(),
        }
    }

    /// Add a soft deprecation
    pub fn deprecate(
        mut self,
        subject: impl Into<String>,
        version: impl AsRef<str>,
        reason: impl Into<String>,
    ) -> Self {
        let version = Version::parse(version.as_ref()).expect("Invalid version");
        self.registry.register(Deprecation::soft(subject, version, reason));
        self
    }

    /// Add a hard deprecation with removal version
    pub fn deprecate_for_removal(
        mut self,
        subject: impl Into<String>,
        deprecated_in: impl AsRef<str>,
        removal_in: impl AsRef<str>,
        reason: impl Into<String>,
    ) -> Self {
        let deprecated = Version::parse(deprecated_in.as_ref()).expect("Invalid version");
        let removal = Version::parse(removal_in.as_ref()).expect("Invalid version");
        self.registry.register(Deprecation::hard(subject, deprecated, removal, reason));
        self
    }

    /// Mark as removed
    pub fn removed(
        mut self,
        subject: impl Into<String>,
        deprecated_in: impl AsRef<str>,
        removed_in: impl AsRef<str>,
        reason: impl Into<String>,
    ) -> Self {
        let deprecated = Version::parse(deprecated_in.as_ref()).expect("Invalid version");
        let removed = Version::parse(removed_in.as_ref()).expect("Invalid version");
        self.registry.register(Deprecation::removed(subject, deprecated, removed, reason));
        self
    }

    /// Build the registry
    pub fn build(self) -> DeprecationRegistry {
        self.registry
    }
}

impl Default for DeprecationRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecation_levels() {
        assert!(DeprecationLevel::Soft < DeprecationLevel::Hard);
        assert!(DeprecationLevel::Hard < DeprecationLevel::Removed);
    }

    #[test]
    fn test_soft_deprecation() {
        let dep = Deprecation::soft(
            "Widget::old_method",
            Version::parse("0.5.0").unwrap(),
            "Use Widget::new_method instead",
        )
        .with_migration("Replace old_method() with new_method()");

        assert_eq!(dep.level, DeprecationLevel::Soft);
        assert!(dep.migration.is_some());
    }

    #[test]
    fn test_hard_deprecation() {
        let dep = Deprecation::hard(
            "deprecated_api",
            Version::parse("0.5.0").unwrap(),
            Version::parse("1.0.0").unwrap(),
            "API is being replaced",
        );

        assert_eq!(dep.level, DeprecationLevel::Hard);
        assert_eq!(dep.removal_version, Some(Version::parse("1.0.0").unwrap()));
    }

    #[test]
    fn test_deprecation_active_for() {
        let dep = Deprecation::soft(
            "test",
            Version::parse("0.5.0").unwrap(),
            "Test reason",
        );

        assert!(!dep.is_active_for(&Version::parse("0.4.0").unwrap()));
        assert!(dep.is_active_for(&Version::parse("0.5.0").unwrap()));
        assert!(dep.is_active_for(&Version::parse("0.6.0").unwrap()));
    }

    #[test]
    fn test_registry() {
        let registry = DeprecationRegistryBuilder::new()
            .deprecate("old_api", "0.5.0", "Use new_api instead")
            .deprecate_for_removal("deprecated_func", "0.5.0", "1.0.0", "Being removed")
            .build();

        assert_eq!(registry.len(), 2);
        assert!(registry.is_deprecated("old_api"));
        assert!(registry.is_deprecated("deprecated_func"));
        assert!(!registry.is_deprecated("other"));
    }

    #[test]
    fn test_upgrade_warnings() {
        let registry = DeprecationRegistryBuilder::new()
            .deprecate("api_a", "0.5.0", "Use api_b")
            .deprecate_for_removal("api_c", "0.4.0", "0.6.0", "Removed in 0.6")
            .build();

        let warnings = registry.check_upgrade(
            &Version::parse("0.4.0").unwrap(),
            &Version::parse("0.6.0").unwrap(),
        );

        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_warning_output() {
        let dep = Deprecation::hard(
            "Widget::render",
            Version::parse("0.5.0").unwrap(),
            Version::parse("1.0.0").unwrap(),
            "Use Widget::draw instead",
        )
        .with_migration("Replace render() with draw()")
        .with_docs("https://docs.oxidekit.com/migration/0.5-to-1.0");

        let warning = dep.to_warning();
        let output = warning.to_terminal();

        assert!(output.contains("WARNING"));
        assert!(output.contains("Widget::render"));
        assert!(output.contains("Migration"));
        assert!(output.contains("Documentation"));
    }
}
