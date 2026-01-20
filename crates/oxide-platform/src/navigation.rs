//! Navigation Idioms
//!
//! Provides platform-specific navigation behaviors including:
//! - iOS: Back swipe gesture, modal sheets slide up, large titles
//! - Android: Back button handling, predictive back animation
//! - Desktop: Window controls, menu bar, keyboard shortcuts

use crate::detect::Platform;
use serde::{Deserialize, Serialize};

/// Navigation action that can be triggered by platform-specific gestures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NavigationAction {
    /// Navigate back in the navigation stack
    Back,
    /// Close the current modal/sheet
    CloseModal,
    /// Open home/main screen
    Home,
    /// Open recent apps/windows
    Recents,
    /// Custom navigation action
    Custom(String),
}

/// iOS-specific navigation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IosNavigationConfig {
    /// Enable edge swipe to go back
    pub back_swipe_enabled: bool,
    /// Threshold (0.0-1.0) of screen width to trigger back action
    pub back_swipe_threshold: f32,
    /// Use large titles in navigation bar (iOS 11+)
    pub large_title_mode: LargeTitleMode,
    /// Modal presentation style
    pub modal_presentation: ModalPresentation,
    /// Enable interactive dismissal for modals
    pub interactive_dismiss: bool,
}

impl Default for IosNavigationConfig {
    fn default() -> Self {
        Self {
            back_swipe_enabled: true,
            back_swipe_threshold: 0.4,
            large_title_mode: LargeTitleMode::Automatic,
            modal_presentation: ModalPresentation::PageSheet,
            interactive_dismiss: true,
        }
    }
}

/// Large title display mode for iOS navigation bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LargeTitleMode {
    /// System decides based on context
    Automatic,
    /// Always show large title
    Always,
    /// Never show large title
    Never,
}

/// Modal presentation style for iOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModalPresentation {
    /// Full screen modal
    FullScreen,
    /// Page sheet (default iOS 13+)
    PageSheet,
    /// Form sheet (smaller modal)
    FormSheet,
    /// Slide over current context
    OverCurrentContext,
    /// Custom presentation
    Custom,
}

/// Android-specific navigation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AndroidNavigationConfig {
    /// Handle system back button
    pub handle_back_button: bool,
    /// Enable predictive back animation (Android 13+)
    pub predictive_back_enabled: bool,
    /// Animation duration for predictive back (ms)
    pub predictive_back_duration_ms: u32,
    /// Navigation bar mode
    pub navigation_mode: AndroidNavigationMode,
    /// Enable gesture navigation
    pub gesture_navigation: bool,
}

impl Default for AndroidNavigationConfig {
    fn default() -> Self {
        Self {
            handle_back_button: true,
            predictive_back_enabled: true,
            predictive_back_duration_ms: 300,
            navigation_mode: AndroidNavigationMode::Gesture,
            gesture_navigation: true,
        }
    }
}

/// Android navigation bar mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AndroidNavigationMode {
    /// Three-button navigation (back, home, recents)
    ThreeButton,
    /// Two-button navigation (back gesture, home pill)
    TwoButton,
    /// Full gesture navigation
    Gesture,
}

/// Desktop-specific navigation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesktopNavigationConfig {
    /// Show window controls (minimize, maximize, close)
    pub show_window_controls: bool,
    /// Window control position
    pub window_control_position: WindowControlPosition,
    /// Enable menu bar
    pub menu_bar_enabled: bool,
    /// Enable keyboard shortcuts for navigation
    pub keyboard_shortcuts: bool,
    /// Enable tab navigation between windows
    pub tab_navigation: bool,
}

impl Default for DesktopNavigationConfig {
    fn default() -> Self {
        Self {
            show_window_controls: true,
            window_control_position: WindowControlPosition::from_platform(Platform::current()),
            menu_bar_enabled: true,
            keyboard_shortcuts: true,
            tab_navigation: true,
        }
    }
}

/// Position of window controls on desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowControlPosition {
    /// Left side (macOS style)
    Left,
    /// Right side (Windows/Linux style)
    Right,
}

impl WindowControlPosition {
    /// Get the default position for a platform.
    #[must_use]
    pub fn from_platform(platform: Platform) -> Self {
        match platform {
            Platform::MacOS => WindowControlPosition::Left,
            _ => WindowControlPosition::Right,
        }
    }
}

/// Unified navigation configuration that adapts to the current platform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavigationConfig {
    /// iOS-specific settings
    pub ios: IosNavigationConfig,
    /// Android-specific settings
    pub android: AndroidNavigationConfig,
    /// Desktop-specific settings
    pub desktop: DesktopNavigationConfig,
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self {
            ios: IosNavigationConfig::default(),
            android: AndroidNavigationConfig::default(),
            desktop: DesktopNavigationConfig::default(),
        }
    }
}

impl NavigationConfig {
    /// Create a new navigation config with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the active configuration for the current platform.
    pub fn active(&self) -> ActiveNavigationConfig {
        self.for_platform(Platform::current())
    }

    /// Get the configuration for a specific platform.
    pub fn for_platform(&self, platform: Platform) -> ActiveNavigationConfig {
        match platform {
            Platform::IOS => ActiveNavigationConfig::Ios(self.ios.clone()),
            Platform::Android => ActiveNavigationConfig::Android(self.android.clone()),
            Platform::MacOS | Platform::Windows | Platform::Linux => {
                ActiveNavigationConfig::Desktop(self.desktop.clone())
            }
            Platform::Web => ActiveNavigationConfig::Web,
            Platform::Unknown => ActiveNavigationConfig::Web,
        }
    }
}

/// Active navigation configuration for the current platform.
#[derive(Debug, Clone, PartialEq)]
pub enum ActiveNavigationConfig {
    /// iOS navigation
    Ios(IosNavigationConfig),
    /// Android navigation
    Android(AndroidNavigationConfig),
    /// Desktop navigation
    Desktop(DesktopNavigationConfig),
    /// Web navigation (minimal)
    Web,
}

impl ActiveNavigationConfig {
    /// Check if back swipe gesture is supported.
    #[must_use]
    pub fn supports_back_swipe(&self) -> bool {
        matches!(self, ActiveNavigationConfig::Ios(config) if config.back_swipe_enabled)
    }

    /// Check if predictive back is supported.
    #[must_use]
    pub fn supports_predictive_back(&self) -> bool {
        matches!(self, ActiveNavigationConfig::Android(config) if config.predictive_back_enabled)
    }

    /// Check if hardware back button should be handled.
    #[must_use]
    pub fn handles_back_button(&self) -> bool {
        matches!(self, ActiveNavigationConfig::Android(config) if config.handle_back_button)
    }
}

/// Back swipe gesture state for iOS-style navigation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackSwipeState {
    /// Current progress (0.0 = not started, 1.0 = fully swiped)
    pub progress: f32,
    /// Whether the gesture is currently active
    pub is_active: bool,
    /// Starting X position of the gesture
    pub start_x: f32,
    /// Current X position of the gesture
    pub current_x: f32,
    /// Screen width for threshold calculation
    pub screen_width: f32,
}

impl BackSwipeState {
    /// Create a new back swipe state.
    #[must_use]
    pub fn new(screen_width: f32) -> Self {
        Self {
            progress: 0.0,
            is_active: false,
            start_x: 0.0,
            current_x: 0.0,
            screen_width,
        }
    }

    /// Start the back swipe gesture.
    pub fn start(&mut self, x: f32) {
        self.is_active = true;
        self.start_x = x;
        self.current_x = x;
        self.progress = 0.0;
    }

    /// Update the gesture with new position.
    pub fn update(&mut self, x: f32) {
        if self.is_active {
            self.current_x = x;
            let delta = (x - self.start_x).max(0.0);
            self.progress = (delta / self.screen_width).clamp(0.0, 1.0);
        }
    }

    /// End the gesture and return whether it should trigger back action.
    pub fn end(&mut self, threshold: f32) -> bool {
        let should_trigger = self.is_active && self.progress >= threshold;
        self.reset();
        should_trigger
    }

    /// Cancel the gesture.
    pub fn cancel(&mut self) {
        self.reset();
    }

    /// Reset the gesture state.
    fn reset(&mut self) {
        self.is_active = false;
        self.progress = 0.0;
        self.start_x = 0.0;
        self.current_x = 0.0;
    }
}

/// Predictive back animation state for Android.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictiveBackState {
    /// Current progress (0.0 = not started, 1.0 = fully swiped)
    pub progress: f32,
    /// Whether the animation is active
    pub is_active: bool,
    /// Direction of the back gesture
    pub direction: BackDirection,
    /// Scale factor for the preview
    pub scale: f32,
    /// X translation for the preview
    pub translate_x: f32,
}

impl PredictiveBackState {
    /// Create a new predictive back state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            is_active: false,
            direction: BackDirection::Left,
            scale: 1.0,
            translate_x: 0.0,
        }
    }

    /// Update the state based on gesture progress.
    pub fn update(&mut self, progress: f32, direction: BackDirection) {
        self.is_active = progress > 0.0;
        self.progress = progress.clamp(0.0, 1.0);
        self.direction = direction;

        // Calculate scale and translation based on Material Design guidelines
        self.scale = 1.0 - (progress * 0.1); // Scale down to 90%
        self.translate_x = match direction {
            BackDirection::Left => progress * -32.0,
            BackDirection::Right => progress * 32.0,
        };
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.is_active = false;
        self.scale = 1.0;
        self.translate_x = 0.0;
    }
}

impl Default for PredictiveBackState {
    fn default() -> Self {
        Self::new()
    }
}

/// Direction of back gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackDirection {
    /// Swiping from left edge
    Left,
    /// Swiping from right edge
    Right,
}

/// Navigation stack manager for platform-aware navigation.
#[derive(Debug, Clone)]
pub struct NavigationStack<T> {
    /// Stack of navigation entries
    entries: Vec<T>,
    /// Current index in the stack
    current_index: usize,
}

impl<T> NavigationStack<T> {
    /// Create a new navigation stack.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
        }
    }

    /// Push a new entry onto the stack.
    pub fn push(&mut self, entry: T) {
        // Remove any forward history
        self.entries.truncate(self.current_index + 1);
        self.entries.push(entry);
        self.current_index = self.entries.len() - 1;
    }

    /// Pop the current entry and go back.
    pub fn pop(&mut self) -> Option<&T> {
        if self.can_go_back() {
            self.current_index -= 1;
            self.entries.get(self.current_index)
        } else {
            None
        }
    }

    /// Check if we can go back.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        self.current_index > 0
    }

    /// Check if we can go forward.
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        self.current_index < self.entries.len().saturating_sub(1)
    }

    /// Go forward in navigation history.
    pub fn go_forward(&mut self) -> Option<&T> {
        if self.can_go_forward() {
            self.current_index += 1;
            self.entries.get(self.current_index)
        } else {
            None
        }
    }

    /// Get the current entry.
    #[must_use]
    pub fn current(&self) -> Option<&T> {
        self.entries.get(self.current_index)
    }

    /// Get the number of entries in the stack.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the stack is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear the navigation stack.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = 0;
    }
}

impl<T> Default for NavigationStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_navigation_default() {
        let config = IosNavigationConfig::default();
        assert!(config.back_swipe_enabled);
        assert!((config.back_swipe_threshold - 0.4).abs() < f32::EPSILON);
        assert_eq!(config.large_title_mode, LargeTitleMode::Automatic);
    }

    #[test]
    fn test_android_navigation_default() {
        let config = AndroidNavigationConfig::default();
        assert!(config.handle_back_button);
        assert!(config.predictive_back_enabled);
        assert_eq!(config.navigation_mode, AndroidNavigationMode::Gesture);
    }

    #[test]
    fn test_window_control_position() {
        assert_eq!(WindowControlPosition::from_platform(Platform::MacOS), WindowControlPosition::Left);
        assert_eq!(WindowControlPosition::from_platform(Platform::Windows), WindowControlPosition::Right);
        assert_eq!(WindowControlPosition::from_platform(Platform::Linux), WindowControlPosition::Right);
    }

    #[test]
    fn test_back_swipe_state() {
        let mut state = BackSwipeState::new(375.0);
        assert!(!state.is_active);

        state.start(0.0);
        assert!(state.is_active);

        state.update(150.0);
        assert!((state.progress - 0.4).abs() < 0.01);

        let triggered = state.end(0.4);
        assert!(triggered);
        assert!(!state.is_active);
    }

    #[test]
    fn test_back_swipe_not_triggered() {
        let mut state = BackSwipeState::new(375.0);
        state.start(0.0);
        state.update(100.0);
        let triggered = state.end(0.4);
        assert!(!triggered);
    }

    #[test]
    fn test_predictive_back_state() {
        let mut state = PredictiveBackState::new();
        assert!(!state.is_active);

        state.update(0.5, BackDirection::Left);
        assert!(state.is_active);
        assert!((state.scale - 0.95).abs() < 0.01);
        assert!(state.translate_x < 0.0);
    }

    #[test]
    fn test_navigation_stack() {
        let mut stack: NavigationStack<&str> = NavigationStack::new();
        assert!(stack.is_empty());
        assert!(!stack.can_go_back());

        stack.push("page1");
        stack.push("page2");
        stack.push("page3");

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.current(), Some(&"page3"));
        assert!(stack.can_go_back());

        stack.pop();
        assert_eq!(stack.current(), Some(&"page2"));
        assert!(stack.can_go_forward());

        stack.go_forward();
        assert_eq!(stack.current(), Some(&"page3"));
    }

    #[test]
    fn test_navigation_stack_truncates_forward() {
        let mut stack: NavigationStack<i32> = NavigationStack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        stack.pop();
        stack.push(4);

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.current(), Some(&4));
        assert!(!stack.can_go_forward());
    }
}
