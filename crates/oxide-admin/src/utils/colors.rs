//! Color utilities

use oxide_render::Color;

/// Convert hex color string to RGBA array
pub fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex)
        .map(|c| c.to_array())
        .unwrap_or([1.0, 1.0, 1.0, 1.0])
}

/// Convert hex with alpha to RGBA
pub fn hex_alpha(hex: &str, alpha: f32) -> [f32; 4] {
    let mut color = hex_to_rgba(hex);
    color[3] = alpha;
    color
}

/// Lighten a color by a percentage (0.0-1.0)
pub fn lighten(color: [f32; 4], amount: f32) -> [f32; 4] {
    [
        (color[0] + (1.0 - color[0]) * amount).min(1.0),
        (color[1] + (1.0 - color[1]) * amount).min(1.0),
        (color[2] + (1.0 - color[2]) * amount).min(1.0),
        color[3],
    ]
}

/// Darken a color by a percentage (0.0-1.0)
pub fn darken(color: [f32; 4], amount: f32) -> [f32; 4] {
    [
        (color[0] * (1.0 - amount)).max(0.0),
        (color[1] * (1.0 - amount)).max(0.0),
        (color[2] * (1.0 - amount)).max(0.0),
        color[3],
    ]
}

/// Calculate relative luminance (for contrast calculations)
pub fn luminance(color: [f32; 4]) -> f32 {
    let r = if color[0] <= 0.03928 {
        color[0] / 12.92
    } else {
        ((color[0] + 0.055) / 1.055).powf(2.4)
    };
    let g = if color[1] <= 0.03928 {
        color[1] / 12.92
    } else {
        ((color[1] + 0.055) / 1.055).powf(2.4)
    };
    let b = if color[2] <= 0.03928 {
        color[2] / 12.92
    } else {
        ((color[2] + 0.055) / 1.055).powf(2.4)
    };
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Calculate contrast ratio between two colors
pub fn contrast_ratio(color1: [f32; 4], color2: [f32; 4]) -> f32 {
    let l1 = luminance(color1);
    let l2 = luminance(color2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Get a contrasting text color (black or white) for a background
pub fn contrasting_text(background: [f32; 4]) -> [f32; 4] {
    let lum = luminance(background);
    if lum > 0.179 {
        [0.0, 0.0, 0.0, 1.0] // Black
    } else {
        [1.0, 1.0, 1.0, 1.0] // White
    }
}

/// Color palette for admin theme
pub mod palette {
    use super::hex_to_rgba;

    // Primary colors
    pub const PRIMARY: [f32; 4] = [0.231, 0.510, 0.965, 1.0]; // #3B82F6
    pub const PRIMARY_DARK: [f32; 4] = [0.145, 0.388, 0.922, 1.0]; // #2563EB
    pub const PRIMARY_LIGHT: [f32; 4] = [0.376, 0.647, 0.980, 1.0]; // #60A5FA

    // Semantic colors
    pub const SUCCESS: [f32; 4] = [0.133, 0.773, 0.369, 1.0]; // #22C55E
    pub const WARNING: [f32; 4] = [0.961, 0.620, 0.043, 1.0]; // #F59E0B
    pub const DANGER: [f32; 4] = [0.937, 0.267, 0.267, 1.0]; // #EF4444
    pub const INFO: [f32; 4] = [0.024, 0.714, 0.831, 1.0]; // #06B6D4

    // Neutral colors (dark theme)
    pub const BACKGROUND: [f32; 4] = [0.043, 0.059, 0.078, 1.0]; // #0B0F14
    pub const SURFACE: [f32; 4] = [0.122, 0.161, 0.216, 1.0]; // #1F2937
    pub const SURFACE_VARIANT: [f32; 4] = [0.216, 0.255, 0.318, 1.0]; // #374151
    pub const BORDER: [f32; 4] = [0.216, 0.255, 0.318, 1.0]; // #374151

    // Text colors
    pub const TEXT_PRIMARY: [f32; 4] = [0.898, 0.906, 0.922, 1.0]; // #E5E7EB
    pub const TEXT_SECONDARY: [f32; 4] = [0.612, 0.639, 0.686, 1.0]; // #9CA3AF
    pub const TEXT_DISABLED: [f32; 4] = [0.420, 0.447, 0.502, 1.0]; // #6B7280
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgba() {
        let color = hex_to_rgba("#FF0000");
        assert!((color[0] - 1.0).abs() < 0.01);
        assert!(color[1] < 0.01);
        assert!(color[2] < 0.01);
    }

    #[test]
    fn test_contrast() {
        let white = [1.0, 1.0, 1.0, 1.0];
        let black = [0.0, 0.0, 0.0, 1.0];
        let ratio = contrast_ratio(white, black);
        assert!(ratio > 20.0); // Should be 21:1
    }

    #[test]
    fn test_contrasting_text() {
        let dark_bg = hex_to_rgba("#1F2937");
        let text = contrasting_text(dark_bg);
        assert!(text[0] > 0.5); // Should be white

        let light_bg = hex_to_rgba("#FFFFFF");
        let text = contrasting_text(light_bg);
        assert!(text[0] < 0.5); // Should be black
    }
}
