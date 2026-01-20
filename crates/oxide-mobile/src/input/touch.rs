//! Touch event handling.
//!
//! Provides low-level touch input primitives for tracking individual
//! touch points across their lifecycle.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Unique identifier for a touch point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TouchId(pub u64);

impl TouchId {
    /// Create a new touch ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TouchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Touch({})", self.0)
    }
}

/// Phase of a touch event in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TouchPhase {
    /// Touch has just begun.
    Began,
    /// Touch has moved.
    Moved,
    /// Touch was held stationary (iOS specific).
    Stationary,
    /// Touch has ended normally.
    Ended,
    /// Touch was cancelled (e.g., by system gesture).
    Cancelled,
}

impl TouchPhase {
    /// Returns true if this is an active touch phase.
    pub fn is_active(&self) -> bool {
        matches!(self, TouchPhase::Began | TouchPhase::Moved | TouchPhase::Stationary)
    }

    /// Returns true if this is a terminal phase.
    pub fn is_terminal(&self) -> bool {
        matches!(self, TouchPhase::Ended | TouchPhase::Cancelled)
    }

    /// Returns true if the touch is moving.
    pub fn is_moving(&self) -> bool {
        matches!(self, TouchPhase::Moved)
    }
}

impl std::fmt::Display for TouchPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            TouchPhase::Began => "Began",
            TouchPhase::Moved => "Moved",
            TouchPhase::Stationary => "Stationary",
            TouchPhase::Ended => "Ended",
            TouchPhase::Cancelled => "Cancelled",
        };
        write!(f, "{}", name)
    }
}

/// A single touch point with its current state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TouchPoint {
    /// Unique identifier for this touch.
    pub id: TouchId,
    /// Current position in screen coordinates.
    pub position: (f32, f32),
    /// Previous position (for calculating delta).
    pub previous_position: (f32, f32),
    /// Touch pressure (0.0-1.0, if supported).
    pub pressure: f32,
    /// Touch radius in points (if supported).
    pub radius: Option<f32>,
    /// Touch altitude angle in radians (Apple Pencil).
    pub altitude_angle: Option<f32>,
    /// Touch azimuth angle in radians (Apple Pencil).
    pub azimuth_angle: Option<f32>,
}

impl TouchPoint {
    /// Create a new touch point.
    pub fn new(id: TouchId, position: (f32, f32)) -> Self {
        Self {
            id,
            position,
            previous_position: position,
            pressure: 1.0,
            radius: None,
            altitude_angle: None,
            azimuth_angle: None,
        }
    }

    /// Get the x coordinate.
    pub fn x(&self) -> f32 {
        self.position.0
    }

    /// Get the y coordinate.
    pub fn y(&self) -> f32 {
        self.position.1
    }

    /// Get the movement delta since last update.
    pub fn delta(&self) -> (f32, f32) {
        (
            self.position.0 - self.previous_position.0,
            self.position.1 - self.previous_position.1,
        )
    }

    /// Get the movement distance since last update.
    pub fn delta_distance(&self) -> f32 {
        let (dx, dy) = self.delta();
        (dx * dx + dy * dy).sqrt()
    }

    /// Update position and store previous.
    pub fn update_position(&mut self, new_position: (f32, f32)) {
        self.previous_position = self.position;
        self.position = new_position;
    }

    /// Set pressure value.
    pub fn with_pressure(mut self, pressure: f32) -> Self {
        self.pressure = pressure.clamp(0.0, 1.0);
        self
    }

    /// Set radius value.
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }
}

/// A complete touch event with all active touches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchEvent {
    /// The touch that triggered this event.
    pub touch: TouchPoint,
    /// Current phase of the triggering touch.
    pub phase: TouchPhase,
    /// Timestamp of the event.
    #[serde(skip, default = "default_timestamp")]
    pub timestamp: Instant,
    /// All currently active touches.
    pub all_touches: Vec<TouchPoint>,
    /// Number of taps (for tap detection).
    pub tap_count: u32,
}

fn default_timestamp() -> Instant {
    Instant::now()
}

impl TouchEvent {
    /// Create a new touch event.
    pub fn new(touch: TouchPoint, phase: TouchPhase) -> Self {
        Self {
            touch,
            phase,
            timestamp: Instant::now(),
            all_touches: vec![touch],
            tap_count: 1,
        }
    }

    /// Create a touch event with multiple touches.
    pub fn with_all_touches(mut self, touches: Vec<TouchPoint>) -> Self {
        self.all_touches = touches;
        self
    }

    /// Set the tap count.
    pub fn with_tap_count(mut self, count: u32) -> Self {
        self.tap_count = count;
        self
    }

    /// Get the position of the primary touch.
    pub fn position(&self) -> (f32, f32) {
        self.touch.position
    }

    /// Get the number of active touches.
    pub fn touch_count(&self) -> usize {
        self.all_touches.len()
    }

    /// Check if this is a multi-touch event.
    pub fn is_multi_touch(&self) -> bool {
        self.all_touches.len() > 1
    }

    /// Get center point of all touches.
    pub fn center(&self) -> (f32, f32) {
        if self.all_touches.is_empty() {
            return self.touch.position;
        }

        let sum: (f32, f32) = self
            .all_touches
            .iter()
            .fold((0.0, 0.0), |acc, t| (acc.0 + t.x(), acc.1 + t.y()));

        let count = self.all_touches.len() as f32;
        (sum.0 / count, sum.1 / count)
    }

    /// Get the span (distance) between two touches.
    ///
    /// Returns None if there are fewer than 2 touches.
    pub fn span(&self) -> Option<f32> {
        if self.all_touches.len() < 2 {
            return None;
        }

        let t1 = &self.all_touches[0];
        let t2 = &self.all_touches[1];

        let dx = t2.x() - t1.x();
        let dy = t2.y() - t1.y();

        Some((dx * dx + dy * dy).sqrt())
    }

    /// Get the angle between two touches.
    ///
    /// Returns None if there are fewer than 2 touches.
    pub fn angle(&self) -> Option<f32> {
        if self.all_touches.len() < 2 {
            return None;
        }

        let t1 = &self.all_touches[0];
        let t2 = &self.all_touches[1];

        let dx = t2.x() - t1.x();
        let dy = t2.y() - t1.y();

        Some(dy.atan2(dx))
    }
}

/// Tracks active touches across events.
#[derive(Debug, Default)]
pub struct TouchTracker {
    /// Currently active touches by ID.
    active_touches: HashMap<TouchId, TrackedTouch>,
    /// Next touch ID to assign.
    next_id: u64,
}

/// Internal touch tracking state.
#[derive(Debug, Clone)]
struct TrackedTouch {
    point: TouchPoint,
    start_position: (f32, f32),
    start_time: Instant,
    last_update: Instant,
}

impl TouchTracker {
    /// Create a new touch tracker.
    pub fn new() -> Self {
        Self {
            active_touches: HashMap::new(),
            next_id: 1,
        }
    }

    /// Process a touch event and update tracking state.
    pub fn process(&mut self, event: &TouchEvent) {
        match event.phase {
            TouchPhase::Began => {
                let tracked = TrackedTouch {
                    point: event.touch,
                    start_position: event.touch.position,
                    start_time: event.timestamp,
                    last_update: event.timestamp,
                };
                self.active_touches.insert(event.touch.id, tracked);
            }
            TouchPhase::Moved | TouchPhase::Stationary => {
                if let Some(tracked) = self.active_touches.get_mut(&event.touch.id) {
                    tracked.point = event.touch;
                    tracked.last_update = event.timestamp;
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.active_touches.remove(&event.touch.id);
            }
        }
    }

    /// Get the number of active touches.
    pub fn active_count(&self) -> usize {
        self.active_touches.len()
    }

    /// Get all active touch points.
    pub fn active_touches(&self) -> Vec<TouchPoint> {
        self.active_touches.values().map(|t| t.point).collect()
    }

    /// Get a specific touch by ID.
    pub fn get_touch(&self, id: TouchId) -> Option<&TouchPoint> {
        self.active_touches.get(&id).map(|t| &t.point)
    }

    /// Get the start position of a touch.
    pub fn start_position(&self, id: TouchId) -> Option<(f32, f32)> {
        self.active_touches.get(&id).map(|t| t.start_position)
    }

    /// Get how long a touch has been active.
    pub fn touch_duration(&self, id: TouchId) -> Option<Duration> {
        self.active_touches
            .get(&id)
            .map(|t| t.last_update.duration_since(t.start_time))
    }

    /// Get the total distance a touch has traveled.
    pub fn touch_distance(&self, id: TouchId) -> Option<f32> {
        self.active_touches.get(&id).map(|t| {
            let dx = t.point.x() - t.start_position.0;
            let dy = t.point.y() - t.start_position.1;
            (dx * dx + dy * dy).sqrt()
        })
    }

    /// Generate a new touch ID.
    pub fn next_touch_id(&mut self) -> TouchId {
        let id = TouchId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Clear all tracked touches.
    pub fn clear(&mut self) {
        self.active_touches.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_id() {
        let id = TouchId::new(42);
        assert_eq!(id.raw(), 42);
        assert_eq!(format!("{}", id), "Touch(42)");
    }

    #[test]
    fn test_touch_phase() {
        assert!(TouchPhase::Began.is_active());
        assert!(TouchPhase::Moved.is_active());
        assert!(!TouchPhase::Ended.is_active());

        assert!(TouchPhase::Ended.is_terminal());
        assert!(TouchPhase::Cancelled.is_terminal());
        assert!(!TouchPhase::Began.is_terminal());

        assert!(TouchPhase::Moved.is_moving());
        assert!(!TouchPhase::Stationary.is_moving());
    }

    #[test]
    fn test_touch_point_delta() {
        let mut point = TouchPoint::new(TouchId::new(1), (100.0, 100.0));
        point.update_position((110.0, 105.0));

        let (dx, dy) = point.delta();
        assert!((dx - 10.0).abs() < 0.001);
        assert!((dy - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_touch_event_center() {
        let t1 = TouchPoint::new(TouchId::new(1), (0.0, 0.0));
        let t2 = TouchPoint::new(TouchId::new(2), (100.0, 100.0));

        let event = TouchEvent::new(t1, TouchPhase::Began).with_all_touches(vec![t1, t2]);

        let (cx, cy) = event.center();
        assert!((cx - 50.0).abs() < 0.001);
        assert!((cy - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_touch_event_span() {
        let t1 = TouchPoint::new(TouchId::new(1), (0.0, 0.0));
        let t2 = TouchPoint::new(TouchId::new(2), (100.0, 0.0));

        let event = TouchEvent::new(t1, TouchPhase::Began).with_all_touches(vec![t1, t2]);

        let span = event.span().unwrap();
        assert!((span - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_touch_tracker() {
        let mut tracker = TouchTracker::new();

        // Start a touch
        let id = tracker.next_touch_id();
        let point = TouchPoint::new(id, (100.0, 100.0));
        let event = TouchEvent::new(point, TouchPhase::Began);
        tracker.process(&event);

        assert_eq!(tracker.active_count(), 1);
        assert!(tracker.get_touch(id).is_some());
        assert_eq!(tracker.start_position(id), Some((100.0, 100.0)));

        // Move the touch
        let mut moved_point = point;
        moved_point.update_position((150.0, 150.0));
        let move_event = TouchEvent::new(moved_point, TouchPhase::Moved);
        tracker.process(&move_event);

        let distance = tracker.touch_distance(id).unwrap();
        assert!(distance > 70.0); // sqrt(50^2 + 50^2) ~= 70.7

        // End the touch
        let end_event = TouchEvent::new(moved_point, TouchPhase::Ended);
        tracker.process(&end_event);

        assert_eq!(tracker.active_count(), 0);
        assert!(tracker.get_touch(id).is_none());
    }
}
