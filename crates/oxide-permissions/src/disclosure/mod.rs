//! Permission disclosure UI components.
//!
//! Provides the data models and utilities for displaying permission
//! information to users, including first-launch consent screens and
//! in-app permission pages.

mod prompt;
mod page;
mod consent;

pub use prompt::*;
pub use page::*;
pub use consent::*;
