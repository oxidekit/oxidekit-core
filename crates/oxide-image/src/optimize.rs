//! Image optimization.

use serde::{Deserialize, Serialize};

/// Resolution hint for responsive images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionHint {
    /// 1x (standard)
    Standard,
    /// 2x (retina)
    Retina,
    /// 3x (high-DPI)
    HighDpi,
    /// Custom scale
    Custom(u8),
}

impl Default for ResolutionHint {
    fn default() -> Self {
        ResolutionHint::Standard
    }
}

impl ResolutionHint {
    /// Get scale factor
    pub fn scale(&self) -> f32 {
        match self {
            ResolutionHint::Standard => 1.0,
            ResolutionHint::Retina => 2.0,
            ResolutionHint::HighDpi => 3.0,
            ResolutionHint::Custom(s) => *s as f32,
        }
    }
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Target quality (0-100)
    pub quality: u8,
    /// Maximum width
    pub max_width: Option<u32>,
    /// Maximum height
    pub max_height: Option<u32>,
    /// Preferred format
    pub format: Option<String>,
    /// Strip metadata
    pub strip_metadata: bool,
    /// Enable progressive loading
    pub progressive: bool,
    /// Resolution hint
    pub resolution: ResolutionHint,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            quality: 85,
            max_width: None,
            max_height: None,
            format: None,
            strip_metadata: true,
            progressive: true,
            resolution: ResolutionHint::Standard,
        }
    }
}

impl OptimizationConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set quality
    pub fn quality(mut self, quality: u8) -> Self {
        self.quality = quality.min(100);
        self
    }

    /// Set max dimensions
    pub fn max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    /// Set max width
    pub fn max_width(mut self, width: u32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set max height
    pub fn max_height(mut self, height: u32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Prefer WebP format
    pub fn webp(mut self) -> Self {
        self.format = Some("webp".to_string());
        self
    }

    /// Set resolution
    pub fn resolution(mut self, hint: ResolutionHint) -> Self {
        self.resolution = hint;
        self
    }

    /// For thumbnails
    pub fn thumbnail(size: u32) -> Self {
        Self {
            quality: 75,
            max_width: Some(size),
            max_height: Some(size),
            strip_metadata: true,
            progressive: false,
            ..Default::default()
        }
    }

    /// For previews
    pub fn preview() -> Self {
        Self {
            quality: 60,
            max_width: Some(400),
            max_height: Some(400),
            strip_metadata: true,
            progressive: true,
            ..Default::default()
        }
    }

    /// Calculate target dimensions
    pub fn target_dimensions(&self, original_width: u32, original_height: u32) -> (u32, u32) {
        let scale = self.resolution.scale();
        let mut width = original_width;
        let mut height = original_height;

        // Apply max constraints
        if let Some(max_w) = self.max_width {
            let max_w = (max_w as f32 * scale) as u32;
            if width > max_w {
                let ratio = max_w as f32 / width as f32;
                width = max_w;
                height = (height as f32 * ratio) as u32;
            }
        }

        if let Some(max_h) = self.max_height {
            let max_h = (max_h as f32 * scale) as u32;
            if height > max_h {
                let ratio = max_h as f32 / height as f32;
                height = max_h;
                width = (width as f32 * ratio) as u32;
            }
        }

        (width, height)
    }
}
