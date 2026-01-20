//! Virtual List Implementation
//!
//! Provides efficient rendering of large lists by only rendering visible items.

use crate::{Rect, Result, Size, VisibleRange, VirtualListError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Item height configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ItemHeight {
    /// All items have the same fixed height
    Fixed(f32),
    /// Items have variable heights that need measurement
    Variable {
        /// Estimated height for unmeasured items
        estimated: f32,
    },
}

impl Default for ItemHeight {
    fn default() -> Self {
        Self::Fixed(48.0)
    }
}

/// Separator style between items
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SeparatorStyle {
    /// No separator
    None,
    /// Full width line
    Full {
        /// Line thickness in pixels
        thickness: f32,
        /// RGBA color
        color: [f32; 4],
    },
    /// Inset line with padding
    Inset {
        /// Line thickness in pixels
        thickness: f32,
        /// RGBA color
        color: [f32; 4],
        /// Left inset in pixels
        left_inset: f32,
        /// Right inset in pixels
        right_inset: f32,
    },
}

impl Default for SeparatorStyle {
    fn default() -> Self {
        Self::None
    }
}

/// Separator configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Separator {
    pub style: SeparatorStyle,
}

impl Separator {
    pub fn none() -> Self {
        Self {
            style: SeparatorStyle::None,
        }
    }

    pub fn full(thickness: f32, color: [f32; 4]) -> Self {
        Self {
            style: SeparatorStyle::Full { thickness, color },
        }
    }

    pub fn inset(thickness: f32, color: [f32; 4], left: f32, right: f32) -> Self {
        Self {
            style: SeparatorStyle::Inset {
                thickness,
                color,
                left_inset: left,
                right_inset: right,
            },
        }
    }

    /// Get the height added by this separator
    pub fn height(&self) -> f32 {
        match self.style {
            SeparatorStyle::None => 0.0,
            SeparatorStyle::Full { thickness, .. } => thickness,
            SeparatorStyle::Inset { thickness, .. } => thickness,
        }
    }
}

/// Section header for grouped lists
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionHeader {
    /// Section title
    pub title: String,
    /// Section ID
    pub id: String,
    /// Header height
    pub height: f32,
    /// Whether header is sticky (stays at top while scrolling)
    pub sticky: bool,
    /// First item index in this section
    pub start_index: usize,
    /// Number of items in this section
    pub item_count: usize,
}

impl SectionHeader {
    pub fn new(id: impl Into<String>, title: impl Into<String>, height: f32) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            height,
            sticky: true,
            start_index: 0,
            item_count: 0,
        }
    }

    pub fn with_sticky(mut self, sticky: bool) -> Self {
        self.sticky = sticky;
        self
    }

    pub fn with_items(mut self, start_index: usize, item_count: usize) -> Self {
        self.start_index = start_index;
        self.item_count = item_count;
        self
    }
}

/// Section configuration for grouped lists
#[derive(Debug, Clone, Default)]
pub struct SectionConfig {
    /// Section headers
    pub sections: Vec<SectionHeader>,
}

impl SectionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_section(mut self, header: SectionHeader) -> Self {
        self.sections.push(header);
        self
    }

    /// Find which section an item belongs to
    pub fn section_for_item(&self, item_index: usize) -> Option<&SectionHeader> {
        self.sections.iter().find(|s| {
            item_index >= s.start_index && item_index < s.start_index + s.item_count
        })
    }

    /// Get total header height before an item
    pub fn header_height_before(&self, item_index: usize) -> f32 {
        self.sections
            .iter()
            .filter(|s| s.start_index <= item_index)
            .map(|s| s.height)
            .sum()
    }
}

/// A list item with computed layout
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    /// Item index in the data source
    pub index: usize,
    /// Computed bounds
    pub bounds: Rect,
    /// Whether this item is currently visible
    pub visible: bool,
}

impl ListItem {
    pub fn new(index: usize, bounds: Rect) -> Self {
        Self {
            index,
            bounds,
            visible: true,
        }
    }
}

/// Configuration for the virtual list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualListConfig {
    /// Item height mode
    pub item_height: ItemHeight,
    /// Number of extra items to render above/below viewport
    pub overscan: usize,
    /// Separator between items
    pub separator: Separator,
    /// Whether to enable smooth scrolling
    pub smooth_scroll: bool,
    /// Viewport padding
    pub padding: f32,
    /// Whether pull-to-refresh is enabled
    pub pull_to_refresh: bool,
}

impl Default for VirtualListConfig {
    fn default() -> Self {
        Self {
            item_height: ItemHeight::default(),
            overscan: 3,
            separator: Separator::default(),
            smooth_scroll: true,
            padding: 0.0,
            pull_to_refresh: false,
        }
    }
}

/// State of the virtual list
#[derive(Debug, Clone)]
pub struct VirtualListState {
    /// Total number of items
    pub total_items: usize,
    /// Computed total content height
    pub content_height: f32,
    /// Current scroll offset (Y)
    pub scroll_offset: f32,
    /// Viewport height
    pub viewport_height: f32,
    /// Currently visible range (with overscan)
    pub visible_range: VisibleRange,
    /// Measured item heights (for variable height mode)
    pub measured_heights: HashMap<usize, f32>,
    /// Cached item positions (Y offsets)
    pub item_positions: Vec<f32>,
    /// Whether the list is currently scrolling
    pub is_scrolling: bool,
    /// Last scroll direction (1 = down, -1 = up, 0 = stopped)
    pub scroll_direction: i8,
    /// Pull-to-refresh progress (0.0 to 1.0)
    pub pull_progress: f32,
    /// Whether refresh is currently in progress
    pub is_refreshing: bool,
}

impl Default for VirtualListState {
    fn default() -> Self {
        Self {
            total_items: 0,
            content_height: 0.0,
            scroll_offset: 0.0,
            viewport_height: 0.0,
            visible_range: VisibleRange::empty(),
            measured_heights: HashMap::new(),
            item_positions: Vec::new(),
            is_scrolling: false,
            scroll_direction: 0,
            pull_progress: 0.0,
            is_refreshing: false,
        }
    }
}

/// Virtual list for efficiently rendering large datasets
#[derive(Debug, Clone)]
pub struct VirtualList {
    config: VirtualListConfig,
    state: VirtualListState,
    sections: Option<SectionConfig>,
}

impl Default for VirtualList {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualList {
    /// Create a new virtual list with default configuration
    pub fn new() -> Self {
        Self {
            config: VirtualListConfig::default(),
            state: VirtualListState::default(),
            sections: None,
        }
    }

    /// Set the total number of items
    pub fn items(mut self, count: usize) -> Self {
        self.state.total_items = count;
        self.recalculate_layout();
        self
    }

    /// Set item height mode
    pub fn item_height(mut self, height: ItemHeight) -> Self {
        self.config.item_height = height;
        self.recalculate_layout();
        self
    }

    /// Set fixed item height (convenience method)
    pub fn fixed_height(self, height: f32) -> Self {
        self.item_height(ItemHeight::Fixed(height))
    }

    /// Set variable item height with estimated size
    pub fn variable_height(self, estimated: f32) -> Self {
        self.item_height(ItemHeight::Variable { estimated })
    }

    /// Set overscan (extra items rendered above/below viewport)
    pub fn overscan(mut self, count: usize) -> Self {
        self.config.overscan = count;
        self
    }

    /// Set separator style
    pub fn separator(mut self, separator: Separator) -> Self {
        self.config.separator = separator;
        self.recalculate_layout();
        self
    }

    /// Enable smooth scrolling
    pub fn smooth_scroll(mut self, enabled: bool) -> Self {
        self.config.smooth_scroll = enabled;
        self
    }

    /// Set padding around the list
    pub fn padding(mut self, padding: f32) -> Self {
        self.config.padding = padding;
        self.recalculate_layout();
        self
    }

    /// Enable pull-to-refresh
    pub fn pull_to_refresh(mut self, enabled: bool) -> Self {
        self.config.pull_to_refresh = enabled;
        self
    }

    /// Add section headers for grouped lists
    pub fn sections(mut self, sections: SectionConfig) -> Self {
        self.sections = Some(sections);
        self.recalculate_layout();
        self
    }

    /// Set the viewport height
    pub fn viewport_height(mut self, height: f32) -> Self {
        self.state.viewport_height = height;
        self.update_visible_range();
        self
    }

    /// Get the current configuration
    pub fn config(&self) -> &VirtualListConfig {
        &self.config
    }

    /// Get the current state
    pub fn state(&self) -> &VirtualListState {
        &self.state
    }

    /// Get mutable state reference
    pub fn state_mut(&mut self) -> &mut VirtualListState {
        &mut self.state
    }

    /// Update the total item count
    pub fn set_item_count(&mut self, count: usize) {
        self.state.total_items = count;
        self.recalculate_layout();
    }

    /// Get the total content height
    pub fn content_height(&self) -> f32 {
        self.state.content_height
    }

    /// Get the current visible range
    pub fn visible_range(&self) -> VisibleRange {
        self.state.visible_range
    }

    /// Calculate visible range for given scroll position and viewport
    pub fn visible_range_for(&self, scroll_offset: f32, viewport_height: f32) -> VisibleRange {
        if self.state.total_items == 0 || viewport_height <= 0.0 {
            return VisibleRange::empty();
        }

        let start = self.index_at_offset(scroll_offset);
        let end = self.index_at_offset(scroll_offset + viewport_height);
        let end = (end + 1).min(self.state.total_items);

        VisibleRange::new(start, end).with_overscan(self.config.overscan, self.state.total_items)
    }

    /// Set the scroll offset
    pub fn set_scroll_offset(&mut self, offset: f32) {
        let prev_offset = self.state.scroll_offset;
        self.state.scroll_offset = offset.max(0.0).min(self.max_scroll_offset());

        // Track scroll direction
        if self.state.scroll_offset > prev_offset {
            self.state.scroll_direction = 1;
        } else if self.state.scroll_offset < prev_offset {
            self.state.scroll_direction = -1;
        }

        self.update_visible_range();
    }

    /// Get the maximum scroll offset
    pub fn max_scroll_offset(&self) -> f32 {
        (self.state.content_height - self.state.viewport_height).max(0.0)
    }

    /// Scroll by a delta amount
    pub fn scroll_by(&mut self, delta: f32) {
        let new_offset = self.state.scroll_offset + delta;
        self.set_scroll_offset(new_offset);
    }

    /// Scroll to a specific index
    pub fn scroll_to_index(&mut self, index: usize) -> Result<()> {
        if index >= self.state.total_items {
            return Err(VirtualListError::InvalidIndex {
                index,
                total: self.state.total_items,
            });
        }

        let offset = self.offset_for_index(index);
        self.set_scroll_offset(offset);
        Ok(())
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.set_scroll_offset(0.0);
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.set_scroll_offset(self.max_scroll_offset());
    }

    /// Get the current scroll position (normalized 0-1)
    pub fn scroll_progress(&self) -> f32 {
        if self.max_scroll_offset() <= 0.0 {
            0.0
        } else {
            self.state.scroll_offset / self.max_scroll_offset()
        }
    }

    /// Update measured height for a variable-height item
    pub fn set_item_height(&mut self, index: usize, height: f32) {
        if matches!(self.config.item_height, ItemHeight::Variable { .. }) {
            self.state.measured_heights.insert(index, height);
            self.recalculate_layout();
        }
    }

    /// Get the height for an item
    pub fn get_item_height(&self, index: usize) -> f32 {
        match self.config.item_height {
            ItemHeight::Fixed(h) => h,
            ItemHeight::Variable { estimated } => {
                *self.state.measured_heights.get(&index).unwrap_or(&estimated)
            }
        }
    }

    /// Get the Y offset for an item
    pub fn offset_for_index(&self, index: usize) -> f32 {
        if index == 0 {
            return self.config.padding;
        }

        if !self.state.item_positions.is_empty() && index < self.state.item_positions.len() {
            return self.state.item_positions[index];
        }

        // Fallback calculation for fixed height
        match self.config.item_height {
            ItemHeight::Fixed(h) => {
                let item_total_height = h + self.config.separator.height();
                self.config.padding + (index as f32 * item_total_height)
            }
            ItemHeight::Variable { estimated } => {
                // Calculate on the fly for variable heights
                let mut offset = self.config.padding;
                for i in 0..index {
                    let height = self
                        .state
                        .measured_heights
                        .get(&i)
                        .copied()
                        .unwrap_or(estimated);
                    offset += height + self.config.separator.height();
                }
                offset
            }
        }
    }

    /// Find the item index at a given Y offset
    pub fn index_at_offset(&self, offset: f32) -> usize {
        if self.state.total_items == 0 {
            return 0;
        }

        let adjusted_offset = offset - self.config.padding;
        if adjusted_offset <= 0.0 {
            return 0;
        }

        match self.config.item_height {
            ItemHeight::Fixed(h) => {
                let item_total_height = h + self.config.separator.height();
                if item_total_height <= 0.0 {
                    return 0;
                }
                let index = (adjusted_offset / item_total_height).floor() as usize;
                index.min(self.state.total_items.saturating_sub(1))
            }
            ItemHeight::Variable { estimated } => {
                // Binary search through cached positions
                if !self.state.item_positions.is_empty() {
                    match self
                        .state
                        .item_positions
                        .binary_search_by(|pos| pos.partial_cmp(&offset).unwrap())
                    {
                        Ok(i) => i,
                        Err(i) => i.saturating_sub(1).min(self.state.total_items - 1),
                    }
                } else {
                    // Fallback for uncached
                    let item_total_height = estimated + self.config.separator.height();
                    let index = (adjusted_offset / item_total_height).floor() as usize;
                    index.min(self.state.total_items.saturating_sub(1))
                }
            }
        }
    }

    /// Get bounds for an item
    pub fn item_bounds(&self, index: usize, viewport_width: f32) -> Rect {
        let y = self.offset_for_index(index) - self.state.scroll_offset;
        let height = self.get_item_height(index);

        Rect::new(self.config.padding, y, viewport_width - 2.0 * self.config.padding, height)
    }

    /// Get visible items with their bounds
    pub fn visible_items(&self, viewport_width: f32) -> Vec<ListItem> {
        let range = self.state.visible_range;
        range
            .iter()
            .map(|index| {
                let bounds = self.item_bounds(index, viewport_width);
                ListItem::new(index, bounds)
            })
            .collect()
    }

    /// Check if an item is currently visible
    pub fn is_item_visible(&self, index: usize) -> bool {
        self.state.visible_range.contains(index)
    }

    /// Get sticky header that should be displayed (if any)
    pub fn current_sticky_header(&self) -> Option<&SectionHeader> {
        let sections = self.sections.as_ref()?;

        // Find the section that contains the first visible item
        let first_visible = self.state.visible_range.start;

        sections
            .sections
            .iter()
            .filter(|s| s.sticky)
            .filter(|s| first_visible >= s.start_index)
            .last()
    }

    /// Recalculate the layout (positions and content height)
    fn recalculate_layout(&mut self) {
        if self.state.total_items == 0 {
            self.state.content_height = self.config.padding * 2.0;
            self.state.item_positions.clear();
            return;
        }

        let separator_height = self.config.separator.height();

        match self.config.item_height {
            ItemHeight::Fixed(h) => {
                // Simple calculation for fixed heights
                let item_total_height = h + separator_height;
                self.state.content_height = self.config.padding * 2.0
                    + (self.state.total_items as f32 * item_total_height)
                    - separator_height; // No separator after last item

                // Cache positions for fast lookup
                self.state.item_positions.clear();
                self.state.item_positions.reserve(self.state.total_items);
                for i in 0..self.state.total_items {
                    self.state
                        .item_positions
                        .push(self.config.padding + (i as f32 * item_total_height));
                }
            }
            ItemHeight::Variable { estimated } => {
                // Calculate positions incrementally
                self.state.item_positions.clear();
                self.state.item_positions.reserve(self.state.total_items);

                let mut offset = self.config.padding;
                for i in 0..self.state.total_items {
                    self.state.item_positions.push(offset);
                    let height = self
                        .state
                        .measured_heights
                        .get(&i)
                        .copied()
                        .unwrap_or(estimated);
                    offset += height + separator_height;
                }

                // Remove last separator
                self.state.content_height = offset - separator_height + self.config.padding;
            }
        }

        // Add section header heights
        if let Some(ref sections) = self.sections {
            let total_header_height: f32 = sections.sections.iter().map(|s| s.height).sum();
            self.state.content_height += total_header_height;
        }

        self.update_visible_range();
    }

    /// Update the visible range based on current scroll position
    fn update_visible_range(&mut self) {
        self.state.visible_range =
            self.visible_range_for(self.state.scroll_offset, self.state.viewport_height);
    }

    /// Handle pull-to-refresh gesture
    pub fn handle_pull(&mut self, delta: f32) {
        if !self.config.pull_to_refresh || self.state.scroll_offset > 0.0 {
            return;
        }

        if delta < 0.0 {
            // Pulling down
            self.state.pull_progress = (-delta / 100.0).min(1.0);
        }
    }

    /// Start refresh
    pub fn start_refresh(&mut self) {
        if self.state.pull_progress >= 1.0 {
            self.state.is_refreshing = true;
        }
    }

    /// End refresh
    pub fn end_refresh(&mut self) {
        self.state.is_refreshing = false;
        self.state.pull_progress = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_list_creation() {
        let list = VirtualList::new()
            .items(1000)
            .fixed_height(48.0)
            .viewport_height(600.0);

        assert_eq!(list.state().total_items, 1000);
        assert!(!list.visible_range().is_empty());
    }

    #[test]
    fn test_fixed_height_content_calculation() {
        let list = VirtualList::new()
            .items(100)
            .fixed_height(50.0)
            .viewport_height(500.0);

        // 100 items * 50px = 5000px
        assert_eq!(list.content_height(), 5000.0);
    }

    #[test]
    fn test_visible_range_calculation() {
        let list = VirtualList::new()
            .items(1000)
            .fixed_height(50.0)
            .overscan(2)
            .viewport_height(300.0);

        // 300px viewport / 50px per item = 6 visible items
        // Plus 2 overscan on each side = 10 total
        let range = list.visible_range();
        assert_eq!(range.len(), 8); // 0-5 visible + 2 overscan below
    }

    #[test]
    fn test_scroll_to_index() {
        let mut list = VirtualList::new()
            .items(1000)
            .fixed_height(50.0)
            .viewport_height(300.0);

        list.scroll_to_index(50).unwrap();

        // Should scroll to item 50 (offset = 50 * 50 = 2500)
        assert!(list.state().scroll_offset >= 2500.0);
    }

    #[test]
    fn test_scroll_to_invalid_index() {
        let mut list = VirtualList::new()
            .items(100)
            .fixed_height(50.0)
            .viewport_height(300.0);

        let result = list.scroll_to_index(200);
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_height_items() {
        let mut list = VirtualList::new()
            .items(10)
            .variable_height(50.0)
            .viewport_height(300.0);

        // Set some measured heights
        list.set_item_height(0, 100.0);
        list.set_item_height(1, 30.0);
        list.set_item_height(2, 80.0);

        assert_eq!(list.get_item_height(0), 100.0);
        assert_eq!(list.get_item_height(1), 30.0);
        assert_eq!(list.get_item_height(5), 50.0); // Estimated
    }

    #[test]
    fn test_separator_height() {
        let list = VirtualList::new()
            .items(10)
            .fixed_height(50.0)
            .separator(Separator::full(1.0, [0.5, 0.5, 0.5, 1.0]))
            .viewport_height(300.0);

        // 10 items * 50px + 9 separators * 1px = 509px
        assert_eq!(list.content_height(), 509.0);
    }

    #[test]
    fn test_padding() {
        let list = VirtualList::new()
            .items(10)
            .fixed_height(50.0)
            .padding(20.0)
            .viewport_height(300.0);

        // 10 * 50 + 2 * 20 padding = 540
        assert_eq!(list.content_height(), 540.0);

        // First item should be at padding offset
        assert_eq!(list.offset_for_index(0), 20.0);
    }

    #[test]
    fn test_scroll_progress() {
        let mut list = VirtualList::new()
            .items(100)
            .fixed_height(50.0)
            .viewport_height(500.0);

        // Content: 5000px, viewport: 500px, max scroll: 4500px
        assert_eq!(list.scroll_progress(), 0.0);

        list.scroll_to_bottom();
        assert!((list.scroll_progress() - 1.0).abs() < 0.01);

        list.set_scroll_offset(2250.0);
        assert!((list.scroll_progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_item_bounds() {
        let list = VirtualList::new()
            .items(100)
            .fixed_height(50.0)
            .viewport_height(300.0);

        let bounds = list.item_bounds(0, 400.0);
        assert_eq!(bounds.x, 0.0);
        assert_eq!(bounds.y, 0.0);
        assert_eq!(bounds.width, 400.0);
        assert_eq!(bounds.height, 50.0);

        let bounds = list.item_bounds(5, 400.0);
        assert_eq!(bounds.y, 250.0); // 5 * 50
    }

    #[test]
    fn test_visible_items() {
        let list = VirtualList::new()
            .items(100)
            .fixed_height(50.0)
            .overscan(0) // No overscan for easier testing
            .viewport_height(150.0);

        let visible = list.visible_items(400.0);
        assert_eq!(visible.len(), 3); // 150 / 50 = 3 items

        assert_eq!(visible[0].index, 0);
        assert_eq!(visible[1].index, 1);
        assert_eq!(visible[2].index, 2);
    }

    #[test]
    fn test_scroll_clamping() {
        let mut list = VirtualList::new()
            .items(10)
            .fixed_height(50.0)
            .viewport_height(300.0);

        // Content: 500px, viewport: 300px, max: 200px
        list.set_scroll_offset(-100.0);
        assert_eq!(list.state().scroll_offset, 0.0);

        list.set_scroll_offset(1000.0);
        assert_eq!(list.state().scroll_offset, 200.0);
    }

    #[test]
    fn test_sections() {
        let sections = SectionConfig::new()
            .add_section(
                SectionHeader::new("a", "Section A", 40.0).with_items(0, 5)
            )
            .add_section(
                SectionHeader::new("b", "Section B", 40.0).with_items(5, 5)
            );

        let list = VirtualList::new()
            .items(10)
            .fixed_height(50.0)
            .sections(sections)
            .viewport_height(400.0);

        // Content should include section header heights
        // 10 * 50 + 2 * 40 = 580
        assert_eq!(list.content_height(), 580.0);
    }

    #[test]
    fn test_empty_list() {
        let list = VirtualList::new()
            .items(0)
            .fixed_height(50.0)
            .viewport_height(300.0);

        assert!(list.visible_range().is_empty());
        assert_eq!(list.content_height(), 0.0);
    }

    #[test]
    fn test_index_at_offset() {
        let list = VirtualList::new()
            .items(100)
            .fixed_height(50.0)
            .viewport_height(300.0);

        assert_eq!(list.index_at_offset(0.0), 0);
        assert_eq!(list.index_at_offset(25.0), 0);
        assert_eq!(list.index_at_offset(50.0), 1);
        assert_eq!(list.index_at_offset(100.0), 2);
        assert_eq!(list.index_at_offset(10000.0), 99); // Clamped to last
    }

    #[test]
    fn test_is_item_visible() {
        let list = VirtualList::new()
            .items(100)
            .fixed_height(50.0)
            .overscan(0)
            .viewport_height(100.0); // Shows items 0-1

        assert!(list.is_item_visible(0));
        assert!(list.is_item_visible(1));
        assert!(!list.is_item_visible(5));
    }
}
