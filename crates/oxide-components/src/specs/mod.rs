//! Material Design 3 Component Specifications
//!
//! This module contains comprehensive specifications for M3 components following
//! Google's Material Design 3 guidelines. Each component specification includes:
//! - Property definitions with types and defaults
//! - Event handlers
//! - Accessibility roles and keyboard interactions
//! - Style tokens
//! - Usage examples

pub mod buttons;
pub mod cards;
pub mod dialogs;
pub mod lists;
pub mod navigation;
pub mod progress;
pub mod selection;
pub mod text_fields;

pub use buttons::*;
pub use cards::*;
pub use dialogs::*;
pub use lists::*;
pub use navigation::*;
pub use progress::*;
pub use selection::*;
pub use text_fields::*;

use crate::spec::{ComponentSpec, ComponentSpecBuilder};

/// Component category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentCategory {
    /// Action components (buttons, FABs)
    Action,
    /// Communication components (progress, snackbar)
    Communication,
    /// Containment components (cards, dialogs)
    Containment,
    /// Navigation components (bars, rails, drawers)
    Navigation,
    /// Selection components (checkboxes, radios, switches)
    Selection,
    /// Text input components (text fields)
    TextInput,
}

impl ComponentCategory {
    /// Returns the pack name for this category
    pub fn pack_name(&self) -> &'static str {
        match self {
            ComponentCategory::Action => "m3.action",
            ComponentCategory::Communication => "m3.communication",
            ComponentCategory::Containment => "m3.containment",
            ComponentCategory::Navigation => "m3.navigation",
            ComponentCategory::Selection => "m3.selection",
            ComponentCategory::TextInput => "m3.input",
        }
    }
}

/// Helper to create M3 component builder with standard settings
pub fn m3_component(id: &str, category: ComponentCategory) -> ComponentSpecBuilder {
    ComponentSpec::builder(id, category.pack_name()).version("1.0.0")
}
