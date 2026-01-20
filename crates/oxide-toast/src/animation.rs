//! Toast animation system.
//!
//! Provides smooth 60fps animations for toast entrance, exit, and transitions.

use serde::{Deserialize, Serialize};

use crate::options::SlideDirection;

/// Animation easing function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Easing {
    /// Linear interpolation
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    #[default]
    EaseOut,
    /// Ease in and out (slow start and end)
    EaseInOut,
    /// Cubic ease in
    CubicIn,
    /// Cubic ease out
    CubicOut,
    /// Cubic ease in and out
    CubicInOut,
    /// Bounce effect at the end
    BounceOut,
    /// Elastic spring effect
    ElasticOut,
    /// Back overshoot
    BackOut,
}

impl Easing {
    /// Applies the easing function to a normalized time value (0.0 to 1.0).
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(2),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::CubicIn => t * t * t,
            Easing::CubicOut => 1.0 - (1.0 - t).powi(3),
            Easing::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::BounceOut => {
                const N1: f32 = 7.5625;
                const D1: f32 = 2.75;

                if t < 1.0 / D1 {
                    N1 * t * t
                } else if t < 2.0 / D1 {
                    let t = t - 1.5 / D1;
                    N1 * t * t + 0.75
                } else if t < 2.5 / D1 {
                    let t = t - 2.25 / D1;
                    N1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / D1;
                    N1 * t * t + 0.984375
                }
            }
            Easing::ElasticOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Easing::BackOut => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                let t_minus_1 = t - 1.0;
                1.0 + c3 * t_minus_1.powi(3) + c1 * t_minus_1.powi(2)
            }
        }
    }
}

/// Animation type for toasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AnimationType {
    /// No animation
    None,
    /// Slide in/out from edge
    #[default]
    Slide,
    /// Fade in/out
    Fade,
    /// Scale in/out
    Scale,
    /// Combined slide and fade
    SlideAndFade,
    /// Combined scale and fade
    ScaleAndFade,
    /// Slide with bounce
    SlideBounce,
    /// Spring physics animation
    Spring,
}

/// Animation state representing current values.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AnimationState {
    /// X translation in pixels
    pub translate_x: f32,
    /// Y translation in pixels
    pub translate_y: f32,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Scale factor (1.0 = normal size)
    pub scale: f32,
}

impl AnimationState {
    /// Creates a new animation state at full visibility.
    pub fn visible() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            opacity: 1.0,
            scale: 1.0,
        }
    }

    /// Creates a new animation state for hidden (pre-entrance or post-exit).
    pub fn hidden() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            opacity: 0.0,
            scale: 0.0,
        }
    }

    /// Linearly interpolates between two states.
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            translate_x: a.translate_x + (b.translate_x - a.translate_x) * t,
            translate_y: a.translate_y + (b.translate_y - a.translate_y) * t,
            opacity: a.opacity + (b.opacity - a.opacity) * t,
            scale: a.scale + (b.scale - a.scale) * t,
        }
    }

    /// Checks if the state represents a fully visible toast.
    pub fn is_visible(&self) -> bool {
        (self.opacity - 1.0).abs() < 0.01
            && (self.scale - 1.0).abs() < 0.01
            && self.translate_x.abs() < 0.5
            && self.translate_y.abs() < 0.5
    }

    /// Checks if the state represents a hidden toast.
    pub fn is_hidden(&self) -> bool {
        self.opacity < 0.01 || self.scale < 0.01
    }
}

/// Toast animation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToastAnimation {
    /// Type of animation
    pub animation_type: AnimationType,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Easing function
    pub easing: Easing,
    /// Slide direction (for slide animations)
    pub slide_direction: SlideDirection,
    /// Slide distance in pixels
    pub slide_distance: f32,
    /// Scale starting value for entrance (ending value for exit)
    pub scale_from: f32,
}

impl Default for ToastAnimation {
    fn default() -> Self {
        Self {
            animation_type: AnimationType::SlideAndFade,
            duration_ms: 200,
            easing: Easing::EaseOut,
            slide_direction: SlideDirection::Down,
            slide_distance: 50.0,
            scale_from: 0.95,
        }
    }
}

impl ToastAnimation {
    /// Creates a new animation with the specified type.
    pub fn new(animation_type: AnimationType) -> Self {
        Self {
            animation_type,
            ..Default::default()
        }
    }

    /// Creates a slide animation.
    pub fn slide(direction: SlideDirection) -> Self {
        Self {
            animation_type: AnimationType::Slide,
            slide_direction: direction,
            ..Default::default()
        }
    }

    /// Creates a fade animation.
    pub fn fade() -> Self {
        Self {
            animation_type: AnimationType::Fade,
            ..Default::default()
        }
    }

    /// Creates a scale animation.
    pub fn scale() -> Self {
        Self {
            animation_type: AnimationType::Scale,
            ..Default::default()
        }
    }

    /// Creates a slide and fade animation.
    pub fn slide_and_fade(direction: SlideDirection) -> Self {
        Self {
            animation_type: AnimationType::SlideAndFade,
            slide_direction: direction,
            ..Default::default()
        }
    }

    /// Creates a scale and fade animation.
    pub fn scale_and_fade() -> Self {
        Self {
            animation_type: AnimationType::ScaleAndFade,
            ..Default::default()
        }
    }

    /// Creates a bouncy slide animation.
    pub fn bounce(direction: SlideDirection) -> Self {
        Self {
            animation_type: AnimationType::SlideBounce,
            slide_direction: direction,
            easing: Easing::BounceOut,
            ..Default::default()
        }
    }

    /// Creates a spring animation.
    pub fn spring() -> Self {
        Self {
            animation_type: AnimationType::Spring,
            easing: Easing::ElasticOut,
            duration_ms: 400,
            ..Default::default()
        }
    }

    /// Creates a "no animation" config for instant show/hide.
    pub fn none() -> Self {
        Self {
            animation_type: AnimationType::None,
            duration_ms: 0,
            ..Default::default()
        }
    }

    /// Sets the animation duration in milliseconds.
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Sets the animation duration in seconds.
    pub fn duration_secs(mut self, secs: f32) -> Self {
        self.duration_ms = (secs * 1000.0) as u64;
        self
    }

    /// Sets the easing function.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the slide direction.
    pub fn slide_direction(mut self, direction: SlideDirection) -> Self {
        self.slide_direction = direction;
        self
    }

    /// Sets the slide distance in pixels.
    pub fn slide_distance(mut self, distance: f32) -> Self {
        self.slide_distance = distance;
        self
    }

    /// Sets the scale starting value for entrance.
    pub fn scale_from(mut self, scale: f32) -> Self {
        self.scale_from = scale;
        self
    }

    /// Returns the starting state for an entrance animation.
    pub fn entrance_start(&self) -> AnimationState {
        match self.animation_type {
            AnimationType::None => AnimationState::visible(),
            AnimationType::Slide | AnimationType::SlideBounce | AnimationType::Spring => {
                AnimationState {
                    translate_x: self.slide_offset_x(),
                    translate_y: self.slide_offset_y(),
                    opacity: 1.0,
                    scale: 1.0,
                }
            }
            AnimationType::Fade => AnimationState {
                translate_x: 0.0,
                translate_y: 0.0,
                opacity: 0.0,
                scale: 1.0,
            },
            AnimationType::Scale => AnimationState {
                translate_x: 0.0,
                translate_y: 0.0,
                opacity: 1.0,
                scale: self.scale_from,
            },
            AnimationType::SlideAndFade => AnimationState {
                translate_x: self.slide_offset_x(),
                translate_y: self.slide_offset_y(),
                opacity: 0.0,
                scale: 1.0,
            },
            AnimationType::ScaleAndFade => AnimationState {
                translate_x: 0.0,
                translate_y: 0.0,
                opacity: 0.0,
                scale: self.scale_from,
            },
        }
    }

    /// Returns the ending state for an entrance animation (full visibility).
    pub fn entrance_end(&self) -> AnimationState {
        AnimationState::visible()
    }

    /// Returns the starting state for an exit animation (full visibility).
    pub fn exit_start(&self) -> AnimationState {
        AnimationState::visible()
    }

    /// Returns the ending state for an exit animation.
    pub fn exit_end(&self) -> AnimationState {
        match self.animation_type {
            AnimationType::None => AnimationState::hidden(),
            AnimationType::Slide | AnimationType::SlideBounce | AnimationType::Spring => {
                AnimationState {
                    translate_x: self.slide_offset_x(),
                    translate_y: self.slide_offset_y(),
                    opacity: 1.0,
                    scale: 1.0,
                }
            }
            AnimationType::Fade => AnimationState {
                translate_x: 0.0,
                translate_y: 0.0,
                opacity: 0.0,
                scale: 1.0,
            },
            AnimationType::Scale => AnimationState {
                translate_x: 0.0,
                translate_y: 0.0,
                opacity: 1.0,
                scale: self.scale_from,
            },
            AnimationType::SlideAndFade => AnimationState {
                translate_x: self.slide_offset_x(),
                translate_y: self.slide_offset_y(),
                opacity: 0.0,
                scale: 1.0,
            },
            AnimationType::ScaleAndFade => AnimationState {
                translate_x: 0.0,
                translate_y: 0.0,
                opacity: 0.0,
                scale: self.scale_from,
            },
        }
    }

    /// Computes the animation state at a given progress (0.0 to 1.0) for entrance.
    pub fn compute_entrance_state(&self, progress: f32) -> AnimationState {
        let eased = self.easing.apply(progress);
        AnimationState::lerp(&self.entrance_start(), &self.entrance_end(), eased)
    }

    /// Computes the animation state at a given progress (0.0 to 1.0) for exit.
    pub fn compute_exit_state(&self, progress: f32) -> AnimationState {
        let eased = self.easing.apply(progress);
        AnimationState::lerp(&self.exit_start(), &self.exit_end(), eased)
    }

    /// Returns the X offset for slide animations.
    fn slide_offset_x(&self) -> f32 {
        match self.slide_direction {
            SlideDirection::Left => self.slide_distance,
            SlideDirection::Right => -self.slide_distance,
            _ => 0.0,
        }
    }

    /// Returns the Y offset for slide animations.
    fn slide_offset_y(&self) -> f32 {
        match self.slide_direction {
            SlideDirection::Up => self.slide_distance,
            SlideDirection::Down => -self.slide_distance,
            _ => 0.0,
        }
    }
}

/// An active animation being played.
#[derive(Debug, Clone)]
pub struct ActiveAnimation {
    /// Animation configuration
    pub config: ToastAnimation,
    /// Whether this is an entrance (true) or exit (false) animation
    pub is_entrance: bool,
    /// Animation start timestamp in milliseconds
    pub start_time: u64,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Current animation state
    pub state: AnimationState,
    /// Whether the animation has completed
    pub completed: bool,
}

impl ActiveAnimation {
    /// Creates a new entrance animation.
    pub fn entrance(config: ToastAnimation, start_time: u64) -> Self {
        Self {
            state: config.entrance_start(),
            config,
            is_entrance: true,
            start_time,
            progress: 0.0,
            completed: false,
        }
    }

    /// Creates a new exit animation.
    pub fn exit(config: ToastAnimation, start_time: u64) -> Self {
        Self {
            state: config.exit_start(),
            config,
            is_entrance: false,
            start_time,
            progress: 0.0,
            completed: false,
        }
    }

    /// Updates the animation with the current timestamp.
    /// Returns true if the animation state changed.
    pub fn update(&mut self, current_time: u64) -> bool {
        if self.completed {
            return false;
        }

        if self.config.duration_ms == 0 {
            self.progress = 1.0;
            self.completed = true;
            self.state = if self.is_entrance {
                self.config.entrance_end()
            } else {
                self.config.exit_end()
            };
            return true;
        }

        let elapsed = current_time.saturating_sub(self.start_time);
        self.progress = (elapsed as f32 / self.config.duration_ms as f32).min(1.0);

        self.state = if self.is_entrance {
            self.config.compute_entrance_state(self.progress)
        } else {
            self.config.compute_exit_state(self.progress)
        };

        if self.progress >= 1.0 {
            self.completed = true;
        }

        true
    }

    /// Returns the remaining duration in milliseconds.
    pub fn remaining_ms(&self) -> u64 {
        if self.completed {
            return 0;
        }
        let remaining_progress = 1.0 - self.progress;
        (remaining_progress * self.config.duration_ms as f32) as u64
    }
}

/// Calculates the optimal frame rate for animations.
pub const TARGET_FPS: u32 = 60;

/// Calculates the frame interval for 60fps animations.
pub const FRAME_INTERVAL_MS: u64 = 1000 / TARGET_FPS as u64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_linear() {
        let easing = Easing::Linear;
        assert!((easing.apply(0.0) - 0.0).abs() < 0.001);
        assert!((easing.apply(0.5) - 0.5).abs() < 0.001);
        assert!((easing.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_in() {
        let easing = Easing::EaseIn;
        assert!((easing.apply(0.0) - 0.0).abs() < 0.001);
        assert!(easing.apply(0.5) < 0.5); // Should be slower in the beginning
        assert!((easing.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_out() {
        let easing = Easing::EaseOut;
        assert!((easing.apply(0.0) - 0.0).abs() < 0.001);
        assert!(easing.apply(0.5) > 0.5); // Should be faster in the beginning
        assert!((easing.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_clamping() {
        let easing = Easing::Linear;
        assert!((easing.apply(-0.5) - 0.0).abs() < 0.001);
        assert!((easing.apply(1.5) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_state_visible() {
        let state = AnimationState::visible();
        assert!(state.is_visible());
        assert!(!state.is_hidden());
    }

    #[test]
    fn test_animation_state_hidden() {
        let state = AnimationState::hidden();
        assert!(!state.is_visible());
        assert!(state.is_hidden());
    }

    #[test]
    fn test_animation_state_lerp() {
        let a = AnimationState::hidden();
        let b = AnimationState::visible();

        let mid = AnimationState::lerp(&a, &b, 0.5);
        assert!((mid.opacity - 0.5).abs() < 0.001);
        assert!((mid.scale - 0.5).abs() < 0.001);

        let start = AnimationState::lerp(&a, &b, 0.0);
        assert!(start.is_hidden());

        let end = AnimationState::lerp(&a, &b, 1.0);
        assert!(end.is_visible());
    }

    #[test]
    fn test_toast_animation_default() {
        let anim = ToastAnimation::default();
        assert_eq!(anim.animation_type, AnimationType::SlideAndFade);
        assert_eq!(anim.duration_ms, 200);
        assert_eq!(anim.easing, Easing::EaseOut);
    }

    #[test]
    fn test_toast_animation_builders() {
        let slide = ToastAnimation::slide(SlideDirection::Up);
        assert_eq!(slide.animation_type, AnimationType::Slide);
        assert_eq!(slide.slide_direction, SlideDirection::Up);

        let fade = ToastAnimation::fade();
        assert_eq!(fade.animation_type, AnimationType::Fade);

        let scale = ToastAnimation::scale();
        assert_eq!(scale.animation_type, AnimationType::Scale);

        let bounce = ToastAnimation::bounce(SlideDirection::Down);
        assert_eq!(bounce.animation_type, AnimationType::SlideBounce);
        assert_eq!(bounce.easing, Easing::BounceOut);
    }

    #[test]
    fn test_toast_animation_entrance_states() {
        let anim = ToastAnimation::slide_and_fade(SlideDirection::Down);
        let start = anim.entrance_start();
        let end = anim.entrance_end();

        assert!(!start.is_visible());
        assert!(end.is_visible());
    }

    #[test]
    fn test_toast_animation_exit_states() {
        let anim = ToastAnimation::fade();
        let start = anim.exit_start();
        let end = anim.exit_end();

        assert!(start.is_visible());
        assert!(end.is_hidden());
    }

    #[test]
    fn test_active_animation_entrance() {
        let config = ToastAnimation::fade().duration_ms(100);
        let mut anim = ActiveAnimation::entrance(config, 0);

        assert!(!anim.completed);
        assert_eq!(anim.progress, 0.0);

        anim.update(50);
        assert!((anim.progress - 0.5).abs() < 0.01);
        assert!(!anim.completed);

        anim.update(100);
        assert!((anim.progress - 1.0).abs() < 0.01);
        assert!(anim.completed);
    }

    #[test]
    fn test_active_animation_exit() {
        let config = ToastAnimation::scale().duration_ms(200);
        let mut anim = ActiveAnimation::exit(config, 1000);

        assert!(anim.state.is_visible());

        anim.update(1200);
        assert!(anim.completed);
        assert!(!anim.state.is_visible());
    }

    #[test]
    fn test_active_animation_no_animation() {
        let config = ToastAnimation::none();
        let mut anim = ActiveAnimation::entrance(config, 0);

        assert!(!anim.completed);
        anim.update(0);
        assert!(anim.completed);
        assert!(anim.state.is_visible());
    }

    #[test]
    fn test_active_animation_remaining() {
        let config = ToastAnimation::fade().duration_ms(100);
        let mut anim = ActiveAnimation::entrance(config, 0);

        assert_eq!(anim.remaining_ms(), 100);

        anim.update(50);
        assert!((anim.remaining_ms() as i64 - 50).abs() < 5);

        anim.update(100);
        assert_eq!(anim.remaining_ms(), 0);
    }

    #[test]
    fn test_frame_rate_constants() {
        assert_eq!(TARGET_FPS, 60);
        assert_eq!(FRAME_INTERVAL_MS, 16); // ~16.67ms rounded down
    }
}
