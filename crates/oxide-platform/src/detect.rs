//! Platform Detection
//!
//! Provides runtime and compile-time platform detection for OxideKit applications.
//! Supports desktop platforms (macOS, Windows, Linux), mobile platforms (iOS, Android),
//! and web (WebAssembly).

use serde::{Deserialize, Serialize};

/// Represents the current operating system/platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// macOS desktop
    MacOS,
    /// Windows desktop
    Windows,
    /// Linux desktop
    Linux,
    /// iOS mobile (iPhone, iPad)
    IOS,
    /// Android mobile
    Android,
    /// Web (WebAssembly)
    Web,
    /// Unknown or unsupported platform
    Unknown,
}

impl Platform {
    /// Detect the current platform at runtime.
    ///
    /// Uses conditional compilation to determine the platform.
    #[must_use]
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            Platform::MacOS
        }
        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }
        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }
        #[cfg(target_os = "ios")]
        {
            Platform::IOS
        }
        #[cfg(target_os = "android")]
        {
            Platform::Android
        }
        #[cfg(target_family = "wasm")]
        {
            Platform::Web
        }
        #[cfg(not(any(
            target_os = "macos",
            target_os = "windows",
            target_os = "linux",
            target_os = "ios",
            target_os = "android",
            target_family = "wasm"
        )))]
        {
            Platform::Unknown
        }
    }

    /// Returns true if running on a desktop platform (macOS, Windows, Linux).
    #[must_use]
    pub fn is_desktop(&self) -> bool {
        matches!(self, Platform::MacOS | Platform::Windows | Platform::Linux)
    }

    /// Returns true if running on a mobile platform (iOS, Android).
    #[must_use]
    pub fn is_mobile(&self) -> bool {
        matches!(self, Platform::IOS | Platform::Android)
    }

    /// Returns true if running on an Apple platform (macOS, iOS).
    #[must_use]
    pub fn is_apple(&self) -> bool {
        matches!(self, Platform::MacOS | Platform::IOS)
    }

    /// Returns true if running on Web/WebAssembly.
    #[must_use]
    pub fn is_web(&self) -> bool {
        matches!(self, Platform::Web)
    }

    /// Returns true if the platform uses Cupertino-style UI (Apple platforms).
    #[must_use]
    pub fn uses_cupertino_style(&self) -> bool {
        self.is_apple()
    }

    /// Returns true if the platform uses Material Design style (Android, Web).
    #[must_use]
    pub fn uses_material_style(&self) -> bool {
        matches!(self, Platform::Android | Platform::Web)
    }

    /// Returns true if the platform has touch as primary input.
    #[must_use]
    pub fn is_touch_primary(&self) -> bool {
        self.is_mobile()
    }

    /// Returns true if the platform typically has a hardware keyboard.
    #[must_use]
    pub fn has_hardware_keyboard(&self) -> bool {
        self.is_desktop()
    }

    /// Returns true if the platform supports haptic feedback.
    #[must_use]
    pub fn supports_haptics(&self) -> bool {
        self.is_mobile()
    }

    /// Returns true if the platform has a system back button (Android).
    #[must_use]
    pub fn has_back_button(&self) -> bool {
        matches!(self, Platform::Android)
    }

    /// Returns true if the platform supports back swipe gesture (iOS).
    #[must_use]
    pub fn supports_back_swipe(&self) -> bool {
        matches!(self, Platform::IOS)
    }

    /// Returns the platform's display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Platform::MacOS => "macOS",
            Platform::Windows => "Windows",
            Platform::Linux => "Linux",
            Platform::IOS => "iOS",
            Platform::Android => "Android",
            Platform::Web => "Web",
            Platform::Unknown => "Unknown",
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::current()
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Platform family grouping related platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformFamily {
    /// Apple platforms (macOS, iOS)
    Apple,
    /// Microsoft platforms (Windows)
    Microsoft,
    /// Linux-based platforms (Linux, Android)
    Linux,
    /// Web platforms
    Web,
    /// Unknown platform family
    Unknown,
}

impl PlatformFamily {
    /// Get the family for a given platform.
    #[must_use]
    pub fn from_platform(platform: Platform) -> Self {
        match platform {
            Platform::MacOS | Platform::IOS => PlatformFamily::Apple,
            Platform::Windows => PlatformFamily::Microsoft,
            Platform::Linux => PlatformFamily::Linux,
            Platform::Android => PlatformFamily::Linux,
            Platform::Web => PlatformFamily::Web,
            Platform::Unknown => PlatformFamily::Unknown,
        }
    }

    /// Get the current platform family.
    #[must_use]
    pub fn current() -> Self {
        Self::from_platform(Platform::current())
    }
}

/// Device form factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormFactor {
    /// Desktop computer (large screen, keyboard, mouse)
    Desktop,
    /// Laptop (portable, integrated keyboard)
    Laptop,
    /// Tablet (touch-primary, medium screen)
    Tablet,
    /// Phone (touch-primary, small screen)
    Phone,
    /// Web browser (variable)
    Browser,
    /// Unknown form factor
    Unknown,
}

impl FormFactor {
    /// Determine form factor from screen dimensions (in logical pixels).
    ///
    /// This is a heuristic and may not be accurate for all devices.
    #[must_use]
    pub fn from_screen_size(width: f32, height: f32, platform: Platform) -> Self {
        let min_dimension = width.min(height);
        let max_dimension = width.max(height);

        match platform {
            Platform::IOS | Platform::Android => {
                // Mobile platforms - distinguish phone vs tablet
                if min_dimension >= 600.0 || max_dimension >= 900.0 {
                    FormFactor::Tablet
                } else {
                    FormFactor::Phone
                }
            }
            Platform::MacOS | Platform::Windows | Platform::Linux => {
                // Desktop platforms - assume desktop/laptop based on typical sizes
                if max_dimension <= 1920.0 && min_dimension <= 1200.0 {
                    FormFactor::Laptop
                } else {
                    FormFactor::Desktop
                }
            }
            Platform::Web => FormFactor::Browser,
            Platform::Unknown => FormFactor::Unknown,
        }
    }

    /// Returns true if this is a mobile form factor (phone or tablet).
    #[must_use]
    pub fn is_mobile(&self) -> bool {
        matches!(self, FormFactor::Phone | FormFactor::Tablet)
    }

    /// Returns true if this is a desktop form factor (desktop or laptop).
    #[must_use]
    pub fn is_desktop(&self) -> bool {
        matches!(self, FormFactor::Desktop | FormFactor::Laptop)
    }
}

/// Platform capabilities and features.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// Whether the platform supports haptic feedback
    pub haptics: bool,
    /// Whether the platform has a hardware back button
    pub hardware_back_button: bool,
    /// Whether the platform supports back swipe gesture
    pub back_swipe_gesture: bool,
    /// Whether the platform has a notch or dynamic island
    pub has_notch: bool,
    /// Whether the platform supports dynamic type/font scaling
    pub dynamic_type: bool,
    /// Whether the platform has a status bar
    pub has_status_bar: bool,
    /// Whether the platform has window controls (minimize, maximize, close)
    pub window_controls: bool,
    /// Whether the platform has a menu bar
    pub menu_bar: bool,
    /// Whether the platform supports predictive back animation
    pub predictive_back: bool,
    /// Whether the platform uses edge glow for overscroll
    pub edge_glow_overscroll: bool,
    /// Whether the platform uses rubber-band bouncing for overscroll
    pub rubber_band_overscroll: bool,
}

impl PlatformCapabilities {
    /// Get capabilities for a specific platform.
    #[must_use]
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::MacOS => Self {
                haptics: false,
                hardware_back_button: false,
                back_swipe_gesture: true,
                has_notch: true, // Some Macs have notch
                dynamic_type: false,
                has_status_bar: true,
                window_controls: true,
                menu_bar: true,
                predictive_back: false,
                edge_glow_overscroll: false,
                rubber_band_overscroll: true,
            },
            Platform::Windows => Self {
                haptics: false,
                hardware_back_button: false,
                back_swipe_gesture: false,
                has_notch: false,
                dynamic_type: false,
                has_status_bar: false,
                window_controls: true,
                menu_bar: true,
                predictive_back: false,
                edge_glow_overscroll: false,
                rubber_band_overscroll: false,
            },
            Platform::Linux => Self {
                haptics: false,
                hardware_back_button: false,
                back_swipe_gesture: false,
                has_notch: false,
                dynamic_type: false,
                has_status_bar: false,
                window_controls: true,
                menu_bar: true,
                predictive_back: false,
                edge_glow_overscroll: false,
                rubber_band_overscroll: false,
            },
            Platform::IOS => Self {
                haptics: true,
                hardware_back_button: false,
                back_swipe_gesture: true,
                has_notch: true,
                dynamic_type: true,
                has_status_bar: true,
                window_controls: false,
                menu_bar: false,
                predictive_back: false,
                edge_glow_overscroll: false,
                rubber_band_overscroll: true,
            },
            Platform::Android => Self {
                haptics: true,
                hardware_back_button: true,
                back_swipe_gesture: false,
                has_notch: true, // Many Android devices have punch-hole cameras
                dynamic_type: false,
                has_status_bar: true,
                window_controls: false,
                menu_bar: false,
                predictive_back: true,
                edge_glow_overscroll: true,
                rubber_band_overscroll: false,
            },
            Platform::Web => Self {
                haptics: false,
                hardware_back_button: false,
                back_swipe_gesture: false,
                has_notch: false,
                dynamic_type: false,
                has_status_bar: false,
                window_controls: false,
                menu_bar: false,
                predictive_back: false,
                edge_glow_overscroll: false,
                rubber_band_overscroll: false,
            },
            Platform::Unknown => Self::default(),
        }
    }

    /// Get capabilities for the current platform.
    #[must_use]
    pub fn current() -> Self {
        Self::for_platform(Platform::current())
    }
}

impl Default for PlatformCapabilities {
    fn default() -> Self {
        Self {
            haptics: false,
            hardware_back_button: false,
            back_swipe_gesture: false,
            has_notch: false,
            dynamic_type: false,
            has_status_bar: false,
            window_controls: false,
            menu_bar: false,
            predictive_back: false,
            edge_glow_overscroll: false,
            rubber_band_overscroll: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_desktop_detection() {
        assert!(Platform::MacOS.is_desktop());
        assert!(Platform::Windows.is_desktop());
        assert!(Platform::Linux.is_desktop());
        assert!(!Platform::IOS.is_desktop());
        assert!(!Platform::Android.is_desktop());
        assert!(!Platform::Web.is_desktop());
    }

    #[test]
    fn test_platform_mobile_detection() {
        assert!(!Platform::MacOS.is_mobile());
        assert!(!Platform::Windows.is_mobile());
        assert!(!Platform::Linux.is_mobile());
        assert!(Platform::IOS.is_mobile());
        assert!(Platform::Android.is_mobile());
        assert!(!Platform::Web.is_mobile());
    }

    #[test]
    fn test_platform_apple_detection() {
        assert!(Platform::MacOS.is_apple());
        assert!(Platform::IOS.is_apple());
        assert!(!Platform::Windows.is_apple());
        assert!(!Platform::Android.is_apple());
    }

    #[test]
    fn test_platform_cupertino_style() {
        assert!(Platform::MacOS.uses_cupertino_style());
        assert!(Platform::IOS.uses_cupertino_style());
        assert!(!Platform::Android.uses_cupertino_style());
        assert!(!Platform::Windows.uses_cupertino_style());
    }

    #[test]
    fn test_platform_material_style() {
        assert!(Platform::Android.uses_material_style());
        assert!(Platform::Web.uses_material_style());
        assert!(!Platform::MacOS.uses_material_style());
        assert!(!Platform::IOS.uses_material_style());
    }

    #[test]
    fn test_platform_display_name() {
        assert_eq!(Platform::MacOS.display_name(), "macOS");
        assert_eq!(Platform::Windows.display_name(), "Windows");
        assert_eq!(Platform::IOS.display_name(), "iOS");
        assert_eq!(Platform::Android.display_name(), "Android");
    }

    #[test]
    fn test_platform_family() {
        assert_eq!(PlatformFamily::from_platform(Platform::MacOS), PlatformFamily::Apple);
        assert_eq!(PlatformFamily::from_platform(Platform::IOS), PlatformFamily::Apple);
        assert_eq!(PlatformFamily::from_platform(Platform::Windows), PlatformFamily::Microsoft);
        assert_eq!(PlatformFamily::from_platform(Platform::Android), PlatformFamily::Linux);
    }

    #[test]
    fn test_form_factor_mobile() {
        // Phone-sized
        let phone = FormFactor::from_screen_size(375.0, 812.0, Platform::IOS);
        assert_eq!(phone, FormFactor::Phone);

        // Tablet-sized
        let tablet = FormFactor::from_screen_size(768.0, 1024.0, Platform::IOS);
        assert_eq!(tablet, FormFactor::Tablet);
    }

    #[test]
    fn test_capabilities_ios() {
        let caps = PlatformCapabilities::for_platform(Platform::IOS);
        assert!(caps.haptics);
        assert!(caps.back_swipe_gesture);
        assert!(caps.has_notch);
        assert!(caps.dynamic_type);
        assert!(caps.rubber_band_overscroll);
        assert!(!caps.hardware_back_button);
    }

    #[test]
    fn test_capabilities_android() {
        let caps = PlatformCapabilities::for_platform(Platform::Android);
        assert!(caps.haptics);
        assert!(caps.hardware_back_button);
        assert!(caps.predictive_back);
        assert!(caps.edge_glow_overscroll);
        assert!(!caps.back_swipe_gesture);
    }

    #[test]
    fn test_capabilities_desktop() {
        let caps = PlatformCapabilities::for_platform(Platform::Windows);
        assert!(caps.window_controls);
        assert!(caps.menu_bar);
        assert!(!caps.haptics);
        assert!(!caps.has_status_bar);
    }
}
