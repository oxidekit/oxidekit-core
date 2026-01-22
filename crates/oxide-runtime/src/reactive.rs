//! Reactive state management for OxideKit runtime
//!
//! Provides dynamic state storage, mutation operations, and change notification
//! for reactive UI updates.

use crate::events::{ActionValue, MutationOp};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Dynamic state value that can be stored and mutated
#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<StateValue>),
    Object(HashMap<String, StateValue>),
}

impl Default for StateValue {
    fn default() -> Self {
        StateValue::Null
    }
}

impl StateValue {
    /// Create a number value
    pub fn number(n: impl Into<f64>) -> Self {
        StateValue::Number(n.into())
    }

    /// Create a string value
    pub fn string(s: impl Into<String>) -> Self {
        StateValue::String(s.into())
    }

    /// Create a bool value
    pub fn bool(b: bool) -> Self {
        StateValue::Bool(b)
    }

    /// Try to get as f64
    pub fn as_number(&self) -> Option<f64> {
        match self {
            StateValue::Number(n) => Some(*n),
            StateValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            StateValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Try to get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            StateValue::Bool(b) => Some(*b),
            StateValue::Number(n) => Some(*n != 0.0),
            StateValue::String(s) => Some(!s.is_empty()),
            StateValue::Null => Some(false),
            _ => None,
        }
    }

    /// Try to get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            StateValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_string_value(&self) -> String {
        match self {
            StateValue::Null => "null".to_string(),
            StateValue::Bool(b) => b.to_string(),
            StateValue::Number(n) => n.to_string(),
            StateValue::String(s) => s.clone(),
            StateValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string_value()).collect();
                format!("[{}]", items.join(", "))
            }
            StateValue::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string_value()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
        }
    }
}

impl From<ActionValue> for StateValue {
    fn from(av: ActionValue) -> Self {
        match av {
            ActionValue::Number(n) => StateValue::Number(n),
            ActionValue::String(s) => StateValue::String(s),
            ActionValue::Bool(b) => StateValue::Bool(b),
        }
    }
}

/// Version counter for tracking state changes
static VERSION: AtomicU64 = AtomicU64::new(0);

/// Reactive state container
///
/// Stores dynamic key-value state and tracks changes for reactive updates.
#[derive(Default)]
pub struct ReactiveState {
    /// State values by key (e.g., "count", "user.name")
    values: HashMap<String, StateValue>,
    /// Version number - incremented on each change
    version: u64,
    /// Subscribers for specific keys (node IDs that depend on state)
    subscribers: HashMap<String, Vec<SubscriberId>>,
    /// Global change listeners
    change_listeners: Vec<Box<dyn Fn(&str, &StateValue) + Send + Sync>>,
}

impl std::fmt::Debug for ReactiveState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactiveState")
            .field("values", &self.values)
            .field("version", &self.version)
            .field("subscribers", &self.subscribers)
            .field("change_listeners", &format!("[{} listeners]", self.change_listeners.len()))
            .finish()
    }
}

/// Subscriber identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriberId(u64);

impl SubscriberId {
    /// Generate a new unique subscriber ID
    pub fn new() -> Self {
        Self(VERSION.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for SubscriberId {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactiveState {
    /// Create a new empty reactive state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create state with initial values
    pub fn with_values(values: HashMap<String, StateValue>) -> Self {
        Self {
            values,
            version: 0,
            subscribers: HashMap::new(),
            change_listeners: Vec::new(),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&StateValue> {
        // Handle dot notation for nested access
        if key.contains('.') {
            let parts: Vec<&str> = key.splitn(2, '.').collect();
            if let Some(StateValue::Object(obj)) = self.values.get(parts[0]) {
                return obj.get(parts[1]);
            }
            return None;
        }
        self.values.get(key)
    }

    /// Set a value by key
    pub fn set(&mut self, key: impl Into<String>, value: StateValue) {
        let key = key.into();
        self.values.insert(key.clone(), value.clone());
        self.version += 1;
        self.notify_change(&key, &value);
    }

    /// Apply a mutation operation to a field
    pub fn mutate(&mut self, field: &str, op: MutationOp, value: &ActionValue) -> bool {
        let current = self.values.get(field).cloned().unwrap_or(StateValue::Null);
        let new_value = match op {
            MutationOp::Set => StateValue::from(value.clone()),
            MutationOp::Add => {
                if let (Some(current_num), ActionValue::Number(delta)) = (current.as_number(), value) {
                    StateValue::Number(current_num + delta)
                } else if let (StateValue::String(s), ActionValue::String(suffix)) = (&current, value) {
                    StateValue::String(format!("{}{}", s, suffix))
                } else {
                    return false;
                }
            }
            MutationOp::Subtract => {
                if let (Some(current_num), ActionValue::Number(delta)) = (current.as_number(), value) {
                    StateValue::Number(current_num - delta)
                } else {
                    return false;
                }
            }
            MutationOp::Multiply => {
                if let (Some(current_num), ActionValue::Number(factor)) = (current.as_number(), value) {
                    StateValue::Number(current_num * factor)
                } else {
                    return false;
                }
            }
            MutationOp::Divide => {
                if let (Some(current_num), ActionValue::Number(divisor)) = (current.as_number(), value) {
                    if *divisor != 0.0 {
                        StateValue::Number(current_num / divisor)
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            MutationOp::Toggle => {
                if let Some(b) = current.as_bool() {
                    StateValue::Bool(!b)
                } else {
                    return false;
                }
            }
        };

        self.set(field, new_value);
        true
    }

    /// Subscribe to changes on a specific key
    pub fn subscribe(&mut self, key: impl Into<String>, subscriber: SubscriberId) {
        self.subscribers
            .entry(key.into())
            .or_default()
            .push(subscriber);
    }

    /// Unsubscribe from a key
    pub fn unsubscribe(&mut self, key: &str, subscriber: SubscriberId) {
        if let Some(subs) = self.subscribers.get_mut(key) {
            subs.retain(|s| *s != subscriber);
        }
    }

    /// Get subscribers for a key
    pub fn subscribers(&self, key: &str) -> &[SubscriberId] {
        self.subscribers.get(key).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get current version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Check if state has changed since a given version
    pub fn has_changed_since(&self, version: u64) -> bool {
        self.version > version
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }

    /// Iterate over all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &StateValue)> {
        self.values.iter()
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.values.clear();
        self.version += 1;
    }

    /// Add a global change listener
    pub fn on_change<F>(&mut self, listener: F)
    where
        F: Fn(&str, &StateValue) + Send + Sync + 'static,
    {
        self.change_listeners.push(Box::new(listener));
    }

    /// Notify listeners of a change
    fn notify_change(&self, key: &str, value: &StateValue) {
        for listener in &self.change_listeners {
            listener(key, value);
        }
    }

    /// Initialize state from a default schema
    ///
    /// Schema format:
    /// ```json
    /// {
    ///   "count": 0,
    ///   "user": { "name": "Guest" }
    /// }
    /// ```
    pub fn init_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let parsed: serde_json::Value = serde_json::from_str(json)?;
        self.init_from_json_value(&parsed);
        Ok(())
    }

    fn init_from_json_value(&mut self, value: &serde_json::Value) {
        if let serde_json::Value::Object(obj) = value {
            for (key, val) in obj {
                let state_val = json_to_state_value(val);
                self.values.insert(key.clone(), state_val);
            }
        }
    }
}

/// Convert JSON value to StateValue
fn json_to_state_value(value: &serde_json::Value) -> StateValue {
    match value {
        serde_json::Value::Null => StateValue::Null,
        serde_json::Value::Bool(b) => StateValue::Bool(*b),
        serde_json::Value::Number(n) => StateValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => StateValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            StateValue::Array(arr.iter().map(json_to_state_value).collect())
        }
        serde_json::Value::Object(obj) => StateValue::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), json_to_state_value(v)))
                .collect(),
        ),
    }
}

/// State binding for components
///
/// Represents a reference to a state value that can be read and mutated.
#[derive(Debug, Clone)]
pub struct StateBinding {
    /// The key in the state store
    pub key: String,
    /// Optional path within the value (for nested access)
    pub path: Option<String>,
}

impl StateBinding {
    /// Create a binding to a state key
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            path: None,
        }
    }

    /// Create a binding with a nested path
    pub fn with_path(key: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            path: Some(path.into()),
        }
    }

    /// Get the full key path
    pub fn full_key(&self) -> String {
        match &self.path {
            Some(p) => format!("{}.{}", self.key, p),
            None => self.key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_value_creation() {
        let num = StateValue::number(42);
        assert_eq!(num.as_number(), Some(42.0));

        let s = StateValue::string("hello");
        assert_eq!(s.as_string(), Some("hello"));

        let b = StateValue::bool(true);
        assert_eq!(b.as_bool(), Some(true));
    }

    #[test]
    fn test_reactive_state_set_get() {
        let mut state = ReactiveState::new();
        state.set("count", StateValue::number(0));

        let val = state.get("count");
        assert_eq!(val, Some(&StateValue::Number(0.0)));
    }

    #[test]
    fn test_mutation_add() {
        let mut state = ReactiveState::new();
        state.set("count", StateValue::number(5));

        state.mutate("count", MutationOp::Add, &ActionValue::Number(3.0));
        assert_eq!(state.get("count"), Some(&StateValue::Number(8.0)));
    }

    #[test]
    fn test_mutation_subtract() {
        let mut state = ReactiveState::new();
        state.set("count", StateValue::number(10));

        state.mutate("count", MutationOp::Subtract, &ActionValue::Number(4.0));
        assert_eq!(state.get("count"), Some(&StateValue::Number(6.0)));
    }

    #[test]
    fn test_mutation_multiply() {
        let mut state = ReactiveState::new();
        state.set("value", StateValue::number(7));

        state.mutate("value", MutationOp::Multiply, &ActionValue::Number(3.0));
        assert_eq!(state.get("value"), Some(&StateValue::Number(21.0)));
    }

    #[test]
    fn test_mutation_divide() {
        let mut state = ReactiveState::new();
        state.set("value", StateValue::number(20));

        state.mutate("value", MutationOp::Divide, &ActionValue::Number(4.0));
        assert_eq!(state.get("value"), Some(&StateValue::Number(5.0)));
    }

    #[test]
    fn test_mutation_toggle() {
        let mut state = ReactiveState::new();
        state.set("enabled", StateValue::bool(false));

        state.mutate("enabled", MutationOp::Toggle, &ActionValue::Bool(true));
        assert_eq!(state.get("enabled"), Some(&StateValue::Bool(true)));

        state.mutate("enabled", MutationOp::Toggle, &ActionValue::Bool(true));
        assert_eq!(state.get("enabled"), Some(&StateValue::Bool(false)));
    }

    #[test]
    fn test_mutation_set() {
        let mut state = ReactiveState::new();
        state.set("name", StateValue::string("old"));

        state.mutate("name", MutationOp::Set, &ActionValue::String("new".to_string()));
        assert_eq!(state.get("name"), Some(&StateValue::String("new".to_string())));
    }

    #[test]
    fn test_version_tracking() {
        let mut state = ReactiveState::new();
        assert_eq!(state.version(), 0);

        state.set("a", StateValue::number(1));
        assert_eq!(state.version(), 1);

        state.set("b", StateValue::number(2));
        assert_eq!(state.version(), 2);

        assert!(state.has_changed_since(0));
        assert!(state.has_changed_since(1));
        assert!(!state.has_changed_since(2));
    }

    #[test]
    fn test_init_from_json() {
        let mut state = ReactiveState::new();
        state.init_from_json(r#"{"count": 0, "name": "test", "active": true}"#).unwrap();

        assert_eq!(state.get("count"), Some(&StateValue::Number(0.0)));
        assert_eq!(state.get("name"), Some(&StateValue::String("test".to_string())));
        assert_eq!(state.get("active"), Some(&StateValue::Bool(true)));
    }

    #[test]
    fn test_string_concatenation() {
        let mut state = ReactiveState::new();
        state.set("text", StateValue::string("Hello"));

        state.mutate("text", MutationOp::Add, &ActionValue::String(" World".to_string()));
        assert_eq!(state.get("text"), Some(&StateValue::String("Hello World".to_string())));
    }

    #[test]
    fn test_subscriber() {
        let mut state = ReactiveState::new();
        let sub_id = SubscriberId::new();

        state.subscribe("count", sub_id);
        assert_eq!(state.subscribers("count").len(), 1);

        state.unsubscribe("count", sub_id);
        assert_eq!(state.subscribers("count").len(), 0);
    }
}
