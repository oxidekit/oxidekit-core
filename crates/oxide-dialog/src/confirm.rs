//! Confirm Dialog
//!
//! Dialog for yes/no or ok/cancel confirmations.

use serde::{Deserialize, Serialize};

use crate::alert::{ActionStyle, DialogAction};
use crate::dialog::{Dialog, DialogData, DialogResult, DialogType, DismissReason};
use crate::options::DialogOptions;

/// Confirmation style presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConfirmStyle {
    /// OK / Cancel buttons
    #[default]
    OkCancel,
    /// Yes / No buttons
    YesNo,
    /// Confirm / Cancel buttons
    ConfirmCancel,
    /// Delete / Cancel (destructive)
    DeleteCancel,
    /// Save / Discard (for unsaved changes)
    SaveDiscard,
    /// Custom (use set_actions)
    Custom,
}

impl ConfirmStyle {
    /// Get the default actions for this style
    pub fn actions(&self) -> (DialogAction, DialogAction) {
        match self {
            ConfirmStyle::OkCancel => (DialogAction::ok(), DialogAction::cancel()),
            ConfirmStyle::YesNo => (DialogAction::yes(), DialogAction::no()),
            ConfirmStyle::ConfirmCancel => (
                DialogAction::new("Confirm").primary(),
                DialogAction::cancel(),
            ),
            ConfirmStyle::DeleteCancel => (DialogAction::delete(), DialogAction::cancel()),
            ConfirmStyle::SaveDiscard => (
                DialogAction::new("Save").primary(),
                DialogAction::new("Discard").secondary(),
            ),
            ConfirmStyle::Custom => (DialogAction::ok(), DialogAction::cancel()),
        }
    }

    /// Get the positive action label
    pub fn positive_label(&self) -> &'static str {
        match self {
            ConfirmStyle::OkCancel => "OK",
            ConfirmStyle::YesNo => "Yes",
            ConfirmStyle::ConfirmCancel => "Confirm",
            ConfirmStyle::DeleteCancel => "Delete",
            ConfirmStyle::SaveDiscard => "Save",
            ConfirmStyle::Custom => "OK",
        }
    }

    /// Get the negative action label
    pub fn negative_label(&self) -> &'static str {
        match self {
            ConfirmStyle::OkCancel => "Cancel",
            ConfirmStyle::YesNo => "No",
            ConfirmStyle::ConfirmCancel => "Cancel",
            ConfirmStyle::DeleteCancel => "Cancel",
            ConfirmStyle::SaveDiscard => "Discard",
            ConfirmStyle::Custom => "Cancel",
        }
    }
}

/// Confirm dialog for yes/no decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmDialog {
    /// Base dialog data
    pub data: DialogData,
    /// Confirmation style
    pub style: ConfirmStyle,
    /// Positive action (confirm, yes, ok, etc.)
    pub positive_action: DialogAction,
    /// Negative action (cancel, no, etc.)
    pub negative_action: DialogAction,
    /// Optional third action (e.g., "Don't Save" for save dialogs)
    pub tertiary_action: Option<DialogAction>,
    /// Icon name (optional)
    pub icon: Option<String>,
    /// Whether this is a destructive confirmation
    pub is_destructive: bool,
    /// Result
    result: Option<DialogResult<bool>>,
}

impl ConfirmDialog {
    /// Create a new confirm dialog with default OK/Cancel style
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let data = DialogData::new(DialogType::Confirm)
            .with_title(title)
            .with_message(message);

        let (positive, negative) = ConfirmStyle::default().actions();

        Self {
            data,
            style: ConfirmStyle::default(),
            positive_action: positive,
            negative_action: negative,
            tertiary_action: None,
            icon: None,
            is_destructive: false,
            result: None,
        }
    }

    /// Create with a specific style
    pub fn with_style(title: impl Into<String>, message: impl Into<String>, style: ConfirmStyle) -> Self {
        let data = DialogData::new(DialogType::Confirm)
            .with_title(title)
            .with_message(message);

        let (positive, negative) = style.actions();
        let is_destructive = matches!(style, ConfirmStyle::DeleteCancel);

        Self {
            data,
            style,
            positive_action: positive,
            negative_action: negative,
            tertiary_action: None,
            icon: None,
            is_destructive,
            result: None,
        }
    }

    /// Set the confirmation style
    pub fn style(mut self, style: ConfirmStyle) -> Self {
        self.style = style;
        let (positive, negative) = style.actions();
        self.positive_action = positive;
        self.negative_action = negative;
        self.is_destructive = matches!(style, ConfirmStyle::DeleteCancel);
        self
    }

    /// Set custom positive action label
    pub fn positive_label(mut self, label: impl Into<String>) -> Self {
        self.positive_action.label = label.into();
        self
    }

    /// Set custom negative action label
    pub fn negative_label(mut self, label: impl Into<String>) -> Self {
        self.negative_action.label = label.into();
        self
    }

    /// Set positive action
    pub fn positive_action(mut self, action: DialogAction) -> Self {
        self.positive_action = action;
        self
    }

    /// Set negative action
    pub fn negative_action(mut self, action: DialogAction) -> Self {
        self.negative_action = action;
        self
    }

    /// Add a tertiary action (e.g., "Don't Save")
    pub fn tertiary_action(mut self, action: DialogAction) -> Self {
        self.tertiary_action = Some(action);
        self
    }

    /// Mark as destructive (positive action becomes red)
    pub fn destructive(mut self) -> Self {
        self.is_destructive = true;
        self.positive_action.style = ActionStyle::Destructive;
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

    /// Get all actions
    pub fn actions(&self) -> Vec<&DialogAction> {
        let mut actions = vec![&self.negative_action, &self.positive_action];
        if let Some(ref tertiary) = self.tertiary_action {
            actions.insert(1, tertiary);
        }
        actions
    }

    /// Confirm (positive action)
    pub fn confirm(&mut self) -> DialogResult<bool> {
        self.data.mark_exiting();
        let result = DialogResult::Confirmed(true);
        self.result = Some(result.clone());
        result
    }

    /// Cancel (negative action)
    pub fn cancel(&mut self) -> DialogResult<bool> {
        self.data.mark_exiting();
        let result = DialogResult::Confirmed(false);
        self.result = Some(result.clone());
        result
    }

    /// Trigger tertiary action
    pub fn trigger_tertiary(&mut self) -> Option<DialogResult<bool>> {
        if self.tertiary_action.is_some() {
            self.data.mark_exiting();
            // Tertiary is typically "don't save" or similar, so we return a special value
            // We use Cancelled to indicate the user chose the neutral option
            let result = DialogResult::Cancelled;
            self.result = Some(DialogResult::Confirmed(false));
            Some(result)
        } else {
            None
        }
    }

    /// Get the result
    pub fn result(&self) -> Option<&DialogResult<bool>> {
        self.result.as_ref()
    }

    /// Show the dialog (placeholder for integration with DialogManager)
    pub fn show(self) -> Self {
        tracing::debug!("Showing confirm dialog: {:?}", self.data.title);
        self
    }
}

impl Dialog for ConfirmDialog {
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

/// Convenience function to create a confirm dialog
pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> ConfirmDialog {
    ConfirmDialog::new(title, message)
}

/// Convenience function to create a yes/no dialog
pub fn yes_no(title: impl Into<String>, message: impl Into<String>) -> ConfirmDialog {
    ConfirmDialog::with_style(title, message, ConfirmStyle::YesNo)
}

/// Convenience function to create a delete confirmation dialog
pub fn delete_confirm(title: impl Into<String>, message: impl Into<String>) -> ConfirmDialog {
    ConfirmDialog::with_style(title, message, ConfirmStyle::DeleteCancel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_style_actions() {
        let (pos, neg) = ConfirmStyle::OkCancel.actions();
        assert_eq!(pos.label, "OK");
        assert_eq!(neg.label, "Cancel");

        let (pos, neg) = ConfirmStyle::YesNo.actions();
        assert_eq!(pos.label, "Yes");
        assert_eq!(neg.label, "No");

        let (pos, neg) = ConfirmStyle::DeleteCancel.actions();
        assert_eq!(pos.label, "Delete");
        assert!(pos.style.is_destructive());
    }

    #[test]
    fn test_confirm_style_labels() {
        assert_eq!(ConfirmStyle::OkCancel.positive_label(), "OK");
        assert_eq!(ConfirmStyle::OkCancel.negative_label(), "Cancel");
        assert_eq!(ConfirmStyle::YesNo.positive_label(), "Yes");
        assert_eq!(ConfirmStyle::YesNo.negative_label(), "No");
    }

    #[test]
    fn test_confirm_dialog_creation() {
        let dialog = ConfirmDialog::new("Delete File?", "This action cannot be undone.");
        assert_eq!(dialog.data.title, Some("Delete File?".to_string()));
        assert_eq!(dialog.data.message, Some("This action cannot be undone.".to_string()));
        assert_eq!(dialog.positive_action.label, "OK");
        assert_eq!(dialog.negative_action.label, "Cancel");
    }

    #[test]
    fn test_confirm_dialog_with_style() {
        let dialog = ConfirmDialog::with_style("Confirm", "Are you sure?", ConfirmStyle::YesNo);
        assert_eq!(dialog.positive_action.label, "Yes");
        assert_eq!(dialog.negative_action.label, "No");
    }

    #[test]
    fn test_confirm_dialog_destructive() {
        let dialog = ConfirmDialog::new("Delete?", "Cannot undo").destructive();
        assert!(dialog.is_destructive);
        assert!(dialog.positive_action.style.is_destructive());
    }

    #[test]
    fn test_confirm_dialog_tertiary_action() {
        let dialog = ConfirmDialog::with_style("Save?", "You have unsaved changes", ConfirmStyle::SaveDiscard)
            .tertiary_action(DialogAction::new("Don't Save").secondary());

        assert!(dialog.tertiary_action.is_some());
        assert_eq!(dialog.actions().len(), 3);
    }

    #[test]
    fn test_confirm_dialog_custom_labels() {
        let dialog = ConfirmDialog::new("Logout?", "You will be signed out")
            .positive_label("Sign Out")
            .negative_label("Stay Signed In");

        assert_eq!(dialog.positive_action.label, "Sign Out");
        assert_eq!(dialog.negative_action.label, "Stay Signed In");
    }

    #[test]
    fn test_confirm_dialog_confirm() {
        let mut dialog = ConfirmDialog::new("Test", "Test");
        let result = dialog.confirm();
        assert!(result.is_confirmed());
        assert_eq!(result.value(), Some(true));
    }

    #[test]
    fn test_confirm_dialog_cancel() {
        let mut dialog = ConfirmDialog::new("Test", "Test");
        let result = dialog.cancel();
        assert!(result.is_confirmed()); // Confirmed(false), not Cancelled
        assert_eq!(result.value(), Some(false));
    }

    #[test]
    fn test_confirm_dialog_trigger_tertiary() {
        let mut dialog = ConfirmDialog::new("Test", "Test")
            .tertiary_action(DialogAction::new("Third"));

        let result = dialog.trigger_tertiary();
        assert!(result.is_some());
    }

    #[test]
    fn test_confirm_dialog_no_tertiary() {
        let mut dialog = ConfirmDialog::new("Test", "Test");
        let result = dialog.trigger_tertiary();
        assert!(result.is_none());
    }

    #[test]
    fn test_confirm_convenience_functions() {
        let dialog = confirm("Title", "Message");
        assert!(matches!(dialog.style, ConfirmStyle::OkCancel));

        let dialog = yes_no("Question", "Answer");
        assert!(matches!(dialog.style, ConfirmStyle::YesNo));

        let dialog = delete_confirm("Delete", "Are you sure?");
        assert!(matches!(dialog.style, ConfirmStyle::DeleteCancel));
        assert!(dialog.is_destructive);
    }
}
