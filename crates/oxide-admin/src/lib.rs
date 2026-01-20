//! OxideKit Admin Platform
//!
//! A production-grade admin dashboard built with OxideKit itself (dogfooding).
//! Provides comprehensive tools for:
//! - Project management and configuration
//! - Plugin and extension management
//! - Theme preview and customization
//! - Diagnostic viewing and debugging
//! - Update checking and installation
//!
//! # Architecture
//!
//! The admin platform follows a component-based architecture:
//! - `components/` - Reusable UI components (sidebar, topbar, cards, tables, etc.)
//! - `views/` - Full-page views (dashboard, projects, plugins, themes, settings)
//! - `state/` - Application state management
//! - `utils/` - Utility functions and helpers

pub mod components;
pub mod state;
pub mod utils;
pub mod views;

use anyhow::Result;
use state::AdminState;
use std::path::PathBuf;
use tracing::info;

/// The OxideKit Admin Application
pub struct AdminApp {
    state: AdminState,
    config_path: Option<PathBuf>,
}

impl AdminApp {
    /// Create a new admin application instance
    pub fn new() -> Self {
        Self {
            state: AdminState::new(),
            config_path: None,
        }
    }

    /// Create admin app with custom config path
    pub fn with_config(config_path: PathBuf) -> Self {
        let mut app = Self::new();
        app.config_path = Some(config_path.clone());
        app.state.set_config_path(config_path);
        app
    }

    /// Initialize the admin application
    pub fn init(&mut self) -> Result<()> {
        info!("Initializing OxideKit Admin Platform");

        // Load configuration
        self.state.load_config()?;

        // Scan for projects
        self.state.scan_projects()?;

        // Load installed plugins
        self.state.load_plugins()?;

        // Load available themes
        self.state.load_themes()?;

        info!("Admin platform initialized successfully");
        Ok(())
    }

    /// Run the admin application (launches the native window)
    pub fn run(mut self) -> Result<()> {
        self.init()?;

        info!("Launching OxideKit Admin Platform window");

        // Write a temporary manifest file for the admin app
        let temp_dir = std::env::temp_dir();
        let manifest_path = temp_dir.join("oxide-admin-manifest.toml");

        let manifest_content = format!(
            r#"[app]
id = "dev.oxidekit.admin"
name = "OxideKit Admin"
version = "{}"
description = "OxideKit Admin Platform"

[window]
title = "OxideKit Admin"
width = 1400
height = 900
min_width = 1000
min_height = 600
resizable = true
decorations = true

[dev]
hot_reload = false
inspector = {}
"#,
            env!("CARGO_PKG_VERSION"),
            cfg!(debug_assertions)
        );

        std::fs::write(&manifest_path, manifest_content)?;

        // Load and run using the runtime
        let app = oxide_runtime::Application::from_manifest(&manifest_path)?;
        app.run()
    }

    /// Get a reference to the admin state
    pub fn state(&self) -> &AdminState {
        &self.state
    }

    /// Get a mutable reference to the admin state
    pub fn state_mut(&mut self) -> &mut AdminState {
        &mut self.state
    }
}

impl Default for AdminApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Launch the admin platform
pub fn launch() -> Result<()> {
    let app = AdminApp::new();
    app.run()
}

/// Launch the admin platform with custom configuration
pub fn launch_with_config(config_path: &std::path::Path) -> Result<()> {
    let app = AdminApp::with_config(config_path.to_path_buf());
    app.run()
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::components::*;
    pub use crate::state::{AdminState, ProjectInfo, PluginInfo, ThemeInfo};
    pub use crate::views::*;
    pub use crate::{AdminApp, launch};
}
