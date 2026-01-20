//! Property Interpolation
//!
//! Provides interpolation for various types used in animations.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Trait for types that can be interpolated between two values
pub trait Interpolatable: Clone + Debug + Send + Sync {
    /// Interpolate between self and other by factor t (0.0 = self, 1.0 = other)
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

// Implement for primitive numeric types
impl Interpolatable for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolatable for f64 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

impl Interpolatable for i32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + ((other - self) as f32 * t) as i32
    }
}

impl Interpolatable for u32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let diff = *other as f32 - *self as f32;
        (*self as f32 + diff * t).max(0.0) as u32
    }
}

impl Interpolatable for u8 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let diff = *other as f32 - *self as f32;
        (*self as f32 + diff * t).clamp(0.0, 255.0) as u8
    }
}

// RGBA color as [f32; 4]
impl Interpolatable for [f32; 4] {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        [
            self[0].interpolate(&other[0], t),
            self[1].interpolate(&other[1], t),
            self[2].interpolate(&other[2], t),
            self[3].interpolate(&other[3], t),
        ]
    }
}

// RGB color as [f32; 3]
impl Interpolatable for [f32; 3] {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        [
            self[0].interpolate(&other[0], t),
            self[1].interpolate(&other[1], t),
            self[2].interpolate(&other[2], t),
        ]
    }
}

// 2D point/size as [f32; 2]
impl Interpolatable for [f32; 2] {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        [
            self[0].interpolate(&other[0], t),
            self[1].interpolate(&other[1], t),
        ]
    }
}

// 2D vector
impl Interpolatable for (f32, f32) {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        (
            self.0.interpolate(&other.0, t),
            self.1.interpolate(&other.1, t),
        )
    }
}

// 4-tuple (for rect: x, y, w, h or for color)
impl Interpolatable for (f32, f32, f32, f32) {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        (
            self.0.interpolate(&other.0, t),
            self.1.interpolate(&other.1, t),
            self.2.interpolate(&other.2, t),
            self.3.interpolate(&other.3, t),
        )
    }
}

/// RGBA color with u8 components (0-255)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color8 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn to_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn from_f32_array(arr: [f32; 4]) -> Self {
        Self {
            r: (arr[0] * 255.0).clamp(0.0, 255.0) as u8,
            g: (arr[1] * 255.0).clamp(0.0, 255.0) as u8,
            b: (arr[2] * 255.0).clamp(0.0, 255.0) as u8,
            a: (arr[3] * 255.0).clamp(0.0, 255.0) as u8,
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::new(r, g, b, a))
            }
            _ => None,
        }
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Interpolatable for Color8 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        // Interpolate in linear color space for better results
        let self_f32 = self.to_f32_array();
        let other_f32 = other.to_f32_array();
        let result = self_f32.interpolate(&other_f32, t);
        Self::from_f32_array(result)
    }
}

impl Default for Color8 {
    fn default() -> Self {
        Self::rgb(0, 0, 0)
    }
}

/// 2D point
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Self = Self::new(0.0, 0.0);
}

impl Interpolatable for Point2D {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x.interpolate(&other.x, t),
            y: self.y.interpolate(&other.y, t),
        }
    }
}

/// 2D size
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Size2D {
    pub width: f32,
    pub height: f32,
}

impl Size2D {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const ZERO: Self = Self::new(0.0, 0.0);
}

impl Interpolatable for Size2D {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            width: self.width.interpolate(&other.width, t),
            height: self.height.interpolate(&other.height, t),
        }
    }
}

/// Rectangle (position + size)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Rect2D {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect2D {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn position(&self) -> Point2D {
        Point2D::new(self.x, self.y)
    }

    pub fn size(&self) -> Size2D {
        Size2D::new(self.width, self.height)
    }
}

impl Interpolatable for Rect2D {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x.interpolate(&other.x, t),
            y: self.y.interpolate(&other.y, t),
            width: self.width.interpolate(&other.width, t),
            height: self.height.interpolate(&other.height, t),
        }
    }
}

/// Transform (translation, rotation, scale)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    pub translate_x: f32,
    pub translate_y: f32,
    pub rotate: f32, // radians
    pub scale_x: f32,
    pub scale_y: f32,
}

impl Transform2D {
    pub const fn identity() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            rotate: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            translate_x: x,
            translate_y: y,
            ..Self::identity()
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            scale_x: sx,
            scale_y: sy,
            ..Self::identity()
        }
    }

    pub fn rotate(angle: f32) -> Self {
        Self {
            rotate: angle,
            ..Self::identity()
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Interpolatable for Transform2D {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            translate_x: self.translate_x.interpolate(&other.translate_x, t),
            translate_y: self.translate_y.interpolate(&other.translate_y, t),
            rotate: self.rotate.interpolate(&other.rotate, t),
            scale_x: self.scale_x.interpolate(&other.scale_x, t),
            scale_y: self.scale_y.interpolate(&other.scale_y, t),
        }
    }
}

/// A type-erased animatable value for dynamic animations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AnimatableValue {
    /// Single float value (opacity, scale, rotation, etc.)
    Float(f32),
    /// 2D point/vector (position, offset)
    Point(Point2D),
    /// 2D size (width, height)
    Size(Size2D),
    /// Rectangle
    Rect(Rect2D),
    /// Color (RGBA)
    Color([f32; 4]),
    /// Color as u8 components
    Color8(Color8),
    /// Transform
    Transform(Transform2D),
    /// Corner radius (single value)
    Radius(f32),
    /// Corner radii (top-left, top-right, bottom-right, bottom-left)
    Radii([f32; 4]),
    /// Boolean (for discrete animations)
    Bool(bool),
    /// Integer (for discrete animations)
    Int(i32),
}

impl AnimatableValue {
    /// Interpolate between two values of the same type
    pub fn interpolate(&self, other: &Self, t: f32) -> Option<Self> {
        match (self, other) {
            (AnimatableValue::Float(a), AnimatableValue::Float(b)) => {
                Some(AnimatableValue::Float(a.interpolate(b, t)))
            }
            (AnimatableValue::Point(a), AnimatableValue::Point(b)) => {
                Some(AnimatableValue::Point(a.interpolate(b, t)))
            }
            (AnimatableValue::Size(a), AnimatableValue::Size(b)) => {
                Some(AnimatableValue::Size(a.interpolate(b, t)))
            }
            (AnimatableValue::Rect(a), AnimatableValue::Rect(b)) => {
                Some(AnimatableValue::Rect(a.interpolate(b, t)))
            }
            (AnimatableValue::Color(a), AnimatableValue::Color(b)) => {
                Some(AnimatableValue::Color(a.interpolate(b, t)))
            }
            (AnimatableValue::Color8(a), AnimatableValue::Color8(b)) => {
                Some(AnimatableValue::Color8(a.interpolate(b, t)))
            }
            (AnimatableValue::Transform(a), AnimatableValue::Transform(b)) => {
                Some(AnimatableValue::Transform(a.interpolate(b, t)))
            }
            (AnimatableValue::Radius(a), AnimatableValue::Radius(b)) => {
                Some(AnimatableValue::Radius(a.interpolate(b, t)))
            }
            (AnimatableValue::Radii(a), AnimatableValue::Radii(b)) => {
                Some(AnimatableValue::Radii(a.interpolate(b, t)))
            }
            // Discrete values snap at t >= 0.5
            (AnimatableValue::Bool(a), AnimatableValue::Bool(b)) => {
                Some(AnimatableValue::Bool(if t < 0.5 { *a } else { *b }))
            }
            (AnimatableValue::Int(a), AnimatableValue::Int(b)) => {
                Some(AnimatableValue::Int(a.interpolate(b, t)))
            }
            // Mismatched types cannot be interpolated
            _ => None,
        }
    }

    /// Get the value as f32 if it's a float
    pub fn as_float(&self) -> Option<f32> {
        match self {
            AnimatableValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as Color if it's a color
    pub fn as_color(&self) -> Option<[f32; 4]> {
        match self {
            AnimatableValue::Color(v) => Some(*v),
            AnimatableValue::Color8(v) => Some(v.to_f32_array()),
            _ => None,
        }
    }

    /// Get the value as Point2D if it's a point
    pub fn as_point(&self) -> Option<Point2D> {
        match self {
            AnimatableValue::Point(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as Size2D if it's a size
    pub fn as_size(&self) -> Option<Size2D> {
        match self {
            AnimatableValue::Size(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as Rect2D if it's a rect
    pub fn as_rect(&self) -> Option<Rect2D> {
        match self {
            AnimatableValue::Rect(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as Transform2D if it's a transform
    pub fn as_transform(&self) -> Option<Transform2D> {
        match self {
            AnimatableValue::Transform(v) => Some(*v),
            _ => None,
        }
    }
}

impl From<f32> for AnimatableValue {
    fn from(v: f32) -> Self {
        AnimatableValue::Float(v)
    }
}

impl From<[f32; 4]> for AnimatableValue {
    fn from(v: [f32; 4]) -> Self {
        AnimatableValue::Color(v)
    }
}

impl From<Color8> for AnimatableValue {
    fn from(v: Color8) -> Self {
        AnimatableValue::Color8(v)
    }
}

impl From<Point2D> for AnimatableValue {
    fn from(v: Point2D) -> Self {
        AnimatableValue::Point(v)
    }
}

impl From<Size2D> for AnimatableValue {
    fn from(v: Size2D) -> Self {
        AnimatableValue::Size(v)
    }
}

impl From<Rect2D> for AnimatableValue {
    fn from(v: Rect2D) -> Self {
        AnimatableValue::Rect(v)
    }
}

impl From<Transform2D> for AnimatableValue {
    fn from(v: Transform2D) -> Self {
        AnimatableValue::Transform(v)
    }
}

impl From<bool> for AnimatableValue {
    fn from(v: bool) -> Self {
        AnimatableValue::Bool(v)
    }
}

impl From<i32> for AnimatableValue {
    fn from(v: i32) -> Self {
        AnimatableValue::Int(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_interpolate() {
        let a = 0.0_f32;
        let b = 100.0_f32;
        assert!((a.interpolate(&b, 0.0) - 0.0).abs() < 0.001);
        assert!((a.interpolate(&b, 0.5) - 50.0).abs() < 0.001);
        assert!((a.interpolate(&b, 1.0) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_color_interpolate() {
        let a = [0.0_f32, 0.0, 0.0, 1.0];
        let b = [1.0_f32, 1.0, 1.0, 1.0];
        let mid = a.interpolate(&b, 0.5);
        assert!((mid[0] - 0.5).abs() < 0.001);
        assert!((mid[1] - 0.5).abs() < 0.001);
        assert!((mid[2] - 0.5).abs() < 0.001);
        assert!((mid[3] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_color8_interpolate() {
        let a = Color8::rgb(0, 0, 0);
        let b = Color8::rgb(255, 255, 255);
        let mid = a.interpolate(&b, 0.5);
        assert_eq!(mid.r, 127);
        assert_eq!(mid.g, 127);
        assert_eq!(mid.b, 127);
    }

    #[test]
    fn test_point_interpolate() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(100.0, 200.0);
        let mid = a.interpolate(&b, 0.5);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!((mid.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_interpolate() {
        let a = Transform2D::identity();
        let b = Transform2D {
            translate_x: 100.0,
            translate_y: 200.0,
            rotate: std::f32::consts::PI,
            scale_x: 2.0,
            scale_y: 2.0,
        };
        let mid = a.interpolate(&b, 0.5);
        assert!((mid.translate_x - 50.0).abs() < 0.001);
        assert!((mid.translate_y - 100.0).abs() < 0.001);
        assert!((mid.scale_x - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_animatable_value() {
        let a = AnimatableValue::Float(0.0);
        let b = AnimatableValue::Float(100.0);
        let mid = a.interpolate(&b, 0.5).unwrap();
        assert!((mid.as_float().unwrap() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_animatable_value_type_mismatch() {
        let a = AnimatableValue::Float(0.0);
        let b = AnimatableValue::Bool(true);
        assert!(a.interpolate(&b, 0.5).is_none());
    }

    #[test]
    fn test_color8_hex() {
        let c = Color8::from_hex("#FF5500").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 85);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);

        let hex = c.to_hex();
        assert_eq!(hex, "#FF5500");
    }
}
