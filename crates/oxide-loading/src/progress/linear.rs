//! Linear Progress Indicator
//!
//! A horizontal progress bar with support for determinate, indeterminate, and buffer modes.
//!
//! # Example
//!
//! ```
//! use oxide_loading::progress::{LinearProgress, ProgressMode};
//!
//! // Determinate progress
//! let progress = LinearProgress::new()
//!     .mode(ProgressMode::Determinate)
//!     .value(0.65)
//!     .label("Uploading file");
//!
//! // Indeterminate loading
//! let loading = LinearProgress::new()
//!     .mode(ProgressMode::Indeterminate)
//!     .label("Loading data");
//!
//! // Buffer mode for streaming
//! let streaming = LinearProgress::new()
//!     .mode(ProgressMode::Buffer)
//!     .value(0.3)
//!     .buffer(0.5);
//! ```

use serde::{Deserialize, Serialize};

use super::{AnimationEasing, AnimationTiming, ProgressAccessibility, ProgressColor, ProgressMode};

/// Linear (horizontal bar) progress indicator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearProgress {
    /// Progress mode (determinate, indeterminate, buffer)
    pub mode: ProgressMode,
    /// Current progress value (0.0 to 1.0)
    pub value: f32,
    /// Buffer value for buffer mode (0.0 to 1.0)
    pub buffer: f32,
    /// Color scheme
    pub color: ProgressColor,
    /// Track (background) color
    pub track_color: ProgressColor,
    /// Buffer color (for buffer mode)
    pub buffer_color: ProgressColor,
    /// Height of the progress bar in pixels
    pub height: f32,
    /// Border radius in pixels
    pub border_radius: f32,
    /// Animation timing configuration
    pub animation: AnimationTiming,
    /// Accessibility attributes
    pub accessibility: ProgressAccessibility,
    /// Current animation time (0.0 to 1.0, loops)
    animation_time: f32,
    /// Previous value for animation
    previous_value: f32,
    /// Value animation progress
    value_animation_progress: f32,
}

impl Default for LinearProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearProgress {
    /// Create a new linear progress indicator with default settings
    pub fn new() -> Self {
        Self {
            mode: ProgressMode::default(),
            value: 0.0,
            buffer: 0.0,
            color: ProgressColor::Primary,
            track_color: ProgressColor::Custom {
                r: 200,
                g: 200,
                b: 200,
                a: 100,
            },
            buffer_color: ProgressColor::Custom {
                r: 150,
                g: 200,
                b: 255,
                a: 150,
            },
            height: 4.0,
            border_radius: 2.0,
            animation: AnimationTiming::default(),
            accessibility: ProgressAccessibility::default(),
            animation_time: 0.0,
            previous_value: 0.0,
            value_animation_progress: 1.0,
        }
    }

    /// Set the progress mode
    pub fn mode(mut self, mode: ProgressMode) -> Self {
        self.mode = mode;
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

    /// Set the buffer value for buffer mode (0.0 to 1.0)
    pub fn buffer(mut self, buffer: f32) -> Self {
        self.buffer = buffer.clamp(0.0, 1.0);
        self
    }

    /// Set the progress color
    pub fn color(mut self, color: ProgressColor) -> Self {
        self.color = color;
        self
    }

    /// Set the track (background) color
    pub fn track_color(mut self, color: ProgressColor) -> Self {
        self.track_color = color;
        self
    }

    /// Set the buffer color
    pub fn buffer_color(mut self, color: ProgressColor) -> Self {
        self.buffer_color = color;
        self
    }

    /// Set the height in pixels
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }

    /// Set the border radius in pixels
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius.max(0.0);
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
    /// `true` if animation needs to continue (indeterminate mode or value animating)
    pub fn update(&mut self, delta_time: f32) -> bool {
        let needs_update = match self.mode {
            ProgressMode::Indeterminate => {
                self.animation_time += delta_time / self.animation.duration;
                if self.animation_time >= 1.0 {
                    self.animation_time -= 1.0;
                }
                true
            }
            ProgressMode::Determinate | ProgressMode::Buffer => {
                if self.value_animation_progress < 1.0 {
                    // Animate value changes over 300ms
                    self.value_animation_progress += delta_time / 0.3;
                    self.value_animation_progress = self.value_animation_progress.min(1.0);
                    true
                } else {
                    false
                }
            }
        };

        needs_update
    }

    /// Get the animated progress value for rendering
    pub fn animated_value(&self) -> f32 {
        if self.value_animation_progress >= 1.0 {
            self.value
        } else {
            let t = self.animation.easing.apply(self.value_animation_progress);
            self.previous_value + (self.value - self.previous_value) * t
        }
    }

    /// Get the indeterminate animation position (0.0 to 1.0)
    pub fn indeterminate_position(&self) -> f32 {
        self.animation.easing.apply(self.animation_time)
    }

    /// Get rendering parameters for the progress bar
    pub fn render_params(&self, width: f32) -> LinearProgressRenderParams {
        match self.mode {
            ProgressMode::Determinate => {
                let progress_width = width * self.animated_value();
                LinearProgressRenderParams {
                    track_width: width,
                    track_height: self.height,
                    track_color: self.track_color.to_rgba(),
                    progress_x: 0.0,
                    progress_width,
                    progress_color: self.color.to_rgba(),
                    buffer_width: None,
                    buffer_color: None,
                    border_radius: self.border_radius,
                }
            }
            ProgressMode::Indeterminate => {
                // Create a moving bar effect
                let bar_width = width * 0.3; // Bar is 30% of track width
                let pos = self.indeterminate_position();

                // Use sine wave for smooth back-and-forth motion
                let cycle = (pos * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;
                let progress_x = (width - bar_width) * cycle;

                LinearProgressRenderParams {
                    track_width: width,
                    track_height: self.height,
                    track_color: self.track_color.to_rgba(),
                    progress_x,
                    progress_width: bar_width,
                    progress_color: self.color.to_rgba(),
                    buffer_width: None,
                    buffer_color: None,
                    border_radius: self.border_radius,
                }
            }
            ProgressMode::Buffer => {
                let progress_width = width * self.animated_value();
                let buffer_width = width * self.buffer;
                LinearProgressRenderParams {
                    track_width: width,
                    track_height: self.height,
                    track_color: self.track_color.to_rgba(),
                    progress_x: 0.0,
                    progress_width,
                    progress_color: self.color.to_rgba(),
                    buffer_width: Some(buffer_width),
                    buffer_color: Some(self.buffer_color.to_rgba()),
                    border_radius: self.border_radius,
                }
            }
        }
    }

    /// Get accessibility attributes for screen readers
    pub fn aria_attributes(&self) -> Vec<(&'static str, String)> {
        self.accessibility.to_aria_attributes()
    }

    /// Check if screen reader should announce the current state
    pub fn should_announce(&self) -> bool {
        self.accessibility.announce_updates
            && self.mode.requires_value()
            && self.value_animation_progress >= 1.0
    }

    /// Get announcement text for screen readers
    pub fn announcement(&self) -> Option<String> {
        if self.should_announce() {
            self.accessibility.value_text.clone()
        } else {
            None
        }
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

/// Rendering parameters for linear progress bar
#[derive(Debug, Clone, PartialEq)]
pub struct LinearProgressRenderParams {
    /// Total track width
    pub track_width: f32,
    /// Track height
    pub track_height: f32,
    /// Track background color (r, g, b, a)
    pub track_color: (u8, u8, u8, u8),
    /// X position of progress bar (for indeterminate mode)
    pub progress_x: f32,
    /// Width of filled progress area
    pub progress_width: f32,
    /// Progress bar color (r, g, b, a)
    pub progress_color: (u8, u8, u8, u8),
    /// Buffer width (for buffer mode)
    pub buffer_width: Option<f32>,
    /// Buffer color (r, g, b, a)
    pub buffer_color: Option<(u8, u8, u8, u8)>,
    /// Corner radius
    pub border_radius: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_progress_default() {
        let progress = LinearProgress::new();
        assert_eq!(progress.mode, ProgressMode::Determinate);
        assert!((progress.value - 0.0).abs() < f32::EPSILON);
        assert!((progress.height - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_linear_progress_builder() {
        let progress = LinearProgress::new()
            .mode(ProgressMode::Determinate)
            .value(0.75)
            .height(8.0)
            .color(ProgressColor::Success)
            .label("Upload progress");

        assert_eq!(progress.mode, ProgressMode::Determinate);
        assert!((progress.value - 0.75).abs() < f32::EPSILON);
        assert!((progress.height - 8.0).abs() < f32::EPSILON);
        assert_eq!(progress.color, ProgressColor::Success);
        assert_eq!(
            progress.accessibility.label,
            Some("Upload progress".to_string())
        );
    }

    #[test]
    fn test_linear_progress_value_clamping() {
        let progress = LinearProgress::new().value(1.5);
        assert!((progress.value - 1.0).abs() < f32::EPSILON);

        let progress = LinearProgress::new().value(-0.5);
        assert!((progress.value - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_linear_progress_buffer_mode() {
        let progress = LinearProgress::new()
            .mode(ProgressMode::Buffer)
            .value(0.3)
            .buffer(0.5);

        assert_eq!(progress.mode, ProgressMode::Buffer);
        assert!((progress.value - 0.3).abs() < f32::EPSILON);
        assert!((progress.buffer - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_linear_progress_render_params_determinate() {
        let progress = LinearProgress::new()
            .mode(ProgressMode::Determinate)
            .value(0.5);

        // Force animation to complete
        let mut progress = progress;
        progress.value_animation_progress = 1.0;

        let params = progress.render_params(200.0);
        assert!((params.track_width - 200.0).abs() < f32::EPSILON);
        assert!((params.progress_width - 100.0).abs() < f32::EPSILON);
        assert!(params.buffer_width.is_none());
    }

    #[test]
    fn test_linear_progress_render_params_buffer() {
        let progress = LinearProgress::new()
            .mode(ProgressMode::Buffer)
            .value(0.25)
            .buffer(0.75);

        let mut progress = progress;
        progress.value_animation_progress = 1.0;

        let params = progress.render_params(400.0);
        assert!((params.progress_width - 100.0).abs() < f32::EPSILON);
        assert_eq!(params.buffer_width, Some(300.0));
    }

    #[test]
    fn test_linear_progress_update_determinate() {
        let mut progress = LinearProgress::new().mode(ProgressMode::Determinate).value(0.5);

        // Should animate initially
        assert!(progress.update(0.016)); // ~60fps frame

        // After animation completes
        progress.value_animation_progress = 1.0;
        assert!(!progress.update(0.016));
    }

    #[test]
    fn test_linear_progress_update_indeterminate() {
        let mut progress = LinearProgress::new().mode(ProgressMode::Indeterminate);

        // Should always need updates
        assert!(progress.update(0.016));
        assert!(progress.update(0.016));
    }

    #[test]
    fn test_linear_progress_animated_value() {
        let mut progress = LinearProgress::new().value(0.0);
        progress.value_animation_progress = 1.0;

        // Change value
        progress = progress.value(0.8);

        // Animation not started
        assert!((progress.animated_value() - 0.0).abs() < f32::EPSILON);

        // Midway through animation
        progress.value_animation_progress = 0.5;
        let animated = progress.animated_value();
        assert!(animated > 0.0 && animated < 0.8);

        // Animation complete
        progress.value_animation_progress = 1.0;
        assert!((progress.animated_value() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_linear_progress_accessibility() {
        let progress = LinearProgress::new()
            .mode(ProgressMode::Determinate)
            .value(0.75)
            .label("File upload");

        let attrs = progress.aria_attributes();
        assert!(attrs.contains(&("role", "progressbar".to_string())));
        assert!(attrs.contains(&("aria-valuenow", "0.75".to_string())));
    }

    #[test]
    fn test_linear_progress_indeterminate_accessibility() {
        let progress = LinearProgress::new()
            .mode(ProgressMode::Indeterminate)
            .label("Loading");

        let attrs = progress.aria_attributes();
        assert!(attrs.contains(&("aria-busy", "true".to_string())));
    }
}
