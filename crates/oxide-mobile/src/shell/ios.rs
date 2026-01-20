//! iOS-specific shell types and lifecycle management.
//!
//! Provides types and abstractions for iOS application lifecycle,
//! scene management, and platform-specific features.

use serde::{Deserialize, Serialize};

use super::{MobileShell, Orientation, OrientationMask};

/// iOS-specific lifecycle events.
///
/// These map directly to UIKit application delegate callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IosLifecycleEvent {
    /// Application finished launching.
    DidFinishLaunching,
    /// Application will enter foreground.
    WillEnterForeground,
    /// Application did become active.
    DidBecomeActive,
    /// Application will resign active.
    WillResignActive,
    /// Application did enter background.
    DidEnterBackground,
    /// Application will terminate.
    WillTerminate,
    /// Received memory warning.
    DidReceiveMemoryWarning,
    /// Significant time change (midnight, timezone, etc.).
    SignificantTimeChange,
    /// Protected data will become unavailable.
    ProtectedDataWillBecomeUnavailable,
    /// Protected data did become available.
    ProtectedDataDidBecomeAvailable,
    /// User took a screenshot.
    UserDidTakeScreenshot,
}

impl IosLifecycleEvent {
    /// Convert to a generic mobile lifecycle event if applicable.
    pub fn to_mobile_event(&self) -> Option<super::MobileLifecycleEvent> {
        match self {
            IosLifecycleEvent::WillEnterForeground => {
                Some(super::MobileLifecycleEvent::WillEnterForeground)
            }
            IosLifecycleEvent::DidBecomeActive => {
                Some(super::MobileLifecycleEvent::DidEnterForeground)
            }
            IosLifecycleEvent::WillResignActive => {
                Some(super::MobileLifecycleEvent::WillResignActive)
            }
            IosLifecycleEvent::DidEnterBackground => {
                Some(super::MobileLifecycleEvent::DidEnterBackground)
            }
            IosLifecycleEvent::WillTerminate => Some(super::MobileLifecycleEvent::WillTerminate),
            IosLifecycleEvent::DidReceiveMemoryWarning => {
                Some(super::MobileLifecycleEvent::MemoryWarning)
            }
            _ => None,
        }
    }
}

/// iOS scene activation state.
///
/// Represents the activation state of a UIScene in iOS 13+.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum IosSceneState {
    /// Scene is not attached to the app.
    Unattached,
    /// Scene is in the foreground but not active.
    #[default]
    ForegroundInactive,
    /// Scene is in the foreground and active.
    ForegroundActive,
    /// Scene is in the background.
    Background,
}

impl IosSceneState {
    /// Returns true if the scene is in the foreground.
    pub fn is_foreground(&self) -> bool {
        matches!(
            self,
            IosSceneState::ForegroundActive | IosSceneState::ForegroundInactive
        )
    }

    /// Returns true if the scene is active.
    pub fn is_active(&self) -> bool {
        matches!(self, IosSceneState::ForegroundActive)
    }

    /// Returns true if the scene is in the background.
    pub fn is_background(&self) -> bool {
        matches!(self, IosSceneState::Background)
    }
}

impl std::fmt::Display for IosSceneState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            IosSceneState::Unattached => "Unattached",
            IosSceneState::ForegroundInactive => "Foreground Inactive",
            IosSceneState::ForegroundActive => "Foreground Active",
            IosSceneState::Background => "Background",
        };
        write!(f, "{}", name)
    }
}

/// iOS status bar style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum IosStatusBarStyle {
    /// Default style (dark content on light background).
    #[default]
    Default,
    /// Light content (for dark backgrounds).
    LightContent,
    /// Dark content (for light backgrounds).
    DarkContent,
}

impl IosStatusBarStyle {
    /// Get the UIStatusBarStyle raw value.
    pub fn raw_value(&self) -> i32 {
        match self {
            IosStatusBarStyle::Default => 0,
            IosStatusBarStyle::LightContent => 1,
            IosStatusBarStyle::DarkContent => 3,
        }
    }
}

/// iOS user interface style (light/dark mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum IosUserInterfaceStyle {
    /// System determines the style.
    #[default]
    Unspecified,
    /// Light mode.
    Light,
    /// Dark mode.
    Dark,
}

impl IosUserInterfaceStyle {
    /// Get the UIUserInterfaceStyle raw value.
    pub fn raw_value(&self) -> i32 {
        match self {
            IosUserInterfaceStyle::Unspecified => 0,
            IosUserInterfaceStyle::Light => 1,
            IosUserInterfaceStyle::Dark => 2,
        }
    }
}

/// iOS shell implementation.
#[derive(Debug, Clone)]
pub struct IosShell {
    /// Current scene state.
    scene_state: IosSceneState,
    /// Current device orientation.
    orientation: Orientation,
    /// Allowed orientations.
    orientation_mask: OrientationMask,
    /// Status bar visibility.
    status_bar_visible: bool,
    /// Status bar style.
    status_bar_style: IosStatusBarStyle,
    /// User interface style.
    user_interface_style: IosUserInterfaceStyle,
    /// Status bar height in points.
    status_bar_height: f32,
    /// Home indicator height in points.
    home_indicator_height: f32,
    /// Whether the device has a notch.
    has_notch: bool,
    /// Whether the device has Dynamic Island.
    has_dynamic_island: bool,
}

impl IosShell {
    /// Create a new iOS shell with default values.
    pub fn new() -> Self {
        Self {
            scene_state: IosSceneState::ForegroundInactive,
            orientation: Orientation::Portrait,
            orientation_mask: OrientationMask::standard(),
            status_bar_visible: true,
            status_bar_style: IosStatusBarStyle::Default,
            user_interface_style: IosUserInterfaceStyle::Unspecified,
            status_bar_height: 47.0, // Modern iPhone with notch
            home_indicator_height: 34.0,
            has_notch: true,
            has_dynamic_island: false,
        }
    }

    /// Create an iOS shell for iPhone with notch.
    pub fn iphone_notch() -> Self {
        Self {
            status_bar_height: 47.0,
            home_indicator_height: 34.0,
            has_notch: true,
            has_dynamic_island: false,
            ..Self::new()
        }
    }

    /// Create an iOS shell for iPhone with Dynamic Island.
    pub fn iphone_dynamic_island() -> Self {
        Self {
            status_bar_height: 59.0,
            home_indicator_height: 34.0,
            has_notch: false,
            has_dynamic_island: true,
            ..Self::new()
        }
    }

    /// Create an iOS shell for classic iPhone (with home button).
    pub fn iphone_classic() -> Self {
        Self {
            status_bar_height: 20.0,
            home_indicator_height: 0.0,
            has_notch: false,
            has_dynamic_island: false,
            ..Self::new()
        }
    }

    /// Create an iOS shell for iPad.
    pub fn ipad() -> Self {
        Self {
            status_bar_height: 24.0,
            home_indicator_height: 20.0,
            has_notch: false,
            has_dynamic_island: false,
            orientation_mask: OrientationMask::all(),
            ..Self::new()
        }
    }

    /// Get the current scene state.
    pub fn scene_state(&self) -> IosSceneState {
        self.scene_state
    }

    /// Set the scene state.
    pub fn set_scene_state(&mut self, state: IosSceneState) {
        self.scene_state = state;
    }

    /// Get the status bar style.
    pub fn status_bar_style(&self) -> IosStatusBarStyle {
        self.status_bar_style
    }

    /// Set the status bar style.
    pub fn set_status_bar_style(&mut self, style: IosStatusBarStyle) {
        self.status_bar_style = style;
    }

    /// Get the user interface style.
    pub fn user_interface_style(&self) -> IosUserInterfaceStyle {
        self.user_interface_style
    }

    /// Set the user interface style.
    pub fn set_user_interface_style(&mut self, style: IosUserInterfaceStyle) {
        self.user_interface_style = style;
    }

    /// Check if the device has a notch.
    pub fn has_notch(&self) -> bool {
        self.has_notch
    }

    /// Check if the device has Dynamic Island.
    pub fn has_dynamic_island(&self) -> bool {
        self.has_dynamic_island
    }

    /// Handle a lifecycle event.
    pub fn handle_lifecycle_event(&mut self, event: IosLifecycleEvent) {
        match event {
            IosLifecycleEvent::DidBecomeActive => {
                self.scene_state = IosSceneState::ForegroundActive;
            }
            IosLifecycleEvent::WillResignActive => {
                self.scene_state = IosSceneState::ForegroundInactive;
            }
            IosLifecycleEvent::DidEnterBackground => {
                self.scene_state = IosSceneState::Background;
            }
            IosLifecycleEvent::WillEnterForeground => {
                self.scene_state = IosSceneState::ForegroundInactive;
            }
            _ => {}
        }
    }
}

impl Default for IosShell {
    fn default() -> Self {
        Self::new()
    }
}

impl MobileShell for IosShell {
    fn is_active(&self) -> bool {
        self.scene_state.is_active()
    }

    fn orientation(&self) -> Orientation {
        self.orientation
    }

    fn orientation_mask(&self) -> OrientationMask {
        self.orientation_mask.clone()
    }

    fn set_orientation_mask(&mut self, mask: OrientationMask) {
        self.orientation_mask = mask;
    }

    fn status_bar_height(&self) -> f32 {
        if self.status_bar_visible {
            self.status_bar_height
        } else {
            0.0
        }
    }

    fn is_status_bar_visible(&self) -> bool {
        self.status_bar_visible
    }

    fn set_status_bar_visible(&mut self, visible: bool) {
        self.status_bar_visible = visible;
    }

    fn home_indicator_height(&self) -> f32 {
        self.home_indicator_height
    }

    fn request_orientation(&mut self, orientation: Orientation) {
        if self.orientation_mask.allows(orientation) {
            self.orientation = orientation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_lifecycle_to_mobile() {
        assert!(IosLifecycleEvent::WillEnterForeground.to_mobile_event().is_some());
        assert!(IosLifecycleEvent::DidBecomeActive.to_mobile_event().is_some());
        assert!(IosLifecycleEvent::SignificantTimeChange.to_mobile_event().is_none());
    }

    #[test]
    fn test_ios_scene_state() {
        assert!(IosSceneState::ForegroundActive.is_foreground());
        assert!(IosSceneState::ForegroundActive.is_active());
        assert!(!IosSceneState::ForegroundInactive.is_active());
        assert!(IosSceneState::Background.is_background());
    }

    #[test]
    fn test_ios_shell_lifecycle() {
        let mut shell = IosShell::new();

        shell.handle_lifecycle_event(IosLifecycleEvent::DidBecomeActive);
        assert!(shell.is_active());

        shell.handle_lifecycle_event(IosLifecycleEvent::WillResignActive);
        assert!(!shell.is_active());

        shell.handle_lifecycle_event(IosLifecycleEvent::DidEnterBackground);
        assert!(shell.scene_state().is_background());
    }

    #[test]
    fn test_ios_shell_variants() {
        let notch = IosShell::iphone_notch();
        assert!(notch.has_notch());
        assert!(!notch.has_dynamic_island());
        assert_eq!(notch.status_bar_height, 47.0);

        let dynamic_island = IosShell::iphone_dynamic_island();
        assert!(!dynamic_island.has_notch());
        assert!(dynamic_island.has_dynamic_island());
        assert_eq!(dynamic_island.status_bar_height, 59.0);

        let classic = IosShell::iphone_classic();
        assert!(!classic.has_notch());
        assert_eq!(classic.home_indicator_height, 0.0);
    }

    #[test]
    fn test_status_bar_style_raw_values() {
        assert_eq!(IosStatusBarStyle::Default.raw_value(), 0);
        assert_eq!(IosStatusBarStyle::LightContent.raw_value(), 1);
        assert_eq!(IosStatusBarStyle::DarkContent.raw_value(), 3);
    }
}
