//! Chart themes.

use oxide_render::Color;
use serde::{Deserialize, Serialize};

/// Default color palette
pub const DEFAULT_COLORS: &[&str] = &[
    "#2196F3", // Blue
    "#4CAF50", // Green
    "#FF9800", // Orange
    "#E91E63", // Pink
    "#9C27B0", // Purple
    "#00BCD4", // Cyan
    "#FF5722", // Deep Orange
    "#3F51B5", // Indigo
];

/// Chart colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartColors {
    /// Series colors (palette)
    pub series: Vec<String>,
    /// Background color
    pub background: String,
    /// Text color
    pub text: String,
    /// Grid color
    pub grid: String,
    /// Axis color
    pub axis: String,
    /// Tooltip background
    pub tooltip_background: String,
    /// Tooltip text
    pub tooltip_text: String,
}

impl Default for ChartColors {
    fn default() -> Self {
        Self {
            series: DEFAULT_COLORS.iter().map(|s| s.to_string()).collect(),
            background: "#FFFFFF".to_string(),
            text: "#333333".to_string(),
            grid: "#E0E0E0".to_string(),
            axis: "#666666".to_string(),
            tooltip_background: "#333333".to_string(),
            tooltip_text: "#FFFFFF".to_string(),
        }
    }
}

impl ChartColors {
    /// Create dark theme colors
    pub fn dark() -> Self {
        Self {
            series: DEFAULT_COLORS.iter().map(|s| s.to_string()).collect(),
            background: "#1E1E1E".to_string(),
            text: "#E0E0E0".to_string(),
            grid: "#404040".to_string(),
            axis: "#808080".to_string(),
            tooltip_background: "#404040".to_string(),
            tooltip_text: "#FFFFFF".to_string(),
        }
    }

    /// Get color for series index
    pub fn color_for(&self, index: usize) -> &str {
        &self.series[index % self.series.len()]
    }
}

/// Chart theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartTheme {
    /// Theme name
    pub name: String,
    /// Colors
    pub colors: ChartColors,
    /// Font family
    pub font_family: String,
    /// Base font size
    pub font_size: f32,
    /// Title font size
    pub title_size: f32,
    /// Padding
    pub padding: f32,
    /// Background color (convenience accessor)
    #[serde(skip)]
    pub background: Color,
    /// Grid color (convenience accessor)
    #[serde(skip)]
    pub grid_color: Color,
}

impl Default for ChartTheme {
    fn default() -> Self {
        let colors = ChartColors::default();
        Self {
            name: "default".to_string(),
            background: Color::from_hex(&colors.background).unwrap_or(Color::WHITE),
            grid_color: Color::from_hex(&colors.grid).unwrap_or(Color::WHITE),
            colors,
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
            title_size: 16.0,
            padding: 20.0,
        }
    }
}

impl ChartTheme {
    /// Create new theme
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create light theme
    pub fn light() -> Self {
        Self::new("light")
    }

    /// Create dark theme
    pub fn dark() -> Self {
        let colors = ChartColors::dark();
        Self {
            name: "dark".to_string(),
            background: Color::from_hex(&colors.background).unwrap_or(Color::BLACK),
            grid_color: Color::from_hex(&colors.grid).unwrap_or(Color::WHITE),
            colors,
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
            title_size: 16.0,
            padding: 20.0,
        }
    }

    /// Get series color by index
    pub fn series_color(&self, index: usize) -> Color {
        let hex = self.colors.color_for(index);
        Color::from_hex(hex).unwrap_or(Color::WHITE)
    }

    /// Set colors
    pub fn colors(mut self, colors: ChartColors) -> Self {
        self.colors = colors;
        self
    }

    /// Set font family
    pub fn font_family(mut self, font: impl Into<String>) -> Self {
        self.font_family = font.into();
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}
