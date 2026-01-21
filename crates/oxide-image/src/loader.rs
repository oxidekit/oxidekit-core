//! Image loading from various sources.
//!
//! Provides async image loading from:
//! - File paths
//! - URLs (HTTP/HTTPS)
//! - Raw bytes
//! - Base64 encoded strings
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_image::loader::{ImageLoader, ImageSource};
//!
//! let loader = ImageLoader::builder()
//!     .timeout(Duration::from_secs(30))
//!     .max_size(10 * 1024 * 1024)
//!     .build();
//!
//! // Load from URL
//! let image = loader.load("https://example.com/photo.jpg").await?;
//!
//! // Load from file
//! let image = loader.load_file("/path/to/image.png").await?;
//!
//! // Load from bytes
//! let image = loader.load_bytes(&bytes).await?;
//! ```

use crate::error::{ImageError, ImageResult};
use crate::formats::ImageFormat;
use async_trait::async_trait;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, instrument, warn};
use url::Url;

/// Default timeout for HTTP requests.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default maximum image size (50 MB).
const DEFAULT_MAX_SIZE: usize = 50 * 1024 * 1024;

/// Represents the source of an image.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageSource {
    /// Load from a file path.
    File(PathBuf),
    /// Load from a URL.
    Url(String),
    /// Load from raw bytes (stored as base64 for serialization).
    Bytes(Vec<u8>),
    /// Load from a base64 encoded string.
    Base64(String),
    /// Asset bundled with the application.
    Asset(String),
}

impl ImageSource {
    /// Create a file source.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    /// Create a URL source.
    pub fn url(url: impl Into<String>) -> Self {
        Self::Url(url.into())
    }

    /// Create a bytes source.
    pub fn bytes(data: impl Into<Vec<u8>>) -> Self {
        Self::Bytes(data.into())
    }

    /// Create a base64 source.
    pub fn base64(data: impl Into<String>) -> Self {
        Self::Base64(data.into())
    }

    /// Create an asset source.
    pub fn asset(name: impl Into<String>) -> Self {
        Self::Asset(name.into())
    }

    /// Generate a cache key for this source.
    pub fn cache_key(&self) -> String {
        let mut hasher = Sha256::new();
        match self {
            Self::File(path) => {
                hasher.update(b"file:");
                hasher.update(path.to_string_lossy().as_bytes());
            }
            Self::Url(url) => {
                hasher.update(b"url:");
                hasher.update(url.as_bytes());
            }
            Self::Bytes(data) => {
                hasher.update(b"bytes:");
                hasher.update(data);
            }
            Self::Base64(data) => {
                hasher.update(b"base64:");
                hasher.update(data.as_bytes());
            }
            Self::Asset(name) => {
                hasher.update(b"asset:");
                hasher.update(name.as_bytes());
            }
        }
        hex::encode(hasher.finalize())
    }

    /// Check if this source is a URL.
    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    /// Check if this source is a file.
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Get the URL if this is a URL source.
    pub fn as_url(&self) -> Option<&str> {
        match self {
            Self::Url(url) => Some(url),
            _ => None,
        }
    }

    /// Get the file path if this is a file source.
    pub fn as_file(&self) -> Option<&Path> {
        match self {
            Self::File(path) => Some(path),
            _ => None,
        }
    }
}

impl From<&str> for ImageSource {
    fn from(s: &str) -> Self {
        if s.starts_with("http://") || s.starts_with("https://") {
            Self::Url(s.to_string())
        } else if s.starts_with("data:image/") {
            // Data URL
            if let Some(comma_pos) = s.find(',') {
                Self::Base64(s[comma_pos + 1..].to_string())
            } else {
                Self::Base64(s.to_string())
            }
        } else {
            Self::File(PathBuf::from(s))
        }
    }
}

impl From<String> for ImageSource {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<PathBuf> for ImageSource {
    fn from(path: PathBuf) -> Self {
        Self::File(path)
    }
}

impl From<&Path> for ImageSource {
    fn from(path: &Path) -> Self {
        Self::File(path.to_path_buf())
    }
}

/// Loaded image data.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Raw image bytes.
    pub bytes: Vec<u8>,
    /// Detected image format.
    pub format: ImageFormat,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// The source this image was loaded from.
    pub source: ImageSource,
    /// Size in bytes.
    pub size_bytes: usize,
}

impl ImageData {
    /// Create new image data.
    pub fn new(
        bytes: Vec<u8>,
        format: ImageFormat,
        width: u32,
        height: u32,
        source: ImageSource,
    ) -> Self {
        let size_bytes = bytes.len();
        Self {
            bytes,
            format,
            width,
            height,
            source,
            size_bytes,
        }
    }

    /// Get the aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    /// Get the cache key for this image.
    pub fn cache_key(&self) -> String {
        self.source.cache_key()
    }

    /// Check if this image is larger than the specified dimensions.
    pub fn is_larger_than(&self, width: u32, height: u32) -> bool {
        self.width > width || self.height > height
    }

    /// Estimate memory usage when decoded.
    pub fn estimated_memory_usage(&self) -> usize {
        // Assume 4 bytes per pixel (RGBA)
        (self.width as usize) * (self.height as usize) * 4
    }
}

/// Configuration for image loading.
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// HTTP request timeout.
    pub timeout: Duration,
    /// Maximum allowed image size in bytes.
    pub max_size: usize,
    /// User agent for HTTP requests.
    pub user_agent: String,
    /// Whether to follow redirects.
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow.
    pub max_redirects: usize,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            max_size: DEFAULT_MAX_SIZE,
            user_agent: format!("OxideKit-Image/{}", env!("CARGO_PKG_VERSION")),
            follow_redirects: true,
            max_redirects: 10,
        }
    }
}

/// Builder for ImageLoader.
#[derive(Debug, Clone)]
pub struct ImageLoaderBuilder {
    config: LoaderConfig,
}

impl ImageLoaderBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            config: LoaderConfig::default(),
        }
    }

    /// Set the HTTP request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the maximum allowed image size.
    pub fn max_size(mut self, size: usize) -> Self {
        self.config.max_size = size;
        self
    }

    /// Set the user agent for HTTP requests.
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.config.user_agent = agent.into();
        self
    }

    /// Set whether to follow redirects.
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.config.follow_redirects = follow;
        self
    }

    /// Set the maximum number of redirects.
    pub fn max_redirects(mut self, max: usize) -> Self {
        self.config.max_redirects = max;
        self
    }

    /// Build the ImageLoader.
    pub fn build(self) -> ImageResult<ImageLoader> {
        let client = Client::builder()
            .timeout(self.config.timeout)
            .user_agent(&self.config.user_agent)
            .redirect(if self.config.follow_redirects {
                reqwest::redirect::Policy::limited(self.config.max_redirects)
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()?;

        Ok(ImageLoader {
            config: self.config,
            client: Arc::new(client),
        })
    }
}

impl Default for ImageLoaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Async image loader.
///
/// Loads images from various sources with configurable timeouts,
/// size limits, and HTTP settings.
#[derive(Debug, Clone)]
pub struct ImageLoader {
    config: LoaderConfig,
    client: Arc<Client>,
}

impl ImageLoader {
    /// Create a new ImageLoader with default settings.
    pub fn new() -> ImageResult<Self> {
        ImageLoaderBuilder::new().build()
    }

    /// Create a builder for custom configuration.
    pub fn builder() -> ImageLoaderBuilder {
        ImageLoaderBuilder::new()
    }

    /// Load an image from any supported source.
    #[instrument(skip(self, source))]
    pub async fn load(&self, source: impl Into<ImageSource>) -> ImageResult<ImageData> {
        let source = source.into();
        debug!(?source, "Loading image");

        match source {
            ImageSource::File(path) => {
                let source_clone = ImageSource::File(path.clone());
                self.load_file_internal(&path, source_clone).await
            }
            ImageSource::Url(url) => {
                let source_clone = ImageSource::Url(url.clone());
                self.load_url_internal(&url, source_clone).await
            }
            ImageSource::Bytes(data) => {
                let source_clone = ImageSource::Bytes(data.clone());
                self.load_bytes_internal(&data, source_clone).await
            }
            ImageSource::Base64(data) => {
                let source_clone = ImageSource::Base64(data.clone());
                self.load_base64_internal(&data, source_clone).await
            }
            ImageSource::Asset(name) => {
                let source_clone = ImageSource::Asset(name.clone());
                self.load_asset_internal(&name, source_clone).await
            }
        }
    }

    /// Load an image from a file path.
    #[instrument(skip(self, path))]
    pub async fn load_file(&self, path: impl AsRef<Path>) -> ImageResult<ImageData> {
        let path = path.as_ref();
        let source = ImageSource::File(path.to_path_buf());
        self.load_file_internal(path, source).await
    }

    /// Load an image from a URL.
    #[instrument(skip(self))]
    pub async fn load_url(&self, url: &str) -> ImageResult<ImageData> {
        let source = ImageSource::Url(url.to_string());
        self.load_url_internal(url, source).await
    }

    /// Load an image from raw bytes.
    #[instrument(skip(self, data))]
    pub async fn load_bytes(&self, data: &[u8]) -> ImageResult<ImageData> {
        let source = ImageSource::Bytes(data.to_vec());
        self.load_bytes_internal(data, source).await
    }

    /// Load an image from a base64 encoded string.
    #[instrument(skip(self))]
    pub async fn load_base64(&self, data: &str) -> ImageResult<ImageData> {
        let source = ImageSource::Base64(data.to_string());
        self.load_base64_internal(data, source).await
    }

    async fn load_file_internal(&self, path: &Path, source: ImageSource) -> ImageResult<ImageData> {
        let mut file = fs::File::open(path).await.map_err(|e| ImageError::FileLoad {
            path: path.to_path_buf(),
            source: e,
        })?;

        let metadata = file.metadata().await.map_err(|e| ImageError::FileLoad {
            path: path.to_path_buf(),
            source: e,
        })?;

        let size = metadata.len() as usize;
        if size > self.config.max_size {
            return Err(ImageError::MemoryLimitExceeded {
                requested: size,
                limit: self.config.max_size,
            });
        }

        let mut bytes = Vec::with_capacity(size);
        file.read_to_end(&mut bytes)
            .await
            .map_err(|e| ImageError::FileLoad {
                path: path.to_path_buf(),
                source: e,
            })?;

        self.decode_image(bytes, source).await
    }

    async fn load_url_internal(&self, url: &str, source: ImageSource) -> ImageResult<ImageData> {
        // Validate URL
        let _parsed = Url::parse(url)?;

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(ImageError::url_load(
                url,
                format!("HTTP {}", response.status()),
            ));
        }

        // Check content length if available
        if let Some(len) = response.content_length() {
            if len as usize > self.config.max_size {
                return Err(ImageError::MemoryLimitExceeded {
                    requested: len as usize,
                    limit: self.config.max_size,
                });
            }
        }

        let bytes = response.bytes().await?.to_vec();

        if bytes.len() > self.config.max_size {
            return Err(ImageError::MemoryLimitExceeded {
                requested: bytes.len(),
                limit: self.config.max_size,
            });
        }

        self.decode_image(bytes, source).await
    }

    async fn load_bytes_internal(
        &self,
        data: &[u8],
        source: ImageSource,
    ) -> ImageResult<ImageData> {
        if data.is_empty() {
            return Err(ImageError::EmptyData);
        }

        if data.len() > self.config.max_size {
            return Err(ImageError::MemoryLimitExceeded {
                requested: data.len(),
                limit: self.config.max_size,
            });
        }

        self.decode_image(data.to_vec(), source).await
    }

    async fn load_base64_internal(
        &self,
        data: &str,
        source: ImageSource,
    ) -> ImageResult<ImageData> {
        let bytes = base64::engine::general_purpose::STANDARD.decode(data)?;
        self.load_bytes_internal(&bytes, source).await
    }

    async fn load_asset_internal(
        &self,
        name: &str,
        source: ImageSource,
    ) -> ImageResult<ImageData> {
        // Assets are typically bundled with the application
        // For now, we treat them as file paths relative to an assets directory
        // This can be customized based on the application's asset bundling strategy
        let path = PathBuf::from("assets").join(name);
        if path.exists() {
            self.load_file_internal(&path, source).await
        } else {
            Err(ImageError::FileLoad {
                path,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "Asset not found"),
            })
        }
    }

    async fn decode_image(&self, bytes: Vec<u8>, source: ImageSource) -> ImageResult<ImageData> {
        // Detect format from magic bytes
        let format = ImageFormat::from_magic_bytes(&bytes);
        if format == ImageFormat::Unknown {
            return Err(ImageError::decode("Unable to detect image format"));
        }

        // Decode to get dimensions
        let (width, height) = Self::get_dimensions(&bytes, format)?;

        Ok(ImageData::new(bytes, format, width, height, source))
    }

    fn get_dimensions(bytes: &[u8], format: ImageFormat) -> ImageResult<(u32, u32)> {
        match format {
            ImageFormat::Svg => {
                // SVG dimensions need special handling
                // For now, return a default size that can be overridden
                Ok((0, 0))
            }
            _ => {
                // Use the image crate for raster formats
                let reader = image::io::Reader::new(std::io::Cursor::new(bytes))
                    .with_guessed_format()
                    .map_err(|e| ImageError::decode(e.to_string()))?;

                let dimensions = reader
                    .into_dimensions()
                    .map_err(|e| ImageError::decode(e.to_string()))?;

                Ok(dimensions)
            }
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &LoaderConfig {
        &self.config
    }
}

impl Default for ImageLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create default ImageLoader")
    }
}

/// Trait for custom image source loaders.
#[async_trait]
pub trait CustomLoader: Send + Sync {
    /// Load image data from a custom source.
    async fn load(&self, source: &str) -> ImageResult<Vec<u8>>;

    /// Check if this loader can handle the given source.
    fn can_handle(&self, source: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_source_from_string() {
        let source: ImageSource = "https://example.com/image.jpg".into();
        assert!(matches!(source, ImageSource::Url(_)));

        let source: ImageSource = "/path/to/image.png".into();
        assert!(matches!(source, ImageSource::File(_)));

        let source: ImageSource = "data:image/png;base64,ABC123".into();
        assert!(matches!(source, ImageSource::Base64(s) if s == "ABC123"));
    }

    #[test]
    fn test_image_source_cache_key() {
        let source1 = ImageSource::url("https://example.com/image.jpg");
        let source2 = ImageSource::url("https://example.com/image.jpg");
        let source3 = ImageSource::url("https://example.com/other.jpg");

        assert_eq!(source1.cache_key(), source2.cache_key());
        assert_ne!(source1.cache_key(), source3.cache_key());
    }

    #[test]
    fn test_image_source_is_methods() {
        let url = ImageSource::url("https://example.com/image.jpg");
        assert!(url.is_url());
        assert!(!url.is_file());
        assert_eq!(url.as_url(), Some("https://example.com/image.jpg"));
        assert_eq!(url.as_file(), None);

        let file = ImageSource::file("/path/to/image.png");
        assert!(!file.is_url());
        assert!(file.is_file());
        assert_eq!(file.as_url(), None);
        assert_eq!(file.as_file(), Some(Path::new("/path/to/image.png")));
    }

    #[test]
    fn test_image_data_aspect_ratio() {
        let data = ImageData::new(
            vec![],
            ImageFormat::Png,
            1920,
            1080,
            ImageSource::url("test"),
        );
        assert!((data.aspect_ratio() - 16.0 / 9.0).abs() < 0.01);

        let data = ImageData::new(vec![], ImageFormat::Png, 100, 0, ImageSource::url("test"));
        assert!((data.aspect_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_image_data_is_larger_than() {
        let data = ImageData::new(
            vec![],
            ImageFormat::Png,
            1920,
            1080,
            ImageSource::url("test"),
        );
        assert!(data.is_larger_than(1000, 1000));
        assert!(!data.is_larger_than(2000, 2000));
    }

    #[test]
    fn test_image_data_estimated_memory() {
        let data = ImageData::new(
            vec![],
            ImageFormat::Png,
            1000,
            1000,
            ImageSource::url("test"),
        );
        assert_eq!(data.estimated_memory_usage(), 4_000_000);
    }

    #[test]
    fn test_loader_builder() {
        let loader = ImageLoader::builder()
            .timeout(Duration::from_secs(60))
            .max_size(100 * 1024 * 1024)
            .user_agent("Test/1.0")
            .follow_redirects(false)
            .max_redirects(5)
            .build()
            .unwrap();

        assert_eq!(loader.config.timeout, Duration::from_secs(60));
        assert_eq!(loader.config.max_size, 100 * 1024 * 1024);
        assert_eq!(loader.config.user_agent, "Test/1.0");
        assert!(!loader.config.follow_redirects);
        assert_eq!(loader.config.max_redirects, 5);
    }

    #[tokio::test]
    async fn test_load_bytes_empty() {
        let loader = ImageLoader::new().unwrap();
        let result = loader.load_bytes(&[]).await;
        assert!(matches!(result, Err(ImageError::EmptyData)));
    }

    #[tokio::test]
    async fn test_load_bytes_too_large() {
        let loader = ImageLoader::builder()
            .max_size(100)
            .build()
            .unwrap();

        let data = vec![0u8; 200];
        let result = loader.load_bytes(&data).await;
        assert!(matches!(result, Err(ImageError::MemoryLimitExceeded { .. })));
    }

    #[test]
    fn test_loader_config_default() {
        let config = LoaderConfig::default();
        assert_eq!(config.timeout, DEFAULT_TIMEOUT);
        assert_eq!(config.max_size, DEFAULT_MAX_SIZE);
        assert!(config.follow_redirects);
        assert_eq!(config.max_redirects, 10);
    }
}
