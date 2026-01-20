//! Theme management state

use anyhow::Result;
use chrono::{DateTime, Utc};
use oxide_components::{Theme, DesignTokens};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

/// Information about a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    /// Theme ID
    pub id: String,

    /// Theme name
    pub name: String,

    /// Theme description
    pub description: String,

    /// Theme author
    pub author: String,

    /// Theme version
    pub version: String,

    /// Whether this is a dark theme
    pub is_dark: bool,

    /// Whether this is the currently active theme
    pub is_active: bool,

    /// Whether this is a built-in theme
    pub is_builtin: bool,

    /// Theme file path (if custom)
    pub path: Option<PathBuf>,

    /// Installation/creation date
    pub created_at: DateTime<Utc>,

    /// Last modified date
    pub updated_at: DateTime<Utc>,

    /// Preview colors (for thumbnails)
    pub preview_colors: ThemePreviewColors,

    /// Theme tags/categories
    pub tags: Vec<String>,

    /// Full theme data (lazy loaded)
    #[serde(skip)]
    pub theme_data: Option<Theme>,
}

/// Preview colors for theme thumbnails
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemePreviewColors {
    pub background: String,
    pub surface: String,
    pub primary: String,
    pub secondary: String,
    pub text: String,
    pub accent: String,
}

/// Registry of available themes
pub struct ThemeRegistry {
    themes: HashMap<String, ThemeInfo>,
    active_theme_id: Option<String>,
}

impl ThemeRegistry {
    /// Create a new theme registry
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
            active_theme_id: None,
        }
    }

    /// Add built-in themes
    pub fn add_builtin_themes(&mut self) {
        // Dark theme
        let dark = Theme::dark();
        let dark_info = ThemeInfo {
            id: "oxide.dark".to_string(),
            name: dark.name.clone(),
            description: dark.description.clone(),
            author: dark.metadata.author.clone(),
            version: dark.metadata.version.clone(),
            is_dark: true,
            is_active: true, // Default active
            is_builtin: true,
            path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            preview_colors: extract_preview_colors(&dark.tokens),
            tags: vec!["dark".to_string(), "default".to_string(), "official".to_string()],
            theme_data: Some(dark),
        };
        self.themes.insert(dark_info.id.clone(), dark_info);
        self.active_theme_id = Some("oxide.dark".to_string());

        // Light theme
        let light = Theme::light();
        let light_info = ThemeInfo {
            id: "oxide.light".to_string(),
            name: light.name.clone(),
            description: light.description.clone(),
            author: light.metadata.author.clone(),
            version: light.metadata.version.clone(),
            is_dark: false,
            is_active: false,
            is_builtin: true,
            path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            preview_colors: extract_preview_colors(&light.tokens),
            tags: vec!["light".to_string(), "default".to_string(), "official".to_string()],
            theme_data: Some(light),
        };
        self.themes.insert(light_info.id.clone(), light_info);

        // Add additional preset themes
        self.add_preset_themes();
    }

    /// Add preset themes (variations of dark/light)
    fn add_preset_themes(&mut self) {
        // Nord-inspired dark theme
        let nord_dark = ThemeInfo {
            id: "oxide.nord".to_string(),
            name: "Nord".to_string(),
            description: "A Nord-inspired dark theme with cool blue tones".to_string(),
            author: "OxideKit".to_string(),
            version: "1.0.0".to_string(),
            is_dark: true,
            is_active: false,
            is_builtin: true,
            path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            preview_colors: ThemePreviewColors {
                background: "#2E3440".to_string(),
                surface: "#3B4252".to_string(),
                primary: "#88C0D0".to_string(),
                secondary: "#81A1C1".to_string(),
                text: "#ECEFF4".to_string(),
                accent: "#8FBCBB".to_string(),
            },
            tags: vec!["dark".to_string(), "nord".to_string(), "cool".to_string()],
            theme_data: None, // Will be generated on demand
        };
        self.themes.insert(nord_dark.id.clone(), nord_dark);

        // Dracula-inspired theme
        let dracula = ThemeInfo {
            id: "oxide.dracula".to_string(),
            name: "Dracula".to_string(),
            description: "A Dracula-inspired dark theme with purple accents".to_string(),
            author: "OxideKit".to_string(),
            version: "1.0.0".to_string(),
            is_dark: true,
            is_active: false,
            is_builtin: true,
            path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            preview_colors: ThemePreviewColors {
                background: "#282A36".to_string(),
                surface: "#44475A".to_string(),
                primary: "#BD93F9".to_string(),
                secondary: "#6272A4".to_string(),
                text: "#F8F8F2".to_string(),
                accent: "#FF79C6".to_string(),
            },
            tags: vec!["dark".to_string(), "dracula".to_string(), "purple".to_string()],
            theme_data: None,
        };
        self.themes.insert(dracula.id.clone(), dracula);

        // High contrast theme
        let high_contrast = ThemeInfo {
            id: "oxide.high-contrast".to_string(),
            name: "High Contrast".to_string(),
            description: "A high contrast theme for accessibility".to_string(),
            author: "OxideKit".to_string(),
            version: "1.0.0".to_string(),
            is_dark: true,
            is_active: false,
            is_builtin: true,
            path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            preview_colors: ThemePreviewColors {
                background: "#000000".to_string(),
                surface: "#1A1A1A".to_string(),
                primary: "#00FF00".to_string(),
                secondary: "#FFFF00".to_string(),
                text: "#FFFFFF".to_string(),
                accent: "#00FFFF".to_string(),
            },
            tags: vec!["dark".to_string(), "high-contrast".to_string(), "accessibility".to_string()],
            theme_data: None,
        };
        self.themes.insert(high_contrast.id.clone(), high_contrast);

        // Warm light theme
        let warm_light = ThemeInfo {
            id: "oxide.warm-light".to_string(),
            name: "Warm Light".to_string(),
            description: "A warm light theme with sepia tones".to_string(),
            author: "OxideKit".to_string(),
            version: "1.0.0".to_string(),
            is_dark: false,
            is_active: false,
            is_builtin: true,
            path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            preview_colors: ThemePreviewColors {
                background: "#FDF6E3".to_string(),
                surface: "#EEE8D5".to_string(),
                primary: "#B58900".to_string(),
                secondary: "#657B83".to_string(),
                text: "#073642".to_string(),
                accent: "#CB4B16".to_string(),
            },
            tags: vec!["light".to_string(), "warm".to_string(), "sepia".to_string()],
            theme_data: None,
        };
        self.themes.insert(warm_light.id.clone(), warm_light);
    }

    /// Scan a directory for custom themes
    pub fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        info!("Scanning {:?} for themes", dir);

        for entry in WalkDir::new(dir)
            .max_depth(2)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Err(e) = self.load_theme(path) {
                    warn!("Failed to load theme at {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    /// Load a theme from a TOML file
    pub fn load_theme(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let theme: Theme = Theme::from_toml(&content)?;

        // Get file metadata
        let metadata = std::fs::metadata(path)?;
        let created_at = metadata.created()
            .map(|t| DateTime::<Utc>::from(t))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = metadata.modified()
            .map(|t| DateTime::<Utc>::from(t))
            .unwrap_or_else(|_| Utc::now());

        // Generate ID from filename
        let id = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| format!("custom.{}", s))
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let info = ThemeInfo {
            id: id.clone(),
            name: theme.name.clone(),
            description: theme.description.clone(),
            author: theme.metadata.author.clone(),
            version: theme.metadata.version.clone(),
            is_dark: theme.metadata.is_dark,
            is_active: false,
            is_builtin: false,
            path: Some(path.to_path_buf()),
            created_at,
            updated_at,
            preview_colors: extract_preview_colors(&theme.tokens),
            tags: vec![
                if theme.metadata.is_dark { "dark" } else { "light" }.to_string(),
                "custom".to_string(),
            ],
            theme_data: Some(theme),
        };

        self.themes.insert(id, info);
        Ok(())
    }

    /// Get a theme by ID
    pub fn get(&self, id: &str) -> Option<&ThemeInfo> {
        self.themes.get(id)
    }

    /// Get a mutable theme by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ThemeInfo> {
        self.themes.get_mut(id)
    }

    /// Get all themes
    pub fn all(&self) -> Vec<&ThemeInfo> {
        self.themes.values().collect()
    }

    /// Get dark themes
    pub fn dark_themes(&self) -> Vec<&ThemeInfo> {
        self.themes.values().filter(|t| t.is_dark).collect()
    }

    /// Get light themes
    pub fn light_themes(&self) -> Vec<&ThemeInfo> {
        self.themes.values().filter(|t| !t.is_dark).collect()
    }

    /// Get the active theme
    pub fn active(&self) -> Option<&ThemeInfo> {
        self.active_theme_id.as_ref().and_then(|id| self.themes.get(id))
    }

    /// Get the active theme ID
    pub fn active_id(&self) -> Option<&str> {
        self.active_theme_id.as_deref()
    }

    /// Set the active theme
    pub fn set_active(&mut self, id: &str) -> bool {
        if self.themes.contains_key(id) {
            // Deactivate current theme
            if let Some(current_id) = &self.active_theme_id {
                if let Some(theme) = self.themes.get_mut(current_id) {
                    theme.is_active = false;
                }
            }
            // Activate new theme
            if let Some(theme) = self.themes.get_mut(id) {
                theme.is_active = true;
            }
            self.active_theme_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// Get number of themes
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }

    /// Search themes
    pub fn search(&self, query: &str) -> Vec<&ThemeInfo> {
        let query = query.to_lowercase();
        self.themes
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query)
                    || t.description.to_lowercase().contains(&query)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            })
            .collect()
    }

    /// Get themes by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&ThemeInfo> {
        self.themes
            .values()
            .filter(|t| t.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Remove a custom theme
    pub fn remove(&mut self, id: &str) -> Option<ThemeInfo> {
        // Don't allow removing built-in themes
        if let Some(theme) = self.themes.get(id) {
            if theme.is_builtin {
                return None;
            }
        }
        self.themes.remove(id)
    }

    /// Create a new custom theme
    pub fn create_theme(&mut self, name: &str, base_theme_id: &str, is_dark: bool) -> Option<String> {
        let base = self.themes.get(base_theme_id)?.theme_data.clone()?;

        let id = format!("custom.{}", uuid::Uuid::new_v4());
        let mut new_theme = base;
        new_theme.name = name.to_string();
        new_theme.metadata.is_dark = is_dark;

        let info = ThemeInfo {
            id: id.clone(),
            name: name.to_string(),
            description: format!("Custom theme based on {}", base_theme_id),
            author: "User".to_string(),
            version: "1.0.0".to_string(),
            is_dark,
            is_active: false,
            is_builtin: false,
            path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            preview_colors: extract_preview_colors(&new_theme.tokens),
            tags: vec![
                if is_dark { "dark" } else { "light" }.to_string(),
                "custom".to_string(),
            ],
            theme_data: Some(new_theme),
        };

        self.themes.insert(id.clone(), info);
        Some(id)
    }

    /// Export a theme to TOML
    pub fn export_theme(&self, id: &str) -> Option<String> {
        let theme = self.themes.get(id)?;
        let theme_data = theme.theme_data.as_ref()?;
        theme_data.to_toml().ok()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract preview colors from design tokens
fn extract_preview_colors(tokens: &DesignTokens) -> ThemePreviewColors {
    ThemePreviewColors {
        background: tokens.color.background.value.clone(),
        surface: tokens.color.surface.value.clone(),
        primary: tokens.color.primary.value.clone(),
        secondary: tokens.color.secondary.value.clone(),
        text: tokens.color.text.value.clone(),
        accent: tokens.color.info.value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_registry() {
        let mut registry = ThemeRegistry::new();
        registry.add_builtin_themes();

        assert!(!registry.is_empty());
        assert!(registry.get("oxide.dark").is_some());
        assert!(registry.get("oxide.light").is_some());
    }

    #[test]
    fn test_active_theme() {
        let mut registry = ThemeRegistry::new();
        registry.add_builtin_themes();

        // Default active should be dark
        assert_eq!(registry.active_id(), Some("oxide.dark"));

        // Switch to light
        assert!(registry.set_active("oxide.light"));
        assert_eq!(registry.active_id(), Some("oxide.light"));
    }

    #[test]
    fn test_theme_search() {
        let mut registry = ThemeRegistry::new();
        registry.add_builtin_themes();

        let dark_themes = registry.search("dark");
        assert!(!dark_themes.is_empty());
    }
}
