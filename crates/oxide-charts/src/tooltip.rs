//! Chart tooltips.

use serde::{Deserialize, Serialize};

/// Tooltip position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TooltipPosition {
    /// Follow mouse
    #[default]
    Mouse,
    /// Fixed at top
    Top,
    /// Fixed at bottom
    Bottom,
    /// Fixed at left
    Left,
    /// Fixed at right
    Right,
}

/// Tooltip configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooltipConfig {
    /// Enabled
    pub enabled: bool,
    /// Position
    pub position: TooltipPosition,
    /// Background color
    pub background: String,
    /// Text color
    pub text_color: String,
    /// Border radius
    pub border_radius: f32,
    /// Padding
    pub padding: f32,
    /// Show series color
    pub show_color: bool,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            position: TooltipPosition::Mouse,
            background: "#333333".to_string(),
            text_color: "#FFFFFF".to_string(),
            border_radius: 4.0,
            padding: 8.0,
            show_color: true,
        }
    }
}

/// Tooltip state
#[derive(Debug, Clone, Default)]
pub struct Tooltip {
    /// Configuration
    pub config: TooltipConfig,
    /// Visible
    pub visible: bool,
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Content lines
    pub content: Vec<TooltipLine>,
}

/// A line in the tooltip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooltipLine {
    /// Label
    pub label: String,
    /// Value
    pub value: String,
    /// Color (for series indicator)
    pub color: Option<String>,
}

impl TooltipLine {
    /// Create new line
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: None,
        }
    }

    /// With color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl Tooltip {
    /// Create new tooltip
    pub fn new() -> Self {
        Self::default()
    }

    /// Set configuration
    pub fn config(mut self, config: TooltipConfig) -> Self {
        self.config = config;
        self
    }

    /// Show at position
    pub fn show(&mut self, x: f32, y: f32, content: Vec<TooltipLine>) {
        self.visible = true;
        self.x = x;
        self.y = y;
        self.content = content;
    }

    /// Hide tooltip
    pub fn hide(&mut self) {
        self.visible = false;
        self.content.clear();
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Crosshair configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crosshair {
    /// Enabled
    pub enabled: bool,
    /// Show X line
    pub x_line: bool,
    /// Show Y line
    pub y_line: bool,
    /// Line color
    pub color: String,
    /// Line width
    pub width: f32,
    /// Dash pattern
    pub dash: Option<Vec<f32>>,
}

impl Default for Crosshair {
    fn default() -> Self {
        Self {
            enabled: false,
            x_line: true,
            y_line: true,
            color: "#999999".to_string(),
            width: 1.0,
            dash: Some(vec![4.0, 4.0]),
        }
    }
}

impl Crosshair {
    /// Create enabled crosshair
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// X-axis only
    pub fn x_only(mut self) -> Self {
        self.x_line = true;
        self.y_line = false;
        self
    }

    /// Y-axis only
    pub fn y_only(mut self) -> Self {
        self.x_line = false;
        self.y_line = true;
        self
    }
}
