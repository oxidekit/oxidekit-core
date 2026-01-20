//! Color Format Handling
//!
//! Provides color format types and formatting utilities for displaying
//! and parsing colors in various formats.

use crate::color::Color;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported color formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColorFormat {
    /// Hexadecimal format (#RRGGBB)
    #[default]
    Hex,
    /// Hexadecimal with alpha (#RRGGBBAA)
    HexAlpha,
    /// RGB format (rgb(r, g, b))
    Rgb,
    /// RGBA format (rgba(r, g, b, a))
    Rgba,
    /// HSL format (hsl(h, s%, l%))
    Hsl,
    /// HSLA format (hsla(h, s%, l%, a))
    Hsla,
    /// HSV/HSB format (hsv(h, s%, v%))
    Hsv,
    /// HSVA/HSBA format (hsva(h, s%, v%, a))
    Hsva,
}

impl ColorFormat {
    /// Get all available color formats
    pub fn all() -> &'static [ColorFormat] {
        &[
            ColorFormat::Hex,
            ColorFormat::HexAlpha,
            ColorFormat::Rgb,
            ColorFormat::Rgba,
            ColorFormat::Hsl,
            ColorFormat::Hsla,
            ColorFormat::Hsv,
            ColorFormat::Hsva,
        ]
    }

    /// Get formats without alpha channel
    pub fn without_alpha() -> &'static [ColorFormat] {
        &[
            ColorFormat::Hex,
            ColorFormat::Rgb,
            ColorFormat::Hsl,
            ColorFormat::Hsv,
        ]
    }

    /// Get formats with alpha channel
    pub fn with_alpha() -> &'static [ColorFormat] {
        &[
            ColorFormat::HexAlpha,
            ColorFormat::Rgba,
            ColorFormat::Hsla,
            ColorFormat::Hsva,
        ]
    }

    /// Check if this format includes alpha
    pub fn has_alpha(&self) -> bool {
        matches!(
            self,
            ColorFormat::HexAlpha | ColorFormat::Rgba | ColorFormat::Hsla | ColorFormat::Hsva
        )
    }

    /// Get the corresponding format with alpha
    pub fn with_alpha_variant(self) -> ColorFormat {
        match self {
            ColorFormat::Hex => ColorFormat::HexAlpha,
            ColorFormat::Rgb => ColorFormat::Rgba,
            ColorFormat::Hsl => ColorFormat::Hsla,
            ColorFormat::Hsv => ColorFormat::Hsva,
            other => other,
        }
    }

    /// Get the corresponding format without alpha
    pub fn without_alpha_variant(self) -> ColorFormat {
        match self {
            ColorFormat::HexAlpha => ColorFormat::Hex,
            ColorFormat::Rgba => ColorFormat::Rgb,
            ColorFormat::Hsla => ColorFormat::Hsl,
            ColorFormat::Hsva => ColorFormat::Hsv,
            other => other,
        }
    }

    /// Get a human-readable name for the format
    pub fn name(&self) -> &'static str {
        match self {
            ColorFormat::Hex => "HEX",
            ColorFormat::HexAlpha => "HEXA",
            ColorFormat::Rgb => "RGB",
            ColorFormat::Rgba => "RGBA",
            ColorFormat::Hsl => "HSL",
            ColorFormat::Hsla => "HSLA",
            ColorFormat::Hsv => "HSV",
            ColorFormat::Hsva => "HSVA",
        }
    }

    /// Format a color in this format
    pub fn format(&self, color: &Color) -> String {
        match self {
            ColorFormat::Hex => color.to_hex(),
            ColorFormat::HexAlpha => color.to_hex_alpha(),
            ColorFormat::Rgb => {
                let (r, g, b) = color.to_rgb8();
                format!("rgb({}, {}, {})", r, g, b)
            }
            ColorFormat::Rgba => {
                let (r, g, b) = color.to_rgb8();
                format!("rgba({}, {}, {}, {:.2})", r, g, b, color.a)
            }
            ColorFormat::Hsl => {
                let (h, s, l) = color.to_hsl();
                format!("hsl({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, l * 100.0)
            }
            ColorFormat::Hsla => {
                let (h, s, l) = color.to_hsl();
                format!(
                    "hsla({:.0}, {:.0}%, {:.0}%, {:.2})",
                    h,
                    s * 100.0,
                    l * 100.0,
                    color.a
                )
            }
            ColorFormat::Hsv => {
                let (h, s, v) = color.to_hsv();
                format!("hsv({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, v * 100.0)
            }
            ColorFormat::Hsva => {
                let (h, s, v) = color.to_hsv();
                format!(
                    "hsva({:.0}, {:.0}%, {:.0}%, {:.2})",
                    h,
                    s * 100.0,
                    v * 100.0,
                    color.a
                )
            }
        }
    }
}

impl fmt::Display for ColorFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A color value with its format information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedColor {
    /// The color value
    pub color: Color,
    /// The format to use for display
    pub format: ColorFormat,
}

impl FormattedColor {
    /// Create a new formatted color
    pub fn new(color: Color, format: ColorFormat) -> Self {
        Self { color, format }
    }

    /// Get the formatted string representation
    pub fn to_string(&self) -> String {
        self.format.format(&self.color)
    }

    /// Change the format
    pub fn with_format(self, format: ColorFormat) -> Self {
        Self { format, ..self }
    }

    /// Change the color
    pub fn with_color(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

impl Default for FormattedColor {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            format: ColorFormat::Hex,
        }
    }
}

impl fmt::Display for FormattedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// CSS color output options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CssOutputOptions {
    /// Whether to use modern CSS syntax (e.g., "rgb(255 128 64 / 0.5)")
    pub modern_syntax: bool,
    /// Number of decimal places for alpha values
    pub alpha_precision: u8,
    /// Whether to include alpha when it's 1.0
    pub always_include_alpha: bool,
    /// Whether to use lowercase hex
    pub lowercase_hex: bool,
}

impl Default for CssOutputOptions {
    fn default() -> Self {
        Self {
            modern_syntax: false,
            alpha_precision: 2,
            always_include_alpha: false,
            lowercase_hex: false,
        }
    }
}

impl CssOutputOptions {
    /// Create options for modern CSS syntax
    pub fn modern() -> Self {
        Self {
            modern_syntax: true,
            ..Default::default()
        }
    }

    /// Format a color with these options
    pub fn format(&self, color: &Color, format: ColorFormat) -> String {
        match format {
            ColorFormat::Hex => {
                let hex = color.to_hex();
                if self.lowercase_hex {
                    hex.to_lowercase()
                } else {
                    hex
                }
            }
            ColorFormat::HexAlpha => {
                let hex = color.to_hex_alpha();
                if self.lowercase_hex {
                    hex.to_lowercase()
                } else {
                    hex
                }
            }
            ColorFormat::Rgb | ColorFormat::Rgba => {
                let (r, g, b) = color.to_rgb8();
                let include_alpha = format.has_alpha() || (self.always_include_alpha && color.a < 1.0);

                if self.modern_syntax {
                    if include_alpha {
                        format!(
                            "rgb({} {} {} / {:.prec$})",
                            r,
                            g,
                            b,
                            color.a,
                            prec = self.alpha_precision as usize
                        )
                    } else {
                        format!("rgb({} {} {})", r, g, b)
                    }
                } else if include_alpha {
                    format!(
                        "rgba({}, {}, {}, {:.prec$})",
                        r,
                        g,
                        b,
                        color.a,
                        prec = self.alpha_precision as usize
                    )
                } else {
                    format!("rgb({}, {}, {})", r, g, b)
                }
            }
            ColorFormat::Hsl | ColorFormat::Hsla => {
                let (h, s, l) = color.to_hsl();
                let include_alpha = format.has_alpha() || (self.always_include_alpha && color.a < 1.0);

                if self.modern_syntax {
                    if include_alpha {
                        format!(
                            "hsl({:.0} {:.0}% {:.0}% / {:.prec$})",
                            h,
                            s * 100.0,
                            l * 100.0,
                            color.a,
                            prec = self.alpha_precision as usize
                        )
                    } else {
                        format!("hsl({:.0} {:.0}% {:.0}%)", h, s * 100.0, l * 100.0)
                    }
                } else if include_alpha {
                    format!(
                        "hsla({:.0}, {:.0}%, {:.0}%, {:.prec$})",
                        h,
                        s * 100.0,
                        l * 100.0,
                        color.a,
                        prec = self.alpha_precision as usize
                    )
                } else {
                    format!("hsl({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, l * 100.0)
                }
            }
            ColorFormat::Hsv | ColorFormat::Hsva => {
                let (h, s, v) = color.to_hsv();
                let include_alpha = format.has_alpha() || (self.always_include_alpha && color.a < 1.0);

                // HSV is not standard CSS, use custom format
                if include_alpha {
                    format!(
                        "hsv({:.0}, {:.0}%, {:.0}%, {:.prec$})",
                        h,
                        s * 100.0,
                        v * 100.0,
                        color.a,
                        prec = self.alpha_precision as usize
                    )
                } else {
                    format!("hsv({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, v * 100.0)
                }
            }
        }
    }
}

/// Color string validator
pub struct ColorValidator;

impl ColorValidator {
    /// Check if a string is a valid hex color
    pub fn is_valid_hex(s: &str) -> bool {
        let s = s.trim().trim_start_matches('#');
        matches!(s.len(), 3 | 4 | 6 | 8) && s.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Check if a string is a valid RGB color
    pub fn is_valid_rgb(s: &str) -> bool {
        s.trim().starts_with("rgb") && Color::hex(s).is_ok()
    }

    /// Check if a string is a valid HSL color
    pub fn is_valid_hsl(s: &str) -> bool {
        s.trim().starts_with("hsl") && Color::hex(s).is_ok()
    }

    /// Check if a string represents a valid color in any format
    pub fn is_valid_color(s: &str) -> bool {
        s.parse::<Color>().is_ok()
    }

    /// Detect the format of a color string
    pub fn detect_format(s: &str) -> Option<ColorFormat> {
        let s = s.trim();

        if s.starts_with('#') || s.chars().all(|c| c.is_ascii_hexdigit()) {
            let hex_len = s.trim_start_matches('#').len();
            return match hex_len {
                3 | 6 => Some(ColorFormat::Hex),
                4 | 8 => Some(ColorFormat::HexAlpha),
                _ => None,
            };
        }

        if s.starts_with("rgba") {
            return Some(ColorFormat::Rgba);
        }
        if s.starts_with("rgb") {
            return Some(ColorFormat::Rgb);
        }
        if s.starts_with("hsla") {
            return Some(ColorFormat::Hsla);
        }
        if s.starts_with("hsl") {
            return Some(ColorFormat::Hsl);
        }
        if s.starts_with("hsva") {
            return Some(ColorFormat::Hsva);
        }
        if s.starts_with("hsv") {
            return Some(ColorFormat::Hsv);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hex() {
        let color = Color::rgb8(255, 128, 64);
        assert_eq!(ColorFormat::Hex.format(&color), "#FF8040");
    }

    #[test]
    fn test_format_hex_alpha() {
        let color = Color::rgba8(255, 128, 64, 128);
        assert_eq!(ColorFormat::HexAlpha.format(&color), "#FF804080");
    }

    #[test]
    fn test_format_rgb() {
        let color = Color::rgb8(255, 128, 64);
        assert_eq!(ColorFormat::Rgb.format(&color), "rgb(255, 128, 64)");
    }

    #[test]
    fn test_format_rgba() {
        let color = Color::rgba8(255, 128, 64, 128);
        assert_eq!(ColorFormat::Rgba.format(&color), "rgba(255, 128, 64, 0.50)");
    }

    #[test]
    fn test_format_hsl() {
        let color = Color::RED;
        assert_eq!(ColorFormat::Hsl.format(&color), "hsl(0, 100%, 50%)");
    }

    #[test]
    fn test_format_hsv() {
        let color = Color::RED;
        assert_eq!(ColorFormat::Hsv.format(&color), "hsv(0, 100%, 100%)");
    }

    #[test]
    fn test_format_has_alpha() {
        assert!(!ColorFormat::Hex.has_alpha());
        assert!(ColorFormat::HexAlpha.has_alpha());
        assert!(!ColorFormat::Rgb.has_alpha());
        assert!(ColorFormat::Rgba.has_alpha());
        assert!(!ColorFormat::Hsl.has_alpha());
        assert!(ColorFormat::Hsla.has_alpha());
    }

    #[test]
    fn test_alpha_variants() {
        assert_eq!(ColorFormat::Hex.with_alpha_variant(), ColorFormat::HexAlpha);
        assert_eq!(ColorFormat::Rgb.with_alpha_variant(), ColorFormat::Rgba);
        assert_eq!(ColorFormat::HexAlpha.without_alpha_variant(), ColorFormat::Hex);
        assert_eq!(ColorFormat::Rgba.without_alpha_variant(), ColorFormat::Rgb);
    }

    #[test]
    fn test_css_output_modern() {
        let options = CssOutputOptions::modern();
        let color = Color::rgba8(255, 128, 64, 128);

        let output = options.format(&color, ColorFormat::Rgba);
        assert_eq!(output, "rgb(255 128 64 / 0.50)");
    }

    #[test]
    fn test_css_output_lowercase_hex() {
        let options = CssOutputOptions {
            lowercase_hex: true,
            ..Default::default()
        };
        let color = Color::rgb8(255, 128, 64);

        let output = options.format(&color, ColorFormat::Hex);
        assert_eq!(output, "#ff8040");
    }

    #[test]
    fn test_validator_hex() {
        assert!(ColorValidator::is_valid_hex("#FF0000"));
        assert!(ColorValidator::is_valid_hex("#F00"));
        assert!(ColorValidator::is_valid_hex("FF0000"));
        assert!(ColorValidator::is_valid_hex("#FF000080"));
        assert!(!ColorValidator::is_valid_hex("#GG0000"));
        assert!(!ColorValidator::is_valid_hex("#FF00"));
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(ColorValidator::detect_format("#FF0000"), Some(ColorFormat::Hex));
        assert_eq!(ColorValidator::detect_format("#FF000080"), Some(ColorFormat::HexAlpha));
        assert_eq!(ColorValidator::detect_format("rgb(255, 0, 0)"), Some(ColorFormat::Rgb));
        assert_eq!(ColorValidator::detect_format("rgba(255, 0, 0, 0.5)"), Some(ColorFormat::Rgba));
        assert_eq!(ColorValidator::detect_format("hsl(0, 100%, 50%)"), Some(ColorFormat::Hsl));
    }

    #[test]
    fn test_formatted_color() {
        let fc = FormattedColor::new(Color::RED, ColorFormat::Hex);
        assert_eq!(fc.to_string(), "#FF0000");

        let fc = fc.with_format(ColorFormat::Rgb);
        assert_eq!(fc.to_string(), "rgb(255, 0, 0)");
    }
}
