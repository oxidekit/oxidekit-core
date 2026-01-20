//! Core state traits and types.
//!
//! This module defines the fundamental abstractions for state management in OxideKit.

use crate::error::{StateError, StateResult};
use crate::tier::PersistenceTier;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Unique identifier for a state instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateId(String);

impl StateId {
    /// Create a new state ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new unique state ID.
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a state ID from type name.
    pub fn from_type<T>() -> Self {
        Self(std::any::type_name::<T>().to_string())
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for StateId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for StateId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Metadata associated with persisted state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    /// The state identifier.
    pub id: StateId,
    /// Type name for debugging.
    pub type_name: String,
    /// Persistence tier.
    pub tier: PersistenceTier,
    /// State schema version.
    pub version: u32,
    /// When the state was created.
    pub created_at: DateTime<Utc>,
    /// When the state was last modified.
    pub modified_at: DateTime<Utc>,
    /// Hash of the state data for change detection.
    pub content_hash: u64,
    /// Additional custom metadata.
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl StateMetadata {
    /// Create new metadata for a state type.
    pub fn new<T: AppState>() -> Self {
        let now = Utc::now();
        Self {
            id: StateId::from_type::<T>(),
            type_name: std::any::type_name::<T>().to_string(),
            tier: T::tier(),
            version: T::version(),
            created_at: now,
            modified_at: now,
            content_hash: 0,
            custom: HashMap::new(),
        }
    }

    /// Update the modification timestamp.
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Update the content hash.
    pub fn update_hash(&mut self, data: &[u8]) {
        self.content_hash = seahash::hash(data);
    }

    /// Check if the content has changed based on hash.
    pub fn has_changed(&self, data: &[u8]) -> bool {
        seahash::hash(data) != self.content_hash
    }

    /// Add custom metadata.
    pub fn with_custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }
}

/// The core trait for application state.
///
/// Application state represents persistent, application-wide data that survives
/// across sessions. This is distinct from UI state, which is ephemeral.
///
/// # Implementation
///
/// Types implementing `AppState` should also implement `Serialize` and `Deserialize`.
/// The trait provides metadata about how the state should be persisted and versioned.
///
/// # Example
///
/// ```rust,ignore
/// use oxide_state::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// pub struct UserSettings {
///     pub theme: String,
///     pub language: String,
///     pub notifications_enabled: bool,
/// }
///
/// impl AppState for UserSettings {
///     fn state_id(&self) -> StateId {
///         StateId::new("user_settings")
///     }
///
///     fn tier() -> PersistenceTier {
///         PersistenceTier::Local
///     }
///
///     fn version() -> u32 {
///         1
///     }
/// }
/// ```
pub trait AppState: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Get the unique identifier for this state instance.
    fn state_id(&self) -> StateId {
        StateId::from_type::<Self>()
    }

    /// Get the persistence tier for this state type.
    fn tier() -> PersistenceTier
    where
        Self: Sized,
    {
        PersistenceTier::Local
    }

    /// Get the schema version for this state type.
    fn version() -> u32
    where
        Self: Sized,
    {
        1
    }

    /// Validate the state before persistence.
    ///
    /// Returns an error if the state is invalid and should not be persisted.
    fn validate(&self) -> StateResult<()> {
        Ok(())
    }

    /// Called before the state is persisted.
    fn on_before_save(&mut self) {}

    /// Called after the state is loaded.
    fn on_after_load(&mut self) {}

    /// Create metadata for this state.
    fn metadata(&self) -> StateMetadata
    where
        Self: Sized,
    {
        StateMetadata {
            id: self.state_id(),
            type_name: std::any::type_name::<Self>().to_string(),
            tier: Self::tier(),
            version: Self::version(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            content_hash: 0,
            custom: HashMap::new(),
        }
    }
}

/// Trait for ephemeral UI state.
///
/// UI state is view-specific and not persisted. It's used for things like:
/// - Scroll positions
/// - Focus state
/// - Animation state
/// - Form input before submission
///
/// UI state can optionally be preserved during hot reload.
pub trait UiState: Send + Sync + 'static {
    /// Get a unique identifier for this UI state.
    fn ui_state_id(&self) -> StateId {
        StateId::generate()
    }

    /// Whether this UI state should be preserved during hot reload.
    fn preserve_on_hot_reload(&self) -> bool {
        true
    }

    /// Reset the UI state to its default.
    fn reset(&mut self);
}

/// Container for managing multiple state instances.
///
/// The state container provides a centralized registry for all application state,
/// handling serialization, persistence, and lifecycle management.
pub struct StateContainer {
    /// Registered state instances.
    states: RwLock<HashMap<TypeId, Arc<RwLock<Box<dyn StateAny>>>>>,
    /// State metadata.
    metadata: RwLock<HashMap<StateId, StateMetadata>>,
    /// Dirty flags for change tracking.
    dirty: RwLock<HashMap<StateId, bool>>,
}

/// Trait object support for any state type.
pub trait StateAny: Send + Sync {
    /// Get the state ID.
    fn state_id(&self) -> StateId;
    /// Get the type ID.
    fn type_id(&self) -> TypeId;
    /// Serialize to JSON.
    fn serialize_json(&self) -> StateResult<String>;
    /// Get the tier.
    fn tier(&self) -> PersistenceTier;
    /// Get the version.
    fn version(&self) -> u32;
}

impl<T: AppState> StateAny for T {
    fn state_id(&self) -> StateId {
        AppState::state_id(self)
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn serialize_json(&self) -> StateResult<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn tier(&self) -> PersistenceTier {
        T::tier()
    }

    fn version(&self) -> u32 {
        T::version()
    }
}

impl Default for StateContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl StateContainer {
    /// Create a new state container.
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
            dirty: RwLock::new(HashMap::new()),
        }
    }

    /// Register a state type with an initial value.
    pub async fn register<T: AppState + Clone>(&self, initial: T) -> StateResult<()> {
        let type_id = TypeId::of::<T>();
        let state_id = initial.state_id();
        let metadata = initial.metadata();

        let mut states = self.states.write().await;
        let mut meta = self.metadata.write().await;
        let mut dirty = self.dirty.write().await;

        states.insert(type_id, Arc::new(RwLock::new(Box::new(initial))));
        dirty.insert(state_id.clone(), false);
        meta.insert(state_id, metadata);

        Ok(())
    }

    /// Get a read-only reference to a state.
    pub async fn get<T: AppState + Clone>(&self) -> StateResult<T> {
        let type_id = TypeId::of::<T>();
        let states = self.states.read().await;

        let state = states
            .get(&type_id)
            .ok_or_else(|| StateError::not_found(std::any::type_name::<T>()))?;

        let guard = state.read().await;
        let any = guard.as_ref();

        // Safe downcast using JSON serialization (type-erased)
        let json = any.serialize_json()?;
        let value: T = serde_json::from_str(&json)?;
        Ok(value)
    }

    /// Update a state with a closure.
    pub async fn update<T: AppState + Clone, F>(&self, f: F) -> StateResult<()>
    where
        F: FnOnce(&mut T),
    {
        let type_id = TypeId::of::<T>();
        let states = self.states.read().await;

        let state = states
            .get(&type_id)
            .ok_or_else(|| StateError::not_found(std::any::type_name::<T>()))?;

        // Get current value
        let current = {
            let guard = state.read().await;
            let json = guard.serialize_json()?;
            serde_json::from_str::<T>(&json)?
        };

        // Apply update
        let mut updated = current;
        f(&mut updated);

        // Validate
        updated.validate()?;

        // Store updated value
        {
            let mut guard = state.write().await;
            *guard = Box::new(updated.clone());
        }

        // Mark as dirty
        let state_id = updated.state_id();
        let mut dirty = self.dirty.write().await;
        dirty.insert(state_id.clone(), true);

        // Update metadata
        let mut meta = self.metadata.write().await;
        if let Some(m) = meta.get_mut(&state_id) {
            m.touch();
            let json = serde_json::to_vec(&updated)?;
            m.update_hash(&json);
        }

        Ok(())
    }

    /// Check if a state has unsaved changes.
    pub async fn is_dirty<T: AppState + Clone>(&self) -> bool {
        // First get the actual state to retrieve its state_id
        if let Ok(state) = self.get::<T>().await {
            let state_id = state.state_id();
            let dirty = self.dirty.read().await;
            return dirty.get(&state_id).copied().unwrap_or(false);
        }
        false
    }

    /// Mark a state as clean (saved).
    pub async fn mark_clean<T: AppState + Clone>(&self) {
        // First get the actual state to retrieve its state_id
        if let Ok(state) = self.get::<T>().await {
            let state_id = state.state_id();
            let mut dirty = self.dirty.write().await;
            dirty.insert(state_id, false);
        }
    }

    /// Get all dirty state IDs.
    pub async fn dirty_states(&self) -> Vec<StateId> {
        let dirty = self.dirty.read().await;
        dirty
            .iter()
            .filter(|(_, is_dirty)| **is_dirty)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get metadata for a state.
    pub async fn get_metadata(&self, id: &StateId) -> Option<StateMetadata> {
        let meta = self.metadata.read().await;
        meta.get(id).cloned()
    }

    /// Check if a state type is registered.
    pub async fn is_registered<T: AppState>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let states = self.states.read().await;
        states.contains_key(&type_id)
    }
}

/// Wrapper for state with change tracking.
#[derive(Debug, Clone)]
pub struct Tracked<T> {
    value: T,
    dirty: bool,
    version: u64,
}

impl<T: Default> Default for Tracked<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            dirty: false,
            version: 0,
        }
    }
}

impl<T> Tracked<T> {
    /// Create a new tracked value.
    pub fn new(value: T) -> Self {
        Self {
            value,
            dirty: false,
            version: 0,
        }
    }

    /// Get a reference to the value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference, marking the value as dirty.
    pub fn get_mut(&mut self) -> &mut T {
        self.dirty = true;
        self.version += 1;
        &mut self.value
    }

    /// Update the value with a closure.
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.value);
        self.dirty = true;
        self.version += 1;
    }

    /// Set a new value.
    pub fn set(&mut self, value: T) {
        self.value = value;
        self.dirty = true;
        self.version += 1;
    }

    /// Check if the value has been modified.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark as clean.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get the current version.
    pub fn version(&self) -> u64 {
        self.version
    }
}

impl<T: Serialize> Tracked<T> {
    /// Serialize the inner value to JSON.
    pub fn to_json(&self) -> StateResult<String> {
        Ok(serde_json::to_string(&self.value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
    struct TestState {
        counter: i32,
        name: String,
    }

    impl AppState for TestState {
        // Use default state_id() which uses StateId::from_type::<Self>()

        fn tier() -> PersistenceTier {
            PersistenceTier::Local
        }

        fn version() -> u32 {
            1
        }
    }

    #[tokio::test]
    async fn test_state_container_register_get() {
        let container = StateContainer::new();
        let state = TestState {
            counter: 42,
            name: "test".to_string(),
        };

        container.register(state.clone()).await.unwrap();
        let retrieved: TestState = container.get().await.unwrap();

        assert_eq!(retrieved.counter, 42);
        assert_eq!(retrieved.name, "test");
    }

    #[tokio::test]
    async fn test_state_container_update() {
        let container = StateContainer::new();
        let state = TestState {
            counter: 0,
            name: "initial".to_string(),
        };

        container.register(state).await.unwrap();

        container
            .update::<TestState, _>(|s| {
                s.counter = 100;
                s.name = "updated".to_string();
            })
            .await
            .unwrap();

        let retrieved: TestState = container.get().await.unwrap();
        assert_eq!(retrieved.counter, 100);
        assert_eq!(retrieved.name, "updated");
    }

    #[tokio::test]
    async fn test_state_dirty_tracking() {
        let container = StateContainer::new();
        let state = TestState::default();

        container.register(state).await.unwrap();
        assert!(!container.is_dirty::<TestState>().await);

        container
            .update::<TestState, _>(|s| s.counter = 1)
            .await
            .unwrap();
        assert!(container.is_dirty::<TestState>().await);

        container.mark_clean::<TestState>().await;
        assert!(!container.is_dirty::<TestState>().await);
    }

    #[test]
    fn test_tracked_value() {
        let mut tracked = Tracked::new(42);

        assert_eq!(*tracked.get(), 42);
        assert!(!tracked.is_dirty());

        *tracked.get_mut() = 100;
        assert_eq!(*tracked.get(), 100);
        assert!(tracked.is_dirty());

        tracked.mark_clean();
        assert!(!tracked.is_dirty());
    }

    #[test]
    fn test_state_id_from_type() {
        let id = StateId::from_type::<TestState>();
        assert!(id.as_str().contains("TestState"));
    }
}
