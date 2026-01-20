//! Gesture recognition.
//!
//! Provides high-level gesture recognition built on top of touch events.
//! Supports common gestures like tap, pan, pinch, rotate, and swipe.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::touch::{TouchEvent, TouchPhase};

/// Types of gestures that can be recognized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GestureType {
    /// Single tap.
    Tap,
    /// Double tap.
    DoubleTap,
    /// Long press.
    LongPress,
    /// Pan (drag).
    Pan,
    /// Pinch (scale).
    Pinch,
    /// Rotation.
    Rotate,
    /// Swipe.
    Swipe,
}

impl GestureType {
    /// Returns true if this gesture requires multiple touches.
    pub fn is_multi_touch(&self) -> bool {
        matches!(self, GestureType::Pinch | GestureType::Rotate)
    }

    /// Returns true if this is a discrete gesture (one-shot).
    pub fn is_discrete(&self) -> bool {
        matches!(
            self,
            GestureType::Tap | GestureType::DoubleTap | GestureType::Swipe
        )
    }

    /// Returns true if this is a continuous gesture.
    pub fn is_continuous(&self) -> bool {
        matches!(
            self,
            GestureType::Pan | GestureType::Pinch | GestureType::Rotate | GestureType::LongPress
        )
    }
}

impl std::fmt::Display for GestureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            GestureType::Tap => "Tap",
            GestureType::DoubleTap => "Double Tap",
            GestureType::LongPress => "Long Press",
            GestureType::Pan => "Pan",
            GestureType::Pinch => "Pinch",
            GestureType::Rotate => "Rotate",
            GestureType::Swipe => "Swipe",
        };
        write!(f, "{}", name)
    }
}

/// State of a gesture recognizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum GestureState {
    /// Recognizer is waiting for touches.
    #[default]
    Possible,
    /// Gesture has begun.
    Began,
    /// Gesture is in progress (continuous gestures).
    Changed,
    /// Gesture has ended successfully.
    Ended,
    /// Gesture was cancelled.
    Cancelled,
    /// Gesture recognition failed.
    Failed,
}

impl GestureState {
    /// Returns true if the gesture is active.
    pub fn is_active(&self) -> bool {
        matches!(self, GestureState::Began | GestureState::Changed)
    }

    /// Returns true if the gesture has completed (success or failure).
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            GestureState::Ended | GestureState::Cancelled | GestureState::Failed
        )
    }

    /// Returns true if the gesture succeeded.
    pub fn succeeded(&self) -> bool {
        matches!(self, GestureState::Ended)
    }
}

/// Direction for swipe gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwipeDirection {
    /// Swipe up.
    Up,
    /// Swipe down.
    Down,
    /// Swipe left.
    Left,
    /// Swipe right.
    Right,
}

impl SwipeDirection {
    /// Determine direction from a velocity vector.
    pub fn from_velocity(vx: f32, vy: f32) -> Self {
        if vx.abs() > vy.abs() {
            if vx > 0.0 {
                SwipeDirection::Right
            } else {
                SwipeDirection::Left
            }
        } else if vy > 0.0 {
            SwipeDirection::Down
        } else {
            SwipeDirection::Up
        }
    }

    /// Returns true if this is a horizontal swipe.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, SwipeDirection::Left | SwipeDirection::Right)
    }

    /// Returns true if this is a vertical swipe.
    pub fn is_vertical(&self) -> bool {
        matches!(self, SwipeDirection::Up | SwipeDirection::Down)
    }
}

/// A recognized gesture event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureEvent {
    /// Type of gesture.
    pub gesture_type: GestureType,
    /// Current state of the gesture.
    pub state: GestureState,
    /// Position of the gesture (center for multi-touch).
    pub position: (f32, f32),
    /// Additional gesture-specific data.
    pub data: GestureData,
}

/// Gesture-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureData {
    /// No additional data.
    None,
    /// Tap data.
    Tap {
        /// Number of taps.
        tap_count: u32,
    },
    /// Pan data.
    Pan {
        /// Translation from start.
        translation: (f32, f32),
        /// Current velocity.
        velocity: (f32, f32),
    },
    /// Pinch data.
    Pinch {
        /// Scale factor (1.0 = no change).
        scale: f32,
        /// Velocity of scale change.
        velocity: f32,
    },
    /// Rotation data.
    Rotate {
        /// Rotation in radians.
        rotation: f32,
        /// Velocity of rotation.
        velocity: f32,
    },
    /// Swipe data.
    Swipe {
        /// Direction of the swipe.
        direction: SwipeDirection,
        /// Velocity of the swipe.
        velocity: f32,
    },
    /// Long press data.
    LongPress {
        /// Duration of the press.
        #[serde(skip, default = "default_duration")]
        duration: Duration,
    },
}

fn default_duration() -> Duration {
    Duration::from_millis(0)
}

impl GestureEvent {
    /// Create a new gesture event.
    pub fn new(gesture_type: GestureType, state: GestureState, position: (f32, f32)) -> Self {
        Self {
            gesture_type,
            state,
            position,
            data: GestureData::None,
        }
    }

    /// Set the gesture data.
    pub fn with_data(mut self, data: GestureData) -> Self {
        self.data = data;
        self
    }
}

/// Base gesture recognizer configuration.
#[derive(Debug, Clone)]
pub struct GestureRecognizer {
    /// Type of gesture to recognize.
    pub gesture_type: GestureType,
    /// Current state.
    pub state: GestureState,
    /// Whether the recognizer is enabled.
    pub enabled: bool,
    /// Whether to cancel touches on recognition.
    pub cancels_touches_in_view: bool,
    /// Whether to delay touches until recognition fails.
    pub delays_touches_began: bool,
}

impl GestureRecognizer {
    /// Create a new gesture recognizer.
    pub fn new(gesture_type: GestureType) -> Self {
        Self {
            gesture_type,
            state: GestureState::Possible,
            enabled: true,
            cancels_touches_in_view: true,
            delays_touches_began: false,
        }
    }

    /// Reset the recognizer to initial state.
    pub fn reset(&mut self) {
        self.state = GestureState::Possible;
    }

    /// Set enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset();
        }
    }
}

/// Tap gesture recognizer.
#[derive(Debug, Clone)]
pub struct TapRecognizer {
    base: GestureRecognizer,
    /// Number of taps required.
    pub required_taps: u32,
    /// Number of touches required.
    pub required_touches: u32,
    /// Maximum movement allowed.
    pub allowable_movement: f32,
    /// Current tap count.
    current_taps: u32,
    /// Last tap time.
    last_tap_time: Option<Instant>,
    /// Last tap position.
    last_tap_position: Option<(f32, f32)>,
    /// Start position of current gesture.
    start_position: Option<(f32, f32)>,
}

impl TapRecognizer {
    /// Create a new tap recognizer.
    pub fn new() -> Self {
        Self {
            base: GestureRecognizer::new(GestureType::Tap),
            required_taps: 1,
            required_touches: 1,
            allowable_movement: 10.0,
            current_taps: 0,
            last_tap_time: None,
            last_tap_position: None,
            start_position: None,
        }
    }

    /// Create a double-tap recognizer.
    pub fn double_tap() -> Self {
        let mut recognizer = Self::new();
        recognizer.required_taps = 2;
        recognizer.base.gesture_type = GestureType::DoubleTap;
        recognizer
    }

    /// Process a touch event.
    pub fn process_touch(&mut self, event: &TouchEvent) -> Option<GestureEvent> {
        if !self.base.enabled {
            return None;
        }

        match event.phase {
            TouchPhase::Began => {
                self.start_position = Some(event.position());
                self.base.state = GestureState::Possible;
                None
            }
            TouchPhase::Moved => {
                if let Some(start) = self.start_position {
                    let (x, y) = event.position();
                    let distance = ((x - start.0).powi(2) + (y - start.1).powi(2)).sqrt();
                    if distance > self.allowable_movement {
                        self.base.state = GestureState::Failed;
                    }
                }
                None
            }
            TouchPhase::Ended => {
                if self.base.state == GestureState::Failed {
                    self.reset();
                    return None;
                }

                let now = Instant::now();
                let position = event.position();

                // Check if this continues a tap sequence
                let continues_sequence = self.last_tap_time.map_or(false, |t| {
                    now.duration_since(t) < Duration::from_millis(300)
                }) && self.last_tap_position.map_or(true, |p| {
                    let dx = position.0 - p.0;
                    let dy = position.1 - p.1;
                    (dx * dx + dy * dy).sqrt() < self.allowable_movement * 2.0
                });

                if continues_sequence {
                    self.current_taps += 1;
                } else {
                    self.current_taps = 1;
                }

                self.last_tap_time = Some(now);
                self.last_tap_position = Some(position);

                if self.current_taps >= self.required_taps {
                    self.base.state = GestureState::Ended;
                    let event = GestureEvent::new(self.base.gesture_type, GestureState::Ended, position)
                        .with_data(GestureData::Tap {
                            tap_count: self.current_taps,
                        });
                    self.reset();
                    Some(event)
                } else {
                    None
                }
            }
            TouchPhase::Cancelled => {
                self.reset();
                None
            }
            _ => None,
        }
    }

    /// Reset the recognizer.
    pub fn reset(&mut self) {
        self.base.reset();
        self.current_taps = 0;
        self.start_position = None;
    }
}

impl Default for TapRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Pan gesture recognizer.
#[derive(Debug, Clone)]
pub struct PanRecognizer {
    base: GestureRecognizer,
    /// Minimum distance to trigger pan.
    pub min_distance: f32,
    /// Start position.
    start_position: Option<(f32, f32)>,
    /// Previous position.
    prev_position: Option<(f32, f32)>,
    /// Previous time for velocity calculation.
    prev_time: Option<Instant>,
    /// Current translation.
    translation: (f32, f32),
    /// Current velocity.
    velocity: (f32, f32),
}

impl PanRecognizer {
    /// Create a new pan recognizer.
    pub fn new() -> Self {
        Self {
            base: GestureRecognizer::new(GestureType::Pan),
            min_distance: 10.0,
            start_position: None,
            prev_position: None,
            prev_time: None,
            translation: (0.0, 0.0),
            velocity: (0.0, 0.0),
        }
    }

    /// Process a touch event.
    pub fn process_touch(&mut self, event: &TouchEvent) -> Option<GestureEvent> {
        if !self.base.enabled {
            return None;
        }

        let position = event.position();
        let now = Instant::now();

        match event.phase {
            TouchPhase::Began => {
                self.start_position = Some(position);
                self.prev_position = Some(position);
                self.prev_time = Some(now);
                self.translation = (0.0, 0.0);
                self.velocity = (0.0, 0.0);
                self.base.state = GestureState::Possible;
                None
            }
            TouchPhase::Moved => {
                let start = self.start_position?;
                let prev = self.prev_position.unwrap_or(start);

                self.translation = (position.0 - start.0, position.1 - start.1);

                // Calculate velocity
                if let Some(prev_time) = self.prev_time {
                    let dt = now.duration_since(prev_time).as_secs_f32();
                    if dt > 0.0 {
                        self.velocity = (
                            (position.0 - prev.0) / dt,
                            (position.1 - prev.1) / dt,
                        );
                    }
                }

                self.prev_position = Some(position);
                self.prev_time = Some(now);

                let distance = (self.translation.0.powi(2) + self.translation.1.powi(2)).sqrt();

                if self.base.state == GestureState::Possible && distance >= self.min_distance {
                    self.base.state = GestureState::Began;
                    Some(
                        GestureEvent::new(GestureType::Pan, GestureState::Began, position)
                            .with_data(GestureData::Pan {
                                translation: self.translation,
                                velocity: self.velocity,
                            }),
                    )
                } else if self.base.state == GestureState::Began
                    || self.base.state == GestureState::Changed
                {
                    self.base.state = GestureState::Changed;
                    Some(
                        GestureEvent::new(GestureType::Pan, GestureState::Changed, position)
                            .with_data(GestureData::Pan {
                                translation: self.translation,
                                velocity: self.velocity,
                            }),
                    )
                } else {
                    None
                }
            }
            TouchPhase::Ended => {
                if self.base.state.is_active() {
                    self.base.state = GestureState::Ended;
                    let event = GestureEvent::new(GestureType::Pan, GestureState::Ended, position)
                        .with_data(GestureData::Pan {
                            translation: self.translation,
                            velocity: self.velocity,
                        });
                    self.reset();
                    Some(event)
                } else {
                    self.reset();
                    None
                }
            }
            TouchPhase::Cancelled => {
                if self.base.state.is_active() {
                    let event =
                        GestureEvent::new(GestureType::Pan, GestureState::Cancelled, position);
                    self.reset();
                    Some(event)
                } else {
                    self.reset();
                    None
                }
            }
            _ => None,
        }
    }

    /// Reset the recognizer.
    pub fn reset(&mut self) {
        self.base.reset();
        self.start_position = None;
        self.prev_position = None;
        self.prev_time = None;
        self.translation = (0.0, 0.0);
        self.velocity = (0.0, 0.0);
    }

    /// Get current translation.
    pub fn translation(&self) -> (f32, f32) {
        self.translation
    }

    /// Get current velocity.
    pub fn velocity(&self) -> (f32, f32) {
        self.velocity
    }
}

impl Default for PanRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Pinch gesture recognizer.
#[derive(Debug, Clone)]
pub struct PinchRecognizer {
    base: GestureRecognizer,
    /// Initial span between touches.
    initial_span: Option<f32>,
    /// Previous span for velocity.
    prev_span: Option<f32>,
    /// Previous time.
    prev_time: Option<Instant>,
    /// Current scale.
    scale: f32,
    /// Scale velocity.
    velocity: f32,
}

impl PinchRecognizer {
    /// Create a new pinch recognizer.
    pub fn new() -> Self {
        Self {
            base: GestureRecognizer::new(GestureType::Pinch),
            initial_span: None,
            prev_span: None,
            prev_time: None,
            scale: 1.0,
            velocity: 0.0,
        }
    }

    /// Process a touch event.
    pub fn process_touch(&mut self, event: &TouchEvent) -> Option<GestureEvent> {
        if !self.base.enabled || !event.is_multi_touch() {
            return None;
        }

        let span = event.span()?;
        let center = event.center();
        let now = Instant::now();

        match event.phase {
            TouchPhase::Began => {
                self.initial_span = Some(span);
                self.prev_span = Some(span);
                self.prev_time = Some(now);
                self.scale = 1.0;
                self.velocity = 0.0;
                self.base.state = GestureState::Began;
                Some(
                    GestureEvent::new(GestureType::Pinch, GestureState::Began, center)
                        .with_data(GestureData::Pinch {
                            scale: 1.0,
                            velocity: 0.0,
                        }),
                )
            }
            TouchPhase::Moved => {
                let initial = self.initial_span?;
                self.scale = span / initial;

                // Calculate velocity
                if let (Some(prev_span), Some(prev_time)) = (self.prev_span, self.prev_time) {
                    let dt = now.duration_since(prev_time).as_secs_f32();
                    if dt > 0.0 {
                        self.velocity = (span - prev_span) / (initial * dt);
                    }
                }

                self.prev_span = Some(span);
                self.prev_time = Some(now);
                self.base.state = GestureState::Changed;

                Some(
                    GestureEvent::new(GestureType::Pinch, GestureState::Changed, center)
                        .with_data(GestureData::Pinch {
                            scale: self.scale,
                            velocity: self.velocity,
                        }),
                )
            }
            TouchPhase::Ended => {
                self.base.state = GestureState::Ended;
                let event = GestureEvent::new(GestureType::Pinch, GestureState::Ended, center)
                    .with_data(GestureData::Pinch {
                        scale: self.scale,
                        velocity: self.velocity,
                    });
                self.reset();
                Some(event)
            }
            TouchPhase::Cancelled => {
                let event = GestureEvent::new(GestureType::Pinch, GestureState::Cancelled, center);
                self.reset();
                Some(event)
            }
            _ => None,
        }
    }

    /// Reset the recognizer.
    pub fn reset(&mut self) {
        self.base.reset();
        self.initial_span = None;
        self.prev_span = None;
        self.prev_time = None;
        self.scale = 1.0;
        self.velocity = 0.0;
    }

    /// Get current scale.
    pub fn scale(&self) -> f32 {
        self.scale
    }
}

impl Default for PinchRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Rotation gesture recognizer.
#[derive(Debug, Clone)]
pub struct RotationRecognizer {
    base: GestureRecognizer,
    /// Initial angle between touches.
    initial_angle: Option<f32>,
    /// Previous angle for velocity.
    prev_angle: Option<f32>,
    /// Previous time.
    prev_time: Option<Instant>,
    /// Current rotation.
    rotation: f32,
    /// Rotation velocity.
    velocity: f32,
}

impl RotationRecognizer {
    /// Create a new rotation recognizer.
    pub fn new() -> Self {
        Self {
            base: GestureRecognizer::new(GestureType::Rotate),
            initial_angle: None,
            prev_angle: None,
            prev_time: None,
            rotation: 0.0,
            velocity: 0.0,
        }
    }

    /// Process a touch event.
    pub fn process_touch(&mut self, event: &TouchEvent) -> Option<GestureEvent> {
        if !self.base.enabled || !event.is_multi_touch() {
            return None;
        }

        let angle = event.angle()?;
        let center = event.center();
        let now = Instant::now();

        match event.phase {
            TouchPhase::Began => {
                self.initial_angle = Some(angle);
                self.prev_angle = Some(angle);
                self.prev_time = Some(now);
                self.rotation = 0.0;
                self.velocity = 0.0;
                self.base.state = GestureState::Began;
                Some(
                    GestureEvent::new(GestureType::Rotate, GestureState::Began, center)
                        .with_data(GestureData::Rotate {
                            rotation: 0.0,
                            velocity: 0.0,
                        }),
                )
            }
            TouchPhase::Moved => {
                let initial = self.initial_angle?;
                self.rotation = angle - initial;

                // Normalize rotation to -PI..PI
                while self.rotation > std::f32::consts::PI {
                    self.rotation -= 2.0 * std::f32::consts::PI;
                }
                while self.rotation < -std::f32::consts::PI {
                    self.rotation += 2.0 * std::f32::consts::PI;
                }

                // Calculate velocity
                if let (Some(prev_angle), Some(prev_time)) = (self.prev_angle, self.prev_time) {
                    let dt = now.duration_since(prev_time).as_secs_f32();
                    if dt > 0.0 {
                        let mut delta = angle - prev_angle;
                        // Normalize delta
                        while delta > std::f32::consts::PI {
                            delta -= 2.0 * std::f32::consts::PI;
                        }
                        while delta < -std::f32::consts::PI {
                            delta += 2.0 * std::f32::consts::PI;
                        }
                        self.velocity = delta / dt;
                    }
                }

                self.prev_angle = Some(angle);
                self.prev_time = Some(now);
                self.base.state = GestureState::Changed;

                Some(
                    GestureEvent::new(GestureType::Rotate, GestureState::Changed, center)
                        .with_data(GestureData::Rotate {
                            rotation: self.rotation,
                            velocity: self.velocity,
                        }),
                )
            }
            TouchPhase::Ended => {
                self.base.state = GestureState::Ended;
                let event = GestureEvent::new(GestureType::Rotate, GestureState::Ended, center)
                    .with_data(GestureData::Rotate {
                        rotation: self.rotation,
                        velocity: self.velocity,
                    });
                self.reset();
                Some(event)
            }
            TouchPhase::Cancelled => {
                let event = GestureEvent::new(GestureType::Rotate, GestureState::Cancelled, center);
                self.reset();
                Some(event)
            }
            _ => None,
        }
    }

    /// Reset the recognizer.
    pub fn reset(&mut self) {
        self.base.reset();
        self.initial_angle = None;
        self.prev_angle = None;
        self.prev_time = None;
        self.rotation = 0.0;
        self.velocity = 0.0;
    }

    /// Get current rotation in radians.
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
}

impl Default for RotationRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Swipe gesture recognizer.
#[derive(Debug, Clone)]
pub struct SwipeRecognizer {
    base: GestureRecognizer,
    /// Allowed directions.
    pub allowed_directions: Vec<SwipeDirection>,
    /// Minimum velocity required.
    pub min_velocity: f32,
    /// Maximum duration allowed.
    pub max_duration: Duration,
    /// Start position.
    start_position: Option<(f32, f32)>,
    /// Start time.
    start_time: Option<Instant>,
}

impl SwipeRecognizer {
    /// Create a new swipe recognizer.
    pub fn new() -> Self {
        Self {
            base: GestureRecognizer::new(GestureType::Swipe),
            allowed_directions: vec![
                SwipeDirection::Up,
                SwipeDirection::Down,
                SwipeDirection::Left,
                SwipeDirection::Right,
            ],
            min_velocity: 500.0,
            max_duration: Duration::from_millis(500),
            start_position: None,
            start_time: None,
        }
    }

    /// Create a horizontal swipe recognizer.
    pub fn horizontal() -> Self {
        let mut recognizer = Self::new();
        recognizer.allowed_directions = vec![SwipeDirection::Left, SwipeDirection::Right];
        recognizer
    }

    /// Create a vertical swipe recognizer.
    pub fn vertical() -> Self {
        let mut recognizer = Self::new();
        recognizer.allowed_directions = vec![SwipeDirection::Up, SwipeDirection::Down];
        recognizer
    }

    /// Process a touch event.
    pub fn process_touch(&mut self, event: &TouchEvent) -> Option<GestureEvent> {
        if !self.base.enabled {
            return None;
        }

        let position = event.position();
        let now = Instant::now();

        match event.phase {
            TouchPhase::Began => {
                self.start_position = Some(position);
                self.start_time = Some(now);
                self.base.state = GestureState::Possible;
                None
            }
            TouchPhase::Ended => {
                let start = self.start_position?;
                let start_time = self.start_time?;

                let duration = now.duration_since(start_time);
                if duration > self.max_duration {
                    self.reset();
                    return None;
                }

                let dx = position.0 - start.0;
                let dy = position.1 - start.1;
                let distance = (dx * dx + dy * dy).sqrt();
                let dt = duration.as_secs_f32();

                if dt > 0.0 {
                    let velocity = distance / dt;
                    if velocity >= self.min_velocity {
                        let vx = dx / dt;
                        let vy = dy / dt;
                        let direction = SwipeDirection::from_velocity(vx, vy);

                        if self.allowed_directions.contains(&direction) {
                            self.base.state = GestureState::Ended;
                            let event =
                                GestureEvent::new(GestureType::Swipe, GestureState::Ended, position)
                                    .with_data(GestureData::Swipe { direction, velocity });
                            self.reset();
                            return Some(event);
                        }
                    }
                }

                self.reset();
                None
            }
            TouchPhase::Cancelled => {
                self.reset();
                None
            }
            _ => None,
        }
    }

    /// Reset the recognizer.
    pub fn reset(&mut self) {
        self.base.reset();
        self.start_position = None;
        self.start_time = None;
    }
}

impl Default for SwipeRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::touch::{TouchId, TouchPoint};

    fn make_touch_event(x: f32, y: f32, phase: TouchPhase) -> TouchEvent {
        let point = TouchPoint::new(TouchId::new(1), (x, y));
        TouchEvent::new(point, phase)
    }

    #[test]
    fn test_gesture_type() {
        assert!(GestureType::Pinch.is_multi_touch());
        assert!(!GestureType::Tap.is_multi_touch());

        assert!(GestureType::Tap.is_discrete());
        assert!(!GestureType::Pan.is_discrete());

        assert!(GestureType::Pan.is_continuous());
        assert!(!GestureType::Tap.is_continuous());
    }

    #[test]
    fn test_gesture_state() {
        assert!(GestureState::Began.is_active());
        assert!(GestureState::Changed.is_active());
        assert!(!GestureState::Ended.is_active());

        assert!(GestureState::Ended.is_finished());
        assert!(GestureState::Failed.is_finished());
        assert!(!GestureState::Began.is_finished());

        assert!(GestureState::Ended.succeeded());
        assert!(!GestureState::Failed.succeeded());
    }

    #[test]
    fn test_swipe_direction() {
        assert_eq!(SwipeDirection::from_velocity(100.0, 0.0), SwipeDirection::Right);
        assert_eq!(SwipeDirection::from_velocity(-100.0, 0.0), SwipeDirection::Left);
        assert_eq!(SwipeDirection::from_velocity(0.0, 100.0), SwipeDirection::Down);
        assert_eq!(SwipeDirection::from_velocity(0.0, -100.0), SwipeDirection::Up);

        assert!(SwipeDirection::Left.is_horizontal());
        assert!(!SwipeDirection::Up.is_horizontal());
        assert!(SwipeDirection::Up.is_vertical());
    }

    #[test]
    fn test_tap_recognizer() {
        let mut recognizer = TapRecognizer::new();

        let began = make_touch_event(100.0, 100.0, TouchPhase::Began);
        assert!(recognizer.process_touch(&began).is_none());

        let ended = make_touch_event(100.0, 100.0, TouchPhase::Ended);
        let event = recognizer.process_touch(&ended);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.gesture_type, GestureType::Tap);
        assert_eq!(event.state, GestureState::Ended);
    }

    #[test]
    fn test_pan_recognizer() {
        let mut recognizer = PanRecognizer::new();

        let began = make_touch_event(100.0, 100.0, TouchPhase::Began);
        assert!(recognizer.process_touch(&began).is_none());

        // Small movement - no pan yet
        let small_move = make_touch_event(105.0, 105.0, TouchPhase::Moved);
        assert!(recognizer.process_touch(&small_move).is_none());

        // Large movement - pan starts
        let large_move = make_touch_event(150.0, 150.0, TouchPhase::Moved);
        let event = recognizer.process_touch(&large_move);
        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.gesture_type, GestureType::Pan);
        assert_eq!(event.state, GestureState::Began);
    }
}
