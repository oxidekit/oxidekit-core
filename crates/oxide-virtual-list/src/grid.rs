//! Virtual grid component for 2D layouts.

use serde::{Deserialize, Serialize};

/// Column width configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColumnWidth {
    /// Fixed width in pixels
    Fixed(f32),
    /// Fraction of available width
    Fraction(f32),
    /// Auto-size based on content
    Auto,
}

impl Default for ColumnWidth {
    fn default() -> Self {
        ColumnWidth::Auto
    }
}

/// Responsive column configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponsiveColumns {
    /// Columns for different breakpoints
    pub breakpoints: Vec<(f32, usize)>,
}

impl Default for ResponsiveColumns {
    fn default() -> Self {
        Self {
            breakpoints: vec![(0.0, 1), (600.0, 2), (900.0, 3), (1200.0, 4)],
        }
    }
}

/// Grid item data
#[derive(Debug, Clone)]
pub struct GridItem<T> {
    /// Item data
    pub data: T,
    /// Column span
    pub col_span: usize,
    /// Row span
    pub row_span: usize,
}

impl<T> GridItem<T> {
    /// Create a new grid item
    pub fn new(data: T) -> Self {
        Self {
            data,
            col_span: 1,
            row_span: 1,
        }
    }
}

/// Grid layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLayout {
    /// Number of columns
    pub columns: usize,
    /// Column width
    pub column_width: ColumnWidth,
    /// Row height
    pub row_height: f32,
    /// Gap between cells
    pub gap: f32,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            columns: 4,
            column_width: ColumnWidth::Auto,
            row_height: 100.0,
            gap: 8.0,
        }
    }
}

/// Virtual grid state
#[derive(Debug, Clone, Default)]
pub struct VirtualGridState {
    /// Scroll position
    pub scroll_top: f32,
    /// Container width
    pub container_width: f32,
    /// Container height
    pub container_height: f32,
}

/// Virtual grid component
#[derive(Debug, Clone)]
pub struct VirtualGrid {
    /// Grid layout
    pub layout: GridLayout,
    /// Responsive columns
    pub responsive: Option<ResponsiveColumns>,
    /// Total items
    pub total_items: usize,
    /// Overscan
    pub overscan: usize,
}

impl Default for VirtualGrid {
    fn default() -> Self {
        Self {
            layout: GridLayout::default(),
            responsive: None,
            total_items: 0,
            overscan: 2,
        }
    }
}

impl VirtualGrid {
    /// Create a new virtual grid
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of columns
    pub fn columns(mut self, columns: usize) -> Self {
        self.layout.columns = columns;
        self
    }

    /// Set the total number of items
    pub fn items(mut self, count: usize) -> Self {
        self.total_items = count;
        self
    }

    /// Get visible range of items
    pub fn visible_range(&self, scroll_top: f32, viewport_height: f32) -> std::ops::Range<usize> {
        let row_height = self.layout.row_height + self.layout.gap;
        let start_row = (scroll_top / row_height).floor() as usize;
        let visible_rows = (viewport_height / row_height).ceil() as usize + self.overscan * 2;
        let start_idx = start_row.saturating_sub(self.overscan) * self.layout.columns;
        let end_idx = ((start_row + visible_rows) * self.layout.columns).min(self.total_items);
        start_idx..end_idx
    }
}
