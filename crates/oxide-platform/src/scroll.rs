//! Scrolling Behavior
//!
//! Provides platform-specific scrolling behaviors including:
//! - iOS: Rubber-band bouncing, momentum scrolling
//! - Android: Edge glow effect (overscroll)
//! - Desktop: Smooth scrolling, scroll bars

use crate::detect::Platform;
use serde::{Deserialize, Serialize};

/// Overscroll effect style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverscrollEffect {
    /// Rubber-band bounce (iOS/macOS style)
    RubberBand,
    /// Edge glow/ripple (Android style)
    EdgeGlow,
    /// No overscroll effect (clamp)
    None,
}

impl OverscrollEffect {
    /// Get the default overscroll effect for a platform.
    #[must_use]
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::MacOS | Platform::IOS => OverscrollEffect::RubberBand,
            Platform::Android => OverscrollEffect::EdgeGlow,
            _ => OverscrollEffect::None,
        }
    }

    /// Get the default overscroll effect for the current platform.
    #[must_use]
    pub fn current() -> Self {
        Self::for_platform(Platform::current())
    }
}

/// Scroll physics configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollPhysics {
    /// Overscroll effect type
    pub overscroll_effect: OverscrollEffect,
    /// Friction coefficient for momentum scrolling (0.0-1.0)
    pub friction: f32,
    /// Velocity threshold to start momentum scroll
    pub velocity_threshold: f32,
    /// Maximum overscroll distance (in pixels or percentage)
    pub max_overscroll: f32,
    /// Rubber band tension (for iOS-style bounce)
    pub rubber_band_tension: f32,
    /// Edge glow intensity (for Android-style)
    pub edge_glow_intensity: f32,
    /// Enable momentum scrolling
    pub momentum_enabled: bool,
    /// Enable scroll snapping
    pub snap_enabled: bool,
}

impl ScrollPhysics {
    /// Create scroll physics for a specific platform.
    #[must_use]
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::IOS => Self::ios(),
            Platform::Android => Self::android(),
            Platform::MacOS => Self::macos(),
            Platform::Windows | Platform::Linux => Self::desktop(),
            Platform::Web => Self::web(),
            Platform::Unknown => Self::default(),
        }
    }

    /// Create iOS-style scroll physics.
    #[must_use]
    pub fn ios() -> Self {
        Self {
            overscroll_effect: OverscrollEffect::RubberBand,
            friction: 0.015,
            velocity_threshold: 50.0,
            max_overscroll: 120.0,
            rubber_band_tension: 0.55,
            edge_glow_intensity: 0.0,
            momentum_enabled: true,
            snap_enabled: false,
        }
    }

    /// Create Android-style scroll physics.
    #[must_use]
    pub fn android() -> Self {
        Self {
            overscroll_effect: OverscrollEffect::EdgeGlow,
            friction: 0.015,
            velocity_threshold: 50.0,
            max_overscroll: 0.0,
            rubber_band_tension: 0.0,
            edge_glow_intensity: 1.0,
            momentum_enabled: true,
            snap_enabled: false,
        }
    }

    /// Create macOS-style scroll physics.
    #[must_use]
    pub fn macos() -> Self {
        Self {
            overscroll_effect: OverscrollEffect::RubberBand,
            friction: 0.02,
            velocity_threshold: 30.0,
            max_overscroll: 80.0,
            rubber_band_tension: 0.6,
            edge_glow_intensity: 0.0,
            momentum_enabled: true,
            snap_enabled: false,
        }
    }

    /// Create desktop-style scroll physics (Windows/Linux).
    #[must_use]
    pub fn desktop() -> Self {
        Self {
            overscroll_effect: OverscrollEffect::None,
            friction: 0.03,
            velocity_threshold: 20.0,
            max_overscroll: 0.0,
            rubber_band_tension: 0.0,
            edge_glow_intensity: 0.0,
            momentum_enabled: true,
            snap_enabled: false,
        }
    }

    /// Create web-style scroll physics.
    #[must_use]
    pub fn web() -> Self {
        Self {
            overscroll_effect: OverscrollEffect::None,
            friction: 0.02,
            velocity_threshold: 30.0,
            max_overscroll: 0.0,
            rubber_band_tension: 0.0,
            edge_glow_intensity: 0.0,
            momentum_enabled: true,
            snap_enabled: false,
        }
    }

    /// Create physics with no overscroll.
    #[must_use]
    pub fn clamped() -> Self {
        Self {
            overscroll_effect: OverscrollEffect::None,
            friction: 0.02,
            velocity_threshold: 30.0,
            max_overscroll: 0.0,
            rubber_band_tension: 0.0,
            edge_glow_intensity: 0.0,
            momentum_enabled: true,
            snap_enabled: false,
        }
    }

    /// Get physics for the current platform.
    #[must_use]
    pub fn current() -> Self {
        Self::for_platform(Platform::current())
    }
}

impl Default for ScrollPhysics {
    fn default() -> Self {
        Self::current()
    }
}

/// Scroll state tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollState {
    /// Current scroll offset
    pub offset: ScrollOffset,
    /// Current scroll velocity
    pub velocity: ScrollVelocity,
    /// Content size (scrollable area)
    pub content_size: ScrollSize,
    /// Viewport size (visible area)
    pub viewport_size: ScrollSize,
    /// Whether currently dragging
    pub is_dragging: bool,
    /// Whether momentum scroll is active
    pub is_momentum_scrolling: bool,
    /// Current overscroll amount
    pub overscroll: ScrollOffset,
}

impl ScrollState {
    /// Create a new scroll state.
    #[must_use]
    pub fn new(content_size: ScrollSize, viewport_size: ScrollSize) -> Self {
        Self {
            offset: ScrollOffset::zero(),
            velocity: ScrollVelocity::zero(),
            content_size,
            viewport_size,
            is_dragging: false,
            is_momentum_scrolling: false,
            overscroll: ScrollOffset::zero(),
        }
    }

    /// Get the maximum scroll offset.
    #[must_use]
    pub fn max_offset(&self) -> ScrollOffset {
        ScrollOffset {
            x: (self.content_size.width - self.viewport_size.width).max(0.0),
            y: (self.content_size.height - self.viewport_size.height).max(0.0),
        }
    }

    /// Check if at the start (top/left) of scroll.
    #[must_use]
    pub fn is_at_start(&self) -> bool {
        self.offset.x <= 0.0 && self.offset.y <= 0.0
    }

    /// Check if at the end (bottom/right) of scroll.
    #[must_use]
    pub fn is_at_end(&self) -> bool {
        let max = self.max_offset();
        self.offset.x >= max.x && self.offset.y >= max.y
    }

    /// Check if content is scrollable vertically.
    #[must_use]
    pub fn is_scrollable_vertical(&self) -> bool {
        self.content_size.height > self.viewport_size.height
    }

    /// Check if content is scrollable horizontally.
    #[must_use]
    pub fn is_scrollable_horizontal(&self) -> bool {
        self.content_size.width > self.viewport_size.width
    }

    /// Get scroll progress (0.0-1.0) for vertical scroll.
    #[must_use]
    pub fn progress_vertical(&self) -> f32 {
        let max = self.max_offset().y;
        if max > 0.0 {
            (self.offset.y / max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Get scroll progress (0.0-1.0) for horizontal scroll.
    #[must_use]
    pub fn progress_horizontal(&self) -> f32 {
        let max = self.max_offset().x;
        if max > 0.0 {
            (self.offset.x / max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new(ScrollSize::zero(), ScrollSize::zero())
    }
}

/// 2D scroll offset.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ScrollOffset {
    /// Horizontal offset
    pub x: f32,
    /// Vertical offset
    pub y: f32,
}

impl ScrollOffset {
    /// Create a new scroll offset.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a zero offset.
    #[must_use]
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Clamp the offset within bounds.
    #[must_use]
    pub fn clamp(&self, min: ScrollOffset, max: ScrollOffset) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }
}

/// 2D scroll velocity.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ScrollVelocity {
    /// Horizontal velocity (pixels per second)
    pub x: f32,
    /// Vertical velocity (pixels per second)
    pub y: f32,
}

impl ScrollVelocity {
    /// Create a new scroll velocity.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a zero velocity.
    #[must_use]
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Get the magnitude of the velocity.
    #[must_use]
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// 2D scroll size.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ScrollSize {
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl ScrollSize {
    /// Create a new scroll size.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a zero size.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Rubber band bounce calculator for iOS-style overscroll.
#[derive(Debug, Clone)]
pub struct RubberBandCalculator {
    /// Tension coefficient
    pub tension: f32,
    /// Maximum distance
    pub max_distance: f32,
}

impl RubberBandCalculator {
    /// Create a new rubber band calculator.
    #[must_use]
    pub fn new(tension: f32, max_distance: f32) -> Self {
        Self {
            tension,
            max_distance,
        }
    }

    /// Calculate the rubber band offset for a given raw offset.
    ///
    /// This applies the characteristic iOS rubber band curve.
    #[must_use]
    pub fn calculate(&self, raw_offset: f32) -> f32 {
        if raw_offset.abs() < f32::EPSILON {
            return 0.0;
        }

        let sign = raw_offset.signum();
        let distance = raw_offset.abs();

        // Use the iOS rubber band formula:
        // result = (1 - (1 / (distance * c / d + 1))) * d
        // where c is coefficient and d is dimension
        let coefficient = self.tension;
        let dimension = self.max_distance;

        let result = (1.0 - (1.0 / (distance * coefficient / dimension + 1.0))) * dimension;
        result * sign
    }

    /// Calculate the animation spring back.
    #[must_use]
    pub fn spring_back(&self, current_offset: f32, dt: f32) -> f32 {
        // Simple spring animation toward zero
        let spring_constant = 200.0;
        let damping = 25.0;
        let velocity = -current_offset * spring_constant;
        let new_offset = current_offset + velocity * dt - current_offset * damping * dt;

        if new_offset.abs() < 0.5 {
            0.0
        } else {
            new_offset
        }
    }
}

impl Default for RubberBandCalculator {
    fn default() -> Self {
        Self::new(0.55, 120.0)
    }
}

/// Edge glow state for Android-style overscroll.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeGlowState {
    /// Top edge glow intensity (0.0-1.0)
    pub top: f32,
    /// Bottom edge glow intensity (0.0-1.0)
    pub bottom: f32,
    /// Left edge glow intensity (0.0-1.0)
    pub left: f32,
    /// Right edge glow intensity (0.0-1.0)
    pub right: f32,
}

impl EdgeGlowState {
    /// Create a new edge glow state with no glow.
    #[must_use]
    pub fn none() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }

    /// Update glow based on overscroll amount.
    pub fn update_from_overscroll(&mut self, overscroll: ScrollOffset, max_glow: f32) {
        // Top glow when scrolled past top
        self.top = if overscroll.y < 0.0 {
            (-overscroll.y / 100.0).min(max_glow)
        } else {
            (self.top - 0.1).max(0.0)
        };

        // Bottom glow when scrolled past bottom
        self.bottom = if overscroll.y > 0.0 {
            (overscroll.y / 100.0).min(max_glow)
        } else {
            (self.bottom - 0.1).max(0.0)
        };

        // Left glow
        self.left = if overscroll.x < 0.0 {
            (-overscroll.x / 100.0).min(max_glow)
        } else {
            (self.left - 0.1).max(0.0)
        };

        // Right glow
        self.right = if overscroll.x > 0.0 {
            (overscroll.x / 100.0).min(max_glow)
        } else {
            (self.right - 0.1).max(0.0)
        };
    }

    /// Check if any edge has glow.
    #[must_use]
    pub fn has_glow(&self) -> bool {
        self.top > 0.0 || self.bottom > 0.0 || self.left > 0.0 || self.right > 0.0
    }

    /// Decay all glow values.
    pub fn decay(&mut self, amount: f32) {
        self.top = (self.top - amount).max(0.0);
        self.bottom = (self.bottom - amount).max(0.0);
        self.left = (self.left - amount).max(0.0);
        self.right = (self.right - amount).max(0.0);
    }
}

impl Default for EdgeGlowState {
    fn default() -> Self {
        Self::none()
    }
}

/// Scrollbar appearance style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollbarStyle {
    /// Always visible
    AlwaysVisible,
    /// Visible only when scrolling
    AutoHide,
    /// Never visible
    Hidden,
    /// Thin overlay (iOS style)
    ThinOverlay,
}

impl ScrollbarStyle {
    /// Get the default scrollbar style for a platform.
    #[must_use]
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::IOS | Platform::Android => ScrollbarStyle::ThinOverlay,
            Platform::MacOS => ScrollbarStyle::AutoHide,
            Platform::Windows | Platform::Linux => ScrollbarStyle::AlwaysVisible,
            Platform::Web | Platform::Unknown => ScrollbarStyle::AutoHide,
        }
    }

    /// Get the default scrollbar style for the current platform.
    #[must_use]
    pub fn current() -> Self {
        Self::for_platform(Platform::current())
    }
}

/// Scrollbar configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollbarConfig {
    /// Scrollbar style
    pub style: ScrollbarStyle,
    /// Width of the scrollbar (in pixels)
    pub width: f32,
    /// Minimum thumb size (in pixels)
    pub min_thumb_size: f32,
    /// Border radius of the thumb
    pub thumb_radius: f32,
    /// Auto-hide delay (in milliseconds)
    pub auto_hide_delay_ms: u32,
    /// Show horizontal scrollbar
    pub show_horizontal: bool,
    /// Show vertical scrollbar
    pub show_vertical: bool,
}

impl ScrollbarConfig {
    /// Create scrollbar config for a specific platform.
    #[must_use]
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::IOS | Platform::Android => Self {
                style: ScrollbarStyle::ThinOverlay,
                width: 3.0,
                min_thumb_size: 30.0,
                thumb_radius: 1.5,
                auto_hide_delay_ms: 500,
                show_horizontal: false,
                show_vertical: true,
            },
            Platform::MacOS => Self {
                style: ScrollbarStyle::AutoHide,
                width: 8.0,
                min_thumb_size: 20.0,
                thumb_radius: 4.0,
                auto_hide_delay_ms: 1000,
                show_horizontal: true,
                show_vertical: true,
            },
            Platform::Windows => Self {
                style: ScrollbarStyle::AlwaysVisible,
                width: 17.0,
                min_thumb_size: 40.0,
                thumb_radius: 0.0,
                auto_hide_delay_ms: 0,
                show_horizontal: true,
                show_vertical: true,
            },
            Platform::Linux => Self {
                style: ScrollbarStyle::AlwaysVisible,
                width: 12.0,
                min_thumb_size: 30.0,
                thumb_radius: 0.0,
                auto_hide_delay_ms: 0,
                show_horizontal: true,
                show_vertical: true,
            },
            Platform::Web | Platform::Unknown => Self {
                style: ScrollbarStyle::AutoHide,
                width: 8.0,
                min_thumb_size: 20.0,
                thumb_radius: 4.0,
                auto_hide_delay_ms: 1000,
                show_horizontal: true,
                show_vertical: true,
            },
        }
    }

    /// Get scrollbar config for the current platform.
    #[must_use]
    pub fn current() -> Self {
        Self::for_platform(Platform::current())
    }
}

impl Default for ScrollbarConfig {
    fn default() -> Self {
        Self::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overscroll_effect_for_platform() {
        assert_eq!(OverscrollEffect::for_platform(Platform::IOS), OverscrollEffect::RubberBand);
        assert_eq!(OverscrollEffect::for_platform(Platform::MacOS), OverscrollEffect::RubberBand);
        assert_eq!(OverscrollEffect::for_platform(Platform::Android), OverscrollEffect::EdgeGlow);
        assert_eq!(OverscrollEffect::for_platform(Platform::Windows), OverscrollEffect::None);
    }

    #[test]
    fn test_scroll_physics_ios() {
        let physics = ScrollPhysics::ios();
        assert_eq!(physics.overscroll_effect, OverscrollEffect::RubberBand);
        assert!(physics.momentum_enabled);
        assert!(physics.max_overscroll > 0.0);
    }

    #[test]
    fn test_scroll_physics_android() {
        let physics = ScrollPhysics::android();
        assert_eq!(physics.overscroll_effect, OverscrollEffect::EdgeGlow);
        assert!(physics.edge_glow_intensity > 0.0);
    }

    #[test]
    fn test_scroll_state_max_offset() {
        let state = ScrollState::new(
            ScrollSize::new(100.0, 500.0),
            ScrollSize::new(100.0, 200.0),
        );
        let max = state.max_offset();
        assert!((max.x).abs() < f32::EPSILON);
        assert!((max.y - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scroll_state_progress() {
        let mut state = ScrollState::new(
            ScrollSize::new(100.0, 500.0),
            ScrollSize::new(100.0, 200.0),
        );
        state.offset = ScrollOffset::new(0.0, 150.0);
        assert!((state.progress_vertical() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_rubber_band_calculator() {
        let calc = RubberBandCalculator::new(0.55, 120.0);

        // Small offset should be close to linear
        let small = calc.calculate(10.0);
        assert!(small > 0.0 && small < 10.0);

        // Large offset should be dampened significantly
        let large = calc.calculate(200.0);
        assert!(large < 200.0);
        assert!(large < calc.max_distance * 1.5);
    }

    #[test]
    fn test_edge_glow_update() {
        let mut glow = EdgeGlowState::none();
        glow.update_from_overscroll(ScrollOffset::new(0.0, -50.0), 1.0);
        assert!(glow.top > 0.0);
        assert!((glow.bottom).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scrollbar_style_platform() {
        assert_eq!(ScrollbarStyle::for_platform(Platform::IOS), ScrollbarStyle::ThinOverlay);
        assert_eq!(ScrollbarStyle::for_platform(Platform::Windows), ScrollbarStyle::AlwaysVisible);
        assert_eq!(ScrollbarStyle::for_platform(Platform::MacOS), ScrollbarStyle::AutoHide);
    }

    #[test]
    fn test_scrollbar_config_width() {
        let ios_config = ScrollbarConfig::for_platform(Platform::IOS);
        let windows_config = ScrollbarConfig::for_platform(Platform::Windows);
        assert!(ios_config.width < windows_config.width);
    }
}
