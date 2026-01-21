//! Avatar component.

use serde::{Deserialize, Serialize};

/// Avatar size preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AvatarSize {
    /// Extra small (24px)
    XSmall,
    /// Small (32px)
    Small,
    /// Medium (40px)
    #[default]
    Medium,
    /// Large (48px)
    Large,
    /// Extra large (64px)
    XLarge,
    /// Custom size
    Custom(u32),
}

impl AvatarSize {
    /// Get size in pixels
    pub fn pixels(&self) -> u32 {
        match self {
            AvatarSize::XSmall => 24,
            AvatarSize::Small => 32,
            AvatarSize::Medium => 40,
            AvatarSize::Large => 48,
            AvatarSize::XLarge => 64,
            AvatarSize::Custom(px) => *px,
        }
    }
}

/// Online status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Status {
    /// No status shown
    #[default]
    None,
    /// Online
    Online,
    /// Offline
    Offline,
    /// Away
    Away,
    /// Busy/Do not disturb
    Busy,
}

impl Status {
    /// Get status color
    pub fn color(&self) -> Option<&'static str> {
        match self {
            Status::None => None,
            Status::Online => Some("#4CAF50"),
            Status::Offline => Some("#9E9E9E"),
            Status::Away => Some("#FFC107"),
            Status::Busy => Some("#F44336"),
        }
    }
}

/// Avatar component
#[derive(Debug, Clone)]
pub struct Avatar {
    /// Image source URL
    pub image: Option<String>,
    /// Fallback initials
    pub initials: Option<String>,
    /// Size
    pub size: AvatarSize,
    /// Status indicator
    pub status: Status,
    /// Alt text
    pub alt: String,
    /// Border radius (0.0 = square, 1.0 = circle)
    pub border_radius: f32,
    /// Background color for initials
    pub background: String,
    /// Text color for initials
    pub text_color: String,
}

impl Avatar {
    /// Create new avatar
    pub fn new() -> Self {
        Self {
            image: None,
            initials: None,
            size: AvatarSize::Medium,
            status: Status::None,
            alt: "Avatar".to_string(),
            border_radius: 1.0,
            background: "#E0E0E0".to_string(),
            text_color: "#616161".to_string(),
        }
    }

    /// Set image source
    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.image = Some(url.into());
        self
    }

    /// Set fallback initials
    pub fn fallback_initials(mut self, initials: impl Into<String>) -> Self {
        self.initials = Some(initials.into());
        self
    }

    /// Set size
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// Set status
    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// Set alt text
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }

    /// Set square shape
    pub fn square(mut self) -> Self {
        self.border_radius = 0.0;
        self
    }

    /// Set rounded shape
    pub fn rounded(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Generate initials from name
    pub fn from_name(name: &str) -> String {
        name.split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}
