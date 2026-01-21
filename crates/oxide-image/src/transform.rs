//! Image transformations.

use serde::{Deserialize, Serialize};

/// Resize mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ResizeMode {
    /// Cover area (may crop)
    #[default]
    Cover,
    /// Fit within area (may letterbox)
    Contain,
    /// Stretch to fit
    Fill,
    /// No resizing
    None,
}

/// Border configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Border {
    /// Border width
    pub width: f32,
    /// Border color
    pub color: String,
    /// Border radius
    pub radius: f32,
}

impl Border {
    /// Create new border
    pub fn new(width: f32, color: impl Into<String>) -> Self {
        Self {
            width,
            color: color.into(),
            radius: 0.0,
        }
    }

    /// With radius
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

/// Shadow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shadow {
    /// X offset
    pub offset_x: f32,
    /// Y offset
    pub offset_y: f32,
    /// Blur radius
    pub blur: f32,
    /// Shadow color
    pub color: String,
}

impl Shadow {
    /// Create new shadow
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, color: impl Into<String>) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            color: color.into(),
        }
    }

    /// Simple drop shadow
    pub fn drop(blur: f32) -> Self {
        Self::new(0.0, 4.0, blur, "rgba(0,0,0,0.25)")
    }
}

/// A single transform operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transform {
    /// Resize
    Resize {
        width: u32,
        height: u32,
        mode: ResizeMode,
    },
    /// Crop
    Crop {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// Rotate (degrees)
    Rotate(f32),
    /// Flip horizontal
    FlipH,
    /// Flip vertical
    FlipV,
    /// Blur
    Blur(f32),
    /// Sharpen
    Sharpen(f32),
    /// Brightness (-1.0 to 1.0)
    Brightness(f32),
    /// Contrast (-1.0 to 1.0)
    Contrast(f32),
    /// Saturation (-1.0 to 1.0)
    Saturation(f32),
    /// Grayscale
    Grayscale,
    /// Sepia
    Sepia,
    /// Invert colors
    Invert,
    /// Rounded corners
    RoundedCorners(f32),
    /// Add border
    Border(Border),
    /// Add shadow
    Shadow(Shadow),
}

impl Transform {
    /// Create resize transform
    pub fn resize(width: u32, height: u32, mode: ResizeMode) -> Self {
        Transform::Resize { width, height, mode }
    }

    /// Create crop transform
    pub fn crop(x: u32, y: u32, width: u32, height: u32) -> Self {
        Transform::Crop { x, y, width, height }
    }
}

/// Pipeline of transforms
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformPipeline {
    /// Operations
    pub operations: Vec<Transform>,
}

impl TransformPipeline {
    /// Create empty pipeline
    pub fn new() -> Self {
        Self::default()
    }

    /// Add transform
    pub fn add(mut self, transform: Transform) -> Self {
        self.operations.push(transform);
        self
    }

    /// Resize
    pub fn resize(self, width: u32, height: u32, mode: ResizeMode) -> Self {
        self.add(Transform::resize(width, height, mode))
    }

    /// Blur
    pub fn blur(self, radius: f32) -> Self {
        self.add(Transform::Blur(radius))
    }

    /// Grayscale
    pub fn grayscale(self) -> Self {
        self.add(Transform::Grayscale)
    }

    /// Rounded corners
    pub fn rounded_corners(self, radius: f32) -> Self {
        self.add(Transform::RoundedCorners(radius))
    }

    /// Add border
    pub fn border(self, width: f32, color: impl Into<String>) -> Self {
        self.add(Transform::Border(Border::new(width, color)))
    }

    /// Add shadow
    pub fn shadow(self, offset_x: f32, offset_y: f32, blur: f32, color: impl Into<String>) -> Self {
        self.add(Transform::Shadow(Shadow::new(offset_x, offset_y, blur, color)))
    }

    /// Rotate
    pub fn rotate(self, degrees: f32) -> Self {
        self.add(Transform::Rotate(degrees))
    }

    /// Flip horizontal
    pub fn flip_h(self) -> Self {
        self.add(Transform::FlipH)
    }

    /// Flip vertical
    pub fn flip_v(self) -> Self {
        self.add(Transform::FlipV)
    }

    /// Brightness
    pub fn brightness(self, value: f32) -> Self {
        self.add(Transform::Brightness(value.clamp(-1.0, 1.0)))
    }

    /// Contrast
    pub fn contrast(self, value: f32) -> Self {
        self.add(Transform::Contrast(value.clamp(-1.0, 1.0)))
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Operation count
    pub fn len(&self) -> usize {
        self.operations.len()
    }
}
