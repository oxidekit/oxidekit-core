//! Touch and gesture input handling for mobile platforms.
//!
//! Provides a unified interface for handling touch input and gesture recognition
//! across iOS and Android platforms.
//!
//! ## Modules
//!
//! - [`touch`]: Low-level touch event handling
//! - [`gesture`]: High-level gesture recognition

pub mod touch;
pub mod gesture;

pub use touch::{TouchEvent, TouchPhase, TouchPoint, TouchId, TouchTracker};
pub use gesture::{
    GestureType, GestureState, GestureRecognizer, GestureEvent,
    TapRecognizer, PanRecognizer, PinchRecognizer, RotationRecognizer, SwipeRecognizer,
};

/// Hit test result indicating what was touched.
#[derive(Debug, Clone, PartialEq)]
pub struct HitTestResult {
    /// The element ID that was hit, if any.
    pub element_id: Option<String>,
    /// The position within the element (0.0-1.0).
    pub local_position: (f32, f32),
    /// The global position in screen coordinates.
    pub global_position: (f32, f32),
}

impl HitTestResult {
    /// Create a new hit test result.
    pub fn new(
        element_id: Option<String>,
        local_position: (f32, f32),
        global_position: (f32, f32),
    ) -> Self {
        Self {
            element_id,
            local_position,
            global_position,
        }
    }

    /// Create a hit test result with no hit.
    pub fn none(global_position: (f32, f32)) -> Self {
        Self {
            element_id: None,
            local_position: (0.0, 0.0),
            global_position,
        }
    }

    /// Returns true if an element was hit.
    pub fn hit(&self) -> bool {
        self.element_id.is_some()
    }
}

/// Input handler trait for processing touch and gesture events.
pub trait InputHandler {
    /// Handle a touch event.
    fn on_touch(&mut self, event: &TouchEvent);

    /// Handle a gesture event.
    fn on_gesture(&mut self, event: &GestureEvent);

    /// Perform hit testing at a position.
    fn hit_test(&self, x: f32, y: f32) -> HitTestResult;
}

/// Process touch events through gesture recognizers.
pub fn process_touch_through_recognizers(
    event: &TouchEvent,
    recognizers: &mut [Box<dyn GestureRecognizerTrait>],
) -> Vec<GestureEvent> {
    let mut events = Vec::new();

    for recognizer in recognizers.iter_mut() {
        if let Some(gesture_event) = recognizer.process_touch(event) {
            events.push(gesture_event);
        }
    }

    events
}

/// Trait for gesture recognizers.
pub trait GestureRecognizerTrait {
    /// Process a touch event and optionally produce a gesture event.
    fn process_touch(&mut self, event: &TouchEvent) -> Option<GestureEvent>;

    /// Reset the recognizer state.
    fn reset(&mut self);

    /// Get the current state of the recognizer.
    fn state(&self) -> GestureState;

    /// Get the gesture type this recognizer handles.
    fn gesture_type(&self) -> GestureType;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_test_result() {
        let hit = HitTestResult::new(Some("button1".into()), (0.5, 0.5), (100.0, 200.0));
        assert!(hit.hit());
        assert_eq!(hit.element_id, Some("button1".into()));

        let miss = HitTestResult::none((50.0, 50.0));
        assert!(!miss.hit());
        assert_eq!(miss.element_id, None);
    }
}
