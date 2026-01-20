//! State management for hot reload
//!
//! Captures and restores application state during hot reloads to preserve
//! user interactions like scroll position, form input, focus, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;
use thiserror::Error;

/// Errors that can occur during state management
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Failed to serialize state: {0}")]
    SerializeError(#[from] serde_json::Error),

    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    #[error("State version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u64, actual: u64 },

    #[error("State restoration failed: {0}")]
    RestorationFailed(String),
}

/// A snapshot of the entire application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Unique identifier for this snapshot
    pub id: String,
    /// Timestamp when the snapshot was created
    #[serde(with = "instant_serde")]
    pub timestamp: Instant,
    /// Version counter for conflict detection
    pub version: u64,
    /// Component states by component ID
    pub components: HashMap<String, ComponentState>,
    /// Global application state
    pub global: GlobalState,
    /// Focus path (component hierarchy leading to focused element)
    pub focus_path: Option<Vec<String>>,
}

mod instant_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Instant;

    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let elapsed = instant.elapsed().as_millis() as u64;
        elapsed.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _ = u64::deserialize(deserializer)?;
        Ok(Instant::now())
    }
}

impl StateSnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Instant::now(),
            version: 0,
            components: HashMap::new(),
            global: GlobalState::default(),
            focus_path: None,
        }
    }

    /// Get the number of components in this snapshot
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Estimate the size of this snapshot in bytes
    pub fn size_bytes(&self) -> usize {
        serde_json::to_string(self)
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

impl Default for StateSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// State of an individual component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentState {
    /// Component ID (should be stable across reloads)
    pub id: String,
    /// Component type name
    pub kind: String,
    /// Local state values
    pub local_state: HashMap<String, StateValue>,
    /// Whether this component has focus
    pub has_focus: bool,
    /// Scroll position (x, y)
    pub scroll_position: Option<(f32, f32)>,
    /// Selection state (for text inputs, lists, etc.)
    pub selection: Option<SelectionState>,
    /// Animation state
    pub animation: Option<AnimationState>,
    /// Custom state for component-specific data
    pub custom: Option<serde_json::Value>,
}

impl ComponentState {
    /// Create a new component state
    pub fn new(id: impl Into<String>, kind: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            local_state: HashMap::new(),
            has_focus: false,
            scroll_position: None,
            selection: None,
            animation: None,
            custom: None,
        }
    }

    /// Set a local state value
    pub fn set_state(&mut self, key: impl Into<String>, value: StateValue) {
        self.local_state.insert(key.into(), value);
    }

    /// Get a local state value
    pub fn get_state(&self, key: &str) -> Option<&StateValue> {
        self.local_state.get(key)
    }
}

/// A value in the state store
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StateValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<StateValue>),
    Object(HashMap<String, StateValue>),
}

impl StateValue {
    /// Convert to a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            StateValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Convert to a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            StateValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Convert to a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            StateValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl From<bool> for StateValue {
    fn from(v: bool) -> Self {
        StateValue::Bool(v)
    }
}

impl From<f64> for StateValue {
    fn from(v: f64) -> Self {
        StateValue::Number(v)
    }
}

impl From<String> for StateValue {
    fn from(v: String) -> Self {
        StateValue::String(v)
    }
}

impl From<&str> for StateValue {
    fn from(v: &str) -> Self {
        StateValue::String(v.to_string())
    }
}

/// Selection state for text inputs, lists, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionState {
    /// Start index of selection
    pub start: usize,
    /// End index of selection
    pub end: usize,
    /// Direction of selection
    pub direction: SelectionDirection,
}

/// Direction of text selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectionDirection {
    Forward,
    Backward,
    None,
}

/// Animation state for preserving animations across reloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationState {
    /// Animation name/identifier
    pub name: String,
    /// Current progress (0.0 - 1.0)
    pub progress: f32,
    /// Whether the animation is playing
    pub playing: bool,
    /// Animation direction
    pub direction: AnimationDirection,
}

/// Animation playback direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationDirection {
    Forward,
    Backward,
    Alternate,
}

/// Global application state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalState {
    /// Current route/path
    pub route: Option<String>,
    /// Query parameters
    pub query_params: HashMap<String, String>,
    /// Theme (light/dark/custom)
    pub theme: Option<String>,
    /// Locale/language
    pub locale: Option<String>,
    /// Custom global state
    pub custom: HashMap<String, StateValue>,
}

/// Difference between two state snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    /// Components that were added
    pub added: Vec<String>,
    /// Components that were removed
    pub removed: Vec<String>,
    /// Components that were modified
    pub modified: Vec<ComponentDiff>,
    /// Whether global state changed
    pub global_changed: bool,
}

/// Difference in a single component's state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDiff {
    pub id: String,
    pub changed_keys: Vec<String>,
}

/// State manager for capturing and restoring application state
pub struct StateManager {
    /// Current state snapshot
    current: Arc<RwLock<StateSnapshot>>,
    /// Previous snapshots for history/undo
    history: Arc<RwLock<Vec<StateSnapshot>>>,
    /// Maximum number of snapshots to keep
    max_history: usize,
    /// Version counter
    version: Arc<RwLock<u64>>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            current: Arc::new(RwLock::new(StateSnapshot::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 10,
            version: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new state manager with custom history size
    pub fn with_history_size(max_history: usize) -> Self {
        Self {
            current: Arc::new(RwLock::new(StateSnapshot::new())),
            history: Arc::new(RwLock::new(Vec::with_capacity(max_history))),
            max_history,
            version: Arc::new(RwLock::new(0)),
        }
    }

    /// Capture the current application state
    pub fn capture(&self) -> StateSnapshot {
        let mut version = self.version.write();
        *version += 1;

        let mut snapshot = self.current.read().clone();
        snapshot.version = *version;
        snapshot.timestamp = Instant::now();
        snapshot.id = uuid::Uuid::new_v4().to_string();

        // Store in history
        let mut history = self.history.write();
        if history.len() >= self.max_history {
            history.remove(0);
        }
        history.push(snapshot.clone());

        tracing::debug!(
            "Captured state snapshot {} with {} components",
            snapshot.id,
            snapshot.component_count()
        );

        snapshot
    }

    /// Register a component's state
    pub fn register_component(&self, state: ComponentState) {
        let mut current = self.current.write();
        current.components.insert(state.id.clone(), state);
    }

    /// Update a component's state
    pub fn update_component<F>(&self, component_id: &str, f: F) -> Result<(), StateError>
    where
        F: FnOnce(&mut ComponentState),
    {
        let mut current = self.current.write();
        let component = current
            .components
            .get_mut(component_id)
            .ok_or_else(|| StateError::ComponentNotFound(component_id.to_string()))?;
        f(component);
        Ok(())
    }

    /// Get a component's state
    pub fn get_component(&self, component_id: &str) -> Option<ComponentState> {
        self.current.read().components.get(component_id).cloned()
    }

    /// Update global state
    pub fn update_global<F>(&self, f: F)
    where
        F: FnOnce(&mut GlobalState),
    {
        let mut current = self.current.write();
        f(&mut current.global);
    }

    /// Restore state from a snapshot
    pub fn restore(&self, snapshot: StateSnapshot) -> Result<StateDiff, StateError> {
        let current_snapshot = self.current.read().clone();

        // Calculate diff
        let diff = Self::calculate_diff(&current_snapshot, &snapshot);

        // Update current state
        *self.current.write() = snapshot;

        tracing::info!(
            "Restored state: {} added, {} removed, {} modified",
            diff.added.len(),
            diff.removed.len(),
            diff.modified.len()
        );

        Ok(diff)
    }

    /// Calculate the difference between two snapshots
    pub fn calculate_diff(old: &StateSnapshot, new: &StateSnapshot) -> StateDiff {
        let old_ids: std::collections::HashSet<_> = old.components.keys().collect();
        let new_ids: std::collections::HashSet<_> = new.components.keys().collect();

        let added: Vec<_> = new_ids
            .difference(&old_ids)
            .map(|s| (*s).clone())
            .collect();

        let removed: Vec<_> = old_ids
            .difference(&new_ids)
            .map(|s| (*s).clone())
            .collect();

        let modified: Vec<_> = old_ids
            .intersection(&new_ids)
            .filter_map(|id| {
                let old_component = old.components.get(*id)?;
                let new_component = new.components.get(*id)?;

                let changed_keys: Vec<_> = old_component
                    .local_state
                    .keys()
                    .chain(new_component.local_state.keys())
                    .filter(|key| {
                        old_component.local_state.get(*key) != new_component.local_state.get(*key)
                    })
                    .cloned()
                    .collect();

                if changed_keys.is_empty()
                    && old_component.scroll_position == new_component.scroll_position
                    && old_component.has_focus == new_component.has_focus
                {
                    None
                } else {
                    Some(ComponentDiff {
                        id: (*id).clone(),
                        changed_keys,
                    })
                }
            })
            .collect();

        let global_changed = old.global.route != new.global.route
            || old.global.theme != new.global.theme
            || old.global.locale != new.global.locale;

        StateDiff {
            added,
            removed,
            modified,
            global_changed,
        }
    }

    /// Get the current focus path
    pub fn get_focus_path(&self) -> Option<Vec<String>> {
        self.current.read().focus_path.clone()
    }

    /// Set the focus path
    pub fn set_focus_path(&self, path: Option<Vec<String>>) {
        self.current.write().focus_path = path;
    }

    /// Clear a component's state
    pub fn clear_component(&self, component_id: &str) {
        self.current.write().components.remove(component_id);
    }

    /// Get the latest snapshot from history
    pub fn get_latest_snapshot(&self) -> Option<StateSnapshot> {
        self.history.read().last().cloned()
    }

    /// Get all snapshots in history
    pub fn get_history(&self) -> Vec<StateSnapshot> {
        self.history.read().clone()
    }

    /// Export current state as JSON
    pub fn export_json(&self) -> Result<String, StateError> {
        let snapshot = self.current.read();
        Ok(serde_json::to_string_pretty(&*snapshot)?)
    }

    /// Import state from JSON
    pub fn import_json(&self, json: &str) -> Result<(), StateError> {
        let snapshot: StateSnapshot = serde_json::from_str(json)?;
        *self.current.write() = snapshot;
        Ok(())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_basics() {
        let manager = StateManager::new();

        // Register a component
        let mut component = ComponentState::new("btn_1", "Button");
        component.set_state("pressed", StateValue::Bool(false));
        manager.register_component(component);

        // Update component state
        manager
            .update_component("btn_1", |c| {
                c.set_state("pressed", StateValue::Bool(true));
            })
            .unwrap();

        // Get component state
        let component = manager.get_component("btn_1").unwrap();
        assert_eq!(component.get_state("pressed").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_state_snapshot() {
        let manager = StateManager::new();

        let component = ComponentState::new("input_1", "TextInput");
        manager.register_component(component);

        let snapshot = manager.capture();
        assert_eq!(snapshot.component_count(), 1);
        assert!(snapshot.version > 0);
    }

    #[test]
    fn test_state_diff() {
        let mut old = StateSnapshot::new();
        let mut new = StateSnapshot::new();

        old.components
            .insert("a".to_string(), ComponentState::new("a", "Button"));
        old.components
            .insert("b".to_string(), ComponentState::new("b", "Button"));

        new.components
            .insert("b".to_string(), ComponentState::new("b", "Button"));
        new.components
            .insert("c".to_string(), ComponentState::new("c", "Button"));

        let diff = StateManager::calculate_diff(&old, &new);

        assert_eq!(diff.added, vec!["c".to_string()]);
        assert_eq!(diff.removed, vec!["a".to_string()]);
    }
}
