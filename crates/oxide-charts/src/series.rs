//! Data Series and Points
//!
//! Structures for representing chart data.

use oxide_render::Color;
use serde::{Deserialize, Serialize};

/// A single data point in a chart
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DataPoint {
    /// X coordinate value
    pub x: f64,
    /// Y coordinate value
    pub y: f64,
    /// Optional additional value (e.g., for bubble size)
    pub z: Option<f64>,
    /// Whether this point is highlighted
    #[serde(skip)]
    pub highlighted: bool,
    /// Whether this point is selected
    #[serde(skip)]
    pub selected: bool,
}

impl DataPoint {
    /// Create a new data point
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            z: None,
            highlighted: false,
            selected: false,
        }
    }

    /// Create a data point with a Z value (for bubble charts)
    pub fn with_z(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z: Some(z),
            highlighted: false,
            selected: false,
        }
    }

    /// Create from a tuple
    pub fn from_tuple(xy: (f64, f64)) -> Self {
        Self::new(xy.0, xy.1)
    }

    /// Create from a 3-tuple with Z value
    pub fn from_tuple3(xyz: (f64, f64, f64)) -> Self {
        Self::with_z(xyz.0, xyz.1, xyz.2)
    }
}

impl From<(f64, f64)> for DataPoint {
    fn from((x, y): (f64, f64)) -> Self {
        Self::new(x, y)
    }
}

impl From<(f64, f64, f64)> for DataPoint {
    fn from((x, y, z): (f64, f64, f64)) -> Self {
        Self::with_z(x, y, z)
    }
}

impl From<(f32, f32)> for DataPoint {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x as f64, y as f64)
    }
}

impl From<f64> for DataPoint {
    /// Creates a point with automatic X index
    fn from(y: f64) -> Self {
        Self::new(0.0, y) // X will be set by index
    }
}

/// Style for a data series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesStyle {
    /// Primary color
    pub color: Option<String>,
    /// Line width (for line charts)
    pub line_width: f32,
    /// Point radius
    pub point_radius: f32,
    /// Whether to show points
    pub show_points: bool,
    /// Fill opacity (for area charts)
    pub fill_opacity: f32,
    /// Dash pattern (e.g., [5, 5] for dashed line)
    pub dash_pattern: Option<Vec<f32>>,
}

impl Default for SeriesStyle {
    fn default() -> Self {
        Self {
            color: None,
            line_width: 2.0,
            point_radius: 4.0,
            show_points: true,
            fill_opacity: 0.3,
            dash_pattern: None,
        }
    }
}

impl SeriesStyle {
    /// Create a new series style
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the line width
    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Set the point radius
    pub fn point_radius(mut self, radius: f32) -> Self {
        self.point_radius = radius;
        self
    }

    /// Set whether to show points
    pub fn show_points(mut self, show: bool) -> Self {
        self.show_points = show;
        self
    }

    /// Set the fill opacity
    pub fn fill_opacity(mut self, opacity: f32) -> Self {
        self.fill_opacity = opacity;
        self
    }

    /// Set a dashed line pattern
    pub fn dashed(mut self, dash: f32, gap: f32) -> Self {
        self.dash_pattern = Some(vec![dash, gap]);
        self
    }
}

/// A series of data points with a name and styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    /// Series name (for legend)
    pub name: String,
    /// Data points
    pub data: Vec<DataPoint>,
    /// Series style
    pub style: SeriesStyle,
    /// Whether this series is visible
    #[serde(skip)]
    pub visible: bool,
    /// Category labels (for categorical data)
    pub labels: Option<Vec<String>>,
}

impl DataSeries {
    /// Create a new named data series
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            style: SeriesStyle::default(),
            visible: true,
            labels: None,
        }
    }

    /// Create a series with data points
    pub fn with_data(mut self, data: impl IntoIterator<Item = impl Into<DataPoint>>) -> Self {
        self.data = data.into_iter().enumerate().map(|(i, p)| {
            let mut point: DataPoint = p.into();
            // If X is 0 and this isn't the first point, auto-index
            if point.x == 0.0 && i > 0 {
                point.x = i as f64;
            } else if point.x == 0.0 && i == 0 {
                // First point at index 0 is valid
            }
            point
        }).collect();
        self
    }

    /// Create a series from Y values only (X will be indices)
    pub fn from_values(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        let data: Vec<DataPoint> = values
            .into_iter()
            .enumerate()
            .map(|(i, y)| DataPoint::new(i as f64, y))
            .collect();

        Self {
            name: name.into(),
            data,
            style: SeriesStyle::default(),
            visible: true,
            labels: None,
        }
    }

    /// Add a data point
    pub fn add_point(&mut self, point: impl Into<DataPoint>) {
        self.data.push(point.into());
    }

    /// Add a point by coordinates
    pub fn add(&mut self, x: f64, y: f64) {
        self.data.push(DataPoint::new(x, y));
    }

    /// Set the style
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.style.color = Some(color.into());
        self
    }

    /// Set category labels
    pub fn with_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    /// Get the number of points
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the minimum Y value
    pub fn min_y(&self) -> Option<f64> {
        self.data.iter().map(|p| p.y).reduce(f64::min)
    }

    /// Get the maximum Y value
    pub fn max_y(&self) -> Option<f64> {
        self.data.iter().map(|p| p.y).reduce(f64::max)
    }

    /// Get the minimum X value
    pub fn min_x(&self) -> Option<f64> {
        self.data.iter().map(|p| p.x).reduce(f64::min)
    }

    /// Get the maximum X value
    pub fn max_x(&self) -> Option<f64> {
        self.data.iter().map(|p| p.x).reduce(f64::max)
    }

    /// Get the sum of all Y values
    pub fn sum_y(&self) -> f64 {
        self.data.iter().map(|p| p.y).sum()
    }

    /// Get the average Y value
    pub fn avg_y(&self) -> Option<f64> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.sum_y() / self.data.len() as f64)
        }
    }

    /// Get a label for an index
    pub fn label_at(&self, index: usize) -> Option<&str> {
        self.labels.as_ref().and_then(|l| l.get(index).map(|s| s.as_str()))
    }

    /// Get the resolved color (from style or default)
    pub fn resolved_color(&self, default: Color) -> Color {
        self.style
            .color
            .as_ref()
            .and_then(|c| Color::from_hex(c))
            .unwrap_or(default)
    }
}

/// Builder for creating pie/donut chart data
#[derive(Debug, Clone)]
pub struct PieData {
    /// Segments of the pie
    pub segments: Vec<PieSegment>,
}

/// A segment of a pie chart
#[derive(Debug, Clone)]
pub struct PieSegment {
    /// Segment label
    pub label: String,
    /// Segment value
    pub value: f64,
    /// Optional color override
    pub color: Option<Color>,
    /// Whether this segment is exploded (pulled out)
    pub exploded: bool,
}

impl PieSegment {
    /// Create a new pie segment
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
            exploded: false,
        }
    }

    /// Set the color
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Mark as exploded
    pub fn exploded(mut self) -> Self {
        self.exploded = true;
        self
    }
}

impl PieData {
    /// Create new pie data
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    /// Add a segment
    pub fn segment(mut self, label: impl Into<String>, value: f64) -> Self {
        self.segments.push(PieSegment::new(label, value));
        self
    }

    /// Add a segment with color
    pub fn segment_colored(mut self, label: impl Into<String>, value: f64, color: Color) -> Self {
        self.segments.push(PieSegment::new(label, value).color(color));
        self
    }

    /// Add an exploded segment
    pub fn segment_exploded(mut self, label: impl Into<String>, value: f64) -> Self {
        self.segments.push(PieSegment::new(label, value).exploded());
        self
    }

    /// Get total value
    pub fn total(&self) -> f64 {
        self.segments.iter().map(|s| s.value).sum()
    }

    /// Get percentage for a segment
    pub fn percentage(&self, index: usize) -> f64 {
        let total = self.total();
        if total == 0.0 {
            0.0
        } else {
            self.segments.get(index).map(|s| s.value / total * 100.0).unwrap_or(0.0)
        }
    }
}

impl Default for PieData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_point_creation() {
        let p = DataPoint::new(5.0, 10.0);
        assert_eq!(p.x, 5.0);
        assert_eq!(p.y, 10.0);
        assert!(p.z.is_none());
    }

    #[test]
    fn test_data_point_with_z() {
        let p = DataPoint::with_z(1.0, 2.0, 3.0);
        assert_eq!(p.z, Some(3.0));
    }

    #[test]
    fn test_data_point_from_tuple() {
        let p: DataPoint = (5.0, 10.0).into();
        assert_eq!(p.x, 5.0);
        assert_eq!(p.y, 10.0);
    }

    #[test]
    fn test_series_creation() {
        let series = DataSeries::new("Test Series")
            .with_data(vec![(0.0, 10.0), (1.0, 20.0), (2.0, 30.0)]);

        assert_eq!(series.name, "Test Series");
        assert_eq!(series.len(), 3);
    }

    #[test]
    fn test_series_from_values() {
        let series = DataSeries::from_values("Y Values", vec![10.0, 20.0, 30.0]);

        assert_eq!(series.len(), 3);
        assert_eq!(series.data[0].x, 0.0);
        assert_eq!(series.data[1].x, 1.0);
        assert_eq!(series.data[2].x, 2.0);
    }

    #[test]
    fn test_series_statistics() {
        let series = DataSeries::from_values("Stats", vec![10.0, 20.0, 30.0, 40.0]);

        assert_eq!(series.min_y(), Some(10.0));
        assert_eq!(series.max_y(), Some(40.0));
        assert_eq!(series.sum_y(), 100.0);
        assert_eq!(series.avg_y(), Some(25.0));
    }

    #[test]
    fn test_series_with_labels() {
        let series = DataSeries::from_values("Labeled", vec![10.0, 20.0, 30.0])
            .with_labels(vec!["Jan", "Feb", "Mar"]);

        assert_eq!(series.label_at(0), Some("Jan"));
        assert_eq!(series.label_at(1), Some("Feb"));
        assert_eq!(series.label_at(2), Some("Mar"));
        assert_eq!(series.label_at(3), None);
    }

    #[test]
    fn test_series_style() {
        let style = SeriesStyle::new()
            .color("#FF5500")
            .line_width(3.0)
            .point_radius(5.0)
            .dashed(5.0, 3.0);

        assert_eq!(style.color, Some("#FF5500".to_string()));
        assert_eq!(style.line_width, 3.0);
        assert_eq!(style.point_radius, 5.0);
        assert_eq!(style.dash_pattern, Some(vec![5.0, 3.0]));
    }

    #[test]
    fn test_pie_data() {
        let pie = PieData::new()
            .segment("Chrome", 65.0)
            .segment("Firefox", 15.0)
            .segment("Safari", 10.0)
            .segment("Other", 10.0);

        assert_eq!(pie.segments.len(), 4);
        assert_eq!(pie.total(), 100.0);
        assert!((pie.percentage(0) - 65.0).abs() < 0.01);
    }

    #[test]
    fn test_pie_segment_exploded() {
        let segment = PieSegment::new("Test", 50.0).exploded();
        assert!(segment.exploded);
    }
}
