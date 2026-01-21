//! Line and Area Charts
//!
//! Provides line charts with support for:
//! - Single and multi-series lines
//! - Curved (spline) and straight lines
//! - Area charts (filled under line)
//! - Gradient fills

use oxide_render::Color;
use serde::{Deserialize, Serialize};

use crate::animation::AnimationConfig;
use crate::axis::Axis;
use crate::chart::{Chart, ChartArea, ChartBounds, ChartInteraction, ChartPrimitive, TextAlign};
use crate::legend::Legend;
use crate::series::{DataPoint, DataSeries};
use crate::theme::ChartTheme;
use crate::tooltip::TooltipConfig;

/// Line style configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineStyle {
    /// Straight lines between points
    Straight,
    /// Smooth curved lines (spline interpolation)
    Curved,
    /// Stepped lines (horizontal then vertical)
    Stepped,
    /// Stepped lines (vertical then horizontal)
    SteppedBefore,
}

impl Default for LineStyle {
    fn default() -> Self {
        LineStyle::Straight
    }
}

/// Point style configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PointStyle {
    /// No points shown
    None,
    /// Circular points
    Circle { radius: f32 },
    /// Square points
    Square { size: f32 },
    /// Diamond points
    Diamond { size: f32 },
    /// Triangle points
    Triangle { size: f32 },
}

impl Default for PointStyle {
    fn default() -> Self {
        PointStyle::Circle { radius: 4.0 }
    }
}

impl PointStyle {
    /// Get the effective radius/size for hit testing
    pub fn hit_radius(&self) -> f32 {
        match self {
            PointStyle::None => 8.0, // Still allow hovering
            PointStyle::Circle { radius } => *radius + 4.0,
            PointStyle::Square { size } => *size / 2.0 + 4.0,
            PointStyle::Diamond { size } => *size / 2.0 + 4.0,
            PointStyle::Triangle { size } => *size / 2.0 + 4.0,
        }
    }
}

/// Line chart implementation
#[derive(Debug, Clone)]
pub struct LineChart {
    /// Data series
    series: Vec<DataSeries>,
    /// Chart theme
    theme: ChartTheme,
    /// X axis configuration
    x_axis: Option<Axis>,
    /// Y axis configuration
    y_axis: Option<Axis>,
    /// Legend configuration
    legend: Option<Legend>,
    /// Tooltip configuration
    tooltip: Option<TooltipConfig>,
    /// Animation configuration
    animation: Option<AnimationConfig>,
    /// Chart area (set during rendering)
    area: ChartArea,
    /// Line style
    line_style: LineStyle,
    /// Point style
    point_style: PointStyle,
    /// Line width
    line_width: f32,
    /// Show grid lines
    show_grid: bool,
    /// Tension for curved lines (0.0-1.0)
    tension: f32,
}

impl LineChart {
    /// Create a new line chart
    pub fn new() -> Self {
        Self {
            series: Vec::new(),
            theme: ChartTheme::default(),
            x_axis: Some(Axis::new()),
            y_axis: Some(Axis::new()),
            legend: None,
            tooltip: Some(TooltipConfig::default()),
            animation: None,
            area: ChartArea::default(),
            line_style: LineStyle::Straight,
            point_style: PointStyle::default(),
            line_width: 2.0,
            show_grid: true,
            tension: 0.4,
        }
    }

    /// Add a data series
    pub fn series(mut self, name: impl Into<String>, data: impl IntoIterator<Item = impl Into<DataPoint>>) -> Self {
        let series = DataSeries::new(name).with_data(data);
        self.series.push(series);
        self
    }

    /// Add a pre-built data series
    pub fn add_series(mut self, series: DataSeries) -> Self {
        self.series.push(series);
        self
    }

    /// Set the theme
    pub fn theme(mut self, theme: ChartTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the X axis
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = Some(axis);
        self
    }

    /// Set the Y axis
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = Some(axis);
        self
    }

    /// Set the legend
    pub fn legend(mut self, legend: Legend) -> Self {
        self.legend = Some(legend);
        self
    }

    /// Enable/disable tooltip
    pub fn tooltip(mut self, enabled: bool) -> Self {
        self.tooltip = if enabled { Some(TooltipConfig::default()) } else { None };
        self
    }

    /// Set tooltip configuration
    pub fn tooltip_config(mut self, config: TooltipConfig) -> Self {
        self.tooltip = Some(config);
        self
    }

    /// Enable animation
    pub fn animate(mut self, enabled: bool) -> Self {
        self.animation = if enabled { Some(AnimationConfig::default()) } else { None };
        self
    }

    /// Set animation configuration
    pub fn animation_config(mut self, config: AnimationConfig) -> Self {
        self.animation = Some(config);
        self
    }

    /// Set line style
    pub fn line_style(mut self, style: LineStyle) -> Self {
        self.line_style = style;
        self
    }

    /// Use curved lines
    pub fn curved(mut self) -> Self {
        self.line_style = LineStyle::Curved;
        self
    }

    /// Use stepped lines
    pub fn stepped(mut self) -> Self {
        self.line_style = LineStyle::Stepped;
        self
    }

    /// Set point style
    pub fn point_style(mut self, style: PointStyle) -> Self {
        self.point_style = style;
        self
    }

    /// Hide points
    pub fn hide_points(mut self) -> Self {
        self.point_style = PointStyle::None;
        self
    }

    /// Set line width
    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Show/hide grid
    pub fn grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Set curve tension (0.0-1.0)
    pub fn tension(mut self, tension: f32) -> Self {
        self.tension = tension.clamp(0.0, 1.0);
        self
    }

    /// Generate line points with the configured style
    fn generate_line_points(&self, series: &DataSeries, bounds: &ChartBounds) -> Vec<(f32, f32)> {
        if series.data.is_empty() {
            return Vec::new();
        }

        let points: Vec<(f32, f32)> = series.data.iter().map(|p| {
            let x_norm = ((p.x - bounds.x_min) / bounds.width()) as f32;
            let y_norm = ((p.y - bounds.y_min) / bounds.height()) as f32;
            self.area.map_point(x_norm, y_norm)
        }).collect();

        match self.line_style {
            LineStyle::Straight => points,
            LineStyle::Curved => self.catmull_rom_spline(&points),
            LineStyle::Stepped => self.stepped_points(&points, false),
            LineStyle::SteppedBefore => self.stepped_points(&points, true),
        }
    }

    /// Generate Catmull-Rom spline points for smooth curves
    fn catmull_rom_spline(&self, points: &[(f32, f32)]) -> Vec<(f32, f32)> {
        if points.len() < 2 {
            return points.to_vec();
        }

        let mut result = Vec::new();
        let segments = 16; // Points per segment

        for i in 0..points.len() - 1 {
            let p0 = if i == 0 { points[0] } else { points[i - 1] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < points.len() { points[i + 2] } else { points[i + 1] };

            for j in 0..segments {
                let t = j as f32 / segments as f32;
                let t2 = t * t;
                let t3 = t2 * t;

                // Catmull-Rom spline formula
                let x = 0.5 * ((2.0 * p1.0)
                    + (-p0.0 + p2.0) * t
                    + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
                    + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);

                let y = 0.5 * ((2.0 * p1.1)
                    + (-p0.1 + p2.1) * t
                    + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
                    + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);

                result.push((x, y));
            }
        }

        // Add the last point
        if let Some(&last) = points.last() {
            result.push(last);
        }

        result
    }

    /// Generate stepped line points
    fn stepped_points(&self, points: &[(f32, f32)], before: bool) -> Vec<(f32, f32)> {
        if points.len() < 2 {
            return points.to_vec();
        }

        let mut result = Vec::with_capacity(points.len() * 2);

        for i in 0..points.len() - 1 {
            let current = points[i];
            let next = points[i + 1];

            result.push(current);

            if before {
                result.push((current.0, next.1));
            } else {
                result.push((next.0, current.1));
            }
        }

        if let Some(&last) = points.last() {
            result.push(last);
        }

        result
    }
}

impl Default for LineChart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart for LineChart {
    fn series(&self) -> &[DataSeries] {
        &self.series
    }

    fn bounds(&self) -> ChartBounds {
        ChartBounds::from_series(&self.series).expand_nice()
    }

    fn area(&self) -> ChartArea {
        self.area
    }

    fn set_area(&mut self, area: ChartArea) {
        self.area = area;
    }

    fn theme(&self) -> &ChartTheme {
        &self.theme
    }

    fn x_axis(&self) -> Option<&Axis> {
        self.x_axis.as_ref()
    }

    fn y_axis(&self) -> Option<&Axis> {
        self.y_axis.as_ref()
    }

    fn legend(&self) -> Option<&Legend> {
        self.legend.as_ref()
    }

    fn tooltip_config(&self) -> Option<&TooltipConfig> {
        self.tooltip.as_ref()
    }

    fn animation_config(&self) -> Option<&AnimationConfig> {
        self.animation.as_ref()
    }

    fn render(&self, interaction: &ChartInteraction) -> Vec<ChartPrimitive> {
        let mut primitives = Vec::new();
        let bounds = self.bounds();

        // Background
        primitives.push(ChartPrimitive::Rect {
            x: self.area.x,
            y: self.area.y,
            width: self.area.width,
            height: self.area.height,
            color: self.theme.background,
            radius: 0.0,
        });

        // Grid lines
        if self.show_grid {
            let grid_color = self.theme.grid_color;
            let num_lines = 5;

            // Horizontal grid lines
            for i in 0..=num_lines {
                let y = self.area.y + (i as f32 / num_lines as f32) * self.area.height;
                primitives.push(ChartPrimitive::Line {
                    x1: self.area.x,
                    y1: y,
                    x2: self.area.right(),
                    y2: y,
                    color: grid_color,
                    width: 1.0,
                });
            }

            // Vertical grid lines
            for i in 0..=num_lines {
                let x = self.area.x + (i as f32 / num_lines as f32) * self.area.width;
                primitives.push(ChartPrimitive::Line {
                    x1: x,
                    y1: self.area.y,
                    x2: x,
                    y2: self.area.bottom(),
                    color: grid_color,
                    width: 1.0,
                });
            }
        }

        // Render each series
        for (series_idx, series) in self.series.iter().enumerate() {
            if !interaction.is_series_visible(series_idx) {
                continue;
            }

            let color = series.resolved_color(self.theme.series_color(series_idx));
            let line_points = self.generate_line_points(series, &bounds);

            // Draw line
            if line_points.len() >= 2 {
                primitives.push(ChartPrimitive::Path {
                    points: line_points.clone(),
                    color,
                    width: self.line_width,
                    closed: false,
                });
            }

            // Draw points
            if !matches!(self.point_style, PointStyle::None) {
                for (point_idx, point) in series.data.iter().enumerate() {
                    let x_norm = ((point.x - bounds.x_min) / bounds.width()) as f32;
                    let y_norm = ((point.y - bounds.y_min) / bounds.height()) as f32;
                    let (px, py) = self.area.map_point(x_norm, y_norm);

                    let is_hovered = interaction.hover_point == Some((series_idx, point_idx));
                    let radius_mult = if is_hovered { 1.5 } else { 1.0 };

                    match self.point_style {
                        PointStyle::Circle { radius } => {
                            primitives.push(ChartPrimitive::Circle {
                                cx: px,
                                cy: py,
                                radius: radius * radius_mult,
                                color,
                                stroke_color: Some(self.theme.background),
                                stroke_width: 2.0,
                            });
                        }
                        PointStyle::Square { size } => {
                            let half = size * radius_mult / 2.0;
                            primitives.push(ChartPrimitive::Rect {
                                x: px - half,
                                y: py - half,
                                width: size * radius_mult,
                                height: size * radius_mult,
                                color,
                                radius: 0.0,
                            });
                        }
                        PointStyle::Diamond { size } => {
                            let half = size * radius_mult / 2.0;
                            let diamond_points = vec![
                                (px, py - half),
                                (px + half, py),
                                (px, py + half),
                                (px - half, py),
                            ];
                            primitives.push(ChartPrimitive::Path {
                                points: diamond_points,
                                color,
                                width: 0.0,
                                closed: true,
                            });
                        }
                        PointStyle::Triangle { size } => {
                            let half = size * radius_mult / 2.0;
                            let height = size * radius_mult * 0.866;
                            let triangle_points = vec![
                                (px, py - height / 2.0),
                                (px + half, py + height / 2.0),
                                (px - half, py + height / 2.0),
                            ];
                            primitives.push(ChartPrimitive::Path {
                                points: triangle_points,
                                color,
                                width: 0.0,
                                closed: true,
                            });
                        }
                        PointStyle::None => {}
                    }
                }
            }
        }

        primitives
    }

    fn hit_test(&self, x: f32, y: f32, threshold: f32) -> Option<(usize, usize)> {
        if !self.area.contains(x, y) {
            return None;
        }

        let bounds = self.bounds();
        let hit_radius = self.point_style.hit_radius().max(threshold);

        for (series_idx, series) in self.series.iter().enumerate() {
            for (point_idx, point) in series.data.iter().enumerate() {
                let x_norm = ((point.x - bounds.x_min) / bounds.width()) as f32;
                let y_norm = ((point.y - bounds.y_min) / bounds.height()) as f32;
                let (px, py) = self.area.map_point(x_norm, y_norm);

                let dx = x - px;
                let dy = y - py;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= hit_radius {
                    return Some((series_idx, point_idx));
                }
            }
        }

        None
    }
}

/// Area chart (line chart with filled area under the line)
#[derive(Debug, Clone)]
pub struct AreaChart {
    /// Inner line chart
    line_chart: LineChart,
    /// Fill opacity
    fill_opacity: f32,
    /// Gradient fill (top color, bottom color)
    gradient: Option<(Color, Color)>,
    /// Stack areas
    stacked: bool,
}

impl AreaChart {
    /// Create a new area chart
    pub fn new() -> Self {
        Self {
            line_chart: LineChart::new(),
            fill_opacity: 0.3,
            gradient: None,
            stacked: false,
        }
    }

    /// Add a data series
    pub fn series(mut self, name: impl Into<String>, data: impl IntoIterator<Item = impl Into<DataPoint>>) -> Self {
        self.line_chart = self.line_chart.series(name, data);
        self
    }

    /// Add a pre-built data series
    pub fn add_series(mut self, series: DataSeries) -> Self {
        self.line_chart = self.line_chart.add_series(series);
        self
    }

    /// Set the theme
    pub fn theme(mut self, theme: ChartTheme) -> Self {
        self.line_chart = self.line_chart.theme(theme);
        self
    }

    /// Set the X axis
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.line_chart = self.line_chart.x_axis(axis);
        self
    }

    /// Set the Y axis
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.line_chart = self.line_chart.y_axis(axis);
        self
    }

    /// Set the legend
    pub fn legend(mut self, legend: Legend) -> Self {
        self.line_chart = self.line_chart.legend(legend);
        self
    }

    /// Enable/disable tooltip
    pub fn tooltip(mut self, enabled: bool) -> Self {
        self.line_chart = self.line_chart.tooltip(enabled);
        self
    }

    /// Enable animation
    pub fn animate(mut self, enabled: bool) -> Self {
        self.line_chart = self.line_chart.animate(enabled);
        self
    }

    /// Use curved lines
    pub fn curved(mut self) -> Self {
        self.line_chart = self.line_chart.curved();
        self
    }

    /// Set fill opacity
    pub fn fill_opacity(mut self, opacity: f32) -> Self {
        self.fill_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set gradient fill
    pub fn gradient(mut self, top: Color, bottom: Color) -> Self {
        self.gradient = Some((top, bottom));
        self
    }

    /// Enable stacked areas
    pub fn stacked(mut self) -> Self {
        self.stacked = true;
        self
    }
}

impl Default for AreaChart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart for AreaChart {
    fn series(&self) -> &[DataSeries] {
        Chart::series(&self.line_chart)
    }

    fn bounds(&self) -> ChartBounds {
        Chart::bounds(&self.line_chart)
    }

    fn area(&self) -> ChartArea {
        Chart::area(&self.line_chart)
    }

    fn set_area(&mut self, area: ChartArea) {
        Chart::set_area(&mut self.line_chart, area);
    }

    fn theme(&self) -> &ChartTheme {
        Chart::theme(&self.line_chart)
    }

    fn x_axis(&self) -> Option<&Axis> {
        Chart::x_axis(&self.line_chart)
    }

    fn y_axis(&self) -> Option<&Axis> {
        Chart::y_axis(&self.line_chart)
    }

    fn legend(&self) -> Option<&Legend> {
        Chart::legend(&self.line_chart)
    }

    fn tooltip_config(&self) -> Option<&TooltipConfig> {
        Chart::tooltip_config(&self.line_chart)
    }

    fn animation_config(&self) -> Option<&AnimationConfig> {
        Chart::animation_config(&self.line_chart)
    }

    fn render(&self, interaction: &ChartInteraction) -> Vec<ChartPrimitive> {
        let mut primitives = Vec::new();
        let bounds = Chart::bounds(&self.line_chart);
        let area = Chart::area(&self.line_chart);
        let theme = Chart::theme(&self.line_chart);

        // Background and grid from line chart
        let line_primitives = Chart::render(&self.line_chart, interaction);

        // Add background and grid only
        for prim in line_primitives.iter() {
            match prim {
                ChartPrimitive::Rect { .. } | ChartPrimitive::Line { .. } => {
                    primitives.push(prim.clone());
                }
                _ => {}
            }
        }

        // Render filled areas first (so lines are on top)
        for (series_idx, series) in self.line_chart.series.iter().enumerate() {
            if !interaction.is_series_visible(series_idx) {
                continue;
            }

            let color = series.resolved_color(theme.series_color(series_idx));
            let fill_color = Color::new(color.r, color.g, color.b, self.fill_opacity);

            let line_points = self.line_chart.generate_line_points(series, &bounds);

            if line_points.len() >= 2 {
                // Create area points (line + bottom edge)
                let mut area_points = line_points.clone();

                // Add bottom edge points
                let base_y = area.bottom();
                if let Some(&(last_x, _)) = area_points.last() {
                    area_points.push((last_x, base_y));
                }
                if let Some(&(first_x, _)) = line_points.first() {
                    area_points.push((first_x, base_y));
                }

                if let Some(gradient) = &self.gradient {
                    primitives.push(ChartPrimitive::Area {
                        points: area_points,
                        color: fill_color,
                        gradient: Some((gradient.0, gradient.1)),
                    });
                } else {
                    primitives.push(ChartPrimitive::Area {
                        points: area_points,
                        color: fill_color,
                        gradient: None,
                    });
                }
            }
        }

        // Add lines and points from line chart
        for prim in line_primitives.iter() {
            match prim {
                ChartPrimitive::Path { .. } | ChartPrimitive::Circle { .. } => {
                    primitives.push(prim.clone());
                }
                _ => {}
            }
        }

        primitives
    }

    fn hit_test(&self, x: f32, y: f32, threshold: f32) -> Option<(usize, usize)> {
        Chart::hit_test(&self.line_chart, x, y, threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_chart_creation() {
        let chart = LineChart::new()
            .series("Sales", vec![(0.0, 100.0), (1.0, 150.0), (2.0, 200.0)])
            .series("Revenue", vec![(0.0, 80.0), (1.0, 120.0), (2.0, 160.0)]);

        assert_eq!(Chart::series(&chart).len(), 2);
        assert_eq!(Chart::series(&chart)[0].name, "Sales");
        assert_eq!(Chart::series(&chart)[1].name, "Revenue");
    }

    #[test]
    fn test_line_chart_bounds() {
        let chart = LineChart::new()
            .series("Test", vec![(0.0, 10.0), (5.0, 50.0), (10.0, 30.0)]);

        let bounds = chart.bounds();
        assert!(bounds.x_min <= 0.0);
        assert!(bounds.x_max >= 10.0);
        assert!(bounds.y_min <= 10.0);
        assert!(bounds.y_max >= 50.0);
    }

    #[test]
    fn test_line_chart_curved() {
        let chart = LineChart::new().curved();
        assert_eq!(chart.line_style, LineStyle::Curved);
    }

    #[test]
    fn test_line_chart_stepped() {
        let chart = LineChart::new().stepped();
        assert_eq!(chart.line_style, LineStyle::Stepped);
    }

    #[test]
    fn test_point_style_hit_radius() {
        assert!(PointStyle::None.hit_radius() > 0.0);
        assert_eq!(PointStyle::Circle { radius: 4.0 }.hit_radius(), 8.0);
    }

    #[test]
    fn test_area_chart_creation() {
        let chart = AreaChart::new()
            .series("Data", vec![(0.0, 10.0), (1.0, 20.0), (2.0, 15.0)])
            .fill_opacity(0.5)
            .curved();

        assert_eq!(chart.fill_opacity, 0.5);
        assert_eq!(Chart::series(&chart).len(), 1);
    }

    #[test]
    fn test_area_chart_gradient() {
        let chart = AreaChart::new()
            .gradient(Color::rgb(1.0, 0.0, 0.0), Color::rgb(0.0, 0.0, 1.0));

        assert!(chart.gradient.is_some());
    }

    #[test]
    fn test_line_chart_render() {
        let mut chart = LineChart::new()
            .series("Test", vec![(0.0, 10.0), (1.0, 20.0)]);

        chart.set_area(ChartArea::new(0.0, 0.0, 400.0, 300.0));

        let interaction = ChartInteraction::new();
        let primitives = chart.render(&interaction);

        assert!(!primitives.is_empty());
    }

    #[test]
    fn test_line_chart_hit_test() {
        let mut chart = LineChart::new()
            .series("Test", vec![(0.0, 0.0), (1.0, 1.0)]);

        chart.set_area(ChartArea::new(0.0, 0.0, 100.0, 100.0));

        // With expanded bounds (x: -1 to 2, y: 0 to 2) point (1.0, 1.0) maps to approximately:
        // x = (1.0 - (-1)) / 3 * 100 ≈ 66.7
        // y = (1.0 - (1.0 - 0.5)) * 100 = 50.0 (Y inverted)
        // Use large threshold to find the point
        let result = chart.hit_test(67.0, 50.0, 25.0);
        assert!(result.is_some(), "Expected hit on data point");
    }
}
