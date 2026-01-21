//! Image format detection and handling.

use serde::{Deserialize, Serialize};

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ImageFormat {
    /// PNG
    #[default]
    Png,
    /// JPEG
    Jpeg,
    /// WebP
    WebP,
    /// GIF
    Gif,
    /// SVG
    Svg,
    /// ICO
    Ico,
    /// BMP
    Bmp,
    /// Unknown format
    Unknown,
}

impl ImageFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" => ImageFormat::Png,
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "webp" => ImageFormat::WebP,
            "gif" => ImageFormat::Gif,
            "svg" => ImageFormat::Svg,
            "ico" => ImageFormat::Ico,
            "bmp" => ImageFormat::Bmp,
            _ => ImageFormat::Unknown,
        }
    }

    /// Detect format from MIME type
    pub fn from_mime(mime: &str) -> Self {
        match mime.to_lowercase().as_str() {
            "image/png" => ImageFormat::Png,
            "image/jpeg" => ImageFormat::Jpeg,
            "image/webp" => ImageFormat::WebP,
            "image/gif" => ImageFormat::Gif,
            "image/svg+xml" => ImageFormat::Svg,
            "image/x-icon" | "image/vnd.microsoft.icon" => ImageFormat::Ico,
            "image/bmp" => ImageFormat::Bmp,
            _ => ImageFormat::Unknown,
        }
    }

    /// Detect format from magic bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::from_magic_bytes(bytes)
    }

    /// Detect format from magic bytes (alias)
    pub fn from_magic_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 8 {
            return ImageFormat::Unknown;
        }

        // PNG signature
        if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return ImageFormat::Png;
        }

        // JPEG signature
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return ImageFormat::Jpeg;
        }

        // WebP signature
        if bytes.len() >= 12
            && bytes.starts_with(b"RIFF")
            && &bytes[8..12] == b"WEBP"
        {
            return ImageFormat::WebP;
        }

        // GIF signature
        if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            return ImageFormat::Gif;
        }

        // BMP signature
        if bytes.starts_with(b"BM") {
            return ImageFormat::Bmp;
        }

        // ICO signature
        if bytes.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
            return ImageFormat::Ico;
        }

        // SVG (check for XML-like content)
        if bytes.starts_with(b"<?xml") || bytes.starts_with(b"<svg") {
            return ImageFormat::Svg;
        }

        ImageFormat::Unknown
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Svg => "image/svg+xml",
            ImageFormat::Ico => "image/x-icon",
            ImageFormat::Bmp => "image/bmp",
            ImageFormat::Unknown => "application/octet-stream",
        }
    }

    /// Get common file extension
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::WebP => "webp",
            ImageFormat::Gif => "gif",
            ImageFormat::Svg => "svg",
            ImageFormat::Ico => "ico",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Unknown => "bin",
        }
    }

    /// Check if format supports transparency
    pub fn supports_transparency(&self) -> bool {
        matches!(
            self,
            ImageFormat::Png | ImageFormat::WebP | ImageFormat::Gif | ImageFormat::Svg | ImageFormat::Ico
        )
    }

    /// Check if format supports animation
    pub fn supports_animation(&self) -> bool {
        matches!(self, ImageFormat::Gif | ImageFormat::WebP | ImageFormat::Svg)
    }
}
