//! Gauge charts.

use serde::{Deserialize, Serialize};
use crate::animation::AnimationConfig;

/// Gauge style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeStyle {
    /// Track color
    pub track_color: String,
    /// Fill color
    pub fill_color: String,
    /// Thickness
    pub thickness: f32,
    /// Round caps
    pub round_caps: bool,
}

impl Default for GaugeStyle {
    fn default() -> Self {
        Self {
            track_color: "#E0E0E0".to_string(),
            fill_color: "#2196F3".to_string(),
            thickness: 20.0,
            round_caps: true,
        }
    }
}

/// Radial gauge
#[derive(Debug, Clone)]
pub struct RadialGauge {
    /// Current value
    pub value: f64,
    /// Min value
    pub min: f64,
    /// Max value
    pub max: f64,
    /// Style
    pub style: GaugeStyle,
    /// Animation
    pub animation: AnimationConfig,
    /// Start angle (degrees)
    pub start_angle: f32,
    /// End angle (degrees)
    pub end_angle: f32,
    /// Show value
    pub show_value: bool,
    /// Value format
    pub format: Option<String>,
}

impl RadialGauge {
    /// Create new gauge
    pub fn new(value: f64, max: f64) -> Self {
        Self {
            value,
            min: 0.0,
            max,
            style: GaugeStyle::default(),
            animation: AnimationConfig::default(),
            start_angle: -135.0,
            end_angle: 135.0,
            show_value: true,
            format: None,
        }
    }

    /// Set value
    pub fn value(mut self, value: f64) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    /// Set range
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set style
    pub fn style(mut self, style: GaugeStyle) -> Self {
        self.style = style;
        self
    }

    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.style.fill_color = color.into();
        self
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.max == self.min {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }
}

impl Default for RadialGauge {
    fn default() -> Self {
        Self::new(0.0, 100.0)
    }
}

/// Linear gauge
#[derive(Debug, Clone)]
pub struct LinearGauge {
    /// Current value
    pub value: f64,
    /// Min value
    pub min: f64,
    /// Max value
    pub max: f64,
    /// Style
    pub style: GaugeStyle,
    /// Animation
    pub animation: AnimationConfig,
    /// Horizontal or vertical
    pub horizontal: bool,
    /// Show value
    pub show_value: bool,
}

impl LinearGauge {
    /// Create new linear gauge
    pub fn new(value: f64, max: f64) -> Self {
        Self {
            value,
            min: 0.0,
            max,
            style: GaugeStyle::default(),
            animation: AnimationConfig::default(),
            horizontal: true,
            show_value: true,
        }
    }

    /// Set value
    pub fn value(mut self, value: f64) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    /// Set vertical orientation
    pub fn vertical(mut self) -> Self {
        self.horizontal = false;
        self
    }

    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.style.fill_color = color.into();
        self
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.max == self.min {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }
}

impl Default for LinearGauge {
    fn default() -> Self {
        Self::new(0.0, 100.0)
    }
}

/// Progress ring (circular progress)
#[derive(Debug, Clone)]
pub struct ProgressRing {
    /// Progress (0.0 to 1.0)
    pub progress: f64,
    /// Style
    pub style: GaugeStyle,
    /// Animation
    pub animation: AnimationConfig,
    /// Size
    pub size: f32,
    /// Show percentage
    pub show_percent: bool,
}

impl ProgressRing {
    /// Create new progress ring
    pub fn new(progress: f64) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            style: GaugeStyle::default(),
            animation: AnimationConfig::default(),
            size: 100.0,
            show_percent: true,
        }
    }

    /// Set progress
    pub fn progress(mut self, progress: f64) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Set size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.style.fill_color = color.into();
        self
    }

    /// Get percentage
    pub fn percent(&self) -> f64 {
        self.progress * 100.0
    }
}

impl Default for ProgressRing {
    fn default() -> Self {
        Self::new(0.0)
    }
}
