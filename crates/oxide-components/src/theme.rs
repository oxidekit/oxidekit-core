//! Theme and Design Tokens System
//!
//! Provides structured theming with semantic tokens for colors, spacing,
//! typography, shadows, and other design properties.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,

    /// Theme description
    #[serde(default)]
    pub description: String,

    /// Base theme to extend (if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Design tokens
    pub tokens: DesignTokens,

    /// Theme metadata
    #[serde(default)]
    pub metadata: ThemeMetadata,
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeMetadata {
    /// Theme author
    #[serde(default)]
    pub author: String,

    /// Theme version
    #[serde(default)]
    pub version: String,

    /// Whether this is a dark theme
    #[serde(default)]
    pub is_dark: bool,

    /// Theme license
    #[serde(default)]
    pub license: String,
}

/// All design tokens for a theme
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesignTokens {
    /// Color tokens
    #[serde(default)]
    pub color: ColorTokens,

    /// Spacing tokens
    #[serde(default)]
    pub spacing: SpacingTokens,

    /// Radius tokens
    #[serde(default)]
    pub radius: RadiusTokens,

    /// Shadow tokens
    #[serde(default)]
    pub shadow: ShadowTokens,

    /// Typography tokens
    #[serde(default)]
    pub typography: TypographyTokens,

    /// Motion/animation tokens
    #[serde(default)]
    pub motion: MotionTokens,

    /// Density settings
    #[serde(default)]
    pub density: DensityTokens,

    /// Custom tokens (for extensions)
    #[serde(default, flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Color tokens
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorTokens {
    // Semantic colors
    pub primary: ColorToken,
    pub secondary: ColorToken,
    pub success: ColorToken,
    pub warning: ColorToken,
    pub danger: ColorToken,
    pub info: ColorToken,

    // Surface colors
    pub background: ColorToken,
    pub surface: ColorToken,
    pub surface_variant: ColorToken,

    // Text colors
    pub text: ColorToken,
    pub text_secondary: ColorToken,
    pub text_disabled: ColorToken,
    pub text_inverse: ColorToken,

    // Border colors
    pub border: ColorToken,
    pub border_strong: ColorToken,
    pub divider: ColorToken,

    // State colors
    pub hover: ColorToken,
    pub focus: ColorToken,
    pub active: ColorToken,
    pub disabled: ColorToken,

    /// Additional named colors
    #[serde(flatten)]
    pub custom: HashMap<String, ColorToken>,
}

/// A color token with variants
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorToken {
    /// Main color value (hex, rgba)
    #[serde(default)]
    pub value: String,

    /// Lighter variant
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<String>,

    /// Darker variant
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dark: Option<String>,

    /// Contrast text color
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contrast: Option<String>,

    /// Alpha/opacity variants
    #[serde(default)]
    pub alpha: HashMap<String, String>,
}

impl ColorToken {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            light: None,
            dark: None,
            contrast: None,
            alpha: HashMap::new(),
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
            alpha: HashMap::new(),
        }
    }

    pub fn with_contrast(mut self, contrast: impl Into<String>) -> Self {
        self.contrast = Some(contrast.into());
        self
    }
}

/// Spacing tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingTokens {
    /// Base spacing unit (typically 4 or 8)
    pub base: f32,

    /// Named spacing values
    pub xs: SpacingToken,
    pub sm: SpacingToken,
    pub md: SpacingToken,
    pub lg: SpacingToken,
    pub xl: SpacingToken,
    pub xxl: SpacingToken,

    /// Component-specific spacing
    #[serde(default)]
    pub button: SpacingToken,
    #[serde(default)]
    pub input: SpacingToken,
    #[serde(default)]
    pub card: SpacingToken,

    /// Additional named spacing
    #[serde(flatten)]
    pub custom: HashMap<String, SpacingToken>,
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            base: 4.0,
            xs: SpacingToken::new(4.0),
            sm: SpacingToken::new(8.0),
            md: SpacingToken::new(16.0),
            lg: SpacingToken::new(24.0),
            xl: SpacingToken::new(32.0),
            xxl: SpacingToken::new(48.0),
            button: SpacingToken::padding(12.0, 16.0),
            input: SpacingToken::padding(8.0, 12.0),
            card: SpacingToken::new(16.0),
            custom: HashMap::new(),
        }
    }
}

/// A spacing token
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpacingToken {
    /// Single value (all sides)
    #[serde(default)]
    pub value: f32,

    /// Horizontal (x-axis) value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,

    /// Vertical (y-axis) value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,

    /// Individual sides (top, right, bottom, left)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<f32>,
}

impl SpacingToken {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }

    pub fn padding(y: f32, x: f32) -> Self {
        Self {
            value: 0.0,
            x: Some(x),
            y: Some(y),
            ..Default::default()
        }
    }

    pub fn sides(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            value: 0.0,
            x: None,
            y: None,
            top: Some(top),
            right: Some(right),
            bottom: Some(bottom),
            left: Some(left),
        }
    }
}

/// Radius tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusTokens {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub full: f32,

    /// Component-specific radii
    #[serde(default)]
    pub button: f32,
    #[serde(default)]
    pub input: f32,
    #[serde(default)]
    pub card: f32,
    #[serde(default)]
    pub dialog: f32,

    /// Additional named radii
    #[serde(flatten)]
    pub custom: HashMap<String, f32>,
}

impl Default for RadiusTokens {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            full: 9999.0,
            button: 6.0,
            input: 6.0,
            card: 12.0,
            dialog: 16.0,
            custom: HashMap::new(),
        }
    }
}

/// Shadow tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowTokens {
    pub none: ShadowToken,
    pub sm: ShadowToken,
    pub md: ShadowToken,
    pub lg: ShadowToken,
    pub xl: ShadowToken,

    /// Component-specific shadows
    #[serde(default)]
    pub card: ShadowToken,
    #[serde(default)]
    pub dialog: ShadowToken,
    #[serde(default)]
    pub dropdown: ShadowToken,

    /// Additional named shadows
    #[serde(flatten)]
    pub custom: HashMap<String, ShadowToken>,
}

impl Default for ShadowTokens {
    fn default() -> Self {
        Self {
            none: ShadowToken::none(),
            sm: ShadowToken::new(0.0, 1.0, 2.0, "rgba(0,0,0,0.05)"),
            md: ShadowToken::new(0.0, 4.0, 6.0, "rgba(0,0,0,0.1)"),
            lg: ShadowToken::new(0.0, 10.0, 15.0, "rgba(0,0,0,0.1)"),
            xl: ShadowToken::new(0.0, 20.0, 25.0, "rgba(0,0,0,0.15)"),
            card: ShadowToken::new(0.0, 4.0, 6.0, "rgba(0,0,0,0.1)"),
            dialog: ShadowToken::new(0.0, 25.0, 50.0, "rgba(0,0,0,0.25)"),
            dropdown: ShadowToken::new(0.0, 10.0, 15.0, "rgba(0,0,0,0.1)"),
            custom: HashMap::new(),
        }
    }
}

/// A shadow token
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShadowToken {
    /// X offset
    pub x: f32,
    /// Y offset
    pub y: f32,
    /// Blur radius
    pub blur: f32,
    /// Spread radius
    #[serde(default)]
    pub spread: f32,
    /// Shadow color
    pub color: String,
}

impl ShadowToken {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn new(x: f32, y: f32, blur: f32, color: impl Into<String>) -> Self {
        Self {
            x,
            y,
            blur,
            spread: 0.0,
            color: color.into(),
        }
    }
}

/// Typography tokens
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypographyTokens {
    /// Font families
    #[serde(default)]
    pub font_family: FontFamilyTokens,

    /// Font sizes
    #[serde(default)]
    pub font_size: FontSizeTokens,

    /// Font weights
    #[serde(default)]
    pub font_weight: FontWeightTokens,

    /// Line heights
    #[serde(default)]
    pub line_height: LineHeightTokens,

    /// Letter spacing
    #[serde(default)]
    pub letter_spacing: LetterSpacingTokens,
}

/// Font family tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamilyTokens {
    pub sans: String,
    pub serif: String,
    pub mono: String,

    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

impl Default for FontFamilyTokens {
    fn default() -> Self {
        Self {
            sans: "Inter, system-ui, sans-serif".into(),
            serif: "Georgia, serif".into(),
            mono: "JetBrains Mono, monospace".into(),
            custom: HashMap::new(),
        }
    }
}

/// Font size tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizeTokens {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub xxxl: f32,

    #[serde(flatten)]
    pub custom: HashMap<String, f32>,
}

impl Default for FontSizeTokens {
    fn default() -> Self {
        Self {
            xs: 12.0,
            sm: 14.0,
            md: 16.0,
            lg: 18.0,
            xl: 20.0,
            xxl: 24.0,
            xxxl: 32.0,
            custom: HashMap::new(),
        }
    }
}

/// Font weight tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontWeightTokens {
    pub thin: u16,
    pub light: u16,
    pub normal: u16,
    pub medium: u16,
    pub semibold: u16,
    pub bold: u16,
    pub extrabold: u16,

    #[serde(flatten)]
    pub custom: HashMap<String, u16>,
}

impl Default for FontWeightTokens {
    fn default() -> Self {
        Self {
            thin: 100,
            light: 300,
            normal: 400,
            medium: 500,
            semibold: 600,
            bold: 700,
            extrabold: 800,
            custom: HashMap::new(),
        }
    }
}

/// Line height tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineHeightTokens {
    pub tight: f32,
    pub normal: f32,
    pub relaxed: f32,
    pub loose: f32,

    #[serde(flatten)]
    pub custom: HashMap<String, f32>,
}

impl Default for LineHeightTokens {
    fn default() -> Self {
        Self {
            tight: 1.25,
            normal: 1.5,
            relaxed: 1.75,
            loose: 2.0,
            custom: HashMap::new(),
        }
    }
}

/// Letter spacing tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetterSpacingTokens {
    pub tighter: f32,
    pub tight: f32,
    pub normal: f32,
    pub wide: f32,
    pub wider: f32,

    #[serde(flatten)]
    pub custom: HashMap<String, f32>,
}

impl Default for LetterSpacingTokens {
    fn default() -> Self {
        Self {
            tighter: -0.05,
            tight: -0.025,
            normal: 0.0,
            wide: 0.025,
            wider: 0.05,
            custom: HashMap::new(),
        }
    }
}

/// Motion/animation tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionTokens {
    /// Duration values (in ms)
    #[serde(default)]
    pub duration: DurationTokens,

    /// Easing functions
    #[serde(default)]
    pub easing: EasingTokens,
}

impl Default for MotionTokens {
    fn default() -> Self {
        Self {
            duration: DurationTokens::default(),
            easing: EasingTokens::default(),
        }
    }
}

/// Duration tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationTokens {
    pub instant: u32,
    pub fast: u32,
    pub normal: u32,
    pub slow: u32,

    #[serde(flatten)]
    pub custom: HashMap<String, u32>,
}

impl Default for DurationTokens {
    fn default() -> Self {
        Self {
            instant: 50,
            fast: 150,
            normal: 300,
            slow: 500,
            custom: HashMap::new(),
        }
    }
}

/// Easing tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EasingTokens {
    pub linear: String,
    pub ease_in: String,
    pub ease_out: String,
    pub ease_in_out: String,

    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

impl Default for EasingTokens {
    fn default() -> Self {
        Self {
            linear: "linear".into(),
            ease_in: "cubic-bezier(0.4, 0, 1, 1)".into(),
            ease_out: "cubic-bezier(0, 0, 0.2, 1)".into(),
            ease_in_out: "cubic-bezier(0.4, 0, 0.2, 1)".into(),
            custom: HashMap::new(),
        }
    }
}

/// Density tokens for compact/comfortable/spacious modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DensityTokens {
    /// Current density mode
    pub mode: DensityMode,

    /// Scale factor for spacing (1.0 = normal)
    pub scale: f32,
}

impl Default for DensityTokens {
    fn default() -> Self {
        Self {
            mode: DensityMode::Comfortable,
            scale: 1.0,
        }
    }
}

/// Density modes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DensityMode {
    Compact,
    #[default]
    Comfortable,
    Spacious,
}

impl Theme {
    /// Create a default dark theme
    pub fn dark() -> Self {
        Self {
            name: "OxideKit Dark".into(),
            description: "Default dark theme for OxideKit".into(),
            extends: None,
            tokens: DesignTokens {
                color: ColorTokens {
                    primary: ColorToken::with_variants("#3B82F6", "#60A5FA", "#2563EB")
                        .with_contrast("#FFFFFF"),
                    secondary: ColorToken::with_variants("#6B7280", "#9CA3AF", "#4B5563")
                        .with_contrast("#FFFFFF"),
                    success: ColorToken::with_variants("#22C55E", "#4ADE80", "#16A34A")
                        .with_contrast("#FFFFFF"),
                    warning: ColorToken::with_variants("#F59E0B", "#FBBF24", "#D97706")
                        .with_contrast("#000000"),
                    danger: ColorToken::with_variants("#EF4444", "#F87171", "#DC2626")
                        .with_contrast("#FFFFFF"),
                    info: ColorToken::with_variants("#06B6D4", "#22D3EE", "#0891B2")
                        .with_contrast("#FFFFFF"),
                    background: ColorToken::new("#0B0F14"),
                    surface: ColorToken::new("#1F2937"),
                    surface_variant: ColorToken::new("#374151"),
                    text: ColorToken::new("#E5E7EB"),
                    text_secondary: ColorToken::new("#9CA3AF"),
                    text_disabled: ColorToken::new("#6B7280"),
                    text_inverse: ColorToken::new("#111827"),
                    border: ColorToken::new("#374151"),
                    border_strong: ColorToken::new("#4B5563"),
                    divider: ColorToken::new("#374151"),
                    hover: ColorToken::new("rgba(255,255,255,0.05)"),
                    focus: ColorToken::new("rgba(59,130,246,0.5)"),
                    active: ColorToken::new("rgba(255,255,255,0.1)"),
                    disabled: ColorToken::new("rgba(255,255,255,0.1)"),
                    custom: HashMap::new(),
                },
                spacing: SpacingTokens::default(),
                radius: RadiusTokens::default(),
                shadow: ShadowTokens::default(),
                typography: TypographyTokens::default(),
                motion: MotionTokens::default(),
                density: DensityTokens::default(),
                custom: HashMap::new(),
            },
            metadata: ThemeMetadata {
                author: "OxideKit".into(),
                version: "1.0.0".into(),
                is_dark: true,
                license: "MIT".into(),
            },
        }
    }

    /// Create a default light theme
    pub fn light() -> Self {
        Self {
            name: "OxideKit Light".into(),
            description: "Default light theme for OxideKit".into(),
            extends: None,
            tokens: DesignTokens {
                color: ColorTokens {
                    primary: ColorToken::with_variants("#2563EB", "#3B82F6", "#1D4ED8")
                        .with_contrast("#FFFFFF"),
                    secondary: ColorToken::with_variants("#6B7280", "#9CA3AF", "#4B5563")
                        .with_contrast("#FFFFFF"),
                    success: ColorToken::with_variants("#16A34A", "#22C55E", "#15803D")
                        .with_contrast("#FFFFFF"),
                    warning: ColorToken::with_variants("#D97706", "#F59E0B", "#B45309")
                        .with_contrast("#000000"),
                    danger: ColorToken::with_variants("#DC2626", "#EF4444", "#B91C1C")
                        .with_contrast("#FFFFFF"),
                    info: ColorToken::with_variants("#0891B2", "#06B6D4", "#0E7490")
                        .with_contrast("#FFFFFF"),
                    background: ColorToken::new("#FFFFFF"),
                    surface: ColorToken::new("#F9FAFB"),
                    surface_variant: ColorToken::new("#F3F4F6"),
                    text: ColorToken::new("#111827"),
                    text_secondary: ColorToken::new("#6B7280"),
                    text_disabled: ColorToken::new("#9CA3AF"),
                    text_inverse: ColorToken::new("#F9FAFB"),
                    border: ColorToken::new("#E5E7EB"),
                    border_strong: ColorToken::new("#D1D5DB"),
                    divider: ColorToken::new("#E5E7EB"),
                    hover: ColorToken::new("rgba(0,0,0,0.05)"),
                    focus: ColorToken::new("rgba(37,99,235,0.5)"),
                    active: ColorToken::new("rgba(0,0,0,0.1)"),
                    disabled: ColorToken::new("rgba(0,0,0,0.1)"),
                    custom: HashMap::new(),
                },
                spacing: SpacingTokens::default(),
                radius: RadiusTokens::default(),
                shadow: ShadowTokens::default(),
                typography: TypographyTokens::default(),
                motion: MotionTokens::default(),
                density: DensityTokens::default(),
                custom: HashMap::new(),
            },
            metadata: ThemeMetadata {
                author: "OxideKit".into(),
                version: "1.0.0".into(),
                is_dark: false,
                license: "MIT".into(),
            },
        }
    }

    /// Export theme to TOML
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Load theme from TOML
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();
        assert!(theme.metadata.is_dark);
        assert_eq!(theme.tokens.color.background.value, "#0B0F14");
    }

    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert!(!theme.metadata.is_dark);
        assert_eq!(theme.tokens.color.background.value, "#FFFFFF");
    }

    #[test]
    fn test_serialize_theme() {
        let theme = Theme::dark();
        let toml = theme.to_toml().unwrap();
        assert!(toml.contains("OxideKit Dark"));
    }

    #[test]
    fn test_default_spacing() {
        let spacing = SpacingTokens::default();
        assert_eq!(spacing.base, 4.0);
        assert_eq!(spacing.md.value, 16.0);
    }
}
