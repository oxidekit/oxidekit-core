//! Admin Platform State Management
//!
//! Provides centralized state management for the admin platform including:
//! - Project discovery and management
//! - Plugin registry and configuration
//! - Theme management
//! - Application settings
//! - Update tracking

mod admin_state;
mod project;
mod plugin;
mod theme;
mod settings;
pub mod updates;

pub use admin_state::*;
pub use project::*;
pub use plugin::*;
pub use theme::*;
pub use settings::*;
pub use updates::*;
