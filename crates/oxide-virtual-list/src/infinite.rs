//! Infinite scroll support for virtual lists.

use serde::{Deserialize, Serialize};

/// Loading state for infinite scroll
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LoadingState {
    /// Not loading
    #[default]
    Idle,
    /// Loading more items
    Loading,
    /// All items loaded
    Complete,
    /// Loading failed
    Error,
}

/// Trigger position for loading more items
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoadMoreTrigger {
    /// Trigger when reaching end of list
    EndReached,
    /// Trigger when items from end threshold
    Threshold(usize),
    /// Trigger at specific scroll percentage
    Percentage(f32),
}

impl Default for LoadMoreTrigger {
    fn default() -> Self {
        LoadMoreTrigger::Threshold(5)
    }
}

/// Configuration for infinite scroll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfiniteScrollConfig {
    /// When to trigger loading
    pub trigger: LoadMoreTrigger,
    /// Debounce delay in milliseconds
    pub debounce_ms: u32,
    /// Minimum time between load requests
    pub cooldown_ms: u32,
}

impl Default for InfiniteScrollConfig {
    fn default() -> Self {
        Self {
            trigger: LoadMoreTrigger::default(),
            debounce_ms: 150,
            cooldown_ms: 500,
        }
    }
}

/// State for infinite scroll
#[derive(Debug, Clone, Default)]
pub struct InfiniteScrollState {
    /// Current loading state
    pub loading_state: LoadingState,
    /// Total items loaded
    pub items_loaded: usize,
    /// Whether more items are available
    pub has_more: bool,
}

/// Infinite scroll controller
#[derive(Debug, Clone)]
pub struct InfiniteScroll {
    /// Configuration
    pub config: InfiniteScrollConfig,
    /// State
    pub state: InfiniteScrollState,
}

impl Default for InfiniteScroll {
    fn default() -> Self {
        Self {
            config: InfiniteScrollConfig::default(),
            state: InfiniteScrollState::default(),
        }
    }
}

impl InfiniteScroll {
    /// Create a new infinite scroll controller
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the trigger
    pub fn trigger(mut self, trigger: LoadMoreTrigger) -> Self {
        self.config.trigger = trigger;
        self
    }

    /// Check if should load more
    pub fn should_load(&self, visible_end: usize, total_items: usize) -> bool {
        if self.state.loading_state != LoadingState::Idle || !self.state.has_more {
            return false;
        }

        match self.config.trigger {
            LoadMoreTrigger::EndReached => visible_end >= total_items,
            LoadMoreTrigger::Threshold(n) => visible_end + n >= total_items,
            LoadMoreTrigger::Percentage(p) => {
                (visible_end as f32 / total_items as f32) >= p
            }
        }
    }

    /// Start loading
    pub fn start_loading(&mut self) {
        self.state.loading_state = LoadingState::Loading;
    }

    /// Finish loading
    pub fn finish_loading(&mut self, new_items: usize, has_more: bool) {
        self.state.loading_state = LoadingState::Idle;
        self.state.items_loaded += new_items;
        self.state.has_more = has_more;
    }

    /// Set error state
    pub fn set_error(&mut self) {
        self.state.loading_state = LoadingState::Error;
    }
}
