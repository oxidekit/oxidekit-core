//! Alert Dialog
//!
//! Simple dialog for displaying messages with an action button.

use serde::{Deserialize, Serialize};

use crate::dialog::{Dialog, DialogData, DialogResult, DialogType, DismissReason};
use crate::options::DialogOptions;

/// Action button style for alert dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ActionStyle {
    /// Default style (primary color)
    #[default]
    Default,
    /// Primary action (emphasized)
    Primary,
    /// Destructive action (red/warning color)
    Destructive,
    /// Secondary action (less emphasis)
    Secondary,
    /// Disabled action
    Disabled,
}

impl ActionStyle {
    /// Whether this action is destructive
    pub fn is_destructive(&self) -> bool {
        matches!(self, ActionStyle::Destructive)
    }

    /// Whether this action is disabled
    pub fn is_disabled(&self) -> bool {
        matches!(self, ActionStyle::Disabled)
    }
}

/// Action button for dialogs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogAction {
    /// Button label
    pub label: String,
    /// Button style
    pub style: ActionStyle,
    /// Whether this action dismisses the dialog
    pub dismisses: bool,
    /// Action identifier for handling
    pub action_id: String,
    /// Whether the action is currently loading
    pub loading: bool,
}

impl DialogAction {
    /// Create a new action
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let action_id = label.to_lowercase().replace(' ', "_");
        Self {
            label,
            style: ActionStyle::Default,
            dismisses: true,
            action_id,
            loading: false,
        }
    }

    /// Create an "OK" action
    pub fn ok() -> Self {
        Self::new("OK").primary()
    }

    /// Create a "Cancel" action
    pub fn cancel() -> Self {
        Self::new("Cancel").secondary()
    }

    /// Create a "Yes" action
    pub fn yes() -> Self {
        Self::new("Yes").primary()
    }

    /// Create a "No" action
    pub fn no() -> Self {
        Self::new("No").secondary()
    }

    /// Create a "Delete" action
    pub fn delete() -> Self {
        Self::new("Delete").destructive()
    }

    /// Create a "Close" action
    pub fn close() -> Self {
        Self::new("Close").secondary()
    }

    /// Set action style to primary
    pub fn primary(mut self) -> Self {
        self.style = ActionStyle::Primary;
        self
    }

    /// Set action style to secondary
    pub fn secondary(mut self) -> Self {
        self.style = ActionStyle::Secondary;
        self
    }

    /// Set action style to destructive
    pub fn destructive(mut self) -> Self {
        self.style = ActionStyle::Destructive;
        self
    }

    /// Set action style to disabled
    pub fn disabled(mut self) -> Self {
        self.style = ActionStyle::Disabled;
        self
    }

    /// Set whether action dismisses dialog
    pub fn dismisses(mut self, dismisses: bool) -> Self {
        self.dismisses = dismisses;
        self
    }

    /// Set custom action ID
    pub fn action_id(mut self, id: impl Into<String>) -> Self {
        self.action_id = id.into();
        self
    }

    /// Set loading state
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
}

/// Alert dialog for displaying messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDialog {
    /// Base dialog data
    pub data: DialogData,
    /// Actions (buttons)
    pub actions: Vec<DialogAction>,
    /// Icon name (optional)
    pub icon: Option<String>,
    /// Result when dismissed without action
    result: Option<DialogResult<String>>,
}

impl AlertDialog {
    /// Create a new alert dialog
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let data = DialogData::new(DialogType::Alert)
            .with_title(title)
            .with_message(message);

        Self {
            data,
            actions: vec![DialogAction::ok()],
            icon: None,
            result: None,
        }
    }

    /// Create a simple alert with just a message (no title)
    pub fn message(message: impl Into<String>) -> Self {
        let data = DialogData::new(DialogType::Alert)
            .with_message(message);

        Self {
            data,
            actions: vec![DialogAction::ok()],
            icon: None,
            result: None,
        }
    }

    /// Set the title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.data.title = Some(title.into());
        self
    }

    /// Set the message
    pub fn message_text(mut self, message: impl Into<String>) -> Self {
        self.data.message = Some(message.into());
        self
    }

    /// Set the actions
    pub fn actions(mut self, actions: Vec<DialogAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Add an action
    pub fn action(mut self, action: DialogAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set the primary action (replaces existing actions)
    pub fn primary_action(mut self, label: impl Into<String>) -> Self {
        self.actions = vec![DialogAction::new(label).primary()];
        self
    }

    /// Set icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set options
    pub fn options(mut self, options: DialogOptions) -> Self {
        self.data.options = options;
        self
    }

    /// Handle an action being triggered
    pub fn trigger_action(&mut self, action_id: &str) -> Option<DialogResult<String>> {
        if let Some(action) = self.actions.iter().find(|a| a.action_id == action_id) {
            if action.style.is_disabled() {
                return None;
            }

            let result = DialogResult::Confirmed(action.action_id.clone());

            if action.dismisses {
                self.data.mark_exiting();
                self.result = Some(result.clone());
            }

            Some(result)
        } else {
            None
        }
    }

    /// Get the result
    pub fn result(&self) -> Option<&DialogResult<String>> {
        self.result.as_ref()
    }

    /// Show the dialog (placeholder for integration with DialogManager)
    pub fn show(self) -> Self {
        // In a real implementation, this would register with DialogManager
        tracing::debug!("Showing alert dialog: {:?}", self.data.title);
        self
    }
}

impl Dialog for AlertDialog {
    fn data(&self) -> &DialogData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut DialogData {
        &mut self.data
    }

    fn handle_dismiss(&mut self, reason: DismissReason) -> bool {
        let can_dismiss = match reason {
            DismissReason::BackdropTap => self.data.options.dismiss.on_backdrop_tap,
            DismissReason::EscapeKey => self.data.options.dismiss.on_escape_key,
            DismissReason::UserAction => self.data.options.dismiss.allow_dismiss,
            DismissReason::Programmatic => true,
            DismissReason::Timeout => true,
            DismissReason::Preempted => true,
        };

        if can_dismiss {
            self.data.mark_exiting();
            self.result = Some(DialogResult::Cancelled);
        }

        can_dismiss
    }
}

/// Convenience function to create an alert dialog
pub fn alert(title: impl Into<String>, message: impl Into<String>) -> AlertDialog {
    AlertDialog::new(title, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_style() {
        assert!(ActionStyle::Destructive.is_destructive());
        assert!(!ActionStyle::Primary.is_destructive());
        assert!(ActionStyle::Disabled.is_disabled());
        assert!(!ActionStyle::Default.is_disabled());
    }

    #[test]
    fn test_dialog_action_creation() {
        let action = DialogAction::new("Submit");
        assert_eq!(action.label, "Submit");
        assert_eq!(action.action_id, "submit");
        assert!(action.dismisses);
        assert!(!action.loading);
    }

    #[test]
    fn test_dialog_action_presets() {
        let ok = DialogAction::ok();
        assert_eq!(ok.label, "OK");
        assert!(matches!(ok.style, ActionStyle::Primary));

        let cancel = DialogAction::cancel();
        assert_eq!(cancel.label, "Cancel");
        assert!(matches!(cancel.style, ActionStyle::Secondary));

        let delete = DialogAction::delete();
        assert_eq!(delete.label, "Delete");
        assert!(matches!(delete.style, ActionStyle::Destructive));
    }

    #[test]
    fn test_dialog_action_builder() {
        let action = DialogAction::new("Custom")
            .destructive()
            .dismisses(false)
            .action_id("custom_action");

        assert!(matches!(action.style, ActionStyle::Destructive));
        assert!(!action.dismisses);
        assert_eq!(action.action_id, "custom_action");
    }

    #[test]
    fn test_alert_dialog_creation() {
        let dialog = AlertDialog::new("Title", "Message");
        assert_eq!(dialog.data.title, Some("Title".to_string()));
        assert_eq!(dialog.data.message, Some("Message".to_string()));
        assert_eq!(dialog.actions.len(), 1);
        assert_eq!(dialog.actions[0].label, "OK");
    }

    #[test]
    fn test_alert_dialog_message_only() {
        let dialog = AlertDialog::message("Just a message");
        assert!(dialog.data.title.is_none());
        assert_eq!(dialog.data.message, Some("Just a message".to_string()));
    }

    #[test]
    fn test_alert_dialog_builder() {
        let dialog = AlertDialog::new("Warning", "Are you sure?")
            .icon("warning")
            .actions(vec![
                DialogAction::cancel(),
                DialogAction::delete(),
            ]);

        assert_eq!(dialog.icon, Some("warning".to_string()));
        assert_eq!(dialog.actions.len(), 2);
    }

    #[test]
    fn test_alert_dialog_trigger_action() {
        let mut dialog = AlertDialog::new("Test", "Test message")
            .actions(vec![
                DialogAction::cancel(),
                DialogAction::ok(),
            ]);

        let result = dialog.trigger_action("ok");
        assert!(result.is_some());
        assert!(result.unwrap().is_confirmed());
        assert!(dialog.data.state.is_visible() || matches!(dialog.data.state, crate::dialog::DialogState::Exiting));
    }

    #[test]
    fn test_alert_dialog_trigger_disabled_action() {
        let mut dialog = AlertDialog::new("Test", "Test")
            .actions(vec![DialogAction::new("Disabled").disabled()]);

        let result = dialog.trigger_action("disabled");
        assert!(result.is_none());
    }

    #[test]
    fn test_alert_dialog_trigger_unknown_action() {
        let mut dialog = AlertDialog::new("Test", "Test");
        let result = dialog.trigger_action("unknown");
        assert!(result.is_none());
    }

    #[test]
    fn test_alert_convenience_function() {
        let dialog = alert("Hello", "World");
        assert_eq!(dialog.data.title, Some("Hello".to_string()));
        assert_eq!(dialog.data.message, Some("World".to_string()));
    }
}
