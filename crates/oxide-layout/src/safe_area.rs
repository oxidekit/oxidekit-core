//! Safe Area Handling
//!
//! Provides support for device-specific safe areas like notches, home indicators,
//! and status bars commonly found on modern mobile devices.

use bitflags::bitflags;

/// Safe area insets representing the spacing needed to avoid device-specific UI elements.
///
/// These insets are typically used to handle:
/// - Notches on modern smartphones (top inset)
/// - Home indicators on gesture-based devices (bottom inset)
/// - Camera cutouts (various positions)
/// - Status bars (top inset)
/// - Navigation bars (bottom inset)
///
/// # Example
///
/// ```
/// use oxide_layout::safe_area::SafeAreaInsets;
///
/// // iPhone-style insets with notch and home indicator
/// let iphone_insets = SafeAreaInsets::new(47.0, 34.0, 0.0, 0.0);
///
/// // Apply padding to content
/// let content_top = iphone_insets.top;
/// let content_bottom = iphone_insets.bottom;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SafeAreaInsets {
    /// Top inset (for notches, status bar)
    pub top: f32,
    /// Bottom inset (for home indicator, navigation bar)
    pub bottom: f32,
    /// Left inset (for camera cutouts, display curves)
    pub left: f32,
    /// Right inset (for camera cutouts, display curves)
    pub right: f32,
}

impl SafeAreaInsets {
    /// Create new safe area insets
    pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Create zero insets (no safe area adjustments needed)
    pub fn zero() -> Self {
        Self::default()
    }

    /// Create uniform insets on all sides
    pub fn uniform(inset: f32) -> Self {
        Self {
            top: inset,
            bottom: inset,
            left: inset,
            right: inset,
        }
    }

    /// Create insets with only vertical values
    pub fn vertical(top: f32, bottom: f32) -> Self {
        Self {
            top,
            bottom,
            left: 0.0,
            right: 0.0,
        }
    }

    /// Create insets with only horizontal values
    pub fn horizontal(left: f32, right: f32) -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            left,
            right,
        }
    }

    /// Check if all insets are zero
    pub fn is_zero(&self) -> bool {
        self.top == 0.0 && self.bottom == 0.0 && self.left == 0.0 && self.right == 0.0
    }

    /// Get the total horizontal inset (left + right)
    pub fn horizontal_total(&self) -> f32 {
        self.left + self.right
    }

    /// Get the total vertical inset (top + bottom)
    pub fn vertical_total(&self) -> f32 {
        self.top + self.bottom
    }

    /// Apply a scale factor to all insets
    pub fn scaled(&self, factor: f32) -> Self {
        Self {
            top: self.top * factor,
            bottom: self.bottom * factor,
            left: self.left * factor,
            right: self.right * factor,
        }
    }

    /// Return insets with only the specified edges preserved
    pub fn with_edges(&self, edges: SafeAreaEdges) -> Self {
        Self {
            top: if edges.contains(SafeAreaEdges::TOP) {
                self.top
            } else {
                0.0
            },
            bottom: if edges.contains(SafeAreaEdges::BOTTOM) {
                self.bottom
            } else {
                0.0
            },
            left: if edges.contains(SafeAreaEdges::LEFT) {
                self.left
            } else {
                0.0
            },
            right: if edges.contains(SafeAreaEdges::RIGHT) {
                self.right
            } else {
                0.0
            },
        }
    }

    /// Combine with other insets, taking the maximum of each edge
    pub fn max(&self, other: &Self) -> Self {
        Self {
            top: self.top.max(other.top),
            bottom: self.bottom.max(other.bottom),
            left: self.left.max(other.left),
            right: self.right.max(other.right),
        }
    }

    /// Add additional padding to the insets
    pub fn with_padding(&self, padding: f32) -> Self {
        Self {
            top: self.top + padding,
            bottom: self.bottom + padding,
            left: self.left + padding,
            right: self.right + padding,
        }
    }
}

bitflags! {
    /// Flags indicating which edges should respect safe area insets
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SafeAreaEdges: u8 {
        /// Top edge (for notches, status bar)
        const TOP = 0b0001;
        /// Bottom edge (for home indicator)
        const BOTTOM = 0b0010;
        /// Left edge (for display curves)
        const LEFT = 0b0100;
        /// Right edge (for display curves)
        const RIGHT = 0b1000;
        /// All edges
        const ALL = Self::TOP.bits() | Self::BOTTOM.bits() | Self::LEFT.bits() | Self::RIGHT.bits();
        /// Vertical edges only (top and bottom)
        const VERTICAL = Self::TOP.bits() | Self::BOTTOM.bits();
        /// Horizontal edges only (left and right)
        const HORIZONTAL = Self::LEFT.bits() | Self::RIGHT.bits();
    }
}

impl Default for SafeAreaEdges {
    fn default() -> Self {
        Self::ALL
    }
}

/// Container that manages safe area handling for its content
///
/// # Example
///
/// ```
/// use oxide_layout::safe_area::{SafeAreaContainer, SafeAreaInsets, SafeAreaEdges};
///
/// // Create a container that respects only top and bottom safe areas
/// let container = SafeAreaContainer::new()
///     .with_insets(SafeAreaInsets::new(47.0, 34.0, 0.0, 0.0))
///     .with_edges(SafeAreaEdges::VERTICAL);
///
/// let effective = container.effective_insets();
/// assert_eq!(effective.top, 47.0);
/// assert_eq!(effective.bottom, 34.0);
/// assert_eq!(effective.left, 0.0);
/// assert_eq!(effective.right, 0.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SafeAreaContainer {
    /// The device's safe area insets
    insets: SafeAreaInsets,
    /// Which edges should respect safe area insets
    edges: SafeAreaEdges,
    /// Additional minimum padding beyond safe area
    minimum_padding: SafeAreaInsets,
}

impl SafeAreaContainer {
    /// Create a new safe area container with default settings
    pub fn new() -> Self {
        Self {
            insets: SafeAreaInsets::zero(),
            edges: SafeAreaEdges::ALL,
            minimum_padding: SafeAreaInsets::zero(),
        }
    }

    /// Set the safe area insets
    pub fn with_insets(mut self, insets: SafeAreaInsets) -> Self {
        self.insets = insets;
        self
    }

    /// Set which edges should respect safe area insets
    pub fn with_edges(mut self, edges: SafeAreaEdges) -> Self {
        self.edges = edges;
        self
    }

    /// Set minimum padding that should always be applied regardless of safe area
    pub fn with_minimum_padding(mut self, padding: SafeAreaInsets) -> Self {
        self.minimum_padding = padding;
        self
    }

    /// Get the safe area insets
    pub fn insets(&self) -> &SafeAreaInsets {
        &self.insets
    }

    /// Get the active edges
    pub fn edges(&self) -> SafeAreaEdges {
        self.edges
    }

    /// Calculate the effective insets (safe area + minimum padding for active edges)
    pub fn effective_insets(&self) -> SafeAreaInsets {
        let safe_insets = self.insets.with_edges(self.edges);
        safe_insets.max(&self.minimum_padding)
    }

    /// Update the safe area insets (e.g., when device reports new values)
    pub fn update_insets(&mut self, insets: SafeAreaInsets) {
        self.insets = insets;
    }

    /// Check if top edge is active
    pub fn has_top(&self) -> bool {
        self.edges.contains(SafeAreaEdges::TOP)
    }

    /// Check if bottom edge is active
    pub fn has_bottom(&self) -> bool {
        self.edges.contains(SafeAreaEdges::BOTTOM)
    }

    /// Check if left edge is active
    pub fn has_left(&self) -> bool {
        self.edges.contains(SafeAreaEdges::LEFT)
    }

    /// Check if right edge is active
    pub fn has_right(&self) -> bool {
        self.edges.contains(SafeAreaEdges::RIGHT)
    }
}

impl Default for SafeAreaContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Common device safe area presets
pub mod presets {
    use super::SafeAreaInsets;

    /// iPhone with notch (iPhone X/11/12/13/14 series)
    pub fn iphone_notch() -> SafeAreaInsets {
        SafeAreaInsets::new(47.0, 34.0, 0.0, 0.0)
    }

    /// iPhone with Dynamic Island (iPhone 14 Pro and later)
    pub fn iphone_dynamic_island() -> SafeAreaInsets {
        SafeAreaInsets::new(59.0, 34.0, 0.0, 0.0)
    }

    /// iPhone without notch (iPhone SE, older models)
    pub fn iphone_classic() -> SafeAreaInsets {
        SafeAreaInsets::new(20.0, 0.0, 0.0, 0.0)
    }

    /// Android with standard status bar
    pub fn android_standard() -> SafeAreaInsets {
        SafeAreaInsets::new(24.0, 0.0, 0.0, 0.0)
    }

    /// Android with gesture navigation
    pub fn android_gesture_nav() -> SafeAreaInsets {
        SafeAreaInsets::new(24.0, 20.0, 0.0, 0.0)
    }

    /// iPad (various models with similar insets)
    pub fn ipad() -> SafeAreaInsets {
        SafeAreaInsets::new(24.0, 20.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_area_insets_zero() {
        let insets = SafeAreaInsets::zero();
        assert!(insets.is_zero());
        assert_eq!(insets.top, 0.0);
        assert_eq!(insets.bottom, 0.0);
        assert_eq!(insets.left, 0.0);
        assert_eq!(insets.right, 0.0);
    }

    #[test]
    fn test_safe_area_insets_uniform() {
        let insets = SafeAreaInsets::uniform(10.0);
        assert!(!insets.is_zero());
        assert_eq!(insets.top, 10.0);
        assert_eq!(insets.bottom, 10.0);
        assert_eq!(insets.left, 10.0);
        assert_eq!(insets.right, 10.0);
    }

    #[test]
    fn test_safe_area_insets_totals() {
        let insets = SafeAreaInsets::new(10.0, 20.0, 5.0, 15.0);
        assert_eq!(insets.horizontal_total(), 20.0);
        assert_eq!(insets.vertical_total(), 30.0);
    }

    #[test]
    fn test_safe_area_insets_scaled() {
        let insets = SafeAreaInsets::new(10.0, 20.0, 5.0, 15.0);
        let scaled = insets.scaled(2.0);
        assert_eq!(scaled.top, 20.0);
        assert_eq!(scaled.bottom, 40.0);
        assert_eq!(scaled.left, 10.0);
        assert_eq!(scaled.right, 30.0);
    }

    #[test]
    fn test_safe_area_insets_with_edges() {
        let insets = SafeAreaInsets::new(47.0, 34.0, 10.0, 10.0);

        let vertical_only = insets.with_edges(SafeAreaEdges::VERTICAL);
        assert_eq!(vertical_only.top, 47.0);
        assert_eq!(vertical_only.bottom, 34.0);
        assert_eq!(vertical_only.left, 0.0);
        assert_eq!(vertical_only.right, 0.0);

        let top_only = insets.with_edges(SafeAreaEdges::TOP);
        assert_eq!(top_only.top, 47.0);
        assert_eq!(top_only.bottom, 0.0);
        assert_eq!(top_only.left, 0.0);
        assert_eq!(top_only.right, 0.0);
    }

    #[test]
    fn test_safe_area_insets_max() {
        let a = SafeAreaInsets::new(10.0, 20.0, 5.0, 15.0);
        let b = SafeAreaInsets::new(15.0, 10.0, 10.0, 5.0);
        let max = a.max(&b);
        assert_eq!(max.top, 15.0);
        assert_eq!(max.bottom, 20.0);
        assert_eq!(max.left, 10.0);
        assert_eq!(max.right, 15.0);
    }

    #[test]
    fn test_safe_area_edges_flags() {
        let all = SafeAreaEdges::ALL;
        assert!(all.contains(SafeAreaEdges::TOP));
        assert!(all.contains(SafeAreaEdges::BOTTOM));
        assert!(all.contains(SafeAreaEdges::LEFT));
        assert!(all.contains(SafeAreaEdges::RIGHT));

        let vertical = SafeAreaEdges::VERTICAL;
        assert!(vertical.contains(SafeAreaEdges::TOP));
        assert!(vertical.contains(SafeAreaEdges::BOTTOM));
        assert!(!vertical.contains(SafeAreaEdges::LEFT));
        assert!(!vertical.contains(SafeAreaEdges::RIGHT));
    }

    #[test]
    fn test_safe_area_container() {
        let container = SafeAreaContainer::new()
            .with_insets(SafeAreaInsets::new(47.0, 34.0, 0.0, 0.0))
            .with_edges(SafeAreaEdges::VERTICAL);

        let effective = container.effective_insets();
        assert_eq!(effective.top, 47.0);
        assert_eq!(effective.bottom, 34.0);
        assert_eq!(effective.left, 0.0);
        assert_eq!(effective.right, 0.0);
    }

    #[test]
    fn test_safe_area_container_with_minimum_padding() {
        let container = SafeAreaContainer::new()
            .with_insets(SafeAreaInsets::new(10.0, 5.0, 0.0, 0.0))
            .with_minimum_padding(SafeAreaInsets::uniform(20.0))
            .with_edges(SafeAreaEdges::ALL);

        let effective = container.effective_insets();
        // Safe area is 10, but minimum is 20, so should be 20
        assert_eq!(effective.top, 20.0);
        // Safe area is 5, but minimum is 20, so should be 20
        assert_eq!(effective.bottom, 20.0);
        // Safe area is 0, but minimum is 20, so should be 20
        assert_eq!(effective.left, 20.0);
        assert_eq!(effective.right, 20.0);
    }

    #[test]
    fn test_presets() {
        let iphone = presets::iphone_notch();
        assert_eq!(iphone.top, 47.0);
        assert_eq!(iphone.bottom, 34.0);

        let dynamic_island = presets::iphone_dynamic_island();
        assert_eq!(dynamic_island.top, 59.0);

        let classic = presets::iphone_classic();
        assert_eq!(classic.top, 20.0);
        assert_eq!(classic.bottom, 0.0);
    }

    #[test]
    fn test_container_edge_checks() {
        let container = SafeAreaContainer::new().with_edges(SafeAreaEdges::TOP | SafeAreaEdges::LEFT);

        assert!(container.has_top());
        assert!(!container.has_bottom());
        assert!(container.has_left());
        assert!(!container.has_right());
    }
}
