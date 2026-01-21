//! Image caching system with memory and disk caching.
//!
//! Provides a two-tier caching system:
//! - **Memory cache**: Fast LRU cache for frequently accessed images
//! - **Disk cache**: Persistent cache for offline access and app restarts
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_image::cache::{ImageCache, CacheConfig, CachePolicy};
//!
//! // Create cache with custom config
//! let cache = ImageCache::builder()
//!     .memory_size_limit(100 * 1024 * 1024) // 100 MB
//!     .disk_size_limit(500 * 1024 * 1024)   // 500 MB
//!     .disk_path("/custom/cache/path")
//!     .build()?;
//!
//! // Store an image
//! cache.store(&image_data).await?;
//!
//! // Retrieve from cache
//! if let Some(data) = cache.get("cache_key").await? {
//!     println!("Found in cache!");
//! }
//!
//! // Preload multiple images
//! cache.preload(&["url1", "url2", "url3"]).await;
//!
//! // Invalidate cache
//! cache.invalidate("cache_key").await?;
//! cache.clear_all().await?;
//! ```

use crate::error::{ImageError, ImageResult};
use crate::loader::{ImageData, ImageLoader, ImageSource};
use chrono::{DateTime, Utc};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Default memory cache size limit (100 MB).
const DEFAULT_MEMORY_SIZE_LIMIT: usize = 100 * 1024 * 1024;

/// Default disk cache size limit (500 MB).
const DEFAULT_DISK_SIZE_LIMIT: usize = 500 * 1024 * 1024;

/// Default maximum number of items in memory cache.
const DEFAULT_MEMORY_ITEM_LIMIT: usize = 100;

/// Cache policy defining where images should be cached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CachePolicy {
    /// Don't cache images.
    NoCache,
    /// Only cache in memory (fast but not persistent).
    MemoryOnly,
    /// Only cache on disk (persistent but slower).
    DiskOnly,
    /// Cache in both memory and disk (default).
    #[default]
    MemoryAndDisk,
}

impl CachePolicy {
    /// Check if memory caching is enabled.
    pub fn uses_memory(&self) -> bool {
        matches!(self, Self::MemoryOnly | Self::MemoryAndDisk)
    }

    /// Check if disk caching is enabled.
    pub fn uses_disk(&self) -> bool {
        matches!(self, Self::DiskOnly | Self::MemoryAndDisk)
    }
}

/// Cache entry metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cache key.
    pub key: String,
    /// Size in bytes.
    pub size: usize,
    /// When the entry was created.
    pub created_at: DateTime<Utc>,
    /// When the entry was last accessed.
    pub last_accessed: DateTime<Utc>,
    /// Number of times accessed.
    pub access_count: u64,
    /// Original image source.
    pub source: ImageSource,
    /// Image format.
    pub format: String,
    /// Image dimensions.
    pub width: u32,
    pub height: u32,
}

impl CacheEntry {
    /// Create a new cache entry from image data.
    pub fn from_image_data(data: &ImageData) -> Self {
        let now = Utc::now();
        Self {
            key: data.cache_key(),
            size: data.size_bytes,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            source: data.source.clone(),
            format: format!("{:?}", data.format),
            width: data.width,
            height: data.height,
        }
    }
}

/// Memory cache entry.
#[derive(Debug, Clone)]
struct MemoryCacheEntry {
    data: Arc<ImageData>,
    metadata: CacheEntry,
}

/// Configuration for the image cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum memory cache size in bytes.
    pub memory_size_limit: usize,
    /// Maximum disk cache size in bytes.
    pub disk_size_limit: usize,
    /// Maximum number of items in memory cache.
    pub memory_item_limit: usize,
    /// Path for disk cache storage.
    pub disk_path: Option<PathBuf>,
    /// Cache policy.
    pub policy: CachePolicy,
    /// Time-to-live for cache entries (None = never expire).
    pub ttl: Option<std::time::Duration>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memory_size_limit: DEFAULT_MEMORY_SIZE_LIMIT,
            disk_size_limit: DEFAULT_DISK_SIZE_LIMIT,
            memory_item_limit: DEFAULT_MEMORY_ITEM_LIMIT,
            disk_path: None,
            policy: CachePolicy::MemoryAndDisk,
            ttl: None,
        }
    }
}

/// Builder for ImageCache.
#[derive(Debug, Clone)]
pub struct CacheBuilder {
    config: CacheConfig,
}

impl CacheBuilder {
    /// Create a new cache builder.
    pub fn new() -> Self {
        Self {
            config: CacheConfig::default(),
        }
    }

    /// Set the memory size limit.
    pub fn memory_size_limit(mut self, limit: usize) -> Self {
        self.config.memory_size_limit = limit;
        self
    }

    /// Set the disk size limit.
    pub fn disk_size_limit(mut self, limit: usize) -> Self {
        self.config.disk_size_limit = limit;
        self
    }

    /// Set the memory item limit.
    pub fn memory_item_limit(mut self, limit: usize) -> Self {
        self.config.memory_item_limit = limit;
        self
    }

    /// Set the disk cache path.
    pub fn disk_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.disk_path = Some(path.into());
        self
    }

    /// Set the cache policy.
    pub fn policy(mut self, policy: CachePolicy) -> Self {
        self.config.policy = policy;
        self
    }

    /// Set the time-to-live for cache entries.
    pub fn ttl(mut self, ttl: std::time::Duration) -> Self {
        self.config.ttl = Some(ttl);
        self
    }

    /// Build the ImageCache.
    pub async fn build(self) -> ImageResult<ImageCache> {
        ImageCache::new(self.config).await
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Image cache with memory and disk caching.
pub struct ImageCache {
    config: CacheConfig,
    memory_cache: RwLock<LruCache<String, MemoryCacheEntry>>,
    current_memory_size: RwLock<usize>,
    disk_path: PathBuf,
    metadata_path: PathBuf,
    disk_metadata: RwLock<HashMap<String, CacheEntry>>,
    loader: Arc<ImageLoader>,
}

impl ImageCache {
    /// Create a new image cache with the given configuration.
    pub async fn new(config: CacheConfig) -> ImageResult<Self> {
        let disk_path = config.disk_path.clone().unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("oxide-image-cache")
        });

        // Ensure cache directory exists
        fs::create_dir_all(&disk_path).await?;

        let metadata_path = disk_path.join("metadata.json");

        // Load existing disk metadata
        let disk_metadata = if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).await.unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let loader = Arc::new(ImageLoader::new()?);

        let capacity = NonZeroUsize::new(config.memory_item_limit).unwrap_or(NonZeroUsize::MIN);

        Ok(Self {
            config,
            memory_cache: RwLock::new(LruCache::new(capacity)),
            current_memory_size: RwLock::new(0),
            disk_path,
            metadata_path,
            disk_metadata: RwLock::new(disk_metadata),
            loader,
        })
    }

    /// Create a builder for custom configuration.
    pub fn builder() -> CacheBuilder {
        CacheBuilder::new()
    }

    /// Get an image from the cache by key.
    #[instrument(skip(self))]
    pub async fn get(&self, key: &str) -> ImageResult<Option<Arc<ImageData>>> {
        // Check memory cache first
        if self.config.policy.uses_memory() {
            let mut cache = self.memory_cache.write().await;
            if let Some(entry) = cache.get_mut(key) {
                // Check TTL
                if let Some(ttl) = self.config.ttl {
                    let age = Utc::now()
                        .signed_duration_since(entry.metadata.created_at)
                        .to_std()
                        .unwrap_or(std::time::Duration::ZERO);
                    if age > ttl {
                        cache.pop(key);
                        debug!(key, "Memory cache entry expired");
                        // Fall through to disk cache
                    } else {
                        entry.metadata.last_accessed = Utc::now();
                        entry.metadata.access_count += 1;
                        debug!(key, "Memory cache hit");
                        return Ok(Some(entry.data.clone()));
                    }
                } else {
                    entry.metadata.last_accessed = Utc::now();
                    entry.metadata.access_count += 1;
                    debug!(key, "Memory cache hit");
                    return Ok(Some(entry.data.clone()));
                }
            }
        }

        // Check disk cache
        if self.config.policy.uses_disk() {
            if let Some(data) = self.get_from_disk(key).await? {
                // Promote to memory cache if enabled
                if self.config.policy.uses_memory() {
                    self.store_in_memory(&data).await?;
                }
                debug!(key, "Disk cache hit");
                return Ok(Some(Arc::new(data)));
            }
        }

        debug!(key, "Cache miss");
        Ok(None)
    }

    /// Store an image in the cache.
    #[instrument(skip(self, data))]
    pub async fn store(&self, data: &ImageData) -> ImageResult<()> {
        let key = data.cache_key();
        debug!(key, size = data.size_bytes, "Storing image in cache");

        if self.config.policy.uses_memory() {
            self.store_in_memory(data).await?;
        }

        if self.config.policy.uses_disk() {
            self.store_on_disk(data).await?;
        }

        Ok(())
    }

    /// Load an image from source, using cache if available.
    #[instrument(skip(self, source))]
    pub async fn load(&self, source: impl Into<ImageSource>) -> ImageResult<Arc<ImageData>> {
        let source = source.into();
        let key = source.cache_key();

        // Try cache first
        if let Some(data) = self.get(&key).await? {
            return Ok(data);
        }

        // Load from source
        let data = self.loader.load(source).await?;

        // Store in cache
        self.store(&data).await?;

        Ok(Arc::new(data))
    }

    /// Invalidate a cache entry.
    #[instrument(skip(self))]
    pub async fn invalidate(&self, key: &str) -> ImageResult<()> {
        info!(key, "Invalidating cache entry");

        // Remove from memory cache
        if self.config.policy.uses_memory() {
            let mut cache = self.memory_cache.write().await;
            if let Some(entry) = cache.pop(key) {
                let mut size = self.current_memory_size.write().await;
                *size = size.saturating_sub(entry.data.size_bytes);
            }
        }

        // Remove from disk cache
        if self.config.policy.uses_disk() {
            self.remove_from_disk(key).await?;
        }

        Ok(())
    }

    /// Clear all cache entries.
    #[instrument(skip(self))]
    pub async fn clear_all(&self) -> ImageResult<()> {
        info!("Clearing all cache entries");

        // Clear memory cache
        {
            let mut cache = self.memory_cache.write().await;
            cache.clear();
            let mut size = self.current_memory_size.write().await;
            *size = 0;
        }

        // Clear disk cache
        if self.config.policy.uses_disk() {
            // Remove all files in cache directory
            let mut entries = fs::read_dir(&self.disk_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path).await?;
                }
            }

            // Clear metadata
            let mut metadata = self.disk_metadata.write().await;
            metadata.clear();
            self.save_disk_metadata().await?;
        }

        Ok(())
    }

    /// Preload images into the cache.
    #[instrument(skip(self, sources))]
    pub async fn preload<S: Into<ImageSource> + Clone>(&self, sources: &[S]) -> Vec<ImageResult<()>> {
        let futures: Vec<_> = sources
            .iter()
            .map(|source| {
                let source = source.clone().into();
                async move {
                    self.load(source).await.map(|_| ())
                }
            })
            .collect();

        futures::future::join_all(futures).await
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let memory_cache = self.memory_cache.read().await;
        let current_memory_size = *self.current_memory_size.read().await;
        let disk_metadata = self.disk_metadata.read().await;

        let disk_size: usize = disk_metadata.values().map(|e| e.size).sum();

        CacheStats {
            memory_entries: memory_cache.len(),
            memory_size: current_memory_size,
            memory_size_limit: self.config.memory_size_limit,
            disk_entries: disk_metadata.len(),
            disk_size,
            disk_size_limit: self.config.disk_size_limit,
        }
    }

    /// Check if an entry exists in the cache.
    pub async fn contains(&self, key: &str) -> bool {
        // Check memory cache
        if self.config.policy.uses_memory() {
            let cache = self.memory_cache.read().await;
            if cache.contains(key) {
                return true;
            }
        }

        // Check disk cache
        if self.config.policy.uses_disk() {
            let metadata = self.disk_metadata.read().await;
            if metadata.contains_key(key) {
                return true;
            }
        }

        false
    }

    /// Get the configuration.
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    // Private methods

    async fn store_in_memory(&self, data: &ImageData) -> ImageResult<()> {
        let entry = MemoryCacheEntry {
            data: Arc::new(data.clone()),
            metadata: CacheEntry::from_image_data(data),
        };

        let mut cache = self.memory_cache.write().await;
        let mut current_size = self.current_memory_size.write().await;

        // Evict entries if needed
        while *current_size + data.size_bytes > self.config.memory_size_limit {
            if let Some((_, evicted)) = cache.pop_lru() {
                *current_size = current_size.saturating_sub(evicted.data.size_bytes);
                debug!(
                    key = evicted.metadata.key,
                    "Evicted from memory cache"
                );
            } else {
                break;
            }
        }

        // Check if single entry is too large
        if data.size_bytes > self.config.memory_size_limit {
            warn!(
                size = data.size_bytes,
                limit = self.config.memory_size_limit,
                "Image too large for memory cache"
            );
            return Ok(());
        }

        *current_size += data.size_bytes;
        cache.put(data.cache_key(), entry);

        Ok(())
    }

    async fn store_on_disk(&self, data: &ImageData) -> ImageResult<()> {
        let key = data.cache_key();
        let file_path = self.disk_path.join(&key);

        // Check disk size limit
        let metadata = self.disk_metadata.read().await;
        let current_size: usize = metadata.values().map(|e| e.size).sum();
        drop(metadata);

        if current_size + data.size_bytes > self.config.disk_size_limit {
            // Evict oldest entries
            self.evict_disk_entries(data.size_bytes).await?;
        }

        // Write image data
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(&data.bytes).await?;
        file.sync_all().await?;

        // Update metadata
        let entry = CacheEntry::from_image_data(data);
        let mut metadata = self.disk_metadata.write().await;
        metadata.insert(key, entry);
        drop(metadata);

        self.save_disk_metadata().await?;

        Ok(())
    }

    async fn get_from_disk(&self, key: &str) -> ImageResult<Option<ImageData>> {
        let metadata = self.disk_metadata.read().await;
        let entry = match metadata.get(key) {
            Some(e) => e.clone(),
            None => return Ok(None),
        };
        drop(metadata);

        // Check TTL
        if let Some(ttl) = self.config.ttl {
            let age = Utc::now()
                .signed_duration_since(entry.created_at)
                .to_std()
                .unwrap_or(std::time::Duration::ZERO);
            if age > ttl {
                debug!(key, "Disk cache entry expired");
                self.remove_from_disk(key).await?;
                return Ok(None);
            }
        }

        let file_path = self.disk_path.join(key);
        if !file_path.exists() {
            // Metadata exists but file doesn't - clean up
            self.remove_from_disk(key).await?;
            return Ok(None);
        }

        let mut file = fs::File::open(&file_path).await?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;

        // Update last accessed
        let mut metadata = self.disk_metadata.write().await;
        if let Some(e) = metadata.get_mut(key) {
            e.last_accessed = Utc::now();
            e.access_count += 1;
        }
        drop(metadata);

        // Parse format from metadata
        let format = crate::formats::ImageFormat::from_magic_bytes(&bytes);

        Ok(Some(ImageData::new(
            bytes,
            format,
            entry.width,
            entry.height,
            entry.source,
        )))
    }

    async fn remove_from_disk(&self, key: &str) -> ImageResult<()> {
        let file_path = self.disk_path.join(key);
        if file_path.exists() {
            fs::remove_file(file_path).await?;
        }

        let mut metadata = self.disk_metadata.write().await;
        metadata.remove(key);
        drop(metadata);

        self.save_disk_metadata().await?;

        Ok(())
    }

    async fn evict_disk_entries(&self, needed_bytes: usize) -> ImageResult<()> {
        let mut metadata = self.disk_metadata.write().await;
        let mut entries: Vec<_> = metadata.iter().collect();

        // Sort by last accessed (oldest first)
        entries.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));

        let mut freed = 0usize;
        let mut to_remove = Vec::new();

        for (key, entry) in entries {
            if freed >= needed_bytes {
                break;
            }
            to_remove.push(key.clone());
            freed += entry.size;
        }

        for key in &to_remove {
            metadata.remove(key);
            let file_path = self.disk_path.join(key);
            if file_path.exists() {
                let _ = fs::remove_file(file_path).await;
            }
        }

        drop(metadata);
        self.save_disk_metadata().await?;

        info!(freed_bytes = freed, "Evicted disk cache entries");

        Ok(())
    }

    async fn save_disk_metadata(&self) -> ImageResult<()> {
        let metadata = self.disk_metadata.read().await;
        let content = serde_json::to_string_pretty(&*metadata)
            .map_err(|e| ImageError::CacheMetadata(e.to_string()))?;
        drop(metadata);

        fs::write(&self.metadata_path, content).await?;

        Ok(())
    }
}

impl std::fmt::Debug for ImageCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageCache")
            .field("config", &self.config)
            .field("disk_path", &self.disk_path)
            .finish()
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in memory cache.
    pub memory_entries: usize,
    /// Current memory cache size in bytes.
    pub memory_size: usize,
    /// Memory cache size limit.
    pub memory_size_limit: usize,
    /// Number of entries in disk cache.
    pub disk_entries: usize,
    /// Current disk cache size in bytes.
    pub disk_size: usize,
    /// Disk cache size limit.
    pub disk_size_limit: usize,
}

impl CacheStats {
    /// Get memory usage as a percentage.
    pub fn memory_usage_percent(&self) -> f32 {
        if self.memory_size_limit == 0 {
            0.0
        } else {
            (self.memory_size as f32 / self.memory_size_limit as f32) * 100.0
        }
    }

    /// Get disk usage as a percentage.
    pub fn disk_usage_percent(&self) -> f32 {
        if self.disk_size_limit == 0 {
            0.0
        } else {
            (self.disk_size as f32 / self.disk_size_limit as f32) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::ImageFormat;

    fn create_test_image_data(key: &str, size: usize) -> ImageData {
        ImageData::new(
            vec![0u8; size],
            ImageFormat::Png,
            100,
            100,
            ImageSource::url(format!("https://example.com/{key}.png")),
        )
    }

    #[test]
    fn test_cache_policy() {
        assert!(CachePolicy::MemoryOnly.uses_memory());
        assert!(!CachePolicy::MemoryOnly.uses_disk());

        assert!(!CachePolicy::DiskOnly.uses_memory());
        assert!(CachePolicy::DiskOnly.uses_disk());

        assert!(CachePolicy::MemoryAndDisk.uses_memory());
        assert!(CachePolicy::MemoryAndDisk.uses_disk());

        assert!(!CachePolicy::NoCache.uses_memory());
        assert!(!CachePolicy::NoCache.uses_disk());
    }

    #[test]
    fn test_cache_entry_from_image_data() {
        let data = create_test_image_data("test", 1000);
        let entry = CacheEntry::from_image_data(&data);

        assert_eq!(entry.size, 1000);
        assert_eq!(entry.width, 100);
        assert_eq!(entry.height, 100);
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn test_cache_stats_percentages() {
        let stats = CacheStats {
            memory_entries: 10,
            memory_size: 50 * 1024 * 1024,
            memory_size_limit: 100 * 1024 * 1024,
            disk_entries: 20,
            disk_size: 250 * 1024 * 1024,
            disk_size_limit: 500 * 1024 * 1024,
        };

        assert!((stats.memory_usage_percent() - 50.0).abs() < 0.01);
        assert!((stats.disk_usage_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_cache_stats_zero_limit() {
        let stats = CacheStats {
            memory_entries: 0,
            memory_size: 0,
            memory_size_limit: 0,
            disk_entries: 0,
            disk_size: 0,
            disk_size_limit: 0,
        };

        assert_eq!(stats.memory_usage_percent(), 0.0);
        assert_eq!(stats.disk_usage_percent(), 0.0);
    }

    #[tokio::test]
    async fn test_cache_builder() {
        let cache = ImageCache::builder()
            .memory_size_limit(10 * 1024 * 1024)
            .disk_size_limit(50 * 1024 * 1024)
            .memory_item_limit(50)
            .policy(CachePolicy::MemoryOnly)
            .ttl(std::time::Duration::from_secs(3600))
            .build()
            .await
            .unwrap();

        assert_eq!(cache.config.memory_size_limit, 10 * 1024 * 1024);
        assert_eq!(cache.config.disk_size_limit, 50 * 1024 * 1024);
        assert_eq!(cache.config.memory_item_limit, 50);
        assert_eq!(cache.config.policy, CachePolicy::MemoryOnly);
    }

    #[tokio::test]
    async fn test_cache_store_and_get() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = ImageCache::builder()
            .disk_path(temp_dir.path())
            .policy(CachePolicy::MemoryAndDisk)
            .build()
            .await
            .unwrap();

        let data = create_test_image_data("test1", 1000);
        let key = data.cache_key();

        cache.store(&data).await.unwrap();

        let retrieved = cache.get(&key).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().size_bytes, 1000);
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = ImageCache::builder()
            .disk_path(temp_dir.path())
            .build()
            .await
            .unwrap();

        let data = create_test_image_data("test2", 1000);
        let key = data.cache_key();

        cache.store(&data).await.unwrap();
        assert!(cache.contains(&key).await);

        cache.invalidate(&key).await.unwrap();
        assert!(!cache.contains(&key).await);
    }

    #[tokio::test]
    async fn test_cache_clear_all() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = ImageCache::builder()
            .disk_path(temp_dir.path())
            .build()
            .await
            .unwrap();

        let data1 = create_test_image_data("test3", 1000);
        let data2 = create_test_image_data("test4", 2000);

        cache.store(&data1).await.unwrap();
        cache.store(&data2).await.unwrap();

        let stats = cache.stats().await;
        assert_eq!(stats.memory_entries, 2);

        cache.clear_all().await.unwrap();

        let stats = cache.stats().await;
        assert_eq!(stats.memory_entries, 0);
        assert_eq!(stats.disk_entries, 0);
    }

    #[tokio::test]
    async fn test_cache_memory_eviction() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = ImageCache::builder()
            .disk_path(temp_dir.path())
            .memory_size_limit(2500) // Small limit
            .memory_item_limit(10)
            .policy(CachePolicy::MemoryOnly)
            .build()
            .await
            .unwrap();

        // Store three 1000-byte images, but limit is 2500
        let data1 = create_test_image_data("evict1", 1000);
        let data2 = create_test_image_data("evict2", 1000);
        let data3 = create_test_image_data("evict3", 1000);

        cache.store(&data1).await.unwrap();
        cache.store(&data2).await.unwrap();
        cache.store(&data3).await.unwrap();

        let stats = cache.stats().await;
        // Should have evicted at least one entry
        assert!(stats.memory_size <= 2500);
    }

    #[tokio::test]
    async fn test_cache_contains() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = ImageCache::builder()
            .disk_path(temp_dir.path())
            .build()
            .await
            .unwrap();

        let data = create_test_image_data("contains_test", 1000);
        let key = data.cache_key();

        assert!(!cache.contains(&key).await);

        cache.store(&data).await.unwrap();

        assert!(cache.contains(&key).await);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.memory_size_limit, DEFAULT_MEMORY_SIZE_LIMIT);
        assert_eq!(config.disk_size_limit, DEFAULT_DISK_SIZE_LIMIT);
        assert_eq!(config.memory_item_limit, DEFAULT_MEMORY_ITEM_LIMIT);
        assert_eq!(config.policy, CachePolicy::MemoryAndDisk);
        assert!(config.ttl.is_none());
    }
}
