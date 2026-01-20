//! OxideKit Component System
//!
//! Provides a structured, machine-readable UI component system with:
//! - Component registration and discovery
//! - Design tokens and theming
//! - Font registry and typography roles
//! - Validation and AI compatibility

pub mod registry;
pub mod spec;
pub mod theme;
pub mod typography;
pub mod validation;

pub use registry::*;
pub use spec::*;
pub use theme::*;
pub use typography::*;
pub use validation::*;

/// Re-export for convenience
pub mod prelude {
    pub use crate::registry::ComponentRegistry;
    pub use crate::spec::{ComponentSpec, PropSpec, PropType, EventSpec, SlotSpec};
    pub use crate::theme::{Theme, DesignTokens, ColorToken, SpacingToken};
    pub use crate::typography::{FontRegistry, TypographyRole, FontFamily};
    pub use crate::validation::{ValidationError, ValidationResult};
}
