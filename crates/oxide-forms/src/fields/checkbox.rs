//! Checkbox Field Type
//!
//! Provides CheckboxField for boolean toggle inputs.

use crate::error::ValidationError;
use crate::field::FieldState;
use crate::mode::{ValidationConfig, ValidationMode};
use crate::validator::{ValidatorChain, Validator};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A checkbox input field
#[derive(Debug)]
pub struct CheckboxField {
    /// Field name
    pub name: String,
    /// Current state
    pub state: FieldState<bool>,
    /// Validation configuration
    pub config: ValidationConfig,
    /// Validators
    validators: ValidatorChain<bool>,
    /// Field label
    pub label: Option<String>,
    /// Help text
    pub help_text: Option<String>,
    /// Whether the field is disabled
    pub disabled: bool,
    /// Indeterminate state (for tree-like checkboxes)
    pub indeterminate: bool,
}

impl CheckboxField {
    /// Create a new checkbox field
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(false),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            label: None,
            help_text: None,
            disabled: false,
            indeterminate: false,
        }
    }

    /// Create a new checkbox field with an initial value
    pub fn with_value(name: impl Into<String>, initial_value: bool) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(initial_value),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            label: None,
            help_text: None,
            disabled: false,
            indeterminate: false,
        }
    }

    /// Set the validation mode
    pub fn validate_on(mut self, mode: ValidationMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Add a validator
    pub fn validate<V: Validator<bool> + 'static>(mut self, validator: V) -> Self {
        self.validators = self.validators.add(validator);
        self
    }

    /// Set the label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the help text
    pub fn help_text(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    /// Set whether disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set indeterminate state
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Get the current value
    pub fn value(&self) -> bool {
        self.state.value
    }

    /// Check if checked
    pub fn is_checked(&self) -> bool {
        self.state.value
    }

    /// Set the field value
    pub fn set_value(&mut self, value: bool) {
        if self.disabled {
            return;
        }
        self.indeterminate = false;
        self.state.set_value(value);
    }

    /// Toggle the checkbox
    pub fn toggle(&mut self) {
        if self.disabled {
            return;
        }
        self.indeterminate = false;
        self.state.set_value(!self.state.value);
    }

    /// Handle change event
    pub fn on_change(&mut self, value: bool) {
        self.set_value(value);
        if self.config.mode.validates_on_change(self.state.validated) {
            self.run_validation();
        }
    }

    /// Handle click event (toggle)
    pub fn on_click(&mut self) {
        self.toggle();
        // Always validate on click for checkboxes
        self.run_validation();
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
        self.indeterminate = false;
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

/// Validator that requires the checkbox to be checked
#[derive(Debug, Clone, Copy, Default)]
pub struct MustBeChecked;

impl Validator<bool> for MustBeChecked {
    fn validate(&self, value: &bool, field_name: &str) -> Option<ValidationError> {
        if !*value {
            Some(ValidationError::with_code(
                field_name,
                "This must be checked",
                "must_be_checked",
            ))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        "MustBeChecked".to_string()
    }
}

/// Validator that requires the checkbox to be unchecked
#[derive(Debug, Clone, Copy, Default)]
pub struct MustBeUnchecked;

impl Validator<bool> for MustBeUnchecked {
    fn validate(&self, value: &bool, field_name: &str) -> Option<ValidationError> {
        if *value {
            Some(ValidationError::with_code(
                field_name,
                "This must be unchecked",
                "must_be_unchecked",
            ))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        "MustBeUnchecked".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_field_new() {
        let field = CheckboxField::new("agree");
        assert_eq!(field.name, "agree");
        assert!(!field.value());
    }

    #[test]
    fn test_checkbox_field_with_value() {
        let field = CheckboxField::with_value("subscribe", true);
        assert!(field.value());
    }

    #[test]
    fn test_checkbox_field_toggle() {
        let mut field = CheckboxField::new("option");
        assert!(!field.is_checked());

        field.toggle();
        assert!(field.is_checked());

        field.toggle();
        assert!(!field.is_checked());
    }

    #[test]
    fn test_checkbox_field_disabled() {
        let mut field = CheckboxField::new("locked").disabled(true);
        field.set_value(true);
        assert!(!field.is_checked()); // Should not change
    }

    #[test]
    fn test_must_be_checked_validator() {
        let mut field = CheckboxField::new("terms")
            .validate(MustBeChecked);

        field.run_validation();
        assert!(!field.is_valid());

        field.set_value(true);
        field.run_validation();
        assert!(field.is_valid());
    }

    #[test]
    fn test_checkbox_indeterminate() {
        let mut field = CheckboxField::new("all").indeterminate(true);
        assert!(field.indeterminate);

        field.set_value(true);
        assert!(!field.indeterminate); // Cleared on value change
    }

    #[test]
    fn test_checkbox_dirty() {
        let mut field = CheckboxField::new("option");
        assert!(!field.is_dirty());

        field.toggle();
        assert!(field.is_dirty());

        field.reset();
        assert!(!field.is_dirty());
    }
}
