//! OxideKit Manifest Schema
//!
//! Defines the comprehensive `oxide.toml` manifest format for OxideKit projects.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// The complete OxideKit manifest (oxide.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideManifest {
    /// Application metadata
    pub app: AppSection,

    /// Author information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<AuthorSection>,

    /// Maintainer information (may differ from author)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<MaintainerSection>,

    /// Repository information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySection>,

    /// License information
    #[serde(default)]
    pub license: LicenseSection,

    /// Core OxideKit requirements
    #[serde(default)]
    pub core: CoreSection,

    /// Window configuration
    #[serde(default)]
    pub window: WindowSection,

    /// Permissions allowlist
    #[serde(default)]
    pub permissions: PermissionsSection,

    /// Plugin/extension policy
    #[serde(default)]
    pub policy: PolicySection,

    /// Extensions/plugins configuration
    #[serde(default)]
    pub extensions: ExtensionsSection,

    /// Build configuration
    #[serde(default)]
    pub build: BuildSection,

    /// Development configuration
    #[serde(default)]
    pub dev: DevSection,

    /// Localization configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub i18n: Option<I18nSection>,
}

impl OxideManifest {
    /// Create a new manifest with minimal required fields
    pub fn new(name: &str, id: &str) -> Self {
        Self {
            app: AppSection {
                name: name.to_string(),
                id: id.to_string(),
                version: "0.1.0".to_string(),
                description: Some(format!("An OxideKit application")),
            },
            author: None,
            maintainer: None,
            repository: None,
            license: LicenseSection::default(),
            core: CoreSection::default(),
            window: WindowSection::default(),
            permissions: PermissionsSection::default(),
            policy: PolicySection::default(),
            extensions: ExtensionsSection::default(),
            build: BuildSection::default(),
            dev: DevSection::default(),
            i18n: None,
        }
    }

    /// Load manifest from file
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Save manifest to file
    pub fn to_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let content = self.to_toml()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Generate a well-formatted TOML string with comments
    pub fn to_toml_with_comments(&self) -> String {
        let mut output = String::new();

        // App section
        output.push_str("# =============================================================================\n");
        output.push_str("# OxideKit Application Manifest\n");
        output.push_str("# =============================================================================\n\n");

        output.push_str("[app]\n");
        output.push_str(&format!("name = \"{}\"\n", self.app.name));
        output.push_str(&format!("id = \"{}\"  # Reverse-DNS identifier (stable, used for publishing)\n", self.app.id));
        output.push_str(&format!("version = \"{}\"\n", self.app.version));
        if let Some(ref desc) = self.app.description {
            output.push_str(&format!("description = \"{}\"\n", desc));
        }
        output.push('\n');

        // Author section
        if let Some(ref author) = self.author {
            output.push_str("# -----------------------------------------------------------------------------\n");
            output.push_str("# Author Information\n");
            output.push_str("# -----------------------------------------------------------------------------\n\n");
            output.push_str("[author]\n");
            output.push_str(&format!("name = \"{}\"\n", author.name));
            if let Some(ref email) = author.email {
                output.push_str(&format!("email = \"{}\"\n", email));
            }
            if let Some(ref url) = author.url {
                output.push_str(&format!("url = \"{}\"\n", url));
            }
            output.push('\n');
        }

        // Maintainer section
        if let Some(ref maintainer) = self.maintainer {
            output.push_str("[maintainer]\n");
            output.push_str(&format!("name = \"{}\"\n", maintainer.name));
            if let Some(ref email) = maintainer.email {
                output.push_str(&format!("email = \"{}\"\n", email));
            }
            output.push('\n');
        }

        // Repository section
        if let Some(ref repo) = self.repository {
            output.push_str("# -----------------------------------------------------------------------------\n");
            output.push_str("# Repository Information\n");
            output.push_str("# -----------------------------------------------------------------------------\n\n");
            output.push_str("[repository]\n");
            output.push_str(&format!("url = \"{}\"\n", repo.url));
            if let Some(ref issues) = repo.issues {
                output.push_str(&format!("issues = \"{}\"\n", issues));
            }
            if let Some(ref homepage) = repo.homepage {
                output.push_str(&format!("homepage = \"{}\"\n", homepage));
            }
            output.push('\n');
        }

        // License section
        output.push_str("# -----------------------------------------------------------------------------\n");
        output.push_str("# License\n");
        output.push_str("# -----------------------------------------------------------------------------\n\n");
        output.push_str("[license]\n");
        output.push_str(&format!("spdx = \"{}\"  # SPDX license identifier\n", self.license.spdx));
        output.push_str(&format!("file = \"{}\"\n", self.license.file));
        output.push('\n');

        // Core section
        output.push_str("# -----------------------------------------------------------------------------\n");
        output.push_str("# OxideKit Core Requirements\n");
        output.push_str("# -----------------------------------------------------------------------------\n\n");
        output.push_str("[core]\n");
        output.push_str(&format!("requires = \"{}\"  # Minimum OxideKit version\n", self.core.requires));
        output.push('\n');

        // Window section
        output.push_str("# -----------------------------------------------------------------------------\n");
        output.push_str("# Window Configuration\n");
        output.push_str("# -----------------------------------------------------------------------------\n\n");
        output.push_str("[window]\n");
        output.push_str(&format!("title = \"{}\"\n", self.window.title));
        output.push_str(&format!("width = {}\n", self.window.width));
        output.push_str(&format!("height = {}\n", self.window.height));
        if let Some(min_w) = self.window.min_width {
            output.push_str(&format!("min_width = {}\n", min_w));
        }
        if let Some(min_h) = self.window.min_height {
            output.push_str(&format!("min_height = {}\n", min_h));
        }
        output.push_str(&format!("resizable = {}\n", self.window.resizable));
        output.push_str(&format!("decorations = {}\n", self.window.decorations));
        output.push('\n');

        // Permissions section
        output.push_str("# -----------------------------------------------------------------------------\n");
        output.push_str("# Permissions (explicit allowlist - no implicit permissions)\n");
        output.push_str("# -----------------------------------------------------------------------------\n\n");
        output.push_str("[permissions]\n");
        output.push_str("# Format: \"extension.capability\" = [\"specific.permission\", ...]\n");
        if !self.permissions.capabilities.is_empty() {
            for (ext, perms) in &self.permissions.capabilities {
                let perms_str: Vec<_> = perms.iter().map(|p| format!("\"{}\"", p)).collect();
                output.push_str(&format!("\"{}\" = [{}]\n", ext, perms_str.join(", ")));
            }
        } else {
            output.push_str("# Example:\n");
            output.push_str("# \"native.filesystem\" = [\"filesystem.read\", \"filesystem.write\"]\n");
            output.push_str("# \"native.network\" = [\"network.http\"]\n");
        }
        output.push('\n');

        // Policy section
        output.push_str("# -----------------------------------------------------------------------------\n");
        output.push_str("# Plugin Policy\n");
        output.push_str("# -----------------------------------------------------------------------------\n\n");
        output.push_str("[policy]\n");
        output.push_str(&format!("allow_unverified_native = {}  # Allow unverified native plugins\n",
            self.policy.allow_unverified_native));
        output.push_str(&format!("default_plugin_lane = \"{}\"  # Default execution lane: wasm, native, or auto\n",
            self.policy.default_plugin_lane));
        output.push_str(&format!("trust_level = \"{}\"  # official, verified, or community\n",
            self.policy.trust_level));
        output.push('\n');

        // Extensions section
        output.push_str("# -----------------------------------------------------------------------------\n");
        output.push_str("# Extensions/Plugins\n");
        output.push_str("# -----------------------------------------------------------------------------\n\n");
        output.push_str("[extensions]\n");
        output.push_str(&format!("allow = [{}]\n",
            self.extensions.allow.iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ")));
        if !self.extensions.deny.is_empty() {
            output.push_str(&format!("deny = [{}]\n",
                self.extensions.deny.iter()
                    .map(|e| format!("\"{}\"", e))
                    .collect::<Vec<_>>()
                    .join(", ")));
        }
        output.push('\n');

        // Build section
        output.push_str("# -----------------------------------------------------------------------------\n");
        output.push_str("# Build Configuration\n");
        output.push_str("# -----------------------------------------------------------------------------\n\n");
        output.push_str("[build]\n");
        output.push_str(&format!("entry = \"{}\"\n", self.build.entry));
        output.push_str(&format!("assets = \"{}\"\n", self.build.assets));
        if let Some(ref out) = self.build.output {
            output.push_str(&format!("output = \"{}\"\n", out));
        }
        output.push('\n');

        // Dev section
        output.push_str("# -----------------------------------------------------------------------------\n");
        output.push_str("# Development Configuration\n");
        output.push_str("# -----------------------------------------------------------------------------\n\n");
        output.push_str("[dev]\n");
        output.push_str(&format!("hot_reload = {}\n", self.dev.hot_reload));
        output.push_str(&format!("inspector = {}\n", self.dev.inspector));
        output.push_str(&format!("port = {}\n", self.dev.port));

        // I18n section (if present)
        if let Some(ref i18n) = self.i18n {
            output.push('\n');
            output.push_str("# -----------------------------------------------------------------------------\n");
            output.push_str("# Internationalization\n");
            output.push_str("# -----------------------------------------------------------------------------\n\n");
            output.push_str("[i18n]\n");
            output.push_str(&format!("default_locale = \"{}\"\n", i18n.default_locale));
            output.push_str(&format!("locales = [{}]\n",
                i18n.locales.iter()
                    .map(|l| format!("\"{}\"", l))
                    .collect::<Vec<_>>()
                    .join(", ")));
        }

        output
    }
}

/// Application metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSection {
    /// Human-readable application name
    pub name: String,

    /// Reverse-DNS application identifier (stable, used for publishing)
    pub id: String,

    /// Semantic version
    pub version: String,

    /// Application description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorSection {
    /// Author name
    pub name: String,

    /// Contact email
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Author website/URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Maintainer information (may differ from original author)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainerSection {
    /// Maintainer name
    pub name: String,

    /// Contact email
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySection {
    /// Repository URL
    pub url: String,

    /// Issues URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issues: Option<String>,

    /// Homepage URL (if different from repository)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSection {
    /// SPDX license identifier
    pub spdx: String,

    /// Path to license file
    pub file: String,
}

impl Default for LicenseSection {
    fn default() -> Self {
        Self {
            spdx: "MIT".to_string(),
            file: "LICENSE".to_string(),
        }
    }
}

/// Core OxideKit requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSection {
    /// Minimum OxideKit version requirement (semver)
    pub requires: String,
}

impl Default for CoreSection {
    fn default() -> Self {
        Self {
            requires: ">=0.1.0".to_string(),
        }
    }
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSection {
    /// Window title
    pub title: String,

    /// Initial width
    pub width: u32,

    /// Initial height
    pub height: u32,

    /// Minimum width
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_width: Option<u32>,

    /// Minimum height
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_height: Option<u32>,

    /// Whether window is resizable
    pub resizable: bool,

    /// Whether window has decorations
    pub decorations: bool,
}

impl Default for WindowSection {
    fn default() -> Self {
        Self {
            title: "OxideKit App".to_string(),
            width: 1280,
            height: 720,
            min_width: None,
            min_height: None,
            resizable: true,
            decorations: true,
        }
    }
}

/// Permissions allowlist
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionsSection {
    /// Capability permissions by extension
    /// Format: "extension.capability" -> ["specific.permission", ...]
    #[serde(flatten)]
    pub capabilities: HashMap<String, Vec<String>>,
}

impl PermissionsSection {
    /// Add a permission
    pub fn add(&mut self, extension: &str, permission: &str) {
        self.capabilities
            .entry(extension.to_string())
            .or_default()
            .push(permission.to_string());
    }

    /// Check if a permission is granted
    pub fn has(&self, extension: &str, permission: &str) -> bool {
        self.capabilities
            .get(extension)
            .map(|perms| perms.contains(&permission.to_string()))
            .unwrap_or(false)
    }
}

/// Plugin policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySection {
    /// Allow unverified native plugins
    pub allow_unverified_native: bool,

    /// Default plugin execution lane: "wasm", "native", or "auto"
    pub default_plugin_lane: String,

    /// Trust level: "official", "verified", or "community"
    pub trust_level: String,
}

impl Default for PolicySection {
    fn default() -> Self {
        Self {
            allow_unverified_native: false,
            default_plugin_lane: "wasm".to_string(),
            trust_level: "official".to_string(),
        }
    }
}

/// Extensions/plugins configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionsSection {
    /// Allowed extensions
    #[serde(default)]
    pub allow: Vec<String>,

    /// Denied extensions
    #[serde(default)]
    pub deny: Vec<String>,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSection {
    /// Entry point file
    pub entry: String,

    /// Assets directory
    pub assets: String,

    /// Output directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

impl Default for BuildSection {
    fn default() -> Self {
        Self {
            entry: "ui/app.oui".to_string(),
            assets: "assets".to_string(),
            output: None,
        }
    }
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevSection {
    /// Enable hot reload
    pub hot_reload: bool,

    /// Enable inspector
    pub inspector: bool,

    /// Dev server port
    pub port: u16,
}

impl Default for DevSection {
    fn default() -> Self {
        Self {
            hot_reload: true,
            inspector: true,
            port: 3000,
        }
    }
}

/// Internationalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nSection {
    /// Default locale
    pub default_locale: String,

    /// Supported locales
    pub locales: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_roundtrip() {
        let mut manifest = OxideManifest::new("test-app", "com.example.test");
        manifest.author = Some(AuthorSection {
            name: "Test Author".to_string(),
            email: Some("test@example.com".to_string()),
            url: None,
        });

        let toml_str = manifest.to_toml().unwrap();
        let parsed: OxideManifest = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.app.name, "test-app");
        assert_eq!(parsed.author.unwrap().name, "Test Author");
    }

    #[test]
    fn test_permissions() {
        let mut perms = PermissionsSection::default();
        perms.add("native.filesystem", "filesystem.read");
        perms.add("native.filesystem", "filesystem.write");

        assert!(perms.has("native.filesystem", "filesystem.read"));
        assert!(perms.has("native.filesystem", "filesystem.write"));
        assert!(!perms.has("native.network", "network.http"));
    }
}
