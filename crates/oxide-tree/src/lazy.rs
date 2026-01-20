//! Lazy loading support for tree nodes.
//!
//! This module provides types and traits for loading tree node children on demand,
//! with support for async loading, caching, and error handling.

use crate::node::{NodeId, TreeNode};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Error type for lazy loading operations.
#[derive(Error, Debug, Clone)]
pub enum LoadError {
    /// The node was not found.
    #[error("Node not found: {0}")]
    NotFound(String),

    /// The load operation timed out.
    #[error("Load operation timed out")]
    Timeout,

    /// The load operation was cancelled.
    #[error("Load operation was cancelled")]
    Cancelled,

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// A network error occurred.
    #[error("Network error: {0}")]
    Network(String),

    /// A custom error.
    #[error("{0}")]
    Custom(String),
}

impl LoadError {
    /// Create a custom error.
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

/// Result type for lazy loading operations.
pub type LoadResult<T> = Result<T, LoadError>;

/// State of a lazy load operation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LoadState {
    /// Not yet loaded.
    #[default]
    Idle,
    /// Currently loading.
    Loading,
    /// Successfully loaded.
    Loaded,
    /// Failed to load with an error.
    Failed(String),
}

impl LoadState {
    /// Check if the state is loading.
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Check if the state is loaded.
    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded)
    }

    /// Check if the state is failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    /// Check if the state is idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Get the error message if failed.
    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Synchronous loader trait for tree nodes.
pub trait NodeLoader: Send + Sync {
    /// Load children for a node.
    fn load_children(&self, node_id: &NodeId) -> LoadResult<Vec<TreeNode>>;

    /// Check if a node can have children (optional optimization).
    fn can_have_children(&self, _node_id: &NodeId) -> bool {
        true
    }

    /// Get the estimated child count (optional optimization for UI).
    fn estimated_child_count(&self, _node_id: &NodeId) -> Option<usize> {
        None
    }
}

/// Async loader trait for tree nodes.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncNodeLoader: Send + Sync {
    /// Load children for a node asynchronously.
    async fn load_children(&self, node_id: &NodeId) -> LoadResult<Vec<TreeNode>>;

    /// Check if a node can have children (optional optimization).
    fn can_have_children(&self, _node_id: &NodeId) -> bool {
        true
    }

    /// Get the estimated child count (optional optimization for UI).
    fn estimated_child_count(&self, _node_id: &NodeId) -> Option<usize> {
        None
    }
}

/// Cache for loaded children.
#[derive(Default)]
pub struct LoadCache {
    /// Cached children by parent node ID.
    cache: HashMap<NodeId, Vec<TreeNode>>,
    /// Load state by node ID.
    states: HashMap<NodeId, LoadState>,
    /// Maximum cache size (0 for unlimited).
    max_size: usize,
    /// Order of cache entries for LRU eviction.
    order: Vec<NodeId>,
}

impl LoadCache {
    /// Create a new load cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new load cache with a maximum size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            max_size,
            ..Default::default()
        }
    }

    /// Get cached children for a node.
    pub fn get(&self, node_id: &NodeId) -> Option<&Vec<TreeNode>> {
        self.cache.get(node_id)
    }

    /// Get the load state for a node.
    pub fn get_state(&self, node_id: &NodeId) -> LoadState {
        self.states.get(node_id).cloned().unwrap_or_default()
    }

    /// Set the load state for a node.
    pub fn set_state(&mut self, node_id: NodeId, state: LoadState) {
        self.states.insert(node_id, state);
    }

    /// Cache children for a node.
    pub fn set(&mut self, node_id: NodeId, children: Vec<TreeNode>) {
        // Evict if at max capacity
        if self.max_size > 0 && self.cache.len() >= self.max_size && !self.cache.contains_key(&node_id) {
            self.evict_oldest();
        }

        // Remove from order if already present
        self.order.retain(|id| id != &node_id);
        self.order.push(node_id.clone());

        self.cache.insert(node_id.clone(), children);
        self.states.insert(node_id, LoadState::Loaded);
    }

    /// Set a load error for a node.
    pub fn set_error(&mut self, node_id: NodeId, error: &LoadError) {
        self.states.insert(node_id, LoadState::Failed(error.to_string()));
    }

    /// Check if children are cached for a node.
    pub fn is_cached(&self, node_id: &NodeId) -> bool {
        self.cache.contains_key(node_id)
    }

    /// Check if a node is currently loading.
    pub fn is_loading(&self, node_id: &NodeId) -> bool {
        self.get_state(node_id).is_loading()
    }

    /// Mark a node as loading.
    pub fn mark_loading(&mut self, node_id: NodeId) {
        self.states.insert(node_id, LoadState::Loading);
    }

    /// Invalidate the cache for a node.
    pub fn invalidate(&mut self, node_id: &NodeId) {
        self.cache.remove(node_id);
        self.states.remove(node_id);
        self.order.retain(|id| id != node_id);
    }

    /// Invalidate the cache for a node and all its descendants.
    pub fn invalidate_subtree(&mut self, node_id: &NodeId) {
        // First, collect all node IDs to invalidate
        let mut to_invalidate = vec![node_id.clone()];
        let mut i = 0;

        while i < to_invalidate.len() {
            if let Some(children) = self.cache.get(&to_invalidate[i]) {
                for child in children {
                    to_invalidate.push(child.id.clone());
                }
            }
            i += 1;
        }

        // Now invalidate all
        for id in to_invalidate {
            self.cache.remove(&id);
            self.states.remove(&id);
            self.order.retain(|oid| oid != &id);
        }
    }

    /// Clear the entire cache.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.states.clear();
        self.order.clear();
    }

    /// Get the number of cached nodes.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Evict the oldest cache entry.
    fn evict_oldest(&mut self) {
        if let Some(oldest) = self.order.first().cloned() {
            self.invalidate(&oldest);
        }
    }

    /// Touch a cache entry to mark it as recently used.
    pub fn touch(&mut self, node_id: &NodeId) {
        if self.cache.contains_key(node_id) {
            self.order.retain(|id| id != node_id);
            self.order.push(node_id.clone());
        }
    }
}

impl fmt::Debug for LoadCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadCache")
            .field("cached_count", &self.cache.len())
            .field("max_size", &self.max_size)
            .finish()
    }
}

/// Configuration for lazy loading behavior.
#[derive(Clone, Debug)]
pub struct LazyLoadConfig {
    /// Preload children of visible nodes.
    pub preload_visible: bool,
    /// Preload N levels deep when expanding.
    pub preload_depth: usize,
    /// Retry failed loads automatically.
    pub auto_retry: bool,
    /// Maximum retry attempts.
    pub max_retries: usize,
    /// Retry delay in milliseconds.
    pub retry_delay_ms: u64,
    /// Timeout for load operations in milliseconds.
    pub timeout_ms: u64,
    /// Cache settings.
    pub cache_size: usize,
}

impl Default for LazyLoadConfig {
    fn default() -> Self {
        Self {
            preload_visible: false,
            preload_depth: 0,
            auto_retry: false,
            max_retries: 3,
            retry_delay_ms: 1000,
            timeout_ms: 30000,
            cache_size: 1000,
        }
    }
}

impl LazyLoadConfig {
    /// Create a new config with preloading enabled.
    pub fn with_preload(depth: usize) -> Self {
        Self {
            preload_visible: true,
            preload_depth: depth,
            ..Default::default()
        }
    }

    /// Enable auto-retry with the specified number of attempts.
    pub fn with_retry(mut self, max_retries: usize) -> Self {
        self.auto_retry = true;
        self.max_retries = max_retries;
        self
    }

    /// Set the load timeout.
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set the cache size.
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }
}

/// Manager for lazy loading operations.
pub struct LazyLoadManager<L> {
    /// The node loader.
    loader: L,
    /// Load cache.
    cache: LoadCache,
    /// Configuration.
    config: LazyLoadConfig,
    /// Retry counts per node.
    retry_counts: HashMap<NodeId, usize>,
}

impl<L: NodeLoader> LazyLoadManager<L> {
    /// Create a new lazy load manager.
    pub fn new(loader: L) -> Self {
        Self::with_config(loader, LazyLoadConfig::default())
    }

    /// Create a new lazy load manager with configuration.
    pub fn with_config(loader: L, config: LazyLoadConfig) -> Self {
        Self {
            loader,
            cache: LoadCache::with_max_size(config.cache_size),
            config,
            retry_counts: HashMap::new(),
        }
    }

    /// Get cached children for a node.
    pub fn get_cached(&self, node_id: &NodeId) -> Option<&Vec<TreeNode>> {
        self.cache.get(node_id)
    }

    /// Get the load state for a node.
    pub fn get_state(&self, node_id: &NodeId) -> LoadState {
        self.cache.get_state(node_id)
    }

    /// Load children for a node (synchronous).
    pub fn load(&mut self, node_id: &NodeId) -> LoadResult<&Vec<TreeNode>> {
        // Check cache first
        if self.cache.is_cached(node_id) {
            self.cache.touch(node_id);
            return Ok(self.cache.get(node_id).unwrap());
        }

        // Mark as loading
        self.cache.mark_loading(node_id.clone());

        // Attempt load
        match self.loader.load_children(node_id) {
            Ok(children) => {
                self.cache.set(node_id.clone(), children);
                self.retry_counts.remove(node_id);
                Ok(self.cache.get(node_id).unwrap())
            }
            Err(e) => {
                self.cache.set_error(node_id.clone(), &e);

                // Handle retry
                if self.config.auto_retry {
                    let count = self.retry_counts.entry(node_id.clone()).or_insert(0);
                    if *count < self.config.max_retries {
                        *count += 1;
                        // In sync mode, we can't delay, so just try again
                        return self.load(node_id);
                    }
                }

                Err(e)
            }
        }
    }

    /// Check if a node can have children.
    pub fn can_have_children(&self, node_id: &NodeId) -> bool {
        self.loader.can_have_children(node_id)
    }

    /// Get the estimated child count for a node.
    pub fn estimated_child_count(&self, node_id: &NodeId) -> Option<usize> {
        self.loader.estimated_child_count(node_id)
    }

    /// Invalidate the cache for a node.
    pub fn invalidate(&mut self, node_id: &NodeId) {
        self.cache.invalidate(node_id);
        self.retry_counts.remove(node_id);
    }

    /// Invalidate the entire cache.
    pub fn invalidate_all(&mut self) {
        self.cache.clear();
        self.retry_counts.clear();
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &LazyLoadConfig {
        &self.config
    }

    /// Update the configuration.
    pub fn set_config(&mut self, config: LazyLoadConfig) {
        if config.cache_size != self.config.cache_size {
            self.cache = LoadCache::with_max_size(config.cache_size);
        }
        self.config = config;
    }
}

impl<L: fmt::Debug> fmt::Debug for LazyLoadManager<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyLoadManager")
            .field("loader", &self.loader)
            .field("cache", &self.cache)
            .field("config", &self.config)
            .finish()
    }
}

/// A simple in-memory loader for testing.
#[derive(Default, Debug)]
pub struct MemoryLoader {
    /// Children by parent node ID.
    children: HashMap<NodeId, Vec<TreeNode>>,
}

impl MemoryLoader {
    /// Create a new memory loader.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add children for a parent node.
    pub fn add_children(&mut self, parent_id: NodeId, children: Vec<TreeNode>) {
        self.children.insert(parent_id, children);
    }

    /// Remove children for a parent node.
    pub fn remove_children(&mut self, parent_id: &NodeId) {
        self.children.remove(parent_id);
    }
}

impl NodeLoader for MemoryLoader {
    fn load_children(&self, node_id: &NodeId) -> LoadResult<Vec<TreeNode>> {
        self.children
            .get(node_id)
            .cloned()
            .ok_or_else(|| LoadError::NotFound(node_id.to_string()))
    }

    fn can_have_children(&self, node_id: &NodeId) -> bool {
        self.children.contains_key(node_id)
    }

    fn estimated_child_count(&self, node_id: &NodeId) -> Option<usize> {
        self.children.get(node_id).map(|c| c.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_state() {
        assert!(LoadState::Idle.is_idle());
        assert!(LoadState::Loading.is_loading());
        assert!(LoadState::Loaded.is_loaded());
        assert!(LoadState::Failed("error".to_string()).is_failed());

        let failed = LoadState::Failed("test error".to_string());
        assert_eq!(failed.error(), Some("test error"));
    }

    #[test]
    fn test_load_cache_basic() {
        let mut cache = LoadCache::new();

        let node_id = NodeId::from_string("test");
        let children = vec![TreeNode::new("child1", "Child 1")];

        cache.set(node_id.clone(), children.clone());

        assert!(cache.is_cached(&node_id));
        assert_eq!(cache.get(&node_id).unwrap().len(), 1);
        assert!(cache.get_state(&node_id).is_loaded());
    }

    #[test]
    fn test_load_cache_invalidate() {
        let mut cache = LoadCache::new();

        let node_id = NodeId::from_string("test");
        cache.set(node_id.clone(), vec![]);

        assert!(cache.is_cached(&node_id));

        cache.invalidate(&node_id);
        assert!(!cache.is_cached(&node_id));
    }

    #[test]
    fn test_load_cache_max_size() {
        let mut cache = LoadCache::with_max_size(2);

        cache.set(NodeId::from_string("a"), vec![]);
        cache.set(NodeId::from_string("b"), vec![]);
        cache.set(NodeId::from_string("c"), vec![]);

        assert_eq!(cache.len(), 2);
        assert!(!cache.is_cached(&NodeId::from_string("a"))); // Evicted
        assert!(cache.is_cached(&NodeId::from_string("b")));
        assert!(cache.is_cached(&NodeId::from_string("c")));
    }

    #[test]
    fn test_load_cache_touch() {
        let mut cache = LoadCache::with_max_size(2);

        cache.set(NodeId::from_string("a"), vec![]);
        cache.set(NodeId::from_string("b"), vec![]);

        // Touch 'a' to make it recently used
        cache.touch(&NodeId::from_string("a"));

        // Add 'c', should evict 'b'
        cache.set(NodeId::from_string("c"), vec![]);

        assert!(cache.is_cached(&NodeId::from_string("a")));
        assert!(!cache.is_cached(&NodeId::from_string("b")));
        assert!(cache.is_cached(&NodeId::from_string("c")));
    }

    #[test]
    fn test_memory_loader() {
        let mut loader = MemoryLoader::new();
        let parent_id = NodeId::from_string("parent");

        loader.add_children(
            parent_id.clone(),
            vec![
                TreeNode::new("child1", "Child 1"),
                TreeNode::new("child2", "Child 2"),
            ],
        );

        let children = loader.load_children(&parent_id).unwrap();
        assert_eq!(children.len(), 2);
        assert!(loader.can_have_children(&parent_id));
        assert_eq!(loader.estimated_child_count(&parent_id), Some(2));
    }

    #[test]
    fn test_memory_loader_not_found() {
        let loader = MemoryLoader::new();
        let result = loader.load_children(&NodeId::from_string("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_lazy_load_manager() {
        let mut loader = MemoryLoader::new();
        let parent_id = NodeId::from_string("parent");

        loader.add_children(
            parent_id.clone(),
            vec![TreeNode::new("child1", "Child 1")],
        );

        let mut manager = LazyLoadManager::new(loader);

        // First load
        let children = manager.load(&parent_id).unwrap();
        assert_eq!(children.len(), 1);

        // Should be cached now
        assert!(manager.get_state(&parent_id).is_loaded());

        // Second load should come from cache
        let cached = manager.get_cached(&parent_id).unwrap();
        assert_eq!(cached.len(), 1);
    }

    #[test]
    fn test_lazy_load_manager_invalidate() {
        let mut loader = MemoryLoader::new();
        let parent_id = NodeId::from_string("parent");

        loader.add_children(parent_id.clone(), vec![]);

        let mut manager = LazyLoadManager::new(loader);
        manager.load(&parent_id).unwrap();

        assert!(manager.get_cached(&parent_id).is_some());

        manager.invalidate(&parent_id);
        assert!(manager.get_cached(&parent_id).is_none());
    }

    #[test]
    fn test_lazy_load_config() {
        let config = LazyLoadConfig::with_preload(2)
            .with_retry(5)
            .with_timeout(60000)
            .with_cache_size(500);

        assert!(config.preload_visible);
        assert_eq!(config.preload_depth, 2);
        assert!(config.auto_retry);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.timeout_ms, 60000);
        assert_eq!(config.cache_size, 500);
    }

    #[test]
    fn test_load_error_display() {
        let err = LoadError::NotFound("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = LoadError::custom("custom error");
        assert_eq!(err.to_string(), "custom error");
    }

    #[test]
    fn test_load_cache_loading_state() {
        let mut cache = LoadCache::new();
        let node_id = NodeId::from_string("test");

        cache.mark_loading(node_id.clone());
        assert!(cache.is_loading(&node_id));

        cache.set(node_id.clone(), vec![]);
        assert!(!cache.is_loading(&node_id));
    }

    #[test]
    fn test_load_cache_error_state() {
        let mut cache = LoadCache::new();
        let node_id = NodeId::from_string("test");

        cache.set_error(node_id.clone(), &LoadError::Timeout);
        assert!(cache.get_state(&node_id).is_failed());
    }
}
