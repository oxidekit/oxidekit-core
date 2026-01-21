//! Code minimap for navigation.

use serde::{Deserialize, Serialize};

/// Minimap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapConfig {
    /// Show minimap
    pub visible: bool,
    /// Width in pixels
    pub width: usize,
    /// Scale factor
    pub scale: f32,
    /// Show slider
    pub show_slider: bool,
    /// Position on left or right
    pub position_right: bool,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            visible: true,
            width: 120,
            scale: 0.1,
            show_slider: true,
            position_right: true,
        }
    }
}

impl MinimapConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set width
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set scale
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

/// Minimap view
#[derive(Debug, Clone)]
pub struct Minimap {
    /// Configuration
    pub config: MinimapConfig,
    /// Total lines
    pub total_lines: usize,
    /// Visible start line
    pub visible_start: usize,
    /// Visible end line
    pub visible_end: usize,
}

impl Minimap {
    /// Create new minimap
    pub fn new() -> Self {
        Self {
            config: MinimapConfig::default(),
            total_lines: 0,
            visible_start: 0,
            visible_end: 0,
        }
    }

    /// Set configuration
    pub fn config(mut self, config: MinimapConfig) -> Self {
        self.config = config;
        self
    }

    /// Update visible range
    pub fn set_visible_range(&mut self, start: usize, end: usize) {
        self.visible_start = start;
        self.visible_end = end;
    }

    /// Update total lines
    pub fn set_total_lines(&mut self, total: usize) {
        self.total_lines = total;
    }

    /// Get slider position as percentage
    pub fn slider_position(&self) -> f32 {
        if self.total_lines == 0 {
            return 0.0;
        }
        self.visible_start as f32 / self.total_lines as f32
    }

    /// Get slider height as percentage
    pub fn slider_height(&self) -> f32 {
        if self.total_lines == 0 {
            return 1.0;
        }
        let visible = self.visible_end.saturating_sub(self.visible_start);
        (visible as f32 / self.total_lines as f32).min(1.0)
    }

    /// Convert click position to line number
    pub fn position_to_line(&self, y_percentage: f32) -> usize {
        ((y_percentage * self.total_lines as f32) as usize).min(self.total_lines.saturating_sub(1))
    }
}

impl Default for Minimap {
    fn default() -> Self {
        Self::new()
    }
}
