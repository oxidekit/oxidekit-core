//! Responsive layout primitives for mobile platforms.
//!
//! This module provides tools for building adaptive layouts that respond
//! to different screen sizes, safe areas, and screen densities.
//!
//! ## Modules
//!
//! - [`breakpoint`]: Screen size breakpoints for responsive layouts
//! - [`safe_area`]: Safe area insets for notches and system UI
//! - [`density`]: Screen density handling for proper scaling

pub mod breakpoint;
pub mod safe_area;
pub mod density;

pub use breakpoint::{Breakpoint, BreakpointConfig, BreakpointRange};
pub use safe_area::{SafeAreaInsets, SafeAreaEdge, SafeAreaProvider};
pub use density::{ScreenDensity, DensityBucket, DensityScaler};

/// Calculate the current breakpoint for a given screen width.
///
/// Uses the default breakpoint configuration.
///
/// # Example
///
/// ```rust
/// use oxide_mobile::responsive::current_breakpoint;
///
/// let bp = current_breakpoint(375.0);
/// // Returns Breakpoint::Sm for typical iPhone width
/// ```
pub fn current_breakpoint(width: f32) -> Breakpoint {
    BreakpointConfig::default().get(width)
}

/// Convert pixels to density-independent pixels.
///
/// # Arguments
///
/// * `px` - Pixel value
/// * `density` - Screen density multiplier (e.g., 2.0 for @2x, 3.0 for @3x)
///
/// # Example
///
/// ```rust
/// use oxide_mobile::responsive::px_to_dp;
///
/// let dp = px_to_dp(100.0, 2.0);
/// assert_eq!(dp, 50.0);
/// ```
pub fn px_to_dp(px: f32, density: f32) -> f32 {
    px / density
}

/// Convert density-independent pixels to pixels.
///
/// # Arguments
///
/// * `dp` - Density-independent pixel value
/// * `density` - Screen density multiplier (e.g., 2.0 for @2x, 3.0 for @3x)
///
/// # Example
///
/// ```rust
/// use oxide_mobile::responsive::dp_to_px;
///
/// let px = dp_to_px(50.0, 2.0);
/// assert_eq!(px, 100.0);
/// ```
pub fn dp_to_px(dp: f32, density: f32) -> f32 {
    dp * density
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_px_dp_conversion() {
        assert_eq!(px_to_dp(100.0, 2.0), 50.0);
        assert_eq!(dp_to_px(50.0, 2.0), 100.0);

        // Round trip
        let original = 75.0;
        let density = 3.0;
        let converted = dp_to_px(px_to_dp(original, density), density);
        assert!((converted - original).abs() < 0.001);
    }

    #[test]
    fn test_current_breakpoint() {
        assert_eq!(current_breakpoint(300.0), Breakpoint::Xs);
        assert_eq!(current_breakpoint(640.0), Breakpoint::Sm);
    }
}
