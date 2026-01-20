//! Compatibility matrix for ecosystem components
//!
//! Generates and queries compatibility matrices for OxideKit components.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::semver::Version;
use crate::constraint::VersionReq;
use crate::compatibility::{ComponentType, ComponentVersion};

/// A compatibility matrix entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixEntry {
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: ComponentType,
    /// Component version
    pub version: Version,
    /// Compatible core versions
    pub core_versions: VersionReq,
    /// Compatible plugin versions (name -> version req)
    #[serde(default)]
    pub compatible_plugins: HashMap<String, VersionReq>,
    /// Compatible theme versions (name -> version req)
    #[serde(default)]
    pub compatible_themes: HashMap<String, VersionReq>,
    /// Status of this entry
    pub status: EntryStatus,
}

/// Status of a matrix entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryStatus {
    /// Actively supported
    Active,
    /// Deprecated (works but not recommended)
    Deprecated,
    /// End of life (no longer supported)
    EndOfLife,
    /// Development/pre-release
    Development,
}

impl Default for EntryStatus {
    fn default() -> Self {
        EntryStatus::Active
    }
}

/// The full compatibility matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityMatrix {
    /// Matrix version/schema
    pub version: String,
    /// Last updated timestamp
    pub updated: String,
    /// Core versions
    pub core: Vec<CoreEntry>,
    /// Plugin entries
    pub plugins: Vec<MatrixEntry>,
    /// Theme entries
    pub themes: Vec<MatrixEntry>,
    /// Starter entries
    pub starters: Vec<MatrixEntry>,
}

/// Core version entry in the matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreEntry {
    /// Version
    pub version: Version,
    /// Release date
    pub release_date: String,
    /// Status
    pub status: EntryStatus,
    /// Minimum Rust version
    pub min_rust: String,
    /// Breaking changes from previous version
    #[serde(default)]
    pub breaking_changes: Vec<String>,
}

impl CompatibilityMatrix {
    /// Create a new empty matrix
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            updated: chrono::Utc::now().to_rfc3339(),
            core: Vec::new(),
            plugins: Vec::new(),
            themes: Vec::new(),
            starters: Vec::new(),
        }
    }

    /// Add a core version
    pub fn add_core(&mut self, entry: CoreEntry) {
        self.core.push(entry);
        self.core.sort_by(|a, b| b.version.cmp(&a.version));
    }

    /// Add a plugin entry
    pub fn add_plugin(&mut self, entry: MatrixEntry) {
        self.plugins.push(entry);
    }

    /// Add a theme entry
    pub fn add_theme(&mut self, entry: MatrixEntry) {
        self.themes.push(entry);
    }

    /// Add a starter entry
    pub fn add_starter(&mut self, entry: MatrixEntry) {
        self.starters.push(entry);
    }

    /// Get the latest core version
    pub fn latest_core(&self) -> Option<&CoreEntry> {
        self.core.first()
    }

    /// Get the latest stable core version
    pub fn latest_stable_core(&self) -> Option<&CoreEntry> {
        self.core.iter().find(|c| {
            c.status == EntryStatus::Active && c.version.is_stable()
        })
    }

    /// Find all plugins compatible with a core version
    pub fn plugins_for_core(&self, core_version: &Version) -> Vec<&MatrixEntry> {
        self.plugins.iter()
            .filter(|p| p.core_versions.matches(core_version))
            .collect()
    }

    /// Find all themes compatible with a core version
    pub fn themes_for_core(&self, core_version: &Version) -> Vec<&MatrixEntry> {
        self.themes.iter()
            .filter(|t| t.core_versions.matches(core_version))
            .collect()
    }

    /// Find all starters compatible with a core version
    pub fn starters_for_core(&self, core_version: &Version) -> Vec<&MatrixEntry> {
        self.starters.iter()
            .filter(|s| s.core_versions.matches(core_version))
            .collect()
    }

    /// Find a specific plugin version
    pub fn find_plugin(&self, name: &str, version: &Version) -> Option<&MatrixEntry> {
        self.plugins.iter().find(|p| p.name == name && &p.version == version)
    }

    /// Find a specific theme version
    pub fn find_theme(&self, name: &str, version: &Version) -> Option<&MatrixEntry> {
        self.themes.iter().find(|t| t.name == name && &t.version == version)
    }

    /// Find the latest version of a plugin compatible with a core version
    pub fn latest_plugin_for_core(&self, plugin_name: &str, core_version: &Version) -> Option<&MatrixEntry> {
        self.plugins.iter()
            .filter(|p| p.name == plugin_name && p.core_versions.matches(core_version))
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// Find the latest version of a theme compatible with a core version
    pub fn latest_theme_for_core(&self, theme_name: &str, core_version: &Version) -> Option<&MatrixEntry> {
        self.themes.iter()
            .filter(|t| t.name == theme_name && t.core_versions.matches(core_version))
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// Check if a set of components are mutually compatible
    pub fn check_mutual_compatibility(
        &self,
        components: &[&ComponentVersion],
    ) -> MutualCompatibilityResult {
        let mut result = MutualCompatibilityResult {
            compatible: true,
            issues: Vec::new(),
        };

        // Check each pair of components
        for (i, comp_a) in components.iter().enumerate() {
            for comp_b in components.iter().skip(i + 1) {
                if let Some(issue) = self.check_pair_compatibility(comp_a, comp_b) {
                    result.compatible = false;
                    result.issues.push(issue);
                }
            }
        }

        result
    }

    /// Check compatibility between two components
    fn check_pair_compatibility(
        &self,
        a: &ComponentVersion,
        b: &ComponentVersion,
    ) -> Option<CompatibilityIssue> {
        // Find matrix entries
        let entry_a = match a.component_type {
            ComponentType::Plugin => self.find_plugin(&a.name, &a.version),
            ComponentType::Theme => self.find_theme(&a.name, &a.version),
            _ => None,
        };

        let entry_b = match b.component_type {
            ComponentType::Plugin => self.find_plugin(&b.name, &b.version),
            ComponentType::Theme => self.find_theme(&b.name, &b.version),
            _ => None,
        };

        // Check if A has specific requirements for B
        if let Some(entry) = entry_a {
            let compatible_map = match b.component_type {
                ComponentType::Plugin => &entry.compatible_plugins,
                ComponentType::Theme => &entry.compatible_themes,
                _ => return None,
            };

            if let Some(req) = compatible_map.get(&b.name) {
                if !req.matches(&b.version) {
                    return Some(CompatibilityIssue {
                        component_a: a.name.clone(),
                        version_a: a.version.clone(),
                        component_b: b.name.clone(),
                        version_b: b.version.clone(),
                        reason: format!(
                            "{} {} requires {} {}, but {} is specified",
                            a.name, a.version, b.name, req, b.version
                        ),
                    });
                }
            }
        }

        // Check if B has specific requirements for A
        if let Some(entry) = entry_b {
            let compatible_map = match a.component_type {
                ComponentType::Plugin => &entry.compatible_plugins,
                ComponentType::Theme => &entry.compatible_themes,
                _ => return None,
            };

            if let Some(req) = compatible_map.get(&a.name) {
                if !req.matches(&a.version) {
                    return Some(CompatibilityIssue {
                        component_a: b.name.clone(),
                        version_a: b.version.clone(),
                        component_b: a.name.clone(),
                        version_b: a.version.clone(),
                        reason: format!(
                            "{} {} requires {} {}, but {} is specified",
                            b.name, b.version, a.name, req, a.version
                        ),
                    });
                }
            }
        }

        None
    }

    /// Generate a human-readable compatibility report
    pub fn generate_report(&self, core_version: &Version) -> CompatibilityReport {
        let plugins = self.plugins_for_core(core_version);
        let themes = self.themes_for_core(core_version);
        let starters = self.starters_for_core(core_version);

        CompatibilityReport {
            core_version: core_version.clone(),
            compatible_plugins: plugins.len(),
            compatible_themes: themes.len(),
            compatible_starters: starters.len(),
            plugin_list: plugins.iter().map(|p| format!("{} {}", p.name, p.version)).collect(),
            theme_list: themes.iter().map(|t| format!("{} {}", t.name, t.version)).collect(),
            starter_list: starters.iter().map(|s| format!("{} {}", s.name, s.version)).collect(),
        }
    }

    /// Parse matrix from TOML
    pub fn from_toml(toml_str: &str) -> Result<Self, crate::error::VersionError> {
        toml::from_str(toml_str)
            .map_err(|e| crate::error::VersionError::ManifestParse(e.to_string()))
    }

    /// Serialize matrix to TOML
    pub fn to_toml(&self) -> Result<String, crate::error::VersionError> {
        toml::to_string_pretty(self)
            .map_err(|e| crate::error::VersionError::ManifestParse(e.to_string()))
    }
}

impl Default for CompatibilityMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of mutual compatibility check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualCompatibilityResult {
    /// Whether all components are mutually compatible
    pub compatible: bool,
    /// List of compatibility issues
    pub issues: Vec<CompatibilityIssue>,
}

/// A compatibility issue between two components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    /// First component name
    pub component_a: String,
    /// First component version
    pub version_a: Version,
    /// Second component name
    pub component_b: String,
    /// Second component version
    pub version_b: Version,
    /// Reason for incompatibility
    pub reason: String,
}

/// A compatibility report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    /// Core version
    pub core_version: Version,
    /// Number of compatible plugins
    pub compatible_plugins: usize,
    /// Number of compatible themes
    pub compatible_themes: usize,
    /// Number of compatible starters
    pub compatible_starters: usize,
    /// List of compatible plugins
    pub plugin_list: Vec<String>,
    /// List of compatible themes
    pub theme_list: Vec<String>,
    /// List of compatible starters
    pub starter_list: Vec<String>,
}

impl CompatibilityReport {
    /// Format as human-readable text
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Compatibility Report for OxideKit {}\n",
            self.core_version
        ));
        output.push_str(&format!("{}\n\n", "=".repeat(50)));

        output.push_str(&format!(
            "Compatible Components:\n"
        ));
        output.push_str(&format!("  Plugins: {}\n", self.compatible_plugins));
        output.push_str(&format!("  Themes:  {}\n", self.compatible_themes));
        output.push_str(&format!("  Starters: {}\n", self.compatible_starters));

        if !self.plugin_list.is_empty() {
            output.push_str("\nPlugins:\n");
            for plugin in &self.plugin_list {
                output.push_str(&format!("  - {}\n", plugin));
            }
        }

        if !self.theme_list.is_empty() {
            output.push_str("\nThemes:\n");
            for theme in &self.theme_list {
                output.push_str(&format!("  - {}\n", theme));
            }
        }

        if !self.starter_list.is_empty() {
            output.push_str("\nStarters:\n");
            for starter in &self.starter_list {
                output.push_str(&format!("  - {}\n", starter));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_matrix() -> CompatibilityMatrix {
        let mut matrix = CompatibilityMatrix::new();

        matrix.add_core(CoreEntry {
            version: Version::parse("0.5.0").unwrap(),
            release_date: "2024-01-01".to_string(),
            status: EntryStatus::Active,
            min_rust: "1.70".to_string(),
            breaking_changes: vec![],
        });

        matrix.add_core(CoreEntry {
            version: Version::parse("0.6.0").unwrap(),
            release_date: "2024-06-01".to_string(),
            status: EntryStatus::Active,
            min_rust: "1.75".to_string(),
            breaking_changes: vec!["Removed deprecated Widget::render method".to_string()],
        });

        matrix.add_plugin(MatrixEntry {
            name: "icons".to_string(),
            component_type: ComponentType::Plugin,
            version: Version::parse("1.0.0").unwrap(),
            core_versions: VersionReq::parse(">=0.5.0").unwrap(),
            compatible_plugins: HashMap::new(),
            compatible_themes: HashMap::new(),
            status: EntryStatus::Active,
        });

        matrix.add_plugin(MatrixEntry {
            name: "icons".to_string(),
            component_type: ComponentType::Plugin,
            version: Version::parse("2.0.0").unwrap(),
            core_versions: VersionReq::parse(">=0.6.0").unwrap(),
            compatible_plugins: HashMap::new(),
            compatible_themes: HashMap::new(),
            status: EntryStatus::Active,
        });

        matrix.add_theme(MatrixEntry {
            name: "dark-mode".to_string(),
            component_type: ComponentType::Theme,
            version: Version::parse("1.0.0").unwrap(),
            core_versions: VersionReq::parse(">=0.5.0").unwrap(),
            compatible_plugins: HashMap::new(),
            compatible_themes: HashMap::new(),
            status: EntryStatus::Active,
        });

        matrix
    }

    #[test]
    fn test_latest_core() {
        let matrix = create_test_matrix();
        let latest = matrix.latest_core().unwrap();
        assert_eq!(latest.version, Version::parse("0.6.0").unwrap());
    }

    #[test]
    fn test_plugins_for_core() {
        let matrix = create_test_matrix();

        let v05 = Version::parse("0.5.0").unwrap();
        let v06 = Version::parse("0.6.0").unwrap();

        let plugins_05 = matrix.plugins_for_core(&v05);
        assert_eq!(plugins_05.len(), 1);
        assert_eq!(plugins_05[0].version, Version::parse("1.0.0").unwrap());

        let plugins_06 = matrix.plugins_for_core(&v06);
        assert_eq!(plugins_06.len(), 2);
    }

    #[test]
    fn test_latest_plugin_for_core() {
        let matrix = create_test_matrix();

        let v06 = Version::parse("0.6.0").unwrap();
        let latest = matrix.latest_plugin_for_core("icons", &v06).unwrap();
        assert_eq!(latest.version, Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_generate_report() {
        let matrix = create_test_matrix();
        let v06 = Version::parse("0.6.0").unwrap();
        let report = matrix.generate_report(&v06);

        assert_eq!(report.compatible_plugins, 2);
        assert_eq!(report.compatible_themes, 1);
    }
}
