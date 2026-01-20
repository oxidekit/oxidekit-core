//! Migration registry for managing state migrations.

use super::{Migration, MigrationPlan, MigrationResult, MigrationStep};
use crate::error::{StateError, StateResult};
use crate::persistence::{StateStore, StoredState};
use crate::state::StateId;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tracing::{debug, info};

/// Registry for managing migrations across different state types.
pub struct MigrationRegistry {
    /// Migrations keyed by state type name, then by source version.
    migrations: HashMap<String, BTreeMap<u32, Arc<dyn Migration>>>,
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRegistry {
    /// Create a new empty migration registry.
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
        }
    }

    /// Register a migration for a state type.
    pub fn register<M: Migration + 'static>(&mut self, state_type: &str, migration: M) {
        let type_migrations = self.migrations.entry(state_type.to_string()).or_default();
        let from_version = migration.from_version();
        let to_version = migration.to_version();
        type_migrations.insert(from_version, Arc::new(migration));

        debug!(
            "Registered migration for {} from v{} to v{}",
            state_type,
            from_version,
            to_version
        );
    }

    /// Register a migration using a boxed trait object.
    pub fn register_boxed(&mut self, state_type: &str, migration: Arc<dyn Migration>) {
        let type_migrations = self.migrations.entry(state_type.to_string()).or_default();
        let from_version = migration.from_version();
        type_migrations.insert(from_version, migration);
    }

    /// Get the migration path from one version to another.
    pub fn get_migration_path(
        &self,
        state_type: &str,
        from: u32,
        to: u32,
    ) -> Option<Vec<Arc<dyn Migration>>> {
        let type_migrations = self.migrations.get(state_type)?;

        if from == to {
            return Some(Vec::new());
        }

        if from < to {
            // Forward migration
            self.find_upgrade_path(type_migrations, from, to)
        } else {
            // Backward migration (downgrade)
            self.find_downgrade_path(type_migrations, from, to)
        }
    }

    /// Find the upgrade path from one version to another.
    fn find_upgrade_path(
        &self,
        migrations: &BTreeMap<u32, Arc<dyn Migration>>,
        from: u32,
        to: u32,
    ) -> Option<Vec<Arc<dyn Migration>>> {
        let mut path = Vec::new();
        let mut current = from;

        while current < to {
            let migration = migrations.get(&current)?;
            if migration.to_version() > to {
                // This migration goes beyond our target
                return None;
            }
            path.push(migration.clone());
            current = migration.to_version();
        }

        if current == to {
            Some(path)
        } else {
            None
        }
    }

    /// Find the downgrade path from one version to another.
    fn find_downgrade_path(
        &self,
        migrations: &BTreeMap<u32, Arc<dyn Migration>>,
        from: u32,
        to: u32,
    ) -> Option<Vec<Arc<dyn Migration>>> {
        // Find all migrations that can bring us from `from` back to `to`
        let mut path = Vec::new();
        let mut current = from;

        // Collect migrations in reverse order
        let mut reverse_migrations: Vec<_> = migrations
            .iter()
            .filter(|(_, m)| m.to_version() <= from && m.from_version() >= to && m.is_reversible())
            .collect();

        // Sort by to_version descending
        reverse_migrations.sort_by(|a, b| b.1.to_version().cmp(&a.1.to_version()));

        for (_, migration) in reverse_migrations {
            if migration.to_version() == current {
                path.push(migration.clone());
                current = migration.from_version();
                if current == to {
                    return Some(path);
                }
            }
        }

        if current == to {
            Some(path)
        } else {
            None
        }
    }

    /// Check if a migration path exists.
    pub fn has_migration_path(&self, state_type: &str, from: u32, to: u32) -> bool {
        self.get_migration_path(state_type, from, to).is_some()
    }

    /// Create a migration plan.
    pub fn plan_migration(
        &self,
        state_id: StateId,
        state_type: &str,
        from: u32,
        to: u32,
    ) -> StateResult<MigrationPlan> {
        let path = self
            .get_migration_path(state_type, from, to)
            .ok_or_else(|| StateError::NoMigrationPath { from, to })?;

        let is_upgrade = to > from;

        let steps: Vec<MigrationStep> = path
            .iter()
            .map(|m| MigrationStep {
                from: if is_upgrade {
                    m.from_version()
                } else {
                    m.to_version()
                },
                to: if is_upgrade {
                    m.to_version()
                } else {
                    m.from_version()
                },
                description: m.description().to_string(),
                reversible: m.is_reversible(),
            })
            .collect();

        Ok(MigrationPlan {
            state_id,
            from_version: from,
            to_version: to,
            is_upgrade,
            steps,
        })
    }

    /// Execute a migration on JSON data.
    pub fn migrate(
        &self,
        state_type: &str,
        data: Value,
        from: u32,
        to: u32,
    ) -> StateResult<Value> {
        let path = self
            .get_migration_path(state_type, from, to)
            .ok_or_else(|| StateError::NoMigrationPath { from, to })?;

        let is_upgrade = to > from;
        let mut current_data = data;

        for migration in &path {
            if is_upgrade {
                current_data = migration.migrate_up(current_data).map_err(|e| {
                    StateError::migration_failed(
                        migration.from_version(),
                        migration.to_version(),
                        e.to_string(),
                    )
                })?;
            } else {
                current_data = migration
                    .migrate_down(current_data)
                    .map_err(|e| {
                        StateError::migration_failed(
                            migration.to_version(),
                            migration.from_version(),
                            e.to_string(),
                        )
                    })?
                    .ok_or_else(|| {
                        StateError::migration_failed(
                            migration.to_version(),
                            migration.from_version(),
                            "Migration is not reversible",
                        )
                    })?;
            }
        }

        Ok(current_data)
    }

    /// Execute a migration and return detailed results.
    pub fn migrate_with_result(
        &self,
        state_id: StateId,
        state_type: &str,
        data: Value,
        from: u32,
        to: u32,
    ) -> StateResult<MigrationResult> {
        let path = self
            .get_migration_path(state_type, from, to)
            .ok_or_else(|| StateError::NoMigrationPath { from, to })?;

        let is_upgrade = to > from;
        let mut current_data = data;
        let mut steps_applied = 0;
        let fully_reversible = path.iter().all(|m| m.is_reversible());

        for migration in &path {
            info!(
                "Applying migration {} -> {} for {}",
                migration.from_version(),
                migration.to_version(),
                state_type
            );

            if is_upgrade {
                current_data = migration.migrate_up(current_data).map_err(|e| {
                    StateError::migration_failed(
                        migration.from_version(),
                        migration.to_version(),
                        e.to_string(),
                    )
                })?;
            } else {
                current_data = migration
                    .migrate_down(current_data)
                    .map_err(|e| {
                        StateError::migration_failed(
                            migration.to_version(),
                            migration.from_version(),
                            e.to_string(),
                        )
                    })?
                    .ok_or_else(|| {
                        StateError::migration_failed(
                            migration.to_version(),
                            migration.from_version(),
                            "Migration is not reversible",
                        )
                    })?;
            }

            steps_applied += 1;
        }

        Ok(MigrationResult {
            state_id,
            from_version: from,
            to_version: to,
            steps_applied,
            fully_reversible,
            data: current_data,
        })
    }

    /// Get all registered state types.
    pub fn registered_types(&self) -> Vec<&str> {
        self.migrations.keys().map(|s| s.as_str()).collect()
    }

    /// Get all migrations for a state type.
    pub fn migrations_for(&self, state_type: &str) -> Vec<(u32, u32, &str)> {
        self.migrations
            .get(state_type)
            .map(|m| {
                m.values()
                    .map(|migration| {
                        (
                            migration.from_version(),
                            migration.to_version(),
                            migration.description(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the latest version for a state type.
    pub fn latest_version(&self, state_type: &str) -> Option<u32> {
        self.migrations
            .get(state_type)?
            .values()
            .map(|m| m.to_version())
            .max()
    }
}

/// A migration runner that applies migrations using a state store.
pub struct MigrationRunner {
    registry: Arc<MigrationRegistry>,
}

impl MigrationRunner {
    /// Create a new migration runner.
    pub fn new(registry: Arc<MigrationRegistry>) -> Self {
        Self { registry }
    }

    /// Migrate a stored state to the target version.
    pub async fn migrate_stored(
        &self,
        store: &StateStore,
        state_id: &StateId,
        state_type: &str,
        target_version: u32,
    ) -> StateResult<Option<MigrationResult>> {
        // Load the current state
        let stored = match store.get_raw(state_id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let current_version = stored.metadata.version;

        if current_version == target_version {
            debug!("State {} is already at version {}", state_id, target_version);
            return Ok(None);
        }

        // Parse the JSON data
        let data: Value = serde_json::from_str(&stored.data)?;

        // Run the migration
        let result = self.registry.migrate_with_result(
            state_id.clone(),
            state_type,
            data,
            current_version,
            target_version,
        )?;

        // Save the migrated state
        let new_data = serde_json::to_string(&result.data)?;
        let mut new_metadata = stored.metadata.clone();
        new_metadata.version = target_version;
        new_metadata.touch();
        new_metadata.update_hash(new_data.as_bytes());

        let new_stored = StoredState {
            metadata: new_metadata,
            data: new_data,
        };

        store.save_raw(state_id, &new_stored).await?;

        info!(
            "Migrated {} from v{} to v{} ({} steps)",
            state_id, current_version, target_version, result.steps_applied
        );

        Ok(Some(result))
    }

    /// Migrate all states of a given type to the target version.
    pub async fn migrate_all(
        &self,
        store: &StateStore,
        state_type: &str,
        target_version: u32,
    ) -> StateResult<Vec<MigrationResult>> {
        let mut results = Vec::new();

        for state_id in store.list().await? {
            if let Some(stored) = store.get_raw(&state_id).await? {
                if stored.metadata.type_name == state_type {
                    if let Some(result) = self
                        .migrate_stored(store, &state_id, state_type, target_version)
                        .await?
                    {
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::SimpleMigration;
    use serde_json::json;

    fn create_test_migrations() -> MigrationRegistry {
        let mut registry = MigrationRegistry::new();

        registry.register(
            "TestState",
            SimpleMigration::new(1, 2, "Add created_at field", |mut data| {
                if let Value::Object(ref mut map) = data {
                    map.insert("created_at".to_string(), json!("2024-01-01"));
                }
                Ok(data)
            }),
        );

        registry.register(
            "TestState",
            SimpleMigration::new(2, 3, "Rename name to title", |mut data| {
                if let Value::Object(ref mut map) = data {
                    if let Some(name) = map.remove("name") {
                        map.insert("title".to_string(), name);
                    }
                }
                Ok(data)
            }),
        );

        registry
    }

    #[test]
    fn test_migration_path() {
        let registry = create_test_migrations();

        assert!(registry.has_migration_path("TestState", 1, 3));
        assert!(registry.has_migration_path("TestState", 1, 2));
        assert!(registry.has_migration_path("TestState", 2, 3));
        assert!(!registry.has_migration_path("TestState", 1, 4));
        assert!(!registry.has_migration_path("UnknownState", 1, 2));
    }

    #[test]
    fn test_migration_execution() {
        let registry = create_test_migrations();

        let input = json!({
            "name": "Test",
            "value": 42
        });

        let output = registry.migrate("TestState", input, 1, 3).unwrap();

        assert_eq!(output["title"], "Test");
        assert_eq!(output["value"], 42);
        assert_eq!(output["created_at"], "2024-01-01");
    }

    #[test]
    fn test_migration_plan() {
        let registry = create_test_migrations();
        let state_id = StateId::new("test");

        let plan = registry
            .plan_migration(state_id.clone(), "TestState", 1, 3)
            .unwrap();

        assert_eq!(plan.from_version, 1);
        assert_eq!(plan.to_version, 3);
        assert!(plan.is_upgrade);
        assert_eq!(plan.steps.len(), 2);
    }

    #[test]
    fn test_latest_version() {
        let registry = create_test_migrations();

        assert_eq!(registry.latest_version("TestState"), Some(3));
        assert_eq!(registry.latest_version("UnknownState"), None);
    }
}
