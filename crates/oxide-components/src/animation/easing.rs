//! Easing Functions
//!
//! Provides standard easing functions for animations.
//! These map an input progress value (0.0-1.0) to an output value (0.0-1.0).

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// A cubic bezier curve defined by two control points.
/// The curve starts at (0,0) and ends at (1,1).
/// Control points are (x1, y1) and (x2, y2).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezier {
    /// Create a new cubic bezier with control points
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Standard ease curve
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

    /// Evaluate the bezier curve at progress t (0.0-1.0)
    /// Uses Newton-Raphson iteration to find x, then evaluates y
    pub fn evaluate(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }

        // Find the parameter that gives us x = t using Newton-Raphson
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

    /// Calculate x value of bezier at parameter t
    fn bezier_x(&self, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        3.0 * mt2 * t * self.x1 + 3.0 * mt * t2 * self.x2 + t3
    }

    /// Calculate y value of bezier at parameter t
    fn bezier_y(&self, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        3.0 * mt2 * t * self.y1 + 3.0 * mt * t2 * self.y2 + t3
    }

    /// Derivative of x with respect to t
    fn bezier_x_derivative(&self, t: f32) -> f32 {
        let t2 = t * t;
        let mt = 1.0 - t;

        3.0 * mt * mt * self.x1 + 6.0 * mt * t * (self.x2 - self.x1) + 3.0 * t2 * (1.0 - self.x2)
    }

    /// Parse from CSS cubic-bezier string: "cubic-bezier(x1, y1, x2, y2)"
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

        let x1: f32 = parts[0].parse().ok()?;
        let y1: f32 = parts[1].parse().ok()?;
        let x2: f32 = parts[2].parse().ok()?;
        let y2: f32 = parts[3].parse().ok()?;

        Some(Self::new(x1, y1, x2, y2))
    }

    /// Convert to CSS string format
    pub fn to_css(&self) -> String {
        format!("cubic-bezier({}, {}, {}, {})", self.x1, self.y1, self.x2, self.y2)
    }
}

impl Default for CubicBezier {
    fn default() -> Self {
        Self::EASE
    }
}

/// Easing function for animations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Easing {
    /// Linear interpolation (no easing)
    Linear,

    /// Quadratic ease-in (slow start)
    EaseInQuad,
    /// Quadratic ease-out (slow end)
    EaseOutQuad,
    /// Quadratic ease-in-out
    EaseInOutQuad,

    /// Cubic ease-in
    EaseInCubic,
    /// Cubic ease-out
    EaseOutCubic,
    /// Cubic ease-in-out
    EaseInOutCubic,

    /// Quartic ease-in
    EaseInQuart,
    /// Quartic ease-out
    EaseOutQuart,
    /// Quartic ease-in-out
    EaseInOutQuart,

    /// Quintic ease-in
    EaseInQuint,
    /// Quintic ease-out
    EaseOutQuint,
    /// Quintic ease-in-out
    EaseInOutQuint,

    /// Sine ease-in
    EaseInSine,
    /// Sine ease-out
    EaseOutSine,
    /// Sine ease-in-out
    EaseInOutSine,

    /// Exponential ease-in
    EaseInExpo,
    /// Exponential ease-out
    EaseOutExpo,
    /// Exponential ease-in-out
    EaseInOutExpo,

    /// Circular ease-in
    EaseInCirc,
    /// Circular ease-out
    EaseOutCirc,
    /// Circular ease-in-out
    EaseInOutCirc,

    /// Elastic ease-in (spring effect)
    EaseInElastic,
    /// Elastic ease-out
    EaseOutElastic,
    /// Elastic ease-in-out
    EaseInOutElastic,

    /// Back ease-in (overshoot at start)
    EaseInBack,
    /// Back ease-out (overshoot at end)
    EaseOutBack,
    /// Back ease-in-out
    EaseInOutBack,

    /// Bounce ease-in
    EaseInBounce,
    /// Bounce ease-out
    EaseOutBounce,
    /// Bounce ease-in-out
    EaseInOutBounce,

    /// Standard ease (CSS default)
    Ease,
    /// Simple ease-in (alias for EaseInQuad)
    EaseIn,
    /// Simple ease-out (alias for EaseOutQuad)
    EaseOut,
    /// Simple ease-in-out (alias for EaseInOutQuad)
    EaseInOut,

    /// Custom cubic bezier curve
    CubicBezier(CubicBezier),

    /// Spring physics-based easing
    Spring {
        /// Stiffness (default: 100)
        stiffness: f32,
        /// Damping (default: 10)
        damping: f32,
        /// Mass (default: 1)
        mass: f32,
    },
}

impl Easing {
    /// Evaluate the easing function at progress t (0.0-1.0)
    /// Returns the eased value (typically 0.0-1.0, but can overshoot for elastic/back)
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Easing::Linear => t,

            // Quadratic
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }

            // Cubic
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }

            // Quartic
            Easing::EaseInQuart => t * t * t * t,
            Easing::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Easing::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }

            // Quintic
            Easing::EaseInQuint => t * t * t * t * t,
            Easing::EaseOutQuint => 1.0 - (1.0 - t).powi(5),
            Easing::EaseInOutQuint => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
                }
            }

            // Sine
            Easing::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            Easing::EaseOutSine => (t * PI / 2.0).sin(),
            Easing::EaseInOutSine => -(((t * PI).cos() - 1.0) / 2.0),

            // Exponential
            Easing::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * t - 10.0)
                }
            }
            Easing::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * t)
                }
            }
            Easing::EaseInOutExpo => {
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
            Easing::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            Easing::EaseOutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Easing::EaseInOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }

            // Elastic
            Easing::EaseInElastic => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -2.0_f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c4).sin()
                }
            }
            Easing::EaseOutElastic => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Easing::EaseInOutElastic => {
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

            // Back (overshoot)
            Easing::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Easing::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Easing::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }

            // Bounce
            Easing::EaseOutBounce => ease_out_bounce(t),
            Easing::EaseInBounce => 1.0 - ease_out_bounce(1.0 - t),
            Easing::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
                }
            }

            // CSS standard easings (using cubic bezier)
            Easing::Ease => CubicBezier::EASE.evaluate(t),
            Easing::EaseIn => CubicBezier::EASE_IN.evaluate(t),
            Easing::EaseOut => CubicBezier::EASE_OUT.evaluate(t),
            Easing::EaseInOut => CubicBezier::EASE_IN_OUT.evaluate(t),

            // Custom cubic bezier
            Easing::CubicBezier(bezier) => bezier.evaluate(t),

            // Spring physics
            Easing::Spring { stiffness, damping, mass } => {
                spring_evaluate(t, *stiffness, *damping, *mass)
            }
        }
    }

    /// Create a custom cubic bezier easing
    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Easing::CubicBezier(CubicBezier::new(x1, y1, x2, y2))
    }

    /// Create a spring easing with default parameters
    pub fn spring() -> Self {
        Easing::Spring {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
        }
    }

    /// Create a spring easing with custom parameters
    pub fn spring_with(stiffness: f32, damping: f32, mass: f32) -> Self {
        Easing::Spring { stiffness, damping, mass }
    }

    /// Parse from string (CSS-like syntax)
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "linear" => Some(Easing::Linear),
            "ease" => Some(Easing::Ease),
            "ease-in" => Some(Easing::EaseIn),
            "ease-out" => Some(Easing::EaseOut),
            "ease-in-out" => Some(Easing::EaseInOut),
            "ease-in-quad" => Some(Easing::EaseInQuad),
            "ease-out-quad" => Some(Easing::EaseOutQuad),
            "ease-in-out-quad" => Some(Easing::EaseInOutQuad),
            "ease-in-cubic" => Some(Easing::EaseInCubic),
            "ease-out-cubic" => Some(Easing::EaseOutCubic),
            "ease-in-out-cubic" => Some(Easing::EaseInOutCubic),
            "ease-in-quart" => Some(Easing::EaseInQuart),
            "ease-out-quart" => Some(Easing::EaseOutQuart),
            "ease-in-out-quart" => Some(Easing::EaseInOutQuart),
            "ease-in-quint" => Some(Easing::EaseInQuint),
            "ease-out-quint" => Some(Easing::EaseOutQuint),
            "ease-in-out-quint" => Some(Easing::EaseInOutQuint),
            "ease-in-sine" => Some(Easing::EaseInSine),
            "ease-out-sine" => Some(Easing::EaseOutSine),
            "ease-in-out-sine" => Some(Easing::EaseInOutSine),
            "ease-in-expo" => Some(Easing::EaseInExpo),
            "ease-out-expo" => Some(Easing::EaseOutExpo),
            "ease-in-out-expo" => Some(Easing::EaseInOutExpo),
            "ease-in-circ" => Some(Easing::EaseInCirc),
            "ease-out-circ" => Some(Easing::EaseOutCirc),
            "ease-in-out-circ" => Some(Easing::EaseInOutCirc),
            "ease-in-elastic" => Some(Easing::EaseInElastic),
            "ease-out-elastic" => Some(Easing::EaseOutElastic),
            "ease-in-out-elastic" => Some(Easing::EaseInOutElastic),
            "ease-in-back" => Some(Easing::EaseInBack),
            "ease-out-back" => Some(Easing::EaseOutBack),
            "ease-in-out-back" => Some(Easing::EaseInOutBack),
            "ease-in-bounce" => Some(Easing::EaseInBounce),
            "ease-out-bounce" => Some(Easing::EaseOutBounce),
            "ease-in-out-bounce" => Some(Easing::EaseInOutBounce),
            _ if s.starts_with("cubic-bezier(") => {
                CubicBezier::from_css(&s).map(Easing::CubicBezier)
            }
            _ => None,
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::Ease
    }
}

/// Helper function for bounce easing
fn ease_out_bounce(t: f32) -> f32 {
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

/// Spring physics evaluation
/// Uses a simplified damped harmonic oscillator model
fn spring_evaluate(t: f32, stiffness: f32, damping: f32, mass: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }

    // Angular frequency
    let omega = (stiffness / mass).sqrt();
    // Damping ratio
    let zeta = damping / (2.0 * (stiffness * mass).sqrt());

    if zeta < 1.0 {
        // Underdamped (oscillates)
        let omega_d = omega * (1.0 - zeta * zeta).sqrt();
        let envelope = (-zeta * omega * t).exp();
        let oscillation = (omega_d * t).cos() + (zeta * omega / omega_d) * (omega_d * t).sin();
        1.0 - envelope * oscillation
    } else if zeta == 1.0 {
        // Critically damped
        1.0 - (1.0 + omega * t) * (-omega * t).exp()
    } else {
        // Overdamped
        let s1 = -omega * (zeta - (zeta * zeta - 1.0).sqrt());
        let s2 = -omega * (zeta + (zeta * zeta - 1.0).sqrt());
        let c2 = s1 / (s1 - s2);
        let c1 = 1.0 - c2;
        1.0 - c1 * (s1 * t).exp() - c2 * (s2 * t).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let easing = Easing::Linear;
        assert!((easing.evaluate(0.0) - 0.0).abs() < 0.001);
        assert!((easing.evaluate(0.5) - 0.5).abs() < 0.001);
        assert!((easing.evaluate(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_in_quad() {
        let easing = Easing::EaseInQuad;
        assert!((easing.evaluate(0.0) - 0.0).abs() < 0.001);
        assert!((easing.evaluate(0.5) - 0.25).abs() < 0.001);
        assert!((easing.evaluate(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_out_quad() {
        let easing = Easing::EaseOutQuad;
        assert!((easing.evaluate(0.0) - 0.0).abs() < 0.001);
        assert!((easing.evaluate(0.5) - 0.75).abs() < 0.001);
        assert!((easing.evaluate(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cubic_bezier() {
        let bezier = CubicBezier::EASE_IN_OUT;
        assert!((bezier.evaluate(0.0) - 0.0).abs() < 0.001);
        assert!((bezier.evaluate(1.0) - 1.0).abs() < 0.001);
        // Midpoint should be close to 0.5
        assert!((bezier.evaluate(0.5) - 0.5).abs() < 0.1);
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
    fn test_easing_from_str() {
        assert_eq!(Easing::from_str("linear"), Some(Easing::Linear));
        assert_eq!(Easing::from_str("ease-in"), Some(Easing::EaseIn));
        assert_eq!(Easing::from_str("ease-out-bounce"), Some(Easing::EaseOutBounce));
    }

    #[test]
    fn test_spring() {
        let spring = Easing::spring();
        assert!((spring.evaluate(0.0) - 0.0).abs() < 0.001);
        // Spring should approach 1.0 at t=1.0
        assert!((spring.evaluate(1.0) - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_elastic_overshoots() {
        let elastic = Easing::EaseOutElastic;
        // Elastic should overshoot
        let mid = elastic.evaluate(0.3);
        assert!(mid > 0.0);
    }

    #[test]
    fn test_back_overshoots() {
        let back = Easing::EaseInBack;
        // Back should go negative at the start
        let early = back.evaluate(0.1);
        assert!(early < 0.0);
    }
}
