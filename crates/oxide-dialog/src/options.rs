//! Dialog configuration options
//!
//! Provides comprehensive configuration for dialog appearance and behavior.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Backdrop style for dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Backdrop {
    /// No backdrop
    None,
    /// Transparent backdrop (captures taps but no visual)
    Transparent,
    /// Dimmed backdrop (semi-transparent dark)
    #[default]
    Dim,
    /// Blurred backdrop (iOS-style)
    Blur,
    /// Custom opacity backdrop
    Custom {
        /// Opacity value (0.0 - 1.0)
        opacity: u8,
    },
}

impl Backdrop {
    /// Create a custom backdrop with specified opacity (0-255 maps to 0.0-1.0)
    pub fn custom(opacity: u8) -> Self {
        Self::Custom { opacity }
    }

    /// Get the opacity value for the backdrop
    pub fn opacity(&self) -> f32 {
        match self {
            Backdrop::None => 0.0,
            Backdrop::Transparent => 0.0,
            Backdrop::Dim => 0.5,
            Backdrop::Blur => 0.3,
            Backdrop::Custom { opacity } => *opacity as f32 / 255.0,
        }
    }

    /// Whether the backdrop should capture taps
    pub fn captures_taps(&self) -> bool {
        !matches!(self, Backdrop::None)
    }

    /// Whether the backdrop should be blurred
    pub fn is_blurred(&self) -> bool {
        matches!(self, Backdrop::Blur)
    }
}

/// Priority level for dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum DialogPriority {
    /// Low priority - can be dismissed by higher priority dialogs
    Low = 0,
    /// Normal priority (default)
    #[default]
    Normal = 1,
    /// High priority - shows above normal dialogs
    High = 2,
    /// System priority - highest, for critical system messages
    System = 3,
}

impl DialogPriority {
    /// Get numeric value for comparison
    pub fn value(&self) -> u8 {
        match self {
            DialogPriority::Low => 0,
            DialogPriority::Normal => 1,
            DialogPriority::High => 2,
            DialogPriority::System => 3,
        }
    }
}

/// Dismiss behavior configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DismissBehavior {
    /// Whether tapping the backdrop dismisses the dialog
    pub on_backdrop_tap: bool,
    /// Whether pressing Escape dismisses the dialog
    pub on_escape_key: bool,
    /// Whether the dialog can be dismissed at all (for critical dialogs)
    pub allow_dismiss: bool,
    /// Auto-dismiss after duration (None = no auto-dismiss)
    pub auto_dismiss: Option<Duration>,
}

impl DismissBehavior {
    /// Create a dismissible dialog (default behavior)
    pub fn dismissible() -> Self {
        Self {
            on_backdrop_tap: true,
            on_escape_key: true,
            allow_dismiss: true,
            auto_dismiss: None,
        }
    }

    /// Create a non-dismissible dialog (for critical messages)
    pub fn prevent_dismiss() -> Self {
        Self {
            on_backdrop_tap: false,
            on_escape_key: false,
            allow_dismiss: false,
            auto_dismiss: None,
        }
    }

    /// Create a toast-like dialog that auto-dismisses
    pub fn auto_dismiss(duration: Duration) -> Self {
        Self {
            on_backdrop_tap: true,
            on_escape_key: true,
            allow_dismiss: true,
            auto_dismiss: Some(duration),
        }
    }

    /// Set backdrop tap dismissal
    pub fn with_backdrop_tap(mut self, enabled: bool) -> Self {
        self.on_backdrop_tap = enabled;
        self
    }

    /// Set escape key dismissal
    pub fn with_escape_key(mut self, enabled: bool) -> Self {
        self.on_escape_key = enabled;
        self
    }

    /// Set whether dismissal is allowed
    pub fn with_allow_dismiss(mut self, enabled: bool) -> Self {
        self.allow_dismiss = enabled;
        self
    }

    /// Set auto-dismiss duration
    pub fn with_auto_dismiss(mut self, duration: Option<Duration>) -> Self {
        self.auto_dismiss = duration;
        self
    }
}

/// Dialog size configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DialogSize {
    /// Small dialog (280px width)
    Small,
    /// Medium dialog (360px width)
    Medium,
    /// Large dialog (480px width)
    Large,
    /// Full width (with margins)
    FullWidth,
    /// Full screen
    FullScreen,
    /// Custom size
    Custom {
        /// Width in logical pixels
        width: f32,
        /// Height in logical pixels (None = auto)
        height: Option<f32>,
    },
}

impl Default for DialogSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl DialogSize {
    /// Create a custom sized dialog
    pub fn custom(width: f32, height: Option<f32>) -> Self {
        Self::Custom { width, height }
    }

    /// Get the width in logical pixels
    pub fn width(&self) -> Option<f32> {
        match self {
            DialogSize::Small => Some(280.0),
            DialogSize::Medium => Some(360.0),
            DialogSize::Large => Some(480.0),
            DialogSize::FullWidth => None,
            DialogSize::FullScreen => None,
            DialogSize::Custom { width, .. } => Some(*width),
        }
    }

    /// Get the height in logical pixels (None = auto)
    pub fn height(&self) -> Option<f32> {
        match self {
            DialogSize::Custom { height, .. } => *height,
            DialogSize::FullScreen => None,
            _ => None,
        }
    }

    /// Whether the dialog should be full screen
    pub fn is_full_screen(&self) -> bool {
        matches!(self, DialogSize::FullScreen)
    }
}

/// Dialog position configuration
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum DialogPosition {
    /// Centered on screen (default)
    #[default]
    Center,
    /// Top of screen
    Top,
    /// Bottom of screen
    Bottom,
    /// Custom position
    Custom {
        /// X offset from center
        x: f32,
        /// Y offset from center
        y: f32,
    },
}

impl DialogPosition {
    /// Create a custom position
    pub fn custom(x: f32, y: f32) -> Self {
        Self::Custom { x, y }
    }
}

/// Complete dialog options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogOptions {
    /// Backdrop style
    pub backdrop: Backdrop,
    /// Dialog priority
    pub priority: DialogPriority,
    /// Dismiss behavior
    pub dismiss: DismissBehavior,
    /// Dialog size
    pub size: DialogSize,
    /// Dialog position
    pub position: DialogPosition,
    /// Border radius in logical pixels
    pub border_radius: f32,
    /// Whether to show a close button
    pub show_close_button: bool,
    /// Z-index offset (for stacking)
    pub z_index_offset: i32,
    /// Whether to trap focus within dialog
    pub trap_focus: bool,
    /// Whether to restore focus on dismiss
    pub restore_focus: bool,
    /// Custom class name for styling
    pub class_name: Option<String>,
}

impl Default for DialogOptions {
    fn default() -> Self {
        Self {
            backdrop: Backdrop::default(),
            priority: DialogPriority::default(),
            dismiss: DismissBehavior::dismissible(),
            size: DialogSize::default(),
            position: DialogPosition::default(),
            border_radius: 12.0,
            show_close_button: false,
            z_index_offset: 0,
            trap_focus: true,
            restore_focus: true,
            class_name: None,
        }
    }
}

impl DialogOptions {
    /// Create new default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set backdrop style
    pub fn backdrop(mut self, backdrop: Backdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    /// Set dialog priority
    pub fn priority(mut self, priority: DialogPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set dismiss behavior
    pub fn dismiss(mut self, dismiss: DismissBehavior) -> Self {
        self.dismiss = dismiss;
        self
    }

    /// Set dialog size
    pub fn size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }

    /// Set dialog position
    pub fn position(mut self, position: DialogPosition) -> Self {
        self.position = position;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Show close button
    pub fn with_close_button(mut self) -> Self {
        self.show_close_button = true;
        self
    }

    /// Set z-index offset
    pub fn z_index_offset(mut self, offset: i32) -> Self {
        self.z_index_offset = offset;
        self
    }

    /// Set focus trapping
    pub fn trap_focus(mut self, trap: bool) -> Self {
        self.trap_focus = trap;
        self
    }

    /// Set focus restoration
    pub fn restore_focus(mut self, restore: bool) -> Self {
        self.restore_focus = restore;
        self
    }

    /// Set custom class name
    pub fn class_name(mut self, name: impl Into<String>) -> Self {
        self.class_name = Some(name.into());
        self
    }

    /// Create options for a modal dialog (non-dismissible backdrop tap)
    pub fn modal() -> Self {
        Self::default()
            .dismiss(DismissBehavior::dismissible().with_backdrop_tap(false))
    }

    /// Create options for an alert dialog
    pub fn alert() -> Self {
        Self::default()
            .size(DialogSize::Small)
    }

    /// Create options for a full screen dialog
    pub fn full_screen() -> Self {
        Self::default()
            .size(DialogSize::FullScreen)
            .backdrop(Backdrop::None)
            .border_radius(0.0)
    }

    /// Create options for a system critical dialog
    pub fn system_critical() -> Self {
        Self::default()
            .priority(DialogPriority::System)
            .dismiss(DismissBehavior::prevent_dismiss())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backdrop_opacity() {
        assert_eq!(Backdrop::None.opacity(), 0.0);
        assert_eq!(Backdrop::Transparent.opacity(), 0.0);
        assert_eq!(Backdrop::Dim.opacity(), 0.5);
        assert_eq!(Backdrop::Blur.opacity(), 0.3);
        assert!((Backdrop::custom(128).opacity() - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_backdrop_captures_taps() {
        assert!(!Backdrop::None.captures_taps());
        assert!(Backdrop::Transparent.captures_taps());
        assert!(Backdrop::Dim.captures_taps());
        assert!(Backdrop::Blur.captures_taps());
    }

    #[test]
    fn test_dialog_priority_ordering() {
        assert!(DialogPriority::Low < DialogPriority::Normal);
        assert!(DialogPriority::Normal < DialogPriority::High);
        assert!(DialogPriority::High < DialogPriority::System);
    }

    #[test]
    fn test_dismiss_behavior_presets() {
        let dismissible = DismissBehavior::dismissible();
        assert!(dismissible.on_backdrop_tap);
        assert!(dismissible.on_escape_key);
        assert!(dismissible.allow_dismiss);
        assert!(dismissible.auto_dismiss.is_none());

        let prevent = DismissBehavior::prevent_dismiss();
        assert!(!prevent.on_backdrop_tap);
        assert!(!prevent.on_escape_key);
        assert!(!prevent.allow_dismiss);

        let auto = DismissBehavior::auto_dismiss(Duration::from_secs(3));
        assert!(auto.auto_dismiss.is_some());
    }

    #[test]
    fn test_dialog_size_widths() {
        assert_eq!(DialogSize::Small.width(), Some(280.0));
        assert_eq!(DialogSize::Medium.width(), Some(360.0));
        assert_eq!(DialogSize::Large.width(), Some(480.0));
        assert_eq!(DialogSize::FullWidth.width(), None);
        assert_eq!(DialogSize::custom(500.0, None).width(), Some(500.0));
    }

    #[test]
    fn test_dialog_options_builder() {
        let opts = DialogOptions::new()
            .backdrop(Backdrop::Blur)
            .priority(DialogPriority::High)
            .size(DialogSize::Large)
            .border_radius(16.0)
            .with_close_button();

        assert!(matches!(opts.backdrop, Backdrop::Blur));
        assert!(matches!(opts.priority, DialogPriority::High));
        assert!(matches!(opts.size, DialogSize::Large));
        assert_eq!(opts.border_radius, 16.0);
        assert!(opts.show_close_button);
    }

    #[test]
    fn test_dialog_options_presets() {
        let modal = DialogOptions::modal();
        assert!(!modal.dismiss.on_backdrop_tap);

        let alert = DialogOptions::alert();
        assert!(matches!(alert.size, DialogSize::Small));

        let full = DialogOptions::full_screen();
        assert!(matches!(full.size, DialogSize::FullScreen));
        assert!(matches!(full.backdrop, Backdrop::None));
        assert_eq!(full.border_radius, 0.0);

        let critical = DialogOptions::system_critical();
        assert!(matches!(critical.priority, DialogPriority::System));
        assert!(!critical.dismiss.allow_dismiss);
    }
}
