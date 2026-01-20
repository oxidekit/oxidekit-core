//! Pagination for DataTable.
//!
//! This module provides pagination functionality including page size options,
//! page navigation, and server-side pagination support.

use serde::{Deserialize, Serialize};

/// Predefined page size options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PageSize {
    /// 10 rows per page.
    Ten,
    /// 25 rows per page.
    TwentyFive,
    /// 50 rows per page.
    Fifty,
    /// 100 rows per page.
    Hundred,
    /// Custom page size.
    Custom(usize),
    /// No pagination (show all rows).
    All,
}

impl PageSize {
    /// Get the numeric value of the page size.
    pub fn value(&self) -> Option<usize> {
        match self {
            PageSize::Ten => Some(10),
            PageSize::TwentyFive => Some(25),
            PageSize::Fifty => Some(50),
            PageSize::Hundred => Some(100),
            PageSize::Custom(n) => Some(*n),
            PageSize::All => None,
        }
    }

    /// Get the display label for the page size.
    pub fn label(&self) -> String {
        match self {
            PageSize::Ten => "10".to_string(),
            PageSize::TwentyFive => "25".to_string(),
            PageSize::Fifty => "50".to_string(),
            PageSize::Hundred => "100".to_string(),
            PageSize::Custom(n) => n.to_string(),
            PageSize::All => "All".to_string(),
        }
    }
}

impl Default for PageSize {
    fn default() -> Self {
        PageSize::TwentyFive
    }
}

impl From<usize> for PageSize {
    fn from(value: usize) -> Self {
        match value {
            10 => PageSize::Ten,
            25 => PageSize::TwentyFive,
            50 => PageSize::Fifty,
            100 => PageSize::Hundred,
            n => PageSize::Custom(n),
        }
    }
}

/// Pagination state for the table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationState {
    /// Current page (0-indexed).
    pub page: usize,
    /// Page size.
    pub page_size: PageSize,
    /// Total number of items (for display).
    pub total_items: usize,
    /// Whether pagination is server-side.
    pub server_side: bool,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: PageSize::TwentyFive,
            total_items: 0,
            server_side: false,
        }
    }
}

impl PaginationState {
    /// Create a new pagination state.
    pub fn new(page_size: PageSize) -> Self {
        Self {
            page: 0,
            page_size,
            total_items: 0,
            server_side: false,
        }
    }

    /// Create a server-side pagination state.
    pub fn server_side(page_size: PageSize) -> Self {
        Self {
            page: 0,
            page_size,
            total_items: 0,
            server_side: true,
        }
    }

    /// Get the effective page size (returns total_items if PageSize::All).
    pub fn effective_page_size(&self) -> usize {
        self.page_size.value().unwrap_or(self.total_items.max(1))
    }

    /// Get the total number of pages.
    pub fn total_pages(&self) -> usize {
        let page_size = self.effective_page_size();
        if page_size == 0 {
            1
        } else {
            (self.total_items + page_size - 1) / page_size
        }
    }

    /// Get the start index for the current page.
    pub fn start_index(&self) -> usize {
        self.page * self.effective_page_size()
    }

    /// Get the end index for the current page (exclusive).
    pub fn end_index(&self) -> usize {
        let end = self.start_index() + self.effective_page_size();
        end.min(self.total_items)
    }

    /// Get the number of items on the current page.
    pub fn items_on_page(&self) -> usize {
        self.end_index().saturating_sub(self.start_index())
    }

    /// Check if there is a previous page.
    pub fn has_prev(&self) -> bool {
        self.page > 0
    }

    /// Check if there is a next page.
    pub fn has_next(&self) -> bool {
        self.page + 1 < self.total_pages()
    }

    /// Check if on the first page.
    pub fn is_first(&self) -> bool {
        self.page == 0
    }

    /// Check if on the last page.
    pub fn is_last(&self) -> bool {
        self.page + 1 >= self.total_pages()
    }

    /// Go to the first page.
    pub fn first(&mut self) {
        self.page = 0;
    }

    /// Go to the previous page.
    pub fn prev(&mut self) {
        if self.has_prev() {
            self.page -= 1;
        }
    }

    /// Go to the next page.
    pub fn next(&mut self) {
        if self.has_next() {
            self.page += 1;
        }
    }

    /// Go to the last page.
    pub fn last(&mut self) {
        let total = self.total_pages();
        self.page = if total > 0 { total - 1 } else { 0 };
    }

    /// Go to a specific page.
    pub fn go_to(&mut self, page: usize) {
        let max_page = self.total_pages().saturating_sub(1);
        self.page = page.min(max_page);
    }

    /// Set the page size.
    pub fn set_page_size(&mut self, page_size: PageSize) {
        // Calculate new page to keep the first item of current page visible
        let first_item = self.start_index();
        self.page_size = page_size;
        let new_page_size = self.effective_page_size();
        self.page = if new_page_size > 0 {
            first_item / new_page_size
        } else {
            0
        };
    }

    /// Set the total number of items (typically called after data loading).
    pub fn set_total(&mut self, total: usize) {
        self.total_items = total;
        // Ensure current page is still valid
        let max_page = self.total_pages().saturating_sub(1);
        if self.page > max_page {
            self.page = max_page;
        }
    }

    /// Get a slice of data for the current page.
    pub fn paginate<'a, T>(&self, data: &'a [T]) -> &'a [T] {
        if self.page_size == PageSize::All {
            return data;
        }
        let start = self.start_index().min(data.len());
        let end = self.end_index().min(data.len());
        &data[start..end]
    }

    /// Get paginated indices instead of data.
    pub fn paginated_indices(&self) -> std::ops::Range<usize> {
        self.start_index()..self.end_index()
    }
}

/// Pagination display info for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Current page (1-indexed for display).
    pub current_page: usize,
    /// Total pages.
    pub total_pages: usize,
    /// First item index (1-indexed for display).
    pub first_item: usize,
    /// Last item index (1-indexed for display).
    pub last_item: usize,
    /// Total items.
    pub total_items: usize,
    /// Can go to previous page.
    pub can_prev: bool,
    /// Can go to next page.
    pub can_next: bool,
}

impl PaginationInfo {
    /// Create pagination info from state.
    pub fn from_state(state: &PaginationState) -> Self {
        Self {
            current_page: state.page + 1,
            total_pages: state.total_pages(),
            first_item: if state.total_items > 0 { state.start_index() + 1 } else { 0 },
            last_item: state.end_index(),
            total_items: state.total_items,
            can_prev: state.has_prev(),
            can_next: state.has_next(),
        }
    }

    /// Get the display text (e.g., "1-25 of 100").
    pub fn display_text(&self) -> String {
        if self.total_items == 0 {
            "No items".to_string()
        } else {
            format!("{}-{} of {}", self.first_item, self.last_item, self.total_items)
        }
    }

    /// Get the page display text (e.g., "Page 1 of 4").
    pub fn page_text(&self) -> String {
        if self.total_pages == 0 {
            "Page 0 of 0".to_string()
        } else {
            format!("Page {} of {}", self.current_page, self.total_pages)
        }
    }
}

/// Configuration for page size selector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSizeOptions {
    /// Available page size options.
    pub options: Vec<PageSize>,
    /// Whether to include "All" option.
    pub include_all: bool,
}

impl Default for PageSizeOptions {
    fn default() -> Self {
        Self {
            options: vec![
                PageSize::Ten,
                PageSize::TwentyFive,
                PageSize::Fifty,
                PageSize::Hundred,
            ],
            include_all: false,
        }
    }
}

impl PageSizeOptions {
    /// Create page size options with specific sizes.
    pub fn new(sizes: Vec<usize>) -> Self {
        Self {
            options: sizes.into_iter().map(PageSize::from).collect(),
            include_all: false,
        }
    }

    /// Include the "All" option.
    pub fn with_all(mut self) -> Self {
        self.include_all = true;
        self
    }

    /// Get all options including "All" if enabled.
    pub fn all_options(&self) -> Vec<PageSize> {
        let mut options = self.options.clone();
        if self.include_all {
            options.push(PageSize::All);
        }
        options
    }
}

/// Server-side pagination request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationRequest {
    /// Page number (0-indexed).
    pub page: usize,
    /// Page size.
    pub page_size: usize,
    /// Skip (offset) - same as page * page_size.
    pub skip: usize,
    /// Take (limit) - same as page_size.
    pub take: usize,
}

impl PaginationRequest {
    /// Create a pagination request from state.
    pub fn from_state(state: &PaginationState) -> Self {
        let page_size = state.effective_page_size();
        Self {
            page: state.page,
            page_size,
            skip: state.start_index(),
            take: page_size,
        }
    }
}

/// Server-side pagination response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponse<T> {
    /// Data for the current page.
    pub data: Vec<T>,
    /// Total number of items.
    pub total: usize,
    /// Current page (0-indexed).
    pub page: usize,
    /// Page size.
    pub page_size: usize,
    /// Total pages.
    pub total_pages: usize,
}

impl<T> PaginationResponse<T> {
    /// Create a pagination response.
    pub fn new(data: Vec<T>, total: usize, page: usize, page_size: usize) -> Self {
        let total_pages = if page_size > 0 {
            (total + page_size - 1) / page_size
        } else {
            1
        };
        Self {
            data,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// Callback type for page change events.
pub type OnPageChange = Box<dyn Fn(usize) + Send + Sync>;

/// Callback type for page size change events.
pub type OnPageSizeChange = Box<dyn Fn(PageSize) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size_values() {
        assert_eq!(PageSize::Ten.value(), Some(10));
        assert_eq!(PageSize::TwentyFive.value(), Some(25));
        assert_eq!(PageSize::Fifty.value(), Some(50));
        assert_eq!(PageSize::Hundred.value(), Some(100));
        assert_eq!(PageSize::Custom(15).value(), Some(15));
        assert_eq!(PageSize::All.value(), None);
    }

    #[test]
    fn test_page_size_from() {
        assert_eq!(PageSize::from(10), PageSize::Ten);
        assert_eq!(PageSize::from(25), PageSize::TwentyFive);
        assert_eq!(PageSize::from(15), PageSize::Custom(15));
    }

    #[test]
    fn test_pagination_state_new() {
        let state = PaginationState::new(PageSize::TwentyFive);
        assert_eq!(state.page, 0);
        assert_eq!(state.page_size, PageSize::TwentyFive);
        assert!(!state.server_side);
    }

    #[test]
    fn test_pagination_state_server_side() {
        let state = PaginationState::server_side(PageSize::Fifty);
        assert!(state.server_side);
    }

    #[test]
    fn test_total_pages() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(95);

        assert_eq!(state.total_pages(), 10); // Ceiling of 95/10

        state.set_total(100);
        assert_eq!(state.total_pages(), 10); // Exactly 100/10
    }

    #[test]
    fn test_start_end_indices() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(95);

        assert_eq!(state.start_index(), 0);
        assert_eq!(state.end_index(), 10);

        state.go_to(1);
        assert_eq!(state.start_index(), 10);
        assert_eq!(state.end_index(), 20);

        state.last();
        assert_eq!(state.start_index(), 90);
        assert_eq!(state.end_index(), 95); // Last page has only 5 items
    }

    #[test]
    fn test_navigation() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(50);

        assert!(state.is_first());
        assert!(!state.is_last());
        assert!(!state.has_prev());
        assert!(state.has_next());

        state.next();
        assert_eq!(state.page, 1);
        assert!(state.has_prev());

        state.last();
        assert_eq!(state.page, 4);
        assert!(state.is_last());
        assert!(!state.has_next());

        state.prev();
        assert_eq!(state.page, 3);

        state.first();
        assert_eq!(state.page, 0);
    }

    #[test]
    fn test_go_to() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(50);

        state.go_to(2);
        assert_eq!(state.page, 2);

        state.go_to(100); // Beyond max
        assert_eq!(state.page, 4); // Clamped to last page
    }

    #[test]
    fn test_set_page_size() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(100);
        state.go_to(3); // Items 30-39

        state.set_page_size(PageSize::TwentyFive);
        // New page should show item 30 (page 1 in 25-item pages)
        assert_eq!(state.page, 1);
    }

    #[test]
    fn test_set_total_adjusts_page() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(100);
        state.go_to(9); // Page 9 (last page)

        state.set_total(50); // Now only 5 pages
        assert_eq!(state.page, 4); // Adjusted to last valid page
    }

    #[test]
    fn test_paginate() {
        let mut state = PaginationState::new(PageSize::Custom(3));
        let data: Vec<i32> = (0..10).collect();
        state.set_total(data.len());

        assert_eq!(state.paginate(&data), &[0, 1, 2]);

        state.next();
        assert_eq!(state.paginate(&data), &[3, 4, 5]);

        state.last();
        assert_eq!(state.paginate(&data), &[9]); // Last page has 1 item
    }

    #[test]
    fn test_paginate_all() {
        let mut state = PaginationState::new(PageSize::All);
        let data: Vec<i32> = (0..10).collect();
        state.set_total(data.len());

        assert_eq!(state.paginate(&data), &data[..]);
    }

    #[test]
    fn test_pagination_info() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(95);

        let info = PaginationInfo::from_state(&state);
        assert_eq!(info.current_page, 1);
        assert_eq!(info.total_pages, 10);
        assert_eq!(info.first_item, 1);
        assert_eq!(info.last_item, 10);
        assert_eq!(info.total_items, 95);
        assert_eq!(info.display_text(), "1-10 of 95");
        assert_eq!(info.page_text(), "Page 1 of 10");
    }

    #[test]
    fn test_pagination_info_empty() {
        let state = PaginationState::new(PageSize::Ten);
        let info = PaginationInfo::from_state(&state);

        assert_eq!(info.display_text(), "No items");
    }

    #[test]
    fn test_page_size_options() {
        let options = PageSizeOptions::default();
        assert_eq!(options.options.len(), 4);
        assert!(!options.include_all);

        let options = PageSizeOptions::new(vec![5, 10, 20]).with_all();
        let all = options.all_options();
        assert_eq!(all.len(), 4);
        assert_eq!(all.last(), Some(&PageSize::All));
    }

    #[test]
    fn test_pagination_request() {
        let mut state = PaginationState::new(PageSize::TwentyFive);
        state.set_total(100);
        state.go_to(2);

        let request = PaginationRequest::from_state(&state);
        assert_eq!(request.page, 2);
        assert_eq!(request.page_size, 25);
        assert_eq!(request.skip, 50);
        assert_eq!(request.take, 25);
    }

    #[test]
    fn test_pagination_response() {
        let data = vec![1, 2, 3, 4, 5];
        let response = PaginationResponse::new(data, 50, 0, 5);

        assert_eq!(response.total, 50);
        assert_eq!(response.total_pages, 10);
        assert_eq!(response.data.len(), 5);
    }

    #[test]
    fn test_items_on_page() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(95);

        assert_eq!(state.items_on_page(), 10);

        state.last();
        assert_eq!(state.items_on_page(), 5); // Last page has 5 items
    }

    #[test]
    fn test_paginated_indices() {
        let mut state = PaginationState::new(PageSize::Ten);
        state.set_total(50);

        assert_eq!(state.paginated_indices(), 0..10);

        state.go_to(2);
        assert_eq!(state.paginated_indices(), 20..30);
    }
}
