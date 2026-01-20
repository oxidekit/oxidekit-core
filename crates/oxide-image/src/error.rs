//! Error types for the image system.
//!
//! Provides a comprehensive error type that covers all failure modes
//! in image loading, caching, and transformation operations.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for image operations.
pub type ImageResult<T> = Result<T, ImageError>;

/// Errors that can occur during image operations.
#[derive(Error, Debug)]
pub enum ImageError {
    /// Failed to load image from file.
    #[error("Failed to load image from file '{path}': {source}")]
    FileLoad {
        /// The path that failed to load.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to load image from URL.
    #[error("Failed to load image from URL '{url}': {message}")]
    UrlLoad {
        /// The URL that failed to load.
        url: String,
        /// Error message.
        message: String,
    },

    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    /// Invalid URL format.
    #[error("Invalid URL format: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// Failed to decode image data.
    #[error("Failed to decode image: {0}")]
    Decode(String),

    /// Unsupported image format.
    #[error("Unsupported image format: {format}")]
    UnsupportedFormat {
        /// The unsupported format.
        format: String,
    },

    /// Invalid base64 encoding.
    #[error("Invalid base64 encoding: {0}")]
    InvalidBase64(#[from] base64::DecodeError),

    /// Cache operation failed.
    #[error("Cache operation failed: {0}")]
    Cache(String),

    /// Disk cache I/O error.
    #[error("Disk cache I/O error: {0}")]
    DiskCache(#[from] std::io::Error),

    /// Transform operation failed.
    #[error("Transform operation failed: {0}")]
    Transform(String),

    /// Image dimensions invalid.
    #[error("Invalid image dimensions: {width}x{height}")]
    InvalidDimensions {
        /// Width.
        width: u32,
        /// Height.
        height: u32,
    },

    /// Operation timed out.
    #[error("Operation timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// Image not found in cache.
    #[error("Image not found in cache: {key}")]
    NotInCache {
        /// The cache key.
        key: String,
    },

    /// SVG rendering error (when svg feature is enabled).
    #[error("SVG rendering error: {0}")]
    SvgRender(String),

    /// Memory limit exceeded.
    #[error("Memory limit exceeded: requested {requested} bytes, limit is {limit} bytes")]
    MemoryLimitExceeded {
        /// Requested memory.
        requested: usize,
        /// Memory limit.
        limit: usize,
    },

    /// Invalid image source.
    #[error("Invalid image source: {0}")]
    InvalidSource(String),

    /// Operation cancelled.
    #[error("Operation cancelled")]
    Cancelled,

    /// Image data is empty.
    #[error("Image data is empty")]
    EmptyData,

    /// Failed to serialize/deserialize cache metadata.
    #[error("Cache metadata error: {0}")]
    CacheMetadata(String),
}

impl ImageError {
    /// Create a decode error.
    pub fn decode(msg: impl Into<String>) -> Self {
        Self::Decode(msg.into())
    }

    /// Create an unsupported format error.
    pub fn unsupported_format(format: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
        }
    }

    /// Create a cache error.
    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }

    /// Create a transform error.
    pub fn transform(msg: impl Into<String>) -> Self {
        Self::Transform(msg.into())
    }

    /// Create an SVG render error.
    pub fn svg_render(msg: impl Into<String>) -> Self {
        Self::SvgRender(msg.into())
    }

    /// Create a URL load error.
    pub fn url_load(url: impl Into<String>, message: impl Into<String>) -> Self {
        Self::UrlLoad {
            url: url.into(),
            message: message.into(),
        }
    }

    /// Check if this error is retriable.
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::HttpRequest(_) | Self::Timeout(_) | Self::UrlLoad { .. }
        )
    }

    /// Check if this is a not-found error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotInCache { .. } | Self::FileLoad { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ImageError::decode("corrupt header");
        assert_eq!(err.to_string(), "Failed to decode image: corrupt header");

        let err = ImageError::unsupported_format("bmp");
        assert_eq!(err.to_string(), "Unsupported image format: bmp");

        let err = ImageError::InvalidDimensions {
            width: 0,
            height: 100,
        };
        assert_eq!(err.to_string(), "Invalid image dimensions: 0x100");
    }

    #[test]
    fn test_error_is_retriable() {
        let err = ImageError::Timeout(std::time::Duration::from_secs(30));
        assert!(err.is_retriable());

        let err = ImageError::decode("corrupt");
        assert!(!err.is_retriable());

        let err = ImageError::url_load("http://example.com", "connection refused");
        assert!(err.is_retriable());
    }

    #[test]
    fn test_error_is_not_found() {
        let err = ImageError::NotInCache {
            key: "abc123".into(),
        };
        assert!(err.is_not_found());

        let err = ImageError::decode("corrupt");
        assert!(!err.is_not_found());
    }

    #[test]
    fn test_error_constructors() {
        let err = ImageError::cache("disk full");
        assert!(matches!(err, ImageError::Cache(_)));

        let err = ImageError::transform("invalid blur radius");
        assert!(matches!(err, ImageError::Transform(_)));

        let err = ImageError::svg_render("invalid path");
        assert!(matches!(err, ImageError::SvgRender(_)));
    }
}
