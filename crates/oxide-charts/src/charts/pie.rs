//! Pie and donut charts.

use serde::{Deserialize, Serialize};
use crate::animation::AnimationConfig;
use crate::legend::Legend;

/// Slice style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceStyle {
    /// Border width
    pub border_width: f32,
    /// Border color
    pub border_color: String,
    /// Hover offset
    pub hover_offset: f32,
}

impl Default for SliceStyle {
    fn default() -> Self {
        Self {
            border_width: 1.0,
            border_color: "#FFFFFF".to_string(),
            hover_offset: 10.0,
        }
    }
}

/// Pie chart data item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieSlice {
    /// Label
    pub label: String,
    /// Value
    pub value: f64,
    /// Color (optional)
    pub color: Option<String>,
}

impl PieSlice {
    /// Create new slice
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    /// With color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// Pie chart
#[derive(Debug, Clone)]
pub struct PieChart {
    /// Data slices
    pub data: Vec<PieSlice>,
    /// Legend
    pub legend: Option<Legend>,
    /// Animation config
    pub animation: AnimationConfig,
    /// Style
    pub style: SliceStyle,
    /// Show labels
    pub show_labels: bool,
    /// Show values
    pub show_values: bool,
}

impl PieChart {
    /// Create new pie chart
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            legend: Some(Legend::right()),
            animation: AnimationConfig::default(),
            style: SliceStyle::default(),
            show_labels: true,
            show_values: false,
        }
    }

    /// Add slice
    pub fn slice(mut self, label: impl Into<String>, value: f64) -> Self {
        self.data.push(PieSlice::new(label, value));
        self
    }

    /// Set data
    pub fn data(mut self, data: Vec<PieSlice>) -> Self {
        self.data = data;
        self
    }

    /// Set legend
    pub fn legend(mut self, legend: Legend) -> Self {
        self.legend = Some(legend);
        self
    }

    /// Show values on slices
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Enable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.animation.enabled = animate;
        self
    }

    /// Calculate total
    pub fn total(&self) -> f64 {
        self.data.iter().map(|s| s.value).sum()
    }
}

impl Default for PieChart {
    fn default() -> Self {
        Self::new()
    }
}

/// Donut chart (pie with hole)
#[derive(Debug, Clone)]
pub struct DonutChart {
    /// Inner pie chart
    pub inner: PieChart,
    /// Hole radius (0.0 to 1.0)
    pub hole_radius: f32,
    /// Center text
    pub center_text: Option<String>,
}

impl DonutChart {
    /// Create new donut chart
    pub fn new() -> Self {
        Self {
            inner: PieChart::new(),
            hole_radius: 0.5,
            center_text: None,
        }
    }

    /// Add slice
    pub fn slice(mut self, label: impl Into<String>, value: f64) -> Self {
        self.inner = self.inner.slice(label, value);
        self
    }

    /// Set hole radius
    pub fn hole_radius(mut self, radius: f32) -> Self {
        self.hole_radius = radius.clamp(0.0, 0.9);
        self
    }

    /// Set center text
    pub fn center_text(mut self, text: impl Into<String>) -> Self {
        self.center_text = Some(text.into());
        self
    }

    /// Enable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.inner.animation.enabled = animate;
        self
    }
}

impl Default for DonutChart {
    fn default() -> Self {
        Self::new()
    }
}
