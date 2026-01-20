//! Velocity tracking for gesture calculations
//!
//! This module provides velocity tracking capabilities for calculating
//! fling velocities and momentum-based animations.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// A sample point for velocity calculation
#[derive(Debug, Clone, Copy)]
struct VelocitySample {
    position: (f32, f32),
    timestamp: Instant,
}

/// Tracks pointer movement to calculate velocity for fling gestures
///
/// The velocity tracker uses a sliding window of recent position samples
/// to calculate instantaneous velocity. This is useful for implementing
/// momentum scrolling, fling gestures, and physics-based animations.
///
/// # Example
///
/// ```
/// use oxide_gestures::velocity::VelocityTracker;
/// use std::time::Instant;
///
/// let mut tracker = VelocityTracker::new();
/// tracker.add_position(0.0, 0.0, Instant::now());
/// // ... after some movement
/// // tracker.add_position(100.0, 50.0, Instant::now());
/// // let velocity = tracker.calculate_velocity();
/// ```
#[derive(Debug)]
pub struct VelocityTracker {
    /// Recent position samples
    samples: VecDeque<VelocitySample>,
    /// Maximum number of samples to keep
    max_samples: usize,
    /// Maximum age of samples to consider (in milliseconds)
    max_age_ms: u64,
}

impl Default for VelocityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl VelocityTracker {
    /// Create a new velocity tracker with default settings
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(20),
            max_samples: 20,
            max_age_ms: 100,
        }
    }

    /// Create a velocity tracker with custom settings
    ///
    /// # Arguments
    ///
    /// * `max_samples` - Maximum number of samples to retain
    /// * `max_age_ms` - Maximum age of samples in milliseconds
    pub fn with_config(max_samples: usize, max_age_ms: u64) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            max_age_ms,
        }
    }

    /// Add a position sample at the given timestamp
    pub fn add_position(&mut self, x: f32, y: f32, timestamp: Instant) {
        // Remove old samples
        self.prune_old_samples(timestamp);

        // Add new sample
        self.samples.push_back(VelocitySample {
            position: (x, y),
            timestamp,
        });

        // Keep within max samples
        while self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    /// Add a position sample at the current time
    pub fn add_position_now(&mut self, x: f32, y: f32) {
        self.add_position(x, y, Instant::now());
    }

    /// Remove samples older than max_age_ms
    fn prune_old_samples(&mut self, now: Instant) {
        let max_age = Duration::from_millis(self.max_age_ms);
        while let Some(front) = self.samples.front() {
            if now.duration_since(front.timestamp) > max_age {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Calculate the current velocity in pixels per second
    ///
    /// Returns `(vx, vy)` where `vx` is horizontal velocity and `vy` is vertical velocity.
    /// Returns `(0.0, 0.0)` if there are insufficient samples.
    pub fn calculate_velocity(&self) -> (f32, f32) {
        if self.samples.len() < 2 {
            return (0.0, 0.0);
        }

        // Use linear regression for smoother velocity estimation
        let n = self.samples.len() as f32;
        let first_time = self.samples.front().unwrap().timestamp;

        let mut sum_t = 0.0;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_t2 = 0.0;
        let mut sum_tx = 0.0;
        let mut sum_ty = 0.0;

        for sample in &self.samples {
            let t = sample.timestamp.duration_since(first_time).as_secs_f32();
            let x = sample.position.0;
            let y = sample.position.1;

            sum_t += t;
            sum_x += x;
            sum_y += y;
            sum_t2 += t * t;
            sum_tx += t * x;
            sum_ty += t * y;
        }

        let denominator = n * sum_t2 - sum_t * sum_t;
        if denominator.abs() < 1e-10 {
            return (0.0, 0.0);
        }

        let vx = (n * sum_tx - sum_t * sum_x) / denominator;
        let vy = (n * sum_ty - sum_t * sum_y) / denominator;

        (vx, vy)
    }

    /// Calculate velocity magnitude (speed) in pixels per second
    pub fn calculate_speed(&self) -> f32 {
        let (vx, vy) = self.calculate_velocity();
        (vx * vx + vy * vy).sqrt()
    }

    /// Clear all samples
    pub fn reset(&mut self) {
        self.samples.clear();
    }

    /// Check if there are enough samples to calculate velocity
    pub fn has_sufficient_samples(&self) -> bool {
        self.samples.len() >= 2
    }

    /// Get the number of current samples
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

/// Velocity in 2D space
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Velocity {
    /// Horizontal velocity in pixels per second
    pub x: f32,
    /// Vertical velocity in pixels per second
    pub y: f32,
}

impl Velocity {
    /// Create a new velocity
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero velocity
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Calculate the magnitude (speed)
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Get the direction in radians
    pub fn direction(&self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Create velocity from magnitude and direction
    pub fn from_polar(magnitude: f32, direction: f32) -> Self {
        Self {
            x: magnitude * direction.cos(),
            y: magnitude * direction.sin(),
        }
    }

    /// Clamp velocity to a maximum magnitude
    pub fn clamped(&self, max_magnitude: f32) -> Self {
        let mag = self.magnitude();
        if mag > max_magnitude && mag > 0.0 {
            let scale = max_magnitude / mag;
            Self {
                x: self.x * scale,
                y: self.y * scale,
            }
        } else {
            *self
        }
    }

    /// Apply friction (deceleration factor per second)
    pub fn with_friction(&self, friction: f32, dt: f32) -> Self {
        let decay = (1.0 - friction).powf(dt);
        Self {
            x: self.x * decay,
            y: self.y * decay,
        }
    }
}

impl From<(f32, f32)> for Velocity {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<Velocity> for (f32, f32) {
    fn from(v: Velocity) -> Self {
        (v.x, v.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_velocity_tracker_creation() {
        let tracker = VelocityTracker::new();
        assert_eq!(tracker.sample_count(), 0);
        assert!(!tracker.has_sufficient_samples());
    }

    #[test]
    fn test_velocity_tracker_add_samples() {
        let mut tracker = VelocityTracker::new();
        let now = Instant::now();

        tracker.add_position(0.0, 0.0, now);
        assert_eq!(tracker.sample_count(), 1);

        tracker.add_position(10.0, 10.0, now + Duration::from_millis(16));
        assert_eq!(tracker.sample_count(), 2);
        assert!(tracker.has_sufficient_samples());
    }

    #[test]
    fn test_velocity_calculation_horizontal() {
        let mut tracker = VelocityTracker::new();
        let now = Instant::now();

        // 100 pixels in 100ms = 1000 pixels/second
        tracker.add_position(0.0, 0.0, now);
        tracker.add_position(100.0, 0.0, now + Duration::from_millis(100));

        let (vx, vy) = tracker.calculate_velocity();
        assert!((vx - 1000.0).abs() < 1.0, "Expected ~1000, got {}", vx);
        assert!(vy.abs() < 1.0, "Expected ~0, got {}", vy);
    }

    #[test]
    fn test_velocity_calculation_vertical() {
        let mut tracker = VelocityTracker::new();
        let now = Instant::now();

        // 200 pixels in 100ms = 2000 pixels/second
        tracker.add_position(0.0, 0.0, now);
        tracker.add_position(0.0, 200.0, now + Duration::from_millis(100));

        let (vx, vy) = tracker.calculate_velocity();
        assert!(vx.abs() < 1.0);
        assert!((vy - 2000.0).abs() < 1.0, "Expected ~2000, got {}", vy);
    }

    #[test]
    fn test_velocity_calculation_diagonal() {
        let mut tracker = VelocityTracker::new();
        let now = Instant::now();

        tracker.add_position(0.0, 0.0, now);
        tracker.add_position(100.0, 100.0, now + Duration::from_millis(100));

        let (vx, vy) = tracker.calculate_velocity();
        assert!((vx - 1000.0).abs() < 1.0);
        assert!((vy - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_velocity_reset() {
        let mut tracker = VelocityTracker::new();
        tracker.add_position_now(0.0, 0.0);
        tracker.add_position_now(100.0, 100.0);

        assert!(tracker.has_sufficient_samples());
        tracker.reset();
        assert!(!tracker.has_sufficient_samples());
        assert_eq!(tracker.sample_count(), 0);
    }

    #[test]
    fn test_velocity_speed() {
        let mut tracker = VelocityTracker::new();
        let now = Instant::now();

        tracker.add_position(0.0, 0.0, now);
        tracker.add_position(30.0, 40.0, now + Duration::from_millis(100));

        // Pythagorean: sqrt(300^2 + 400^2) = 500 pixels/second
        let speed = tracker.calculate_speed();
        assert!((speed - 500.0).abs() < 1.0, "Expected ~500, got {}", speed);
    }

    #[test]
    fn test_velocity_struct_magnitude() {
        let v = Velocity::new(3.0, 4.0);
        assert!((v.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_clamped() {
        let v = Velocity::new(300.0, 400.0); // magnitude = 500
        let clamped = v.clamped(250.0);
        assert!((clamped.magnitude() - 250.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_from_polar() {
        let v = Velocity::from_polar(100.0, 0.0);
        assert!((v.x - 100.0).abs() < 0.001);
        assert!(v.y.abs() < 0.001);

        let v2 = Velocity::from_polar(100.0, std::f32::consts::FRAC_PI_2);
        assert!(v2.x.abs() < 0.001);
        assert!((v2.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_with_friction() {
        let v = Velocity::new(1000.0, 0.0);
        let v_after = v.with_friction(0.9, 1.0);
        assert!((v_after.x - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_velocity_conversions() {
        let v = Velocity::new(10.0, 20.0);
        let tuple: (f32, f32) = v.into();
        assert_eq!(tuple, (10.0, 20.0));

        let v2: Velocity = (30.0, 40.0).into();
        assert_eq!(v2.x, 30.0);
        assert_eq!(v2.y, 40.0);
    }

    #[test]
    fn test_velocity_zero() {
        let v = Velocity::zero();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.magnitude(), 0.0);
    }
}
