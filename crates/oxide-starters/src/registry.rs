//! Starter Registry
//!
//! Discovery and management of available starters.

use crate::{StarterSpec, StarterCategory, StarterTarget};
use std::collections::HashMap;
use std::path::Path;

/// Registry of available starters
#[derive(Debug, Default)]
pub struct StarterRegistry {
    /// Registered starters by ID
    starters: HashMap<String, StarterSpec>,
}

impl StarterRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            starters: HashMap::new(),
        }
    }

    /// Create a registry with built-in official starters
    pub fn with_builtin() -> Self {
        let mut registry = Self::new();
        registry.register_builtin_starters();
        registry
    }

    /// Register a starter
    pub fn register(&mut self, spec: StarterSpec) {
        self.starters.insert(spec.id.clone(), spec);
    }

    /// Get a starter by ID
    pub fn get(&self, id: &str) -> Option<&StarterSpec> {
        self.starters.get(id)
    }

    /// List all registered starters
    pub fn list(&self) -> Vec<&StarterSpec> {
        let mut starters: Vec<_> = self.starters.values().collect();
        starters.sort_by(|a, b| a.id.cmp(&b.id));
        starters
    }

    /// List starters by category
    pub fn list_by_category(&self, category: StarterCategory) -> Vec<&StarterSpec> {
        self.starters
            .values()
            .filter(|s| s.metadata.category == category)
            .collect()
    }

    /// List starters by target
    pub fn list_by_target(&self, target: StarterTarget) -> Vec<&StarterSpec> {
        self.starters
            .values()
            .filter(|s| s.targets.contains(&target))
            .collect()
    }

    /// Search starters by query (matches id, name, description, tags)
    pub fn search(&self, query: &str) -> Vec<&StarterSpec> {
        let query = query.to_lowercase();
        self.starters
            .values()
            .filter(|s| {
                s.id.to_lowercase().contains(&query)
                    || s.name.to_lowercase().contains(&query)
                    || s.description.to_lowercase().contains(&query)
                    || s.metadata.tags.iter().any(|t| t.to_lowercase().contains(&query))
            })
            .collect()
    }

    /// Load starters from a directory
    pub fn load_from_directory(&mut self, dir: &Path) -> anyhow::Result<usize> {
        let mut count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in walkdir::WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name() == Some(std::ffi::OsStr::new("starter.toml")) {
                match self.load_starter_file(path) {
                    Ok(spec) => {
                        tracing::info!(starter_id = %spec.id, "Loaded starter from {}", path.display());
                        self.register(spec);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load starter from {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load a single starter file
    fn load_starter_file(&self, path: &Path) -> anyhow::Result<StarterSpec> {
        let content = std::fs::read_to_string(path)?;
        let spec = StarterSpec::from_toml(&content)?;
        Ok(spec)
    }

    /// Register all built-in official starters
    fn register_builtin_starters(&mut self) {
        self.register(crate::templates::admin_panel::create_spec());
        self.register(crate::templates::desktop_wallet::create_spec());
        self.register(crate::templates::docs_site::create_spec());
        self.register(crate::templates::website_single::create_spec());
        self.register(crate::templates::logger_dashboard::create_spec());
    }

    /// Get number of registered starters
    pub fn len(&self) -> usize {
        self.starters.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.starters.is_empty()
    }
}

/// Summary of a starter for listing
#[derive(Debug, Clone)]
pub struct StarterSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: StarterCategory,
    pub targets: Vec<StarterTarget>,
    pub official: bool,
}

impl From<&StarterSpec> for StarterSummary {
    fn from(spec: &StarterSpec) -> Self {
        Self {
            id: spec.id.clone(),
            name: spec.name.clone(),
            description: spec.description.clone(),
            category: spec.metadata.category,
            targets: spec.targets.clone(),
            official: spec.metadata.official,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = StarterRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_builtin_registry() {
        let registry = StarterRegistry::with_builtin();
        assert!(!registry.is_empty());

        // Should have at least the official starters
        assert!(registry.get("admin-panel").is_some());
        assert!(registry.get("desktop-wallet").is_some());
        assert!(registry.get("docs-site").is_some());
    }

    #[test]
    fn test_search() {
        let registry = StarterRegistry::with_builtin();

        let results = registry.search("admin");
        assert!(!results.is_empty());

        let results = registry.search("wallet");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_list_by_category() {
        let registry = StarterRegistry::with_builtin();

        let admin_starters = registry.list_by_category(StarterCategory::Admin);
        assert!(!admin_starters.is_empty());
    }
}
