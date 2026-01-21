//! Line number display.

use serde::{Deserialize, Serialize};

/// Line number display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineNumberConfig {
    /// Show line numbers
    pub visible: bool,
    /// Relative line numbers
    pub relative: bool,
    /// Minimum width (in digits)
    pub min_width: usize,
    /// Padding
    pub padding: usize,
}

impl Default for LineNumberConfig {
    fn default() -> Self {
        Self {
            visible: true,
            relative: false,
            min_width: 3,
            padding: 1,
        }
    }
}

impl LineNumberConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set relative mode
    pub fn relative(mut self, relative: bool) -> Self {
        self.relative = relative;
        self
    }
}

/// Line number view
#[derive(Debug, Clone)]
pub struct LineNumberView {
    /// Configuration
    pub config: LineNumberConfig,
    /// Total line count
    pub total_lines: usize,
    /// Current line (for relative mode)
    pub current_line: usize,
}

impl LineNumberView {
    /// Create new view
    pub fn new(total_lines: usize) -> Self {
        Self {
            config: LineNumberConfig::default(),
            total_lines,
            current_line: 0,
        }
    }

    /// Set configuration
    pub fn config(mut self, config: LineNumberConfig) -> Self {
        self.config = config;
        self
    }

    /// Set current line
    pub fn set_current_line(&mut self, line: usize) {
        self.current_line = line;
    }

    /// Get display text for a line
    pub fn display_text(&self, line: usize) -> String {
        if !self.config.visible {
            return String::new();
        }

        let number = if self.config.relative && line != self.current_line {
            let diff = (line as isize - self.current_line as isize).abs() as usize;
            diff
        } else {
            line + 1 // 1-indexed display
        };

        format!("{:>width$}", number, width = self.config.min_width)
    }

    /// Calculate width needed
    pub fn width(&self) -> usize {
        if !self.config.visible {
            return 0;
        }
        let digits = (self.total_lines as f64).log10().ceil() as usize;
        digits.max(self.config.min_width) + self.config.padding * 2
    }
}

impl Default for LineNumberView {
    fn default() -> Self {
        Self::new(0)
    }
}
