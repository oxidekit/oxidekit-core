//! Skeleton Text Loader
//!
//! Placeholder for text content with support for single and multi-line layouts.
//!
//! # Example
//!
//! ```
//! use oxide_loading::skeleton::{SkeletonText, SkeletonAnimation};
//!
//! // Single line skeleton
//! let line = SkeletonText::line()
//!     .width(200.0);
//!
//! // Multi-line paragraph skeleton
//! let paragraph = SkeletonText::paragraph(4)
//!     .width(300.0)
//!     .animation(SkeletonAnimation::Shimmer);
//! ```

use serde::{Deserialize, Serialize};

use super::{SkeletonAnimation, SkeletonConfig, SkeletonRenderParams};

/// Skeleton placeholder for text content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonText {
    /// Number of lines
    pub lines: usize,
    /// Width of the skeleton (or max width for multi-line)
    pub width: f32,
    /// Height of each line
    pub line_height: f32,
    /// Spacing between lines
    pub line_spacing: f32,
    /// Whether the last line should be shorter
    pub vary_last_line: bool,
    /// Last line width multiplier (0.0 to 1.0)
    pub last_line_width: f32,
    /// Configuration (animation, colors, etc.)
    pub config: SkeletonConfig,
}

impl Default for SkeletonText {
    fn default() -> Self {
        Self::line()
    }
}

impl SkeletonText {
    /// Create a single-line skeleton
    pub fn line() -> Self {
        Self {
            lines: 1,
            width: 150.0,
            line_height: 16.0,
            line_spacing: 8.0,
            vary_last_line: false,
            last_line_width: 0.6,
            config: SkeletonConfig::default(),
        }
    }

    /// Create a multi-line paragraph skeleton
    pub fn paragraph(lines: usize) -> Self {
        Self {
            lines: lines.max(1),
            width: 250.0,
            line_height: 14.0,
            line_spacing: 6.0,
            vary_last_line: true,
            last_line_width: 0.6,
            config: SkeletonConfig::default(),
        }
    }

    /// Create a heading skeleton
    pub fn heading() -> Self {
        Self {
            lines: 1,
            width: 200.0,
            line_height: 24.0,
            line_spacing: 0.0,
            vary_last_line: false,
            last_line_width: 1.0,
            config: SkeletonConfig::default().border_radius(4.0),
        }
    }

    /// Set the width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(10.0);
        self
    }

    /// Set the line height
    pub fn line_height(mut self, height: f32) -> Self {
        self.line_height = height.max(4.0);
        self
    }

    /// Set the line spacing
    pub fn line_spacing(mut self, spacing: f32) -> Self {
        self.line_spacing = spacing.max(0.0);
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

    /// Set whether to vary the last line width
    pub fn vary_last_line(mut self, vary: bool) -> Self {
        self.vary_last_line = vary;
        self
    }

    /// Set the last line width multiplier
    pub fn last_line_width(mut self, multiplier: f32) -> Self {
        self.last_line_width = multiplier.clamp(0.1, 1.0);
        self
    }

    /// Update animation (call each frame)
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.config.update(delta_time)
    }

    /// Calculate total height
    pub fn total_height(&self) -> f32 {
        if self.lines == 0 {
            0.0
        } else {
            self.line_height * self.lines as f32 + self.line_spacing * (self.lines - 1) as f32
        }
    }

    /// Get render parameters for each line
    pub fn render_params(&self, x: f32, y: f32) -> Vec<SkeletonRenderParams> {
        let mut params = Vec::with_capacity(self.lines);

        for i in 0..self.lines {
            let is_last = i == self.lines - 1;
            let line_width = if is_last && self.vary_last_line && self.lines > 1 {
                self.width * self.last_line_width
            } else {
                self.width
            };

            let line_y = y + (self.line_height + self.line_spacing) * i as f32;

            params.push(SkeletonRenderParams {
                x,
                y: line_y,
                width: line_width,
                height: self.line_height,
                border_radius: self.config.border_radius,
                base_color: self.config.colors.base,
                animation: self.config.animation,
                shimmer_position: self.config.shimmer_position(),
                wave_opacity: self.config.wave_opacity(),
                highlight_color: self.config.colors.highlight,
            });
        }

        params
    }

    /// Get ARIA attributes
    pub fn aria_attributes(&self) -> Vec<(&'static str, String)> {
        self.config.aria_attributes("Text")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_text_line() {
        let skeleton = SkeletonText::line();
        assert_eq!(skeleton.lines, 1);
        assert!((skeleton.line_height - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_text_paragraph() {
        let skeleton = SkeletonText::paragraph(4);
        assert_eq!(skeleton.lines, 4);
        assert!(skeleton.vary_last_line);
    }

    #[test]
    fn test_skeleton_text_heading() {
        let skeleton = SkeletonText::heading();
        assert_eq!(skeleton.lines, 1);
        assert!((skeleton.line_height - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_text_builder() {
        let skeleton = SkeletonText::line()
            .width(300.0)
            .line_height(20.0)
            .animation(SkeletonAnimation::Wave);

        assert!((skeleton.width - 300.0).abs() < f32::EPSILON);
        assert!((skeleton.line_height - 20.0).abs() < f32::EPSILON);
        assert_eq!(skeleton.config.animation, SkeletonAnimation::Wave);
    }

    #[test]
    fn test_skeleton_text_total_height() {
        let skeleton = SkeletonText::paragraph(3)
            .line_height(14.0)
            .line_spacing(6.0);

        // 3 lines * 14px + 2 gaps * 6px = 42 + 12 = 54
        assert!((skeleton.total_height() - 54.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_text_render_params() {
        let skeleton = SkeletonText::paragraph(3)
            .width(200.0)
            .vary_last_line(true)
            .last_line_width(0.5);

        let params = skeleton.render_params(10.0, 20.0);
        assert_eq!(params.len(), 3);

        // First line full width
        assert!((params[0].width - 200.0).abs() < f32::EPSILON);
        // Last line half width
        assert!((params[2].width - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_text_last_line_width_clamping() {
        let skeleton = SkeletonText::line().last_line_width(2.0);
        assert!((skeleton.last_line_width - 1.0).abs() < f32::EPSILON);

        let skeleton = SkeletonText::line().last_line_width(0.0);
        assert!((skeleton.last_line_width - 0.1).abs() < f32::EPSILON);
    }
}
