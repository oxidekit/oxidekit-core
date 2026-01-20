//! Animation System for OxideKit
//!
//! A comprehensive, production-ready animation system inspired by Framer Motion.
//! Provides everything needed for fluid, physics-based UI animations.
//!
//! ## Features
//!
//! - **Easing Functions**: Linear, quadratic, cubic, quartic, quintic, sine,
//!   exponential, circular, elastic, back, bounce, and custom cubic-bezier
//! - **Spring Physics**: Configurable spring animations with stiffness, damping, and mass
//! - **Property Interpolation**: Support for f32, colors, transforms, and custom types
//! - **Animation State Machine**: Play, pause, reverse, seek controls
//! - **Timelines**: Sequence and orchestrate multiple animations
//! - **Transitions**: Property-specific automatic animations
//! - **Motion Tokens**: Semantic animation presets (quick, normal, slow, emphasis)
//! - **Gesture Support**: Velocity-driven springs, inertial scrolling, snap points
//! - **Keyframe Animations**: Multi-step animations with per-keyframe easing
//! - **Variants**: Framer Motion-style declarative animation states
//!
//! ## Quick Start
//!
//! ```rust
//! use oxide_components::animation::{Animation, Easing, AnimationController};
//!
//! // Create an animation
//! let anim = Animation::new("opacity")
//!     .from(0.0_f32)
//!     .to(1.0_f32)
//!     .duration_ms(300)
//!     .easing(Easing::EaseOut)
//!     .build();
//!
//! // Create a controller and add the animation
//! let mut controller = AnimationController::new();
//! let id = controller.add(anim);
//!
//! // Start the animation
//! controller.play(id);
//!
//! // Update each frame (returns current interpolated values)
//! let dt = 0.016; // 16ms frame time
//! let values = controller.update(dt);
//! ```
//!
//! ## Motion Tokens
//!
//! Use semantic motion tokens for consistent animations:
//!
//! ```rust
//! use oxide_components::animation::motion_tokens::{Motion, MotionPreset, Duration, MotionEasing};
//!
//! // Use a preset
//! let fade_in = Motion::preset(MotionPreset::FadeIn);
//!
//! // Create custom motion
//! let custom = Motion::new()
//!     .duration(Duration::Normal)
//!     .easing(MotionEasing::SpringBouncy)
//!     .build();
//! ```
//!
//! ## Gesture-Driven Animations
//!
//! Respond to user gestures with physics-based animations:
//!
//! ```rust
//! use oxide_components::animation::gesture::{GestureSpring, InertialScroll};
//!
//! // Spring that responds to drag velocity
//! let mut spring = GestureSpring::new(0.0, 100.0);
//! spring.set_velocity(500.0); // from drag gesture
//!
//! // Update each frame
//! let value = spring.update(0.016);
//! ```
//!
//! ## Variants (Declarative States)
//!
//! Define animation states declaratively:
//!
//! ```rust
//! use oxide_components::animation::motion_tokens::{Variants, VariantDefinition, Motion, Duration};
//!
//! let variants = Variants::new()
//!     .variant("hidden", VariantDefinition::new().opacity(0.0).x(-100.0))
//!     .variant("visible", VariantDefinition::new().opacity(1.0).x(0.0))
//!     .default_transition(Motion::new().duration(Duration::Normal).build());
//!
//! // Animate between states
//! let animations = variants.transition_to("hidden", "visible");
//! ```

mod easing;
pub mod interpolate;
mod state;
mod timeline;
mod transition;
mod controller;
pub mod motion_tokens;
pub mod gesture;

pub use easing::{Easing, CubicBezier};
pub use interpolate::{Interpolatable, AnimatableValue, Color8, Point2D, Size2D, Rect2D, Transform2D};
pub use state::{Animation, AnimationState, AnimationStatus, PlayDirection, FillMode, AnimationBuilder};
pub use timeline::{Timeline, TimelineEntry, TimelineEntryKind, TimelinePosition, TimelineStatus, TimelineBuilder};
pub use transition::{Transition, TransitionConfig, PropertyTransition, TransitionManager};
pub use controller::{AnimationController, AnimationId, AnimationEvent};

// Re-export motion tokens at top level for convenience
pub use motion_tokens::{
    Duration as MotionDuration,
    MotionEasing,
    MotionPreset,
    MotionConfig,
    Motion,
    SpringParams,
    SpringPreset,
    Keyframe,
    KeyframeAnimation,
    Variants,
    VariantDefinition,
    StaggerConfig,
    StaggerDirection,
    ReducedMotionMode,
};

// Re-export gesture types
pub use gesture::{
    GestureSpring,
    GestureSpring2D,
    InertialScroll,
    InertialScroll2D,
    SnapSpring,
    DecayAnimation,
    Rubberband,
    VelocityTracker,
    VelocityTracker2D,
};

/// Re-export common types for convenience
pub mod prelude {
    pub use super::{
        // Core animation types
        Animation, AnimationBuilder, AnimationController, AnimationEvent, AnimationId,
        AnimationState, AnimationStatus, PlayDirection, FillMode,

        // Easing
        Easing, CubicBezier,

        // Interpolation
        Interpolatable, AnimatableValue, Color8, Point2D, Size2D, Rect2D, Transform2D,

        // Timeline
        Timeline, TimelineBuilder, TimelineEntry, TimelineEntryKind, TimelinePosition, TimelineStatus,

        // Transitions
        Transition, TransitionConfig, PropertyTransition, TransitionManager,

        // Motion tokens
        MotionDuration, MotionEasing, MotionPreset, MotionConfig, Motion,
        SpringParams, SpringPreset,
        Keyframe, KeyframeAnimation,
        Variants, VariantDefinition,
        StaggerConfig, StaggerDirection,
        ReducedMotionMode,

        // Gesture animations
        GestureSpring, GestureSpring2D,
        InertialScroll, InertialScroll2D,
        SnapSpring, DecayAnimation, Rubberband,
        VelocityTracker, VelocityTracker2D,
    };
}
