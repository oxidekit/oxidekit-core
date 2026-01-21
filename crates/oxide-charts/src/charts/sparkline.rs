//! Sparkline charts.

use serde::{Deserialize, Serialize};

/// Base sparkline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sparkline {
    /// Data points
    pub data: Vec<f64>,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
    /// Color
    pub color: String,
}

impl Sparkline {
    /// Create new sparkline
    pub fn new(data: Vec<f64>) -> Self {
        Self {
            data,
            width: 100.0,
            height: 30.0,
            color: "#2196F3".to_string(),
        }
    }

    /// Set size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Get min value
    pub fn min(&self) -> f64 {
        self.data.iter().copied().fold(f64::INFINITY, f64::min)
    }

    /// Get max value
    pub fn max(&self) -> f64 {
        self.data.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    /// Get latest value
    pub fn latest(&self) -> Option<f64> {
        self.data.last().copied()
    }
}

/// Line sparkline
#[derive(Debug, Clone)]
pub struct LineSparkline {
    /// Base sparkline
    pub inner: Sparkline,
    /// Line width
    pub line_width: f32,
    /// Show area fill
    pub fill: bool,
    /// Fill opacity
    pub fill_opacity: f32,
    /// Show dots
    pub show_dots: bool,
    /// Highlight last point
    pub highlight_last: bool,
}

impl LineSparkline {
    /// Create new line sparkline
    pub fn new(data: Vec<f64>) -> Self {
        Self {
            inner: Sparkline::new(data),
            line_width: 2.0,
            fill: false,
            fill_opacity: 0.2,
            show_dots: false,
            highlight_last: true,
        }
    }

    /// Enable fill
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Show dots
    pub fn dots(mut self, show: bool) -> Self {
        self.show_dots = show;
        self
    }

    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.inner.color = color.into();
        self
    }

    /// Set size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.inner = self.inner.size(width, height);
        self
    }
}

/// Bar sparkline
#[derive(Debug, Clone)]
pub struct BarSparkline {
    /// Base sparkline
    pub inner: Sparkline,
    /// Bar spacing
    pub spacing: f32,
    /// Negative color
    pub negative_color: String,
}

impl BarSparkline {
    /// Create new bar sparkline
    pub fn new(data: Vec<f64>) -> Self {
        Self {
            inner: Sparkline::new(data),
            spacing: 1.0,
            negative_color: "#F44336".to_string(),
        }
    }

    /// Set spacing
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.inner.color = color.into();
        self
    }

    /// Set negative color
    pub fn negative_color(mut self, color: impl Into<String>) -> Self {
        self.negative_color = color.into();
        self
    }

    /// Set size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.inner = self.inner.size(width, height);
        self
    }
}

/// Win/loss sparkline (for binary outcomes)
#[derive(Debug, Clone)]
pub struct WinLossSparkline {
    /// Data points (positive = win, negative = loss, zero = neutral)
    pub data: Vec<f64>,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
    /// Win color
    pub win_color: String,
    /// Loss color
    pub loss_color: String,
    /// Neutral color
    pub neutral_color: String,
    /// Bar spacing
    pub spacing: f32,
}

impl WinLossSparkline {
    /// Create new win/loss sparkline
    pub fn new(data: Vec<f64>) -> Self {
        Self {
            data,
            width: 100.0,
            height: 30.0,
            win_color: "#4CAF50".to_string(),
            loss_color: "#F44336".to_string(),
            neutral_color: "#9E9E9E".to_string(),
            spacing: 1.0,
        }
    }

    /// Set size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set win color
    pub fn win_color(mut self, color: impl Into<String>) -> Self {
        self.win_color = color.into();
        self
    }

    /// Set loss color
    pub fn loss_color(mut self, color: impl Into<String>) -> Self {
        self.loss_color = color.into();
        self
    }

    /// Get win count
    pub fn wins(&self) -> usize {
        self.data.iter().filter(|&&v| v > 0.0).count()
    }

    /// Get loss count
    pub fn losses(&self) -> usize {
        self.data.iter().filter(|&&v| v < 0.0).count()
    }

    /// Get win rate
    pub fn win_rate(&self) -> f64 {
        let total = self.data.len();
        if total == 0 {
            0.0
        } else {
            self.wins() as f64 / total as f64
        }
    }
}
