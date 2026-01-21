//! Infinite scroll component.

use serde::{Deserialize, Serialize};

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScrollDirection {
    /// Scroll down to load more
    #[default]
    Down,
    /// Scroll up to load more
    Up,
    /// Both directions
    Both,
}

/// Infinite scroll component
#[derive(Debug, Clone)]
pub struct InfiniteScroll {
    /// Direction
    pub direction: ScrollDirection,
    /// Loading state
    pub loading: bool,
    /// Has more items
    pub has_more: bool,
    /// Threshold in pixels
    pub threshold: f32,
    /// Current page
    pub page: usize,
    /// Items per page
    pub page_size: usize,
}

impl Default for InfiniteScroll {
    fn default() -> Self {
        Self::new()
    }
}

impl InfiniteScroll {
    /// Create new infinite scroll
    pub fn new() -> Self {
        Self {
            direction: ScrollDirection::Down,
            loading: false,
            has_more: true,
            threshold: 200.0,
            page: 1,
            page_size: 20,
        }
    }

    /// Set direction
    pub fn direction(mut self, direction: ScrollDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set threshold
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Set page size
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size;
        self
    }

    /// Start loading
    pub fn start_loading(&mut self) {
        self.loading = true;
    }

    /// Finish loading
    pub fn finish_loading(&mut self, has_more: bool) {
        self.loading = false;
        self.has_more = has_more;
        if has_more {
            self.page += 1;
        }
    }

    /// Reset
    pub fn reset(&mut self) {
        self.page = 1;
        self.has_more = true;
        self.loading = false;
    }

    /// Check if should load more
    pub fn should_load(&self) -> bool {
        !self.loading && self.has_more
    }
}

/// End indicator component
#[derive(Debug, Clone, Default)]
pub struct EndIndicator {
    /// Message
    pub message: String,
    /// Show indicator
    pub visible: bool,
}

impl EndIndicator {
    /// Create new end indicator
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            visible: true,
        }
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}
