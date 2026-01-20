//! Verified Build Profile - Policy-driven build hardening.
//!
//! Provides checks and enforcement for creating verified builds that
//! meet OxideKit's security requirements.

mod profile;
mod checker;
mod forbidden;

pub use profile::*;
pub use checker::*;
pub use forbidden::*;
