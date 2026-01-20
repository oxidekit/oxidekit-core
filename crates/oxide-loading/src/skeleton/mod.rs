//! Skeleton Loaders
//!
//! Placeholder components that show while content is loading:
//! - Text skeletons for paragraphs and lines
//! - Avatar skeletons for profile images
//! - Image skeletons for media content
//! - Card skeletons for card layouts
//! - Table skeletons for tabular data
//!
//! # Animation Effects
//!
//! All skeleton components support shimmer and wave animations for visual feedback.
//!
//! # Accessibility
//!
//! Skeletons include `aria-busy="true"` and `aria-label` to indicate loading state.

mod avatar;
mod card;
mod image;
mod table;
mod text;

pub use avatar::*;
pub use card::*;
pub use image::*;
pub use table::*;
pub use text::*;

use serde::{Deserialize, Serialize};

/// Animation style for skeleton loaders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SkeletonAnimation {
    /// No animation (static)
    None,
    /// Shimmer effect (gradient moving across)
    #[default]
    Shimmer,
    /// Wave/pulse effect (opacity fading)
    Wave,
}

impl SkeletonAnimation {
    /// Get animation duration in seconds
    pub fn duration(&self) -> f32 {
        match self {
            SkeletonAnimation::None => 0.0,
            SkeletonAnimation::Shimmer => 1.5,
            SkeletonAnimation::Wave => 2.0,
        }
    }

    /// Check if this animation is active
    pub fn is_animated(&self) -> bool {
        !matches!(self, SkeletonAnimation::None)
    }
}

/// Base color configuration for skeletons
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SkeletonColors {
    /// Base background color (r, g, b, a)
    pub base: (u8, u8, u8, u8),
    /// Highlight color for shimmer (r, g, b, a)
    pub highlight: (u8, u8, u8, u8),
}

impl Default for SkeletonColors {
    fn default() -> Self {
        Self {
            base: (224, 224, 224, 255),      // Light gray
            highlight: (240, 240, 240, 255), // Lighter gray
        }
    }
}

impl SkeletonColors {
    /// Create colors for dark mode
    pub fn dark() -> Self {
        Self {
            base: (66, 66, 66, 255),    // Dark gray
            highlight: (88, 88, 88, 255), // Slightly lighter
        }
    }

    /// Create custom colors
    pub fn custom(base: (u8, u8, u8, u8), highlight: (u8, u8, u8, u8)) -> Self {
        Self { base, highlight }
    }
}

/// Shared skeleton configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonConfig {
    /// Animation style
    pub animation: SkeletonAnimation,
    /// Color scheme
    pub colors: SkeletonColors,
    /// Border radius in pixels
    pub border_radius: f32,
    /// Current animation time (0.0 to 1.0)
    animation_time: f32,
}

impl Default for SkeletonConfig {
    fn default() -> Self {
        Self {
            animation: SkeletonAnimation::default(),
            colors: SkeletonColors::default(),
            border_radius: 4.0,
            animation_time: 0.0,
        }
    }
}

impl SkeletonConfig {
    /// Create a new skeleton configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the animation style
    pub fn animation(mut self, animation: SkeletonAnimation) -> Self {
        self.animation = animation;
        self
    }

    /// Set the colors
    pub fn colors(mut self, colors: SkeletonColors) -> Self {
        self.colors = colors;
        self
    }

    /// Set the border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius.max(0.0);
        self
    }

    /// Update animation state (call each frame)
    ///
    /// # Returns
    /// `true` if animation is active and needs redraws
    pub fn update(&mut self, delta_time: f32) -> bool {
        if !self.animation.is_animated() {
            return false;
        }

        self.animation_time += delta_time / self.animation.duration();
        if self.animation_time >= 1.0 {
            self.animation_time -= 1.0;
        }
        true
    }

    /// Get the current animation progress (0.0 to 1.0)
    pub fn animation_progress(&self) -> f32 {
        self.animation_time
    }

    /// Calculate shimmer gradient position
    pub fn shimmer_position(&self) -> f32 {
        // Returns position from -0.5 to 1.5 (gradient wider than element)
        self.animation_time * 2.0 - 0.5
    }

    /// Calculate wave opacity multiplier
    pub fn wave_opacity(&self) -> f32 {
        // Sine wave between 0.5 and 1.0
        let t = self.animation_time * std::f32::consts::TAU;
        0.75 + 0.25 * t.sin()
    }

    /// Get ARIA attributes for accessibility
    pub fn aria_attributes(&self, label: &str) -> Vec<(&'static str, String)> {
        vec![
            ("role", "presentation".to_string()),
            ("aria-busy", "true".to_string()),
            ("aria-label", format!("{} loading", label)),
        ]
    }
}

/// Common render parameters for skeleton shapes
#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonRenderParams {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
    /// Border radius
    pub border_radius: f32,
    /// Base color (r, g, b, a)
    pub base_color: (u8, u8, u8, u8),
    /// Animation type
    pub animation: SkeletonAnimation,
    /// Shimmer gradient position (0.0 to 1.0)
    pub shimmer_position: f32,
    /// Wave opacity multiplier (0.0 to 1.0)
    pub wave_opacity: f32,
    /// Highlight color for shimmer (r, g, b, a)
    pub highlight_color: (u8, u8, u8, u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_animation_duration() {
        assert!((SkeletonAnimation::None.duration() - 0.0).abs() < f32::EPSILON);
        assert!((SkeletonAnimation::Shimmer.duration() - 1.5).abs() < f32::EPSILON);
        assert!((SkeletonAnimation::Wave.duration() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_animation_is_animated() {
        assert!(!SkeletonAnimation::None.is_animated());
        assert!(SkeletonAnimation::Shimmer.is_animated());
        assert!(SkeletonAnimation::Wave.is_animated());
    }

    #[test]
    fn test_skeleton_colors_default() {
        let colors = SkeletonColors::default();
        assert_eq!(colors.base, (224, 224, 224, 255));
    }

    #[test]
    fn test_skeleton_colors_dark() {
        let colors = SkeletonColors::dark();
        assert_eq!(colors.base, (66, 66, 66, 255));
    }

    #[test]
    fn test_skeleton_config_update() {
        let mut config = SkeletonConfig::new().animation(SkeletonAnimation::Shimmer);

        assert!(config.update(0.1));
        assert!(config.animation_progress() > 0.0);
    }

    #[test]
    fn test_skeleton_config_no_animation() {
        let mut config = SkeletonConfig::new().animation(SkeletonAnimation::None);

        assert!(!config.update(0.1));
    }

    #[test]
    fn test_skeleton_shimmer_position() {
        let mut config = SkeletonConfig::new();
        config.animation_time = 0.0;
        assert!((config.shimmer_position() - (-0.5)).abs() < f32::EPSILON);

        config.animation_time = 1.0;
        assert!((config.shimmer_position() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_wave_opacity() {
        let mut config = SkeletonConfig::new();
        config.animation_time = 0.0;
        let opacity = config.wave_opacity();
        assert!(opacity >= 0.5 && opacity <= 1.0);
    }

    #[test]
    fn test_skeleton_aria_attributes() {
        let config = SkeletonConfig::new();
        let attrs = config.aria_attributes("Content");
        assert!(attrs.contains(&("aria-busy", "true".to_string())));
        assert!(attrs.contains(&("aria-label", "Content loading".to_string())));
    }
}
