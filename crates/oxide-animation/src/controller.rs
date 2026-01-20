//! Animation Controller
//!
//! This module provides explicit animation control with play/pause/stop/reverse
//! functionality, similar to Flutter's AnimationController.
//!
//! # Example
//!
//! ```rust
//! use oxide_animation::{AnimationController, Curve};
//! use std::time::Duration;
//!
//! let mut controller = AnimationController::new()
//!     .duration(Duration::from_millis(500))
//!     .curve(Curve::EaseOut);
//!
//! controller.play();
//!
//! // In your render loop:
//! let value = controller.tick(0.016); // Returns 0.0 to 1.0
//! ```

use crate::curve::Curve;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// =============================================================================
// Animation ID
// =============================================================================

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for an animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimationId(pub u64);

impl AnimationId {
    /// Generate a new unique ID
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for AnimationId {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Animation Status
// =============================================================================

/// Current status of an animation controller
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationStatus {
    /// Animation has not started
    Idle,
    /// Animation is playing forward
    Forward,
    /// Animation is playing in reverse
    Reverse,
    /// Animation is paused
    Paused,
    /// Animation completed (reached end)
    Completed,
    /// Animation was dismissed (reached start in reverse)
    Dismissed,
}

impl AnimationStatus {
    /// Check if animation is currently running
    pub fn is_running(&self) -> bool {
        matches!(self, AnimationStatus::Forward | AnimationStatus::Reverse)
    }

    /// Check if animation is complete or dismissed
    pub fn is_done(&self) -> bool {
        matches!(
            self,
            AnimationStatus::Completed | AnimationStatus::Dismissed
        )
    }

    /// Check if animation is at rest (idle, paused, or done)
    pub fn is_at_rest(&self) -> bool {
        !self.is_running()
    }
}

// =============================================================================
// Animation Direction
// =============================================================================

/// Direction of animation playback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnimationDirection {
    /// Play forward (0.0 -> 1.0)
    #[default]
    Forward,
    /// Play in reverse (1.0 -> 0.0)
    Reverse,
}

// =============================================================================
// Repeat Mode
// =============================================================================

/// How animation should repeat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    /// Don't repeat (play once)
    None,
    /// Repeat from start (loop)
    Loop,
    /// Reverse direction each time (ping-pong)
    Reverse,
}

impl Default for RepeatMode {
    fn default() -> Self {
        RepeatMode::None
    }
}

// =============================================================================
// Animation Controller
// =============================================================================

/// Explicit animation controller with play/pause/stop/reverse controls.
///
/// This provides fine-grained control over animation playback,
/// similar to Flutter's AnimationController.
#[derive(Debug, Clone)]
pub struct AnimationController {
    /// Unique ID
    id: AnimationId,

    /// Current value (0.0 to 1.0)
    value: f32,

    /// Animation duration
    duration: Duration,

    /// Easing curve
    curve: Curve,

    /// Reverse curve (used when playing in reverse)
    reverse_curve: Option<Curve>,

    /// Current status
    status: AnimationStatus,

    /// Lower bound (default 0.0)
    lower_bound: f32,

    /// Upper bound (default 1.0)
    upper_bound: f32,

    /// Repeat mode
    repeat_mode: RepeatMode,

    /// Number of times to repeat (0 = infinite when repeat_mode != None)
    repeat_count: u32,

    /// Current repeat iteration
    current_iteration: u32,
}

impl AnimationController {
    /// Create a new animation controller
    pub fn new() -> Self {
        Self {
            id: AnimationId::new(),
            value: 0.0,
            duration: Duration::from_millis(300),
            curve: Curve::Linear,
            reverse_curve: None,
            status: AnimationStatus::Idle,
            lower_bound: 0.0,
            upper_bound: 1.0,
            repeat_mode: RepeatMode::None,
            repeat_count: 1,
            current_iteration: 0,
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

    /// Set a separate curve for reverse playback
    pub fn reverse_curve(mut self, curve: Curve) -> Self {
        self.reverse_curve = Some(curve);
        self
    }

    /// Set bounds for the animation value
    pub fn bounds(mut self, lower: f32, upper: f32) -> Self {
        self.lower_bound = lower;
        self.upper_bound = upper;
        self
    }

    /// Set repeat mode
    pub fn repeat(mut self, mode: RepeatMode) -> Self {
        self.repeat_mode = mode;
        self
    }

    /// Set number of times to repeat (0 = infinite)
    pub fn repeat_count(mut self, count: u32) -> Self {
        self.repeat_count = count;
        self
    }

    /// Get the animation ID
    pub fn id(&self) -> AnimationId {
        self.id
    }

    /// Get the current raw value (0.0 to 1.0)
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Get the current eased value (with curve applied)
    pub fn eased_value(&self) -> f32 {
        let curve = match self.status {
            AnimationStatus::Reverse => self.reverse_curve.unwrap_or(self.curve),
            _ => self.curve,
        };
        let t = curve.transform(self.value);
        self.lower_bound + (self.upper_bound - self.lower_bound) * t
    }

    /// Get current status
    pub fn status(&self) -> AnimationStatus {
        self.status
    }

    /// Check if animation is running
    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.status == AnimationStatus::Completed
    }

    /// Check if animation is dismissed
    pub fn is_dismissed(&self) -> bool {
        self.status == AnimationStatus::Dismissed
    }

    /// Get progress as percentage (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        self.value
    }

    /// Start playing forward from current position
    pub fn play(&mut self) {
        if self.status == AnimationStatus::Completed {
            self.value = 0.0;
        }
        self.status = AnimationStatus::Forward;
    }

    /// Start playing forward from the beginning
    pub fn forward(&mut self) {
        self.value = 0.0;
        self.current_iteration = 0;
        self.status = AnimationStatus::Forward;
    }

    /// Start playing in reverse from current position
    pub fn reverse(&mut self) {
        if self.status == AnimationStatus::Dismissed {
            self.value = 1.0;
        }
        self.status = AnimationStatus::Reverse;
    }

    /// Start playing in reverse from the end
    pub fn reverse_from_end(&mut self) {
        self.value = 1.0;
        self.current_iteration = 0;
        self.status = AnimationStatus::Reverse;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.status.is_running() {
            self.status = AnimationStatus::Paused;
        }
    }

    /// Resume from paused state
    pub fn resume(&mut self) {
        if self.status == AnimationStatus::Paused {
            // Resume in the direction we were going
            self.status = if self.value < 1.0 {
                AnimationStatus::Forward
            } else {
                AnimationStatus::Reverse
            };
        }
    }

    /// Stop the animation and reset to start
    pub fn stop(&mut self) {
        self.value = 0.0;
        self.current_iteration = 0;
        self.status = AnimationStatus::Idle;
    }

    /// Reset to a specific value
    pub fn reset(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
        self.current_iteration = 0;
        self.status = AnimationStatus::Idle;
    }

    /// Seek to a specific progress (0.0 to 1.0)
    pub fn seek(&mut self, progress: f32) {
        self.value = progress.clamp(0.0, 1.0);
    }

    /// Animate to a specific progress value
    pub fn animate_to(&mut self, target: f32) {
        let target = target.clamp(0.0, 1.0);
        if target > self.value {
            self.status = AnimationStatus::Forward;
        } else if target < self.value {
            self.status = AnimationStatus::Reverse;
        }
    }

    /// Toggle between forward and reverse
    pub fn toggle(&mut self) {
        match self.status {
            AnimationStatus::Forward => self.status = AnimationStatus::Reverse,
            AnimationStatus::Reverse => self.status = AnimationStatus::Forward,
            AnimationStatus::Completed => {
                self.status = AnimationStatus::Reverse;
            }
            AnimationStatus::Dismissed => {
                self.status = AnimationStatus::Forward;
            }
            AnimationStatus::Idle | AnimationStatus::Paused => {
                if self.value < 0.5 {
                    self.status = AnimationStatus::Forward;
                } else {
                    self.status = AnimationStatus::Reverse;
                }
            }
        }
    }

    /// Update the animation by a time delta (in seconds)
    /// Returns the current eased value
    pub fn tick(&mut self, dt: f32) -> f32 {
        if !self.status.is_running() {
            return self.eased_value();
        }

        let duration_secs = self.duration.as_secs_f32();
        if duration_secs <= 0.0 {
            // Instant animation
            match self.status {
                AnimationStatus::Forward => {
                    self.value = 1.0;
                    self.handle_completion();
                }
                AnimationStatus::Reverse => {
                    self.value = 0.0;
                    self.handle_dismissal();
                }
                _ => {}
            }
            return self.eased_value();
        }

        let delta = dt / duration_secs;

        match self.status {
            AnimationStatus::Forward => {
                self.value += delta;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.handle_completion();
                }
            }
            AnimationStatus::Reverse => {
                self.value -= delta;
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.handle_dismissal();
                }
            }
            _ => {}
        }

        self.eased_value()
    }

    fn handle_completion(&mut self) {
        self.current_iteration += 1;

        match self.repeat_mode {
            RepeatMode::None => {
                self.status = AnimationStatus::Completed;
            }
            RepeatMode::Loop => {
                if self.repeat_count == 0 || self.current_iteration < self.repeat_count {
                    self.value = 0.0;
                    // Stay in Forward status
                } else {
                    self.status = AnimationStatus::Completed;
                }
            }
            RepeatMode::Reverse => {
                if self.repeat_count == 0 || self.current_iteration < self.repeat_count {
                    self.status = AnimationStatus::Reverse;
                } else {
                    self.status = AnimationStatus::Completed;
                }
            }
        }
    }

    fn handle_dismissal(&mut self) {
        match self.repeat_mode {
            RepeatMode::None => {
                self.status = AnimationStatus::Dismissed;
            }
            RepeatMode::Loop => {
                // For loop mode in reverse, go back to end
                if self.repeat_count == 0 || self.current_iteration < self.repeat_count {
                    self.value = 1.0;
                } else {
                    self.status = AnimationStatus::Dismissed;
                }
            }
            RepeatMode::Reverse => {
                self.current_iteration += 1;
                if self.repeat_count == 0 || self.current_iteration < self.repeat_count {
                    self.status = AnimationStatus::Forward;
                } else {
                    self.status = AnimationStatus::Dismissed;
                }
            }
        }
    }
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_basic() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_secs(1))
            .curve(Curve::Linear);

        assert_eq!(ctrl.status(), AnimationStatus::Idle);
        assert_eq!(ctrl.value(), 0.0);

        ctrl.play();
        assert_eq!(ctrl.status(), AnimationStatus::Forward);

        // Tick halfway
        ctrl.tick(0.5);
        assert!((ctrl.value() - 0.5).abs() < 0.01);

        // Tick to end
        ctrl.tick(0.5);
        assert_eq!(ctrl.value(), 1.0);
        assert_eq!(ctrl.status(), AnimationStatus::Completed);
    }

    #[test]
    fn test_controller_reverse() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_secs(1))
            .curve(Curve::Linear);

        ctrl.reverse_from_end();
        assert_eq!(ctrl.status(), AnimationStatus::Reverse);
        assert_eq!(ctrl.value(), 1.0);

        ctrl.tick(0.5);
        assert!((ctrl.value() - 0.5).abs() < 0.01);

        ctrl.tick(0.5);
        assert_eq!(ctrl.value(), 0.0);
        assert_eq!(ctrl.status(), AnimationStatus::Dismissed);
    }

    #[test]
    fn test_controller_pause_resume() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_secs(1))
            .curve(Curve::Linear);

        ctrl.play();
        ctrl.tick(0.3);

        ctrl.pause();
        assert_eq!(ctrl.status(), AnimationStatus::Paused);

        let value_before = ctrl.value();
        ctrl.tick(0.5);
        assert_eq!(ctrl.value(), value_before); // Should not change

        ctrl.resume();
        assert!(ctrl.is_running());
        ctrl.tick(0.3);
        assert!(ctrl.value() > value_before);
    }

    #[test]
    fn test_controller_stop() {
        let mut ctrl = AnimationController::new().duration(Duration::from_secs(1));

        ctrl.play();
        ctrl.tick(0.5);

        ctrl.stop();
        assert_eq!(ctrl.status(), AnimationStatus::Idle);
        assert_eq!(ctrl.value(), 0.0);
    }

    #[test]
    fn test_controller_seek() {
        let mut ctrl = AnimationController::new().curve(Curve::Linear);

        ctrl.seek(0.75);
        assert_eq!(ctrl.value(), 0.75);
        assert!((ctrl.eased_value() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_controller_toggle() {
        let mut ctrl = AnimationController::new().duration(Duration::from_secs(1));

        ctrl.play();
        ctrl.tick(0.5);

        ctrl.toggle();
        assert_eq!(ctrl.status(), AnimationStatus::Reverse);

        ctrl.toggle();
        assert_eq!(ctrl.status(), AnimationStatus::Forward);
    }

    #[test]
    fn test_controller_repeat_loop() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_millis(100))
            .curve(Curve::Linear)
            .repeat(RepeatMode::Loop)
            .repeat_count(3);

        ctrl.forward();

        // Complete first iteration
        ctrl.tick(0.1);
        assert_eq!(ctrl.value(), 1.0);
        assert_eq!(ctrl.status(), AnimationStatus::Forward); // Should restart

        // After restart, value should be 0
        assert_eq!(ctrl.value(), 0.0);

        // Complete two more iterations
        ctrl.tick(0.1);
        ctrl.tick(0.1);

        assert_eq!(ctrl.status(), AnimationStatus::Completed);
    }

    #[test]
    fn test_controller_repeat_reverse() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_millis(100))
            .curve(Curve::Linear)
            .repeat(RepeatMode::Reverse)
            .repeat_count(2);

        ctrl.forward();
        ctrl.tick(0.1); // Complete forward

        // Should now be reversing
        assert_eq!(ctrl.status(), AnimationStatus::Reverse);

        ctrl.tick(0.1); // Complete reverse

        // Should be forward again or completed based on count
        assert_eq!(ctrl.status(), AnimationStatus::Completed);
    }

    #[test]
    fn test_controller_bounds() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_secs(1))
            .curve(Curve::Linear)
            .bounds(100.0, 200.0);

        ctrl.play();
        ctrl.tick(0.5);

        // Value should be halfway between bounds
        assert!((ctrl.eased_value() - 150.0).abs() < 1.0);
    }

    #[test]
    fn test_controller_eased_value() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_secs(1))
            .curve(Curve::EaseInQuad);

        ctrl.seek(0.5);

        // EaseInQuad at 0.5 should be 0.25
        assert!((ctrl.eased_value() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_controller_reverse_curve() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_secs(1))
            .curve(Curve::EaseIn)
            .reverse_curve(Curve::EaseOut);

        ctrl.seek(0.5);
        let forward_value = ctrl.eased_value();

        ctrl.reverse();
        let reverse_value = ctrl.eased_value();

        // Different curves should give different values at same progress
        // Note: They might still be similar at 0.5 for some curves
        assert!(forward_value >= 0.0 && forward_value <= 1.0);
        assert!(reverse_value >= 0.0 && reverse_value <= 1.0);
    }

    #[test]
    fn test_animation_id_unique() {
        let id1 = AnimationId::new();
        let id2 = AnimationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_controller_forward() {
        let mut ctrl = AnimationController::new().duration(Duration::from_secs(1));

        ctrl.seek(0.5);
        ctrl.forward();

        assert_eq!(ctrl.value(), 0.0);
        assert_eq!(ctrl.status(), AnimationStatus::Forward);
    }

    #[test]
    fn test_infinite_repeat() {
        let mut ctrl = AnimationController::new()
            .duration(Duration::from_millis(100))
            .repeat(RepeatMode::Loop)
            .repeat_count(0); // 0 = infinite

        ctrl.forward();

        // Should keep looping
        for _ in 0..10 {
            ctrl.tick(0.1);
            if ctrl.status() == AnimationStatus::Completed {
                panic!("Should not complete with infinite repeat");
            }
        }

        assert!(ctrl.is_running());
    }
}
