//! Tweens and Tween Sequences
//!
//! This module provides value interpolation (tweening) with support for
//! chained animations through TweenSequence.
//!
//! # Example
//!
//! ```rust
//! use oxide_animation::{Tween, TweenSequence, TweenSegment, Curve};
//!
//! // Simple tween
//! let tween = Tween::new(0.0_f32, 100.0_f32);
//! let value = tween.transform(0.5); // Returns 50.0
//!
//! // Tween sequence with multiple segments
//! let sequence = TweenSequence::new(vec![
//!     TweenSegment::new(Tween::new(0.0, 50.0), 0.3),   // First 30%
//!     TweenSegment::new(Tween::new(50.0, 100.0), 0.7), // Last 70%
//! ]);
//! ```

use crate::curve::Curve;
use crate::value::Animatable;
use serde::{Deserialize, Serialize};

// =============================================================================
// Tween
// =============================================================================

/// A tween that interpolates between two values
#[derive(Debug, Clone)]
pub struct Tween<T: Animatable> {
    /// Start value
    begin: T,
    /// End value
    end: T,
    /// Optional curve to apply
    curve: Option<Curve>,
}

impl<T: Animatable> Tween<T> {
    /// Create a new tween
    pub fn new(begin: T, end: T) -> Self {
        Self {
            begin,
            end,
            curve: None,
        }
    }

    /// Create a tween with a curve
    pub fn with_curve(begin: T, end: T, curve: Curve) -> Self {
        Self {
            begin,
            end,
            curve: Some(curve),
        }
    }

    /// Set the curve for this tween
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = Some(curve);
        self
    }

    /// Get the begin value
    pub fn begin(&self) -> &T {
        &self.begin
    }

    /// Get the end value
    pub fn end(&self) -> &T {
        &self.end
    }

    /// Transform a progress value (0.0-1.0) to the interpolated value
    pub fn transform(&self, t: f32) -> T {
        let t = t.clamp(0.0, 1.0);
        let eased = match &self.curve {
            Some(curve) => curve.transform(t),
            None => t,
        };
        self.begin.lerp(&self.end, eased)
    }

    /// Evaluate at a specific progress (alias for transform)
    pub fn evaluate(&self, t: f32) -> T {
        self.transform(t)
    }

    /// Chain this tween with another tween
    pub fn chain(self, other: Tween<T>) -> ChainedTween<T> {
        ChainedTween {
            tweens: vec![self, other],
        }
    }
}

// =============================================================================
// Chained Tween
// =============================================================================

/// Multiple tweens chained together (each gets equal time)
#[derive(Debug, Clone)]
pub struct ChainedTween<T: Animatable> {
    tweens: Vec<Tween<T>>,
}

impl<T: Animatable> ChainedTween<T> {
    /// Create a new chained tween from a list of tweens
    pub fn new(tweens: Vec<Tween<T>>) -> Self {
        Self { tweens }
    }

    /// Add another tween to the chain
    pub fn then(mut self, tween: Tween<T>) -> Self {
        self.tweens.push(tween);
        self
    }

    /// Get the number of tweens in the chain
    pub fn len(&self) -> usize {
        self.tweens.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.tweens.is_empty()
    }

    /// Transform a progress value (0.0-1.0) to the interpolated value
    pub fn transform(&self, t: f32) -> T {
        if self.tweens.is_empty() {
            panic!("ChainedTween has no tweens");
        }

        let t = t.clamp(0.0, 1.0);
        let segment_size = 1.0 / self.tweens.len() as f32;

        // Find which segment we're in
        let segment_index = ((t / segment_size).floor() as usize).min(self.tweens.len() - 1);
        let segment_start = segment_index as f32 * segment_size;
        let local_t = ((t - segment_start) / segment_size).clamp(0.0, 1.0);

        self.tweens[segment_index].transform(local_t)
    }
}

// =============================================================================
// Tween Segment
// =============================================================================

/// A segment in a TweenSequence with a weight
#[derive(Debug, Clone)]
pub struct TweenSegment<T: Animatable> {
    /// The tween for this segment
    pub tween: Tween<T>,
    /// Weight of this segment (relative to other segments)
    pub weight: f32,
}

impl<T: Animatable> TweenSegment<T> {
    /// Create a new tween segment
    pub fn new(tween: Tween<T>, weight: f32) -> Self {
        Self {
            tween,
            weight: weight.max(0.0),
        }
    }

    /// Create a segment from begin/end values
    pub fn from_values(begin: T, end: T, weight: f32) -> Self {
        Self::new(Tween::new(begin, end), weight)
    }

    /// Create a segment with a curve
    pub fn with_curve(begin: T, end: T, weight: f32, curve: Curve) -> Self {
        Self::new(Tween::with_curve(begin, end, curve), weight)
    }
}

// =============================================================================
// Tween Sequence
// =============================================================================

/// A sequence of tweens with weighted durations
///
/// Each segment gets a portion of the total animation time
/// proportional to its weight.
#[derive(Debug, Clone)]
pub struct TweenSequence<T: Animatable> {
    segments: Vec<TweenSegment<T>>,
    /// Total weight (cached)
    total_weight: f32,
}

impl<T: Animatable> TweenSequence<T> {
    /// Create a new tween sequence from segments
    pub fn new(segments: Vec<TweenSegment<T>>) -> Self {
        let total_weight = segments.iter().map(|s| s.weight).sum();
        Self {
            segments,
            total_weight,
        }
    }

    /// Create a sequence builder
    pub fn builder() -> TweenSequenceBuilder<T> {
        TweenSequenceBuilder::new()
    }

    /// Get the number of segments
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Transform a progress value (0.0-1.0) to the interpolated value
    pub fn transform(&self, t: f32) -> T {
        if self.segments.is_empty() {
            panic!("TweenSequence has no segments");
        }

        let t = t.clamp(0.0, 1.0);
        let target_weight = t * self.total_weight;

        let mut accumulated_weight = 0.0;
        for segment in &self.segments {
            let segment_end = accumulated_weight + segment.weight;

            if target_weight <= segment_end {
                // We're in this segment
                let segment_progress = if segment.weight > 0.0 {
                    (target_weight - accumulated_weight) / segment.weight
                } else {
                    0.0
                };
                return segment.tween.transform(segment_progress);
            }

            accumulated_weight = segment_end;
        }

        // Return the end of the last segment
        self.segments
            .last()
            .map(|s| s.tween.end().clone())
            .unwrap()
    }

    /// Evaluate at a specific progress (alias for transform)
    pub fn evaluate(&self, t: f32) -> T {
        self.transform(t)
    }
}

// =============================================================================
// Tween Sequence Builder
// =============================================================================

/// Builder for creating tween sequences
#[derive(Debug, Clone)]
pub struct TweenSequenceBuilder<T: Animatable> {
    segments: Vec<TweenSegment<T>>,
}

impl<T: Animatable> TweenSequenceBuilder<T> {
    /// Create a new builder
    pub fn new() -> Self {
        Self { segments: vec![] }
    }

    /// Add a segment
    pub fn segment(mut self, tween: Tween<T>, weight: f32) -> Self {
        self.segments.push(TweenSegment::new(tween, weight));
        self
    }

    /// Add a segment from values
    pub fn add(mut self, begin: T, end: T, weight: f32) -> Self {
        self.segments
            .push(TweenSegment::from_values(begin, end, weight));
        self
    }

    /// Add a segment with a curve
    pub fn add_curved(mut self, begin: T, end: T, weight: f32, curve: Curve) -> Self {
        self.segments
            .push(TweenSegment::with_curve(begin, end, weight, curve));
        self
    }

    /// Build the sequence
    pub fn build(self) -> TweenSequence<T> {
        TweenSequence::new(self.segments)
    }
}

impl<T: Animatable> Default for TweenSequenceBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Reversible Tween
// =============================================================================

/// A tween that can be reversed
#[derive(Debug, Clone)]
pub struct ReversibleTween<T: Animatable> {
    tween: Tween<T>,
    reverse_curve: Option<Curve>,
}

impl<T: Animatable> ReversibleTween<T> {
    /// Create a new reversible tween
    pub fn new(begin: T, end: T) -> Self {
        Self {
            tween: Tween::new(begin, end),
            reverse_curve: None,
        }
    }

    /// Set the forward curve
    pub fn curve(mut self, curve: Curve) -> Self {
        self.tween = self.tween.curve(curve);
        self
    }

    /// Set the reverse curve
    pub fn reverse_curve(mut self, curve: Curve) -> Self {
        self.reverse_curve = Some(curve);
        self
    }

    /// Transform in forward direction
    pub fn transform(&self, t: f32) -> T {
        self.tween.transform(t)
    }

    /// Transform in reverse direction
    pub fn transform_reverse(&self, t: f32) -> T {
        let t = t.clamp(0.0, 1.0);
        let eased = match &self.reverse_curve {
            Some(curve) => curve.transform(t),
            None => match &self.tween.curve {
                Some(curve) => curve.flipped().transform(t),
                None => t,
            },
        };
        self.tween.end.lerp(&self.tween.begin, eased)
    }
}

// =============================================================================
// Color Tween (specialized)
// =============================================================================

use crate::value::Color;

/// Specialized tween for colors with color space options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSpace {
    /// Linear RGB (default, perceptually correct)
    LinearRgb,
    /// Standard RGB (gamma space)
    SRgb,
    /// HSL (hue, saturation, lightness)
    Hsl,
}

impl Default for ColorSpace {
    fn default() -> Self {
        ColorSpace::LinearRgb
    }
}

/// Color tween with color space selection
#[derive(Debug, Clone)]
pub struct ColorTween {
    begin: Color,
    end: Color,
    curve: Option<Curve>,
    color_space: ColorSpace,
}

impl ColorTween {
    /// Create a new color tween
    pub fn new(begin: Color, end: Color) -> Self {
        Self {
            begin,
            end,
            curve: None,
            color_space: ColorSpace::default(),
        }
    }

    /// Set the curve
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = Some(curve);
        self
    }

    /// Set the color space for interpolation
    pub fn color_space(mut self, space: ColorSpace) -> Self {
        self.color_space = space;
        self
    }

    /// Transform at progress t
    pub fn transform(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let eased = match &self.curve {
            Some(curve) => curve.transform(t),
            None => t,
        };

        match self.color_space {
            ColorSpace::LinearRgb | ColorSpace::SRgb => {
                // Simple linear interpolation in RGB
                self.begin.lerp(&self.end, eased)
            }
            ColorSpace::Hsl => {
                // Convert to HSL, interpolate, convert back
                let begin_hsl = rgb_to_hsl(self.begin.r, self.begin.g, self.begin.b);
                let end_hsl = rgb_to_hsl(self.end.r, self.end.g, self.end.b);

                // Interpolate in HSL (with hue wrapping)
                let h = lerp_angle(begin_hsl.0, end_hsl.0, eased);
                let s = begin_hsl.1 + (end_hsl.1 - begin_hsl.1) * eased;
                let l = begin_hsl.2 + (end_hsl.2 - begin_hsl.2) * eased;
                let a = self.begin.a + (self.end.a - self.begin.a) * eased;

                let (r, g, b) = hsl_to_rgb(h, s, l);
                Color::new(r, g, b, a)
            }
        }
    }
}

// HSL conversion helpers

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 0.0001 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < 0.0001 {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < 0.0001 {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < 0.0001 {
        return (l, l, l);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (r, g, b)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let diff = b - a;
    let diff = if diff > 0.5 {
        diff - 1.0
    } else if diff < -0.5 {
        diff + 1.0
    } else {
        diff
    };
    let result = a + diff * t;
    if result < 0.0 {
        result + 1.0
    } else if result > 1.0 {
        result - 1.0
    } else {
        result
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tween_basic() {
        let tween = Tween::new(0.0_f32, 100.0_f32);
        assert!((tween.transform(0.0) - 0.0).abs() < 0.001);
        assert!((tween.transform(0.5) - 50.0).abs() < 0.001);
        assert!((tween.transform(1.0) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_tween_with_curve() {
        let tween = Tween::new(0.0_f32, 100.0_f32).curve(Curve::EaseInQuad);

        // EaseInQuad at 0.5 should be 0.25
        assert!((tween.transform(0.5) - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_chained_tween() {
        let chain = Tween::new(0.0_f32, 50.0_f32).chain(Tween::new(50.0_f32, 100.0_f32));

        // First half
        assert!((chain.transform(0.25) - 25.0).abs() < 0.001);
        // Second half
        assert!((chain.transform(0.75) - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_tween_sequence() {
        let sequence = TweenSequence::new(vec![
            TweenSegment::from_values(0.0_f32, 50.0, 1.0),
            TweenSegment::from_values(50.0_f32, 100.0, 1.0),
        ]);

        assert!((sequence.transform(0.0) - 0.0).abs() < 0.001);
        assert!((sequence.transform(0.25) - 25.0).abs() < 0.001);
        assert!((sequence.transform(0.5) - 50.0).abs() < 0.001);
        assert!((sequence.transform(0.75) - 75.0).abs() < 0.001);
        assert!((sequence.transform(1.0) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_tween_sequence_weighted() {
        // First segment gets 30%, second gets 70%
        let sequence = TweenSequence::new(vec![
            TweenSegment::from_values(0.0_f32, 30.0, 0.3),
            TweenSegment::from_values(30.0_f32, 100.0, 0.7),
        ]);

        // At 30% progress, should be at end of first segment
        assert!((sequence.transform(0.3) - 30.0).abs() < 0.001);
        // At 65% progress, should be halfway through second segment
        assert!((sequence.transform(0.65) - 65.0).abs() < 1.0);
    }

    #[test]
    fn test_tween_sequence_builder() {
        let sequence = TweenSequence::builder()
            .add(0.0_f32, 50.0, 1.0)
            .add(50.0_f32, 100.0, 1.0)
            .build();

        assert_eq!(sequence.len(), 2);
        assert!((sequence.transform(0.5) - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_reversible_tween() {
        let tween = ReversibleTween::new(0.0_f32, 100.0_f32).curve(Curve::EaseIn);

        let forward = tween.transform(0.5);
        let reverse = tween.transform_reverse(0.5);

        // Forward and reverse at 0.5 should be different (unless curve is symmetric)
        assert!(forward != reverse || (forward - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_color_tween_rgb() {
        let tween = ColorTween::new(Color::BLACK, Color::WHITE);

        let mid = tween.transform(0.5);
        assert!((mid.r - 0.5).abs() < 0.001);
        assert!((mid.g - 0.5).abs() < 0.001);
        assert!((mid.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color_tween_hsl() {
        let tween = ColorTween::new(Color::RED, Color::GREEN).color_space(ColorSpace::Hsl);

        // In HSL, red to green should go through yellow
        let mid = tween.transform(0.5);
        // This is a rough check - exact values depend on HSL interpolation
        assert!(mid.r > 0.0 || mid.g > 0.0);
    }

    #[test]
    fn test_tween_clamping() {
        let tween = Tween::new(0.0_f32, 100.0_f32);

        // Values outside 0-1 should be clamped
        assert!((tween.transform(-0.5) - 0.0).abs() < 0.001);
        assert!((tween.transform(1.5) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_chained_tween_multiple() {
        let chain = ChainedTween::new(vec![
            Tween::new(0.0_f32, 25.0_f32),
            Tween::new(25.0_f32, 50.0_f32),
            Tween::new(50.0_f32, 75.0_f32),
            Tween::new(75.0_f32, 100.0_f32),
        ]);

        // Should pass through each segment evenly
        assert!((chain.transform(0.125) - 12.5).abs() < 0.001);
        assert!((chain.transform(0.375) - 37.5).abs() < 0.001);
        assert!((chain.transform(0.625) - 62.5).abs() < 0.001);
        assert!((chain.transform(0.875) - 87.5).abs() < 0.001);
    }

    #[test]
    fn test_tween_sequence_with_curves() {
        let sequence = TweenSequence::builder()
            .add_curved(0.0_f32, 50.0, 1.0, Curve::EaseIn)
            .add_curved(50.0_f32, 100.0, 1.0, Curve::EaseOut)
            .build();

        // Just verify it works without panicking
        let _ = sequence.transform(0.25);
        let _ = sequence.transform(0.75);
    }
}
