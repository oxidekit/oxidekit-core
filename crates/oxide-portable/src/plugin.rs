//! Plugin portability declarations and manifest support.
//!
//! Allows plugins to declare their portability requirements and enables
//! the system to verify compatibility before installation.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::error::{PortabilityError, PortabilityResult};
use crate::level::PortabilityLevel;
use crate::target::{Platform, Target, TargetFamily};

/// Portability information for a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPortability {
    /// Overall portability level
    pub level: PortabilityLevel,
    /// Supported platforms (empty = all supported for the level)
    #[serde(default)]
    pub platforms: HashSet<Platform>,
    /// Unsupported platforms (explicit exclusions)
    #[serde(default)]
    pub unsupported_platforms: HashSet<Platform>,
    /// Required capabilities from the target
    #[serde(default)]
    pub required_capabilities: HashSet<String>,
    /// Optional capabilities that enhance functionality
    #[serde(default)]
    pub optional_capabilities: HashSet<String>,
    /// Platform-specific notes
    #[serde(default)]
    pub platform_notes: HashMap<Platform, String>,
    /// Whether the plugin uses native code
    #[serde(default)]
    pub uses_native_code: bool,
    /// Whether the plugin requires specific Rust features
    #[serde(default)]
    pub required_features: HashSet<String>,
}

impl Default for PluginPortability {
    fn default() -> Self {
        Self {
            level: PortabilityLevel::DesktopOnly,
            platforms: HashSet::new(),
            unsupported_platforms: HashSet::new(),
            required_capabilities: HashSet::new(),
            optional_capabilities: HashSet::new(),
            platform_notes: HashMap::new(),
            uses_native_code: false,
            required_features: HashSet::new(),
        }
    }
}

impl PluginPortability {
    /// Create a new portable plugin declaration.
    pub fn portable() -> Self {
        Self {
            level: PortabilityLevel::Portable,
            ..Default::default()
        }
    }

    /// Create a desktop-only plugin declaration.
    pub fn desktop_only() -> Self {
        Self {
            level: PortabilityLevel::DesktopOnly,
            ..Default::default()
        }
    }

    /// Create a web-only plugin declaration.
    pub fn web_only() -> Self {
        Self {
            level: PortabilityLevel::WebOnly,
            platforms: [Platform::Web].into_iter().collect(),
            ..Default::default()
        }
    }

    /// Create a mobile-only plugin declaration.
    pub fn mobile_only() -> Self {
        Self {
            level: PortabilityLevel::MobileOnly,
            platforms: [Platform::IOS, Platform::Android].into_iter().collect(),
            ..Default::default()
        }
    }

    /// Add a supported platform.
    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platforms.insert(platform);
        self
    }

    /// Exclude a platform.
    pub fn without_platform(mut self, platform: Platform) -> Self {
        self.unsupported_platforms.insert(platform);
        self
    }

    /// Require a capability.
    pub fn requires(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.insert(capability.into());
        self
    }

    /// Add an optional capability.
    pub fn optionally(mut self, capability: impl Into<String>) -> Self {
        self.optional_capabilities.insert(capability.into());
        self
    }

    /// Mark as using native code.
    pub fn with_native_code(mut self) -> Self {
        self.uses_native_code = true;
        self
    }

    /// Add a platform note.
    pub fn with_note(mut self, platform: Platform, note: impl Into<String>) -> Self {
        self.platform_notes.insert(platform, note.into());
        self
    }

    /// Check if the plugin supports a specific target.
    pub fn supports(&self, target: &Target) -> bool {
        // Check if explicitly unsupported
        if self.unsupported_platforms.contains(&target.platform()) {
            return false;
        }

        // Check if level allows this target family
        let family_ok = match self.level {
            PortabilityLevel::Portable => true,
            PortabilityLevel::DesktopOnly => target.is_desktop(),
            PortabilityLevel::WebOnly => target.is_web(),
            PortabilityLevel::MobileOnly => target.is_mobile(),
            PortabilityLevel::IosOnly => target.platform() == Platform::IOS,
            PortabilityLevel::AndroidOnly => target.platform() == Platform::Android,
            PortabilityLevel::MacosOnly => target.platform() == Platform::MacOS,
            PortabilityLevel::WindowsOnly => target.platform() == Platform::Windows,
            PortabilityLevel::LinuxOnly => target.platform() == Platform::Linux,
            PortabilityLevel::NativeOnly => target.is_desktop() || target.is_mobile(),
            PortabilityLevel::Experimental => true,
        };

        if !family_ok {
            return false;
        }

        // Check platform list if specified
        if !self.platforms.is_empty() && !self.platforms.contains(&target.platform()) {
            return false;
        }

        // Check required capabilities
        for cap in &self.required_capabilities {
            if !target.has_capability(cap) {
                return false;
            }
        }

        true
    }

    /// Get missing capabilities for a target.
    pub fn missing_capabilities(&self, target: &Target) -> Vec<String> {
        self.required_capabilities
            .iter()
            .filter(|cap| !target.has_capability(cap))
            .cloned()
            .collect()
    }

    /// Get a reason why the plugin doesn't support a target.
    pub fn unsupported_reason(&self, target: &Target) -> Option<String> {
        if self.unsupported_platforms.contains(&target.platform()) {
            return Some(format!(
                "{} is explicitly unsupported",
                target.platform()
            ));
        }

        let family_reason = match self.level {
            PortabilityLevel::DesktopOnly if !target.is_desktop() => {
                Some("This plugin is desktop-only")
            }
            PortabilityLevel::WebOnly if !target.is_web() => {
                Some("This plugin is web-only")
            }
            PortabilityLevel::MobileOnly if !target.is_mobile() => {
                Some("This plugin is mobile-only")
            }
            PortabilityLevel::NativeOnly if target.is_web() => {
                Some("This plugin requires native code (not available on web)")
            }
            _ => None,
        };

        if let Some(reason) = family_reason {
            return Some(reason.to_string());
        }

        let missing = self.missing_capabilities(target);
        if !missing.is_empty() {
            return Some(format!(
                "Missing required capabilities: {}",
                missing.join(", ")
            ));
        }

        None
    }
}

/// Full portability manifest for a plugin.
///
/// This is stored in the plugin's manifest file (e.g., `oxide-plugin.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortabilityManifest {
    /// Plugin identifier
    pub id: String,
    /// Plugin version
    pub version: String,
    /// Core portability declaration
    pub portability: PluginPortability,
    /// Per-API portability overrides
    #[serde(default)]
    pub apis: HashMap<String, PluginPortability>,
    /// Target-specific implementations
    #[serde(default)]
    pub target_implementations: Vec<TargetImplementation>,
    /// Dependencies and their portability requirements
    #[serde(default)]
    pub dependencies: Vec<DependencyPortability>,
}

impl PortabilityManifest {
    /// Create a new manifest.
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            portability: PluginPortability::default(),
            apis: HashMap::new(),
            target_implementations: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Set the portability.
    pub fn with_portability(mut self, portability: PluginPortability) -> Self {
        self.portability = portability;
        self
    }

    /// Add an API portability override.
    pub fn with_api(mut self, api: impl Into<String>, portability: PluginPortability) -> Self {
        self.apis.insert(api.into(), portability);
        self
    }

    /// Add a target implementation.
    pub fn with_implementation(mut self, impl_: TargetImplementation) -> Self {
        self.target_implementations.push(impl_);
        self
    }

    /// Add a dependency.
    pub fn with_dependency(mut self, dep: DependencyPortability) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Load from a TOML file.
    pub fn load(path: impl AsRef<Path>) -> PortabilityResult<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(PortabilityError::ManifestNotFound(path.to_path_buf()));
        }

        let content = std::fs::read_to_string(path)?;
        let manifest: Self = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Save to a TOML file.
    pub fn save(&self, path: impl AsRef<Path>) -> PortabilityResult<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Check if the plugin supports a target.
    pub fn supports(&self, target: &Target) -> bool {
        self.portability.supports(target)
    }

    /// Check if a specific API supports a target.
    pub fn api_supports(&self, api: &str, target: &Target) -> bool {
        if let Some(api_portability) = self.apis.get(api) {
            api_portability.supports(target)
        } else {
            self.portability.supports(target)
        }
    }

    /// Get all supported targets.
    pub fn supported_targets(&self) -> Vec<Target> {
        crate::target::targets::all()
            .into_iter()
            .filter(|t| self.supports(t))
            .collect()
    }

    /// Validate the manifest for consistency.
    pub fn validate(&self) -> PortabilityResult<()> {
        // Check that API portability doesn't exceed plugin portability
        for (api_name, api_port) in &self.apis {
            if api_port.level == PortabilityLevel::Portable
                && self.portability.level != PortabilityLevel::Portable
            {
                return Err(PortabilityError::InvalidManifest(format!(
                    "API '{}' is marked portable but plugin is {}",
                    api_name, self.portability.level
                )));
            }
        }

        // Check dependencies don't have stricter portability
        for dep in &self.dependencies {
            if dep.required && dep.min_level_is_stricter(&self.portability.level) {
                return Err(PortabilityError::InvalidManifest(format!(
                    "Required dependency '{}' has stricter portability ({}) than plugin ({})",
                    dep.name, dep.min_level, self.portability.level
                )));
            }
        }

        Ok(())
    }
}

/// Target-specific implementation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetImplementation {
    /// Target family or specific platform
    pub target: TargetSpec,
    /// Module path for this implementation
    pub module: String,
    /// Feature flag to enable this implementation
    pub feature: Option<String>,
    /// Notes about this implementation
    pub notes: Option<String>,
}

/// Specification for a target (family or specific platform).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TargetSpec {
    /// A target family
    Family(TargetFamily),
    /// A specific platform
    Platform(Platform),
    /// A target triple
    Triple(String),
}

impl TargetSpec {
    /// Check if this spec matches a target.
    pub fn matches(&self, target: &Target) -> bool {
        match self {
            TargetSpec::Family(family) => target.family() == *family,
            TargetSpec::Platform(platform) => target.platform() == *platform,
            TargetSpec::Triple(triple) => target.triple() == triple,
        }
    }
}

/// Dependency portability information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyPortability {
    /// Dependency name
    pub name: String,
    /// Minimum portability level required
    pub min_level: PortabilityLevel,
    /// Whether this dependency is required
    #[serde(default = "default_true")]
    pub required: bool,
    /// Platforms where this dependency is used
    #[serde(default)]
    pub platforms: HashSet<Platform>,
}

fn default_true() -> bool {
    true
}

impl DependencyPortability {
    /// Create a new required dependency.
    pub fn required(name: impl Into<String>, level: PortabilityLevel) -> Self {
        Self {
            name: name.into(),
            min_level: level,
            required: true,
            platforms: HashSet::new(),
        }
    }

    /// Create an optional dependency.
    pub fn optional(name: impl Into<String>, level: PortabilityLevel) -> Self {
        Self {
            name: name.into(),
            min_level: level,
            required: false,
            platforms: HashSet::new(),
        }
    }

    /// Restrict to specific platforms.
    pub fn on_platforms(mut self, platforms: impl IntoIterator<Item = Platform>) -> Self {
        self.platforms = platforms.into_iter().collect();
        self
    }

    /// Check if the minimum level is stricter than the given level.
    fn min_level_is_stricter(&self, level: &PortabilityLevel) -> bool {
        // Portable is least strict, specific platforms are most strict
        let strictness = |l: &PortabilityLevel| -> u8 {
            match l {
                PortabilityLevel::Portable => 0,
                PortabilityLevel::NativeOnly => 1,
                PortabilityLevel::DesktopOnly => 2,
                PortabilityLevel::MobileOnly => 2,
                PortabilityLevel::WebOnly => 2,
                _ => 3,
            }
        };

        strictness(&self.min_level) > strictness(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::targets;

    #[test]
    fn test_plugin_portability() {
        let portable = PluginPortability::portable();
        assert!(portable.supports(&targets::macos_arm64()));
        assert!(portable.supports(&targets::web_wasm32()));
        assert!(portable.supports(&targets::ios_arm64()));

        let desktop = PluginPortability::desktop_only();
        assert!(desktop.supports(&targets::macos_arm64()));
        assert!(!desktop.supports(&targets::web_wasm32()));
        assert!(!desktop.supports(&targets::ios_arm64()));
    }

    #[test]
    fn test_plugin_portability_with_capabilities() {
        let plugin = PluginPortability::portable()
            .requires("filesystem");

        // Desktop has filesystem
        assert!(plugin.supports(&targets::macos_arm64()));
        // Web doesn't have filesystem
        assert!(!plugin.supports(&targets::web_wasm32()));
    }

    #[test]
    fn test_unsupported_reason() {
        let plugin = PluginPortability::desktop_only();
        let web = targets::web_wasm32();

        let reason = plugin.unsupported_reason(&web);
        assert!(reason.is_some());
        assert!(reason.unwrap().contains("desktop-only"));
    }

    #[test]
    fn test_manifest_creation() {
        let manifest = PortabilityManifest::new("ui.tables", "1.0.0")
            .with_portability(PluginPortability::portable())
            .with_api("export_csv", PluginPortability::desktop_only().requires("filesystem"));

        assert!(manifest.supports(&targets::macos_arm64()));
        assert!(manifest.supports(&targets::web_wasm32()));

        assert!(manifest.api_supports("export_csv", &targets::macos_arm64()));
        assert!(!manifest.api_supports("export_csv", &targets::web_wasm32()));
    }

    #[test]
    fn test_manifest_validation() {
        // Valid manifest
        let valid = PortabilityManifest::new("test", "1.0.0")
            .with_portability(PluginPortability::portable());
        assert!(valid.validate().is_ok());

        // Invalid: API claims portable but plugin is desktop-only
        let invalid = PortabilityManifest::new("test", "1.0.0")
            .with_portability(PluginPortability::desktop_only())
            .with_api("my_api", PluginPortability::portable());
        assert!(invalid.validate().is_err());
    }
}
