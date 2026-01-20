//! OxideKit Starters
//!
//! First-class starter templates and boilerplate system for OxideKit.
//!
//! Starters allow developers to go from zero to working app/site/docs in minutes,
//! with real layouts, components, plugins, and production-grade structure.
//!
//! # Key Concepts
//!
//! - **StarterSpec**: Metadata defining a starter (plugins, permissions, files, etc.)
//! - **StarterRegistry**: Discovery and management of available starters
//! - **StarterGenerator**: Project generation from starter templates
//!
//! # Example
//!
//! ```ignore
//! use oxide_starters::{StarterRegistry, StarterGenerator};
//!
//! let registry = StarterRegistry::with_builtin();
//! let starter = registry.get("admin-panel").unwrap();
//! let generator = StarterGenerator::new(starter);
//! generator.generate("my-app", "/path/to/output")?;
//! ```

mod spec;
mod registry;
mod generator;
mod templates;

pub use spec::*;
pub use registry::*;
pub use generator::*;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        StarterSpec, StarterMetadata, StarterTarget, StarterCategory,
        PluginRequirement, PermissionPreset, GeneratedFile,
        StarterRegistry, StarterGenerator,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry() {
        let registry = StarterRegistry::with_builtin();
        assert!(registry.list().len() > 0);
    }

    #[test]
    fn test_get_admin_starter() {
        let registry = StarterRegistry::with_builtin();
        let starter = registry.get("admin-panel");
        assert!(starter.is_some());
    }
}
