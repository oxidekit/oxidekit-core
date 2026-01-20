//! Masonry (waterfall) layout for variable height items.

use serde::{Deserialize, Serialize};

/// Configuration for masonry layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasonryConfig {
    /// Number of columns
    pub columns: usize,
    /// Gap between items
    pub gap: f32,
    /// Minimum column width
    pub min_column_width: f32,
}

impl Default for MasonryConfig {
    fn default() -> Self {
        Self {
            columns: 3,
            gap: 8.0,
            min_column_width: 200.0,
        }
    }
}

/// Column state in masonry layout
#[derive(Debug, Clone, Default)]
pub struct MasonryColumn {
    /// Items in this column
    pub items: Vec<usize>,
    /// Current height
    pub height: f32,
}

/// Item in masonry layout
#[derive(Debug, Clone)]
pub struct MasonryItem<T> {
    /// Item data
    pub data: T,
    /// Measured height
    pub height: f32,
    /// Assigned column
    pub column: usize,
    /// Y position
    pub y: f32,
}

impl<T> MasonryItem<T> {
    /// Create a new masonry item
    pub fn new(data: T, height: f32) -> Self {
        Self {
            data,
            height,
            column: 0,
            y: 0.0,
        }
    }
}

/// State for masonry grid
#[derive(Debug, Clone, Default)]
pub struct MasonryState {
    /// Column states
    pub columns: Vec<MasonryColumn>,
    /// Total height
    pub total_height: f32,
    /// Scroll position
    pub scroll_top: f32,
}

/// Masonry grid component
#[derive(Debug, Clone)]
pub struct MasonryGrid {
    /// Configuration
    pub config: MasonryConfig,
    /// State
    pub state: MasonryState,
    /// Overscan
    pub overscan: usize,
}

impl Default for MasonryGrid {
    fn default() -> Self {
        Self {
            config: MasonryConfig::default(),
            state: MasonryState::default(),
            overscan: 3,
        }
    }
}

impl MasonryGrid {
    /// Create a new masonry grid
    pub fn new() -> Self {
        Self::default()
    }

    /// Set number of columns
    pub fn columns(mut self, columns: usize) -> Self {
        self.config.columns = columns;
        self
    }

    /// Set gap between items
    pub fn gap(mut self, gap: f32) -> Self {
        self.config.gap = gap;
        self
    }

    /// Initialize columns
    pub fn init_columns(&mut self) {
        self.state.columns = (0..self.config.columns)
            .map(|_| MasonryColumn::default())
            .collect();
    }

    /// Find shortest column
    pub fn shortest_column(&self) -> usize {
        self.state
            .columns
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.height.partial_cmp(&b.height).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Add item to layout
    pub fn add_item(&mut self, index: usize, height: f32) -> (usize, f32) {
        let col = self.shortest_column();
        let y = self.state.columns[col].height;
        self.state.columns[col].items.push(index);
        self.state.columns[col].height += height + self.config.gap;
        self.state.total_height = self
            .state
            .columns
            .iter()
            .map(|c| c.height)
            .fold(0.0f32, f32::max);
        (col, y)
    }

    /// Calculate column width for container
    pub fn column_width(&self, container_width: f32) -> f32 {
        let total_gap = self.config.gap * (self.config.columns - 1) as f32;
        (container_width - total_gap) / self.config.columns as f32
    }
}
