//! Toast struct and toast types.
//!
//! Provides the core toast data structure and type definitions.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::animation::ToastAnimation;
use crate::options::{ToastDuration, ToastOptions, ToastPosition};

/// Toast notification type determining visual styling and default icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToastType {
    /// Informational toast (blue/neutral styling)
    #[default]
    Info,
    /// Success toast (green styling)
    Success,
    /// Warning toast (yellow/orange styling)
    Warning,
    /// Error toast (red styling)
    Error,
    /// Custom styled toast
    Custom,
}

impl ToastType {
    /// Returns the default icon for this toast type.
    pub fn default_icon(&self) -> Option<&'static str> {
        match self {
            ToastType::Info => Some("info-circle"),
            ToastType::Success => Some("check-circle"),
            ToastType::Warning => Some("warning-triangle"),
            ToastType::Error => Some("x-circle"),
            ToastType::Custom => None,
        }
    }

    /// Returns the ARIA role for this toast type.
    pub fn aria_role(&self) -> &'static str {
        match self {
            ToastType::Error | ToastType::Warning => "alert",
            _ => "status",
        }
    }

    /// Returns the ARIA live region politeness level.
    pub fn aria_live(&self) -> &'static str {
        match self {
            ToastType::Error | ToastType::Warning => "assertive",
            _ => "polite",
        }
    }

    /// Returns the default color token name for this toast type.
    pub fn color_token(&self) -> &'static str {
        match self {
            ToastType::Info => "toast.info",
            ToastType::Success => "toast.success",
            ToastType::Warning => "toast.warning",
            ToastType::Error => "toast.error",
            ToastType::Custom => "toast.custom",
        }
    }
}

/// Toast action button configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastAction {
    /// Action button label text
    pub label: String,
    /// Action identifier for handling
    pub action_id: String,
    /// Whether clicking the action dismisses the toast
    pub dismiss_on_click: bool,
}

impl ToastAction {
    /// Creates a new toast action.
    pub fn new(label: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action_id: action_id.into(),
            dismiss_on_click: true,
        }
    }

    /// Sets whether the toast is dismissed when the action is clicked.
    pub fn dismiss_on_click(mut self, dismiss: bool) -> Self {
        self.dismiss_on_click = dismiss;
        self
    }
}

/// Custom styling for a toast.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToastStyle {
    /// Background color (CSS color string or token reference)
    pub background: Option<String>,
    /// Text color
    pub text_color: Option<String>,
    /// Border color
    pub border_color: Option<String>,
    /// Border radius in pixels
    pub border_radius: Option<f32>,
    /// Padding in pixels
    pub padding: Option<f32>,
    /// Shadow configuration
    pub shadow: Option<String>,
    /// Custom icon name or path
    pub icon: Option<String>,
    /// Icon color
    pub icon_color: Option<String>,
}

/// Current state of a toast in the display lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToastState {
    /// Toast is queued but not yet visible
    #[default]
    Queued,
    /// Toast is playing entrance animation
    Entering,
    /// Toast is fully visible
    Visible,
    /// Toast timer is paused (e.g., on hover)
    Paused,
    /// Toast is playing exit animation
    Exiting,
    /// Toast has been dismissed and removed
    Dismissed,
}

/// A toast notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toast {
    /// Unique identifier for this toast
    pub id: Uuid,
    /// Toast type determining styling
    pub toast_type: ToastType,
    /// Main message content
    pub message: String,
    /// Optional title/heading
    pub title: Option<String>,
    /// Display duration
    pub duration: ToastDuration,
    /// Screen position
    pub position: ToastPosition,
    /// Whether the toast can be dismissed by the user
    pub dismissible: bool,
    /// Optional action button
    pub action: Option<ToastAction>,
    /// Custom icon override (None uses type default)
    pub icon: Option<String>,
    /// Whether to show an icon
    pub show_icon: bool,
    /// Custom styling
    pub style: Option<ToastStyle>,
    /// Entrance animation
    pub entrance_animation: ToastAnimation,
    /// Exit animation
    pub exit_animation: ToastAnimation,
    /// Current display state
    pub state: ToastState,
    /// Timestamp when toast was created
    pub created_at: u64,
    /// Timestamp when toast became visible
    pub shown_at: Option<u64>,
    /// Progress through duration (0.0 to 1.0)
    pub progress: f32,
    /// Whether to pause timer on hover
    pub pause_on_hover: bool,
    /// Whether to enable swipe-to-dismiss
    pub swipe_to_dismiss: bool,
}

impl Toast {
    /// Creates a new toast with the given type and message.
    pub fn new(toast_type: ToastType, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            toast_type,
            message: message.into(),
            title: None,
            duration: ToastDuration::default(),
            position: ToastPosition::default(),
            dismissible: true,
            action: None,
            icon: None,
            show_icon: true,
            style: None,
            entrance_animation: ToastAnimation::default(),
            exit_animation: ToastAnimation::default(),
            state: ToastState::Queued,
            created_at: current_timestamp(),
            shown_at: None,
            progress: 0.0,
            pause_on_hover: true,
            swipe_to_dismiss: true,
        }
    }

    /// Creates an info toast.
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(ToastType::Info, message)
    }

    /// Creates a success toast.
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(ToastType::Success, message)
    }

    /// Creates a warning toast.
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(ToastType::Warning, message)
    }

    /// Creates an error toast.
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(ToastType::Error, message)
    }

    /// Creates a custom styled toast.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::new(ToastType::Custom, message)
    }

    /// Sets the toast title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the display duration.
    pub fn duration(mut self, duration: ToastDuration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the screen position.
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets whether the toast is dismissible.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Adds an action button.
    pub fn action(mut self, label: impl Into<String>, action_id: impl Into<String>) -> Self {
        self.action = Some(ToastAction::new(label, action_id));
        self
    }

    /// Adds an action button with full configuration.
    pub fn with_action(mut self, action: ToastAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Sets a custom icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets whether to show an icon.
    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    /// Applies custom styling.
    pub fn style(mut self, style: ToastStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Sets the entrance animation.
    pub fn entrance_animation(mut self, animation: ToastAnimation) -> Self {
        self.entrance_animation = animation;
        self
    }

    /// Sets the exit animation.
    pub fn exit_animation(mut self, animation: ToastAnimation) -> Self {
        self.exit_animation = animation;
        self
    }

    /// Sets whether to pause the timer on hover.
    pub fn pause_on_hover(mut self, pause: bool) -> Self {
        self.pause_on_hover = pause;
        self
    }

    /// Sets whether swipe-to-dismiss is enabled.
    pub fn swipe_to_dismiss(mut self, enabled: bool) -> Self {
        self.swipe_to_dismiss = enabled;
        self
    }

    /// Applies toast options.
    pub fn with_options(mut self, options: ToastOptions) -> Self {
        if let Some(duration) = options.duration {
            self.duration = duration;
        }
        if let Some(position) = options.position {
            self.position = position;
        }
        if let Some(dismissible) = options.dismissible {
            self.dismissible = dismissible;
        }
        if let Some(action) = options.action {
            self.action = Some(action);
        }
        if let Some(icon) = options.icon {
            self.icon = Some(icon);
        }
        if let Some(show_icon) = options.show_icon {
            self.show_icon = show_icon;
        }
        if let Some(style) = options.style {
            self.style = Some(style);
        }
        if let Some(entrance) = options.entrance_animation {
            self.entrance_animation = entrance;
        }
        if let Some(exit) = options.exit_animation {
            self.exit_animation = exit;
        }
        if let Some(pause) = options.pause_on_hover {
            self.pause_on_hover = pause;
        }
        if let Some(swipe) = options.swipe_to_dismiss {
            self.swipe_to_dismiss = swipe;
        }
        self
    }

    /// Returns the effective icon for this toast.
    pub fn effective_icon(&self) -> Option<&str> {
        if !self.show_icon {
            return None;
        }
        self.icon
            .as_deref()
            .or_else(|| self.toast_type.default_icon())
    }

    /// Returns the remaining duration in milliseconds.
    pub fn remaining_ms(&self) -> Option<u64> {
        let total_ms = self.duration.as_millis()?;
        let elapsed = (self.progress * total_ms as f32) as u64;
        Some(total_ms.saturating_sub(elapsed))
    }

    /// Checks if the toast should auto-dismiss.
    pub fn should_auto_dismiss(&self) -> bool {
        !matches!(self.duration, ToastDuration::Persistent)
    }

    /// Marks the toast as entering.
    pub fn enter(&mut self) {
        self.state = ToastState::Entering;
    }

    /// Marks the toast as visible.
    pub fn show(&mut self) {
        self.state = ToastState::Visible;
        self.shown_at = Some(current_timestamp());
    }

    /// Pauses the toast timer.
    pub fn pause(&mut self) {
        if self.state == ToastState::Visible {
            self.state = ToastState::Paused;
        }
    }

    /// Resumes the toast timer.
    pub fn resume(&mut self) {
        if self.state == ToastState::Paused {
            self.state = ToastState::Visible;
        }
    }

    /// Marks the toast as exiting.
    pub fn exit(&mut self) {
        self.state = ToastState::Exiting;
    }

    /// Marks the toast as dismissed.
    pub fn dismiss(&mut self) {
        self.state = ToastState::Dismissed;
    }

    /// Checks if the toast is currently visible or animating.
    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            ToastState::Entering | ToastState::Visible | ToastState::Paused | ToastState::Exiting
        )
    }
}

/// Returns the current timestamp in milliseconds.
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_type_default_icon() {
        assert_eq!(ToastType::Info.default_icon(), Some("info-circle"));
        assert_eq!(ToastType::Success.default_icon(), Some("check-circle"));
        assert_eq!(ToastType::Warning.default_icon(), Some("warning-triangle"));
        assert_eq!(ToastType::Error.default_icon(), Some("x-circle"));
        assert_eq!(ToastType::Custom.default_icon(), None);
    }

    #[test]
    fn test_toast_type_aria_role() {
        assert_eq!(ToastType::Info.aria_role(), "status");
        assert_eq!(ToastType::Success.aria_role(), "status");
        assert_eq!(ToastType::Warning.aria_role(), "alert");
        assert_eq!(ToastType::Error.aria_role(), "alert");
    }

    #[test]
    fn test_toast_type_aria_live() {
        assert_eq!(ToastType::Info.aria_live(), "polite");
        assert_eq!(ToastType::Success.aria_live(), "polite");
        assert_eq!(ToastType::Warning.aria_live(), "assertive");
        assert_eq!(ToastType::Error.aria_live(), "assertive");
    }

    #[test]
    fn test_toast_creation() {
        let toast = Toast::info("Test message");
        assert_eq!(toast.toast_type, ToastType::Info);
        assert_eq!(toast.message, "Test message");
        assert_eq!(toast.state, ToastState::Queued);
        assert!(toast.dismissible);
    }

    #[test]
    fn test_toast_builder_pattern() {
        let toast = Toast::success("Saved!")
            .title("Success")
            .duration(ToastDuration::Long)
            .position(ToastPosition::TopRight)
            .dismissible(false)
            .action("Undo", "undo_action");

        assert_eq!(toast.toast_type, ToastType::Success);
        assert_eq!(toast.title, Some("Success".to_string()));
        assert_eq!(toast.duration, ToastDuration::Long);
        assert_eq!(toast.position, ToastPosition::TopRight);
        assert!(!toast.dismissible);
        assert!(toast.action.is_some());
    }

    #[test]
    fn test_toast_effective_icon() {
        let toast = Toast::info("Test");
        assert_eq!(toast.effective_icon(), Some("info-circle"));

        let toast = Toast::info("Test").icon("custom-icon");
        assert_eq!(toast.effective_icon(), Some("custom-icon"));

        let toast = Toast::info("Test").show_icon(false);
        assert_eq!(toast.effective_icon(), None);
    }

    #[test]
    fn test_toast_state_transitions() {
        let mut toast = Toast::info("Test");
        assert_eq!(toast.state, ToastState::Queued);

        toast.enter();
        assert_eq!(toast.state, ToastState::Entering);

        toast.show();
        assert_eq!(toast.state, ToastState::Visible);
        assert!(toast.shown_at.is_some());

        toast.pause();
        assert_eq!(toast.state, ToastState::Paused);

        toast.resume();
        assert_eq!(toast.state, ToastState::Visible);

        toast.exit();
        assert_eq!(toast.state, ToastState::Exiting);

        toast.dismiss();
        assert_eq!(toast.state, ToastState::Dismissed);
    }

    #[test]
    fn test_toast_is_active() {
        let mut toast = Toast::info("Test");
        assert!(!toast.is_active()); // Queued

        toast.enter();
        assert!(toast.is_active()); // Entering

        toast.show();
        assert!(toast.is_active()); // Visible

        toast.pause();
        assert!(toast.is_active()); // Paused

        toast.exit();
        assert!(toast.is_active()); // Exiting

        toast.dismiss();
        assert!(!toast.is_active()); // Dismissed
    }

    #[test]
    fn test_toast_action() {
        let action = ToastAction::new("Undo", "undo_action").dismiss_on_click(false);
        assert_eq!(action.label, "Undo");
        assert_eq!(action.action_id, "undo_action");
        assert!(!action.dismiss_on_click);
    }

    #[test]
    fn test_toast_remaining_duration() {
        let toast = Toast::info("Test").duration(ToastDuration::Short);
        assert_eq!(toast.remaining_ms(), Some(2000));

        let mut toast = toast;
        toast.progress = 0.5;
        assert_eq!(toast.remaining_ms(), Some(1000));

        let toast = Toast::info("Test").duration(ToastDuration::Persistent);
        assert_eq!(toast.remaining_ms(), None);
    }

    #[test]
    fn test_toast_should_auto_dismiss() {
        let toast = Toast::info("Test").duration(ToastDuration::Short);
        assert!(toast.should_auto_dismiss());

        let toast = Toast::info("Test").duration(ToastDuration::Persistent);
        assert!(!toast.should_auto_dismiss());
    }

    #[test]
    fn test_toast_custom_style() {
        let style = ToastStyle {
            background: Some("#333".to_string()),
            text_color: Some("#fff".to_string()),
            border_radius: Some(8.0),
            ..Default::default()
        };

        let toast = Toast::custom("Custom").style(style.clone());
        assert!(toast.style.is_some());
        let s = toast.style.unwrap();
        assert_eq!(s.background, Some("#333".to_string()));
        assert_eq!(s.text_color, Some("#fff".to_string()));
        assert_eq!(s.border_radius, Some(8.0));
    }
}
