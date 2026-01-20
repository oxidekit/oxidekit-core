//! Gesture-Driven Animations
//!
//! Provides physics-based animations that respond to gesture input,
//! including velocity-based springs, inertial scrolling, and drag animations.
//!
//! # Example
//!
//! ```rust
//! use oxide_components::animation::gesture::{GestureSpring, InertialScroll};
//!
//! // Create a spring that responds to drag velocity
//! let mut spring = GestureSpring::new(0.0, 100.0);
//! spring.set_velocity(500.0); // pixels per second from drag gesture
//!
//! // Update each frame
//! let value = spring.update(0.016);
//! ```

use super::motion_tokens::SpringParams;

// =============================================================================
// Gesture Spring
// =============================================================================

/// A spring animation driven by gesture velocity
#[derive(Debug, Clone)]
pub struct GestureSpring {
    /// Current position
    position: f32,
    /// Current velocity
    velocity: f32,
    /// Target position
    target: f32,
    /// Spring parameters
    params: SpringParams,
    /// Whether the spring has settled
    at_rest: bool,
    /// Velocity threshold for considering spring at rest
    rest_velocity_threshold: f32,
    /// Position threshold for considering spring at rest
    rest_position_threshold: f32,
}

impl GestureSpring {
    /// Create a new gesture spring
    pub fn new(position: f32, target: f32) -> Self {
        Self {
            position,
            velocity: 0.0,
            target,
            params: SpringParams::default(),
            at_rest: position == target,
            rest_velocity_threshold: 0.01,
            rest_position_threshold: 0.001,
        }
    }

    /// Create with initial velocity (from gesture)
    pub fn with_velocity(position: f32, target: f32, velocity: f32) -> Self {
        let mut spring = Self::new(position, target);
        spring.velocity = velocity;
        spring.at_rest = false;
        spring
    }

    /// Set spring parameters
    pub fn params(mut self, params: SpringParams) -> Self {
        self.params = params;
        self
    }

    /// Set stiffness
    pub fn stiffness(mut self, stiffness: f32) -> Self {
        self.params.stiffness = stiffness;
        self
    }

    /// Set damping
    pub fn damping(mut self, damping: f32) -> Self {
        self.params.damping = damping;
        self
    }

    /// Set mass
    pub fn mass(mut self, mass: f32) -> Self {
        self.params.mass = mass;
        self
    }

    /// Set velocity (from drag gesture)
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
        self.at_rest = false;
    }

    /// Set target position
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
        self.at_rest = false;
    }

    /// Set current position (for dragging)
    pub fn set_position(&mut self, position: f32) {
        self.position = position;
        self.at_rest = false;
    }

    /// Snap to target instantly
    pub fn snap_to_target(&mut self) {
        self.position = self.target;
        self.velocity = 0.0;
        self.at_rest = true;
    }

    /// Update the spring and return current position
    pub fn update(&mut self, dt: f32) -> f32 {
        if self.at_rest {
            return self.position;
        }

        // Spring force: F = -k * displacement
        let displacement = self.position - self.target;
        let spring_force = -self.params.stiffness * displacement;

        // Damping force: F = -c * velocity
        let damping_force = -self.params.damping * self.velocity;

        // Total force and acceleration
        let force = spring_force + damping_force;
        let acceleration = force / self.params.mass;

        // Integration (semi-implicit Euler for stability)
        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;

        // Check if at rest
        if self.velocity.abs() < self.rest_velocity_threshold
            && (self.position - self.target).abs() < self.rest_position_threshold
        {
            self.position = self.target;
            self.velocity = 0.0;
            self.at_rest = true;
        }

        self.position
    }

    /// Get current position
    pub fn position(&self) -> f32 {
        self.position
    }

    /// Get current velocity
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Get target position
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Check if spring is at rest
    pub fn is_at_rest(&self) -> bool {
        self.at_rest
    }

    /// Check if spring is still animating
    pub fn is_animating(&self) -> bool {
        !self.at_rest
    }
}

// =============================================================================
// 2D Gesture Spring
// =============================================================================

/// A 2D spring for x/y position animations (like drag)
#[derive(Debug, Clone)]
pub struct GestureSpring2D {
    /// X-axis spring
    x: GestureSpring,
    /// Y-axis spring
    y: GestureSpring,
}

impl GestureSpring2D {
    /// Create a new 2D gesture spring
    pub fn new(position: (f32, f32), target: (f32, f32)) -> Self {
        Self {
            x: GestureSpring::new(position.0, target.0),
            y: GestureSpring::new(position.1, target.1),
        }
    }

    /// Create with initial velocity
    pub fn with_velocity(
        position: (f32, f32),
        target: (f32, f32),
        velocity: (f32, f32),
    ) -> Self {
        Self {
            x: GestureSpring::with_velocity(position.0, target.0, velocity.0),
            y: GestureSpring::with_velocity(position.1, target.1, velocity.1),
        }
    }

    /// Set spring parameters for both axes
    pub fn params(mut self, params: SpringParams) -> Self {
        self.x.params = params;
        self.y.params = params;
        self
    }

    /// Set velocity
    pub fn set_velocity(&mut self, velocity: (f32, f32)) {
        self.x.set_velocity(velocity.0);
        self.y.set_velocity(velocity.1);
    }

    /// Set target
    pub fn set_target(&mut self, target: (f32, f32)) {
        self.x.set_target(target.0);
        self.y.set_target(target.1);
    }

    /// Set position
    pub fn set_position(&mut self, position: (f32, f32)) {
        self.x.set_position(position.0);
        self.y.set_position(position.1);
    }

    /// Update and return current position
    pub fn update(&mut self, dt: f32) -> (f32, f32) {
        (self.x.update(dt), self.y.update(dt))
    }

    /// Get current position
    pub fn position(&self) -> (f32, f32) {
        (self.x.position(), self.y.position())
    }

    /// Get current velocity
    pub fn velocity(&self) -> (f32, f32) {
        (self.x.velocity(), self.y.velocity())
    }

    /// Check if at rest
    pub fn is_at_rest(&self) -> bool {
        self.x.is_at_rest() && self.y.is_at_rest()
    }

    /// Check if animating
    pub fn is_animating(&self) -> bool {
        !self.is_at_rest()
    }
}

// =============================================================================
// Inertial Scroll
// =============================================================================

/// Inertial scrolling with deceleration (like iOS scroll views)
#[derive(Debug, Clone)]
pub struct InertialScroll {
    /// Current position
    position: f32,
    /// Current velocity
    velocity: f32,
    /// Deceleration rate (friction)
    deceleration: f32,
    /// Minimum velocity to stop
    min_velocity: f32,
    /// Content bounds (min, max)
    bounds: Option<(f32, f32)>,
    /// Bounce amount when hitting bounds
    bounce_stiffness: f32,
    /// Whether to allow bouncing past bounds
    bounce_enabled: bool,
}

impl InertialScroll {
    /// Create new inertial scroll
    pub fn new(position: f32) -> Self {
        Self {
            position,
            velocity: 0.0,
            deceleration: 0.998, // Natural feeling deceleration
            min_velocity: 0.1,
            bounds: None,
            bounce_stiffness: 300.0,
            bounce_enabled: true,
        }
    }

    /// Set initial velocity (from fling gesture)
    pub fn fling(&mut self, velocity: f32) {
        self.velocity = velocity;
    }

    /// Set content bounds
    pub fn with_bounds(mut self, min: f32, max: f32) -> Self {
        self.bounds = Some((min, max));
        self
    }

    /// Set deceleration rate
    pub fn with_deceleration(mut self, rate: f32) -> Self {
        self.deceleration = rate.clamp(0.9, 0.999);
        self
    }

    /// Enable/disable bouncing
    pub fn with_bounce(mut self, enabled: bool) -> Self {
        self.bounce_enabled = enabled;
        self
    }

    /// Set position (for dragging)
    pub fn set_position(&mut self, position: f32) {
        self.position = position;
        self.velocity = 0.0;
    }

    /// Update and return current position
    pub fn update(&mut self, dt: f32) -> f32 {
        // Apply deceleration
        self.velocity *= self.deceleration.powf(dt * 60.0);

        // Update position
        self.position += self.velocity * dt;

        // Handle bounds
        if let Some((min, max)) = self.bounds {
            if self.position < min {
                if self.bounce_enabled {
                    // Spring-like rubber band effect with damping
                    let overshoot = min - self.position;
                    let spring_force = self.bounce_stiffness * overshoot;
                    let damping_force = -self.bounce_stiffness * 0.3 * self.velocity;
                    self.velocity += (spring_force + damping_force) * dt;
                } else {
                    self.position = min;
                    self.velocity = 0.0;
                }
            } else if self.position > max {
                if self.bounce_enabled {
                    let overshoot = self.position - max;
                    let spring_force = -self.bounce_stiffness * overshoot;
                    let damping_force = -self.bounce_stiffness * 0.3 * self.velocity;
                    self.velocity += (spring_force + damping_force) * dt;
                } else {
                    self.position = max;
                    self.velocity = 0.0;
                }
            }
        }

        // Stop if velocity is very low and position is near bounds
        if self.velocity.abs() < self.min_velocity {
            if let Some((min, max)) = self.bounds {
                let near_min = (self.position - min).abs() < 1.0;
                let near_max = (self.position - max).abs() < 1.0;
                let in_bounds = self.position >= min && self.position <= max;

                if near_min || near_max || in_bounds {
                    self.velocity = 0.0;
                    // Snap to bounds if close
                    if self.position < min {
                        self.position = min;
                    } else if self.position > max {
                        self.position = max;
                    }
                }
            } else {
                self.velocity = 0.0;
            }
        }

        self.position
    }

    /// Get current position
    pub fn position(&self) -> f32 {
        self.position
    }

    /// Get current velocity
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Check if scrolling has stopped
    pub fn is_at_rest(&self) -> bool {
        self.velocity == 0.0
    }

    /// Check if still animating
    pub fn is_animating(&self) -> bool {
        self.velocity.abs() > self.min_velocity
    }

    /// Calculate final resting position
    pub fn projected_position(&self) -> f32 {
        // Sum of geometric series for deceleration
        let frames = 60.0; // Approximate frames per second
        let sum = self.velocity / (1.0 - self.deceleration.powf(1.0 / frames));
        let projected = self.position + sum / frames;

        // Clamp to bounds
        if let Some((min, max)) = self.bounds {
            projected.clamp(min, max)
        } else {
            projected
        }
    }
}

// =============================================================================
// 2D Inertial Scroll
// =============================================================================

/// 2D inertial scrolling for scroll views
#[derive(Debug, Clone)]
pub struct InertialScroll2D {
    /// X-axis scroll
    x: InertialScroll,
    /// Y-axis scroll
    y: InertialScroll,
}

impl InertialScroll2D {
    /// Create new 2D inertial scroll
    pub fn new(position: (f32, f32)) -> Self {
        Self {
            x: InertialScroll::new(position.0),
            y: InertialScroll::new(position.1),
        }
    }

    /// Set content bounds
    pub fn with_bounds(mut self, min: (f32, f32), max: (f32, f32)) -> Self {
        self.x = self.x.with_bounds(min.0, max.0);
        self.y = self.y.with_bounds(min.1, max.1);
        self
    }

    /// Fling with velocity
    pub fn fling(&mut self, velocity: (f32, f32)) {
        self.x.fling(velocity.0);
        self.y.fling(velocity.1);
    }

    /// Set position
    pub fn set_position(&mut self, position: (f32, f32)) {
        self.x.set_position(position.0);
        self.y.set_position(position.1);
    }

    /// Update and return position
    pub fn update(&mut self, dt: f32) -> (f32, f32) {
        (self.x.update(dt), self.y.update(dt))
    }

    /// Get current position
    pub fn position(&self) -> (f32, f32) {
        (self.x.position(), self.y.position())
    }

    /// Check if at rest
    pub fn is_at_rest(&self) -> bool {
        self.x.is_at_rest() && self.y.is_at_rest()
    }
}

// =============================================================================
// Snap Points
// =============================================================================

/// Snap to discrete points with spring physics
#[derive(Debug, Clone)]
pub struct SnapSpring {
    /// Current position
    position: f32,
    /// Current velocity
    velocity: f32,
    /// Snap points (must be sorted)
    snap_points: Vec<f32>,
    /// Current target snap point index
    target_index: usize,
    /// Spring parameters
    params: SpringParams,
    /// Velocity threshold for snapping
    snap_velocity_threshold: f32,
    /// Position threshold for being "at" a snap point
    snap_position_threshold: f32,
    /// Whether currently at rest
    at_rest: bool,
}

impl SnapSpring {
    /// Create a new snap spring with snap points
    pub fn new(position: f32, snap_points: Vec<f32>) -> Self {
        let mut sorted_points = snap_points;
        sorted_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let target_index = Self::find_nearest_index(&sorted_points, position);

        Self {
            position,
            velocity: 0.0,
            snap_points: sorted_points,
            target_index,
            params: SpringParams::snappy(),
            snap_velocity_threshold: 50.0, // Min velocity to change snap point
            snap_position_threshold: 0.5,
            at_rest: true,
        }
    }

    /// Find the index of the nearest snap point
    fn find_nearest_index(points: &[f32], position: f32) -> usize {
        points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (position - *a).abs().partial_cmp(&(position - *b).abs()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Set velocity (from drag gesture end)
    pub fn release_with_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
        self.at_rest = false;

        // Determine target based on velocity and position
        if velocity.abs() > self.snap_velocity_threshold {
            // Snap in direction of velocity
            if velocity > 0.0 && self.target_index < self.snap_points.len() - 1 {
                self.target_index += 1;
            } else if velocity < 0.0 && self.target_index > 0 {
                self.target_index -= 1;
            }
        } else {
            // Snap to nearest
            self.target_index = Self::find_nearest_index(&self.snap_points, self.position);
        }
    }

    /// Snap to a specific index
    pub fn snap_to_index(&mut self, index: usize) {
        if index < self.snap_points.len() {
            self.target_index = index;
            self.at_rest = false;
        }
    }

    /// Snap to next point
    pub fn snap_next(&mut self) {
        if self.target_index < self.snap_points.len() - 1 {
            self.target_index += 1;
            self.at_rest = false;
        }
    }

    /// Snap to previous point
    pub fn snap_previous(&mut self) {
        if self.target_index > 0 {
            self.target_index -= 1;
            self.at_rest = false;
        }
    }

    /// Set position (while dragging)
    pub fn set_position(&mut self, position: f32) {
        self.position = position;
        self.velocity = 0.0;
        self.at_rest = false;
    }

    /// Update and return position
    pub fn update(&mut self, dt: f32) -> f32 {
        if self.at_rest || self.snap_points.is_empty() {
            return self.position;
        }

        let target = self.snap_points[self.target_index];

        // Spring physics
        let displacement = self.position - target;
        let spring_force = -self.params.stiffness * displacement;
        let damping_force = -self.params.damping * self.velocity;
        let acceleration = (spring_force + damping_force) / self.params.mass;

        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;

        // Check if at rest
        if self.velocity.abs() < 0.01 && displacement.abs() < self.snap_position_threshold {
            self.position = target;
            self.velocity = 0.0;
            self.at_rest = true;
        }

        self.position
    }

    /// Get current position
    pub fn position(&self) -> f32 {
        self.position
    }

    /// Get target snap point position
    pub fn target(&self) -> f32 {
        self.snap_points.get(self.target_index).copied().unwrap_or(self.position)
    }

    /// Get current snap point index
    pub fn current_index(&self) -> usize {
        self.target_index
    }

    /// Check if at rest
    pub fn is_at_rest(&self) -> bool {
        self.at_rest
    }
}

// =============================================================================
// Decay Animation (momentum-based)
// =============================================================================

/// Decay animation that preserves momentum and gradually slows
#[derive(Debug, Clone)]
pub struct DecayAnimation {
    /// Current value
    value: f32,
    /// Current velocity
    velocity: f32,
    /// Decay constant (time constant)
    time_constant: f32,
    /// Whether animation has stopped
    stopped: bool,
    /// Minimum velocity threshold
    min_velocity: f32,
}

impl DecayAnimation {
    /// Create a new decay animation
    pub fn new(value: f32, velocity: f32) -> Self {
        Self {
            value,
            velocity,
            time_constant: 0.325, // Similar to iOS
            stopped: velocity.abs() < 0.001,
            min_velocity: 0.01,
        }
    }

    /// Set time constant (lower = faster decay)
    pub fn with_time_constant(mut self, tc: f32) -> Self {
        self.time_constant = tc.max(0.01);
        self
    }

    /// Update and return current value
    pub fn update(&mut self, dt: f32) -> f32 {
        if self.stopped {
            return self.value;
        }

        // Exponential decay: v(t) = v0 * e^(-t/tc)
        let decay = (-dt / self.time_constant).exp();
        self.velocity *= decay;
        self.value += self.velocity * dt;

        if self.velocity.abs() < self.min_velocity {
            self.velocity = 0.0;
            self.stopped = true;
        }

        self.value
    }

    /// Get current value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Get current velocity
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// Project final resting position
    pub fn projected_value(&self) -> f32 {
        self.value + self.velocity * self.time_constant
    }
}

// =============================================================================
// Rubberband (elastic boundary)
// =============================================================================

/// Rubberband effect for elastic boundaries
#[derive(Debug, Clone, Copy)]
pub struct Rubberband {
    /// Resistance factor (0-1, lower = more resistance)
    resistance: f32,
    /// Maximum stretch distance
    max_stretch: f32,
}

impl Rubberband {
    /// Create a new rubberband effect
    pub fn new(resistance: f32) -> Self {
        Self {
            resistance: resistance.clamp(0.1, 1.0),
            max_stretch: f32::MAX,
        }
    }

    /// Set maximum stretch
    pub fn with_max_stretch(mut self, max: f32) -> Self {
        self.max_stretch = max;
        self
    }

    /// Calculate rubberbanded position
    pub fn apply(&self, overshoot: f32) -> f32 {
        // Logarithmic resistance
        let sign = overshoot.signum();
        let abs_overshoot = overshoot.abs().min(self.max_stretch);

        let stretched = abs_overshoot * self.resistance
            * (1.0 + (abs_overshoot / self.max_stretch).ln_1p());

        sign * stretched.min(self.max_stretch * self.resistance)
    }
}

impl Default for Rubberband {
    fn default() -> Self {
        Self::new(0.55) // iOS-like resistance
    }
}

// =============================================================================
// Velocity Tracker (for gesture handling)
// =============================================================================

/// Tracks velocity from position samples
#[derive(Debug, Clone)]
pub struct VelocityTracker {
    /// Recent samples (position, timestamp)
    samples: Vec<(f32, f32)>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Maximum age of samples (seconds)
    max_age: f32,
}

impl VelocityTracker {
    /// Create a new velocity tracker
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(20),
            max_samples: 20,
            max_age: 0.1, // 100ms window
        }
    }

    /// Add a position sample
    pub fn add_sample(&mut self, position: f32, timestamp: f32) {
        // Remove old samples
        self.samples.retain(|(_, t)| timestamp - t < self.max_age);

        // Add new sample
        self.samples.push((position, timestamp));

        // Limit sample count
        while self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }
    }

    /// Reset the tracker
    pub fn reset(&mut self) {
        self.samples.clear();
    }

    /// Calculate current velocity
    pub fn velocity(&self) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }

        // Use weighted average of recent velocities
        let mut total_velocity = 0.0;
        let mut total_weight = 0.0;

        for i in 1..self.samples.len() {
            let (pos0, t0) = self.samples[i - 1];
            let (pos1, t1) = self.samples[i];

            let dt = t1 - t0;
            if dt > 0.0001 {
                let v = (pos1 - pos0) / dt;
                // Weight more recent samples higher
                let weight = (i as f32) / (self.samples.len() as f32);
                total_velocity += v * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            total_velocity / total_weight
        } else {
            0.0
        }
    }
}

impl Default for VelocityTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 2D velocity tracker
#[derive(Debug, Clone, Default)]
pub struct VelocityTracker2D {
    /// X-axis tracker
    x: VelocityTracker,
    /// Y-axis tracker
    y: VelocityTracker,
}

impl VelocityTracker2D {
    /// Create a new 2D velocity tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a position sample
    pub fn add_sample(&mut self, position: (f32, f32), timestamp: f32) {
        self.x.add_sample(position.0, timestamp);
        self.y.add_sample(position.1, timestamp);
    }

    /// Reset the tracker
    pub fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    /// Calculate current velocity
    pub fn velocity(&self) -> (f32, f32) {
        (self.x.velocity(), self.y.velocity())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gesture_spring_basic() {
        let mut spring = GestureSpring::new(0.0, 100.0);
        spring.set_velocity(500.0);

        // Update several frames
        let mut position = 0.0;
        for _ in 0..100 {
            position = spring.update(0.016);
        }

        // Should approach target
        assert!((position - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_gesture_spring_at_rest() {
        let mut spring = GestureSpring::new(100.0, 100.0);
        assert!(spring.is_at_rest());

        // Setting velocity should wake it up
        spring.set_velocity(50.0);
        assert!(!spring.is_at_rest());

        // Eventually settle
        for _ in 0..200 {
            spring.update(0.016);
        }
        assert!(spring.is_at_rest());
    }

    #[test]
    fn test_gesture_spring_2d() {
        let mut spring = GestureSpring2D::new((0.0, 0.0), (100.0, 200.0));
        spring.set_velocity((300.0, 600.0));

        for _ in 0..150 {
            spring.update(0.016);
        }

        let pos = spring.position();
        assert!((pos.0 - 100.0).abs() < 1.0);
        assert!((pos.1 - 200.0).abs() < 1.0);
    }

    #[test]
    fn test_inertial_scroll() {
        let mut scroll = InertialScroll::new(0.0)
            .with_bounds(0.0, 1000.0)
            .with_bounce(false); // Disable bounce for this test

        scroll.fling(500.0);

        // Update several frames
        for _ in 0..200 {
            scroll.update(0.016);
        }

        // Should have moved and be within bounds
        assert!(scroll.position() >= 0.0);
        assert!(scroll.position() <= 1000.0);
    }

    #[test]
    fn test_inertial_scroll_bounce() {
        let mut scroll = InertialScroll::new(50.0)
            .with_bounds(0.0, 100.0)
            .with_bounce(true);

        scroll.fling(-1000.0); // Fling towards min bound

        // Update many frames for bounce to settle
        for _ in 0..1000 {
            scroll.update(0.016);
        }

        // Should be very close to min bound
        assert!(scroll.position().abs() < 5.0);
    }

    #[test]
    fn test_snap_spring() {
        let mut snap = SnapSpring::new(0.0, vec![0.0, 50.0, 100.0]);

        // Set position between snap points and release
        snap.set_position(30.0);
        snap.release_with_velocity(100.0); // Positive velocity

        // Should snap to 50.0
        for _ in 0..100 {
            snap.update(0.016);
        }

        assert!((snap.position() - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_snap_spring_velocity_override() {
        let mut snap = SnapSpring::new(0.0, vec![0.0, 100.0, 200.0]);

        snap.set_position(25.0); // Closer to 0.0
        snap.release_with_velocity(500.0); // Strong velocity towards 100.0

        for _ in 0..100 {
            snap.update(0.016);
        }

        // Should snap to 100.0 due to velocity
        assert_eq!(snap.current_index(), 1);
    }

    #[test]
    fn test_decay_animation() {
        let mut decay = DecayAnimation::new(0.0, 100.0);

        // Run animation until stopped
        for _ in 0..1000 {
            decay.update(0.016);
            if decay.is_stopped() {
                break;
            }
        }

        // Should have stopped
        assert!(decay.is_stopped());
        // Value should be positive (moved in direction of initial velocity)
        assert!(decay.value() > 0.0);
    }

    #[test]
    fn test_rubberband() {
        let rubber = Rubberband::new(0.55).with_max_stretch(200.0);

        // Small overshoot
        let stretched = rubber.apply(10.0);
        assert!(stretched > 0.0);

        // Large overshoot
        let large_stretch = rubber.apply(100.0);
        assert!(large_stretch > stretched);

        // Negative overshoot
        let neg_stretch = rubber.apply(-10.0);
        assert!(neg_stretch < 0.0);
    }

    #[test]
    fn test_velocity_tracker() {
        let mut tracker = VelocityTracker::new();

        // Add samples moving at 100 units per second
        // Use timestamps within max_age window
        let base_time = 1.0;
        tracker.add_sample(0.0, base_time);
        tracker.add_sample(1.0, base_time + 0.01);
        tracker.add_sample(2.0, base_time + 0.02);
        tracker.add_sample(3.0, base_time + 0.03);

        let velocity = tracker.velocity();
        // Velocity should be approximately 100 units/second
        assert!(velocity > 50.0);
        assert!(velocity < 150.0);
    }

    #[test]
    fn test_velocity_tracker_2d() {
        let mut tracker = VelocityTracker2D::new();

        // Use timestamps within max_age window
        let base_time = 1.0;
        tracker.add_sample((0.0, 0.0), base_time);
        tracker.add_sample((1.0, 2.0), base_time + 0.01);
        tracker.add_sample((2.0, 4.0), base_time + 0.02);

        let velocity = tracker.velocity();
        // X velocity should be ~100, Y velocity should be ~200
        assert!(velocity.0 > 50.0);
        assert!(velocity.1 > 100.0);
    }

    #[test]
    fn test_gesture_spring_snap() {
        let mut spring = GestureSpring::new(50.0, 100.0);
        spring.snap_to_target();

        assert_eq!(spring.position(), 100.0);
        assert_eq!(spring.velocity(), 0.0);
        assert!(spring.is_at_rest());
    }
}
