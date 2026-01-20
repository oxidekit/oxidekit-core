//! Gesture recognizer trait and core types
//!
//! This module defines the core trait that all gesture recognizers implement,
//! along with the fundamental types used throughout the gesture system.

use crate::velocity::Velocity;
use std::time::Instant;

/// A 2D point in screen coordinates
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Origin point (0, 0)
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate squared distance (faster, avoids sqrt)
    pub fn distance_squared_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Get midpoint between two points
    pub fn midpoint(&self, other: &Point) -> Point {
        Point {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<Point> for (f32, f32) {
    fn from(p: Point) -> Self {
        (p.x, p.y)
    }
}

/// A 2D vector representing displacement or direction
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector {
    pub dx: f32,
    pub dy: f32,
}

impl Vector {
    /// Create a new vector
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }

    /// Zero vector
    pub fn zero() -> Self {
        Self { dx: 0.0, dy: 0.0 }
    }

    /// Calculate the magnitude (length)
    pub fn magnitude(&self) -> f32 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }

    /// Get the direction in radians
    pub fn direction(&self) -> f32 {
        self.dy.atan2(self.dx)
    }

    /// Normalize to unit vector
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                dx: self.dx / mag,
                dy: self.dy / mag,
            }
        } else {
            Self::zero()
        }
    }

    /// Create vector from two points
    pub fn from_points(from: &Point, to: &Point) -> Self {
        Self {
            dx: to.x - from.x,
            dy: to.y - from.y,
        }
    }
}

impl From<(f32, f32)> for Vector {
    fn from((dx, dy): (f32, f32)) -> Self {
        Self { dx, dy }
    }
}

/// The state of a gesture recognizer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureState {
    /// Gesture recognizer is idle, waiting for input
    Possible,
    /// Gesture has begun (first recognition)
    Began,
    /// Gesture is ongoing and has changed
    Changed,
    /// Gesture has completed successfully
    Ended,
    /// Gesture was cancelled (e.g., by system or another gesture)
    Cancelled,
    /// Gesture recognition failed (input didn't match)
    Failed,
}

impl GestureState {
    /// Check if the gesture is in an active state (began or changed)
    pub fn is_active(&self) -> bool {
        matches!(self, GestureState::Began | GestureState::Changed)
    }

    /// Check if the gesture has finished (ended, cancelled, or failed)
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            GestureState::Ended | GestureState::Cancelled | GestureState::Failed
        )
    }

    /// Check if the gesture completed successfully
    pub fn is_successful(&self) -> bool {
        matches!(self, GestureState::Ended)
    }
}

/// Types of recognized gestures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureType {
    Tap,
    DoubleTap,
    LongPress,
    PressAndHold,
    Pan,
    Drag,
    Swipe,
    Fling,
    Pinch,
    Rotation,
    TwoFingerPan,
}

/// Direction of swipe gestures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

impl SwipeDirection {
    /// Get the direction from a vector
    pub fn from_vector(v: &Vector) -> Option<Self> {
        if v.dx.abs() > v.dy.abs() {
            // Horizontal swipe
            if v.dx > 0.0 {
                Some(SwipeDirection::Right)
            } else {
                Some(SwipeDirection::Left)
            }
        } else if v.dy.abs() > 0.0 {
            // Vertical swipe
            if v.dy > 0.0 {
                Some(SwipeDirection::Down)
            } else {
                Some(SwipeDirection::Up)
            }
        } else {
            None
        }
    }
}

/// A pointer (touch or mouse) event
#[derive(Debug, Clone)]
pub struct PointerEvent {
    /// Unique identifier for this pointer
    pub pointer_id: u64,
    /// Current position
    pub position: Point,
    /// Event phase
    pub phase: PointerPhase,
    /// Timestamp of the event
    pub timestamp: Instant,
    /// Pressure (0.0 to 1.0, if available)
    pub pressure: Option<f32>,
    /// Whether this is a touch or mouse event
    pub pointer_type: PointerType,
}

impl PointerEvent {
    /// Create a new pointer event
    pub fn new(pointer_id: u64, position: Point, phase: PointerPhase) -> Self {
        Self {
            pointer_id,
            position,
            phase,
            timestamp: Instant::now(),
            pressure: None,
            pointer_type: PointerType::Touch,
        }
    }

    /// Create a touch down event
    pub fn touch_down(pointer_id: u64, x: f32, y: f32) -> Self {
        Self::new(pointer_id, Point::new(x, y), PointerPhase::Began)
    }

    /// Create a touch move event
    pub fn touch_move(pointer_id: u64, x: f32, y: f32) -> Self {
        Self::new(pointer_id, Point::new(x, y), PointerPhase::Moved)
    }

    /// Create a touch up event
    pub fn touch_up(pointer_id: u64, x: f32, y: f32) -> Self {
        Self::new(pointer_id, Point::new(x, y), PointerPhase::Ended)
    }

    /// Create a cancelled event
    pub fn cancelled(pointer_id: u64, x: f32, y: f32) -> Self {
        Self::new(pointer_id, Point::new(x, y), PointerPhase::Cancelled)
    }
}

/// Phase of a pointer event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerPhase {
    /// Touch/press started
    Began,
    /// Touch/press moved
    Moved,
    /// Touch/press ended
    Ended,
    /// Touch/press was cancelled
    Cancelled,
}

/// Type of pointer device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PointerType {
    #[default]
    Touch,
    Mouse,
    Stylus,
}

/// Event emitted when a gesture is recognized
#[derive(Debug, Clone)]
pub struct GestureEvent {
    /// Type of gesture
    pub gesture_type: GestureType,
    /// Current state of the gesture
    pub state: GestureState,
    /// Position where the gesture occurred (or center for multi-touch)
    pub position: Point,
    /// Movement delta since last event
    pub delta: Option<Vector>,
    /// Current velocity
    pub velocity: Option<Velocity>,
    /// Scale factor for pinch gestures (1.0 = no change)
    pub scale: Option<f32>,
    /// Rotation angle in radians for rotation gestures
    pub rotation: Option<f32>,
    /// Number of pointers involved
    pub pointer_count: usize,
    /// Swipe direction (for swipe gestures)
    pub direction: Option<SwipeDirection>,
    /// Number of taps (for multi-tap gestures)
    pub tap_count: usize,
}

impl GestureEvent {
    /// Create a new gesture event
    pub fn new(gesture_type: GestureType, state: GestureState, position: Point) -> Self {
        Self {
            gesture_type,
            state,
            position,
            delta: None,
            velocity: None,
            scale: None,
            rotation: None,
            pointer_count: 1,
            direction: None,
            tap_count: 1,
        }
    }

    /// Set the delta
    pub fn with_delta(mut self, delta: Vector) -> Self {
        self.delta = Some(delta);
        self
    }

    /// Set the velocity
    pub fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = Some(velocity);
        self
    }

    /// Set the scale
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }

    /// Set the rotation
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = Some(rotation);
        self
    }

    /// Set the pointer count
    pub fn with_pointer_count(mut self, count: usize) -> Self {
        self.pointer_count = count;
        self
    }

    /// Set the swipe direction
    pub fn with_direction(mut self, direction: SwipeDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set the tap count
    pub fn with_tap_count(mut self, count: usize) -> Self {
        self.tap_count = count;
        self
    }
}

/// Unique identifier for a gesture recognizer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GestureRecognizerId(pub u64);

impl GestureRecognizerId {
    /// Generate a new unique ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for GestureRecognizerId {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait implemented by all gesture recognizers
pub trait GestureRecognizer: Send {
    /// Get the unique identifier for this recognizer
    fn id(&self) -> GestureRecognizerId;

    /// Get the type of gesture this recognizer detects
    fn gesture_type(&self) -> GestureType;

    /// Get the current state
    fn state(&self) -> GestureState;

    /// Process a pointer event
    ///
    /// Returns a gesture event if the gesture was recognized or updated.
    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent>;

    /// Reset the recognizer to its initial state
    fn reset(&mut self);

    /// Get the priority of this recognizer (higher = more priority)
    fn priority(&self) -> i32 {
        0
    }

    /// Check if this recognizer should receive events before another
    fn should_recognize_simultaneously_with(&self, _other: &dyn GestureRecognizer) -> bool {
        false
    }

    /// Called when another recognizer claims the gesture
    fn cancel(&mut self) {
        self.reset();
    }

    /// Check if this recognizer requires exclusive access
    fn requires_exclusive(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_point_midpoint() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 20.0);
        let mid = p1.midpoint(&p2);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 10.0);
    }

    #[test]
    fn test_vector_magnitude() {
        let v = Vector::new(3.0, 4.0);
        assert!((v.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_normalized() {
        let v = Vector::new(3.0, 4.0);
        let n = v.normalized();
        assert!((n.magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_from_points() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(4.0, 6.0);
        let v = Vector::from_points(&p1, &p2);
        assert_eq!(v.dx, 3.0);
        assert_eq!(v.dy, 4.0);
    }

    #[test]
    fn test_gesture_state() {
        assert!(GestureState::Began.is_active());
        assert!(GestureState::Changed.is_active());
        assert!(!GestureState::Ended.is_active());

        assert!(GestureState::Ended.is_finished());
        assert!(GestureState::Cancelled.is_finished());
        assert!(GestureState::Failed.is_finished());
        assert!(!GestureState::Began.is_finished());

        assert!(GestureState::Ended.is_successful());
        assert!(!GestureState::Failed.is_successful());
    }

    #[test]
    fn test_swipe_direction() {
        assert_eq!(
            SwipeDirection::from_vector(&Vector::new(100.0, 10.0)),
            Some(SwipeDirection::Right)
        );
        assert_eq!(
            SwipeDirection::from_vector(&Vector::new(-100.0, 10.0)),
            Some(SwipeDirection::Left)
        );
        assert_eq!(
            SwipeDirection::from_vector(&Vector::new(10.0, 100.0)),
            Some(SwipeDirection::Down)
        );
        assert_eq!(
            SwipeDirection::from_vector(&Vector::new(10.0, -100.0)),
            Some(SwipeDirection::Up)
        );
    }

    #[test]
    fn test_gesture_event_builder() {
        let event = GestureEvent::new(GestureType::Pan, GestureState::Changed, Point::new(100.0, 200.0))
            .with_delta(Vector::new(10.0, 20.0))
            .with_velocity(Velocity::new(500.0, 1000.0))
            .with_pointer_count(2);

        assert_eq!(event.gesture_type, GestureType::Pan);
        assert_eq!(event.state, GestureState::Changed);
        assert_eq!(event.position.x, 100.0);
        assert!(event.delta.is_some());
        assert!(event.velocity.is_some());
        assert_eq!(event.pointer_count, 2);
    }

    #[test]
    fn test_pointer_event_creation() {
        let event = PointerEvent::touch_down(1, 100.0, 200.0);
        assert_eq!(event.pointer_id, 1);
        assert_eq!(event.position.x, 100.0);
        assert_eq!(event.position.y, 200.0);
        assert_eq!(event.phase, PointerPhase::Began);
    }

    #[test]
    fn test_gesture_recognizer_id_uniqueness() {
        let id1 = GestureRecognizerId::new();
        let id2 = GestureRecognizerId::new();
        let id3 = GestureRecognizerId::new();
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
}
