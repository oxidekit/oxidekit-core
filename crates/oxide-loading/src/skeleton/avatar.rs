//! Skeleton Avatar Loader
//!
//! Circular placeholder for profile images and avatars.
//!
//! # Example
//!
//! ```
//! use oxide_loading::skeleton::{SkeletonAvatar, SkeletonAnimation};
//!
//! // Default avatar
//! let avatar = SkeletonAvatar::new();
//!
//! // Large avatar with wave animation
//! let large = SkeletonAvatar::new()
//!     .size(64.0)
//!     .animation(SkeletonAnimation::Wave);
//! ```

use serde::{Deserialize, Serialize};

use super::{SkeletonAnimation, SkeletonConfig, SkeletonRenderParams};

/// Avatar size presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AvatarSize {
    /// Extra small (24px)
    ExtraSmall,
    /// Small (32px)
    Small,
    /// Medium (40px)
    #[default]
    Medium,
    /// Large (48px)
    Large,
    /// Extra large (64px)
    ExtraLarge,
}

impl AvatarSize {
    /// Get the size in pixels
    pub fn to_pixels(&self) -> f32 {
        match self {
            AvatarSize::ExtraSmall => 24.0,
            AvatarSize::Small => 32.0,
            AvatarSize::Medium => 40.0,
            AvatarSize::Large => 48.0,
            AvatarSize::ExtraLarge => 64.0,
        }
    }
}

/// Skeleton placeholder for avatar/profile images
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonAvatar {
    /// Diameter in pixels
    pub size: f32,
    /// Configuration
    pub config: SkeletonConfig,
}

impl Default for SkeletonAvatar {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonAvatar {
    /// Create a new avatar skeleton with default size
    pub fn new() -> Self {
        Self {
            size: AvatarSize::default().to_pixels(),
            config: SkeletonConfig::default(),
        }
    }

    /// Create with preset size
    pub fn preset(preset: AvatarSize) -> Self {
        Self {
            size: preset.to_pixels(),
            config: SkeletonConfig::default(),
        }
    }

    /// Set custom size in pixels
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(8.0);
        self
    }

    /// Set the animation style
    pub fn animation(mut self, animation: SkeletonAnimation) -> Self {
        self.config = self.config.animation(animation);
        self
    }

    /// Update animation (call each frame)
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.config.update(delta_time)
    }

    /// Get render parameters (circle represented as rectangle with full border radius)
    pub fn render_params(&self, x: f32, y: f32) -> SkeletonRenderParams {
        SkeletonRenderParams {
            x,
            y,
            width: self.size,
            height: self.size,
            border_radius: self.size / 2.0, // Full circle
            base_color: self.config.colors.base,
            animation: self.config.animation,
            shimmer_position: self.config.shimmer_position(),
            wave_opacity: self.config.wave_opacity(),
            highlight_color: self.config.colors.highlight,
        }
    }

    /// Get ARIA attributes
    pub fn aria_attributes(&self) -> Vec<(&'static str, String)> {
        self.config.aria_attributes("Avatar")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_size_presets() {
        assert!((AvatarSize::ExtraSmall.to_pixels() - 24.0).abs() < f32::EPSILON);
        assert!((AvatarSize::Small.to_pixels() - 32.0).abs() < f32::EPSILON);
        assert!((AvatarSize::Medium.to_pixels() - 40.0).abs() < f32::EPSILON);
        assert!((AvatarSize::Large.to_pixels() - 48.0).abs() < f32::EPSILON);
        assert!((AvatarSize::ExtraLarge.to_pixels() - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_avatar_default() {
        let avatar = SkeletonAvatar::new();
        assert!((avatar.size - 40.0).abs() < f32::EPSILON); // Medium
    }

    #[test]
    fn test_skeleton_avatar_preset() {
        let avatar = SkeletonAvatar::preset(AvatarSize::Large);
        assert!((avatar.size - 48.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_avatar_custom_size() {
        let avatar = SkeletonAvatar::new().size(100.0);
        assert!((avatar.size - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_avatar_size_minimum() {
        let avatar = SkeletonAvatar::new().size(2.0);
        assert!((avatar.size - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_avatar_render_params_circular() {
        let avatar = SkeletonAvatar::new().size(50.0);
        let params = avatar.render_params(10.0, 20.0);

        // Border radius should be half the size for a circle
        assert!((params.border_radius - 25.0).abs() < f32::EPSILON);
        assert!((params.width - 50.0).abs() < f32::EPSILON);
        assert!((params.height - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_avatar_animation() {
        let avatar = SkeletonAvatar::new().animation(SkeletonAnimation::Wave);
        assert_eq!(avatar.config.animation, SkeletonAnimation::Wave);
    }
}
