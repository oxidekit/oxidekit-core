//! Marketplace integration for verified builds and badges.
//!
//! Provides APIs for marketplace systems to display trust information,
//! badges, and verification status.

mod display;
mod registry;

pub use display::*;
pub use registry::*;
