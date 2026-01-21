//! Skeleton table loader.

use super::SkeletonConfig;
use serde::{Deserialize, Serialize};

/// Skeleton table for tabular data placeholders
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonTable {
    /// Number of rows
    pub rows: usize,
    /// Number of columns
    pub columns: usize,
    /// Show header row
    pub show_header: bool,
    /// Row height
    pub row_height: f32,
    /// Column widths (None = equal distribution)
    pub column_widths: Option<Vec<f32>>,
    /// Configuration
    #[serde(skip)]
    config: SkeletonConfig,
}

impl Default for SkeletonTable {
    fn default() -> Self {
        Self::new(5, 4)
    }
}

impl SkeletonTable {
    /// Create a new skeleton table
    pub fn new(rows: usize, columns: usize) -> Self {
        Self {
            rows: rows.max(1),
            columns: columns.max(1),
            show_header: true,
            row_height: 48.0,
            column_widths: None,
            config: SkeletonConfig::default(),
        }
    }

    /// Set number of rows
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    /// Set number of columns
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    /// Show or hide header row
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Set row height
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(20.0);
        self
    }

    /// Set custom column widths
    pub fn column_widths(mut self, widths: Vec<f32>) -> Self {
        self.column_widths = Some(widths);
        self
    }

    /// Set animation
    pub fn animation(mut self, animation: super::SkeletonAnimation) -> Self {
        self.config.animation = animation;
        self
    }

    /// Get configuration
    pub fn config(&self) -> &SkeletonConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut SkeletonConfig {
        &mut self.config
    }

    /// Update animation state
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.config.update(delta_time)
    }

    /// Calculate total height
    pub fn total_height(&self) -> f32 {
        let row_count = if self.show_header {
            self.rows + 1
        } else {
            self.rows
        };
        row_count as f32 * self.row_height
    }

    /// Get row count including header
    pub fn total_rows(&self) -> usize {
        if self.show_header {
            self.rows + 1
        } else {
            self.rows
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_table_default() {
        let table = SkeletonTable::default();
        assert_eq!(table.rows, 5);
        assert_eq!(table.columns, 4);
        assert!(table.show_header);
    }

    #[test]
    fn test_skeleton_table_builder() {
        let table = SkeletonTable::new(3, 2)
            .show_header(false)
            .row_height(32.0);

        assert_eq!(table.rows, 3);
        assert_eq!(table.columns, 2);
        assert!(!table.show_header);
        assert!((table.row_height - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_table_height() {
        let table = SkeletonTable::new(5, 3).row_height(40.0);
        // 5 data rows + 1 header = 6 rows
        assert!((table.total_height() - 240.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skeleton_table_no_header() {
        let table = SkeletonTable::new(5, 3)
            .show_header(false)
            .row_height(40.0);
        // 5 data rows only
        assert!((table.total_height() - 200.0).abs() < f32::EPSILON);
    }
}
