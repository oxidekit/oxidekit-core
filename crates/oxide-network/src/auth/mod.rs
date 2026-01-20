//! Authentication providers and management for OxideKit.
//!
//! Provides a unified auth system with:
//! - Multiple provider types (OAuth, JWT, API keys, Basic auth)
//! - Automatic token refresh
//! - Auth state management
//! - Credential storage integration

mod manager;
mod provider;
mod token;

pub use manager::*;
pub use provider::*;
pub use token::*;
