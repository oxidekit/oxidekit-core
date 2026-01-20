//! # OxideKit Mobile Platform Support
//!
//! Provides mobile platform support for OxideKit applications targeting iOS and Android.
//!
//! ## Core Concepts
//!
//! ### Responsive Design
//!
//! - **Breakpoints**: Screen size categories (xs/sm/md/lg/xl) for responsive layouts
//! - **Safe Areas**: Handle device notches, home indicators, and system UI
//! - **Screen Density**: Handle various screen densities (ldpi to xxxhdpi)
//!
//! ### Platform Integration
//!
//! - **Shell**: Native platform lifecycle and window management
//! - **Input**: Touch events and gesture recognition
//! - **IME**: Mobile input method editor integration
//!
//! ### Build System
//!
//! - **Build Outputs**: Generate APK/AAB for Android, IPA for iOS
//! - **Code Signing**: Manage certificates and provisioning profiles
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use oxide_mobile::prelude::*;
//!
//! // Configure breakpoints
//! let breakpoints = BreakpointConfig::default();
//! let current = breakpoints.get(screen_width);
//!
//! // Handle safe areas
//! let insets = SafeAreaInsets::from_device();
//! let content_rect = insets.apply_to_rect(screen_rect);
//!
//! // Handle touch input
//! fn on_touch(event: TouchEvent) {
//!     match event.phase {
//!         TouchPhase::Began => start_interaction(event.position),
//!         TouchPhase::Moved => update_interaction(event.position),
//!         TouchPhase::Ended => end_interaction(),
//!         TouchPhase::Cancelled => cancel_interaction(),
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod error;
pub mod target;
pub mod config;
pub mod responsive;
pub mod shell;
pub mod input;
pub mod ime;
pub mod build;
pub mod signing;

// Re-exports for convenient access
pub use error::{MobileError, MobileResult};
pub use target::{MobileTarget, MobilePlatform, DeviceClass};
pub use config::MobileConfig;

// Responsive re-exports
pub use responsive::{
    Breakpoint, BreakpointConfig,
    SafeAreaInsets, SafeAreaEdge,
    ScreenDensity, DensityBucket,
};

// Shell re-exports
pub use shell::{MobileLifecycleEvent, Orientation, MobileShell};

// Input re-exports
pub use input::{
    TouchEvent, TouchPhase, TouchPoint,
    GestureType, GestureState, GestureRecognizer,
};

// Build re-exports
pub use build::{BuildOutput, AndroidBuildOutput, IosBuildOutput};

// Signing re-exports
pub use signing::{SigningConfig, SigningIdentity};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{MobileError, MobileResult};
    pub use crate::target::{MobileTarget, MobilePlatform};
    pub use crate::config::MobileConfig;

    // Responsive
    pub use crate::responsive::{
        Breakpoint, BreakpointConfig,
        SafeAreaInsets,
        ScreenDensity, DensityBucket,
    };

    // Shell
    pub use crate::shell::{MobileLifecycleEvent, Orientation, MobileShell};

    // Input
    pub use crate::input::{
        TouchEvent, TouchPhase,
        GestureType, GestureState, GestureRecognizer,
    };

    // Build
    pub use crate::build::{BuildOutput, AndroidBuildOutput, IosBuildOutput};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_target_creation() {
        let target = MobileTarget::IosDevice;
        assert!(matches!(target.platform(), MobilePlatform::Ios));
    }

    #[test]
    fn test_breakpoint_default() {
        let config = BreakpointConfig::default();
        assert!(config.thresholds().contains_key(&Breakpoint::Sm));
    }

    #[test]
    fn test_safe_area_insets_default() {
        let insets = SafeAreaInsets::default();
        assert_eq!(insets.top(), 0.0);
        assert_eq!(insets.bottom(), 0.0);
    }
}
