//! Capability definitions and permission model for OxideKit.
//!
//! This module defines the canonical capability and permission system used
//! throughout OxideKit for declaring and enforcing application permissions.

mod types;
mod manifest;
mod registry;

pub use types::*;
pub use manifest::*;
pub use registry::*;
