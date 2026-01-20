//! Responsive UI System
//!
//! A comprehensive responsive design system for OxideKit components that provides:
//!
//! - **Breakpoints**: Named viewport size categories (mobile, tablet, desktop, wide)
//! - **Media Queries**: Flexible conditions for viewport characteristics
//! - **Responsive Values**: Values that adapt to viewport size
//! - **Container Queries**: Size-based queries relative to container elements
//! - **Aspect Ratio Utilities**: Common aspect ratios and calculations
//! - **Safe Area Insets**: Mobile device insets for notches and home indicators
//!
//! # Architecture
//!
//! This module is designed to work with OxideKit's layout system while providing
//! a higher-level API for component-based responsive design. It builds upon the
//! primitives in `oxide-layout` to offer a more ergonomic interface.
//!
//! # Example
//!
//! ```
//! use oxide_components::responsive::*;
//!
//! // Define responsive padding that increases with screen size
//! let padding = ResponsiveValue::new(8.0)
//!     .mobile(12.0)
//!     .tablet(16.0)
//!     .desktop(24.0)
//!     .wide(32.0);
//!
//! // Create a viewport context (900px is tablet range: 768-1023)
//! let viewport = ViewportContext::new(900.0, 600.0, 2.0);
//!
//! // Resolve the value for the current viewport
//! let current_padding = viewport.resolve(&padding);
//! assert_eq!(*current_padding, 16.0); // tablet breakpoint
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// Breakpoints
// ============================================================================

/// Semantic breakpoint categories for responsive design.
///
/// These breakpoints are designed around common device form factors:
/// - `Mobile`: Phones in portrait or landscape (< 768px)
/// - `Tablet`: Tablets and small laptops (768px - 1023px)
/// - `Desktop`: Standard desktop displays (1024px - 1439px)
/// - `Wide`: Large displays and ultrawide monitors (>= 1440px)
///
/// The breakpoint values follow industry conventions and align with common
/// device widths for optimal responsive behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Breakpoint {
    /// Mobile devices (< 768px)
    Mobile,
    /// Tablet devices (>= 768px, < 1024px)
    Tablet,
    /// Desktop displays (>= 1024px, < 1440px)
    Desktop,
    /// Wide/large displays (>= 1440px)
    Wide,
}

impl Breakpoint {
    /// Get the minimum width in logical pixels for this breakpoint.
    pub const fn min_width(&self) -> f32 {
        match self {
            Breakpoint::Mobile => 0.0,
            Breakpoint::Tablet => 768.0,
            Breakpoint::Desktop => 1024.0,
            Breakpoint::Wide => 1440.0,
        }
    }

    /// Get the maximum width in logical pixels for this breakpoint (exclusive).
    ///
    /// Returns `None` for the `Wide` breakpoint as it has no upper limit.
    pub const fn max_width(&self) -> Option<f32> {
        match self {
            Breakpoint::Mobile => Some(768.0),
            Breakpoint::Tablet => Some(1024.0),
            Breakpoint::Desktop => Some(1440.0),
            Breakpoint::Wide => None,
        }
    }

    /// Determine the breakpoint from a viewport width.
    pub fn from_width(width: f32) -> Self {
        if width >= 1440.0 {
            Breakpoint::Wide
        } else if width >= 1024.0 {
            Breakpoint::Desktop
        } else if width >= 768.0 {
            Breakpoint::Tablet
        } else {
            Breakpoint::Mobile
        }
    }

    /// Get all breakpoints in order from smallest to largest.
    pub const fn all() -> [Breakpoint; 4] {
        [
            Breakpoint::Mobile,
            Breakpoint::Tablet,
            Breakpoint::Desktop,
            Breakpoint::Wide,
        ]
    }

    /// Check if this breakpoint is at least the given breakpoint.
    pub fn is_at_least(&self, other: Breakpoint) -> bool {
        *self >= other
    }

    /// Check if this breakpoint is at most the given breakpoint.
    pub fn is_at_most(&self, other: Breakpoint) -> bool {
        *self <= other
    }

    /// Check if this breakpoint is between two breakpoints (inclusive).
    pub fn is_between(&self, min: Breakpoint, max: Breakpoint) -> bool {
        *self >= min && *self <= max
    }

    /// Get a human-readable name for this breakpoint.
    pub const fn name(&self) -> &'static str {
        match self {
            Breakpoint::Mobile => "mobile",
            Breakpoint::Tablet => "tablet",
            Breakpoint::Desktop => "desktop",
            Breakpoint::Wide => "wide",
        }
    }

    /// Get the typical device category for this breakpoint.
    pub const fn device_category(&self) -> DeviceCategory {
        match self {
            Breakpoint::Mobile => DeviceCategory::Phone,
            Breakpoint::Tablet => DeviceCategory::Tablet,
            Breakpoint::Desktop => DeviceCategory::Desktop,
            Breakpoint::Wide => DeviceCategory::LargeDesktop,
        }
    }
}

impl Default for Breakpoint {
    fn default() -> Self {
        Breakpoint::Desktop
    }
}

impl fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Device category for more semantic device detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceCategory {
    /// Mobile phones
    Phone,
    /// Tablets and phablets
    Tablet,
    /// Standard desktop/laptop displays
    Desktop,
    /// Large monitors, ultrawide displays
    LargeDesktop,
}

impl DeviceCategory {
    /// Check if this is a touch-primary device category.
    pub const fn is_touch_primary(&self) -> bool {
        matches!(self, DeviceCategory::Phone | DeviceCategory::Tablet)
    }

    /// Check if this is a pointer-primary device category.
    pub const fn is_pointer_primary(&self) -> bool {
        matches!(self, DeviceCategory::Desktop | DeviceCategory::LargeDesktop)
    }
}

// ============================================================================
// Orientation
// ============================================================================

/// Screen orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    /// Portrait orientation (height > width)
    Portrait,
    /// Landscape orientation (width >= height)
    Landscape,
}

impl Orientation {
    /// Determine orientation from dimensions.
    pub fn from_dimensions(width: f32, height: f32) -> Self {
        if height > width {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }

    /// Check if this is portrait orientation.
    pub const fn is_portrait(&self) -> bool {
        matches!(self, Orientation::Portrait)
    }

    /// Check if this is landscape orientation.
    pub const fn is_landscape(&self) -> bool {
        matches!(self, Orientation::Landscape)
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Landscape
    }
}

// ============================================================================
// Color Scheme
// ============================================================================

/// Preferred color scheme for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    /// Light color scheme
    #[default]
    Light,
    /// Dark color scheme
    Dark,
}

impl ColorScheme {
    /// Check if this is light mode.
    pub const fn is_light(&self) -> bool {
        matches!(self, ColorScheme::Light)
    }

    /// Check if this is dark mode.
    pub const fn is_dark(&self) -> bool {
        matches!(self, ColorScheme::Dark)
    }

    /// Toggle the color scheme.
    pub const fn toggle(&self) -> Self {
        match self {
            ColorScheme::Light => ColorScheme::Dark,
            ColorScheme::Dark => ColorScheme::Light,
        }
    }
}

// ============================================================================
// Media Query
// ============================================================================

/// A media query condition for responsive styling.
///
/// Media queries allow you to define conditions based on viewport characteristics.
/// Multiple conditions can be combined to create complex queries.
///
/// # Example
///
/// ```
/// use oxide_components::responsive::{MediaQuery, Breakpoint, Orientation};
///
/// // Query for tablet in landscape
/// let query = MediaQuery::new()
///     .min_width(768.0)
///     .max_width(1024.0)
///     .orientation(Orientation::Landscape);
///
/// // Check if viewport matches
/// assert!(query.matches_viewport(900.0, 600.0, 1.0));
/// assert!(!query.matches_viewport(900.0, 1200.0, 1.0)); // portrait
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaQuery {
    /// Minimum viewport width (inclusive)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f32>,

    /// Maximum viewport width (exclusive)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,

    /// Minimum viewport height (inclusive)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f32>,

    /// Maximum viewport height (exclusive)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f32>,

    /// Required orientation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,

    /// Minimum device pixel ratio
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_resolution: Option<f32>,

    /// Maximum device pixel ratio
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_resolution: Option<f32>,

    /// Required color scheme
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<ColorScheme>,

    /// Required aspect ratio (width / height)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<AspectRatioCondition>,

    /// Minimum aspect ratio
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_aspect_ratio: Option<f32>,

    /// Maximum aspect ratio
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_aspect_ratio: Option<f32>,

    /// Whether reduced motion is preferred
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_motion: Option<bool>,

    /// Whether high contrast is preferred
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefers_contrast: Option<ContrastPreference>,
}

/// Aspect ratio condition for media queries.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AspectRatioCondition {
    /// Width component of the ratio
    pub width: u32,
    /// Height component of the ratio
    pub height: u32,
}

impl AspectRatioCondition {
    /// Create a new aspect ratio condition.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Get the aspect ratio as a floating point value.
    pub fn ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Check if a given aspect ratio matches within tolerance.
    pub fn matches(&self, ratio: f32, tolerance: f32) -> bool {
        (self.ratio() - ratio).abs() <= tolerance
    }
}

/// Contrast preference for accessibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContrastPreference {
    /// No preference
    NoPreference,
    /// Prefers more contrast
    More,
    /// Prefers less contrast
    Less,
    /// Custom contrast level
    Custom,
}

impl MediaQuery {
    /// Create a new empty media query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a media query for a specific breakpoint.
    pub fn breakpoint(bp: Breakpoint) -> Self {
        let mut query = Self::new();
        query.min_width = Some(bp.min_width());
        query.max_width = bp.max_width();
        query
    }

    /// Create a media query for mobile breakpoint and above.
    pub fn mobile_up() -> Self {
        Self::new()
    }

    /// Create a media query for tablet breakpoint and above.
    pub fn tablet_up() -> Self {
        Self::new().min_width(Breakpoint::Tablet.min_width())
    }

    /// Create a media query for desktop breakpoint and above.
    pub fn desktop_up() -> Self {
        Self::new().min_width(Breakpoint::Desktop.min_width())
    }

    /// Create a media query for wide breakpoint and above.
    pub fn wide_up() -> Self {
        Self::new().min_width(Breakpoint::Wide.min_width())
    }

    /// Create a media query for mobile only.
    pub fn mobile_only() -> Self {
        Self::breakpoint(Breakpoint::Mobile)
    }

    /// Create a media query for tablet only.
    pub fn tablet_only() -> Self {
        Self::breakpoint(Breakpoint::Tablet)
    }

    /// Create a media query for desktop only.
    pub fn desktop_only() -> Self {
        Self::breakpoint(Breakpoint::Desktop)
    }

    /// Create a media query for portrait orientation.
    pub fn portrait() -> Self {
        Self::new().orientation(Orientation::Portrait)
    }

    /// Create a media query for landscape orientation.
    pub fn landscape() -> Self {
        Self::new().orientation(Orientation::Landscape)
    }

    /// Create a media query for dark mode.
    pub fn dark_mode() -> Self {
        Self::new().color_scheme(ColorScheme::Dark)
    }

    /// Create a media query for light mode.
    pub fn light_mode() -> Self {
        Self::new().color_scheme(ColorScheme::Light)
    }

    /// Create a media query for high-DPI (retina) displays.
    pub fn retina() -> Self {
        Self::new().min_resolution(2.0)
    }

    /// Set minimum width condition.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set maximum width condition.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set minimum height condition.
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Set maximum height condition.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set orientation condition.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Set minimum resolution (device pixel ratio) condition.
    pub fn min_resolution(mut self, dpr: f32) -> Self {
        self.min_resolution = Some(dpr);
        self
    }

    /// Set maximum resolution (device pixel ratio) condition.
    pub fn max_resolution(mut self, dpr: f32) -> Self {
        self.max_resolution = Some(dpr);
        self
    }

    /// Set color scheme condition.
    pub fn color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.color_scheme = Some(scheme);
        self
    }

    /// Set aspect ratio condition.
    pub fn aspect_ratio(mut self, width: u32, height: u32) -> Self {
        self.aspect_ratio = Some(AspectRatioCondition::new(width, height));
        self
    }

    /// Set minimum aspect ratio condition.
    pub fn min_aspect_ratio(mut self, ratio: f32) -> Self {
        self.min_aspect_ratio = Some(ratio);
        self
    }

    /// Set maximum aspect ratio condition.
    pub fn max_aspect_ratio(mut self, ratio: f32) -> Self {
        self.max_aspect_ratio = Some(ratio);
        self
    }

    /// Set reduced motion preference condition.
    pub fn prefers_reduced_motion(mut self, reduced: bool) -> Self {
        self.prefers_reduced_motion = Some(reduced);
        self
    }

    /// Set contrast preference condition.
    pub fn prefers_contrast(mut self, preference: ContrastPreference) -> Self {
        self.prefers_contrast = Some(preference);
        self
    }

    /// Check if the query matches the given viewport characteristics.
    pub fn matches_viewport(&self, width: f32, height: f32, scale_factor: f32) -> bool {
        // Check width conditions
        if let Some(min_w) = self.min_width {
            if width < min_w {
                return false;
            }
        }
        if let Some(max_w) = self.max_width {
            if width >= max_w {
                return false;
            }
        }

        // Check height conditions
        if let Some(min_h) = self.min_height {
            if height < min_h {
                return false;
            }
        }
        if let Some(max_h) = self.max_height {
            if height >= max_h {
                return false;
            }
        }

        // Check orientation
        if let Some(required_orientation) = self.orientation {
            let current_orientation = Orientation::from_dimensions(width, height);
            if current_orientation != required_orientation {
                return false;
            }
        }

        // Check resolution
        if let Some(min_res) = self.min_resolution {
            if scale_factor < min_res {
                return false;
            }
        }
        if let Some(max_res) = self.max_resolution {
            if scale_factor >= max_res {
                return false;
            }
        }

        // Check aspect ratio
        let current_ratio = width / height;
        if let Some(ar) = &self.aspect_ratio {
            if !ar.matches(current_ratio, 0.01) {
                return false;
            }
        }
        if let Some(min_ar) = self.min_aspect_ratio {
            if current_ratio < min_ar {
                return false;
            }
        }
        if let Some(max_ar) = self.max_aspect_ratio {
            if current_ratio > max_ar {
                return false;
            }
        }

        true
    }

    /// Check if the query matches the given viewport context.
    pub fn matches(&self, context: &ViewportContext) -> bool {
        // Check basic viewport conditions
        if !self.matches_viewport(context.width, context.height, context.scale_factor) {
            return false;
        }

        // Check color scheme
        if let Some(required_scheme) = self.color_scheme {
            if context.color_scheme != required_scheme {
                return false;
            }
        }

        // Check reduced motion preference
        if let Some(required_reduced) = self.prefers_reduced_motion {
            if context.prefers_reduced_motion != required_reduced {
                return false;
            }
        }

        // Check contrast preference
        if let Some(required_contrast) = self.prefers_contrast {
            if context.prefers_contrast != required_contrast {
                return false;
            }
        }

        true
    }

    /// Check if this query has any conditions set.
    pub fn is_empty(&self) -> bool {
        self.min_width.is_none()
            && self.max_width.is_none()
            && self.min_height.is_none()
            && self.max_height.is_none()
            && self.orientation.is_none()
            && self.min_resolution.is_none()
            && self.max_resolution.is_none()
            && self.color_scheme.is_none()
            && self.aspect_ratio.is_none()
            && self.min_aspect_ratio.is_none()
            && self.max_aspect_ratio.is_none()
            && self.prefers_reduced_motion.is_none()
            && self.prefers_contrast.is_none()
    }
}

// ============================================================================
// Responsive Value
// ============================================================================

/// A value that changes based on viewport breakpoint.
///
/// `ResponsiveValue` provides a mobile-first approach where values cascade
/// up from smaller to larger breakpoints. If a breakpoint doesn't have an
/// explicit value, it inherits from the next smaller breakpoint.
///
/// # Example
///
/// ```
/// use oxide_components::responsive::{ResponsiveValue, Breakpoint};
///
/// // Font size that increases with screen size
/// let font_size = ResponsiveValue::new(14.0)
///     .tablet(16.0)
///     .desktop(18.0);
///
/// // Mobile inherits the base value (14.0)
/// assert_eq!(*font_size.get(Breakpoint::Mobile), 14.0);
/// // Tablet uses its explicit value (16.0)
/// assert_eq!(*font_size.get(Breakpoint::Tablet), 16.0);
/// // Desktop uses its explicit value (18.0)
/// assert_eq!(*font_size.get(Breakpoint::Desktop), 18.0);
/// // Wide inherits from desktop (18.0)
/// assert_eq!(*font_size.get(Breakpoint::Wide), 18.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveValue<T> {
    /// Base value (mobile-first default)
    base: T,
    /// Override values for specific breakpoints
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    overrides: HashMap<Breakpoint, T>,
}

impl<T: Clone> ResponsiveValue<T> {
    /// Create a new responsive value with a base value.
    ///
    /// The base value is used for all breakpoints unless overridden.
    pub fn new(base: T) -> Self {
        Self {
            base,
            overrides: HashMap::new(),
        }
    }

    /// Create a responsive value from explicit values for each breakpoint.
    pub fn explicit(mobile: T, tablet: T, desktop: T, wide: T) -> Self
    where
        T: Clone,
    {
        let mut value = Self::new(mobile);
        value.overrides.insert(Breakpoint::Tablet, tablet);
        value.overrides.insert(Breakpoint::Desktop, desktop);
        value.overrides.insert(Breakpoint::Wide, wide);
        value
    }

    /// Set the value for mobile breakpoint.
    pub fn mobile(mut self, value: T) -> Self {
        self.overrides.insert(Breakpoint::Mobile, value);
        self
    }

    /// Set the value for tablet breakpoint.
    pub fn tablet(mut self, value: T) -> Self {
        self.overrides.insert(Breakpoint::Tablet, value);
        self
    }

    /// Set the value for desktop breakpoint.
    pub fn desktop(mut self, value: T) -> Self {
        self.overrides.insert(Breakpoint::Desktop, value);
        self
    }

    /// Set the value for wide breakpoint.
    pub fn wide(mut self, value: T) -> Self {
        self.overrides.insert(Breakpoint::Wide, value);
        self
    }

    /// Set the value for a specific breakpoint.
    pub fn at(mut self, breakpoint: Breakpoint, value: T) -> Self {
        self.overrides.insert(breakpoint, value);
        self
    }

    /// Get the value for a specific breakpoint.
    ///
    /// Returns the value for the specified breakpoint, or cascades down
    /// to find the nearest smaller breakpoint with a value.
    pub fn get(&self, breakpoint: Breakpoint) -> &T {
        // Check breakpoints from current down to Mobile
        let all = Breakpoint::all();
        let current_idx = all.iter().position(|&b| b == breakpoint).unwrap_or(0);

        for i in (0..=current_idx).rev() {
            if let Some(value) = self.overrides.get(&all[i]) {
                return value;
            }
        }

        &self.base
    }

    /// Get a mutable reference to the value for a specific breakpoint.
    pub fn get_mut(&mut self, breakpoint: Breakpoint) -> &mut T {
        self.overrides.entry(breakpoint).or_insert_with(|| self.base.clone())
    }

    /// Check if a specific breakpoint has an explicit override.
    pub fn has_override(&self, breakpoint: Breakpoint) -> bool {
        self.overrides.contains_key(&breakpoint)
    }

    /// Get the base value.
    pub fn base(&self) -> &T {
        &self.base
    }

    /// Set the base value.
    pub fn set_base(&mut self, value: T) {
        self.base = value;
    }

    /// Map the value using a transformation function.
    pub fn map<U, F>(&self, f: F) -> ResponsiveValue<U>
    where
        F: Fn(&T) -> U,
        U: Clone,
    {
        ResponsiveValue {
            base: f(&self.base),
            overrides: self.overrides.iter().map(|(k, v)| (*k, f(v))).collect(),
        }
    }

    /// Combine two responsive values using a function.
    pub fn zip_with<U, V, F>(&self, other: &ResponsiveValue<U>, f: F) -> ResponsiveValue<V>
    where
        U: Clone,
        V: Clone,
        F: Fn(&T, &U) -> V,
    {
        let base = f(&self.base, &other.base);
        let mut overrides = HashMap::new();

        for bp in Breakpoint::all() {
            let self_val = self.get(bp);
            let other_val = other.get(bp);
            overrides.insert(bp, f(self_val, other_val));
        }

        ResponsiveValue { base, overrides }
    }
}

impl<T: Clone + Default> Default for ResponsiveValue<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone + PartialEq> PartialEq for ResponsiveValue<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.base != other.base {
            return false;
        }
        for bp in Breakpoint::all() {
            if self.get(bp) != other.get(bp) {
                return false;
            }
        }
        true
    }
}

/// Convenience trait for creating responsive values from static values.
pub trait IntoResponsive<T> {
    fn into_responsive(self) -> ResponsiveValue<T>;
}

impl<T: Clone> IntoResponsive<T> for T {
    fn into_responsive(self) -> ResponsiveValue<T> {
        ResponsiveValue::new(self)
    }
}

impl<T: Clone> IntoResponsive<T> for ResponsiveValue<T> {
    fn into_responsive(self) -> ResponsiveValue<T> {
        self
    }
}

// ============================================================================
// Container Queries
// ============================================================================

/// Container size information for container queries.
///
/// Container queries allow responsive styling based on the size of a parent
/// container rather than the viewport. This is useful for components that
/// may appear in different contexts (e.g., sidebar vs main content).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ContainerSize {
    /// Container width in logical pixels
    pub width: f32,
    /// Container height in logical pixels
    pub height: f32,
}

impl ContainerSize {
    /// Create a new container size.
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a container size from just width (height unknown).
    pub const fn from_width(width: f32) -> Self {
        Self { width, height: 0.0 }
    }

    /// Create a container size from just height (width unknown).
    pub const fn from_height(height: f32) -> Self {
        Self { width: 0.0, height }
    }

    /// Get the aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        if self.height > 0.0 {
            self.width / self.height
        } else {
            0.0
        }
    }

    /// Get the orientation of the container.
    pub fn orientation(&self) -> Orientation {
        Orientation::from_dimensions(self.width, self.height)
    }

    /// Check if the container is larger than the given dimensions.
    pub fn is_larger_than(&self, width: f32, height: f32) -> bool {
        self.width > width && self.height > height
    }

    /// Check if the container is smaller than the given dimensions.
    pub fn is_smaller_than(&self, width: f32, height: f32) -> bool {
        self.width < width && self.height < height
    }
}

/// A container query for responsive container-based styling.
///
/// Similar to CSS container queries, this allows defining conditions
/// based on parent container size rather than viewport size.
///
/// # Example
///
/// ```
/// use oxide_components::responsive::{ContainerQuery, ContainerSize};
///
/// let query = ContainerQuery::new()
///     .min_width(400.0)
///     .max_width(800.0);
///
/// let container = ContainerSize::new(500.0, 300.0);
/// assert!(query.matches(&container));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContainerQuery {
    /// Minimum container width
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f32>,

    /// Maximum container width
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,

    /// Minimum container height
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f32>,

    /// Maximum container height
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f32>,

    /// Required orientation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,

    /// Minimum aspect ratio
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_aspect_ratio: Option<f32>,

    /// Maximum aspect ratio
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_aspect_ratio: Option<f32>,
}

impl ContainerQuery {
    /// Create a new empty container query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum width condition.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set maximum width condition.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set minimum height condition.
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Set maximum height condition.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set orientation condition.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Set minimum aspect ratio condition.
    pub fn min_aspect_ratio(mut self, ratio: f32) -> Self {
        self.min_aspect_ratio = Some(ratio);
        self
    }

    /// Set maximum aspect ratio condition.
    pub fn max_aspect_ratio(mut self, ratio: f32) -> Self {
        self.max_aspect_ratio = Some(ratio);
        self
    }

    /// Check if the query matches the given container size.
    pub fn matches(&self, size: &ContainerSize) -> bool {
        // Check width conditions
        if let Some(min_w) = self.min_width {
            if size.width < min_w {
                return false;
            }
        }
        if let Some(max_w) = self.max_width {
            if size.width >= max_w {
                return false;
            }
        }

        // Check height conditions
        if let Some(min_h) = self.min_height {
            if size.height < min_h {
                return false;
            }
        }
        if let Some(max_h) = self.max_height {
            if size.height >= max_h {
                return false;
            }
        }

        // Check orientation
        if let Some(required_orientation) = self.orientation {
            if size.orientation() != required_orientation {
                return false;
            }
        }

        // Check aspect ratio
        let ratio = size.aspect_ratio();
        if let Some(min_ar) = self.min_aspect_ratio {
            if ratio < min_ar {
                return false;
            }
        }
        if let Some(max_ar) = self.max_aspect_ratio {
            if ratio > max_ar {
                return false;
            }
        }

        true
    }

    /// Check if this query has any conditions set.
    pub fn is_empty(&self) -> bool {
        self.min_width.is_none()
            && self.max_width.is_none()
            && self.min_height.is_none()
            && self.max_height.is_none()
            && self.orientation.is_none()
            && self.min_aspect_ratio.is_none()
            && self.max_aspect_ratio.is_none()
    }
}

/// Container breakpoint categories for container queries.
///
/// These are smaller than viewport breakpoints since containers
/// are typically smaller than the full viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerBreakpoint {
    /// Extra small container (< 300px)
    Xs,
    /// Small container (>= 300px, < 500px)
    Sm,
    /// Medium container (>= 500px, < 700px)
    Md,
    /// Large container (>= 700px, < 900px)
    Lg,
    /// Extra large container (>= 900px)
    Xl,
}

impl ContainerBreakpoint {
    /// Get the minimum width for this container breakpoint.
    pub const fn min_width(&self) -> f32 {
        match self {
            ContainerBreakpoint::Xs => 0.0,
            ContainerBreakpoint::Sm => 300.0,
            ContainerBreakpoint::Md => 500.0,
            ContainerBreakpoint::Lg => 700.0,
            ContainerBreakpoint::Xl => 900.0,
        }
    }

    /// Determine the container breakpoint from a width.
    pub fn from_width(width: f32) -> Self {
        if width >= 900.0 {
            ContainerBreakpoint::Xl
        } else if width >= 700.0 {
            ContainerBreakpoint::Lg
        } else if width >= 500.0 {
            ContainerBreakpoint::Md
        } else if width >= 300.0 {
            ContainerBreakpoint::Sm
        } else {
            ContainerBreakpoint::Xs
        }
    }

    /// Get all container breakpoints in order.
    pub const fn all() -> [ContainerBreakpoint; 5] {
        [
            ContainerBreakpoint::Xs,
            ContainerBreakpoint::Sm,
            ContainerBreakpoint::Md,
            ContainerBreakpoint::Lg,
            ContainerBreakpoint::Xl,
        ]
    }
}

impl Default for ContainerBreakpoint {
    fn default() -> Self {
        ContainerBreakpoint::Md
    }
}

/// Container-responsive value that changes based on container size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResponsiveValue<T> {
    base: T,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    overrides: HashMap<ContainerBreakpoint, T>,
}

impl<T: Clone> ContainerResponsiveValue<T> {
    /// Create a new container-responsive value.
    pub fn new(base: T) -> Self {
        Self {
            base,
            overrides: HashMap::new(),
        }
    }

    /// Set the value for a container breakpoint.
    pub fn at(mut self, breakpoint: ContainerBreakpoint, value: T) -> Self {
        self.overrides.insert(breakpoint, value);
        self
    }

    /// Get the value for a container size.
    pub fn get(&self, size: &ContainerSize) -> &T {
        let bp = ContainerBreakpoint::from_width(size.width);
        self.get_for_breakpoint(bp)
    }

    /// Get the value for a specific container breakpoint.
    pub fn get_for_breakpoint(&self, breakpoint: ContainerBreakpoint) -> &T {
        let all = ContainerBreakpoint::all();
        let current_idx = all.iter().position(|&b| b == breakpoint).unwrap_or(0);

        for i in (0..=current_idx).rev() {
            if let Some(value) = self.overrides.get(&all[i]) {
                return value;
            }
        }

        &self.base
    }
}

// ============================================================================
// Aspect Ratio Utilities
// ============================================================================

/// Common aspect ratios for UI elements.
///
/// Provides predefined aspect ratios commonly used in UI design,
/// along with utilities for calculating dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AspectRatio {
    /// Width component of the ratio
    pub width: f32,
    /// Height component of the ratio
    pub height: f32,
}

impl AspectRatio {
    /// Create a custom aspect ratio.
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Square aspect ratio (1:1).
    pub const SQUARE: Self = Self::new(1.0, 1.0);

    /// Standard video aspect ratio (16:9).
    pub const WIDESCREEN: Self = Self::new(16.0, 9.0);

    /// Classic video aspect ratio (4:3).
    pub const STANDARD: Self = Self::new(4.0, 3.0);

    /// Portrait photo aspect ratio (3:4).
    pub const PORTRAIT: Self = Self::new(3.0, 4.0);

    /// Landscape photo aspect ratio (4:3).
    pub const LANDSCAPE: Self = Self::new(4.0, 3.0);

    /// Ultrawide aspect ratio (21:9).
    pub const ULTRAWIDE: Self = Self::new(21.0, 9.0);

    /// Cinema aspect ratio (2.35:1).
    pub const CINEMA: Self = Self::new(2.35, 1.0);

    /// Social media story aspect ratio (9:16).
    pub const STORY: Self = Self::new(9.0, 16.0);

    /// Instagram post aspect ratio (1:1).
    pub const INSTAGRAM_POST: Self = Self::SQUARE;

    /// Instagram story aspect ratio (9:16).
    pub const INSTAGRAM_STORY: Self = Self::STORY;

    /// Twitter card aspect ratio (16:9).
    pub const TWITTER_CARD: Self = Self::WIDESCREEN;

    /// LinkedIn cover aspect ratio (4:1).
    pub const LINKEDIN_COVER: Self = Self::new(4.0, 1.0);

    /// Golden ratio (1.618:1).
    pub const GOLDEN: Self = Self::new(1.618, 1.0);

    /// A4 paper aspect ratio (1:1.414).
    pub const A4: Self = Self::new(1.0, 1.414);

    /// Get the aspect ratio as a floating point value (width / height).
    pub fn ratio(&self) -> f32 {
        self.width / self.height
    }

    /// Get the inverse aspect ratio (height / width).
    pub fn inverse_ratio(&self) -> f32 {
        self.height / self.width
    }

    /// Calculate height for a given width.
    pub fn height_for_width(&self, width: f32) -> f32 {
        width / self.ratio()
    }

    /// Calculate width for a given height.
    pub fn width_for_height(&self, height: f32) -> f32 {
        height * self.ratio()
    }

    /// Get dimensions that fit within a container while maintaining aspect ratio.
    ///
    /// Returns (width, height) that fit within the container.
    pub fn fit_within(&self, container_width: f32, container_height: f32) -> (f32, f32) {
        let container_ratio = container_width / container_height;
        let content_ratio = self.ratio();

        if content_ratio > container_ratio {
            // Width-constrained
            (container_width, container_width / content_ratio)
        } else {
            // Height-constrained
            (container_height * content_ratio, container_height)
        }
    }

    /// Get dimensions that cover a container while maintaining aspect ratio.
    ///
    /// Returns (width, height) that cover the container.
    pub fn cover(&self, container_width: f32, container_height: f32) -> (f32, f32) {
        let container_ratio = container_width / container_height;
        let content_ratio = self.ratio();

        if content_ratio > container_ratio {
            // Height-constrained
            (container_height * content_ratio, container_height)
        } else {
            // Width-constrained
            (container_width, container_width / content_ratio)
        }
    }

    /// Check if two aspect ratios are approximately equal.
    pub fn approx_eq(&self, other: &Self, tolerance: f32) -> bool {
        (self.ratio() - other.ratio()).abs() <= tolerance
    }

    /// Get the simplified integer ratio (e.g., 16:9 instead of 1920:1080).
    pub fn simplified(&self) -> (u32, u32) {
        fn gcd(a: u32, b: u32) -> u32 {
            if b == 0 { a } else { gcd(b, a % b) }
        }

        let w = (self.width * 1000.0) as u32;
        let h = (self.height * 1000.0) as u32;
        let divisor = gcd(w, h);

        (w / divisor, h / divisor)
    }

    /// Get a common name for this aspect ratio, if known.
    pub fn common_name(&self) -> Option<&'static str> {
        let ratio = self.ratio();
        let tolerance = 0.01;

        if (ratio - 1.0).abs() < tolerance {
            Some("1:1 (Square)")
        } else if (ratio - 16.0 / 9.0).abs() < tolerance {
            Some("16:9 (Widescreen)")
        } else if (ratio - 4.0 / 3.0).abs() < tolerance {
            Some("4:3 (Standard)")
        } else if (ratio - 9.0 / 16.0).abs() < tolerance {
            Some("9:16 (Story)")
        } else if (ratio - 21.0 / 9.0).abs() < tolerance {
            Some("21:9 (Ultrawide)")
        } else if (ratio - 1.618).abs() < tolerance {
            Some("Golden Ratio")
        } else {
            None
        }
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::SQUARE
    }
}

impl From<(f32, f32)> for AspectRatio {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new(width, height)
    }
}

impl From<f32> for AspectRatio {
    fn from(ratio: f32) -> Self {
        Self::new(ratio, 1.0)
    }
}

// ============================================================================
// Safe Area Insets
// ============================================================================

/// Safe area insets for mobile device displays.
///
/// Handles device-specific UI elements like notches, home indicators,
/// status bars, and camera cutouts.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SafeAreaInsets {
    /// Top inset (for notches, status bar)
    pub top: f32,
    /// Bottom inset (for home indicator, navigation bar)
    pub bottom: f32,
    /// Left inset (for display curves, cutouts)
    pub left: f32,
    /// Right inset (for display curves, cutouts)
    pub right: f32,
}

impl SafeAreaInsets {
    /// Create new safe area insets.
    pub const fn new(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self { top, bottom, left, right }
    }

    /// Create zero insets.
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Create uniform insets on all sides.
    pub const fn uniform(inset: f32) -> Self {
        Self::new(inset, inset, inset, inset)
    }

    /// Create vertical insets only (top and bottom).
    pub const fn vertical(top: f32, bottom: f32) -> Self {
        Self::new(top, bottom, 0.0, 0.0)
    }

    /// Create horizontal insets only (left and right).
    pub const fn horizontal(left: f32, right: f32) -> Self {
        Self::new(0.0, 0.0, left, right)
    }

    /// Check if all insets are zero.
    pub fn is_zero(&self) -> bool {
        self.top == 0.0 && self.bottom == 0.0 && self.left == 0.0 && self.right == 0.0
    }

    /// Get total horizontal inset (left + right).
    pub fn horizontal_total(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical inset (top + bottom).
    pub fn vertical_total(&self) -> f32 {
        self.top + self.bottom
    }

    /// Apply a scale factor to all insets.
    pub fn scaled(&self, factor: f32) -> Self {
        Self {
            top: self.top * factor,
            bottom: self.bottom * factor,
            left: self.left * factor,
            right: self.right * factor,
        }
    }

    /// Combine with other insets, taking the maximum of each edge.
    pub fn max(&self, other: &Self) -> Self {
        Self {
            top: self.top.max(other.top),
            bottom: self.bottom.max(other.bottom),
            left: self.left.max(other.left),
            right: self.right.max(other.right),
        }
    }

    /// Add padding to the insets.
    pub fn with_padding(&self, padding: f32) -> Self {
        Self {
            top: self.top + padding,
            bottom: self.bottom + padding,
            left: self.left + padding,
            right: self.right + padding,
        }
    }

    /// Preset for iPhone with notch (X, 11, 12, 13, 14 series).
    pub const fn iphone_notch() -> Self {
        Self::new(47.0, 34.0, 0.0, 0.0)
    }

    /// Preset for iPhone with Dynamic Island (14 Pro, 15 series).
    pub const fn iphone_dynamic_island() -> Self {
        Self::new(59.0, 34.0, 0.0, 0.0)
    }

    /// Preset for classic iPhone (SE, 8 and earlier).
    pub const fn iphone_classic() -> Self {
        Self::new(20.0, 0.0, 0.0, 0.0)
    }

    /// Preset for Android with standard status bar.
    pub const fn android_standard() -> Self {
        Self::new(24.0, 0.0, 0.0, 0.0)
    }

    /// Preset for Android with gesture navigation.
    pub const fn android_gesture_nav() -> Self {
        Self::new(24.0, 20.0, 0.0, 0.0)
    }

    /// Preset for iPad.
    pub const fn ipad() -> Self {
        Self::new(24.0, 20.0, 0.0, 0.0)
    }
}

/// Edge flags for safe area application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SafeAreaEdges {
    bits: u8,
}

impl SafeAreaEdges {
    /// No edges.
    pub const NONE: Self = Self { bits: 0 };
    /// Top edge only.
    pub const TOP: Self = Self { bits: 0b0001 };
    /// Bottom edge only.
    pub const BOTTOM: Self = Self { bits: 0b0010 };
    /// Left edge only.
    pub const LEFT: Self = Self { bits: 0b0100 };
    /// Right edge only.
    pub const RIGHT: Self = Self { bits: 0b1000 };
    /// Vertical edges (top and bottom).
    pub const VERTICAL: Self = Self { bits: 0b0011 };
    /// Horizontal edges (left and right).
    pub const HORIZONTAL: Self = Self { bits: 0b1100 };
    /// All edges.
    pub const ALL: Self = Self { bits: 0b1111 };

    /// Create from individual edge flags.
    pub const fn new(top: bool, bottom: bool, left: bool, right: bool) -> Self {
        let mut bits = 0;
        if top { bits |= 0b0001; }
        if bottom { bits |= 0b0010; }
        if left { bits |= 0b0100; }
        if right { bits |= 0b1000; }
        Self { bits }
    }

    /// Check if the top edge is included.
    pub const fn has_top(&self) -> bool {
        self.bits & 0b0001 != 0
    }

    /// Check if the bottom edge is included.
    pub const fn has_bottom(&self) -> bool {
        self.bits & 0b0010 != 0
    }

    /// Check if the left edge is included.
    pub const fn has_left(&self) -> bool {
        self.bits & 0b0100 != 0
    }

    /// Check if the right edge is included.
    pub const fn has_right(&self) -> bool {
        self.bits & 0b1000 != 0
    }

    /// Combine with another set of edges.
    pub const fn union(self, other: Self) -> Self {
        Self { bits: self.bits | other.bits }
    }

    /// Get the intersection with another set of edges.
    pub const fn intersection(self, other: Self) -> Self {
        Self { bits: self.bits & other.bits }
    }

    /// Filter insets to only the specified edges.
    pub fn apply_to(&self, insets: &SafeAreaInsets) -> SafeAreaInsets {
        SafeAreaInsets {
            top: if self.has_top() { insets.top } else { 0.0 },
            bottom: if self.has_bottom() { insets.bottom } else { 0.0 },
            left: if self.has_left() { insets.left } else { 0.0 },
            right: if self.has_right() { insets.right } else { 0.0 },
        }
    }
}

// ============================================================================
// Viewport Context
// ============================================================================

/// Complete viewport context for responsive decisions.
///
/// Contains all information needed to make responsive styling decisions,
/// including size, orientation, device characteristics, and user preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportContext {
    /// Viewport width in logical pixels.
    pub width: f32,

    /// Viewport height in logical pixels.
    pub height: f32,

    /// Device pixel ratio (scale factor).
    pub scale_factor: f32,

    /// Current breakpoint based on width.
    #[serde(skip)]
    breakpoint: Breakpoint,

    /// Current orientation.
    #[serde(skip)]
    orientation: Orientation,

    /// Safe area insets for the device.
    #[serde(default)]
    pub safe_area: SafeAreaInsets,

    /// Preferred color scheme.
    #[serde(default)]
    pub color_scheme: ColorScheme,

    /// Whether reduced motion is preferred.
    #[serde(default)]
    pub prefers_reduced_motion: bool,

    /// Contrast preference.
    #[serde(default)]
    pub prefers_contrast: ContrastPreference,
}

impl ViewportContext {
    /// Create a new viewport context.
    pub fn new(width: f32, height: f32, scale_factor: f32) -> Self {
        let breakpoint = Breakpoint::from_width(width);
        let orientation = Orientation::from_dimensions(width, height);

        Self {
            width,
            height,
            scale_factor,
            breakpoint,
            orientation,
            safe_area: SafeAreaInsets::zero(),
            color_scheme: ColorScheme::Light,
            prefers_reduced_motion: false,
            prefers_contrast: ContrastPreference::NoPreference,
        }
    }

    /// Create a viewport context with safe area insets.
    pub fn with_safe_area(mut self, insets: SafeAreaInsets) -> Self {
        self.safe_area = insets;
        self
    }

    /// Create a viewport context with color scheme.
    pub fn with_color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.color_scheme = scheme;
        self
    }

    /// Create a viewport context with reduced motion preference.
    pub fn with_reduced_motion(mut self, reduced: bool) -> Self {
        self.prefers_reduced_motion = reduced;
        self
    }

    /// Create a viewport context with contrast preference.
    pub fn with_contrast(mut self, preference: ContrastPreference) -> Self {
        self.prefers_contrast = preference;
        self
    }

    /// Update the viewport dimensions.
    pub fn update(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.breakpoint = Breakpoint::from_width(width);
        self.orientation = Orientation::from_dimensions(width, height);
    }

    /// Update the scale factor.
    pub fn update_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    /// Get the current breakpoint.
    pub fn breakpoint(&self) -> Breakpoint {
        self.breakpoint
    }

    /// Get the current orientation.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Get the physical width in device pixels.
    pub fn physical_width(&self) -> f32 {
        self.width * self.scale_factor
    }

    /// Get the physical height in device pixels.
    pub fn physical_height(&self) -> f32 {
        self.height * self.scale_factor
    }

    /// Get the aspect ratio of the viewport.
    pub fn aspect_ratio(&self) -> f32 {
        self.width / self.height
    }

    /// Check if the current device is mobile.
    pub fn is_mobile(&self) -> bool {
        self.breakpoint == Breakpoint::Mobile
    }

    /// Check if the current device is tablet.
    pub fn is_tablet(&self) -> bool {
        self.breakpoint == Breakpoint::Tablet
    }

    /// Check if the current device is desktop.
    pub fn is_desktop(&self) -> bool {
        self.breakpoint == Breakpoint::Desktop
    }

    /// Check if the current device is wide.
    pub fn is_wide(&self) -> bool {
        self.breakpoint == Breakpoint::Wide
    }

    /// Check if the viewport is in portrait orientation.
    pub fn is_portrait(&self) -> bool {
        self.orientation.is_portrait()
    }

    /// Check if the viewport is in landscape orientation.
    pub fn is_landscape(&self) -> bool {
        self.orientation.is_landscape()
    }

    /// Check if the device is high-DPI (retina).
    pub fn is_retina(&self) -> bool {
        self.scale_factor >= 2.0
    }

    /// Resolve a responsive value for the current breakpoint.
    pub fn resolve<'a, T: Clone>(&self, value: &'a ResponsiveValue<T>) -> &'a T {
        value.get(self.breakpoint)
    }

    /// Check if a media query matches the current context.
    pub fn matches(&self, query: &MediaQuery) -> bool {
        query.matches(self)
    }

    /// Get the usable content area after safe area insets.
    pub fn content_area(&self) -> (f32, f32, f32, f32) {
        let x = self.safe_area.left;
        let y = self.safe_area.top;
        let width = self.width - self.safe_area.horizontal_total();
        let height = self.height - self.safe_area.vertical_total();
        (x, y, width, height)
    }

    /// Get device category based on breakpoint.
    pub fn device_category(&self) -> DeviceCategory {
        self.breakpoint.device_category()
    }

    /// Check if touch is likely the primary input method.
    pub fn is_touch_device(&self) -> bool {
        self.device_category().is_touch_primary()
    }
}

impl Default for ViewportContext {
    fn default() -> Self {
        // Default to a standard desktop size
        Self::new(1280.0, 720.0, 1.0)
    }
}

impl Default for ContrastPreference {
    fn default() -> Self {
        ContrastPreference::NoPreference
    }
}

// ============================================================================
// Responsive Style Builder
// ============================================================================

/// Builder for creating responsive styles.
///
/// Provides a fluent API for building styles that vary across breakpoints.
///
/// # Example
///
/// ```
/// use oxide_components::responsive::{ResponsiveStyleBuilder, Breakpoint};
///
/// let styles = ResponsiveStyleBuilder::new()
///     .base("padding", "8px")
///     .tablet("padding", "16px")
///     .desktop("padding", "24px")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct ResponsiveStyleBuilder {
    styles: HashMap<String, ResponsiveValue<String>>,
}

impl ResponsiveStyleBuilder {
    /// Create a new responsive style builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a base value for a style property.
    pub fn base(mut self, property: &str, value: impl Into<String>) -> Self {
        let responsive = self
            .styles
            .entry(property.to_string())
            .or_insert_with(|| ResponsiveValue::new(String::new()));
        responsive.set_base(value.into());
        self
    }

    /// Set a value for mobile breakpoint.
    pub fn mobile(mut self, property: &str, value: impl Into<String>) -> Self {
        let responsive = self
            .styles
            .entry(property.to_string())
            .or_insert_with(|| ResponsiveValue::new(String::new()));
        responsive.overrides.insert(Breakpoint::Mobile, value.into());
        self
    }

    /// Set a value for tablet breakpoint.
    pub fn tablet(mut self, property: &str, value: impl Into<String>) -> Self {
        let responsive = self
            .styles
            .entry(property.to_string())
            .or_insert_with(|| ResponsiveValue::new(String::new()));
        responsive.overrides.insert(Breakpoint::Tablet, value.into());
        self
    }

    /// Set a value for desktop breakpoint.
    pub fn desktop(mut self, property: &str, value: impl Into<String>) -> Self {
        let responsive = self
            .styles
            .entry(property.to_string())
            .or_insert_with(|| ResponsiveValue::new(String::new()));
        responsive.overrides.insert(Breakpoint::Desktop, value.into());
        self
    }

    /// Set a value for wide breakpoint.
    pub fn wide(mut self, property: &str, value: impl Into<String>) -> Self {
        let responsive = self
            .styles
            .entry(property.to_string())
            .or_insert_with(|| ResponsiveValue::new(String::new()));
        responsive.overrides.insert(Breakpoint::Wide, value.into());
        self
    }

    /// Build the responsive styles.
    pub fn build(self) -> HashMap<String, ResponsiveValue<String>> {
        self.styles
    }

    /// Get a specific style property.
    pub fn get(&self, property: &str) -> Option<&ResponsiveValue<String>> {
        self.styles.get(property)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Breakpoint tests
    mod breakpoint_tests {
        use super::*;

        #[test]
        fn test_breakpoint_from_width() {
            assert_eq!(Breakpoint::from_width(320.0), Breakpoint::Mobile);
            assert_eq!(Breakpoint::from_width(767.0), Breakpoint::Mobile);
            assert_eq!(Breakpoint::from_width(768.0), Breakpoint::Tablet);
            assert_eq!(Breakpoint::from_width(1023.0), Breakpoint::Tablet);
            assert_eq!(Breakpoint::from_width(1024.0), Breakpoint::Desktop);
            assert_eq!(Breakpoint::from_width(1439.0), Breakpoint::Desktop);
            assert_eq!(Breakpoint::from_width(1440.0), Breakpoint::Wide);
            assert_eq!(Breakpoint::from_width(2560.0), Breakpoint::Wide);
        }

        #[test]
        fn test_breakpoint_ordering() {
            assert!(Breakpoint::Mobile < Breakpoint::Tablet);
            assert!(Breakpoint::Tablet < Breakpoint::Desktop);
            assert!(Breakpoint::Desktop < Breakpoint::Wide);
        }

        #[test]
        fn test_breakpoint_min_width() {
            assert_eq!(Breakpoint::Mobile.min_width(), 0.0);
            assert_eq!(Breakpoint::Tablet.min_width(), 768.0);
            assert_eq!(Breakpoint::Desktop.min_width(), 1024.0);
            assert_eq!(Breakpoint::Wide.min_width(), 1440.0);
        }

        #[test]
        fn test_breakpoint_max_width() {
            assert_eq!(Breakpoint::Mobile.max_width(), Some(768.0));
            assert_eq!(Breakpoint::Tablet.max_width(), Some(1024.0));
            assert_eq!(Breakpoint::Desktop.max_width(), Some(1440.0));
            assert_eq!(Breakpoint::Wide.max_width(), None);
        }

        #[test]
        fn test_breakpoint_is_between() {
            assert!(Breakpoint::Tablet.is_between(Breakpoint::Mobile, Breakpoint::Desktop));
            assert!(!Breakpoint::Wide.is_between(Breakpoint::Mobile, Breakpoint::Desktop));
        }
    }

    // MediaQuery tests
    mod media_query_tests {
        use super::*;

        #[test]
        fn test_media_query_width() {
            let query = MediaQuery::new()
                .min_width(768.0)
                .max_width(1024.0);

            assert!(!query.matches_viewport(600.0, 400.0, 1.0));
            assert!(query.matches_viewport(800.0, 600.0, 1.0));
            assert!(!query.matches_viewport(1100.0, 700.0, 1.0));
        }

        #[test]
        fn test_media_query_orientation() {
            let portrait_query = MediaQuery::portrait();
            let landscape_query = MediaQuery::landscape();

            assert!(portrait_query.matches_viewport(400.0, 800.0, 1.0));
            assert!(!portrait_query.matches_viewport(800.0, 400.0, 1.0));
            assert!(landscape_query.matches_viewport(800.0, 400.0, 1.0));
            assert!(!landscape_query.matches_viewport(400.0, 800.0, 1.0));
        }

        #[test]
        fn test_media_query_resolution() {
            let retina_query = MediaQuery::retina();

            assert!(!retina_query.matches_viewport(800.0, 600.0, 1.0));
            assert!(retina_query.matches_viewport(800.0, 600.0, 2.0));
            assert!(retina_query.matches_viewport(800.0, 600.0, 3.0));
        }

        #[test]
        fn test_media_query_combined() {
            let query = MediaQuery::new()
                .min_width(768.0)
                .max_width(1024.0)
                .orientation(Orientation::Landscape);

            assert!(!query.matches_viewport(800.0, 1000.0, 1.0)); // portrait
            assert!(query.matches_viewport(900.0, 600.0, 1.0)); // landscape, in range
            assert!(!query.matches_viewport(600.0, 400.0, 1.0)); // too narrow
        }

        #[test]
        fn test_media_query_aspect_ratio() {
            let query = MediaQuery::new()
                .min_aspect_ratio(1.5)
                .max_aspect_ratio(2.0);

            assert!(!query.matches_viewport(400.0, 400.0, 1.0)); // 1:1
            assert!(query.matches_viewport(800.0, 450.0, 1.0)); // 16:9 = 1.78
            assert!(!query.matches_viewport(800.0, 200.0, 1.0)); // 4:1
        }

        #[test]
        fn test_media_query_presets() {
            let mobile = MediaQuery::mobile_only();
            let tablet_up = MediaQuery::tablet_up();
            let desktop_up = MediaQuery::desktop_up();

            assert!(mobile.matches_viewport(500.0, 800.0, 1.0));
            assert!(!mobile.matches_viewport(800.0, 600.0, 1.0));

            assert!(!tablet_up.matches_viewport(500.0, 800.0, 1.0));
            assert!(tablet_up.matches_viewport(800.0, 600.0, 1.0));
            assert!(tablet_up.matches_viewport(1200.0, 800.0, 1.0));

            assert!(!desktop_up.matches_viewport(800.0, 600.0, 1.0));
            assert!(desktop_up.matches_viewport(1200.0, 800.0, 1.0));
        }

        #[test]
        fn test_media_query_with_context() {
            let context = ViewportContext::new(1024.0, 768.0, 2.0)
                .with_color_scheme(ColorScheme::Dark);

            let dark_desktop = MediaQuery::desktop_up().color_scheme(ColorScheme::Dark);
            let light_query = MediaQuery::light_mode();

            assert!(dark_desktop.matches(&context));
            assert!(!light_query.matches(&context));
        }
    }

    // ResponsiveValue tests
    mod responsive_value_tests {
        use super::*;

        #[test]
        fn test_responsive_value_basic() {
            let value = ResponsiveValue::new(14.0)
                .tablet(16.0)
                .desktop(18.0);

            assert_eq!(*value.get(Breakpoint::Mobile), 14.0);
            assert_eq!(*value.get(Breakpoint::Tablet), 16.0);
            assert_eq!(*value.get(Breakpoint::Desktop), 18.0);
            assert_eq!(*value.get(Breakpoint::Wide), 18.0); // inherits from desktop
        }

        #[test]
        fn test_responsive_value_cascade() {
            let value = ResponsiveValue::new(10.0).desktop(20.0);

            assert_eq!(*value.get(Breakpoint::Mobile), 10.0);
            assert_eq!(*value.get(Breakpoint::Tablet), 10.0);
            assert_eq!(*value.get(Breakpoint::Desktop), 20.0);
            assert_eq!(*value.get(Breakpoint::Wide), 20.0);
        }

        #[test]
        fn test_responsive_value_explicit() {
            let value = ResponsiveValue::explicit(10.0, 20.0, 30.0, 40.0);

            assert_eq!(*value.get(Breakpoint::Mobile), 10.0);
            assert_eq!(*value.get(Breakpoint::Tablet), 20.0);
            assert_eq!(*value.get(Breakpoint::Desktop), 30.0);
            assert_eq!(*value.get(Breakpoint::Wide), 40.0);
        }

        #[test]
        fn test_responsive_value_map() {
            let value = ResponsiveValue::new(10.0).tablet(20.0);
            let doubled = value.map(|v| v * 2.0);

            assert_eq!(*doubled.get(Breakpoint::Mobile), 20.0);
            assert_eq!(*doubled.get(Breakpoint::Tablet), 40.0);
        }

        #[test]
        fn test_responsive_value_zip_with() {
            let a = ResponsiveValue::new(10.0).tablet(20.0);
            let b = ResponsiveValue::new(5.0).tablet(10.0);
            let sum = a.zip_with(&b, |x, y| x + y);

            assert_eq!(*sum.get(Breakpoint::Mobile), 15.0);
            assert_eq!(*sum.get(Breakpoint::Tablet), 30.0);
        }

        #[test]
        fn test_responsive_value_with_context() {
            let value = ResponsiveValue::new(8.0)
                .mobile(12.0)
                .tablet(16.0)
                .desktop(24.0);

            let mobile_ctx = ViewportContext::new(375.0, 667.0, 2.0);
            let tablet_ctx = ViewportContext::new(768.0, 1024.0, 2.0);
            let desktop_ctx = ViewportContext::new(1280.0, 800.0, 1.0);

            assert_eq!(*mobile_ctx.resolve(&value), 12.0);
            assert_eq!(*tablet_ctx.resolve(&value), 16.0);
            assert_eq!(*desktop_ctx.resolve(&value), 24.0);
        }
    }

    // Container query tests
    mod container_query_tests {
        use super::*;

        #[test]
        fn test_container_size() {
            let size = ContainerSize::new(800.0, 600.0);

            assert_eq!(size.orientation(), Orientation::Landscape);
            assert!((size.aspect_ratio() - 1.333).abs() < 0.01);
            assert!(size.is_larger_than(400.0, 300.0));
            assert!(size.is_smaller_than(1000.0, 800.0));
        }

        #[test]
        fn test_container_query_basic() {
            let query = ContainerQuery::new()
                .min_width(400.0)
                .max_width(800.0);

            assert!(!query.matches(&ContainerSize::new(300.0, 200.0)));
            assert!(query.matches(&ContainerSize::new(500.0, 300.0)));
            assert!(!query.matches(&ContainerSize::new(900.0, 500.0)));
        }

        #[test]
        fn test_container_breakpoint() {
            assert_eq!(ContainerBreakpoint::from_width(200.0), ContainerBreakpoint::Xs);
            assert_eq!(ContainerBreakpoint::from_width(400.0), ContainerBreakpoint::Sm);
            assert_eq!(ContainerBreakpoint::from_width(600.0), ContainerBreakpoint::Md);
            assert_eq!(ContainerBreakpoint::from_width(800.0), ContainerBreakpoint::Lg);
            assert_eq!(ContainerBreakpoint::from_width(1000.0), ContainerBreakpoint::Xl);
        }

        #[test]
        fn test_container_responsive_value() {
            let value = ContainerResponsiveValue::new(8.0)
                .at(ContainerBreakpoint::Md, 16.0)
                .at(ContainerBreakpoint::Lg, 24.0);

            let small = ContainerSize::new(200.0, 100.0);
            let medium = ContainerSize::new(600.0, 300.0);
            let large = ContainerSize::new(800.0, 400.0);

            assert_eq!(*value.get(&small), 8.0);
            assert_eq!(*value.get(&medium), 16.0);
            assert_eq!(*value.get(&large), 24.0);
        }
    }

    // Aspect ratio tests
    mod aspect_ratio_tests {
        use super::*;

        #[test]
        fn test_aspect_ratio_basic() {
            let ratio = AspectRatio::WIDESCREEN;
            assert!((ratio.ratio() - 1.777).abs() < 0.01);
        }

        #[test]
        fn test_aspect_ratio_calculations() {
            let ratio = AspectRatio::WIDESCREEN;

            assert!((ratio.height_for_width(1920.0) - 1080.0).abs() < 1.0);
            assert!((ratio.width_for_height(1080.0) - 1920.0).abs() < 1.0);
        }

        #[test]
        fn test_aspect_ratio_fit_within() {
            let ratio = AspectRatio::WIDESCREEN;

            // Container is wider than content ratio
            let (w, h) = ratio.fit_within(1920.0, 800.0);
            assert!((w / h - ratio.ratio()).abs() < 0.01);
            assert!(h <= 800.0);

            // Container is taller than content ratio
            let (w, h) = ratio.fit_within(800.0, 1080.0);
            assert!((w / h - ratio.ratio()).abs() < 0.01);
            assert!(w <= 800.0);
        }

        #[test]
        fn test_aspect_ratio_cover() {
            let ratio = AspectRatio::WIDESCREEN;

            let (w, h) = ratio.cover(800.0, 600.0);
            assert!(w >= 800.0 || h >= 600.0);
            assert!((w / h - ratio.ratio()).abs() < 0.01);
        }

        #[test]
        fn test_aspect_ratio_common_name() {
            assert_eq!(AspectRatio::SQUARE.common_name(), Some("1:1 (Square)"));
            assert_eq!(AspectRatio::WIDESCREEN.common_name(), Some("16:9 (Widescreen)"));
            assert_eq!(AspectRatio::STANDARD.common_name(), Some("4:3 (Standard)"));
        }
    }

    // Safe area tests
    mod safe_area_tests {
        use super::*;

        #[test]
        fn test_safe_area_insets_basic() {
            let insets = SafeAreaInsets::new(47.0, 34.0, 0.0, 0.0);

            assert_eq!(insets.vertical_total(), 81.0);
            assert_eq!(insets.horizontal_total(), 0.0);
            assert!(!insets.is_zero());
        }

        #[test]
        fn test_safe_area_insets_uniform() {
            let insets = SafeAreaInsets::uniform(10.0);

            assert_eq!(insets.top, 10.0);
            assert_eq!(insets.bottom, 10.0);
            assert_eq!(insets.left, 10.0);
            assert_eq!(insets.right, 10.0);
        }

        #[test]
        fn test_safe_area_insets_scaled() {
            let insets = SafeAreaInsets::new(10.0, 20.0, 5.0, 15.0);
            let scaled = insets.scaled(2.0);

            assert_eq!(scaled.top, 20.0);
            assert_eq!(scaled.bottom, 40.0);
            assert_eq!(scaled.left, 10.0);
            assert_eq!(scaled.right, 30.0);
        }

        #[test]
        fn test_safe_area_insets_max() {
            let a = SafeAreaInsets::new(10.0, 20.0, 5.0, 15.0);
            let b = SafeAreaInsets::new(15.0, 10.0, 10.0, 5.0);
            let max = a.max(&b);

            assert_eq!(max.top, 15.0);
            assert_eq!(max.bottom, 20.0);
            assert_eq!(max.left, 10.0);
            assert_eq!(max.right, 15.0);
        }

        #[test]
        fn test_safe_area_edges() {
            let insets = SafeAreaInsets::new(47.0, 34.0, 10.0, 10.0);

            let vertical = SafeAreaEdges::VERTICAL.apply_to(&insets);
            assert_eq!(vertical.top, 47.0);
            assert_eq!(vertical.bottom, 34.0);
            assert_eq!(vertical.left, 0.0);
            assert_eq!(vertical.right, 0.0);

            let top_only = SafeAreaEdges::TOP.apply_to(&insets);
            assert_eq!(top_only.top, 47.0);
            assert_eq!(top_only.bottom, 0.0);
        }

        #[test]
        fn test_safe_area_presets() {
            let notch = SafeAreaInsets::iphone_notch();
            assert_eq!(notch.top, 47.0);
            assert_eq!(notch.bottom, 34.0);

            let island = SafeAreaInsets::iphone_dynamic_island();
            assert_eq!(island.top, 59.0);

            let classic = SafeAreaInsets::iphone_classic();
            assert_eq!(classic.top, 20.0);
            assert_eq!(classic.bottom, 0.0);
        }
    }

    // ViewportContext tests
    mod viewport_context_tests {
        use super::*;

        #[test]
        fn test_viewport_context_basic() {
            let ctx = ViewportContext::new(1024.0, 768.0, 2.0);

            assert_eq!(ctx.breakpoint(), Breakpoint::Desktop);
            assert_eq!(ctx.orientation(), Orientation::Landscape);
            assert_eq!(ctx.physical_width(), 2048.0);
            assert_eq!(ctx.physical_height(), 1536.0);
        }

        #[test]
        fn test_viewport_context_device_checks() {
            let mobile = ViewportContext::new(375.0, 667.0, 2.0);
            assert!(mobile.is_mobile());
            assert!(!mobile.is_tablet());
            assert!(mobile.is_portrait());
            assert!(mobile.is_touch_device());

            let tablet = ViewportContext::new(1024.0, 768.0, 2.0);
            assert!(!tablet.is_mobile());
            assert!(tablet.is_desktop()); // 1024 is desktop threshold
            assert!(tablet.is_landscape());

            let desktop = ViewportContext::new(1920.0, 1080.0, 1.0);
            assert!(desktop.is_wide());
            assert!(!desktop.is_touch_device());
        }

        #[test]
        fn test_viewport_context_update() {
            let mut ctx = ViewportContext::new(375.0, 667.0, 2.0);
            assert!(ctx.is_mobile());

            ctx.update(1024.0, 768.0);
            assert!(ctx.is_desktop());
            assert!(ctx.is_landscape());
        }

        #[test]
        fn test_viewport_context_content_area() {
            let ctx = ViewportContext::new(1024.0, 768.0, 1.0)
                .with_safe_area(SafeAreaInsets::new(47.0, 34.0, 0.0, 0.0));

            let (x, y, w, h) = ctx.content_area();

            assert_eq!(x, 0.0);
            assert_eq!(y, 47.0);
            assert_eq!(w, 1024.0);
            assert_eq!(h, 687.0);
        }

        #[test]
        fn test_viewport_context_matches_query() {
            let ctx = ViewportContext::new(1024.0, 768.0, 2.0)
                .with_color_scheme(ColorScheme::Dark);

            let desktop_dark = MediaQuery::desktop_up().color_scheme(ColorScheme::Dark);
            let mobile_query = MediaQuery::mobile_only();

            assert!(ctx.matches(&desktop_dark));
            assert!(!ctx.matches(&mobile_query));
        }
    }

    // ResponsiveStyleBuilder tests
    mod style_builder_tests {
        use super::*;

        #[test]
        fn test_responsive_style_builder() {
            let styles = ResponsiveStyleBuilder::new()
                .base("padding", "8px")
                .tablet("padding", "16px")
                .desktop("padding", "24px")
                .base("gap", "4px")
                .desktop("gap", "8px")
                .build();

            let padding = styles.get("padding").unwrap();
            assert_eq!(padding.get(Breakpoint::Mobile), "8px");
            assert_eq!(padding.get(Breakpoint::Tablet), "16px");
            assert_eq!(padding.get(Breakpoint::Desktop), "24px");

            let gap = styles.get("gap").unwrap();
            assert_eq!(gap.get(Breakpoint::Mobile), "4px");
            assert_eq!(gap.get(Breakpoint::Desktop), "8px");
        }
    }

    // Serialization tests
    mod serialization_tests {
        use super::*;

        #[test]
        fn test_breakpoint_serialization() {
            let bp = Breakpoint::Desktop;
            let json = serde_json::to_string(&bp).unwrap();
            assert_eq!(json, "\"desktop\"");

            let parsed: Breakpoint = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, bp);
        }

        #[test]
        fn test_responsive_value_serialization() {
            let value = ResponsiveValue::new(10.0).tablet(20.0);
            let json = serde_json::to_string(&value).unwrap();
            let parsed: ResponsiveValue<f64> = serde_json::from_str(&json).unwrap();

            assert_eq!(*parsed.base(), 10.0);
            assert_eq!(*parsed.get(Breakpoint::Tablet), 20.0);
        }

        #[test]
        fn test_media_query_serialization() {
            let query = MediaQuery::new()
                .min_width(768.0)
                .orientation(Orientation::Landscape);

            let json = serde_json::to_string(&query).unwrap();
            let parsed: MediaQuery = serde_json::from_str(&json).unwrap();

            assert_eq!(parsed.min_width, Some(768.0));
            assert_eq!(parsed.orientation, Some(Orientation::Landscape));
        }

        #[test]
        fn test_safe_area_serialization() {
            let insets = SafeAreaInsets::iphone_notch();
            let json = serde_json::to_string(&insets).unwrap();
            let parsed: SafeAreaInsets = serde_json::from_str(&json).unwrap();

            assert_eq!(parsed, insets);
        }
    }
}
