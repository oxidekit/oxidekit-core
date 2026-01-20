//! Multi-touch gesture recognizers
//!
//! This module provides recognizers for multi-touch gestures:
//! - Pinch (zoom in/out)
//! - Rotation (two-finger rotation)
//! - TwoFingerPan (scrolling with two fingers)

use crate::recognizer::{
    GestureEvent, GestureRecognizer, GestureRecognizerId, GestureState, GestureType, Point,
    PointerEvent, PointerPhase, Vector,
};
use crate::velocity::{Velocity, VelocityTracker};
use std::collections::HashMap;

/// Track state of a single pointer
#[derive(Debug, Clone)]
struct PointerState {
    position: Point,
    start_position: Point,
}

/// Configuration for pinch gesture recognition
#[derive(Debug, Clone)]
pub struct PinchConfig {
    /// Minimum scale change to start recognizing (e.g., 0.05 = 5% change)
    pub min_scale_delta: f32,
    /// Minimum distance between fingers to track
    pub min_span: f32,
}

impl Default for PinchConfig {
    fn default() -> Self {
        Self {
            min_scale_delta: 0.02,
            min_span: 10.0,
        }
    }
}

/// Recognizes pinch (zoom) gestures
#[derive(Debug)]
pub struct PinchGesture {
    id: GestureRecognizerId,
    config: PinchConfig,
    state: GestureState,
    pointers: HashMap<u64, PointerState>,
    initial_span: Option<f32>,
    current_scale: f32,
    last_scale: f32,
}

impl PinchGesture {
    /// Create a new pinch gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: PinchConfig::default(),
            state: GestureState::Possible,
            pointers: HashMap::new(),
            initial_span: None,
            current_scale: 1.0,
            last_scale: 1.0,
        }
    }

    /// Create with custom config
    pub fn with_config(config: PinchConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Get the current scale factor
    pub fn scale(&self) -> f32 {
        self.current_scale
    }

    /// Calculate the span (distance) between two pointers
    fn calculate_span(&self) -> Option<f32> {
        if self.pointers.len() != 2 {
            return None;
        }

        let positions: Vec<_> = self.pointers.values().map(|p| p.position).collect();
        Some(positions[0].distance_to(&positions[1]))
    }

    /// Calculate the center point between pointers
    fn calculate_center(&self) -> Option<Point> {
        if self.pointers.is_empty() {
            return None;
        }

        let count = self.pointers.len() as f32;
        let sum: (f32, f32) = self
            .pointers
            .values()
            .fold((0.0, 0.0), |acc, p| (acc.0 + p.position.x, acc.1 + p.position.y));

        Some(Point::new(sum.0 / count, sum.1 / count))
    }
}

impl Default for PinchGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for PinchGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::Pinch
    }

    fn state(&self) -> GestureState {
        self.state
    }

    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Began => {
                self.pointers.insert(
                    event.pointer_id,
                    PointerState {
                        position: event.position,
                        start_position: event.position,
                    },
                );

                // Start tracking when we have exactly 2 pointers
                if self.pointers.len() == 2 {
                    if let Some(span) = self.calculate_span() {
                        if span >= self.config.min_span {
                            self.initial_span = Some(span);
                            self.current_scale = 1.0;
                            self.last_scale = 1.0;
                        }
                    }
                }
                None
            }
            PointerPhase::Moved => {
                if let Some(pointer) = self.pointers.get_mut(&event.pointer_id) {
                    pointer.position = event.position;
                } else {
                    return None;
                }

                // Only track with exactly 2 pointers
                if self.pointers.len() != 2 {
                    return None;
                }

                if let (Some(initial), Some(current)) = (self.initial_span, self.calculate_span()) {
                    if initial < 0.001 {
                        return None;
                    }

                    let new_scale = current / initial;
                    let scale_delta = (new_scale - self.last_scale).abs();

                    let center = self.calculate_center().unwrap_or(Point::zero());

                    match self.state {
                        GestureState::Possible => {
                            if scale_delta >= self.config.min_scale_delta {
                                self.state = GestureState::Began;
                                self.current_scale = new_scale;
                                self.last_scale = new_scale;
                                return Some(
                                    GestureEvent::new(
                                        GestureType::Pinch,
                                        GestureState::Began,
                                        center,
                                    )
                                    .with_scale(new_scale)
                                    .with_pointer_count(2),
                                );
                            }
                        }
                        GestureState::Began | GestureState::Changed => {
                            self.state = GestureState::Changed;
                            self.current_scale = new_scale;
                            self.last_scale = new_scale;
                            return Some(
                                GestureEvent::new(
                                    GestureType::Pinch,
                                    GestureState::Changed,
                                    center,
                                )
                                .with_scale(new_scale)
                                .with_pointer_count(2),
                            );
                        }
                        _ => {}
                    }
                }
                None
            }
            PointerPhase::Ended | PointerPhase::Cancelled => {
                self.pointers.remove(&event.pointer_id);

                if self.state.is_active() {
                    let center = self.calculate_center().unwrap_or(event.position);
                    let state = if event.phase == PointerPhase::Cancelled {
                        GestureState::Cancelled
                    } else {
                        GestureState::Ended
                    };

                    let result = Some(
                        GestureEvent::new(GestureType::Pinch, state, center)
                            .with_scale(self.current_scale)
                            .with_pointer_count(self.pointers.len() + 1),
                    );

                    // Reset if we're down to one or zero pointers
                    if self.pointers.len() <= 1 {
                        self.initial_span = None;
                        self.current_scale = 1.0;
                        self.last_scale = 1.0;
                        self.state = GestureState::Possible;
                    }

                    return result;
                }

                // Reset tracking if needed
                if self.pointers.len() <= 1 {
                    self.initial_span = None;
                    self.current_scale = 1.0;
                    self.last_scale = 1.0;
                    self.state = GestureState::Possible;
                }
                None
            }
        }
    }

    fn reset(&mut self) {
        self.pointers.clear();
        self.initial_span = None;
        self.current_scale = 1.0;
        self.last_scale = 1.0;
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        25
    }

    fn should_recognize_simultaneously_with(&self, other: &dyn GestureRecognizer) -> bool {
        // Pinch can work with rotation
        matches!(other.gesture_type(), GestureType::Rotation | GestureType::TwoFingerPan)
    }
}

/// Configuration for rotation gesture recognition
#[derive(Debug, Clone)]
pub struct RotationConfig {
    /// Minimum rotation to start recognizing (in radians)
    pub min_rotation: f32,
    /// Minimum distance between fingers
    pub min_span: f32,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            min_rotation: 0.05, // ~3 degrees
            min_span: 10.0,
        }
    }
}

/// Recognizes rotation gestures (two-finger rotation)
#[derive(Debug)]
pub struct RotationGesture {
    id: GestureRecognizerId,
    config: RotationConfig,
    state: GestureState,
    pointers: HashMap<u64, PointerState>,
    initial_angle: Option<f32>,
    current_rotation: f32,
    last_rotation: f32,
}

impl RotationGesture {
    /// Create a new rotation gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: RotationConfig::default(),
            state: GestureState::Possible,
            pointers: HashMap::new(),
            initial_angle: None,
            current_rotation: 0.0,
            last_rotation: 0.0,
        }
    }

    /// Create with custom config
    pub fn with_config(config: RotationConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Get the current rotation in radians
    pub fn rotation(&self) -> f32 {
        self.current_rotation
    }

    /// Calculate the angle between two pointers
    fn calculate_angle(&self) -> Option<f32> {
        if self.pointers.len() != 2 {
            return None;
        }

        let positions: Vec<_> = self.pointers.values().map(|p| p.position).collect();
        let dx = positions[1].x - positions[0].x;
        let dy = positions[1].y - positions[0].y;
        Some(dy.atan2(dx))
    }

    /// Calculate the span between pointers
    fn calculate_span(&self) -> Option<f32> {
        if self.pointers.len() != 2 {
            return None;
        }

        let positions: Vec<_> = self.pointers.values().map(|p| p.position).collect();
        Some(positions[0].distance_to(&positions[1]))
    }

    /// Calculate center point
    fn calculate_center(&self) -> Option<Point> {
        if self.pointers.is_empty() {
            return None;
        }

        let count = self.pointers.len() as f32;
        let sum: (f32, f32) = self
            .pointers
            .values()
            .fold((0.0, 0.0), |acc, p| (acc.0 + p.position.x, acc.1 + p.position.y));

        Some(Point::new(sum.0 / count, sum.1 / count))
    }

    /// Normalize angle difference to [-PI, PI]
    fn normalize_angle(angle: f32) -> f32 {
        let mut a = angle;
        while a > std::f32::consts::PI {
            a -= 2.0 * std::f32::consts::PI;
        }
        while a < -std::f32::consts::PI {
            a += 2.0 * std::f32::consts::PI;
        }
        a
    }
}

impl Default for RotationGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for RotationGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::Rotation
    }

    fn state(&self) -> GestureState {
        self.state
    }

    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Began => {
                self.pointers.insert(
                    event.pointer_id,
                    PointerState {
                        position: event.position,
                        start_position: event.position,
                    },
                );

                if self.pointers.len() == 2 {
                    if let Some(span) = self.calculate_span() {
                        if span >= self.config.min_span {
                            self.initial_angle = self.calculate_angle();
                            self.current_rotation = 0.0;
                            self.last_rotation = 0.0;
                        }
                    }
                }
                None
            }
            PointerPhase::Moved => {
                if let Some(pointer) = self.pointers.get_mut(&event.pointer_id) {
                    pointer.position = event.position;
                } else {
                    return None;
                }

                if self.pointers.len() != 2 {
                    return None;
                }

                if let (Some(initial), Some(current)) = (self.initial_angle, self.calculate_angle())
                {
                    let rotation = Self::normalize_angle(current - initial);
                    let rotation_delta = (rotation - self.last_rotation).abs();
                    let center = self.calculate_center().unwrap_or(Point::zero());

                    match self.state {
                        GestureState::Possible => {
                            if rotation_delta >= self.config.min_rotation {
                                self.state = GestureState::Began;
                                self.current_rotation = rotation;
                                self.last_rotation = rotation;
                                return Some(
                                    GestureEvent::new(
                                        GestureType::Rotation,
                                        GestureState::Began,
                                        center,
                                    )
                                    .with_rotation(rotation)
                                    .with_pointer_count(2),
                                );
                            }
                        }
                        GestureState::Began | GestureState::Changed => {
                            self.state = GestureState::Changed;
                            self.current_rotation = rotation;
                            self.last_rotation = rotation;
                            return Some(
                                GestureEvent::new(
                                    GestureType::Rotation,
                                    GestureState::Changed,
                                    center,
                                )
                                .with_rotation(rotation)
                                .with_pointer_count(2),
                            );
                        }
                        _ => {}
                    }
                }
                None
            }
            PointerPhase::Ended | PointerPhase::Cancelled => {
                self.pointers.remove(&event.pointer_id);

                if self.state.is_active() {
                    let center = self.calculate_center().unwrap_or(event.position);
                    let state = if event.phase == PointerPhase::Cancelled {
                        GestureState::Cancelled
                    } else {
                        GestureState::Ended
                    };

                    let result = Some(
                        GestureEvent::new(GestureType::Rotation, state, center)
                            .with_rotation(self.current_rotation)
                            .with_pointer_count(self.pointers.len() + 1),
                    );

                    if self.pointers.len() <= 1 {
                        self.initial_angle = None;
                        self.current_rotation = 0.0;
                        self.last_rotation = 0.0;
                        self.state = GestureState::Possible;
                    }

                    return result;
                }

                if self.pointers.len() <= 1 {
                    self.initial_angle = None;
                    self.current_rotation = 0.0;
                    self.last_rotation = 0.0;
                    self.state = GestureState::Possible;
                }
                None
            }
        }
    }

    fn reset(&mut self) {
        self.pointers.clear();
        self.initial_angle = None;
        self.current_rotation = 0.0;
        self.last_rotation = 0.0;
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        25
    }

    fn should_recognize_simultaneously_with(&self, other: &dyn GestureRecognizer) -> bool {
        matches!(other.gesture_type(), GestureType::Pinch | GestureType::TwoFingerPan)
    }
}

/// Configuration for two-finger pan gesture
#[derive(Debug, Clone)]
pub struct TwoFingerPanConfig {
    /// Minimum movement to start panning
    pub min_distance: f32,
}

impl Default for TwoFingerPanConfig {
    fn default() -> Self {
        Self { min_distance: 10.0 }
    }
}

/// Recognizes two-finger pan gestures (scrolling)
#[derive(Debug)]
pub struct TwoFingerPanGesture {
    id: GestureRecognizerId,
    config: TwoFingerPanConfig,
    state: GestureState,
    pointers: HashMap<u64, PointerState>,
    start_center: Option<Point>,
    last_center: Option<Point>,
    velocity_tracker: VelocityTracker,
    total_delta: Vector,
}

impl TwoFingerPanGesture {
    /// Create a new two-finger pan gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: TwoFingerPanConfig::default(),
            state: GestureState::Possible,
            pointers: HashMap::new(),
            start_center: None,
            last_center: None,
            velocity_tracker: VelocityTracker::new(),
            total_delta: Vector::zero(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: TwoFingerPanConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Calculate center point of all pointers
    fn calculate_center(&self) -> Option<Point> {
        if self.pointers.len() < 2 {
            return None;
        }

        let count = self.pointers.len() as f32;
        let sum: (f32, f32) = self
            .pointers
            .values()
            .fold((0.0, 0.0), |acc, p| (acc.0 + p.position.x, acc.1 + p.position.y));

        Some(Point::new(sum.0 / count, sum.1 / count))
    }
}

impl Default for TwoFingerPanGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for TwoFingerPanGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::TwoFingerPan
    }

    fn state(&self) -> GestureState {
        self.state
    }

    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Began => {
                self.pointers.insert(
                    event.pointer_id,
                    PointerState {
                        position: event.position,
                        start_position: event.position,
                    },
                );

                if self.pointers.len() == 2 {
                    let center = self.calculate_center();
                    self.start_center = center;
                    self.last_center = center;
                    self.velocity_tracker.reset();
                    if let Some(c) = center {
                        self.velocity_tracker
                            .add_position(c.x, c.y, event.timestamp);
                    }
                    self.total_delta = Vector::zero();
                }
                None
            }
            PointerPhase::Moved => {
                if let Some(pointer) = self.pointers.get_mut(&event.pointer_id) {
                    pointer.position = event.position;
                } else {
                    return None;
                }

                if self.pointers.len() != 2 {
                    return None;
                }

                if let Some(current_center) = self.calculate_center() {
                    self.velocity_tracker
                        .add_position(current_center.x, current_center.y, event.timestamp);

                    if let Some(last) = self.last_center {
                        let delta = Vector::from_points(&last, &current_center);
                        self.last_center = Some(current_center);

                        match self.state {
                            GestureState::Possible => {
                                if let Some(start) = self.start_center {
                                    let total_distance = start.distance_to(&current_center);
                                    if total_distance >= self.config.min_distance {
                                        self.state = GestureState::Began;
                                        self.total_delta =
                                            Vector::from_points(&start, &current_center);
                                        let (vx, vy) = self.velocity_tracker.calculate_velocity();
                                        return Some(
                                            GestureEvent::new(
                                                GestureType::TwoFingerPan,
                                                GestureState::Began,
                                                current_center,
                                            )
                                            .with_delta(self.total_delta)
                                            .with_velocity(Velocity::new(vx, vy))
                                            .with_pointer_count(2),
                                        );
                                    }
                                }
                            }
                            GestureState::Began | GestureState::Changed => {
                                self.state = GestureState::Changed;
                                self.total_delta.dx += delta.dx;
                                self.total_delta.dy += delta.dy;
                                let (vx, vy) = self.velocity_tracker.calculate_velocity();
                                return Some(
                                    GestureEvent::new(
                                        GestureType::TwoFingerPan,
                                        GestureState::Changed,
                                        current_center,
                                    )
                                    .with_delta(delta)
                                    .with_velocity(Velocity::new(vx, vy))
                                    .with_pointer_count(2),
                                );
                            }
                            _ => {}
                        }
                    }
                }
                None
            }
            PointerPhase::Ended | PointerPhase::Cancelled => {
                self.pointers.remove(&event.pointer_id);

                if self.state.is_active() {
                    let center = self.last_center.unwrap_or(event.position);
                    let state = if event.phase == PointerPhase::Cancelled {
                        GestureState::Cancelled
                    } else {
                        GestureState::Ended
                    };

                    let (vx, vy) = self.velocity_tracker.calculate_velocity();
                    let result = Some(
                        GestureEvent::new(GestureType::TwoFingerPan, state, center)
                            .with_delta(Vector::zero())
                            .with_velocity(Velocity::new(vx, vy))
                            .with_pointer_count(self.pointers.len() + 1),
                    );

                    if self.pointers.len() <= 1 {
                        self.start_center = None;
                        self.last_center = None;
                        self.total_delta = Vector::zero();
                        self.state = GestureState::Possible;
                    }

                    return result;
                }

                if self.pointers.len() <= 1 {
                    self.start_center = None;
                    self.last_center = None;
                    self.total_delta = Vector::zero();
                    self.state = GestureState::Possible;
                }
                None
            }
        }
    }

    fn reset(&mut self) {
        self.pointers.clear();
        self.start_center = None;
        self.last_center = None;
        self.velocity_tracker.reset();
        self.total_delta = Vector::zero();
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        20
    }

    fn should_recognize_simultaneously_with(&self, other: &dyn GestureRecognizer) -> bool {
        matches!(other.gesture_type(), GestureType::Pinch | GestureType::Rotation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_down(id: u64, x: f32, y: f32) -> PointerEvent {
        PointerEvent::touch_down(id, x, y)
    }

    fn make_move(id: u64, x: f32, y: f32) -> PointerEvent {
        PointerEvent::touch_move(id, x, y)
    }

    fn make_up(id: u64, x: f32, y: f32) -> PointerEvent {
        PointerEvent::touch_up(id, x, y)
    }

    #[test]
    fn test_pinch_gesture_zoom_in() {
        let mut pinch = PinchGesture::new();

        // Two fingers down
        let down1 = make_down(1, 100.0, 100.0);
        let down2 = make_down(2, 200.0, 100.0);
        pinch.handle_event(&down1);
        pinch.handle_event(&down2);

        // Move fingers apart (zoom in)
        let move1 = make_move(1, 50.0, 100.0);
        let move2 = make_move(2, 250.0, 100.0);
        pinch.handle_event(&move1);
        let result = pinch.handle_event(&move2);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.gesture_type, GestureType::Pinch);
        assert!(event.scale.unwrap() > 1.0); // Scale > 1 means zoom in
    }

    #[test]
    fn test_pinch_gesture_zoom_out() {
        let mut pinch = PinchGesture::new();

        // Two fingers down, far apart
        let down1 = make_down(1, 50.0, 100.0);
        let down2 = make_down(2, 250.0, 100.0);
        pinch.handle_event(&down1);
        pinch.handle_event(&down2);

        // Move fingers closer (zoom out)
        let move1 = make_move(1, 100.0, 100.0);
        let move2 = make_move(2, 200.0, 100.0);
        pinch.handle_event(&move1);
        let result = pinch.handle_event(&move2);

        assert!(result.is_some());
        let event = result.unwrap();
        assert!(event.scale.unwrap() < 1.0); // Scale < 1 means zoom out
    }

    #[test]
    fn test_pinch_needs_two_pointers() {
        let mut pinch = PinchGesture::new();

        // Only one finger
        let down1 = make_down(1, 100.0, 100.0);
        pinch.handle_event(&down1);

        let move1 = make_move(1, 150.0, 100.0);
        let result = pinch.handle_event(&move1);

        assert!(result.is_none());
    }

    #[test]
    fn test_pinch_gesture_ends() {
        let mut pinch = PinchGesture::new();

        let down1 = make_down(1, 100.0, 100.0);
        let down2 = make_down(2, 200.0, 100.0);
        pinch.handle_event(&down1);
        pinch.handle_event(&down2);

        // Start pinch
        let move1 = make_move(1, 50.0, 100.0);
        let move2 = make_move(2, 250.0, 100.0);
        pinch.handle_event(&move1);
        pinch.handle_event(&move2);

        // Lift one finger
        let up1 = make_up(1, 50.0, 100.0);
        let result = pinch.handle_event(&up1);

        assert!(result.is_some());
        assert_eq!(result.unwrap().state, GestureState::Ended);
    }

    #[test]
    fn test_rotation_gesture_clockwise() {
        let mut rotation = RotationGesture::new();

        // Two fingers in horizontal line
        let down1 = make_down(1, 100.0, 100.0);
        let down2 = make_down(2, 200.0, 100.0);
        rotation.handle_event(&down1);
        rotation.handle_event(&down2);

        // Rotate clockwise (finger 2 moves down)
        let move2 = make_move(2, 200.0, 150.0);
        let result = rotation.handle_event(&move2);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.gesture_type, GestureType::Rotation);
        assert!(event.rotation.is_some());
    }

    #[test]
    fn test_rotation_gesture_counterclockwise() {
        let mut rotation = RotationGesture::new();

        // Two fingers in horizontal line
        let down1 = make_down(1, 100.0, 100.0);
        let down2 = make_down(2, 200.0, 100.0);
        rotation.handle_event(&down1);
        rotation.handle_event(&down2);

        // Rotate counterclockwise (finger 2 moves up)
        let move2 = make_move(2, 200.0, 50.0);
        let result = rotation.handle_event(&move2);

        assert!(result.is_some());
        let event = result.unwrap();
        assert!(event.rotation.unwrap() < 0.0); // Negative = counterclockwise
    }

    #[test]
    fn test_two_finger_pan() {
        let mut pan = TwoFingerPanGesture::new();

        // Two fingers down
        let down1 = make_down(1, 100.0, 100.0);
        let down2 = make_down(2, 150.0, 100.0);
        pan.handle_event(&down1);
        pan.handle_event(&down2);

        // Both fingers move together
        let move1 = make_move(1, 100.0, 120.0);
        let move2 = make_move(2, 150.0, 120.0);
        pan.handle_event(&move1);
        let result = pan.handle_event(&move2);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.gesture_type, GestureType::TwoFingerPan);
        assert!(event.delta.is_some());
    }

    #[test]
    fn test_two_finger_pan_needs_two_pointers() {
        let mut pan = TwoFingerPanGesture::new();

        let down1 = make_down(1, 100.0, 100.0);
        pan.handle_event(&down1);

        let move1 = make_move(1, 100.0, 150.0);
        let result = pan.handle_event(&move1);

        assert!(result.is_none());
    }

    #[test]
    fn test_pinch_center_calculation() {
        let mut pinch = PinchGesture::new();

        let down1 = make_down(1, 0.0, 0.0);
        let down2 = make_down(2, 100.0, 100.0);
        pinch.handle_event(&down1);
        pinch.handle_event(&down2);

        let center = pinch.calculate_center();
        assert!(center.is_some());
        let c = center.unwrap();
        assert!((c.x - 50.0).abs() < 0.001);
        assert!((c.y - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_gesture_reset() {
        let mut pinch = PinchGesture::new();

        let down1 = make_down(1, 100.0, 100.0);
        let down2 = make_down(2, 200.0, 100.0);
        pinch.handle_event(&down1);
        pinch.handle_event(&down2);

        pinch.reset();

        assert!(pinch.pointers.is_empty());
        assert_eq!(pinch.state(), GestureState::Possible);
        assert_eq!(pinch.scale(), 1.0);
    }

    #[test]
    fn test_pinch_simultaneous_with_rotation() {
        let pinch = PinchGesture::new();
        let rotation = RotationGesture::new();

        assert!(pinch.should_recognize_simultaneously_with(&rotation));
        assert!(rotation.should_recognize_simultaneously_with(&pinch));
    }

    #[test]
    fn test_two_finger_pan_velocity() {
        let mut pan = TwoFingerPanGesture::new();

        let down1 = make_down(1, 100.0, 100.0);
        let down2 = make_down(2, 150.0, 100.0);
        pan.handle_event(&down1);
        pan.handle_event(&down2);

        // Move multiple times
        for i in 1..5 {
            let move1 = make_move(1, 100.0, 100.0 + i as f32 * 20.0);
            let move2 = make_move(2, 150.0, 100.0 + i as f32 * 20.0);
            pan.handle_event(&move1);
            pan.handle_event(&move2);
        }

        let up1 = make_up(1, 100.0, 180.0);
        let result = pan.handle_event(&up1);

        assert!(result.is_some());
        let event = result.unwrap();
        assert!(event.velocity.is_some());
    }
}
