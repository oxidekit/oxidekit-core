//! Text Field Types
//!
//! Provides TextField and TextAreaField for single-line and multi-line text input.

use crate::error::ValidationError;
use crate::field::{FieldState, FieldValue, FormField};
use crate::mode::{ValidationConfig, ValidationMode};
use crate::validator::{BoxedValidator, ValidatorChain, Validator};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Input type for text fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextInputType {
    #[default]
    Text,
    Password,
    Email,
    Url,
    Tel,
    Search,
}

/// A single-line text input field
pub struct TextField {
    /// Field name
    pub name: String,
    /// Current state
    pub state: FieldState<String>,
    /// Validation configuration
    pub config: ValidationConfig,
    /// Validators
    validators: ValidatorChain<String>,
    /// Input type
    pub input_type: TextInputType,
    /// Field label
    pub label: Option<String>,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Help text
    pub help_text: Option<String>,
    /// Max length attribute
    pub max_length: Option<usize>,
    /// Autocomplete attribute
    pub autocomplete: Option<String>,
    /// Whether the field is disabled
    pub disabled: bool,
    /// Whether the field is read-only
    pub read_only: bool,
}

impl TextField {
    /// Create a new text field
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(String::new()),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            input_type: TextInputType::Text,
            label: None,
            placeholder: None,
            help_text: None,
            max_length: None,
            autocomplete: None,
            disabled: false,
            read_only: false,
        }
    }

    /// Create a new text field with an initial value
    pub fn with_value(name: impl Into<String>, initial_value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(initial_value.into()),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            input_type: TextInputType::Text,
            label: None,
            placeholder: None,
            help_text: None,
            max_length: None,
            autocomplete: None,
            disabled: false,
            read_only: false,
        }
    }

    /// Set as password field
    pub fn password(mut self) -> Self {
        self.input_type = TextInputType::Password;
        self
    }

    /// Set as email field
    pub fn email(mut self) -> Self {
        self.input_type = TextInputType::Email;
        self
    }

    /// Set as URL field
    pub fn url(mut self) -> Self {
        self.input_type = TextInputType::Url;
        self
    }

    /// Set as telephone field
    pub fn tel(mut self) -> Self {
        self.input_type = TextInputType::Tel;
        self
    }

    /// Set as search field
    pub fn search(mut self) -> Self {
        self.input_type = TextInputType::Search;
        self
    }

    /// Set the input type
    pub fn input_type(mut self, input_type: TextInputType) -> Self {
        self.input_type = input_type;
        self
    }

    /// Set the validation mode
    pub fn validate_on(mut self, mode: ValidationMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Add a validator
    pub fn validate<V: Validator<String> + 'static>(mut self, validator: V) -> Self {
        self.validators = self.validators.add(validator);
        self
    }

    /// Set the label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the placeholder
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set the help text
    pub fn help_text(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    /// Set the max length
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set the autocomplete attribute
    pub fn autocomplete(mut self, value: impl Into<String>) -> Self {
        self.autocomplete = Some(value.into());
        self
    }

    /// Set whether disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.state.value
    }

    /// Set the field value
    pub fn set_value(&mut self, value: impl Into<String>) {
        if self.disabled || self.read_only {
            return;
        }
        self.state.set_value(value.into());
    }

    /// Handle change event
    pub fn on_change(&mut self, value: impl Into<String>) {
        self.set_value(value);
        if self.config.mode.validates_on_change(self.state.validated) {
            self.run_validation();
        }
    }

    /// Handle focus event
    pub fn on_focus(&mut self) {
        self.state.mark_focused();
    }

    /// Handle blur event
    pub fn on_blur(&mut self) {
        self.state.mark_blurred();
        if self.config.mode.validates_on_blur(self.state.validated) {
            self.run_validation();
        }
    }

    /// Run validation
    pub fn run_validation(&mut self) -> bool {
        if self.disabled {
            self.state.clear_errors();
            return true;
        }

        let errors = if self.config.stop_on_first_error {
            self.validators
                .validate_first(&self.state.value, &self.name)
                .into_iter()
                .collect()
        } else {
            self.validators.validate(&self.state.value, &self.name)
        };

        self.state.set_errors(errors);
        self.state.is_valid()
    }

    /// Reset the field
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Clear the field
    pub fn clear(&mut self) {
        self.state.clear();
    }

    /// Check if valid
    pub fn is_valid(&self) -> bool {
        self.state.is_valid()
    }

    /// Check if dirty
    pub fn is_dirty(&self) -> bool {
        self.state.dirty
    }

    /// Check if touched
    pub fn is_touched(&self) -> bool {
        self.state.touched
    }

    /// Get errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.state.errors
    }

    /// Get first error
    pub fn first_error(&self) -> Option<&str> {
        self.state.first_error()
    }

    /// Should show error
    pub fn should_show_error(&self) -> bool {
        self.state.should_show_error(self.config.mode)
    }

    /// Get display label
    pub fn display_label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.name)
    }
}

impl fmt::Debug for TextField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextField")
            .field("name", &self.name)
            .field("state", &self.state)
            .field("input_type", &self.input_type)
            .field("disabled", &self.disabled)
            .finish()
    }
}

/// A multi-line text input field
pub struct TextAreaField {
    /// Field name
    pub name: String,
    /// Current state
    pub state: FieldState<String>,
    /// Validation configuration
    pub config: ValidationConfig,
    /// Validators
    validators: ValidatorChain<String>,
    /// Field label
    pub label: Option<String>,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Help text
    pub help_text: Option<String>,
    /// Number of rows
    pub rows: Option<u32>,
    /// Number of columns
    pub cols: Option<u32>,
    /// Max length
    pub max_length: Option<usize>,
    /// Whether the field is disabled
    pub disabled: bool,
    /// Whether the field is read-only
    pub read_only: bool,
    /// Whether to auto-resize
    pub auto_resize: bool,
}

impl TextAreaField {
    /// Create a new textarea field
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(String::new()),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            label: None,
            placeholder: None,
            help_text: None,
            rows: None,
            cols: None,
            max_length: None,
            disabled: false,
            read_only: false,
            auto_resize: false,
        }
    }

    /// Create a new textarea field with initial value
    pub fn with_value(name: impl Into<String>, initial_value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(initial_value.into()),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            label: None,
            placeholder: None,
            help_text: None,
            rows: None,
            cols: None,
            max_length: None,
            disabled: false,
            read_only: false,
            auto_resize: false,
        }
    }

    /// Set the validation mode
    pub fn validate_on(mut self, mode: ValidationMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Add a validator
    pub fn validate<V: Validator<String> + 'static>(mut self, validator: V) -> Self {
        self.validators = self.validators.add(validator);
        self
    }

    /// Set the label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the placeholder
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set the help text
    pub fn help_text(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    /// Set the number of rows
    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = Some(rows);
        self
    }

    /// Set the number of columns
    pub fn cols(mut self, cols: u32) -> Self {
        self.cols = Some(cols);
        self
    }

    /// Set the max length
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set whether disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Set whether to auto-resize
    pub fn auto_resize(mut self, auto_resize: bool) -> Self {
        self.auto_resize = auto_resize;
        self
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.state.value
    }

    /// Set the field value
    pub fn set_value(&mut self, value: impl Into<String>) {
        if self.disabled || self.read_only {
            return;
        }
        self.state.set_value(value.into());
    }

    /// Handle change event
    pub fn on_change(&mut self, value: impl Into<String>) {
        self.set_value(value);
        if self.config.mode.validates_on_change(self.state.validated) {
            self.run_validation();
        }
    }

    /// Handle focus event
    pub fn on_focus(&mut self) {
        self.state.mark_focused();
    }

    /// Handle blur event
    pub fn on_blur(&mut self) {
        self.state.mark_blurred();
        if self.config.mode.validates_on_blur(self.state.validated) {
            self.run_validation();
        }
    }

    /// Run validation
    pub fn run_validation(&mut self) -> bool {
        if self.disabled {
            self.state.clear_errors();
            return true;
        }

        let errors = if self.config.stop_on_first_error {
            self.validators
                .validate_first(&self.state.value, &self.name)
                .into_iter()
                .collect()
        } else {
            self.validators.validate(&self.state.value, &self.name)
        };

        self.state.set_errors(errors);
        self.state.is_valid()
    }

    /// Reset the field
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Clear the field
    pub fn clear(&mut self) {
        self.state.clear();
    }

    /// Check if valid
    pub fn is_valid(&self) -> bool {
        self.state.is_valid()
    }

    /// Check if dirty
    pub fn is_dirty(&self) -> bool {
        self.state.dirty
    }

    /// Check if touched
    pub fn is_touched(&self) -> bool {
        self.state.touched
    }

    /// Get errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.state.errors
    }

    /// Get first error
    pub fn first_error(&self) -> Option<&str> {
        self.state.first_error()
    }

    /// Should show error
    pub fn should_show_error(&self) -> bool {
        self.state.should_show_error(self.config.mode)
    }
}

impl fmt::Debug for TextAreaField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextAreaField")
            .field("name", &self.name)
            .field("state", &self.state)
            .field("rows", &self.rows)
            .field("cols", &self.cols)
            .field("disabled", &self.disabled)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::{MinLength, Required};

    #[test]
    fn test_text_field_new() {
        let field = TextField::new("username");
        assert_eq!(field.name, "username");
        assert_eq!(field.value(), "");
        assert_eq!(field.input_type, TextInputType::Text);
    }

    #[test]
    fn test_text_field_password() {
        let field = TextField::new("password").password();
        assert_eq!(field.input_type, TextInputType::Password);
    }

    #[test]
    fn test_text_field_email() {
        let field = TextField::new("email").email();
        assert_eq!(field.input_type, TextInputType::Email);
    }

    #[test]
    fn test_text_field_validation() {
        let mut field = TextField::new("username")
            .validate(Required)
            .validate(MinLength(3));

        field.run_validation();
        assert!(!field.is_valid());

        field.set_value("ab");
        field.run_validation();
        assert!(!field.is_valid());

        field.set_value("abc");
        field.run_validation();
        assert!(field.is_valid());
    }

    #[test]
    fn test_text_field_on_change() {
        let mut field = TextField::new("name")
            .validate_on(ValidationMode::OnChange)
            .validate(Required);

        field.on_change("hello");
        assert!(field.is_valid());
        assert!(field.is_dirty());
    }

    #[test]
    fn test_textarea_field() {
        let mut field = TextAreaField::new("description")
            .rows(5)
            .cols(40)
            .validate(Required);

        assert_eq!(field.rows, Some(5));
        assert_eq!(field.cols, Some(40));

        field.run_validation();
        assert!(!field.is_valid());

        field.set_value("Some text");
        field.run_validation();
        assert!(field.is_valid());
    }
}
