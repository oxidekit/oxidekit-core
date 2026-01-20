//! Plugin management state

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

/// Information about an installed plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID (e.g., "ui.charts", "data.query")
    pub id: String,

    /// Display name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Description
    pub description: String,

    /// Plugin author
    pub author: String,

    /// Plugin category
    pub category: PluginCategory,

    /// Whether the plugin is enabled
    pub enabled: bool,

    /// Installation path
    pub path: Option<PathBuf>,

    /// Installation date
    pub installed_at: DateTime<Utc>,

    /// Last updated date
    pub updated_at: DateTime<Utc>,

    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,

    /// Plugin capabilities/features
    pub capabilities: Vec<String>,

    /// Plugin size in bytes
    pub size_bytes: u64,

    /// Configuration schema (JSON Schema)
    pub config_schema: Option<serde_json::Value>,

    /// Current configuration
    pub config: serde_json::Value,

    /// Plugin health status
    pub health: PluginHealth,

    /// Plugin metadata
    pub metadata: PluginMetadata,
}

/// Plugin category
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginCategory {
    /// UI components (ui.*)
    #[default]
    UI,
    /// Data handling (data.*)
    Data,
    /// Native capabilities (native.*)
    Native,
    /// Design packs (design.*)
    Design,
    /// Developer tools (dev.*)
    Dev,
    /// Integrations (integrations.*)
    Integration,
    /// Other plugins
    Other,
}

impl PluginCategory {
    /// Get category from plugin ID
    pub fn from_id(id: &str) -> Self {
        if id.starts_with("ui.") {
            Self::UI
        } else if id.starts_with("data.") {
            Self::Data
        } else if id.starts_with("native.") {
            Self::Native
        } else if id.starts_with("design.") {
            Self::Design
        } else if id.starts_with("dev.") {
            Self::Dev
        } else if id.starts_with("integration.") || id.starts_with("integrations.") {
            Self::Integration
        } else {
            Self::Other
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::UI => "UI Components",
            Self::Data => "Data & API",
            Self::Native => "Native Capabilities",
            Self::Design => "Design Packs",
            Self::Dev => "Developer Tools",
            Self::Integration => "Integrations",
            Self::Other => "Other",
        }
    }
}

/// Plugin dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency plugin ID
    pub id: String,
    /// Version requirement
    pub version: String,
    /// Whether this is an optional dependency
    pub optional: bool,
}

/// Plugin health status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginHealth {
    #[default]
    Healthy,
    Degraded,
    Error,
    Unknown,
}

/// Additional plugin metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Documentation URL
    pub documentation: Option<String>,
    /// License
    pub license: Option<String>,
    /// Keywords
    pub keywords: Vec<String>,
    /// Minimum OxideKit version
    pub min_oxide_version: Option<String>,
    /// Whether this is an official plugin
    pub official: bool,
    /// Number of downloads (from registry)
    pub downloads: Option<u64>,
    /// Rating (1-5)
    pub rating: Option<f32>,
}

/// Registry of installed plugins
pub struct PluginRegistry {
    plugins: HashMap<String, PluginInfo>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Scan a directory for installed plugins
    pub fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        info!("Scanning {:?} for plugins", dir);

        for entry in WalkDir::new(dir)
            .max_depth(2)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name().map(|f| f == "plugin.toml").unwrap_or(false) {
                if let Some(plugin_dir) = path.parent() {
                    if let Err(e) = self.load_plugin(plugin_dir) {
                        warn!("Failed to load plugin at {:?}: {}", plugin_dir, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a plugin from its directory
    pub fn load_plugin(&mut self, plugin_dir: &Path) -> Result<()> {
        let manifest_path = plugin_dir.join("plugin.toml");
        if !manifest_path.exists() {
            anyhow::bail!("No plugin.toml found in {:?}", plugin_dir);
        }

        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = toml::from_str(&content)?;

        // Get file metadata for dates
        let metadata = std::fs::metadata(&manifest_path)?;
        let installed_at = metadata.created()
            .map(|t| DateTime::<Utc>::from(t))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = metadata.modified()
            .map(|t| DateTime::<Utc>::from(t))
            .unwrap_or_else(|_| Utc::now());

        // Calculate plugin size
        let size_bytes = calculate_plugin_size(plugin_dir);

        // Load configuration if exists
        let config_path = plugin_dir.join("config.json");
        let config = if config_path.exists() {
            let config_str = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&config_str)?
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        let info = PluginInfo {
            id: manifest.plugin.id.clone(),
            name: manifest.plugin.name.clone(),
            version: manifest.plugin.version.clone(),
            description: manifest.plugin.description.clone().unwrap_or_default(),
            author: manifest.plugin.author.clone().unwrap_or_else(|| "Unknown".to_string()),
            category: PluginCategory::from_id(&manifest.plugin.id),
            enabled: manifest.plugin.enabled.unwrap_or(true),
            path: Some(plugin_dir.to_path_buf()),
            installed_at,
            updated_at,
            dependencies: manifest.dependencies.unwrap_or_default().into_iter()
                .map(|(id, version)| PluginDependency {
                    id,
                    version,
                    optional: false,
                })
                .collect(),
            capabilities: manifest.plugin.capabilities.unwrap_or_default(),
            size_bytes,
            config_schema: manifest.config_schema,
            config,
            health: PluginHealth::Healthy,
            metadata: PluginMetadata {
                homepage: manifest.metadata.as_ref().and_then(|m| m.homepage.clone()),
                repository: manifest.metadata.as_ref().and_then(|m| m.repository.clone()),
                documentation: manifest.metadata.as_ref().and_then(|m| m.documentation.clone()),
                license: manifest.plugin.license.clone(),
                keywords: manifest.plugin.keywords.unwrap_or_default(),
                min_oxide_version: manifest.plugin.min_oxide_version.clone(),
                official: manifest.metadata.as_ref().map(|m| m.official).unwrap_or(false),
                downloads: None,
                rating: None,
            },
        };

        self.plugins.insert(info.id.clone(), info);
        Ok(())
    }

    /// Add built-in plugins
    pub fn add_builtins(&mut self) {
        // Add core built-in plugins
        let builtins = vec![
            create_builtin_plugin("ui.core", "Core UI Components", "Basic UI components: buttons, cards, inputs", PluginCategory::UI),
            create_builtin_plugin("ui.forms", "Form Components", "Form handling with validation and schemas", PluginCategory::UI),
            create_builtin_plugin("ui.tables", "Table Components", "Advanced data tables with sorting and filtering", PluginCategory::UI),
            create_builtin_plugin("ui.charts", "Chart Components", "Charts and data visualization", PluginCategory::UI),
            create_builtin_plugin("data.query", "Data Query", "Reactive data fetching and caching", PluginCategory::Data),
            create_builtin_plugin("data.forms", "Form State", "Form state management", PluginCategory::Data),
            create_builtin_plugin("native.fs", "Filesystem", "Native filesystem access", PluginCategory::Native),
            create_builtin_plugin("native.dialog", "Dialogs", "Native file and message dialogs", PluginCategory::Native),
            create_builtin_plugin("native.notification", "Notifications", "System notifications", PluginCategory::Native),
            create_builtin_plugin("native.network.http", "HTTP Client", "HTTP networking", PluginCategory::Native),
            create_builtin_plugin("native.network.websocket", "WebSocket", "WebSocket support", PluginCategory::Native),
        ];

        for plugin in builtins {
            self.plugins.insert(plugin.id.clone(), plugin);
        }
    }

    /// Get a plugin by ID
    pub fn get(&self, id: &str) -> Option<&PluginInfo> {
        self.plugins.get(id)
    }

    /// Get a mutable plugin by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut PluginInfo> {
        self.plugins.get_mut(id)
    }

    /// Get all plugins
    pub fn all(&self) -> Vec<&PluginInfo> {
        self.plugins.values().collect()
    }

    /// Search plugins
    pub fn search(&self, query: &str) -> Vec<&PluginInfo> {
        let query = query.to_lowercase();
        self.plugins
            .values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query)
                    || p.id.to_lowercase().contains(&query)
                    || p.description.to_lowercase().contains(&query)
            })
            .collect()
    }

    /// Get plugins by category
    pub fn by_category(&self, category: PluginCategory) -> Vec<&PluginInfo> {
        self.plugins
            .values()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get enabled plugins
    pub fn enabled(&self) -> Vec<&PluginInfo> {
        self.plugins
            .values()
            .filter(|p| p.enabled)
            .collect()
    }

    /// Get count of all plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Get count of enabled plugins
    pub fn enabled_count(&self) -> usize {
        self.plugins.values().filter(|p| p.enabled).count()
    }

    /// Enable a plugin
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(plugin) = self.plugins.get_mut(id) {
            plugin.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a plugin
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(plugin) = self.plugins.get_mut(id) {
            plugin.enabled = false;
            true
        } else {
            false
        }
    }

    /// Update plugin configuration
    pub fn update_config(&mut self, id: &str, config: serde_json::Value) -> bool {
        if let Some(plugin) = self.plugins.get_mut(id) {
            plugin.config = config;
            true
        } else {
            false
        }
    }

    /// Remove a plugin
    pub fn remove(&mut self, id: &str) -> Option<PluginInfo> {
        self.plugins.remove(id)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin manifest structure
#[derive(Debug, Deserialize)]
struct PluginManifest {
    plugin: PluginSection,
    #[serde(default)]
    dependencies: Option<HashMap<String, String>>,
    #[serde(default)]
    config_schema: Option<serde_json::Value>,
    #[serde(default)]
    metadata: Option<MetadataSection>,
}

#[derive(Debug, Deserialize)]
struct PluginSection {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    capabilities: Option<Vec<String>>,
    #[serde(default)]
    keywords: Option<Vec<String>>,
    #[serde(default)]
    min_oxide_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataSection {
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    documentation: Option<String>,
    #[serde(default)]
    official: bool,
}

/// Calculate plugin directory size
fn calculate_plugin_size(dir: &Path) -> u64 {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Create a built-in plugin info
fn create_builtin_plugin(id: &str, name: &str, description: &str, category: PluginCategory) -> PluginInfo {
    PluginInfo {
        id: id.to_string(),
        name: name.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: description.to_string(),
        author: "OxideKit Team".to_string(),
        category,
        enabled: true,
        path: None,
        installed_at: Utc::now(),
        updated_at: Utc::now(),
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        size_bytes: 0,
        config_schema: None,
        config: serde_json::Value::Object(serde_json::Map::new()),
        health: PluginHealth::Healthy,
        metadata: PluginMetadata {
            homepage: Some("https://oxidekit.com".to_string()),
            repository: Some("https://github.com/oxidekit/oxidekit-core".to_string()),
            documentation: Some("https://oxidekit.com/docs".to_string()),
            license: Some("MIT OR Apache-2.0".to_string()),
            keywords: Vec::new(),
            min_oxide_version: None,
            official: true,
            downloads: None,
            rating: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_category_from_id() {
        assert_eq!(PluginCategory::from_id("ui.buttons"), PluginCategory::UI);
        assert_eq!(PluginCategory::from_id("data.query"), PluginCategory::Data);
        assert_eq!(PluginCategory::from_id("native.fs"), PluginCategory::Native);
        assert_eq!(PluginCategory::from_id("custom.thing"), PluginCategory::Other);
    }

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();
        registry.add_builtins();
        assert!(!registry.is_empty());
        assert!(registry.get("ui.core").is_some());
    }
}
