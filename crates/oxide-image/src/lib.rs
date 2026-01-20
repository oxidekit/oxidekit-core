//! # OxideKit Image System
//!
//! A comprehensive image loading, caching, and display library for OxideKit applications.
//!
//! ## Features
//!
//! - **Image Loading**: Load images from files, URLs, bytes, or base64
//! - **Caching**: LRU memory cache and persistent disk cache
//! - **Transformations**: Resize, blur, grayscale, rounded corners, and more
//! - **Components**: Ready-to-use Image, Avatar, and Gallery components
//! - **Lazy Loading**: Load images on visibility with LQIP placeholders
//! - **Optimization**: Resolution selection and memory-efficient loading
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use oxide_image::prelude::*;
//!
//! // Basic image loading
//! let loader = ImageLoader::new()
//!     .cache(CachePolicy::MemoryAndDisk)
//!     .build();
//!
//! let image = loader.load("https://example.com/photo.jpg").await?;
//!
//! // Image component with placeholder
//! let img = Image::new("https://example.com/photo.jpg")
//!     .placeholder(Placeholder::Shimmer)
//!     .error_fallback(Image::asset("error.png"))
//!     .fit(ImageFit::Cover);
//!
//! // Avatar with fallback initials
//! let avatar = Avatar::new()
//!     .image("https://example.com/avatar.jpg")
//!     .fallback_initials("JD")
//!     .size(AvatarSize::Large)
//!     .status(Status::Online);
//! ```
//!
//! ## Supported Formats
//!
//! - PNG (default)
//! - JPEG (default)
//! - WebP (default)
//! - GIF (optional: `gif` feature)
//! - SVG (optional: `svg` feature)
//! - ICO (optional: `ico` feature)
//!
//! ## Caching
//!
//! The caching system provides both memory and disk caching:
//!
//! ```rust,ignore
//! use oxide_image::cache::{ImageCache, CacheConfig};
//!
//! // Create cache with custom settings
//! let cache = ImageCache::builder()
//!     .memory_size_limit(100 * 1024 * 1024) // 100 MB
//!     .disk_size_limit(500 * 1024 * 1024)   // 500 MB
//!     .disk_path("/custom/cache/path")
//!     .build()?;
//!
//! // Preload images
//! cache.preload(&["url1", "url2", "url3"]).await;
//!
//! // Invalidate cache
//! cache.invalidate("https://example.com/old.jpg").await;
//! cache.clear_all().await;
//! ```
//!
//! ## Transformations
//!
//! Apply transformations to images:
//!
//! ```rust,ignore
//! use oxide_image::transform::{Transform, ResizeMode};
//!
//! let transformed = image
//!     .resize(200, 200, ResizeMode::Cover)
//!     .blur(5.0)
//!     .grayscale()
//!     .rounded_corners(16.0)
//!     .border(2.0, Color::WHITE)
//!     .shadow(4.0, 4.0, 8.0, Color::rgba(0, 0, 0, 128));
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod avatar;
pub mod cache;
pub mod component;
pub mod error;
pub mod formats;
pub mod gallery;
pub mod lazy;
pub mod loader;
pub mod optimize;
pub mod transform;

// Re-exports
pub use avatar::{Avatar, AvatarSize, Status};
pub use cache::{CacheConfig, CachePolicy, ImageCache};
pub use component::{Image, ImageFit, ImageState, Placeholder};
pub use error::{ImageError, ImageResult};
pub use formats::ImageFormat;
pub use gallery::{Gallery, GalleryItem, GalleryLayout, Lightbox};
pub use lazy::{LazyImage, LazyLoadConfig, Visibility};
pub use loader::{ImageData, ImageLoader, ImageLoaderBuilder, ImageSource};
pub use optimize::{OptimizationConfig, ResolutionHint};
pub use transform::{Border, ResizeMode, Shadow, Transform, TransformPipeline};

/// Convenient re-exports for common usage patterns.
pub mod prelude {
    pub use crate::avatar::{Avatar, AvatarSize, Status};
    pub use crate::cache::{CacheConfig, CachePolicy, ImageCache};
    pub use crate::component::{Image, ImageFit, ImageState, Placeholder};
    pub use crate::error::{ImageError, ImageResult};
    pub use crate::formats::ImageFormat;
    pub use crate::gallery::{Gallery, GalleryItem, GalleryLayout, Lightbox};
    pub use crate::lazy::{LazyImage, LazyLoadConfig, Visibility};
    pub use crate::loader::{ImageData, ImageLoader, ImageLoaderBuilder, ImageSource};
    pub use crate::optimize::{OptimizationConfig, ResolutionHint};
    pub use crate::transform::{Border, ResizeMode, Shadow, Transform, TransformPipeline};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_exports() {
        // Verify all prelude items are accessible
        let _: fn() -> CachePolicy = || CachePolicy::MemoryOnly;
        let _: fn() -> ImageFit = || ImageFit::Cover;
        let _: fn() -> AvatarSize = || AvatarSize::Medium;
    }
}
