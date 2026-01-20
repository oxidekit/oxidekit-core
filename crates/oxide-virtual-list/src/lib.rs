//! OxideKit Virtual List
//!
//! High-performance virtualized list, grid, and masonry components
//! designed for rendering 1M+ items efficiently with constant memory usage.
//!
//! # Features
//!
//! - **VirtualList**: Vertical list with fixed or variable height items
//! - **VirtualGrid**: 2D grid with fixed or variable column widths
//! - **MasonryGrid**: Pinterest-style waterfall layout
//! - **Infinite scroll**: Load more items on demand
//! - **Selection**: Single/multi selection with keyboard support
//! - **Scroll features**: Scroll to index, position restoration, sticky headers
//!
//! # Example
//!
//! ```
//! use oxide_virtual_list::prelude::*;
//!
//! // Create a simple virtual list with fixed height items
//! let list = VirtualList::new()
//!     .items(100_000)
//!     .item_height(ItemHeight::Fixed(48.0))
//!     .overscan(5);
//!
//! // Calculate visible range for viewport
//! let range = list.visible_range(0.0, 600.0);
//! ```

mod grid;
mod infinite;
mod list;
mod masonry;
mod measure;
mod scroll;
mod selection;

pub use grid::{
    ColumnWidth, GridItem, GridLayout, ResponsiveColumns, VirtualGrid, VirtualGridState,
};
pub use infinite::{
    InfiniteScroll, InfiniteScrollConfig, InfiniteScrollState, LoadMoreTrigger, LoadingState,
};
pub use list::{
    ItemHeight, ListItem, SectionConfig, SectionHeader, Separator, SeparatorStyle, VirtualList,
    VirtualListConfig, VirtualListState,
};
pub use masonry::{MasonryColumn, MasonryConfig, MasonryGrid, MasonryItem, MasonryState};
pub use measure::{ItemMeasureCache, ItemMeasurement, MeasureContext, MeasureStrategy};
pub use scroll::{
    ScrollBehavior, ScrollConfig, ScrollController, ScrollDirection, ScrollEvent, ScrollPosition,
    ScrollState, StickyConfig, StickyElement,
};
pub use selection::{
    KeyboardAction, MultiSelectMode, SelectionChange, SelectionConfig, SelectionController,
    SelectionRange, SelectionState,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        // Grid
        ColumnWidth,
        GridItem,
        GridLayout,
        ResponsiveColumns,
        VirtualGrid,
        VirtualGridState,
        // Infinite scroll
        InfiniteScroll,
        InfiniteScrollConfig,
        InfiniteScrollState,
        LoadMoreTrigger,
        LoadingState,
        // List
        ItemHeight,
        ListItem,
        SectionConfig,
        SectionHeader,
        Separator,
        SeparatorStyle,
        VirtualList,
        VirtualListConfig,
        VirtualListState,
        // Masonry
        MasonryColumn,
        MasonryConfig,
        MasonryGrid,
        MasonryItem,
        MasonryState,
        // Measure
        ItemMeasureCache,
        ItemMeasurement,
        MeasureContext,
        MeasureStrategy,
        // Scroll
        ScrollBehavior,
        ScrollConfig,
        ScrollController,
        ScrollDirection,
        ScrollEvent,
        ScrollPosition,
        ScrollState,
        StickyConfig,
        StickyElement,
        // Selection
        KeyboardAction,
        MultiSelectMode,
        SelectionChange,
        SelectionConfig,
        SelectionController,
        SelectionRange,
        SelectionState,
    };
}

/// Error types for virtual list operations
#[derive(Debug, thiserror::Error)]
pub enum VirtualListError {
    #[error("Invalid index: {index} (total items: {total})")]
    InvalidIndex { index: usize, total: usize },

    #[error("Invalid scroll position: {position} (content height: {content_height})")]
    InvalidScrollPosition { position: f32, content_height: f32 },

    #[error("Item measurement failed for index {index}: {reason}")]
    MeasurementFailed { index: usize, reason: String },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Column count must be at least 1")]
    InvalidColumnCount,

    #[error("Viewport dimensions must be positive")]
    InvalidViewport,
}

/// Result type for virtual list operations
pub type Result<T> = std::result::Result<T, VirtualListError>;

/// A range of visible items in the virtualized view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleRange {
    /// First visible item index (inclusive)
    pub start: usize,
    /// Last visible item index (exclusive)
    pub end: usize,
}

impl VisibleRange {
    /// Create a new visible range
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create an empty range
    pub fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Check if the range is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Get the number of items in the range
    pub fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.end - self.start
        }
    }

    /// Check if an index is within this range
    pub fn contains(&self, index: usize) -> bool {
        index >= self.start && index < self.end
    }

    /// Iterate over indices in this range
    pub fn iter(&self) -> impl Iterator<Item = usize> {
        self.start..self.end
    }

    /// Expand the range by overscan amount (render extra items)
    pub fn with_overscan(self, overscan: usize, total_items: usize) -> Self {
        let start = self.start.saturating_sub(overscan);
        let end = (self.end + overscan).min(total_items);
        Self { start, end }
    }
}

/// Common size struct
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::zero()
    }
}

/// Common point/position struct
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Rectangle representing item bounds
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_position_size(pos: Point, size: Size) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Check if this rect intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Check if a point is inside this rect
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Get the bottom edge Y position
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Get the right edge X position
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_range_creation() {
        let range = VisibleRange::new(10, 20);
        assert_eq!(range.start, 10);
        assert_eq!(range.end, 20);
        assert_eq!(range.len(), 10);
        assert!(!range.is_empty());
    }

    #[test]
    fn test_visible_range_empty() {
        let range = VisibleRange::empty();
        assert!(range.is_empty());
        assert_eq!(range.len(), 0);
    }

    #[test]
    fn test_visible_range_contains() {
        let range = VisibleRange::new(10, 20);
        assert!(!range.contains(9));
        assert!(range.contains(10));
        assert!(range.contains(15));
        assert!(range.contains(19));
        assert!(!range.contains(20));
    }

    #[test]
    fn test_visible_range_overscan() {
        let range = VisibleRange::new(10, 20);
        let expanded = range.with_overscan(3, 100);
        assert_eq!(expanded.start, 7);
        assert_eq!(expanded.end, 23);

        // Test edge clamping
        let edge_range = VisibleRange::new(0, 5);
        let clamped = edge_range.with_overscan(3, 10);
        assert_eq!(clamped.start, 0);
        assert_eq!(clamped.end, 8);
    }

    #[test]
    fn test_visible_range_iter() {
        let range = VisibleRange::new(5, 8);
        let indices: Vec<_> = range.iter().collect();
        assert_eq!(indices, vec![5, 6, 7]);
    }

    #[test]
    fn test_size() {
        let size = Size::new(100.0, 50.0);
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 50.0);
        assert!(size.is_valid());

        let zero = Size::zero();
        assert!(!zero.is_valid());
    }

    #[test]
    fn test_point() {
        let point = Point::new(10.0, 20.0);
        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);
    }

    #[test]
    fn test_rect() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.right(), 110.0);
        assert_eq!(rect.bottom(), 70.0);
        assert!(rect.contains_point(50.0, 40.0));
        assert!(!rect.contains_point(5.0, 40.0));
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rect2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let rect3 = Rect::new(200.0, 200.0, 50.0, 50.0);

        assert!(rect1.intersects(&rect2));
        assert!(rect2.intersects(&rect1));
        assert!(!rect1.intersects(&rect3));
    }
}
