//! Toast options and configuration.
//!
//! Provides configuration types for toast duration, position, and other options.

use serde::{Deserialize, Serialize};

use crate::animation::ToastAnimation;
use crate::toast::{ToastAction, ToastStyle};

/// Toast display duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToastDuration {
    /// Short duration (2 seconds)
    Short,
    /// Medium duration (4 seconds) - default
    #[default]
    Medium,
    /// Long duration (6 seconds)
    Long,
    /// Toast persists until manually dismissed
    Persistent,
    /// Custom duration in milliseconds
    Custom(u64),
}

impl ToastDuration {
    /// Returns the duration in milliseconds, or None for persistent.
    pub fn as_millis(&self) -> Option<u64> {
        match self {
            ToastDuration::Short => Some(2000),
            ToastDuration::Medium => Some(4000),
            ToastDuration::Long => Some(6000),
            ToastDuration::Persistent => None,
            ToastDuration::Custom(ms) => Some(*ms),
        }
    }

    /// Returns the duration in seconds, or None for persistent.
    pub fn as_secs(&self) -> Option<f32> {
        self.as_millis().map(|ms| ms as f32 / 1000.0)
    }

    /// Creates a custom duration from milliseconds.
    pub fn millis(ms: u64) -> Self {
        ToastDuration::Custom(ms)
    }

    /// Creates a custom duration from seconds.
    pub fn secs(secs: f32) -> Self {
        ToastDuration::Custom((secs * 1000.0) as u64)
    }
}

/// Toast screen position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ToastPosition {
    /// Top center of the screen
    Top,
    /// Top left corner
    TopLeft,
    /// Top right corner - default
    #[default]
    TopRight,
    /// Bottom center of the screen
    Bottom,
    /// Bottom left corner
    BottomLeft,
    /// Bottom right corner
    BottomRight,
}

impl ToastPosition {
    /// Returns whether this position is at the top of the screen.
    pub fn is_top(&self) -> bool {
        matches!(
            self,
            ToastPosition::Top | ToastPosition::TopLeft | ToastPosition::TopRight
        )
    }

    /// Returns whether this position is at the bottom of the screen.
    pub fn is_bottom(&self) -> bool {
        !self.is_top()
    }

    /// Returns whether this position is on the left side.
    pub fn is_left(&self) -> bool {
        matches!(self, ToastPosition::TopLeft | ToastPosition::BottomLeft)
    }

    /// Returns whether this position is on the right side.
    pub fn is_right(&self) -> bool {
        matches!(self, ToastPosition::TopRight | ToastPosition::BottomRight)
    }

    /// Returns whether this position is centered horizontally.
    pub fn is_center(&self) -> bool {
        matches!(self, ToastPosition::Top | ToastPosition::Bottom)
    }

    /// Returns the CSS-like anchor point.
    pub fn anchor(&self) -> (VerticalAnchor, HorizontalAnchor) {
        let vertical = if self.is_top() {
            VerticalAnchor::Top
        } else {
            VerticalAnchor::Bottom
        };

        let horizontal = if self.is_left() {
            HorizontalAnchor::Left
        } else if self.is_right() {
            HorizontalAnchor::Right
        } else {
            HorizontalAnchor::Center
        };

        (vertical, horizontal)
    }

    /// Returns the default slide direction for entrance animation.
    pub fn default_slide_direction(&self) -> SlideDirection {
        match self {
            ToastPosition::Top | ToastPosition::TopLeft | ToastPosition::TopRight => {
                SlideDirection::Down
            }
            ToastPosition::Bottom | ToastPosition::BottomLeft | ToastPosition::BottomRight => {
                SlideDirection::Up
            }
        }
    }
}

/// Vertical anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAnchor {
    Top,
    Bottom,
}

/// Horizontal anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAnchor {
    Left,
    Center,
    Right,
}

/// Direction for slide animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SlideDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

/// Toast stacking order - how new toasts are added relative to existing ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ToastStackOrder {
    /// Newest toasts appear on top (closer to screen edge)
    #[default]
    NewestOnTop,
    /// Newest toasts appear on bottom (farther from screen edge)
    NewestOnBottom,
}

/// Configuration options for creating toasts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToastOptions {
    /// Display duration
    pub duration: Option<ToastDuration>,
    /// Screen position
    pub position: Option<ToastPosition>,
    /// Whether the toast can be dismissed
    pub dismissible: Option<bool>,
    /// Action button
    pub action: Option<ToastAction>,
    /// Custom icon
    pub icon: Option<String>,
    /// Whether to show an icon
    pub show_icon: Option<bool>,
    /// Custom styling
    pub style: Option<ToastStyle>,
    /// Entrance animation
    pub entrance_animation: Option<ToastAnimation>,
    /// Exit animation
    pub exit_animation: Option<ToastAnimation>,
    /// Whether to pause timer on hover
    pub pause_on_hover: Option<bool>,
    /// Whether to enable swipe-to-dismiss
    pub swipe_to_dismiss: Option<bool>,
}

impl ToastOptions {
    /// Creates a new empty options builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the display duration.
    pub fn duration(mut self, duration: ToastDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Sets the screen position.
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets whether the toast is dismissible.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = Some(dismissible);
        self
    }

    /// Adds an action button.
    pub fn action(mut self, label: impl Into<String>, action_id: impl Into<String>) -> Self {
        self.action = Some(ToastAction::new(label, action_id));
        self
    }

    /// Sets a custom icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets whether to show an icon.
    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = Some(show);
        self
    }

    /// Sets custom styling.
    pub fn style(mut self, style: ToastStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Sets the entrance animation.
    pub fn entrance_animation(mut self, animation: ToastAnimation) -> Self {
        self.entrance_animation = Some(animation);
        self
    }

    /// Sets the exit animation.
    pub fn exit_animation(mut self, animation: ToastAnimation) -> Self {
        self.exit_animation = Some(animation);
        self
    }

    /// Sets whether to pause the timer on hover.
    pub fn pause_on_hover(mut self, pause: bool) -> Self {
        self.pause_on_hover = Some(pause);
        self
    }

    /// Sets whether swipe-to-dismiss is enabled.
    pub fn swipe_to_dismiss(mut self, enabled: bool) -> Self {
        self.swipe_to_dismiss = Some(enabled);
        self
    }
}

/// Global default options for all toasts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastDefaults {
    /// Default duration for all toast types
    pub duration: ToastDuration,
    /// Default position for all toasts
    pub position: ToastPosition,
    /// Default dismissible setting
    pub dismissible: bool,
    /// Default show_icon setting
    pub show_icon: bool,
    /// Default entrance animation
    pub entrance_animation: ToastAnimation,
    /// Default exit animation
    pub exit_animation: ToastAnimation,
    /// Default pause_on_hover setting
    pub pause_on_hover: bool,
    /// Default swipe_to_dismiss setting
    pub swipe_to_dismiss: bool,
}

impl Default for ToastDefaults {
    fn default() -> Self {
        Self {
            duration: ToastDuration::Medium,
            position: ToastPosition::TopRight,
            dismissible: true,
            show_icon: true,
            entrance_animation: ToastAnimation::default(),
            exit_animation: ToastAnimation::default(),
            pause_on_hover: true,
            swipe_to_dismiss: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_as_millis() {
        assert_eq!(ToastDuration::Short.as_millis(), Some(2000));
        assert_eq!(ToastDuration::Medium.as_millis(), Some(4000));
        assert_eq!(ToastDuration::Long.as_millis(), Some(6000));
        assert_eq!(ToastDuration::Persistent.as_millis(), None);
        assert_eq!(ToastDuration::Custom(5000).as_millis(), Some(5000));
    }

    #[test]
    fn test_duration_as_secs() {
        assert_eq!(ToastDuration::Short.as_secs(), Some(2.0));
        assert_eq!(ToastDuration::Medium.as_secs(), Some(4.0));
        assert_eq!(ToastDuration::Persistent.as_secs(), None);
    }

    #[test]
    fn test_duration_constructors() {
        assert_eq!(ToastDuration::millis(3000), ToastDuration::Custom(3000));
        assert_eq!(ToastDuration::secs(3.5), ToastDuration::Custom(3500));
    }

    #[test]
    fn test_position_is_top() {
        assert!(ToastPosition::Top.is_top());
        assert!(ToastPosition::TopLeft.is_top());
        assert!(ToastPosition::TopRight.is_top());
        assert!(!ToastPosition::Bottom.is_top());
        assert!(!ToastPosition::BottomLeft.is_top());
        assert!(!ToastPosition::BottomRight.is_top());
    }

    #[test]
    fn test_position_is_bottom() {
        assert!(!ToastPosition::Top.is_bottom());
        assert!(ToastPosition::Bottom.is_bottom());
        assert!(ToastPosition::BottomLeft.is_bottom());
        assert!(ToastPosition::BottomRight.is_bottom());
    }

    #[test]
    fn test_position_is_left_right_center() {
        assert!(ToastPosition::TopLeft.is_left());
        assert!(ToastPosition::BottomLeft.is_left());
        assert!(!ToastPosition::TopRight.is_left());

        assert!(ToastPosition::TopRight.is_right());
        assert!(ToastPosition::BottomRight.is_right());
        assert!(!ToastPosition::TopLeft.is_right());

        assert!(ToastPosition::Top.is_center());
        assert!(ToastPosition::Bottom.is_center());
        assert!(!ToastPosition::TopLeft.is_center());
    }

    #[test]
    fn test_position_anchor() {
        assert_eq!(
            ToastPosition::TopLeft.anchor(),
            (VerticalAnchor::Top, HorizontalAnchor::Left)
        );
        assert_eq!(
            ToastPosition::TopRight.anchor(),
            (VerticalAnchor::Top, HorizontalAnchor::Right)
        );
        assert_eq!(
            ToastPosition::Bottom.anchor(),
            (VerticalAnchor::Bottom, HorizontalAnchor::Center)
        );
    }

    #[test]
    fn test_position_default_slide_direction() {
        assert_eq!(
            ToastPosition::Top.default_slide_direction(),
            SlideDirection::Down
        );
        assert_eq!(
            ToastPosition::TopRight.default_slide_direction(),
            SlideDirection::Down
        );
        assert_eq!(
            ToastPosition::Bottom.default_slide_direction(),
            SlideDirection::Up
        );
        assert_eq!(
            ToastPosition::BottomLeft.default_slide_direction(),
            SlideDirection::Up
        );
    }

    #[test]
    fn test_toast_options_builder() {
        let options = ToastOptions::new()
            .duration(ToastDuration::Long)
            .position(ToastPosition::BottomRight)
            .dismissible(false)
            .action("Retry", "retry_action")
            .pause_on_hover(false);

        assert_eq!(options.duration, Some(ToastDuration::Long));
        assert_eq!(options.position, Some(ToastPosition::BottomRight));
        assert_eq!(options.dismissible, Some(false));
        assert!(options.action.is_some());
        assert_eq!(options.pause_on_hover, Some(false));
    }

    #[test]
    fn test_toast_defaults() {
        let defaults = ToastDefaults::default();
        assert_eq!(defaults.duration, ToastDuration::Medium);
        assert_eq!(defaults.position, ToastPosition::TopRight);
        assert!(defaults.dismissible);
        assert!(defaults.show_icon);
        assert!(defaults.pause_on_hover);
        assert!(defaults.swipe_to_dismiss);
    }
}
