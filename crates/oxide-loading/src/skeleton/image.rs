//! Skeleton Image Loader
//!
//! Rectangular placeholder for images and media content.
//!
//! # Example
//!
//! ```
//! use oxide_loading::skeleton::{SkeletonImage, SkeletonAnimation};
//!
//! // Fixed size image placeholder
//! let image = SkeletonImage::new(300.0, 200.0);
//!
//! // Square thumbnail
//! let thumb = SkeletonImage::square(150.0)
//!     .animation(SkeletonAnimation::Shimmer);
//!
//! // Aspect ratio based
//! let video = SkeletonImage::aspect_ratio(16.0 / 9.0, 400.0);
//! ```

use serde::{Deserialize, Serialize};

use super::{SkeletonAnimation, SkeletonConfig, SkeletonRenderParams};

/// Common aspect ratios
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AspectRatio {
    /// 1:1 square
    Square,
    /// 4:3 standard
    Standard,
    /// 16:9 widescreen
    Widescreen,
    /// 21:9 ultrawide
    Ultrawide,
    /// 3:4 portrait
    Portrait,
    /// 9:16 vertical video
    VerticalVideo,
    /// Custom ratio (width / height)
    Custom(f32),
}

impl AspectRatio {
    /// Get the ratio as a float (width / height)
    pub fn to_ratio(&self) -> f32 {
        match self {
            AspectRatio::Square => 1.0,
            AspectRatio::Standard => 4.0 / 3.0,
            AspectRatio::Widescreen => 16.0 / 9.0,
            AspectRatio::Ultrawide => 21.0 / 9.0,
            AspectRatio::Portrait => 3.0 / 4.0,
            AspectRatio::VerticalVideo => 9.0 / 16.0,
            AspectRatio::Custom(r) => *r,
        }
    }

    /// Calculate height from width
    pub fn height_for_width(&self, width: f32) -> f32 {
        width / self.to_ratio()
    }

    /// Calculate width from height
    pub fn width_for_height(&self, height: f32) -> f32 {
        height * self.to_ratio()
    }
}

/// Skeleton placeholder for images
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonImage {
    /// Width in pixels
    pub width: f32,
    /// Height in pixels
    pub height: f32,
    /// Whether to show an icon placeholder
    pub show_icon: bool,
    /// Configuration
    pub config: SkeletonConfig,
}

impl Default for SkeletonImage {
    fn default() -> Self {
        Self::new(200.0, 150.0)
    }
}

impl SkeletonImage {
    /// Create an image skeleton with specific dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: width.max(10.0),
            height: height.max(10.0),
            show_icon: true,
            config: SkeletonConfig::default().border_radius(8.0),
        }
    }

    /// Create a square image skeleton
    pub fn square(size: f32) -> Self {
        Self::new(size, size)
    }

    /// Create an image skeleton with aspect ratio and width
    pub fn aspect_ratio(ratio: impl Into<AspectRatioValue>, width: f32) -> Self {
        let r = ratio.into().to_ratio();
        Self::new(width, width / r)
    }

    /// Set the width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(10.0);
        self
    }

    /// Set the height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(10.0);
        self
    }

    /// Set whether to show a placeholder icon
    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    /// Set the animation style
    pub fn animation(mut self, animation: SkeletonAnimation) -> Self {
        self.config = self.config.animation(animation);
        self
    }

    /// Set the border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.config = self.config.border_radius(radius);
        self
    }

    /// Update animation (call each frame)
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.config.update(delta_time)
    }

    /// Get render parameters
    pub fn render_params(&self, x: f32, y: f32) -> SkeletonImageRenderParams {
        SkeletonImageRenderParams {
            base: SkeletonRenderParams {
                x,
                y,
                width: self.width,
                height: self.height,
                border_radius: self.config.border_radius,
                base_color: self.config.colors.base,
                animation: self.config.animation,
                shimmer_position: self.config.shimmer_position(),
                wave_opacity: self.config.wave_opacity(),
                highlight_color: self.config.colors.highlight,
            },
            show_icon: self.show_icon,
            icon_size: (self.width.min(self.height) * 0.3).min(48.0),
        }
    }

    /// Get ARIA attributes
    pub fn aria_attributes(&self) -> Vec<(&'static str, String)> {
        self.config.aria_attributes("Image")
    }
}

/// Render parameters for skeleton image
#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonImageRenderParams {
    /// Base skeleton parameters
    pub base: SkeletonRenderParams,
    /// Whether to show icon
    pub show_icon: bool,
    /// Icon size (for image placeholder icon)
    pub icon_size: f32,
}

/// Helper type for aspect ratio conversion
#[derive(Debug, Clone, Copy)]
pub struct AspectRatioValue(f32);

impl AspectRatioValue {
    pub fn to_ratio(&self) -> f32 {
        self.0
    }
}

impl From<AspectRatio> for AspectRatioValue {
    fn from(r: AspectRatio) -> Self {
        AspectRatioValue(r.to_ratio())
    }
}

impl From<f32> for AspectRatioValue {
    fn from(r: f32) -> Self {
        AspectRatioValue(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_ratio_to_ratio() {
        assert!((AspectRatio::Square.to_ratio() - 1.0).abs() < f32::EPSILON);
        assert!((AspectRatio::Widescreen.to_ratio() - 16.0 / 9.0).abs() < 0.001);
        assert!((AspectRatio::Custom(2.0).to_ratio() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_aspect_ratio_calculations() {
        let ratio = AspectRatio::Widescreen;
        let height = ratio.height_for_width(1920.0);
        assert!((height - 1080.0).abs() < 1.0);

        let width = ratio.width_for_height(1080.0);
        assert!((width - 1920.0).abs() < 1.0);
    }

    #[test]
    fn test_skeleton_image_new() {
        let image = SkeletonImage::new(300.0, 200.0);
        assert!((image.width - 300.0).abs() < f32::EPSILON);
        assert!((image.height - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_image_square() {
        let image = SkeletonImage::square(100.0);
        assert!((image.width - 100.0).abs() < f32::EPSILON);
        assert!((image.height - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_image_aspect_ratio() {
        let image = SkeletonImage::aspect_ratio(AspectRatio::Widescreen, 160.0);
        assert!((image.width - 160.0).abs() < f32::EPSILON);
        assert!((image.height - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_skeleton_image_minimum_size() {
        let image = SkeletonImage::new(5.0, 5.0);
        assert!((image.width - 10.0).abs() < f32::EPSILON);
        assert!((image.height - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_image_render_params_icon_size() {
        let image = SkeletonImage::new(200.0, 100.0);
        let params = image.render_params(0.0, 0.0);

        // Icon should be 30% of smaller dimension, max 48
        assert!((params.icon_size - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_image_render_params_icon_size_cap() {
        let image = SkeletonImage::new(400.0, 300.0);
        let params = image.render_params(0.0, 0.0);

        // Should be capped at 48
        assert!((params.icon_size - 48.0).abs() < f32::EPSILON);
    }
}
