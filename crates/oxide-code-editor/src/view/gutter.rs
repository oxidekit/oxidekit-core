//! Gutter view (line numbers, fold markers, breakpoints).

use serde::{Deserialize, Serialize};

/// Gutter marker type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GutterMarker {
    /// Breakpoint
    Breakpoint,
    /// Error
    Error,
    /// Warning
    Warning,
    /// Info
    Info,
    /// Fold (collapsed)
    FoldCollapsed,
    /// Fold (expanded)
    FoldExpanded,
    /// Git added
    GitAdded,
    /// Git modified
    GitModified,
    /// Git deleted
    GitDeleted,
}

/// An item in the gutter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GutterItem {
    /// Line number
    pub line: usize,
    /// Marker type
    pub marker: GutterMarker,
    /// Tooltip text
    pub tooltip: Option<String>,
}

impl GutterItem {
    /// Create a new gutter item
    pub fn new(line: usize, marker: GutterMarker) -> Self {
        Self {
            line,
            marker,
            tooltip: None,
        }
    }

    /// Set tooltip
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

/// Gutter configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GutterConfig {
    /// Show line numbers
    pub line_numbers: bool,
    /// Show fold markers
    pub fold_markers: bool,
    /// Show breakpoints
    pub breakpoints: bool,
    /// Show git markers
    pub git_markers: bool,
    /// Width in characters
    pub width: usize,
}

impl GutterConfig {
    /// Create with defaults
    pub fn new() -> Self {
        Self {
            line_numbers: true,
            fold_markers: true,
            breakpoints: true,
            git_markers: true,
            width: 4,
        }
    }
}

/// Gutter view
#[derive(Debug, Clone, Default)]
pub struct GutterView {
    /// Configuration
    pub config: GutterConfig,
    /// Items to display
    pub items: Vec<GutterItem>,
}

impl GutterView {
    /// Create a new gutter view
    pub fn new() -> Self {
        Self {
            config: GutterConfig::new(),
            items: Vec::new(),
        }
    }

    /// Set configuration
    pub fn config(mut self, config: GutterConfig) -> Self {
        self.config = config;
        self
    }

    /// Add an item
    pub fn add_item(&mut self, item: GutterItem) {
        self.items.push(item);
    }

    /// Get items for a line
    pub fn items_for_line(&self, line: usize) -> Vec<&GutterItem> {
        self.items.iter().filter(|i| i.line == line).collect()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }
}
