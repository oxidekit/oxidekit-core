//! Image component.

use serde::{Deserialize, Serialize};
use crate::loader::ImageSource;

/// Image fit mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ImageFit {
    /// Cover the entire area
    #[default]
    Cover,
    /// Fit within area (may have letterboxing)
    Contain,
    /// Stretch to fill
    Fill,
    /// No scaling
    None,
    /// Scale down only
    ScaleDown,
}

/// Image state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ImageState {
    /// Not loaded
    #[default]
    Idle,
    /// Loading
    Loading,
    /// Loaded successfully
    Loaded,
    /// Failed to load
    Error,
}

/// Placeholder type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Placeholder {
    /// No placeholder
    None,
    /// Solid color
    Color(String),
    /// Shimmer animation
    Shimmer,
    /// Blur (LQIP)
    Blur(String),
    /// Custom component
    Custom,
}

impl Default for Placeholder {
    fn default() -> Self {
        Placeholder::Shimmer
    }
}

/// Image component
#[derive(Debug, Clone)]
pub struct Image {
    /// Source
    pub source: ImageSource,
    /// Alt text
    pub alt: String,
    /// Fit mode
    pub fit: ImageFit,
    /// State
    pub state: ImageState,
    /// Width (optional)
    pub width: Option<u32>,
    /// Height (optional)
    pub height: Option<u32>,
    /// Placeholder
    pub placeholder: Placeholder,
    /// Error fallback source
    pub error_fallback: Option<ImageSource>,
    /// Lazy loading
    pub lazy: bool,
    /// Border radius
    pub border_radius: f32,
}

impl Image {
    /// Create new image from URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            source: ImageSource::Url(url.into()),
            alt: String::new(),
            fit: ImageFit::Cover,
            state: ImageState::Idle,
            width: None,
            height: None,
            placeholder: Placeholder::Shimmer,
            error_fallback: None,
            lazy: true,
            border_radius: 0.0,
        }
    }

    /// Create from asset path
    pub fn asset(path: impl Into<String>) -> Self {
        Self {
            source: ImageSource::File(path.into().into()),
            alt: String::new(),
            fit: ImageFit::Cover,
            state: ImageState::Idle,
            width: None,
            height: None,
            placeholder: Placeholder::Shimmer,
            error_fallback: None,
            lazy: true,
            border_radius: 0.0,
        }
    }

    /// Set alt text
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }

    /// Set fit mode
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    /// Set size
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set placeholder
    pub fn placeholder(mut self, placeholder: Placeholder) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Set error fallback
    pub fn error_fallback(mut self, fallback: Image) -> Self {
        self.error_fallback = Some(fallback.source);
        self
    }

    /// Disable lazy loading
    pub fn eager(mut self) -> Self {
        self.lazy = false;
        self
    }

    /// Set border radius
    pub fn rounded(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }
}
