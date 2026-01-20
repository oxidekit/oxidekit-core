//! State migration system for version changes.
//!
//! This module provides a framework for migrating state data between schema versions,
//! ensuring backward compatibility as applications evolve.

mod registry;

pub use registry::*;

use crate::error::StateResult;
use crate::state::StateId;
use serde_json::Value;

/// A single migration step between two versions.
///
/// Migrations transform JSON state data from one version to the next.
/// They should be reversible when possible to support rollbacks.
pub trait Migration: Send + Sync {
    /// Get the source version this migration upgrades from.
    fn from_version(&self) -> u32;

    /// Get the target version this migration upgrades to.
    fn to_version(&self) -> u32;

    /// Get a description of what this migration does.
    fn description(&self) -> &str;

    /// Perform the forward migration.
    ///
    /// Transforms the state JSON from `from_version` to `to_version`.
    fn migrate_up(&self, data: Value) -> StateResult<Value>;

    /// Perform the reverse migration.
    ///
    /// Transforms the state JSON from `to_version` back to `from_version`.
    /// Returns `None` if the migration is not reversible.
    fn migrate_down(&self, _data: Value) -> StateResult<Option<Value>> {
        Ok(None)
    }

    /// Check if this migration is reversible.
    fn is_reversible(&self) -> bool {
        false
    }
}

/// A simple migration defined by closures.
pub struct SimpleMigration {
    from: u32,
    to: u32,
    description: String,
    up: Box<dyn Fn(Value) -> StateResult<Value> + Send + Sync>,
    down: Option<Box<dyn Fn(Value) -> StateResult<Value> + Send + Sync>>,
}

impl SimpleMigration {
    /// Create a new simple migration.
    pub fn new(
        from: u32,
        to: u32,
        description: impl Into<String>,
        up: impl Fn(Value) -> StateResult<Value> + Send + Sync + 'static,
    ) -> Self {
        Self {
            from,
            to,
            description: description.into(),
            up: Box::new(up),
            down: None,
        }
    }

    /// Add a reverse migration function.
    pub fn with_down(
        mut self,
        down: impl Fn(Value) -> StateResult<Value> + Send + Sync + 'static,
    ) -> Self {
        self.down = Some(Box::new(down));
        self
    }
}

impl Migration for SimpleMigration {
    fn from_version(&self) -> u32 {
        self.from
    }

    fn to_version(&self) -> u32 {
        self.to
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn migrate_up(&self, data: Value) -> StateResult<Value> {
        (self.up)(data)
    }

    fn migrate_down(&self, data: Value) -> StateResult<Option<Value>> {
        match &self.down {
            Some(down) => Ok(Some((down)(data)?)),
            None => Ok(None),
        }
    }

    fn is_reversible(&self) -> bool {
        self.down.is_some()
    }
}

/// Result of a migration operation.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// The state ID that was migrated.
    pub state_id: StateId,
    /// The starting version.
    pub from_version: u32,
    /// The final version.
    pub to_version: u32,
    /// Number of migration steps applied.
    pub steps_applied: usize,
    /// Whether all steps were reversible.
    pub fully_reversible: bool,
    /// The migrated data as JSON.
    pub data: Value,
}

/// A migration plan describing the steps needed to migrate between versions.
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    /// The state ID to migrate.
    pub state_id: StateId,
    /// The starting version.
    pub from_version: u32,
    /// The target version.
    pub to_version: u32,
    /// Whether this is an upgrade (forward) or downgrade (backward).
    pub is_upgrade: bool,
    /// The migration steps to apply.
    pub steps: Vec<MigrationStep>,
}

/// A single step in a migration plan.
#[derive(Debug, Clone)]
pub struct MigrationStep {
    /// Source version.
    pub from: u32,
    /// Target version.
    pub to: u32,
    /// Description of the step.
    pub description: String,
    /// Whether this step is reversible.
    pub reversible: bool,
}

impl MigrationPlan {
    /// Check if all steps in the plan are reversible.
    pub fn is_fully_reversible(&self) -> bool {
        self.steps.iter().all(|s| s.reversible)
    }

    /// Get the total number of steps.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

/// Builder for creating common migration patterns.
pub struct MigrationBuilder {
    from: u32,
    to: u32,
    description: String,
    transforms: Vec<Transform>,
}

/// A single transform operation in a migration.
enum Transform {
    RenameField { from: String, to: String },
    AddField { name: String, default: Value },
    RemoveField { name: String },
    TransformField { name: String, transform: Box<dyn Fn(Value) -> Value + Send + Sync> },
    Custom(Box<dyn Fn(Value) -> StateResult<Value> + Send + Sync>),
}

impl MigrationBuilder {
    /// Create a new migration builder.
    pub fn new(from: u32, to: u32, description: impl Into<String>) -> Self {
        Self {
            from,
            to,
            description: description.into(),
            transforms: Vec::new(),
        }
    }

    /// Rename a field.
    pub fn rename_field(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.transforms.push(Transform::RenameField {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    /// Add a new field with a default value.
    pub fn add_field(mut self, name: impl Into<String>, default: impl Into<Value>) -> Self {
        self.transforms.push(Transform::AddField {
            name: name.into(),
            default: default.into(),
        });
        self
    }

    /// Remove a field.
    pub fn remove_field(mut self, name: impl Into<String>) -> Self {
        self.transforms.push(Transform::RemoveField {
            name: name.into(),
        });
        self
    }

    /// Transform a field's value.
    pub fn transform_field(
        mut self,
        name: impl Into<String>,
        transform: impl Fn(Value) -> Value + Send + Sync + 'static,
    ) -> Self {
        self.transforms.push(Transform::TransformField {
            name: name.into(),
            transform: Box::new(transform),
        });
        self
    }

    /// Add a custom transformation.
    pub fn custom(
        mut self,
        transform: impl Fn(Value) -> StateResult<Value> + Send + Sync + 'static,
    ) -> Self {
        self.transforms.push(Transform::Custom(Box::new(transform)));
        self
    }

    /// Build the migration.
    pub fn build(self) -> BuiltMigration {
        BuiltMigration {
            from: self.from,
            to: self.to,
            description: self.description,
            transforms: self.transforms,
        }
    }
}

/// A migration built from the builder.
pub struct BuiltMigration {
    from: u32,
    to: u32,
    description: String,
    transforms: Vec<Transform>,
}

impl Migration for BuiltMigration {
    fn from_version(&self) -> u32 {
        self.from
    }

    fn to_version(&self) -> u32 {
        self.to
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn migrate_up(&self, mut data: Value) -> StateResult<Value> {
        for transform in &self.transforms {
            match transform {
                Transform::RenameField { from, to } => {
                    if let Value::Object(ref mut map) = data {
                        if let Some(value) = map.remove(from) {
                            map.insert(to.clone(), value);
                        }
                    }
                }
                Transform::AddField { name, default } => {
                    if let Value::Object(ref mut map) = data {
                        if !map.contains_key(name) {
                            map.insert(name.clone(), default.clone());
                        }
                    }
                }
                Transform::RemoveField { name } => {
                    if let Value::Object(ref mut map) = data {
                        map.remove(name);
                    }
                }
                Transform::TransformField { name, transform } => {
                    if let Value::Object(ref mut map) = data {
                        if let Some(value) = map.remove(name) {
                            map.insert(name.clone(), transform(value));
                        }
                    }
                }
                Transform::Custom(transform) => {
                    data = transform(data)?;
                }
            }
        }
        Ok(data)
    }
}

/// Helper functions for common migration operations.
pub mod helpers {
    use super::*;

    /// Wrap a value in an option if it's null.
    pub fn make_optional(value: Value) -> Value {
        if value.is_null() {
            Value::Null
        } else {
            value
        }
    }

    /// Convert a string field to an array of strings.
    pub fn string_to_array(value: Value) -> Value {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    Value::Array(vec![])
                } else {
                    Value::Array(vec![Value::String(s)])
                }
            }
            arr @ Value::Array(_) => arr,
            _ => Value::Array(vec![]),
        }
    }

    /// Convert an array of strings to a single string (first element or empty).
    pub fn array_to_string(value: Value) -> Value {
        match value {
            Value::Array(arr) => arr
                .into_iter()
                .next()
                .unwrap_or(Value::String(String::new())),
            s @ Value::String(_) => s,
            _ => Value::String(String::new()),
        }
    }

    /// Parse a string as JSON if possible.
    pub fn parse_json_string(value: Value) -> Value {
        match value {
            Value::String(s) => serde_json::from_str(&s).unwrap_or(Value::String(s)),
            other => other,
        }
    }

    /// Convert a value to a string representation.
    pub fn stringify(value: Value) -> Value {
        match value {
            s @ Value::String(_) => s,
            other => Value::String(other.to_string()),
        }
    }

    /// Deep merge two JSON objects.
    pub fn deep_merge(base: Value, overlay: Value) -> Value {
        match (base, overlay) {
            (Value::Object(mut base_map), Value::Object(overlay_map)) => {
                for (key, value) in overlay_map {
                    let new_value = if let Some(base_value) = base_map.remove(&key) {
                        deep_merge(base_value, value)
                    } else {
                        value
                    };
                    base_map.insert(key, new_value);
                }
                Value::Object(base_map)
            }
            (_, overlay) => overlay,
        }
    }

    /// Get a nested value from a JSON object using a dot-separated path.
    pub fn get_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
        let mut current = value;
        for part in path.split('.') {
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

    /// Set a nested value in a JSON object using a dot-separated path.
    pub fn set_path(value: &mut Value, path: &str, new_value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set the value
                if let Value::Object(map) = current {
                    map.insert(part.to_string(), new_value);
                }
                return;
            }

            // Navigate to the next level
            if let Value::Object(map) = current {
                if !map.contains_key(*part) {
                    map.insert(part.to_string(), Value::Object(Default::default()));
                }
                current = map.get_mut(*part).unwrap();
            } else {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_migration() {
        let migration = SimpleMigration::new(1, 2, "Test migration", |mut data| {
            if let Value::Object(ref mut map) = data {
                map.insert("new_field".to_string(), json!("added"));
            }
            Ok(data)
        });

        assert_eq!(migration.from_version(), 1);
        assert_eq!(migration.to_version(), 2);

        let input = json!({"existing": "value"});
        let output = migration.migrate_up(input).unwrap();

        assert_eq!(output["existing"], "value");
        assert_eq!(output["new_field"], "added");
    }

    #[test]
    fn test_migration_builder() {
        let migration = MigrationBuilder::new(1, 2, "Upgrade to v2")
            .rename_field("old_name", "new_name")
            .add_field("created_at", "2024-01-01")
            .remove_field("deprecated")
            .build();

        let input = json!({
            "old_name": "test",
            "deprecated": true,
            "keep": "this"
        });

        let output = migration.migrate_up(input).unwrap();

        assert!(output.get("old_name").is_none());
        assert_eq!(output["new_name"], "test");
        assert_eq!(output["created_at"], "2024-01-01");
        assert!(output.get("deprecated").is_none());
        assert_eq!(output["keep"], "this");
    }

    #[test]
    fn test_helper_string_to_array() {
        let input = json!("single");
        let output = helpers::string_to_array(input);
        assert_eq!(output, json!(["single"]));

        let input = json!("");
        let output = helpers::string_to_array(input);
        assert_eq!(output, json!([]));
    }

    #[test]
    fn test_helper_deep_merge() {
        let base = json!({
            "a": 1,
            "b": {"c": 2, "d": 3}
        });
        let overlay = json!({
            "b": {"c": 20, "e": 5},
            "f": 6
        });

        let result = helpers::deep_merge(base, overlay);
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"]["c"], 20);
        assert_eq!(result["b"]["d"], 3);
        assert_eq!(result["b"]["e"], 5);
        assert_eq!(result["f"], 6);
    }

    #[test]
    fn test_helper_path_operations() {
        let value = json!({
            "user": {
                "name": "Alice",
                "address": {
                    "city": "NYC"
                }
            }
        });

        assert_eq!(helpers::get_path(&value, "user.name"), Some(&json!("Alice")));
        assert_eq!(
            helpers::get_path(&value, "user.address.city"),
            Some(&json!("NYC"))
        );
        assert_eq!(helpers::get_path(&value, "user.unknown"), None);

        let mut mutable = value.clone();
        helpers::set_path(&mut mutable, "user.address.country", json!("USA"));
        assert_eq!(
            helpers::get_path(&mutable, "user.address.country"),
            Some(&json!("USA"))
        );
    }
}
