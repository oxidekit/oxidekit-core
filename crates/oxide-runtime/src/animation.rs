//! Animation Runtime Integration
//!
//! Provides animation support for the OxideKit runtime, integrating the
//! animation system with the render loop and layout tree.

use oxide_components::animation::{
    AnimationController, AnimationEvent, AnimationId, AnimatableValue,
    Animation, Easing, Timeline, TransitionConfig, Transition,
};
use oxide_components::theme::MotionTokens;
use std::collections::HashMap;
use std::time::Instant;

/// Animation runtime state
pub struct AnimationRuntime {
    /// The animation controller
    controller: AnimationController,

    /// Last update time for delta calculation
    last_update: Instant,

    /// Whether animations are enabled
    enabled: bool,

    /// Callbacks for animation events
    event_handlers: Vec<Box<dyn Fn(&AnimationEvent) + Send + Sync>>,

    /// Element animations by element ID
    element_animations: HashMap<String, Vec<AnimationId>>,

    /// Motion tokens from theme
    motion_tokens: Option<MotionTokens>,
}

impl std::fmt::Debug for AnimationRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationRuntime")
            .field("enabled", &self.enabled)
            .field("element_animations", &self.element_animations)
            .field("motion_tokens", &self.motion_tokens)
            .field("event_handlers_count", &self.event_handlers.len())
            .finish()
    }
}

impl Default for AnimationRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationRuntime {
    /// Create a new animation runtime
    pub fn new() -> Self {
        Self {
            controller: AnimationController::new(),
            last_update: Instant::now(),
            enabled: true,
            event_handlers: Vec::new(),
            element_animations: HashMap::new(),
            motion_tokens: None,
        }
    }

    /// Create with motion tokens from theme
    pub fn with_motion_tokens(mut self, tokens: MotionTokens) -> Self {
        self.motion_tokens = Some(tokens.clone());
        self.controller = AnimationController::new().with_motion_tokens(tokens);
        self
    }

    /// Set whether animations are enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.controller.pause_all();
        } else {
            self.controller.resume_all();
        }
    }

    /// Check if animations are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add an animation for an element
    pub fn animate_element(
        &mut self,
        element_id: &str,
        animation: Animation,
    ) -> AnimationId {
        let id = self.controller.add(animation);
        self.controller.play(id);

        self.element_animations
            .entry(element_id.to_string())
            .or_default()
            .push(id);

        id
    }

    /// Animate multiple properties on an element with a timeline
    pub fn animate_element_timeline(
        &mut self,
        element_id: &str,
        timeline: Timeline,
    ) -> AnimationId {
        let id = self.controller.add_timeline(timeline);
        self.controller.play(id);

        self.element_animations
            .entry(element_id.to_string())
            .or_default()
            .push(id);

        id
    }

    /// Configure transitions for an element
    pub fn configure_transitions(
        &mut self,
        element_id: &str,
        transitions: Vec<Transition>,
    ) {
        self.controller.configure_transitions(element_id, |mut m| {
            for t in transitions {
                m.apply_transition(&t);
            }
            m
        });
    }

    /// Set a property value on an element (triggers transition if configured)
    pub fn set_property(
        &mut self,
        element_id: &str,
        property: &str,
        value: AnimatableValue,
    ) -> AnimatableValue {
        self.controller.set_property(element_id, property, value)
    }

    /// Get current property value for an element
    pub fn get_property(&self, element_id: &str, property: &str) -> Option<AnimatableValue> {
        self.controller.get_property(element_id, property)
    }

    /// Get all current values for an element's animations
    pub fn get_element_values(&self, element_id: &str) -> HashMap<String, AnimatableValue> {
        let mut values = HashMap::new();

        if let Some(ids) = self.element_animations.get(element_id) {
            for &id in ids {
                if let Some(_value) = self.controller.get_value(id) {
                    // Note: To properly map values, we'd need to track property names
                    // This is handled better through timeline values below
                }
                if let Some(timeline_values) = self.controller.get_timeline_values(id) {
                    values.extend(timeline_values);
                }
            }
        }

        values
    }

    /// Cancel all animations for an element
    pub fn cancel_element(&mut self, element_id: &str) {
        if let Some(ids) = self.element_animations.remove(element_id) {
            for id in ids {
                self.controller.stop(id);
            }
        }
    }

    /// Update animations (call each frame)
    /// Returns events that occurred during this update
    pub fn update(&mut self) -> Vec<AnimationEvent> {
        if !self.enabled {
            return Vec::new();
        }

        let now = Instant::now();
        let dt = (now - self.last_update).as_secs_f32();
        self.last_update = now;

        let events = self.controller.update(dt);

        // Call event handlers
        for event in &events {
            for handler in &self.event_handlers {
                handler(event);
            }
        }

        // Clean up completed element animations
        for event in &events {
            match event {
                AnimationEvent::Completed { id }
                | AnimationEvent::Cancelled { id }
                | AnimationEvent::TimelineCompleted { id } => {
                    for ids in self.element_animations.values_mut() {
                        ids.retain(|i| i != id);
                    }
                }
                _ => {}
            }
        }

        events
    }

    /// Update with explicit delta time (useful for fixed timestep)
    pub fn update_with_dt(&mut self, dt: f32) -> Vec<AnimationEvent> {
        if !self.enabled {
            return Vec::new();
        }

        self.last_update = Instant::now();
        let events = self.controller.update(dt);

        // Call event handlers
        for event in &events {
            for handler in &self.event_handlers {
                handler(event);
            }
        }

        events
    }

    /// Check if there are any active animations
    pub fn has_active_animations(&self) -> bool {
        self.controller.has_active()
    }

    /// Get the count of active animations
    pub fn active_count(&self) -> usize {
        self.controller.active_count()
    }

    /// Get all current animation values
    pub fn all_values(&self) -> HashMap<String, AnimatableValue> {
        self.controller.all_values()
    }

    /// Add an event handler
    pub fn on_event<F>(&mut self, handler: F)
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }

    /// Create a fade-in animation
    pub fn fade_in(&mut self, element_id: &str, duration_ms: u32) -> AnimationId {
        let duration = self.get_duration(duration_ms);
        let anim = Animation::new("opacity")
            .from(0.0_f32)
            .to(1.0_f32)
            .duration(duration)
            .easing(Easing::EaseOut)
            .build();
        self.animate_element(element_id, anim)
    }

    /// Create a fade-out animation
    pub fn fade_out(&mut self, element_id: &str, duration_ms: u32) -> AnimationId {
        let duration = self.get_duration(duration_ms);
        let anim = Animation::new("opacity")
            .from(1.0_f32)
            .to(0.0_f32)
            .duration(duration)
            .easing(Easing::EaseIn)
            .build();
        self.animate_element(element_id, anim)
    }

    /// Create a slide-in animation from a direction
    pub fn slide_in(
        &mut self,
        element_id: &str,
        from_x: f32,
        from_y: f32,
        duration_ms: u32,
    ) -> AnimationId {
        use oxide_components::animation::interpolate::Point2D;

        let duration = self.get_duration(duration_ms);
        let anim = Animation::new("position")
            .from(Point2D::new(from_x, from_y))
            .to(Point2D::new(0.0, 0.0))
            .duration(duration)
            .easing(Easing::EaseOutCubic)
            .build();
        self.animate_element(element_id, anim)
    }

    /// Create a scale animation
    pub fn scale(
        &mut self,
        element_id: &str,
        from: f32,
        to: f32,
        duration_ms: u32,
    ) -> AnimationId {
        let duration = self.get_duration(duration_ms);
        let anim = Animation::new("scale")
            .from(from)
            .to(to)
            .duration(duration)
            .easing(Easing::EaseInOutCubic)
            .build();
        self.animate_element(element_id, anim)
    }

    /// Create a color transition animation
    pub fn color_transition(
        &mut self,
        element_id: &str,
        property: &str,
        from: [f32; 4],
        to: [f32; 4],
        duration_ms: u32,
    ) -> AnimationId {
        let duration = self.get_duration(duration_ms);
        let anim = Animation::new(property)
            .from(from)
            .to(to)
            .duration(duration)
            .easing(Easing::EaseInOut)
            .build();
        self.animate_element(element_id, anim)
    }

    /// Get duration in seconds, using motion tokens if available
    fn get_duration(&self, default_ms: u32) -> f32 {
        default_ms as f32 / 1000.0
    }

    /// Get a default transition config based on speed name
    pub fn get_transition_config(&self, speed: &str) -> TransitionConfig {
        self.controller.default_transition_config(speed)
    }

    /// Access the underlying controller
    pub fn controller(&self) -> &AnimationController {
        &self.controller
    }

    /// Access the underlying controller mutably
    pub fn controller_mut(&mut self) -> &mut AnimationController {
        &mut self.controller
    }
}

/// Trait for elements that can be animated
pub trait Animatable {
    /// Get the element ID for animation tracking
    fn animation_id(&self) -> &str;

    /// Get animatable properties and their current values
    fn get_animatable_properties(&self) -> HashMap<String, AnimatableValue>;

    /// Apply animated property values
    fn apply_animated_values(&mut self, values: &HashMap<String, AnimatableValue>);
}

/// Common animatable property names
pub mod properties {
    /// Opacity (0.0 to 1.0)
    pub const OPACITY: &str = "opacity";

    /// X position
    pub const X: &str = "x";

    /// Y position
    pub const Y: &str = "y";

    /// Position (Point2D)
    pub const POSITION: &str = "position";

    /// Width
    pub const WIDTH: &str = "width";

    /// Height
    pub const HEIGHT: &str = "height";

    /// Size (Size2D)
    pub const SIZE: &str = "size";

    /// Uniform scale
    pub const SCALE: &str = "scale";

    /// X scale
    pub const SCALE_X: &str = "scale_x";

    /// Y scale
    pub const SCALE_Y: &str = "scale_y";

    /// Rotation (radians)
    pub const ROTATION: &str = "rotation";

    /// Background color
    pub const BACKGROUND: &str = "background";

    /// Border color
    pub const BORDER_COLOR: &str = "border_color";

    /// Border width
    pub const BORDER_WIDTH: &str = "border_width";

    /// Corner radius
    pub const RADIUS: &str = "radius";

    /// Transform (full Transform2D)
    pub const TRANSFORM: &str = "transform";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_runtime() {
        let mut runtime = AnimationRuntime::new();
        assert!(runtime.is_enabled());
        assert!(!runtime.has_active_animations());
    }

    #[test]
    fn test_fade_in() {
        let mut runtime = AnimationRuntime::new();
        let id = runtime.fade_in("button", 300);
        assert!(runtime.has_active_animations());
    }

    #[test]
    fn test_update() {
        let mut runtime = AnimationRuntime::new();
        runtime.fade_in("button", 100);

        // Update with delta time
        let events = runtime.update_with_dt(0.05);
        assert!(runtime.has_active_animations());

        // Complete the animation
        let events = runtime.update_with_dt(0.1);
        // Animation should complete
    }

    #[test]
    fn test_disable_animations() {
        let mut runtime = AnimationRuntime::new();
        runtime.set_enabled(false);
        assert!(!runtime.is_enabled());

        runtime.fade_in("button", 300);
        let events = runtime.update_with_dt(0.1);
        // Should return empty when disabled
        assert!(events.is_empty());
    }

    #[test]
    fn test_cancel_element() {
        let mut runtime = AnimationRuntime::new();
        runtime.fade_in("button", 300);
        assert!(runtime.has_active_animations());

        runtime.cancel_element("button");
        runtime.update_with_dt(0.0);
        assert!(!runtime.has_active_animations());
    }
}
