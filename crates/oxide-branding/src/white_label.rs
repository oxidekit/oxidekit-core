//! White-Label Configuration System
//!
//! Provides comprehensive white-labeling capabilities for OxideKit applications,
//! allowing complete rebranding while maintaining brand governance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::brand_pack::BrandPack;
use crate::app_pack::AppPack;
use crate::error::{BrandingError, BrandingResult};

/// White-label configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabelConfig {
    /// Configuration name/identifier
    pub name: String,

    /// White-label mode
    pub mode: WhiteLabelMode,

    /// Base brand pack reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_brand: Option<String>,

    /// Identity overrides
    pub identity: WhiteLabelIdentity,

    /// Visual overrides
    #[serde(default)]
    pub overrides: WhiteLabelOverrides,

    /// Feature configuration
    #[serde(default)]
    pub features: WhiteLabelFeatures,

    /// Content replacements
    #[serde(default)]
    pub content: ContentReplacements,

    /// Environment-specific settings
    #[serde(default)]
    pub environments: HashMap<String, EnvironmentConfig>,

    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl WhiteLabelConfig {
    /// Create a new white-label config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mode: WhiteLabelMode::Full,
            base_brand: None,
            identity: WhiteLabelIdentity::default(),
            overrides: WhiteLabelOverrides::default(),
            features: WhiteLabelFeatures::default(),
            content: ContentReplacements::default(),
            environments: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create from a brand pack
    pub fn from_brand(brand: &BrandPack) -> Self {
        let mut config = Self::new(&brand.identity.name);
        config.base_brand = Some(brand.id.clone());
        config.identity = WhiteLabelIdentity {
            app_name: brand.identity.name.clone(),
            display_name: brand.identity.display_name.clone(),
            company_name: None,
            tagline: brand.identity.tagline.clone(),
            description: Some(brand.identity.description.clone()),
            website: brand.identity.website.clone(),
            support_email: None,
            copyright: Some(brand.identity.legal.copyright.clone()),
        };
        config
    }

    /// Load from file
    pub fn from_file(path: impl AsRef<Path>) -> BrandingResult<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Ok(toml::from_str(&content)?)
    }

    /// Save to file
    pub fn save(&self, path: impl AsRef<Path>) -> BrandingResult<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> BrandingResult<()> {
        if self.identity.app_name.is_empty() {
            return Err(BrandingError::WhiteLabelError(
                "App name is required".into()
            ));
        }

        // Check that colors are valid hex
        for (name, color) in &self.overrides.colors {
            if !color.starts_with('#') || (color.len() != 7 && color.len() != 9) {
                return Err(BrandingError::WhiteLabelError(
                    format!("Invalid color '{}' for '{}'", color, name)
                ));
            }
        }

        Ok(())
    }

    /// Get config for a specific environment
    pub fn for_environment(&self, env: &str) -> WhiteLabelConfig {
        let mut config = self.clone();

        if let Some(env_config) = self.environments.get(env) {
            // Merge environment-specific overrides
            config.overrides.colors.extend(env_config.colors.clone());
            config.overrides.tokens.extend(env_config.tokens.clone());

            if let Some(ref identity) = env_config.identity {
                if let Some(ref name) = identity.app_name {
                    config.identity.app_name = name.clone();
                }
                if let Some(ref display) = identity.display_name {
                    config.identity.display_name = display.clone();
                }
            }
        }

        config
    }

    /// Apply white-label config to an app pack
    pub fn apply_to(&self, app: &mut AppPack) -> BrandingResult<()> {
        // Apply identity
        app.app_info.name = self.identity.app_name.clone();
        app.app_info.display_name = self.identity.display_name.clone();
        app.app_info.description = self.identity.description.clone().unwrap_or_default();

        // Apply color overrides
        for (name, color) in &self.overrides.colors {
            app.colors.overrides.insert(
                name.clone(),
                crate::brand_pack::BrandColor::new(color),
            );
        }

        // Apply token overrides
        for (name, value) in &self.overrides.tokens {
            if let Some(f) = value.as_f64() {
                app.tokens.custom.insert(
                    name.clone(),
                    serde_json::json!(f),
                );
            } else {
                app.tokens.custom.insert(name.clone(), value.clone());
            }
        }

        Ok(())
    }
}

impl Default for WhiteLabelConfig {
    fn default() -> Self {
        Self::new("Default")
    }
}

/// White-label mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WhiteLabelMode {
    /// Full white-label (complete rebranding)
    #[default]
    Full,
    /// Partial white-label (some brand elements retained)
    Partial,
    /// Co-branded (both brands visible)
    CoBranded,
    /// Powered-by (original brand in footer/attribution)
    PoweredBy,
    /// Reseller (reseller branding with attribution)
    Reseller,
}

/// White-label identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabelIdentity {
    /// Application name
    pub app_name: String,

    /// Display name
    pub display_name: String,

    /// Company/organization name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,

    /// Tagline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Support email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_email: Option<String>,

    /// Copyright notice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
}

impl Default for WhiteLabelIdentity {
    fn default() -> Self {
        Self {
            app_name: "App".into(),
            display_name: "App".into(),
            company_name: None,
            tagline: None,
            description: None,
            website: None,
            support_email: None,
            copyright: None,
        }
    }
}

/// White-label visual overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhiteLabelOverrides {
    /// Color overrides (name -> hex)
    #[serde(default)]
    pub colors: HashMap<String, String>,

    /// Logo path overrides
    #[serde(default)]
    pub logos: LogoOverrides,

    /// Icon path overrides
    #[serde(default)]
    pub icons: IconOverrides,

    /// Font overrides
    #[serde(default)]
    pub fonts: FontOverrides,

    /// Token overrides (any design token)
    #[serde(default)]
    pub tokens: HashMap<String, serde_json::Value>,
}

/// Logo overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogoOverrides {
    /// Primary logo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<PathBuf>,

    /// Secondary/alternate logo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<PathBuf>,

    /// Icon/mark
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<PathBuf>,

    /// Wordmark
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wordmark: Option<PathBuf>,

    /// Dark theme logo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark: Option<PathBuf>,

    /// Light theme logo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<PathBuf>,
}

/// Icon overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IconOverrides {
    /// App icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_icon: Option<PathBuf>,

    /// Favicon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<PathBuf>,

    /// Touch icon (iOS/Android)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touch_icon: Option<PathBuf>,

    /// Notification icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<PathBuf>,
}

/// Font overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FontOverrides {
    /// Primary font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,

    /// Secondary font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,

    /// Monospace font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mono: Option<String>,

    /// Custom font files
    #[serde(default)]
    pub files: Vec<FontFile>,
}

/// Custom font file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFile {
    /// Font family name
    pub family: String,

    /// Font file path
    pub path: PathBuf,

    /// Font weight
    pub weight: u16,

    /// Font style (normal, italic)
    #[serde(default)]
    pub style: String,
}

/// White-label feature configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabelFeatures {
    /// Show "Powered by" attribution
    #[serde(default)]
    pub show_powered_by: bool,

    /// Custom powered-by text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powered_by_text: Option<String>,

    /// Allow users to see original branding
    #[serde(default)]
    pub allow_brand_toggle: bool,

    /// Enable custom domain support
    #[serde(default = "default_true")]
    pub custom_domains: bool,

    /// Enable custom email domain
    #[serde(default)]
    pub custom_email_domain: bool,

    /// Disable specific features
    #[serde(default)]
    pub disabled_features: Vec<String>,

    /// Custom feature flags
    #[serde(default)]
    pub feature_flags: HashMap<String, bool>,
}

fn default_true() -> bool {
    true
}

impl Default for WhiteLabelFeatures {
    fn default() -> Self {
        Self {
            show_powered_by: true,
            powered_by_text: None,
            allow_brand_toggle: false,
            custom_domains: true,
            custom_email_domain: false,
            disabled_features: Vec::new(),
            feature_flags: HashMap::new(),
        }
    }
}

/// Content replacements for white-labeling
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentReplacements {
    /// String replacements (search -> replace)
    #[serde(default)]
    pub strings: HashMap<String, String>,

    /// URL replacements
    #[serde(default)]
    pub urls: HashMap<String, String>,

    /// Email replacements
    #[serde(default)]
    pub emails: HashMap<String, String>,

    /// Social media links
    #[serde(default)]
    pub social: SocialLinks,

    /// Legal document URLs
    #[serde(default)]
    pub legal: LegalLinks,
}

/// Social media links
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocialLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub facebook: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instagram: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,

    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// Legal document links
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegalLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gdpr: Option<String>,

    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentConfig {
    /// Identity overrides for this environment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<PartialIdentity>,

    /// Color overrides
    #[serde(default)]
    pub colors: HashMap<String, String>,

    /// Token overrides
    #[serde(default)]
    pub tokens: HashMap<String, serde_json::Value>,

    /// Feature flag overrides
    #[serde(default)]
    pub features: HashMap<String, bool>,
}

/// Partial identity for environment overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
}

/// White-label builder for fluent API
#[derive(Debug, Default)]
pub struct WhiteLabelBuilder {
    config: WhiteLabelConfig,
}

impl WhiteLabelBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: WhiteLabelConfig::new(name),
        }
    }

    pub fn mode(mut self, mode: WhiteLabelMode) -> Self {
        self.config.mode = mode;
        self
    }

    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.config.identity.app_name = name.into();
        self
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.config.identity.display_name = name.into();
        self
    }

    pub fn company(mut self, name: impl Into<String>) -> Self {
        self.config.identity.company_name = Some(name.into());
        self
    }

    pub fn tagline(mut self, tagline: impl Into<String>) -> Self {
        self.config.identity.tagline = Some(tagline.into());
        self
    }

    pub fn website(mut self, url: impl Into<String>) -> Self {
        self.config.identity.website = Some(url.into());
        self
    }

    pub fn color(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.overrides.colors.insert(name.into(), value.into());
        self
    }

    pub fn primary_logo(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.overrides.logos.primary = Some(path.into());
        self
    }

    pub fn app_icon(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.overrides.icons.app_icon = Some(path.into());
        self
    }

    pub fn show_powered_by(mut self, show: bool) -> Self {
        self.config.features.show_powered_by = show;
        self
    }

    pub fn powered_by_text(mut self, text: impl Into<String>) -> Self {
        self.config.features.powered_by_text = Some(text.into());
        self
    }

    pub fn build(self) -> BrandingResult<WhiteLabelConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white_label_creation() {
        let config = WhiteLabelConfig::new("Test App");
        assert_eq!(config.name, "Test App");
        assert_eq!(config.mode, WhiteLabelMode::Full);
    }

    #[test]
    fn test_white_label_builder() {
        let config = WhiteLabelBuilder::new("My App")
            .app_name("My Custom App")
            .display_name("My Custom App")
            .company("My Company")
            .color("primary", "#FF5500")
            .show_powered_by(false)
            .build()
            .unwrap();

        assert_eq!(config.identity.app_name, "My Custom App");
        assert_eq!(config.identity.company_name, Some("My Company".into()));
        assert!(!config.features.show_powered_by);
        assert_eq!(
            config.overrides.colors.get("primary"),
            Some(&"#FF5500".into())
        );
    }

    #[test]
    fn test_environment_specific_config() {
        let mut config = WhiteLabelConfig::new("Test");
        config.identity.app_name = "Test App".into();
        config.overrides.colors.insert("primary".into(), "#FF0000".into());

        config.environments.insert("staging".into(), EnvironmentConfig {
            identity: Some(PartialIdentity {
                app_name: Some("Test App (Staging)".into()),
                ..Default::default()
            }),
            colors: {
                let mut c = HashMap::new();
                c.insert("primary".into(), "#00FF00".into());
                c
            },
            ..Default::default()
        });

        let staging = config.for_environment("staging");
        assert_eq!(staging.identity.app_name, "Test App (Staging)");
        assert_eq!(staging.overrides.colors.get("primary"), Some(&"#00FF00".into()));
    }

    #[test]
    fn test_validation() {
        let mut config = WhiteLabelConfig::new("Test");
        config.identity.app_name = String::new();

        assert!(config.validate().is_err());

        config.identity.app_name = "Test App".into();
        config.overrides.colors.insert("bad".into(), "not-a-color".into());

        assert!(config.validate().is_err());
    }
}
