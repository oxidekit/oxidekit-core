//! Wizard configuration types
//!
//! Defines the configuration structures used by the project wizard.

use std::path::PathBuf;

/// Configuration collected during the wizard process
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Project name (directory name and package name)
    pub name: String,
    /// Project description
    pub description: String,
    /// Author name/email
    pub author: Option<String>,
    /// Selected starter template (None for blank project)
    pub starter: Option<String>,
    /// Selected plugin presets
    pub plugins: Vec<PluginPreset>,
    /// Selected theme
    pub theme: ThemeChoice,
    /// Whether to initialize git repository
    pub init_git: bool,
    /// Output directory path
    pub output_dir: PathBuf,
}

impl ProjectConfig {
    /// Create a default configuration for a given project name
    pub fn default_for_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: format!("An OxideKit application"),
            author: None,
            starter: None,
            plugins: vec![PluginPreset::Core],
            theme: ThemeChoice::Dark,
            init_git: true,
            output_dir: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(name),
        }
    }

    /// Get the app ID from the project name
    pub fn app_id(&self) -> String {
        let sanitized = self.name
            .chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
            .collect::<String>();
        format!("com.oxidekit.{}", sanitized)
    }
}

/// Pre-configured wizard options (from CLI flags)
#[derive(Debug, Clone, Default)]
pub struct WizardConfig {
    /// Project description
    pub description: Option<String>,
    /// Author name
    pub author: Option<String>,
    /// Selected starter template ID
    pub starter: Option<String>,
    /// Selected plugin presets
    pub plugins: Vec<PluginPreset>,
    /// Selected theme
    pub theme: ThemeChoice,
    /// Initialize git repository
    pub init_git: bool,
    /// Output directory
    pub output_dir: Option<PathBuf>,
}

/// Plugin presets for quick selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginPreset {
    /// Core UI components only
    Core,
    /// Desktop application capabilities
    Desktop,
    /// Web/WASM build target
    Web,
    /// Full native access (filesystem, system tray, etc.)
    Native,
    /// Network and HTTP capabilities
    Network,
    /// Data persistence (local storage, database)
    Storage,
    /// Cryptographic operations
    Crypto,
    /// All capabilities enabled
    Full,
}

impl PluginPreset {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Core => "Core UI",
            Self::Desktop => "Desktop App",
            Self::Web => "Web/WASM",
            Self::Native => "Native Access",
            Self::Network => "Network/HTTP",
            Self::Storage => "Data Storage",
            Self::Crypto => "Cryptography",
            Self::Full => "Everything",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Core => "Basic UI components and layout",
            Self::Desktop => "Window management, menus, system tray",
            Self::Web => "WASM compilation, web APIs",
            Self::Native => "Filesystem, shell commands, native dialogs",
            Self::Network => "HTTP client, WebSocket, fetch API",
            Self::Storage => "Local storage, SQLite, key-value store",
            Self::Crypto => "Encryption, signing, key management",
            Self::Full => "All available plugins and capabilities",
        }
    }

    /// Get the plugin IDs associated with this preset
    pub fn plugin_ids(&self) -> Vec<&'static str> {
        match self {
            Self::Core => vec!["ui.core", "ui.components"],
            Self::Desktop => vec!["native.window", "native.menu", "native.tray", "native.dialog"],
            Self::Web => vec!["web.wasm", "web.dom", "web.fetch"],
            Self::Native => vec!["native.filesystem", "native.shell", "native.dialog", "native.clipboard"],
            Self::Network => vec!["network.http", "network.websocket", "network.fetch"],
            Self::Storage => vec!["storage.local", "storage.sqlite", "storage.keyvalue"],
            Self::Crypto => vec!["crypto.encryption", "crypto.signing", "crypto.keychain"],
            Self::Full => vec![
                "ui.core", "ui.components",
                "native.window", "native.menu", "native.tray", "native.dialog",
                "native.filesystem", "native.shell", "native.clipboard",
                "network.http", "network.websocket",
                "storage.local", "storage.sqlite",
                "crypto.encryption", "crypto.signing",
            ],
        }
    }

    /// Get all available presets
    pub fn all() -> Vec<Self> {
        vec![
            Self::Core,
            Self::Desktop,
            Self::Web,
            Self::Native,
            Self::Network,
            Self::Storage,
            Self::Crypto,
            Self::Full,
        ]
    }
}

impl std::fmt::Display for PluginPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Theme choice for the project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeChoice {
    /// Dark theme (default)
    #[default]
    Dark,
    /// Light theme
    Light,
    /// System preference (auto-switch)
    System,
    /// High contrast for accessibility
    HighContrast,
    /// Custom theme (user will configure later)
    Custom,
}

impl ThemeChoice {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::System => "System (auto)",
            Self::HighContrast => "High Contrast",
            Self::Custom => "Custom",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Dark => "Dark background with light text - easy on the eyes",
            Self::Light => "Light background with dark text - classic appearance",
            Self::System => "Automatically match system preferences",
            Self::HighContrast => "Maximum contrast for accessibility",
            Self::Custom => "Configure your own theme colors later",
        }
    }

    /// Get theme configuration for oxide.toml
    pub fn to_toml_value(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::System => "system",
            Self::HighContrast => "high-contrast",
            Self::Custom => "custom",
        }
    }

    /// Get all available themes
    pub fn all() -> Vec<Self> {
        vec![
            Self::Dark,
            Self::Light,
            Self::System,
            Self::HighContrast,
            Self::Custom,
        ]
    }
}

impl std::fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default_for_name("test-app");
        assert_eq!(config.name, "test-app");
        assert!(config.init_git);
        assert_eq!(config.theme, ThemeChoice::Dark);
    }

    #[test]
    fn test_app_id_generation() {
        let config = ProjectConfig::default_for_name("my-cool-app");
        assert_eq!(config.app_id(), "com.oxidekit.my_cool_app");
    }

    #[test]
    fn test_plugin_preset_ids() {
        let core = PluginPreset::Core;
        assert!(core.plugin_ids().contains(&"ui.core"));

        let full = PluginPreset::Full;
        assert!(full.plugin_ids().len() > 5);
    }
}
