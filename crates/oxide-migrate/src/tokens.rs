//! Token Extraction
//!
//! Extracts design tokens (colors, typography, spacing, radii, shadows) from CSS
//! variables and common CSS patterns, normalizing them to OxideKit token format.

use crate::analyzer::{AnalysisResult, Framework};
use crate::error::{IssueCategory, MigrateResult, MigrationIssue};
use oxide_components::theme::{
    ColorToken, ColorTokens, DesignTokens, RadiusTokens, ShadowToken, ShadowTokens, SpacingToken,
    SpacingTokens, Theme, ThemeMetadata, TypographyTokens,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTokens {
    /// Generated OxideKit theme
    pub theme: Theme,
    /// Extracted typography configuration
    pub typography: ExtractedTypography,
    /// Fonts found (for fonts.toml)
    pub fonts: ExtractedFonts,
    /// Mapping of original names to normalized names
    pub name_mapping: HashMap<String, String>,
    /// Issues found during extraction
    pub issues: Vec<MigrationIssue>,
    /// Confidence scores for each token category
    pub confidence: TokenConfidence,
}

/// Typography extraction result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractedTypography {
    /// Font family definitions
    pub families: Vec<FontFamilyDef>,
    /// Typography scale (font sizes)
    pub scale: Vec<TypographyScaleEntry>,
    /// Typography roles inferred from usage
    pub roles: Vec<InferredRole>,
}

/// A font family definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamilyDef {
    /// Logical ID (e.g., "sans", "heading")
    pub id: String,
    /// Display name
    pub name: String,
    /// Font stack
    pub stack: Vec<String>,
    /// Available weights
    pub weights: Vec<u16>,
}

/// A typography scale entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyScaleEntry {
    /// Size name (e.g., "xs", "sm", "base", "lg")
    pub name: String,
    /// Size in pixels
    pub size_px: f32,
    /// Recommended line height
    pub line_height: f32,
}

/// An inferred typography role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredRole {
    /// Role name (e.g., "heading", "body", "caption")
    pub name: String,
    /// Font family ID to use
    pub family_id: String,
    /// Size in pixels
    pub size: f32,
    /// Weight
    pub weight: u16,
    /// Line height multiplier
    pub line_height: f32,
    /// Letter spacing in em
    pub letter_spacing: f32,
}

/// Extracted font information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractedFonts {
    /// Primary font (body text)
    pub primary: Option<String>,
    /// Heading font
    pub heading: Option<String>,
    /// Monospace font
    pub mono: Option<String>,
    /// All detected font families
    pub all_fonts: Vec<String>,
    /// Font file paths found (if any)
    pub font_files: Vec<String>,
}

/// Confidence scores for extracted tokens
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenConfidence {
    /// Color token confidence (0.0 to 1.0)
    pub colors: f32,
    /// Typography confidence
    pub typography: f32,
    /// Spacing confidence
    pub spacing: f32,
    /// Radius confidence
    pub radius: f32,
    /// Shadow confidence
    pub shadows: f32,
    /// Overall confidence
    pub overall: f32,
}

/// Token extractor
pub struct TokenExtractor {
    /// Regex patterns for color extraction
    hex_color: Regex,
    rgb_color: Regex,
    hsl_color: Regex,
    /// Spacing value pattern
    spacing_pattern: Regex,
    /// Radius value pattern
    radius_pattern: Regex,
    /// Shadow value pattern
    shadow_pattern: Regex,
}

impl TokenExtractor {
    /// Create a new token extractor
    pub fn new() -> MigrateResult<Self> {
        Ok(Self {
            hex_color: Regex::new(r"#([0-9a-fA-F]{3,8})")?,
            rgb_color: Regex::new(r"rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*([\d.]+))?\s*\)")?,
            hsl_color: Regex::new(r"hsla?\(\s*(\d+)\s*,\s*([\d.]+)%\s*,\s*([\d.]+)%\s*(?:,\s*([\d.]+))?\s*\)")?,
            spacing_pattern: Regex::new(r"(\d+(?:\.\d+)?)(px|rem|em)")?,
            radius_pattern: Regex::new(r"(\d+(?:\.\d+)?)(px|rem|em|%)")?,
            shadow_pattern: Regex::new(
                r"([\d.]+)(px|rem)?\s+([\d.]+)(px|rem)?\s+([\d.]+)(px|rem)?\s*(?:([\d.]+)(px|rem)?\s*)?(rgba?\([^)]+\)|#[0-9a-fA-F]+)"
            )?,
        })
    }

    /// Extract tokens from analysis result
    pub fn extract(&self, analysis: &AnalysisResult) -> MigrateResult<ExtractedTokens> {
        let mut issues = Vec::new();
        let mut name_mapping = HashMap::new();

        // Extract colors
        let color_tokens = self.extract_colors(analysis, &mut issues, &mut name_mapping)?;

        // Extract spacing
        let spacing_tokens = self.extract_spacing(analysis, &mut issues)?;

        // Extract radii
        let radius_tokens = self.extract_radii(analysis, &mut issues)?;

        // Extract shadows
        let shadow_tokens = self.extract_shadows(analysis, &mut issues)?;

        // Extract typography
        let (typography_tokens, extracted_typography) =
            self.extract_typography(analysis, &mut issues)?;

        // Extract fonts
        let fonts = self.extract_fonts(analysis)?;

        // Build theme
        let is_dark = self.detect_dark_mode(analysis);
        let theme = Theme {
            name: format!("Migrated {} Theme", analysis.framework),
            description: format!(
                "Theme migrated from {} (confidence: {:.0}%)",
                analysis.framework,
                analysis.migration_confidence * 100.0
            ),
            extends: None,
            tokens: DesignTokens {
                color: color_tokens,
                spacing: spacing_tokens,
                radius: radius_tokens,
                shadow: shadow_tokens,
                typography: typography_tokens,
                ..Default::default()
            },
            metadata: ThemeMetadata {
                author: "oxide-migrate".into(),
                version: "1.0.0".into(),
                is_dark,
                license: "".into(),
            },
        };

        // Calculate confidence scores
        let confidence = self.calculate_confidence(analysis, &theme);

        Ok(ExtractedTokens {
            theme,
            typography: extracted_typography,
            fonts,
            name_mapping,
            issues,
            confidence,
        })
    }

    /// Extract color tokens from CSS variables and detected colors
    fn extract_colors(
        &self,
        analysis: &AnalysisResult,
        issues: &mut Vec<MigrationIssue>,
        name_mapping: &mut HashMap<String, String>,
    ) -> MigrateResult<ColorTokens> {
        let mut colors = ColorTokens::default();
        let vars = &analysis.css_variables;

        // Map common naming patterns to OxideKit semantic colors
        let semantic_mappings = [
            // Primary
            (
                vec![
                    "primary",
                    "brand",
                    "accent",
                    "main",
                    "blue",
                    "primary-color",
                ],
                "primary",
            ),
            // Secondary
            (
                vec!["secondary", "gray", "grey", "neutral", "muted"],
                "secondary",
            ),
            // Success
            (
                vec!["success", "green", "positive", "valid", "ok"],
                "success",
            ),
            // Warning
            (
                vec!["warning", "yellow", "orange", "caution", "warn"],
                "warning",
            ),
            // Danger/Error
            (
                vec!["danger", "error", "red", "destructive", "negative", "invalid"],
                "danger",
            ),
            // Info
            (vec!["info", "cyan", "teal", "notice"], "info"),
            // Background
            (
                vec!["background", "bg", "body-bg", "page-bg"],
                "background",
            ),
            // Surface
            (
                vec!["surface", "card", "card-bg", "panel", "modal-bg"],
                "surface",
            ),
            // Text
            (vec!["text", "foreground", "body-color", "font-color"], "text"),
            // Border
            (
                vec!["border", "border-color", "outline", "stroke"],
                "border",
            ),
        ];

        for (patterns, semantic_name) in &semantic_mappings {
            for (var_name, value) in vars {
                let lower_name = var_name.to_lowercase();
                for pattern in patterns {
                    if lower_name.contains(pattern) {
                        let color = self.parse_color_value(value);
                        if let Some(color_value) = color {
                            self.set_semantic_color(&mut colors, semantic_name, &color_value);
                            name_mapping.insert(var_name.clone(), semantic_name.to_string());
                            break;
                        }
                    }
                }
            }
        }

        // Also extract Bootstrap-specific variables if detected
        if analysis.framework == Framework::Bootstrap {
            self.extract_bootstrap_colors(vars, &mut colors, name_mapping);
        }

        // Extract Tailwind-specific colors
        if analysis.framework == Framework::Tailwind {
            self.extract_tailwind_colors(vars, &mut colors, name_mapping);
        }

        // Add any detected colors that weren't mapped
        for color in &analysis.detected_colors {
            if !name_mapping.values().any(|v| v == color) {
                // Try to classify by hue
                if let Some(normalized) = self.parse_color_value(color) {
                    colors.custom.insert(
                        format!("detected_{}", colors.custom.len()),
                        ColorToken::new(normalized),
                    );
                }
            }
        }

        // Warn if we couldn't extract key colors
        if colors.primary.value.is_empty() {
            colors.primary = ColorToken::new("#3B82F6"); // Default blue
            issues.push(
                MigrationIssue::warning(
                    IssueCategory::ColorToken,
                    "Could not detect primary color, using default",
                )
                .with_suggestion("Set primary color manually in theme.generated.toml"),
            );
        }

        if colors.background.value.is_empty() {
            colors.background = ColorToken::new("#FFFFFF");
            issues.push(MigrationIssue::info(
                IssueCategory::ColorToken,
                "Background color not detected, using white",
            ));
        }

        if colors.text.value.is_empty() {
            colors.text = ColorToken::new("#1F2937");
            issues.push(MigrationIssue::info(
                IssueCategory::ColorToken,
                "Text color not detected, using default dark",
            ));
        }

        Ok(colors)
    }

    /// Parse a color value to normalized hex format
    fn parse_color_value(&self, value: &str) -> Option<String> {
        let value = value.trim();

        // Hex color
        if let Some(caps) = self.hex_color.captures(value) {
            let hex = caps.get(1)?.as_str();
            return Some(self.normalize_hex(hex));
        }

        // RGB/RGBA
        if let Some(caps) = self.rgb_color.captures(value) {
            let r: u8 = caps.get(1)?.as_str().parse().ok()?;
            let g: u8 = caps.get(2)?.as_str().parse().ok()?;
            let b: u8 = caps.get(3)?.as_str().parse().ok()?;
            let a: Option<f32> = caps.get(4).and_then(|m| m.as_str().parse().ok());

            if let Some(alpha) = a {
                if alpha < 1.0 {
                    return Some(format!("rgba({}, {}, {}, {})", r, g, b, alpha));
                }
            }
            return Some(format!("#{:02X}{:02X}{:02X}", r, g, b));
        }

        // HSL/HSLA - convert to hex
        if let Some(caps) = self.hsl_color.captures(value) {
            let h: f32 = caps.get(1)?.as_str().parse().ok()?;
            let s_pct: f32 = caps.get(2)?.as_str().parse().ok()?;
            let l_pct: f32 = caps.get(3)?.as_str().parse().ok()?;
            let s = s_pct / 100.0;
            let l = l_pct / 100.0;

            let (r, g, b) = self.hsl_to_rgb(h, s, l);
            return Some(format!("#{:02X}{:02X}{:02X}", r, g, b));
        }

        None
    }

    /// Normalize hex color to 6-digit uppercase format
    fn normalize_hex(&self, hex: &str) -> String {
        let hex = hex.to_uppercase();
        if hex.len() == 3 {
            // Expand shorthand
            let chars: Vec<char> = hex.chars().collect();
            format!(
                "#{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]
            )
        } else if hex.len() == 6 {
            format!("#{}", hex)
        } else if hex.len() == 8 {
            // Has alpha
            format!("#{}", hex)
        } else {
            format!("#{}", hex)
        }
    }

    /// Convert HSL to RGB
    fn hsl_to_rgb(&self, h: f32, s: f32, l: f32) -> (u8, u8, u8) {
        let h = h / 360.0;

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let r = self.hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = self.hue_to_rgb(p, q, h);
        let b = self.hue_to_rgb(p, q, h - 1.0 / 3.0);

        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    fn hue_to_rgb(&self, p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    /// Set a semantic color token
    fn set_semantic_color(&self, colors: &mut ColorTokens, name: &str, value: &str) {
        let token = ColorToken::new(value);
        match name {
            "primary" => colors.primary = token,
            "secondary" => colors.secondary = token,
            "success" => colors.success = token,
            "warning" => colors.warning = token,
            "danger" => colors.danger = token,
            "info" => colors.info = token,
            "background" => colors.background = token,
            "surface" => colors.surface = token,
            "text" => colors.text = token,
            "border" => colors.border = token,
            _ => {
                colors.custom.insert(name.to_string(), token);
            }
        }
    }

    /// Extract Bootstrap-specific color variables
    fn extract_bootstrap_colors(
        &self,
        vars: &HashMap<String, String>,
        colors: &mut ColorTokens,
        name_mapping: &mut HashMap<String, String>,
    ) {
        // Bootstrap variable patterns
        let bootstrap_vars = [
            ("bs-primary", "primary"),
            ("bs-secondary", "secondary"),
            ("bs-success", "success"),
            ("bs-info", "info"),
            ("bs-warning", "warning"),
            ("bs-danger", "danger"),
            ("bs-light", "surface"),
            ("bs-dark", "text"),
            ("bs-body-bg", "background"),
            ("bs-body-color", "text"),
            ("bs-border-color", "border"),
        ];

        for (var_pattern, semantic_name) in &bootstrap_vars {
            if let Some(value) = vars.get(*var_pattern) {
                if let Some(color_value) = self.parse_color_value(value) {
                    self.set_semantic_color(colors, semantic_name, &color_value);
                    name_mapping.insert(var_pattern.to_string(), semantic_name.to_string());
                }
            }
        }
    }

    /// Extract Tailwind-specific color variables
    fn extract_tailwind_colors(
        &self,
        vars: &HashMap<String, String>,
        colors: &mut ColorTokens,
        name_mapping: &mut HashMap<String, String>,
    ) {
        // Tailwind typically uses color scales like blue-500, but the main
        // colors might be defined as CSS variables in custom configs
        let tailwind_vars = [
            ("tw-primary", "primary"),
            ("tw-secondary", "secondary"),
            ("color-primary", "primary"),
            ("color-secondary", "secondary"),
        ];

        for (var_pattern, semantic_name) in &tailwind_vars {
            for (var_name, value) in vars {
                if var_name.contains(var_pattern) {
                    if let Some(color_value) = self.parse_color_value(value) {
                        self.set_semantic_color(colors, semantic_name, &color_value);
                        name_mapping.insert(var_name.clone(), semantic_name.to_string());
                    }
                }
            }
        }
    }

    /// Extract spacing tokens
    fn extract_spacing(
        &self,
        analysis: &AnalysisResult,
        issues: &mut Vec<MigrationIssue>,
    ) -> MigrateResult<SpacingTokens> {
        let mut spacing = SpacingTokens::default();
        let vars = &analysis.css_variables;

        // Look for spacing variables
        let spacing_patterns = [
            ("spacer", "base"),
            ("spacing-xs", "xs"),
            ("spacing-sm", "sm"),
            ("spacing-md", "md"),
            ("spacing-lg", "lg"),
            ("spacing-xl", "xl"),
            ("spacing-xxl", "xxl"),
            ("space-1", "xs"),
            ("space-2", "sm"),
            ("space-3", "md"),
            ("space-4", "lg"),
            ("space-5", "xl"),
            ("gap-", "md"),
            ("padding-", "md"),
            ("margin-", "md"),
        ];

        for (var_name, value) in vars {
            let lower_name = var_name.to_lowercase();
            for (pattern, size_name) in &spacing_patterns {
                if lower_name.contains(pattern) {
                    if let Some(px_value) = self.parse_spacing_value(value) {
                        match *size_name {
                            "base" => spacing.base = px_value,
                            "xs" => spacing.xs = SpacingToken::new(px_value),
                            "sm" => spacing.sm = SpacingToken::new(px_value),
                            "md" => spacing.md = SpacingToken::new(px_value),
                            "lg" => spacing.lg = SpacingToken::new(px_value),
                            "xl" => spacing.xl = SpacingToken::new(px_value),
                            "xxl" => spacing.xxl = SpacingToken::new(px_value),
                            _ => {}
                        }
                    }
                }
            }
        }

        // Bootstrap spacing scale detection
        if analysis.framework == Framework::Bootstrap {
            // Bootstrap uses a $spacer variable (default 1rem = 16px)
            if vars.contains_key("spacer") {
                if let Some(base) = vars.get("spacer").and_then(|v| self.parse_spacing_value(v)) {
                    spacing.base = base / 4.0; // Bootstrap spacer is typically 1rem
                    spacing.xs = SpacingToken::new(base * 0.25);
                    spacing.sm = SpacingToken::new(base * 0.5);
                    spacing.md = SpacingToken::new(base);
                    spacing.lg = SpacingToken::new(base * 1.5);
                    spacing.xl = SpacingToken::new(base * 3.0);
                }
            }
        }

        // Tailwind spacing scale detection
        if analysis.framework == Framework::Tailwind {
            // Tailwind uses 4px base by default
            spacing.base = 4.0;
            spacing.xs = SpacingToken::new(4.0);  // space-1
            spacing.sm = SpacingToken::new(8.0);  // space-2
            spacing.md = SpacingToken::new(16.0); // space-4
            spacing.lg = SpacingToken::new(24.0); // space-6
            spacing.xl = SpacingToken::new(32.0); // space-8
            spacing.xxl = SpacingToken::new(48.0); // space-12

            issues.push(MigrationIssue::info(
                IssueCategory::Spacing,
                "Using Tailwind default spacing scale (4px base)",
            ));
        }

        Ok(spacing)
    }

    /// Parse a spacing value to pixels
    fn parse_spacing_value(&self, value: &str) -> Option<f32> {
        if let Some(caps) = self.spacing_pattern.captures(value.trim()) {
            let num: f32 = caps.get(1)?.as_str().parse().ok()?;
            let unit = caps.get(2)?.as_str();

            return Some(match unit {
                "px" => num,
                "rem" => num * 16.0,
                "em" => num * 16.0,
                _ => num,
            });
        }
        None
    }

    /// Extract radius tokens
    fn extract_radii(
        &self,
        analysis: &AnalysisResult,
        issues: &mut Vec<MigrationIssue>,
    ) -> MigrateResult<RadiusTokens> {
        let mut radius = RadiusTokens::default();
        let vars = &analysis.css_variables;

        // Look for radius variables
        let radius_patterns = [
            ("radius-none", "none"),
            ("radius-sm", "sm"),
            ("radius-md", "md"),
            ("radius-lg", "lg"),
            ("radius-xl", "xl"),
            ("radius-full", "full"),
            ("border-radius-sm", "sm"),
            ("border-radius", "md"),
            ("border-radius-lg", "lg"),
            ("border-radius-pill", "full"),
            ("rounded-sm", "sm"),
            ("rounded-md", "md"),
            ("rounded-lg", "lg"),
            ("rounded-xl", "xl"),
            ("rounded-full", "full"),
        ];

        for (var_name, value) in vars {
            let lower_name = var_name.to_lowercase();
            for (pattern, size_name) in &radius_patterns {
                if lower_name.contains(pattern) {
                    if let Some(px_value) = self.parse_radius_value(value) {
                        match *size_name {
                            "none" => radius.none = px_value,
                            "sm" => radius.sm = px_value,
                            "md" => radius.md = px_value,
                            "lg" => radius.lg = px_value,
                            "xl" => radius.xl = px_value,
                            "full" => radius.full = px_value,
                            _ => {}
                        }
                    }
                }
            }
        }

        // Bootstrap radius defaults
        if analysis.framework == Framework::Bootstrap && radius.md == 8.0 {
            // Check if we found any Bootstrap-specific radius
            if let Some(val) = vars
                .get("border-radius")
                .and_then(|v| self.parse_radius_value(v))
            {
                radius.md = val;
                radius.sm = val * 0.5;
                radius.lg = val * 1.5;
                issues.push(MigrationIssue::info(
                    IssueCategory::Spacing,
                    format!("Detected Bootstrap border-radius: {}px", val),
                ));
            }
        }

        // Tailwind radius defaults
        if analysis.framework == Framework::Tailwind {
            radius.none = 0.0;
            radius.sm = 2.0;
            radius.md = 4.0;
            radius.lg = 8.0;
            radius.xl = 12.0;
            radius.full = 9999.0;
        }

        Ok(radius)
    }

    /// Parse a radius value to pixels
    fn parse_radius_value(&self, value: &str) -> Option<f32> {
        if let Some(caps) = self.radius_pattern.captures(value.trim()) {
            let num: f32 = caps.get(1)?.as_str().parse().ok()?;
            let unit = caps.get(2)?.as_str();

            return Some(match unit {
                "px" => num,
                "rem" => num * 16.0,
                "em" => num * 16.0,
                "%" => 9999.0, // Treat percentage as "full" for pill shapes
                _ => num,
            });
        }
        None
    }

    /// Extract shadow tokens
    fn extract_shadows(
        &self,
        analysis: &AnalysisResult,
        issues: &mut Vec<MigrationIssue>,
    ) -> MigrateResult<ShadowTokens> {
        let mut shadows = ShadowTokens::default();
        let vars = &analysis.css_variables;

        // Look for shadow variables
        let shadow_patterns = [
            ("shadow-sm", "sm"),
            ("shadow-md", "md"),
            ("shadow-lg", "lg"),
            ("shadow-xl", "xl"),
            ("shadow-none", "none"),
            ("box-shadow-sm", "sm"),
            ("box-shadow", "md"),
            ("box-shadow-lg", "lg"),
            ("drop-shadow", "md"),
        ];

        for (var_name, value) in vars {
            let lower_name = var_name.to_lowercase();
            for (pattern, size_name) in &shadow_patterns {
                if lower_name.contains(pattern) && !lower_name.contains("inset") {
                    if let Some(shadow) = self.parse_shadow_value(value) {
                        match *size_name {
                            "none" => shadows.none = shadow,
                            "sm" => shadows.sm = shadow,
                            "md" => shadows.md = shadow,
                            "lg" => shadows.lg = shadow,
                            "xl" => shadows.xl = shadow,
                            _ => {}
                        }
                    }
                }
            }
        }

        // Apply framework defaults if no shadows were extracted
        if shadows.sm.blur == 0.0 && shadows.md.blur == 0.0 {
            match analysis.framework {
                Framework::Bootstrap => {
                    shadows.sm = ShadowToken::new(0.0, 0.125 * 16.0, 0.25 * 16.0, "rgba(0,0,0,0.075)");
                    shadows.md = ShadowToken::new(0.0, 0.5 * 16.0, 1.0 * 16.0, "rgba(0,0,0,0.15)");
                    shadows.lg = ShadowToken::new(0.0, 1.0 * 16.0, 3.0 * 16.0, "rgba(0,0,0,0.175)");
                    issues.push(MigrationIssue::info(
                        IssueCategory::General,
                        "Using Bootstrap default shadows",
                    ));
                }
                Framework::Tailwind => {
                    shadows.sm = ShadowToken::new(0.0, 1.0, 2.0, "rgba(0,0,0,0.05)");
                    shadows.md = ShadowToken::new(0.0, 4.0, 6.0, "rgba(0,0,0,0.1)");
                    shadows.lg = ShadowToken::new(0.0, 10.0, 15.0, "rgba(0,0,0,0.1)");
                    shadows.xl = ShadowToken::new(0.0, 20.0, 25.0, "rgba(0,0,0,0.1)");
                    issues.push(MigrationIssue::info(
                        IssueCategory::General,
                        "Using Tailwind default shadows",
                    ));
                }
                _ => {}
            }
        }

        Ok(shadows)
    }

    /// Parse a shadow value
    fn parse_shadow_value(&self, value: &str) -> Option<ShadowToken> {
        // Try to parse common shadow formats
        // Format: offset-x offset-y blur spread? color
        if let Some(caps) = self.shadow_pattern.captures(value.trim()) {
            let x: f32 = caps.get(1)?.as_str().parse().ok()?;
            let y: f32 = caps.get(3)?.as_str().parse().ok()?;
            let blur: f32 = caps.get(5)?.as_str().parse().ok()?;
            let spread: f32 = caps.get(7).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);
            let color = caps.get(9)?.as_str().to_string();

            return Some(ShadowToken {
                x,
                y,
                blur,
                spread,
                color,
            });
        }
        None
    }

    /// Extract typography tokens
    fn extract_typography(
        &self,
        analysis: &AnalysisResult,
        issues: &mut Vec<MigrationIssue>,
    ) -> MigrateResult<(TypographyTokens, ExtractedTypography)> {
        let mut typography = TypographyTokens::default();
        let mut extracted = ExtractedTypography::default();
        let vars = &analysis.css_variables;

        // Extract font families
        let font_family_patterns = [
            ("font-family-sans", "sans"),
            ("font-family-serif", "serif"),
            ("font-family-mono", "mono"),
            ("font-sans", "sans"),
            ("font-serif", "serif"),
            ("font-mono", "mono"),
            ("body-font", "sans"),
            ("heading-font", "sans"),
        ];

        for (var_name, value) in vars {
            let lower_name = var_name.to_lowercase();
            for (pattern, family_type) in &font_family_patterns {
                if lower_name.contains(pattern) {
                    let stack: Vec<String> = value
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .collect();

                    if !stack.is_empty() {
                        match *family_type {
                            "sans" => typography.font_family.sans = stack.join(", "),
                            "serif" => typography.font_family.serif = stack.join(", "),
                            "mono" => typography.font_family.mono = stack.join(", "),
                            _ => {}
                        }

                        extracted.families.push(FontFamilyDef {
                            id: family_type.to_string(),
                            name: stack.first().cloned().unwrap_or_default(),
                            stack,
                            weights: vec![400, 500, 600, 700],
                        });
                    }
                }
            }
        }

        // Extract font sizes
        let size_patterns = [
            ("font-size-xs", "xs"),
            ("font-size-sm", "sm"),
            ("font-size-base", "md"),
            ("font-size-md", "md"),
            ("font-size-lg", "lg"),
            ("font-size-xl", "xl"),
            ("font-size-xxl", "xxl"),
            ("font-size-2xl", "xxl"),
            ("font-size-3xl", "xxxl"),
            ("text-xs", "xs"),
            ("text-sm", "sm"),
            ("text-base", "md"),
            ("text-lg", "lg"),
            ("text-xl", "xl"),
            ("text-2xl", "xxl"),
            ("text-3xl", "xxxl"),
        ];

        for (var_name, value) in vars {
            let lower_name = var_name.to_lowercase();
            for (pattern, size_name) in &size_patterns {
                if lower_name.contains(pattern) {
                    if let Some(px_value) = self.parse_spacing_value(value) {
                        match *size_name {
                            "xs" => typography.font_size.xs = px_value,
                            "sm" => typography.font_size.sm = px_value,
                            "md" => typography.font_size.md = px_value,
                            "lg" => typography.font_size.lg = px_value,
                            "xl" => typography.font_size.xl = px_value,
                            "xxl" => typography.font_size.xxl = px_value,
                            "xxxl" => typography.font_size.xxxl = px_value,
                            _ => {}
                        }

                        extracted.scale.push(TypographyScaleEntry {
                            name: size_name.to_string(),
                            size_px: px_value,
                            line_height: 1.5,
                        });
                    }
                }
            }
        }

        // Add inferred typography roles
        extracted.roles = vec![
            InferredRole {
                name: "body".into(),
                family_id: "sans".into(),
                size: typography.font_size.md,
                weight: 400,
                line_height: 1.5,
                letter_spacing: 0.0,
            },
            InferredRole {
                name: "heading".into(),
                family_id: "sans".into(),
                size: typography.font_size.xxl,
                weight: 600,
                line_height: 1.25,
                letter_spacing: -0.025,
            },
            InferredRole {
                name: "caption".into(),
                family_id: "sans".into(),
                size: typography.font_size.xs,
                weight: 400,
                line_height: 1.4,
                letter_spacing: 0.0,
            },
        ];

        if extracted.families.is_empty() {
            issues.push(MigrationIssue::info(
                IssueCategory::Typography,
                "No font family variables found, using system defaults",
            ));
        }

        Ok((typography, extracted))
    }

    /// Extract font information
    fn extract_fonts(&self, analysis: &AnalysisResult) -> MigrateResult<ExtractedFonts> {
        let mut fonts = ExtractedFonts::default();

        // Use detected fonts from analysis
        fonts.all_fonts = analysis.detected_fonts.clone();

        // Try to identify primary, heading, and mono fonts
        for font in &analysis.detected_fonts {
            let lower = font.to_lowercase();

            // Mono fonts
            if lower.contains("mono")
                || lower.contains("code")
                || lower.contains("consolas")
                || lower.contains("courier")
            {
                if fonts.mono.is_none() {
                    fonts.mono = Some(font.clone());
                }
            }
            // Serif fonts (often used for headings)
            else if lower.contains("serif")
                || lower.contains("georgia")
                || lower.contains("times")
                || lower.contains("playfair")
            {
                if fonts.heading.is_none() {
                    fonts.heading = Some(font.clone());
                }
            }
            // Sans-serif fonts (body text)
            else if lower.contains("inter")
                || lower.contains("roboto")
                || lower.contains("open sans")
                || lower.contains("system-ui")
                || lower.contains("arial")
                || lower.contains("helvetica")
            {
                if fonts.primary.is_none() {
                    fonts.primary = Some(font.clone());
                }
            }
        }

        // Default primary if none found
        if fonts.primary.is_none() && !fonts.all_fonts.is_empty() {
            fonts.primary = fonts.all_fonts.first().cloned();
        }

        Ok(fonts)
    }

    /// Detect if the theme is dark mode
    fn detect_dark_mode(&self, analysis: &AnalysisResult) -> bool {
        let vars = &analysis.css_variables;

        // Check for dark mode indicators
        for (name, value) in vars {
            let lower_name = name.to_lowercase();

            // Check variable names
            if lower_name.contains("dark") {
                return true;
            }

            // Check background color brightness
            if lower_name.contains("background") || lower_name.contains("body-bg") {
                if let Some(color) = self.parse_color_value(value) {
                    if let Some(brightness) = self.calculate_brightness(&color) {
                        if brightness < 0.5 {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Calculate perceived brightness of a color (0.0 = black, 1.0 = white)
    fn calculate_brightness(&self, color: &str) -> Option<f32> {
        // Parse hex color
        if color.starts_with('#') && color.len() >= 7 {
            let r = u8::from_str_radix(&color[1..3], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&color[3..5], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&color[5..7], 16).ok()? as f32 / 255.0;

            // ITU-R BT.709 formula
            return Some(0.2126 * r + 0.7152 * g + 0.0722 * b);
        }

        None
    }

    /// Calculate confidence scores
    fn calculate_confidence(&self, analysis: &AnalysisResult, theme: &Theme) -> TokenConfidence {
        let vars_count = analysis.css_variables.len();

        // Color confidence based on how many semantic colors we extracted
        let colors = &theme.tokens.color;
        let color_score = [
            !colors.primary.value.is_empty(),
            !colors.secondary.value.is_empty(),
            !colors.success.value.is_empty(),
            !colors.danger.value.is_empty(),
            !colors.background.value.is_empty(),
            !colors.text.value.is_empty(),
        ]
        .iter()
        .filter(|&&x| x)
        .count() as f32
            / 6.0;

        // Typography confidence
        let typo = &theme.tokens.typography;
        let typo_score = if !typo.font_family.sans.is_empty() { 0.5 } else { 0.0 }
            + if typo.font_size.md != 16.0 { 0.3 } else { 0.1 }
            + 0.2;

        // Spacing confidence
        let spacing_score = if vars_count > 0 && theme.tokens.spacing.base != 4.0 {
            0.8
        } else {
            0.5
        };

        // Radius confidence
        let radius_score = if theme.tokens.radius.md != 8.0 { 0.8 } else { 0.5 };

        // Shadow confidence
        let shadow_score = if theme.tokens.shadow.md.blur > 0.0 {
            0.8
        } else {
            0.4
        };

        let overall = (color_score + typo_score + spacing_score + radius_score + shadow_score) / 5.0;

        TokenConfidence {
            colors: color_score,
            typography: typo_score,
            spacing: spacing_score,
            radius: radius_score,
            shadows: shadow_score,
            overall,
        }
    }

    /// Export extracted tokens to TOML format
    pub fn to_theme_toml(&self, tokens: &ExtractedTokens) -> MigrateResult<String> {
        Ok(toml::to_string_pretty(&tokens.theme)?)
    }

    /// Export typography to TOML format
    pub fn to_typography_toml(&self, tokens: &ExtractedTokens) -> MigrateResult<String> {
        Ok(toml::to_string_pretty(&tokens.typography)?)
    }

    /// Export fonts to TOML format
    pub fn to_fonts_toml(&self, tokens: &ExtractedTokens) -> MigrateResult<String> {
        Ok(toml::to_string_pretty(&tokens.fonts)?)
    }
}

impl Default for TokenExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to create default token extractor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        let extractor = TokenExtractor::new().unwrap();

        assert_eq!(
            extractor.parse_color_value("#fff"),
            Some("#FFFFFF".to_string())
        );
        assert_eq!(
            extractor.parse_color_value("#3B82F6"),
            Some("#3B82F6".to_string())
        );
        assert_eq!(
            extractor.parse_color_value("#3b82f6"),
            Some("#3B82F6".to_string())
        );
    }

    #[test]
    fn test_parse_rgb_color() {
        let extractor = TokenExtractor::new().unwrap();

        assert_eq!(
            extractor.parse_color_value("rgb(59, 130, 246)"),
            Some("#3B82F6".to_string())
        );
        assert_eq!(
            extractor.parse_color_value("rgba(59, 130, 246, 0.5)"),
            Some("rgba(59, 130, 246, 0.5)".to_string())
        );
    }

    #[test]
    fn test_parse_spacing_value() {
        let extractor = TokenExtractor::new().unwrap();

        assert_eq!(extractor.parse_spacing_value("16px"), Some(16.0));
        assert_eq!(extractor.parse_spacing_value("1rem"), Some(16.0));
        assert_eq!(extractor.parse_spacing_value("0.5rem"), Some(8.0));
    }

    #[test]
    fn test_brightness_calculation() {
        let extractor = TokenExtractor::new().unwrap();

        // White should be bright
        let white_brightness = extractor.calculate_brightness("#FFFFFF").unwrap();
        assert!(white_brightness > 0.9);

        // Black should be dark
        let black_brightness = extractor.calculate_brightness("#000000").unwrap();
        assert!(black_brightness < 0.1);

        // Mid-gray
        let gray_brightness = extractor.calculate_brightness("#808080").unwrap();
        assert!(gray_brightness > 0.4 && gray_brightness < 0.6);
    }

    #[test]
    fn test_extract_from_analysis() {
        let extractor = TokenExtractor::new().unwrap();

        let mut analysis = AnalysisResult::default();
        analysis.framework = Framework::Bootstrap;
        analysis
            .css_variables
            .insert("primary-color".into(), "#3B82F6".into());
        analysis
            .css_variables
            .insert("body-bg".into(), "#FFFFFF".into());
        analysis
            .css_variables
            .insert("text-color".into(), "#1F2937".into());

        let tokens = extractor.extract(&analysis).unwrap();

        assert_eq!(tokens.theme.tokens.color.primary.value, "#3B82F6");
        assert_eq!(tokens.theme.tokens.color.background.value, "#FFFFFF");
        assert!(!tokens.theme.metadata.is_dark);
    }

    #[test]
    fn test_hsl_to_rgb() {
        let extractor = TokenExtractor::new().unwrap();

        // Red: hsl(0, 100%, 50%)
        let (r, g, b) = extractor.hsl_to_rgb(0.0, 1.0, 0.5);
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);

        // Green: hsl(120, 100%, 50%)
        let (r, g, b) = extractor.hsl_to_rgb(120.0, 1.0, 0.5);
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }
}
