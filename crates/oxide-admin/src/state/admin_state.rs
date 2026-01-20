//! Main admin state container

use super::{PluginInfo, PluginRegistry, ProjectInfo, ProjectRegistry, Settings, ThemeInfo, ThemeRegistry, UpdateInfo};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};

/// Navigation route for the admin platform
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AdminRoute {
    #[default]
    Dashboard,
    Projects,
    ProjectDetail(String),
    Plugins,
    PluginDetail(String),
    Themes,
    ThemePreview(String),
    Settings,
    Diagnostics,
    Updates,
}

/// View mode for lists
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
    Table,
}

/// Main admin application state
pub struct AdminState {
    /// Current navigation route
    pub route: AdminRoute,

    /// Project registry
    pub projects: ProjectRegistry,

    /// Plugin registry
    pub plugins: PluginRegistry,

    /// Theme registry
    pub themes: ThemeRegistry,

    /// Application settings
    pub settings: Settings,

    /// Update information
    pub updates: Option<UpdateInfo>,

    /// Search query (global)
    pub search_query: String,

    /// Current view mode
    pub view_mode: ViewMode,

    /// Sidebar collapsed state
    pub sidebar_collapsed: bool,

    /// Command palette open state
    pub command_palette_open: bool,

    /// Configuration file path
    config_path: Option<PathBuf>,

    /// Loading states
    pub loading: LoadingState,

    /// Toast notifications queue
    pub toasts: Vec<Toast>,

    /// Modal state
    pub modal: Option<ModalState>,
}

/// Loading state for async operations
#[derive(Debug, Default)]
pub struct LoadingState {
    pub projects: bool,
    pub plugins: bool,
    pub themes: bool,
    pub updates: bool,
    pub diagnostics: bool,
}

/// Toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: uuid::Uuid,
    pub message: String,
    pub kind: ToastKind,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
}

/// Toast notification kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastKind {
    Success,
    Warning,
    Error,
    Info,
}

/// Modal dialog state
#[derive(Debug, Clone)]
pub struct ModalState {
    pub title: String,
    pub content: ModalContent,
    pub actions: Vec<ModalAction>,
}

/// Modal content types
#[derive(Debug, Clone)]
pub enum ModalContent {
    Confirm(String),
    Input { label: String, placeholder: String, value: String },
    ProjectCreate,
    PluginInstall,
    ThemeCreate,
    Custom(String),
}

/// Modal action button
#[derive(Debug, Clone)]
pub struct ModalAction {
    pub label: String,
    pub variant: ModalActionVariant,
    pub action_id: String,
}

/// Modal action button variant
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalActionVariant {
    Primary,
    Secondary,
    Danger,
}

impl AdminState {
    /// Create a new admin state
    pub fn new() -> Self {
        Self {
            route: AdminRoute::default(),
            projects: ProjectRegistry::new(),
            plugins: PluginRegistry::new(),
            themes: ThemeRegistry::new(),
            settings: Settings::default(),
            updates: None,
            search_query: String::new(),
            view_mode: ViewMode::default(),
            sidebar_collapsed: false,
            command_palette_open: false,
            config_path: None,
            loading: LoadingState::default(),
            toasts: Vec::new(),
            modal: None,
        }
    }

    /// Set the configuration file path
    pub fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = Some(path);
    }

    /// Load configuration from file
    pub fn load_config(&mut self) -> Result<()> {
        let config_path = self.config_path.clone().unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("oxidekit")
                .join("admin.toml")
        });

        if config_path.exists() {
            info!("Loading config from {:?}", config_path);
            let content = std::fs::read_to_string(&config_path)?;
            self.settings = toml::from_str(&content)?;
        } else {
            info!("No config file found, using defaults");
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save_config(&self) -> Result<()> {
        let config_path = self.config_path.clone().unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("oxidekit")
                .join("admin.toml")
        });

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(&self.settings)?;
        std::fs::write(&config_path, content)?;

        info!("Saved config to {:?}", config_path);
        Ok(())
    }

    /// Scan for OxideKit projects
    pub fn scan_projects(&mut self) -> Result<()> {
        self.loading.projects = true;
        info!("Scanning for OxideKit projects...");

        // Scan configured project directories
        for dir in &self.settings.project_directories {
            if let Err(e) = self.projects.scan_directory(dir) {
                warn!("Failed to scan directory {:?}: {}", dir, e);
            }
        }

        // Also scan common locations
        if let Some(home) = dirs::home_dir() {
            let dev_dirs = ["workspace", "projects", "dev", "code"];
            for dir in dev_dirs {
                let path = home.join(dir);
                if path.exists() {
                    if let Err(e) = self.projects.scan_directory(&path) {
                        warn!("Failed to scan {:?}: {}", path, e);
                    }
                }
            }
        }

        self.loading.projects = false;
        info!("Found {} projects", self.projects.len());
        Ok(())
    }

    /// Load installed plugins
    pub fn load_plugins(&mut self) -> Result<()> {
        self.loading.plugins = true;
        info!("Loading installed plugins...");

        // Load from plugins directory
        let plugins_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("oxidekit")
            .join("plugins");

        if plugins_dir.exists() {
            self.plugins.scan_directory(&plugins_dir)?;
        }

        self.loading.plugins = false;
        info!("Loaded {} plugins", self.plugins.len());
        Ok(())
    }

    /// Load available themes
    pub fn load_themes(&mut self) -> Result<()> {
        self.loading.themes = true;
        info!("Loading themes...");

        // Load built-in themes
        self.themes.add_builtin_themes();

        // Load custom themes from themes directory
        let themes_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("oxidekit")
            .join("themes");

        if themes_dir.exists() {
            self.themes.scan_directory(&themes_dir)?;
        }

        self.loading.themes = false;
        info!("Loaded {} themes", self.themes.len());
        Ok(())
    }

    /// Navigate to a route
    pub fn navigate(&mut self, route: AdminRoute) {
        self.route = route;
    }

    /// Toggle sidebar collapsed state
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
    }

    /// Toggle command palette
    pub fn toggle_command_palette(&mut self) {
        self.command_palette_open = !self.command_palette_open;
    }

    /// Add a toast notification
    pub fn add_toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        let toast = Toast {
            id: uuid::Uuid::new_v4(),
            message: message.into(),
            kind,
            created_at: chrono::Utc::now(),
            duration_ms: 5000,
        };
        self.toasts.push(toast);
    }

    /// Remove a toast by ID
    pub fn remove_toast(&mut self, id: uuid::Uuid) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Show a modal
    pub fn show_modal(&mut self, title: impl Into<String>, content: ModalContent, actions: Vec<ModalAction>) {
        self.modal = Some(ModalState {
            title: title.into(),
            content,
            actions,
        });
    }

    /// Close the modal
    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    /// Get current project (if viewing project detail)
    pub fn current_project(&self) -> Option<&ProjectInfo> {
        if let AdminRoute::ProjectDetail(id) = &self.route {
            self.projects.get(id)
        } else {
            None
        }
    }

    /// Get current plugin (if viewing plugin detail)
    pub fn current_plugin(&self) -> Option<&PluginInfo> {
        if let AdminRoute::PluginDetail(id) = &self.route {
            self.plugins.get(id)
        } else {
            None
        }
    }

    /// Get current theme (if previewing)
    pub fn current_theme(&self) -> Option<&ThemeInfo> {
        if let AdminRoute::ThemePreview(id) = &self.route {
            self.themes.get(id)
        } else {
            None
        }
    }

    /// Filter projects by search query
    pub fn filtered_projects(&self) -> Vec<&ProjectInfo> {
        if self.search_query.is_empty() {
            self.projects.all()
        } else {
            self.projects.search(&self.search_query)
        }
    }

    /// Filter plugins by search query
    pub fn filtered_plugins(&self) -> Vec<&PluginInfo> {
        if self.search_query.is_empty() {
            self.plugins.all()
        } else {
            self.plugins.search(&self.search_query)
        }
    }

    /// Get dashboard statistics
    pub fn dashboard_stats(&self) -> DashboardStats {
        DashboardStats {
            total_projects: self.projects.len(),
            active_projects: self.projects.active_count(),
            installed_plugins: self.plugins.len(),
            enabled_plugins: self.plugins.enabled_count(),
            available_themes: self.themes.len(),
            updates_available: self.updates.as_ref().map(|u| u.available_count()).unwrap_or(0),
        }
    }
}

impl Default for AdminState {
    fn default() -> Self {
        Self::new()
    }
}

/// Dashboard statistics
#[derive(Debug, Clone, Default)]
pub struct DashboardStats {
    pub total_projects: usize,
    pub active_projects: usize,
    pub installed_plugins: usize,
    pub enabled_plugins: usize,
    pub available_themes: usize,
    pub updates_available: usize,
}

/// Directory utilities (using dirs crate pattern)
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }

    pub fn config_dir() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            home_dir().map(|h| h.join("Library/Application Support"))
        }
        #[cfg(target_os = "linux")]
        {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| home_dir().map(|h| h.join(".config")))
        }
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("APPDATA").map(PathBuf::from)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            home_dir().map(|h| h.join(".config"))
        }
    }

    pub fn data_dir() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            home_dir().map(|h| h.join("Library/Application Support"))
        }
        #[cfg(target_os = "linux")]
        {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|| home_dir().map(|h| h.join(".local/share")))
        }
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            home_dir().map(|h| h.join(".local/share"))
        }
    }
}
