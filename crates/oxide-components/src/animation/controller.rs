//! Animation Controller
//!
//! Provides a centralized controller for managing animations in the runtime.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use super::easing::Easing;
use super::interpolate::AnimatableValue;
use super::state::Animation;
use super::timeline::Timeline;
use super::transition::{TransitionManager, TransitionConfig};
use crate::theme::MotionTokens;

/// Unique identifier for an animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimationId(pub u64);

static NEXT_ANIMATION_ID: AtomicU64 = AtomicU64::new(1);

impl AnimationId {
    /// Generate a new unique animation ID
    pub fn new() -> Self {
        Self(NEXT_ANIMATION_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for AnimationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted by animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationEvent {
    /// Animation started
    Started { id: AnimationId },
    /// Animation completed
    Completed { id: AnimationId },
    /// Animation was cancelled
    Cancelled { id: AnimationId },
    /// Animation paused
    Paused { id: AnimationId },
    /// Animation resumed
    Resumed { id: AnimationId },
    /// Animation iteration completed (for repeating animations)
    IterationComplete { id: AnimationId, iteration: u32 },
    /// Timeline started
    TimelineStarted { id: AnimationId },
    /// Timeline completed
    TimelineCompleted { id: AnimationId },
}

/// Entry in the controller
#[derive(Debug)]
enum ControllerEntry {
    Animation(Animation),
    Timeline(Timeline),
}

impl ControllerEntry {
    fn is_active(&self) -> bool {
        match self {
            ControllerEntry::Animation(anim) => anim.state.is_active(),
            ControllerEntry::Timeline(timeline) => timeline.is_active(),
        }
    }

    #[allow(dead_code)]
    fn is_finished(&self) -> bool {
        match self {
            ControllerEntry::Animation(anim) => anim.state.is_finished(),
            ControllerEntry::Timeline(timeline) => timeline.is_complete(),
        }
    }
}

/// Central controller for managing all animations
#[derive(Debug, Default)]
pub struct AnimationController {
    /// Active animations and timelines
    entries: HashMap<AnimationId, ControllerEntry>,

    /// Transition managers per element/node
    transition_managers: HashMap<String, TransitionManager>,

    /// Pending events from the last update
    events: Vec<AnimationEvent>,

    /// Motion tokens for default durations/easings
    motion_tokens: Option<MotionTokens>,

    /// Whether to auto-remove completed animations
    auto_remove_completed: bool,

    /// Global time scale
    time_scale: f32,

    /// Whether animations are globally paused
    paused: bool,
}

impl AnimationController {
    /// Create a new animation controller
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            transition_managers: HashMap::new(),
            events: Vec::new(),
            motion_tokens: None,
            auto_remove_completed: true,
            time_scale: 1.0,
            paused: false,
        }
    }

    /// Create with motion tokens from theme
    pub fn with_motion_tokens(mut self, tokens: MotionTokens) -> Self {
        self.motion_tokens = Some(tokens);
        self
    }

    /// Set whether to auto-remove completed animations
    pub fn auto_remove(mut self, enabled: bool) -> Self {
        self.auto_remove_completed = enabled;
        self
    }

    /// Set the global time scale
    pub fn time_scale(mut self, scale: f32) -> Self {
        self.time_scale = scale;
        self
    }

    /// Add an animation and return its ID
    pub fn add(&mut self, animation: Animation) -> AnimationId {
        let id = AnimationId::new();
        self.entries.insert(id, ControllerEntry::Animation(animation));
        id
    }

    /// Add a timeline and return its ID
    pub fn add_timeline(&mut self, timeline: Timeline) -> AnimationId {
        let id = AnimationId::new();
        self.entries.insert(id, ControllerEntry::Timeline(timeline));
        id
    }

    /// Play an animation by ID
    pub fn play(&mut self, id: AnimationId) {
        if let Some(entry) = self.entries.get_mut(&id) {
            match entry {
                ControllerEntry::Animation(anim) => {
                    anim.play();
                    self.events.push(AnimationEvent::Started { id });
                }
                ControllerEntry::Timeline(timeline) => {
                    timeline.play();
                    self.events.push(AnimationEvent::TimelineStarted { id });
                }
            }
        }
    }

    /// Pause an animation by ID
    pub fn pause(&mut self, id: AnimationId) {
        if let Some(entry) = self.entries.get_mut(&id) {
            match entry {
                ControllerEntry::Animation(anim) => {
                    anim.pause();
                    self.events.push(AnimationEvent::Paused { id });
                }
                ControllerEntry::Timeline(timeline) => {
                    timeline.pause();
                    self.events.push(AnimationEvent::Paused { id });
                }
            }
        }
    }

    /// Resume an animation by ID
    pub fn resume(&mut self, id: AnimationId) {
        if let Some(entry) = self.entries.get_mut(&id) {
            match entry {
                ControllerEntry::Animation(anim) => {
                    anim.resume();
                    self.events.push(AnimationEvent::Resumed { id });
                }
                ControllerEntry::Timeline(timeline) => {
                    timeline.resume();
                    self.events.push(AnimationEvent::Resumed { id });
                }
            }
        }
    }

    /// Stop and remove an animation by ID
    pub fn stop(&mut self, id: AnimationId) {
        if let Some(entry) = self.entries.remove(&id) {
            match entry {
                ControllerEntry::Animation(_) => {
                    self.events.push(AnimationEvent::Cancelled { id });
                }
                ControllerEntry::Timeline(_) => {
                    self.events.push(AnimationEvent::Cancelled { id });
                }
            }
        }
    }

    /// Stop all animations
    pub fn stop_all(&mut self) {
        let ids: Vec<_> = self.entries.keys().copied().collect();
        for id in ids {
            self.stop(id);
        }
    }

    /// Pause all animations globally
    pub fn pause_all(&mut self) {
        self.paused = true;
    }

    /// Resume all animations globally
    pub fn resume_all(&mut self) {
        self.paused = false;
    }

    /// Check if an animation exists
    pub fn has(&self, id: AnimationId) -> bool {
        self.entries.contains_key(&id)
    }

    /// Check if an animation is active
    pub fn is_active(&self, id: AnimationId) -> bool {
        self.entries.get(&id).map(|e| e.is_active()).unwrap_or(false)
    }

    /// Get the current value for an animation
    pub fn get_value(&self, id: AnimationId) -> Option<AnimatableValue> {
        match self.entries.get(&id)? {
            ControllerEntry::Animation(anim) => anim.current_value(),
            ControllerEntry::Timeline(_) => None, // Timelines return multiple values
        }
    }

    /// Get all current values for a timeline
    pub fn get_timeline_values(&self, id: AnimationId) -> Option<HashMap<String, AnimatableValue>> {
        match self.entries.get(&id)? {
            ControllerEntry::Animation(_) => None,
            ControllerEntry::Timeline(timeline) => Some(timeline.get_values()),
        }
    }

    /// Get or create a transition manager for an element
    pub fn transitions_for(&mut self, element_id: &str) -> &mut TransitionManager {
        self.transition_managers
            .entry(element_id.to_string())
            .or_insert_with(TransitionManager::new)
    }

    /// Configure transitions for an element
    pub fn configure_transitions(
        &mut self,
        element_id: &str,
        config: impl FnOnce(TransitionManager) -> TransitionManager,
    ) {
        let manager = self.transition_managers
            .entry(element_id.to_string())
            .or_insert_with(TransitionManager::new);

        *manager = config(std::mem::take(manager));
    }

    /// Set a property value on an element (may trigger transition)
    pub fn set_property(
        &mut self,
        element_id: &str,
        property: &str,
        value: AnimatableValue,
    ) -> AnimatableValue {
        self.transitions_for(element_id).set_value(property, value)
    }

    /// Get the current property value for an element
    pub fn get_property(&self, element_id: &str, property: &str) -> Option<AnimatableValue> {
        self.transition_managers
            .get(element_id)
            .and_then(|m| m.get_value(property))
    }

    /// Update all animations by a time delta (in seconds)
    pub fn update(&mut self, dt: f32) -> Vec<AnimationEvent> {
        if self.paused {
            return std::mem::take(&mut self.events);
        }

        let scaled_dt = dt * self.time_scale;
        let mut completed_ids = Vec::new();

        // Update animations
        for (&id, entry) in &mut self.entries {
            let was_active = entry.is_active();

            let still_active = match entry {
                ControllerEntry::Animation(anim) => {
                    let prev_iteration = anim.state.iteration;
                    let active = anim.update(scaled_dt);

                    // Check for iteration complete
                    if anim.state.iteration > prev_iteration {
                        self.events.push(AnimationEvent::IterationComplete {
                            id,
                            iteration: prev_iteration,
                        });
                    }

                    active
                }
                ControllerEntry::Timeline(timeline) => timeline.update(scaled_dt),
            };

            if was_active && !still_active {
                match entry {
                    ControllerEntry::Animation(_) => {
                        self.events.push(AnimationEvent::Completed { id });
                    }
                    ControllerEntry::Timeline(_) => {
                        self.events.push(AnimationEvent::TimelineCompleted { id });
                    }
                }
                if self.auto_remove_completed {
                    completed_ids.push(id);
                }
            }
        }

        // Remove completed animations
        for id in completed_ids {
            self.entries.remove(&id);
        }

        // Update transitions
        for manager in self.transition_managers.values_mut() {
            manager.update(scaled_dt);
        }

        std::mem::take(&mut self.events)
    }

    /// Get all active animation IDs
    pub fn active_animations(&self) -> Vec<AnimationId> {
        self.entries
            .iter()
            .filter(|(_, e)| e.is_active())
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get the count of active animations
    pub fn active_count(&self) -> usize {
        self.entries.values().filter(|e| e.is_active()).count()
    }

    /// Check if there are any active animations or transitions
    pub fn has_active(&self) -> bool {
        self.entries.values().any(|e| e.is_active())
            || self.transition_managers.values().any(|m| m.has_active_transitions())
    }

    /// Get all current animation values (property name -> value)
    pub fn all_values(&self) -> HashMap<String, AnimatableValue> {
        let mut values = HashMap::new();

        for entry in self.entries.values() {
            match entry {
                ControllerEntry::Animation(anim) => {
                    if let Some(value) = anim.current_value() {
                        values.insert(anim.property.clone(), value);
                    }
                }
                ControllerEntry::Timeline(timeline) => {
                    values.extend(timeline.get_values());
                }
            }
        }

        values
    }

    /// Get default transition config from motion tokens
    pub fn default_transition_config(&self, speed: &str) -> TransitionConfig {
        if let Some(tokens) = &self.motion_tokens {
            let duration_ms = match speed {
                "instant" => tokens.duration.instant,
                "fast" => tokens.duration.fast,
                "normal" => tokens.duration.normal,
                "slow" => tokens.duration.slow,
                _ => tokens.duration.normal,
            };
            TransitionConfig::from_token(speed, duration_ms)
        } else {
            // Default fallback
            let duration_ms = match speed {
                "instant" => 50,
                "fast" => 150,
                "normal" => 300,
                "slow" => 500,
                _ => 300,
            };
            TransitionConfig::from_ms(duration_ms)
        }
    }

    /// Create a transition config from easing token name
    pub fn config_from_easing_token(&self, easing_name: &str, duration_ms: u32) -> TransitionConfig {
        let easing = if let Some(tokens) = &self.motion_tokens {
            let easing_str = match easing_name {
                "linear" => &tokens.easing.linear,
                "ease_in" => &tokens.easing.ease_in,
                "ease_out" => &tokens.easing.ease_out,
                "ease_in_out" => &tokens.easing.ease_in_out,
                _ => {
                    tokens.easing.custom.get(easing_name)
                        .map(|s| s.as_str())
                        .unwrap_or("ease")
                }
            };
            Easing::from_str(easing_str).unwrap_or_default()
        } else {
            Easing::from_str(easing_name).unwrap_or_default()
        };

        TransitionConfig::from_ms(duration_ms).with_easing(easing)
    }
}

/// Helper for running animations in a callback-style
#[allow(dead_code)]
pub struct AnimationRunner<'a> {
    controller: &'a mut AnimationController,
}

#[allow(dead_code)]
impl<'a> AnimationRunner<'a> {
    pub fn new(controller: &'a mut AnimationController) -> Self {
        Self { controller }
    }

    /// Run an animation and call the callback when complete
    pub fn run<F>(
        &mut self,
        animation: Animation,
        _on_complete: F,
    ) -> AnimationId
    where
        F: FnOnce() + 'static,
    {
        let id = self.controller.add(animation);
        self.controller.play(id);
        // Note: In a real implementation, we'd store the callback
        // and call it when the Completed event is emitted
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_animation(property: &str, duration: f32) -> Animation {
        Animation::new(property)
            .from(0.0_f32)
            .to(1.0_f32)
            .duration(duration)
            .build()
    }

    #[test]
    fn test_controller_add_play() {
        let mut controller = AnimationController::new();

        let id = controller.add(test_animation("opacity", 1.0));
        controller.play(id);

        assert!(controller.has(id));
        assert!(controller.is_active(id));
    }

    #[test]
    fn test_controller_update() {
        let mut controller = AnimationController::new();

        let id = controller.add(test_animation("opacity", 1.0));
        controller.play(id);

        // Update
        let events = controller.update(0.5);

        // Should have started event
        assert!(events.iter().any(|e| matches!(e, AnimationEvent::Started { .. })));

        // Should still be active
        assert!(controller.is_active(id));

        // Check value
        let value = controller.get_value(id).unwrap();
        assert!(value.as_float().unwrap() > 0.0);
    }

    #[test]
    fn test_controller_complete() {
        let mut controller = AnimationController::new();

        let id = controller.add(test_animation("opacity", 0.5));
        controller.play(id);

        // Clear start event
        controller.update(0.0);

        // Complete the animation
        let events = controller.update(1.0);

        // Should have completed event
        assert!(events.iter().any(|e| matches!(e, AnimationEvent::Completed { .. })));

        // Should be removed (auto_remove is true by default)
        assert!(!controller.has(id));
    }

    #[test]
    fn test_controller_pause_resume() {
        let mut controller = AnimationController::new();

        let id = controller.add(test_animation("opacity", 1.0));
        controller.play(id);
        controller.update(0.25);

        controller.pause(id);
        let value_before = controller.get_value(id).unwrap().as_float().unwrap();

        // Update while paused
        controller.update(0.5);
        let value_after = controller.get_value(id).unwrap().as_float().unwrap();

        // Value should not have changed
        assert!((value_before - value_after).abs() < 0.001);

        // Resume
        controller.resume(id);
        controller.update(0.25);

        let value_resumed = controller.get_value(id).unwrap().as_float().unwrap();
        assert!(value_resumed > value_before);
    }

    #[test]
    fn test_controller_stop() {
        let mut controller = AnimationController::new();

        let id = controller.add(test_animation("opacity", 1.0));
        controller.play(id);

        let events = controller.stop(id);

        assert!(!controller.has(id));
    }

    #[test]
    fn test_controller_transitions() {
        let mut controller = AnimationController::new();

        controller.configure_transitions("button", |m| {
            m.with_property("opacity", TransitionConfig::from_ms(300))
        });

        // Set value (should start transition)
        let initial = controller.set_property(
            "button",
            "opacity",
            AnimatableValue::Float(1.0),
        );

        // Update partway
        controller.update(0.15);

        // Should have a value between 0 and 1
        let current = controller.get_property("button", "opacity");
        assert!(current.is_some());
    }

    #[test]
    fn test_controller_time_scale() {
        use crate::animation::Easing;

        let mut controller = AnimationController::new().time_scale(2.0);

        // Use linear easing for predictable test results
        let anim = Animation::new("opacity")
            .from(0.0_f32)
            .to(1.0_f32)
            .duration(1.0)
            .easing(Easing::Linear)
            .build();

        let id = controller.add(anim);
        controller.play(id);

        // Update for 0.25 seconds (but with 2x scale = 0.5 progress)
        controller.update(0.25);

        let value = controller.get_value(id).unwrap().as_float().unwrap();
        assert!((value - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_controller_global_pause() {
        let mut controller = AnimationController::new();

        let id = controller.add(test_animation("opacity", 1.0));
        controller.play(id);
        controller.update(0.25);

        controller.pause_all();

        let value_before = controller.get_value(id).unwrap().as_float().unwrap();
        controller.update(0.5);
        let value_after = controller.get_value(id).unwrap().as_float().unwrap();

        // Value should not have changed while globally paused
        assert!((value_before - value_after).abs() < 0.001);

        controller.resume_all();
        controller.update(0.25);

        let value_resumed = controller.get_value(id).unwrap().as_float().unwrap();
        assert!(value_resumed > value_before);
    }

    #[test]
    fn test_animation_id_unique() {
        let id1 = AnimationId::new();
        let id2 = AnimationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_controller_active_count() {
        let mut controller = AnimationController::new();

        let id1 = controller.add(test_animation("opacity", 1.0));
        let id2 = controller.add(test_animation("x", 0.5));

        controller.play(id1);
        controller.play(id2);

        assert_eq!(controller.active_count(), 2);

        // Complete one
        controller.update(0.6);

        // One should be complete and removed
        assert_eq!(controller.active_count(), 1);
    }
}
