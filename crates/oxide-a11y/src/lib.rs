//! Accessibility system for OxideKit
//!
//! Provides accessibility tree, roles, states, and focus management.

pub mod focus;
pub mod role;
pub mod state;
pub mod tree;

pub use focus::*;
pub use role::*;
pub use state::*;
pub use tree::*;
