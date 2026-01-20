//! Safe area insets handling.
//!
//! Handles device safe areas including notches, home indicators,
//! camera cutouts, and system UI overlays.
//!
//! ## Common Safe Area Scenarios
//!
//! - **iPhone notch**: Top inset for the notch, bottom for home indicator
//! - **Android camera cutout**: Top or corner insets
//! - **System navigation**: Bottom inset for gesture navigation

use serde::{Deserialize, Serialize};

/// Safe area insets for a device.
///
/// Represents the areas of the screen that should not contain
/// interactive content due to hardware features (notches, cutouts)
/// or system UI (status bar, navigation bar).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct SafeAreaInsets {
    /// Top inset in pixels (status bar, notch).
    top: f32,
    /// Bottom inset in pixels (home indicator, navigation bar).
    bottom: f32,
    /// Left inset in pixels (camera cutout in landscape).
    left: f32,
    /// Right inset in pixels (camera cutout in landscape).
    right: f32,
}

impl SafeAreaInsets {
    /// Create new safe area insets with all values.
    pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Create safe area insets with symmetric vertical/horizontal values.
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }

    /// Create safe area insets with uniform value on all edges.
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    /// Create safe area insets with only top inset.
    pub fn top_only(top: f32) -> Self {
        Self {
            top,
            ..Default::default()
        }
    }

    /// Create safe area insets with only bottom inset.
    pub fn bottom_only(bottom: f32) -> Self {
        Self {
            bottom,
            ..Default::default()
        }
    }

    /// Create safe area insets with top and bottom (vertical only).
    pub fn vertical(top: f32, bottom: f32) -> Self {
        Self {
            top,
            bottom,
            left: 0.0,
            right: 0.0,
        }
    }

    /// Get the top inset.
    pub fn top(&self) -> f32 {
        self.top
    }

    /// Get the bottom inset.
    pub fn bottom(&self) -> f32 {
        self.bottom
    }

    /// Get the left inset.
    pub fn left(&self) -> f32 {
        self.left
    }

    /// Get the right inset.
    pub fn right(&self) -> f32 {
        self.right
    }

    /// Get total horizontal inset (left + right).
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical inset (top + bottom).
    pub fn vertical_total(&self) -> f32 {
        self.top + self.bottom
    }

    /// Get inset for a specific edge.
    pub fn edge(&self, edge: SafeAreaEdge) -> f32 {
        match edge {
            SafeAreaEdge::Top => self.top,
            SafeAreaEdge::Bottom => self.bottom,
            SafeAreaEdge::Left => self.left,
            SafeAreaEdge::Right => self.right,
        }
    }

    /// Create a new SafeAreaInsets with a modified edge value.
    pub fn with_edge(mut self, edge: SafeAreaEdge, value: f32) -> Self {
        match edge {
            SafeAreaEdge::Top => self.top = value,
            SafeAreaEdge::Bottom => self.bottom = value,
            SafeAreaEdge::Left => self.left = value,
            SafeAreaEdge::Right => self.right = value,
        }
        self
    }

    /// Apply minimum values from another SafeAreaInsets.
    ///
    /// Takes the maximum of each edge from self and other.
    pub fn merge(&self, other: &SafeAreaInsets) -> SafeAreaInsets {
        SafeAreaInsets {
            top: self.top.max(other.top),
            bottom: self.bottom.max(other.bottom),
            left: self.left.max(other.left),
            right: self.right.max(other.right),
        }
    }

    /// Scale all insets by a factor.
    pub fn scale(&self, factor: f32) -> SafeAreaInsets {
        SafeAreaInsets {
            top: self.top * factor,
            bottom: self.bottom * factor,
            left: self.left * factor,
            right: self.right * factor,
        }
    }

    /// Add padding to all edges.
    pub fn with_padding(&self, padding: f32) -> SafeAreaInsets {
        SafeAreaInsets {
            top: self.top + padding,
            bottom: self.bottom + padding,
            left: self.left + padding,
            right: self.right + padding,
        }
    }

    /// Check if any inset is non-zero.
    pub fn has_insets(&self) -> bool {
        self.top > 0.0 || self.bottom > 0.0 || self.left > 0.0 || self.right > 0.0
    }

    /// Get insets for a rotated orientation.
    ///
    /// Rotates the insets 90 degrees clockwise.
    pub fn rotate_clockwise(&self) -> SafeAreaInsets {
        SafeAreaInsets {
            top: self.left,
            right: self.top,
            bottom: self.right,
            left: self.bottom,
        }
    }

    /// Get insets for a rotated orientation.
    ///
    /// Rotates the insets 90 degrees counter-clockwise.
    pub fn rotate_counter_clockwise(&self) -> SafeAreaInsets {
        SafeAreaInsets {
            top: self.right,
            right: self.bottom,
            bottom: self.left,
            left: self.top,
        }
    }

    /// Calculate content rect given screen dimensions.
    ///
    /// Returns (x, y, width, height) of the safe content area.
    pub fn content_rect(&self, screen_width: f32, screen_height: f32) -> (f32, f32, f32, f32) {
        let x = self.left;
        let y = self.top;
        let width = (screen_width - self.horizontal()).max(0.0);
        let height = (screen_height - self.vertical_total()).max(0.0);
        (x, y, width, height)
    }
}

/// Edge identifiers for safe area insets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafeAreaEdge {
    /// Top edge (status bar, notch).
    Top,
    /// Bottom edge (home indicator, navigation bar).
    Bottom,
    /// Left edge.
    Left,
    /// Right edge.
    Right,
}

impl SafeAreaEdge {
    /// Get all edges.
    pub fn all() -> &'static [SafeAreaEdge] {
        &[
            SafeAreaEdge::Top,
            SafeAreaEdge::Bottom,
            SafeAreaEdge::Left,
            SafeAreaEdge::Right,
        ]
    }

    /// Get vertical edges (top and bottom).
    pub fn vertical() -> &'static [SafeAreaEdge] {
        &[SafeAreaEdge::Top, SafeAreaEdge::Bottom]
    }

    /// Get horizontal edges (left and right).
    pub fn horizontal() -> &'static [SafeAreaEdge] {
        &[SafeAreaEdge::Left, SafeAreaEdge::Right]
    }

    /// Returns true if this is a vertical edge.
    pub fn is_vertical(&self) -> bool {
        matches!(self, SafeAreaEdge::Top | SafeAreaEdge::Bottom)
    }

    /// Returns true if this is a horizontal edge.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, SafeAreaEdge::Left | SafeAreaEdge::Right)
    }
}

/// Provider trait for safe area insets.
///
/// Implement this trait to provide safe area insets from
/// platform-specific sources.
pub trait SafeAreaProvider {
    /// Get the current safe area insets.
    fn safe_area_insets(&self) -> SafeAreaInsets;

    /// Check if safe area insets are available.
    fn has_safe_area(&self) -> bool {
        self.safe_area_insets().has_insets()
    }
}

/// Common device safe area presets.
pub mod presets {
    use super::SafeAreaInsets;

    /// iPhone with notch (iPhone X and later).
    pub fn iphone_notch() -> SafeAreaInsets {
        SafeAreaInsets::new(47.0, 34.0, 0.0, 0.0)
    }

    /// iPhone with Dynamic Island (iPhone 14 Pro and later).
    pub fn iphone_dynamic_island() -> SafeAreaInsets {
        SafeAreaInsets::new(59.0, 34.0, 0.0, 0.0)
    }

    /// iPhone SE / older iPhones without notch.
    pub fn iphone_classic() -> SafeAreaInsets {
        SafeAreaInsets::new(20.0, 0.0, 0.0, 0.0)
    }

    /// iPad without home button.
    pub fn ipad_modern() -> SafeAreaInsets {
        SafeAreaInsets::new(24.0, 20.0, 0.0, 0.0)
    }

    /// iPad with home button.
    pub fn ipad_classic() -> SafeAreaInsets {
        SafeAreaInsets::new(20.0, 0.0, 0.0, 0.0)
    }

    /// Android with gesture navigation.
    pub fn android_gesture_nav() -> SafeAreaInsets {
        SafeAreaInsets::new(24.0, 48.0, 0.0, 0.0)
    }

    /// Android with 3-button navigation.
    pub fn android_button_nav() -> SafeAreaInsets {
        SafeAreaInsets::new(24.0, 48.0, 0.0, 0.0)
    }

    /// Android with camera cutout.
    pub fn android_camera_cutout() -> SafeAreaInsets {
        SafeAreaInsets::new(32.0, 48.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_area_insets_default() {
        let insets = SafeAreaInsets::default();
        assert_eq!(insets.top(), 0.0);
        assert_eq!(insets.bottom(), 0.0);
        assert_eq!(insets.left(), 0.0);
        assert_eq!(insets.right(), 0.0);
        assert!(!insets.has_insets());
    }

    #[test]
    fn test_safe_area_insets_new() {
        let insets = SafeAreaInsets::new(44.0, 34.0, 0.0, 0.0);
        assert_eq!(insets.top(), 44.0);
        assert_eq!(insets.bottom(), 34.0);
        assert_eq!(insets.horizontal(), 0.0);
        assert_eq!(insets.vertical_total(), 78.0);
        assert!(insets.has_insets());
    }

    #[test]
    fn test_safe_area_symmetric() {
        let insets = SafeAreaInsets::symmetric(20.0, 10.0);
        assert_eq!(insets.top(), 20.0);
        assert_eq!(insets.bottom(), 20.0);
        assert_eq!(insets.left(), 10.0);
        assert_eq!(insets.right(), 10.0);
    }

    #[test]
    fn test_safe_area_edge() {
        let insets = SafeAreaInsets::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(insets.edge(SafeAreaEdge::Top), 1.0);
        assert_eq!(insets.edge(SafeAreaEdge::Bottom), 2.0);
        assert_eq!(insets.edge(SafeAreaEdge::Left), 3.0);
        assert_eq!(insets.edge(SafeAreaEdge::Right), 4.0);
    }

    #[test]
    fn test_safe_area_merge() {
        let a = SafeAreaInsets::new(10.0, 20.0, 5.0, 5.0);
        let b = SafeAreaInsets::new(15.0, 10.0, 10.0, 0.0);
        let merged = a.merge(&b);

        assert_eq!(merged.top(), 15.0);
        assert_eq!(merged.bottom(), 20.0);
        assert_eq!(merged.left(), 10.0);
        assert_eq!(merged.right(), 5.0);
    }

    #[test]
    fn test_safe_area_scale() {
        let insets = SafeAreaInsets::new(10.0, 20.0, 5.0, 5.0);
        let scaled = insets.scale(2.0);

        assert_eq!(scaled.top(), 20.0);
        assert_eq!(scaled.bottom(), 40.0);
        assert_eq!(scaled.left(), 10.0);
        assert_eq!(scaled.right(), 10.0);
    }

    #[test]
    fn test_safe_area_content_rect() {
        let insets = SafeAreaInsets::new(44.0, 34.0, 0.0, 0.0);
        let (x, y, width, height) = insets.content_rect(375.0, 812.0);

        assert_eq!(x, 0.0);
        assert_eq!(y, 44.0);
        assert_eq!(width, 375.0);
        assert_eq!(height, 734.0); // 812 - 44 - 34
    }

    #[test]
    fn test_safe_area_rotate() {
        let insets = SafeAreaInsets::new(44.0, 34.0, 0.0, 0.0);
        let rotated = insets.rotate_clockwise();

        assert_eq!(rotated.top(), 0.0);
        assert_eq!(rotated.right(), 44.0);
        assert_eq!(rotated.bottom(), 0.0);
        assert_eq!(rotated.left(), 34.0);
    }

    #[test]
    fn test_presets() {
        let iphone = presets::iphone_notch();
        assert!(iphone.top() > 0.0);
        assert!(iphone.bottom() > 0.0);

        let android = presets::android_gesture_nav();
        assert!(android.top() > 0.0);
        assert!(android.bottom() > 0.0);
    }
}
