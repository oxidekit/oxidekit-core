//! Tap-based gesture recognizers
//!
//! This module provides recognizers for tap gestures including:
//! - Single tap
//! - Double tap
//! - Long press
//! - Press and hold

use crate::recognizer::{
    GestureEvent, GestureRecognizer, GestureRecognizerId, GestureState, GestureType, Point,
    PointerEvent, PointerPhase,
};
use std::time::{Duration, Instant};

/// Configuration for tap gesture recognition
#[derive(Debug, Clone)]
pub struct TapConfig {
    /// Maximum movement allowed during a tap (in pixels)
    pub movement_tolerance: f32,
    /// Maximum duration for a tap (in milliseconds)
    pub max_duration_ms: u64,
}

impl Default for TapConfig {
    fn default() -> Self {
        Self {
            movement_tolerance: 10.0,
            max_duration_ms: 300,
        }
    }
}

/// Recognizes single tap gestures
#[derive(Debug)]
pub struct TapGesture {
    id: GestureRecognizerId,
    config: TapConfig,
    state: GestureState,
    start_position: Option<Point>,
    start_time: Option<Instant>,
    pointer_id: Option<u64>,
}

impl TapGesture {
    /// Create a new tap gesture recognizer with default config
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: TapConfig::default(),
            state: GestureState::Possible,
            start_position: None,
            start_time: None,
            pointer_id: None,
        }
    }

    /// Create a new tap gesture recognizer with custom config
    pub fn with_config(config: TapConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set movement tolerance
    pub fn movement_tolerance(mut self, tolerance: f32) -> Self {
        self.config.movement_tolerance = tolerance;
        self
    }

    /// Set maximum tap duration
    pub fn max_duration(mut self, duration_ms: u64) -> Self {
        self.config.max_duration_ms = duration_ms;
        self
    }
}

impl Default for TapGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for TapGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::Tap
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
                    self.state = GestureState::Possible;
                }
                None
            }
            PointerPhase::Moved => {
                if self.pointer_id == Some(event.pointer_id) {
                    if let Some(start) = self.start_position {
                        let distance = start.distance_to(&event.position);
                        if distance > self.config.movement_tolerance {
                            self.state = GestureState::Failed;
                            self.reset();
                        }
                    }
                }
                None
            }
            PointerPhase::Ended => {
                if self.pointer_id == Some(event.pointer_id) {
                    if let (Some(start_pos), Some(start_time)) =
                        (self.start_position, self.start_time)
                    {
                        let distance = start_pos.distance_to(&event.position);
                        let duration = event.timestamp.duration_since(start_time);

                        if distance <= self.config.movement_tolerance
                            && duration <= Duration::from_millis(self.config.max_duration_ms)
                        {
                            self.state = GestureState::Ended;
                            let gesture_event =
                                GestureEvent::new(GestureType::Tap, GestureState::Ended, event.position);
                            self.reset();
                            return Some(gesture_event);
                        }
                    }
                    self.state = GestureState::Failed;
                    self.reset();
                }
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
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        10
    }
}

/// Configuration for double tap gesture recognition
#[derive(Debug, Clone)]
pub struct DoubleTapConfig {
    /// Maximum movement allowed during each tap
    pub movement_tolerance: f32,
    /// Maximum duration for each tap
    pub max_tap_duration_ms: u64,
    /// Maximum time between taps
    pub max_interval_ms: u64,
    /// Maximum distance between tap locations
    pub max_distance: f32,
}

impl Default for DoubleTapConfig {
    fn default() -> Self {
        Self {
            movement_tolerance: 10.0,
            max_tap_duration_ms: 300,
            max_interval_ms: 300,
            max_distance: 40.0,
        }
    }
}

/// Recognizes double tap gestures
#[derive(Debug)]
pub struct DoubleTapGesture {
    id: GestureRecognizerId,
    config: DoubleTapConfig,
    state: GestureState,
    first_tap_position: Option<Point>,
    first_tap_time: Option<Instant>,
    current_tap_start: Option<Point>,
    current_tap_time: Option<Instant>,
    pointer_id: Option<u64>,
    tap_count: usize,
}

impl DoubleTapGesture {
    /// Create a new double tap gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: DoubleTapConfig::default(),
            state: GestureState::Possible,
            first_tap_position: None,
            first_tap_time: None,
            current_tap_start: None,
            current_tap_time: None,
            pointer_id: None,
            tap_count: 0,
        }
    }

    /// Create with custom config
    pub fn with_config(config: DoubleTapConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set the timeout between taps
    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.config.max_interval_ms = timeout_ms;
        self
    }
}

impl Default for DoubleTapGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for DoubleTapGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::DoubleTap
    }

    fn state(&self) -> GestureState {
        self.state
    }

    fn handle_event(&mut self, event: &PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Began => {
                // Check if this could be the second tap
                if let Some(first_time) = self.first_tap_time {
                    let interval = event.timestamp.duration_since(first_time);
                    if interval > Duration::from_millis(self.config.max_interval_ms) {
                        // Too long since first tap, start over
                        self.first_tap_position = None;
                        self.first_tap_time = None;
                        self.tap_count = 0;
                    }
                }

                self.pointer_id = Some(event.pointer_id);
                self.current_tap_start = Some(event.position);
                self.current_tap_time = Some(event.timestamp);
                None
            }
            PointerPhase::Moved => {
                if self.pointer_id == Some(event.pointer_id) {
                    if let Some(start) = self.current_tap_start {
                        let distance = start.distance_to(&event.position);
                        if distance > self.config.movement_tolerance {
                            self.state = GestureState::Failed;
                            self.reset();
                        }
                    }
                }
                None
            }
            PointerPhase::Ended => {
                if self.pointer_id != Some(event.pointer_id) {
                    return None;
                }

                if let (Some(start_pos), Some(start_time)) =
                    (self.current_tap_start, self.current_tap_time)
                {
                    let distance = start_pos.distance_to(&event.position);
                    let duration = event.timestamp.duration_since(start_time);

                    // Valid tap?
                    if distance <= self.config.movement_tolerance
                        && duration <= Duration::from_millis(self.config.max_tap_duration_ms)
                    {
                        if self.tap_count == 0 {
                            // First tap completed
                            self.first_tap_position = Some(event.position);
                            self.first_tap_time = Some(event.timestamp);
                            self.tap_count = 1;
                            self.pointer_id = None;
                            self.current_tap_start = None;
                            self.current_tap_time = None;
                        } else if self.tap_count == 1 {
                            // Check if second tap is close enough to first
                            if let Some(first_pos) = self.first_tap_position {
                                let tap_distance = first_pos.distance_to(&event.position);
                                if tap_distance <= self.config.max_distance {
                                    // Double tap recognized!
                                    self.state = GestureState::Ended;
                                    let gesture_event = GestureEvent::new(
                                        GestureType::DoubleTap,
                                        GestureState::Ended,
                                        event.position,
                                    )
                                    .with_tap_count(2);
                                    self.reset();
                                    return Some(gesture_event);
                                }
                            }
                            // Second tap too far from first
                            self.state = GestureState::Failed;
                            self.reset();
                        }
                    } else {
                        self.state = GestureState::Failed;
                        self.reset();
                    }
                }
                None
            }
            PointerPhase::Cancelled => {
                self.state = GestureState::Cancelled;
                self.reset();
                None
            }
        }
    }

    fn reset(&mut self) {
        self.first_tap_position = None;
        self.first_tap_time = None;
        self.current_tap_start = None;
        self.current_tap_time = None;
        self.pointer_id = None;
        self.tap_count = 0;
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        20 // Higher priority than single tap
    }
}

/// Configuration for long press gesture recognition
#[derive(Debug, Clone)]
pub struct LongPressConfig {
    /// Duration required for a long press (in milliseconds)
    pub min_duration_ms: u64,
    /// Maximum movement allowed during long press
    pub movement_tolerance: f32,
}

impl Default for LongPressConfig {
    fn default() -> Self {
        Self {
            min_duration_ms: 500,
            movement_tolerance: 10.0,
        }
    }
}

/// Recognizes long press gestures
#[derive(Debug)]
pub struct LongPressGesture {
    id: GestureRecognizerId,
    config: LongPressConfig,
    state: GestureState,
    start_position: Option<Point>,
    start_time: Option<Instant>,
    pointer_id: Option<u64>,
    recognized: bool,
}

impl LongPressGesture {
    /// Create a new long press gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: LongPressConfig::default(),
            state: GestureState::Possible,
            start_position: None,
            start_time: None,
            pointer_id: None,
            recognized: false,
        }
    }

    /// Create with custom config
    pub fn with_config(config: LongPressConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set the duration threshold
    pub fn duration(mut self, duration_ms: u64) -> Self {
        self.config.min_duration_ms = duration_ms;
        self
    }

    /// Check if enough time has passed for long press
    /// This should be called periodically (e.g., in an event loop)
    pub fn check_timeout(&mut self) -> Option<GestureEvent> {
        if self.recognized || self.state != GestureState::Possible {
            return None;
        }

        if let (Some(start_pos), Some(start_time)) = (self.start_position, self.start_time) {
            let duration = Instant::now().duration_since(start_time);
            if duration >= Duration::from_millis(self.config.min_duration_ms) {
                self.state = GestureState::Began;
                self.recognized = true;
                return Some(GestureEvent::new(
                    GestureType::LongPress,
                    GestureState::Began,
                    start_pos,
                ));
            }
        }
        None
    }
}

impl Default for LongPressGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for LongPressGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::LongPress
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
                    self.state = GestureState::Possible;
                    self.recognized = false;
                }
                None
            }
            PointerPhase::Moved => {
                if self.pointer_id == Some(event.pointer_id) {
                    if let Some(start) = self.start_position {
                        let distance = start.distance_to(&event.position);
                        if distance > self.config.movement_tolerance {
                            self.state = GestureState::Failed;
                            self.reset();
                        }
                    }
                }
                None
            }
            PointerPhase::Ended => {
                if self.pointer_id == Some(event.pointer_id) {
                    if self.recognized {
                        self.state = GestureState::Ended;
                        let gesture_event = GestureEvent::new(
                            GestureType::LongPress,
                            GestureState::Ended,
                            event.position,
                        );
                        self.reset();
                        return Some(gesture_event);
                    }
                    self.state = GestureState::Failed;
                    self.reset();
                }
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
        self.state = GestureState::Possible;
        self.recognized = false;
    }

    fn priority(&self) -> i32 {
        15
    }
}

/// Configuration for press and hold gesture
#[derive(Debug, Clone)]
pub struct PressAndHoldConfig {
    /// Minimum duration before recognizing as press and hold
    pub min_duration_ms: u64,
    /// Maximum movement allowed
    pub movement_tolerance: f32,
    /// Interval for emitting "holding" events
    pub hold_interval_ms: u64,
}

impl Default for PressAndHoldConfig {
    fn default() -> Self {
        Self {
            min_duration_ms: 150,
            movement_tolerance: 10.0,
            hold_interval_ms: 100,
        }
    }
}

/// Recognizes press and hold gestures (continuous while held)
#[derive(Debug)]
pub struct PressAndHoldGesture {
    id: GestureRecognizerId,
    config: PressAndHoldConfig,
    state: GestureState,
    start_position: Option<Point>,
    start_time: Option<Instant>,
    last_hold_event: Option<Instant>,
    pointer_id: Option<u64>,
}

impl PressAndHoldGesture {
    /// Create a new press and hold gesture recognizer
    pub fn new() -> Self {
        Self {
            id: GestureRecognizerId::new(),
            config: PressAndHoldConfig::default(),
            state: GestureState::Possible,
            start_position: None,
            start_time: None,
            last_hold_event: None,
            pointer_id: None,
        }
    }

    /// Create with custom config
    pub fn with_config(config: PressAndHoldConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Check for hold events (should be called periodically)
    pub fn check_hold(&mut self) -> Option<GestureEvent> {
        if self.state != GestureState::Began && self.state != GestureState::Changed {
            // Check if we should begin
            if let (Some(start_pos), Some(start_time)) = (self.start_position, self.start_time) {
                let now = Instant::now();
                let duration = now.duration_since(start_time);
                if duration >= Duration::from_millis(self.config.min_duration_ms) {
                    self.state = GestureState::Began;
                    self.last_hold_event = Some(now);
                    return Some(GestureEvent::new(
                        GestureType::PressAndHold,
                        GestureState::Began,
                        start_pos,
                    ));
                }
            }
            return None;
        }

        // Already began, check for interval events
        if let (Some(pos), Some(last)) = (self.start_position, self.last_hold_event) {
            let now = Instant::now();
            let since_last = now.duration_since(last);
            if since_last >= Duration::from_millis(self.config.hold_interval_ms) {
                self.last_hold_event = Some(now);
                self.state = GestureState::Changed;
                return Some(GestureEvent::new(
                    GestureType::PressAndHold,
                    GestureState::Changed,
                    pos,
                ));
            }
        }
        None
    }
}

impl Default for PressAndHoldGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer for PressAndHoldGesture {
    fn id(&self) -> GestureRecognizerId {
        self.id
    }

    fn gesture_type(&self) -> GestureType {
        GestureType::PressAndHold
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
                    self.state = GestureState::Possible;
                }
                None
            }
            PointerPhase::Moved => {
                if self.pointer_id == Some(event.pointer_id) {
                    if let Some(start) = self.start_position {
                        let distance = start.distance_to(&event.position);
                        if distance > self.config.movement_tolerance {
                            let was_active = self.state.is_active();
                            self.state = GestureState::Cancelled;
                            if was_active {
                                let result = Some(GestureEvent::new(
                                    GestureType::PressAndHold,
                                    GestureState::Cancelled,
                                    event.position,
                                ));
                                self.reset();
                                return result;
                            }
                            self.reset();
                        }
                    }
                }
                None
            }
            PointerPhase::Ended => {
                if self.pointer_id == Some(event.pointer_id) {
                    let was_active = self.state.is_active();
                    self.state = GestureState::Ended;
                    if was_active {
                        let result = Some(GestureEvent::new(
                            GestureType::PressAndHold,
                            GestureState::Ended,
                            event.position,
                        ));
                        self.reset();
                        return result;
                    }
                    self.reset();
                }
                None
            }
            PointerPhase::Cancelled => {
                if self.pointer_id == Some(event.pointer_id) {
                    let was_active = self.state.is_active();
                    self.state = GestureState::Cancelled;
                    if was_active {
                        let result = Some(GestureEvent::new(
                            GestureType::PressAndHold,
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
        self.start_time = None;
        self.last_hold_event = None;
        self.pointer_id = None;
        self.state = GestureState::Possible;
    }

    fn priority(&self) -> i32 {
        12
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
    fn test_tap_gesture_success() {
        let mut tap = TapGesture::new();

        let down = make_down(1, 100.0, 100.0);
        assert!(tap.handle_event(&down).is_none());

        let up = make_up(1, 100.0, 100.0);
        let result = tap.handle_event(&up);
        assert!(result.is_some());

        let event = result.unwrap();
        assert_eq!(event.gesture_type, GestureType::Tap);
        assert_eq!(event.state, GestureState::Ended);
    }

    #[test]
    fn test_tap_gesture_fails_on_movement() {
        let mut tap = TapGesture::new().movement_tolerance(10.0);

        let down = make_down(1, 100.0, 100.0);
        tap.handle_event(&down);

        let move_event = make_move(1, 150.0, 100.0); // 50px movement
        tap.handle_event(&move_event);

        let up = make_up(1, 150.0, 100.0);
        let result = tap.handle_event(&up);
        assert!(result.is_none());
    }

    #[test]
    fn test_tap_gesture_within_tolerance() {
        let mut tap = TapGesture::new().movement_tolerance(20.0);

        let down = make_down(1, 100.0, 100.0);
        tap.handle_event(&down);

        let move_event = make_move(1, 105.0, 105.0); // ~7px movement
        tap.handle_event(&move_event);

        let up = make_up(1, 105.0, 105.0);
        let result = tap.handle_event(&up);
        assert!(result.is_some());
    }

    #[test]
    fn test_double_tap_success() {
        let mut double_tap = DoubleTapGesture::new();

        // First tap
        let down1 = make_down(1, 100.0, 100.0);
        double_tap.handle_event(&down1);
        let up1 = make_up(1, 100.0, 100.0);
        assert!(double_tap.handle_event(&up1).is_none());

        // Second tap
        let down2 = make_down(1, 100.0, 100.0);
        double_tap.handle_event(&down2);
        let up2 = make_up(1, 100.0, 100.0);
        let result = double_tap.handle_event(&up2);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.gesture_type, GestureType::DoubleTap);
        assert_eq!(event.tap_count, 2);
    }

    #[test]
    fn test_double_tap_fails_on_distance() {
        let mut double_tap = DoubleTapGesture::with_config(DoubleTapConfig {
            max_distance: 30.0,
            ..Default::default()
        });

        // First tap
        let down1 = make_down(1, 100.0, 100.0);
        double_tap.handle_event(&down1);
        let up1 = make_up(1, 100.0, 100.0);
        double_tap.handle_event(&up1);

        // Second tap too far away
        let down2 = make_down(1, 200.0, 100.0); // 100px away
        double_tap.handle_event(&down2);
        let up2 = make_up(1, 200.0, 100.0);
        let result = double_tap.handle_event(&up2);

        assert!(result.is_none());
    }

    #[test]
    fn test_long_press_check_timeout() {
        let mut long_press = LongPressGesture::new().duration(100);

        let down = make_down(1, 100.0, 100.0);
        long_press.handle_event(&down);

        // Immediately after, not enough time
        assert!(long_press.check_timeout().is_none());

        // Simulate time passing by modifying start_time
        long_press.start_time = Some(Instant::now() - Duration::from_millis(150));

        let result = long_press.check_timeout();
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.gesture_type, GestureType::LongPress);
        assert_eq!(event.state, GestureState::Began);
    }

    #[test]
    fn test_long_press_fails_on_movement() {
        let mut long_press = LongPressGesture::new();

        let down = make_down(1, 100.0, 100.0);
        long_press.handle_event(&down);

        let move_event = make_move(1, 150.0, 100.0);
        long_press.handle_event(&move_event);

        assert_eq!(long_press.state(), GestureState::Possible);
    }

    #[test]
    fn test_press_and_hold_lifecycle() {
        let mut press_hold = PressAndHoldGesture::with_config(PressAndHoldConfig {
            min_duration_ms: 100,
            hold_interval_ms: 50,
            ..Default::default()
        });

        let down = make_down(1, 100.0, 100.0);
        press_hold.handle_event(&down);

        // Simulate time passing
        press_hold.start_time = Some(Instant::now() - Duration::from_millis(150));

        // Should begin
        let result = press_hold.check_hold();
        assert!(result.is_some());
        assert_eq!(result.unwrap().state, GestureState::Began);

        // Simulate more time
        press_hold.last_hold_event = Some(Instant::now() - Duration::from_millis(60));

        // Should get changed event
        let result2 = press_hold.check_hold();
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().state, GestureState::Changed);

        // End
        let up = make_up(1, 100.0, 100.0);
        let end_result = press_hold.handle_event(&up);
        assert!(end_result.is_some());
        assert_eq!(end_result.unwrap().state, GestureState::Ended);
    }

    #[test]
    fn test_tap_ignores_other_pointers() {
        let mut tap = TapGesture::new();

        let down1 = make_down(1, 100.0, 100.0);
        tap.handle_event(&down1);

        // Different pointer
        let down2 = make_down(2, 200.0, 200.0);
        tap.handle_event(&down2);
        let up2 = make_up(2, 200.0, 200.0);
        assert!(tap.handle_event(&up2).is_none());

        // Original pointer should still work
        let up1 = make_up(1, 100.0, 100.0);
        assert!(tap.handle_event(&up1).is_some());
    }

    #[test]
    fn test_tap_config_customization() {
        let config = TapConfig {
            movement_tolerance: 20.0,
            max_duration_ms: 500,
        };
        let tap = TapGesture::with_config(config.clone());
        assert_eq!(tap.config.movement_tolerance, 20.0);
        assert_eq!(tap.config.max_duration_ms, 500);
    }

    #[test]
    fn test_gesture_reset() {
        let mut tap = TapGesture::new();

        let down = make_down(1, 100.0, 100.0);
        tap.handle_event(&down);

        tap.reset();

        assert_eq!(tap.state(), GestureState::Possible);
        assert!(tap.start_position.is_none());
        assert!(tap.pointer_id.is_none());
    }
}
