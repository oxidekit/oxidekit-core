//! Color Representation and Conversions
//!
//! Provides a unified Color struct with accurate conversions between
//! RGB, HSL, HSV/HSB, and HEX color formats.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A color with RGBA components (0.0-1.0 range)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    /// Red component (0.0-1.0)
    pub r: f32,
    /// Green component (0.0-1.0)
    pub g: f32,
    /// Blue component (0.0-1.0)
    pub b: f32,
    /// Alpha/opacity component (0.0-1.0)
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    // ========================================================================
    // Common color constants
    // ========================================================================

    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create a color from RGB components (0.0-1.0 range)
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from RGBA components (0.0-1.0 range)
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB components (0-255 range)
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    /// Create a color from RGBA components (0-255 range)
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Create a color from a hex string (e.g., "#FF0000", "#FF0000FF", "FF0000")
    pub fn hex(hex: &str) -> Result<Self, ColorParseError> {
        Self::from_str(hex)
    }

    /// Create a color from HSL components
    /// - h: Hue (0.0-360.0)
    /// - s: Saturation (0.0-1.0)
    /// - l: Lightness (0.0-1.0)
    pub fn hsl(h: f32, s: f32, l: f32) -> Self {
        Self::hsla(h, s, l, 1.0)
    }

    /// Create a color from HSLA components
    /// - h: Hue (0.0-360.0)
    /// - s: Saturation (0.0-1.0)
    /// - l: Lightness (0.0-1.0)
    /// - a: Alpha (0.0-1.0)
    pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        let h = h % 360.0;
        let h = if h < 0.0 { h + 360.0 } else { h };

        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);

        if s == 0.0 {
            return Self::rgba(l, l, l, a);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let h_normalized = h / 360.0;

        let r = hue_to_rgb(p, q, h_normalized + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h_normalized);
        let b = hue_to_rgb(p, q, h_normalized - 1.0 / 3.0);

        Self::rgba(r, g, b, a)
    }

    /// Create a color from HSV/HSB components
    /// - h: Hue (0.0-360.0)
    /// - s: Saturation (0.0-1.0)
    /// - v: Value/Brightness (0.0-1.0)
    pub fn hsv(h: f32, s: f32, v: f32) -> Self {
        Self::hsva(h, s, v, 1.0)
    }

    /// Create a color from HSVA/HSBA components
    /// - h: Hue (0.0-360.0)
    /// - s: Saturation (0.0-1.0)
    /// - v: Value/Brightness (0.0-1.0)
    /// - a: Alpha (0.0-1.0)
    pub fn hsva(h: f32, s: f32, v: f32, a: f32) -> Self {
        let h = h % 360.0;
        let h = if h < 0.0 { h + 360.0 } else { h };

        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        if s == 0.0 {
            return Self::rgba(v, v, v, a);
        }

        let h = h / 60.0;
        let i = h.floor() as i32;
        let f = h - i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        let (r, g, b) = match i % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        Self::rgba(r, g, b, a)
    }

    // ========================================================================
    // Conversions TO other formats
    // ========================================================================

    /// Convert to RGB components (0-255 range)
    pub fn to_rgb8(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
        )
    }

    /// Convert to RGBA components (0-255 range)
    pub fn to_rgba8(&self) -> (u8, u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
            (self.a * 255.0).round() as u8,
        )
    }

    /// Convert to hex string (e.g., "#FF0000")
    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb8();
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    /// Convert to hex string with alpha (e.g., "#FF0000FF")
    pub fn to_hex_alpha(&self) -> String {
        let (r, g, b, a) = self.to_rgba8();
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    }

    /// Convert to HSL components
    /// Returns (hue: 0-360, saturation: 0-1, lightness: 0-1)
    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let r = self.r;
        let g = self.g;
        let b = self.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta == 0.0 {
            return (0.0, 0.0, l);
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let h = if max == r {
            ((g - b) / delta) % 6.0
        } else if max == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };

        let h = h * 60.0;
        let h = if h < 0.0 { h + 360.0 } else { h };

        (h, s, l)
    }

    /// Convert to HSLA components
    /// Returns (hue: 0-360, saturation: 0-1, lightness: 0-1, alpha: 0-1)
    pub fn to_hsla(&self) -> (f32, f32, f32, f32) {
        let (h, s, l) = self.to_hsl();
        (h, s, l, self.a)
    }

    /// Convert to HSV/HSB components
    /// Returns (hue: 0-360, saturation: 0-1, value: 0-1)
    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let r = self.r;
        let g = self.g;
        let b = self.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let v = max;

        if delta == 0.0 {
            return (0.0, 0.0, v);
        }

        let s = delta / max;

        let h = if max == r {
            ((g - b) / delta) % 6.0
        } else if max == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };

        let h = h * 60.0;
        let h = if h < 0.0 { h + 360.0 } else { h };

        (h, s, v)
    }

    /// Convert to HSVA/HSBA components
    /// Returns (hue: 0-360, saturation: 0-1, value: 0-1, alpha: 0-1)
    pub fn to_hsva(&self) -> (f32, f32, f32, f32) {
        let (h, s, v) = self.to_hsv();
        (h, s, v, self.a)
    }

    // ========================================================================
    // Color manipulation
    // ========================================================================

    /// Set the alpha/opacity value
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            a: alpha.clamp(0.0, 1.0),
            ..self
        }
    }

    /// Lighten the color by a factor (0.0-1.0)
    pub fn lighten(self, factor: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_l = (l + factor).clamp(0.0, 1.0);
        Self::hsla(h, s, new_l, self.a)
    }

    /// Darken the color by a factor (0.0-1.0)
    pub fn darken(self, factor: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_l = (l - factor).clamp(0.0, 1.0);
        Self::hsla(h, s, new_l, self.a)
    }

    /// Saturate the color by a factor (0.0-1.0)
    pub fn saturate(self, factor: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_s = (s + factor).clamp(0.0, 1.0);
        Self::hsla(h, new_s, l, self.a)
    }

    /// Desaturate the color by a factor (0.0-1.0)
    pub fn desaturate(self, factor: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_s = (s - factor).clamp(0.0, 1.0);
        Self::hsla(h, new_s, l, self.a)
    }

    /// Rotate the hue by degrees
    pub fn rotate_hue(self, degrees: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::hsla(h + degrees, s, l, self.a)
    }

    /// Get the grayscale version of the color
    pub fn grayscale(self) -> Self {
        // Using luminosity method
        let gray = 0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b;
        Self::rgba(gray, gray, gray, self.a)
    }

    /// Invert the color
    pub fn invert(self) -> Self {
        Self::rgba(1.0 - self.r, 1.0 - self.g, 1.0 - self.b, self.a)
    }

    /// Mix two colors by a factor (0.0 = self, 1.0 = other)
    pub fn mix(self, other: Color, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self::rgba(
            self.r + (other.r - self.r) * factor,
            self.g + (other.g - self.g) * factor,
            self.b + (other.b - self.b) * factor,
            self.a + (other.a - self.a) * factor,
        )
    }

    /// Get the complementary color (180 degree hue rotation)
    pub fn complement(self) -> Self {
        self.rotate_hue(180.0)
    }

    /// Calculate relative luminance (for contrast calculations)
    pub fn luminance(&self) -> f32 {
        fn adjust(c: f32) -> f32 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        0.2126 * adjust(self.r) + 0.7152 * adjust(self.g) + 0.0722 * adjust(self.b)
    }

    /// Calculate contrast ratio with another color (WCAG)
    pub fn contrast_ratio(&self, other: &Color) -> f32 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Check if the color is light (luminance > 0.5)
    pub fn is_light(&self) -> bool {
        self.luminance() > 0.5
    }

    /// Check if the color is dark (luminance <= 0.5)
    pub fn is_dark(&self) -> bool {
        !self.is_light()
    }
}

// Helper function for HSL to RGB conversion
fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let t = if t < 0.0 {
        t + 1.0
    } else if t > 1.0 {
        t - 1.0
    } else {
        t
    };

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

/// Error parsing a color string
#[derive(Debug, Clone, thiserror::Error)]
pub enum ColorParseError {
    #[error("Invalid hex format: {0}")]
    InvalidHexFormat(String),
    #[error("Invalid hex character: {0}")]
    InvalidHexCharacter(char),
    #[error("Invalid color format: {0}")]
    InvalidFormat(String),
}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Handle hex format
        if s.starts_with('#') || s.chars().all(|c| c.is_ascii_hexdigit()) {
            return parse_hex(s);
        }

        // Handle rgb/rgba format
        if s.starts_with("rgb") {
            return parse_rgb_function(s);
        }

        // Handle hsl/hsla format
        if s.starts_with("hsl") {
            return parse_hsl_function(s);
        }

        // Handle named colors
        if let Some(color) = named_color(s) {
            return Ok(color);
        }

        Err(ColorParseError::InvalidFormat(s.to_string()))
    }
}

fn parse_hex(s: &str) -> Result<Color, ColorParseError> {
    let hex = s.trim_start_matches('#');

    let (r, g, b, a) = match hex.len() {
        // #RGB
        3 => {
            let r = parse_hex_digit(hex.chars().nth(0).unwrap())? * 17;
            let g = parse_hex_digit(hex.chars().nth(1).unwrap())? * 17;
            let b = parse_hex_digit(hex.chars().nth(2).unwrap())? * 17;
            (r, g, b, 255)
        }
        // #RGBA
        4 => {
            let r = parse_hex_digit(hex.chars().nth(0).unwrap())? * 17;
            let g = parse_hex_digit(hex.chars().nth(1).unwrap())? * 17;
            let b = parse_hex_digit(hex.chars().nth(2).unwrap())? * 17;
            let a = parse_hex_digit(hex.chars().nth(3).unwrap())? * 17;
            (r, g, b, a)
        }
        // #RRGGBB
        6 => {
            let r = parse_hex_pair(&hex[0..2])?;
            let g = parse_hex_pair(&hex[2..4])?;
            let b = parse_hex_pair(&hex[4..6])?;
            (r, g, b, 255)
        }
        // #RRGGBBAA
        8 => {
            let r = parse_hex_pair(&hex[0..2])?;
            let g = parse_hex_pair(&hex[2..4])?;
            let b = parse_hex_pair(&hex[4..6])?;
            let a = parse_hex_pair(&hex[6..8])?;
            (r, g, b, a)
        }
        _ => return Err(ColorParseError::InvalidHexFormat(s.to_string())),
    };

    Ok(Color::rgba8(r, g, b, a))
}

fn parse_hex_digit(c: char) -> Result<u8, ColorParseError> {
    match c.to_ascii_lowercase() {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        _ => Err(ColorParseError::InvalidHexCharacter(c)),
    }
}

fn parse_hex_pair(s: &str) -> Result<u8, ColorParseError> {
    let high = parse_hex_digit(s.chars().nth(0).unwrap())?;
    let low = parse_hex_digit(s.chars().nth(1).unwrap())?;
    Ok(high * 16 + low)
}

fn parse_rgb_function(s: &str) -> Result<Color, ColorParseError> {
    let inner = s
        .trim_start_matches("rgba")
        .trim_start_matches("rgb")
        .trim_start_matches('(')
        .trim_end_matches(')');

    let parts: Vec<&str> = inner.split(|c| c == ',' || c == ' ' || c == '/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    match parts.len() {
        3 => {
            let r = parse_rgb_value(parts[0])?;
            let g = parse_rgb_value(parts[1])?;
            let b = parse_rgb_value(parts[2])?;
            Ok(Color::rgb(r, g, b))
        }
        4 => {
            let r = parse_rgb_value(parts[0])?;
            let g = parse_rgb_value(parts[1])?;
            let b = parse_rgb_value(parts[2])?;
            let a = parse_alpha_value(parts[3])?;
            Ok(Color::rgba(r, g, b, a))
        }
        _ => Err(ColorParseError::InvalidFormat(s.to_string())),
    }
}

fn parse_rgb_value(s: &str) -> Result<f32, ColorParseError> {
    if s.ends_with('%') {
        let percent: f32 = s.trim_end_matches('%').parse()
            .map_err(|_| ColorParseError::InvalidFormat(s.to_string()))?;
        Ok(percent / 100.0)
    } else {
        let value: f32 = s.parse()
            .map_err(|_| ColorParseError::InvalidFormat(s.to_string()))?;
        Ok(value / 255.0)
    }
}

fn parse_alpha_value(s: &str) -> Result<f32, ColorParseError> {
    if s.ends_with('%') {
        let percent: f32 = s.trim_end_matches('%').parse()
            .map_err(|_| ColorParseError::InvalidFormat(s.to_string()))?;
        Ok(percent / 100.0)
    } else {
        let value: f32 = s.parse()
            .map_err(|_| ColorParseError::InvalidFormat(s.to_string()))?;
        // If value > 1, assume it's 0-255 range
        if value > 1.0 {
            Ok(value / 255.0)
        } else {
            Ok(value)
        }
    }
}

fn parse_hsl_function(s: &str) -> Result<Color, ColorParseError> {
    let inner = s
        .trim_start_matches("hsla")
        .trim_start_matches("hsl")
        .trim_start_matches('(')
        .trim_end_matches(')');

    let parts: Vec<&str> = inner.split(|c| c == ',' || c == ' ' || c == '/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    match parts.len() {
        3 => {
            let h = parse_hue_value(parts[0])?;
            let s = parse_percent_value(parts[1])?;
            let l = parse_percent_value(parts[2])?;
            Ok(Color::hsl(h, s, l))
        }
        4 => {
            let h = parse_hue_value(parts[0])?;
            let s = parse_percent_value(parts[1])?;
            let l = parse_percent_value(parts[2])?;
            let a = parse_alpha_value(parts[3])?;
            Ok(Color::hsla(h, s, l, a))
        }
        _ => Err(ColorParseError::InvalidFormat(s.to_string())),
    }
}

fn parse_hue_value(s: &str) -> Result<f32, ColorParseError> {
    let s = s.trim_end_matches("deg");
    s.parse().map_err(|_| ColorParseError::InvalidFormat(s.to_string()))
}

fn parse_percent_value(s: &str) -> Result<f32, ColorParseError> {
    let s = s.trim_end_matches('%');
    let value: f32 = s.parse().map_err(|_| ColorParseError::InvalidFormat(s.to_string()))?;
    Ok(value / 100.0)
}

fn named_color(name: &str) -> Option<Color> {
    match name.to_lowercase().as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::RED),
        "green" => Some(Color::rgb8(0, 128, 0)),
        "blue" => Some(Color::BLUE),
        "yellow" => Some(Color::YELLOW),
        "cyan" | "aqua" => Some(Color::CYAN),
        "magenta" | "fuchsia" => Some(Color::MAGENTA),
        "lime" => Some(Color::GREEN),
        "orange" => Some(Color::rgb8(255, 165, 0)),
        "purple" => Some(Color::rgb8(128, 0, 128)),
        "pink" => Some(Color::rgb8(255, 192, 203)),
        "brown" => Some(Color::rgb8(165, 42, 42)),
        "gray" | "grey" => Some(Color::rgb8(128, 128, 128)),
        "silver" => Some(Color::rgb8(192, 192, 192)),
        "navy" => Some(Color::rgb8(0, 0, 128)),
        "teal" => Some(Color::rgb8(0, 128, 128)),
        "olive" => Some(Color::rgb8(128, 128, 0)),
        "maroon" => Some(Color::rgb8(128, 0, 0)),
        "transparent" => Some(Color::TRANSPARENT),
        _ => None,
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 1.0 {
            write!(f, "{}", self.to_hex())
        } else {
            write!(f, "{}", self.to_hex_alpha())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to check float equality with tolerance
    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn test_rgb_constructors() {
        let c = Color::rgb(0.5, 0.25, 0.75);
        assert!(approx_eq(c.r, 0.5));
        assert!(approx_eq(c.g, 0.25));
        assert!(approx_eq(c.b, 0.75));
        assert!(approx_eq(c.a, 1.0));
    }

    #[test]
    fn test_rgb8_constructors() {
        let c = Color::rgb8(255, 128, 0);
        assert!(approx_eq(c.r, 1.0));
        assert!(approx_eq(c.g, 0.502));
        assert!(approx_eq(c.b, 0.0));
    }

    #[test]
    fn test_hex_parsing() {
        // 6-digit hex
        let c = Color::hex("#FF0000").unwrap();
        assert!(approx_eq(c.r, 1.0));
        assert!(approx_eq(c.g, 0.0));
        assert!(approx_eq(c.b, 0.0));

        // 8-digit hex with alpha
        let c = Color::hex("#FF000080").unwrap();
        assert!(approx_eq(c.r, 1.0));
        assert!(approx_eq(c.a, 0.502));

        // 3-digit hex
        let c = Color::hex("#F00").unwrap();
        assert!(approx_eq(c.r, 1.0));
        assert!(approx_eq(c.g, 0.0));

        // Without hash
        let c = Color::hex("00FF00").unwrap();
        assert!(approx_eq(c.g, 1.0));
    }

    #[test]
    fn test_hex_output() {
        let c = Color::rgb8(255, 128, 64);
        assert_eq!(c.to_hex(), "#FF8040");

        let c = Color::rgba8(255, 128, 64, 128);
        assert_eq!(c.to_hex_alpha(), "#FF804080");
    }

    #[test]
    fn test_hsl_conversion() {
        // Pure red
        let c = Color::RED;
        let (h, s, l) = c.to_hsl();
        assert!(approx_eq(h, 0.0));
        assert!(approx_eq(s, 1.0));
        assert!(approx_eq(l, 0.5));

        // Pure green (lime)
        let c = Color::GREEN;
        let (h, s, l) = c.to_hsl();
        assert!(approx_eq(h, 120.0));
        assert!(approx_eq(s, 1.0));
        assert!(approx_eq(l, 0.5));

        // Pure blue
        let c = Color::BLUE;
        let (h, s, l) = c.to_hsl();
        assert!(approx_eq(h, 240.0));
        assert!(approx_eq(s, 1.0));
        assert!(approx_eq(l, 0.5));
    }

    #[test]
    fn test_hsl_roundtrip() {
        let original = Color::rgb8(180, 90, 45);
        let (h, s, l) = original.to_hsl();
        let converted = Color::hsl(h, s, l);

        assert!(approx_eq(original.r, converted.r));
        assert!(approx_eq(original.g, converted.g));
        assert!(approx_eq(original.b, converted.b));
    }

    #[test]
    fn test_hsv_conversion() {
        // Pure red
        let c = Color::RED;
        let (h, s, v) = c.to_hsv();
        assert!(approx_eq(h, 0.0));
        assert!(approx_eq(s, 1.0));
        assert!(approx_eq(v, 1.0));

        // Gray (no saturation)
        let c = Color::rgb(0.5, 0.5, 0.5);
        let (_, s, _) = c.to_hsv();
        assert!(approx_eq(s, 0.0));
    }

    #[test]
    fn test_hsv_roundtrip() {
        let original = Color::rgb8(180, 90, 45);
        let (h, s, v) = original.to_hsv();
        let converted = Color::hsv(h, s, v);

        assert!(approx_eq(original.r, converted.r));
        assert!(approx_eq(original.g, converted.g));
        assert!(approx_eq(original.b, converted.b));
    }

    #[test]
    fn test_lighten_darken() {
        let c = Color::rgb(0.5, 0.5, 0.5);

        let lighter = c.lighten(0.2);
        assert!(lighter.luminance() > c.luminance());

        let darker = c.darken(0.2);
        assert!(darker.luminance() < c.luminance());
    }

    #[test]
    fn test_saturate_desaturate() {
        let c = Color::hsl(180.0, 0.5, 0.5);

        let (_, s1, _) = c.to_hsl();
        let (_, s2, _) = c.saturate(0.2).to_hsl();
        let (_, s3, _) = c.desaturate(0.2).to_hsl();

        assert!(s2 > s1);
        assert!(s3 < s1);
    }

    #[test]
    fn test_rotate_hue() {
        let c = Color::RED;
        let rotated = c.rotate_hue(120.0);
        let (h, _, _) = rotated.to_hsl();
        assert!(approx_eq(h, 120.0));
    }

    #[test]
    fn test_complement() {
        let c = Color::RED;
        let comp = c.complement();
        let (h, _, _) = comp.to_hsl();
        assert!(approx_eq(h, 180.0));
    }

    #[test]
    fn test_grayscale() {
        let c = Color::RED;
        let gray = c.grayscale();
        assert!(approx_eq(gray.r, gray.g));
        assert!(approx_eq(gray.g, gray.b));
    }

    #[test]
    fn test_invert() {
        let c = Color::RED;
        let inverted = c.invert();
        assert!(approx_eq(inverted.r, 0.0));
        assert!(approx_eq(inverted.g, 1.0));
        assert!(approx_eq(inverted.b, 1.0));
    }

    #[test]
    fn test_mix() {
        let c1 = Color::BLACK;
        let c2 = Color::WHITE;
        let mixed = c1.mix(c2, 0.5);

        assert!(approx_eq(mixed.r, 0.5));
        assert!(approx_eq(mixed.g, 0.5));
        assert!(approx_eq(mixed.b, 0.5));
    }

    #[test]
    fn test_contrast_ratio() {
        let black = Color::BLACK;
        let white = Color::WHITE;

        let ratio = black.contrast_ratio(&white);
        assert!(ratio > 20.0); // Should be ~21:1
    }

    #[test]
    fn test_is_light_dark() {
        assert!(Color::WHITE.is_light());
        assert!(!Color::WHITE.is_dark());
        assert!(Color::BLACK.is_dark());
        assert!(!Color::BLACK.is_light());
    }

    #[test]
    fn test_parse_rgb_function() {
        let c: Color = "rgb(255, 128, 0)".parse().unwrap();
        assert!(approx_eq(c.r, 1.0));
        assert!(approx_eq(c.g, 0.502));
        assert!(approx_eq(c.b, 0.0));

        let c: Color = "rgba(255, 128, 0, 0.5)".parse().unwrap();
        assert!(approx_eq(c.a, 0.5));
    }

    #[test]
    fn test_parse_hsl_function() {
        let c: Color = "hsl(0, 100%, 50%)".parse().unwrap();
        assert!(approx_eq(c.r, 1.0));
        assert!(approx_eq(c.g, 0.0));
        assert!(approx_eq(c.b, 0.0));
    }

    #[test]
    fn test_named_colors() {
        let c: Color = "red".parse().unwrap();
        assert_eq!(c, Color::RED);

        let c: Color = "transparent".parse().unwrap();
        assert_eq!(c, Color::TRANSPARENT);
    }

    #[test]
    fn test_display() {
        let c = Color::rgb8(255, 128, 64);
        assert_eq!(format!("{}", c), "#FF8040");

        let c = Color::rgba8(255, 128, 64, 128);
        assert_eq!(format!("{}", c), "#FF804080");
    }

    #[test]
    fn test_with_alpha() {
        let c = Color::RED.with_alpha(0.5);
        assert!(approx_eq(c.a, 0.5));
        assert!(approx_eq(c.r, 1.0)); // RGB unchanged
    }

    #[test]
    fn test_constants() {
        assert_eq!(Color::BLACK, Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(Color::WHITE, Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::RED, Color::rgb(1.0, 0.0, 0.0));
        assert_eq!(Color::GREEN, Color::rgb(0.0, 1.0, 0.0));
        assert_eq!(Color::BLUE, Color::rgb(0.0, 0.0, 1.0));
    }
}
