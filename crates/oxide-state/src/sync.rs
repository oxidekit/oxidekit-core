//! State synchronization utilities.
//!
//! This module provides utilities for synchronizing state between
//! local storage and remote sources, supporting offline-first patterns.

use crate::error::{StateError, StateResult};
use crate::persistence::{PersistenceBackend, StoredState};
use crate::state::{StateId, StateMetadata};
use crate::tier::PersistenceTier;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info, warn};

/// Synchronization status for a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// State is in sync with remote.
    Synced,
    /// State has local changes pending upload.
    LocalPending,
    /// State has remote changes pending download.
    RemotePending,
    /// State has conflicts between local and remote.
    Conflict,
    /// State has never been synced.
    NeverSynced,
    /// Sync is in progress.
    Syncing,
    /// Sync failed.
    SyncFailed,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::NeverSynced
    }
}

/// Metadata for tracking synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    /// State ID.
    pub state_id: StateId,
    /// Current sync status.
    pub status: SyncStatus,
    /// Last successful sync time.
    pub last_synced_at: Option<DateTime<Utc>>,
    /// Local version (monotonically increasing).
    pub local_version: u64,
    /// Remote version from last sync.
    pub remote_version: Option<u64>,
    /// Content hash at last sync.
    pub synced_hash: Option<u64>,
    /// Number of failed sync attempts.
    pub failed_attempts: u32,
    /// Last sync error message.
    pub last_error: Option<String>,
}

impl SyncMetadata {
    /// Create new sync metadata for a state.
    pub fn new(state_id: StateId) -> Self {
        Self {
            state_id,
            status: SyncStatus::NeverSynced,
            last_synced_at: None,
            local_version: 0,
            remote_version: None,
            synced_hash: None,
            failed_attempts: 0,
            last_error: None,
        }
    }

    /// Mark as synced.
    pub fn mark_synced(&mut self, remote_version: u64, content_hash: u64) {
        self.status = SyncStatus::Synced;
        self.last_synced_at = Some(Utc::now());
        self.remote_version = Some(remote_version);
        self.synced_hash = Some(content_hash);
        self.failed_attempts = 0;
        self.last_error = None;
    }

    /// Mark as having local changes.
    pub fn mark_local_pending(&mut self) {
        if self.status != SyncStatus::Conflict {
            self.status = SyncStatus::LocalPending;
        }
        self.local_version += 1;
    }

    /// Mark as having remote changes.
    pub fn mark_remote_pending(&mut self) {
        if self.status != SyncStatus::Conflict && self.status != SyncStatus::LocalPending {
            self.status = SyncStatus::RemotePending;
        } else if self.status == SyncStatus::LocalPending {
            self.status = SyncStatus::Conflict;
        }
    }

    /// Mark sync as failed.
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.status = SyncStatus::SyncFailed;
        self.failed_attempts += 1;
        self.last_error = Some(error.into());
    }

    /// Check if sync is needed.
    pub fn needs_sync(&self) -> bool {
        matches!(
            self.status,
            SyncStatus::LocalPending
                | SyncStatus::RemotePending
                | SyncStatus::NeverSynced
                | SyncStatus::Conflict
        )
    }

    /// Check if there's a conflict.
    pub fn has_conflict(&self) -> bool {
        self.status == SyncStatus::Conflict
    }
}

/// Conflict resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Keep local version, discard remote.
    KeepLocal,
    /// Keep remote version, discard local.
    KeepRemote,
    /// Merge local and remote (requires merge function).
    Merge,
    /// Fail and require manual resolution.
    Fail,
}

impl Default for ConflictResolution {
    fn default() -> Self {
        Self::Fail
    }
}

/// Event emitted during sync operations.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Sync started for a state.
    Started(StateId),
    /// Sync completed successfully.
    Completed(StateId),
    /// Sync failed.
    Failed(StateId, String),
    /// Conflict detected.
    Conflict(StateId),
    /// Conflict resolved.
    ConflictResolved(StateId, ConflictResolution),
    /// State updated from remote.
    RemoteUpdate(StateId),
    /// State uploaded to remote.
    Uploaded(StateId),
}

/// Trait for remote sync providers.
///
/// Implement this trait to add support for syncing to different backends
/// (cloud storage, servers, etc.).
#[async_trait::async_trait]
pub trait SyncProvider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &str;

    /// Check if the provider is available (e.g., network connected).
    async fn is_available(&self) -> bool;

    /// Fetch remote state.
    async fn fetch(&self, id: &StateId) -> StateResult<Option<RemoteState>>;

    /// Push local state to remote.
    async fn push(&self, id: &StateId, state: &StoredState) -> StateResult<u64>;

    /// Delete remote state.
    async fn delete(&self, id: &StateId) -> StateResult<bool>;

    /// List all remote state IDs.
    async fn list(&self) -> StateResult<Vec<StateId>>;

    /// Get remote version without fetching full state.
    async fn get_version(&self, id: &StateId) -> StateResult<Option<u64>>;
}

/// Remote state representation.
#[derive(Debug, Clone)]
pub struct RemoteState {
    /// The state data.
    pub data: String,
    /// Remote version.
    pub version: u64,
    /// Last modified time.
    pub modified_at: DateTime<Utc>,
    /// Content hash.
    pub content_hash: u64,
}

/// State synchronization service.
pub struct StateSync {
    /// Local persistence backend.
    local: Arc<dyn PersistenceBackend>,
    /// Remote sync provider (optional).
    remote: Option<Arc<dyn SyncProvider>>,
    /// Sync metadata for tracked states.
    metadata: Arc<RwLock<HashMap<StateId, SyncMetadata>>>,
    /// Default conflict resolution strategy.
    default_resolution: ConflictResolution,
    /// Event sender.
    event_tx: broadcast::Sender<SyncEvent>,
    /// Pending operations queue (for future background sync).
    #[allow(dead_code)]
    pending_tx: mpsc::Sender<SyncOperation>,
}

/// A pending sync operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SyncOperation {
    state_id: StateId,
    operation: SyncOpType,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum SyncOpType {
    Upload,
    Download,
    Delete,
}

impl StateSync {
    /// Create a new state sync service.
    pub fn new(local: Arc<dyn PersistenceBackend>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        let (pending_tx, _) = mpsc::channel(1000);

        Self {
            local,
            remote: None,
            metadata: Arc::new(RwLock::new(HashMap::new())),
            default_resolution: ConflictResolution::default(),
            event_tx,
            pending_tx,
        }
    }

    /// Set the remote sync provider.
    pub fn with_remote(mut self, remote: Arc<dyn SyncProvider>) -> Self {
        self.remote = Some(remote);
        self
    }

    /// Set the default conflict resolution strategy.
    pub fn with_conflict_resolution(mut self, resolution: ConflictResolution) -> Self {
        self.default_resolution = resolution;
        self
    }

    /// Subscribe to sync events.
    pub fn subscribe(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_tx.subscribe()
    }

    /// Get sync metadata for a state.
    pub async fn get_metadata(&self, id: &StateId) -> Option<SyncMetadata> {
        let metadata = self.metadata.read().await;
        metadata.get(id).cloned()
    }

    /// Get sync status for a state.
    pub async fn get_status(&self, id: &StateId) -> SyncStatus {
        let metadata = self.metadata.read().await;
        metadata
            .get(id)
            .map(|m| m.status)
            .unwrap_or(SyncStatus::NeverSynced)
    }

    /// Register a state for synchronization.
    pub async fn register(&self, id: StateId) {
        let mut metadata = self.metadata.write().await;
        if !metadata.contains_key(&id) {
            metadata.insert(id.clone(), SyncMetadata::new(id));
        }
    }

    /// Mark a state as having local changes.
    pub async fn mark_changed(&self, id: &StateId) {
        let mut metadata = self.metadata.write().await;
        if let Some(meta) = metadata.get_mut(id) {
            meta.mark_local_pending();
            debug!("Marked {} as having local changes", id);
        }
    }

    /// Sync a specific state.
    pub async fn sync(&self, id: &StateId) -> StateResult<SyncStatus> {
        let provider = match &self.remote {
            Some(r) => r.clone(),
            None => return Ok(SyncStatus::NeverSynced),
        };

        if !provider.is_available().await {
            return Err(StateError::SyncError("Remote provider not available".into()));
        }

        // Emit started event
        let _ = self.event_tx.send(SyncEvent::Started(id.clone()));

        // Get local state
        let local_state = self.local.load(id).await?;

        // Get remote state
        let remote_state = provider.fetch(id).await?;

        // Get current metadata
        let mut metadata = self.metadata.write().await;
        let meta = metadata.entry(id.clone()).or_insert_with(|| SyncMetadata::new(id.clone()));

        let result = match (local_state, remote_state) {
            // Both exist - check for conflicts
            (Some(local), Some(remote_data)) => {
                self.handle_both_exist(id, local, remote_data, meta, provider.clone()).await
            }
            // Only local exists - upload
            (Some(local), None) => {
                self.handle_local_only(id, local, meta, provider.clone()).await
            }
            // Only remote exists - download
            (None, Some(remote_data)) => {
                self.handle_remote_only(id, remote_data, meta).await
            }
            // Neither exists
            (None, None) => {
                meta.status = SyncStatus::Synced;
                Ok(SyncStatus::Synced)
            }
        };

        match &result {
            Ok(status) => {
                let _ = self.event_tx.send(SyncEvent::Completed(id.clone()));
                debug!("Sync completed for {}: {:?}", id, status);
            }
            Err(e) => {
                meta.mark_failed(e.to_string());
                let _ = self.event_tx.send(SyncEvent::Failed(id.clone(), e.to_string()));
                warn!("Sync failed for {}: {}", id, e);
            }
        }

        result
    }

    /// Handle sync when both local and remote exist.
    async fn handle_both_exist(
        &self,
        id: &StateId,
        local: StoredState,
        remote: RemoteState,
        meta: &mut SyncMetadata,
        provider: Arc<dyn SyncProvider>,
    ) -> StateResult<SyncStatus> {
        let local_hash = seahash::hash(local.data.as_bytes());
        let remote_hash = remote.content_hash;

        // If hashes match, we're in sync
        if local_hash == remote_hash {
            meta.mark_synced(remote.version, local_hash);
            return Ok(SyncStatus::Synced);
        }

        // Check if local has changes since last sync
        let local_changed = meta
            .synced_hash
            .map(|h| h != local_hash)
            .unwrap_or(true);

        // Check if remote has changes since last sync
        let remote_changed = meta
            .remote_version
            .map(|v| v != remote.version)
            .unwrap_or(true);

        match (local_changed, remote_changed) {
            // Only local changed - upload
            (true, false) => {
                let new_version = provider.push(id, &local).await?;
                meta.mark_synced(new_version, local_hash);
                let _ = self.event_tx.send(SyncEvent::Uploaded(id.clone()));
                Ok(SyncStatus::Synced)
            }
            // Only remote changed - download
            (false, true) => {
                let new_stored = StoredState {
                    metadata: local.metadata.clone(),
                    data: remote.data,
                };
                self.local.save(id, &new_stored).await?;
                meta.mark_synced(remote.version, remote_hash);
                let _ = self.event_tx.send(SyncEvent::RemoteUpdate(id.clone()));
                Ok(SyncStatus::Synced)
            }
            // Both changed - conflict
            (true, true) => {
                meta.status = SyncStatus::Conflict;
                let _ = self.event_tx.send(SyncEvent::Conflict(id.clone()));

                match self.default_resolution {
                    ConflictResolution::KeepLocal => {
                        let new_version = provider.push(id, &local).await?;
                        meta.mark_synced(new_version, local_hash);
                        let _ = self.event_tx.send(SyncEvent::ConflictResolved(
                            id.clone(),
                            ConflictResolution::KeepLocal,
                        ));
                        Ok(SyncStatus::Synced)
                    }
                    ConflictResolution::KeepRemote => {
                        let new_stored = StoredState {
                            metadata: local.metadata.clone(),
                            data: remote.data,
                        };
                        self.local.save(id, &new_stored).await?;
                        meta.mark_synced(remote.version, remote_hash);
                        let _ = self.event_tx.send(SyncEvent::ConflictResolved(
                            id.clone(),
                            ConflictResolution::KeepRemote,
                        ));
                        Ok(SyncStatus::Synced)
                    }
                    ConflictResolution::Fail => {
                        Err(StateError::SyncError("Conflict requires manual resolution".into()))
                    }
                    ConflictResolution::Merge => {
                        Err(StateError::SyncError("Merge not implemented".into()))
                    }
                }
            }
            // Neither changed - already in sync
            (false, false) => {
                meta.status = SyncStatus::Synced;
                Ok(SyncStatus::Synced)
            }
        }
    }

    /// Handle sync when only local exists.
    async fn handle_local_only(
        &self,
        id: &StateId,
        local: StoredState,
        meta: &mut SyncMetadata,
        provider: Arc<dyn SyncProvider>,
    ) -> StateResult<SyncStatus> {
        let local_hash = seahash::hash(local.data.as_bytes());
        let new_version = provider.push(id, &local).await?;
        meta.mark_synced(new_version, local_hash);
        let _ = self.event_tx.send(SyncEvent::Uploaded(id.clone()));
        Ok(SyncStatus::Synced)
    }

    /// Handle sync when only remote exists.
    async fn handle_remote_only(
        &self,
        id: &StateId,
        remote: RemoteState,
        meta: &mut SyncMetadata,
    ) -> StateResult<SyncStatus> {
        // Create local state from remote
        let stored = StoredState {
            metadata: StateMetadata {
                id: id.clone(),
                type_name: "unknown".to_string(),
                tier: PersistenceTier::Syncable,
                version: 1,
                created_at: remote.modified_at,
                modified_at: remote.modified_at,
                content_hash: remote.content_hash,
                custom: HashMap::new(),
            },
            data: remote.data,
        };

        self.local.save(id, &stored).await?;
        meta.mark_synced(remote.version, remote.content_hash);
        let _ = self.event_tx.send(SyncEvent::RemoteUpdate(id.clone()));
        Ok(SyncStatus::Synced)
    }

    /// Sync all registered states.
    pub async fn sync_all(&self) -> StateResult<HashMap<StateId, SyncStatus>> {
        let mut results = HashMap::new();

        let ids: Vec<StateId> = {
            let metadata = self.metadata.read().await;
            metadata.keys().cloned().collect()
        };

        for id in ids {
            let status = self.sync(&id).await.unwrap_or(SyncStatus::SyncFailed);
            results.insert(id, status);
        }

        Ok(results)
    }

    /// Get all states that need syncing.
    pub async fn pending_sync(&self) -> Vec<StateId> {
        let metadata = self.metadata.read().await;
        metadata
            .iter()
            .filter(|(_, m)| m.needs_sync())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all states with conflicts.
    pub async fn conflicts(&self) -> Vec<StateId> {
        let metadata = self.metadata.read().await;
        metadata
            .iter()
            .filter(|(_, m)| m.has_conflict())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Resolve a conflict manually.
    pub async fn resolve_conflict(
        &self,
        id: &StateId,
        resolution: ConflictResolution,
    ) -> StateResult<()> {
        let remote = match &self.remote {
            Some(r) => r,
            None => return Err(StateError::SyncError("No remote provider".into())),
        };

        let local_state = self.local.load(id).await?;
        let remote_state = remote.fetch(id).await?;

        let mut metadata = self.metadata.write().await;
        let meta = metadata
            .get_mut(id)
            .ok_or_else(|| StateError::not_found(id.as_str()))?;

        match resolution {
            ConflictResolution::KeepLocal => {
                if let Some(local) = local_state {
                    let local_hash = seahash::hash(local.data.as_bytes());
                    let new_version = remote.push(id, &local).await?;
                    meta.mark_synced(new_version, local_hash);
                }
            }
            ConflictResolution::KeepRemote => {
                if let (Some(local), Some(remote_data)) = (local_state, remote_state) {
                    let new_stored = StoredState {
                        metadata: local.metadata.clone(),
                        data: remote_data.data,
                    };
                    self.local.save(id, &new_stored).await?;
                    meta.mark_synced(remote_data.version, remote_data.content_hash);
                }
            }
            ConflictResolution::Fail => {
                return Err(StateError::SyncError("Cannot resolve with Fail strategy".into()));
            }
            ConflictResolution::Merge => {
                return Err(StateError::SyncError("Merge not implemented".into()));
            }
        }

        let _ = self.event_tx.send(SyncEvent::ConflictResolved(id.clone(), resolution));
        Ok(())
    }
}

/// Offline-first state manager.
///
/// Wraps state operations to ensure they work offline and queue
/// sync operations for when connectivity is restored.
pub struct OfflineFirstManager {
    sync: Arc<StateSync>,
    /// Whether we're currently online.
    online: Arc<RwLock<bool>>,
}

impl OfflineFirstManager {
    /// Create a new offline-first manager.
    pub fn new(sync: Arc<StateSync>) -> Self {
        Self {
            sync,
            online: Arc::new(RwLock::new(true)),
        }
    }

    /// Set online status.
    pub async fn set_online(&self, online: bool) {
        let mut status = self.online.write().await;
        *status = online;

        if online {
            // Trigger sync of pending changes
            info!("Back online, syncing pending changes");
            let _ = self.sync.sync_all().await;
        }
    }

    /// Check if online.
    pub async fn is_online(&self) -> bool {
        *self.online.read().await
    }

    /// Save state (works offline).
    pub async fn save(
        &self,
        id: &StateId,
        state: &StoredState,
    ) -> StateResult<()> {
        // Always save locally first
        self.sync.local.save(id, state).await?;

        // Mark as having local changes
        self.sync.mark_changed(id).await;

        // Try to sync if online
        let online = *self.online.read().await;
        if online {
            let _ = self.sync.sync(id).await;
        }

        Ok(())
    }

    /// Load state (works offline).
    pub async fn load(&self, id: &StateId) -> StateResult<Option<StoredState>> {
        // Try to sync first if online
        let online = *self.online.read().await;
        if online {
            let _ = self.sync.sync(id).await;
        }

        // Always return local state
        self.sync.local.load(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::MemoryBackend;

    #[tokio::test]
    async fn test_sync_metadata() {
        let mut meta = SyncMetadata::new(StateId::new("test"));
        assert_eq!(meta.status, SyncStatus::NeverSynced);
        assert!(meta.needs_sync());

        meta.mark_local_pending();
        assert_eq!(meta.status, SyncStatus::LocalPending);
        assert!(meta.needs_sync());

        meta.mark_synced(1, 12345);
        assert_eq!(meta.status, SyncStatus::Synced);
        assert!(!meta.needs_sync());
    }

    #[tokio::test]
    async fn test_sync_registration() {
        let backend = Arc::new(MemoryBackend::new());
        let sync = StateSync::new(backend);

        let id = StateId::new("test");
        sync.register(id.clone()).await;

        let meta = sync.get_metadata(&id).await;
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().status, SyncStatus::NeverSynced);
    }

    #[tokio::test]
    async fn test_mark_changed() {
        let backend = Arc::new(MemoryBackend::new());
        let sync = StateSync::new(backend);

        let id = StateId::new("test");
        sync.register(id.clone()).await;
        sync.mark_changed(&id).await;

        let status = sync.get_status(&id).await;
        assert_eq!(status, SyncStatus::LocalPending);
    }
}
