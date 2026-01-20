//! Compatibility rules for OxideKit ecosystem components
//!
//! Defines compatibility rules for Core, Plugins, Themes, and Starters.

use std::fmt;
use serde::{Deserialize, Serialize};
use crate::semver::Version;
use crate::constraint::VersionReq;
use crate::error::VersionError;

/// Types of components in the OxideKit ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentType {
    /// OxideKit core runtime
    Core,
    /// Plugin (extends functionality)
    Plugin,
    /// Theme (styling and appearance)
    Theme,
    /// Starter template (project scaffolding)
    Starter,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentType::Core => write!(f, "core"),
            ComponentType::Plugin => write!(f, "plugin"),
            ComponentType::Theme => write!(f, "theme"),
            ComponentType::Starter => write!(f, "starter"),
        }
    }
}

/// A versioned component in the ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVersion {
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: ComponentType,
    /// Current version
    pub version: Version,
    /// Required OxideKit core version
    pub core_requirement: VersionReq,
    /// Dependencies on other components
    #[serde(default)]
    pub dependencies: Vec<ComponentDependency>,
}

impl ComponentVersion {
    /// Create a new component version
    pub fn new(
        name: impl Into<String>,
        component_type: ComponentType,
        version: Version,
        core_requirement: VersionReq,
    ) -> Self {
        Self {
            name: name.into(),
            component_type,
            version,
            core_requirement,
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dep: ComponentDependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Check if this component is compatible with a core version
    pub fn is_compatible_with_core(&self, core_version: &Version) -> bool {
        self.core_requirement.matches(core_version)
    }
}

/// A dependency on another component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDependency {
    /// Name of the required component
    pub name: String,
    /// Type of the required component
    pub component_type: ComponentType,
    /// Version requirement
    pub version_req: VersionReq,
    /// Whether this is an optional dependency
    #[serde(default)]
    pub optional: bool,
}

impl ComponentDependency {
    /// Create a required dependency
    pub fn required(
        name: impl Into<String>,
        component_type: ComponentType,
        version_req: VersionReq,
    ) -> Self {
        Self {
            name: name.into(),
            component_type,
            version_req,
            optional: false,
        }
    }

    /// Create an optional dependency
    pub fn optional(
        name: impl Into<String>,
        component_type: ComponentType,
        version_req: VersionReq,
    ) -> Self {
        Self {
            name: name.into(),
            component_type,
            version_req,
            optional: true,
        }
    }
}

/// Component manifest (oxide.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentManifest {
    /// Package information
    pub package: PackageInfo,
    /// Compatibility requirements
    #[serde(default)]
    pub compatibility: CompatibilitySpec,
    /// Dependencies
    #[serde(default)]
    pub dependencies: DependencySpec,
}

/// Package information in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    #[serde(default)]
    pub description: String,
    /// Component type
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    /// Authors
    #[serde(default)]
    pub authors: Vec<String>,
    /// License
    #[serde(default)]
    pub license: Option<String>,
    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,
}

/// Compatibility specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatibilitySpec {
    /// Required OxideKit core version
    #[serde(default = "default_core_req")]
    pub oxidekit: String,
    /// Minimum Rust version (if applicable)
    #[serde(default)]
    pub rust: Option<String>,
}

fn default_core_req() -> String {
    "*".to_string()
}

/// Dependencies specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencySpec {
    /// Plugin dependencies
    #[serde(default)]
    pub plugins: std::collections::HashMap<String, DependencyEntry>,
    /// Theme dependencies
    #[serde(default)]
    pub themes: std::collections::HashMap<String, DependencyEntry>,
}

/// A dependency entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencyEntry {
    /// Simple version string
    Simple(String),
    /// Detailed specification
    Detailed {
        version: String,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        features: Vec<String>,
    },
}

impl DependencyEntry {
    /// Get the version requirement string
    pub fn version_str(&self) -> &str {
        match self {
            DependencyEntry::Simple(v) => v,
            DependencyEntry::Detailed { version, .. } => version,
        }
    }

    /// Check if this is an optional dependency
    pub fn is_optional(&self) -> bool {
        match self {
            DependencyEntry::Simple(_) => false,
            DependencyEntry::Detailed { optional, .. } => *optional,
        }
    }
}

/// Compatibility check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    /// Whether the check passed
    pub compatible: bool,
    /// Compatibility level
    pub level: CompatibilityLevel,
    /// Component being checked
    pub component: String,
    /// Component version
    pub version: Version,
    /// Required version
    pub required: VersionReq,
    /// Actual version being compared against
    pub actual: Version,
    /// Human-readable explanation
    pub explanation: String,
    /// Suggested action if incompatible
    pub suggestion: Option<String>,
}

impl CompatibilityResult {
    /// Create a compatible result
    pub fn compatible(
        component: impl Into<String>,
        version: Version,
        required: VersionReq,
        actual: Version,
    ) -> Self {
        Self {
            compatible: true,
            level: CompatibilityLevel::Full,
            component: component.into(),
            version,
            required,
            actual,
            explanation: "Versions are compatible".to_string(),
            suggestion: None,
        }
    }

    /// Create an incompatible result
    pub fn incompatible(
        component: impl Into<String>,
        version: Version,
        required: VersionReq,
        actual: Version,
        explanation: impl Into<String>,
        suggestion: Option<String>,
    ) -> Self {
        Self {
            compatible: false,
            level: CompatibilityLevel::None,
            component: component.into(),
            version,
            required,
            actual,
            explanation: explanation.into(),
            suggestion,
        }
    }
}

/// Compatibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatibilityLevel {
    /// Fully compatible (no issues)
    Full,
    /// Compatible with warnings (deprecated features, etc.)
    Partial,
    /// Compatibility unknown (unable to verify)
    Unknown,
    /// Not compatible
    None,
}

impl fmt::Display for CompatibilityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatibilityLevel::Full => write!(f, "compatible"),
            CompatibilityLevel::Partial => write!(f, "partially compatible"),
            CompatibilityLevel::Unknown => write!(f, "unknown compatibility"),
            CompatibilityLevel::None => write!(f, "incompatible"),
        }
    }
}

/// Compatibility checker
pub struct Compatibility {
    /// Current OxideKit core version
    core_version: Version,
}

impl Compatibility {
    /// Create a new compatibility checker
    pub fn new(core_version: Version) -> Self {
        Self { core_version }
    }

    /// Check if a component is compatible with the current core
    pub fn check_component(&self, component: &ComponentVersion) -> CompatibilityResult {
        if component.core_requirement.matches(&self.core_version) {
            CompatibilityResult::compatible(
                &component.name,
                component.version.clone(),
                component.core_requirement.clone(),
                self.core_version.clone(),
            )
        } else {
            let suggestion = self.suggest_fix(component);
            CompatibilityResult::incompatible(
                &component.name,
                component.version.clone(),
                component.core_requirement.clone(),
                self.core_version.clone(),
                format!(
                    "{} {} requires OxideKit {}, but {} is installed",
                    component.component_type,
                    component.name,
                    component.core_requirement,
                    self.core_version
                ),
                suggestion,
            )
        }
    }

    /// Check all dependencies of a component
    pub fn check_dependencies(
        &self,
        component: &ComponentVersion,
        available: &[ComponentVersion],
    ) -> Vec<CompatibilityResult> {
        let mut results = Vec::new();

        for dep in &component.dependencies {
            if dep.optional {
                continue; // Skip optional dependencies
            }

            let found = available.iter().find(|c| {
                c.name == dep.name && c.component_type == dep.component_type
            });

            match found {
                Some(c) => {
                    if dep.version_req.matches(&c.version) {
                        results.push(CompatibilityResult::compatible(
                            &dep.name,
                            c.version.clone(),
                            dep.version_req.clone(),
                            c.version.clone(),
                        ));
                    } else {
                        results.push(CompatibilityResult::incompatible(
                            &dep.name,
                            c.version.clone(),
                            dep.version_req.clone(),
                            c.version.clone(),
                            format!(
                                "{} requires {} {}, but {} is available",
                                component.name,
                                dep.name,
                                dep.version_req,
                                c.version
                            ),
                            Some(format!(
                                "Install {} version {}",
                                dep.name, dep.version_req
                            )),
                        ));
                    }
                }
                None => {
                    results.push(CompatibilityResult {
                        compatible: false,
                        level: CompatibilityLevel::None,
                        component: dep.name.clone(),
                        version: Version::new(0, 0, 0),
                        required: dep.version_req.clone(),
                        actual: Version::new(0, 0, 0),
                        explanation: format!(
                            "Required {} '{}' is not installed",
                            dep.component_type, dep.name
                        ),
                        suggestion: Some(format!(
                            "Install {} version {}",
                            dep.name, dep.version_req
                        )),
                    });
                }
            }
        }

        results
    }

    /// Suggest a fix for an incompatible component
    fn suggest_fix(&self, component: &ComponentVersion) -> Option<String> {
        // Check if upgrading core would help
        if let Some(min) = component.core_requirement.minimum_version() {
            if min > self.core_version {
                return Some(format!(
                    "Upgrade OxideKit core to version {} or higher",
                    min
                ));
            }
        }

        // Check if downgrading component might help
        if let Some(max) = component.core_requirement.maximum_version() {
            if max <= self.core_version {
                return Some(format!(
                    "Use an older version of {} compatible with OxideKit {}",
                    component.name, self.core_version
                ));
            }
        }

        None
    }

    /// Check if an upgrade from one version to another is safe
    pub fn check_upgrade_safety(
        &self,
        from: &ComponentVersion,
        to: &ComponentVersion,
    ) -> UpgradeAnalysis {
        let mut analysis = UpgradeAnalysis {
            safe: true,
            warnings: Vec::new(),
            blocking_issues: Vec::new(),
        };

        // Check major version bump
        if to.version.major > from.version.major {
            analysis.warnings.push(format!(
                "Major version upgrade from {} to {} may include breaking changes",
                from.version, to.version
            ));
        }

        // Check core requirement changes
        if from.core_requirement != to.core_requirement {
            if !to.core_requirement.matches(&self.core_version) {
                analysis.safe = false;
                analysis.blocking_issues.push(format!(
                    "New version requires OxideKit {}, but {} is installed",
                    to.core_requirement, self.core_version
                ));
            }
        }

        // Check if dependencies changed
        for new_dep in &to.dependencies {
            let old_dep = from.dependencies.iter().find(|d| d.name == new_dep.name);
            match old_dep {
                Some(old) if old.version_req != new_dep.version_req => {
                    analysis.warnings.push(format!(
                        "Dependency '{}' requirement changed from {} to {}",
                        new_dep.name, old.version_req, new_dep.version_req
                    ));
                }
                None if !new_dep.optional => {
                    analysis.warnings.push(format!(
                        "New required dependency: {} {}",
                        new_dep.name, new_dep.version_req
                    ));
                }
                _ => {}
            }
        }

        analysis
    }
}

/// Result of analyzing an upgrade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeAnalysis {
    /// Whether the upgrade is safe to proceed
    pub safe: bool,
    /// Warnings (non-blocking)
    pub warnings: Vec<String>,
    /// Blocking issues that prevent the upgrade
    pub blocking_issues: Vec<String>,
}

impl UpgradeAnalysis {
    /// Format the analysis as a human-readable report
    pub fn to_report(&self) -> String {
        let mut report = String::new();

        if self.safe {
            report.push_str("Upgrade is safe to proceed.\n");
        } else {
            report.push_str("Upgrade is BLOCKED due to compatibility issues:\n");
            for issue in &self.blocking_issues {
                report.push_str(&format!("  - {}\n", issue));
            }
        }

        if !self.warnings.is_empty() {
            report.push_str("\nWarnings:\n");
            for warning in &self.warnings {
                report.push_str(&format!("  - {}\n", warning));
            }
        }

        report
    }
}

/// Parse a component manifest from TOML
pub fn parse_manifest(toml_str: &str) -> Result<ComponentManifest, VersionError> {
    toml::from_str(toml_str)
        .map_err(|e| VersionError::ManifestParse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_version() {
        let comp = ComponentVersion::new(
            "my-plugin",
            ComponentType::Plugin,
            Version::parse("1.0.0").unwrap(),
            VersionReq::parse(">=0.5.0, <1.0.0").unwrap(),
        );

        assert!(comp.is_compatible_with_core(&Version::parse("0.5.0").unwrap()));
        assert!(comp.is_compatible_with_core(&Version::parse("0.9.0").unwrap()));
        assert!(!comp.is_compatible_with_core(&Version::parse("1.0.0").unwrap()));
    }

    #[test]
    fn test_compatibility_check() {
        let compat = Compatibility::new(Version::parse("0.8.0").unwrap());

        let comp = ComponentVersion::new(
            "my-plugin",
            ComponentType::Plugin,
            Version::parse("1.0.0").unwrap(),
            VersionReq::parse(">=0.5.0, <1.0.0").unwrap(),
        );

        let result = compat.check_component(&comp);
        assert!(result.compatible);
    }

    #[test]
    fn test_incompatibility_check() {
        let compat = Compatibility::new(Version::parse("1.0.0").unwrap());

        let comp = ComponentVersion::new(
            "my-plugin",
            ComponentType::Plugin,
            Version::parse("1.0.0").unwrap(),
            VersionReq::parse(">=0.5.0, <1.0.0").unwrap(),
        );

        let result = compat.check_component(&comp);
        assert!(!result.compatible);
        assert!(result.suggestion.is_some());
    }

    #[test]
    fn test_parse_manifest() {
        let toml_str = r#"
            [package]
            name = "my-theme"
            version = "1.0.0"
            type = "theme"
            description = "A beautiful theme"

            [compatibility]
            oxidekit = ">=0.5.0"

            [dependencies.plugins]
            icons = "^1.0.0"
        "#;

        let manifest = parse_manifest(toml_str).unwrap();
        assert_eq!(manifest.package.name, "my-theme");
        assert_eq!(manifest.package.component_type, ComponentType::Theme);
        assert_eq!(manifest.compatibility.oxidekit, ">=0.5.0");
        assert!(manifest.dependencies.plugins.contains_key("icons"));
    }

    #[test]
    fn test_upgrade_analysis() {
        let compat = Compatibility::new(Version::parse("0.8.0").unwrap());

        let from = ComponentVersion::new(
            "my-plugin",
            ComponentType::Plugin,
            Version::parse("1.0.0").unwrap(),
            VersionReq::parse(">=0.5.0").unwrap(),
        );

        let to = ComponentVersion::new(
            "my-plugin",
            ComponentType::Plugin,
            Version::parse("2.0.0").unwrap(),
            VersionReq::parse(">=0.5.0").unwrap(),
        );

        let analysis = compat.check_upgrade_safety(&from, &to);
        assert!(analysis.safe);
        assert!(!analysis.warnings.is_empty()); // Major version warning
    }
}
