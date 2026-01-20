//! State debugging and inspection tools.
//!
//! This module provides tools for inspecting, debugging, and monitoring
//! application state during development.

use crate::error::StateResult;
use crate::persistence::PersistenceBackend;
use crate::state::{StateId, StateMetadata};
use crate::tier::PersistenceTier;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// State inspection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateInspection {
    /// State ID.
    pub id: StateId,
    /// State metadata.
    pub metadata: StateMetadata,
    /// Parsed JSON data.
    pub data: Value,
    /// Data size in bytes.
    pub size_bytes: usize,
    /// Inspection timestamp.
    pub inspected_at: DateTime<Utc>,
}

impl StateInspection {
    /// Get a summary string.
    pub fn summary(&self) -> String {
        format!(
            "{} (v{}, {}, {} bytes)",
            self.id,
            self.metadata.version,
            self.metadata.tier,
            self.size_bytes
        )
    }

    /// Check if this is a large state.
    pub fn is_large(&self, threshold: usize) -> bool {
        self.size_bytes > threshold
    }

    /// Get the number of top-level fields.
    pub fn field_count(&self) -> usize {
        match &self.data {
            Value::Object(map) => map.len(),
            Value::Array(arr) => arr.len(),
            _ => 1,
        }
    }

    /// Get top-level field names.
    pub fn field_names(&self) -> Vec<String> {
        match &self.data {
            Value::Object(map) => map.keys().cloned().collect(),
            _ => Vec::new(),
        }
    }

    /// Get a specific field value.
    pub fn get_field(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.data;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    let index: usize = part.parse().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }
}

/// State change event for monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// State ID.
    pub state_id: StateId,
    /// Change type.
    pub change_type: ChangeType,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Previous value (if available).
    pub previous: Option<Value>,
    /// New value (if available).
    pub current: Option<Value>,
    /// Changed fields (for updates).
    pub changed_fields: Vec<String>,
}

/// Type of state change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// State was created.
    Created,
    /// State was updated.
    Updated,
    /// State was deleted.
    Deleted,
    /// State was loaded from storage.
    Loaded,
    /// State was saved to storage.
    Saved,
}

/// State inspector for examining stored states.
pub struct StateInspector {
    /// Backend to inspect.
    backend: Arc<dyn PersistenceBackend>,
}

impl StateInspector {
    /// Create a new state inspector.
    pub fn new(backend: Arc<dyn PersistenceBackend>) -> Self {
        Self { backend }
    }

    /// Inspect a specific state.
    pub async fn inspect(&self, id: &StateId) -> StateResult<Option<StateInspection>> {
        let stored = match self.backend.load(id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let data: Value = serde_json::from_str(&stored.data)?;

        Ok(Some(StateInspection {
            id: id.clone(),
            metadata: stored.metadata,
            data,
            size_bytes: stored.data.len(),
            inspected_at: Utc::now(),
        }))
    }

    /// Inspect all states.
    pub async fn inspect_all(&self) -> StateResult<Vec<StateInspection>> {
        let ids = self.backend.list().await?;
        let mut inspections = Vec::new();

        for id in ids {
            if let Some(inspection) = self.inspect(&id).await? {
                inspections.push(inspection);
            }
        }

        Ok(inspections)
    }

    /// Get a summary of all states.
    pub async fn summary(&self) -> StateResult<StoreSummary> {
        let inspections = self.inspect_all().await?;

        let total_size: usize = inspections.iter().map(|i| i.size_bytes).sum();
        let mut by_tier: HashMap<PersistenceTier, Vec<StateId>> = HashMap::new();

        for inspection in &inspections {
            by_tier
                .entry(inspection.metadata.tier)
                .or_default()
                .push(inspection.id.clone());
        }

        Ok(StoreSummary {
            state_count: inspections.len(),
            total_size_bytes: total_size,
            by_tier,
            inspected_at: Utc::now(),
        })
    }

    /// Search for states matching a predicate.
    pub async fn search<F>(&self, predicate: F) -> StateResult<Vec<StateInspection>>
    where
        F: Fn(&StateInspection) -> bool,
    {
        let inspections = self.inspect_all().await?;
        Ok(inspections.into_iter().filter(predicate).collect())
    }

    /// Find states containing a specific field.
    pub async fn find_with_field(&self, field_path: &str) -> StateResult<Vec<StateInspection>> {
        self.search(|i| i.get_field(field_path).is_some()).await
    }

    /// Find states by tier.
    pub async fn find_by_tier(&self, tier: PersistenceTier) -> StateResult<Vec<StateInspection>> {
        self.search(|i| i.metadata.tier == tier).await
    }

    /// Find large states.
    pub async fn find_large(&self, threshold: usize) -> StateResult<Vec<StateInspection>> {
        self.search(|i| i.is_large(threshold)).await
    }

    /// Compare two states and return differences.
    pub async fn diff(&self, id1: &StateId, id2: &StateId) -> StateResult<StateDiff> {
        let s1 = self.inspect(id1).await?;
        let s2 = self.inspect(id2).await?;

        let diff = match (s1, s2) {
            (Some(a), Some(b)) => {
                let differences = diff_json(&a.data, &b.data, "");
                StateDiff {
                    state_a: Some(id1.clone()),
                    state_b: Some(id2.clone()),
                    differences,
                }
            }
            (Some(_), None) => StateDiff {
                state_a: Some(id1.clone()),
                state_b: None,
                differences: vec![DiffEntry {
                    path: "".to_string(),
                    diff_type: DiffType::Removed,
                    left: None,
                    right: None,
                }],
            },
            (None, Some(_)) => StateDiff {
                state_a: None,
                state_b: Some(id2.clone()),
                differences: vec![DiffEntry {
                    path: "".to_string(),
                    diff_type: DiffType::Added,
                    left: None,
                    right: None,
                }],
            },
            (None, None) => StateDiff {
                state_a: None,
                state_b: None,
                differences: Vec::new(),
            },
        };

        Ok(diff)
    }
}

/// Summary of the state store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSummary {
    /// Total number of states.
    pub state_count: usize,
    /// Total size in bytes.
    pub total_size_bytes: usize,
    /// States grouped by tier.
    pub by_tier: HashMap<PersistenceTier, Vec<StateId>>,
    /// Summary timestamp.
    pub inspected_at: DateTime<Utc>,
}

impl StoreSummary {
    /// Get a human-readable summary.
    pub fn to_string_pretty(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("State Store Summary\n"));
        s.push_str(&format!("==================\n"));
        s.push_str(&format!("Total states: {}\n", self.state_count));
        s.push_str(&format!("Total size: {} bytes\n", self.total_size_bytes));
        s.push_str(&format!("\nBy tier:\n"));

        for (tier, ids) in &self.by_tier {
            s.push_str(&format!("  {:?}: {} states\n", tier, ids.len()));
        }

        s
    }
}

/// Difference between two states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    /// First state ID.
    pub state_a: Option<StateId>,
    /// Second state ID.
    pub state_b: Option<StateId>,
    /// List of differences.
    pub differences: Vec<DiffEntry>,
}

impl StateDiff {
    /// Check if there are no differences.
    pub fn is_empty(&self) -> bool {
        self.differences.is_empty()
    }

    /// Get differences of a specific type.
    pub fn get_type(&self, diff_type: DiffType) -> Vec<&DiffEntry> {
        self.differences
            .iter()
            .filter(|d| d.diff_type == diff_type)
            .collect()
    }
}

/// A single difference entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    /// JSON path to the difference.
    pub path: String,
    /// Type of difference.
    pub diff_type: DiffType,
    /// Left (original) value.
    pub left: Option<Value>,
    /// Right (new) value.
    pub right: Option<Value>,
}

/// Type of difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    /// Value was added.
    Added,
    /// Value was removed.
    Removed,
    /// Value was modified.
    Modified,
}

/// Recursively diff two JSON values.
fn diff_json(left: &Value, right: &Value, path: &str) -> Vec<DiffEntry> {
    let mut diffs = Vec::new();

    match (left, right) {
        (Value::Object(l), Value::Object(r)) => {
            // Check for removed keys
            for key in l.keys() {
                if !r.contains_key(key) {
                    let p = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    diffs.push(DiffEntry {
                        path: p,
                        diff_type: DiffType::Removed,
                        left: l.get(key).cloned(),
                        right: None,
                    });
                }
            }

            // Check for added keys
            for key in r.keys() {
                if !l.contains_key(key) {
                    let p = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    diffs.push(DiffEntry {
                        path: p,
                        diff_type: DiffType::Added,
                        left: None,
                        right: r.get(key).cloned(),
                    });
                }
            }

            // Check for modified values
            for key in l.keys() {
                if let (Some(lv), Some(rv)) = (l.get(key), r.get(key)) {
                    let p = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    diffs.extend(diff_json(lv, rv, &p));
                }
            }
        }
        (Value::Array(l), Value::Array(r)) => {
            let max_len = l.len().max(r.len());
            for i in 0..max_len {
                let p = if path.is_empty() {
                    format!("[{}]", i)
                } else {
                    format!("{}[{}]", path, i)
                };

                match (l.get(i), r.get(i)) {
                    (Some(lv), Some(rv)) => {
                        diffs.extend(diff_json(lv, rv, &p));
                    }
                    (Some(lv), None) => {
                        diffs.push(DiffEntry {
                            path: p,
                            diff_type: DiffType::Removed,
                            left: Some(lv.clone()),
                            right: None,
                        });
                    }
                    (None, Some(rv)) => {
                        diffs.push(DiffEntry {
                            path: p,
                            diff_type: DiffType::Added,
                            left: None,
                            right: Some(rv.clone()),
                        });
                    }
                    (None, None) => {}
                }
            }
        }
        (l, r) if l != r => {
            diffs.push(DiffEntry {
                path: path.to_string(),
                diff_type: DiffType::Modified,
                left: Some(l.clone()),
                right: Some(r.clone()),
            });
        }
        _ => {}
    }

    diffs
}

/// State debugger with change tracking.
pub struct StateDebugger {
    /// Change history.
    history: Arc<RwLock<Vec<StateChange>>>,
    /// Maximum history size.
    max_history: usize,
    /// Whether tracking is enabled.
    enabled: Arc<RwLock<bool>>,
}

impl Default for StateDebugger {
    fn default() -> Self {
        Self::new()
    }
}

impl StateDebugger {
    /// Create a new state debugger.
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 1000,
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Set maximum history size.
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Enable or disable tracking.
    pub async fn set_enabled(&self, enabled: bool) {
        let mut e = self.enabled.write().await;
        *e = enabled;
    }

    /// Check if tracking is enabled.
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Record a state change.
    pub async fn record(&self, change: StateChange) {
        if !*self.enabled.read().await {
            return;
        }

        let mut history = self.history.write().await;
        history.push(change);

        // Trim if over limit
        if history.len() > self.max_history {
            let excess = history.len() - self.max_history;
            history.drain(0..excess);
        }

        debug!("Recorded state change");
    }

    /// Record a create event.
    pub async fn record_create(&self, state_id: StateId, value: Value) {
        self.record(StateChange {
            state_id,
            change_type: ChangeType::Created,
            timestamp: Utc::now(),
            previous: None,
            current: Some(value),
            changed_fields: Vec::new(),
        })
        .await;
    }

    /// Record an update event.
    pub async fn record_update(
        &self,
        state_id: StateId,
        previous: Value,
        current: Value,
    ) {
        let changed_fields = diff_json(&previous, &current, "")
            .into_iter()
            .map(|d| d.path)
            .collect();

        self.record(StateChange {
            state_id,
            change_type: ChangeType::Updated,
            timestamp: Utc::now(),
            previous: Some(previous),
            current: Some(current),
            changed_fields,
        })
        .await;
    }

    /// Record a delete event.
    pub async fn record_delete(&self, state_id: StateId, value: Value) {
        self.record(StateChange {
            state_id,
            change_type: ChangeType::Deleted,
            timestamp: Utc::now(),
            previous: Some(value),
            current: None,
            changed_fields: Vec::new(),
        })
        .await;
    }

    /// Get all history.
    pub async fn get_history(&self) -> Vec<StateChange> {
        let history = self.history.read().await;
        history.clone()
    }

    /// Get history for a specific state.
    pub async fn get_state_history(&self, state_id: &StateId) -> Vec<StateChange> {
        let history = self.history.read().await;
        history
            .iter()
            .filter(|c| &c.state_id == state_id)
            .cloned()
            .collect()
    }

    /// Get recent changes.
    pub async fn get_recent(&self, count: usize) -> Vec<StateChange> {
        let history = self.history.read().await;
        history.iter().rev().take(count).cloned().collect()
    }

    /// Clear history.
    pub async fn clear(&self) {
        let mut history = self.history.write().await;
        history.clear();
        info!("Cleared state debugger history");
    }

    /// Export history to JSON.
    pub async fn export_json(&self) -> StateResult<String> {
        let history = self.history.read().await;
        Ok(serde_json::to_string_pretty(&*history)?)
    }

    /// Get statistics about changes.
    pub async fn statistics(&self) -> DebugStatistics {
        let history = self.history.read().await;

        let mut by_state: HashMap<StateId, usize> = HashMap::new();
        let mut by_type: HashMap<ChangeType, usize> = HashMap::new();

        for change in history.iter() {
            *by_state.entry(change.state_id.clone()).or_default() += 1;
            *by_type.entry(change.change_type).or_default() += 1;
        }

        DebugStatistics {
            total_changes: history.len(),
            changes_by_state: by_state,
            changes_by_type: by_type,
        }
    }
}

/// Statistics about state changes.
#[derive(Debug, Clone)]
pub struct DebugStatistics {
    /// Total number of changes recorded.
    pub total_changes: usize,
    /// Changes grouped by state ID.
    pub changes_by_state: HashMap<StateId, usize>,
    /// Changes grouped by type.
    pub changes_by_type: HashMap<ChangeType, usize>,
}

impl DebugStatistics {
    /// Get the most active states.
    pub fn most_active(&self, count: usize) -> Vec<(&StateId, usize)> {
        let mut states: Vec<_> = self.changes_by_state.iter().map(|(k, v)| (k, *v)).collect();
        states.sort_by(|a, b| b.1.cmp(&a.1));
        states.truncate(count);
        states
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::MemoryBackend;
    use serde_json::json;

    #[test]
    fn test_diff_json() {
        let left = json!({
            "name": "Alice",
            "age": 30,
            "address": {
                "city": "NYC"
            }
        });

        let right = json!({
            "name": "Alice",
            "age": 31,
            "address": {
                "city": "LA"
            },
            "email": "alice@example.com"
        });

        let diffs = diff_json(&left, &right, "");

        assert!(diffs.iter().any(|d| d.path == "age" && d.diff_type == DiffType::Modified));
        assert!(diffs.iter().any(|d| d.path == "address.city" && d.diff_type == DiffType::Modified));
        assert!(diffs.iter().any(|d| d.path == "email" && d.diff_type == DiffType::Added));
    }

    #[tokio::test]
    async fn test_state_debugger() {
        let debugger = StateDebugger::new();
        let state_id = StateId::new("test");

        debugger
            .record_create(state_id.clone(), json!({"value": 1}))
            .await;

        debugger
            .record_update(
                state_id.clone(),
                json!({"value": 1}),
                json!({"value": 2}),
            )
            .await;

        let history = debugger.get_history().await;
        assert_eq!(history.len(), 2);

        let state_history = debugger.get_state_history(&state_id).await;
        assert_eq!(state_history.len(), 2);

        let stats = debugger.statistics().await;
        assert_eq!(stats.total_changes, 2);
    }

    #[tokio::test]
    async fn test_state_inspector() {
        let backend = Arc::new(MemoryBackend::new());
        let inspector = StateInspector::new(backend.clone());

        // Add some test data
        let id = StateId::new("test");
        let stored = StoredState {
            metadata: StateMetadata {
                id: id.clone(),
                type_name: "Test".to_string(),
                tier: PersistenceTier::Local,
                version: 1,
                created_at: Utc::now(),
                modified_at: Utc::now(),
                content_hash: 0,
                custom: HashMap::new(),
            },
            data: r#"{"name": "test", "value": 42}"#.to_string(),
        };
        backend.save(&id, &stored).await.unwrap();

        // Inspect
        let inspection = inspector.inspect(&id).await.unwrap().unwrap();
        assert_eq!(inspection.id, id);
        assert_eq!(inspection.get_field("name"), Some(&json!("test")));
        assert_eq!(inspection.get_field("value"), Some(&json!(42)));
    }
}
