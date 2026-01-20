//! Core dialog types and traits
//!
//! Provides the foundational dialog structures and enums used throughout the system.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::options::{DialogOptions, DialogPriority};

/// Unique identifier for a dialog instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DialogId(Uuid);

impl DialogId {
    /// Create a new unique dialog ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for DialogId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DialogId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dialog-{}", self.0)
    }
}

/// Current state of a dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DialogState {
    /// Dialog is pending (in queue, not yet shown)
    #[default]
    Pending,
    /// Dialog is currently animating in
    Entering,
    /// Dialog is fully visible and interactive
    Visible,
    /// Dialog is currently animating out
    Exiting,
    /// Dialog has been dismissed
    Dismissed,
}

impl DialogState {
    /// Whether the dialog is currently visible (including animating)
    pub fn is_visible(&self) -> bool {
        matches!(self, DialogState::Entering | DialogState::Visible | DialogState::Exiting)
    }

    /// Whether the dialog is fully interactive
    pub fn is_interactive(&self) -> bool {
        matches!(self, DialogState::Visible)
    }

    /// Whether the dialog has been dismissed
    pub fn is_dismissed(&self) -> bool {
        matches!(self, DialogState::Dismissed)
    }
}

/// Type of dialog for semantics and default styling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogType {
    /// Simple alert with message and action
    Alert,
    /// Confirmation dialog with yes/no or ok/cancel
    Confirm,
    /// Prompt dialog with text input
    Prompt,
    /// Custom content dialog
    Custom,
    /// Full screen dialog
    FullScreen,
    /// Bottom sheet
    BottomSheet,
    /// Side sheet (left or right)
    SideSheet,
    /// Popover anchored to element
    Popover,
    /// Tooltip
    Tooltip,
}

impl DialogType {
    /// Get default priority for this dialog type
    pub fn default_priority(&self) -> DialogPriority {
        match self {
            DialogType::Alert => DialogPriority::Normal,
            DialogType::Confirm => DialogPriority::Normal,
            DialogType::Prompt => DialogPriority::Normal,
            DialogType::Custom => DialogPriority::Normal,
            DialogType::FullScreen => DialogPriority::High,
            DialogType::BottomSheet => DialogPriority::Normal,
            DialogType::SideSheet => DialogPriority::Normal,
            DialogType::Popover => DialogPriority::Low,
            DialogType::Tooltip => DialogPriority::Low,
        }
    }

    /// Whether this dialog type is a sheet
    pub fn is_sheet(&self) -> bool {
        matches!(self, DialogType::BottomSheet | DialogType::SideSheet)
    }

    /// Whether this dialog type is an overlay (popover/tooltip)
    pub fn is_overlay(&self) -> bool {
        matches!(self, DialogType::Popover | DialogType::Tooltip)
    }
}

/// Result from a dialog interaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DialogResult<T = ()> {
    /// User confirmed the dialog (e.g., clicked OK, Yes, Submit)
    Confirmed(T),
    /// User cancelled the dialog (e.g., clicked Cancel, No, backdrop tap)
    Cancelled,
    /// Dialog was dismissed programmatically
    Dismissed,
    /// Dialog timed out (auto-dismiss)
    TimedOut,
}

impl<T> DialogResult<T> {
    /// Create a confirmed result with value
    pub fn confirmed(value: T) -> Self {
        Self::Confirmed(value)
    }

    /// Whether the result is confirmed
    pub fn is_confirmed(&self) -> bool {
        matches!(self, DialogResult::Confirmed(_))
    }

    /// Whether the result is cancelled
    pub fn is_cancelled(&self) -> bool {
        matches!(self, DialogResult::Cancelled)
    }

    /// Get the confirmed value if present
    pub fn value(self) -> Option<T> {
        match self {
            DialogResult::Confirmed(v) => Some(v),
            _ => None,
        }
    }

    /// Map the confirmed value
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> DialogResult<U> {
        match self {
            DialogResult::Confirmed(v) => DialogResult::Confirmed(f(v)),
            DialogResult::Cancelled => DialogResult::Cancelled,
            DialogResult::Dismissed => DialogResult::Dismissed,
            DialogResult::TimedOut => DialogResult::TimedOut,
        }
    }
}

impl DialogResult<()> {
    /// Create a simple confirmed result
    pub fn ok() -> Self {
        Self::Confirmed(())
    }
}

/// Reason for dialog dismissal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DismissReason {
    /// User tapped/clicked the backdrop
    BackdropTap,
    /// User pressed Escape key
    EscapeKey,
    /// User clicked a dismiss action (Cancel, Close, etc.)
    UserAction,
    /// Dialog was dismissed programmatically
    Programmatic,
    /// Dialog timed out
    Timeout,
    /// Higher priority dialog took over
    Preempted,
}

/// Base dialog data common to all dialog types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogData {
    /// Unique identifier
    pub id: DialogId,
    /// Type of dialog
    pub dialog_type: DialogType,
    /// Current state
    pub state: DialogState,
    /// Configuration options
    pub options: DialogOptions,
    /// Optional title
    pub title: Option<String>,
    /// Optional message/content description
    pub message: Option<String>,
    /// Creation timestamp (milliseconds since epoch)
    pub created_at: u64,
    /// When the dialog became visible (milliseconds since epoch)
    pub shown_at: Option<u64>,
}

impl DialogData {
    /// Create new dialog data
    pub fn new(dialog_type: DialogType) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut options = DialogOptions::default();
        options.priority = dialog_type.default_priority();

        Self {
            id: DialogId::new(),
            dialog_type,
            state: DialogState::Pending,
            options,
            title: None,
            message: None,
            created_at,
            shown_at: None,
        }
    }

    /// Set title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set options
    pub fn with_options(mut self, options: DialogOptions) -> Self {
        self.options = options;
        self
    }

    /// Mark dialog as entering (animating in)
    pub fn mark_entering(&mut self) {
        self.state = DialogState::Entering;
    }

    /// Mark dialog as visible
    pub fn mark_visible(&mut self) {
        self.state = DialogState::Visible;
        self.shown_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        );
    }

    /// Mark dialog as exiting (animating out)
    pub fn mark_exiting(&mut self) {
        self.state = DialogState::Exiting;
    }

    /// Mark dialog as dismissed
    pub fn mark_dismissed(&mut self) {
        self.state = DialogState::Dismissed;
    }

    /// Get time dialog has been visible (milliseconds)
    pub fn visible_duration(&self) -> Option<u64> {
        self.shown_at.map(|shown| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            now.saturating_sub(shown)
        })
    }
}

/// Trait for all dialog types
pub trait Dialog: Send + Sync {
    /// Get the dialog data
    fn data(&self) -> &DialogData;

    /// Get mutable dialog data
    fn data_mut(&mut self) -> &mut DialogData;

    /// Get the dialog ID
    fn id(&self) -> DialogId {
        self.data().id
    }

    /// Get the dialog type
    fn dialog_type(&self) -> DialogType {
        self.data().dialog_type
    }

    /// Get the current state
    fn state(&self) -> DialogState {
        self.data().state
    }

    /// Get the options
    fn options(&self) -> &DialogOptions {
        &self.data().options
    }

    /// Get the priority
    fn priority(&self) -> DialogPriority {
        self.data().options.priority
    }

    /// Whether the dialog can be dismissed
    fn can_dismiss(&self) -> bool {
        self.data().options.dismiss.allow_dismiss
    }

    /// Handle a dismiss request
    fn handle_dismiss(&mut self, reason: DismissReason) -> bool {
        let can_dismiss = match reason {
            DismissReason::BackdropTap => self.data().options.dismiss.on_backdrop_tap,
            DismissReason::EscapeKey => self.data().options.dismiss.on_escape_key,
            DismissReason::UserAction => self.data().options.dismiss.allow_dismiss,
            DismissReason::Programmatic => true, // Always allow programmatic dismiss
            DismissReason::Timeout => true,
            DismissReason::Preempted => true,
        };

        if can_dismiss {
            self.data_mut().mark_exiting();
        }

        can_dismiss
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_id_unique() {
        let id1 = DialogId::new();
        let id2 = DialogId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_dialog_id_display() {
        let id = DialogId::new();
        let display = format!("{}", id);
        assert!(display.starts_with("dialog-"));
    }

    #[test]
    fn test_dialog_state_visibility() {
        assert!(!DialogState::Pending.is_visible());
        assert!(DialogState::Entering.is_visible());
        assert!(DialogState::Visible.is_visible());
        assert!(DialogState::Exiting.is_visible());
        assert!(!DialogState::Dismissed.is_visible());
    }

    #[test]
    fn test_dialog_state_interactive() {
        assert!(!DialogState::Pending.is_interactive());
        assert!(!DialogState::Entering.is_interactive());
        assert!(DialogState::Visible.is_interactive());
        assert!(!DialogState::Exiting.is_interactive());
        assert!(!DialogState::Dismissed.is_interactive());
    }

    #[test]
    fn test_dialog_type_priority() {
        assert!(matches!(DialogType::Alert.default_priority(), DialogPriority::Normal));
        assert!(matches!(DialogType::FullScreen.default_priority(), DialogPriority::High));
        assert!(matches!(DialogType::Tooltip.default_priority(), DialogPriority::Low));
    }

    #[test]
    fn test_dialog_type_categorization() {
        assert!(DialogType::BottomSheet.is_sheet());
        assert!(DialogType::SideSheet.is_sheet());
        assert!(!DialogType::Alert.is_sheet());

        assert!(DialogType::Popover.is_overlay());
        assert!(DialogType::Tooltip.is_overlay());
        assert!(!DialogType::Alert.is_overlay());
    }

    #[test]
    fn test_dialog_result_confirmed() {
        let result: DialogResult<i32> = DialogResult::confirmed(42);
        assert!(result.is_confirmed());
        assert!(!result.is_cancelled());
        assert_eq!(result.value(), Some(42));
    }

    #[test]
    fn test_dialog_result_cancelled() {
        let result: DialogResult<i32> = DialogResult::Cancelled;
        assert!(!result.is_confirmed());
        assert!(result.is_cancelled());
        assert_eq!(result.value(), None);
    }

    #[test]
    fn test_dialog_result_map() {
        let result: DialogResult<i32> = DialogResult::confirmed(10);
        let mapped = result.map(|x| x * 2);
        assert_eq!(mapped.value(), Some(20));

        let cancelled: DialogResult<i32> = DialogResult::Cancelled;
        let mapped_cancelled = cancelled.map(|x| x * 2);
        assert!(matches!(mapped_cancelled, DialogResult::Cancelled));
    }

    #[test]
    fn test_dialog_data_creation() {
        let data = DialogData::new(DialogType::Alert);
        assert!(matches!(data.dialog_type, DialogType::Alert));
        assert!(matches!(data.state, DialogState::Pending));
        assert!(data.title.is_none());
        assert!(data.message.is_none());
    }

    #[test]
    fn test_dialog_data_builder() {
        let data = DialogData::new(DialogType::Confirm)
            .with_title("Confirm Action")
            .with_message("Are you sure?");

        assert_eq!(data.title, Some("Confirm Action".to_string()));
        assert_eq!(data.message, Some("Are you sure?".to_string()));
    }

    #[test]
    fn test_dialog_data_state_transitions() {
        let mut data = DialogData::new(DialogType::Alert);

        assert!(matches!(data.state, DialogState::Pending));
        assert!(data.shown_at.is_none());

        data.mark_entering();
        assert!(matches!(data.state, DialogState::Entering));

        data.mark_visible();
        assert!(matches!(data.state, DialogState::Visible));
        assert!(data.shown_at.is_some());

        data.mark_exiting();
        assert!(matches!(data.state, DialogState::Exiting));

        data.mark_dismissed();
        assert!(matches!(data.state, DialogState::Dismissed));
    }
}
