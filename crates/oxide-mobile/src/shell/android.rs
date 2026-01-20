//! Android-specific shell types and lifecycle management.
//!
//! Provides types and abstractions for Android activity lifecycle,
//! configuration changes, and platform-specific features.

use serde::{Deserialize, Serialize};

use super::{MobileShell, Orientation, OrientationMask};

/// Android-specific lifecycle events.
///
/// These map to Android Activity lifecycle callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AndroidLifecycleEvent {
    /// Activity created.
    OnCreate,
    /// Activity started (becoming visible).
    OnStart,
    /// Activity resumed (interactive).
    OnResume,
    /// Activity paused (losing focus).
    OnPause,
    /// Activity stopped (no longer visible).
    OnStop,
    /// Activity destroyed.
    OnDestroy,
    /// Activity restarted.
    OnRestart,
    /// Low memory callback.
    OnLowMemory,
    /// Trim memory callback with level.
    OnTrimMemory(TrimMemoryLevel),
    /// Configuration changed.
    OnConfigurationChanged,
    /// Window focus changed.
    OnWindowFocusChanged(bool),
    /// Back button pressed.
    OnBackPressed,
}

impl AndroidLifecycleEvent {
    /// Convert to a generic mobile lifecycle event if applicable.
    pub fn to_mobile_event(&self) -> Option<super::MobileLifecycleEvent> {
        match self {
            AndroidLifecycleEvent::OnStart => {
                Some(super::MobileLifecycleEvent::WillEnterForeground)
            }
            AndroidLifecycleEvent::OnResume => {
                Some(super::MobileLifecycleEvent::DidEnterForeground)
            }
            AndroidLifecycleEvent::OnPause => Some(super::MobileLifecycleEvent::WillResignActive),
            AndroidLifecycleEvent::OnStop => Some(super::MobileLifecycleEvent::DidEnterBackground),
            AndroidLifecycleEvent::OnDestroy => Some(super::MobileLifecycleEvent::WillTerminate),
            AndroidLifecycleEvent::OnLowMemory => Some(super::MobileLifecycleEvent::MemoryWarning),
            AndroidLifecycleEvent::OnTrimMemory(level) if level.is_critical() => {
                Some(super::MobileLifecycleEvent::MemoryWarning)
            }
            _ => None,
        }
    }
}

/// Android trim memory levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrimMemoryLevel {
    /// App is running and not killable.
    RunningModerate,
    /// App is running and somewhat low on memory.
    RunningLow,
    /// App is running and extremely low on memory.
    RunningCritical,
    /// App UI is hidden.
    UiHidden,
    /// App is in background, memory is moderate.
    BackgroundModerate,
    /// App is in background, memory is low.
    BackgroundLow,
    /// App is in background, will be killed soon.
    BackgroundCritical,
    /// Complete memory trim.
    Complete,
}

impl TrimMemoryLevel {
    /// Get the Android constant value.
    pub fn raw_value(&self) -> i32 {
        match self {
            TrimMemoryLevel::RunningModerate => 5,
            TrimMemoryLevel::RunningLow => 10,
            TrimMemoryLevel::RunningCritical => 15,
            TrimMemoryLevel::UiHidden => 20,
            TrimMemoryLevel::BackgroundModerate => 40,
            TrimMemoryLevel::BackgroundLow => 60,
            TrimMemoryLevel::BackgroundCritical => 80,
            TrimMemoryLevel::Complete => 80,
        }
    }

    /// Create from Android constant value.
    pub fn from_raw(value: i32) -> Self {
        match value {
            5 => TrimMemoryLevel::RunningModerate,
            10 => TrimMemoryLevel::RunningLow,
            15 => TrimMemoryLevel::RunningCritical,
            20 => TrimMemoryLevel::UiHidden,
            40 => TrimMemoryLevel::BackgroundModerate,
            60 => TrimMemoryLevel::BackgroundLow,
            80 => TrimMemoryLevel::BackgroundCritical,
            _ => TrimMemoryLevel::Complete,
        }
    }

    /// Returns true if this is a critical memory state.
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            TrimMemoryLevel::RunningCritical
                | TrimMemoryLevel::BackgroundCritical
                | TrimMemoryLevel::Complete
        )
    }

    /// Returns true if the app is in the background.
    pub fn is_background(&self) -> bool {
        matches!(
            self,
            TrimMemoryLevel::UiHidden
                | TrimMemoryLevel::BackgroundModerate
                | TrimMemoryLevel::BackgroundLow
                | TrimMemoryLevel::BackgroundCritical
        )
    }
}

/// Android activity state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AndroidActivityState {
    /// Activity is created but not started.
    Created,
    /// Activity is started (visible).
    #[default]
    Started,
    /// Activity is resumed (interactive).
    Resumed,
    /// Activity is paused.
    Paused,
    /// Activity is stopped.
    Stopped,
    /// Activity is destroyed.
    Destroyed,
}

impl AndroidActivityState {
    /// Returns true if the activity is visible.
    pub fn is_visible(&self) -> bool {
        matches!(
            self,
            AndroidActivityState::Started
                | AndroidActivityState::Resumed
                | AndroidActivityState::Paused
        )
    }

    /// Returns true if the activity is interactive.
    pub fn is_interactive(&self) -> bool {
        matches!(self, AndroidActivityState::Resumed)
    }

    /// Returns true if the activity is in the background.
    pub fn is_background(&self) -> bool {
        matches!(
            self,
            AndroidActivityState::Stopped | AndroidActivityState::Destroyed
        )
    }
}

impl std::fmt::Display for AndroidActivityState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            AndroidActivityState::Created => "Created",
            AndroidActivityState::Started => "Started",
            AndroidActivityState::Resumed => "Resumed",
            AndroidActivityState::Paused => "Paused",
            AndroidActivityState::Stopped => "Stopped",
            AndroidActivityState::Destroyed => "Destroyed",
        };
        write!(f, "{}", name)
    }
}

/// Android navigation bar mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AndroidNavigationMode {
    /// Three-button navigation (back, home, recent).
    ThreeButton,
    /// Two-button navigation (back gesture, home+recent button).
    TwoButton,
    /// Full gesture navigation.
    #[default]
    GestureNavigation,
}

impl AndroidNavigationMode {
    /// Get the typical navigation bar height in dp.
    pub fn typical_height_dp(&self) -> f32 {
        match self {
            AndroidNavigationMode::ThreeButton => 48.0,
            AndroidNavigationMode::TwoButton => 48.0,
            AndroidNavigationMode::GestureNavigation => 20.0,
        }
    }
}

/// Android system UI visibility flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AndroidSystemUiFlags {
    /// Hide status bar.
    pub hide_status_bar: bool,
    /// Hide navigation bar.
    pub hide_navigation_bar: bool,
    /// Fullscreen mode.
    pub fullscreen: bool,
    /// Immersive mode.
    pub immersive: bool,
    /// Sticky immersive mode.
    pub immersive_sticky: bool,
    /// Light status bar (dark icons).
    pub light_status_bar: bool,
    /// Light navigation bar (dark icons).
    pub light_navigation_bar: bool,
}

impl AndroidSystemUiFlags {
    /// Create flags for normal mode.
    pub fn normal() -> Self {
        Self::default()
    }

    /// Create flags for fullscreen mode.
    pub fn fullscreen() -> Self {
        Self {
            fullscreen: true,
            hide_status_bar: true,
            hide_navigation_bar: true,
            immersive_sticky: true,
            ..Default::default()
        }
    }

    /// Create flags for immersive mode.
    pub fn immersive() -> Self {
        Self {
            fullscreen: true,
            hide_status_bar: true,
            hide_navigation_bar: true,
            immersive: true,
            ..Default::default()
        }
    }

    /// Get the system UI visibility integer value.
    pub fn to_visibility_flags(&self) -> u32 {
        let mut flags = 0u32;

        if self.fullscreen {
            flags |= 0x00000004; // SYSTEM_UI_FLAG_FULLSCREEN
        }
        if self.hide_status_bar {
            flags |= 0x00000004; // SYSTEM_UI_FLAG_FULLSCREEN
        }
        if self.hide_navigation_bar {
            flags |= 0x00000002; // SYSTEM_UI_FLAG_HIDE_NAVIGATION
        }
        if self.immersive {
            flags |= 0x00000800; // SYSTEM_UI_FLAG_IMMERSIVE
        }
        if self.immersive_sticky {
            flags |= 0x00001000; // SYSTEM_UI_FLAG_IMMERSIVE_STICKY
        }
        if self.light_status_bar {
            flags |= 0x00002000; // SYSTEM_UI_FLAG_LIGHT_STATUS_BAR
        }
        if self.light_navigation_bar {
            flags |= 0x00000010; // SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR
        }

        flags
    }
}

/// Android shell implementation.
#[derive(Debug, Clone)]
pub struct AndroidShell {
    /// Current activity state.
    activity_state: AndroidActivityState,
    /// Current device orientation.
    orientation: Orientation,
    /// Allowed orientations.
    orientation_mask: OrientationMask,
    /// Status bar height in pixels.
    status_bar_height_px: f32,
    /// Navigation bar height in pixels.
    navigation_bar_height_px: f32,
    /// Navigation mode.
    navigation_mode: AndroidNavigationMode,
    /// System UI flags.
    system_ui_flags: AndroidSystemUiFlags,
    /// Screen density.
    density: f32,
    /// Has window focus.
    has_focus: bool,
    /// Has camera cutout.
    has_cutout: bool,
}

impl AndroidShell {
    /// Create a new Android shell with default values.
    pub fn new() -> Self {
        Self {
            activity_state: AndroidActivityState::Started,
            orientation: Orientation::Portrait,
            orientation_mask: OrientationMask::standard(),
            status_bar_height_px: 72.0, // 24dp * 3x density
            navigation_bar_height_px: 144.0, // 48dp * 3x density
            navigation_mode: AndroidNavigationMode::GestureNavigation,
            system_ui_flags: AndroidSystemUiFlags::default(),
            density: 3.0,
            has_focus: true,
            has_cutout: false,
        }
    }

    /// Create an Android shell with gesture navigation.
    pub fn gesture_navigation() -> Self {
        Self {
            navigation_mode: AndroidNavigationMode::GestureNavigation,
            navigation_bar_height_px: 60.0, // 20dp * 3x density
            ..Self::new()
        }
    }

    /// Create an Android shell with three-button navigation.
    pub fn three_button_navigation() -> Self {
        Self {
            navigation_mode: AndroidNavigationMode::ThreeButton,
            navigation_bar_height_px: 144.0, // 48dp * 3x density
            ..Self::new()
        }
    }

    /// Get the current activity state.
    pub fn activity_state(&self) -> AndroidActivityState {
        self.activity_state
    }

    /// Set the activity state.
    pub fn set_activity_state(&mut self, state: AndroidActivityState) {
        self.activity_state = state;
    }

    /// Get the navigation mode.
    pub fn navigation_mode(&self) -> AndroidNavigationMode {
        self.navigation_mode
    }

    /// Set the navigation mode.
    pub fn set_navigation_mode(&mut self, mode: AndroidNavigationMode) {
        self.navigation_mode = mode;
        self.navigation_bar_height_px = mode.typical_height_dp() * self.density;
    }

    /// Get the system UI flags.
    pub fn system_ui_flags(&self) -> &AndroidSystemUiFlags {
        &self.system_ui_flags
    }

    /// Set the system UI flags.
    pub fn set_system_ui_flags(&mut self, flags: AndroidSystemUiFlags) {
        self.system_ui_flags = flags;
    }

    /// Get the screen density.
    pub fn density(&self) -> f32 {
        self.density
    }

    /// Set the screen density.
    pub fn set_density(&mut self, density: f32) {
        self.density = density;
    }

    /// Check if the window has focus.
    pub fn has_focus(&self) -> bool {
        self.has_focus
    }

    /// Check if the device has a camera cutout.
    pub fn has_cutout(&self) -> bool {
        self.has_cutout
    }

    /// Handle a lifecycle event.
    pub fn handle_lifecycle_event(&mut self, event: AndroidLifecycleEvent) {
        match event {
            AndroidLifecycleEvent::OnCreate => {
                self.activity_state = AndroidActivityState::Created;
            }
            AndroidLifecycleEvent::OnStart => {
                self.activity_state = AndroidActivityState::Started;
            }
            AndroidLifecycleEvent::OnResume => {
                self.activity_state = AndroidActivityState::Resumed;
            }
            AndroidLifecycleEvent::OnPause => {
                self.activity_state = AndroidActivityState::Paused;
            }
            AndroidLifecycleEvent::OnStop => {
                self.activity_state = AndroidActivityState::Stopped;
            }
            AndroidLifecycleEvent::OnDestroy => {
                self.activity_state = AndroidActivityState::Destroyed;
            }
            AndroidLifecycleEvent::OnWindowFocusChanged(has_focus) => {
                self.has_focus = has_focus;
            }
            _ => {}
        }
    }

    /// Convert pixels to density-independent pixels.
    pub fn px_to_dp(&self, px: f32) -> f32 {
        px / self.density
    }

    /// Convert density-independent pixels to pixels.
    pub fn dp_to_px(&self, dp: f32) -> f32 {
        dp * self.density
    }
}

impl Default for AndroidShell {
    fn default() -> Self {
        Self::new()
    }
}

impl MobileShell for AndroidShell {
    fn is_active(&self) -> bool {
        self.activity_state.is_interactive() && self.has_focus
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
        if self.system_ui_flags.hide_status_bar {
            0.0
        } else {
            self.px_to_dp(self.status_bar_height_px)
        }
    }

    fn is_status_bar_visible(&self) -> bool {
        !self.system_ui_flags.hide_status_bar
    }

    fn set_status_bar_visible(&mut self, visible: bool) {
        self.system_ui_flags.hide_status_bar = !visible;
    }

    fn home_indicator_height(&self) -> f32 {
        if self.system_ui_flags.hide_navigation_bar {
            0.0
        } else {
            self.px_to_dp(self.navigation_bar_height_px)
        }
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
    fn test_android_lifecycle_to_mobile() {
        assert!(AndroidLifecycleEvent::OnStart.to_mobile_event().is_some());
        assert!(AndroidLifecycleEvent::OnResume.to_mobile_event().is_some());
        assert!(AndroidLifecycleEvent::OnCreate.to_mobile_event().is_none());
    }

    #[test]
    fn test_trim_memory_level() {
        assert!(TrimMemoryLevel::RunningCritical.is_critical());
        assert!(TrimMemoryLevel::BackgroundCritical.is_critical());
        assert!(!TrimMemoryLevel::RunningModerate.is_critical());

        assert!(TrimMemoryLevel::UiHidden.is_background());
        assert!(!TrimMemoryLevel::RunningModerate.is_background());
    }

    #[test]
    fn test_android_activity_state() {
        assert!(AndroidActivityState::Resumed.is_interactive());
        assert!(!AndroidActivityState::Started.is_interactive());

        assert!(AndroidActivityState::Resumed.is_visible());
        assert!(AndroidActivityState::Started.is_visible());
        assert!(!AndroidActivityState::Stopped.is_visible());
    }

    #[test]
    fn test_android_shell_lifecycle() {
        let mut shell = AndroidShell::new();

        shell.handle_lifecycle_event(AndroidLifecycleEvent::OnResume);
        assert!(shell.activity_state().is_interactive());

        shell.handle_lifecycle_event(AndroidLifecycleEvent::OnPause);
        assert!(!shell.activity_state().is_interactive());

        shell.handle_lifecycle_event(AndroidLifecycleEvent::OnStop);
        assert!(shell.activity_state().is_background());
    }

    #[test]
    fn test_android_navigation_modes() {
        let gesture = AndroidShell::gesture_navigation();
        assert!(matches!(
            gesture.navigation_mode(),
            AndroidNavigationMode::GestureNavigation
        ));

        let three_button = AndroidShell::three_button_navigation();
        assert!(matches!(
            three_button.navigation_mode(),
            AndroidNavigationMode::ThreeButton
        ));
    }

    #[test]
    fn test_system_ui_flags() {
        let normal = AndroidSystemUiFlags::normal();
        assert!(!normal.fullscreen);

        let fullscreen = AndroidSystemUiFlags::fullscreen();
        assert!(fullscreen.fullscreen);
        assert!(fullscreen.hide_status_bar);
        assert!(fullscreen.hide_navigation_bar);
    }

    #[test]
    fn test_dp_px_conversion() {
        let shell = AndroidShell::new();

        let dp = 10.0;
        let px = shell.dp_to_px(dp);
        let back = shell.px_to_dp(px);
        assert!((back - dp).abs() < 0.001);
    }
}
