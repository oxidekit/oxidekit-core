//! Native shell abstraction for mobile platforms.
//!
//! Provides a unified interface for mobile application lifecycle management,
//! orientation handling, and platform-specific shell features.
//!
//! ## Modules
//!
//! - [`ios`]: iOS-specific shell types and lifecycle events
//! - [`android`]: Android-specific shell types and lifecycle events

pub mod ios;
pub mod android;

use serde::{Deserialize, Serialize};

pub use ios::{IosShell, IosLifecycleEvent, IosSceneState};
pub use android::{AndroidShell, AndroidLifecycleEvent, AndroidActivityState};

/// Mobile application lifecycle events.
///
/// These events map to native platform lifecycle callbacks and should be
/// handled to properly manage app state, resources, and user experience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MobileLifecycleEvent {
    /// App is about to enter the foreground (becoming active).
    ///
    /// - iOS: `applicationWillEnterForeground`
    /// - Android: `onStart`
    WillEnterForeground,

    /// App has entered the foreground and is now active.
    ///
    /// - iOS: `applicationDidBecomeActive`
    /// - Android: `onResume`
    DidEnterForeground,

    /// App is about to lose focus (user interaction).
    ///
    /// - iOS: `applicationWillResignActive`
    /// - Android: `onPause`
    WillResignActive,

    /// App has entered the background.
    ///
    /// - iOS: `applicationDidEnterBackground`
    /// - Android: `onStop`
    DidEnterBackground,

    /// App is about to be terminated.
    ///
    /// - iOS: `applicationWillTerminate`
    /// - Android: `onDestroy`
    WillTerminate,

    /// System memory warning received.
    ///
    /// App should release non-critical resources.
    MemoryWarning,

    /// Device orientation changed.
    OrientationChanged(Orientation),

    /// App gained focus after being interrupted.
    DidBecomeActive,

    /// App is being suspended (iOS specific).
    WillSuspend,
}

impl MobileLifecycleEvent {
    /// Returns true if the app is transitioning to an active state.
    pub fn is_activating(&self) -> bool {
        matches!(
            self,
            MobileLifecycleEvent::WillEnterForeground
                | MobileLifecycleEvent::DidEnterForeground
                | MobileLifecycleEvent::DidBecomeActive
        )
    }

    /// Returns true if the app is transitioning to an inactive state.
    pub fn is_deactivating(&self) -> bool {
        matches!(
            self,
            MobileLifecycleEvent::WillResignActive
                | MobileLifecycleEvent::DidEnterBackground
                | MobileLifecycleEvent::WillSuspend
                | MobileLifecycleEvent::WillTerminate
        )
    }

    /// Returns true if this is a critical event requiring immediate action.
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            MobileLifecycleEvent::MemoryWarning | MobileLifecycleEvent::WillTerminate
        )
    }
}

/// Device orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Orientation {
    /// Portrait mode (device held upright).
    #[default]
    Portrait,
    /// Portrait mode upside down.
    PortraitUpsideDown,
    /// Landscape mode with home button/indicator on the right.
    LandscapeLeft,
    /// Landscape mode with home button/indicator on the left.
    LandscapeRight,
    /// Face up (screen facing ceiling).
    FaceUp,
    /// Face down (screen facing floor).
    FaceDown,
    /// Unknown orientation.
    Unknown,
}

impl Orientation {
    /// Returns true if this is a portrait orientation.
    pub fn is_portrait(&self) -> bool {
        matches!(self, Orientation::Portrait | Orientation::PortraitUpsideDown)
    }

    /// Returns true if this is a landscape orientation.
    pub fn is_landscape(&self) -> bool {
        matches!(
            self,
            Orientation::LandscapeLeft | Orientation::LandscapeRight
        )
    }

    /// Returns true if the device is flat.
    pub fn is_flat(&self) -> bool {
        matches!(self, Orientation::FaceUp | Orientation::FaceDown)
    }

    /// Returns true if this is a valid interactive orientation.
    pub fn is_valid(&self) -> bool {
        !matches!(self, Orientation::Unknown | Orientation::FaceUp | Orientation::FaceDown)
    }

    /// Get the rotation angle in degrees from portrait.
    pub fn rotation_degrees(&self) -> f32 {
        match self {
            Orientation::Portrait => 0.0,
            Orientation::LandscapeRight => 90.0,
            Orientation::PortraitUpsideDown => 180.0,
            Orientation::LandscapeLeft => 270.0,
            _ => 0.0,
        }
    }

    /// Get the rotation angle in radians from portrait.
    pub fn rotation_radians(&self) -> f32 {
        self.rotation_degrees().to_radians()
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Orientation::Portrait => "Portrait",
            Orientation::PortraitUpsideDown => "Portrait Upside Down",
            Orientation::LandscapeLeft => "Landscape Left",
            Orientation::LandscapeRight => "Landscape Right",
            Orientation::FaceUp => "Face Up",
            Orientation::FaceDown => "Face Down",
            Orientation::Unknown => "Unknown",
        };
        write!(f, "{}", name)
    }
}

/// Allowed orientations for the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrientationMask {
    portrait: bool,
    portrait_upside_down: bool,
    landscape_left: bool,
    landscape_right: bool,
}

impl OrientationMask {
    /// Allow all orientations.
    pub fn all() -> Self {
        Self {
            portrait: true,
            portrait_upside_down: true,
            landscape_left: true,
            landscape_right: true,
        }
    }

    /// Allow only portrait orientations.
    pub fn portrait_only() -> Self {
        Self {
            portrait: true,
            portrait_upside_down: false,
            landscape_left: false,
            landscape_right: false,
        }
    }

    /// Allow only landscape orientations.
    pub fn landscape_only() -> Self {
        Self {
            portrait: false,
            portrait_upside_down: false,
            landscape_left: true,
            landscape_right: true,
        }
    }

    /// Allow portrait and landscape (no upside down).
    pub fn standard() -> Self {
        Self {
            portrait: true,
            portrait_upside_down: false,
            landscape_left: true,
            landscape_right: true,
        }
    }

    /// Check if an orientation is allowed.
    pub fn allows(&self, orientation: Orientation) -> bool {
        match orientation {
            Orientation::Portrait => self.portrait,
            Orientation::PortraitUpsideDown => self.portrait_upside_down,
            Orientation::LandscapeLeft => self.landscape_left,
            Orientation::LandscapeRight => self.landscape_right,
            _ => false,
        }
    }

    /// Get iOS orientation mask value.
    pub fn ios_mask(&self) -> u32 {
        let mut mask = 0u32;
        if self.portrait {
            mask |= 1 << 1; // UIInterfaceOrientationMaskPortrait
        }
        if self.portrait_upside_down {
            mask |= 1 << 2; // UIInterfaceOrientationMaskPortraitUpsideDown
        }
        if self.landscape_left {
            mask |= 1 << 4; // UIInterfaceOrientationMaskLandscapeLeft
        }
        if self.landscape_right {
            mask |= 1 << 3; // UIInterfaceOrientationMaskLandscapeRight
        }
        mask
    }

    /// Get Android screen orientation value.
    pub fn android_orientation(&self) -> &'static str {
        if self.portrait && !self.landscape_left && !self.landscape_right {
            "portrait"
        } else if !self.portrait && self.landscape_left && self.landscape_right {
            "landscape"
        } else if self.portrait && self.landscape_left && self.landscape_right {
            "fullSensor"
        } else {
            "unspecified"
        }
    }
}

impl Default for OrientationMask {
    fn default() -> Self {
        Self::standard()
    }
}

/// Trait for mobile shell implementations.
pub trait MobileShell {
    /// Get the current application state.
    fn is_active(&self) -> bool;

    /// Get the current device orientation.
    fn orientation(&self) -> Orientation;

    /// Get the allowed orientation mask.
    fn orientation_mask(&self) -> OrientationMask;

    /// Set the allowed orientation mask.
    fn set_orientation_mask(&mut self, mask: OrientationMask);

    /// Get the status bar height in points.
    fn status_bar_height(&self) -> f32;

    /// Check if the status bar is visible.
    fn is_status_bar_visible(&self) -> bool;

    /// Set status bar visibility.
    fn set_status_bar_visible(&mut self, visible: bool);

    /// Get the home indicator height (iOS) or navigation bar height (Android).
    fn home_indicator_height(&self) -> f32;

    /// Request a specific orientation.
    fn request_orientation(&mut self, orientation: Orientation);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_event_classification() {
        assert!(MobileLifecycleEvent::WillEnterForeground.is_activating());
        assert!(MobileLifecycleEvent::DidEnterForeground.is_activating());
        assert!(!MobileLifecycleEvent::WillResignActive.is_activating());

        assert!(MobileLifecycleEvent::WillResignActive.is_deactivating());
        assert!(MobileLifecycleEvent::DidEnterBackground.is_deactivating());
        assert!(!MobileLifecycleEvent::DidEnterForeground.is_deactivating());

        assert!(MobileLifecycleEvent::MemoryWarning.is_critical());
        assert!(MobileLifecycleEvent::WillTerminate.is_critical());
        assert!(!MobileLifecycleEvent::DidEnterBackground.is_critical());
    }

    #[test]
    fn test_orientation() {
        assert!(Orientation::Portrait.is_portrait());
        assert!(Orientation::PortraitUpsideDown.is_portrait());
        assert!(!Orientation::LandscapeLeft.is_portrait());

        assert!(Orientation::LandscapeLeft.is_landscape());
        assert!(Orientation::LandscapeRight.is_landscape());
        assert!(!Orientation::Portrait.is_landscape());

        assert!(Orientation::FaceUp.is_flat());
        assert!(Orientation::FaceDown.is_flat());
        assert!(!Orientation::Portrait.is_flat());
    }

    #[test]
    fn test_orientation_rotation() {
        assert_eq!(Orientation::Portrait.rotation_degrees(), 0.0);
        assert_eq!(Orientation::LandscapeRight.rotation_degrees(), 90.0);
        assert_eq!(Orientation::PortraitUpsideDown.rotation_degrees(), 180.0);
        assert_eq!(Orientation::LandscapeLeft.rotation_degrees(), 270.0);
    }

    #[test]
    fn test_orientation_mask() {
        let all = OrientationMask::all();
        assert!(all.allows(Orientation::Portrait));
        assert!(all.allows(Orientation::LandscapeLeft));

        let portrait = OrientationMask::portrait_only();
        assert!(portrait.allows(Orientation::Portrait));
        assert!(!portrait.allows(Orientation::LandscapeLeft));

        let landscape = OrientationMask::landscape_only();
        assert!(!landscape.allows(Orientation::Portrait));
        assert!(landscape.allows(Orientation::LandscapeLeft));
    }

    #[test]
    fn test_android_orientation_string() {
        assert_eq!(OrientationMask::portrait_only().android_orientation(), "portrait");
        assert_eq!(OrientationMask::landscape_only().android_orientation(), "landscape");
        assert_eq!(OrientationMask::standard().android_orientation(), "fullSensor");
    }
}
