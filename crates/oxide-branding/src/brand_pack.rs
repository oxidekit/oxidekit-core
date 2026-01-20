//! Brand Pack - Complete Brand Identity Package
//!
//! A BrandPack defines an organization's complete brand identity including:
//! - Visual identity (logos, colors, typography)
//! - Design tokens (locked and customizable)
//! - Asset library
//! - Brand guidelines
//! - Governance rules

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::asset::{BrandAsset, AssetGuidelines};
use crate::governance::GovernanceRules;
use crate::error::{BrandingError, BrandingResult};

/// Complete brand identity package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandPack {
    /// Unique brand pack ID
    #[serde(default = "generate_uuid")]
    pub id: String,

    /// Brand pack version
    pub version: String,

    /// Brand identity information
    pub identity: BrandIdentity,

    /// Brand colors
    pub colors: BrandColors,

    /// Brand typography
    pub typography: BrandTypography,

    /// Brand assets (logos, icons, etc.)
    #[serde(default)]
    pub assets: BrandAssets,

    /// Design token overrides/additions
    #[serde(default)]
    pub tokens: BrandTokens,

    /// Governance rules (what can be customized)
    #[serde(default)]
    pub governance: GovernanceRules,

    /// Brand guidelines
    #[serde(default)]
    pub guidelines: BrandGuidelines,

    /// Metadata
    #[serde(default)]
    pub metadata: BrandMetadata,
}

fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

impl Default for BrandPack {
    fn default() -> Self {
        Self {
            id: generate_uuid(),
            version: "1.0.0".into(),
            identity: BrandIdentity::default(),
            colors: BrandColors::default(),
            typography: BrandTypography::default(),
            assets: BrandAssets::default(),
            tokens: BrandTokens::default(),
            governance: GovernanceRules::default(),
            guidelines: BrandGuidelines::default(),
            metadata: BrandMetadata::default(),
        }
    }
}

impl BrandPack {
    /// Create a new brand pack with the given name
    pub fn new(name: impl Into<String>) -> Self {
        let mut pack = Self::default();
        pack.identity.name = name.into();
        pack
    }

    /// Load brand pack from TOML file
    pub fn from_file(path: impl AsRef<Path>) -> BrandingResult<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let pack: Self = toml::from_str(&content)?;
        pack.validate()?;
        Ok(pack)
    }

    /// Save brand pack to TOML file
    pub fn save(&self, path: impl AsRef<Path>) -> BrandingResult<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the brand pack
    pub fn validate(&self) -> BrandingResult<()> {
        // Check required fields
        if self.identity.name.is_empty() {
            return Err(BrandingError::ValidationError(
                "Brand name is required".into()
            ));
        }

        // Validate primary color
        if self.colors.primary.value.is_empty() {
            return Err(BrandingError::ValidationError(
                "Primary color is required".into()
            ));
        }

        // Validate asset references
        for asset in self.assets.all() {
            if asset.path.as_os_str().is_empty() {
                return Err(BrandingError::ValidationError(
                    format!("Asset '{}' has empty path", asset.id)
                ));
            }
        }

        Ok(())
    }

    /// Export to JSON
    pub fn to_json(&self) -> BrandingResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Load from JSON
    pub fn from_json(content: &str) -> BrandingResult<Self> {
        let pack: Self = serde_json::from_str(content)?;
        pack.validate()?;
        Ok(pack)
    }

    /// Get all brand colors as a map
    pub fn color_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("primary".into(), self.colors.primary.value.clone());
        map.insert("secondary".into(), self.colors.secondary.value.clone());
        map.insert("accent".into(), self.colors.accent.value.clone());

        if let Some(bg) = &self.colors.background {
            map.insert("background".into(), bg.value.clone());
        }
        if let Some(fg) = &self.colors.foreground {
            map.insert("foreground".into(), fg.value.clone());
        }

        for (name, color) in &self.colors.semantic {
            map.insert(name.clone(), color.value.clone());
        }

        for (name, color) in &self.colors.custom {
            map.insert(name.clone(), color.value.clone());
        }

        map
    }
}

/// Brand identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandIdentity {
    /// Brand/organization name
    pub name: String,

    /// Brand display name (may differ from legal name)
    #[serde(default)]
    pub display_name: String,

    /// Short name/abbreviation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,

    /// Brand tagline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,

    /// Brand description
    #[serde(default)]
    pub description: String,

    /// Primary logo asset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_logo: Option<BrandAsset>,

    /// Secondary/alternate logo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_logo: Option<BrandAsset>,

    /// Icon (simplified logo for small sizes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<BrandAsset>,

    /// Wordmark (text-only logo)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wordmark: Option<BrandAsset>,

    /// Website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Legal/copyright information
    #[serde(default)]
    pub legal: LegalInfo,
}

impl Default for BrandIdentity {
    fn default() -> Self {
        Self {
            name: "OxideKit Brand".into(),
            display_name: "OxideKit".into(),
            short_name: None,
            tagline: None,
            description: "Default OxideKit brand pack".into(),
            primary_logo: None,
            secondary_logo: None,
            icon: None,
            wordmark: None,
            website: None,
            legal: LegalInfo::default(),
        }
    }
}

/// Legal information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegalInfo {
    /// Copyright holder
    #[serde(default)]
    pub copyright: String,

    /// Copyright year
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,

    /// Trademark notice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trademark: Option<String>,

    /// License
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Brand colors definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandColors {
    /// Primary brand color
    pub primary: BrandColor,

    /// Secondary brand color
    pub secondary: BrandColor,

    /// Accent color
    pub accent: BrandColor,

    /// Background color (optional, for brand-specific backgrounds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<BrandColor>,

    /// Foreground/text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground: Option<BrandColor>,

    /// Semantic colors (success, warning, error, info)
    #[serde(default)]
    pub semantic: HashMap<String, BrandColor>,

    /// Additional custom colors
    #[serde(default)]
    pub custom: HashMap<String, BrandColor>,
}

impl Default for BrandColors {
    fn default() -> Self {
        Self {
            primary: BrandColor::new("#3B82F6"),
            secondary: BrandColor::new("#6B7280"),
            accent: BrandColor::new("#F59E0B"),
            background: None,
            foreground: None,
            semantic: HashMap::new(),
            custom: HashMap::new(),
        }
    }
}

/// A brand color with variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandColor {
    /// Primary color value (hex)
    pub value: String,

    /// Lighter variant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<String>,

    /// Darker variant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark: Option<String>,

    /// Contrast color (for text on this color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contrast: Option<String>,

    /// Color name (for reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Usage notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,

    /// Whether this color is locked (cannot be overridden)
    #[serde(default)]
    pub locked: bool,
}

impl BrandColor {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            light: None,
            dark: None,
            contrast: None,
            name: None,
            usage: None,
            locked: false,
        }
    }

    pub fn with_variants(
        value: impl Into<String>,
        light: impl Into<String>,
        dark: impl Into<String>,
    ) -> Self {
        Self {
            value: value.into(),
            light: Some(light.into()),
            dark: Some(dark.into()),
            contrast: None,
            name: None,
            usage: None,
            locked: false,
        }
    }

    pub fn locked(mut self) -> Self {
        self.locked = true;
        self
    }
}

/// Brand typography definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandTypography {
    /// Primary font family
    pub primary_family: FontFamilySpec,

    /// Secondary font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_family: Option<FontFamilySpec>,

    /// Monospace font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mono_family: Option<FontFamilySpec>,

    /// Display/heading font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_family: Option<FontFamilySpec>,

    /// Typography scale
    #[serde(default)]
    pub scale: TypographyScale,

    /// Custom font files to include
    #[serde(default)]
    pub fonts: Vec<BrandAsset>,
}

impl Default for BrandTypography {
    fn default() -> Self {
        Self {
            primary_family: FontFamilySpec {
                name: "Inter".into(),
                fallbacks: vec!["system-ui".into(), "sans-serif".into()],
                weights: vec![400, 500, 600, 700],
                locked: false,
            },
            secondary_family: None,
            mono_family: Some(FontFamilySpec {
                name: "JetBrains Mono".into(),
                fallbacks: vec!["Fira Code".into(), "monospace".into()],
                weights: vec![400, 500, 700],
                locked: false,
            }),
            display_family: None,
            scale: TypographyScale::default(),
            fonts: Vec::new(),
        }
    }
}

/// Font family specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamilySpec {
    /// Font family name
    pub name: String,

    /// Fallback fonts
    #[serde(default)]
    pub fallbacks: Vec<String>,

    /// Available weights
    #[serde(default)]
    pub weights: Vec<u16>,

    /// Whether this font is locked
    #[serde(default)]
    pub locked: bool,
}

/// Typography scale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyScale {
    /// Base font size (px)
    pub base_size: f32,

    /// Scale ratio
    pub ratio: f32,

    /// Minimum font size
    pub min_size: f32,

    /// Maximum font size
    pub max_size: f32,
}

impl Default for TypographyScale {
    fn default() -> Self {
        Self {
            base_size: 16.0,
            ratio: 1.25, // Major third
            min_size: 12.0,
            max_size: 72.0,
        }
    }
}

/// Brand assets collection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrandAssets {
    /// Logos
    #[serde(default)]
    pub logos: Vec<BrandAsset>,

    /// Icons
    #[serde(default)]
    pub icons: Vec<BrandAsset>,

    /// Illustrations
    #[serde(default)]
    pub illustrations: Vec<BrandAsset>,

    /// Patterns/backgrounds
    #[serde(default)]
    pub patterns: Vec<BrandAsset>,

    /// Other assets
    #[serde(default)]
    pub other: Vec<BrandAsset>,
}

impl BrandAssets {
    /// Get all assets as a flat list
    pub fn all(&self) -> Vec<&BrandAsset> {
        let mut all = Vec::new();
        all.extend(self.logos.iter());
        all.extend(self.icons.iter());
        all.extend(self.illustrations.iter());
        all.extend(self.patterns.iter());
        all.extend(self.other.iter());
        all
    }

    /// Find asset by ID
    pub fn find(&self, id: &str) -> Option<&BrandAsset> {
        self.all().into_iter().find(|a| a.id == id)
    }
}

/// Brand-specific design tokens
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrandTokens {
    /// Custom spacing tokens
    #[serde(default)]
    pub spacing: HashMap<String, f32>,

    /// Custom radius tokens
    #[serde(default)]
    pub radius: HashMap<String, f32>,

    /// Custom shadow tokens
    #[serde(default)]
    pub shadows: HashMap<String, String>,

    /// Custom animation/motion tokens
    #[serde(default)]
    pub motion: HashMap<String, String>,

    /// Any other custom tokens
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Brand guidelines
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrandGuidelines {
    /// Logo usage guidelines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<AssetGuidelines>,

    /// Color usage guidelines
    #[serde(default)]
    pub color_usage: Vec<String>,

    /// Typography guidelines
    #[serde(default)]
    pub typography_rules: Vec<String>,

    /// Do's and don'ts
    #[serde(default)]
    pub dos_and_donts: DosDonts,

    /// Additional guidelines document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<PathBuf>,
}

/// Do's and don'ts guidelines
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DosDonts {
    /// Things to do
    #[serde(default, rename = "do")]
    pub dos: Vec<String>,

    /// Things not to do
    #[serde(default, rename = "dont")]
    pub donts: Vec<String>,
}

/// Brand pack metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandMetadata {
    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Author/creator
    #[serde(default)]
    pub author: String,

    /// Contact email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Additional metadata
    #[serde(default, flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for BrandMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            author: String::new(),
            contact: None,
            tags: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brand_pack_creation() {
        let pack = BrandPack::new("Test Brand");
        assert_eq!(pack.identity.name, "Test Brand");
        assert!(!pack.id.is_empty());
    }

    #[test]
    fn test_brand_pack_validation() {
        let mut pack = BrandPack::default();
        assert!(pack.validate().is_ok());

        pack.identity.name = String::new();
        assert!(pack.validate().is_err());
    }

    #[test]
    fn test_brand_color() {
        let color = BrandColor::with_variants("#3B82F6", "#60A5FA", "#2563EB").locked();
        assert!(color.locked);
        assert_eq!(color.light, Some("#60A5FA".into()));
    }

    #[test]
    fn test_color_map() {
        let mut pack = BrandPack::default();
        pack.colors.custom.insert("brand-blue".into(), BrandColor::new("#0066CC"));

        let map = pack.color_map();
        assert!(map.contains_key("primary"));
        assert!(map.contains_key("brand-blue"));
    }

    #[test]
    fn test_serialization() {
        let pack = BrandPack::new("Serialize Test");
        let json = pack.to_json().unwrap();
        let loaded = BrandPack::from_json(&json).unwrap();
        assert_eq!(loaded.identity.name, "Serialize Test");
    }
}
