//! Application settings state

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// General settings
    #[serde(default)]
    pub general: GeneralSettings,

    /// Appearance settings
    #[serde(default)]
    pub appearance: AppearanceSettings,

    /// Editor settings
    #[serde(default)]
    pub editor: EditorSettings,

    /// Build settings
    #[serde(default)]
    pub build: BuildSettings,

    /// Developer settings
    #[serde(default)]
    pub developer: DeveloperSettings,

    /// Project directories to scan
    #[serde(default)]
    pub project_directories: Vec<PathBuf>,

    /// Recently opened projects
    #[serde(default)]
    pub recent_projects: Vec<PathBuf>,

    /// Pinned projects
    #[serde(default)]
    pub pinned_projects: Vec<String>,

    /// Update settings
    #[serde(default)]
    pub updates: UpdateSettings,

    /// Telemetry settings
    #[serde(default)]
    pub telemetry: TelemetrySettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            appearance: AppearanceSettings::default(),
            editor: EditorSettings::default(),
            build: BuildSettings::default(),
            developer: DeveloperSettings::default(),
            project_directories: Vec::new(),
            recent_projects: Vec::new(),
            pinned_projects: Vec::new(),
            updates: UpdateSettings::default(),
            telemetry: TelemetrySettings::default(),
        }
    }
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    /// Language (e.g., "en", "de", "ja")
    #[serde(default = "default_language")]
    pub language: String,

    /// Start on login
    #[serde(default)]
    pub start_on_login: bool,

    /// Start minimized
    #[serde(default)]
    pub start_minimized: bool,

    /// Check for updates automatically
    #[serde(default = "default_true")]
    pub check_updates: bool,

    /// Confirm before exit
    #[serde(default = "default_true")]
    pub confirm_exit: bool,

    /// Default project location
    #[serde(default)]
    pub default_project_location: Option<PathBuf>,

    /// Maximum recent projects
    #[serde(default = "default_max_recent")]
    pub max_recent_projects: usize,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: default_language(),
            start_on_login: false,
            start_minimized: false,
            check_updates: true,
            confirm_exit: true,
            default_project_location: None,
            max_recent_projects: default_max_recent(),
        }
    }
}

/// Appearance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    /// Theme ID
    #[serde(default = "default_theme")]
    pub theme: String,

    /// UI scale factor (1.0 = 100%)
    #[serde(default = "default_scale")]
    pub scale: f32,

    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: f32,

    /// Font family
    #[serde(default = "default_font_family")]
    pub font_family: String,

    /// Monospace font family
    #[serde(default = "default_mono_font")]
    pub mono_font_family: String,

    /// Enable animations
    #[serde(default = "default_true")]
    pub animations: bool,

    /// Reduce motion (accessibility)
    #[serde(default)]
    pub reduce_motion: bool,

    /// Sidebar width
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,

    /// Show sidebar icons only (collapsed)
    #[serde(default)]
    pub sidebar_collapsed: bool,

    /// Density mode (compact, comfortable, spacious)
    #[serde(default)]
    pub density: DensityMode,

    /// Accent color override (hex)
    #[serde(default)]
    pub accent_color: Option<String>,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            scale: default_scale(),
            font_size: default_font_size(),
            font_family: default_font_family(),
            mono_font_family: default_mono_font(),
            animations: true,
            reduce_motion: false,
            sidebar_width: default_sidebar_width(),
            sidebar_collapsed: false,
            density: DensityMode::default(),
            accent_color: None,
        }
    }
}

/// Density mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DensityMode {
    Compact,
    #[default]
    Comfortable,
    Spacious,
}

/// Editor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    /// Default code editor (for opening files)
    #[serde(default)]
    pub external_editor: Option<String>,

    /// Tab size
    #[serde(default = "default_tab_size")]
    pub tab_size: usize,

    /// Use spaces instead of tabs
    #[serde(default = "default_true")]
    pub use_spaces: bool,

    /// Show line numbers
    #[serde(default = "default_true")]
    pub line_numbers: bool,

    /// Word wrap
    #[serde(default = "default_true")]
    pub word_wrap: bool,

    /// Format on save
    #[serde(default = "default_true")]
    pub format_on_save: bool,

    /// Auto-save delay (ms, 0 = disabled)
    #[serde(default)]
    pub auto_save_delay: u32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            external_editor: None,
            tab_size: default_tab_size(),
            use_spaces: true,
            line_numbers: true,
            word_wrap: true,
            format_on_save: true,
            auto_save_delay: 0,
        }
    }
}

/// Build settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSettings {
    /// Default build target
    #[serde(default = "default_target")]
    pub default_target: String,

    /// Enable release builds by default
    #[serde(default)]
    pub release_by_default: bool,

    /// Parallel jobs (0 = auto)
    #[serde(default)]
    pub parallel_jobs: usize,

    /// Enable incremental builds
    #[serde(default = "default_true")]
    pub incremental: bool,

    /// Show build output
    #[serde(default = "default_true")]
    pub show_output: bool,

    /// Clean before build
    #[serde(default)]
    pub clean_before_build: bool,

    /// Cache directory
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
}

impl Default for BuildSettings {
    fn default() -> Self {
        Self {
            default_target: default_target(),
            release_by_default: false,
            parallel_jobs: 0,
            incremental: true,
            show_output: true,
            clean_before_build: false,
            cache_dir: None,
        }
    }
}

/// Developer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperSettings {
    /// Enable developer mode
    #[serde(default)]
    pub developer_mode: bool,

    /// Enable debug logging
    #[serde(default)]
    pub debug_logging: bool,

    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable hot reload
    #[serde(default = "default_true")]
    pub hot_reload: bool,

    /// Hot reload delay (ms)
    #[serde(default = "default_hot_reload_delay")]
    pub hot_reload_delay: u32,

    /// Enable inspector
    #[serde(default)]
    pub inspector: bool,

    /// Show performance overlay
    #[serde(default)]
    pub perf_overlay: bool,

    /// Enable experimental features
    #[serde(default)]
    pub experimental: bool,
}

impl Default for DeveloperSettings {
    fn default() -> Self {
        Self {
            developer_mode: false,
            debug_logging: false,
            log_level: default_log_level(),
            hot_reload: true,
            hot_reload_delay: default_hot_reload_delay(),
            inspector: false,
            perf_overlay: false,
            experimental: false,
        }
    }
}

/// Update settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    /// Check for updates automatically
    #[serde(default = "default_true")]
    pub auto_check: bool,

    /// Auto-install updates
    #[serde(default)]
    pub auto_install: bool,

    /// Update channel (stable, beta, nightly)
    #[serde(default = "default_channel")]
    pub channel: String,

    /// Include pre-releases
    #[serde(default)]
    pub include_prereleases: bool,

    /// Last check timestamp
    #[serde(default)]
    pub last_check: Option<String>,

    /// Dismissed update version
    #[serde(default)]
    pub dismissed_version: Option<String>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_check: true,
            auto_install: false,
            channel: default_channel(),
            include_prereleases: false,
            last_check: None,
            dismissed_version: None,
        }
    }
}

/// Telemetry settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    /// Enable telemetry
    #[serde(default)]
    pub enabled: bool,

    /// Share crash reports
    #[serde(default = "default_true")]
    pub crash_reports: bool,

    /// Share usage statistics
    #[serde(default)]
    pub usage_statistics: bool,
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            crash_reports: true,
            usage_statistics: false,
        }
    }
}

// Default value functions
fn default_true() -> bool { true }
fn default_language() -> String { "en".to_string() }
fn default_max_recent() -> usize { 10 }
fn default_theme() -> String { "oxide.dark".to_string() }
fn default_scale() -> f32 { 1.0 }
fn default_font_size() -> f32 { 14.0 }
fn default_font_family() -> String { "Inter".to_string() }
fn default_mono_font() -> String { "JetBrains Mono".to_string() }
fn default_sidebar_width() -> f32 { 240.0 }
fn default_tab_size() -> usize { 4 }
fn default_target() -> String { "desktop".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_hot_reload_delay() -> u32 { 300 }
fn default_channel() -> String { "stable".to_string() }

impl Settings {
    /// Add a recent project
    pub fn add_recent_project(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_projects.retain(|p| p != &path);
        // Add to front
        self.recent_projects.insert(0, path);
        // Trim to max
        if self.recent_projects.len() > self.general.max_recent_projects {
            self.recent_projects.truncate(self.general.max_recent_projects);
        }
    }

    /// Remove a recent project
    pub fn remove_recent_project(&mut self, path: &PathBuf) {
        self.recent_projects.retain(|p| p != path);
    }

    /// Clear recent projects
    pub fn clear_recent_projects(&mut self) {
        self.recent_projects.clear();
    }

    /// Pin a project
    pub fn pin_project(&mut self, id: String) {
        if !self.pinned_projects.contains(&id) {
            self.pinned_projects.push(id);
        }
    }

    /// Unpin a project
    pub fn unpin_project(&mut self, id: &str) {
        self.pinned_projects.retain(|p| p != id);
    }

    /// Check if a project is pinned
    pub fn is_pinned(&self, id: &str) -> bool {
        self.pinned_projects.contains(&id.to_string())
    }

    /// Add a project directory
    pub fn add_project_directory(&mut self, path: PathBuf) {
        if !self.project_directories.contains(&path) {
            self.project_directories.push(path);
        }
    }

    /// Remove a project directory
    pub fn remove_project_directory(&mut self, path: &PathBuf) {
        self.project_directories.retain(|p| p != path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.appearance.theme, "oxide.dark");
        assert_eq!(settings.general.language, "en");
    }

    #[test]
    fn test_recent_projects() {
        let mut settings = Settings::default();
        settings.add_recent_project(PathBuf::from("/test/project1"));
        settings.add_recent_project(PathBuf::from("/test/project2"));

        assert_eq!(settings.recent_projects.len(), 2);
        assert_eq!(settings.recent_projects[0], PathBuf::from("/test/project2"));
    }

    #[test]
    fn test_pinned_projects() {
        let mut settings = Settings::default();
        settings.pin_project("test.project".to_string());

        assert!(settings.is_pinned("test.project"));
        assert!(!settings.is_pinned("other.project"));

        settings.unpin_project("test.project");
        assert!(!settings.is_pinned("test.project"));
    }
}
