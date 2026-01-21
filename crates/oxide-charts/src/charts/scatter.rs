//! Scatter plots and bubble charts.

use serde::{Deserialize, Serialize};
use crate::axis::Axis;
use crate::animation::AnimationConfig;
use crate::legend::Legend;

/// A point in a scatter plot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterPoint {
    /// X value
    pub x: f64,
    /// Y value
    pub y: f64,
    /// Size (for bubble charts)
    pub size: Option<f64>,
    /// Label
    pub label: Option<String>,
}

impl ScatterPoint {
    /// Create new point
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            size: None,
            label: None,
        }
    }

    /// With size (for bubble)
    pub fn with_size(mut self, size: f64) -> Self {
        self.size = Some(size);
        self
    }

    /// With label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Trend line type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendLineType {
    /// Linear regression
    Linear,
    /// Exponential
    Exponential,
    /// Polynomial
    Polynomial(usize),
    /// Moving average
    MovingAverage(usize),
}

/// Trend line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendLine {
    /// Type
    pub line_type: TrendLineType,
    /// Color
    pub color: String,
    /// Width
    pub width: f32,
    /// Dash pattern
    pub dash: Option<Vec<f32>>,
}

impl TrendLine {
    /// Create linear trend line
    pub fn linear() -> Self {
        Self {
            line_type: TrendLineType::Linear,
            color: "#FF0000".to_string(),
            width: 2.0,
            dash: Some(vec![4.0, 4.0]),
        }
    }

    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }
}

/// Scatter plot
#[derive(Debug, Clone)]
pub struct ScatterPlot {
    /// Data points
    pub data: Vec<ScatterPoint>,
    /// X-axis
    pub x_axis: Axis,
    /// Y-axis
    pub y_axis: Axis,
    /// Legend
    pub legend: Option<Legend>,
    /// Animation config
    pub animation: AnimationConfig,
    /// Point size
    pub point_size: f32,
    /// Point color
    pub point_color: String,
    /// Trend line
    pub trend_line: Option<TrendLine>,
}

impl ScatterPlot {
    /// Create new scatter plot
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            x_axis: Axis::new(),
            y_axis: Axis::new(),
            legend: None,
            animation: AnimationConfig::default(),
            point_size: 6.0,
            point_color: "#2196F3".to_string(),
            trend_line: None,
        }
    }

    /// Add point
    pub fn point(mut self, x: f64, y: f64) -> Self {
        self.data.push(ScatterPoint::new(x, y));
        self
    }

    /// Set data
    pub fn data(mut self, data: Vec<ScatterPoint>) -> Self {
        self.data = data;
        self
    }

    /// Set X-axis
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        self
    }

    /// Set Y-axis
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = axis;
        self
    }

    /// Add trend line
    pub fn trend_line(mut self, trend: TrendLine) -> Self {
        self.trend_line = Some(trend);
        self
    }

    /// Set point size
    pub fn point_size(mut self, size: f32) -> Self {
        self.point_size = size;
        self
    }

    /// Enable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.animation.enabled = animate;
        self
    }
}

impl Default for ScatterPlot {
    fn default() -> Self {
        Self::new()
    }
}

/// Bubble chart (scatter with size dimension)
#[derive(Debug, Clone)]
pub struct BubbleChart {
    /// Inner scatter plot
    pub inner: ScatterPlot,
    /// Min bubble size
    pub min_size: f32,
    /// Max bubble size
    pub max_size: f32,
}

impl BubbleChart {
    /// Create new bubble chart
    pub fn new() -> Self {
        Self {
            inner: ScatterPlot::new(),
            min_size: 5.0,
            max_size: 50.0,
        }
    }

    /// Add bubble
    pub fn bubble(mut self, x: f64, y: f64, size: f64) -> Self {
        self.inner.data.push(ScatterPoint::new(x, y).with_size(size));
        self
    }

    /// Set size range
    pub fn size_range(mut self, min: f32, max: f32) -> Self {
        self.min_size = min;
        self.max_size = max;
        self
    }

    /// Enable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.inner.animation.enabled = animate;
        self
    }
}

impl Default for BubbleChart {
    fn default() -> Self {
        Self::new()
    }
}
