//! Color representation and utilities

/// RGBA color with f32 components (0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGBA components (0.0-1.0)
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB components with full opacity
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from 8-bit RGBA components (0-255)
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Create a color from a hex string (e.g., "#FF5500" or "#FF550080")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert to an array for shader uniforms
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Transparent color
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Black
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);

    /// White
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    /// OxideKit primary background (#0B0F14)
    pub const OXIDE_BG: Self = Self::rgb(0.043, 0.059, 0.078);

    /// OxideKit surface (#111827)
    pub const OXIDE_SURFACE: Self = Self::rgb(0.067, 0.094, 0.153);

    /// OxideKit text primary (#E5E7EB)
    pub const OXIDE_TEXT: Self = Self::rgb(0.898, 0.906, 0.922);

    /// OxideKit text secondary (#9CA3AF)
    pub const OXIDE_TEXT_SECONDARY: Self = Self::rgb(0.612, 0.639, 0.686);

    /// OxideKit accent (#3B82F6)
    pub const OXIDE_ACCENT: Self = Self::rgb(0.231, 0.510, 0.965);
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        let color = Color::from_hex("#FF5500").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.333).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_from_hex_with_alpha() {
        let color = Color::from_hex("#FF550080").unwrap();
        assert!((color.a - 0.502).abs() < 0.01);
    }
}
