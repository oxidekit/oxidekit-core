//! Extension Specifications
//!
//! Defines the structure for OxideKit extensions and their capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specification for an OxideKit extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionSpec {
    /// Unique extension ID (e.g., "oxide.charts")
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Extension version
    pub version: String,

    /// Extension kind
    pub kind: ExtensionKind,

    /// Compatibility range with core
    pub compatibility: CompatibilityRange,

    /// Capabilities this extension provides
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// Permissions this extension requires
    #[serde(default)]
    pub permissions: Vec<Permission>,

    /// Components provided by this extension
    #[serde(default)]
    pub components: Vec<String>,

    /// Dependencies on other extensions
    #[serde(default)]
    pub dependencies: Vec<ExtensionDependency>,

    /// Installation instructions
    pub install: InstallInfo,

    /// API entrypoints exposed to apps
    #[serde(default)]
    pub api: Vec<ApiEntrypoint>,

    /// Author information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Repository URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// License
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Types of extensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionKind {
    /// UI component pack
    Components,
    /// Theme/design pack
    Theme,
    /// Data integration
    DataProvider,
    /// Platform capability
    Platform,
    /// Development tooling
    DevTools,
    /// Other/custom
    Other,
}

impl std::fmt::Display for ExtensionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Components => write!(f, "components"),
            Self::Theme => write!(f, "theme"),
            Self::DataProvider => write!(f, "data_provider"),
            Self::Platform => write!(f, "platform"),
            Self::DevTools => write!(f, "devtools"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Version compatibility range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityRange {
    /// Minimum supported core version
    pub min: String,

    /// Maximum supported core version (if known)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,
}

impl CompatibilityRange {
    pub fn from_min(min: &str) -> Self {
        Self {
            min: min.to_string(),
            max: None,
        }
    }

    pub fn new(min: &str, max: &str) -> Self {
        Self {
            min: min.to_string(),
            max: Some(max.to_string()),
        }
    }

    /// Check if a version is compatible
    pub fn is_compatible(&self, version: &str) -> bool {
        match (semver::Version::parse(&self.min), semver::Version::parse(version)) {
            (Ok(min), Ok(ver)) => {
                if ver < min {
                    return false;
                }
                if let Some(max) = &self.max {
                    if let Ok(max) = semver::Version::parse(max) {
                        return ver <= max;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

/// Capability an extension provides
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Capability {
    /// Capability identifier
    pub id: String,

    /// Description
    pub description: String,

    /// Whether this capability is optional
    #[serde(default)]
    pub optional: bool,
}

impl Capability {
    pub fn new(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            optional: false,
        }
    }

    pub fn optional(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            optional: true,
        }
    }
}

/// Permission an extension requires
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    /// Permission identifier
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Whether this is required or optional
    #[serde(default)]
    pub required: bool,
}

impl Permission {
    pub fn required(id: &str, description: &str, risk: RiskLevel) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            risk_level: risk,
            required: true,
        }
    }

    pub fn optional(id: &str, description: &str, risk: RiskLevel) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            risk_level: risk,
            required: false,
        }
    }
}

/// Risk level for permissions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// No special access needed
    None,
    /// Limited access, generally safe
    Low,
    /// Access that could affect user data
    Medium,
    /// Potentially dangerous access
    High,
    /// System-level or security-critical
    Critical,
}

/// Common permission IDs
pub mod permissions {
    use super::*;

    pub fn network_read() -> Permission {
        Permission::required(
            "network.read",
            "Read data from network",
            RiskLevel::Low,
        )
    }

    pub fn network_write() -> Permission {
        Permission::required(
            "network.write",
            "Send data over network",
            RiskLevel::Medium,
        )
    }

    pub fn filesystem_read() -> Permission {
        Permission::required(
            "filesystem.read",
            "Read files from disk",
            RiskLevel::Medium,
        )
    }

    pub fn filesystem_write() -> Permission {
        Permission::required(
            "filesystem.write",
            "Write files to disk",
            RiskLevel::High,
        )
    }

    pub fn clipboard_read() -> Permission {
        Permission::required(
            "clipboard.read",
            "Read clipboard contents",
            RiskLevel::Low,
        )
    }

    pub fn clipboard_write() -> Permission {
        Permission::required(
            "clipboard.write",
            "Write to clipboard",
            RiskLevel::Low,
        )
    }

    pub fn notification() -> Permission {
        Permission::required(
            "notification",
            "Display system notifications",
            RiskLevel::Low,
        )
    }

    pub fn camera() -> Permission {
        Permission::required(
            "camera",
            "Access device camera",
            RiskLevel::High,
        )
    }

    pub fn microphone() -> Permission {
        Permission::required(
            "microphone",
            "Access device microphone",
            RiskLevel::High,
        )
    }

    pub fn geolocation() -> Permission {
        Permission::required(
            "geolocation",
            "Access device location",
            RiskLevel::Medium,
        )
    }
}

/// Dependency on another extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDependency {
    /// Extension ID
    pub extension_id: String,

    /// Required version range
    pub version: String,

    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

/// Installation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallInfo {
    /// Package name for cargo/npm
    pub package: String,

    /// Installation command
    pub command: String,

    /// Additional setup steps
    #[serde(default)]
    pub setup_steps: Vec<String>,

    /// Configuration file changes
    #[serde(default)]
    pub config_changes: Vec<ConfigChange>,
}

/// Configuration file change required for installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    /// File path (relative to project root)
    pub file: String,

    /// What to add/change
    pub change: String,

    /// Description of the change
    pub description: String,
}

/// API entrypoint exposed by an extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEntrypoint {
    /// Entrypoint name
    pub name: String,

    /// Description
    pub description: String,

    /// Function signature
    pub signature: String,

    /// Example usage
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// Builder for creating extension specs
pub struct ExtensionSpecBuilder {
    spec: ExtensionSpec,
}

impl ExtensionSpecBuilder {
    pub fn new(id: &str, kind: ExtensionKind) -> Self {
        Self {
            spec: ExtensionSpec {
                id: id.to_string(),
                name: id.to_string(),
                description: String::new(),
                version: "0.1.0".to_string(),
                kind,
                compatibility: CompatibilityRange::from_min("0.1.0"),
                capabilities: Vec::new(),
                permissions: Vec::new(),
                components: Vec::new(),
                dependencies: Vec::new(),
                install: InstallInfo {
                    package: id.to_string(),
                    command: format!("cargo add {}", id),
                    setup_steps: Vec::new(),
                    config_changes: Vec::new(),
                },
                api: Vec::new(),
                author: None,
                repository: None,
                license: None,
            },
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.spec.name = name.to_string();
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.spec.description = desc.to_string();
        self
    }

    pub fn version(mut self, version: &str) -> Self {
        self.spec.version = version.to_string();
        self
    }

    pub fn compatibility(mut self, min: &str, max: Option<&str>) -> Self {
        self.spec.compatibility = CompatibilityRange {
            min: min.to_string(),
            max: max.map(String::from),
        };
        self
    }

    pub fn capability(mut self, cap: Capability) -> Self {
        self.spec.capabilities.push(cap);
        self
    }

    pub fn permission(mut self, perm: Permission) -> Self {
        self.spec.permissions.push(perm);
        self
    }

    pub fn component(mut self, component_id: &str) -> Self {
        self.spec.components.push(component_id.to_string());
        self
    }

    pub fn dependency(mut self, ext_id: &str, version: &str) -> Self {
        self.spec.dependencies.push(ExtensionDependency {
            extension_id: ext_id.to_string(),
            version: version.to_string(),
            optional: false,
        });
        self
    }

    pub fn author(mut self, author: &str) -> Self {
        self.spec.author = Some(author.to_string());
        self
    }

    pub fn license(mut self, license: &str) -> Self {
        self.spec.license = Some(license.to_string());
        self
    }

    pub fn build(self) -> ExtensionSpec {
        self.spec
    }
}

/// Get core extensions bundled with OxideKit
pub fn core_extensions() -> Vec<ExtensionSpec> {
    vec![
        ExtensionSpecBuilder::new("oxide.charts", ExtensionKind::Components)
            .name("OxideKit Charts")
            .description("Data visualization components: line, bar, pie, area charts")
            .version("0.1.0")
            .compatibility("0.1.0", None)
            .capability(Capability::new("charts.render", "Render chart visualizations"))
            .component("ui.LineChart")
            .component("ui.BarChart")
            .component("ui.PieChart")
            .component("ui.AreaChart")
            .build(),

        ExtensionSpecBuilder::new("oxide.forms", ExtensionKind::Components)
            .name("OxideKit Forms")
            .description("Advanced form components with validation")
            .version("0.1.0")
            .compatibility("0.1.0", None)
            .capability(Capability::new("forms.validation", "Form validation"))
            .component("ui.Form")
            .component("ui.TextField")
            .component("ui.TextArea")
            .component("ui.Select")
            .component("ui.Checkbox")
            .component("ui.Radio")
            .component("ui.Switch")
            .component("ui.DatePicker")
            .build(),

        ExtensionSpecBuilder::new("oxide.tables", ExtensionKind::Components)
            .name("OxideKit Tables")
            .description("Advanced data table with sorting, filtering, pagination")
            .version("0.1.0")
            .compatibility("0.1.0", None)
            .capability(Capability::new("tables.virtual", "Virtual scrolling for large datasets"))
            .component("ui.DataTable")
            .component("ui.TableColumn")
            .component("ui.TableActions")
            .build(),

        ExtensionSpecBuilder::new("oxide.icons", ExtensionKind::Components)
            .name("OxideKit Icons")
            .description("Comprehensive icon library")
            .version("0.1.0")
            .compatibility("0.1.0", None)
            .component("ui.Icon")
            .build(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_builder() {
        let ext = ExtensionSpecBuilder::new("test.ext", ExtensionKind::Components)
            .name("Test Extension")
            .description("A test extension")
            .version("1.0.0")
            .component("ui.TestComponent")
            .build();

        assert_eq!(ext.id, "test.ext");
        assert_eq!(ext.kind, ExtensionKind::Components);
        assert!(ext.components.contains(&"ui.TestComponent".to_string()));
    }

    #[test]
    fn test_compatibility_check() {
        let range = CompatibilityRange::new("0.1.0", "0.5.0");
        assert!(range.is_compatible("0.3.0"));
        assert!(!range.is_compatible("0.0.9"));
        assert!(!range.is_compatible("0.6.0"));
    }

    #[test]
    fn test_core_extensions() {
        let exts = core_extensions();
        assert!(!exts.is_empty());
        assert!(exts.iter().any(|e| e.id == "oxide.charts"));
    }
}
