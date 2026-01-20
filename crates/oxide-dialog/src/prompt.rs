//! Prompt Dialog
//!
//! Dialog with text input for collecting user input.

use serde::{Deserialize, Serialize};

use crate::alert::DialogAction;
use crate::dialog::{Dialog, DialogData, DialogResult, DialogType, DismissReason};
use crate::options::DialogOptions;

/// Input type for prompt dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InputType {
    /// Single line text input
    #[default]
    Text,
    /// Password input (masked)
    Password,
    /// Email input
    Email,
    /// Number input
    Number,
    /// URL input
    Url,
    /// Phone number input
    Phone,
    /// Multi-line text input
    TextArea,
}

impl InputType {
    /// Whether the input should be masked
    pub fn is_masked(&self) -> bool {
        matches!(self, InputType::Password)
    }

    /// Whether the input is multi-line
    pub fn is_multiline(&self) -> bool {
        matches!(self, InputType::TextArea)
    }

    /// Get keyboard type hint for mobile
    pub fn keyboard_type(&self) -> &'static str {
        match self {
            InputType::Text => "text",
            InputType::Password => "text",
            InputType::Email => "email",
            InputType::Number => "number",
            InputType::Url => "url",
            InputType::Phone => "phone",
            InputType::TextArea => "text",
        }
    }
}

/// Validation result for input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Input is valid
    Valid,
    /// Input is invalid with error message
    Invalid(String),
    /// Validation is pending (async)
    Pending,
}

impl ValidationResult {
    /// Whether the result is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    /// Whether the result is invalid
    pub fn is_invalid(&self) -> bool {
        matches!(self, ValidationResult::Invalid(_))
    }

    /// Get error message if invalid
    pub fn error(&self) -> Option<&str> {
        match self {
            ValidationResult::Invalid(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Input configuration for prompt dialog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Input type
    pub input_type: InputType,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Maximum length (0 = no limit)
    pub max_length: usize,
    /// Minimum length
    pub min_length: usize,
    /// Whether input is required
    pub required: bool,
    /// Auto-focus on show
    pub auto_focus: bool,
    /// Auto-select all text on focus
    pub select_on_focus: bool,
    /// Autocomplete hint
    pub autocomplete: Option<String>,
    /// Helper text shown below input
    pub helper_text: Option<String>,
    /// Character counter (for TextArea)
    pub show_character_count: bool,
    /// Number of rows (for TextArea)
    pub rows: u32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            input_type: InputType::default(),
            placeholder: None,
            default_value: None,
            max_length: 0,
            min_length: 0,
            required: false,
            auto_focus: true,
            select_on_focus: false,
            autocomplete: None,
            helper_text: None,
            show_character_count: false,
            rows: 3,
        }
    }
}

impl InputConfig {
    /// Create new input config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set input type
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Set max length
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    /// Set min length
    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = min;
        self
    }

    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set auto focus
    pub fn auto_focus(mut self, enabled: bool) -> Self {
        self.auto_focus = enabled;
        self
    }

    /// Set select on focus
    pub fn select_on_focus(mut self) -> Self {
        self.select_on_focus = true;
        self
    }

    /// Set autocomplete hint
    pub fn autocomplete(mut self, hint: impl Into<String>) -> Self {
        self.autocomplete = Some(hint.into());
        self
    }

    /// Set helper text
    pub fn helper_text(mut self, text: impl Into<String>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Show character count
    pub fn show_character_count(mut self) -> Self {
        self.show_character_count = true;
        self
    }

    /// Set rows for TextArea
    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = rows;
        self
    }

    /// Create a text area config
    pub fn text_area() -> Self {
        Self::default()
            .input_type(InputType::TextArea)
            .show_character_count()
    }

    /// Create a password config
    pub fn password() -> Self {
        Self::default()
            .input_type(InputType::Password)
            .autocomplete("current-password".to_string())
    }

    /// Create an email config
    pub fn email() -> Self {
        Self::default()
            .input_type(InputType::Email)
            .autocomplete("email".to_string())
    }
}

/// Prompt dialog for collecting user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDialog {
    /// Base dialog data
    pub data: DialogData,
    /// Input configuration
    pub input: InputConfig,
    /// Current input value
    pub value: String,
    /// Validation state
    pub validation: ValidationResult,
    /// Submit action
    pub submit_action: DialogAction,
    /// Cancel action
    pub cancel_action: DialogAction,
    /// Icon name (optional)
    pub icon: Option<String>,
    /// Result
    result: Option<DialogResult<String>>,
}

impl PromptDialog {
    /// Create a new prompt dialog
    pub fn new(title: impl Into<String>) -> Self {
        let data = DialogData::new(DialogType::Prompt)
            .with_title(title);

        Self {
            data,
            input: InputConfig::default(),
            value: String::new(),
            validation: ValidationResult::Valid,
            submit_action: DialogAction::ok(),
            cancel_action: DialogAction::cancel(),
            icon: None,
            result: None,
        }
    }

    /// Create a prompt with title and message
    pub fn with_message(title: impl Into<String>, message: impl Into<String>) -> Self {
        let data = DialogData::new(DialogType::Prompt)
            .with_title(title)
            .with_message(message);

        Self {
            data,
            input: InputConfig::default(),
            value: String::new(),
            validation: ValidationResult::Valid,
            submit_action: DialogAction::ok(),
            cancel_action: DialogAction::cancel(),
            icon: None,
            result: None,
        }
    }

    /// Set message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.data.message = Some(message.into());
        self
    }

    /// Set input configuration
    pub fn input(mut self, input: InputConfig) -> Self {
        self.input = input;
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.input.placeholder = Some(placeholder.into());
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.value = value.clone();
        self.input.default_value = Some(value);
        self
    }

    /// Set input type
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input.input_type = input_type;
        self
    }

    /// Mark input as required
    pub fn required(mut self) -> Self {
        self.input.required = true;
        self
    }

    /// Set max length
    pub fn max_length(mut self, max: usize) -> Self {
        self.input.max_length = max;
        self
    }

    /// Set min length
    pub fn min_length(mut self, min: usize) -> Self {
        self.input.min_length = min;
        self
    }

    /// Set helper text
    pub fn helper_text(mut self, text: impl Into<String>) -> Self {
        self.input.helper_text = Some(text.into());
        self
    }

    /// Set submit action label
    pub fn submit_label(mut self, label: impl Into<String>) -> Self {
        self.submit_action.label = label.into();
        self
    }

    /// Set cancel action label
    pub fn cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_action.label = label.into();
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

    /// Update the current value
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.validate();
    }

    /// Validate the current input
    pub fn validate(&mut self) -> bool {
        // Check required
        if self.input.required && self.value.trim().is_empty() {
            self.validation = ValidationResult::Invalid("This field is required".to_string());
            return false;
        }

        // Check min length
        if self.input.min_length > 0 && self.value.len() < self.input.min_length {
            self.validation = ValidationResult::Invalid(
                format!("Minimum {} characters required", self.input.min_length)
            );
            return false;
        }

        // Check max length
        if self.input.max_length > 0 && self.value.len() > self.input.max_length {
            self.validation = ValidationResult::Invalid(
                format!("Maximum {} characters allowed", self.input.max_length)
            );
            return false;
        }

        // Type-specific validation
        match self.input.input_type {
            InputType::Email => {
                if !self.value.is_empty() && !self.value.contains('@') {
                    self.validation = ValidationResult::Invalid("Invalid email address".to_string());
                    return false;
                }
            }
            InputType::Url => {
                if !self.value.is_empty() && !self.value.starts_with("http://") && !self.value.starts_with("https://") {
                    self.validation = ValidationResult::Invalid("Invalid URL".to_string());
                    return false;
                }
            }
            InputType::Number => {
                if !self.value.is_empty() && self.value.parse::<f64>().is_err() {
                    self.validation = ValidationResult::Invalid("Invalid number".to_string());
                    return false;
                }
            }
            _ => {}
        }

        self.validation = ValidationResult::Valid;
        true
    }

    /// Submit the dialog with current value
    pub fn submit(&mut self) -> DialogResult<String> {
        if !self.validate() {
            // Can't submit invalid input
            return DialogResult::Cancelled;
        }

        self.data.mark_exiting();
        let result = DialogResult::Confirmed(self.value.clone());
        self.result = Some(result.clone());
        result
    }

    /// Cancel the dialog
    pub fn cancel(&mut self) -> DialogResult<String> {
        self.data.mark_exiting();
        self.result = Some(DialogResult::Cancelled);
        DialogResult::Cancelled
    }

    /// Get the result
    pub fn result(&self) -> Option<&DialogResult<String>> {
        self.result.as_ref()
    }

    /// Show the dialog (placeholder for integration with DialogManager)
    pub fn show(self) -> Self {
        tracing::debug!("Showing prompt dialog: {:?}", self.data.title);
        self
    }
}

impl Dialog for PromptDialog {
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

/// Convenience function to create a prompt dialog
pub fn prompt(title: impl Into<String>) -> PromptDialog {
    PromptDialog::new(title)
}

/// Convenience function to create a password prompt
pub fn password_prompt(title: impl Into<String>) -> PromptDialog {
    PromptDialog::new(title)
        .input(InputConfig::password())
}

/// Convenience function to create an email prompt
pub fn email_prompt(title: impl Into<String>) -> PromptDialog {
    PromptDialog::new(title)
        .input(InputConfig::email())
        .placeholder("email@example.com")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_type() {
        assert!(InputType::Password.is_masked());
        assert!(!InputType::Text.is_masked());
        assert!(InputType::TextArea.is_multiline());
        assert!(!InputType::Text.is_multiline());
    }

    #[test]
    fn test_input_type_keyboard() {
        assert_eq!(InputType::Email.keyboard_type(), "email");
        assert_eq!(InputType::Number.keyboard_type(), "number");
        assert_eq!(InputType::Phone.keyboard_type(), "phone");
    }

    #[test]
    fn test_validation_result() {
        assert!(ValidationResult::Valid.is_valid());
        assert!(!ValidationResult::Invalid("error".into()).is_valid());
        assert!(ValidationResult::Invalid("error".into()).is_invalid());
        assert_eq!(ValidationResult::Invalid("test".into()).error(), Some("test"));
    }

    #[test]
    fn test_input_config_builder() {
        let config = InputConfig::new()
            .input_type(InputType::Email)
            .placeholder("Enter email")
            .required()
            .max_length(100);

        assert!(matches!(config.input_type, InputType::Email));
        assert_eq!(config.placeholder, Some("Enter email".to_string()));
        assert!(config.required);
        assert_eq!(config.max_length, 100);
    }

    #[test]
    fn test_input_config_presets() {
        let password = InputConfig::password();
        assert!(matches!(password.input_type, InputType::Password));
        assert_eq!(password.autocomplete, Some("current-password".to_string()));

        let email = InputConfig::email();
        assert!(matches!(email.input_type, InputType::Email));

        let text_area = InputConfig::text_area();
        assert!(matches!(text_area.input_type, InputType::TextArea));
        assert!(text_area.show_character_count);
    }

    #[test]
    fn test_prompt_dialog_creation() {
        let dialog = PromptDialog::new("Enter Name");
        assert_eq!(dialog.data.title, Some("Enter Name".to_string()));
        assert!(dialog.data.message.is_none());
        assert!(dialog.value.is_empty());
    }

    #[test]
    fn test_prompt_dialog_with_message() {
        let dialog = PromptDialog::with_message("Name", "Please enter your full name");
        assert_eq!(dialog.data.message, Some("Please enter your full name".to_string()));
    }

    #[test]
    fn test_prompt_dialog_builder() {
        let dialog = PromptDialog::new("Email")
            .placeholder("your@email.com")
            .input_type(InputType::Email)
            .required()
            .helper_text("We'll never share your email");

        assert_eq!(dialog.input.placeholder, Some("your@email.com".to_string()));
        assert!(matches!(dialog.input.input_type, InputType::Email));
        assert!(dialog.input.required);
        assert_eq!(dialog.input.helper_text, Some("We'll never share your email".to_string()));
    }

    #[test]
    fn test_prompt_dialog_default_value() {
        let dialog = PromptDialog::new("Edit Name")
            .default_value("John Doe");

        assert_eq!(dialog.value, "John Doe");
        assert_eq!(dialog.input.default_value, Some("John Doe".to_string()));
    }

    #[test]
    fn test_prompt_dialog_validation_required() {
        let mut dialog = PromptDialog::new("Name").required();

        dialog.set_value("");
        assert!(dialog.validation.is_invalid());

        dialog.set_value("John");
        assert!(dialog.validation.is_valid());
    }

    #[test]
    fn test_prompt_dialog_validation_min_length() {
        let mut dialog = PromptDialog::new("Password").min_length(8);

        dialog.set_value("short");
        assert!(dialog.validation.is_invalid());

        dialog.set_value("longenough");
        assert!(dialog.validation.is_valid());
    }

    #[test]
    fn test_prompt_dialog_validation_max_length() {
        let mut dialog = PromptDialog::new("Code").max_length(4);

        dialog.set_value("12345");
        assert!(dialog.validation.is_invalid());

        dialog.set_value("1234");
        assert!(dialog.validation.is_valid());
    }

    #[test]
    fn test_prompt_dialog_validation_email() {
        let mut dialog = PromptDialog::new("Email").input_type(InputType::Email);

        dialog.set_value("invalid");
        assert!(dialog.validation.is_invalid());

        dialog.set_value("valid@email.com");
        assert!(dialog.validation.is_valid());
    }

    #[test]
    fn test_prompt_dialog_validation_url() {
        let mut dialog = PromptDialog::new("URL").input_type(InputType::Url);

        dialog.set_value("not-a-url");
        assert!(dialog.validation.is_invalid());

        dialog.set_value("https://example.com");
        assert!(dialog.validation.is_valid());
    }

    #[test]
    fn test_prompt_dialog_validation_number() {
        let mut dialog = PromptDialog::new("Age").input_type(InputType::Number);

        dialog.set_value("abc");
        assert!(dialog.validation.is_invalid());

        dialog.set_value("25");
        assert!(dialog.validation.is_valid());
    }

    #[test]
    fn test_prompt_dialog_submit_valid() {
        let mut dialog = PromptDialog::new("Name");
        dialog.set_value("John");

        let result = dialog.submit();
        assert!(result.is_confirmed());
        assert_eq!(result.value(), Some("John".to_string()));
    }

    #[test]
    fn test_prompt_dialog_submit_invalid() {
        let mut dialog = PromptDialog::new("Name").required();
        dialog.set_value("");

        let result = dialog.submit();
        assert!(result.is_cancelled());
    }

    #[test]
    fn test_prompt_dialog_cancel() {
        let mut dialog = PromptDialog::new("Name");
        dialog.set_value("John");

        let result = dialog.cancel();
        assert!(result.is_cancelled());
    }

    #[test]
    fn test_prompt_convenience_functions() {
        let dialog = prompt("Enter value");
        assert_eq!(dialog.data.title, Some("Enter value".to_string()));

        let dialog = password_prompt("Password");
        assert!(matches!(dialog.input.input_type, InputType::Password));

        let dialog = email_prompt("Email");
        assert!(matches!(dialog.input.input_type, InputType::Email));
    }
}
