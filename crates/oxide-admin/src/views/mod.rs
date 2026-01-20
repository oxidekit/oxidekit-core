//! Admin Platform Views
//!
//! Full-page views for the admin platform:
//! - Dashboard - Overview with stats and quick actions
//! - Projects - Project management and listing
//! - Plugins - Plugin/extension management
//! - Themes - Theme preview and customization
//! - Settings - Application configuration
//! - Diagnostics - Debug and diagnostic viewer
//! - Updates - Update checker and installer

mod dashboard;
mod projects;
mod plugins;
mod themes;
mod settings;
mod diagnostics;
mod updates;
mod layout;

pub use dashboard::*;
pub use projects::*;
pub use plugins::*;
pub use themes::*;
pub use settings::*;
pub use diagnostics::*;
pub use updates::*;
pub use layout::*;
