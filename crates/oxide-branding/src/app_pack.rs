//! App Pack - Application-Specific Customizations
//!
//! An AppPack extends a BrandPack with application-specific customizations
//! while respecting the brand's governance rules.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::brand_pack::{BrandPack, BrandColor};
use crate::asset::BrandAsset;
use crate::error::{BrandingError, BrandingResult};

/// Application-specific customization pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPack {
    /// Unique app pack ID
    #[serde(default = "generate_uuid")]
    pub id: String,

    /// App pack version
    pub version: String,

    /// Reference to parent brand pack
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends_brand: Option<BrandReference>,

    /// Application information
    pub app_info: AppInfo,

    /// Color customizations (respecting brand locks)
    #[serde(default)]
    pub colors: AppColors,

    /// Typography customizations
    #[serde(default)]
    pub typography: AppTypography,

    /// App-specific assets
    #[serde(default)]
    pub assets: AppAssets,

    /// Design token overrides
    #[serde(default)]
    pub tokens: AppTokens,

    /// Theme customizations
    #[serde(default)]
    pub themes: HashMap<String, ThemeOverrides>,

    /// Metadata
    #[serde(default)]
    pub metadata: AppMetadata,
}

fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

impl AppPack {
    /// Create a new app pack
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            version: "1.0.0".into(),
            extends_brand: None,
            app_info: AppInfo::new(&id),
            colors: AppColors::default(),
            typography: AppTypography::default(),
            assets: AppAssets::default(),
            tokens: AppTokens::default(),
            themes: HashMap::new(),
            metadata: AppMetadata::default(),
        }
    }

    /// Extend a brand pack
    pub fn extend_brand(mut self, brand: &BrandPack) -> Self {
        self.extends_brand = Some(BrandReference {
            id: brand.id.clone(),
            version: brand.version.clone(),
            name: brand.identity.name.clone(),
        });
        self
    }

    /// Add a theme override
    pub fn with_theme(mut self, name: impl Into<String>, overrides: ThemeOverrides) -> Self {
        self.themes.insert(name.into(), overrides);
        self
    }

    /// Load from file
    pub fn from_file(path: impl AsRef<Path>) -> BrandingResult<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let pack: Self = toml::from_str(&content)?;
        Ok(pack)
    }

    /// Save to file
    pub fn save(&self, path: impl AsRef<Path>) -> BrandingResult<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get effective colors (merged with brand)
    pub fn effective_colors(&self, brand: &BrandPack) -> HashMap<String, String> {
        let mut colors = brand.color_map();

        // Apply unlocked overrides
        for (name, color) in &self.colors.overrides {
            // Check if brand color is locked
            let locked = match name.as_str() {
                "primary" => brand.colors.primary.locked,
                "secondary" => brand.colors.secondary.locked,
                "accent" => brand.colors.accent.locked,
                _ => brand.colors.custom.get(name)
                    .map(|c| c.locked)
                    .unwrap_or(false),
            };

            if !locked {
                colors.insert(name.clone(), color.value.clone());
            }
        }

        // Add app-specific colors
        for (name, color) in &self.colors.custom {
            colors.insert(name.clone(), color.value.clone());
        }

        colors
    }

    /// Check if app pack is valid against a brand
    pub fn validate_against_brand(&self, brand: &BrandPack) -> BrandingResult<()> {
        // Check locked color overrides
        for (name, _color) in &self.colors.overrides {
            let is_locked = match name.as_str() {
                "primary" => brand.colors.primary.locked,
                "secondary" => brand.colors.secondary.locked,
                "accent" => brand.colors.accent.locked,
                _ => brand.colors.custom.get(name)
                    .map(|c| c.locked)
                    .unwrap_or(false),
            };

            if is_locked {
                return Err(BrandingError::TokenLocked {
                    token: format!("colors.{}", name),
                    level: crate::error::LockLevel::Brand,
                });
            }
        }

        // Check locked typography
        if self.typography.primary_family.is_some() && brand.typography.primary_family.locked {
            return Err(BrandingError::TokenLocked {
                token: "typography.primary_family".into(),
                level: crate::error::LockLevel::Brand,
            });
        }

        Ok(())
    }
}

impl Default for AppPack {
    fn default() -> Self {
        Self::new("default-app")
    }
}

/// Reference to a brand pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandReference {
    /// Brand pack ID
    pub id: String,

    /// Brand pack version (semver)
    pub version: String,

    /// Brand name (for display)
    pub name: String,
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// Application name
    pub name: String,

    /// Display name
    #[serde(default)]
    pub display_name: String,

    /// Application description
    #[serde(default)]
    pub description: String,

    /// Application version
    #[serde(default)]
    pub version: String,

    /// Bundle identifier (e.g., com.example.app)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,

    /// Application category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<AppCategory>,
}

impl AppInfo {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            description: String::new(),
            version: "1.0.0".into(),
            bundle_id: None,
            category: None,
        }
    }
}

/// Application category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppCategory {
    Business,
    Developer,
    Education,
    Entertainment,
    Finance,
    Games,
    Health,
    Lifestyle,
    Medical,
    Music,
    News,
    Photo,
    Productivity,
    Reference,
    Social,
    Sports,
    Travel,
    Utilities,
    Weather,
    Other,
}

/// App-specific color customizations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppColors {
    /// Overrides for brand colors (if not locked)
    #[serde(default)]
    pub overrides: HashMap<String, BrandColor>,

    /// App-specific custom colors
    #[serde(default)]
    pub custom: HashMap<String, BrandColor>,

    /// Semantic color mappings
    #[serde(default)]
    pub semantic: HashMap<String, String>,
}

/// App-specific typography customizations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppTypography {
    /// Override primary font (if not locked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_family: Option<String>,

    /// Override secondary font (if not locked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_family: Option<String>,

    /// Override mono font
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mono_family: Option<String>,

    /// Custom typography roles
    #[serde(default)]
    pub roles: HashMap<String, TypographyRoleOverride>,
}

/// Typography role override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyRoleOverride {
    /// Font family (reference to brand or custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,

    /// Font size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f32>,

    /// Font weight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,

    /// Line height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,

    /// Letter spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f32>,
}

/// App-specific assets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppAssets {
    /// App icon (if different from brand)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_icon: Option<BrandAsset>,

    /// Splash screen
    #[serde(skip_serializing_if = "Option::is_none")]
    pub splash: Option<BrandAsset>,

    /// App-specific images
    #[serde(default)]
    pub images: Vec<BrandAsset>,

    /// Custom icons
    #[serde(default)]
    pub icons: Vec<BrandAsset>,
}

/// App-specific token overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppTokens {
    /// Spacing overrides
    #[serde(default)]
    pub spacing: HashMap<String, f32>,

    /// Radius overrides
    #[serde(default)]
    pub radius: HashMap<String, f32>,

    /// Shadow overrides
    #[serde(default)]
    pub shadows: HashMap<String, String>,

    /// Motion/animation overrides
    #[serde(default)]
    pub motion: HashMap<String, String>,

    /// Custom tokens
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Theme-specific overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeOverrides {
    /// Color overrides for this theme
    #[serde(default)]
    pub colors: HashMap<String, String>,

    /// Whether this is a dark theme
    #[serde(default)]
    pub is_dark: bool,

    /// Additional token overrides
    #[serde(default)]
    pub tokens: HashMap<String, serde_json::Value>,
}

/// App pack metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Author
    #[serde(default)]
    pub author: String,

    /// Environment (development, staging, production)
    #[serde(default)]
    pub environment: String,

    /// Additional metadata
    #[serde(default, flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for AppMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            author: String::new(),
            environment: "development".into(),
            extra: HashMap::new(),
        }
    }
}

/// App customization builder
#[derive(Debug, Default)]
pub struct AppCustomization {
    colors: HashMap<String, BrandColor>,
    typography: AppTypography,
    tokens: AppTokens,
}

impl AppCustomization {
    /// Create a new customization builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Override a color
    pub fn color(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.colors.insert(name.into(), BrandColor::new(value));
        self
    }

    /// Set primary font
    pub fn primary_font(mut self, family: impl Into<String>) -> Self {
        self.typography.primary_family = Some(family.into());
        self
    }

    /// Set a spacing token
    pub fn spacing(mut self, name: impl Into<String>, value: f32) -> Self {
        self.tokens.spacing.insert(name.into(), value);
        self
    }

    /// Set a radius token
    pub fn radius(mut self, name: impl Into<String>, value: f32) -> Self {
        self.tokens.radius.insert(name.into(), value);
        self
    }

    /// Apply to an app pack
    pub fn apply_to(self, pack: &mut AppPack) {
        pack.colors.overrides = self.colors;
        pack.typography = self.typography;
        pack.tokens = self.tokens;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_pack_creation() {
        let app = AppPack::new("my-app");
        assert_eq!(app.id, "my-app");
    }

    #[test]
    fn test_extend_brand() {
        let brand = BrandPack::new("Test Brand");
        let app = AppPack::new("my-app").extend_brand(&brand);

        assert!(app.extends_brand.is_some());
        assert_eq!(app.extends_brand.unwrap().name, "Test Brand");
    }

    #[test]
    fn test_effective_colors() {
        let mut brand = BrandPack::default();
        brand.colors.primary = BrandColor::new("#FF0000").locked();
        brand.colors.secondary = BrandColor::new("#00FF00");

        let mut app = AppPack::new("test").extend_brand(&brand);
        app.colors.overrides.insert("primary".into(), BrandColor::new("#0000FF"));
        app.colors.overrides.insert("secondary".into(), BrandColor::new("#FFFF00"));

        let colors = app.effective_colors(&brand);

        // Primary should be brand color (locked)
        assert_eq!(colors.get("primary").unwrap(), "#FF0000");
        // Secondary should be overridden
        assert_eq!(colors.get("secondary").unwrap(), "#FFFF00");
    }

    #[test]
    fn test_validate_locked_colors() {
        let mut brand = BrandPack::default();
        brand.colors.primary = BrandColor::new("#FF0000").locked();

        let mut app = AppPack::new("test").extend_brand(&brand);
        app.colors.overrides.insert("primary".into(), BrandColor::new("#0000FF"));

        let result = app.validate_against_brand(&brand);
        assert!(result.is_err());
    }

    #[test]
    fn test_customization_builder() {
        let mut app = AppPack::new("test");
        AppCustomization::new()
            .color("accent", "#FF5500")
            .spacing("md", 20.0)
            .radius("button", 8.0)
            .apply_to(&mut app);

        assert!(app.colors.overrides.contains_key("accent"));
        assert_eq!(app.tokens.spacing.get("md"), Some(&20.0));
    }
}
