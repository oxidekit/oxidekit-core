//! Scroll management for virtual lists.

use serde::{Deserialize, Serialize};

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScrollDirection {
    /// Vertical scrolling
    #[default]
    Vertical,
    /// Horizontal scrolling
    Horizontal,
    /// Both directions
    Both,
}

/// Scroll behavior for programmatic scrolling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScrollBehavior {
    /// Instant scroll
    #[default]
    Auto,
    /// Smooth animated scroll
    Smooth,
    /// Instant jump
    Instant,
}

/// Scroll position
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ScrollPosition {
    /// Top offset
    pub top: f32,
    /// Left offset
    pub left: f32,
}

impl ScrollPosition {
    /// Create a new scroll position
    pub fn new(top: f32, left: f32) -> Self {
        Self { top, left }
    }

    /// Create from top only
    pub fn from_top(top: f32) -> Self {
        Self { top, left: 0.0 }
    }
}

/// Scroll event data
#[derive(Debug, Clone)]
pub struct ScrollEvent {
    /// Current scroll position
    pub position: ScrollPosition,
    /// Previous scroll position
    pub previous: ScrollPosition,
    /// Scroll delta
    pub delta: ScrollPosition,
    /// Scroll direction
    pub direction: ScrollDirection,
}

/// State for scroll management
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    /// Current position
    pub position: ScrollPosition,
    /// Velocity for momentum scrolling
    pub velocity: ScrollPosition,
    /// Whether currently scrolling
    pub is_scrolling: bool,
    /// Whether momentum is active
    pub is_momentum: bool,
}

/// Configuration for sticky elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickyConfig {
    /// Offset from top when sticky
    pub offset: f32,
    /// Z-index for sticky element
    pub z_index: i32,
}

impl Default for StickyConfig {
    fn default() -> Self {
        Self {
            offset: 0.0,
            z_index: 10,
        }
    }
}

/// Sticky element
#[derive(Debug, Clone)]
pub struct StickyElement {
    /// Item index
    pub index: usize,
    /// Config
    pub config: StickyConfig,
    /// Original position
    pub original_y: f32,
}

/// Scroll configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollConfig {
    /// Scroll direction
    pub direction: ScrollDirection,
    /// Scroll behavior for programmatic scrolling
    pub behavior: ScrollBehavior,
    /// Enable momentum scrolling
    pub momentum: bool,
    /// Friction for momentum
    pub friction: f32,
    /// Bounce on edges
    pub bounce: bool,
}

impl Default for ScrollConfig {
    fn default() -> Self {
        Self {
            direction: ScrollDirection::Vertical,
            behavior: ScrollBehavior::Smooth,
            momentum: true,
            friction: 0.95,
            bounce: false,
        }
    }
}

/// Scroll controller
#[derive(Debug, Clone)]
pub struct ScrollController {
    /// Configuration
    pub config: ScrollConfig,
    /// State
    pub state: ScrollState,
    /// Content height
    pub content_height: f32,
    /// Viewport height
    pub viewport_height: f32,
}

impl Default for ScrollController {
    fn default() -> Self {
        Self {
            config: ScrollConfig::default(),
            state: ScrollState::default(),
            content_height: 0.0,
            viewport_height: 0.0,
        }
    }
}

impl ScrollController {
    /// Create a new scroll controller
    pub fn new() -> Self {
        Self::default()
    }

    /// Set content dimensions
    pub fn set_dimensions(&mut self, content_height: f32, viewport_height: f32) {
        self.content_height = content_height;
        self.viewport_height = viewport_height;
    }

    /// Scroll to position
    pub fn scroll_to(&mut self, position: ScrollPosition) {
        let max_scroll = (self.content_height - self.viewport_height).max(0.0);
        self.state.position.top = position.top.clamp(0.0, max_scroll);
        self.state.position.left = position.left.max(0.0);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_to(ScrollPosition::default());
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        let max_scroll = (self.content_height - self.viewport_height).max(0.0);
        self.scroll_to(ScrollPosition::from_top(max_scroll));
    }

    /// Get max scroll position
    pub fn max_scroll(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }

    /// Check if at top
    pub fn at_top(&self) -> bool {
        self.state.position.top <= 0.0
    }

    /// Check if at bottom
    pub fn at_bottom(&self) -> bool {
        self.state.position.top >= self.max_scroll()
    }
}
