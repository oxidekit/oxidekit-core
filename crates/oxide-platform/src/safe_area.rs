//! Safe Area Insets
//!
//! Provides platform-specific safe area handling including:
//! - iOS notch/Dynamic Island handling
//! - Android punch-hole cameras
//! - Desktop: No safe areas needed

use crate::detect::Platform;
use serde::{Deserialize, Serialize};

/// Safe area insets representing areas that should not be obscured.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SafeAreaInsets {
    /// Top inset (status bar, notch, Dynamic Island)
    pub top: f32,
    /// Bottom inset (home indicator, navigation bar)
    pub bottom: f32,
    /// Left inset (notch in landscape, punch-hole camera)
    pub left: f32,
    /// Right inset (notch in landscape, punch-hole camera)
    pub right: f32,
}

impl SafeAreaInsets {
    /// Create new safe area insets.
    #[must_use]
    pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Create zero insets (no safe area needed).
    #[must_use]
    pub fn zero() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }

    /// Create insets for a specific device type.
    #[must_use]
    pub fn for_device(device: DeviceSafeArea) -> Self {
        match device {
            DeviceSafeArea::IPhoneNotch => Self::new(47.0, 34.0, 0.0, 0.0),
            DeviceSafeArea::IPhoneDynamicIsland => Self::new(59.0, 34.0, 0.0, 0.0),
            DeviceSafeArea::IPhoneLegacy => Self::new(20.0, 0.0, 0.0, 0.0),
            DeviceSafeArea::IPadModern => Self::new(24.0, 20.0, 0.0, 0.0),
            DeviceSafeArea::AndroidPunchHole => Self::new(32.0, 0.0, 0.0, 0.0),
            DeviceSafeArea::AndroidPill => Self::new(0.0, 48.0, 0.0, 0.0),
            DeviceSafeArea::AndroidWaterfall => Self::new(0.0, 0.0, 8.0, 8.0),
            DeviceSafeArea::MacNotch => Self::new(0.0, 0.0, 0.0, 0.0), // Handled by menu bar
            DeviceSafeArea::Desktop => Self::zero(),
            DeviceSafeArea::Custom(insets) => insets,
        }
    }

    /// Get default insets for a platform.
    #[must_use]
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::IOS => Self::for_device(DeviceSafeArea::IPhoneNotch),
            Platform::Android => Self::for_device(DeviceSafeArea::AndroidPunchHole),
            Platform::MacOS => Self::for_device(DeviceSafeArea::Desktop),
            Platform::Windows | Platform::Linux => Self::for_device(DeviceSafeArea::Desktop),
            Platform::Web | Platform::Unknown => Self::zero(),
        }
    }

    /// Get the current platform's default insets.
    #[must_use]
    pub fn current() -> Self {
        Self::for_platform(Platform::current())
    }

    /// Check if there are any insets.
    #[must_use]
    pub fn has_insets(&self) -> bool {
        self.top > 0.0 || self.bottom > 0.0 || self.left > 0.0 || self.right > 0.0
    }

    /// Get total horizontal insets.
    #[must_use]
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical insets.
    #[must_use]
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    /// Combine with another set of insets (take maximum).
    #[must_use]
    pub fn union(&self, other: &SafeAreaInsets) -> Self {
        Self {
            top: self.top.max(other.top),
            bottom: self.bottom.max(other.bottom),
            left: self.left.max(other.left),
            right: self.right.max(other.right),
        }
    }

    /// Add additional padding to all sides.
    #[must_use]
    pub fn with_padding(&self, padding: f32) -> Self {
        Self {
            top: self.top + padding,
            bottom: self.bottom + padding,
            left: self.left + padding,
            right: self.right + padding,
        }
    }

    /// Calculate the safe rectangle within a given frame.
    #[must_use]
    pub fn safe_rect(&self, frame_width: f32, frame_height: f32) -> SafeRect {
        SafeRect {
            x: self.left,
            y: self.top,
            width: (frame_width - self.horizontal()).max(0.0),
            height: (frame_height - self.vertical()).max(0.0),
        }
    }
}

/// A rectangle representing the safe area.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SafeRect {
    /// X origin
    pub x: f32,
    /// Y origin
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl SafeRect {
    /// Create a new safe rect.
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get the right edge.
    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get the bottom edge.
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Get the center point.
    #[must_use]
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Check if a point is inside the rect.
    #[must_use]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.right() && y >= self.y && y <= self.bottom()
    }
}

/// Known device safe area configurations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceSafeArea {
    /// iPhone with notch (iPhone X - 13)
    IPhoneNotch,
    /// iPhone with Dynamic Island (iPhone 14 Pro+)
    IPhoneDynamicIsland,
    /// iPhone without notch (iPhone 8 and earlier, SE)
    IPhoneLegacy,
    /// Modern iPad with rounded corners
    IPadModern,
    /// Android with punch-hole camera
    AndroidPunchHole,
    /// Android with pill-shaped navigation
    AndroidPill,
    /// Android with waterfall display (curved edges)
    AndroidWaterfall,
    /// Mac with notch (MacBook Pro 2021+)
    MacNotch,
    /// Desktop without safe areas
    Desktop,
    /// Custom safe area
    Custom(SafeAreaInsets),
}

/// Safe area edges that can be selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SafeAreaEdges {
    /// Include top edge
    pub top: bool,
    /// Include bottom edge
    pub bottom: bool,
    /// Include left edge
    pub left: bool,
    /// Include right edge
    pub right: bool,
}

impl SafeAreaEdges {
    /// All edges.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            top: true,
            bottom: true,
            left: true,
            right: true,
        }
    }

    /// No edges.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            top: false,
            bottom: false,
            left: false,
            right: false,
        }
    }

    /// Only top edge.
    #[must_use]
    pub const fn top() -> Self {
        Self {
            top: true,
            bottom: false,
            left: false,
            right: false,
        }
    }

    /// Only bottom edge.
    #[must_use]
    pub const fn bottom() -> Self {
        Self {
            top: false,
            bottom: true,
            left: false,
            right: false,
        }
    }

    /// Vertical edges (top and bottom).
    #[must_use]
    pub const fn vertical() -> Self {
        Self {
            top: true,
            bottom: true,
            left: false,
            right: false,
        }
    }

    /// Horizontal edges (left and right).
    #[must_use]
    pub const fn horizontal() -> Self {
        Self {
            top: false,
            bottom: false,
            left: true,
            right: true,
        }
    }

    /// Apply these edges to insets (zero out non-selected edges).
    #[must_use]
    pub fn apply(&self, insets: SafeAreaInsets) -> SafeAreaInsets {
        SafeAreaInsets {
            top: if self.top { insets.top } else { 0.0 },
            bottom: if self.bottom { insets.bottom } else { 0.0 },
            left: if self.left { insets.left } else { 0.0 },
            right: if self.right { insets.right } else { 0.0 },
        }
    }
}

impl Default for SafeAreaEdges {
    fn default() -> Self {
        Self::all()
    }
}

/// Safe area provider for managing dynamic safe area changes.
#[derive(Debug, Clone)]
pub struct SafeAreaProvider {
    /// Current insets
    insets: SafeAreaInsets,
    /// Whether insets are user-defined
    custom: bool,
    /// Keyboard inset (added to bottom when keyboard visible)
    keyboard_inset: f32,
}

impl SafeAreaProvider {
    /// Create a new safe area provider with platform defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            insets: SafeAreaInsets::current(),
            custom: false,
            keyboard_inset: 0.0,
        }
    }

    /// Create a provider with specific insets.
    #[must_use]
    pub fn with_insets(insets: SafeAreaInsets) -> Self {
        Self {
            insets,
            custom: true,
            keyboard_inset: 0.0,
        }
    }

    /// Get the current insets (including keyboard if visible).
    #[must_use]
    pub fn insets(&self) -> SafeAreaInsets {
        SafeAreaInsets {
            top: self.insets.top,
            bottom: self.insets.bottom + self.keyboard_inset,
            left: self.insets.left,
            right: self.insets.right,
        }
    }

    /// Get the base insets (without keyboard).
    #[must_use]
    pub fn base_insets(&self) -> SafeAreaInsets {
        self.insets
    }

    /// Update the base insets.
    pub fn set_insets(&mut self, insets: SafeAreaInsets) {
        self.insets = insets;
        self.custom = true;
    }

    /// Update keyboard inset (call when keyboard shows/hides).
    pub fn set_keyboard_inset(&mut self, inset: f32) {
        self.keyboard_inset = inset;
    }

    /// Check if using custom insets.
    #[must_use]
    pub fn is_custom(&self) -> bool {
        self.custom
    }

    /// Reset to platform defaults.
    pub fn reset_to_default(&mut self) {
        self.insets = SafeAreaInsets::current();
        self.custom = false;
    }
}

impl Default for SafeAreaProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Display cutout information (notch, punch-hole, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayCutout {
    /// Type of cutout
    pub cutout_type: CutoutType,
    /// Bounding rectangle of the cutout
    pub bounds: SafeRect,
    /// Position of the cutout
    pub position: CutoutPosition,
}

impl DisplayCutout {
    /// Create a new display cutout.
    #[must_use]
    pub fn new(cutout_type: CutoutType, bounds: SafeRect, position: CutoutPosition) -> Self {
        Self {
            cutout_type,
            bounds,
            position,
        }
    }

    /// Create an iPhone notch cutout.
    #[must_use]
    pub fn iphone_notch(screen_width: f32) -> Self {
        let notch_width = 209.0;
        let notch_height = 30.0;
        Self::new(
            CutoutType::Notch,
            SafeRect::new(
                (screen_width - notch_width) / 2.0,
                0.0,
                notch_width,
                notch_height,
            ),
            CutoutPosition::Top,
        )
    }

    /// Create an iPhone Dynamic Island cutout.
    #[must_use]
    pub fn dynamic_island(screen_width: f32) -> Self {
        let island_width = 126.0;
        let island_height = 37.0;
        Self::new(
            CutoutType::DynamicIsland,
            SafeRect::new(
                (screen_width - island_width) / 2.0,
                11.0,
                island_width,
                island_height,
            ),
            CutoutPosition::Top,
        )
    }

    /// Create a punch-hole camera cutout.
    #[must_use]
    pub fn punch_hole(x: f32, y: f32, diameter: f32) -> Self {
        Self::new(
            CutoutType::PunchHole,
            SafeRect::new(x - diameter / 2.0, y - diameter / 2.0, diameter, diameter),
            if y < 50.0 {
                CutoutPosition::Top
            } else {
                CutoutPosition::Corner
            },
        )
    }
}

/// Type of display cutout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutoutType {
    /// Notch (iPhone X-13 style)
    Notch,
    /// Dynamic Island (iPhone 14 Pro+)
    DynamicIsland,
    /// Punch-hole camera
    PunchHole,
    /// Teardrop/waterdrop notch
    Teardrop,
    /// Wide notch
    WideNotch,
    /// No cutout
    None,
}

/// Position of display cutout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutoutPosition {
    /// Top center
    Top,
    /// Top left corner
    TopLeft,
    /// Top right corner
    TopRight,
    /// Corner (generic)
    Corner,
    /// No cutout position
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_area_insets_zero() {
        let insets = SafeAreaInsets::zero();
        assert!(!insets.has_insets());
        assert!((insets.horizontal()).abs() < f32::EPSILON);
        assert!((insets.vertical()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_safe_area_insets_iphone_notch() {
        let insets = SafeAreaInsets::for_device(DeviceSafeArea::IPhoneNotch);
        assert!(insets.has_insets());
        assert!(insets.top > 40.0);
        assert!(insets.bottom > 30.0);
    }

    #[test]
    fn test_safe_area_insets_desktop() {
        let insets = SafeAreaInsets::for_device(DeviceSafeArea::Desktop);
        assert!(!insets.has_insets());
    }

    #[test]
    fn test_safe_area_insets_union() {
        let a = SafeAreaInsets::new(10.0, 20.0, 5.0, 5.0);
        let b = SafeAreaInsets::new(15.0, 10.0, 10.0, 0.0);
        let union = a.union(&b);
        assert!((union.top - 15.0).abs() < f32::EPSILON);
        assert!((union.bottom - 20.0).abs() < f32::EPSILON);
        assert!((union.left - 10.0).abs() < f32::EPSILON);
        assert!((union.right - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_safe_rect_calculation() {
        let insets = SafeAreaInsets::new(50.0, 34.0, 0.0, 0.0);
        let rect = insets.safe_rect(375.0, 812.0);
        assert!((rect.x).abs() < f32::EPSILON);
        assert!((rect.y - 50.0).abs() < f32::EPSILON);
        assert!((rect.width - 375.0).abs() < f32::EPSILON);
        assert!((rect.height - 728.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_safe_rect_contains() {
        let rect = SafeRect::new(10.0, 10.0, 100.0, 100.0);
        assert!(rect.contains(50.0, 50.0));
        assert!(!rect.contains(5.0, 5.0));
        assert!(!rect.contains(150.0, 50.0));
    }

    #[test]
    fn test_safe_area_edges_apply() {
        let insets = SafeAreaInsets::new(50.0, 34.0, 10.0, 10.0);
        let filtered = SafeAreaEdges::vertical().apply(insets);
        assert!((filtered.top - 50.0).abs() < f32::EPSILON);
        assert!((filtered.bottom - 34.0).abs() < f32::EPSILON);
        assert!((filtered.left).abs() < f32::EPSILON);
        assert!((filtered.right).abs() < f32::EPSILON);
    }

    #[test]
    fn test_safe_area_provider_keyboard() {
        let mut provider = SafeAreaProvider::with_insets(SafeAreaInsets::new(50.0, 34.0, 0.0, 0.0));
        assert!((provider.insets().bottom - 34.0).abs() < f32::EPSILON);

        provider.set_keyboard_inset(300.0);
        assert!((provider.insets().bottom - 334.0).abs() < f32::EPSILON);
        assert!((provider.base_insets().bottom - 34.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_display_cutout_iphone() {
        let cutout = DisplayCutout::iphone_notch(375.0);
        assert_eq!(cutout.cutout_type, CutoutType::Notch);
        assert_eq!(cutout.position, CutoutPosition::Top);
        assert!(cutout.bounds.width > 200.0);
    }

    #[test]
    fn test_display_cutout_dynamic_island() {
        let cutout = DisplayCutout::dynamic_island(393.0);
        assert_eq!(cutout.cutout_type, CutoutType::DynamicIsland);
        assert!(cutout.bounds.y > 0.0); // Not flush with top
    }
}
