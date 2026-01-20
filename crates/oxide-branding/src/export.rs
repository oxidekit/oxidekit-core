//! Brand Asset Export
//!
//! Exports brand assets in various formats for different platforms and use cases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::brand_pack::BrandPack;
use crate::asset::{AssetPipeline, IconPlatform};
use crate::error::{BrandingError, BrandingResult};

/// Brand exporter
#[derive(Debug)]
pub struct BrandExporter<'a> {
    /// Reference to the brand pack
    brand: &'a BrandPack,

    /// Asset pipeline for processing (used when image-processing feature is enabled)
    #[allow(dead_code)]
    pipeline: AssetPipeline,
}

impl<'a> BrandExporter<'a> {
    /// Create a new exporter for a brand
    pub fn new(brand: &'a BrandPack) -> Self {
        Self {
            brand,
            pipeline: AssetPipeline::new(),
        }
    }

    /// Export brand assets with the given options
    pub fn export(&self, options: ExportOptions) -> BrandingResult<()> {
        std::fs::create_dir_all(&options.output_dir)?;

        // Export based on format
        match options.format {
            ExportFormat::BrandKit => self.export_brand_kit(&options),
            ExportFormat::DesignTokens => self.export_design_tokens(&options),
            ExportFormat::IconSet => self.export_icons(&options),
            ExportFormat::ColorPalette => self.export_color_palette(&options),
            ExportFormat::StyleDictionary => self.export_style_dictionary(&options),
            ExportFormat::Css => self.export_css(&options),
            ExportFormat::Scss => self.export_scss(&options),
            ExportFormat::TailwindConfig => self.export_tailwind(&options),
            ExportFormat::FigmaTokens => self.export_figma_tokens(&options),
            ExportFormat::SwiftUIColors => self.export_swiftui(&options),
            ExportFormat::AndroidColors => self.export_android(&options),
        }
    }

    /// Export complete brand kit
    fn export_brand_kit(&self, options: &ExportOptions) -> BrandingResult<()> {
        let output_dir = &options.output_dir;

        // Create subdirectories
        let colors_dir = output_dir.join("colors");
        let typography_dir = output_dir.join("typography");
        let assets_dir = output_dir.join("assets");
        let icons_dir = output_dir.join("icons");

        for dir in [&colors_dir, &typography_dir, &assets_dir, &icons_dir] {
            std::fs::create_dir_all(dir)?;
        }

        // Export brand pack as TOML
        let brand_path = output_dir.join("brand.toml");
        self.brand.save(&brand_path)?;

        // Export colors
        self.export_color_palette(&ExportOptions {
            output_dir: colors_dir.clone(),
            format: ExportFormat::ColorPalette,
            ..options.clone()
        })?;

        // Export CSS variables
        self.export_css(&ExportOptions {
            output_dir: colors_dir,
            format: ExportFormat::Css,
            ..options.clone()
        })?;

        // Export typography
        let typography_content = self.generate_typography_config()?;
        std::fs::write(typography_dir.join("typography.toml"), typography_content)?;

        // Export README
        let readme = self.generate_brand_readme();
        std::fs::write(output_dir.join("README.md"), readme)?;

        tracing::info!("Exported brand kit to {:?}", output_dir);
        Ok(())
    }

    /// Export design tokens in JSON format
    fn export_design_tokens(&self, options: &ExportOptions) -> BrandingResult<()> {
        let tokens = DesignTokensExport {
            name: self.brand.identity.name.clone(),
            version: self.brand.version.clone(),
            colors: self.brand.color_map(),
            typography: TypographyTokensExport {
                primary_font: self.brand.typography.primary_family.name.clone(),
                secondary_font: self.brand.typography.secondary_family
                    .as_ref()
                    .map(|f| f.name.clone()),
                mono_font: self.brand.typography.mono_family
                    .as_ref()
                    .map(|f| f.name.clone()),
                base_size: self.brand.typography.scale.base_size,
                scale_ratio: self.brand.typography.scale.ratio,
            },
            spacing: self.brand.tokens.spacing.clone(),
            radius: self.brand.tokens.radius.clone(),
            shadows: self.brand.tokens.shadows.clone(),
        };

        let content = serde_json::to_string_pretty(&tokens)?;
        let path = options.output_dir.join("design-tokens.json");
        std::fs::write(path, content)?;

        Ok(())
    }

    /// Export icons for specified platforms
    fn export_icons(&self, _options: &ExportOptions) -> BrandingResult<()> {
        #[cfg(feature = "image-processing")]
        {
            let source = self.brand.identity.icon
                .as_ref()
                .or(self.brand.identity.primary_logo.as_ref())
                .ok_or_else(|| BrandingError::MissingAsset("No icon or logo found".into()))?;

            // Generate icons for each platform
            for platform in &options.platforms {
                let platform_dir = options.output_dir.join(format!("{:?}", platform).to_lowercase());
                std::fs::create_dir_all(&platform_dir)?;
                self.pipeline.generate_icon_set(&source.path, &platform_dir)?;
            }

            // Generate favicons
            let favicon_dir = options.output_dir.join("favicon");
            std::fs::create_dir_all(&favicon_dir)?;
            self.pipeline.generate_favicons(&source.path, &favicon_dir)?;

            Ok(())
        }

        #[cfg(not(feature = "image-processing"))]
        {
            Err(BrandingError::PipelineError(
                "Image processing feature not enabled".into()
            ))
        }
    }

    /// Export color palette
    fn export_color_palette(&self, options: &ExportOptions) -> BrandingResult<()> {
        let colors = self.brand.color_map();
        let palette = ColorPaletteExport {
            name: format!("{} Color Palette", self.brand.identity.name),
            colors: colors.iter().map(|(name, value)| {
                ColorExport {
                    name: name.clone(),
                    value: value.clone(),
                    variants: None,
                }
            }).collect(),
        };

        // Export as JSON
        let json = serde_json::to_string_pretty(&palette)?;
        std::fs::write(options.output_dir.join("colors.json"), json)?;

        // Also export as simple key-value
        let mut simple = String::new();
        for (name, value) in &colors {
            simple.push_str(&format!("{}: {}\n", name, value));
        }
        std::fs::write(options.output_dir.join("colors.txt"), simple)?;

        Ok(())
    }

    /// Export in Style Dictionary format
    fn export_style_dictionary(&self, options: &ExportOptions) -> BrandingResult<()> {
        let mut tokens: HashMap<String, serde_json::Value> = HashMap::new();

        // Colors
        let mut colors: HashMap<String, serde_json::Value> = HashMap::new();
        for (name, value) in self.brand.color_map() {
            colors.insert(name, serde_json::json!({
                "value": value,
                "type": "color"
            }));
        }
        tokens.insert("color".into(), serde_json::json!(colors));

        // Spacing
        let mut spacing: HashMap<String, serde_json::Value> = HashMap::new();
        for (name, value) in &self.brand.tokens.spacing {
            spacing.insert(name.clone(), serde_json::json!({
                "value": format!("{}px", value),
                "type": "spacing"
            }));
        }
        tokens.insert("spacing".into(), serde_json::json!(spacing));

        // Radius
        let mut radius: HashMap<String, serde_json::Value> = HashMap::new();
        for (name, value) in &self.brand.tokens.radius {
            radius.insert(name.clone(), serde_json::json!({
                "value": format!("{}px", value),
                "type": "borderRadius"
            }));
        }
        tokens.insert("radius".into(), serde_json::json!(radius));

        let content = serde_json::to_string_pretty(&tokens)?;
        std::fs::write(options.output_dir.join("tokens.json"), content)?;

        Ok(())
    }

    /// Export as CSS custom properties
    fn export_css(&self, options: &ExportOptions) -> BrandingResult<()> {
        let mut css = String::new();
        css.push_str("/* Brand CSS Variables */\n");
        css.push_str(&format!("/* {} v{} */\n\n", self.brand.identity.name, self.brand.version));

        css.push_str(":root {\n");

        // Colors
        css.push_str("  /* Colors */\n");
        for (name, value) in self.brand.color_map() {
            css.push_str(&format!("  --color-{}: {};\n", name.replace('_', "-"), value));
        }

        // Spacing
        css.push_str("\n  /* Spacing */\n");
        for (name, value) in &self.brand.tokens.spacing {
            css.push_str(&format!("  --spacing-{}: {}px;\n", name.replace('_', "-"), value));
        }

        // Radius
        css.push_str("\n  /* Border Radius */\n");
        for (name, value) in &self.brand.tokens.radius {
            css.push_str(&format!("  --radius-{}: {}px;\n", name.replace('_', "-"), value));
        }

        // Typography
        css.push_str("\n  /* Typography */\n");
        css.push_str(&format!("  --font-primary: {};\n", self.generate_font_stack(&self.brand.typography.primary_family)));
        if let Some(ref mono) = self.brand.typography.mono_family {
            css.push_str(&format!("  --font-mono: {};\n", self.generate_font_stack(mono)));
        }
        css.push_str(&format!("  --font-size-base: {}px;\n", self.brand.typography.scale.base_size));

        css.push_str("}\n");

        std::fs::write(options.output_dir.join("brand.css"), css)?;
        Ok(())
    }

    /// Export as SCSS variables
    fn export_scss(&self, options: &ExportOptions) -> BrandingResult<()> {
        let mut scss = String::new();
        scss.push_str("// Brand SCSS Variables\n");
        scss.push_str(&format!("// {} v{}\n\n", self.brand.identity.name, self.brand.version));

        // Colors
        scss.push_str("// Colors\n");
        for (name, value) in self.brand.color_map() {
            scss.push_str(&format!("$color-{}: {};\n", name.replace('_', "-"), value));
        }

        // Color map
        scss.push_str("\n$colors: (\n");
        for (name, value) in self.brand.color_map() {
            scss.push_str(&format!("  '{}': {},\n", name.replace('_', "-"), value));
        }
        scss.push_str(");\n");

        // Spacing
        scss.push_str("\n// Spacing\n");
        for (name, value) in &self.brand.tokens.spacing {
            scss.push_str(&format!("$spacing-{}: {}px;\n", name.replace('_', "-"), value));
        }

        // Radius
        scss.push_str("\n// Border Radius\n");
        for (name, value) in &self.brand.tokens.radius {
            scss.push_str(&format!("$radius-{}: {}px;\n", name.replace('_', "-"), value));
        }

        std::fs::write(options.output_dir.join("_brand.scss"), scss)?;
        Ok(())
    }

    /// Export as Tailwind CSS config
    fn export_tailwind(&self, options: &ExportOptions) -> BrandingResult<()> {
        let mut config = String::new();
        config.push_str("// Tailwind CSS Brand Configuration\n");
        config.push_str(&format!("// {} v{}\n\n", self.brand.identity.name, self.brand.version));

        config.push_str("module.exports = {\n");
        config.push_str("  theme: {\n");
        config.push_str("    extend: {\n");

        // Colors
        config.push_str("      colors: {\n");
        config.push_str("        brand: {\n");
        for (name, value) in self.brand.color_map() {
            config.push_str(&format!("          '{}': '{}',\n", name.replace('_', "-"), value));
        }
        config.push_str("        },\n");
        config.push_str("      },\n");

        // Font family
        config.push_str("      fontFamily: {\n");
        config.push_str(&format!("        sans: ['{}'],\n", self.brand.typography.primary_family.name));
        if let Some(ref mono) = self.brand.typography.mono_family {
            config.push_str(&format!("        mono: ['{}'],\n", mono.name));
        }
        config.push_str("      },\n");

        // Spacing
        if !self.brand.tokens.spacing.is_empty() {
            config.push_str("      spacing: {\n");
            for (name, value) in &self.brand.tokens.spacing {
                config.push_str(&format!("        '{}': '{}px',\n", name, value));
            }
            config.push_str("      },\n");
        }

        // Border radius
        if !self.brand.tokens.radius.is_empty() {
            config.push_str("      borderRadius: {\n");
            for (name, value) in &self.brand.tokens.radius {
                config.push_str(&format!("        '{}': '{}px',\n", name, value));
            }
            config.push_str("      },\n");
        }

        config.push_str("    },\n");
        config.push_str("  },\n");
        config.push_str("};\n");

        std::fs::write(options.output_dir.join("tailwind.brand.js"), config)?;
        Ok(())
    }

    /// Export as Figma tokens
    fn export_figma_tokens(&self, options: &ExportOptions) -> BrandingResult<()> {
        let mut tokens: HashMap<String, serde_json::Value> = HashMap::new();

        // Colors as Figma token format
        let mut colors: HashMap<String, serde_json::Value> = HashMap::new();
        for (name, value) in self.brand.color_map() {
            colors.insert(name, serde_json::json!({
                "$value": value,
                "$type": "color"
            }));
        }
        tokens.insert("color".into(), serde_json::json!(colors));

        // Typography
        let mut typography: HashMap<String, serde_json::Value> = HashMap::new();
        typography.insert("fontFamilies".into(), serde_json::json!({
            "primary": {
                "$value": self.brand.typography.primary_family.name,
                "$type": "fontFamilies"
            }
        }));
        tokens.insert("typography".into(), serde_json::json!(typography));

        let content = serde_json::to_string_pretty(&tokens)?;
        std::fs::write(options.output_dir.join("figma-tokens.json"), content)?;

        Ok(())
    }

    /// Export as SwiftUI Color extension
    fn export_swiftui(&self, options: &ExportOptions) -> BrandingResult<()> {
        let mut swift = String::new();
        swift.push_str("import SwiftUI\n\n");
        swift.push_str(&format!("// {} Brand Colors\n\n", self.brand.identity.name));

        swift.push_str("extension Color {\n");
        swift.push_str("    struct Brand {\n");

        for (name, value) in self.brand.color_map() {
            let swift_name = to_camel_case(&name);
            if let Some((r, g, b)) = parse_hex_color(&value) {
                swift.push_str(&format!(
                    "        static let {} = Color(red: {:.3}, green: {:.3}, blue: {:.3})\n",
                    swift_name,
                    r as f64 / 255.0,
                    g as f64 / 255.0,
                    b as f64 / 255.0
                ));
            }
        }

        swift.push_str("    }\n");
        swift.push_str("}\n");

        std::fs::write(options.output_dir.join("BrandColors.swift"), swift)?;
        Ok(())
    }

    /// Export as Android colors.xml
    fn export_android(&self, options: &ExportOptions) -> BrandingResult<()> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        xml.push_str(&format!("<!-- {} Brand Colors -->\n", self.brand.identity.name));
        xml.push_str("<resources>\n");

        for (name, value) in self.brand.color_map() {
            let android_name = name.replace('-', "_");
            xml.push_str(&format!("    <color name=\"brand_{}\">{}</color>\n", android_name, value));
        }

        xml.push_str("</resources>\n");

        std::fs::write(options.output_dir.join("colors.xml"), xml)?;
        Ok(())
    }

    fn generate_typography_config(&self) -> BrandingResult<String> {
        let config = serde_json::json!({
            "primary": {
                "family": self.brand.typography.primary_family.name,
                "fallbacks": self.brand.typography.primary_family.fallbacks,
                "weights": self.brand.typography.primary_family.weights,
            },
            "mono": self.brand.typography.mono_family.as_ref().map(|f| serde_json::json!({
                "family": f.name,
                "fallbacks": f.fallbacks,
                "weights": f.weights,
            })),
            "scale": {
                "base_size": self.brand.typography.scale.base_size,
                "ratio": self.brand.typography.scale.ratio,
            }
        });

        Ok(serde_json::to_string_pretty(&config)?)
    }

    fn generate_font_stack(&self, spec: &crate::brand_pack::FontFamilySpec) -> String {
        let mut stack = vec![format!("'{}'", spec.name)];
        stack.extend(spec.fallbacks.iter().map(|f| f.clone()));
        stack.join(", ")
    }

    fn generate_brand_readme(&self) -> String {
        let mut readme = String::new();
        readme.push_str(&format!("# {} Brand Kit\n\n", self.brand.identity.name));

        if !self.brand.identity.description.is_empty() {
            readme.push_str(&format!("{}\n\n", self.brand.identity.description));
        }

        readme.push_str("## Contents\n\n");
        readme.push_str("- `brand.toml` - Complete brand configuration\n");
        readme.push_str("- `colors/` - Color palette exports\n");
        readme.push_str("- `typography/` - Typography configuration\n");
        readme.push_str("- `assets/` - Brand assets\n");
        readme.push_str("- `icons/` - Generated icon sets\n\n");

        readme.push_str("## Colors\n\n");
        readme.push_str("| Name | Value |\n");
        readme.push_str("|------|-------|\n");
        for (name, value) in self.brand.color_map() {
            readme.push_str(&format!("| {} | `{}` |\n", name, value));
        }

        readme.push_str(&format!("\n## Version\n\n{}\n", self.brand.version));

        readme
    }
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Output directory
    pub output_dir: PathBuf,

    /// Export format
    pub format: ExportFormat,

    /// Target platforms (for icon generation)
    pub platforms: Vec<IconPlatform>,

    /// Include assets
    pub include_assets: bool,

    /// Generate documentation
    pub generate_docs: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("brand-export"),
            format: ExportFormat::BrandKit,
            platforms: vec![IconPlatform::Web, IconPlatform::MacOS, IconPlatform::Windows],
            include_assets: true,
            generate_docs: true,
        }
    }
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Complete brand kit
    BrandKit,
    /// Design tokens JSON
    DesignTokens,
    /// Icon set
    IconSet,
    /// Color palette
    ColorPalette,
    /// Style Dictionary format
    StyleDictionary,
    /// CSS custom properties
    Css,
    /// SCSS variables
    Scss,
    /// Tailwind CSS config
    TailwindConfig,
    /// Figma tokens
    FigmaTokens,
    /// SwiftUI colors
    SwiftUIColors,
    /// Android colors.xml
    AndroidColors,
}

/// Design tokens export structure
#[derive(Debug, Serialize, Deserialize)]
struct DesignTokensExport {
    name: String,
    version: String,
    colors: HashMap<String, String>,
    typography: TypographyTokensExport,
    spacing: HashMap<String, f32>,
    radius: HashMap<String, f32>,
    shadows: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TypographyTokensExport {
    primary_font: String,
    secondary_font: Option<String>,
    mono_font: Option<String>,
    base_size: f32,
    scale_ratio: f32,
}

/// Color palette export structure
#[derive(Debug, Serialize, Deserialize)]
struct ColorPaletteExport {
    name: String,
    colors: Vec<ColorExport>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ColorExport {
    name: String,
    value: String,
    variants: Option<HashMap<String, String>>,
}

/// Convert snake_case to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// Parse hex color to RGB
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("primary_color"), "primaryColor");
        assert_eq!(to_camel_case("brand-blue"), "brandBlue");
        assert_eq!(to_camel_case("text"), "text");
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#FF5500"), Some((255, 85, 0)));
        assert_eq!(parse_hex_color("#000000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#FFFFFF"), Some((255, 255, 255)));
    }

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::default();
        assert_eq!(options.format, ExportFormat::BrandKit);
        assert!(options.include_assets);
    }
}
