//! Circular Progress Indicator
//!
//! A spinning/circular progress indicator with support for determinate and indeterminate modes.
//!
//! # Example
//!
//! ```
//! use oxide_loading::progress::{CircularProgress, CircularProgressSize, ProgressMode};
//!
//! // Determinate circular progress
//! let progress = CircularProgress::new()
//!     .mode(ProgressMode::Determinate)
//!     .value(0.75)
//!     .size(CircularProgressSize::Large);
//!
//! // Indeterminate spinner
//! let spinner = CircularProgress::new()
//!     .mode(ProgressMode::Indeterminate)
//!     .size(CircularProgressSize::Small);
//! ```

use serde::{Deserialize, Serialize};

use super::{AnimationEasing, AnimationTiming, ProgressAccessibility, ProgressColor, ProgressMode};

/// Size presets for circular progress indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CircularProgressSize {
    /// Small spinner (16px)
    Small,
    /// Medium spinner (32px)
    #[default]
    Medium,
    /// Large spinner (48px)
    Large,
    /// Custom size in pixels
    Custom(u32),
}

impl CircularProgressSize {
    /// Get the diameter in pixels
    pub fn to_pixels(&self) -> f32 {
        match self {
            CircularProgressSize::Small => 16.0,
            CircularProgressSize::Medium => 32.0,
            CircularProgressSize::Large => 48.0,
            CircularProgressSize::Custom(px) => *px as f32,
        }
    }

    /// Get the default stroke width for this size
    pub fn default_stroke_width(&self) -> f32 {
        match self {
            CircularProgressSize::Small => 2.0,
            CircularProgressSize::Medium => 3.0,
            CircularProgressSize::Large => 4.0,
            CircularProgressSize::Custom(px) => (*px as f32 * 0.1).max(2.0),
        }
    }
}

/// Circular progress indicator (spinner)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircularProgress {
    /// Progress mode (determinate or indeterminate)
    pub mode: ProgressMode,
    /// Current progress value (0.0 to 1.0)
    pub value: f32,
    /// Size of the spinner
    pub size: CircularProgressSize,
    /// Stroke width in pixels
    pub stroke_width: f32,
    /// Progress arc color
    pub color: ProgressColor,
    /// Track (background) color
    pub track_color: ProgressColor,
    /// Whether to show percentage text in center
    pub show_percentage: bool,
    /// Animation timing
    pub animation: AnimationTiming,
    /// Accessibility attributes
    pub accessibility: ProgressAccessibility,
    /// Current rotation angle in radians
    rotation_angle: f32,
    /// Arc sweep angle for indeterminate mode
    arc_angle: f32,
    /// Animation time (0.0 to 1.0)
    animation_time: f32,
    /// Previous value for animation
    previous_value: f32,
    /// Value animation progress
    value_animation_progress: f32,
}

impl Default for CircularProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl CircularProgress {
    /// Create a new circular progress indicator
    pub fn new() -> Self {
        Self {
            mode: ProgressMode::Indeterminate,
            value: 0.0,
            size: CircularProgressSize::Medium,
            stroke_width: 3.0,
            color: ProgressColor::Primary,
            track_color: ProgressColor::Custom {
                r: 200,
                g: 200,
                b: 200,
                a: 100,
            },
            show_percentage: false,
            animation: AnimationTiming {
                duration: 1.4,
                easing: AnimationEasing::EaseInOut,
            },
            accessibility: ProgressAccessibility::default(),
            rotation_angle: 0.0,
            arc_angle: std::f32::consts::FRAC_PI_4, // 45 degrees initial
            animation_time: 0.0,
            previous_value: 0.0,
            value_animation_progress: 1.0,
        }
    }

    /// Set the progress mode
    pub fn mode(mut self, mode: ProgressMode) -> Self {
        // Buffer mode doesn't make sense for circular, treat as determinate
        self.mode = if mode == ProgressMode::Buffer {
            ProgressMode::Determinate
        } else {
            mode
        };
        self.update_accessibility();
        self
    }

    /// Set the progress value (0.0 to 1.0)
    pub fn value(mut self, value: f32) -> Self {
        let new_value = value.clamp(0.0, 1.0);
        if (new_value - self.value).abs() > f32::EPSILON {
            self.previous_value = self.value;
            self.value = new_value;
            self.value_animation_progress = 0.0;
        }
        self.update_accessibility();
        self
    }

    /// Set the size
    pub fn size(mut self, size: CircularProgressSize) -> Self {
        self.size = size;
        // Update stroke width to match size if not custom
        self.stroke_width = size.default_stroke_width();
        self
    }

    /// Set the stroke width in pixels
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width.max(1.0);
        self
    }

    /// Set the progress color
    pub fn color(mut self, color: ProgressColor) -> Self {
        self.color = color;
        self
    }

    /// Set the track color
    pub fn track_color(mut self, color: ProgressColor) -> Self {
        self.track_color = color;
        self
    }

    /// Enable or disable percentage display
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show && self.mode == ProgressMode::Determinate;
        self
    }

    /// Set the animation timing
    pub fn animation(mut self, animation: AnimationTiming) -> Self {
        self.animation = animation;
        self
    }

    /// Set the accessibility label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.accessibility.label = Some(label.into());
        self.update_accessibility();
        self
    }

    /// Update animation state (call each frame)
    ///
    /// # Arguments
    /// * `delta_time` - Time since last frame in seconds
    ///
    /// # Returns
    /// `true` if animation needs to continue
    pub fn update(&mut self, delta_time: f32) -> bool {
        match self.mode {
            ProgressMode::Indeterminate => {
                // Update animation time
                self.animation_time += delta_time / self.animation.duration;
                if self.animation_time >= 1.0 {
                    self.animation_time -= 1.0;
                }

                // Rotate continuously
                self.rotation_angle += delta_time * std::f32::consts::TAU * 0.5;
                if self.rotation_angle >= std::f32::consts::TAU {
                    self.rotation_angle -= std::f32::consts::TAU;
                }

                // Animate arc length (grow and shrink)
                let t = self.animation.easing.apply(self.animation_time);
                let min_arc = std::f32::consts::FRAC_PI_8; // 22.5 degrees
                let max_arc = std::f32::consts::PI * 1.5; // 270 degrees

                // Pulsing arc that grows then shrinks
                let pulse = (t * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;
                self.arc_angle = min_arc + (max_arc - min_arc) * pulse;

                true
            }
            ProgressMode::Determinate | ProgressMode::Buffer => {
                if self.value_animation_progress < 1.0 {
                    self.value_animation_progress += delta_time / 0.3;
                    self.value_animation_progress = self.value_animation_progress.min(1.0);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Get the animated progress value
    pub fn animated_value(&self) -> f32 {
        if self.value_animation_progress >= 1.0 {
            self.value
        } else {
            let t = self.animation.easing.apply(self.value_animation_progress);
            self.previous_value + (self.value - self.previous_value) * t
        }
    }

    /// Get rendering parameters for the circular progress
    pub fn render_params(&self) -> CircularProgressRenderParams {
        let diameter = self.size.to_pixels();
        let radius = (diameter - self.stroke_width) / 2.0;

        match self.mode {
            ProgressMode::Determinate | ProgressMode::Buffer => {
                let sweep_angle = self.animated_value() * std::f32::consts::TAU;
                CircularProgressRenderParams {
                    center_x: diameter / 2.0,
                    center_y: diameter / 2.0,
                    radius,
                    stroke_width: self.stroke_width,
                    track_color: self.track_color.to_rgba(),
                    progress_color: self.color.to_rgba(),
                    start_angle: -std::f32::consts::FRAC_PI_2, // Start at top
                    sweep_angle,
                    percentage_text: if self.show_percentage {
                        Some(format!("{}%", (self.value * 100.0).round() as i32))
                    } else {
                        None
                    },
                    diameter,
                }
            }
            ProgressMode::Indeterminate => CircularProgressRenderParams {
                center_x: diameter / 2.0,
                center_y: diameter / 2.0,
                radius,
                stroke_width: self.stroke_width,
                track_color: self.track_color.to_rgba(),
                progress_color: self.color.to_rgba(),
                start_angle: self.rotation_angle - std::f32::consts::FRAC_PI_2,
                sweep_angle: self.arc_angle,
                percentage_text: None,
                diameter,
            },
        }
    }

    /// Get accessibility attributes
    pub fn aria_attributes(&self) -> Vec<(&'static str, String)> {
        self.accessibility.to_aria_attributes()
    }

    fn update_accessibility(&mut self) {
        match self.mode {
            ProgressMode::Determinate | ProgressMode::Buffer => {
                self.accessibility.value_now = Some(self.value);
                let percentage = (self.value * 100.0).round() as i32;
                self.accessibility.value_text = Some(format!("{}% complete", percentage));
            }
            ProgressMode::Indeterminate => {
                self.accessibility.value_now = None;
                self.accessibility.value_text = Some("Loading...".to_string());
            }
        }
    }
}

/// Rendering parameters for circular progress
#[derive(Debug, Clone, PartialEq)]
pub struct CircularProgressRenderParams {
    /// X coordinate of center
    pub center_x: f32,
    /// Y coordinate of center
    pub center_y: f32,
    /// Arc radius
    pub radius: f32,
    /// Stroke width
    pub stroke_width: f32,
    /// Track color (r, g, b, a)
    pub track_color: (u8, u8, u8, u8),
    /// Progress color (r, g, b, a)
    pub progress_color: (u8, u8, u8, u8),
    /// Start angle in radians (0 = right, increases clockwise)
    pub start_angle: f32,
    /// Sweep angle in radians
    pub sweep_angle: f32,
    /// Percentage text to display (if any)
    pub percentage_text: Option<String>,
    /// Total diameter for bounds
    pub diameter: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_progress_size_to_pixels() {
        assert!((CircularProgressSize::Small.to_pixels() - 16.0).abs() < f32::EPSILON);
        assert!((CircularProgressSize::Medium.to_pixels() - 32.0).abs() < f32::EPSILON);
        assert!((CircularProgressSize::Large.to_pixels() - 48.0).abs() < f32::EPSILON);
        assert!((CircularProgressSize::Custom(64).to_pixels() - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_circular_progress_default() {
        let progress = CircularProgress::new();
        assert_eq!(progress.mode, ProgressMode::Indeterminate);
        assert_eq!(progress.size, CircularProgressSize::Medium);
        assert!((progress.stroke_width - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_circular_progress_builder() {
        let progress = CircularProgress::new()
            .mode(ProgressMode::Determinate)
            .value(0.5)
            .size(CircularProgressSize::Large)
            .color(ProgressColor::Success)
            .show_percentage(true);

        assert_eq!(progress.mode, ProgressMode::Determinate);
        assert!((progress.value - 0.5).abs() < f32::EPSILON);
        assert_eq!(progress.size, CircularProgressSize::Large);
        assert!(progress.show_percentage);
    }

    #[test]
    fn test_circular_progress_buffer_becomes_determinate() {
        let progress = CircularProgress::new().mode(ProgressMode::Buffer);
        assert_eq!(progress.mode, ProgressMode::Determinate);
    }

    #[test]
    fn test_circular_progress_value_clamping() {
        let progress = CircularProgress::new().value(1.5);
        assert!((progress.value - 1.0).abs() < f32::EPSILON);

        let progress = CircularProgress::new().value(-0.5);
        assert!((progress.value - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_circular_progress_render_params_determinate() {
        let progress = CircularProgress::new()
            .mode(ProgressMode::Determinate)
            .value(0.5)
            .size(CircularProgressSize::Medium);

        let mut progress = progress;
        progress.value_animation_progress = 1.0;

        let params = progress.render_params();
        assert!((params.diameter - 32.0).abs() < f32::EPSILON);
        assert!((params.sweep_angle - std::f32::consts::PI).abs() < 0.01); // ~180 degrees
    }

    #[test]
    fn test_circular_progress_render_params_indeterminate() {
        let progress = CircularProgress::new().mode(ProgressMode::Indeterminate);

        let params = progress.render_params();
        assert!(params.percentage_text.is_none());
        assert!(params.sweep_angle > 0.0);
    }

    #[test]
    fn test_circular_progress_percentage_text() {
        let progress = CircularProgress::new()
            .mode(ProgressMode::Determinate)
            .value(0.75)
            .show_percentage(true);

        let mut progress = progress;
        progress.value_animation_progress = 1.0;

        let params = progress.render_params();
        assert_eq!(params.percentage_text, Some("75%".to_string()));
    }

    #[test]
    fn test_circular_progress_update_indeterminate() {
        let mut progress = CircularProgress::new().mode(ProgressMode::Indeterminate);

        let initial_rotation = progress.rotation_angle;
        progress.update(0.1);
        assert!(progress.rotation_angle > initial_rotation);
    }

    #[test]
    fn test_circular_progress_update_determinate() {
        let mut progress = CircularProgress::new()
            .mode(ProgressMode::Determinate)
            .value(0.5);

        // Should animate value change
        assert!(progress.update(0.016));

        progress.value_animation_progress = 1.0;
        assert!(!progress.update(0.016));
    }

    #[test]
    fn test_circular_progress_size_updates_stroke() {
        let progress = CircularProgress::new().size(CircularProgressSize::Small);
        assert!((progress.stroke_width - 2.0).abs() < f32::EPSILON);

        let progress = CircularProgress::new().size(CircularProgressSize::Large);
        assert!((progress.stroke_width - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_circular_progress_accessibility() {
        let progress = CircularProgress::new()
            .mode(ProgressMode::Determinate)
            .value(0.33)
            .label("Loading files");

        let attrs = progress.aria_attributes();
        assert!(attrs.contains(&("role", "progressbar".to_string())));
        assert!(attrs.contains(&("aria-label", "Loading files".to_string())));
    }
}
