//! Bar charts.

use serde::{Deserialize, Serialize};
use crate::series::DataSeries;
use crate::axis::Axis;
use crate::legend::Legend;
use crate::animation::AnimationConfig;

/// Bar orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BarOrientation {
    /// Vertical bars
    #[default]
    Vertical,
    /// Horizontal bars
    Horizontal,
}

/// Stacking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StackMode {
    /// No stacking (grouped)
    #[default]
    None,
    /// Stacked
    Stacked,
    /// Stacked to 100%
    Percent,
}

/// Bar style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarStyle {
    /// Border radius
    pub border_radius: f32,
    /// Border width
    pub border_width: f32,
    /// Border color
    pub border_color: Option<String>,
    /// Opacity
    pub opacity: f32,
    /// Bar spacing (0.0 to 1.0)
    pub spacing: f32,
}

impl Default for BarStyle {
    fn default() -> Self {
        Self {
            border_radius: 0.0,
            border_width: 0.0,
            border_color: None,
            opacity: 1.0,
            spacing: 0.2,
        }
    }
}

/// Bar chart
#[derive(Debug, Clone)]
pub struct BarChart {
    /// Data series
    pub series: Vec<DataSeries>,
    /// X-axis
    pub x_axis: Axis,
    /// Y-axis
    pub y_axis: Axis,
    /// Legend
    pub legend: Option<Legend>,
    /// Animation config
    pub animation: AnimationConfig,
    /// Orientation
    pub orientation: BarOrientation,
    /// Stack mode
    pub stack_mode: StackMode,
    /// Style
    pub style: BarStyle,
}

impl BarChart {
    /// Create new bar chart
    pub fn new() -> Self {
        Self {
            series: Vec::new(),
            x_axis: Axis::new(),
            y_axis: Axis::new(),
            legend: Some(Legend::bottom()),
            animation: AnimationConfig::default(),
            orientation: BarOrientation::Vertical,
            stack_mode: StackMode::None,
            style: BarStyle::default(),
        }
    }

    /// Add series
    pub fn series(mut self, name: impl Into<String>, data: Vec<f64>) -> Self {
        // Convert f64 values to DataPoints with index as x coordinate
        let points: Vec<_> = data.into_iter().enumerate()
            .map(|(i, v)| (i as f64, v))
            .collect();
        self.series.push(DataSeries::new(name).with_data(points));
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

    /// Set legend
    pub fn legend(mut self, legend: Legend) -> Self {
        self.legend = Some(legend);
        self
    }

    /// Set horizontal orientation
    pub fn horizontal(mut self) -> Self {
        self.orientation = BarOrientation::Horizontal;
        self
    }

    /// Set stacked mode
    pub fn stacked(mut self) -> Self {
        self.stack_mode = StackMode::Stacked;
        self
    }

    /// Set 100% stacked mode
    pub fn percent_stacked(mut self) -> Self {
        self.stack_mode = StackMode::Percent;
        self
    }

    /// Enable animation
    pub fn animate(mut self, animate: bool) -> Self {
        self.animation.enabled = animate;
        self
    }

    /// Set style
    pub fn style(mut self, style: BarStyle) -> Self {
        self.style = style;
        self
    }
}

impl Default for BarChart {
    fn default() -> Self {
        Self::new()
    }
}
