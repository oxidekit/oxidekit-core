//! Design Token Extraction
//!
//! Extracts design tokens from Figma files and variables:
//! - Colors (fills, strokes, effects)
//! - Spacing (padding, gaps, margins)
//! - Typography (fonts, sizes, weights)
//! - Radii (corner radius)
//! - Shadows (drop shadows, inner shadows)
//! - Motion (durations, easing - from variables)

use crate::error::{FigmaError, Result};
use crate::types::*;
use oxide_components::{
    ColorToken, ColorTokens, DesignTokens, DurationTokens, EasingTokens,
    FontFamilyTokens, FontSizeTokens, FontWeightTokens, LetterSpacingTokens,
    LineHeightTokens, MotionTokens, RadiusTokens, ShadowToken, ShadowTokens,
    SpacingToken, SpacingTokens, Theme, ThemeMetadata, TypographyTokens,
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

/// Token extractor for Figma files
#[derive(Debug)]
pub struct TokenExtractor {
    /// Configuration for extraction
    config: ExtractorConfig,

    /// Name patterns for semantic detection
    name_patterns: NamePatterns,
}

/// Configuration for token extraction
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    /// Whether to extract colors
    pub extract_colors: bool,

    /// Whether to extract spacing
    pub extract_spacing: bool,

    /// Whether to extract typography
    pub extract_typography: bool,

    /// Whether to extract radii
    pub extract_radii: bool,

    /// Whether to extract shadows
    pub extract_shadows: bool,

    /// Whether to extract motion
    pub extract_motion: bool,

    /// Whether to auto-name tokens semantically
    pub auto_name: bool,

    /// Minimum spacing value to extract (filters noise)
    pub min_spacing: f32,

    /// Spacing values to snap to (for deduplication)
    pub spacing_scale: Vec<f32>,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            extract_colors: true,
            extract_spacing: true,
            extract_typography: true,
            extract_radii: true,
            extract_shadows: true,
            extract_motion: true,
            auto_name: true,
            min_spacing: 2.0,
            spacing_scale: vec![0.0, 2.0, 4.0, 6.0, 8.0, 12.0, 16.0, 20.0, 24.0, 32.0, 40.0, 48.0, 64.0],
        }
    }
}

/// Patterns for recognizing semantic names
#[derive(Debug)]
struct NamePatterns {
    primary: Regex,
    secondary: Regex,
    success: Regex,
    warning: Regex,
    danger: Regex,
    info: Regex,
    background: Regex,
    surface: Regex,
    text: Regex,
    border: Regex,
}

impl NamePatterns {
    fn new() -> Self {
        Self {
            primary: Regex::new(r"(?i)(primary|brand|main|accent)").unwrap(),
            secondary: Regex::new(r"(?i)(secondary|muted|subtle)").unwrap(),
            success: Regex::new(r"(?i)(success|green|positive|valid)").unwrap(),
            warning: Regex::new(r"(?i)(warning|yellow|orange|caution)").unwrap(),
            danger: Regex::new(r"(?i)(danger|error|red|negative|destructive)").unwrap(),
            info: Regex::new(r"(?i)(info|blue|cyan|notice)").unwrap(),
            background: Regex::new(r"(?i)(background|bg|canvas)").unwrap(),
            surface: Regex::new(r"(?i)(surface|card|panel|container)").unwrap(),
            text: Regex::new(r"(?i)(text|foreground|fg|content)").unwrap(),
            border: Regex::new(r"(?i)(border|stroke|outline|divider)").unwrap(),
        }
    }
}

/// Extracted tokens from a Figma file
#[derive(Debug, Clone, Default)]
pub struct ExtractedTokens {
    /// Color tokens
    pub colors: Vec<ExtractedColor>,

    /// Spacing tokens
    pub spacing: Vec<f32>,

    /// Typography styles
    pub typography: Vec<ExtractedTypography>,

    /// Radius values
    pub radii: Vec<f32>,

    /// Shadow tokens
    pub shadows: Vec<ExtractedShadow>,

    /// Motion tokens (from variables)
    pub motion: ExtractedMotion,

    /// Raw variables
    pub variables: HashMap<String, ExtractedVariable>,
}

/// An extracted color
#[derive(Debug, Clone)]
pub struct ExtractedColor {
    /// Suggested semantic name
    pub name: String,

    /// Original Figma name
    pub original_name: Option<String>,

    /// Color value
    pub color: Color,

    /// Source (fill, stroke, effect, variable)
    pub source: ColorSource,

    /// Usage count (for importance ranking)
    pub usage_count: u32,
}

/// Source of a color token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSource {
    Fill,
    Stroke,
    Effect,
    Variable,
    TextFill,
}

/// An extracted typography style
#[derive(Debug, Clone)]
pub struct ExtractedTypography {
    /// Suggested role name (heading, body, caption, etc.)
    pub role: String,

    /// Original Figma name
    pub original_name: Option<String>,

    /// Font family
    pub font_family: String,

    /// Font size
    pub font_size: f32,

    /// Font weight
    pub font_weight: u16,

    /// Line height
    pub line_height: f32,

    /// Letter spacing
    pub letter_spacing: f32,

    /// Usage count
    pub usage_count: u32,
}

/// An extracted shadow
#[derive(Debug, Clone)]
pub struct ExtractedShadow {
    /// Suggested name
    pub name: String,

    /// Original Figma name
    pub original_name: Option<String>,

    /// Shadow type
    pub shadow_type: ShadowType,

    /// X offset
    pub x: f32,

    /// Y offset
    pub y: f32,

    /// Blur radius
    pub blur: f32,

    /// Spread
    pub spread: f32,

    /// Shadow color
    pub color: Color,

    /// Usage count
    pub usage_count: u32,
}

/// Shadow type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowType {
    Drop,
    Inner,
}

/// Extracted motion tokens
#[derive(Debug, Clone, Default)]
pub struct ExtractedMotion {
    pub durations: Vec<(String, u32)>,
    pub easings: Vec<(String, String)>,
}

/// An extracted variable
#[derive(Debug, Clone)]
pub struct ExtractedVariable {
    pub name: String,
    pub variable_type: VariableType,
    pub values: HashMap<String, serde_json::Value>,
    pub scopes: Vec<VariableScope>,
}

impl TokenExtractor {
    /// Create a new token extractor with default config
    pub fn new() -> Self {
        Self::with_config(ExtractorConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: ExtractorConfig) -> Self {
        Self {
            config,
            name_patterns: NamePatterns::new(),
        }
    }

    /// Extract all tokens from a Figma file
    pub fn extract(&self, file: &FigmaFile) -> Result<ExtractedTokens> {
        info!(file_name = %file.name, "Extracting tokens from Figma file");

        let mut tokens = ExtractedTokens::default();

        // Extract from styles
        self.extract_from_styles(file, &mut tokens)?;

        // Walk the document tree
        self.extract_from_document(&file.document, &mut tokens)?;

        // Deduplicate and sort
        self.deduplicate_tokens(&mut tokens);

        info!(
            colors = tokens.colors.len(),
            spacing = tokens.spacing.len(),
            typography = tokens.typography.len(),
            radii = tokens.radii.len(),
            shadows = tokens.shadows.len(),
            "Token extraction complete"
        );

        Ok(tokens)
    }

    /// Extract tokens from variables
    pub fn extract_from_variables(
        &self,
        variables: &VariablesResponse,
    ) -> Result<ExtractedTokens> {
        info!("Extracting tokens from Figma variables");

        let mut tokens = ExtractedTokens::default();

        for (id, var) in &variables.meta.variables {
            let extracted = ExtractedVariable {
                name: var.name.clone(),
                variable_type: var.resolved_type,
                values: var.values_by_mode.clone(),
                scopes: var.scopes.clone(),
            };

            // Extract colors from color variables
            if var.resolved_type == VariableType::Color {
                for (mode_id, value) in &var.values_by_mode {
                    if let Some(color) = self.parse_variable_color(value) {
                        tokens.colors.push(ExtractedColor {
                            name: self.suggest_color_name(&var.name),
                            original_name: Some(var.name.clone()),
                            color,
                            source: ColorSource::Variable,
                            usage_count: 1,
                        });
                    }
                }
            }

            // Extract motion (duration/easing) from float/string variables
            if var.resolved_type == VariableType::Float {
                if var.name.to_lowercase().contains("duration")
                    || var.scopes.contains(&VariableScope::EffectFloat)
                {
                    for (_, value) in &var.values_by_mode {
                        if let Some(duration) = value.as_f64() {
                            tokens.motion.durations.push((
                                var.name.clone(),
                                duration as u32,
                            ));
                        }
                    }
                }
            }

            tokens.variables.insert(id.clone(), extracted);
        }

        Ok(tokens)
    }

    /// Extract from style definitions
    fn extract_from_styles(&self, file: &FigmaFile, tokens: &mut ExtractedTokens) -> Result<()> {
        for (id, style) in &file.styles {
            debug!(style_name = %style.name, style_type = ?style.style_type, "Processing style");

            match style.style_type {
                StyleType::Fill | StyleType::Stroke => {
                    // Will be extracted when we encounter nodes using this style
                }
                StyleType::Text => {
                    // Text styles extracted from nodes
                }
                StyleType::Effect => {
                    // Effects extracted from nodes
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Extract tokens from document tree
    fn extract_from_document(&self, doc: &DocumentNode, tokens: &mut ExtractedTokens) -> Result<()> {
        for page in &doc.children {
            self.extract_from_node(page, tokens, &mut HashSet::new())?;
        }
        Ok(())
    }

    /// Recursively extract from a node
    fn extract_from_node(
        &self,
        node: &Node,
        tokens: &mut ExtractedTokens,
        seen_styles: &mut HashSet<String>,
    ) -> Result<()> {
        // Extract colors from fills
        if self.config.extract_colors {
            for fill in &node.fills {
                if fill.visible && fill.paint_type == PaintType::Solid {
                    if let Some(color) = &fill.color {
                        let source = if node.node_type == NodeType::Text {
                            ColorSource::TextFill
                        } else {
                            ColorSource::Fill
                        };

                        let name = node.styles
                            .as_ref()
                            .and_then(|s| s.fill.as_ref())
                            .cloned()
                            .unwrap_or_else(|| self.suggest_color_name(&node.name));

                        self.add_color(tokens, name, *color, source);
                    }
                }
            }

            // Extract from strokes
            for stroke in &node.strokes {
                if stroke.visible && stroke.paint_type == PaintType::Solid {
                    if let Some(color) = &stroke.color {
                        self.add_color(
                            tokens,
                            self.suggest_color_name(&format!("{}-stroke", node.name)),
                            *color,
                            ColorSource::Stroke,
                        );
                    }
                }
            }
        }

        // Extract spacing
        if self.config.extract_spacing {
            // Padding
            self.add_spacing(tokens, node.padding_left);
            self.add_spacing(tokens, node.padding_right);
            self.add_spacing(tokens, node.padding_top);
            self.add_spacing(tokens, node.padding_bottom);

            // Gap
            self.add_spacing(tokens, node.item_spacing);

            // Sizes (for common component sizes)
            if let Some(bbox) = &node.absolute_bounding_box {
                if bbox.width > 0.0 && bbox.width < 200.0 {
                    self.add_spacing(tokens, bbox.width);
                }
                if bbox.height > 0.0 && bbox.height < 200.0 {
                    self.add_spacing(tokens, bbox.height);
                }
            }
        }

        // Extract radius
        if self.config.extract_radii {
            if node.corner_radius > 0.0 {
                tokens.radii.push(node.corner_radius);
            }
            if let Some(radii) = &node.rectangle_corner_radii {
                for r in radii {
                    if *r > 0.0 {
                        tokens.radii.push(*r);
                    }
                }
            }
        }

        // Extract shadows
        if self.config.extract_shadows {
            for effect in &node.effects {
                if effect.visible {
                    match effect.effect_type {
                        EffectType::DropShadow => {
                            if let Some(color) = &effect.color {
                                let offset = effect.offset.unwrap_or_default();
                                tokens.shadows.push(ExtractedShadow {
                                    name: self.suggest_shadow_name(effect.radius),
                                    original_name: None,
                                    shadow_type: ShadowType::Drop,
                                    x: offset.x,
                                    y: offset.y,
                                    blur: effect.radius,
                                    spread: effect.spread,
                                    color: *color,
                                    usage_count: 1,
                                });
                            }
                        }
                        EffectType::InnerShadow => {
                            if let Some(color) = &effect.color {
                                let offset = effect.offset.unwrap_or_default();
                                tokens.shadows.push(ExtractedShadow {
                                    name: format!("inner-{}", self.suggest_shadow_name(effect.radius)),
                                    original_name: None,
                                    shadow_type: ShadowType::Inner,
                                    x: offset.x,
                                    y: offset.y,
                                    blur: effect.radius,
                                    spread: effect.spread,
                                    color: *color,
                                    usage_count: 1,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Extract typography
        if self.config.extract_typography && node.node_type == NodeType::Text {
            if let Some(style) = &node.style {
                let line_height = if style.line_height_px > 0.0 {
                    style.line_height_px / style.font_size
                } else if style.line_height_percent > 0.0 {
                    style.line_height_percent / 100.0
                } else {
                    1.5
                };

                tokens.typography.push(ExtractedTypography {
                    role: self.suggest_typography_role(style),
                    original_name: node.styles
                        .as_ref()
                        .and_then(|s| s.text.as_ref())
                        .cloned(),
                    font_family: style.font_family.clone(),
                    font_size: style.font_size,
                    font_weight: style.font_weight,
                    line_height,
                    letter_spacing: style.letter_spacing,
                    usage_count: 1,
                });
            }
        }

        // Recurse into children
        for child in &node.children {
            self.extract_from_node(child, tokens, seen_styles)?;
        }

        Ok(())
    }

    /// Add a color to the tokens, incrementing usage count if exists
    fn add_color(&self, tokens: &mut ExtractedTokens, name: String, color: Color, source: ColorSource) {
        // Check for existing similar color
        for existing in &mut tokens.colors {
            if self.colors_similar(&existing.color, &color) {
                existing.usage_count += 1;
                return;
            }
        }

        tokens.colors.push(ExtractedColor {
            name,
            original_name: None,
            color,
            source,
            usage_count: 1,
        });
    }

    /// Add spacing value, snapping to scale
    fn add_spacing(&self, tokens: &mut ExtractedTokens, value: f32) {
        if value < self.config.min_spacing {
            return;
        }

        // Snap to nearest scale value
        let snapped = self.config.spacing_scale
            .iter()
            .min_by(|a, b| {
                ((**a - value).abs())
                    .partial_cmp(&((**b - value).abs()))
                    .unwrap()
            })
            .copied()
            .unwrap_or(value);

        if !tokens.spacing.contains(&snapped) {
            tokens.spacing.push(snapped);
        }
    }

    /// Check if two colors are similar (within threshold)
    fn colors_similar(&self, a: &Color, b: &Color) -> bool {
        let threshold = 0.02; // 2% tolerance
        (a.r - b.r).abs() < threshold
            && (a.g - b.g).abs() < threshold
            && (a.b - b.b).abs() < threshold
            && (a.a - b.a).abs() < threshold
    }

    /// Suggest a semantic name for a color
    fn suggest_color_name(&self, original: &str) -> String {
        let lower = original.to_lowercase();

        if self.name_patterns.primary.is_match(&lower) {
            return "primary".to_string();
        }
        if self.name_patterns.secondary.is_match(&lower) {
            return "secondary".to_string();
        }
        if self.name_patterns.success.is_match(&lower) {
            return "success".to_string();
        }
        if self.name_patterns.warning.is_match(&lower) {
            return "warning".to_string();
        }
        if self.name_patterns.danger.is_match(&lower) {
            return "danger".to_string();
        }
        if self.name_patterns.info.is_match(&lower) {
            return "info".to_string();
        }
        if self.name_patterns.background.is_match(&lower) {
            return "background".to_string();
        }
        if self.name_patterns.surface.is_match(&lower) {
            return "surface".to_string();
        }
        if self.name_patterns.text.is_match(&lower) {
            return "text".to_string();
        }
        if self.name_patterns.border.is_match(&lower) {
            return "border".to_string();
        }

        // Convert to kebab-case
        self.to_kebab_case(original)
    }

    /// Suggest a typography role
    fn suggest_typography_role(&self, style: &TypeStyle) -> String {
        let name_lower = style.font_family.to_lowercase();
        let size = style.font_size;
        let weight = style.font_weight;

        // Heading detection (large size or bold)
        if size >= 24.0 || weight >= 600 {
            if size >= 32.0 {
                return "heading".to_string();
            } else if size >= 24.0 {
                return "subheading".to_string();
            }
        }

        // Caption detection (small size)
        if size <= 12.0 {
            return "caption".to_string();
        }

        // Code detection (mono font)
        if name_lower.contains("mono") || name_lower.contains("code") {
            return "code".to_string();
        }

        // Button text (medium weight, medium size)
        if weight >= 500 && size >= 14.0 && size <= 16.0 {
            return "button".to_string();
        }

        // Default to body
        "body".to_string()
    }

    /// Suggest a shadow name based on blur radius
    fn suggest_shadow_name(&self, blur: f32) -> String {
        match blur as u32 {
            0..=2 => "sm".to_string(),
            3..=6 => "md".to_string(),
            7..=15 => "lg".to_string(),
            _ => "xl".to_string(),
        }
    }

    /// Convert string to kebab-case
    fn to_kebab_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut prev_lower = false;

        for c in s.chars() {
            if c.is_uppercase() && prev_lower {
                result.push('-');
            }
            if c.is_alphanumeric() {
                result.push(c.to_ascii_lowercase());
                prev_lower = c.is_lowercase();
            } else if c == ' ' || c == '_' || c == '-' {
                if !result.ends_with('-') {
                    result.push('-');
                }
                prev_lower = false;
            }
        }

        result.trim_matches('-').to_string()
    }

    /// Parse a variable color value
    fn parse_variable_color(&self, value: &serde_json::Value) -> Option<Color> {
        if let Some(obj) = value.as_object() {
            let r = obj.get("r")?.as_f64()? as f32;
            let g = obj.get("g")?.as_f64()? as f32;
            let b = obj.get("b")?.as_f64()? as f32;
            let a = obj.get("a").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
            return Some(Color { r, g, b, a });
        }
        None
    }

    /// Deduplicate and sort tokens
    fn deduplicate_tokens(&self, tokens: &mut ExtractedTokens) {
        // Sort colors by usage count (most used first)
        tokens.colors.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));

        // Deduplicate and sort spacing
        tokens.spacing.sort_by(|a, b| a.partial_cmp(b).unwrap());
        tokens.spacing.dedup();

        // Sort typography by size (largest first for headings)
        tokens.typography.sort_by(|a, b| {
            b.font_size.partial_cmp(&a.font_size).unwrap()
        });

        // Deduplicate radii
        tokens.radii.sort_by(|a, b| a.partial_cmp(b).unwrap());
        tokens.radii.dedup();

        // Sort shadows by blur (smallest first)
        tokens.shadows.sort_by(|a, b| {
            a.blur.partial_cmp(&b.blur).unwrap()
        });
    }

    /// Convert extracted tokens to OxideKit Theme
    pub fn to_theme(&self, tokens: &ExtractedTokens, name: &str, is_dark: bool) -> Theme {
        Theme {
            name: name.to_string(),
            description: "Generated from Figma".to_string(),
            extends: None,
            tokens: self.build_design_tokens(tokens),
            metadata: ThemeMetadata {
                author: "Figma Import".to_string(),
                version: "1.0.0".to_string(),
                is_dark,
                license: "".to_string(),
            },
        }
    }

    /// Build DesignTokens from extracted tokens
    fn build_design_tokens(&self, tokens: &ExtractedTokens) -> DesignTokens {
        let mut design_tokens = DesignTokens::default();

        // Build color tokens
        let mut color_map: HashMap<String, ColorToken> = HashMap::new();
        for color in &tokens.colors {
            let token = ColorToken::new(color.color.to_hex());
            color_map.insert(color.name.clone(), token);
        }

        // Map to semantic slots
        design_tokens.color = ColorTokens {
            primary: color_map.remove("primary").unwrap_or_default(),
            secondary: color_map.remove("secondary").unwrap_or_default(),
            success: color_map.remove("success").unwrap_or_default(),
            warning: color_map.remove("warning").unwrap_or_default(),
            danger: color_map.remove("danger").unwrap_or_default(),
            info: color_map.remove("info").unwrap_or_default(),
            background: color_map.remove("background").unwrap_or_default(),
            surface: color_map.remove("surface").unwrap_or_default(),
            surface_variant: color_map.remove("surface-variant").unwrap_or_default(),
            text: color_map.remove("text").unwrap_or_default(),
            text_secondary: color_map.remove("text-secondary").unwrap_or_default(),
            text_disabled: color_map.remove("text-disabled").unwrap_or_default(),
            text_inverse: color_map.remove("text-inverse").unwrap_or_default(),
            border: color_map.remove("border").unwrap_or_default(),
            border_strong: color_map.remove("border-strong").unwrap_or_default(),
            divider: color_map.remove("divider").unwrap_or_default(),
            hover: color_map.remove("hover").unwrap_or_default(),
            focus: color_map.remove("focus").unwrap_or_default(),
            active: color_map.remove("active").unwrap_or_default(),
            disabled: color_map.remove("disabled").unwrap_or_default(),
            custom: color_map,
        };

        // Build spacing tokens
        let sorted_spacing: Vec<f32> = tokens.spacing.iter().copied().collect();
        design_tokens.spacing = SpacingTokens {
            base: 4.0,
            xs: SpacingToken::new(*sorted_spacing.get(0).unwrap_or(&4.0)),
            sm: SpacingToken::new(*sorted_spacing.get(1).unwrap_or(&8.0)),
            md: SpacingToken::new(*sorted_spacing.get(2).unwrap_or(&16.0)),
            lg: SpacingToken::new(*sorted_spacing.get(3).unwrap_or(&24.0)),
            xl: SpacingToken::new(*sorted_spacing.get(4).unwrap_or(&32.0)),
            xxl: SpacingToken::new(*sorted_spacing.get(5).unwrap_or(&48.0)),
            button: SpacingToken::default(),
            input: SpacingToken::default(),
            card: SpacingToken::default(),
            custom: HashMap::new(),
        };

        // Build radius tokens
        let sorted_radii: Vec<f32> = tokens.radii.iter().copied().collect();
        design_tokens.radius = RadiusTokens {
            none: 0.0,
            sm: *sorted_radii.get(0).unwrap_or(&4.0),
            md: *sorted_radii.get(1).unwrap_or(&8.0),
            lg: *sorted_radii.get(2).unwrap_or(&12.0),
            xl: *sorted_radii.get(3).unwrap_or(&16.0),
            full: 9999.0,
            button: *sorted_radii.get(1).unwrap_or(&6.0),
            input: *sorted_radii.get(1).unwrap_or(&6.0),
            card: *sorted_radii.get(2).unwrap_or(&12.0),
            dialog: *sorted_radii.get(3).unwrap_or(&16.0),
            custom: HashMap::new(),
        };

        // Build shadow tokens
        let mut shadow_map: HashMap<String, ShadowToken> = HashMap::new();
        for shadow in &tokens.shadows {
            if shadow.shadow_type == ShadowType::Drop {
                shadow_map.insert(
                    shadow.name.clone(),
                    ShadowToken::new(shadow.x, shadow.y, shadow.blur, shadow.color.to_rgba()),
                );
            }
        }

        design_tokens.shadow = ShadowTokens {
            none: ShadowToken::none(),
            sm: shadow_map.remove("sm").unwrap_or_else(|| ShadowToken::new(0.0, 1.0, 2.0, "rgba(0,0,0,0.05)")),
            md: shadow_map.remove("md").unwrap_or_else(|| ShadowToken::new(0.0, 4.0, 6.0, "rgba(0,0,0,0.1)")),
            lg: shadow_map.remove("lg").unwrap_or_else(|| ShadowToken::new(0.0, 10.0, 15.0, "rgba(0,0,0,0.1)")),
            xl: shadow_map.remove("xl").unwrap_or_else(|| ShadowToken::new(0.0, 20.0, 25.0, "rgba(0,0,0,0.15)")),
            card: ShadowToken::new(0.0, 4.0, 6.0, "rgba(0,0,0,0.1)"),
            dialog: ShadowToken::new(0.0, 25.0, 50.0, "rgba(0,0,0,0.25)"),
            dropdown: ShadowToken::new(0.0, 10.0, 15.0, "rgba(0,0,0,0.1)"),
            custom: shadow_map,
        };

        // Build typography tokens
        let fonts: HashSet<String> = tokens.typography.iter()
            .map(|t| t.font_family.clone())
            .collect();
        let sans_font = fonts.iter()
            .find(|f| !f.to_lowercase().contains("mono") && !f.to_lowercase().contains("serif"))
            .cloned()
            .unwrap_or_else(|| "Inter, system-ui, sans-serif".to_string());
        let mono_font = fonts.iter()
            .find(|f| f.to_lowercase().contains("mono") || f.to_lowercase().contains("code"))
            .cloned()
            .unwrap_or_else(|| "JetBrains Mono, monospace".to_string());

        design_tokens.typography = TypographyTokens {
            font_family: FontFamilyTokens {
                sans: sans_font,
                serif: "Georgia, serif".to_string(),
                mono: mono_font,
                custom: HashMap::new(),
            },
            font_size: FontSizeTokens::default(),
            font_weight: FontWeightTokens::default(),
            line_height: LineHeightTokens::default(),
            letter_spacing: LetterSpacingTokens::default(),
        };

        // Build motion tokens
        design_tokens.motion = MotionTokens {
            duration: DurationTokens {
                instant: tokens.motion.durations.iter()
                    .find(|(n, _)| n.to_lowercase().contains("instant"))
                    .map(|(_, v)| *v)
                    .unwrap_or(50),
                fast: tokens.motion.durations.iter()
                    .find(|(n, _)| n.to_lowercase().contains("fast"))
                    .map(|(_, v)| *v)
                    .unwrap_or(150),
                normal: tokens.motion.durations.iter()
                    .find(|(n, _)| n.to_lowercase().contains("normal") || n.to_lowercase().contains("default"))
                    .map(|(_, v)| *v)
                    .unwrap_or(300),
                slow: tokens.motion.durations.iter()
                    .find(|(n, _)| n.to_lowercase().contains("slow"))
                    .map(|(_, v)| *v)
                    .unwrap_or(500),
                custom: HashMap::new(),
            },
            easing: EasingTokens::default(),
        };

        design_tokens
    }
}

impl Default for TokenExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_name_suggestion() {
        let extractor = TokenExtractor::new();
        assert_eq!(extractor.suggest_color_name("Primary Button"), "primary");
        assert_eq!(extractor.suggest_color_name("Error Text"), "danger");
        assert_eq!(extractor.suggest_color_name("Background Color"), "background");
    }

    #[test]
    fn test_kebab_case() {
        let extractor = TokenExtractor::new();
        assert_eq!(extractor.to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(extractor.to_kebab_case("Some Name"), "some-name");
        assert_eq!(extractor.to_kebab_case("already-kebab"), "already-kebab");
    }

    #[test]
    fn test_color_hex() {
        let color = Color { r: 1.0, g: 0.5, b: 0.0, a: 1.0 };
        assert_eq!(color.to_hex(), "#FF7F00");
    }

    #[test]
    fn test_colors_similar() {
        let extractor = TokenExtractor::new();
        let a = Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
        let b = Color { r: 0.51, g: 0.5, b: 0.5, a: 1.0 };
        assert!(extractor.colors_similar(&a, &b));

        let c = Color { r: 0.6, g: 0.5, b: 0.5, a: 1.0 };
        assert!(!extractor.colors_similar(&a, &c));
    }
}
