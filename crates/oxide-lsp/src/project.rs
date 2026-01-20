//! Project Context
//!
//! Manages project-level information including schemas, manifests, and component metadata.

use crate::LspError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Project context containing all metadata for LSP features
#[derive(Debug)]
pub struct ProjectContext {
    /// Project root path
    pub root: std::path::PathBuf,
    /// Project manifest (oxide.toml)
    pub manifest: Option<ProjectManifest>,
    /// AI-generated component metadata (oxide.ai.json)
    pub ai_metadata: Option<AiMetadata>,
    /// Component schemas for validation
    pub component_schemas: HashMap<String, ComponentSchema>,
    /// Design tokens
    pub tokens: TokenRegistry,
    /// Translation keys
    pub i18n_keys: Vec<String>,
    /// Installed plugins
    pub plugins: Vec<PluginInfo>,
}

impl ProjectContext {
    /// Load project context from a directory
    pub async fn load(root: &Path) -> Result<Self, LspError> {
        let root = root.to_path_buf();

        // Load manifest
        let manifest = Self::load_manifest(&root).await.ok();

        // Load AI metadata
        let ai_metadata = Self::load_ai_metadata(&root).await.ok();

        // Build component schemas from AI metadata
        let component_schemas = ai_metadata
            .as_ref()
            .map(|m| Self::build_component_schemas(m))
            .unwrap_or_default();

        // Load tokens
        let tokens = Self::load_tokens(&root).await.unwrap_or_default();

        // Load i18n keys
        let i18n_keys = Self::load_i18n_keys(&root).await.unwrap_or_default();

        // Load plugins
        let plugins = Self::load_plugins(&root).await.unwrap_or_default();

        Ok(Self {
            root,
            manifest,
            ai_metadata,
            component_schemas,
            tokens,
            i18n_keys,
            plugins,
        })
    }

    async fn load_manifest(root: &Path) -> Result<ProjectManifest, LspError> {
        let path = root.join("oxide.toml");
        let content = tokio::fs::read_to_string(&path).await?;
        let manifest: ProjectManifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    async fn load_ai_metadata(root: &Path) -> Result<AiMetadata, LspError> {
        let path = root.join("oxide.ai.json");
        let content = tokio::fs::read_to_string(&path).await?;
        let metadata: AiMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    fn build_component_schemas(metadata: &AiMetadata) -> HashMap<String, ComponentSchema> {
        let mut schemas = HashMap::new();

        for component in &metadata.components {
            schemas.insert(component.id.clone(), ComponentSchema {
                id: component.id.clone(),
                description: component.description.clone(),
                props: component.props.clone(),
                events: component.events.clone().unwrap_or_default(),
                slots: component.slots.clone().unwrap_or_default(),
                deprecated: component.deprecated.unwrap_or(false),
                deprecation_message: component.deprecation_message.clone(),
            });
        }

        schemas
    }

    async fn load_tokens(root: &Path) -> Result<TokenRegistry, LspError> {
        let mut registry = TokenRegistry::default();

        // Load theme.toml if exists
        let theme_path = root.join("theme.toml");
        if theme_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&theme_path).await {
                if let Ok(theme) = toml::from_str::<ThemeFile>(&content) {
                    registry.merge_theme(theme);
                }
            }
        }

        Ok(registry)
    }

    async fn load_i18n_keys(root: &Path) -> Result<Vec<String>, LspError> {
        let mut keys = Vec::new();

        // Look for i18n directory
        let i18n_dir = root.join("i18n");
        if i18n_dir.exists() {
            // Load keys from en.toml or base locale
            let base_path = i18n_dir.join("en.toml");
            if base_path.exists() {
                if let Ok(content) = tokio::fs::read_to_string(&base_path).await {
                    if let Ok(translations) = toml::from_str::<toml::Value>(&content) {
                        keys.extend(Self::extract_keys(&translations, String::new()));
                    }
                }
            }
        }

        Ok(keys)
    }

    fn extract_keys(value: &toml::Value, prefix: String) -> Vec<String> {
        let mut keys = Vec::new();

        if let toml::Value::Table(table) = value {
            for (key, val) in table {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };

                match val {
                    toml::Value::String(_) => keys.push(full_key),
                    toml::Value::Table(_) => keys.extend(Self::extract_keys(val, full_key)),
                    _ => {}
                }
            }
        }

        keys
    }

    async fn load_plugins(root: &Path) -> Result<Vec<PluginInfo>, LspError> {
        let mut plugins = Vec::new();

        // Check extensions.lock
        let lock_path = root.join("extensions.lock");
        if lock_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&lock_path).await {
                if let Ok(lock) = toml::from_str::<ExtensionsLock>(&content) {
                    for (name, info) in lock.plugins {
                        plugins.push(PluginInfo {
                            name,
                            version: info.version,
                            components: info.components.unwrap_or_default(),
                        });
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Get component schema by ID
    pub fn get_component(&self, id: &str) -> Option<&ComponentSchema> {
        self.component_schemas.get(id)
    }

    /// Get all component IDs
    pub fn component_ids(&self) -> impl Iterator<Item = &String> {
        self.component_schemas.keys()
    }

    /// Get all token names
    pub fn token_names(&self) -> impl Iterator<Item = &String> {
        self.tokens.all_names()
    }

    /// Check if a translation key exists
    pub fn has_i18n_key(&self, key: &str) -> bool {
        self.i18n_keys.iter().any(|k| k == key)
    }
}

/// Project manifest (oxide.toml)
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectManifest {
    pub package: Option<PackageInfo>,
    pub build: Option<BuildConfig>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildConfig {
    pub target: Option<String>,
    pub features: Option<Vec<String>>,
}

/// AI-generated metadata (oxide.ai.json)
#[derive(Debug, Deserialize, Serialize)]
pub struct AiMetadata {
    pub version: String,
    pub components: Vec<ComponentMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentMetadata {
    pub id: String,
    pub description: String,
    pub props: Vec<PropSchema>,
    pub events: Option<Vec<EventSchema>>,
    pub slots: Option<Vec<SlotSchema>>,
    pub deprecated: Option<bool>,
    pub deprecation_message: Option<String>,
}

/// Component schema for validation
#[derive(Debug, Clone)]
pub struct ComponentSchema {
    pub id: String,
    pub description: String,
    pub props: Vec<PropSchema>,
    pub events: Vec<EventSchema>,
    pub slots: Vec<SlotSchema>,
    pub deprecated: bool,
    pub deprecation_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PropSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub prop_type: String,
    pub required: Option<bool>,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub allowed_values: Option<Vec<String>>,
    pub deprecated: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventSchema {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlotSchema {
    pub name: String,
    pub description: Option<String>,
}

/// Token registry for design tokens
#[derive(Debug, Default)]
pub struct TokenRegistry {
    colors: HashMap<String, String>,
    spacing: HashMap<String, String>,
    radius: HashMap<String, String>,
    typography: HashMap<String, String>,
}

impl TokenRegistry {
    pub fn merge_theme(&mut self, theme: ThemeFile) {
        if let Some(colors) = theme.colors {
            for (name, value) in colors {
                self.colors.insert(format!("colors.{}", name), value);
            }
        }
        if let Some(spacing) = theme.spacing {
            for (name, value) in spacing {
                self.spacing.insert(format!("spacing.{}", name), value.to_string());
            }
        }
        if let Some(radius) = theme.radius {
            for (name, value) in radius {
                self.radius.insert(format!("radius.{}", name), value.to_string());
            }
        }
    }

    pub fn all_names(&self) -> impl Iterator<Item = &String> {
        self.colors
            .keys()
            .chain(self.spacing.keys())
            .chain(self.radius.keys())
            .chain(self.typography.keys())
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.colors
            .get(name)
            .or_else(|| self.spacing.get(name))
            .or_else(|| self.radius.get(name))
            .or_else(|| self.typography.get(name))
    }
}

#[derive(Debug, Deserialize)]
pub struct ThemeFile {
    pub colors: Option<HashMap<String, String>>,
    pub spacing: Option<HashMap<String, f64>>,
    pub radius: Option<HashMap<String, f64>>,
}

/// Plugin information
#[derive(Debug)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub components: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExtensionsLock {
    plugins: HashMap<String, PluginLockInfo>,
}

#[derive(Debug, Deserialize)]
struct PluginLockInfo {
    version: String,
    components: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keys() {
        let value: toml::Value = toml::from_str(r#"
            [auth]
            title = "Login"

            [auth.errors]
            invalid = "Invalid credentials"
        "#).unwrap();

        let keys = ProjectContext::extract_keys(&value, String::new());
        assert!(keys.contains(&"auth.title".to_string()));
        assert!(keys.contains(&"auth.errors.invalid".to_string()));
    }

    #[test]
    fn test_token_registry() {
        let mut registry = TokenRegistry::default();

        let theme = ThemeFile {
            colors: Some([("primary".to_string(), "#3B82F6".to_string())].into_iter().collect()),
            spacing: Some([("md".to_string(), 16.0)].into_iter().collect()),
            radius: None,
        };

        registry.merge_theme(theme);

        assert_eq!(registry.get("colors.primary"), Some(&"#3B82F6".to_string()));
        assert_eq!(registry.get("spacing.md"), Some(&"16".to_string()));
    }
}
