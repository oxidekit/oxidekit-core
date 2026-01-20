//! Starter Specification
//!
//! Defines the metadata format for starters (starter.toml).

use serde::{Deserialize, Serialize};

/// A starter specification (parsed from starter.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterSpec {
    /// Unique starter ID (e.g., "admin-panel", "desktop-wallet")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Short description
    pub description: String,

    /// Detailed description (supports markdown)
    #[serde(default)]
    pub long_description: Option<String>,

    /// Starter version
    pub version: String,

    /// Minimum OxideKit core version required
    #[serde(default)]
    pub min_core_version: Option<String>,

    /// Starter metadata
    #[serde(default)]
    pub metadata: StarterMetadata,

    /// Supported build targets
    #[serde(default)]
    pub targets: Vec<StarterTarget>,

    /// Plugins to install
    #[serde(default)]
    pub plugins: Vec<PluginRequirement>,

    /// Permission presets
    #[serde(default)]
    pub permissions: PermissionPreset,

    /// Files to generate
    #[serde(default)]
    pub files: Vec<GeneratedFile>,

    /// Post-init steps (commands or instructions)
    #[serde(default)]
    pub post_init: Vec<PostInitStep>,

    /// Template variables that can be customized
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
}

impl StarterSpec {
    /// Create a new starter spec with minimal required fields
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            long_description: None,
            version: "0.1.0".to_string(),
            min_core_version: None,
            metadata: StarterMetadata::default(),
            targets: vec![StarterTarget::Desktop],
            plugins: Vec::new(),
            permissions: PermissionPreset::default(),
            files: Vec::new(),
            post_init: Vec::new(),
            variables: Vec::new(),
        }
    }

    /// Add a plugin requirement
    pub fn with_plugin(mut self, id: &str, version: Option<&str>) -> Self {
        self.plugins.push(PluginRequirement {
            id: id.to_string(),
            version: version.map(|v| v.to_string()),
            optional: false,
        });
        self
    }

    /// Add a target
    pub fn with_target(mut self, target: StarterTarget) -> Self {
        if !self.targets.contains(&target) {
            self.targets.push(target);
        }
        self
    }

    /// Add a generated file
    pub fn with_file(mut self, path: &str, template: &str) -> Self {
        self.files.push(GeneratedFile {
            path: path.to_string(),
            template: template.to_string(),
            condition: None,
        });
        self
    }

    /// Parse from TOML string
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

/// Starter metadata for discovery and display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StarterMetadata {
    /// Category/type of starter
    #[serde(default)]
    pub category: StarterCategory,

    /// Tags for discovery
    #[serde(default)]
    pub tags: Vec<String>,

    /// Author/maintainer
    #[serde(default)]
    pub author: Option<String>,

    /// Homepage/documentation URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// Screenshot URLs for preview
    #[serde(default)]
    pub screenshots: Vec<String>,

    /// Whether this is an official OxideKit starter
    #[serde(default)]
    pub official: bool,

    /// Featured/highlighted starter
    #[serde(default)]
    pub featured: bool,
}

/// Starter category
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StarterCategory {
    /// Desktop application
    #[default]
    App,
    /// Admin/dashboard panel
    Admin,
    /// Documentation site
    Docs,
    /// Marketing/landing website
    Website,
    /// Wallet/crypto application
    Wallet,
    /// Monitoring/logging dashboard
    Monitoring,
    /// Internal tooling
    Internal,
    /// Other/custom
    Other,
}

impl StarterCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Admin => "admin",
            Self::Docs => "docs",
            Self::Website => "website",
            Self::Wallet => "wallet",
            Self::Monitoring => "monitoring",
            Self::Internal => "internal",
            Self::Other => "other",
        }
    }
}

/// Build target for the starter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StarterTarget {
    /// Native desktop application
    Desktop,
    /// Web application (WASM)
    Web,
    /// Static site output
    Static,
}

impl StarterTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Web => "web",
            Self::Static => "static",
        }
    }
}

/// Plugin requirement for the starter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequirement {
    /// Plugin ID (e.g., "ui.core", "native.filesystem")
    pub id: String,

    /// Version requirement (semver)
    #[serde(default)]
    pub version: Option<String>,

    /// Whether this plugin is optional
    #[serde(default)]
    pub optional: bool,
}

/// Permission preset for the starter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionPreset {
    /// Filesystem permissions
    #[serde(default)]
    pub filesystem: FilesystemPermissions,

    /// Network permissions
    #[serde(default)]
    pub network: NetworkPermissions,

    /// System permissions
    #[serde(default)]
    pub system: SystemPermissions,
}

/// Filesystem permission level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesystemPermissions {
    /// Read access to specific paths
    #[serde(default)]
    pub read: Vec<String>,

    /// Write access to specific paths
    #[serde(default)]
    pub write: Vec<String>,
}

/// Network permission level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkPermissions {
    /// Allowed hosts/domains
    #[serde(default)]
    pub hosts: Vec<String>,

    /// Allow all network access
    #[serde(default)]
    pub allow_all: bool,
}

/// System permission level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemPermissions {
    /// Access to keychain/secrets
    #[serde(default)]
    pub keychain: bool,

    /// Access to notifications
    #[serde(default)]
    pub notifications: bool,

    /// Access to clipboard
    #[serde(default)]
    pub clipboard: bool,

    /// Access to system tray
    #[serde(default)]
    pub tray: bool,
}

/// A file to be generated by the starter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// Output path relative to project root
    pub path: String,

    /// Template name or inline content
    pub template: String,

    /// Condition for generation (e.g., "target == 'desktop'")
    #[serde(default)]
    pub condition: Option<String>,
}

/// Post-initialization step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PostInitStep {
    /// Run a command
    Command {
        command: String,
        #[serde(default)]
        description: Option<String>,
    },
    /// Display a message to the user
    Message {
        text: String,
        #[serde(default)]
        level: MessageLevel,
    },
    /// Open a URL in browser
    OpenUrl {
        url: String,
    },
}

/// Message level for post-init messages
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageLevel {
    #[default]
    Info,
    Success,
    Warning,
}

/// Template variable that can be customized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name (used in templates as {{name}})
    pub name: String,

    /// Description of the variable
    pub description: String,

    /// Default value
    #[serde(default)]
    pub default: Option<String>,

    /// Whether this variable is required
    #[serde(default)]
    pub required: bool,

    /// Validation pattern (regex)
    #[serde(default)]
    pub pattern: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starter_spec_creation() {
        let spec = StarterSpec::new("test-starter", "Test Starter", "A test starter")
            .with_plugin("ui.core", None)
            .with_target(StarterTarget::Desktop)
            .with_file("oxide.toml", "oxide_toml");

        assert_eq!(spec.id, "test-starter");
        assert_eq!(spec.plugins.len(), 1);
        assert!(spec.targets.contains(&StarterTarget::Desktop));
    }

    #[test]
    fn test_starter_spec_toml_roundtrip() {
        let spec = StarterSpec::new("test", "Test", "Description")
            .with_plugin("ui.core", Some("^1.0"));

        let toml_str = spec.to_toml().unwrap();
        let parsed: StarterSpec = StarterSpec::from_toml(&toml_str).unwrap();

        assert_eq!(parsed.id, "test");
        assert_eq!(parsed.plugins.len(), 1);
    }

    #[test]
    fn test_category_display() {
        assert_eq!(StarterCategory::Admin.as_str(), "admin");
        assert_eq!(StarterCategory::Wallet.as_str(), "wallet");
    }
}
