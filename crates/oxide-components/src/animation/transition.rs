//! Transitions
//!
//! Provides transition definitions for smooth property changes.
//! Transitions automatically animate property changes when values are updated.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::easing::Easing;
use super::interpolate::AnimatableValue;
use super::state::{AnimationState, AnimationStatus};

/// Configuration for a single property transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionConfig {
    /// Duration in seconds
    pub duration: f32,

    /// Delay before starting (in seconds)
    #[serde(default)]
    pub delay: f32,

    /// Easing function
    #[serde(default)]
    pub easing: Easing,
}

impl TransitionConfig {
    /// Create a new transition config
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            delay: 0.0,
            easing: Easing::default(),
        }
    }

    /// Create from milliseconds
    pub fn from_ms(ms: u32) -> Self {
        Self::new(ms as f32 / 1000.0)
    }

    /// Set the easing function
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Set the delay
    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self
    }

    /// Set delay in milliseconds
    pub fn with_delay_ms(mut self, ms: u32) -> Self {
        self.delay = ms as f32 / 1000.0;
        self
    }

    /// Create from MotionTokens duration names
    pub fn from_token(token: &str, duration_ms: u32) -> Self {
        let duration = duration_ms as f32 / 1000.0;
        match token {
            "instant" => Self::new(duration).with_easing(Easing::Linear),
            "fast" => Self::new(duration).with_easing(Easing::EaseOut),
            "normal" => Self::new(duration).with_easing(Easing::EaseInOut),
            "slow" => Self::new(duration).with_easing(Easing::EaseInOut),
            _ => Self::new(duration),
        }
    }
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self::from_ms(300)
    }
}

/// A transition definition that can be applied to multiple properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Properties to transition (empty = all animatable properties)
    #[serde(default)]
    pub properties: Vec<String>,

    /// Transition configuration
    pub config: TransitionConfig,
}

impl Transition {
    /// Create a transition for all properties
    pub fn all(config: TransitionConfig) -> Self {
        Self {
            properties: Vec::new(),
            config,
        }
    }

    /// Create a transition for specific properties
    pub fn properties(properties: Vec<String>, config: TransitionConfig) -> Self {
        Self { properties, config }
    }

    /// Create a transition for a single property
    pub fn property(property: impl Into<String>, config: TransitionConfig) -> Self {
        Self {
            properties: vec![property.into()],
            config,
        }
    }

    /// Check if this transition applies to a property
    pub fn applies_to(&self, property: &str) -> bool {
        self.properties.is_empty() || self.properties.iter().any(|p| p == property)
    }

    /// Create common transitions
    pub fn opacity(duration_ms: u32) -> Self {
        Self::property("opacity", TransitionConfig::from_ms(duration_ms).with_easing(Easing::EaseOut))
    }

    pub fn transform(duration_ms: u32) -> Self {
        Self::property("transform", TransitionConfig::from_ms(duration_ms).with_easing(Easing::EaseOut))
    }

    pub fn background(duration_ms: u32) -> Self {
        Self::property("background", TransitionConfig::from_ms(duration_ms).with_easing(Easing::EaseInOut))
    }

    pub fn size(duration_ms: u32) -> Self {
        Self::properties(
            vec!["width".into(), "height".into()],
            TransitionConfig::from_ms(duration_ms).with_easing(Easing::EaseInOut),
        )
    }

    pub fn position(duration_ms: u32) -> Self {
        Self::properties(
            vec!["x".into(), "y".into()],
            TransitionConfig::from_ms(duration_ms).with_easing(Easing::EaseOut),
        )
    }
}

/// State for an active property transition
#[derive(Debug, Clone)]
pub struct PropertyTransition {
    /// The property being transitioned
    pub property: String,

    /// Starting value
    pub from: AnimatableValue,

    /// Target value
    pub to: AnimatableValue,

    /// Transition configuration
    pub config: TransitionConfig,

    /// Current state
    pub state: AnimationState,

    /// Time elapsed in the transition
    elapsed: f32,
}

impl PropertyTransition {
    /// Create a new property transition
    pub fn new(
        property: impl Into<String>,
        from: AnimatableValue,
        to: AnimatableValue,
        config: TransitionConfig,
    ) -> Self {
        Self {
            property: property.into(),
            from,
            to,
            config,
            state: AnimationState::default(),
            elapsed: 0.0,
        }
    }

    /// Start the transition
    pub fn start(&mut self) {
        self.elapsed = 0.0;
        self.state.status = if self.config.delay > 0.0 {
            AnimationStatus::Delayed
        } else {
            AnimationStatus::Running
        };
    }

    /// Update the transition by a time delta (in seconds)
    /// Returns true if still active
    pub fn update(&mut self, dt: f32) -> bool {
        match self.state.status {
            AnimationStatus::Idle => {
                self.start();
                true
            }
            AnimationStatus::Delayed => {
                self.elapsed += dt;
                if self.elapsed >= self.config.delay {
                    self.state.status = AnimationStatus::Running;
                    self.elapsed -= self.config.delay;
                }
                true
            }
            AnimationStatus::Running => {
                self.elapsed += dt;

                let progress = if self.config.duration > 0.0 {
                    (self.elapsed / self.config.duration).min(1.0)
                } else {
                    1.0
                };

                self.state.progress = progress;
                self.state.eased_progress = self.config.easing.evaluate(progress);
                self.state.remaining = (self.config.duration - self.elapsed).max(0.0);
                self.state.current_value = self.current_value();

                if self.elapsed >= self.config.duration {
                    self.state.status = AnimationStatus::Completed;
                    self.state.progress = 1.0;
                    self.state.eased_progress = 1.0;
                    false
                } else {
                    true
                }
            }
            AnimationStatus::Paused => true,
            AnimationStatus::Completed | AnimationStatus::Cancelled => false,
        }
    }

    /// Get the current interpolated value
    pub fn current_value(&self) -> Option<AnimatableValue> {
        self.from.interpolate(&self.to, self.state.eased_progress)
    }

    /// Check if the transition is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.state.status, AnimationStatus::Completed | AnimationStatus::Cancelled)
    }

    /// Check if the transition is active
    pub fn is_active(&self) -> bool {
        matches!(self.state.status, AnimationStatus::Running | AnimationStatus::Delayed | AnimationStatus::Idle)
    }

    /// Update the target value (for interrupted transitions)
    pub fn retarget(&mut self, new_target: AnimatableValue) {
        // Start from current value
        if let Some(current) = self.current_value() {
            self.from = current;
        }
        self.to = new_target;
        self.elapsed = 0.0;
        self.state = AnimationState::default();
        self.start();
    }
}

/// Manages transitions for multiple properties on a single element
#[derive(Debug, Clone, Default)]
pub struct TransitionManager {
    /// Active transitions by property name
    transitions: HashMap<String, PropertyTransition>,

    /// Default transition config for unspecified properties
    default_config: Option<TransitionConfig>,

    /// Property-specific transition configs
    configs: HashMap<String, TransitionConfig>,
}

impl TransitionManager {
    /// Create a new transition manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default transition config
    pub fn with_default(mut self, config: TransitionConfig) -> Self {
        self.default_config = Some(config);
        self
    }

    /// Set a transition config for a specific property
    pub fn with_property(mut self, property: impl Into<String>, config: TransitionConfig) -> Self {
        self.configs.insert(property.into(), config);
        self
    }

    /// Apply a Transition definition
    pub fn apply_transition(&mut self, transition: &Transition) {
        if transition.properties.is_empty() {
            // Apply to all
            self.default_config = Some(transition.config.clone());
        } else {
            for prop in &transition.properties {
                self.configs.insert(prop.clone(), transition.config.clone());
            }
        }
    }

    /// Get the config for a property
    pub fn get_config(&self, property: &str) -> Option<TransitionConfig> {
        self.configs
            .get(property)
            .cloned()
            .or_else(|| self.default_config.clone())
    }

    /// Set a property value, starting a transition if configured
    /// Returns the current interpolated value
    pub fn set_value(&mut self, property: &str, value: AnimatableValue) -> AnimatableValue {
        // Check if there's an active transition for this property
        if let Some(transition) = self.transitions.get_mut(property) {
            if transition.is_active() {
                // Retarget the existing transition
                transition.retarget(value.clone());
                return transition.current_value().unwrap_or(value);
            }
        }

        // Check if we should start a new transition
        if let Some(config) = self.get_config(property) {
            // Get the current value (from active transition or the new value as "from")
            let from = self
                .transitions
                .get(property)
                .and_then(|t| t.current_value())
                .unwrap_or_else(|| value.clone());

            let mut transition = PropertyTransition::new(property, from.clone(), value.clone(), config);
            transition.start();

            self.transitions.insert(property.to_string(), transition);
            return from;
        }

        // No transition configured, return value immediately
        value
    }

    /// Update all active transitions
    pub fn update(&mut self, dt: f32) {
        let mut completed = Vec::new();

        for (property, transition) in &mut self.transitions {
            if !transition.update(dt) {
                completed.push(property.clone());
            }
        }

        // Remove completed transitions
        for property in completed {
            self.transitions.remove(&property);
        }
    }

    /// Get the current value for a property (interpolated if transitioning)
    pub fn get_value(&self, property: &str) -> Option<AnimatableValue> {
        self.transitions
            .get(property)
            .and_then(|t| t.current_value())
    }

    /// Check if any transitions are active
    pub fn has_active_transitions(&self) -> bool {
        self.transitions.values().any(|t| t.is_active())
    }

    /// Get all active transitions
    pub fn active_transitions(&self) -> impl Iterator<Item = &PropertyTransition> {
        self.transitions.values().filter(|t| t.is_active())
    }

    /// Cancel all transitions
    pub fn cancel_all(&mut self) {
        self.transitions.clear();
    }

    /// Cancel a specific transition
    pub fn cancel(&mut self, property: &str) {
        self.transitions.remove(property);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_config() {
        let config = TransitionConfig::from_ms(300)
            .with_easing(Easing::EaseOut)
            .with_delay_ms(100);

        assert!((config.duration - 0.3).abs() < 0.001);
        assert!((config.delay - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_transition_applies_to() {
        let all = Transition::all(TransitionConfig::default());
        assert!(all.applies_to("opacity"));
        assert!(all.applies_to("anything"));

        let specific = Transition::property("opacity", TransitionConfig::default());
        assert!(specific.applies_to("opacity"));
        assert!(!specific.applies_to("transform"));
    }

    #[test]
    fn test_property_transition() {
        let mut transition = PropertyTransition::new(
            "opacity",
            AnimatableValue::Float(0.0),
            AnimatableValue::Float(1.0),
            TransitionConfig::new(1.0).with_easing(Easing::Linear),
        );

        transition.start();
        assert!(transition.is_active());

        // Update halfway
        transition.update(0.5);
        let value = transition.current_value().unwrap();
        assert!((value.as_float().unwrap() - 0.5).abs() < 0.001);

        // Complete
        transition.update(0.6);
        assert!(transition.is_complete());
    }

    #[test]
    fn test_property_transition_retarget() {
        let mut transition = PropertyTransition::new(
            "opacity",
            AnimatableValue::Float(0.0),
            AnimatableValue::Float(1.0),
            TransitionConfig::new(1.0).with_easing(Easing::Linear),
        );

        transition.start();
        transition.update(0.5); // At 0.5

        // Retarget to 0.0
        transition.retarget(AnimatableValue::Float(0.0));

        // Should now start from 0.5 going to 0.0
        assert!(transition.is_active());
        let from = transition.from.as_float().unwrap();
        assert!((from - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_transition_manager() {
        let mut manager = TransitionManager::new()
            .with_property("opacity", TransitionConfig::from_ms(300));

        // Set initial value
        manager.set_value("opacity", AnimatableValue::Float(0.0));

        // Set new value (should start transition)
        let current = manager.set_value("opacity", AnimatableValue::Float(1.0));
        assert!((current.as_float().unwrap() - 0.0).abs() < 0.001);

        // Should have active transition
        assert!(manager.has_active_transitions());

        // Update
        manager.update(0.15); // Halfway through 300ms
        let value = manager.get_value("opacity").unwrap();
        assert!(value.as_float().unwrap() > 0.0);
        assert!(value.as_float().unwrap() < 1.0);
    }

    #[test]
    fn test_transition_manager_no_config() {
        let mut manager = TransitionManager::new();

        // No transition configured for this property
        let value = manager.set_value("opacity", AnimatableValue::Float(1.0));

        // Should return value immediately
        assert!((value.as_float().unwrap() - 1.0).abs() < 0.001);
        assert!(!manager.has_active_transitions());
    }

    #[test]
    fn test_transition_with_delay() {
        let mut transition = PropertyTransition::new(
            "opacity",
            AnimatableValue::Float(0.0),
            AnimatableValue::Float(1.0),
            TransitionConfig::new(1.0).with_delay(0.5),
        );

        transition.start();

        // Should be delayed
        assert_eq!(transition.state.status, AnimationStatus::Delayed);

        // Update within delay
        transition.update(0.3);
        assert_eq!(transition.state.status, AnimationStatus::Delayed);

        // Update past delay
        transition.update(0.3);
        assert_eq!(transition.state.status, AnimationStatus::Running);
    }
}
