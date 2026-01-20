//! Base Chart Structure and Configuration
//!
//! Provides the foundational types and traits for all chart implementations.

use oxide_render::{Color, Rect};
use serde::{Deserialize, Serialize};

use crate::axis::Axis;
use crate::legend::Legend;
use crate::series::DataSeries;
use crate::theme::ChartTheme;
use crate::tooltip::TooltipConfig;
use crate::animation::AnimationConfig;

/// Represents the bounds of a chart's data
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChartBounds {
    /// Minimum X value
    pub x_min: f64,
    /// Maximum X value
    pub x_max: f64,
    /// Minimum Y value
    pub y_min: f64,
    /// Maximum Y value
    pub y_max: f64,
}

impl ChartBounds {
    /// Create new chart bounds
    pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self { x_min, x_max, y_min, y_max }
    }

    /// Create bounds from data series
    pub fn from_series(series: &[DataSeries]) -> Self {
        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        for s in series {
            for point in &s.data {
                x_min = x_min.min(point.x);
                x_max = x_max.max(point.x);
                y_min = y_min.min(point.y);
                y_max = y_max.max(point.y);
            }
        }

        // Handle empty series
        if x_min > x_max {
            x_min = 0.0;
            x_max = 1.0;
        }
        if y_min > y_max {
            y_min = 0.0;
            y_max = 1.0;
        }

        Self { x_min, x_max, y_min, y_max }
    }

    /// Expand bounds to include nice round numbers
    pub fn expand_nice(&self) -> Self {
        let y_range = self.y_max - self.y_min;
        let x_range = self.x_max - self.x_min;

        let y_padding = y_range * 0.1;
        let x_padding = x_range * 0.1;

        Self {
            x_min: (self.x_min - x_padding).floor(),
            x_max: (self.x_max + x_padding).ceil(),
            y_min: if self.y_min >= 0.0 { 0.0 } else { (self.y_min - y_padding).floor() },
            y_max: (self.y_max + y_padding).ceil(),
        }
    }

    /// Get the width of the bounds
    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    /// Get the height of the bounds
    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }
}

impl Default for ChartBounds {
    fn default() -> Self {
        Self {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        }
    }
}

/// Represents the drawable area of a chart (excluding margins/padding)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartArea {
    /// Left position
    pub x: f32,
    /// Top position
    pub y: f32,
    /// Width of the chart area
    pub width: f32,
    /// Height of the chart area
    pub height: f32,
}

impl ChartArea {
    /// Create a new chart area
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create from a rect
    pub fn from_rect(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }

    /// Convert to rect
    pub fn to_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }

    /// Get the right edge
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get the bottom edge
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Apply margins to get inner area
    pub fn with_margins(&self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            x: self.x + left,
            y: self.y + top,
            width: (self.width - left - right).max(0.0),
            height: (self.height - top - bottom).max(0.0),
        }
    }

    /// Map a data point (0.0-1.0 normalized) to pixel coordinates
    pub fn map_point(&self, x_norm: f32, y_norm: f32) -> (f32, f32) {
        let px = self.x + x_norm * self.width;
        // Y is inverted (screen coordinates)
        let py = self.y + (1.0 - y_norm) * self.height;
        (px, py)
    }

    /// Check if a point is inside the chart area
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.right() && y >= self.y && y <= self.bottom()
    }
}

impl Default for ChartArea {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
        }
    }
}

/// Chart interaction events
#[derive(Debug, Clone, PartialEq)]
pub enum ChartEvent {
    /// Mouse/touch hover over a data point
    Hover {
        series_index: usize,
        point_index: usize,
        x: f32,
        y: f32,
    },
    /// Click on a data point
    Click {
        series_index: usize,
        point_index: usize,
    },
    /// Selection changed
    SelectionChanged {
        selected: Vec<(usize, usize)>,
    },
    /// Zoom level changed
    Zoom {
        scale: f32,
        center_x: f32,
        center_y: f32,
    },
    /// Pan offset changed
    Pan {
        offset_x: f32,
        offset_y: f32,
    },
    /// Legend item toggled
    LegendToggle {
        series_index: usize,
        visible: bool,
    },
}

/// Chart interaction state
#[derive(Debug, Clone, Default)]
pub struct ChartInteraction {
    /// Currently hovered point (series_index, point_index)
    pub hover_point: Option<(usize, usize)>,
    /// Currently selected points
    pub selected_points: Vec<(usize, usize)>,
    /// Zoom scale (1.0 = no zoom)
    pub zoom_scale: f32,
    /// Pan offset X
    pub pan_x: f32,
    /// Pan offset Y
    pub pan_y: f32,
    /// Hidden series indices
    pub hidden_series: Vec<usize>,
    /// Mouse position
    pub mouse_x: f32,
    pub mouse_y: f32,
}

impl ChartInteraction {
    /// Create new interaction state
    pub fn new() -> Self {
        Self {
            zoom_scale: 1.0,
            ..Default::default()
        }
    }

    /// Check if a series is visible
    pub fn is_series_visible(&self, index: usize) -> bool {
        !self.hidden_series.contains(&index)
    }

    /// Toggle series visibility
    pub fn toggle_series(&mut self, index: usize) {
        if let Some(pos) = self.hidden_series.iter().position(|&i| i == index) {
            self.hidden_series.remove(pos);
        } else {
            self.hidden_series.push(index);
        }
    }

    /// Set hover point
    pub fn set_hover(&mut self, series_index: usize, point_index: usize) {
        self.hover_point = Some((series_index, point_index));
    }

    /// Clear hover
    pub fn clear_hover(&mut self) {
        self.hover_point = None;
    }

    /// Update mouse position
    pub fn update_mouse(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }
}

/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart title
    pub title: Option<String>,
    /// Chart subtitle
    pub subtitle: Option<String>,
    /// Background color
    pub background: Option<String>,
    /// Margin top
    pub margin_top: f32,
    /// Margin right
    pub margin_right: f32,
    /// Margin bottom
    pub margin_bottom: f32,
    /// Margin left
    pub margin_left: f32,
    /// Enable responsiveness
    pub responsive: bool,
    /// Maintain aspect ratio
    pub maintain_aspect_ratio: bool,
    /// Aspect ratio (width/height)
    pub aspect_ratio: f32,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            title: None,
            subtitle: None,
            background: None,
            margin_top: 20.0,
            margin_right: 20.0,
            margin_bottom: 40.0,
            margin_left: 50.0,
            responsive: true,
            maintain_aspect_ratio: true,
            aspect_ratio: 16.0 / 9.0,
        }
    }
}

/// Base chart trait that all chart types implement
pub trait Chart {
    /// Get the data series
    fn series(&self) -> &[DataSeries];

    /// Get chart bounds
    fn bounds(&self) -> ChartBounds;

    /// Get the chart area
    fn area(&self) -> ChartArea;

    /// Set the chart area
    fn set_area(&mut self, area: ChartArea);

    /// Get the theme
    fn theme(&self) -> &ChartTheme;

    /// Get the X axis
    fn x_axis(&self) -> Option<&Axis>;

    /// Get the Y axis
    fn y_axis(&self) -> Option<&Axis>;

    /// Get the legend
    fn legend(&self) -> Option<&Legend>;

    /// Get tooltip config
    fn tooltip_config(&self) -> Option<&TooltipConfig>;

    /// Get animation config
    fn animation_config(&self) -> Option<&AnimationConfig>;

    /// Check if animation is enabled
    fn is_animated(&self) -> bool {
        self.animation_config().is_some()
    }

    /// Generate render primitives for the chart
    fn render(&self, interaction: &ChartInteraction) -> Vec<ChartPrimitive>;

    /// Handle a chart event
    fn handle_event(&mut self, _event: &ChartEvent) {
        // Default: no-op
    }

    /// Find the nearest data point to a position
    fn hit_test(&self, x: f32, y: f32, threshold: f32) -> Option<(usize, usize)>;
}

/// A primitive to render for charts
#[derive(Debug, Clone)]
pub enum ChartPrimitive {
    /// A filled rectangle
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        radius: f32,
    },
    /// A line segment
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
        width: f32,
    },
    /// A circle
    Circle {
        cx: f32,
        cy: f32,
        radius: f32,
        color: Color,
        stroke_color: Option<Color>,
        stroke_width: f32,
    },
    /// An arc (for pie/donut charts)
    Arc {
        cx: f32,
        cy: f32,
        inner_radius: f32,
        outer_radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: Color,
    },
    /// A path (series of connected points)
    Path {
        points: Vec<(f32, f32)>,
        color: Color,
        width: f32,
        closed: bool,
    },
    /// A filled area under a path
    Area {
        points: Vec<(f32, f32)>,
        color: Color,
        gradient: Option<(Color, Color)>,
    },
    /// Text label
    Text {
        x: f32,
        y: f32,
        text: String,
        color: Color,
        size: f32,
        align: TextAlign,
    },
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Left
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_bounds_from_empty_series() {
        let bounds = ChartBounds::from_series(&[]);
        assert_eq!(bounds.x_min, 0.0);
        assert_eq!(bounds.x_max, 1.0);
        assert_eq!(bounds.y_min, 0.0);
        assert_eq!(bounds.y_max, 1.0);
    }

    #[test]
    fn test_chart_bounds_from_series() {
        let series = vec![
            DataSeries::new("test").with_data(vec![(0.0, 10.0), (5.0, 20.0), (10.0, 15.0)]),
        ];
        let bounds = ChartBounds::from_series(&series);
        assert_eq!(bounds.x_min, 0.0);
        assert_eq!(bounds.x_max, 10.0);
        assert_eq!(bounds.y_min, 10.0);
        assert_eq!(bounds.y_max, 20.0);
    }

    #[test]
    fn test_chart_bounds_expand_nice() {
        let bounds = ChartBounds::new(0.5, 9.5, 10.0, 100.0);
        let expanded = bounds.expand_nice();
        assert!(expanded.x_min <= 0.5);
        assert!(expanded.x_max >= 9.5);
        assert_eq!(expanded.y_min, 0.0); // Should start at 0 for positive data
        assert!(expanded.y_max >= 100.0);
    }

    #[test]
    fn test_chart_area_map_point() {
        let area = ChartArea::new(100.0, 50.0, 400.0, 300.0);

        // Top-left corner (0, 1) should map to (100, 50)
        let (x, y) = area.map_point(0.0, 1.0);
        assert!((x - 100.0).abs() < 0.01);
        assert!((y - 50.0).abs() < 0.01);

        // Bottom-right corner (1, 0) should map to (500, 350)
        let (x, y) = area.map_point(1.0, 0.0);
        assert!((x - 500.0).abs() < 0.01);
        assert!((y - 350.0).abs() < 0.01);
    }

    #[test]
    fn test_chart_area_contains() {
        let area = ChartArea::new(100.0, 50.0, 400.0, 300.0);
        assert!(area.contains(200.0, 200.0));
        assert!(!area.contains(50.0, 200.0));
        assert!(!area.contains(600.0, 200.0));
    }

    #[test]
    fn test_chart_area_with_margins() {
        let area = ChartArea::new(0.0, 0.0, 400.0, 300.0);
        let inner = area.with_margins(20.0, 20.0, 40.0, 50.0);
        assert_eq!(inner.x, 50.0);
        assert_eq!(inner.y, 20.0);
        assert_eq!(inner.width, 330.0);
        assert_eq!(inner.height, 240.0);
    }

    #[test]
    fn test_chart_interaction_series_visibility() {
        let mut interaction = ChartInteraction::new();
        assert!(interaction.is_series_visible(0));

        interaction.toggle_series(0);
        assert!(!interaction.is_series_visible(0));

        interaction.toggle_series(0);
        assert!(interaction.is_series_visible(0));
    }

    #[test]
    fn test_chart_config_default() {
        let config = ChartConfig::default();
        assert!(config.responsive);
        assert!(config.maintain_aspect_ratio);
        assert!((config.aspect_ratio - 16.0 / 9.0).abs() < 0.01);
    }
}
