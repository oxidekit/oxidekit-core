//! Animation system for OxideKit
//!
//! Provides animation controllers, easing curves, springs, and tweens.

pub mod controller;
pub mod curve;
pub mod spring;
pub mod tween;
pub mod value;

pub use controller::*;
pub use curve::*;
pub use spring::*;
pub use tween::*;
pub use value::*;
