//! Chart axis configuration.

use serde::{Deserialize, Serialize};

/// Axis type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AxisType {
    /// Numeric values
    #[default]
    Linear,
    /// Logarithmic scale
    Logarithmic,
    /// Category/discrete values
    Category,
    /// Time/date values
    Time,
}

/// Axis position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AxisPosition {
    /// Left side (Y-axis)
    #[default]
    Left,
    /// Right side (Y-axis)
    Right,
    /// Bottom (X-axis)
    Bottom,
    /// Top (X-axis)
    Top,
}

/// Number format for axis labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AxisFormat {
    /// No formatting
    None,
    /// Number with decimals
    Number { decimals: usize },
    /// Currency
    Currency { symbol: String, decimals: usize },
    /// Percentage
    Percent { decimals: usize },
    /// Custom format string
    Custom(String),
}

impl Default for AxisFormat {
    fn default() -> Self {
        AxisFormat::None
    }
}

impl AxisFormat {
    /// Format a value
    pub fn format(&self, value: f64) -> String {
        match self {
            AxisFormat::None => format!("{}", value),
            AxisFormat::Number { decimals } => format!("{:.prec$}", value, prec = decimals),
            AxisFormat::Currency { symbol, decimals } => {
                format!("{}{:.prec$}", symbol, value, prec = decimals)
            }
            AxisFormat::Percent { decimals } => {
                format!("{:.prec$}%", value * 100.0, prec = decimals)
            }
            AxisFormat::Custom(fmt) => fmt.replace("{}", &value.to_string()),
        }
    }
}

/// Grid line configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLine {
    /// Show grid lines
    pub visible: bool,
    /// Line color
    pub color: String,
    /// Line width
    pub width: f32,
    /// Dash pattern
    pub dash: Option<Vec<f32>>,
}

impl Default for GridLine {
    fn default() -> Self {
        Self {
            visible: true,
            color: "#E0E0E0".to_string(),
            width: 1.0,
            dash: None,
        }
    }
}

/// Tick mark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    /// Show ticks
    pub visible: bool,
    /// Tick length
    pub length: f32,
    /// Tick color
    pub color: String,
    /// Number of ticks (auto if None)
    pub count: Option<usize>,
}

impl Default for Tick {
    fn default() -> Self {
        Self {
            visible: true,
            length: 5.0,
            color: "#666666".to_string(),
            count: None,
        }
    }
}

/// Axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axis {
    /// Axis type
    pub axis_type: AxisType,
    /// Position
    pub position: AxisPosition,
    /// Label
    pub label: Option<String>,
    /// Format
    pub format: AxisFormat,
    /// Min value (auto if None)
    pub min: Option<f64>,
    /// Max value (auto if None)
    pub max: Option<f64>,
    /// Grid lines
    pub grid: GridLine,
    /// Ticks
    pub ticks: Tick,
    /// Visible
    pub visible: bool,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            axis_type: AxisType::Linear,
            position: AxisPosition::Left,
            label: None,
            format: AxisFormat::None,
            min: None,
            max: None,
            grid: GridLine::default(),
            ticks: Tick::default(),
            visible: true,
        }
    }
}

impl Axis {
    /// Create new axis
    pub fn new() -> Self {
        Self::default()
    }

    /// Set label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set format
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = AxisFormat::Custom(format.into());
        self
    }

    /// Set position
    pub fn position(mut self, position: AxisPosition) -> Self {
        self.position = position;
        self
    }

    /// Set type
    pub fn axis_type(mut self, axis_type: AxisType) -> Self {
        self.axis_type = axis_type;
        self
    }

    /// Set min value
    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    /// Set max value
    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    /// Hide grid
    pub fn hide_grid(mut self) -> Self {
        self.grid.visible = false;
        self
    }

    /// Hide axis
    pub fn hide(mut self) -> Self {
        self.visible = false;
        self
    }
}
