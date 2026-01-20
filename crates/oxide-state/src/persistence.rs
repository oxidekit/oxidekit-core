//! Persistence backends for state storage.
//!
//! This module provides various backends for persisting application state,
//! including file-based and SQLite storage options.

use crate::error::{StateError, StateResult};
use crate::state::{AppState, StateId, StateMetadata};
use crate::tier::PersistenceTier;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};


/// Stored state entry with metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredState {
    /// State metadata.
    pub metadata: StateMetadata,
    /// Serialized JSON data.
    pub data: String,
}

/// Trait for persistence backends.
///
/// Implementations provide storage mechanisms for persisting application state.
#[async_trait::async_trait]
pub trait PersistenceBackend: Send + Sync {
    /// Save a state to storage.
    async fn save(&self, id: &StateId, state: &StoredState) -> StateResult<()>;

    /// Load a state from storage.
    async fn load(&self, id: &StateId) -> StateResult<Option<StoredState>>;

    /// Delete a state from storage.
    async fn delete(&self, id: &StateId) -> StateResult<bool>;

    /// Check if a state exists.
    async fn exists(&self, id: &StateId) -> StateResult<bool>;

    /// List all stored state IDs.
    async fn list(&self) -> StateResult<Vec<StateId>>;

    /// Clear all stored states.
    async fn clear(&self) -> StateResult<()>;

    /// Get the backend name.
    fn name(&self) -> &'static str;
}

/// File-based persistence backend.
///
/// Stores each state as a separate JSON file in a directory structure.
pub struct FileBackend {
    /// Base directory for state storage.
    base_dir: PathBuf,
    /// In-memory cache of loaded states.
    cache: Arc<RwLock<HashMap<StateId, StoredState>>>,
    /// Whether to use caching.
    use_cache: bool,
}

impl FileBackend {
    /// Create a new file backend with the specified base directory.
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            use_cache: true,
        }
    }

    /// Create a file backend for a specific application.
    pub fn for_app(app_name: &str) -> StateResult<Self> {
        let base_dir = directories::ProjectDirs::from("dev", "oxidekit", app_name)
            .ok_or_else(|| StateError::ConfigError("Could not determine data directory".into()))?
            .data_dir()
            .join("state");

        Ok(Self::new(base_dir))
    }

    /// Disable caching.
    pub fn without_cache(mut self) -> Self {
        self.use_cache = false;
        self
    }

    /// Get the file path for a state ID.
    fn state_path(&self, id: &StateId, tier: PersistenceTier) -> PathBuf {
        let tier_dir = match tier {
            PersistenceTier::Volatile => "volatile",
            PersistenceTier::Local => "local",
            PersistenceTier::Secure => "secure",
            PersistenceTier::Encrypted => "encrypted",
            PersistenceTier::Syncable => "syncable",
        };

        // Sanitize the state ID for use as a filename
        let safe_id: String = id
            .as_str()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect();

        self.base_dir
            .join(tier_dir)
            .join(format!("{}.{}", safe_id, tier.file_extension()))
    }

    /// Ensure the directory for a tier exists.
    async fn ensure_dir(&self, tier: PersistenceTier) -> StateResult<()> {
        let tier_dir = match tier {
            PersistenceTier::Volatile => "volatile",
            PersistenceTier::Local => "local",
            PersistenceTier::Secure => "secure",
            PersistenceTier::Encrypted => "encrypted",
            PersistenceTier::Syncable => "syncable",
        };

        let dir = self.base_dir.join(tier_dir);
        tokio::fs::create_dir_all(&dir).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl PersistenceBackend for FileBackend {
    async fn save(&self, id: &StateId, state: &StoredState) -> StateResult<()> {
        self.ensure_dir(state.metadata.tier).await?;

        let path = self.state_path(id, state.metadata.tier);
        let json = serde_json::to_string_pretty(state)?;

        tokio::fs::write(&path, &json).await?;
        debug!("Saved state {} to {}", id, path.display());

        // Update cache
        if self.use_cache {
            let mut cache = self.cache.write().await;
            cache.insert(id.clone(), state.clone());
        }

        Ok(())
    }

    async fn load(&self, id: &StateId) -> StateResult<Option<StoredState>> {
        // Check cache first
        if self.use_cache {
            let cache = self.cache.read().await;
            if let Some(state) = cache.get(id) {
                debug!("Cache hit for state {}", id);
                return Ok(Some(state.clone()));
            }
        }

        // Try each tier directory
        for tier in [
            PersistenceTier::Local,
            PersistenceTier::Secure,
            PersistenceTier::Encrypted,
            PersistenceTier::Syncable,
            PersistenceTier::Volatile,
        ] {
            let path = self.state_path(id, tier);
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await?;
                let state: StoredState = serde_json::from_str(&content)?;

                // Update cache
                if self.use_cache {
                    let mut cache = self.cache.write().await;
                    cache.insert(id.clone(), state.clone());
                }

                debug!("Loaded state {} from {}", id, path.display());
                return Ok(Some(state));
            }
        }

        Ok(None)
    }

    async fn delete(&self, id: &StateId) -> StateResult<bool> {
        // Remove from cache
        if self.use_cache {
            let mut cache = self.cache.write().await;
            cache.remove(id);
        }

        // Try each tier directory
        for tier in [
            PersistenceTier::Local,
            PersistenceTier::Secure,
            PersistenceTier::Encrypted,
            PersistenceTier::Syncable,
            PersistenceTier::Volatile,
        ] {
            let path = self.state_path(id, tier);
            if path.exists() {
                tokio::fs::remove_file(&path).await?;
                debug!("Deleted state {} from {}", id, path.display());
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn exists(&self, id: &StateId) -> StateResult<bool> {
        // Check cache first
        if self.use_cache {
            let cache = self.cache.read().await;
            if cache.contains_key(id) {
                return Ok(true);
            }
        }

        // Check filesystem
        for tier in [
            PersistenceTier::Local,
            PersistenceTier::Secure,
            PersistenceTier::Encrypted,
            PersistenceTier::Syncable,
            PersistenceTier::Volatile,
        ] {
            let path = self.state_path(id, tier);
            if path.exists() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn list(&self) -> StateResult<Vec<StateId>> {
        let mut ids = Vec::new();

        for tier in [
            PersistenceTier::Local,
            PersistenceTier::Secure,
            PersistenceTier::Encrypted,
            PersistenceTier::Syncable,
        ] {
            let tier_dir = match tier {
                PersistenceTier::Volatile => "volatile",
                PersistenceTier::Local => "local",
                PersistenceTier::Secure => "secure",
                PersistenceTier::Encrypted => "encrypted",
                PersistenceTier::Syncable => "syncable",
            };

            let dir = self.base_dir.join(tier_dir);
            if !dir.exists() {
                continue;
            }

            let mut entries = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        ids.push(StateId::new(name));
                    }
                }
            }
        }

        Ok(ids)
    }

    async fn clear(&self) -> StateResult<()> {
        // Clear cache
        if self.use_cache {
            let mut cache = self.cache.write().await;
            cache.clear();
        }

        // Remove all tier directories
        for tier in [
            PersistenceTier::Local,
            PersistenceTier::Secure,
            PersistenceTier::Encrypted,
            PersistenceTier::Syncable,
            PersistenceTier::Volatile,
        ] {
            let tier_dir = match tier {
                PersistenceTier::Volatile => "volatile",
                PersistenceTier::Local => "local",
                PersistenceTier::Secure => "secure",
                PersistenceTier::Encrypted => "encrypted",
                PersistenceTier::Syncable => "syncable",
            };

            let dir = self.base_dir.join(tier_dir);
            if dir.exists() {
                tokio::fs::remove_dir_all(&dir).await?;
            }
        }

        info!("Cleared all state storage");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "file"
    }
}

/// SQLite persistence backend.
#[cfg(feature = "sqlite")]
pub struct SqliteBackend {
    /// Database connection.
    conn: Arc<tokio::sync::Mutex<rusqlite::Connection>>,
}

#[cfg(feature = "sqlite")]
impl SqliteBackend {
    /// Create a new SQLite backend with the specified database path.
    pub fn new(db_path: impl AsRef<Path>) -> StateResult<Self> {
        let conn = rusqlite::Connection::open(db_path)?;

        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS states (
                id TEXT PRIMARY KEY,
                tier TEXT NOT NULL,
                type_name TEXT NOT NULL,
                version INTEGER NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                content_hash INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_states_tier ON states(tier)",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(tokio::sync::Mutex::new(conn)),
        })
    }

    /// Create an in-memory SQLite backend.
    pub fn in_memory() -> StateResult<Self> {
        let conn = rusqlite::Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS states (
                id TEXT PRIMARY KEY,
                tier TEXT NOT NULL,
                type_name TEXT NOT NULL,
                version INTEGER NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                content_hash INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(tokio::sync::Mutex::new(conn)),
        })
    }

    /// Create a SQLite backend for a specific application.
    pub fn for_app(app_name: &str) -> StateResult<Self> {
        let db_path = directories::ProjectDirs::from("dev", "oxidekit", app_name)
            .ok_or_else(|| StateError::ConfigError("Could not determine data directory".into()))?
            .data_dir()
            .join("state.db");

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Self::new(db_path)
    }
}

#[cfg(feature = "sqlite")]
#[async_trait::async_trait]
impl PersistenceBackend for SqliteBackend {
    async fn save(&self, id: &StateId, state: &StoredState) -> StateResult<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            "INSERT OR REPLACE INTO states
             (id, tier, type_name, version, data, created_at, modified_at, content_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                id.as_str(),
                state.metadata.tier.to_string(),
                state.metadata.type_name,
                state.metadata.version,
                state.data,
                state.metadata.created_at.to_rfc3339(),
                state.metadata.modified_at.to_rfc3339(),
                state.metadata.content_hash as i64,
            ],
        )?;

        debug!("Saved state {} to SQLite", id);
        Ok(())
    }

    async fn load(&self, id: &StateId) -> StateResult<Option<StoredState>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT tier, type_name, version, data, created_at, modified_at, content_hash
             FROM states WHERE id = ?1",
        )?;

        let result = stmt.query_row([id.as_str()], |row| {
            let tier_str: String = row.get(0)?;
            let tier = PersistenceTier::from_str(&tier_str).unwrap_or_default();
            let type_name: String = row.get(1)?;
            let version: u32 = row.get(2)?;
            let data: String = row.get(3)?;
            let created_at_str: String = row.get(4)?;
            let modified_at_str: String = row.get(5)?;
            let content_hash: i64 = row.get(6)?;

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let modified_at = chrono::DateTime::parse_from_rfc3339(&modified_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(StoredState {
                metadata: StateMetadata {
                    id: id.clone(),
                    type_name,
                    tier,
                    version,
                    created_at,
                    modified_at,
                    content_hash: content_hash as u64,
                    custom: HashMap::new(),
                },
                data,
            })
        });

        match result {
            Ok(state) => {
                debug!("Loaded state {} from SQLite", id);
                Ok(Some(state))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, id: &StateId) -> StateResult<bool> {
        let conn = self.conn.lock().await;
        let rows = conn.execute("DELETE FROM states WHERE id = ?1", [id.as_str()])?;
        if rows > 0 {
            debug!("Deleted state {} from SQLite", id);
        }
        Ok(rows > 0)
    }

    async fn exists(&self, id: &StateId) -> StateResult<bool> {
        let conn = self.conn.lock().await;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM states WHERE id = ?1",
            [id.as_str()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    async fn list(&self) -> StateResult<Vec<StateId>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id FROM states")?;
        let ids: Result<Vec<_>, _> = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                Ok(StateId::new(id))
            })?
            .collect();
        Ok(ids?)
    }

    async fn clear(&self) -> StateResult<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM states", [])?;
        info!("Cleared all SQLite state storage");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "sqlite"
    }
}

/// High-level state store that combines backend with state management.
pub struct StateStore {
    /// The persistence backend.
    backend: Arc<dyn PersistenceBackend>,
    /// Application name for directory resolution.
    app_name: String,
}

impl StateStore {
    /// Create a new state store with a file backend.
    pub fn new_file(app_name: impl Into<String>) -> StateResult<Self> {
        let app_name = app_name.into();
        let backend = FileBackend::for_app(&app_name)?;
        Ok(Self {
            backend: Arc::new(backend),
            app_name,
        })
    }

    /// Create a state store with a custom backend.
    pub fn with_backend(
        app_name: impl Into<String>,
        backend: Arc<dyn PersistenceBackend>,
    ) -> Self {
        Self {
            backend,
            app_name: app_name.into(),
        }
    }

    /// Create a state store with a SQLite backend.
    #[cfg(feature = "sqlite")]
    pub fn new_sqlite(app_name: impl Into<String>) -> StateResult<Self> {
        let app_name = app_name.into();
        let backend = SqliteBackend::for_app(&app_name)?;
        Ok(Self {
            backend: Arc::new(backend),
            app_name,
        })
    }

    /// Get the application name.
    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    /// Get the backend name.
    pub fn backend_name(&self) -> &'static str {
        self.backend.name()
    }

    /// Save a state to storage.
    pub async fn save<T: AppState>(&self, state: &T) -> StateResult<()> {
        // Skip volatile states
        if T::tier() == PersistenceTier::Volatile {
            debug!("Skipping save for volatile state {}", state.state_id());
            return Ok(());
        }

        // Validate before saving
        state.validate()?;

        let id = state.state_id();
        let mut metadata = state.metadata();
        let json = serde_json::to_string(state)?;

        metadata.update_hash(json.as_bytes());
        metadata.touch();

        let stored = StoredState {
            metadata,
            data: json,
        };

        self.backend.save(&id, &stored).await
    }

    /// Load a state from storage.
    pub async fn load<T: AppState + Default>(&self) -> StateResult<T> {
        let id = StateId::from_type::<T>();

        if let Some(stored) = self.backend.load(&id).await? {
            // Check version
            if stored.metadata.version != T::version() {
                warn!(
                    "State {} version mismatch: stored {}, expected {}",
                    id,
                    stored.metadata.version,
                    T::version()
                );
                return Err(StateError::version_mismatch(
                    stored.metadata.version,
                    T::version(),
                ));
            }

            let mut state: T = serde_json::from_str(&stored.data)?;
            state.on_after_load();
            return Ok(state);
        }

        // Return default if not found
        Ok(T::default())
    }

    /// Load a state or return the provided default.
    pub async fn load_or<T: AppState>(&self, default: T) -> StateResult<T> {
        let id = StateId::from_type::<T>();

        if let Some(stored) = self.backend.load(&id).await? {
            if stored.metadata.version == T::version() {
                let mut state: T = serde_json::from_str(&stored.data)?;
                state.on_after_load();
                return Ok(state);
            }
        }

        Ok(default)
    }

    /// Delete a state from storage.
    pub async fn delete<T: AppState>(&self) -> StateResult<bool> {
        let id = StateId::from_type::<T>();
        self.backend.delete(&id).await
    }

    /// Check if a state exists in storage.
    pub async fn exists<T: AppState>(&self) -> StateResult<bool> {
        let id = StateId::from_type::<T>();
        self.backend.exists(&id).await
    }

    /// List all stored state IDs.
    pub async fn list(&self) -> StateResult<Vec<StateId>> {
        self.backend.list().await
    }

    /// Clear all stored states.
    pub async fn clear(&self) -> StateResult<()> {
        self.backend.clear().await
    }

    /// Get the raw stored state with metadata.
    pub async fn get_raw(&self, id: &StateId) -> StateResult<Option<StoredState>> {
        self.backend.load(id).await
    }

    /// Save raw state data (for migrations, etc.).
    pub async fn save_raw(&self, id: &StateId, state: &StoredState) -> StateResult<()> {
        self.backend.save(id, state).await
    }
}

/// In-memory persistence backend for testing.
pub struct MemoryBackend {
    states: Arc<RwLock<HashMap<StateId, StoredState>>>,
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBackend {
    /// Create a new in-memory backend.
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl PersistenceBackend for MemoryBackend {
    async fn save(&self, id: &StateId, state: &StoredState) -> StateResult<()> {
        let mut states = self.states.write().await;
        states.insert(id.clone(), state.clone());
        Ok(())
    }

    async fn load(&self, id: &StateId) -> StateResult<Option<StoredState>> {
        let states = self.states.read().await;
        Ok(states.get(id).cloned())
    }

    async fn delete(&self, id: &StateId) -> StateResult<bool> {
        let mut states = self.states.write().await;
        Ok(states.remove(id).is_some())
    }

    async fn exists(&self, id: &StateId) -> StateResult<bool> {
        let states = self.states.read().await;
        Ok(states.contains_key(id))
    }

    async fn list(&self) -> StateResult<Vec<StateId>> {
        let states = self.states.read().await;
        Ok(states.keys().cloned().collect())
    }

    async fn clear(&self) -> StateResult<()> {
        let mut states = self.states.write().await;
        states.clear();
        Ok(())
    }

    fn name(&self) -> &'static str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tier::PersistenceTier;

    #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
    struct TestState {
        value: i32,
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
    async fn test_memory_backend() {
        let backend = MemoryBackend::new();
        let id = StateId::new("test");
        let metadata = StateMetadata {
            id: id.clone(),
            type_name: "Test".to_string(),
            tier: PersistenceTier::Local,
            version: 1,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            content_hash: 0,
            custom: HashMap::new(),
        };

        let stored = StoredState {
            metadata,
            data: r#"{"value": 42}"#.to_string(),
        };

        // Save
        backend.save(&id, &stored).await.unwrap();

        // Exists
        assert!(backend.exists(&id).await.unwrap());

        // Load
        let loaded = backend.load(&id).await.unwrap().unwrap();
        assert_eq!(loaded.data, stored.data);

        // List
        let ids = backend.list().await.unwrap();
        assert_eq!(ids.len(), 1);

        // Delete
        assert!(backend.delete(&id).await.unwrap());
        assert!(!backend.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_state_store() {
        let backend = Arc::new(MemoryBackend::new());
        let store = StateStore::with_backend("test_app", backend);

        let state = TestState {
            value: 42,
            name: "test".to_string(),
        };

        // Save
        store.save(&state).await.unwrap();

        // Load
        let loaded: TestState = store.load().await.unwrap();
        assert_eq!(loaded.value, 42);
        assert_eq!(loaded.name, "test");

        // Delete
        assert!(store.delete::<TestState>().await.unwrap());
        assert!(!store.exists::<TestState>().await.unwrap());
    }
}
