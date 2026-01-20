//! Easing Curves
//!
//! This module provides standard easing curves for animations.
//! Curves transform linear progress (0.0-1.0) into eased progress.
//!
//! # Example
//!
//! ```rust
//! use oxide_animation::Curve;
//!
//! let curve = Curve::EaseOut;
//! let eased = curve.transform(0.5); // Returns eased progress
//! ```

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

// =============================================================================
// Cubic Bezier
// =============================================================================

/// A cubic bezier curve defined by two control points.
/// The curve starts at (0,0) and ends at (1,1).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CubicBezier {
    /// X coordinate of first control point
    pub x1: f32,
    /// Y coordinate of first control point
    pub y1: f32,
    /// X coordinate of second control point
    pub x2: f32,
    /// Y coordinate of second control point
    pub y2: f32,
}

impl CubicBezier {
    /// Create a new cubic bezier curve
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Standard ease curve (CSS default)
    pub const EASE: Self = Self::new(0.25, 0.1, 0.25, 1.0);

    /// Ease-in curve (slow start)
    pub const EASE_IN: Self = Self::new(0.42, 0.0, 1.0, 1.0);

    /// Ease-out curve (slow end)
    pub const EASE_OUT: Self = Self::new(0.0, 0.0, 0.58, 1.0);

    /// Ease-in-out curve (slow start and end)
    pub const EASE_IN_OUT: Self = Self::new(0.42, 0.0, 0.58, 1.0);

    /// Material Design standard curve
    pub const MATERIAL_STANDARD: Self = Self::new(0.4, 0.0, 0.2, 1.0);

    /// Material Design deceleration curve
    pub const MATERIAL_DECELERATE: Self = Self::new(0.0, 0.0, 0.2, 1.0);

    /// Material Design acceleration curve
    pub const MATERIAL_ACCELERATE: Self = Self::new(0.4, 0.0, 1.0, 1.0);

    /// Evaluate the bezier curve at progress t
    /// Uses Newton-Raphson iteration to solve for parameter
    pub fn transform(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }

        // Newton-Raphson to find x parameter
        let mut x = t;
        for _ in 0..8 {
            let x_estimate = self.bezier_x(x);
            let derivative = self.bezier_x_derivative(x);
            if derivative.abs() < 1e-6 {
                break;
            }
            x -= (x_estimate - t) / derivative;
            x = x.clamp(0.0, 1.0);
        }

        self.bezier_y(x)
    }

    fn bezier_x(&self, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        3.0 * mt2 * t * self.x1 + 3.0 * mt * t2 * self.x2 + t3
    }

    fn bezier_y(&self, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        3.0 * mt2 * t * self.y1 + 3.0 * mt * t2 * self.y2 + t3
    }

    fn bezier_x_derivative(&self, t: f32) -> f32 {
        let t2 = t * t;
        let mt = 1.0 - t;
        3.0 * mt * mt * self.x1 + 6.0 * mt * t * (self.x2 - self.x1) + 3.0 * t2 * (1.0 - self.x2)
    }

    /// Parse from CSS cubic-bezier() syntax
    pub fn from_css(s: &str) -> Option<Self> {
        let s = s.trim();
        if !s.starts_with("cubic-bezier(") || !s.ends_with(')') {
            return None;
        }
        let inner = &s[13..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() != 4 {
            return None;
        }
        Some(Self::new(
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
            parts[3].parse().ok()?,
        ))
    }

    /// Convert to CSS string format
    pub fn to_css(&self) -> String {
        format!(
            "cubic-bezier({}, {}, {}, {})",
            self.x1, self.y1, self.x2, self.y2
        )
    }
}

impl Default for CubicBezier {
    fn default() -> Self {
        Self::EASE
    }
}

// =============================================================================
// Curve Enum
// =============================================================================

/// Easing curve for animations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Curve {
    // =========================
    // Basic Curves
    // =========================
    /// Linear interpolation (no easing)
    Linear,

    // =========================
    // Quadratic Curves (power of 2)
    // =========================
    /// Quadratic ease-in
    EaseInQuad,
    /// Quadratic ease-out
    EaseOutQuad,
    /// Quadratic ease-in-out
    EaseInOutQuad,

    // =========================
    // Cubic Curves (power of 3)
    // =========================
    /// Cubic ease-in
    EaseInCubic,
    /// Cubic ease-out
    EaseOutCubic,
    /// Cubic ease-in-out
    EaseInOutCubic,

    // =========================
    // Quartic Curves (power of 4)
    // =========================
    /// Quartic ease-in
    EaseInQuart,
    /// Quartic ease-out
    EaseOutQuart,
    /// Quartic ease-in-out
    EaseInOutQuart,

    // =========================
    // Quintic Curves (power of 5)
    // =========================
    /// Quintic ease-in
    EaseInQuint,
    /// Quintic ease-out
    EaseOutQuint,
    /// Quintic ease-in-out
    EaseInOutQuint,

    // =========================
    // Sinusoidal Curves
    // =========================
    /// Sine ease-in
    EaseInSine,
    /// Sine ease-out
    EaseOutSine,
    /// Sine ease-in-out
    EaseInOutSine,

    // =========================
    // Exponential Curves
    // =========================
    /// Exponential ease-in
    EaseInExpo,
    /// Exponential ease-out
    EaseOutExpo,
    /// Exponential ease-in-out
    EaseInOutExpo,

    // =========================
    // Circular Curves
    // =========================
    /// Circular ease-in
    EaseInCirc,
    /// Circular ease-out
    EaseOutCirc,
    /// Circular ease-in-out
    EaseInOutCirc,

    // =========================
    // Back Curves (overshoot)
    // =========================
    /// Back ease-in (pulls back before starting)
    EaseInBack,
    /// Back ease-out (overshoots then settles)
    EaseOutBack,
    /// Back ease-in-out
    EaseInOutBack,

    // =========================
    // Elastic Curves (spring-like oscillation)
    // =========================
    /// Elastic ease-in
    EaseInElastic,
    /// Elastic ease-out
    EaseOutElastic,
    /// Elastic ease-in-out
    EaseInOutElastic,

    // =========================
    // Bounce Curves
    // =========================
    /// Bounce ease-in
    EaseInBounce,
    /// Bounce ease-out
    EaseOutBounce,
    /// Bounce ease-in-out
    EaseInOutBounce,

    // =========================
    // Standard CSS Curves
    // =========================
    /// Standard ease (CSS default)
    Ease,
    /// Simple ease-in
    EaseIn,
    /// Simple ease-out
    EaseOut,
    /// Simple ease-in-out
    EaseInOut,

    // =========================
    // Custom Curves
    // =========================
    /// Custom cubic bezier curve
    CubicBezier(CubicBezier),

    /// Fast out, slow in (Material decelerate)
    FastOutSlowIn,

    /// Slow out, fast in (Material accelerate)
    SlowOutFastIn,

    /// Fast out, linear in
    FastOutLinearIn,

    /// Linear out, slow in
    LinearOutSlowIn,
}

impl Curve {
    /// Transform linear progress (0.0-1.0) to eased progress
    pub fn transform(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            // Linear
            Curve::Linear => t,

            // Quadratic
            Curve::EaseInQuad => t * t,
            Curve::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Curve::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }

            // Cubic
            Curve::EaseInCubic => t * t * t,
            Curve::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Curve::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }

            // Quartic
            Curve::EaseInQuart => t * t * t * t,
            Curve::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Curve::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }

            // Quintic
            Curve::EaseInQuint => t * t * t * t * t,
            Curve::EaseOutQuint => 1.0 - (1.0 - t).powi(5),
            Curve::EaseInOutQuint => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
                }
            }

            // Sine
            Curve::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            Curve::EaseOutSine => (t * PI / 2.0).sin(),
            Curve::EaseInOutSine => -(((t * PI).cos() - 1.0) / 2.0),

            // Exponential
            Curve::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * t - 10.0)
                }
            }
            Curve::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * t)
                }
            }
            Curve::EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    2.0_f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            }

            // Circular
            Curve::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            Curve::EaseOutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Curve::EaseInOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }

            // Back
            Curve::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Curve::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Curve::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }

            // Elastic
            Curve::EaseInElastic => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -2.0_f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c4).sin()
                }
            }
            Curve::EaseOutElastic => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Curve::EaseInOutElastic => {
                let c5 = (2.0 * PI) / 4.5;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                } else {
                    (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
                }
            }

            // Bounce
            Curve::EaseOutBounce => bounce_out(t),
            Curve::EaseInBounce => 1.0 - bounce_out(1.0 - t),
            Curve::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
                }
            }

            // CSS standard curves
            Curve::Ease => CubicBezier::EASE.transform(t),
            Curve::EaseIn => CubicBezier::EASE_IN.transform(t),
            Curve::EaseOut => CubicBezier::EASE_OUT.transform(t),
            Curve::EaseInOut => CubicBezier::EASE_IN_OUT.transform(t),

            // Custom cubic bezier
            Curve::CubicBezier(bezier) => bezier.transform(t),

            // Material Design curves
            Curve::FastOutSlowIn => CubicBezier::MATERIAL_STANDARD.transform(t),
            Curve::SlowOutFastIn => CubicBezier::MATERIAL_ACCELERATE.transform(t),
            Curve::FastOutLinearIn => CubicBezier::new(0.4, 0.0, 1.0, 1.0).transform(t),
            Curve::LinearOutSlowIn => CubicBezier::MATERIAL_DECELERATE.transform(t),
        }
    }

    /// Create a custom cubic bezier curve
    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Curve::CubicBezier(CubicBezier::new(x1, y1, x2, y2))
    }

    /// Parse from string (supports CSS names and cubic-bezier())
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "linear" => Some(Curve::Linear),
            "ease" => Some(Curve::Ease),
            "ease-in" => Some(Curve::EaseIn),
            "ease-out" => Some(Curve::EaseOut),
            "ease-in-out" => Some(Curve::EaseInOut),
            "ease-in-quad" => Some(Curve::EaseInQuad),
            "ease-out-quad" => Some(Curve::EaseOutQuad),
            "ease-in-out-quad" => Some(Curve::EaseInOutQuad),
            "ease-in-cubic" => Some(Curve::EaseInCubic),
            "ease-out-cubic" => Some(Curve::EaseOutCubic),
            "ease-in-out-cubic" => Some(Curve::EaseInOutCubic),
            "ease-in-quart" => Some(Curve::EaseInQuart),
            "ease-out-quart" => Some(Curve::EaseOutQuart),
            "ease-in-out-quart" => Some(Curve::EaseInOutQuart),
            "ease-in-quint" => Some(Curve::EaseInQuint),
            "ease-out-quint" => Some(Curve::EaseOutQuint),
            "ease-in-out-quint" => Some(Curve::EaseInOutQuint),
            "ease-in-sine" => Some(Curve::EaseInSine),
            "ease-out-sine" => Some(Curve::EaseOutSine),
            "ease-in-out-sine" => Some(Curve::EaseInOutSine),
            "ease-in-expo" => Some(Curve::EaseInExpo),
            "ease-out-expo" => Some(Curve::EaseOutExpo),
            "ease-in-out-expo" => Some(Curve::EaseInOutExpo),
            "ease-in-circ" => Some(Curve::EaseInCirc),
            "ease-out-circ" => Some(Curve::EaseOutCirc),
            "ease-in-out-circ" => Some(Curve::EaseInOutCirc),
            "ease-in-back" => Some(Curve::EaseInBack),
            "ease-out-back" => Some(Curve::EaseOutBack),
            "ease-in-out-back" => Some(Curve::EaseInOutBack),
            "ease-in-elastic" => Some(Curve::EaseInElastic),
            "ease-out-elastic" => Some(Curve::EaseOutElastic),
            "ease-in-out-elastic" => Some(Curve::EaseInOutElastic),
            "ease-in-bounce" => Some(Curve::EaseInBounce),
            "ease-out-bounce" => Some(Curve::EaseOutBounce),
            "ease-in-out-bounce" => Some(Curve::EaseInOutBounce),
            _ if s.starts_with("cubic-bezier(") => {
                CubicBezier::from_css(&s).map(Curve::CubicBezier)
            }
            _ => None,
        }
    }

    /// Get the inverse curve (flipped horizontally)
    pub fn flipped(&self) -> Self {
        match self {
            Curve::EaseIn => Curve::EaseOut,
            Curve::EaseOut => Curve::EaseIn,
            Curve::EaseInQuad => Curve::EaseOutQuad,
            Curve::EaseOutQuad => Curve::EaseInQuad,
            Curve::EaseInCubic => Curve::EaseOutCubic,
            Curve::EaseOutCubic => Curve::EaseInCubic,
            Curve::EaseInQuart => Curve::EaseOutQuart,
            Curve::EaseOutQuart => Curve::EaseInQuart,
            Curve::EaseInQuint => Curve::EaseOutQuint,
            Curve::EaseOutQuint => Curve::EaseInQuint,
            Curve::EaseInSine => Curve::EaseOutSine,
            Curve::EaseOutSine => Curve::EaseInSine,
            Curve::EaseInExpo => Curve::EaseOutExpo,
            Curve::EaseOutExpo => Curve::EaseInExpo,
            Curve::EaseInCirc => Curve::EaseOutCirc,
            Curve::EaseOutCirc => Curve::EaseInCirc,
            Curve::EaseInBack => Curve::EaseOutBack,
            Curve::EaseOutBack => Curve::EaseInBack,
            Curve::EaseInElastic => Curve::EaseOutElastic,
            Curve::EaseOutElastic => Curve::EaseInElastic,
            Curve::EaseInBounce => Curve::EaseOutBounce,
            Curve::EaseOutBounce => Curve::EaseInBounce,
            Curve::CubicBezier(b) => {
                Curve::CubicBezier(CubicBezier::new(1.0 - b.x2, 1.0 - b.y2, 1.0 - b.x1, 1.0 - b.y1))
            }
            other => *other,
        }
    }
}

impl Default for Curve {
    fn default() -> Self {
        Curve::Ease
    }
}

/// Helper for bounce easing
fn bounce_out(t: f32) -> f32 {
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let curve = Curve::Linear;
        assert!((curve.transform(0.0) - 0.0).abs() < 0.001);
        assert!((curve.transform(0.5) - 0.5).abs() < 0.001);
        assert!((curve.transform(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_in_quad() {
        let curve = Curve::EaseInQuad;
        assert!((curve.transform(0.0) - 0.0).abs() < 0.001);
        assert!((curve.transform(0.5) - 0.25).abs() < 0.001);
        assert!((curve.transform(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_out_quad() {
        let curve = Curve::EaseOutQuad;
        assert!((curve.transform(0.0) - 0.0).abs() < 0.001);
        assert!((curve.transform(0.5) - 0.75).abs() < 0.001);
        assert!((curve.transform(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cubic_bezier() {
        let curve = Curve::CubicBezier(CubicBezier::EASE_IN_OUT);
        assert!((curve.transform(0.0) - 0.0).abs() < 0.001);
        assert!((curve.transform(1.0) - 1.0).abs() < 0.001);
        // Midpoint should be close to 0.5
        assert!((curve.transform(0.5) - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_cubic_bezier_parse() {
        let bezier = CubicBezier::from_css("cubic-bezier(0.4, 0, 0.2, 1)").unwrap();
        assert!((bezier.x1 - 0.4).abs() < 0.001);
        assert!((bezier.y1 - 0.0).abs() < 0.001);
        assert!((bezier.x2 - 0.2).abs() < 0.001);
        assert!((bezier.y2 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_curve_from_str() {
        assert_eq!(Curve::from_str("linear"), Some(Curve::Linear));
        assert_eq!(Curve::from_str("ease-in"), Some(Curve::EaseIn));
        assert_eq!(Curve::from_str("ease-out-bounce"), Some(Curve::EaseOutBounce));
    }

    #[test]
    fn test_elastic_overshoots() {
        let curve = Curve::EaseOutElastic;
        // Elastic curves can overshoot past 1.0
        let value = curve.transform(0.3);
        assert!(value > 0.0);
    }

    #[test]
    fn test_back_undershoots() {
        let curve = Curve::EaseInBack;
        // Back curves go negative at the start
        let value = curve.transform(0.1);
        assert!(value < 0.0);
    }

    #[test]
    fn test_bounce() {
        let curve = Curve::EaseOutBounce;
        assert!((curve.transform(0.0) - 0.0).abs() < 0.001);
        assert!((curve.transform(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_curve_flipped() {
        let ease_in = Curve::EaseInQuad;
        let ease_out = ease_in.flipped();
        assert_eq!(ease_out, Curve::EaseOutQuad);
    }

    #[test]
    fn test_all_curves_boundary_values() {
        let curves = [
            Curve::Linear,
            Curve::EaseInQuad,
            Curve::EaseOutQuad,
            Curve::EaseInOutQuad,
            Curve::EaseInCubic,
            Curve::EaseOutCubic,
            Curve::EaseInOutCubic,
            Curve::EaseInSine,
            Curve::EaseOutSine,
            Curve::EaseInOutSine,
            Curve::EaseInExpo,
            Curve::EaseOutExpo,
            Curve::EaseInOutExpo,
            Curve::EaseInCirc,
            Curve::EaseOutCirc,
            Curve::EaseInOutCirc,
            Curve::EaseOutBounce,
            Curve::EaseInBounce,
            Curve::EaseInOutBounce,
            Curve::Ease,
            Curve::FastOutSlowIn,
        ];

        for curve in curves {
            // All curves should return 0.0 at t=0.0
            assert!(
                (curve.transform(0.0) - 0.0).abs() < 0.001,
                "{:?} failed at t=0.0",
                curve
            );
            // All curves should return 1.0 at t=1.0
            assert!(
                (curve.transform(1.0) - 1.0).abs() < 0.001,
                "{:?} failed at t=1.0",
                curve
            );
        }
    }

    #[test]
    fn test_sine_curves() {
        let ease_in = Curve::EaseInSine;
        let ease_out = Curve::EaseOutSine;
        let ease_in_out = Curve::EaseInOutSine;

        // Ease-in should be slower at start
        assert!(ease_in.transform(0.25) < 0.25);
        // Ease-out should be faster at start
        assert!(ease_out.transform(0.25) > 0.25);
        // Ease-in-out at midpoint should be close to 0.5
        assert!((ease_in_out.transform(0.5) - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_expo_curves() {
        let ease_in = Curve::EaseInExpo;
        let ease_out = Curve::EaseOutExpo;

        // Exponential ease-in should be very slow at start
        assert!(ease_in.transform(0.2) < 0.1);
        // Exponential ease-out should be very fast at start
        assert!(ease_out.transform(0.2) > 0.5);
    }
}
