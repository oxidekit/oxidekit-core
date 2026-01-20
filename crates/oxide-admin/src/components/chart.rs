//! Chart components (line, bar, time-series)

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Chart type
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ChartType {
    #[default]
    Line,
    Bar,
    Area,
    Donut,
    Sparkline,
}

/// Chart data point
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
    pub color: Option<String>,
}

impl DataPoint {
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// Chart series
#[derive(Debug, Clone)]
pub struct ChartSeries {
    pub name: String,
    pub data: Vec<f64>,
    pub color: String,
}

impl ChartSeries {
    pub fn new(name: impl Into<String>, data: Vec<f64>, color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data,
            color: color.into(),
        }
    }
}

/// Chart properties
#[derive(Debug, Clone)]
pub struct ChartProps {
    pub chart_type: ChartType,
    pub title: Option<String>,
    pub series: Vec<ChartSeries>,
    pub labels: Vec<String>,
    pub show_legend: bool,
    pub show_grid: bool,
    pub show_axes: bool,
    pub height: f32,
    pub animate: bool,
}

impl Default for ChartProps {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Line,
            title: None,
            series: Vec::new(),
            labels: Vec::new(),
            show_legend: true,
            show_grid: true,
            show_axes: true,
            height: 300.0,
            animate: true,
        }
    }
}

/// Chart component
pub struct Chart;

impl Chart {
    pub fn build(tree: &mut LayoutTree, props: ChartProps) -> NodeId {
        let chart_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .gap(16.0)
            .build();

        let mut children = Vec::new();

        // Title
        if let Some(_title) = &props.title {
            let title_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .justify_between()
                .build();

            let title = tree.new_node(title_style);
            children.push(title);
        }

        // Chart area
        let chart_area_style = StyleBuilder::new()
            .flex_row()
            .width_percent(1.0)
            .height(props.height)
            .build();

        // Y-axis
        if props.show_axes {
            let y_axis_style = StyleBuilder::new()
                .flex_column()
                .justify_between()
                .width(40.0)
                .height_percent(1.0)
                .build();

            let y_axis = tree.new_node(y_axis_style);
            children.push(y_axis);
        }

        // Plot area
        let plot_style = StyleBuilder::new()
            .flex_grow(1.0)
            .height_percent(1.0)
            .build();

        let plot_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(8.0);

        // Build chart-specific content
        let plot_content = match props.chart_type {
            ChartType::Line | ChartType::Area => Self::build_line_chart(tree, &props),
            ChartType::Bar => Self::build_bar_chart(tree, &props),
            ChartType::Donut => Self::build_donut_chart(tree, &props),
            ChartType::Sparkline => Self::build_sparkline(tree, &props),
        };

        let plot = tree.new_visual_node_with_children(plot_style, plot_visual, &plot_content);

        let chart_area = tree.new_node_with_children(chart_area_style, &[plot]);
        children.push(chart_area);

        // X-axis
        if props.show_axes {
            let x_axis_style = StyleBuilder::new()
                .flex_row()
                .justify_between()
                .width_percent(1.0)
                .height(24.0)
                .padding_xy(0.0, 40.0)
                .build();

            let x_axis = tree.new_node(x_axis_style);
            children.push(x_axis);
        }

        // Legend
        if props.show_legend && !props.series.is_empty() {
            let legend_style = StyleBuilder::new()
                .flex_row()
                .center()
                .gap(16.0)
                .build();

            let legend_items: Vec<NodeId> = props.series.iter()
                .map(|s| Self::build_legend_item(tree, &s.name, &s.color))
                .collect();

            let legend = tree.new_node_with_children(legend_style, &legend_items);
            children.push(legend);
        }

        tree.new_node_with_children(chart_style, &children)
    }

    fn build_line_chart(tree: &mut LayoutTree, props: &ChartProps) -> Vec<NodeId> {
        // Placeholder for line chart visualization
        // In a real implementation, this would calculate paths and render them
        let mut nodes = Vec::new();

        // Grid lines
        if props.show_grid {
            for i in 0..5 {
                let line_style = StyleBuilder::new()
                    .width_percent(1.0)
                    .height(1.0)
                    .build();

                let line_visual = NodeVisual::default()
                    .with_background(hex_to_rgba("#374151"));

                nodes.push(tree.new_visual_node(line_style, line_visual));
            }
        }

        nodes
    }

    fn build_bar_chart(tree: &mut LayoutTree, props: &ChartProps) -> Vec<NodeId> {
        let bar_container_style = StyleBuilder::new()
            .flex_row()
            .align_end()
            .height_percent(1.0)
            .width_percent(1.0)
            .gap(8.0)
            .padding(16.0)
            .build();

        let bars: Vec<NodeId> = props.series.first()
            .map(|s| {
                let max_val = s.data.iter().cloned().fold(0.0_f64, f64::max);
                s.data.iter()
                    .map(|&val| {
                        let height_pct = (val / max_val).min(1.0) as f32;
                        let bar_style = StyleBuilder::new()
                            .flex_grow(1.0)
                            .height_percent(height_pct)
                            .build();

                        let bar_visual = NodeVisual::default()
                            .with_background(hex_to_rgba(&s.color))
                            .with_radius(4.0);

                        tree.new_visual_node(bar_style, bar_visual)
                    })
                    .collect()
            })
            .unwrap_or_default();

        vec![tree.new_node_with_children(bar_container_style, &bars)]
    }

    fn build_donut_chart(tree: &mut LayoutTree, props: &ChartProps) -> Vec<NodeId> {
        // Placeholder - donut charts require arc rendering
        let center_style = StyleBuilder::new()
            .size(120.0, 120.0)
            .build();

        let center_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#3B82F6"))
            .with_radius(60.0);

        vec![tree.new_visual_node(center_style, center_visual)]
    }

    fn build_sparkline(tree: &mut LayoutTree, props: &ChartProps) -> Vec<NodeId> {
        // Simplified sparkline visualization
        Vec::new()
    }

    fn build_legend_item(tree: &mut LayoutTree, name: &str, color: &str) -> NodeId {
        let item_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(8.0)
            .build();

        // Color dot
        let dot_style = StyleBuilder::new().size(12.0, 12.0).build();
        let dot_visual = NodeVisual::default()
            .with_background(hex_to_rgba(color))
            .with_radius(6.0);
        let dot = tree.new_visual_node(dot_style, dot_visual);

        // Label
        let label_style = StyleBuilder::new().build();
        let label = tree.new_node(label_style);

        tree.new_node_with_children(item_style, &[dot, label])
    }
}

/// Stat with sparkline
pub struct SparklineStat;

impl SparklineStat {
    pub fn build(
        tree: &mut LayoutTree,
        label: &str,
        value: &str,
        data: Vec<f64>,
        trend_positive: bool,
    ) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_column()
            .padding(16.0)
            .gap(8.0)
            .build();

        let container_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(12.0);

        // Header
        let header_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .build();

        let header = tree.new_node(header_style);

        // Sparkline
        let sparkline_style = StyleBuilder::new()
            .width_percent(1.0)
            .height(40.0)
            .build();

        let sparkline = tree.new_node(sparkline_style);

        tree.new_visual_node_with_children(container_style, container_visual, &[header, sparkline])
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
