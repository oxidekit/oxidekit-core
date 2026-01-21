//! Image gallery component.

use serde::{Deserialize, Serialize};

/// Gallery layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GalleryLayout {
    /// Grid layout
    #[default]
    Grid,
    /// Masonry layout
    Masonry,
    /// Carousel/slider
    Carousel,
    /// List layout
    List,
}

/// A gallery item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryItem {
    /// Image source
    pub src: String,
    /// Thumbnail source (optional)
    pub thumbnail: Option<String>,
    /// Alt text
    pub alt: String,
    /// Title
    pub title: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Width (for masonry)
    pub width: Option<u32>,
    /// Height (for masonry)
    pub height: Option<u32>,
}

impl GalleryItem {
    /// Create new item
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            thumbnail: None,
            alt: String::new(),
            title: None,
            description: None,
            width: None,
            height: None,
        }
    }

    /// Set thumbnail
    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.thumbnail = Some(url.into());
        self
    }

    /// Set alt text
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }

    /// Set title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set dimensions
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Get thumbnail or fallback to src
    pub fn thumbnail_src(&self) -> &str {
        self.thumbnail.as_deref().unwrap_or(&self.src)
    }
}

/// Gallery component
#[derive(Debug, Clone)]
pub struct Gallery {
    /// Items
    pub items: Vec<GalleryItem>,
    /// Layout
    pub layout: GalleryLayout,
    /// Columns (for grid/masonry)
    pub columns: usize,
    /// Gap between items
    pub gap: f32,
    /// Enable lightbox
    pub lightbox: bool,
    /// Selected index
    pub selected: Option<usize>,
}

impl Gallery {
    /// Create new gallery
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            layout: GalleryLayout::Grid,
            columns: 3,
            gap: 8.0,
            lightbox: true,
            selected: None,
        }
    }

    /// Add item
    pub fn add(mut self, item: GalleryItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set items
    pub fn items(mut self, items: Vec<GalleryItem>) -> Self {
        self.items = items;
        self
    }

    /// Set layout
    pub fn layout(mut self, layout: GalleryLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set columns
    pub fn columns(mut self, cols: usize) -> Self {
        self.columns = cols.max(1);
        self
    }

    /// Set gap
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Disable lightbox
    pub fn no_lightbox(mut self) -> Self {
        self.lightbox = false;
        self
    }

    /// Select item
    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = Some(index);
        }
    }

    /// Clear selection
    pub fn deselect(&mut self) {
        self.selected = None;
    }

    /// Go to next item
    pub fn next(&mut self) {
        if let Some(current) = self.selected {
            self.selected = Some((current + 1) % self.items.len());
        }
    }

    /// Go to previous item
    pub fn previous(&mut self) {
        if let Some(current) = self.selected {
            self.selected = Some(current.checked_sub(1).unwrap_or(self.items.len() - 1));
        }
    }
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}

/// Lightbox component
#[derive(Debug, Clone, Default)]
pub struct Lightbox {
    /// Visible
    pub visible: bool,
    /// Current index
    pub index: usize,
    /// Show controls
    pub show_controls: bool,
    /// Show counter
    pub show_counter: bool,
    /// Close on backdrop click
    pub close_on_backdrop: bool,
    /// Enable swipe gestures
    pub swipe_enabled: bool,
}

impl Lightbox {
    /// Create new lightbox
    pub fn new() -> Self {
        Self {
            visible: false,
            index: 0,
            show_controls: true,
            show_counter: true,
            close_on_backdrop: true,
            swipe_enabled: true,
        }
    }

    /// Open at index
    pub fn open(&mut self, index: usize) {
        self.visible = true;
        self.index = index;
    }

    /// Close
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Go to next
    pub fn next(&mut self, total: usize) {
        if total > 0 {
            self.index = (self.index + 1) % total;
        }
    }

    /// Go to previous
    pub fn previous(&mut self, total: usize) {
        if total > 0 {
            self.index = self.index.checked_sub(1).unwrap_or(total - 1);
        }
    }
}
