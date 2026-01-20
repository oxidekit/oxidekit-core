//! Plugin manifest schema definition.
//!
//! The `plugin.toml` manifest is the heart of every OxideKit plugin.
//! It defines metadata, capabilities, dependencies, and kind-specific configuration.

use serde::{Deserialize, Serialize};
use semver::{Version, VersionReq};

use crate::namespace::PluginId;
use super::category::{
    PluginCategory, UiConfig, NativeConfig, ServiceConfig,
    ToolingConfig, ThemeConfig, DesignConfig,
};

/// The complete plugin manifest schema.
///
/// # Example `plugin.toml`
///
/// ```toml
/// [plugin]
/// id = "ui.tables"
/// kind = "ui"
/// version = "1.0.0"
/// publisher = "oxidekit"
/// description = "Data tables with sorting, filtering, and pagination"
/// license = "MIT"
/// repository = "https://github.com/oxidekit/oxide-ui-tables"
/// keywords = ["table", "data", "grid", "sorting"]
///
/// [plugin.requires]
/// core = ">=0.1.0"
///
/// [ui]
/// is_pack = false
///
/// [[ui.components]]
/// name = "DataTable"
/// description = "A powerful data table component"
///
/// [[ui.components.props]]
/// name = "columns"
/// type = "Vec<Column>"
/// required = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Core plugin metadata.
    pub plugin: PluginMetadata,

    /// UI-specific configuration.
    #[serde(default)]
    pub ui: Option<UiConfig>,

    /// Native-specific configuration.
    #[serde(default)]
    pub native: Option<NativeConfig>,

    /// Service-specific configuration.
    #[serde(default)]
    pub service: Option<ServiceConfig>,

    /// Tooling-specific configuration.
    #[serde(default)]
    pub tooling: Option<ToolingConfig>,

    /// Theme-specific configuration.
    #[serde(default)]
    pub theme: Option<ThemeConfig>,

    /// Design-specific configuration.
    #[serde(default)]
    pub design: Option<DesignConfig>,

    /// Plugin dependencies.
    #[serde(default)]
    pub dependencies: DependencySection,

    /// Build configuration.
    #[serde(default)]
    pub build: Option<BuildConfig>,
}

/// Core plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier (namespaced).
    pub id: PluginId,

    /// Plugin category/kind.
    pub kind: PluginCategory,

    /// Plugin version (semver).
    #[serde(with = "version_serde")]
    pub version: Version,

    /// Publisher name/organization.
    pub publisher: String,

    /// Short description.
    pub description: String,

    /// License identifier (SPDX).
    #[serde(default = "default_license")]
    pub license: String,

    /// Repository URL.
    #[serde(default)]
    pub repository: Option<String>,

    /// Homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,

    /// Documentation URL.
    #[serde(default)]
    pub documentation: Option<String>,

    /// Keywords/tags for discovery.
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Authors.
    #[serde(default)]
    pub authors: Vec<String>,

    /// Compatibility requirements.
    #[serde(default)]
    pub requires: RequiresSection,

    /// Readme file path.
    #[serde(default)]
    pub readme: Option<String>,

    /// Changelog file path.
    #[serde(default)]
    pub changelog: Option<String>,

    /// Whether this plugin is deprecated.
    #[serde(default)]
    pub deprecated: bool,

    /// Replacement plugin if deprecated.
    #[serde(default)]
    pub replaced_by: Option<String>,
}

fn default_license() -> String {
    "MIT".to_string()
}

/// Compatibility requirements section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequiresSection {
    /// Required OxideKit core version range.
    #[serde(default, with = "version_req_serde_opt")]
    pub core: Option<VersionReq>,

    /// Required Rust version.
    #[serde(default)]
    pub rust: Option<String>,

    /// Required features from core.
    #[serde(default)]
    pub features: Vec<String>,
}

/// Dependency section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencySection {
    /// Required plugin dependencies.
    #[serde(default)]
    pub plugins: Vec<PluginDependency>,

    /// Optional plugin dependencies.
    #[serde(default)]
    pub optional: Vec<PluginDependency>,

    /// Peer dependencies (must be provided by app).
    #[serde(default)]
    pub peer: Vec<PluginDependency>,
}

/// A plugin dependency specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Plugin ID.
    pub id: String,

    /// Version requirement.
    #[serde(with = "version_req_serde")]
    pub version: VersionReq,

    /// Optional features to enable.
    #[serde(default)]
    pub features: Vec<String>,
}

/// Build configuration for plugins that require compilation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Target for compilation (native, wasm).
    #[serde(default = "default_build_target")]
    pub target: String,

    /// Whether the plugin has a build script.
    #[serde(default)]
    pub has_build_script: bool,

    /// Build script path (if custom).
    #[serde(default)]
    pub build_script: Option<String>,

    /// Pre-built binary name (if distributing pre-built).
    #[serde(default)]
    pub binary: Option<String>,

    /// WASM output file (for WASM targets).
    #[serde(default)]
    pub wasm_output: Option<String>,
}

fn default_build_target() -> String {
    "native".to_string()
}

impl PluginManifest {
    /// Get the plugin ID.
    pub fn id(&self) -> &PluginId {
        &self.plugin.id
    }

    /// Get the plugin version.
    pub fn version(&self) -> &Version {
        &self.plugin.version
    }

    /// Get the plugin category.
    pub fn category(&self) -> PluginCategory {
        self.plugin.kind
    }

    /// Get the kind-specific configuration.
    pub fn kind_config(&self) -> Option<KindConfigRef<'_>> {
        match self.plugin.kind {
            PluginCategory::Ui => self.ui.as_ref().map(KindConfigRef::Ui),
            PluginCategory::Native => self.native.as_ref().map(KindConfigRef::Native),
            PluginCategory::Service => self.service.as_ref().map(KindConfigRef::Service),
            PluginCategory::Tooling => self.tooling.as_ref().map(KindConfigRef::Tooling),
            PluginCategory::Theme => self.theme.as_ref().map(KindConfigRef::Theme),
            PluginCategory::Design => self.design.as_ref().map(KindConfigRef::Design),
        }
    }

    /// Check if the manifest has the required kind-specific configuration.
    pub fn has_valid_kind_config(&self) -> bool {
        match self.plugin.kind {
            PluginCategory::Ui => self.ui.is_some(),
            PluginCategory::Native => self.native.is_some(),
            PluginCategory::Service => self.service.is_some(),
            PluginCategory::Tooling => self.tooling.is_some(),
            PluginCategory::Theme => self.theme.is_some(),
            PluginCategory::Design => self.design.is_some(),
        }
    }

    /// Get all required permissions/capabilities.
    pub fn required_capabilities(&self) -> Vec<String> {
        match &self.native {
            Some(native) => native.capabilities.clone(),
            None => Vec::new(),
        }
    }

    /// Check compatibility with a core version.
    pub fn is_compatible_with(&self, core_version: &Version) -> bool {
        match &self.plugin.requires.core {
            Some(req) => req.matches(core_version),
            None => true, // No requirement means compatible with all
        }
    }
}

/// Reference to kind-specific configuration.
pub enum KindConfigRef<'a> {
    /// UI configuration.
    Ui(&'a UiConfig),
    /// Native configuration.
    Native(&'a NativeConfig),
    /// Service configuration.
    Service(&'a ServiceConfig),
    /// Tooling configuration.
    Tooling(&'a ToolingConfig),
    /// Theme configuration.
    Theme(&'a ThemeConfig),
    /// Design configuration.
    Design(&'a DesignConfig),
}

// Custom serde implementations for semver types
mod version_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use semver::Version;

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        version.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

mod version_req_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use semver::VersionReq;

    pub fn serialize<S>(req: &VersionReq, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        req.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<VersionReq, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        VersionReq::parse(&s).map_err(serde::de::Error::custom)
    }
}

mod version_req_serde_opt {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use semver::VersionReq;

    pub fn serialize<S>(req: &Option<VersionReq>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match req {
            Some(r) => r.to_string().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<VersionReq>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => VersionReq::parse(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest_toml() -> &'static str {
        r#"
[plugin]
id = "ui.tables"
kind = "ui"
version = "1.0.0"
publisher = "oxidekit"
description = "Data tables with sorting and filtering"
license = "MIT"
keywords = ["table", "data", "grid"]

[plugin.requires]
core = ">=0.1.0"

[ui]
is_pack = false

[[ui.components]]
name = "DataTable"
description = "A powerful data table component"

[[ui.components.props]]
name = "columns"
type = "Vec<Column>"
required = true
"#
    }

    #[test]
    fn test_parse_manifest() {
        let manifest: PluginManifest = toml::from_str(sample_manifest_toml()).unwrap();

        assert_eq!(manifest.plugin.id.full_name(), "ui.tables");
        assert_eq!(manifest.plugin.kind, PluginCategory::Ui);
        assert_eq!(manifest.plugin.version.to_string(), "1.0.0");
        assert_eq!(manifest.plugin.publisher, "oxidekit");

        // Check UI config
        let ui_config = manifest.ui.as_ref().unwrap();
        assert!(!ui_config.is_pack);
        assert_eq!(ui_config.components.len(), 1);
        assert_eq!(ui_config.components[0].name, "DataTable");
    }

    #[test]
    fn test_compatibility_check() {
        let manifest: PluginManifest = toml::from_str(sample_manifest_toml()).unwrap();

        assert!(manifest.is_compatible_with(&Version::parse("0.1.0").unwrap()));
        assert!(manifest.is_compatible_with(&Version::parse("0.2.0").unwrap()));
        assert!(!manifest.is_compatible_with(&Version::parse("0.0.9").unwrap()));
    }
}
