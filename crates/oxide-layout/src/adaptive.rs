//! Adaptive Navigation Patterns
//!
//! Provides adaptive layout patterns that change based on screen size,
//! enabling responsive navigation and content presentation across devices.

use crate::responsive::{Breakpoint, BreakpointContext};

/// Adaptive navigation patterns for different device types
///
/// Represents common navigation patterns used across mobile and desktop platforms:
/// - `BottomBar`: Bottom tab navigation, common on mobile apps
/// - `Drawer`: Hamburger menu with slide-out drawer, used on mobile
/// - `SplitView`: Master-detail split view, common on tablets
/// - `Sidebar`: Persistent sidebar navigation, used on desktop
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdaptiveNavigation {
    /// Bottom tab bar navigation (mobile)
    BottomBar,
    /// Hamburger menu with slide-out drawer (mobile)
    Drawer,
    /// Master-detail split view (tablet)
    SplitView,
    /// Persistent sidebar navigation (desktop)
    Sidebar,
}

impl AdaptiveNavigation {
    /// Get the recommended navigation pattern for a breakpoint
    pub fn for_breakpoint(breakpoint: Breakpoint) -> Self {
        match breakpoint {
            Breakpoint::Xs | Breakpoint::Sm => AdaptiveNavigation::BottomBar,
            Breakpoint::Md | Breakpoint::Lg => AdaptiveNavigation::SplitView,
            Breakpoint::Xl | Breakpoint::Xxl => AdaptiveNavigation::Sidebar,
        }
    }

    /// Check if this navigation pattern is suitable for mobile
    pub fn is_mobile_pattern(&self) -> bool {
        matches!(self, AdaptiveNavigation::BottomBar | AdaptiveNavigation::Drawer)
    }

    /// Check if this navigation pattern is suitable for tablet
    pub fn is_tablet_pattern(&self) -> bool {
        matches!(self, AdaptiveNavigation::SplitView)
    }

    /// Check if this navigation pattern is suitable for desktop
    pub fn is_desktop_pattern(&self) -> bool {
        matches!(self, AdaptiveNavigation::Sidebar)
    }
}

/// Layout trait for adaptive layouts
///
/// Implement this trait to define custom layouts that can be selected
/// based on the current breakpoint.
pub trait Layout: std::fmt::Debug {
    /// Get the name of this layout for debugging
    fn name(&self) -> &str;

    /// Check if this layout is suitable for the given breakpoint
    fn supports_breakpoint(&self, breakpoint: Breakpoint) -> bool;

    /// Get the recommended minimum width for this layout
    fn min_width(&self) -> f32;

    /// Get the recommended maximum width for this layout (None for unlimited)
    fn max_width(&self) -> Option<f32>;
}

/// A simple layout implementation for testing and basic use
#[derive(Debug, Clone)]
pub struct SimpleLayout {
    name: String,
    min_width: f32,
    max_width: Option<f32>,
    supported_breakpoints: Vec<Breakpoint>,
}

impl SimpleLayout {
    /// Create a new simple layout
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            min_width: 0.0,
            max_width: None,
            supported_breakpoints: Breakpoint::all().to_vec(),
        }
    }

    /// Set the minimum width
    pub fn with_min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Set the maximum width
    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set the supported breakpoints
    pub fn with_breakpoints(mut self, breakpoints: Vec<Breakpoint>) -> Self {
        self.supported_breakpoints = breakpoints;
        self
    }
}

impl Layout for SimpleLayout {
    fn name(&self) -> &str {
        &self.name
    }

    fn supports_breakpoint(&self, breakpoint: Breakpoint) -> bool {
        self.supported_breakpoints.contains(&breakpoint)
    }

    fn min_width(&self) -> f32 {
        self.min_width
    }

    fn max_width(&self) -> Option<f32> {
        self.max_width
    }
}

/// Adaptive layout that provides different layouts for different screen sizes
///
/// # Example
///
/// ```
/// use oxide_layout::adaptive::{AdaptiveLayout, SimpleLayout, Layout};
/// use oxide_layout::responsive::Breakpoint;
///
/// let mobile_layout = SimpleLayout::new("mobile")
///     .with_breakpoints(vec![Breakpoint::Xs, Breakpoint::Sm]);
///
/// let tablet_layout = SimpleLayout::new("tablet")
///     .with_breakpoints(vec![Breakpoint::Md, Breakpoint::Lg]);
///
/// let desktop_layout = SimpleLayout::new("desktop")
///     .with_breakpoints(vec![Breakpoint::Xl, Breakpoint::Xxl]);
///
/// let adaptive = AdaptiveLayout::new(mobile_layout, tablet_layout, desktop_layout);
///
/// assert_eq!(adaptive.resolve(Breakpoint::Xs).name(), "mobile");
/// assert_eq!(adaptive.resolve(Breakpoint::Md).name(), "tablet");
/// assert_eq!(adaptive.resolve(Breakpoint::Xl).name(), "desktop");
/// ```
#[derive(Debug)]
pub struct AdaptiveLayout<M: Layout, T: Layout, D: Layout> {
    mobile_layout: M,
    tablet_layout: T,
    desktop_layout: D,
}

impl<M: Layout, T: Layout, D: Layout> AdaptiveLayout<M, T, D> {
    /// Create a new adaptive layout with layouts for each device category
    pub fn new(mobile_layout: M, tablet_layout: T, desktop_layout: D) -> Self {
        Self {
            mobile_layout,
            tablet_layout,
            desktop_layout,
        }
    }

    /// Resolve the layout for the given breakpoint
    pub fn resolve(&self, breakpoint: Breakpoint) -> &dyn Layout {
        match breakpoint {
            Breakpoint::Xs | Breakpoint::Sm => &self.mobile_layout,
            Breakpoint::Md | Breakpoint::Lg => &self.tablet_layout,
            Breakpoint::Xl | Breakpoint::Xxl => &self.desktop_layout,
        }
    }

    /// Resolve the layout using a breakpoint context
    pub fn resolve_from_context(&self, context: &BreakpointContext) -> &dyn Layout {
        self.resolve(context.current_breakpoint())
    }

    /// Get the mobile layout
    pub fn mobile(&self) -> &M {
        &self.mobile_layout
    }

    /// Get the tablet layout
    pub fn tablet(&self) -> &T {
        &self.tablet_layout
    }

    /// Get the desktop layout
    pub fn desktop(&self) -> &D {
        &self.desktop_layout
    }
}

/// Boxed version of AdaptiveLayout for dynamic dispatch
#[derive(Debug)]
pub struct DynamicAdaptiveLayout {
    mobile_layout: Box<dyn Layout>,
    tablet_layout: Box<dyn Layout>,
    desktop_layout: Box<dyn Layout>,
}

impl DynamicAdaptiveLayout {
    /// Create a new dynamic adaptive layout
    pub fn new(
        mobile_layout: Box<dyn Layout>,
        tablet_layout: Box<dyn Layout>,
        desktop_layout: Box<dyn Layout>,
    ) -> Self {
        Self {
            mobile_layout,
            tablet_layout,
            desktop_layout,
        }
    }

    /// Resolve the layout for the given breakpoint
    pub fn resolve(&self, breakpoint: Breakpoint) -> &dyn Layout {
        match breakpoint {
            Breakpoint::Xs | Breakpoint::Sm => self.mobile_layout.as_ref(),
            Breakpoint::Md | Breakpoint::Lg => self.tablet_layout.as_ref(),
            Breakpoint::Xl | Breakpoint::Xxl => self.desktop_layout.as_ref(),
        }
    }

    /// Resolve the layout using a breakpoint context
    pub fn resolve_from_context(&self, context: &BreakpointContext) -> &dyn Layout {
        self.resolve(context.current_breakpoint())
    }
}

/// Configuration for a split view layout
#[derive(Debug, Clone, Copy)]
pub struct SplitViewConfig {
    /// Width of the master/primary pane
    pub primary_width: SplitViewWidth,
    /// Minimum width of the detail/secondary pane
    pub min_secondary_width: f32,
    /// Whether to show the primary pane when collapsed
    pub show_primary_when_collapsed: bool,
    /// Breakpoint at which to collapse the split view
    pub collapse_breakpoint: Breakpoint,
}

/// Width specification for split view primary pane
#[derive(Debug, Clone, Copy)]
pub enum SplitViewWidth {
    /// Fixed width in pixels
    Fixed(f32),
    /// Percentage of container width
    Percent(f32),
    /// Minimum and maximum width constraints
    Constrained { min: f32, max: f32 },
}

impl Default for SplitViewConfig {
    fn default() -> Self {
        Self {
            primary_width: SplitViewWidth::Fixed(320.0),
            min_secondary_width: 400.0,
            show_primary_when_collapsed: false,
            collapse_breakpoint: Breakpoint::Md,
        }
    }
}

impl SplitViewConfig {
    /// Create a new split view configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the primary pane width
    pub fn with_primary_width(mut self, width: SplitViewWidth) -> Self {
        self.primary_width = width;
        self
    }

    /// Set the minimum secondary pane width
    pub fn with_min_secondary_width(mut self, width: f32) -> Self {
        self.min_secondary_width = width;
        self
    }

    /// Set whether to show primary when collapsed
    pub fn with_show_primary_when_collapsed(mut self, show: bool) -> Self {
        self.show_primary_when_collapsed = show;
        self
    }

    /// Set the collapse breakpoint
    pub fn with_collapse_breakpoint(mut self, breakpoint: Breakpoint) -> Self {
        self.collapse_breakpoint = breakpoint;
        self
    }

    /// Check if the split view should be collapsed at the given breakpoint
    pub fn should_collapse(&self, breakpoint: Breakpoint) -> bool {
        breakpoint < self.collapse_breakpoint
    }

    /// Calculate the primary pane width for a given container width
    pub fn calculate_primary_width(&self, container_width: f32) -> f32 {
        match self.primary_width {
            SplitViewWidth::Fixed(width) => width,
            SplitViewWidth::Percent(pct) => container_width * pct,
            SplitViewWidth::Constrained { min, max } => {
                // Use 1/3 of container as default, constrained to min/max
                (container_width / 3.0).clamp(min, max)
            }
        }
    }
}

/// Configuration for a bottom navigation bar
#[derive(Debug, Clone)]
pub struct BottomNavConfig {
    /// Height of the navigation bar
    pub height: f32,
    /// Maximum number of items to show
    pub max_items: usize,
    /// Whether to show labels
    pub show_labels: bool,
    /// Whether to hide on scroll
    pub hide_on_scroll: bool,
}

impl Default for BottomNavConfig {
    fn default() -> Self {
        Self {
            height: 56.0,
            max_items: 5,
            show_labels: true,
            hide_on_scroll: false,
        }
    }
}

impl BottomNavConfig {
    /// Create a new bottom navigation configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the height
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set the maximum number of items
    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = max;
        self
    }

    /// Set whether to show labels
    pub fn with_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Set whether to hide on scroll
    pub fn with_hide_on_scroll(mut self, hide: bool) -> Self {
        self.hide_on_scroll = hide;
        self
    }
}

/// Configuration for a sidebar navigation
#[derive(Debug, Clone)]
pub struct SidebarConfig {
    /// Width of the sidebar when expanded
    pub expanded_width: f32,
    /// Width of the sidebar when collapsed (icon-only mode)
    pub collapsed_width: f32,
    /// Whether the sidebar can be collapsed
    pub collapsible: bool,
    /// Whether to start collapsed
    pub start_collapsed: bool,
    /// Breakpoint at which to auto-collapse
    pub auto_collapse_breakpoint: Option<Breakpoint>,
}

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            expanded_width: 256.0,
            collapsed_width: 64.0,
            collapsible: true,
            start_collapsed: false,
            auto_collapse_breakpoint: Some(Breakpoint::Lg),
        }
    }
}

impl SidebarConfig {
    /// Create a new sidebar configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the expanded width
    pub fn with_expanded_width(mut self, width: f32) -> Self {
        self.expanded_width = width;
        self
    }

    /// Set the collapsed width
    pub fn with_collapsed_width(mut self, width: f32) -> Self {
        self.collapsed_width = width;
        self
    }

    /// Set whether the sidebar is collapsible
    pub fn with_collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Set whether to start collapsed
    pub fn with_start_collapsed(mut self, collapsed: bool) -> Self {
        self.start_collapsed = collapsed;
        self
    }

    /// Set the auto-collapse breakpoint
    pub fn with_auto_collapse(mut self, breakpoint: Option<Breakpoint>) -> Self {
        self.auto_collapse_breakpoint = breakpoint;
        self
    }

    /// Check if the sidebar should auto-collapse at the given breakpoint
    pub fn should_auto_collapse(&self, breakpoint: Breakpoint) -> bool {
        self.collapsible
            && self
                .auto_collapse_breakpoint
                .map(|bp| breakpoint < bp)
                .unwrap_or(false)
    }

    /// Get the current width based on collapsed state
    pub fn width(&self, collapsed: bool) -> f32 {
        if collapsed && self.collapsible {
            self.collapsed_width
        } else {
            self.expanded_width
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_navigation_for_breakpoint() {
        assert_eq!(
            AdaptiveNavigation::for_breakpoint(Breakpoint::Xs),
            AdaptiveNavigation::BottomBar
        );
        assert_eq!(
            AdaptiveNavigation::for_breakpoint(Breakpoint::Sm),
            AdaptiveNavigation::BottomBar
        );
        assert_eq!(
            AdaptiveNavigation::for_breakpoint(Breakpoint::Md),
            AdaptiveNavigation::SplitView
        );
        assert_eq!(
            AdaptiveNavigation::for_breakpoint(Breakpoint::Lg),
            AdaptiveNavigation::SplitView
        );
        assert_eq!(
            AdaptiveNavigation::for_breakpoint(Breakpoint::Xl),
            AdaptiveNavigation::Sidebar
        );
        assert_eq!(
            AdaptiveNavigation::for_breakpoint(Breakpoint::Xxl),
            AdaptiveNavigation::Sidebar
        );
    }

    #[test]
    fn test_adaptive_navigation_patterns() {
        assert!(AdaptiveNavigation::BottomBar.is_mobile_pattern());
        assert!(AdaptiveNavigation::Drawer.is_mobile_pattern());
        assert!(!AdaptiveNavigation::SplitView.is_mobile_pattern());

        assert!(AdaptiveNavigation::SplitView.is_tablet_pattern());
        assert!(!AdaptiveNavigation::BottomBar.is_tablet_pattern());

        assert!(AdaptiveNavigation::Sidebar.is_desktop_pattern());
        assert!(!AdaptiveNavigation::BottomBar.is_desktop_pattern());
    }

    #[test]
    fn test_simple_layout() {
        let layout = SimpleLayout::new("test")
            .with_min_width(320.0)
            .with_max_width(768.0)
            .with_breakpoints(vec![Breakpoint::Xs, Breakpoint::Sm]);

        assert_eq!(layout.name(), "test");
        assert_eq!(layout.min_width(), 320.0);
        assert_eq!(layout.max_width(), Some(768.0));
        assert!(layout.supports_breakpoint(Breakpoint::Xs));
        assert!(layout.supports_breakpoint(Breakpoint::Sm));
        assert!(!layout.supports_breakpoint(Breakpoint::Md));
    }

    #[test]
    fn test_adaptive_layout_resolve() {
        let mobile = SimpleLayout::new("mobile");
        let tablet = SimpleLayout::new("tablet");
        let desktop = SimpleLayout::new("desktop");

        let adaptive = AdaptiveLayout::new(mobile, tablet, desktop);

        assert_eq!(adaptive.resolve(Breakpoint::Xs).name(), "mobile");
        assert_eq!(adaptive.resolve(Breakpoint::Sm).name(), "mobile");
        assert_eq!(adaptive.resolve(Breakpoint::Md).name(), "tablet");
        assert_eq!(adaptive.resolve(Breakpoint::Lg).name(), "tablet");
        assert_eq!(adaptive.resolve(Breakpoint::Xl).name(), "desktop");
        assert_eq!(adaptive.resolve(Breakpoint::Xxl).name(), "desktop");
    }

    #[test]
    fn test_adaptive_layout_with_context() {
        let mobile = SimpleLayout::new("mobile");
        let tablet = SimpleLayout::new("tablet");
        let desktop = SimpleLayout::new("desktop");

        let adaptive = AdaptiveLayout::new(mobile, tablet, desktop);

        let mobile_ctx = BreakpointContext::from_size(375.0, 667.0, 2.0);
        assert_eq!(adaptive.resolve_from_context(&mobile_ctx).name(), "mobile");

        let tablet_ctx = BreakpointContext::from_size(768.0, 1024.0, 2.0);
        assert_eq!(adaptive.resolve_from_context(&tablet_ctx).name(), "tablet");

        let desktop_ctx = BreakpointContext::from_size(1920.0, 1080.0, 1.0);
        assert_eq!(
            adaptive.resolve_from_context(&desktop_ctx).name(),
            "desktop"
        );
    }

    #[test]
    fn test_split_view_config() {
        let config = SplitViewConfig::new()
            .with_primary_width(SplitViewWidth::Fixed(300.0))
            .with_min_secondary_width(500.0)
            .with_collapse_breakpoint(Breakpoint::Md);

        assert_eq!(config.calculate_primary_width(1000.0), 300.0);
        assert!(!config.should_collapse(Breakpoint::Lg));
        assert!(config.should_collapse(Breakpoint::Sm));
    }

    #[test]
    fn test_split_view_width_percent() {
        let config = SplitViewConfig::new().with_primary_width(SplitViewWidth::Percent(0.3));

        assert_eq!(config.calculate_primary_width(1000.0), 300.0);
        assert_eq!(config.calculate_primary_width(600.0), 180.0);
    }

    #[test]
    fn test_split_view_width_constrained() {
        let config =
            SplitViewConfig::new().with_primary_width(SplitViewWidth::Constrained { min: 250.0, max: 400.0 });

        // 1/3 of 900 = 300, within range
        assert_eq!(config.calculate_primary_width(900.0), 300.0);

        // 1/3 of 600 = 200, below min, should be 250
        assert_eq!(config.calculate_primary_width(600.0), 250.0);

        // 1/3 of 1500 = 500, above max, should be 400
        assert_eq!(config.calculate_primary_width(1500.0), 400.0);
    }

    #[test]
    fn test_sidebar_config() {
        let config = SidebarConfig::new()
            .with_expanded_width(280.0)
            .with_collapsed_width(72.0)
            .with_auto_collapse(Some(Breakpoint::Xl));

        assert_eq!(config.width(false), 280.0);
        assert_eq!(config.width(true), 72.0);
        assert!(config.should_auto_collapse(Breakpoint::Lg));
        assert!(!config.should_auto_collapse(Breakpoint::Xl));
    }

    #[test]
    fn test_bottom_nav_config() {
        let config = BottomNavConfig::new()
            .with_height(64.0)
            .with_max_items(4)
            .with_labels(false)
            .with_hide_on_scroll(true);

        assert_eq!(config.height, 64.0);
        assert_eq!(config.max_items, 4);
        assert!(!config.show_labels);
        assert!(config.hide_on_scroll);
    }
}
