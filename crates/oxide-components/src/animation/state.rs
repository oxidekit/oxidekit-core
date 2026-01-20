//! Animation State Machine
//!
//! Provides core Animation and AnimationState types for managing animation playback.

use serde::{Deserialize, Serialize};
use super::easing::Easing;
use super::interpolate::AnimatableValue;

/// Play direction for an animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlayDirection {
    /// Play forwards (from start to end)
    #[default]
    Forward,
    /// Play backwards (from end to start)
    Reverse,
    /// Alternate between forward and reverse each iteration
    Alternate,
    /// Alternate starting with reverse
    AlternateReverse,
}

/// Fill mode - what to do when animation is not running
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FillMode {
    /// No fill - return to initial state
    #[default]
    None,
    /// Hold the end value after animation completes
    Forwards,
    /// Apply the start value before animation starts (for delayed animations)
    Backwards,
    /// Both forwards and backwards
    Both,
}

/// Current status of an animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimationStatus {
    /// Animation has not started yet
    #[default]
    Idle,
    /// Animation is waiting for delay to complete
    Delayed,
    /// Animation is currently playing
    Running,
    /// Animation is paused
    Paused,
    /// Animation has completed
    Completed,
    /// Animation was cancelled
    Cancelled,
}

/// Current state of an animation at any given moment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationState {
    /// Current status
    pub status: AnimationStatus,

    /// Current progress (0.0 to 1.0)
    pub progress: f32,

    /// Eased progress (after applying easing function)
    pub eased_progress: f32,

    /// Time elapsed since animation started (in seconds)
    pub elapsed: f32,

    /// Time remaining until animation completes (in seconds)
    pub remaining: f32,

    /// Current iteration (for repeating animations)
    pub iteration: u32,

    /// Current direction (may change for alternate modes)
    pub current_direction: PlayDirection,

    /// The current interpolated value (if available)
    pub current_value: Option<AnimatableValue>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            status: AnimationStatus::Idle,
            progress: 0.0,
            eased_progress: 0.0,
            elapsed: 0.0,
            remaining: 0.0,
            iteration: 0,
            current_direction: PlayDirection::Forward,
            current_value: None,
        }
    }
}

impl AnimationState {
    /// Check if the animation is currently active (running or delayed)
    pub fn is_active(&self) -> bool {
        matches!(self.status, AnimationStatus::Running | AnimationStatus::Delayed)
    }

    /// Check if the animation has finished
    pub fn is_finished(&self) -> bool {
        matches!(self.status, AnimationStatus::Completed | AnimationStatus::Cancelled)
    }

    /// Check if the animation can be updated
    pub fn can_update(&self) -> bool {
        matches!(self.status, AnimationStatus::Running | AnimationStatus::Delayed | AnimationStatus::Idle)
    }
}

/// An animation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    /// Property name being animated (e.g., "opacity", "transform.x")
    pub property: String,

    /// Starting value
    pub from: AnimatableValue,

    /// Ending value
    pub to: AnimatableValue,

    /// Duration in seconds
    pub duration: f32,

    /// Delay before starting (in seconds)
    #[serde(default)]
    pub delay: f32,

    /// Easing function
    #[serde(default)]
    pub easing: Easing,

    /// Play direction
    #[serde(default)]
    pub direction: PlayDirection,

    /// Fill mode
    #[serde(default)]
    pub fill_mode: FillMode,

    /// Number of times to repeat (0 = infinite, 1 = play once, 2 = play twice, etc.)
    #[serde(default = "default_iteration_count")]
    pub iteration_count: u32,

    /// Current state
    #[serde(skip)]
    pub state: AnimationState,
}

fn default_iteration_count() -> u32 {
    1
}

impl Animation {
    /// Create a new animation for a property
    pub fn new(property: impl Into<String>) -> AnimationBuilder {
        AnimationBuilder::new(property)
    }

    /// Get the current interpolated value based on progress
    pub fn current_value(&self) -> Option<AnimatableValue> {
        // Allow getting value when running, paused, or completed
        match self.state.status {
            AnimationStatus::Running | AnimationStatus::Paused | AnimationStatus::Completed => {}
            _ => return None,
        }

        // Apply easing to get the actual progress
        let t = self.state.eased_progress;

        // Handle direction
        let t = match self.state.current_direction {
            PlayDirection::Forward | PlayDirection::Alternate => t,
            PlayDirection::Reverse | PlayDirection::AlternateReverse => 1.0 - t,
        };

        self.from.interpolate(&self.to, t)
    }

    /// Update the animation by a time delta (in seconds)
    /// Returns true if the animation is still active
    pub fn update(&mut self, dt: f32) -> bool {
        match self.state.status {
            AnimationStatus::Idle => {
                // Start the animation
                if self.delay > 0.0 {
                    self.state.status = AnimationStatus::Delayed;
                    self.state.elapsed = 0.0;
                } else {
                    self.state.status = AnimationStatus::Running;
                    self.state.elapsed = 0.0;
                    self.state.iteration = 0;
                    self.update_direction();
                }
                true
            }
            AnimationStatus::Delayed => {
                self.state.elapsed += dt;
                if self.state.elapsed >= self.delay {
                    self.state.status = AnimationStatus::Running;
                    self.state.elapsed = self.state.elapsed - self.delay;
                    self.state.iteration = 0;
                    self.update_direction();
                }
                true
            }
            AnimationStatus::Running => {
                self.state.elapsed += dt;

                // Calculate progress within current iteration
                let iteration_progress = if self.duration > 0.0 {
                    (self.state.elapsed / self.duration).min(1.0)
                } else {
                    1.0
                };

                self.state.progress = iteration_progress;
                self.state.eased_progress = self.easing.evaluate(iteration_progress);
                self.state.remaining = (self.duration - self.state.elapsed).max(0.0);
                self.state.current_value = self.current_value();

                // Check if iteration is complete
                if self.state.elapsed >= self.duration {
                    self.state.iteration += 1;

                    if self.iteration_count == 0 || self.state.iteration < self.iteration_count {
                        // Start next iteration
                        self.state.elapsed = self.state.elapsed - self.duration;
                        self.update_direction();
                        true
                    } else {
                        // Animation complete
                        self.state.status = AnimationStatus::Completed;
                        self.state.progress = 1.0;
                        self.state.eased_progress = self.easing.evaluate(1.0);
                        self.state.current_value = self.current_value();
                        false
                    }
                } else {
                    true
                }
            }
            AnimationStatus::Paused => true,
            AnimationStatus::Completed | AnimationStatus::Cancelled => false,
        }
    }

    /// Update the current direction based on direction mode and iteration
    fn update_direction(&mut self) {
        self.state.current_direction = match self.direction {
            PlayDirection::Forward => PlayDirection::Forward,
            PlayDirection::Reverse => PlayDirection::Reverse,
            PlayDirection::Alternate => {
                if self.state.iteration % 2 == 0 {
                    PlayDirection::Forward
                } else {
                    PlayDirection::Reverse
                }
            }
            PlayDirection::AlternateReverse => {
                if self.state.iteration % 2 == 0 {
                    PlayDirection::Reverse
                } else {
                    PlayDirection::Forward
                }
            }
        };
    }

    /// Play the animation
    pub fn play(&mut self) {
        match self.state.status {
            AnimationStatus::Idle | AnimationStatus::Completed | AnimationStatus::Cancelled => {
                self.reset();
                self.state.status = if self.delay > 0.0 {
                    AnimationStatus::Delayed
                } else {
                    AnimationStatus::Running
                };
            }
            AnimationStatus::Paused => {
                self.state.status = AnimationStatus::Running;
            }
            _ => {}
        }
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state.status == AnimationStatus::Running {
            self.state.status = AnimationStatus::Paused;
        }
    }

    /// Resume a paused animation
    pub fn resume(&mut self) {
        if self.state.status == AnimationStatus::Paused {
            self.state.status = AnimationStatus::Running;
        }
    }

    /// Stop and reset the animation
    pub fn stop(&mut self) {
        self.state.status = AnimationStatus::Cancelled;
    }

    /// Reset the animation to its initial state
    pub fn reset(&mut self) {
        self.state = AnimationState::default();
    }

    /// Reverse the animation direction
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
    }

    /// Seek to a specific progress (0.0 to 1.0)
    pub fn seek(&mut self, progress: f32) {
        let progress = progress.clamp(0.0, 1.0);
        self.state.elapsed = progress * self.duration;
        self.state.progress = progress;
        self.state.eased_progress = self.easing.evaluate(progress);
        // Set status to Paused so current_value works
        if self.state.status == AnimationStatus::Idle {
            self.state.status = AnimationStatus::Paused;
        }
        self.state.current_value = self.current_value();
    }

    /// Get the total duration including all iterations
    pub fn total_duration(&self) -> f32 {
        if self.iteration_count == 0 {
            f32::INFINITY
        } else {
            self.delay + self.duration * self.iteration_count as f32
        }
    }
}

/// Builder for creating animations
#[derive(Debug, Clone)]
pub struct AnimationBuilder {
    property: String,
    from: Option<AnimatableValue>,
    to: Option<AnimatableValue>,
    duration: f32,
    delay: f32,
    easing: Easing,
    direction: PlayDirection,
    fill_mode: FillMode,
    iteration_count: u32,
}

impl AnimationBuilder {
    /// Create a new animation builder
    pub fn new(property: impl Into<String>) -> Self {
        Self {
            property: property.into(),
            from: None,
            to: None,
            duration: 0.3, // 300ms default
            delay: 0.0,
            easing: Easing::default(),
            direction: PlayDirection::default(),
            fill_mode: FillMode::default(),
            iteration_count: 1,
        }
    }

    /// Set the starting value
    pub fn from<V: Into<AnimatableValue>>(mut self, value: V) -> Self {
        self.from = Some(value.into());
        self
    }

    /// Set the ending value
    pub fn to<V: Into<AnimatableValue>>(mut self, value: V) -> Self {
        self.to = Some(value.into());
        self
    }

    /// Set duration in seconds
    pub fn duration(mut self, seconds: f32) -> Self {
        self.duration = seconds;
        self
    }

    /// Set duration in milliseconds
    pub fn duration_ms(mut self, ms: u32) -> Self {
        self.duration = ms as f32 / 1000.0;
        self
    }

    /// Set delay in seconds
    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds;
        self
    }

    /// Set delay in milliseconds
    pub fn delay_ms(mut self, ms: u32) -> Self {
        self.delay = ms as f32 / 1000.0;
        self
    }

    /// Set the easing function
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Set the play direction
    pub fn direction(mut self, direction: PlayDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the fill mode
    pub fn fill_mode(mut self, fill_mode: FillMode) -> Self {
        self.fill_mode = fill_mode;
        self
    }

    /// Set iteration count (0 = infinite)
    pub fn iterations(mut self, count: u32) -> Self {
        self.iteration_count = count;
        self
    }

    /// Set to repeat infinitely
    pub fn infinite(mut self) -> Self {
        self.iteration_count = 0;
        self
    }

    /// Set to alternate direction each iteration
    pub fn alternate(mut self) -> Self {
        self.direction = PlayDirection::Alternate;
        self
    }

    /// Build the animation
    /// Panics if from or to values are not set
    pub fn build(self) -> Animation {
        Animation {
            property: self.property,
            from: self.from.expect("Animation 'from' value must be set"),
            to: self.to.expect("Animation 'to' value must be set"),
            duration: self.duration,
            delay: self.delay,
            easing: self.easing,
            direction: self.direction,
            fill_mode: self.fill_mode,
            iteration_count: self.iteration_count,
            state: AnimationState::default(),
        }
    }

    /// Build the animation, returning None if from/to are not set
    pub fn try_build(self) -> Option<Animation> {
        Some(Animation {
            property: self.property,
            from: self.from?,
            to: self.to?,
            duration: self.duration,
            delay: self.delay,
            easing: self.easing,
            direction: self.direction,
            fill_mode: self.fill_mode,
            iteration_count: self.iteration_count,
            state: AnimationState::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_builder() {
        let anim = Animation::new("opacity")
            .from(0.0_f32)
            .to(1.0_f32)
            .duration_ms(300)
            .easing(Easing::EaseOut)
            .build();

        assert_eq!(anim.property, "opacity");
        assert!((anim.duration - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_animation_update() {
        let mut anim = Animation::new("opacity")
            .from(0.0_f32)
            .to(1.0_f32)
            .duration(1.0)
            .easing(Easing::Linear)
            .build();

        // Start animation
        anim.play();

        // Update for half the duration
        anim.update(0.5);

        assert_eq!(anim.state.status, AnimationStatus::Running);
        assert!((anim.state.progress - 0.5).abs() < 0.001);

        // Check interpolated value
        let value = anim.current_value().unwrap();
        assert!((value.as_float().unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_animation_complete() {
        let mut anim = Animation::new("x")
            .from(0.0_f32)
            .to(100.0_f32)
            .duration(1.0)
            .build();

        anim.play();

        // Update past the end
        let active = anim.update(1.5);

        assert!(!active);
        assert_eq!(anim.state.status, AnimationStatus::Completed);
        assert!((anim.state.progress - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_delay() {
        let mut anim = Animation::new("x")
            .from(0.0_f32)
            .to(100.0_f32)
            .duration(1.0)
            .delay(0.5)
            .build();

        anim.play();

        // Update within delay period
        anim.update(0.3);
        assert_eq!(anim.state.status, AnimationStatus::Delayed);

        // Update past delay
        anim.update(0.3);
        assert_eq!(anim.state.status, AnimationStatus::Running);
    }

    #[test]
    fn test_animation_iterations() {
        let mut anim = Animation::new("x")
            .from(0.0_f32)
            .to(100.0_f32)
            .duration(1.0)
            .iterations(2)
            .build();

        anim.play();

        // Complete first iteration
        anim.update(1.0);
        assert_eq!(anim.state.status, AnimationStatus::Running);
        assert_eq!(anim.state.iteration, 1);

        // Complete second iteration
        let active = anim.update(1.0);
        assert!(!active);
        assert_eq!(anim.state.status, AnimationStatus::Completed);
    }

    #[test]
    fn test_animation_alternate() {
        let mut anim = Animation::new("x")
            .from(0.0_f32)
            .to(100.0_f32)
            .duration(1.0)
            .iterations(2)
            .alternate()
            .build();

        anim.play();

        // First iteration should be forward
        assert_eq!(anim.state.current_direction, PlayDirection::Forward);

        // Complete first iteration
        anim.update(1.0);

        // Second iteration should be reverse
        assert_eq!(anim.state.current_direction, PlayDirection::Reverse);
    }

    #[test]
    fn test_animation_seek() {
        let mut anim = Animation::new("x")
            .from(0.0_f32)
            .to(100.0_f32)
            .duration(1.0)
            .easing(Easing::Linear)
            .build();

        anim.seek(0.75);

        assert!((anim.state.progress - 0.75).abs() < 0.001);
        let value = anim.current_value().unwrap();
        assert!((value.as_float().unwrap() - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_pause_resume() {
        let mut anim = Animation::new("x")
            .from(0.0_f32)
            .to(100.0_f32)
            .duration(1.0)
            .build();

        anim.play();
        anim.update(0.5);

        anim.pause();
        assert_eq!(anim.state.status, AnimationStatus::Paused);

        // Update while paused should not advance
        let progress_before = anim.state.progress;
        anim.update(0.2);
        assert!((anim.state.progress - progress_before).abs() < 0.001);

        anim.resume();
        assert_eq!(anim.state.status, AnimationStatus::Running);
    }

    #[test]
    fn test_infinite_animation() {
        let anim = Animation::new("x")
            .from(0.0_f32)
            .to(100.0_f32)
            .duration(1.0)
            .infinite()
            .build();

        assert_eq!(anim.iteration_count, 0);
        assert!(anim.total_duration().is_infinite());
    }
}
