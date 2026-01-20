//! OxideKit Compatibility Layers
//!
//! **WARNING**: These compatibility layers are escape hatches for migration and edge cases.
//! They are NOT the recommended path for OxideKit applications.
//!
//! # Purpose
//!
//! This crate provides compatibility layers for users migrating from Electron/Tauri
//! or needing temporary access to web technologies:
//!
//! - **WebView Plugin** (`webview` feature): Embed web widgets in OxideKit apps
//! - **JS Runtime Plugin** (`js-runtime` feature): Run non-UI JavaScript utilities
//! - **NPM Tooling** (`npm-tooling` feature): Build-time NPM package bundling
//! - **Migration Helpers** (`migration` feature): Tools to migrate from Electron/Tauri
//! - **AI Gap-Filler** (`ai-gap-filler` feature): AI-powered component scaffolding
//!
//! # Security Notice
//!
//! These features increase attack surface and should be:
//! - Explicitly enabled in `oxide.toml`
//! - Clearly labeled in the marketplace
//! - Avoided in production where possible
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_compat::prelude::*;
//!
//! // Check if compatibility features are allowed
//! let policy = CompatPolicy::from_config("oxide.toml")?;
//! if !policy.allow_webview {
//!     eprintln!("WebView is disabled by policy");
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod policy;
pub mod naming;
pub mod scaffold;
pub mod migration;

#[cfg(feature = "webview")]
pub mod webview;

#[cfg(feature = "js-runtime")]
pub mod jsruntime;

#[cfg(feature = "npm-tooling")]
pub mod npm;

#[cfg(feature = "ai-gap-filler")]
pub mod gap_filler;

pub use policy::*;
pub use naming::*;
pub use scaffold::*;
pub use migration::*;

/// Prelude for common imports
pub mod prelude {
    pub use crate::policy::{CompatPolicy, PolicyViolation};
    pub use crate::naming::{CanonicalId, IdGenerator, Namespace};
    pub use crate::scaffold::{Scaffolder, ScaffoldKind, ScaffoldOptions, GeneratedScaffold};
    pub use crate::migration::{MigrationSource, MigrationAnalyzer, MigrationPlan, MigrationStep};

    #[cfg(feature = "webview")]
    pub use crate::webview::{WebWidget, WebWidgetConfig, MessageBridge};

    #[cfg(feature = "js-runtime")]
    pub use crate::jsruntime::{JsRuntime, JsRuntimeConfig, JsValue};

    #[cfg(feature = "npm-tooling")]
    pub use crate::npm::{NpmBundler, NpmPackage, BuildArtifact};

    #[cfg(feature = "ai-gap-filler")]
    pub use crate::gap_filler::{GapFiller, GapFillerResponse, ComponentSuggestion};
}

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compatibility layer schema version
pub const COMPAT_VERSION: &str = "1.0";

/// Warning message displayed when compatibility features are used
pub const COMPAT_WARNING: &str = r#"
╭─────────────────────────────────────────────────────────────────╮
│  WARNING: Compatibility Layer Active                            │
│                                                                 │
│  You are using OxideKit compatibility features which are NOT    │
│  recommended for production applications.                       │
│                                                                 │
│  These features:                                                │
│  - Increase attack surface                                      │
│  - May impact performance                                       │
│  - Should be migrated to native OxideKit components             │
│                                                                 │
│  Run `oxide doctor` to see compatibility usage in your project. │
╰─────────────────────────────────────────────────────────────────╯
"#;

/// Print the compatibility warning to stderr
pub fn print_compat_warning() {
    eprintln!("{}", COMPAT_WARNING);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_compat_version() {
        assert_eq!(COMPAT_VERSION, "1.0");
    }
}
