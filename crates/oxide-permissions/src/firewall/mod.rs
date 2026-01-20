//! Capability Firewall - Runtime enforcement of permissions.
//!
//! The firewall acts as a gatekeeper for all sensitive operations,
//! ensuring that only declared and granted capabilities are allowed.

mod enforcer;
mod guard;
mod policy;

pub use enforcer::*;
pub use guard::*;
pub use policy::*;
