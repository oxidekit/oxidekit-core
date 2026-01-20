//! Animated Values and Interpolation
//!
//! This module provides implicit animations similar to Flutter's AnimatedContainer.
//! `AnimatedValue<T>` automatically interpolates between values when changed.
//!
//! # Example
//!
//! ```rust
//! use oxide_animation::{AnimatedValue, Curve};
//! use std::time::Duration;
//!
//! let mut opacity = AnimatedValue::new(0.0_f32)
//!     .duration(Duration::from_millis(300))
//!     .curve(Curve::EaseOut);
//!
//! // Set new target value - interpolation happens automatically
//! opacity.set(1.0);
//!
//! // Update each frame
//! let current = opacity.tick(0.016); // Returns interpolated value
//! ```

use crate::curve::Curve;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;

// =============================================================================
// Animatable Trait
// =============================================================================

/// Trait for types that can be animated (interpolated between values)
pub trait Animatable: Clone + Debug + Send + Sync + 'static {
    /// Interpolate between self and other by factor t (0.0 = self, 1.0 = other)
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

// Implement Animatable for primitive types

impl Animatable for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Animatable for f64 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

impl Animatable for i32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (*self as f32 + (*other - *self) as f32 * t).round() as i32
    }
}

impl Animatable for u8 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let diff = *other as f32 - *self as f32;
        (*self as f32 + diff * t).clamp(0.0, 255.0) as u8
    }
}

// =============================================================================
// Color Type
// =============================================================================

/// RGBA color with floating-point components (0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGBA components (0.0-1.0)
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create an opaque color from RGB components (0.0-1.0)
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create from u8 components (0-255)
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Create from hex string (e.g., "#FF5500" or "#FF550080")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::from_u8(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::from_u8(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert to u8 array [r, g, b, a]
    pub fn to_u8_array(&self) -> [u8; 4] {
        [
            (self.r * 255.0).clamp(0.0, 255.0) as u8,
            (self.g * 255.0).clamp(0.0, 255.0) as u8,
            (self.b * 255.0).clamp(0.0, 255.0) as u8,
            (self.a * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        let [r, g, b, a] = self.to_u8_array();
        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }

    /// Common colors
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Animatable for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        // Interpolate in linear color space for perceptually correct blending
        Self {
            r: self.r.lerp(&other.r, t),
            g: self.g.lerp(&other.g, t),
            b: self.b.lerp(&other.b, t),
            a: self.a.lerp(&other.a, t),
        }
    }
}

// =============================================================================
// Size Type
// =============================================================================

/// 2D size with width and height
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Create a new size
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a square size
    pub const fn square(size: f32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }

    /// Zero size
    pub const ZERO: Self = Self::new(0.0, 0.0);
}

impl Animatable for Size {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            width: self.width.lerp(&other.width, t),
            height: self.height.lerp(&other.height, t),
        }
    }
}

// =============================================================================
// Offset Type
// =============================================================================

/// 2D offset/point with x and y coordinates
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    /// Create a new offset
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero offset (origin)
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Distance from origin
    pub fn distance(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Distance to another offset
    pub fn distance_to(&self, other: &Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Animatable for Offset {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x.lerp(&other.x, t),
            y: self.y.lerp(&other.y, t),
        }
    }
}

// =============================================================================
// Rect Type
// =============================================================================

/// Rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create from position and size
    pub const fn from_pos_size(pos: Offset, size: Size) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Get the position (top-left corner)
    pub fn position(&self) -> Offset {
        Offset::new(self.x, self.y)
    }

    /// Get the size
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Get the center point
    pub fn center(&self) -> Offset {
        Offset::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Zero rectangle
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);
}

impl Animatable for Rect {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x.lerp(&other.x, t),
            y: self.y.lerp(&other.y, t),
            width: self.width.lerp(&other.width, t),
            height: self.height.lerp(&other.height, t),
        }
    }
}

// =============================================================================
// AnimatedValue - Implicit Animation
// =============================================================================

/// Status of an animated value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatedValueStatus {
    /// Not currently animating
    Idle,
    /// Currently animating towards target
    Animating,
    /// Animation completed
    Completed,
}

/// An implicitly animated value that automatically interpolates when changed.
///
/// Similar to Flutter's `AnimatedContainer`, this provides automatic animations
/// when the target value changes.
#[derive(Debug, Clone)]
pub struct AnimatedValue<T: Animatable> {
    /// Current interpolated value
    current: T,
    /// Starting value for current animation
    from: T,
    /// Target value
    target: T,
    /// Animation duration
    duration: Duration,
    /// Easing curve
    curve: Curve,
    /// Delay before starting animation
    delay: Duration,
    /// Current elapsed time (seconds)
    elapsed: f32,
    /// Time spent in delay
    delay_elapsed: f32,
    /// Current status
    status: AnimatedValueStatus,
}

impl<T: Animatable> AnimatedValue<T> {
    /// Create a new animated value with an initial value
    pub fn new(initial: T) -> Self {
        Self {
            current: initial.clone(),
            from: initial.clone(),
            target: initial,
            duration: Duration::from_millis(300),
            curve: Curve::EaseInOut,
            delay: Duration::ZERO,
            elapsed: 0.0,
            delay_elapsed: 0.0,
            status: AnimatedValueStatus::Idle,
        }
    }

    /// Set the animation duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the animation duration in milliseconds
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration = Duration::from_millis(ms);
        self
    }

    /// Set the easing curve
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    /// Set the delay before animation starts
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Set the delay in milliseconds
    pub fn delay_ms(mut self, ms: u64) -> Self {
        self.delay = Duration::from_millis(ms);
        self
    }

    /// Set a new target value, triggering animation
    pub fn set(&mut self, value: T) {
        self.from = self.current.clone();
        self.target = value;
        self.elapsed = 0.0;
        self.delay_elapsed = 0.0;
        self.status = AnimatedValueStatus::Animating;
    }

    /// Set value immediately without animation
    pub fn set_immediate(&mut self, value: T) {
        self.current = value.clone();
        self.from = value.clone();
        self.target = value;
        self.elapsed = 0.0;
        self.delay_elapsed = 0.0;
        self.status = AnimatedValueStatus::Idle;
    }

    /// Get the current interpolated value
    pub fn get(&self) -> &T {
        &self.current
    }

    /// Get the target value
    pub fn target(&self) -> &T {
        &self.target
    }

    /// Check if currently animating
    pub fn is_animating(&self) -> bool {
        self.status == AnimatedValueStatus::Animating
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.status == AnimatedValueStatus::Completed || self.status == AnimatedValueStatus::Idle
    }

    /// Get the current status
    pub fn status(&self) -> AnimatedValueStatus {
        self.status
    }

    /// Get animation progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.status == AnimatedValueStatus::Idle {
            return 1.0;
        }
        let duration_secs = self.duration.as_secs_f32();
        if duration_secs <= 0.0 {
            return 1.0;
        }
        (self.elapsed / duration_secs).clamp(0.0, 1.0)
    }

    /// Update the animation by a time delta (in seconds)
    /// Returns the current interpolated value
    pub fn tick(&mut self, dt: f32) -> T {
        if self.status != AnimatedValueStatus::Animating {
            return self.current.clone();
        }

        // Handle delay
        if self.delay_elapsed < self.delay.as_secs_f32() {
            self.delay_elapsed += dt;
            if self.delay_elapsed < self.delay.as_secs_f32() {
                return self.current.clone();
            }
            // Carry over extra time past delay
            let extra = self.delay_elapsed - self.delay.as_secs_f32();
            self.elapsed = extra;
        } else {
            self.elapsed += dt;
        }

        let duration_secs = self.duration.as_secs_f32();
        let progress = if duration_secs <= 0.0 {
            1.0
        } else {
            (self.elapsed / duration_secs).clamp(0.0, 1.0)
        };

        // Apply easing curve
        let eased = self.curve.transform(progress);

        // Interpolate value
        self.current = self.from.lerp(&self.target, eased);

        // Check completion
        if progress >= 1.0 {
            self.current = self.target.clone();
            self.status = AnimatedValueStatus::Completed;
        }

        self.current.clone()
    }

    /// Skip to the end of the animation
    pub fn complete(&mut self) {
        self.current = self.target.clone();
        self.elapsed = self.duration.as_secs_f32();
        self.status = AnimatedValueStatus::Completed;
    }

    /// Reset to the starting value
    pub fn reset(&mut self) {
        self.current = self.from.clone();
        self.elapsed = 0.0;
        self.delay_elapsed = 0.0;
        self.status = AnimatedValueStatus::Idle;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_lerp() {
        let a = 0.0_f32;
        let b = 100.0_f32;
        assert!((a.lerp(&b, 0.0) - 0.0).abs() < 0.001);
        assert!((a.lerp(&b, 0.5) - 50.0).abs() < 0.001);
        assert!((a.lerp(&b, 1.0) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_color_lerp() {
        let black = Color::BLACK;
        let white = Color::WHITE;
        let mid = black.lerp(&white, 0.5);
        assert!((mid.r - 0.5).abs() < 0.001);
        assert!((mid.g - 0.5).abs() < 0.001);
        assert!((mid.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex("#FF5500").unwrap();
        assert_eq!((c.r * 255.0).round() as u8, 255);
        assert_eq!((c.g * 255.0).round() as u8, 85);
        assert_eq!((c.b * 255.0).round() as u8, 0);

        let c2 = Color::from_hex("#FF550080").unwrap();
        assert_eq!((c2.a * 255.0).round() as u8, 128);
    }

    #[test]
    fn test_size_lerp() {
        let a = Size::new(0.0, 0.0);
        let b = Size::new(100.0, 200.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.width - 50.0).abs() < 0.001);
        assert!((mid.height - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_offset_lerp() {
        let a = Offset::new(0.0, 0.0);
        let b = Offset::new(100.0, 200.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!((mid.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_rect_lerp() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(100.0, 100.0, 200.0, 200.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!((mid.y - 50.0).abs() < 0.001);
        assert!((mid.width - 150.0).abs() < 0.001);
        assert!((mid.height - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_animated_value_basic() {
        let mut value = AnimatedValue::new(0.0_f32)
            .duration(Duration::from_secs(1))
            .curve(Curve::Linear);

        // Initially at start value
        assert!((value.get() - 0.0).abs() < 0.001);
        assert!(!value.is_animating());

        // Set target
        value.set(100.0);
        assert!(value.is_animating());

        // Tick halfway
        value.tick(0.5);
        assert!((value.get() - 50.0).abs() < 0.001);

        // Tick to completion
        value.tick(0.5);
        assert!((value.get() - 100.0).abs() < 0.001);
        assert!(!value.is_animating());
    }

    #[test]
    fn test_animated_value_with_delay() {
        let mut value = AnimatedValue::new(0.0_f32)
            .duration(Duration::from_secs(1))
            .delay(Duration::from_millis(500))
            .curve(Curve::Linear);

        value.set(100.0);

        // During delay, value shouldn't change
        value.tick(0.25);
        assert!((value.get() - 0.0).abs() < 0.001);

        // Past delay, animation should start
        value.tick(0.5);
        let progress_time = 0.25; // 0.75 - 0.5 delay = 0.25 of animation
        assert!((value.get() - 25.0).abs() < 1.0);
    }

    #[test]
    fn test_animated_value_set_immediate() {
        let mut value = AnimatedValue::new(0.0_f32);
        value.set_immediate(100.0);
        assert!((value.get() - 100.0).abs() < 0.001);
        assert!(!value.is_animating());
    }

    #[test]
    fn test_animated_value_complete() {
        let mut value = AnimatedValue::new(0.0_f32).duration(Duration::from_secs(10));

        value.set(100.0);
        value.tick(0.1); // Small tick

        value.complete();
        assert!((value.get() - 100.0).abs() < 0.001);
        assert!(!value.is_animating());
    }

    #[test]
    fn test_animated_value_reset() {
        let mut value = AnimatedValue::new(0.0_f32).duration(Duration::from_secs(1));

        value.set(100.0);
        value.tick(0.5);

        value.reset();
        assert!((value.get() - 0.0).abs() < 0.001);
        assert!(!value.is_animating());
    }

    #[test]
    fn test_animated_color() {
        let mut color = AnimatedValue::new(Color::BLACK)
            .duration(Duration::from_secs(1))
            .curve(Curve::Linear);

        color.set(Color::WHITE);
        color.tick(0.5);

        let mid = color.get();
        assert!((mid.r - 0.5).abs() < 0.001);
        assert!((mid.g - 0.5).abs() < 0.001);
        assert!((mid.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_offset_distance() {
        let a = Offset::new(0.0, 0.0);
        let b = Offset::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 0.001);
        assert!((b.distance() - 5.0).abs() < 0.001);
    }
}
