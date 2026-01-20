//! Responsive Layout System
//!
//! Provides breakpoint-based responsive layouts for mobile, tablet, and desktop devices.

use std::collections::HashMap;

/// Screen orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// Breakpoint definitions for responsive design
///
/// These breakpoints follow common responsive design conventions:
/// - Xs: Extra small devices (phones in portrait)
/// - Sm: Small devices (phones in landscape)
/// - Md: Medium devices (tablets in portrait)
/// - Lg: Large devices (tablets in landscape)
/// - Xl: Extra large devices (desktops)
/// - Xxl: Extra extra large devices (large desktops)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Breakpoint {
    /// < 576px (phones portrait)
    Xs,
    /// >= 576px (phones landscape)
    Sm,
    /// >= 768px (tablets portrait)
    Md,
    /// >= 992px (tablets landscape)
    Lg,
    /// >= 1200px (desktops)
    Xl,
    /// >= 1400px (large desktops)
    Xxl,
}

impl Breakpoint {
    /// Get the minimum width in pixels for this breakpoint
    pub fn min_width(&self) -> f32 {
        match self {
            Breakpoint::Xs => 0.0,
            Breakpoint::Sm => 576.0,
            Breakpoint::Md => 768.0,
            Breakpoint::Lg => 992.0,
            Breakpoint::Xl => 1200.0,
            Breakpoint::Xxl => 1400.0,
        }
    }

    /// Determine the breakpoint from a screen width
    pub fn from_width(width: f32) -> Self {
        if width >= 1400.0 {
            Breakpoint::Xxl
        } else if width >= 1200.0 {
            Breakpoint::Xl
        } else if width >= 992.0 {
            Breakpoint::Lg
        } else if width >= 768.0 {
            Breakpoint::Md
        } else if width >= 576.0 {
            Breakpoint::Sm
        } else {
            Breakpoint::Xs
        }
    }

    /// Get all breakpoints from smallest to largest
    pub fn all() -> [Breakpoint; 6] {
        [
            Breakpoint::Xs,
            Breakpoint::Sm,
            Breakpoint::Md,
            Breakpoint::Lg,
            Breakpoint::Xl,
            Breakpoint::Xxl,
        ]
    }

    /// Check if this breakpoint is at or above the given breakpoint
    pub fn is_at_least(&self, other: Breakpoint) -> bool {
        *self >= other
    }

    /// Check if this breakpoint is at or below the given breakpoint
    pub fn is_at_most(&self, other: Breakpoint) -> bool {
        *self <= other
    }
}

/// Responsive value that changes based on breakpoint
///
/// Allows defining different values for different screen sizes.
/// Values cascade down from larger breakpoints if not explicitly set.
///
/// # Example
///
/// ```
/// use oxide_layout::responsive::{Responsive, Breakpoint};
///
/// let font_size = Responsive::new(14.0)
///     .at(Breakpoint::Md, 16.0)
///     .at(Breakpoint::Lg, 18.0);
///
/// // On mobile (Xs or Sm), returns 14.0
/// assert_eq!(*font_size.resolve(Breakpoint::Xs), 14.0);
/// assert_eq!(*font_size.resolve(Breakpoint::Sm), 14.0);
///
/// // On tablet (Md), returns 16.0
/// assert_eq!(*font_size.resolve(Breakpoint::Md), 16.0);
///
/// // On desktop (Lg, Xl, Xxl), returns 18.0
/// assert_eq!(*font_size.resolve(Breakpoint::Lg), 18.0);
/// ```
#[derive(Debug, Clone)]
pub struct Responsive<T> {
    base: T,
    overrides: HashMap<Breakpoint, T>,
}

impl<T: Clone> Responsive<T> {
    /// Create a new responsive value with a base (mobile-first) value
    pub fn new(base: T) -> Self {
        Self {
            base,
            overrides: HashMap::new(),
        }
    }

    /// Set the value for a specific breakpoint and above
    pub fn at(mut self, breakpoint: Breakpoint, value: T) -> Self {
        self.overrides.insert(breakpoint, value);
        self
    }

    /// Resolve the value for the current breakpoint
    ///
    /// Returns the value for the largest breakpoint that is at or below
    /// the current breakpoint. Falls back to the base value if no override exists.
    pub fn resolve(&self, current_breakpoint: Breakpoint) -> &T {
        // Check breakpoints from current down to Xs
        let all = Breakpoint::all();
        let current_idx = all.iter().position(|&b| b == current_breakpoint).unwrap();

        for i in (0..=current_idx).rev() {
            if let Some(value) = self.overrides.get(&all[i]) {
                return value;
            }
        }

        &self.base
    }

    /// Check if a specific breakpoint has an override
    pub fn has_override(&self, breakpoint: Breakpoint) -> bool {
        self.overrides.contains_key(&breakpoint)
    }

    /// Get the base value
    pub fn base(&self) -> &T {
        &self.base
    }
}

impl<T: Clone + Default> Default for Responsive<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Breakpoint context for the layout system
///
/// Provides information about the current screen size and device characteristics.
#[derive(Debug, Clone)]
pub struct BreakpointContext {
    /// Current screen width in logical pixels
    pub width: f32,
    /// Current screen height in logical pixels
    pub height: f32,
    /// Current breakpoint based on width
    current: Breakpoint,
    /// Screen orientation
    pub orientation: Orientation,
    /// Device pixel ratio / scale factor
    pub scale_factor: f32,
}

impl BreakpointContext {
    /// Create a new breakpoint context from screen dimensions
    pub fn from_size(width: f32, height: f32, scale_factor: f32) -> Self {
        let current = Breakpoint::from_width(width);
        let orientation = if height > width {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        };

        Self {
            width,
            height,
            current,
            orientation,
            scale_factor,
        }
    }

    /// Get the current breakpoint
    pub fn current_breakpoint(&self) -> Breakpoint {
        self.current
    }

    /// Check if the current device is considered mobile (Xs or Sm)
    pub fn is_mobile(&self) -> bool {
        matches!(self.current, Breakpoint::Xs | Breakpoint::Sm)
    }

    /// Check if the current device is considered a tablet (Md or Lg)
    pub fn is_tablet(&self) -> bool {
        matches!(self.current, Breakpoint::Md | Breakpoint::Lg)
    }

    /// Check if the current device is considered a desktop (Xl or Xxl)
    pub fn is_desktop(&self) -> bool {
        matches!(self.current, Breakpoint::Xl | Breakpoint::Xxl)
    }

    /// Check if the screen is in portrait orientation
    pub fn is_portrait(&self) -> bool {
        self.orientation == Orientation::Portrait
    }

    /// Check if the screen is in landscape orientation
    pub fn is_landscape(&self) -> bool {
        self.orientation == Orientation::Landscape
    }

    /// Get the physical width in device pixels
    pub fn physical_width(&self) -> f32 {
        self.width * self.scale_factor
    }

    /// Get the physical height in device pixels
    pub fn physical_height(&self) -> f32 {
        self.height * self.scale_factor
    }

    /// Update the context with new dimensions
    pub fn update(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.current = Breakpoint::from_width(width);
        self.orientation = if height > width {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        };
    }

    /// Resolve a responsive value for the current breakpoint
    pub fn resolve<'a, T: Clone>(&self, responsive: &'a Responsive<T>) -> &'a T {
        responsive.resolve(self.current)
    }
}

impl Default for BreakpointContext {
    fn default() -> Self {
        // Default to a standard desktop size
        Self::from_size(1280.0, 720.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_from_width() {
        assert_eq!(Breakpoint::from_width(320.0), Breakpoint::Xs);
        assert_eq!(Breakpoint::from_width(575.0), Breakpoint::Xs);
        assert_eq!(Breakpoint::from_width(576.0), Breakpoint::Sm);
        assert_eq!(Breakpoint::from_width(767.0), Breakpoint::Sm);
        assert_eq!(Breakpoint::from_width(768.0), Breakpoint::Md);
        assert_eq!(Breakpoint::from_width(991.0), Breakpoint::Md);
        assert_eq!(Breakpoint::from_width(992.0), Breakpoint::Lg);
        assert_eq!(Breakpoint::from_width(1199.0), Breakpoint::Lg);
        assert_eq!(Breakpoint::from_width(1200.0), Breakpoint::Xl);
        assert_eq!(Breakpoint::from_width(1399.0), Breakpoint::Xl);
        assert_eq!(Breakpoint::from_width(1400.0), Breakpoint::Xxl);
        assert_eq!(Breakpoint::from_width(1920.0), Breakpoint::Xxl);
    }

    #[test]
    fn test_breakpoint_ordering() {
        assert!(Breakpoint::Xs < Breakpoint::Sm);
        assert!(Breakpoint::Sm < Breakpoint::Md);
        assert!(Breakpoint::Md < Breakpoint::Lg);
        assert!(Breakpoint::Lg < Breakpoint::Xl);
        assert!(Breakpoint::Xl < Breakpoint::Xxl);
    }

    #[test]
    fn test_responsive_value() {
        let padding = Responsive::new(8.0)
            .at(Breakpoint::Md, 16.0)
            .at(Breakpoint::Lg, 24.0);

        assert_eq!(*padding.resolve(Breakpoint::Xs), 8.0);
        assert_eq!(*padding.resolve(Breakpoint::Sm), 8.0);
        assert_eq!(*padding.resolve(Breakpoint::Md), 16.0);
        assert_eq!(*padding.resolve(Breakpoint::Lg), 24.0);
        assert_eq!(*padding.resolve(Breakpoint::Xl), 24.0);
        assert_eq!(*padding.resolve(Breakpoint::Xxl), 24.0);
    }

    #[test]
    fn test_responsive_base_only() {
        let value = Responsive::new(42);
        for bp in Breakpoint::all() {
            assert_eq!(*value.resolve(bp), 42);
        }
    }

    #[test]
    fn test_breakpoint_context() {
        // Mobile phone portrait
        let ctx = BreakpointContext::from_size(375.0, 667.0, 2.0);
        assert!(ctx.is_mobile());
        assert!(!ctx.is_tablet());
        assert!(!ctx.is_desktop());
        assert!(ctx.is_portrait());
        assert_eq!(ctx.physical_width(), 750.0);

        // Tablet landscape
        let ctx = BreakpointContext::from_size(1024.0, 768.0, 2.0);
        assert!(!ctx.is_mobile());
        assert!(ctx.is_tablet());
        assert!(!ctx.is_desktop());
        assert!(ctx.is_landscape());

        // Desktop
        let ctx = BreakpointContext::from_size(1920.0, 1080.0, 1.0);
        assert!(!ctx.is_mobile());
        assert!(!ctx.is_tablet());
        assert!(ctx.is_desktop());
    }

    #[test]
    fn test_breakpoint_context_update() {
        let mut ctx = BreakpointContext::from_size(375.0, 667.0, 2.0);
        assert!(ctx.is_mobile());

        ctx.update(1024.0, 768.0);
        assert!(ctx.is_tablet());
        assert!(ctx.is_landscape());
    }

    #[test]
    fn test_context_resolve() {
        let columns = Responsive::new(1)
            .at(Breakpoint::Md, 2)
            .at(Breakpoint::Lg, 3)
            .at(Breakpoint::Xl, 4);

        let mobile_ctx = BreakpointContext::from_size(375.0, 667.0, 2.0);
        assert_eq!(*mobile_ctx.resolve(&columns), 1);

        let tablet_ctx = BreakpointContext::from_size(768.0, 1024.0, 2.0);
        assert_eq!(*tablet_ctx.resolve(&columns), 2);

        let desktop_ctx = BreakpointContext::from_size(1280.0, 720.0, 1.0);
        assert_eq!(*desktop_ctx.resolve(&columns), 4);
    }
}
