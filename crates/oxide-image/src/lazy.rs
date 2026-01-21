//! Lazy loading support.

use serde::{Deserialize, Serialize};

/// Visibility state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Visibility {
    /// Not visible
    #[default]
    Hidden,
    /// Partially visible
    Partial,
    /// Fully visible
    Visible,
}

/// Lazy load configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyLoadConfig {
    /// Root margin (pixels)
    pub root_margin: i32,
    /// Threshold (0.0 to 1.0)
    pub threshold: f32,
    /// Use native lazy loading if available
    pub use_native: bool,
    /// Placeholder while loading
    pub placeholder_type: PlaceholderType,
}

impl Default for LazyLoadConfig {
    fn default() -> Self {
        Self {
            root_margin: 100,
            threshold: 0.1,
            use_native: true,
            placeholder_type: PlaceholderType::Shimmer,
        }
    }
}

/// Placeholder type for lazy loading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlaceholderType {
    /// No placeholder
    None,
    /// Solid color
    Color,
    /// Shimmer/skeleton animation
    #[default]
    Shimmer,
    /// Blurred low-quality image
    BlurHash,
    /// Dominant color
    DominantColor,
}

/// Lazy image component
#[derive(Debug, Clone)]
pub struct LazyImage {
    /// Full resolution source
    pub src: String,
    /// Low quality placeholder (for blur-up)
    pub lqip: Option<String>,
    /// Alt text
    pub alt: String,
    /// Configuration
    pub config: LazyLoadConfig,
    /// Current visibility
    pub visibility: Visibility,
    /// Is loaded
    pub loaded: bool,
}

impl LazyImage {
    /// Create new lazy image
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            lqip: None,
            alt: String::new(),
            config: LazyLoadConfig::default(),
            visibility: Visibility::Hidden,
            loaded: false,
        }
    }

    /// Set low-quality placeholder
    pub fn lqip(mut self, url: impl Into<String>) -> Self {
        self.lqip = Some(url.into());
        self.config.placeholder_type = PlaceholderType::BlurHash;
        self
    }

    /// Set alt text
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }

    /// Set root margin
    pub fn root_margin(mut self, margin: i32) -> Self {
        self.config.root_margin = margin;
        self
    }

    /// Set threshold
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.config.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set placeholder type
    pub fn placeholder(mut self, placeholder: PlaceholderType) -> Self {
        self.config.placeholder_type = placeholder;
        self
    }

    /// Mark as visible (triggers load)
    pub fn set_visible(&mut self) {
        self.visibility = Visibility::Visible;
    }

    /// Mark as loaded
    pub fn set_loaded(&mut self) {
        self.loaded = true;
    }

    /// Should start loading
    pub fn should_load(&self) -> bool {
        self.visibility != Visibility::Hidden && !self.loaded
    }

    /// Get current source to display
    pub fn current_src(&self) -> Option<&str> {
        if self.loaded {
            Some(&self.src)
        } else {
            self.lqip.as_deref()
        }
    }
}
