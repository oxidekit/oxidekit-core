//! Design Tokens for Admin Components
//!
//! Centralized design tokens module providing consistent colors, spacing,
//! typography, and other design properties across all admin components.
//! Based on the oxide-components theme system.

use oxide_render::Color;

// =============================================================================
// COLOR TOKENS
// =============================================================================

/// Semantic color tokens for the admin UI
pub mod colors {
    // Primary colors
    pub const PRIMARY: &str = "#3B82F6";
    pub const PRIMARY_LIGHT: &str = "#60A5FA";
    pub const PRIMARY_DARK: &str = "#2563EB";
    pub const PRIMARY_CONTRAST: &str = "#FFFFFF";

    // Secondary colors
    pub const SECONDARY: &str = "#6B7280";
    pub const SECONDARY_LIGHT: &str = "#9CA3AF";
    pub const SECONDARY_DARK: &str = "#4B5563";

    // Semantic colors
    pub const SUCCESS: &str = "#22C55E";
    pub const SUCCESS_LIGHT: &str = "#4ADE80";
    pub const SUCCESS_DARK: &str = "#16A34A";
    pub const SUCCESS_BG: &str = "#22C55E20";

    pub const WARNING: &str = "#F59E0B";
    pub const WARNING_LIGHT: &str = "#FBBF24";
    pub const WARNING_DARK: &str = "#D97706";
    pub const WARNING_BG: &str = "#F59E0B20";

    pub const DANGER: &str = "#EF4444";
    pub const DANGER_LIGHT: &str = "#F87171";
    pub const DANGER_DARK: &str = "#DC2626";
    pub const DANGER_BG: &str = "#EF444420";

    pub const INFO: &str = "#06B6D4";
    pub const INFO_LIGHT: &str = "#22D3EE";
    pub const INFO_DARK: &str = "#0891B2";
    pub const INFO_BG: &str = "#06B6D420";

    // Surface colors (dark theme)
    pub const BACKGROUND: &str = "#0B0F14";
    pub const SURFACE: &str = "#1F2937";
    pub const SURFACE_VARIANT: &str = "#374151";
    pub const SURFACE_ELEVATED: &str = "#111827";

    // Text colors
    pub const TEXT_PRIMARY: &str = "#E5E7EB";
    pub const TEXT_SECONDARY: &str = "#9CA3AF";
    pub const TEXT_DISABLED: &str = "#6B7280";
    pub const TEXT_INVERSE: &str = "#111827";
    pub const TEXT_LINK: &str = "#3B82F6";

    // Border colors
    pub const BORDER: &str = "#374151";
    pub const BORDER_STRONG: &str = "#4B5563";
    pub const BORDER_FOCUS: &str = "#3B82F6";
    pub const DIVIDER: &str = "#374151";

    // State colors (with alpha for overlay effects)
    pub const HOVER_OVERLAY: &str = "#FFFFFF0D"; // 5% white
    pub const ACTIVE_OVERLAY: &str = "#FFFFFF1A"; // 10% white
    pub const FOCUS_RING: &str = "#3B82F680"; // 50% primary
    pub const DISABLED_BG: &str = "#374151";
    pub const DISABLED_TEXT: &str = "#6B7280";

    // Transparent
    pub const TRANSPARENT: &str = "#00000000";

    // Backdrop overlay
    pub const BACKDROP: &str = "#00000080";
}

/// Alert variant colors
pub mod alert_colors {
    pub const INFO_BG: &str = "#1E3A5F";
    pub const INFO_BORDER: &str = "#3B82F6";
    pub const INFO_TEXT: &str = "#93C5FD";

    pub const SUCCESS_BG: &str = "#14532D";
    pub const SUCCESS_BORDER: &str = "#22C55E";
    pub const SUCCESS_TEXT: &str = "#86EFAC";

    pub const WARNING_BG: &str = "#713F12";
    pub const WARNING_BORDER: &str = "#F59E0B";
    pub const WARNING_TEXT: &str = "#FCD34D";

    pub const ERROR_BG: &str = "#7F1D1D";
    pub const ERROR_BORDER: &str = "#EF4444";
    pub const ERROR_TEXT: &str = "#FCA5A5";
}

// =============================================================================
// SPACING TOKENS
// =============================================================================

/// Spacing tokens in pixels
pub mod spacing {
    pub const NONE: f32 = 0.0;
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
    pub const XXL: f32 = 48.0;

    // Component-specific spacing
    pub const BUTTON_PADDING_X: f32 = 16.0;
    pub const BUTTON_PADDING_Y: f32 = 8.0;
    pub const INPUT_PADDING_X: f32 = 12.0;
    pub const INPUT_PADDING_Y: f32 = 8.0;
    pub const CARD_PADDING: f32 = 16.0;
    pub const MODAL_PADDING: f32 = 24.0;

    // Gap spacing
    pub const GAP_XS: f32 = 4.0;
    pub const GAP_SM: f32 = 8.0;
    pub const GAP_MD: f32 = 12.0;
    pub const GAP_LG: f32 = 16.0;
}

// =============================================================================
// RADIUS TOKENS
// =============================================================================

/// Border radius tokens in pixels
pub mod radius {
    pub const NONE: f32 = 0.0;
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
    pub const LG: f32 = 8.0;
    pub const XL: f32 = 12.0;
    pub const XXL: f32 = 16.0;
    pub const FULL: f32 = 9999.0;

    // Component-specific radii
    pub const BUTTON: f32 = 6.0;
    pub const INPUT: f32 = 8.0;
    pub const CARD: f32 = 12.0;
    pub const MODAL: f32 = 16.0;
    pub const BADGE: f32 = 9999.0; // Pill shape
    pub const AVATAR: f32 = 9999.0; // Circle
}

// =============================================================================
// TYPOGRAPHY TOKENS
// =============================================================================

/// Typography tokens
pub mod typography {
    // Font sizes in pixels
    pub const SIZE_XS: f32 = 10.0;
    pub const SIZE_SM: f32 = 12.0;
    pub const SIZE_BASE: f32 = 14.0;
    pub const SIZE_MD: f32 = 16.0;
    pub const SIZE_LG: f32 = 18.0;
    pub const SIZE_XL: f32 = 20.0;
    pub const SIZE_2XL: f32 = 24.0;
    pub const SIZE_3XL: f32 = 32.0;
    pub const SIZE_4XL: f32 = 48.0;

    // Line heights (multipliers)
    pub const LINE_HEIGHT_TIGHT: f32 = 1.25;
    pub const LINE_HEIGHT_NORMAL: f32 = 1.5;
    pub const LINE_HEIGHT_RELAXED: f32 = 1.75;

    // Font weights
    pub const WEIGHT_NORMAL: u16 = 400;
    pub const WEIGHT_MEDIUM: u16 = 500;
    pub const WEIGHT_SEMIBOLD: u16 = 600;
    pub const WEIGHT_BOLD: u16 = 700;
}

// =============================================================================
// SIZE TOKENS
// =============================================================================

/// Component size tokens
pub mod sizes {
    // Button heights
    pub const BUTTON_SM: f32 = 28.0;
    pub const BUTTON_MD: f32 = 36.0;
    pub const BUTTON_LG: f32 = 44.0;

    // Input heights
    pub const INPUT_SM: f32 = 32.0;
    pub const INPUT_MD: f32 = 40.0;
    pub const INPUT_LG: f32 = 48.0;

    // Icon sizes
    pub const ICON_XS: f32 = 12.0;
    pub const ICON_SM: f32 = 16.0;
    pub const ICON_MD: f32 = 20.0;
    pub const ICON_LG: f32 = 24.0;
    pub const ICON_XL: f32 = 32.0;

    // Avatar sizes
    pub const AVATAR_XS: f32 = 24.0;
    pub const AVATAR_SM: f32 = 32.0;
    pub const AVATAR_MD: f32 = 40.0;
    pub const AVATAR_LG: f32 = 48.0;
    pub const AVATAR_XL: f32 = 64.0;

    // Sidebar widths
    pub const SIDEBAR_EXPANDED: f32 = 240.0;
    pub const SIDEBAR_COLLAPSED: f32 = 64.0;

    // Topbar height
    pub const TOPBAR_HEIGHT: f32 = 64.0;

    // Table row heights
    pub const TABLE_ROW_COMPACT: f32 = 36.0;
    pub const TABLE_ROW_DEFAULT: f32 = 48.0;

    // Modal widths
    pub const MODAL_SM: f32 = 400.0;
    pub const MODAL_MD: f32 = 560.0;
    pub const MODAL_LG: f32 = 800.0;

    // Border widths
    pub const BORDER_DEFAULT: f32 = 1.0;
    pub const BORDER_THICK: f32 = 2.0;
    pub const FOCUS_RING_WIDTH: f32 = 2.0;
}

// =============================================================================
// MOTION TOKENS
// =============================================================================

/// Animation duration tokens in milliseconds
pub mod motion {
    pub const DURATION_INSTANT: u32 = 50;
    pub const DURATION_FAST: u32 = 150;
    pub const DURATION_NORMAL: u32 = 300;
    pub const DURATION_SLOW: u32 = 500;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Convert a hex color string to an RGBA array [r, g, b, a]
pub fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex)
        .map(|c| c.to_array())
        .unwrap_or([1.0, 1.0, 1.0, 1.0])
}

/// Apply alpha transparency to a color
pub fn with_alpha(hex: &str, alpha: f32) -> [f32; 4] {
    let mut rgba = hex_to_rgba(hex);
    rgba[3] = alpha;
    rgba
}

/// Lighten a color by a percentage (0.0 to 1.0)
pub fn lighten(hex: &str, amount: f32) -> [f32; 4] {
    let mut rgba = hex_to_rgba(hex);
    for i in 0..3 {
        rgba[i] = (rgba[i] + amount).min(1.0);
    }
    rgba
}

/// Darken a color by a percentage (0.0 to 1.0)
pub fn darken(hex: &str, amount: f32) -> [f32; 4] {
    let mut rgba = hex_to_rgba(hex);
    for i in 0..3 {
        rgba[i] = (rgba[i] - amount).max(0.0);
    }
    rgba
}

// =============================================================================
// COMPONENT STATE HELPERS
// =============================================================================

/// Represents component interaction states
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InteractionState {
    #[default]
    Default,
    Hover,
    Active,
    Focus,
    Disabled,
}

/// Returns colors for button variants with interaction states
pub fn button_colors(
    variant: &str,
    state: InteractionState,
) -> (Option<[f32; 4]>, Option<[f32; 4]>, [f32; 4]) {
    // Returns (background, border, text_color)
    let (bg, border, text) = match (variant, state) {
        // Primary button
        ("primary", InteractionState::Default) => (Some(colors::PRIMARY), None, colors::PRIMARY_CONTRAST),
        ("primary", InteractionState::Hover) => (Some(colors::PRIMARY_LIGHT), None, colors::PRIMARY_CONTRAST),
        ("primary", InteractionState::Active) => (Some(colors::PRIMARY_DARK), None, colors::PRIMARY_CONTRAST),
        ("primary", InteractionState::Disabled) => (Some(colors::DISABLED_BG), None, colors::DISABLED_TEXT),

        // Secondary button
        ("secondary", InteractionState::Default) => (Some(colors::SECONDARY_DARK), None, colors::TEXT_PRIMARY),
        ("secondary", InteractionState::Hover) => (Some(colors::SECONDARY), None, colors::TEXT_PRIMARY),
        ("secondary", InteractionState::Active) => (Some(colors::SECONDARY_DARK), None, colors::TEXT_PRIMARY),
        ("secondary", InteractionState::Disabled) => (Some(colors::DISABLED_BG), None, colors::DISABLED_TEXT),

        // Outline button
        ("outline", InteractionState::Default) => (None, Some(colors::BORDER), colors::TEXT_PRIMARY),
        ("outline", InteractionState::Hover) => (Some(colors::HOVER_OVERLAY), Some(colors::BORDER), colors::TEXT_PRIMARY),
        ("outline", InteractionState::Active) => (Some(colors::ACTIVE_OVERLAY), Some(colors::BORDER), colors::TEXT_PRIMARY),
        ("outline", InteractionState::Disabled) => (None, Some(colors::DISABLED_BG), colors::DISABLED_TEXT),

        // Ghost button
        ("ghost", InteractionState::Default) => (None, None, colors::TEXT_PRIMARY),
        ("ghost", InteractionState::Hover) => (Some(colors::HOVER_OVERLAY), None, colors::TEXT_PRIMARY),
        ("ghost", InteractionState::Active) => (Some(colors::ACTIVE_OVERLAY), None, colors::TEXT_PRIMARY),
        ("ghost", InteractionState::Disabled) => (None, None, colors::DISABLED_TEXT),

        // Danger button
        ("danger", InteractionState::Default) => (Some(colors::DANGER), None, colors::PRIMARY_CONTRAST),
        ("danger", InteractionState::Hover) => (Some(colors::DANGER_LIGHT), None, colors::PRIMARY_CONTRAST),
        ("danger", InteractionState::Active) => (Some(colors::DANGER_DARK), None, colors::PRIMARY_CONTRAST),
        ("danger", InteractionState::Disabled) => (Some(colors::DISABLED_BG), None, colors::DISABLED_TEXT),

        // Success button
        ("success", InteractionState::Default) => (Some(colors::SUCCESS), None, colors::PRIMARY_CONTRAST),
        ("success", InteractionState::Hover) => (Some(colors::SUCCESS_LIGHT), None, colors::PRIMARY_CONTRAST),
        ("success", InteractionState::Active) => (Some(colors::SUCCESS_DARK), None, colors::PRIMARY_CONTRAST),
        ("success", InteractionState::Disabled) => (Some(colors::DISABLED_BG), None, colors::DISABLED_TEXT),

        // Link button
        ("link", InteractionState::Default) => (None, None, colors::TEXT_LINK),
        ("link", InteractionState::Hover) => (None, None, colors::PRIMARY_LIGHT),
        ("link", InteractionState::Active) => (None, None, colors::PRIMARY_DARK),
        ("link", InteractionState::Disabled) => (None, None, colors::DISABLED_TEXT),

        // Focus state adds ring, doesn't change base colors
        (_, InteractionState::Focus) => return button_colors(variant, InteractionState::Default),

        // Default fallback
        _ => (Some(colors::PRIMARY), None, colors::PRIMARY_CONTRAST),
    };

    (
        bg.map(hex_to_rgba),
        border.map(hex_to_rgba),
        hex_to_rgba(text),
    )
}

/// Returns colors for input fields with interaction states
pub fn input_colors(
    has_error: bool,
    state: InteractionState,
) -> ([f32; 4], [f32; 4], [f32; 4]) {
    // Returns (background, border, text_color)
    match (has_error, state) {
        (true, _) => (
            hex_to_rgba(colors::SURFACE_ELEVATED),
            hex_to_rgba(colors::DANGER),
            hex_to_rgba(colors::TEXT_PRIMARY),
        ),
        (false, InteractionState::Default) => (
            hex_to_rgba(colors::SURFACE_ELEVATED),
            hex_to_rgba(colors::BORDER),
            hex_to_rgba(colors::TEXT_PRIMARY),
        ),
        (false, InteractionState::Hover) => (
            hex_to_rgba(colors::SURFACE_ELEVATED),
            hex_to_rgba(colors::BORDER_STRONG),
            hex_to_rgba(colors::TEXT_PRIMARY),
        ),
        (false, InteractionState::Focus) => (
            hex_to_rgba(colors::SURFACE_ELEVATED),
            hex_to_rgba(colors::BORDER_FOCUS),
            hex_to_rgba(colors::TEXT_PRIMARY),
        ),
        (false, InteractionState::Disabled) => (
            hex_to_rgba(colors::SURFACE),
            hex_to_rgba(colors::SECONDARY_DARK),
            hex_to_rgba(colors::DISABLED_TEXT),
        ),
        _ => (
            hex_to_rgba(colors::SURFACE_ELEVATED),
            hex_to_rgba(colors::BORDER),
            hex_to_rgba(colors::TEXT_PRIMARY),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgba() {
        let rgba = hex_to_rgba("#FFFFFF");
        assert!((rgba[0] - 1.0).abs() < 0.01);
        assert!((rgba[1] - 1.0).abs() < 0.01);
        assert!((rgba[2] - 1.0).abs() < 0.01);
        assert!((rgba[3] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_with_alpha() {
        let rgba = with_alpha("#FFFFFF", 0.5);
        assert!((rgba[3] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_button_colors_primary() {
        let (bg, border, text) = button_colors("primary", InteractionState::Default);
        assert!(bg.is_some());
        assert!(border.is_none());
        assert!((text[0] - 1.0).abs() < 0.01); // White text
    }

    #[test]
    fn test_button_colors_outline() {
        let (bg, border, _text) = button_colors("outline", InteractionState::Default);
        assert!(bg.is_none());
        assert!(border.is_some());
    }
}
