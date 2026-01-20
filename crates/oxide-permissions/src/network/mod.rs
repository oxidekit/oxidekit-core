//! Network allowlist enforcement for OxideKit.
//!
//! This module provides network policy enforcement including:
//! - Domain allowlist/denylist
//! - Private IP range blocking
//! - DNS resolution validation
//! - Connection monitoring

mod policy;
mod enforcer;
mod resolver;

pub use policy::*;
pub use enforcer::*;
pub use resolver::*;
