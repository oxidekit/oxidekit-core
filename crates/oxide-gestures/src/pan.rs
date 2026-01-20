//! Pan and drag gesture recognizers
//!
//! This module provides recognizers for continuous movement gestures:
//! - Pan (2D dragging)
//! - Drag (with start/update/end lifecycle)
//! - Swipe (directional quick movements)
//! - Fling (swipe with velocity for momentum)

use crate::recognizer::{
    GestureEvent, GestureRecognizer, GestureRecognizerId, GestureState, GestureType, Point,
    PointerEvent, PointerPhase, SwipeDirection, Vector,
};
use crate::velocity::{Velocity, VelocityTracker};
use std::time::{Duration, Instant};

/// Configuration for pan gesture recognition
#[derive(Debug, Clone)]
pub struct PanConfig {
    /// Minimum movement to start panning (in pixels)
    pub min_distance: f32,
    /// Maximum number of pointers for this gesture
    pub max_pointers: usize,
    /// Minimum number of pointers required
    pub min_pointers: usize,
}

impl Default for PanConfig {
    fn default() -> Self {
        Self {
            min_distance: 10.0,
            max_pointers: 1,
            min_pointers: 1,
        }
    }
}

/// Recognizes pan (2D drag) gestures
#[derive(Debug)]
pub struct PanGesture {
    id: GestureRecognizerId,
    config: PanConfig,
    state: GestureState,
    start_position: Option<Point>,
    last_position: Option<Point>,
    pointer_id: Option<u64>,
    velocity_tracker: VelocityTracker,
    total_delta: Vector,
}

impl PanGesture {
    /// Create a new pan gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: PanConfig::default(),
            state: GestureState::Possible,
            start_position: None,
            last_position: None,
            pointer_id: None,
            velocity_tracker: VelocityTracker::new(),
            total_delta: Vector::zero(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: PanConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set minimum distance to start panning
    pub fn min_distance(mut self, distance: f32) -> Self {
        self.config.min_distance = distance;
        self
    }
}

impl Default for PanGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for PanGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::Pan
    }

    fn state(&self) -> GestureState {
        self.state
    }

    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Began => {
                if self.pointer_id.is_none() {
                    self.pointer_id = Some(event.pointer_id);
                    self.start_position = Some(event.position);
                    self.last_position = Some(event.position);
                    self.velocity_tracker.reset();
                    self.velocity_tracker
                        .add_position(event.position.x, event.position.y, event.timestamp);
                    self.total_delta = Vector::zero();
                    self.state = GestureState::Possible;
                }
                None
            }
            PointerPhase::Moved => {
                if self.pointer_id != Some(event.pointer_id) {
                    return None;
                }

                self.velocity_tracker
                    .add_position(event.position.x, event.position.y, event.timestamp);

                let last = self.last_position.unwrap_or(event.position);
                let delta = Vector::from_points(&last, &event.position);
                self.last_position = Some(event.position);

                match self.state {
                    GestureState::Possible => {
                        if let Some(start) = self.start_position {
                            let total_distance = start.distance_to(&event.position);
                            if total_distance >= self.config.min_distance {
                                self.state = GestureState::Began;
                                self.total_delta = Vector::from_points(&start, &event.position);
                                let (vx, vy) = self.velocity_tracker.calculate_velocity();
                                return Some(
                                    GestureEvent::new(
                                        GestureType::Pan,
                                        GestureState::Began,
                                        event.position,
                                    )
                                    .with_delta(self.total_delta)
                                    .with_velocity(Velocity::new(vx, vy)),
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
                                GestureType::Pan,
                                GestureState::Changed,
                                event.position,
                            )
                            .with_delta(delta)
                            .with_velocity(Velocity::new(vx, vy)),
                        );
                    }
                    _ => {}
                }
                None
            }
            PointerPhase::Ended => {
                if self.pointer_id != Some(event.pointer_id) {
                    return None;
                }

                let was_active = self.state.is_active();
                self.state = if was_active {
                    GestureState::Ended
                } else {
                    GestureState::Failed
                };

                if was_active {
                    let (vx, vy) = self.velocity_tracker.calculate_velocity();
                    let result = Some(
                        GestureEvent::new(GestureType::Pan, GestureState::Ended, event.position)
                            .with_delta(Vector::zero())
                            .with_velocity(Velocity::new(vx, vy)),
                    );
                    self.reset();
                    return result;
                }
                self.reset();
                None
            }
            PointerPhase::Cancelled => {
                if self.pointer_id == Some(event.pointer_id) {
                    let was_active = self.state.is_active();
                    self.state = GestureState::Cancelled;
                    if was_active {
                        let result = Some(GestureEvent::new(
                            GestureType::Pan,
                            GestureState::Cancelled,
                            event.position,
                        ));
                        self.reset();
                        return result;
                    }
                    self.reset();
                }
                None
            }
        }
    }

    fn reset(&mut self) {
        self.start_position = None;
        self.last_position = None;
        self.pointer_id = None;
        self.velocity_tracker.reset();
        self.total_delta = Vector::zero();
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        5
    }

    fn requires_exclusive(&self) -> bool {
        false // Pan can work with other gestures
    }
}

/// Configuration for drag gesture recognition
#[derive(Debug, Clone)]
pub struct DragConfig {
    /// Minimum movement to start dragging
    pub min_distance: f32,
}

impl Default for DragConfig {
    fn default() -> Self {
        Self { min_distance: 5.0 }
    }
}

/// Recognizes drag gestures with clear lifecycle callbacks
#[derive(Debug)]
pub struct DragGesture {
    id: GestureRecognizerId,
    config: DragConfig,
    state: GestureState,
    start_position: Option<Point>,
    last_position: Option<Point>,
    pointer_id: Option<u64>,
    velocity_tracker: VelocityTracker,
}

impl DragGesture {
    /// Create a new drag gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: DragConfig::default(),
            state: GestureState::Possible,
            start_position: None,
            last_position: None,
            pointer_id: None,
            velocity_tracker: VelocityTracker::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: DragConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Get the start position of the drag
    pub fn start_position(&self) -> Option<Point> {
        self.start_position
    }

    /// Get the current/last position
    pub fn current_position(&self) -> Option<Point> {
        self.last_position
    }
}

impl Default for DragGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for DragGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::Drag
    }

    fn state(&self) -> GestureState {
        self.state
    }

    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Began => {
                if self.pointer_id.is_none() {
                    self.pointer_id = Some(event.pointer_id);
                    self.start_position = Some(event.position);
                    self.last_position = Some(event.position);
                    self.velocity_tracker.reset();
                    self.velocity_tracker
                        .add_position(event.position.x, event.position.y, event.timestamp);
                    self.state = GestureState::Possible;
                }
                None
            }
            PointerPhase::Moved => {
                if self.pointer_id != Some(event.pointer_id) {
                    return None;
                }

                self.velocity_tracker
                    .add_position(event.position.x, event.position.y, event.timestamp);

                let last = self.last_position.unwrap_or(event.position);
                let delta = Vector::from_points(&last, &event.position);
                self.last_position = Some(event.position);

                match self.state {
                    GestureState::Possible => {
                        if let Some(start) = self.start_position {
                            if start.distance_to(&event.position) >= self.config.min_distance {
                                self.state = GestureState::Began;
                                let total_delta = Vector::from_points(&start, &event.position);
                                let (vx, vy) = self.velocity_tracker.calculate_velocity();
                                return Some(
                                    GestureEvent::new(
                                        GestureType::Drag,
                                        GestureState::Began,
                                        event.position,
                                    )
                                    .with_delta(total_delta)
                                    .with_velocity(Velocity::new(vx, vy)),
                                );
                            }
                        }
                    }
                    GestureState::Began | GestureState::Changed => {
                        self.state = GestureState::Changed;
                        let (vx, vy) = self.velocity_tracker.calculate_velocity();
                        return Some(
                            GestureEvent::new(
                                GestureType::Drag,
                                GestureState::Changed,
                                event.position,
                            )
                            .with_delta(delta)
                            .with_velocity(Velocity::new(vx, vy)),
                        );
                    }
                    _ => {}
                }
                None
            }
            PointerPhase::Ended => {
                if self.pointer_id != Some(event.pointer_id) {
                    return None;
                }

                let was_active = self.state.is_active();
                self.state = if was_active {
                    GestureState::Ended
                } else {
                    GestureState::Failed
                };

                if was_active {
                    let (vx, vy) = self.velocity_tracker.calculate_velocity();
                    let result = Some(
                        GestureEvent::new(GestureType::Drag, GestureState::Ended, event.position)
                            .with_velocity(Velocity::new(vx, vy)),
                    );
                    self.reset();
                    return result;
                }
                self.reset();
                None
            }
            PointerPhase::Cancelled => {
                if self.pointer_id == Some(event.pointer_id) {
                    let was_active = self.state.is_active();
                    self.state = GestureState::Cancelled;
                    if was_active {
                        let result = Some(GestureEvent::new(
                            GestureType::Drag,
                            GestureState::Cancelled,
                            event.position,
                        ));
                        self.reset();
                        return result;
                    }
                    self.reset();
                }
                None
            }
        }
    }

    fn reset(&mut self) {
        self.start_position = None;
        self.last_position = None;
        self.pointer_id = None;
        self.velocity_tracker.reset();
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        5
    }
}

/// Configuration for swipe gesture recognition
#[derive(Debug, Clone)]
pub struct SwipeConfig {
    /// Minimum distance for a swipe
    pub min_distance: f32,
    /// Maximum duration for a swipe (milliseconds)
    pub max_duration_ms: u64,
    /// Minimum velocity for a swipe (pixels per second)
    pub min_velocity: f32,
    /// Allowed directions (None = all directions)
    pub allowed_directions: Option<Vec<SwipeDirection>>,
    /// Directional tolerance (0.0 to 1.0, how diagonal the swipe can be)
    pub directional_tolerance: f32,
}

impl Default for SwipeConfig {
    fn default() -> Self {
        Self {
            min_distance: 50.0,
            max_duration_ms: 500,
            min_velocity: 300.0,
            allowed_directions: None,
            directional_tolerance: 0.5, // Allow somewhat diagonal swipes
        }
    }
}

/// Recognizes swipe gestures in cardinal directions
#[derive(Debug)]
pub struct SwipeGesture {
    id: GestureRecognizerId,
    config: SwipeConfig,
    state: GestureState,
    start_position: Option<Point>,
    start_time: Option<Instant>,
    pointer_id: Option<u64>,
    velocity_tracker: VelocityTracker,
}

impl SwipeGesture {
    /// Create a new swipe gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: SwipeConfig::default(),
            state: GestureState::Possible,
            start_position: None,
            start_time: None,
            pointer_id: None,
            velocity_tracker: VelocityTracker::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: SwipeConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Only allow horizontal swipes
    pub fn horizontal(mut self) -> Self {
        self.config.allowed_directions =
            Some(vec![SwipeDirection::Left, SwipeDirection::Right]);
        self
    }

    /// Only allow vertical swipes
    pub fn vertical(mut self) -> Self {
        self.config.allowed_directions = Some(vec![SwipeDirection::Up, SwipeDirection::Down]);
        self
    }

    /// Set minimum distance
    pub fn min_distance(mut self, distance: f32) -> Self {
        self.config.min_distance = distance;
        self
    }

    /// Set minimum velocity
    pub fn min_velocity(mut self, velocity: f32) -> Self {
        self.config.min_velocity = velocity;
        self
    }

    fn is_direction_allowed(&self, direction: SwipeDirection) -> bool {
        match &self.config.allowed_directions {
            Some(allowed) => allowed.contains(&direction),
            None => true,
        }
    }

    fn check_directional_validity(&self, delta: &Vector) -> bool {
        let abs_dx = delta.dx.abs();
        let abs_dy = delta.dy.abs();
        let total = abs_dx + abs_dy;

        if total < 0.001 {
            return false;
        }

        // Check if movement is mostly in one direction
        let max_component = abs_dx.max(abs_dy);
        let ratio = max_component / total;

        ratio >= self.config.directional_tolerance
    }
}

impl Default for SwipeGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for SwipeGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::Swipe
    }

    fn state(&self) -> GestureState {
        self.state
    }

    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Began => {
                if self.pointer_id.is_none() {
                    self.pointer_id = Some(event.pointer_id);
                    self.start_position = Some(event.position);
                    self.start_time = Some(event.timestamp);
                    self.velocity_tracker.reset();
                    self.velocity_tracker
                        .add_position(event.position.x, event.position.y, event.timestamp);
                    self.state = GestureState::Possible;
                }
                None
            }
            PointerPhase::Moved => {
                if self.pointer_id == Some(event.pointer_id) {
                    self.velocity_tracker
                        .add_position(event.position.x, event.position.y, event.timestamp);
                }
                None
            }
            PointerPhase::Ended => {
                if self.pointer_id != Some(event.pointer_id) {
                    return None;
                }

                self.velocity_tracker
                    .add_position(event.position.x, event.position.y, event.timestamp);

                if let (Some(start_pos), Some(start_time)) =
                    (self.start_position, self.start_time)
                {
                    let delta = Vector::from_points(&start_pos, &event.position);
                    let distance = delta.magnitude();
                    let duration = event.timestamp.duration_since(start_time);
                    let (vx, vy) = self.velocity_tracker.calculate_velocity();
                    let velocity = Velocity::new(vx, vy);
                    let speed = velocity.magnitude();

                    // Check all conditions for a valid swipe
                    if distance >= self.config.min_distance
                        && duration <= Duration::from_millis(self.config.max_duration_ms)
                        && speed >= self.config.min_velocity
                        && self.check_directional_validity(&delta)
                    {
                        if let Some(direction) = SwipeDirection::from_vector(&delta) {
                            if self.is_direction_allowed(direction) {
                                self.state = GestureState::Ended;
                                let result = Some(
                                    GestureEvent::new(
                                        GestureType::Swipe,
                                        GestureState::Ended,
                                        event.position,
                                    )
                                    .with_delta(delta)
                                    .with_velocity(velocity)
                                    .with_direction(direction),
                                );
                                self.reset();
                                return result;
                            }
                        }
                    }
                }

                self.state = GestureState::Failed;
                self.reset();
                None
            }
            PointerPhase::Cancelled => {
                if self.pointer_id == Some(event.pointer_id) {
                    self.state = GestureState::Cancelled;
                    self.reset();
                }
                None
            }
        }
    }

    fn reset(&mut self) {
        self.start_position = None;
        self.start_time = None;
        self.pointer_id = None;
        self.velocity_tracker.reset();
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        15
    }
}

/// Configuration for fling gesture recognition
#[derive(Debug, Clone)]
pub struct FlingConfig {
    /// Minimum distance to consider
    pub min_distance: f32,
    /// Minimum velocity for a fling (pixels per second)
    pub min_velocity: f32,
    /// Maximum velocity cap (pixels per second)
    pub max_velocity: f32,
    /// Friction coefficient for momentum calculation
    pub friction: f32,
}

impl Default for FlingConfig {
    fn default() -> Self {
        Self {
            min_distance: 10.0,
            min_velocity: 200.0,
            max_velocity: 8000.0,
            friction: 0.95,
        }
    }
}

/// Recognizes fling gestures (swipe with velocity for momentum scrolling)
#[derive(Debug)]
pub struct FlingGesture {
    id: GestureRecognizerId,
    config: FlingConfig,
    state: GestureState,
    start_position: Option<Point>,
    last_position: Option<Point>,
    pointer_id: Option<u64>,
    velocity_tracker: VelocityTracker,
}

impl FlingGesture {
    /// Create a new fling gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: FlingConfig::default(),
            state: GestureState::Possible,
            start_position: None,
            last_position: None,
            pointer_id: None,
            velocity_tracker: VelocityTracker::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: FlingConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set minimum velocity
    pub fn min_velocity(mut self, velocity: f32) -> Self {
        self.config.min_velocity = velocity;
        self
    }

    /// Set friction coefficient
    pub fn friction(mut self, friction: f32) -> Self {
        self.config.friction = friction;
        self
    }
}

impl Default for FlingGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for FlingGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::Fling
    }

    fn state(&self) -> GestureState {
        self.state
    }

    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Began => {
                if self.pointer_id.is_none() {
                    self.pointer_id = Some(event.pointer_id);
                    self.start_position = Some(event.position);
                    self.last_position = Some(event.position);
                    self.velocity_tracker.reset();
                    self.velocity_tracker
                        .add_position(event.position.x, event.position.y, event.timestamp);
                    self.state = GestureState::Possible;
                }
                None
            }
            PointerPhase::Moved => {
                if self.pointer_id == Some(event.pointer_id) {
                    self.velocity_tracker
                        .add_position(event.position.x, event.position.y, event.timestamp);
                    self.last_position = Some(event.position);
                }
                None
            }
            PointerPhase::Ended => {
                if self.pointer_id != Some(event.pointer_id) {
                    return None;
                }

                self.velocity_tracker
                    .add_position(event.position.x, event.position.y, event.timestamp);

                if let Some(start_pos) = self.start_position {
                    let delta = Vector::from_points(&start_pos, &event.position);
                    let distance = delta.magnitude();
                    let (vx, vy) = self.velocity_tracker.calculate_velocity();
                    let velocity = Velocity::new(vx, vy).clamped(self.config.max_velocity);
                    let speed = velocity.magnitude();

                    if distance >= self.config.min_distance && speed >= self.config.min_velocity {
                        self.state = GestureState::Ended;
                        let direction = SwipeDirection::from_vector(&delta);
                        let mut gesture_event = GestureEvent::new(
                            GestureType::Fling,
                            GestureState::Ended,
                            event.position,
                        )
                        .with_delta(delta)
                        .with_velocity(velocity);

                        if let Some(dir) = direction {
                            gesture_event = gesture_event.with_direction(dir);
                        }

                        self.reset();
                        return Some(gesture_event);
                    }
                }

                self.state = GestureState::Failed;
                self.reset();
                None
            }
            PointerPhase::Cancelled => {
                if self.pointer_id == Some(event.pointer_id) {
                    self.state = GestureState::Cancelled;
                    self.reset();
                }
                None
            }
        }
    }

    fn reset(&mut self) {
        self.start_position = None;
        self.last_position = None;
        self.pointer_id = None;
        self.velocity_tracker.reset();
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_at(id: u64, x: f32, y: f32, phase: PointerPhase, time_offset_ms: u64) -> PointerEvent {
        let mut event = PointerEvent::new(id, Point::new(x, y), phase);
        // Manually set timestamp for testing
        event.timestamp = Instant::now() + Duration::from_millis(time_offset_ms);
        event
    }

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
    fn test_pan_gesture_starts_after_threshold() {
        let mut pan = PanGesture::new().min_distance(10.0);

        let down = make_down(1, 100.0, 100.0);
        assert!(pan.handle_event(&down).is_none());

        // Move less than threshold
        let move1 = make_move(1, 105.0, 100.0);
        assert!(pan.handle_event(&move1).is_none());

        // Move beyond threshold
        let move2 = make_move(1, 115.0, 100.0);
        let result = pan.handle_event(&move2);
        assert!(result.is_some());
        assert_eq!(result.unwrap().state, GestureState::Began);
    }

    #[test]
    fn test_pan_gesture_continues() {
        let mut pan = PanGesture::new().min_distance(5.0);

        let down = make_down(1, 100.0, 100.0);
        pan.handle_event(&down);

        // Start pan
        let move1 = make_move(1, 110.0, 100.0);
        let began = pan.handle_event(&move1);
        assert!(began.is_some());
        assert_eq!(began.unwrap().state, GestureState::Began);

        // Continue pan
        let move2 = make_move(1, 120.0, 105.0);
        let changed = pan.handle_event(&move2);
        assert!(changed.is_some());
        let event = changed.unwrap();
        assert_eq!(event.state, GestureState::Changed);
        assert!(event.delta.is_some());
    }

    #[test]
    fn test_pan_gesture_ends() {
        let mut pan = PanGesture::new().min_distance(5.0);

        let down = make_down(1, 100.0, 100.0);
        pan.handle_event(&down);

        let move1 = make_move(1, 110.0, 100.0);
        pan.handle_event(&move1);

        let up = make_up(1, 120.0, 100.0);
        let result = pan.handle_event(&up);
        assert!(result.is_some());
        assert_eq!(result.unwrap().state, GestureState::Ended);
    }

    #[test]
    fn test_pan_velocity_tracking() {
        let mut pan = PanGesture::new().min_distance(5.0);

        let down = make_down(1, 100.0, 100.0);
        pan.handle_event(&down);

        let move1 = make_move(1, 110.0, 100.0);
        pan.handle_event(&move1);

        let move2 = make_move(1, 150.0, 100.0);
        let result = pan.handle_event(&move2);

        assert!(result.is_some());
        let event = result.unwrap();
        assert!(event.velocity.is_some());
    }

    #[test]
    fn test_drag_gesture_lifecycle() {
        let mut drag = DragGesture::new();

        let down = make_down(1, 100.0, 100.0);
        drag.handle_event(&down);
        assert_eq!(drag.start_position(), Some(Point::new(100.0, 100.0)));

        let move1 = make_move(1, 110.0, 110.0);
        let began = drag.handle_event(&move1);
        assert!(began.is_some());
        assert_eq!(began.unwrap().state, GestureState::Began);

        let move2 = make_move(1, 120.0, 120.0);
        let changed = drag.handle_event(&move2);
        assert!(changed.is_some());
        assert_eq!(changed.unwrap().state, GestureState::Changed);

        let up = make_up(1, 130.0, 130.0);
        let ended = drag.handle_event(&up);
        assert!(ended.is_some());
        assert_eq!(ended.unwrap().state, GestureState::Ended);
    }

    #[test]
    fn test_swipe_gesture_right() {
        let mut swipe = SwipeGesture::new()
            .min_distance(30.0)
            .min_velocity(100.0);

        // Need to simulate a fast swipe with proper timing
        let base_time = Instant::now();

        let mut down = make_down(1, 100.0, 100.0);
        down.timestamp = base_time;
        swipe.handle_event(&down);

        let mut move1 = make_move(1, 150.0, 100.0);
        move1.timestamp = base_time + Duration::from_millis(20);
        swipe.handle_event(&move1);

        let mut up = make_up(1, 200.0, 100.0);
        up.timestamp = base_time + Duration::from_millis(40);
        let result = swipe.handle_event(&up);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.gesture_type, GestureType::Swipe);
        assert_eq!(event.direction, Some(SwipeDirection::Right));
    }

    #[test]
    fn test_swipe_gesture_left() {
        let mut swipe = SwipeGesture::new()
            .min_distance(30.0)
            .min_velocity(100.0);

        let base_time = Instant::now();

        let mut down = make_down(1, 200.0, 100.0);
        down.timestamp = base_time;
        swipe.handle_event(&down);

        let mut move1 = make_move(1, 150.0, 100.0);
        move1.timestamp = base_time + Duration::from_millis(20);
        swipe.handle_event(&move1);

        let mut up = make_up(1, 100.0, 100.0);
        up.timestamp = base_time + Duration::from_millis(40);
        let result = swipe.handle_event(&up);

        assert!(result.is_some());
        assert_eq!(result.unwrap().direction, Some(SwipeDirection::Left));
    }

    #[test]
    fn test_swipe_direction_filter() {
        let mut swipe = SwipeGesture::new()
            .horizontal()
            .min_distance(30.0)
            .min_velocity(100.0);

        let base_time = Instant::now();

        // Try a vertical swipe (should fail)
        let mut down = make_down(1, 100.0, 100.0);
        down.timestamp = base_time;
        swipe.handle_event(&down);

        let mut move1 = make_move(1, 100.0, 50.0);
        move1.timestamp = base_time + Duration::from_millis(20);
        swipe.handle_event(&move1);

        let mut up = make_up(1, 100.0, 0.0);
        up.timestamp = base_time + Duration::from_millis(40);
        let result = swipe.handle_event(&up);

        assert!(result.is_none(), "Vertical swipe should not be recognized for horizontal-only");
    }

    #[test]
    fn test_swipe_fails_too_slow() {
        let mut swipe = SwipeGesture::new()
            .min_distance(30.0)
            .min_velocity(1000.0);

        let base_time = Instant::now();

        let mut down = make_down(1, 100.0, 100.0);
        down.timestamp = base_time;
        swipe.handle_event(&down);

        // Very slow movement
        let mut up = make_up(1, 200.0, 100.0);
        up.timestamp = base_time + Duration::from_millis(5000);
        let result = swipe.handle_event(&up);

        assert!(result.is_none());
    }

    #[test]
    fn test_fling_gesture_success() {
        let mut fling = FlingGesture::new()
            .min_velocity(100.0);

        let base_time = Instant::now();

        let mut down = make_down(1, 100.0, 100.0);
        down.timestamp = base_time;
        fling.handle_event(&down);

        let mut move1 = make_move(1, 150.0, 100.0);
        move1.timestamp = base_time + Duration::from_millis(10);
        fling.handle_event(&move1);

        let mut move2 = make_move(1, 200.0, 100.0);
        move2.timestamp = base_time + Duration::from_millis(20);
        fling.handle_event(&move2);

        let mut up = make_up(1, 250.0, 100.0);
        up.timestamp = base_time + Duration::from_millis(30);
        let result = fling.handle_event(&up);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.gesture_type, GestureType::Fling);
        assert!(event.velocity.is_some());
        assert!(event.velocity.unwrap().magnitude() > 0.0);
    }

    #[test]
    fn test_fling_velocity_clamping() {
        let mut fling = FlingGesture::with_config(FlingConfig {
            min_distance: 5.0,
            min_velocity: 100.0,
            max_velocity: 500.0,
            friction: 0.95,
        });

        let base_time = Instant::now();

        let mut down = make_down(1, 0.0, 0.0);
        down.timestamp = base_time;
        fling.handle_event(&down);

        // Very fast movement
        let mut up = make_up(1, 1000.0, 0.0);
        up.timestamp = base_time + Duration::from_millis(10);
        let result = fling.handle_event(&up);

        if let Some(event) = result {
            let velocity = event.velocity.unwrap();
            assert!(velocity.magnitude() <= 500.0 + 0.1);
        }
    }

    #[test]
    fn test_pan_cancelled() {
        let mut pan = PanGesture::new().min_distance(5.0);

        let down = make_down(1, 100.0, 100.0);
        pan.handle_event(&down);

        let move1 = make_move(1, 110.0, 100.0);
        pan.handle_event(&move1);

        let cancel = PointerEvent::cancelled(1, 110.0, 100.0);
        let result = pan.handle_event(&cancel);

        assert!(result.is_some());
        assert_eq!(result.unwrap().state, GestureState::Cancelled);
    }

    #[test]
    fn test_pan_ignores_other_pointers() {
        let mut pan = PanGesture::new();

        let down1 = make_down(1, 100.0, 100.0);
        pan.handle_event(&down1);

        // Different pointer's movement should be ignored
        let move2 = make_move(2, 200.0, 200.0);
        let result = pan.handle_event(&move2);
        assert!(result.is_none());

        // Original pointer should still work
        let move1 = make_move(1, 120.0, 100.0);
        let result = pan.handle_event(&move1);
        assert!(result.is_some());
    }

    #[test]
    fn test_gesture_reset() {
        let mut pan = PanGesture::new();

        let down = make_down(1, 100.0, 100.0);
        pan.handle_event(&down);

        pan.reset();

        assert_eq!(pan.state(), GestureState::Possible);
        assert!(pan.start_position.is_none());
        assert!(pan.pointer_id.is_none());
    }
}
