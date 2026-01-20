//! Skeleton Card Loader
//!
//! Composite placeholder for card layouts with image, title, and content areas.
//!
//! # Example
//!
//! ```
//! use oxide_loading::skeleton::{SkeletonCard, SkeletonAnimation};
//!
//! // Default card skeleton
//! let card = SkeletonCard::new();
//!
//! // Card without image
//! let text_card = SkeletonCard::new()
//!     .show_image(false)
//!     .content_lines(4);
//! ```

use serde::{Deserialize, Serialize};

use super::{SkeletonAnimation, SkeletonConfig, SkeletonRenderParams};

/// Skeleton placeholder for card layouts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonCard {
    /// Card width
    pub width: f32,
    /// Whether to show image placeholder
    pub show_image: bool,
    /// Image height (if shown)
    pub image_height: f32,
    /// Whether to show avatar
    pub show_avatar: bool,
    /// Avatar size
    pub avatar_size: f32,
    /// Whether to show title
    pub show_title: bool,
    /// Title width multiplier (0.0 to 1.0)
    pub title_width: f32,
    /// Number of content text lines
    pub content_lines: usize,
    /// Padding inside the card
    pub padding: f32,
    /// Spacing between elements
    pub spacing: f32,
    /// Configuration
    pub config: SkeletonConfig,
}

impl Default for SkeletonCard {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonCard {
    /// Create a new card skeleton
    pub fn new() -> Self {
        Self {
            width: 320.0,
            show_image: true,
            image_height: 180.0,
            show_avatar: false,
            avatar_size: 40.0,
            show_title: true,
            title_width: 0.7,
            content_lines: 3,
            padding: 16.0,
            spacing: 12.0,
            config: SkeletonConfig::default().border_radius(8.0),
        }
    }

    /// Create a card with avatar (social media style)
    pub fn social() -> Self {
        Self {
            width: 320.0,
            show_image: false,
            image_height: 0.0,
            show_avatar: true,
            avatar_size: 48.0,
            show_title: true,
            title_width: 0.6,
            content_lines: 4,
            padding: 16.0,
            spacing: 12.0,
            config: SkeletonConfig::default().border_radius(8.0),
        }
    }

    /// Create a media card (image focused)
    pub fn media() -> Self {
        Self {
            width: 280.0,
            show_image: true,
            image_height: 200.0,
            show_avatar: false,
            avatar_size: 0.0,
            show_title: true,
            title_width: 0.8,
            content_lines: 2,
            padding: 12.0,
            spacing: 8.0,
            config: SkeletonConfig::default().border_radius(12.0),
        }
    }

    /// Set the card width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(100.0);
        self
    }

    /// Set whether to show image
    pub fn show_image(mut self, show: bool) -> Self {
        self.show_image = show;
        self
    }

    /// Set the image height
    pub fn image_height(mut self, height: f32) -> Self {
        self.image_height = height.max(0.0);
        self
    }

    /// Set whether to show avatar
    pub fn show_avatar(mut self, show: bool) -> Self {
        self.show_avatar = show;
        self
    }

    /// Set the avatar size
    pub fn avatar_size(mut self, size: f32) -> Self {
        self.avatar_size = size.max(16.0);
        self
    }

    /// Set whether to show title
    pub fn show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }

    /// Set the title width multiplier
    pub fn title_width(mut self, width: f32) -> Self {
        self.title_width = width.clamp(0.2, 1.0);
        self
    }

    /// Set the number of content lines
    pub fn content_lines(mut self, lines: usize) -> Self {
        self.content_lines = lines;
        self
    }

    /// Set the padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding.max(0.0);
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

    /// Calculate total card height
    pub fn total_height(&self) -> f32 {
        let mut height = 0.0;

        // Image
        if self.show_image {
            height += self.image_height;
        }

        // Content area padding
        height += self.padding * 2.0;

        // Avatar row (if shown)
        if self.show_avatar {
            height += self.avatar_size;
            if self.show_title || self.content_lines > 0 {
                height += self.spacing;
            }
        }

        // Title
        if self.show_title {
            height += 20.0; // Title line height
            if self.content_lines > 0 {
                height += self.spacing;
            }
        }

        // Content lines
        if self.content_lines > 0 {
            let line_height = 14.0;
            let line_spacing = 6.0;
            height += line_height * self.content_lines as f32
                + line_spacing * (self.content_lines - 1) as f32;
        }

        height
    }

    /// Get render parameters for all card elements
    pub fn render_params(&self, x: f32, y: f32) -> SkeletonCardRenderParams {
        let mut elements = Vec::new();
        let content_width = self.width - self.padding * 2.0;
        let mut current_y = y;

        // Card background/border
        let card_height = self.total_height();

        // Image
        if self.show_image {
            elements.push(SkeletonCardElement {
                element_type: CardElementType::Image,
                params: SkeletonRenderParams {
                    x,
                    y: current_y,
                    width: self.width,
                    height: self.image_height,
                    border_radius: self.config.border_radius,
                    base_color: self.config.colors.base,
                    animation: self.config.animation,
                    shimmer_position: self.config.shimmer_position(),
                    wave_opacity: self.config.wave_opacity(),
                    highlight_color: self.config.colors.highlight,
                },
            });
            current_y += self.image_height;
        }

        // Content padding
        current_y += self.padding;
        let content_x = x + self.padding;

        // Avatar row
        if self.show_avatar {
            // Avatar circle
            elements.push(SkeletonCardElement {
                element_type: CardElementType::Avatar,
                params: SkeletonRenderParams {
                    x: content_x,
                    y: current_y,
                    width: self.avatar_size,
                    height: self.avatar_size,
                    border_radius: self.avatar_size / 2.0,
                    base_color: self.config.colors.base,
                    animation: self.config.animation,
                    shimmer_position: self.config.shimmer_position(),
                    wave_opacity: self.config.wave_opacity(),
                    highlight_color: self.config.colors.highlight,
                },
            });

            // Name/subtitle next to avatar
            let text_x = content_x + self.avatar_size + self.spacing;
            let text_width = content_width - self.avatar_size - self.spacing;

            elements.push(SkeletonCardElement {
                element_type: CardElementType::Title,
                params: SkeletonRenderParams {
                    x: text_x,
                    y: current_y + 4.0,
                    width: text_width * 0.6,
                    height: 14.0,
                    border_radius: 4.0,
                    base_color: self.config.colors.base,
                    animation: self.config.animation,
                    shimmer_position: self.config.shimmer_position(),
                    wave_opacity: self.config.wave_opacity(),
                    highlight_color: self.config.colors.highlight,
                },
            });

            elements.push(SkeletonCardElement {
                element_type: CardElementType::Subtitle,
                params: SkeletonRenderParams {
                    x: text_x,
                    y: current_y + 22.0,
                    width: text_width * 0.4,
                    height: 12.0,
                    border_radius: 4.0,
                    base_color: self.config.colors.base,
                    animation: self.config.animation,
                    shimmer_position: self.config.shimmer_position(),
                    wave_opacity: self.config.wave_opacity(),
                    highlight_color: self.config.colors.highlight,
                },
            });

            current_y += self.avatar_size + self.spacing;
        }

        // Title (if no avatar shown)
        if self.show_title && !self.show_avatar {
            elements.push(SkeletonCardElement {
                element_type: CardElementType::Title,
                params: SkeletonRenderParams {
                    x: content_x,
                    y: current_y,
                    width: content_width * self.title_width,
                    height: 20.0,
                    border_radius: 4.0,
                    base_color: self.config.colors.base,
                    animation: self.config.animation,
                    shimmer_position: self.config.shimmer_position(),
                    wave_opacity: self.config.wave_opacity(),
                    highlight_color: self.config.colors.highlight,
                },
            });
            current_y += 20.0 + self.spacing;
        }

        // Content lines
        for i in 0..self.content_lines {
            let is_last = i == self.content_lines - 1;
            let line_width = if is_last && self.content_lines > 1 {
                content_width * 0.6
            } else {
                content_width
            };

            elements.push(SkeletonCardElement {
                element_type: CardElementType::ContentLine,
                params: SkeletonRenderParams {
                    x: content_x,
                    y: current_y,
                    width: line_width,
                    height: 14.0,
                    border_radius: 4.0,
                    base_color: self.config.colors.base,
                    animation: self.config.animation,
                    shimmer_position: self.config.shimmer_position(),
                    wave_opacity: self.config.wave_opacity(),
                    highlight_color: self.config.colors.highlight,
                },
            });
            current_y += 14.0 + 6.0;
        }

        SkeletonCardRenderParams {
            card_x: x,
            card_y: y,
            card_width: self.width,
            card_height,
            card_border_radius: self.config.border_radius,
            elements,
        }
    }

    /// Get ARIA attributes
    pub fn aria_attributes(&self) -> Vec<(&'static str, String)> {
        self.config.aria_attributes("Card")
    }
}

/// Types of elements in a card skeleton
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardElementType {
    /// Image placeholder
    Image,
    /// Avatar circle
    Avatar,
    /// Title text
    Title,
    /// Subtitle text
    Subtitle,
    /// Content text line
    ContentLine,
}

/// Single element in card skeleton
#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonCardElement {
    /// Type of element
    pub element_type: CardElementType,
    /// Render parameters
    pub params: SkeletonRenderParams,
}

/// Render parameters for complete card skeleton
#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonCardRenderParams {
    /// Card X position
    pub card_x: f32,
    /// Card Y position
    pub card_y: f32,
    /// Card width
    pub card_width: f32,
    /// Card height
    pub card_height: f32,
    /// Card border radius
    pub card_border_radius: f32,
    /// Individual elements
    pub elements: Vec<SkeletonCardElement>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_card_default() {
        let card = SkeletonCard::new();
        assert!((card.width - 320.0).abs() < f32::EPSILON);
        assert!(card.show_image);
        assert!(card.show_title);
        assert_eq!(card.content_lines, 3);
    }

    #[test]
    fn test_skeleton_card_social() {
        let card = SkeletonCard::social();
        assert!(!card.show_image);
        assert!(card.show_avatar);
    }

    #[test]
    fn test_skeleton_card_media() {
        let card = SkeletonCard::media();
        assert!(card.show_image);
        assert!(!card.show_avatar);
    }

    #[test]
    fn test_skeleton_card_builder() {
        let card = SkeletonCard::new()
            .width(400.0)
            .show_image(false)
            .content_lines(5);

        assert!((card.width - 400.0).abs() < f32::EPSILON);
        assert!(!card.show_image);
        assert_eq!(card.content_lines, 5);
    }

    #[test]
    fn test_skeleton_card_total_height() {
        let card = SkeletonCard::new()
            .show_image(true)
            .image_height(100.0)
            .content_lines(2)
            .padding(10.0);

        let height = card.total_height();
        assert!(height > 100.0); // At least image height
    }

    #[test]
    fn test_skeleton_card_render_params_elements() {
        let card = SkeletonCard::new()
            .show_image(true)
            .show_title(true)
            .content_lines(2);

        let params = card.render_params(0.0, 0.0);

        // Should have: image, title, 2 content lines
        assert!(params.elements.len() >= 4);

        // First element should be image
        assert_eq!(params.elements[0].element_type, CardElementType::Image);
    }

    #[test]
    fn test_skeleton_card_render_params_social() {
        let card = SkeletonCard::social();
        let params = card.render_params(0.0, 0.0);

        // Should have avatar
        let has_avatar = params
            .elements
            .iter()
            .any(|e| e.element_type == CardElementType::Avatar);
        assert!(has_avatar);
    }

    #[test]
    fn test_skeleton_card_width_minimum() {
        let card = SkeletonCard::new().width(50.0);
        assert!((card.width - 100.0).abs() < f32::EPSILON);
    }
}
