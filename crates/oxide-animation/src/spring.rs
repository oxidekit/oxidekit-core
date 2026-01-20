//! Spring Physics Animation
//!
//! This module provides physics-based spring animations including:
//! - Critically damped springs (smooth, no overshoot)
//! - Under-damped springs (bouncy, oscillating)
//! - Over-damped springs (slow, no bounce)
//!
//! # Example
//!
//! ```rust
//! use oxide_animation::{Spring, SpringConfig};
//!
//! // Create a bouncy spring
//! let mut spring = Spring::new(0.0, 100.0, SpringConfig::bouncy());
//!
//! // Update each frame
//! let value = spring.tick(0.016);
//! ```

use crate::value::Animatable;
use serde::{Deserialize, Serialize};

// =============================================================================
// Spring Configuration
// =============================================================================

/// Spring physics configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringConfig {
    /// Spring stiffness (higher = faster oscillation)
    /// Typical range: 100-500
    pub stiffness: f32,

    /// Damping coefficient (higher = less bounce)
    /// Typical range: 10-30
    pub damping: f32,

    /// Mass of the animated object (higher = slower, more momentum)
    /// Typical range: 0.5-2.0
    pub mass: f32,

    /// Velocity threshold to consider spring at rest
    pub rest_velocity: f32,

    /// Displacement threshold to consider spring at rest
    pub rest_displacement: f32,
}

impl SpringConfig {
    /// Create a new spring configuration
    pub const fn new(stiffness: f32, damping: f32, mass: f32) -> Self {
        Self {
            stiffness,
            damping,
            mass,
            rest_velocity: 0.01,
            rest_displacement: 0.001,
        }
    }

    /// Default spring (balanced, slight bounce)
    pub const fn default_spring() -> Self {
        Self::new(170.0, 26.0, 1.0)
    }

    /// Bouncy spring (noticeable oscillation)
    pub const fn bouncy() -> Self {
        Self::new(180.0, 12.0, 1.0)
    }

    /// Very bouncy spring (lots of oscillation)
    pub const fn very_bouncy() -> Self {
        Self::new(200.0, 8.0, 1.0)
    }

    /// Stiff spring (fast, minimal bounce)
    pub const fn stiff() -> Self {
        Self::new(400.0, 30.0, 1.0)
    }

    /// Gentle spring (slow, smooth)
    pub const fn gentle() -> Self {
        Self::new(120.0, 14.0, 1.0)
    }

    /// Slow spring (very slow, smooth)
    pub const fn slow() -> Self {
        Self::new(80.0, 20.0, 2.0)
    }

    /// Critically damped spring (no oscillation)
    pub fn critically_damped(stiffness: f32, mass: f32) -> Self {
        let damping = 2.0 * (stiffness * mass).sqrt();
        Self::new(stiffness, damping, mass)
    }

    /// Under-damped spring (oscillates before settling)
    pub fn under_damped(stiffness: f32, mass: f32, damping_ratio: f32) -> Self {
        let critical_damping = 2.0 * (stiffness * mass).sqrt();
        let damping = critical_damping * damping_ratio.clamp(0.0, 0.99);
        Self::new(stiffness, damping, mass)
    }

    /// Over-damped spring (slow approach, no oscillation)
    pub fn over_damped(stiffness: f32, mass: f32, damping_ratio: f32) -> Self {
        let critical_damping = 2.0 * (stiffness * mass).sqrt();
        let damping = critical_damping * damping_ratio.max(1.01);
        Self::new(stiffness, damping, mass)
    }

    /// Calculate the damping ratio
    pub fn damping_ratio(&self) -> f32 {
        self.damping / (2.0 * (self.stiffness * self.mass).sqrt())
    }

    /// Check if critically damped
    pub fn is_critically_damped(&self) -> bool {
        (self.damping_ratio() - 1.0).abs() < 0.01
    }

    /// Check if under-damped (will oscillate)
    pub fn is_under_damped(&self) -> bool {
        self.damping_ratio() < 1.0
    }

    /// Check if over-damped (slow, no oscillation)
    pub fn is_over_damped(&self) -> bool {
        self.damping_ratio() > 1.0
    }

    /// Angular frequency (natural frequency of oscillation)
    pub fn angular_frequency(&self) -> f32 {
        (self.stiffness / self.mass).sqrt()
    }

    /// Set rest thresholds
    pub fn with_rest_thresholds(mut self, velocity: f32, displacement: f32) -> Self {
        self.rest_velocity = velocity;
        self.rest_displacement = displacement;
        self
    }
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self::default_spring()
    }
}

// =============================================================================
// Spring Animation (scalar)
// =============================================================================

/// Physics-based spring animation for a single value
#[derive(Debug, Clone)]
pub struct Spring {
    /// Current value
    value: f32,

    /// Current velocity
    velocity: f32,

    /// Target value
    target: f32,

    /// Spring configuration
    config: SpringConfig,

    /// Whether spring is at rest
    at_rest: bool,
}

impl Spring {
    /// Create a new spring animation
    pub fn new(start: f32, target: f32, config: SpringConfig) -> Self {
        Self {
            value: start,
            velocity: 0.0,
            target,
            config,
            at_rest: start == target,
        }
    }

    /// Create with initial velocity
    pub fn with_velocity(start: f32, target: f32, velocity: f32, config: SpringConfig) -> Self {
        Self {
            value: start,
            velocity,
            target,
            config,
            at_rest: false,
        }
    }

    /// Get current value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Get current velocity
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Get target value
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Check if at rest
    pub fn is_at_rest(&self) -> bool {
        self.at_rest
    }

    /// Check if animating
    pub fn is_animating(&self) -> bool {
        !self.at_rest
    }

    /// Set a new target
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
        self.at_rest = false;
    }

    /// Set current value (e.g., for dragging)
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
        self.at_rest = false;
    }

    /// Set velocity (e.g., from gesture)
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
        self.at_rest = false;
    }

    /// Snap to target immediately
    pub fn snap_to_target(&mut self) {
        self.value = self.target;
        self.velocity = 0.0;
        self.at_rest = true;
    }

    /// Update spring by time delta (seconds) and return current value
    pub fn tick(&mut self, dt: f32) -> f32 {
        if self.at_rest {
            return self.value;
        }

        // Spring force: F = -k * x
        let displacement = self.value - self.target;
        let spring_force = -self.config.stiffness * displacement;

        // Damping force: F = -c * v
        let damping_force = -self.config.damping * self.velocity;

        // Total acceleration: a = F / m
        let acceleration = (spring_force + damping_force) / self.config.mass;

        // Semi-implicit Euler integration (more stable than explicit)
        self.velocity += acceleration * dt;
        self.value += self.velocity * dt;

        // Check if at rest
        if self.velocity.abs() < self.config.rest_velocity
            && (self.value - self.target).abs() < self.config.rest_displacement
        {
            self.value = self.target;
            self.velocity = 0.0;
            self.at_rest = true;
        }

        self.value
    }

    /// Reset to initial state
    pub fn reset(&mut self, start: f32, target: f32) {
        self.value = start;
        self.velocity = 0.0;
        self.target = target;
        self.at_rest = start == target;
    }
}

// =============================================================================
// Spring2D (for 2D animations)
// =============================================================================

/// Physics-based spring animation for 2D values (x, y)
#[derive(Debug, Clone)]
pub struct Spring2D {
    /// X-axis spring
    x: Spring,
    /// Y-axis spring
    y: Spring,
}

impl Spring2D {
    /// Create a new 2D spring
    pub fn new(start: (f32, f32), target: (f32, f32), config: SpringConfig) -> Self {
        Self {
            x: Spring::new(start.0, target.0, config),
            y: Spring::new(start.1, target.1, config),
        }
    }

    /// Create with initial velocity
    pub fn with_velocity(
        start: (f32, f32),
        target: (f32, f32),
        velocity: (f32, f32),
        config: SpringConfig,
    ) -> Self {
        Self {
            x: Spring::with_velocity(start.0, target.0, velocity.0, config),
            y: Spring::with_velocity(start.1, target.1, velocity.1, config),
        }
    }

    /// Get current value
    pub fn value(&self) -> (f32, f32) {
        (self.x.value(), self.y.value())
    }

    /// Get current velocity
    pub fn velocity(&self) -> (f32, f32) {
        (self.x.velocity(), self.y.velocity())
    }

    /// Get target
    pub fn target(&self) -> (f32, f32) {
        (self.x.target(), self.y.target())
    }

    /// Check if at rest
    pub fn is_at_rest(&self) -> bool {
        self.x.is_at_rest() && self.y.is_at_rest()
    }

    /// Check if animating
    pub fn is_animating(&self) -> bool {
        !self.is_at_rest()
    }

    /// Set target
    pub fn set_target(&mut self, target: (f32, f32)) {
        self.x.set_target(target.0);
        self.y.set_target(target.1);
    }

    /// Set current value
    pub fn set_value(&mut self, value: (f32, f32)) {
        self.x.set_value(value.0);
        self.y.set_value(value.1);
    }

    /// Set velocity
    pub fn set_velocity(&mut self, velocity: (f32, f32)) {
        self.x.set_velocity(velocity.0);
        self.y.set_velocity(velocity.1);
    }

    /// Update and return current value
    pub fn tick(&mut self, dt: f32) -> (f32, f32) {
        (self.x.tick(dt), self.y.tick(dt))
    }
}

// =============================================================================
// SpringAnimation (generic, for Animatable types)
// =============================================================================

/// Generic spring animation for any Animatable type
#[derive(Debug, Clone)]
pub struct SpringAnimation<T: Animatable> {
    /// Current value
    current: T,

    /// Previous value (for velocity calculation)
    previous: T,

    /// Target value
    target: T,

    /// Spring configuration
    config: SpringConfig,

    /// Progress (0.0 to 1.0, approximated)
    progress: f32,

    /// Whether at rest
    at_rest: bool,

    /// Elapsed time
    elapsed: f32,
}

impl<T: Animatable> SpringAnimation<T> {
    /// Create a new spring animation
    pub fn new(start: T, target: T, config: SpringConfig) -> Self {
        let at_rest = false; // Always start animating
        Self {
            current: start.clone(),
            previous: start,
            target,
            config,
            progress: 0.0,
            at_rest,
            elapsed: 0.0,
        }
    }

    /// Get current value
    pub fn value(&self) -> &T {
        &self.current
    }

    /// Get target value
    pub fn target(&self) -> &T {
        &self.target
    }

    /// Get approximate progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Check if at rest
    pub fn is_at_rest(&self) -> bool {
        self.at_rest
    }

    /// Set new target
    pub fn set_target(&mut self, target: T) {
        self.target = target;
        self.at_rest = false;
    }

    /// Update spring animation
    /// Note: For complex types, this uses a simplified spring model
    pub fn tick(&mut self, dt: f32) -> T {
        if self.at_rest {
            return self.current.clone();
        }

        self.elapsed += dt;

        // Use analytical spring solution for progress
        let omega = self.config.angular_frequency();
        let zeta = self.config.damping_ratio();

        // Calculate spring progress using damped harmonic motion
        let spring_progress = if zeta < 1.0 {
            // Under-damped
            let omega_d = omega * (1.0 - zeta * zeta).sqrt();
            let envelope = (-zeta * omega * self.elapsed).exp();
            let cos_term = (omega_d * self.elapsed).cos();
            let sin_term = (zeta * omega / omega_d) * (omega_d * self.elapsed).sin();
            1.0 - envelope * (cos_term + sin_term)
        } else if (zeta - 1.0).abs() < 0.001 {
            // Critically damped
            1.0 - (1.0 + omega * self.elapsed) * (-omega * self.elapsed).exp()
        } else {
            // Over-damped
            let s1 = -omega * (zeta - (zeta * zeta - 1.0).sqrt());
            let s2 = -omega * (zeta + (zeta * zeta - 1.0).sqrt());
            let c2 = s1 / (s1 - s2);
            let c1 = 1.0 - c2;
            1.0 - c1 * (s1 * self.elapsed).exp() - c2 * (s2 * self.elapsed).exp()
        };

        self.progress = spring_progress.clamp(0.0, 1.0);
        self.previous = self.current.clone();

        // Interpolate using spring progress
        self.current = self.previous.lerp(&self.target, self.progress);

        // Check if at rest (progress very close to 1.0 for several frames)
        if (self.progress - 1.0).abs() < self.config.rest_displacement {
            self.current = self.target.clone();
            self.at_rest = true;
        }

        self.current.clone()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_config_damping_ratio() {
        let critical = SpringConfig::critically_damped(100.0, 1.0);
        assert!((critical.damping_ratio() - 1.0).abs() < 0.01);

        let under = SpringConfig::under_damped(100.0, 1.0, 0.5);
        assert!(under.is_under_damped());

        let over = SpringConfig::over_damped(100.0, 1.0, 1.5);
        assert!(over.is_over_damped());
    }

    #[test]
    fn test_spring_basic() {
        let mut spring = Spring::new(0.0, 100.0, SpringConfig::default());

        // Should start animating
        assert!(spring.is_animating());

        // Update several frames
        for _ in 0..100 {
            spring.tick(0.016);
        }

        // Should approach target
        assert!((spring.value() - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_spring_bouncy() {
        let mut spring = Spring::new(0.0, 100.0, SpringConfig::bouncy());

        // Track if we ever overshoot
        let mut overshot = false;
        for _ in 0..200 {
            let v = spring.tick(0.016);
            if v > 100.0 {
                overshot = true;
            }
        }

        // Bouncy spring should overshoot
        assert!(overshot);
        // But should eventually settle
        assert!(spring.is_at_rest());
        assert!((spring.value() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_spring_critically_damped() {
        let config = SpringConfig::critically_damped(200.0, 1.0);
        let mut spring = Spring::new(0.0, 100.0, config);

        // Track if we ever overshoot
        let mut max_value = 0.0_f32;
        for _ in 0..200 {
            let v = spring.tick(0.016);
            max_value = max_value.max(v);
        }

        // Critically damped should not overshoot (or barely)
        assert!(max_value <= 101.0);
        assert!(spring.is_at_rest());
    }

    #[test]
    fn test_spring_with_velocity() {
        let config = SpringConfig::default();
        let mut spring = Spring::with_velocity(50.0, 100.0, 500.0, config);

        // Initial velocity should cause overshoot
        let mut max_value = 0.0_f32;
        for _ in 0..200 {
            let v = spring.tick(0.016);
            max_value = max_value.max(v);
        }

        // Should overshoot due to initial velocity
        assert!(max_value > 100.0);
    }

    #[test]
    fn test_spring_set_target() {
        let mut spring = Spring::new(0.0, 50.0, SpringConfig::stiff());

        // Animate halfway
        for _ in 0..30 {
            spring.tick(0.016);
        }

        // Change target
        spring.set_target(100.0);
        assert!(spring.is_animating());

        // Continue animating
        for _ in 0..100 {
            spring.tick(0.016);
        }

        // Should reach new target
        assert!((spring.value() - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_spring_snap_to_target() {
        let mut spring = Spring::new(0.0, 100.0, SpringConfig::default());
        spring.tick(0.016);

        spring.snap_to_target();
        assert_eq!(spring.value(), 100.0);
        assert_eq!(spring.velocity(), 0.0);
        assert!(spring.is_at_rest());
    }

    #[test]
    fn test_spring_2d() {
        let mut spring = Spring2D::new((0.0, 0.0), (100.0, 200.0), SpringConfig::default());

        for _ in 0..150 {
            spring.tick(0.016);
        }

        let (x, y) = spring.value();
        assert!((x - 100.0).abs() < 1.0);
        assert!((y - 200.0).abs() < 1.0);
    }

    #[test]
    fn test_spring_2d_velocity() {
        let config = SpringConfig::bouncy();
        let mut spring =
            Spring2D::with_velocity((50.0, 50.0), (100.0, 100.0), (200.0, 200.0), config);

        // Initial velocity should cause movement
        spring.tick(0.016);
        let (x, y) = spring.value();
        assert!(x > 50.0);
        assert!(y > 50.0);
    }

    #[test]
    fn test_spring_animation_generic() {
        use crate::value::Color;

        let mut spring = SpringAnimation::new(Color::BLACK, Color::WHITE, SpringConfig::default());

        for _ in 0..150 {
            spring.tick(0.016);
        }

        let color = spring.value();
        assert!((color.r - 1.0).abs() < 0.1);
        assert!((color.g - 1.0).abs() < 0.1);
        assert!((color.b - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_spring_presets() {
        // Verify preset configurations exist and have reasonable values
        let bouncy = SpringConfig::bouncy();
        assert!(bouncy.stiffness > 0.0);
        assert!(bouncy.damping > 0.0);
        assert!(bouncy.is_under_damped());

        let stiff = SpringConfig::stiff();
        assert!(stiff.stiffness > bouncy.stiffness);

        let gentle = SpringConfig::gentle();
        assert!(gentle.stiffness < stiff.stiffness);
    }

    #[test]
    fn test_spring_reset() {
        let mut spring = Spring::new(0.0, 100.0, SpringConfig::default());

        // Animate partway
        for _ in 0..30 {
            spring.tick(0.016);
        }

        // Reset
        spring.reset(50.0, 150.0);
        assert_eq!(spring.value(), 50.0);
        assert_eq!(spring.target(), 150.0);
        assert_eq!(spring.velocity(), 0.0);
    }
}
