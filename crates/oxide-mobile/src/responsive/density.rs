//! Screen density handling.
//!
//! Provides tools for handling different screen densities and DPI values
//! across mobile devices.
//!
//! ## Density Buckets
//!
//! | Bucket  | Scale | Approx DPI | Common Devices |
//! |---------|-------|------------|----------------|
//! | LDPI    | 0.75  | ~120       | Legacy Android |
//! | MDPI    | 1.0   | ~160       | Baseline       |
//! | HDPI    | 1.5   | ~240       | Older phones   |
//! | XHDPI   | 2.0   | ~320       | @2x / iPhone   |
//! | XXHDPI  | 3.0   | ~480       | @3x / Modern   |
//! | XXXHDPI | 4.0   | ~640       | High-end       |

use serde::{Deserialize, Serialize};

/// Screen density information.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScreenDensity {
    /// Density scale factor (1.0 = mdpi baseline).
    scale: f32,
    /// Physical DPI of the screen.
    dpi: f32,
}

impl ScreenDensity {
    /// Create a new screen density from scale factor.
    pub fn from_scale(scale: f32) -> Self {
        Self {
            scale,
            dpi: scale * 160.0, // mdpi baseline is 160 dpi
        }
    }

    /// Create a new screen density from DPI.
    pub fn from_dpi(dpi: f32) -> Self {
        Self {
            scale: dpi / 160.0,
            dpi,
        }
    }

    /// Get the density scale factor.
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Get the physical DPI.
    pub fn dpi(&self) -> f32 {
        self.dpi
    }

    /// Get the density bucket for this screen.
    pub fn bucket(&self) -> DensityBucket {
        DensityBucket::from_scale(self.scale)
    }

    /// Convert density-independent pixels to physical pixels.
    pub fn dp_to_px(&self, dp: f32) -> f32 {
        dp * self.scale
    }

    /// Convert physical pixels to density-independent pixels.
    pub fn px_to_dp(&self, px: f32) -> f32 {
        px / self.scale
    }

    /// Convert scaled pixels (sp) to physical pixels.
    ///
    /// SP is like DP but also scales with user font size preference.
    pub fn sp_to_px(&self, sp: f32, font_scale: f32) -> f32 {
        sp * self.scale * font_scale
    }

    /// Get common iOS density scales.
    pub fn ios_1x() -> Self {
        Self::from_scale(1.0)
    }

    /// iOS @2x (Retina).
    pub fn ios_2x() -> Self {
        Self::from_scale(2.0)
    }

    /// iOS @3x (Super Retina).
    pub fn ios_3x() -> Self {
        Self::from_scale(3.0)
    }

    /// Typical modern iPhone density.
    pub fn iphone_modern() -> Self {
        Self::from_scale(3.0)
    }

    /// Typical iPad density.
    pub fn ipad() -> Self {
        Self::from_scale(2.0)
    }
}

impl Default for ScreenDensity {
    fn default() -> Self {
        Self::from_scale(2.0) // Reasonable default for modern devices
    }
}

/// Android density bucket classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DensityBucket {
    /// Low density (~120 dpi, 0.75x).
    Ldpi,
    /// Medium density (~160 dpi, 1x baseline).
    Mdpi,
    /// High density (~240 dpi, 1.5x).
    Hdpi,
    /// Extra-high density (~320 dpi, 2x).
    Xhdpi,
    /// Extra-extra-high density (~480 dpi, 3x).
    Xxhdpi,
    /// Extra-extra-extra-high density (~640 dpi, 4x).
    Xxxhdpi,
}

impl DensityBucket {
    /// Get the density bucket for a given scale factor.
    pub fn from_scale(scale: f32) -> Self {
        match scale {
            s if s < 0.875 => DensityBucket::Ldpi,
            s if s < 1.25 => DensityBucket::Mdpi,
            s if s < 1.75 => DensityBucket::Hdpi,
            s if s < 2.5 => DensityBucket::Xhdpi,
            s if s < 3.5 => DensityBucket::Xxhdpi,
            _ => DensityBucket::Xxxhdpi,
        }
    }

    /// Get the density bucket for a given DPI.
    pub fn from_dpi(dpi: f32) -> Self {
        Self::from_scale(dpi / 160.0)
    }

    /// Get the scale factor for this bucket.
    pub fn scale(&self) -> f32 {
        match self {
            DensityBucket::Ldpi => 0.75,
            DensityBucket::Mdpi => 1.0,
            DensityBucket::Hdpi => 1.5,
            DensityBucket::Xhdpi => 2.0,
            DensityBucket::Xxhdpi => 3.0,
            DensityBucket::Xxxhdpi => 4.0,
        }
    }

    /// Get the approximate DPI for this bucket.
    pub fn dpi(&self) -> f32 {
        match self {
            DensityBucket::Ldpi => 120.0,
            DensityBucket::Mdpi => 160.0,
            DensityBucket::Hdpi => 240.0,
            DensityBucket::Xhdpi => 320.0,
            DensityBucket::Xxhdpi => 480.0,
            DensityBucket::Xxxhdpi => 640.0,
        }
    }

    /// Get the Android resource qualifier string.
    pub fn android_qualifier(&self) -> &'static str {
        match self {
            DensityBucket::Ldpi => "ldpi",
            DensityBucket::Mdpi => "mdpi",
            DensityBucket::Hdpi => "hdpi",
            DensityBucket::Xhdpi => "xhdpi",
            DensityBucket::Xxhdpi => "xxhdpi",
            DensityBucket::Xxxhdpi => "xxxhdpi",
        }
    }

    /// Get the iOS scale suffix.
    pub fn ios_suffix(&self) -> &'static str {
        match self {
            DensityBucket::Ldpi | DensityBucket::Mdpi => "",
            DensityBucket::Hdpi | DensityBucket::Xhdpi => "@2x",
            _ => "@3x",
        }
    }

    /// Get all density buckets in order.
    pub fn all() -> &'static [DensityBucket] {
        &[
            DensityBucket::Ldpi,
            DensityBucket::Mdpi,
            DensityBucket::Hdpi,
            DensityBucket::Xhdpi,
            DensityBucket::Xxhdpi,
            DensityBucket::Xxxhdpi,
        ]
    }

    /// Get commonly used density buckets for modern apps.
    pub fn modern() -> &'static [DensityBucket] {
        &[
            DensityBucket::Hdpi,
            DensityBucket::Xhdpi,
            DensityBucket::Xxhdpi,
            DensityBucket::Xxxhdpi,
        ]
    }
}

impl std::fmt::Display for DensityBucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.android_qualifier())
    }
}

/// Helper for scaling values across density buckets.
#[derive(Debug, Clone)]
pub struct DensityScaler {
    /// Base density for calculations.
    base: ScreenDensity,
    /// Target density for scaling.
    target: ScreenDensity,
}

impl DensityScaler {
    /// Create a new density scaler.
    pub fn new(base: ScreenDensity, target: ScreenDensity) -> Self {
        Self { base, target }
    }

    /// Create a scaler from base to target scale factors.
    pub fn from_scales(base_scale: f32, target_scale: f32) -> Self {
        Self {
            base: ScreenDensity::from_scale(base_scale),
            target: ScreenDensity::from_scale(target_scale),
        }
    }

    /// Get the scaling ratio (target / base).
    pub fn ratio(&self) -> f32 {
        self.target.scale() / self.base.scale()
    }

    /// Scale a value from base to target density.
    pub fn scale(&self, value: f32) -> f32 {
        value * self.ratio()
    }

    /// Scale a value and round to nearest integer.
    pub fn scale_round(&self, value: f32) -> i32 {
        self.scale(value).round() as i32
    }

    /// Scale a value and round up.
    pub fn scale_ceil(&self, value: f32) -> i32 {
        self.scale(value).ceil() as i32
    }

    /// Scale a value and round down.
    pub fn scale_floor(&self, value: f32) -> i32 {
        self.scale(value).floor() as i32
    }
}

impl Default for DensityScaler {
    fn default() -> Self {
        Self::from_scales(1.0, 2.0) // mdpi to xhdpi
    }
}

/// Calculate icon sizes for different density buckets.
pub fn icon_sizes(base_size: u32) -> Vec<(DensityBucket, u32)> {
    DensityBucket::all()
        .iter()
        .map(|bucket| {
            let size = (base_size as f32 * bucket.scale()).round() as u32;
            (*bucket, size)
        })
        .collect()
}

/// Calculate the appropriate image asset name for a density bucket.
pub fn asset_name(base_name: &str, bucket: DensityBucket, platform: &str) -> String {
    match platform {
        "ios" => format!("{}{}", base_name, bucket.ios_suffix()),
        "android" => format!("drawable-{}/{}", bucket.android_qualifier(), base_name),
        _ => base_name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_density_from_scale() {
        let density = ScreenDensity::from_scale(2.0);
        assert_eq!(density.scale(), 2.0);
        assert_eq!(density.dpi(), 320.0);
    }

    #[test]
    fn test_screen_density_from_dpi() {
        let density = ScreenDensity::from_dpi(320.0);
        assert_eq!(density.scale(), 2.0);
        assert_eq!(density.dpi(), 320.0);
    }

    #[test]
    fn test_density_conversion() {
        let density = ScreenDensity::from_scale(3.0);

        assert_eq!(density.dp_to_px(10.0), 30.0);
        assert_eq!(density.px_to_dp(30.0), 10.0);

        // Round trip
        let original = 15.0;
        let px = density.dp_to_px(original);
        let back = density.px_to_dp(px);
        assert!((back - original).abs() < 0.001);
    }

    #[test]
    fn test_density_bucket_from_scale() {
        assert_eq!(DensityBucket::from_scale(0.5), DensityBucket::Ldpi);
        assert_eq!(DensityBucket::from_scale(0.75), DensityBucket::Ldpi);
        assert_eq!(DensityBucket::from_scale(1.0), DensityBucket::Mdpi);
        assert_eq!(DensityBucket::from_scale(1.5), DensityBucket::Hdpi);
        assert_eq!(DensityBucket::from_scale(2.0), DensityBucket::Xhdpi);
        assert_eq!(DensityBucket::from_scale(3.0), DensityBucket::Xxhdpi);
        assert_eq!(DensityBucket::from_scale(4.0), DensityBucket::Xxxhdpi);
    }

    #[test]
    fn test_density_bucket_scale() {
        assert_eq!(DensityBucket::Ldpi.scale(), 0.75);
        assert_eq!(DensityBucket::Mdpi.scale(), 1.0);
        assert_eq!(DensityBucket::Xhdpi.scale(), 2.0);
        assert_eq!(DensityBucket::Xxhdpi.scale(), 3.0);
    }

    #[test]
    fn test_density_bucket_qualifiers() {
        assert_eq!(DensityBucket::Xhdpi.android_qualifier(), "xhdpi");
        assert_eq!(DensityBucket::Xhdpi.ios_suffix(), "@2x");
        assert_eq!(DensityBucket::Xxhdpi.ios_suffix(), "@3x");
    }

    #[test]
    fn test_density_scaler() {
        let scaler = DensityScaler::from_scales(1.0, 3.0);
        assert_eq!(scaler.ratio(), 3.0);
        assert_eq!(scaler.scale(10.0), 30.0);
        assert_eq!(scaler.scale_round(10.5), 32);
    }

    #[test]
    fn test_icon_sizes() {
        let sizes = icon_sizes(24);

        let xhdpi = sizes.iter().find(|(b, _)| *b == DensityBucket::Xhdpi);
        assert_eq!(xhdpi.map(|(_, s)| *s), Some(48));

        let xxhdpi = sizes.iter().find(|(b, _)| *b == DensityBucket::Xxhdpi);
        assert_eq!(xxhdpi.map(|(_, s)| *s), Some(72));
    }

    #[test]
    fn test_asset_name() {
        assert_eq!(
            asset_name("icon", DensityBucket::Xhdpi, "ios"),
            "icon@2x"
        );
        assert_eq!(
            asset_name("icon", DensityBucket::Xhdpi, "android"),
            "drawable-xhdpi/icon"
        );
    }

    #[test]
    fn test_sp_to_px() {
        let density = ScreenDensity::from_scale(2.0);

        // Normal font scale
        assert_eq!(density.sp_to_px(14.0, 1.0), 28.0);

        // Large font scale (accessibility)
        assert_eq!(density.sp_to_px(14.0, 1.5), 42.0);
    }
}
