//! Loading overlay components.

use serde::{Deserialize, Serialize};

/// Backdrop style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BackdropStyle {
    /// Transparent
    Transparent,
    /// Light blur
    #[default]
    Light,
    /// Dark blur
    Dark,
    /// Solid color
    Solid,
}

/// Full screen loader
#[derive(Debug, Clone)]
pub struct FullScreenLoader {
    /// Message
    pub message: Option<String>,
    /// Backdrop style
    pub backdrop: BackdropStyle,
    /// Visible
    pub visible: bool,
    /// Progress (0.0 - 1.0)
    pub progress: Option<f32>,
}

impl Default for FullScreenLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl FullScreenLoader {
    /// Create new full screen loader
    pub fn new() -> Self {
        Self {
            message: None,
            backdrop: BackdropStyle::Light,
            visible: false,
            progress: None,
        }
    }

    /// Set message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set backdrop
    pub fn backdrop(mut self, style: BackdropStyle) -> Self {
        self.backdrop = style;
        self
    }

    /// Set progress
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }

    /// Show the loader
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the loader
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Update progress
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = Some(progress.clamp(0.0, 1.0));
    }
}

/// Container loader
#[derive(Debug, Clone)]
pub struct ContainerLoader {
    /// Message
    pub message: Option<String>,
    /// Visible
    pub visible: bool,
    /// Cover entire container
    pub cover: bool,
}

impl Default for ContainerLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerLoader {
    /// Create new container loader
    pub fn new() -> Self {
        Self {
            message: None,
            visible: false,
            cover: true,
        }
    }

    /// Set message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set cover mode
    pub fn cover(mut self, cover: bool) -> Self {
        self.cover = cover;
        self
    }

    /// Show the loader
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the loader
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

/// Inline loader
#[derive(Debug, Clone)]
pub struct InlineLoader {
    /// Size
    pub size: f32,
    /// Color
    pub color: Option<String>,
    /// Visible
    pub visible: bool,
}

impl Default for InlineLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl InlineLoader {
    /// Create new inline loader
    pub fn new() -> Self {
        Self {
            size: 24.0,
            color: None,
            visible: true,
        }
    }

    /// Set size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set color
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}
