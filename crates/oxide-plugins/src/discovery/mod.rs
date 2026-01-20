//! Plugin discovery and scanning.
//!
//! Discovers plugins installed in a project by scanning standard locations
//! and parsing their manifests.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use tracing::{debug, warn};

use crate::error::{PluginError, PluginResult};
use crate::manifest::{self, PluginManifest};

/// Plugin discovery system.
///
/// Scans project directories for installed plugins and their manifests.
#[derive(Debug)]
pub struct PluginDiscovery {
    /// Root directory of the project.
    project_root: PathBuf,
    /// Directories to scan for plugins.
    plugin_dirs: Vec<PathBuf>,
}

impl PluginDiscovery {
    /// Create a new discovery instance for a project.
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        let root = project_root.as_ref().to_path_buf();

        // Standard plugin directories
        let plugin_dirs = vec![
            root.join("plugins"),           // Local plugins
            root.join(".oxide/plugins"),    // Installed plugins
            root.join("node_modules/@oxide"), // Legacy/compat location
        ];

        Self {
            project_root: root,
            plugin_dirs,
        }
    }

    /// Add a custom directory to scan.
    pub fn add_scan_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.plugin_dirs.push(path.as_ref().to_path_buf());
    }

    /// Scan all plugin directories and return discovered plugins.
    pub fn scan(&self) -> PluginResult<Vec<(PathBuf, PluginManifest)>> {
        let mut plugins = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                debug!("Plugin directory does not exist: {:?}", dir);
                continue;
            }

            let discovered = self.scan_directory(dir)?;
            plugins.extend(discovered);
        }

        Ok(plugins)
    }

    /// Scan a specific directory for plugins.
    fn scan_directory(&self, dir: &Path) -> PluginResult<Vec<(PathBuf, PluginManifest)>> {
        let mut plugins = Vec::new();

        for entry in WalkDir::new(dir)
            .max_depth(3)  // Limit depth to avoid deep recursion
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Check for plugin.toml
            if path.file_name().map(|n| n == "plugin.toml").unwrap_or(false) {
                match manifest::load_manifest(path) {
                    Ok(manifest) => {
                        let plugin_dir = path.parent().unwrap_or(path).to_path_buf();
                        debug!("Discovered plugin: {} at {:?}", manifest.plugin.id, plugin_dir);
                        plugins.push((plugin_dir, manifest));
                    }
                    Err(e) => {
                        warn!("Failed to load manifest at {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Find a specific plugin by ID.
    pub fn find_plugin(&self, plugin_id: &str) -> PluginResult<Option<(PathBuf, PluginManifest)>> {
        let plugins = self.scan()?;

        for (path, manifest) in plugins {
            if manifest.plugin.id.full_name() == plugin_id {
                return Ok(Some((path, manifest)));
            }
        }

        Ok(None)
    }

    /// Get all plugin directories.
    pub fn plugin_directories(&self) -> &[PathBuf] {
        &self.plugin_dirs
    }

    /// Get the primary plugin installation directory.
    pub fn install_directory(&self) -> PathBuf {
        self.project_root.join(".oxide/plugins")
    }

    /// Check if a plugin is installed.
    pub fn is_installed(&self, plugin_id: &str) -> PluginResult<bool> {
        Ok(self.find_plugin(plugin_id)?.is_some())
    }
}

/// Index of discovered plugins for fast lookup.
#[derive(Debug, Default)]
pub struct PluginIndex {
    /// Plugins indexed by ID.
    by_id: HashMap<String, IndexEntry>,
    /// Plugins indexed by namespace.
    by_namespace: HashMap<String, Vec<String>>,
    /// Plugins indexed by category.
    by_category: HashMap<String, Vec<String>>,
}

/// Entry in the plugin index.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Plugin ID.
    pub id: String,
    /// Plugin version.
    pub version: String,
    /// Plugin path.
    pub path: PathBuf,
    /// Plugin category.
    pub category: String,
    /// Plugin namespace.
    pub namespace: String,
    /// Publisher.
    pub publisher: String,
    /// Description.
    pub description: String,
}

impl PluginIndex {
    /// Create a new empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an index from discovered plugins.
    pub fn from_discovered(plugins: &[(PathBuf, PluginManifest)]) -> Self {
        let mut index = Self::new();

        for (path, manifest) in plugins {
            let entry = IndexEntry {
                id: manifest.plugin.id.full_name().to_string(),
                version: manifest.plugin.version.to_string(),
                path: path.clone(),
                category: manifest.plugin.kind.to_string(),
                namespace: manifest.plugin.id.namespace().to_string(),
                publisher: manifest.plugin.publisher.clone(),
                description: manifest.plugin.description.clone(),
            };

            index.add(entry);
        }

        index
    }

    /// Add an entry to the index.
    pub fn add(&mut self, entry: IndexEntry) {
        let id = entry.id.clone();
        let namespace = entry.namespace.clone();
        let category = entry.category.clone();

        // Index by ID
        self.by_id.insert(id.clone(), entry);

        // Index by namespace
        self.by_namespace
            .entry(namespace)
            .or_default()
            .push(id.clone());

        // Index by category
        self.by_category
            .entry(category)
            .or_default()
            .push(id);
    }

    /// Get a plugin by ID.
    pub fn get(&self, id: &str) -> Option<&IndexEntry> {
        self.by_id.get(id)
    }

    /// Get all plugins in a namespace.
    pub fn by_namespace(&self, namespace: &str) -> Vec<&IndexEntry> {
        self.by_namespace
            .get(namespace)
            .map(|ids| ids.iter().filter_map(|id| self.by_id.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all plugins in a category.
    pub fn by_category(&self, category: &str) -> Vec<&IndexEntry> {
        self.by_category
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.by_id.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all plugins.
    pub fn all(&self) -> impl Iterator<Item = &IndexEntry> {
        self.by_id.values()
    }

    /// Get the number of indexed plugins.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Search plugins by keyword.
    pub fn search(&self, query: &str) -> Vec<&IndexEntry> {
        let query_lower = query.to_lowercase();

        self.by_id
            .values()
            .filter(|entry| {
                entry.id.to_lowercase().contains(&query_lower)
                    || entry.description.to_lowercase().contains(&query_lower)
                    || entry.publisher.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_discovery_new() {
        let dir = tempdir().unwrap();
        let discovery = PluginDiscovery::new(dir.path());

        assert_eq!(discovery.project_root, dir.path());
        assert!(!discovery.plugin_dirs.is_empty());
    }

    #[test]
    fn test_scan_empty_project() {
        let dir = tempdir().unwrap();
        let discovery = PluginDiscovery::new(dir.path());

        let plugins = discovery.scan().unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_plugin_index() {
        let mut index = PluginIndex::new();

        let entry = IndexEntry {
            id: "ui.tables".to_string(),
            version: "1.0.0".to_string(),
            path: PathBuf::from("/test"),
            category: "ui".to_string(),
            namespace: "ui".to_string(),
            publisher: "oxidekit".to_string(),
            description: "Data tables".to_string(),
        };

        index.add(entry);

        assert!(index.get("ui.tables").is_some());
        assert_eq!(index.by_namespace("ui").len(), 1);
        assert_eq!(index.by_category("ui").len(), 1);
    }

    #[test]
    fn test_plugin_index_search() {
        let mut index = PluginIndex::new();

        index.add(IndexEntry {
            id: "ui.tables".to_string(),
            version: "1.0.0".to_string(),
            path: PathBuf::from("/test"),
            category: "ui".to_string(),
            namespace: "ui".to_string(),
            publisher: "oxidekit".to_string(),
            description: "Data tables with sorting".to_string(),
        });

        index.add(IndexEntry {
            id: "ui.forms".to_string(),
            version: "1.0.0".to_string(),
            path: PathBuf::from("/test"),
            category: "ui".to_string(),
            namespace: "ui".to_string(),
            publisher: "oxidekit".to_string(),
            description: "Form components".to_string(),
        });

        let results = index.search("tables");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ui.tables");

        let results = index.search("oxidekit");
        assert_eq!(results.len(), 2);
    }
}
