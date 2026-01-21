//! Chart animations.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Animation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Animation duration
    pub duration: Duration,
    /// Easing function
    pub easing: Easing,
    /// Delay before animation
    pub delay: Duration,
    /// Enable animation
    pub enabled: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(300),
            easing: Easing::EaseOut,
            delay: Duration::ZERO,
            enabled: true,
        }
    }
}

impl AnimationConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set easing
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Disable animation
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Easing function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Easing {
    /// Linear
    Linear,
    /// Ease in
    EaseIn,
    /// Ease out
    #[default]
    EaseOut,
    /// Ease in-out
    EaseInOut,
    /// Bounce
    Bounce,
    /// Elastic
    Elastic,
}

impl Easing {
    /// Apply easing to value (0.0 to 1.0)
    pub fn apply(&self, t: f64) -> f64 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            Easing::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    (2.0_f64).powf(-10.0 * t) * ((t - p / 4.0) * std::f64::consts::TAU / p).sin() + 1.0
                }
            }
        }
    }
}

/// Animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChartAnimationState {
    /// Not started
    #[default]
    Idle,
    /// Currently animating
    Running,
    /// Animation complete
    Complete,
}

/// Chart animation controller
#[derive(Debug, Clone)]
pub struct ChartAnimation {
    /// Configuration
    pub config: AnimationConfig,
    /// Current state
    pub state: ChartAnimationState,
    /// Progress (0.0 to 1.0)
    pub progress: f64,
}

impl ChartAnimation {
    /// Create new animation
    pub fn new() -> Self {
        Self {
            config: AnimationConfig::default(),
            state: ChartAnimationState::Idle,
            progress: 0.0,
        }
    }

    /// Set configuration
    pub fn config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }

    /// Start animation
    pub fn start(&mut self) {
        self.state = ChartAnimationState::Running;
        self.progress = 0.0;
    }

    /// Update progress
    pub fn update(&mut self, delta: Duration) {
        if self.state != ChartAnimationState::Running {
            return;
        }

        let duration = self.config.duration.as_secs_f64();
        if duration > 0.0 {
            self.progress += delta.as_secs_f64() / duration;
            if self.progress >= 1.0 {
                self.progress = 1.0;
                self.state = ChartAnimationState::Complete;
            }
        } else {
            self.progress = 1.0;
            self.state = ChartAnimationState::Complete;
        }
    }

    /// Get eased progress
    pub fn eased_progress(&self) -> f64 {
        self.config.easing.apply(self.progress)
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.state == ChartAnimationState::Complete
    }

    /// Reset animation
    pub fn reset(&mut self) {
        self.state = ChartAnimationState::Idle;
        self.progress = 0.0;
    }
}

impl Default for ChartAnimation {
    fn default() -> Self {
        Self::new()
    }
}
