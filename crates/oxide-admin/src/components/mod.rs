//! Admin UI Components
//!
//! Production-ready UI components for the admin platform:
//! - Layout components (sidebar, topbar, split panes)
//! - Data display (cards, tables, charts)
//! - Form controls (buttons, inputs, selects)
//! - Feedback (alerts, toasts, modals)
//! - Navigation (breadcrumbs, tabs, command palette)

mod button;
mod card;
mod table;
mod sidebar;
mod topbar;
mod modal;
mod toast;
mod alert;
mod tabs;
mod breadcrumb;
mod command_palette;
mod form;
mod skeleton;
mod empty_state;
mod chart;
mod badge;
mod avatar;
mod icon;

pub use button::*;
pub use card::*;
pub use table::*;
pub use sidebar::*;
pub use topbar::*;
pub use modal::*;
pub use toast::*;
pub use alert::*;
pub use tabs::*;
pub use breadcrumb::*;
pub use command_palette::*;
pub use form::*;
pub use skeleton::*;
pub use empty_state::*;
pub use chart::*;
pub use badge::*;
pub use avatar::*;
pub use icon::*;
