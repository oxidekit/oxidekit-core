//! Icon Component
//!
//! The core icon component for rendering icons in OxideKit applications.

use crate::{IconData, IconRegistry, IconSet, PathCommand, SvgPath};
use oxide_render::Color;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Icon size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IconSize {
    /// Extra small: 12px
    ExtraSmall,
    /// Small: 16px
    Small,
    /// Medium: 24px (default)
    #[default]
    Medium,
    /// Large: 32px
    Large,
    /// Extra large: 48px
    ExtraLarge,
    /// Custom size in pixels
    Custom(u32),
}

impl IconSize {
    /// Get the size in pixels
    pub fn pixels(&self) -> u32 {
        match self {
            IconSize::ExtraSmall => 12,
            IconSize::Small => 16,
            IconSize::Medium => 24,
            IconSize::Large => 32,
            IconSize::ExtraLarge => 48,
            IconSize::Custom(px) => *px,
        }
    }

    /// Create an icon size from pixels
    pub fn from_pixels(px: u32) -> Self {
        match px {
            12 => IconSize::ExtraSmall,
            16 => IconSize::Small,
            24 => IconSize::Medium,
            32 => IconSize::Large,
            48 => IconSize::ExtraLarge,
            _ => IconSize::Custom(px),
        }
    }
}

/// Flip direction for icons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IconFlip {
    /// No flip
    #[default]
    None,
    /// Flip horizontally
    Horizontal,
    /// Flip vertically
    Vertical,
    /// Flip both horizontally and vertically
    Both,
}

impl IconFlip {
    /// Get the scale factors for rendering
    pub fn scale_factors(&self) -> (f32, f32) {
        match self {
            IconFlip::None => (1.0, 1.0),
            IconFlip::Horizontal => (-1.0, 1.0),
            IconFlip::Vertical => (1.0, -1.0),
            IconFlip::Both => (-1.0, -1.0),
        }
    }
}

/// Icon weight/thickness variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IconWeight {
    /// Thin stroke width
    Thin,
    /// Light stroke width
    Light,
    /// Regular stroke width (default)
    #[default]
    Regular,
    /// Bold stroke width
    Bold,
    /// Fill variant (solid)
    Fill,
}

impl IconWeight {
    /// Get the stroke width multiplier
    pub fn stroke_multiplier(&self) -> f32 {
        match self {
            IconWeight::Thin => 1.0,
            IconWeight::Light => 1.5,
            IconWeight::Regular => 2.0,
            IconWeight::Bold => 2.5,
            IconWeight::Fill => 0.0, // Filled, no stroke
        }
    }
}

/// An icon component for rendering icons
#[derive(Debug, Clone)]
pub struct Icon {
    /// The icon name
    name: String,
    /// The icon set (optional, for disambiguation)
    set: Option<IconSet>,
    /// Icon size
    size: IconSize,
    /// Icon color
    color: Option<Color>,
    /// Rotation in degrees
    rotation: f32,
    /// Flip direction
    flip: IconFlip,
    /// Icon weight
    weight: IconWeight,
    /// Opacity (0.0 - 1.0)
    opacity: f32,
    /// Optional badge content
    badge: Option<IconBadge>,
    /// Custom SVG data (for inline icons)
    custom_data: Option<IconData>,
}

impl Icon {
    /// Create a new icon by name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            set: None,
            size: IconSize::default(),
            color: None,
            rotation: 0.0,
            flip: IconFlip::default(),
            weight: IconWeight::default(),
            opacity: 1.0,
            badge: None,
            custom_data: None,
        }
    }

    /// Create an icon from the Material Design icon set
    pub fn material(name: impl Into<String>) -> Self {
        Self::new(name).set(IconSet::Material)
    }

    /// Create an icon from the Lucide (Feather) icon set
    pub fn lucide(name: impl Into<String>) -> Self {
        Self::new(name).set(IconSet::Lucide)
    }

    /// Create an icon from the Heroicons set
    pub fn heroicons(name: impl Into<String>) -> Self {
        Self::new(name).set(IconSet::Heroicons)
    }

    /// Create an icon from the Phosphor icon set
    pub fn phosphor(name: impl Into<String>) -> Self {
        Self::new(name).set(IconSet::Phosphor)
    }

    /// Create a custom icon from SVG data
    pub fn custom(name: impl Into<String>, svg: &str) -> Self {
        let name = name.into();
        let data = IconData::from_svg(svg).unwrap_or_else(|| IconData {
            name: name.clone(),
            viewbox: (0, 0, 24, 24),
            paths: vec![],
            categories: vec!["custom".to_string()],
            tags: vec![],
        });
        Self {
            name,
            set: Some(IconSet::Custom),
            size: IconSize::default(),
            color: None,
            rotation: 0.0,
            flip: IconFlip::default(),
            weight: IconWeight::default(),
            opacity: 1.0,
            badge: None,
            custom_data: Some(data),
        }
    }

    /// Set the icon set
    pub fn set(mut self, set: IconSet) -> Self {
        self.set = Some(set);
        self
    }

    /// Set the icon size
    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Set the icon size in pixels
    pub fn size_px(mut self, px: u32) -> Self {
        self.size = IconSize::Custom(px);
        self
    }

    /// Set the icon color
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the icon color from a hex string
    pub fn color_hex(mut self, hex: &str) -> Self {
        if let Some(color) = Color::from_hex(hex) {
            self.color = Some(color);
        }
        self
    }

    /// Set the rotation in degrees
    pub fn rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Rotate 90 degrees clockwise
    pub fn rotate_90(self) -> Self {
        self.rotation(90.0)
    }

    /// Rotate 180 degrees
    pub fn rotate_180(self) -> Self {
        self.rotation(180.0)
    }

    /// Rotate 270 degrees (90 counter-clockwise)
    pub fn rotate_270(self) -> Self {
        self.rotation(270.0)
    }

    /// Set the flip direction
    pub fn flip(mut self, flip: IconFlip) -> Self {
        self.flip = flip;
        self
    }

    /// Flip horizontally
    pub fn flip_horizontal(self) -> Self {
        self.flip(IconFlip::Horizontal)
    }

    /// Flip vertically
    pub fn flip_vertical(self) -> Self {
        self.flip(IconFlip::Vertical)
    }

    /// Set the icon weight
    pub fn weight(mut self, weight: IconWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set opacity (0.0 - 1.0)
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Add a badge to the icon
    pub fn badge(mut self, badge: IconBadge) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Add a notification dot badge
    pub fn with_dot(self) -> Self {
        self.badge(IconBadge::Dot {
            color: Color::from_hex("#EF4444").unwrap_or(Color::OXIDE_ACCENT),
        })
    }

    /// Add a count badge
    pub fn with_count(self, count: u32) -> Self {
        self.badge(IconBadge::Count {
            value: count,
            background: Color::from_hex("#EF4444").unwrap_or(Color::OXIDE_ACCENT),
            text_color: Color::WHITE,
        })
    }

    /// Get the icon name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the icon set
    pub fn icon_set(&self) -> Option<&IconSet> {
        self.set.as_ref()
    }

    /// Get the icon size
    pub fn icon_size(&self) -> &IconSize {
        &self.size
    }

    /// Get the icon color
    pub fn icon_color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    /// Get the rotation in degrees
    pub fn icon_rotation(&self) -> f32 {
        self.rotation
    }

    /// Get the flip direction
    pub fn icon_flip(&self) -> &IconFlip {
        &self.flip
    }

    /// Get the icon weight
    pub fn icon_weight(&self) -> &IconWeight {
        &self.weight
    }

    /// Get the opacity
    pub fn icon_opacity(&self) -> f32 {
        self.opacity
    }

    /// Get the badge
    pub fn icon_badge(&self) -> Option<&IconBadge> {
        self.badge.as_ref()
    }

    /// Get the custom icon data
    pub fn custom_icon_data(&self) -> Option<&IconData> {
        self.custom_data.as_ref()
    }

    /// Resolve the icon data from the registry
    pub fn resolve(&self) -> Option<IconData> {
        // Return custom data if available
        if let Some(data) = &self.custom_data {
            return Some(data.clone());
        }

        // Look up in registry
        let registry = IconRegistry::global();

        if let Some(set) = &self.set {
            registry.get_from_set(set, &self.name)
        } else {
            registry.get(&self.name)
        }
    }

    /// Get the render data for this icon
    pub fn render_data(&self) -> Option<IconRenderData> {
        let data = self.resolve()?;
        let size = self.size.pixels() as f32;
        let (vb_x, vb_y, vb_w, vb_h) = data.viewbox;
        let scale = size / vb_w as f32;

        // Calculate transform
        let rotation_rad = self.rotation * PI / 180.0;
        let (flip_x, flip_y) = self.flip.scale_factors();

        let color = self.color.unwrap_or(Color::OXIDE_TEXT);
        let color_with_opacity = Color::new(
            color.r,
            color.g,
            color.b,
            color.a * self.opacity,
        );

        Some(IconRenderData {
            paths: data.paths,
            viewbox: (vb_x, vb_y, vb_w, vb_h),
            size,
            scale,
            rotation: rotation_rad,
            flip: (flip_x, flip_y),
            color: color_with_opacity,
            stroke_width: self.weight.stroke_multiplier() * scale,
            filled: matches!(self.weight, IconWeight::Fill),
            badge: self.badge.clone(),
        })
    }
}

impl Default for Icon {
    fn default() -> Self {
        Self::new("circle")
    }
}

/// Badge displayed on an icon
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IconBadge {
    /// A simple dot indicator
    Dot { color: Color },
    /// A count badge
    Count {
        value: u32,
        background: Color,
        text_color: Color,
    },
    /// Custom badge content
    Custom {
        /// Badge background color
        background: Color,
        /// Badge content (text)
        content: String,
        /// Text color
        text_color: Color,
    },
}

impl IconBadge {
    /// Create a dot badge with the given color
    pub fn dot(color: Color) -> Self {
        Self::Dot { color }
    }

    /// Create a count badge
    pub fn count(value: u32) -> Self {
        Self::Count {
            value,
            background: Color::from_hex("#EF4444").unwrap_or(Color::OXIDE_ACCENT),
            text_color: Color::WHITE,
        }
    }

    /// Create a custom text badge
    pub fn custom(content: impl Into<String>, background: Color, text_color: Color) -> Self {
        Self::Custom {
            content: content.into(),
            background,
            text_color,
        }
    }

    /// Get the display text for the badge
    pub fn display_text(&self) -> Option<String> {
        match self {
            IconBadge::Dot { .. } => None,
            IconBadge::Count { value, .. } => {
                if *value > 99 {
                    Some("99+".to_string())
                } else {
                    Some(value.to_string())
                }
            }
            IconBadge::Custom { content, .. } => Some(content.clone()),
        }
    }
}

/// Render data for an icon, ready for GPU rendering
#[derive(Debug, Clone)]
pub struct IconRenderData {
    /// SVG paths
    pub paths: Vec<SvgPath>,
    /// Viewbox (x, y, width, height)
    pub viewbox: (i32, i32, u32, u32),
    /// Target size in pixels
    pub size: f32,
    /// Scale factor from viewbox to target size
    pub scale: f32,
    /// Rotation in radians
    pub rotation: f32,
    /// Flip factors (x, y)
    pub flip: (f32, f32),
    /// Icon color with opacity applied
    pub color: Color,
    /// Stroke width (0 for filled icons)
    pub stroke_width: f32,
    /// Whether the icon is filled
    pub filled: bool,
    /// Optional badge
    pub badge: Option<IconBadge>,
}

impl IconRenderData {
    /// Convert paths to a list of path commands for GPU rendering
    pub fn to_path_commands(&self) -> Vec<Vec<PathCommand>> {
        self.paths
            .iter()
            .map(|p| p.commands.clone())
            .collect()
    }

    /// Get the bounding box for this icon
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        (0.0, 0.0, self.size, self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_size_pixels() {
        assert_eq!(IconSize::ExtraSmall.pixels(), 12);
        assert_eq!(IconSize::Small.pixels(), 16);
        assert_eq!(IconSize::Medium.pixels(), 24);
        assert_eq!(IconSize::Large.pixels(), 32);
        assert_eq!(IconSize::ExtraLarge.pixels(), 48);
        assert_eq!(IconSize::Custom(64).pixels(), 64);
    }

    #[test]
    fn test_icon_size_from_pixels() {
        assert_eq!(IconSize::from_pixels(12), IconSize::ExtraSmall);
        assert_eq!(IconSize::from_pixels(16), IconSize::Small);
        assert_eq!(IconSize::from_pixels(24), IconSize::Medium);
        assert_eq!(IconSize::from_pixels(32), IconSize::Large);
        assert_eq!(IconSize::from_pixels(48), IconSize::ExtraLarge);
        assert_eq!(IconSize::from_pixels(64), IconSize::Custom(64));
    }

    #[test]
    fn test_icon_flip_scale_factors() {
        assert_eq!(IconFlip::None.scale_factors(), (1.0, 1.0));
        assert_eq!(IconFlip::Horizontal.scale_factors(), (-1.0, 1.0));
        assert_eq!(IconFlip::Vertical.scale_factors(), (1.0, -1.0));
        assert_eq!(IconFlip::Both.scale_factors(), (-1.0, -1.0));
    }

    #[test]
    fn test_icon_builder_pattern() {
        let icon = Icon::new("home")
            .size(IconSize::Large)
            .color(Color::OXIDE_ACCENT)
            .rotation(45.0)
            .flip(IconFlip::Horizontal)
            .weight(IconWeight::Bold)
            .opacity(0.8);

        assert_eq!(icon.name(), "home");
        assert_eq!(*icon.icon_size(), IconSize::Large);
        assert_eq!(icon.icon_rotation(), 45.0);
        assert_eq!(*icon.icon_flip(), IconFlip::Horizontal);
        assert_eq!(*icon.icon_weight(), IconWeight::Bold);
        assert!((icon.icon_opacity() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_icon_set_constructors() {
        let material = Icon::material("settings");
        assert_eq!(material.icon_set(), Some(&IconSet::Material));

        let lucide = Icon::lucide("user");
        assert_eq!(lucide.icon_set(), Some(&IconSet::Lucide));

        let heroicons = Icon::heroicons("arrow-right");
        assert_eq!(heroicons.icon_set(), Some(&IconSet::Heroicons));

        let phosphor = Icon::phosphor("house");
        assert_eq!(phosphor.icon_set(), Some(&IconSet::Phosphor));
    }

    #[test]
    fn test_icon_badge() {
        let icon = Icon::new("bell").with_dot();
        assert!(icon.icon_badge().is_some());

        let icon = Icon::new("mail").with_count(5);
        if let Some(IconBadge::Count { value, .. }) = icon.icon_badge() {
            assert_eq!(*value, 5);
        } else {
            panic!("Expected Count badge");
        }
    }

    #[test]
    fn test_badge_display_text() {
        assert_eq!(IconBadge::dot(Color::WHITE).display_text(), None);
        assert_eq!(IconBadge::count(5).display_text(), Some("5".to_string()));
        assert_eq!(IconBadge::count(99).display_text(), Some("99".to_string()));
        assert_eq!(IconBadge::count(100).display_text(), Some("99+".to_string()));
    }

    #[test]
    fn test_icon_rotation_helpers() {
        let icon = Icon::new("arrow").rotate_90();
        assert!((icon.icon_rotation() - 90.0).abs() < 0.001);

        let icon = Icon::new("arrow").rotate_180();
        assert!((icon.icon_rotation() - 180.0).abs() < 0.001);

        let icon = Icon::new("arrow").rotate_270();
        assert!((icon.icon_rotation() - 270.0).abs() < 0.001);
    }

    #[test]
    fn test_icon_color_hex() {
        let icon = Icon::new("star").color_hex("#FF5500");
        let color = icon.icon_color().unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.333).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }
}
