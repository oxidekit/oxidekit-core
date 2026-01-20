//! OxideKit Charts and Data Visualization Library
//!
//! A comprehensive charting library for OxideKit applications, providing
//! GPU-accelerated rendering of various chart types including:
//!
//! - Line charts (single/multi-series, area, spline)
//! - Bar charts (vertical, horizontal, grouped, stacked)
//! - Pie/Donut charts with labels and legends
//! - Scatter plots and bubble charts
//! - Gauges (radial, linear, progress ring)
//! - Sparklines (line, bar, win/loss)
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_charts::prelude::*;
//!
//! let chart = LineChart::new()
//!     .series("Sales", vec![100.0, 150.0, 200.0, 180.0])
//!     .series("Revenue", vec![80.0, 120.0, 160.0, 140.0])
//!     .x_axis(Axis::new().label("Month"))
//!     .y_axis(Axis::new().label("Amount").format("$,.0f"))
//!     .legend(Legend::bottom())
//!     .tooltip(true)
//!     .animate(true);
//! ```

pub mod animation;
pub mod axis;
pub mod chart;
pub mod legend;
pub mod series;
pub mod theme;
pub mod tooltip;

pub mod charts;

pub use animation::{AnimationConfig, ChartAnimation, ChartAnimationState};
pub use axis::{Axis, AxisFormat, AxisPosition, AxisType, GridLine, Tick};
pub use chart::{Chart, ChartArea, ChartBounds, ChartConfig, ChartEvent, ChartInteraction};
pub use charts::bar::{BarChart, BarOrientation, BarStyle, StackMode};
pub use charts::gauge::{GaugeStyle, LinearGauge, ProgressRing, RadialGauge};
pub use charts::line::{AreaChart, LineChart, LineStyle, PointStyle};
pub use charts::pie::{DonutChart, PieChart, SliceStyle};
pub use charts::scatter::{BubbleChart, ScatterPlot, TrendLine};
pub use charts::sparkline::{BarSparkline, LineSparkline, Sparkline, WinLossSparkline};
pub use legend::{Legend, LegendItem, LegendPosition};
pub use series::{DataPoint, DataSeries, SeriesStyle};
pub use theme::{ChartColors, ChartTheme};
pub use tooltip::{Crosshair, Tooltip, TooltipConfig, TooltipPosition};

/// Convenient re-exports for common usage
pub mod prelude {
    pub use crate::animation::{AnimationConfig, ChartAnimation, ChartAnimationState};
    pub use crate::axis::{Axis, AxisFormat, AxisPosition, AxisType, GridLine, Tick};
    pub use crate::chart::{Chart, ChartArea, ChartBounds, ChartConfig, ChartEvent, ChartInteraction};
    pub use crate::charts::bar::{BarChart, BarOrientation, BarStyle, StackMode};
    pub use crate::charts::gauge::{GaugeStyle, LinearGauge, ProgressRing, RadialGauge};
    pub use crate::charts::line::{AreaChart, LineChart, LineStyle, PointStyle};
    pub use crate::charts::pie::{DonutChart, PieChart, SliceStyle};
    pub use crate::charts::scatter::{BubbleChart, ScatterPlot, TrendLine};
    pub use crate::charts::sparkline::{BarSparkline, LineSparkline, Sparkline, WinLossSparkline};
    pub use crate::legend::{Legend, LegendItem, LegendPosition};
    pub use crate::series::{DataPoint, DataSeries, SeriesStyle};
    pub use crate::theme::{ChartColors, ChartTheme};
    pub use crate::tooltip::{Crosshair, Tooltip, TooltipConfig, TooltipPosition};
}
