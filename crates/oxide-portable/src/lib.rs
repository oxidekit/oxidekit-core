//! # OxideKit Portability System
//!
//! Ensures desktop decisions do not block future mobile and web targets.
//! Provides API markers, target abstractions, and portability checking.
//!
//! ## Core Concepts
//!
//! ### Portability Levels
//!
//! - **Portable**: Works on all targets (desktop, web, mobile)
//! - **Desktop Only**: Requires desktop OS features (filesystem, native windows)
//! - **Web Only**: Requires browser/WASM environment
//! - **Mobile Only**: Requires iOS or Android specific APIs
//!
//! ### Target Abstraction
//!
//! The target abstraction layer provides unified APIs that work across platforms,
//! with platform-specific implementations selected at compile time.
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use oxide_portable::{PortabilityLevel, Target, PortabilityChecker};
//!
//! // Check if an API is portable
//! let checker = PortabilityChecker::new();
//! let report = checker.check_crate("my-plugin")?;
//!
//! // Get current target info
//! let target = Target::current();
//! println!("Running on: {:?}", target.platform());
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod target;
pub mod level;
pub mod checker;
pub mod api;
pub mod plugin;
pub mod stubs;
mod error;

// Re-exports for convenient access
pub use target::{Target, Platform, Architecture, TargetFamily};
pub use level::{PortabilityLevel, ApiCategory, PortabilityConstraint};
pub use checker::{PortabilityChecker, PortabilityReport, PortabilityIssue, IssueSeverity};
pub use api::{ApiMarker, PortableApi, DesktopOnlyApi, WebOnlyApi, MobileOnlyApi};
pub use plugin::{PluginPortability, PortabilityManifest};
pub use error::{PortabilityError, PortabilityResult};

// Re-export macros when the feature is enabled
#[cfg(feature = "macros")]
pub use oxide_portable_macros::{portable, desktop_only, web_only, mobile_only, target_specific};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::target::{Target, Platform};
    pub use crate::level::PortabilityLevel;
    pub use crate::api::{PortableApi, DesktopOnlyApi};
    pub use crate::checker::PortabilityChecker;

    #[cfg(feature = "macros")]
    pub use oxide_portable_macros::{portable, desktop_only};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_target() {
        let target = Target::current();
        // Should not panic and return valid target
        assert!(!target.triple().is_empty());
    }

    #[test]
    fn test_portability_levels() {
        assert!(PortabilityLevel::Portable.is_portable());
        assert!(!PortabilityLevel::DesktopOnly.is_portable());
    }
}
