//! Screen size breakpoint system.
//!
//! Provides a responsive breakpoint system similar to CSS media queries,
//! allowing layouts to adapt to different screen sizes.
//!
//! ## Default Breakpoints
//!
//! | Breakpoint | Min Width | Typical Devices |
//! |------------|-----------|-----------------|
//! | Xs         | 0px       | Small phones    |
//! | Sm         | 640px     | Large phones    |
//! | Md         | 768px     | Small tablets   |
//! | Lg         | 1024px    | Large tablets   |
//! | Xl         | 1280px    | Large tablets (landscape) |

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Screen size breakpoint categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Breakpoint {
    /// Extra small screens (< 640px).
    Xs,
    /// Small screens (640px - 767px).
    Sm,
    /// Medium screens (768px - 1023px).
    Md,
    /// Large screens (1024px - 1279px).
    Lg,
    /// Extra large screens (>= 1280px).
    Xl,
}

impl Breakpoint {
    /// Get all breakpoints in order from smallest to largest.
    pub fn all() -> &'static [Breakpoint] {
        &[
            Breakpoint::Xs,
            Breakpoint::Sm,
            Breakpoint::Md,
            Breakpoint::Lg,
            Breakpoint::Xl,
        ]
    }

    /// Get the default minimum width for this breakpoint.
    pub fn default_min_width(&self) -> u32 {
        match self {
            Breakpoint::Xs => 0,
            Breakpoint::Sm => 640,
            Breakpoint::Md => 768,
            Breakpoint::Lg => 1024,
            Breakpoint::Xl => 1280,
        }
    }

    /// Get the next larger breakpoint, if any.
    pub fn next(&self) -> Option<Breakpoint> {
        match self {
            Breakpoint::Xs => Some(Breakpoint::Sm),
            Breakpoint::Sm => Some(Breakpoint::Md),
            Breakpoint::Md => Some(Breakpoint::Lg),
            Breakpoint::Lg => Some(Breakpoint::Xl),
            Breakpoint::Xl => None,
        }
    }

    /// Get the previous smaller breakpoint, if any.
    pub fn prev(&self) -> Option<Breakpoint> {
        match self {
            Breakpoint::Xs => None,
            Breakpoint::Sm => Some(Breakpoint::Xs),
            Breakpoint::Md => Some(Breakpoint::Sm),
            Breakpoint::Lg => Some(Breakpoint::Md),
            Breakpoint::Xl => Some(Breakpoint::Lg),
        }
    }

    /// Returns true if this breakpoint is considered "mobile" (Xs or Sm).
    pub fn is_mobile(&self) -> bool {
        matches!(self, Breakpoint::Xs | Breakpoint::Sm)
    }

    /// Returns true if this breakpoint is considered "tablet" (Md or Lg).
    pub fn is_tablet(&self) -> bool {
        matches!(self, Breakpoint::Md | Breakpoint::Lg)
    }

    /// Returns true if this breakpoint is considered "desktop" (Xl).
    pub fn is_desktop(&self) -> bool {
        matches!(self, Breakpoint::Xl)
    }
}

impl std::fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Breakpoint::Xs => "xs",
            Breakpoint::Sm => "sm",
            Breakpoint::Md => "md",
            Breakpoint::Lg => "lg",
            Breakpoint::Xl => "xl",
        };
        write!(f, "{}", name)
    }
}

/// Configuration for breakpoint thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointConfig {
    thresholds: HashMap<Breakpoint, u32>,
}

impl BreakpointConfig {
    /// Create a new breakpoint configuration with custom thresholds.
    ///
    /// Thresholds specify the minimum width (in pixels) for each breakpoint.
    pub fn new(thresholds: HashMap<Breakpoint, u32>) -> Self {
        Self { thresholds }
    }

    /// Get the threshold map.
    pub fn thresholds(&self) -> &HashMap<Breakpoint, u32> {
        &self.thresholds
    }

    /// Get the minimum width threshold for a breakpoint.
    pub fn threshold(&self, breakpoint: Breakpoint) -> u32 {
        self.thresholds
            .get(&breakpoint)
            .copied()
            .unwrap_or_else(|| breakpoint.default_min_width())
    }

    /// Determine the current breakpoint for a given screen width.
    pub fn get(&self, width: f32) -> Breakpoint {
        let width = width as u32;

        // Check from largest to smallest
        for bp in Breakpoint::all().iter().rev() {
            if width >= self.threshold(*bp) {
                return *bp;
            }
        }

        Breakpoint::Xs
    }

    /// Get the range of widths for a specific breakpoint.
    pub fn range(&self, breakpoint: Breakpoint) -> BreakpointRange {
        let min = self.threshold(breakpoint);
        let max = breakpoint
            .next()
            .map(|next| self.threshold(next).saturating_sub(1));

        BreakpointRange { min, max }
    }

    /// Check if a width falls within a specific breakpoint.
    pub fn matches(&self, breakpoint: Breakpoint, width: f32) -> bool {
        self.get(width) == breakpoint
    }

    /// Check if a width is at least a certain breakpoint.
    pub fn at_least(&self, breakpoint: Breakpoint, width: f32) -> bool {
        self.get(width) >= breakpoint
    }

    /// Check if a width is at most a certain breakpoint.
    pub fn at_most(&self, breakpoint: Breakpoint, width: f32) -> bool {
        self.get(width) <= breakpoint
    }

    /// Create a builder for custom configuration.
    pub fn builder() -> BreakpointConfigBuilder {
        BreakpointConfigBuilder::new()
    }
}

impl Default for BreakpointConfig {
    fn default() -> Self {
        let mut thresholds = HashMap::new();
        for bp in Breakpoint::all() {
            thresholds.insert(*bp, bp.default_min_width());
        }
        Self { thresholds }
    }
}

/// Range of widths for a breakpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakpointRange {
    /// Minimum width (inclusive).
    pub min: u32,
    /// Maximum width (inclusive), None if unbounded.
    pub max: Option<u32>,
}

impl BreakpointRange {
    /// Check if a width falls within this range.
    pub fn contains(&self, width: u32) -> bool {
        width >= self.min && self.max.map_or(true, |max| width <= max)
    }

    /// Get the span of this range (max - min), or None if unbounded.
    pub fn span(&self) -> Option<u32> {
        self.max.map(|max| max - self.min)
    }
}

impl std::fmt::Display for BreakpointRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.max {
            Some(max) => write!(f, "{}px - {}px", self.min, max),
            None => write!(f, "{}px+", self.min),
        }
    }
}

/// Builder for custom breakpoint configuration.
#[derive(Debug, Clone, Default)]
pub struct BreakpointConfigBuilder {
    thresholds: HashMap<Breakpoint, u32>,
}

impl BreakpointConfigBuilder {
    /// Create a new builder with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: HashMap::new(),
        }
    }

    /// Set the threshold for Xs breakpoint.
    pub fn xs(mut self, min_width: u32) -> Self {
        self.thresholds.insert(Breakpoint::Xs, min_width);
        self
    }

    /// Set the threshold for Sm breakpoint.
    pub fn sm(mut self, min_width: u32) -> Self {
        self.thresholds.insert(Breakpoint::Sm, min_width);
        self
    }

    /// Set the threshold for Md breakpoint.
    pub fn md(mut self, min_width: u32) -> Self {
        self.thresholds.insert(Breakpoint::Md, min_width);
        self
    }

    /// Set the threshold for Lg breakpoint.
    pub fn lg(mut self, min_width: u32) -> Self {
        self.thresholds.insert(Breakpoint::Lg, min_width);
        self
    }

    /// Set the threshold for Xl breakpoint.
    pub fn xl(mut self, min_width: u32) -> Self {
        self.thresholds.insert(Breakpoint::Xl, min_width);
        self
    }

    /// Build the breakpoint configuration.
    pub fn build(self) -> BreakpointConfig {
        let mut config = BreakpointConfig::default();
        for (bp, threshold) in self.thresholds {
            config.thresholds.insert(bp, threshold);
        }
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_ordering() {
        assert!(Breakpoint::Xs < Breakpoint::Sm);
        assert!(Breakpoint::Sm < Breakpoint::Md);
        assert!(Breakpoint::Md < Breakpoint::Lg);
        assert!(Breakpoint::Lg < Breakpoint::Xl);
    }

    #[test]
    fn test_breakpoint_navigation() {
        assert_eq!(Breakpoint::Xs.next(), Some(Breakpoint::Sm));
        assert_eq!(Breakpoint::Xl.next(), None);
        assert_eq!(Breakpoint::Xl.prev(), Some(Breakpoint::Lg));
        assert_eq!(Breakpoint::Xs.prev(), None);
    }

    #[test]
    fn test_breakpoint_classification() {
        assert!(Breakpoint::Xs.is_mobile());
        assert!(Breakpoint::Sm.is_mobile());
        assert!(!Breakpoint::Md.is_mobile());

        assert!(Breakpoint::Md.is_tablet());
        assert!(Breakpoint::Lg.is_tablet());
        assert!(!Breakpoint::Xl.is_tablet());

        assert!(Breakpoint::Xl.is_desktop());
        assert!(!Breakpoint::Lg.is_desktop());
    }

    #[test]
    fn test_breakpoint_config_get() {
        let config = BreakpointConfig::default();

        assert_eq!(config.get(0.0), Breakpoint::Xs);
        assert_eq!(config.get(320.0), Breakpoint::Xs);
        assert_eq!(config.get(639.0), Breakpoint::Xs);
        assert_eq!(config.get(640.0), Breakpoint::Sm);
        assert_eq!(config.get(768.0), Breakpoint::Md);
        assert_eq!(config.get(1024.0), Breakpoint::Lg);
        assert_eq!(config.get(1280.0), Breakpoint::Xl);
        assert_eq!(config.get(2000.0), Breakpoint::Xl);
    }

    #[test]
    fn test_breakpoint_config_custom() {
        let config = BreakpointConfig::builder()
            .sm(600)
            .md(900)
            .lg(1200)
            .xl(1800)
            .build();

        assert_eq!(config.get(599.0), Breakpoint::Xs);
        assert_eq!(config.get(600.0), Breakpoint::Sm);
        assert_eq!(config.get(900.0), Breakpoint::Md);
        assert_eq!(config.get(1200.0), Breakpoint::Lg);
        assert_eq!(config.get(1800.0), Breakpoint::Xl);
    }

    #[test]
    fn test_breakpoint_range() {
        let config = BreakpointConfig::default();

        let xs_range = config.range(Breakpoint::Xs);
        assert_eq!(xs_range.min, 0);
        assert_eq!(xs_range.max, Some(639));
        assert!(xs_range.contains(320));
        assert!(!xs_range.contains(640));

        let xl_range = config.range(Breakpoint::Xl);
        assert_eq!(xl_range.min, 1280);
        assert_eq!(xl_range.max, None);
        assert!(xl_range.contains(2000));
    }

    #[test]
    fn test_at_least_at_most() {
        let config = BreakpointConfig::default();

        // 800px should be Md
        assert!(config.at_least(Breakpoint::Xs, 800.0));
        assert!(config.at_least(Breakpoint::Sm, 800.0));
        assert!(config.at_least(Breakpoint::Md, 800.0));
        assert!(!config.at_least(Breakpoint::Lg, 800.0));

        assert!(!config.at_most(Breakpoint::Sm, 800.0));
        assert!(config.at_most(Breakpoint::Md, 800.0));
        assert!(config.at_most(Breakpoint::Lg, 800.0));
    }
}
