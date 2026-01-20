//! Motion Tokens - Semantic Animation Presets
//!
//! Provides production-ready animation presets inspired by Framer Motion.
//! Motion tokens define semantic animation configurations for consistent,
//! physics-based animations throughout the application.
//!
//! # Example
//!
//! ```rust
//! use oxide_components::animation::motion_tokens::{MotionPreset, Motion, Duration, MotionEasing};
//!
//! // Use a preset
//! let fade_in = Motion::preset(MotionPreset::FadeIn);
//!
//! // Create a custom motion
//! let custom = Motion::new()
//!     .duration(Duration::Normal)
//!     .easing(MotionEasing::Spring { stiffness: 400.0, damping: 30.0 })
//!     .build();
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::easing::{CubicBezier, Easing};
use super::interpolate::{AnimatableValue, Point2D, Transform2D};
use super::state::{Animation, PlayDirection};

// =============================================================================
// Duration Tokens
// =============================================================================

/// Semantic duration tokens for animations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Duration {
    /// Instant feedback (50ms) - for micro-interactions
    Instant,
    /// Quick response (100ms) - button presses, toggles
    Quick,
    /// Fast transitions (150ms) - hover states, small movements
    Fast,
    /// Normal pace (250ms) - most UI transitions
    Normal,
    /// Moderate (300ms) - default for many animations
    Moderate,
    /// Slow, deliberate (400ms) - emphasis, reveals
    Slow,
    /// Very slow (500ms) - page transitions, modals
    Slower,
    /// Dramatic (700ms) - major transitions
    Dramatic,
    /// Custom duration in milliseconds
    Custom(u32),
}

impl Duration {
    /// Get the duration in seconds
    pub fn seconds(&self) -> f32 {
        self.ms() as f32 / 1000.0
    }

    /// Get the duration in milliseconds
    pub fn ms(&self) -> u32 {
        match self {
            Duration::Instant => 50,
            Duration::Quick => 100,
            Duration::Fast => 150,
            Duration::Normal => 250,
            Duration::Moderate => 300,
            Duration::Slow => 400,
            Duration::Slower => 500,
            Duration::Dramatic => 700,
            Duration::Custom(ms) => *ms,
        }
    }

    /// Create from milliseconds
    pub fn from_ms(ms: u32) -> Self {
        Duration::Custom(ms)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration::Normal
    }
}

// =============================================================================
// Motion Easing Tokens
// =============================================================================

/// Semantic easing tokens with physics-based options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MotionEasing {
    /// Linear - constant speed
    Linear,
    /// Ease - subtle acceleration/deceleration
    Ease,
    /// Ease In - slow start, fast end
    EaseIn,
    /// Ease Out - fast start, slow end (most natural for UI)
    EaseOut,
    /// Ease In Out - slow start and end
    EaseInOut,
    /// Sharp In - aggressive acceleration
    SharpIn,
    /// Sharp Out - aggressive deceleration
    SharpOut,
    /// Anticipate - slight overshoot before settling
    Anticipate,
    /// Overshoot - goes past target then settles
    Overshoot,
    /// Bounce - bounces at the end
    Bounce,
    /// Elastic - spring-like oscillation
    Elastic,
    /// Spring physics with custom parameters
    Spring {
        stiffness: f32,
        damping: f32,
    },
    /// Gentle spring (low stiffness, high damping)
    SpringGentle,
    /// Wobbly spring (medium stiffness, low damping)
    SpringWobbly,
    /// Bouncy spring (high stiffness, low damping)
    SpringBouncy,
    /// Stiff spring (high stiffness, high damping)
    SpringStiff,
    /// Slow spring (low stiffness, medium damping)
    SpringSlow,
    /// Molasses - very slow, heavy feel
    SpringMolasses,
    /// Custom cubic bezier
    CubicBezier {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
}

impl MotionEasing {
    /// Convert to the core Easing type
    pub fn to_easing(&self) -> Easing {
        match self {
            MotionEasing::Linear => Easing::Linear,
            MotionEasing::Ease => Easing::Ease,
            MotionEasing::EaseIn => Easing::EaseIn,
            MotionEasing::EaseOut => Easing::EaseOut,
            MotionEasing::EaseInOut => Easing::EaseInOut,
            MotionEasing::SharpIn => Easing::CubicBezier(CubicBezier::new(0.4, 0.0, 0.6, 1.0)),
            MotionEasing::SharpOut => Easing::CubicBezier(CubicBezier::new(0.0, 0.0, 0.2, 1.0)),
            MotionEasing::Anticipate => Easing::EaseInBack,
            MotionEasing::Overshoot => Easing::EaseOutBack,
            MotionEasing::Bounce => Easing::EaseOutBounce,
            MotionEasing::Elastic => Easing::EaseOutElastic,
            MotionEasing::Spring { stiffness, damping } => {
                Easing::Spring {
                    stiffness: *stiffness,
                    damping: *damping,
                    mass: 1.0,
                }
            }
            MotionEasing::SpringGentle => Easing::Spring {
                stiffness: 120.0,
                damping: 14.0,
                mass: 1.0,
            },
            MotionEasing::SpringWobbly => Easing::Spring {
                stiffness: 180.0,
                damping: 12.0,
                mass: 1.0,
            },
            MotionEasing::SpringBouncy => Easing::Spring {
                stiffness: 600.0,
                damping: 15.0,
                mass: 1.0,
            },
            MotionEasing::SpringStiff => Easing::Spring {
                stiffness: 400.0,
                damping: 40.0,
                mass: 1.0,
            },
            MotionEasing::SpringSlow => Easing::Spring {
                stiffness: 60.0,
                damping: 20.0,
                mass: 1.0,
            },
            MotionEasing::SpringMolasses => Easing::Spring {
                stiffness: 40.0,
                damping: 25.0,
                mass: 1.5,
            },
            MotionEasing::CubicBezier { x1, y1, x2, y2 } => {
                Easing::CubicBezier(CubicBezier::new(*x1, *y1, *x2, *y2))
            }
        }
    }

    /// Get spring parameters if this is a spring easing
    pub fn spring_params(&self) -> Option<SpringParams> {
        match self {
            MotionEasing::Spring { stiffness, damping } => Some(SpringParams {
                stiffness: *stiffness,
                damping: *damping,
                mass: 1.0,
            }),
            MotionEasing::SpringGentle => Some(SpringParams {
                stiffness: 120.0,
                damping: 14.0,
                mass: 1.0,
            }),
            MotionEasing::SpringWobbly => Some(SpringParams {
                stiffness: 180.0,
                damping: 12.0,
                mass: 1.0,
            }),
            MotionEasing::SpringBouncy => Some(SpringParams {
                stiffness: 600.0,
                damping: 15.0,
                mass: 1.0,
            }),
            MotionEasing::SpringStiff => Some(SpringParams {
                stiffness: 400.0,
                damping: 40.0,
                mass: 1.0,
            }),
            MotionEasing::SpringSlow => Some(SpringParams {
                stiffness: 60.0,
                damping: 20.0,
                mass: 1.0,
            }),
            MotionEasing::SpringMolasses => Some(SpringParams {
                stiffness: 40.0,
                damping: 25.0,
                mass: 1.5,
            }),
            _ => None,
        }
    }
}

impl Default for MotionEasing {
    fn default() -> Self {
        MotionEasing::EaseOut
    }
}

/// Spring physics parameters
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringParams {
    /// Spring stiffness (default: 100)
    pub stiffness: f32,
    /// Damping ratio (default: 10)
    pub damping: f32,
    /// Mass (default: 1)
    pub mass: f32,
}

impl SpringParams {
    /// Create new spring params
    pub fn new(stiffness: f32, damping: f32, mass: f32) -> Self {
        Self { stiffness, damping, mass }
    }

    /// Default spring configuration
    pub fn default_spring() -> Self {
        Self {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
        }
    }

    /// Very responsive spring
    pub fn snappy() -> Self {
        Self {
            stiffness: 500.0,
            damping: 30.0,
            mass: 1.0,
        }
    }

    /// Bouncy spring
    pub fn bouncy() -> Self {
        Self {
            stiffness: 300.0,
            damping: 10.0,
            mass: 1.0,
        }
    }

    /// Gentle, slow spring
    pub fn gentle() -> Self {
        Self {
            stiffness: 50.0,
            damping: 15.0,
            mass: 1.0,
        }
    }

    /// Calculate approximate settling time
    pub fn settling_time(&self) -> f32 {
        // For underdamped systems, settling time is approximately 4 * time constant
        let omega = (self.stiffness / self.mass).sqrt();
        let zeta = self.damping / (2.0 * (self.stiffness * self.mass).sqrt());

        if zeta >= 1.0 {
            // Critically or overdamped
            4.0 / (zeta * omega)
        } else {
            // Underdamped - uses decay envelope
            4.0 / (zeta * omega)
        }
    }

    /// Convert to Easing
    pub fn to_easing(&self) -> Easing {
        Easing::Spring {
            stiffness: self.stiffness,
            damping: self.damping,
            mass: self.mass,
        }
    }
}

impl Default for SpringParams {
    fn default() -> Self {
        Self::default_spring()
    }
}

// =============================================================================
// Motion Presets
// =============================================================================

/// Pre-defined motion presets for common animations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotionPreset {
    // === Opacity Animations ===
    /// Fade in from transparent
    FadeIn,
    /// Fade out to transparent
    FadeOut,
    /// Quick fade for tooltips, dropdowns
    FadeQuick,

    // === Scale Animations ===
    /// Scale up from 0.95 (subtle zoom)
    ScaleIn,
    /// Scale down to 0.95
    ScaleOut,
    /// Pop in with slight overshoot
    PopIn,
    /// Pop out with anticipation
    PopOut,
    /// Bounce in with spring physics
    BounceIn,

    // === Slide Animations ===
    /// Slide in from left
    SlideInLeft,
    /// Slide in from right
    SlideInRight,
    /// Slide in from top
    SlideInTop,
    /// Slide in from bottom
    SlideInBottom,
    /// Slide out to left
    SlideOutLeft,
    /// Slide out to right
    SlideOutRight,
    /// Slide out to top
    SlideOutTop,
    /// Slide out to bottom
    SlideOutBottom,

    // === Combined Animations ===
    /// Fade + slide from bottom (common for modals)
    FadeSlideUp,
    /// Fade + slide from top
    FadeSlideDown,
    /// Fade + scale (common for dialogs)
    FadeScale,
    /// Zoom in with fade
    ZoomIn,
    /// Zoom out with fade
    ZoomOut,

    // === Interaction Animations ===
    /// Button press feedback
    Press,
    /// Hover lift effect
    Hover,
    /// Focus ring animation
    Focus,
    /// Tap feedback for mobile
    Tap,

    // === Special Effects ===
    /// Shake horizontally (error feedback)
    ShakeX,
    /// Shake vertically
    ShakeY,
    /// Pulse (subtle scale breathing)
    Pulse,
    /// Wiggle rotation
    Wiggle,
    /// Spin 360 degrees
    Spin,
    /// Flip horizontally
    FlipX,
    /// Flip vertically
    FlipY,

    // === Page Transitions ===
    /// Page enter from right
    PageEnter,
    /// Page exit to left
    PageExit,
    /// Fade between pages
    PageFade,
    /// Shared element morph
    Morph,

    // === Layout Animations ===
    /// Smooth height change
    HeightAuto,
    /// Smooth width change
    WidthAuto,
    /// Layout shift animation
    LayoutShift,
    /// Collapse/expand
    Collapse,
    /// Expand from collapsed
    Expand,

    // === Attention Seekers ===
    /// Subtle attention pulse
    AttentionSubtle,
    /// Strong attention shake
    AttentionStrong,
    /// Heartbeat effect
    Heartbeat,
}

impl MotionPreset {
    /// Get the motion configuration for this preset
    pub fn config(&self) -> MotionConfig {
        match self {
            // Opacity
            MotionPreset::FadeIn => MotionConfig {
                duration: Duration::Normal,
                easing: MotionEasing::EaseOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::FadeOut => MotionConfig {
                duration: Duration::Fast,
                easing: MotionEasing::EaseIn,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::FadeQuick => MotionConfig {
                duration: Duration::Quick,
                easing: MotionEasing::Linear,
                delay: Duration::Instant,
                ..Default::default()
            },

            // Scale
            MotionPreset::ScaleIn => MotionConfig {
                duration: Duration::Normal,
                easing: MotionEasing::EaseOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::ScaleOut => MotionConfig {
                duration: Duration::Fast,
                easing: MotionEasing::EaseIn,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::PopIn => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::SpringBouncy,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::PopOut => MotionConfig {
                duration: Duration::Fast,
                easing: MotionEasing::Anticipate,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::BounceIn => MotionConfig {
                duration: Duration::Slow,
                easing: MotionEasing::Bounce,
                delay: Duration::Instant,
                ..Default::default()
            },

            // Slides
            MotionPreset::SlideInLeft | MotionPreset::SlideInRight |
            MotionPreset::SlideInTop | MotionPreset::SlideInBottom => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::EaseOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::SlideOutLeft | MotionPreset::SlideOutRight |
            MotionPreset::SlideOutTop | MotionPreset::SlideOutBottom => MotionConfig {
                duration: Duration::Fast,
                easing: MotionEasing::EaseIn,
                delay: Duration::Instant,
                ..Default::default()
            },

            // Combined
            MotionPreset::FadeSlideUp | MotionPreset::FadeSlideDown => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::EaseOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::FadeScale => MotionConfig {
                duration: Duration::Normal,
                easing: MotionEasing::SpringGentle,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::ZoomIn | MotionPreset::ZoomOut => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::EaseInOut,
                delay: Duration::Instant,
                ..Default::default()
            },

            // Interactions
            MotionPreset::Press => MotionConfig {
                duration: Duration::Quick,
                easing: MotionEasing::EaseOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Hover => MotionConfig {
                duration: Duration::Fast,
                easing: MotionEasing::EaseOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Focus => MotionConfig {
                duration: Duration::Fast,
                easing: MotionEasing::EaseOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Tap => MotionConfig {
                duration: Duration::Quick,
                easing: MotionEasing::SpringStiff,
                delay: Duration::Instant,
                ..Default::default()
            },

            // Special effects
            MotionPreset::ShakeX | MotionPreset::ShakeY => MotionConfig {
                duration: Duration::Slow,
                easing: MotionEasing::SpringWobbly,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Pulse => MotionConfig {
                duration: Duration::Slower,
                easing: MotionEasing::EaseInOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Wiggle => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::SpringWobbly,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Spin => MotionConfig {
                duration: Duration::Slower,
                easing: MotionEasing::Linear,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::FlipX | MotionPreset::FlipY => MotionConfig {
                duration: Duration::Slow,
                easing: MotionEasing::EaseInOut,
                delay: Duration::Instant,
                ..Default::default()
            },

            // Page transitions
            MotionPreset::PageEnter | MotionPreset::PageExit => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::EaseInOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::PageFade => MotionConfig {
                duration: Duration::Normal,
                easing: MotionEasing::EaseInOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Morph => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::SpringGentle,
                delay: Duration::Instant,
                ..Default::default()
            },

            // Layout
            MotionPreset::HeightAuto | MotionPreset::WidthAuto => MotionConfig {
                duration: Duration::Normal,
                easing: MotionEasing::EaseOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::LayoutShift => MotionConfig {
                duration: Duration::Normal,
                easing: MotionEasing::SpringGentle,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Collapse | MotionPreset::Expand => MotionConfig {
                duration: Duration::Normal,
                easing: MotionEasing::EaseInOut,
                delay: Duration::Instant,
                ..Default::default()
            },

            // Attention
            MotionPreset::AttentionSubtle => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::EaseInOut,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::AttentionStrong => MotionConfig {
                duration: Duration::Slow,
                easing: MotionEasing::SpringWobbly,
                delay: Duration::Instant,
                ..Default::default()
            },
            MotionPreset::Heartbeat => MotionConfig {
                duration: Duration::Moderate,
                easing: MotionEasing::EaseInOut,
                delay: Duration::Instant,
                ..Default::default()
            },
        }
    }

    /// Get keyframes for this preset
    pub fn keyframes(&self) -> Option<Vec<Keyframe>> {
        match self {
            MotionPreset::FadeIn => Some(vec![
                Keyframe::at(0.0).opacity(0.0),
                Keyframe::at(1.0).opacity(1.0),
            ]),
            MotionPreset::FadeOut => Some(vec![
                Keyframe::at(0.0).opacity(1.0),
                Keyframe::at(1.0).opacity(0.0),
            ]),
            MotionPreset::ScaleIn => Some(vec![
                Keyframe::at(0.0).scale(0.95, 0.95).opacity(0.0),
                Keyframe::at(1.0).scale(1.0, 1.0).opacity(1.0),
            ]),
            MotionPreset::ScaleOut => Some(vec![
                Keyframe::at(0.0).scale(1.0, 1.0).opacity(1.0),
                Keyframe::at(1.0).scale(0.95, 0.95).opacity(0.0),
            ]),
            MotionPreset::PopIn => Some(vec![
                Keyframe::at(0.0).scale(0.0, 0.0).opacity(0.0),
                Keyframe::at(0.5).scale(1.1, 1.1).opacity(1.0),
                Keyframe::at(1.0).scale(1.0, 1.0).opacity(1.0),
            ]),
            MotionPreset::SlideInLeft => Some(vec![
                Keyframe::at(0.0).translate(-100.0, 0.0).opacity(0.0),
                Keyframe::at(1.0).translate(0.0, 0.0).opacity(1.0),
            ]),
            MotionPreset::SlideInRight => Some(vec![
                Keyframe::at(0.0).translate(100.0, 0.0).opacity(0.0),
                Keyframe::at(1.0).translate(0.0, 0.0).opacity(1.0),
            ]),
            MotionPreset::SlideInTop => Some(vec![
                Keyframe::at(0.0).translate(0.0, -100.0).opacity(0.0),
                Keyframe::at(1.0).translate(0.0, 0.0).opacity(1.0),
            ]),
            MotionPreset::SlideInBottom => Some(vec![
                Keyframe::at(0.0).translate(0.0, 100.0).opacity(0.0),
                Keyframe::at(1.0).translate(0.0, 0.0).opacity(1.0),
            ]),
            MotionPreset::FadeSlideUp => Some(vec![
                Keyframe::at(0.0).translate(0.0, 20.0).opacity(0.0),
                Keyframe::at(1.0).translate(0.0, 0.0).opacity(1.0),
            ]),
            MotionPreset::FadeSlideDown => Some(vec![
                Keyframe::at(0.0).translate(0.0, -20.0).opacity(0.0),
                Keyframe::at(1.0).translate(0.0, 0.0).opacity(1.0),
            ]),
            MotionPreset::FadeScale => Some(vec![
                Keyframe::at(0.0).scale(0.9, 0.9).opacity(0.0),
                Keyframe::at(1.0).scale(1.0, 1.0).opacity(1.0),
            ]),
            MotionPreset::ShakeX => Some(vec![
                Keyframe::at(0.0).translate(0.0, 0.0),
                Keyframe::at(0.1).translate(-10.0, 0.0),
                Keyframe::at(0.2).translate(10.0, 0.0),
                Keyframe::at(0.3).translate(-10.0, 0.0),
                Keyframe::at(0.4).translate(10.0, 0.0),
                Keyframe::at(0.5).translate(-10.0, 0.0),
                Keyframe::at(0.6).translate(10.0, 0.0),
                Keyframe::at(0.7).translate(-5.0, 0.0),
                Keyframe::at(0.8).translate(5.0, 0.0),
                Keyframe::at(0.9).translate(-2.0, 0.0),
                Keyframe::at(1.0).translate(0.0, 0.0),
            ]),
            MotionPreset::ShakeY => Some(vec![
                Keyframe::at(0.0).translate(0.0, 0.0),
                Keyframe::at(0.1).translate(0.0, -10.0),
                Keyframe::at(0.2).translate(0.0, 10.0),
                Keyframe::at(0.3).translate(0.0, -10.0),
                Keyframe::at(0.4).translate(0.0, 10.0),
                Keyframe::at(0.5).translate(0.0, -5.0),
                Keyframe::at(0.6).translate(0.0, 5.0),
                Keyframe::at(0.7).translate(0.0, -2.0),
                Keyframe::at(0.8).translate(0.0, 2.0),
                Keyframe::at(1.0).translate(0.0, 0.0),
            ]),
            MotionPreset::Pulse => Some(vec![
                Keyframe::at(0.0).scale(1.0, 1.0),
                Keyframe::at(0.5).scale(1.05, 1.05),
                Keyframe::at(1.0).scale(1.0, 1.0),
            ]),
            MotionPreset::Wiggle => Some(vec![
                Keyframe::at(0.0).rotate(0.0),
                Keyframe::at(0.1).rotate(-3.0),
                Keyframe::at(0.2).rotate(3.0),
                Keyframe::at(0.3).rotate(-3.0),
                Keyframe::at(0.4).rotate(3.0),
                Keyframe::at(0.5).rotate(-2.0),
                Keyframe::at(0.6).rotate(2.0),
                Keyframe::at(0.7).rotate(-1.0),
                Keyframe::at(0.8).rotate(1.0),
                Keyframe::at(1.0).rotate(0.0),
            ]),
            MotionPreset::Press => Some(vec![
                Keyframe::at(0.0).scale(1.0, 1.0),
                Keyframe::at(0.5).scale(0.95, 0.95),
                Keyframe::at(1.0).scale(1.0, 1.0),
            ]),
            MotionPreset::Heartbeat => Some(vec![
                Keyframe::at(0.0).scale(1.0, 1.0),
                Keyframe::at(0.14).scale(1.3, 1.3),
                Keyframe::at(0.28).scale(1.0, 1.0),
                Keyframe::at(0.42).scale(1.3, 1.3),
                Keyframe::at(0.7).scale(1.0, 1.0),
                Keyframe::at(1.0).scale(1.0, 1.0),
            ]),
            _ => None,
        }
    }
}

// =============================================================================
// Motion Configuration
// =============================================================================

/// Complete motion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionConfig {
    /// Animation duration
    pub duration: Duration,
    /// Easing function
    pub easing: MotionEasing,
    /// Delay before starting
    pub delay: Duration,
    /// Number of iterations (0 = infinite)
    pub iterations: u32,
    /// Play direction
    pub direction: PlayDirection,
    /// Whether to use reduced motion if user prefers
    pub respect_reduced_motion: bool,
}

impl Default for MotionConfig {
    fn default() -> Self {
        Self {
            duration: Duration::Normal,
            easing: MotionEasing::EaseOut,
            delay: Duration::Instant,
            iterations: 1,
            direction: PlayDirection::Forward,
            respect_reduced_motion: true,
        }
    }
}

impl MotionConfig {
    /// Create a new motion config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set easing
    pub fn easing(mut self, easing: MotionEasing) -> Self {
        self.easing = easing;
        self
    }

    /// Set delay
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Set iterations
    pub fn iterations(mut self, count: u32) -> Self {
        self.iterations = count;
        self
    }

    /// Set to infinite iterations
    pub fn infinite(mut self) -> Self {
        self.iterations = 0;
        self
    }

    /// Set direction
    pub fn direction(mut self, direction: PlayDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Create an Animation from this config
    pub fn to_animation(&self, property: &str, from: AnimatableValue, to: AnimatableValue) -> Animation {
        Animation::new(property)
            .from(from)
            .to(to)
            .duration(self.duration.seconds())
            .delay(self.delay.seconds())
            .easing(self.easing.to_easing())
            .direction(self.direction)
            .iterations(self.iterations)
            .build()
    }
}

// =============================================================================
// Keyframes
// =============================================================================

/// A single keyframe in an animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    /// Progress position (0.0 to 1.0)
    pub offset: f32,
    /// Properties at this keyframe
    pub properties: HashMap<String, AnimatableValue>,
    /// Optional easing for the segment leading to this keyframe
    pub easing: Option<MotionEasing>,
}

impl Keyframe {
    /// Create a keyframe at the given offset
    pub fn at(offset: f32) -> Self {
        Self {
            offset: offset.clamp(0.0, 1.0),
            properties: HashMap::new(),
            easing: None,
        }
    }

    /// Set opacity
    pub fn opacity(mut self, value: f32) -> Self {
        self.properties.insert("opacity".into(), AnimatableValue::Float(value));
        self
    }

    /// Set scale
    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.properties.insert("scale".into(), AnimatableValue::Transform(Transform2D {
            scale_x: x,
            scale_y: y,
            ..Transform2D::identity()
        }));
        self
    }

    /// Set translation
    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.properties.insert("translate".into(), AnimatableValue::Point(Point2D::new(x, y)));
        self
    }

    /// Set rotation (in degrees)
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.properties.insert("rotate".into(), AnimatableValue::Float(degrees.to_radians()));
        self
    }

    /// Set a custom property
    pub fn property(mut self, name: &str, value: AnimatableValue) -> Self {
        self.properties.insert(name.into(), value);
        self
    }

    /// Set the easing for the segment to this keyframe
    pub fn with_easing(mut self, easing: MotionEasing) -> Self {
        self.easing = Some(easing);
        self
    }
}

// =============================================================================
// Keyframe Animation
// =============================================================================

/// A keyframe-based animation (like CSS @keyframes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeAnimation {
    /// Animation name
    pub name: String,
    /// Keyframes sorted by offset
    pub keyframes: Vec<Keyframe>,
    /// Configuration
    pub config: MotionConfig,
}

impl KeyframeAnimation {
    /// Create a new keyframe animation
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            keyframes: Vec::new(),
            config: MotionConfig::default(),
        }
    }

    /// Add a keyframe
    pub fn keyframe(mut self, keyframe: Keyframe) -> Self {
        self.keyframes.push(keyframe);
        self.keyframes.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        self
    }

    /// Set configuration
    pub fn config(mut self, config: MotionConfig) -> Self {
        self.config = config;
        self
    }

    /// Set duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.config.duration = duration;
        self
    }

    /// Set iterations
    pub fn iterations(mut self, count: u32) -> Self {
        self.config.iterations = count;
        self
    }

    /// Get the interpolated properties at a given progress
    pub fn sample(&self, progress: f32) -> HashMap<String, AnimatableValue> {
        let progress = progress.clamp(0.0, 1.0);
        let mut result = HashMap::new();

        if self.keyframes.is_empty() {
            return result;
        }

        // Find the two keyframes we're between
        let mut prev_keyframe: Option<&Keyframe> = None;
        let mut next_keyframe: Option<&Keyframe> = None;

        for keyframe in &self.keyframes {
            if keyframe.offset <= progress {
                prev_keyframe = Some(keyframe);
            }
            if keyframe.offset >= progress && next_keyframe.is_none() {
                next_keyframe = Some(keyframe);
            }
        }

        // Handle edge cases
        let prev = prev_keyframe.or(self.keyframes.first()).unwrap();
        let next = next_keyframe.or(self.keyframes.last()).unwrap();

        // Calculate local progress between keyframes
        let local_progress = if (next.offset - prev.offset).abs() < f32::EPSILON {
            1.0
        } else {
            (progress - prev.offset) / (next.offset - prev.offset)
        };

        // Apply easing if specified
        let eased_progress = if let Some(easing) = &next.easing {
            easing.to_easing().evaluate(local_progress)
        } else {
            self.config.easing.to_easing().evaluate(local_progress)
        };

        // Interpolate all properties
        for (name, from_value) in &prev.properties {
            if let Some(to_value) = next.properties.get(name) {
                if let Some(interpolated) = from_value.interpolate(to_value, eased_progress) {
                    result.insert(name.clone(), interpolated);
                }
            } else {
                result.insert(name.clone(), from_value.clone());
            }
        }

        // Include properties only in next
        for (name, value) in &next.properties {
            if !result.contains_key(name) {
                result.insert(name.clone(), value.clone());
            }
        }

        result
    }
}

// =============================================================================
// Motion Builder (Fluent API)
// =============================================================================

/// Fluent builder for creating motion configurations
#[derive(Debug, Clone)]
pub struct Motion {
    config: MotionConfig,
}

impl Motion {
    /// Create a new motion builder
    pub fn new() -> Self {
        Self {
            config: MotionConfig::default(),
        }
    }

    /// Create from a preset
    pub fn preset(preset: MotionPreset) -> Self {
        Self {
            config: preset.config(),
        }
    }

    /// Set duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.config.duration = duration;
        self
    }

    /// Set duration in milliseconds
    pub fn duration_ms(mut self, ms: u32) -> Self {
        self.config.duration = Duration::Custom(ms);
        self
    }

    /// Set easing
    pub fn easing(mut self, easing: MotionEasing) -> Self {
        self.config.easing = easing;
        self
    }

    /// Use spring physics
    pub fn spring(mut self, stiffness: f32, damping: f32) -> Self {
        self.config.easing = MotionEasing::Spring { stiffness, damping };
        self
    }

    /// Use a preset spring
    pub fn spring_preset(mut self, preset: SpringPreset) -> Self {
        self.config.easing = preset.to_motion_easing();
        self
    }

    /// Set delay
    pub fn delay(mut self, delay: Duration) -> Self {
        self.config.delay = delay;
        self
    }

    /// Set delay in milliseconds
    pub fn delay_ms(mut self, ms: u32) -> Self {
        self.config.delay = Duration::Custom(ms);
        self
    }

    /// Set iterations
    pub fn iterations(mut self, count: u32) -> Self {
        self.config.iterations = count;
        self
    }

    /// Set to infinite
    pub fn infinite(mut self) -> Self {
        self.config.iterations = 0;
        self
    }

    /// Set direction
    pub fn direction(mut self, direction: PlayDirection) -> Self {
        self.config.direction = direction;
        self
    }

    /// Alternate direction
    pub fn alternate(mut self) -> Self {
        self.config.direction = PlayDirection::Alternate;
        self
    }

    /// Build the configuration
    pub fn build(self) -> MotionConfig {
        self.config
    }

    /// Create an animation with this motion
    pub fn animate(self, property: &str, from: AnimatableValue, to: AnimatableValue) -> Animation {
        self.config.to_animation(property, from, to)
    }
}

impl Default for Motion {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Spring Presets
// =============================================================================

/// Pre-defined spring configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpringPreset {
    /// Default balanced spring
    Default,
    /// Very responsive, no overshoot
    Snappy,
    /// Bouncy with overshoot
    Bouncy,
    /// Gentle, slow settle
    Gentle,
    /// Stiff, quick settle
    Stiff,
    /// Wobbly with oscillation
    Wobbly,
    /// Heavy, slow movement
    Molasses,
    /// Quick with slight bounce
    Quick,
}

impl SpringPreset {
    /// Get the spring parameters
    pub fn params(&self) -> SpringParams {
        match self {
            SpringPreset::Default => SpringParams::new(100.0, 10.0, 1.0),
            SpringPreset::Snappy => SpringParams::new(500.0, 30.0, 1.0),
            SpringPreset::Bouncy => SpringParams::new(300.0, 10.0, 1.0),
            SpringPreset::Gentle => SpringParams::new(50.0, 15.0, 1.0),
            SpringPreset::Stiff => SpringParams::new(400.0, 40.0, 1.0),
            SpringPreset::Wobbly => SpringParams::new(180.0, 12.0, 1.0),
            SpringPreset::Molasses => SpringParams::new(40.0, 25.0, 1.5),
            SpringPreset::Quick => SpringParams::new(300.0, 20.0, 1.0),
        }
    }

    /// Convert to MotionEasing
    pub fn to_motion_easing(&self) -> MotionEasing {
        let params = self.params();
        MotionEasing::Spring {
            stiffness: params.stiffness,
            damping: params.damping,
        }
    }

    /// Convert to core Easing
    pub fn to_easing(&self) -> Easing {
        self.params().to_easing()
    }
}

// =============================================================================
// Animation Variants (Framer Motion style)
// =============================================================================

/// Named animation states for declarative animations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Variants {
    /// Named variant definitions
    variants: HashMap<String, VariantDefinition>,
    /// Default transition config
    default_transition: Option<MotionConfig>,
}

/// Definition of a single variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantDefinition {
    /// Properties for this variant
    pub properties: HashMap<String, AnimatableValue>,
    /// Transition override for this variant
    pub transition: Option<MotionConfig>,
}

impl Variants {
    /// Create new empty variants
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a variant
    pub fn variant(mut self, name: &str, definition: VariantDefinition) -> Self {
        self.variants.insert(name.to_string(), definition);
        self
    }

    /// Set default transition
    pub fn default_transition(mut self, transition: MotionConfig) -> Self {
        self.default_transition = Some(transition);
        self
    }

    /// Get a variant by name
    pub fn get(&self, name: &str) -> Option<&VariantDefinition> {
        self.variants.get(name)
    }

    /// Create animations to transition from one variant to another
    pub fn transition_to(&self, from: &str, to: &str) -> Vec<Animation> {
        let from_def = self.variants.get(from);
        let to_def = match self.variants.get(to) {
            Some(def) => def,
            None => return Vec::new(),
        };

        let config = to_def.transition.as_ref()
            .or(self.default_transition.as_ref())
            .cloned()
            .unwrap_or_default();

        let mut animations = Vec::new();

        for (property, to_value) in &to_def.properties {
            let from_value = from_def
                .and_then(|d| d.properties.get(property))
                .cloned()
                .unwrap_or_else(|| to_value.clone());

            let animation = config.to_animation(property, from_value, to_value.clone());
            animations.push(animation);
        }

        animations
    }
}

impl VariantDefinition {
    /// Create a new variant definition
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            transition: None,
        }
    }

    /// Add a property
    pub fn property(mut self, name: &str, value: AnimatableValue) -> Self {
        self.properties.insert(name.to_string(), value);
        self
    }

    /// Set opacity
    pub fn opacity(mut self, value: f32) -> Self {
        self.properties.insert("opacity".to_string(), AnimatableValue::Float(value));
        self
    }

    /// Set scale
    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.properties.insert("scale".to_string(), AnimatableValue::Transform(Transform2D {
            scale_x: x,
            scale_y: y,
            ..Transform2D::identity()
        }));
        self
    }

    /// Set translation
    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.properties.insert("translate".to_string(), AnimatableValue::Point(Point2D::new(x, y)));
        self
    }

    /// Set x position
    pub fn x(mut self, value: f32) -> Self {
        self.properties.insert("x".to_string(), AnimatableValue::Float(value));
        self
    }

    /// Set y position
    pub fn y(mut self, value: f32) -> Self {
        self.properties.insert("y".to_string(), AnimatableValue::Float(value));
        self
    }

    /// Set transition
    pub fn transition(mut self, config: MotionConfig) -> Self {
        self.transition = Some(config);
        self
    }
}

impl Default for VariantDefinition {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Stagger (for list animations)
// =============================================================================

/// Configuration for staggered animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaggerConfig {
    /// Delay between each child
    pub stagger: Duration,
    /// Direction of stagger
    pub direction: StaggerDirection,
    /// Starting delay
    pub delay_start: Duration,
}

/// Direction for staggered animations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaggerDirection {
    /// First to last
    Forward,
    /// Last to first
    Reverse,
    /// From center outward
    Center,
}

impl Default for StaggerConfig {
    fn default() -> Self {
        Self {
            stagger: Duration::Quick,
            direction: StaggerDirection::Forward,
            delay_start: Duration::Instant,
        }
    }
}

impl StaggerConfig {
    /// Create new stagger config
    pub fn new(stagger: Duration) -> Self {
        Self {
            stagger,
            ..Default::default()
        }
    }

    /// Set direction
    pub fn direction(mut self, direction: StaggerDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set starting delay
    pub fn delay_start(mut self, delay: Duration) -> Self {
        self.delay_start = delay;
        self
    }

    /// Calculate delay for a specific child index
    pub fn delay_for(&self, index: usize, total: usize) -> Duration {
        let base_delay = self.delay_start.ms();
        let stagger_delay = self.stagger.ms();

        let effective_index = match self.direction {
            StaggerDirection::Forward => index,
            StaggerDirection::Reverse => total.saturating_sub(1).saturating_sub(index),
            StaggerDirection::Center => {
                let center = total / 2;
                let distance = (index as i32 - center as i32).unsigned_abs() as usize;
                distance
            }
        };

        Duration::Custom(base_delay + (effective_index as u32 * stagger_delay))
    }
}

// =============================================================================
// Reduced Motion Support
// =============================================================================

/// Handles reduced motion preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReducedMotionMode {
    /// Normal animations
    #[default]
    Normal,
    /// Reduced motion (fade only, no movement)
    Reduced,
    /// No motion at all (instant changes)
    None,
}

impl ReducedMotionMode {
    /// Adjust a motion config for reduced motion
    pub fn adjust_config(&self, config: MotionConfig) -> MotionConfig {
        match self {
            ReducedMotionMode::Normal => config,
            ReducedMotionMode::Reduced => MotionConfig {
                duration: Duration::Fast,
                easing: MotionEasing::Linear,
                ..config
            },
            ReducedMotionMode::None => MotionConfig {
                duration: Duration::Instant,
                easing: MotionEasing::Linear,
                ..config
            },
        }
    }

    /// Check if animations should be reduced
    pub fn should_reduce(&self) -> bool {
        matches!(self, ReducedMotionMode::Reduced | ReducedMotionMode::None)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_conversion() {
        assert_eq!(Duration::Instant.ms(), 50);
        assert_eq!(Duration::Quick.ms(), 100);
        assert_eq!(Duration::Normal.ms(), 250);
        assert!((Duration::Normal.seconds() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_motion_easing_to_core_easing() {
        let easing = MotionEasing::EaseOut.to_easing();
        // Just verify it converts without panicking
        let _ = easing.evaluate(0.5);
    }

    #[test]
    fn test_spring_params() {
        let spring = SpringParams::bouncy();
        assert!(spring.stiffness > 100.0);
        assert!(spring.damping < 20.0);
    }

    #[test]
    fn test_motion_preset_config() {
        let config = MotionPreset::FadeIn.config();
        assert_eq!(config.duration, Duration::Normal);
    }

    #[test]
    fn test_motion_preset_keyframes() {
        let keyframes = MotionPreset::FadeIn.keyframes();
        assert!(keyframes.is_some());
        let keyframes = keyframes.unwrap();
        assert_eq!(keyframes.len(), 2);
        assert!((keyframes[0].offset - 0.0).abs() < 0.001);
        assert!((keyframes[1].offset - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_keyframe_animation_sample() {
        let anim = KeyframeAnimation::new("test")
            .keyframe(Keyframe::at(0.0).opacity(0.0))
            .keyframe(Keyframe::at(1.0).opacity(1.0))
            .config(MotionConfig::new().easing(MotionEasing::Linear));

        let sample = anim.sample(0.5);
        let opacity = sample.get("opacity").unwrap().as_float().unwrap();
        assert!((opacity - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_motion_builder() {
        let config = Motion::new()
            .duration(Duration::Fast)
            .easing(MotionEasing::SpringBouncy)
            .delay(Duration::Quick)
            .iterations(3)
            .build();

        assert_eq!(config.duration, Duration::Fast);
        assert_eq!(config.delay, Duration::Quick);
        assert_eq!(config.iterations, 3);
    }

    #[test]
    fn test_motion_builder_preset() {
        let config = Motion::preset(MotionPreset::PopIn).build();
        assert_eq!(config.duration, Duration::Moderate);
    }

    #[test]
    fn test_spring_preset() {
        let params = SpringPreset::Bouncy.params();
        assert!(params.stiffness > 200.0);

        let easing = SpringPreset::Bouncy.to_easing();
        let value = easing.evaluate(0.5);
        assert!(value >= 0.0);
    }

    #[test]
    fn test_variants() {
        let variants = Variants::new()
            .variant("hidden", VariantDefinition::new().opacity(0.0).x(-100.0))
            .variant("visible", VariantDefinition::new().opacity(1.0).x(0.0))
            .default_transition(Motion::new().duration(Duration::Normal).build());

        let hidden = variants.get("hidden").unwrap();
        assert!((hidden.properties.get("opacity").unwrap().as_float().unwrap() - 0.0).abs() < 0.001);

        let animations = variants.transition_to("hidden", "visible");
        assert!(!animations.is_empty());
    }

    #[test]
    fn test_stagger_config() {
        let stagger = StaggerConfig::new(Duration::Quick);

        let delay0 = stagger.delay_for(0, 5);
        let delay2 = stagger.delay_for(2, 5);

        assert!(delay2.ms() > delay0.ms());
    }

    #[test]
    fn test_stagger_center() {
        let stagger = StaggerConfig::new(Duration::Quick)
            .direction(StaggerDirection::Center);

        // Center item should have smallest delay
        let delay_center = stagger.delay_for(2, 5);
        let delay_edge = stagger.delay_for(0, 5);

        assert!(delay_center.ms() <= delay_edge.ms());
    }

    #[test]
    fn test_reduced_motion() {
        let config = MotionConfig::new()
            .duration(Duration::Slow)
            .easing(MotionEasing::SpringBouncy);

        let reduced = ReducedMotionMode::Reduced.adjust_config(config.clone());
        assert_eq!(reduced.duration, Duration::Fast);

        let none = ReducedMotionMode::None.adjust_config(config);
        assert_eq!(none.duration, Duration::Instant);
    }

    #[test]
    fn test_keyframe_builder() {
        let kf = Keyframe::at(0.5)
            .opacity(0.5)
            .scale(1.2, 1.2)
            .translate(10.0, 20.0)
            .rotate(45.0)
            .with_easing(MotionEasing::EaseInOut);

        assert!((kf.offset - 0.5).abs() < 0.001);
        assert!(kf.properties.contains_key("opacity"));
        assert!(kf.properties.contains_key("scale"));
        assert!(kf.properties.contains_key("translate"));
        assert!(kf.properties.contains_key("rotate"));
        assert!(kf.easing.is_some());
    }

    #[test]
    fn test_motion_animate() {
        let animation = Motion::new()
            .duration(Duration::Normal)
            .easing(MotionEasing::EaseOut)
            .animate("opacity", AnimatableValue::Float(0.0), AnimatableValue::Float(1.0));

        assert_eq!(animation.property, "opacity");
        assert!((animation.duration - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_variant_definition_chain() {
        let def = VariantDefinition::new()
            .opacity(0.5)
            .scale(1.5, 1.5)
            .x(100.0)
            .y(200.0)
            .transition(Motion::new().duration(Duration::Fast).build());

        assert_eq!(def.properties.len(), 4);
        assert!(def.transition.is_some());
    }

    #[test]
    fn test_spring_settling_time() {
        let gentle = SpringParams::gentle();
        let snappy = SpringParams::snappy();

        // Gentle spring should take longer to settle
        assert!(gentle.settling_time() > snappy.settling_time());
    }
}
