//! Progress Indicators
//!
//! This module provides linear and circular progress indicators with support for:
//! - Determinate mode (0-100%)
//! - Indeterminate mode (animated)
//! - Buffer mode (for streaming content)
//!
//! # Accessibility
//!
//! Progress indicators include ARIA attributes:
//! - `role="progressbar"`
//! - `aria-valuenow`, `aria-valuemin`, `aria-valuemax` for determinate mode
//! - `aria-busy="true"` for indeterminate mode

mod circular;
mod linear;

pub use circular::*;
pub use linear::*;

use serde::{Deserialize, Serialize};

/// Progress indicator mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProgressMode {
    /// Shows specific progress value (0.0 to 1.0)
    #[default]
    Determinate,
    /// Animated indicator for unknown duration
    Indeterminate,
    /// Shows both progress and buffer (for streaming)
    Buffer,
}

impl ProgressMode {
    /// Returns true if this mode requires a progress value
    pub fn requires_value(&self) -> bool {
        matches!(self, ProgressMode::Determinate | ProgressMode::Buffer)
    }

    /// Returns the ARIA description for this mode
    pub fn aria_description(&self) -> &'static str {
        match self {
            ProgressMode::Determinate => "Progress indicator showing completion percentage",
            ProgressMode::Indeterminate => "Loading in progress",
            ProgressMode::Buffer => "Progress indicator with buffering",
        }
    }
}

/// Color scheme for progress indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProgressColor {
    /// Primary theme color
    #[default]
    Primary,
    /// Secondary theme color
    Secondary,
    /// Success/green color
    Success,
    /// Warning/yellow color
    Warning,
    /// Error/red color
    Error,
    /// Custom color (rgba values)
    Custom {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
}

impl ProgressColor {
    /// Create a custom color from RGBA values
    pub fn custom(r: u8, g: u8, b: u8, a: u8) -> Self {
        ProgressColor::Custom { r, g, b, a }
    }

    /// Get the color as RGBA tuple
    pub fn to_rgba(&self) -> (u8, u8, u8, u8) {
        match self {
            ProgressColor::Primary => (66, 133, 244, 255),   // Blue
            ProgressColor::Secondary => (156, 39, 176, 255), // Purple
            ProgressColor::Success => (76, 175, 80, 255),    // Green
            ProgressColor::Warning => (255, 152, 0, 255),    // Orange
            ProgressColor::Error => (244, 67, 54, 255),      // Red
            ProgressColor::Custom { r, g, b, a } => (*r, *g, *b, *a),
        }
    }
}

/// Animation timing for progress indicators
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnimationTiming {
    /// Duration of one animation cycle in seconds
    pub duration: f32,
    /// Easing function name
    pub easing: AnimationEasing,
}

impl Default for AnimationTiming {
    fn default() -> Self {
        Self {
            duration: 1.5,
            easing: AnimationEasing::EaseInOut,
        }
    }
}

/// Easing functions for animations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnimationEasing {
    /// Linear interpolation
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in and out (slow start and end)
    #[default]
    EaseInOut,
    /// Material Design standard curve
    MaterialStandard,
}

impl AnimationEasing {
    /// Calculate the eased value for a given progress (0.0 to 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            AnimationEasing::Linear => t,
            AnimationEasing::EaseIn => t * t,
            AnimationEasing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            AnimationEasing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            AnimationEasing::MaterialStandard => {
                // Cubic bezier approximation for Material Design curve
                let t2 = t * t;
                let t3 = t2 * t;
                3.0 * t2 - 2.0 * t3
            }
        }
    }
}

/// Accessibility attributes for progress indicators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressAccessibility {
    /// ARIA label for the progress indicator
    pub label: Option<String>,
    /// Minimum value (always 0 for progress)
    pub value_min: f32,
    /// Maximum value (always 1 for progress, or 100 if using percentages)
    pub value_max: f32,
    /// Current value (for determinate mode)
    pub value_now: Option<f32>,
    /// Human-readable value text
    pub value_text: Option<String>,
    /// Whether to announce progress updates to screen readers
    pub announce_updates: bool,
    /// Minimum change before announcing (prevents spam)
    pub announce_threshold: f32,
}

impl Default for ProgressAccessibility {
    fn default() -> Self {
        Self {
            label: None,
            value_min: 0.0,
            value_max: 1.0,
            value_now: None,
            value_text: None,
            announce_updates: true,
            announce_threshold: 0.1, // Announce every 10% change
        }
    }
}

impl ProgressAccessibility {
    /// Create accessibility attributes for determinate progress
    pub fn for_determinate(value: f32, label: impl Into<String>) -> Self {
        let percentage = (value * 100.0).round() as i32;
        Self {
            label: Some(label.into()),
            value_min: 0.0,
            value_max: 1.0,
            value_now: Some(value),
            value_text: Some(format!("{}% complete", percentage)),
            announce_updates: true,
            announce_threshold: 0.1,
        }
    }

    /// Create accessibility attributes for indeterminate progress
    pub fn for_indeterminate(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            value_min: 0.0,
            value_max: 1.0,
            value_now: None,
            value_text: Some("Loading...".to_string()),
            announce_updates: false,
            announce_threshold: 0.1,
        }
    }

    /// Get the percentage as an integer (0-100)
    pub fn percentage(&self) -> Option<i32> {
        self.value_now.map(|v| (v * 100.0).round() as i32)
    }

    /// Generate ARIA attributes as key-value pairs
    pub fn to_aria_attributes(&self) -> Vec<(&'static str, String)> {
        let mut attrs = vec![("role", "progressbar".to_string())];

        if let Some(label) = &self.label {
            attrs.push(("aria-label", label.clone()));
        }

        attrs.push(("aria-valuemin", self.value_min.to_string()));
        attrs.push(("aria-valuemax", self.value_max.to_string()));

        if let Some(value) = self.value_now {
            attrs.push(("aria-valuenow", value.to_string()));
        } else {
            attrs.push(("aria-busy", "true".to_string()));
        }

        if let Some(text) = &self.value_text {
            attrs.push(("aria-valuetext", text.clone()));
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_mode_requires_value() {
        assert!(ProgressMode::Determinate.requires_value());
        assert!(!ProgressMode::Indeterminate.requires_value());
        assert!(ProgressMode::Buffer.requires_value());
    }

    #[test]
    fn test_progress_color_to_rgba() {
        let primary = ProgressColor::Primary;
        assert_eq!(primary.to_rgba(), (66, 133, 244, 255));

        let custom = ProgressColor::custom(255, 128, 64, 200);
        assert_eq!(custom.to_rgba(), (255, 128, 64, 200));
    }

    #[test]
    fn test_animation_easing_linear() {
        let easing = AnimationEasing::Linear;
        assert!((easing.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((easing.apply(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((easing.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_animation_easing_ease_in() {
        let easing = AnimationEasing::EaseIn;
        assert!((easing.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!(easing.apply(0.5) < 0.5); // Slower start
        assert!((easing.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_animation_easing_ease_out() {
        let easing = AnimationEasing::EaseOut;
        assert!((easing.apply(0.0) - 0.0).abs() < f32::EPSILON);
        assert!(easing.apply(0.5) > 0.5); // Faster start
        assert!((easing.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_animation_easing_clamps_input() {
        let easing = AnimationEasing::Linear;
        assert!((easing.apply(-0.5) - 0.0).abs() < f32::EPSILON);
        assert!((easing.apply(1.5) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_accessibility_for_determinate() {
        let a11y = ProgressAccessibility::for_determinate(0.75, "Upload progress");
        assert_eq!(a11y.label, Some("Upload progress".to_string()));
        assert_eq!(a11y.value_now, Some(0.75));
        assert_eq!(a11y.percentage(), Some(75));
        assert_eq!(a11y.value_text, Some("75% complete".to_string()));
    }

    #[test]
    fn test_accessibility_for_indeterminate() {
        let a11y = ProgressAccessibility::for_indeterminate("Loading data");
        assert_eq!(a11y.label, Some("Loading data".to_string()));
        assert!(a11y.value_now.is_none());
        assert!(a11y.percentage().is_none());
    }

    #[test]
    fn test_accessibility_to_aria_attributes() {
        let a11y = ProgressAccessibility::for_determinate(0.5, "Progress");
        let attrs = a11y.to_aria_attributes();

        assert!(attrs.contains(&("role", "progressbar".to_string())));
        assert!(attrs.contains(&("aria-valuenow", "0.5".to_string())));
        assert!(attrs.contains(&("aria-label", "Progress".to_string())));
    }
}
