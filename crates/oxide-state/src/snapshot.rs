//! State snapshot system for hot reload preservation.
//!
//! This module provides mechanisms to capture and restore application state
//! during hot reload cycles, ensuring a seamless development experience.

use crate::error::StateResult;
use crate::state::{StateId, StateMetadata};
use crate::tier::PersistenceTier;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// A captured snapshot of application state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Unique identifier for this snapshot.
    pub id: String,
    /// When the snapshot was created.
    pub created_at: DateTime<Utc>,
    /// Application version that created this snapshot.
    pub app_version: String,
    /// Serialized state data keyed by state ID.
    pub states: HashMap<StateId, SnapshotEntry>,
    /// Additional snapshot metadata.
    pub metadata: HashMap<String, String>,
}

/// A single entry in a state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    /// The state's metadata.
    pub metadata: StateMetadata,
    /// Serialized JSON data.
    pub data: String,
    /// Whether this entry is from UI state (vs app state).
    pub is_ui_state: bool,
}

impl StateSnapshot {
    /// Create a new empty snapshot.
    pub fn new(app_version: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            app_version: app_version.into(),
            states: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a state entry to the snapshot.
    pub fn add_entry(&mut self, id: StateId, entry: SnapshotEntry) {
        self.states.insert(id, entry);
    }

    /// Get a state entry by ID.
    pub fn get_entry(&self, id: &StateId) -> Option<&SnapshotEntry> {
        self.states.get(id)
    }

    /// Remove a state entry.
    pub fn remove_entry(&mut self, id: &StateId) -> Option<SnapshotEntry> {
        self.states.remove(id)
    }

    /// Check if the snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Get the number of state entries.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Get all state IDs in this snapshot.
    pub fn state_ids(&self) -> Vec<StateId> {
        self.states.keys().cloned().collect()
    }

    /// Check if a state exists in the snapshot.
    pub fn contains(&self, id: &StateId) -> bool {
        self.states.contains_key(id)
    }

    /// Add metadata to the snapshot.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Serialize the snapshot to JSON.
    pub fn to_json(&self) -> StateResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Serialize the snapshot to compact JSON.
    pub fn to_json_compact(&self) -> StateResult<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Deserialize a snapshot from JSON.
    pub fn from_json(json: &str) -> StateResult<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Calculate the total size of state data.
    pub fn data_size(&self) -> usize {
        self.states.values().map(|e| e.data.len()).sum()
    }
}

/// Manager for creating and restoring state snapshots.
///
/// The snapshot manager handles the lifecycle of state snapshots,
/// including periodic creation, storage, and restoration during hot reload.
pub struct SnapshotManager {
    /// Directory for storing snapshots.
    snapshot_dir: PathBuf,
    /// Current active snapshot.
    current: Arc<RwLock<Option<StateSnapshot>>>,
    /// Maximum number of snapshots to retain.
    max_snapshots: usize,
    /// Application version.
    app_version: String,
    /// Hot reload mode flag.
    hot_reload_enabled: bool,
}

impl SnapshotManager {
    /// Create a new snapshot manager.
    pub fn new(snapshot_dir: impl AsRef<Path>, app_version: impl Into<String>) -> Self {
        Self {
            snapshot_dir: snapshot_dir.as_ref().to_path_buf(),
            current: Arc::new(RwLock::new(None)),
            max_snapshots: 5,
            app_version: app_version.into(),
            hot_reload_enabled: false,
        }
    }

    /// Enable hot reload mode.
    pub fn enable_hot_reload(&mut self) {
        self.hot_reload_enabled = true;
        info!("Hot reload snapshot mode enabled");
    }

    /// Disable hot reload mode.
    pub fn disable_hot_reload(&mut self) {
        self.hot_reload_enabled = false;
        info!("Hot reload snapshot mode disabled");
    }

    /// Check if hot reload is enabled.
    pub fn is_hot_reload_enabled(&self) -> bool {
        self.hot_reload_enabled
    }

    /// Set maximum number of snapshots to retain.
    pub fn set_max_snapshots(&mut self, max: usize) {
        self.max_snapshots = max;
    }

    /// Create a new snapshot from serialized state data.
    pub async fn create_snapshot(
        &self,
        states: Vec<(StateId, StateMetadata, String, bool)>,
    ) -> StateResult<StateSnapshot> {
        let mut snapshot = StateSnapshot::new(&self.app_version);

        for (id, metadata, data, is_ui) in states {
            let entry = SnapshotEntry {
                metadata,
                data,
                is_ui_state: is_ui,
            };
            snapshot.add_entry(id, entry);
        }

        // Store as current
        let mut current = self.current.write().await;
        *current = Some(snapshot.clone());

        info!(
            "Created snapshot {} with {} states",
            snapshot.id,
            snapshot.len()
        );

        Ok(snapshot)
    }

    /// Save the current snapshot to disk.
    pub async fn save_snapshot(&self, snapshot: &StateSnapshot) -> StateResult<PathBuf> {
        // Ensure directory exists
        tokio::fs::create_dir_all(&self.snapshot_dir).await?;

        let filename = format!("snapshot_{}.json", snapshot.id);
        let path = self.snapshot_dir.join(&filename);

        let json = snapshot.to_json()?;
        tokio::fs::write(&path, json).await?;

        debug!("Saved snapshot to {}", path.display());

        // Cleanup old snapshots
        self.cleanup_old_snapshots().await?;

        Ok(path)
    }

    /// Load the most recent snapshot from disk.
    pub async fn load_latest_snapshot(&self) -> StateResult<Option<StateSnapshot>> {
        if !self.snapshot_dir.exists() {
            return Ok(None);
        }

        let mut entries = tokio::fs::read_dir(&self.snapshot_dir).await?;
        let mut snapshots: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(snapshot) = StateSnapshot::from_json(&content) {
                        snapshots.push((path, snapshot.created_at));
                    }
                }
            }
        }

        // Sort by creation time, most recent first
        snapshots.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((path, _)) = snapshots.first() {
            let content = tokio::fs::read_to_string(path).await?;
            let snapshot = StateSnapshot::from_json(&content)?;
            info!("Loaded snapshot {} from {}", snapshot.id, path.display());
            return Ok(Some(snapshot));
        }

        Ok(None)
    }

    /// Load a specific snapshot by ID.
    pub async fn load_snapshot(&self, id: &str) -> StateResult<Option<StateSnapshot>> {
        let filename = format!("snapshot_{}.json", id);
        let path = self.snapshot_dir.join(&filename);

        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let snapshot = StateSnapshot::from_json(&content)?;
        Ok(Some(snapshot))
    }

    /// Get the current in-memory snapshot.
    pub async fn get_current(&self) -> Option<StateSnapshot> {
        let current = self.current.read().await;
        current.clone()
    }

    /// Clear the current in-memory snapshot.
    pub async fn clear_current(&self) {
        let mut current = self.current.write().await;
        *current = None;
    }

    /// Delete a specific snapshot.
    pub async fn delete_snapshot(&self, id: &str) -> StateResult<bool> {
        let filename = format!("snapshot_{}.json", id);
        let path = self.snapshot_dir.join(&filename);

        if path.exists() {
            tokio::fs::remove_file(&path).await?;
            info!("Deleted snapshot {}", id);
            return Ok(true);
        }

        Ok(false)
    }

    /// List all available snapshots.
    pub async fn list_snapshots(&self) -> StateResult<Vec<SnapshotInfo>> {
        if !self.snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = tokio::fs::read_dir(&self.snapshot_dir).await?;
        let mut infos = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(snapshot) = StateSnapshot::from_json(&content) {
                        let metadata = entry.metadata().await?;
                        let state_count = snapshot.len();
                        infos.push(SnapshotInfo {
                            id: snapshot.id,
                            created_at: snapshot.created_at,
                            app_version: snapshot.app_version,
                            state_count,
                            file_size: metadata.len(),
                            path,
                        });
                    }
                }
            }
        }

        // Sort by creation time, most recent first
        infos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(infos)
    }

    /// Cleanup old snapshots, keeping only the most recent ones.
    async fn cleanup_old_snapshots(&self) -> StateResult<()> {
        let snapshots = self.list_snapshots().await?;

        if snapshots.len() > self.max_snapshots {
            let to_delete = &snapshots[self.max_snapshots..];
            for info in to_delete {
                if let Err(e) = tokio::fs::remove_file(&info.path).await {
                    warn!("Failed to delete old snapshot {}: {}", info.id, e);
                } else {
                    debug!("Cleaned up old snapshot {}", info.id);
                }
            }
        }

        Ok(())
    }

    /// Create a hot reload snapshot (in-memory only, fast).
    pub async fn create_hot_reload_snapshot(
        &self,
        states: Vec<(StateId, StateMetadata, String, bool)>,
    ) -> StateResult<()> {
        if !self.hot_reload_enabled {
            return Ok(());
        }

        let snapshot = self.create_snapshot(states).await?;
        debug!(
            "Created hot reload snapshot {} ({} states)",
            snapshot.id,
            snapshot.len()
        );
        Ok(())
    }

    /// Restore from hot reload snapshot.
    pub async fn restore_hot_reload_snapshot(&self) -> StateResult<Option<StateSnapshot>> {
        if !self.hot_reload_enabled {
            return Ok(None);
        }

        let current = self.current.read().await;
        if let Some(snapshot) = current.as_ref() {
            info!(
                "Restoring from hot reload snapshot {} ({} states)",
                snapshot.id,
                snapshot.len()
            );
            return Ok(Some(snapshot.clone()));
        }

        Ok(None)
    }
}

/// Information about a stored snapshot.
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    /// Snapshot ID.
    pub id: String,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// Application version.
    pub app_version: String,
    /// Number of state entries.
    pub state_count: usize,
    /// File size in bytes.
    pub file_size: u64,
    /// File path.
    pub path: PathBuf,
}

/// Builder for creating snapshots with specific options.
pub struct SnapshotBuilder {
    app_version: String,
    include_ui_state: bool,
    include_volatile: bool,
    filter_tiers: Option<Vec<PersistenceTier>>,
    metadata: HashMap<String, String>,
}

impl SnapshotBuilder {
    /// Create a new snapshot builder.
    pub fn new(app_version: impl Into<String>) -> Self {
        Self {
            app_version: app_version.into(),
            include_ui_state: true,
            include_volatile: false,
            filter_tiers: None,
            metadata: HashMap::new(),
        }
    }

    /// Include UI state in the snapshot.
    pub fn with_ui_state(mut self, include: bool) -> Self {
        self.include_ui_state = include;
        self
    }

    /// Include volatile state in the snapshot.
    pub fn with_volatile(mut self, include: bool) -> Self {
        self.include_volatile = include;
        self
    }

    /// Only include specific persistence tiers.
    pub fn with_tiers(mut self, tiers: Vec<PersistenceTier>) -> Self {
        self.filter_tiers = Some(tiers);
        self
    }

    /// Add metadata to the snapshot.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if an entry should be included based on filters.
    pub fn should_include(&self, metadata: &StateMetadata, is_ui: bool) -> bool {
        // Check UI state filter
        if is_ui && !self.include_ui_state {
            return false;
        }

        // Check volatile filter
        if metadata.tier == PersistenceTier::Volatile && !self.include_volatile {
            return false;
        }

        // Check tier filter
        if let Some(ref tiers) = self.filter_tiers {
            if !tiers.contains(&metadata.tier) {
                return false;
            }
        }

        true
    }

    /// Build the snapshot with the given state entries.
    pub fn build(self, entries: Vec<(StateId, StateMetadata, String, bool)>) -> StateSnapshot {
        let mut snapshot = StateSnapshot::new(self.app_version);

        // Filter entries based on builder settings
        let filter_tiers = self.filter_tiers;
        let include_ui_state = self.include_ui_state;
        let include_volatile = self.include_volatile;

        // Move metadata to snapshot
        snapshot.metadata = self.metadata;

        for (id, metadata, data, is_ui) in entries {
            // Apply filters inline since self is partially moved
            let should_include = {
                // Check UI state filter
                if is_ui && !include_ui_state {
                    false
                // Check volatile filter
                } else if metadata.tier == PersistenceTier::Volatile && !include_volatile {
                    false
                // Check tier filter
                } else if let Some(ref tiers) = filter_tiers {
                    tiers.contains(&metadata.tier)
                } else {
                    true
                }
            };

            if should_include {
                let entry = SnapshotEntry {
                    metadata,
                    data,
                    is_ui_state: is_ui,
                };
                snapshot.add_entry(id, entry);
            }
        }

        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = StateSnapshot::new("1.0.0");
        assert!(!snapshot.id.is_empty());
        assert_eq!(snapshot.app_version, "1.0.0");
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_snapshot_add_entry() {
        let mut snapshot = StateSnapshot::new("1.0.0");
        let id = StateId::new("test");
        let metadata = StateMetadata {
            id: id.clone(),
            type_name: "Test".to_string(),
            tier: PersistenceTier::Local,
            version: 1,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            content_hash: 0,
            custom: HashMap::new(),
        };
        let entry = SnapshotEntry {
            metadata,
            data: r#"{"value": 42}"#.to_string(),
            is_ui_state: false,
        };

        snapshot.add_entry(id.clone(), entry);
        assert_eq!(snapshot.len(), 1);
        assert!(snapshot.contains(&id));
    }

    #[test]
    fn test_snapshot_serialization() {
        let mut snapshot = StateSnapshot::new("1.0.0");
        let id = StateId::new("test");
        let metadata = StateMetadata {
            id: id.clone(),
            type_name: "Test".to_string(),
            tier: PersistenceTier::Local,
            version: 1,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            content_hash: 0,
            custom: HashMap::new(),
        };
        let entry = SnapshotEntry {
            metadata,
            data: r#"{"value": 42}"#.to_string(),
            is_ui_state: false,
        };
        snapshot.add_entry(id, entry);

        let json = snapshot.to_json().unwrap();
        let restored = StateSnapshot::from_json(&json).unwrap();

        assert_eq!(restored.id, snapshot.id);
        assert_eq!(restored.len(), 1);
    }

    #[test]
    fn test_snapshot_builder() {
        let id = StateId::new("test");
        let metadata = StateMetadata {
            id: id.clone(),
            type_name: "Test".to_string(),
            tier: PersistenceTier::Local,
            version: 1,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            content_hash: 0,
            custom: HashMap::new(),
        };

        let builder = SnapshotBuilder::new("1.0.0")
            .with_ui_state(false)
            .with_metadata("key", "value");

        // UI state should be excluded
        assert!(!builder.should_include(&metadata, true));
        // App state should be included
        assert!(builder.should_include(&metadata, false));
    }
}
