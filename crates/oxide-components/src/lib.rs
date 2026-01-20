//! OxideKit Component System
//!
//! Provides a structured, machine-readable UI component system with:
//! - Component registration and discovery
//! - Design tokens and theming
//! - Font registry and typography roles
//! - Validation and AI compatibility
//! - Animation system with easing, interpolation, and timelines
//! - Responsive design system with breakpoints, media queries, and container queries
//! - WCAG 2.1 AA compliant accessibility system

pub mod accessibility;
pub mod animation;
pub mod registry;
pub mod responsive;
pub mod spec;
pub mod theme;
pub mod typography;
pub mod validation;

pub use accessibility::*;
pub use registry::*;
pub use responsive::*;
pub use spec::*;
pub use theme::*;
pub use typography::*;
pub use validation::*;

/// Re-export for convenience
pub mod prelude {
    pub use crate::accessibility::{
        AccessibilityPreferences, AccessibilityTree, AccessibleNode, AriaAttributes, AriaRole,
        FocusManager, FocusTrap, KeyboardNavigator, LabelManager, NavigationPattern,
        ScreenReaderAnnouncer,
    };
    pub use crate::animation::prelude::*;
    pub use crate::registry::ComponentRegistry;
    pub use crate::responsive::{
        AspectRatio, Breakpoint, ColorScheme, ContainerBreakpoint, ContainerQuery,
        ContainerResponsiveValue, ContainerSize, DeviceCategory, MediaQuery, Orientation,
        ResponsiveValue, SafeAreaEdges, SafeAreaInsets, ViewportContext,
    };
    pub use crate::spec::{ComponentSpec, EventSpec, PropSpec, PropType, SlotSpec};
    pub use crate::theme::{ColorToken, DesignTokens, SpacingToken, Theme};
    pub use crate::typography::{FontFamily, FontRegistry, TypographyRole};
    pub use crate::validation::{ValidationError, ValidationResult};
}
